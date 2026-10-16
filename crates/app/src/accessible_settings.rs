//! Appearance selection and bounded persistence; startup read precedes event-loop creation.
use crate::catalog as c;
use dlpad_engine::store::Store;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};
const MAX_THEME_BYTES: usize = 4096;
const SCALES: [u16; 6] = [75, 100, 125, 150, 175, 200];
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}
impl ThemeChoice {
    fn preference(self) -> egui::ThemePreference {
        match self {
            Self::System => egui::ThemePreference::System,
            Self::Light => egui::ThemePreference::Light,
            Self::Dark => egui::ThemePreference::Dark,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::System => c::SYSTEM,
            Self::Light => c::LIGHT,
            Self::Dark => c::DARK,
        }
    }
}
#[derive(Clone, Copy)]
struct Write {
    revision: u64,
    theme: ThemeChoice,
}
enum WorkerCommand {
    Write(Write),
    StopWake,
}

fn stop_worker(signals: &Signals, commands: &SyncSender<WorkerCommand>) {
    signals.stop.store(true, Ordering::Release);
    // Full means an accepted command already wakes recv; never displace it.
    let _ = commands.try_send(WorkerCommand::StopWake);
}

struct ResultMessage {
    revision: u64,
    success: bool,
}
struct Signals {
    stop: AtomicBool,
    stopped: AtomicBool,
    failed: AtomicBool,
    context: Mutex<Option<egui::Context>>,
}
impl Signals {
    fn wake(&self) {
        if let Some(ctx) = self
            .context
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            ctx.request_repaint();
        }
    }
}
struct StopAck(Arc<Signals>);
impl Drop for StopAck {
    fn drop(&mut self) {
        self.0.stopped.store(true, Ordering::Release);
        self.0.wake();
    }
}
fn worker_entry(signals: Arc<Signals>, run: impl FnOnce()) {
    let _ack = StopAck(signals.clone());
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).is_err() {
        signals.failed.store(true, Ordering::Release);
    }
}

pub(crate) struct AppearanceSettings {
    theme: ThemeChoice,
    scale: u16,
    pending_scale_reveal: bool,
    scale_reveal_viewport: Option<(u64, egui::Rect)>,
    scale_reveal_frames_left: u8,
    scale_reveal_origin_frame: u64,
    // One later vertical contraction per selection episode; no timer or held-focus assumption.
    scale_reveal_watch: Option<(u64, egui::Rect)>,
    error: bool,
    commands: SyncSender<WorkerCommand>,
    results: Receiver<ResultMessage>,
    signals: Arc<Signals>,
    revision: u64,
    stopping: bool,
    active: Option<u64>,
    latest: Option<Write>,
}
impl AppearanceSettings {
    /// Blocking startup API: root calls only before a window/event loop is created.
    pub(crate) fn load_initial(data: PathBuf) -> Self {
        let initial = load_theme(&data);
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
        });
        let worker_signals = signals.clone();
        let started = thread::Builder::new()
            .name("dlpad-appearance".into())
            .spawn(move || {
                worker_entry(worker_signals.clone(), || {
                    writer(data, rx, tx, &worker_signals)
                });
            });
        let mut settings = Self {
            theme: initial.unwrap_or_default(),
            scale: 100,
            pending_scale_reveal: false,
            scale_reveal_viewport: None,
            scale_reveal_frames_left: 0,
            scale_reveal_origin_frame: 0,
            scale_reveal_watch: None,
            error: initial.is_err(),
            commands,
            results,
            signals,
            revision: 0,
            stopping: false,
            active: None,
            latest: None,
        };
        if started.is_err() {
            settings.error = true;
            settings.signals.stopped.store(true, Ordering::Release);
        }
        settings
    }
    #[cfg(test)]
    pub(crate) fn fixture_initial_scale(&mut self, scale: u16) {
        assert!(SCALES.contains(&scale));
        assert_eq!(self.revision, 0);
        assert!(self.active.is_none() && self.latest.is_none());
        self.scale = scale;
    }
    pub(crate) fn apply_to_context(&self, ctx: &egui::Context) {
        ctx.set_theme(self.theme.preference());
        let zoom = f32::from(self.scale) / 100.0;
        if ctx.cumulative_pass_nr() == 0 {
            // A fresh host has no previous viewport to rescale. Initialize before
            // its first input; subsequent zoom changes retain egui relayout.
            ctx.options_mut(|options| options.zoom_factor = zoom);
        } else {
            ctx.set_zoom_factor(zoom);
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.active.is_some() || self.latest.is_some()
    }
    pub(crate) fn request_stop(&mut self) {
        self.stopping = true;
        if !self.busy() {
            stop_worker(&self.signals, &self.commands);
        }
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire)
    }
    fn select_theme(&mut self, theme: ThemeChoice, ctx: &egui::Context) {
        if self.stopping || self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        let Some(revision) = self.revision.checked_add(1) else {
            self.error = true;
            return;
        };
        self.theme = theme;
        self.revision = revision;
        ctx.set_theme(theme.preference());
        let write = Write { revision, theme };
        if self.active.is_some() {
            self.latest = Some(write);
        } else {
            self.send(write);
        }
    }
    fn send(&mut self, write: Write) {
        match self.commands.try_send(WorkerCommand::Write(write)) {
            Ok(()) => self.active = Some(write.revision),
            Err(TrySendError::Full(WorkerCommand::Write(write))) => self.latest = Some(write),
            Err(TrySendError::Full(WorkerCommand::StopWake)) => {
                unreachable!("send only submits Write")
            }
            Err(TrySendError::Disconnected(_)) => {
                self.error = true;
                self.latest = None;
            }
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        *self
            .signals
            .context
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(ctx.clone());
        if self.signals.failed.load(Ordering::Acquire) {
            self.error = true;
            self.active = None;
            self.latest = None;
            return;
        }
        if let Ok(result) = self.results.try_recv()
            && self.active == Some(result.revision)
        {
            self.active = None;
            // Old completions cannot restore an obsolete visible theme/error.
            if result.revision == self.revision {
                self.error = !result.success;
            }
            if !self.signals.stop.load(Ordering::Acquire)
                && let Some(write) = self.latest.take()
            {
                self.send(write);
            }
        }
        if self.stopping && !self.busy() {
            stop_worker(&self.signals, &self.commands);
        }
        if self.busy() || (self.stopping && !self.stopped()) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let enabled =
            !self.stopping && !self.signals.stop.load(Ordering::Acquire) && !self.stopped();
        ui.add_enabled_ui(enabled, |ui| {
            let mut theme = self.theme;
            let (label, response, row_rect) = setting_row(ui, c::THEME, c::THEME_HELP, |ui| {
                let response = egui::ComboBox::from_id_salt("appearance-theme")
                    .width(160.0)
                    .selected_text(theme.label())
                    .show_ui(ui, |ui| {
                        for choice in [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark] {
                            if crate::ui_components::select(ui, &mut theme, choice, choice.label())
                                .clicked()
                            {
                                ui.close();
                            }
                        }
                    })
                    .response;
                crate::ui_components::combo_response(ui, response)
            });
            let response = response.labelled_by(label);
            if response.gained_focus()
                && ctx.cumulative_frame_nr() != self.scale_reveal_origin_frame
            {
                self.pending_scale_reveal = false;
                self.scale_reveal_viewport = None;
                self.scale_reveal_watch = None;
            }
            reveal_setting_context(ui, row_rect, &response);
            if theme != self.theme {
                self.select_theme(theme, ctx);
            }
            ui.separator();
            let old_scale = self.scale;
            let (label, response, row_rect) = setting_row(ui, c::SCALE, c::SCALE_HELP, |ui| {
                let response = egui::ComboBox::from_id_salt("appearance-scale")
                    .width(160.0)
                    .selected_text(format!("{}%", self.scale))
                    .show_ui(ui, |ui| {
                        for scale in SCALES {
                            if crate::ui_components::select(
                                ui,
                                &mut self.scale,
                                scale,
                                &format!("{scale}%"),
                            )
                            .clicked()
                            {
                                ui.close();
                            }
                        }
                    })
                    .response;
                crate::ui_components::combo_response(ui, response)
            });
            let response = response.labelled_by(label);
            reveal_setting_context(ui, row_rect, &response);
            let frame = ctx.cumulative_frame_nr();
            let viewport = ui.clip_rect();
            let interrupted = !enabled
                || ui.input(|input| {
                    input.raw_scroll_delta != egui::Vec2::ZERO || input.pointer.any_pressed()
                });
            // A selection event can appear in every pass of its own frame.
            // Cancel user navigation only after that protected origin frame.
            if frame != self.scale_reveal_origin_frame && interrupted {
                self.pending_scale_reveal = false;
                self.scale_reveal_viewport = None;
                self.scale_reveal_watch = None;
            }
            // Later keyboard or composition input ends the selection episode,
            // without cancelling the existing initial relayout request.
            if frame != self.scale_reveal_origin_frame
                && ui.input(|input| {
                    input.events.iter().any(|event| {
                        matches!(
                            event,
                            egui::Event::Key { pressed: true, .. }
                                | egui::Event::Text(_)
                                | egui::Event::Ime(_)
                        )
                    })
                })
            {
                self.scale_reveal_watch = None;
            }
            if !self.pending_scale_reveal
                && let Some((seen, previous)) = self.scale_reveal_watch
                && seen != frame
            {
                if frame > seen.saturating_add(1) {
                    // Settings was not drawn during an intervening app frame.
                    self.scale_reveal_watch = None;
                } else if viewport.height() + 0.5 < previous.height()
                    && !viewport.contains_rect(row_rect)
                {
                    // Consume the one late-contraction allowance; reuse the existing
                    // bounded relayout settling request without rearming the watch.
                    self.scale_reveal_watch = None;
                    self.pending_scale_reveal = true;
                    self.scale_reveal_viewport = None;
                    self.scale_reveal_frames_left = 8;
                    self.scale_reveal_origin_frame = frame;
                    ctx.request_repaint();
                } else {
                    self.scale_reveal_watch = Some((frame, viewport));
                }
            }
            // Zoom and wrapping panels can need another frame to agree on the
            // visible viewport. Never consume this request in the selection pass.
            if self.pending_scale_reveal
                && ctx.cumulative_frame_nr() != self.scale_reveal_origin_frame
            {
                let frame = ctx.cumulative_frame_nr();
                let viewport = ui.clip_rect();
                let previous = self.scale_reveal_viewport;
                if interrupted || self.scale_reveal_frames_left == 0 {
                    self.pending_scale_reveal = false;
                    self.scale_reveal_viewport = None;
                    self.scale_reveal_watch = None;
                } else if previous.is_none_or(|(seen, _)| seen != frame) {
                    self.scale_reveal_frames_left -= 1;
                    let zoom_applied =
                        (ctx.zoom_factor() - f32::from(self.scale) / 100.0).abs() < 0.001;
                    let stable = previous.is_some_and(|(_, clip)| clip == viewport);
                    if zoom_applied && stable {
                        reveal_setting_row(ui, row_rect, &response);
                        self.pending_scale_reveal = false;
                        self.scale_reveal_viewport = None;
                        self.scale_reveal_watch =
                            self.scale_reveal_watch.map(|_| (frame, viewport));
                    } else {
                        self.scale_reveal_viewport = Some((frame, viewport));
                        ctx.request_repaint();
                    }
                }
            }
            if old_scale != self.scale {
                ctx.set_zoom_factor(f32::from(self.scale) / 100.0);
                self.pending_scale_reveal = true;
                self.scale_reveal_viewport = None;
                self.scale_reveal_frames_left = 8;
                self.scale_reveal_origin_frame = ctx.cumulative_frame_nr();
                self.scale_reveal_watch = Some((self.scale_reveal_origin_frame, ui.clip_rect()));
                ctx.request_repaint();
            }
        });
        if self.error {
            ui.colored_label(ui.visuals().error_fg_color, c::APPEARANCE_FAILED);
        }
    }
}
impl Drop for AppearanceSettings {
    fn drop(&mut self) {
        stop_worker(&self.signals, &self.commands);
    }
}

// Reject known symlink components and non-regular settings before any SQLite/file I/O.
// This is not an ancestor-race proof; owned app-data containment remains a separate gate.
fn checked_path(path: &Path, directory: bool) -> Result<(), ()> {
    if !path.is_absolute() {
        return Err(());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => return Err(()),
            Component::Prefix(_) => {
                current.push(component);
                continue;
            }
            _ => current.push(component),
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(());
                }
                let last = current == path;
                if (!last || directory) && !metadata.is_dir() {
                    return Err(());
                }
                if last && !directory && !metadata.is_file() {
                    return Err(());
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(_) => return Err(()),
        }
    }
    Ok(())
}
fn database_path(data: &Path) -> Result<PathBuf, ()> {
    checked_path(data, true)?;
    let database = data.join("theme.sqlite");
    checked_path(&database, false)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        checked_path(&data.join(format!("theme.sqlite{suffix}")), false)?;
    }
    Ok(database)
}
fn load_theme(data: &Path) -> Result<ThemeChoice, ()> {
    let database = database_path(data)?;
    if database.try_exists().map_err(|_| ())?
        && let Some(value) =
            Store::read_setting_readonly(&database, "theme", MAX_THEME_BYTES).map_err(|_| ())?
    {
        return serde_json::from_str(&value).map_err(|_| ());
    }
    let legacy = data.join("theme.json");
    checked_path(&legacy, false)?;
    if !legacy.try_exists().map_err(|_| ())? {
        return Ok(ThemeChoice::System);
    }
    let mut bytes = Vec::new();
    File::open(legacy)
        .map_err(|_| ())?
        .take((MAX_THEME_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() > MAX_THEME_BYTES {
        return Err(());
    }
    serde_json::from_slice(&bytes).map_err(|_| ())
}
fn persist(data: &Path, theme: ThemeChoice) -> Result<(), ()> {
    let database = database_path(data)?;
    if database.try_exists().map_err(|_| ())? {
        // Existing invalid/oversized values are not silently repaired or overwritten.
        if let Some(value) =
            Store::read_setting_readonly(&database, "theme", MAX_THEME_BYTES).map_err(|_| ())?
        {
            serde_json::from_str::<ThemeChoice>(&value).map_err(|_| ())?;
        }
    }
    fs::create_dir_all(data).map_err(|_| ())?;
    let value = serde_json::to_string(&theme).map_err(|_| ())?;
    Store::open(&database)
        .and_then(|store| store.set_setting("theme", &value))
        .map_err(|_| ())
}
fn writer(
    data: PathBuf,
    rx: Receiver<WorkerCommand>,
    tx: SyncSender<ResultMessage>,
    signals: &Signals,
) {
    loop {
        if signals.stop.load(Ordering::Acquire) {
            return;
        }
        let write = match rx.recv() {
            Ok(WorkerCommand::Write(write)) => write,
            Ok(WorkerCommand::StopWake) => continue,
            Err(_) => return,
        };
        if signals.stop.load(Ordering::Acquire) {
            return;
        }
        let success =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| persist(&data, write.theme)))
                .is_ok_and(|value| value.is_ok());
        let mut result = ResultMessage {
            revision: write.revision,
            success,
        };
        loop {
            if signals.stop.load(Ordering::Acquire) {
                return;
            }
            match tx.try_send(result) {
                Ok(()) => {
                    signals.wake();
                    break;
                }
                Err(TrySendError::Disconnected(_)) => return,
                Err(TrySendError::Full(value)) => {
                    result = value;
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn actual_initial_appearance_scale_respects_first_viewport_and_later_deferred_zoom() {
        for scale in [100u16, 200] {
            let fixture = Fixture::new();
            let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
            settings.fixture_initial_scale(scale);
            let ctx = egui::Context::default();
            assert_eq!(ctx.cumulative_pass_nr(), 0);
            settings.apply_to_context(&ctx);
            let zoom = f32::from(scale) / 100.0;
            let expected = egui::vec2(960.0 / zoom, 640.0 / zoom);
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected)),
                ..Default::default()
            };
            raw.viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(1.0);
            let first = ctx.run(raw, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(c::SCALE);
                });
            });
            assert_eq!(
                ctx.viewport_rect().size(),
                expected,
                "real apply_to_context must not replace supplied first viewport with default10000"
            );
            assert_eq!(first.pixels_per_point, zoom);
            assert!(ctx.cumulative_pass_nr() > 0);
            settings.scale = if scale == 100 { 200 } else { 100 };
            let previous = ctx.zoom_factor();
            settings.apply_to_context(&ctx);
            assert_eq!(
                ctx.zoom_factor(),
                previous,
                "after a pass zoom keeps deferred relayout contract"
            );
            let next = ctx.run(
                egui::RawInput {
                    screen_rect: Some(ctx.viewport_rect()),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label(c::SCALE);
                    });
                },
            );
            assert_eq!(next.pixels_per_point, f32::from(settings.scale) / 100.0);
        }
    }

    use super::*;
    use std::{
        sync::atomic::AtomicU64,
        time::{Instant, SystemTime, UNIX_EPOCH},
    };
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            for _ in 0..256 {
                let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "dlpad-appearance-{}-{stamp}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path.canonicalize().unwrap()),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("owned fixture create failed: {error}"),
                }
            }
            panic!("fixture collision retry budget exceeded");
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn wait(settings: &mut AppearanceSettings, predicate: impl Fn(&AppearanceSettings) -> bool) {
        let ctx = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            settings.poll(&ctx);
            if predicate(settings) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "bounded appearance acknowledgment timed out"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(settings: &mut AppearanceSettings) {
        settings.request_stop();
        wait(settings, AppearanceSettings::stopped);
    }
    // Full current Settings page via public Candidate APIs; no mirrored TextEdit render.
    #[cfg(feature = "cpu")]
    struct SettingsImeFixture {
        app: Option<crate::accessible::Candidate>,
        fixture: Option<Fixture>,
        ctx: egui::Context,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        shapes: Vec<egui::epaint::ClippedShape>,
        time: f64,
        zoom: f32,
        closed: bool,
    }
    #[cfg(feature = "cpu")]
    impl SettingsImeFixture {
        fn new(zoom: f32, dark: bool) -> Self {
            let fixture = Fixture::new();
            let data = fixture.0.join("data");
            let media = fixture.0.join("media");
            fs::create_dir(&data).expect("owned data sibling");
            fs::create_dir(&media).expect("owned media sibling");
            let media = media
                .canonicalize()
                .expect("owned media absolute local path");
            let media_text = media.to_str().expect("owned media UTF8 path");
            // library_root is a plain path setting, not a JSON string. Close this
            // fixture setup connection before real Candidate recovery begins.
            let store = Store::open(&data.join("theme.sqlite")).expect("owned setup database");
            store
                .set_setting("library_root", media_text)
                .expect("owned library root seed");
            assert_eq!(
                store.setting("library_root").unwrap().as_deref(),
                Some(media_text)
            );
            drop(store);
            let app = crate::accessible::Candidate::new(data);
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_zoom_factor(zoom);
            ctx.enable_accesskit();
            let mut f = Self {
                app: Some(app),
                fixture: Some(fixture),
                ctx,
                nodes: Default::default(),
                shapes: vec![],
                time: 0.0,
                zoom,
                closed: false,
            };
            f.frame(vec![], false);
            let settings = f.button(c::SETTINGS);
            f.frame(
                vec![Self::action(settings, egui::accesskit::Action::Click)],
                false,
            );
            f.frame(vec![], false);
            let disclosure = f.button(c::INPUT_CHECK);
            f.frame(
                vec![Self::action(disclosure, egui::accesskit::Action::Expand)],
                false,
            );
            // Bounded original disclosure animation; no copied input widget or model seeding.
            for _ in 0..20 {
                f.frame(vec![], false);
            }
            let input = f.input().0;
            assert_eq!(
                input.0,
                egui::Id::new("settings-ime-input").value(),
                "setup exact product logical ID"
            );
            f.frame(
                vec![Self::action(input, egui::accesskit::Action::Focus)],
                false,
            );
            // Only original empty frames advance animated parent-scroll reveal.
            // No direct scroll, resize, style, extra Focus, or tolerance changes.
            let mut settled = false;
            let mut settle_frames = 0;
            for frames in 1..=32 {
                f.frame(vec![], false);
                settle_frames = frames;
                if f.input_frame_fully_painted() {
                    settled = true;
                    break;
                }
            }
            let (rect, clips) = f.input_frame_geometry();
            eprintln!(
                "DLPAD_IME_FOCUS_SETTLE zoom={zoom} dark={dark} frames={settle_frames} settled={settled} input={rect:?} frame_clips={clips:?}"
            );
            assert!(
                settled,
                "setup actual32px input frame remains outside original body clip after32 bounded empty frames"
            );
            assert_eq!(
                f.ctx.memory(|m| m.focused()).unwrap().value(),
                input.0,
                "setup actual field focus"
            );
            assert!(!f.input().1.is_disabled(), "setup actual editable field");
            f
        }
        fn frame(&mut self, events: Vec<egui::Event>, close: bool) {
            self.time += 1.0 / 60.0;
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0 / self.zoom, 600.0 / self.zoom),
                )),
                focused: true,
                time: Some(self.time),
                events,
                ..Default::default()
            };
            if close {
                raw.viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .events
                    .push(egui::ViewportEvent::Close);
            }
            let app = self.app.as_mut().unwrap();
            let output = self.ctx.run(raw, |ctx| app.show_ui(ctx));
            assert!(
                (output.pixels_per_point - self.zoom).abs() < 0.01,
                "actual output zoom"
            );
            self.closed |= output
                .viewport_output
                .values()
                .any(|v| v.commands.contains(&egui::ViewportCommand::Close));
            self.shapes = output.shapes;
            if let Some(update) = output.platform_output.accesskit_update {
                for (id, node) in update.nodes {
                    self.nodes.insert(id, node);
                }
            }
        }
        fn action(target: egui::accesskit::NodeId, action: egui::accesskit::Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action,
                target,
                data: None,
            })
        }
        fn button(&self, label: &str) -> egui::accesskit::NodeId {
            let found: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button && n.label() == Some(label)
                })
                .collect();
            assert_eq!(found.len(), 1, "setup unique actual button {label}");
            *found[0].0
        }
        fn input(&self) -> (egui::accesskit::NodeId, egui::accesskit::Node) {
            let found: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.role() == egui::accesskit::Role::TextInput
                        && n.labelled_by().iter().any(|id| {
                            self.nodes.get(id).is_some_and(|label| {
                                label.role() == egui::accesskit::Role::Label
                                    && label.value() == Some(c::IME)
                            })
                        })
                })
                .collect();
            assert_eq!(found.len(), 1, "setup actual labelled Settings IME field");
            (*found[0].0, found[0].1.clone())
        }
        fn text_shape(
            shape: &egui::epaint::Shape,
            value: &str,
            rect: egui::Rect,
            found: &mut Vec<egui::epaint::TextShape>,
        ) {
            match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.text() == value && rect.contains(text.pos) =>
                {
                    found.push(text.clone())
                }
                egui::epaint::Shape::Vec(shapes) => {
                    for shape in shapes {
                        Self::text_shape(shape, value, rect, found);
                    }
                }
                _ => {}
            }
        }
        fn input_frame_shape(shape: &egui::epaint::Shape, rect: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Rect(frame) => {
                    (frame.rect.min - rect.min).length() < 0.5
                        && (frame.rect.max - rect.max).length() < 0.5
                }
                egui::epaint::Shape::Vec(shapes) => shapes
                    .iter()
                    .any(|shape| Self::input_frame_shape(shape, rect)),
                _ => false,
            }
        }
        fn input_frame_geometry(&self) -> (egui::Rect, Vec<egui::Rect>) {
            let b = self.input().1.bounds().expect("actual IME input bounds");
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            let clips = self
                .shapes
                .iter()
                .filter(|shape| Self::input_frame_shape(&shape.shape, rect))
                .take(4)
                .map(|shape| shape.clip_rect)
                .collect();
            (rect, clips)
        }
        fn input_frame_fully_painted(&self) -> bool {
            let (rect, _) = self.input_frame_geometry();
            (rect.height() - 32.0).abs() < 0.5
                && self.shapes.iter().any(|shape| {
                    shape.clip_rect.contains_rect(rect)
                        && Self::input_frame_shape(&shape.shape, rect)
                })
        }
        fn assert_value_cursor_paint(&self, id: egui::Id, expected: &str) {
            let (node_id, node) = self.input();
            assert_eq!(node_id.0, id.value());
            assert_eq!(node.value(), Some(expected), "actual input node readback");
            assert_eq!(
                self.ctx.memory(|m| m.focused()),
                Some(id),
                "composition retains actual input focus"
            );
            let b = node.bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            assert!(
                (rect.height() - 32.0).abs() < 0.5,
                "original common input geometry"
            );
            let cursor = egui::TextEdit::load_state(&self.ctx, id)
                .unwrap()
                .cursor
                .char_range()
                .unwrap()
                .primary;
            let mut found = vec![];
            for shape in &self.shapes {
                let mut glyphs = vec![];
                Self::text_shape(&shape.shape, expected, rect, &mut glyphs);
                for text in glyphs {
                    found.push((text, shape.clip_rect));
                }
            }
            assert_eq!(
                found.len(),
                1,
                "actual current Settings field paints its exact galley"
            );
            let (text, clip) = &found[0];
            // TextEdit intentionally clips glyphs to its padded inner rectangle.
            // Use the actual outer frame paint for full-control/body containment.
            assert!(
                self.shapes
                    .iter()
                    .any(|shape| shape.clip_rect.contains_rect(rect)
                        && Self::input_frame_shape(&shape.shape, rect)),
                "actual settings scroll reveals full painted input frame: input={rect:?} zoom={}",
                self.zoom
            );
            let glyphs = text.galley.rect.translate(text.pos.to_vec2());
            assert!(
                rect.contains_rect(glyphs),
                "actual Korean/Latin glyphs within input"
            );
            let caret = text
                .galley
                .pos_from_cursor(cursor)
                .translate(text.pos.to_vec2());
            assert!(rect.contains(caret.center_top()) && rect.contains(caret.center_bottom()));
            assert!(
                clip.contains(caret.center_top()) && clip.contains(caret.center_bottom()),
                "actual primary caret within original body paint clip"
            );
        }
        fn finish(&mut self) {
            self.close_bounded();
            assert!(
                self.closed,
                "owned Candidate emitted Close only after its existing stop gates completed"
            );
        }
        fn close_bounded(&mut self) {
            if self.closed {
                return;
            }
            self.frame(vec![], true);
            let end = std::time::Instant::now() + Duration::from_secs(5);
            while !self.closed && std::time::Instant::now() < end {
                let stop = self
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button
                            && node.label() == Some(c::STOP_EXIT)
                            && !node.is_disabled()
                    })
                    .map(|(id, _)| *id);
                let events = stop
                    .map(|id| vec![Self::action(id, egui::accesskit::Action::Click)])
                    .unwrap_or_default();
                self.frame(events, false);
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
    #[cfg(feature = "cpu")]
    impl Drop for SettingsImeFixture {
        fn drop(&mut self) {
            // Keep a primary semantic assertion failure intact; cleanup never masks it.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.close_bounded()));
            drop(self.app.take());
            if !self.closed {
                // Unknown cleanup is not authority to delete the worker-owned fixture.
                if let Some(fixture) = self.fixture.take() {
                    std::mem::forget(fixture);
                }
                eprintln!(
                    "Settings IME fixture stop unconfirmed; owned temporary directory retained"
                );
            }
        }
    }

    #[cfg(feature = "cpu")]
    #[test]
    fn actual_settings_ime_moved_selection_rejects_stale_nonempty_commit() {
        for reverse in [false, true] {
            let mut f = SettingsImeFixture::new(1.0, false);
            let id = f.ctx.memory(|m| m.focused()).unwrap();
            f.frame(vec![egui::Event::Text("A한글Z".into())], false);
            let mut state = egui::TextEdit::load_state(&f.ctx, id).unwrap();
            let (secondary, primary) = if reverse { (3, 1) } else { (1, 3) };
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(secondary),
                    egui::text::CCursor::new(primary),
                )));
            state.store(&f.ctx, id);
            f.frame(vec![], false);
            f.frame(
                vec![
                    egui::Event::Ime(egui::ImeEvent::Enabled),
                    egui::Event::Ime(egui::ImeEvent::Preedit("조".into())),
                ],
                false,
            );
            f.assert_value_cursor_paint(id, "A조Z");
            // Real widget state movement, not physical key transport: move away from
            // the composition anchor so a later stale commit must not replace it.
            let mut state = egui::TextEdit::load_state(&f.ctx, id).unwrap();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(0),
                )));
            state.store(&f.ctx, id);
            f.frame(vec![], false);
            assert_eq!(
                egui::TextEdit::load_state(&f.ctx, id)
                    .unwrap()
                    .cursor
                    .char_range()
                    .unwrap()
                    .secondary
                    .index,
                0,
                "setup actual selection moved away"
            );
            f.frame(
                vec![egui::Event::Ime(egui::ImeEvent::Commit("한국어".into()))],
                false,
            );
            f.assert_value_cursor_paint(id, "A조Z");
            let range = egui::TextEdit::load_state(&f.ctx, id)
                .unwrap()
                .cursor
                .char_range()
                .unwrap();
            assert_eq!((range.primary.index, range.secondary.index), (0, 0));
            f.finish();
        }
    }

    #[cfg(feature = "cpu")]
    #[test]
    fn actual_settings_ime_positive_commit_replaces_directed_selection_and_preserves_caret_clip_100_200()
     {
        for zoom in [1.0, 2.0] {
            for dark in [false, true] {
                for reverse in [false, true] {
                    eprintln!("DLPAD_IME_CASE zoom={zoom} dark={dark} reverse={reverse}");
                    let mut f = SettingsImeFixture::new(zoom, dark);
                    let node_id = f.input().0;
                    let id = f.ctx.memory(|m| m.focused()).unwrap();
                    f.frame(vec![egui::Event::Text("A한글Z".into())], false);
                    f.assert_value_cursor_paint(id, "A한글Z");
                    let mut state = egui::TextEdit::load_state(&f.ctx, id).unwrap();
                    let (secondary, primary) = if reverse { (3, 1) } else { (1, 3) };
                    let directed = egui::text::CCursorRange::two(
                        egui::text::CCursor::new(secondary),
                        egui::text::CCursor::new(primary),
                    );
                    state.cursor.set_char_range(Some(directed));
                    state.store(&f.ctx, id);
                    f.frame(vec![], false);
                    assert_eq!(
                        egui::TextEdit::load_state(&f.ctx, id)
                            .unwrap()
                            .cursor
                            .char_range(),
                        Some(directed),
                        "setup actual directed selection retained"
                    );
                    f.frame(
                        vec![
                            egui::Event::Ime(egui::ImeEvent::Enabled),
                            egui::Event::Ime(egui::ImeEvent::Preedit("조".into())),
                        ],
                        false,
                    );
                    assert_eq!(f.input().0, node_id, "preedit retains provider identity");
                    f.assert_value_cursor_paint(id, "A조Z");
                    let precommit = egui::TextEdit::load_state(&f.ctx, id)
                        .unwrap()
                        .cursor
                        .char_range()
                        .unwrap();
                    eprintln!(
                        "DLPAD_IME_PRECOMMIT zoom={zoom} dark={dark} reverse={reverse} primary={} secondary={} focused_exact={}",
                        precommit.primary.index,
                        precommit.secondary.index,
                        f.ctx.memory(|m| m.focused()) == Some(id)
                    );
                    f.frame(
                        vec![egui::Event::Ime(egui::ImeEvent::Commit("한국어".into()))],
                        false,
                    );
                    let postcommit = egui::TextEdit::load_state(&f.ctx, id)
                        .unwrap()
                        .cursor
                        .char_range()
                        .unwrap();
                    eprintln!(
                        "DLPAD_IME_POSTCOMMIT zoom={zoom} dark={dark} reverse={reverse} primary={} secondary={} focused_exact={}",
                        postcommit.primary.index,
                        postcommit.secondary.index,
                        f.ctx.memory(|m| m.focused()) == Some(id)
                    );
                    assert_eq!(
                        f.input().0,
                        node_id,
                        "nonempty commit retains provider identity"
                    );
                    f.assert_value_cursor_paint(id, "A한국어Z");
                    let range = egui::TextEdit::load_state(&f.ctx, id)
                        .unwrap()
                        .cursor
                        .char_range()
                        .unwrap();
                    assert_eq!(
                        range.primary.index, 4,
                        "commit caret follows inserted Unicode scalars, reverse={reverse} zoom={zoom} dark={dark}"
                    );
                    assert_eq!(
                        range.secondary.index, 4,
                        "commit collapses directed selection"
                    );
                    f.frame(vec![egui::Event::Ime(egui::ImeEvent::Disabled)], false);
                    f.assert_value_cursor_paint(id, "A한국어Z");
                    f.finish();
                }
            }
        }
    }

    #[test]
    fn legacy_and_database_priority_are_compatible_without_startup_writes() {
        let fixture = Fixture::new();
        let legacy = fixture.0.join("theme.json");
        fs::write(&legacy, br#""Light""#).unwrap();
        assert_eq!(load_theme(&fixture.0).unwrap(), ThemeChoice::Light);
        assert!(!fixture.0.join("theme.sqlite").exists());
        persist(&fixture.0, ThemeChoice::Dark).unwrap();
        let database = fixture.0.join("theme.sqlite");
        let previous = fs::read(&database).unwrap();
        assert_eq!(load_theme(&fixture.0).unwrap(), ThemeChoice::Dark);
        assert_eq!(fs::read(database).unwrap(), previous);
        assert_eq!(fs::read(legacy).unwrap(), br#""Light""#);
    }
    #[test]
    fn malformed_and_oversized_legacy_defaults_system_and_preserves_bytes() {
        let fixture = Fixture::new();
        let path = fixture.0.join("theme.json");
        for bytes in [b"not-json".to_vec(), vec![b'x'; MAX_THEME_BYTES + 1]] {
            fs::write(&path, &bytes).unwrap();
            let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
            assert_eq!(settings.theme, ThemeChoice::System);
            assert!(settings.error);
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert!(!fixture.0.join("theme.sqlite").exists());
            stop(&mut settings);
        }
    }
    #[test]
    fn invalid_or_oversized_database_is_not_repaired_on_load_or_save() {
        let fixture = Fixture::new();
        let path = fixture.0.join("theme.sqlite");
        fs::write(&path, b"synthetic corrupt sqlite").unwrap();
        let previous = fs::read(&path).unwrap();
        assert!(load_theme(&fixture.0).is_err());
        assert!(persist(&fixture.0, ThemeChoice::Light).is_err());
        assert_eq!(fs::read(&path).unwrap(), previous);
        fs::remove_file(&path).unwrap();
        for value in ["not-json".to_owned(), "x".repeat(MAX_THEME_BYTES + 1)] {
            let store = Store::open(&path).unwrap();
            store.set_setting("theme", &value).unwrap();
            drop(store);
            let previous = fs::read(&path).unwrap();
            assert!(load_theme(&fixture.0).is_err());
            assert!(persist(&fixture.0, ThemeChoice::Dark).is_err());
            assert_eq!(fs::read(&path).unwrap(), previous);
        }
    }
    #[test]
    fn rapid_changes_coalesce_to_latest_theme_without_stale_result() {
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        let ctx = egui::Context::default();
        settings.select_theme(ThemeChoice::Light, &ctx);
        settings.select_theme(ThemeChoice::Dark, &ctx);
        settings.select_theme(ThemeChoice::System, &ctx);
        assert_eq!(settings.latest.unwrap().theme, ThemeChoice::System);
        wait(&mut settings, |settings| !settings.busy());
        assert_eq!(settings.theme, ThemeChoice::System);
        assert!(!settings.error);
        assert_eq!(
            Store::read_setting_readonly(&fixture.0.join("theme.sqlite"), "theme", MAX_THEME_BYTES)
                .unwrap()
                .as_deref(),
            Some("\"System\"")
        );
        stop(&mut settings);
    }
    #[test]
    fn stop_drains_accepted_latest_selection_and_blocks_new_choices() {
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        let ctx = egui::Context::default();
        settings.select_theme(ThemeChoice::Light, &ctx);
        settings.select_theme(ThemeChoice::Dark, &ctx);
        settings.request_stop();
        let revision = settings.revision;
        settings.select_theme(ThemeChoice::System, &ctx);
        assert_eq!(settings.revision, revision);
        wait(&mut settings, AppearanceSettings::stopped);
        assert_eq!(load_theme(&fixture.0).unwrap(), ThemeChoice::Dark);
        assert_eq!(settings.theme, ThemeChoice::Dark);
    }
    #[test]
    fn obsolete_completion_does_not_replace_visible_theme_or_error() {
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        stop(&mut settings);
        let (tx, rx) = mpsc::sync_channel(1);
        settings.results = rx;
        settings.active = Some(1);
        settings.revision = 2;
        settings.theme = ThemeChoice::Dark;
        settings.error = false;
        tx.send(ResultMessage {
            revision: 1,
            success: false,
        })
        .unwrap();
        settings.poll(&egui::Context::default());
        assert_eq!(settings.theme, ThemeChoice::Dark);
        assert!(!settings.error);
        assert!(settings.active.is_none());
    }
    #[test]
    fn worker_panic_acknowledges_stop_with_fixed_failure() {
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        stop(&mut settings);
        settings.signals.stopped.store(false, Ordering::Release);
        let signals = settings.signals.clone();
        thread::spawn(move || worker_entry(signals, || panic!("synthetic appearance fault")))
            .join()
            .unwrap();
        assert!(settings.stopped());
        settings.poll(&egui::Context::default());
        assert!(settings.error);
        assert!(!settings.busy());
    }
    #[test]
    fn scale_catalog_is_finite_and_matches_requested_range() {
        assert_eq!(SCALES, [75, 100, 125, 150, 175, 200]);
        for scale in SCALES {
            assert!((0.75..=2.0).contains(&(f32::from(scale) / 100.0)));
        }
    }
    #[test]
    fn initial_context_and_system_events_preserve_explicit_theme_choice() {
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        let ctx = egui::Context::default();
        settings.scale = 175;
        settings.apply_to_context(&ctx);
        let _ = ctx.run(
            egui::RawInput {
                system_theme: Some(egui::Theme::Light),
                ..Default::default()
            },
            |ctx| {
                assert_eq!(ctx.theme(), egui::Theme::Light);
                assert!((ctx.zoom_factor() - 1.75).abs() < f32::EPSILON);
            },
        );
        let _ = ctx.run(
            egui::RawInput {
                system_theme: Some(egui::Theme::Dark),
                ..Default::default()
            },
            |ctx| {
                assert_eq!(ctx.theme(), egui::Theme::Dark);
            },
        );
        settings.select_theme(ThemeChoice::Light, &ctx);
        let _ = ctx.run(
            egui::RawInput {
                system_theme: Some(egui::Theme::Dark),
                ..Default::default()
            },
            |ctx| {
                assert_eq!(ctx.theme(), egui::Theme::Light);
            },
        );
        stop(&mut settings);
    }
    #[cfg(unix)]
    #[test]
    fn symlink_settings_and_sidecar_are_rejected_without_touching_target() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let target = fixture.0.join("owned-foreign-fixture");
        fs::write(&target, br#""Dark""#).unwrap();
        let legacy = fixture.0.join("theme.json");
        symlink(&target, &legacy).unwrap();
        assert!(load_theme(&fixture.0).is_err());
        assert_eq!(fs::read(&target).unwrap(), br#""Dark""#);
        fs::remove_file(legacy).unwrap();
        symlink(&target, fixture.0.join("theme.sqlite-wal")).unwrap();
        assert!(persist(&fixture.0, ThemeChoice::Light).is_err());
        assert_eq!(fs::read(target).unwrap(), br#""Dark""#);
    }
    #[cfg(unix)]
    #[test]
    fn denied_legacy_read_preserves_existing_bytes() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::new();
        let path = fixture.0.join("theme.json");
        fs::write(&path, br#""Dark""#).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
        // root/capability environments may bypass permission; do not claim denial there.
        if File::open(&path).is_err() {
            assert!(load_theme(&fixture.0).is_err());
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(fs::read(path).unwrap(), br#""Dark""#);
    }
    #[test]
    fn actual_settings_focus_keeps_stacked_label_help_and_control_in_200_percent_viewport() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        for physical_width in [960.0, 800.0] {
            let fixture = Fixture::new();
            let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
            settings.scale = 200;
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_zoom_factor(2.0);
            ctx.enable_accesskit();
            let mut nodes = HashMap::<NodeId, Node>::new();
            let draw = |settings: &mut AppearanceSettings, frame: u32, events: Vec<egui::Event>| {
                let mut viewport = egui::Rect::NOTHING;
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(physical_width / 2.0, 640.0 / 2.0),
                        )),
                        focused: true,
                        time: Some(f64::from(frame) * 0.016),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.heading(c::SETTINGS);
                            ui.add_space(12.0);
                            let area = egui::ScrollArea::vertical()
                                .id_salt("actual-settings-focus-context")
                                .max_height(100.0);
                            let area = if frame == 0 {
                                area.vertical_scroll_offset(80.0)
                            } else {
                                area
                            };
                            area.show(ui, |ui| {
                                viewport = ui.clip_rect();
                                settings.show(ui, ctx);
                                ui.add_space(200.0);
                            });
                        });
                    },
                );
                assert!((output.pixels_per_point - 2.0).abs() < 0.01);
                (
                    output
                        .platform_output
                        .accesskit_update
                        .expect("actual settings AccessKit tree"),
                    viewport,
                )
            };
            for (id, node) in draw(&mut settings, 0, vec![]).0.nodes {
                nodes.insert(id, node);
            }
            let label_id = *nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Label
                        && (node.label() == Some(c::THEME) || node.value() == Some(c::THEME))
                })
                .unwrap()
                .0;
            let control_id = *nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::ComboBox && node.labelled_by().contains(&label_id)
                })
                .unwrap()
                .0;
            let focus = egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: control_id,
                data: None,
            });
            for (id, node) in draw(&mut settings, 1, vec![focus]).0.nodes {
                nodes.insert(id, node);
            }
            let mut viewport = egui::Rect::NOTHING;
            for frame in 2..12 {
                let (update, clip) = draw(&mut settings, frame, vec![]);
                viewport = clip;
                for (id, node) in update.nodes {
                    nodes.insert(id, node);
                }
            }
            for text in [c::THEME, c::THEME_HELP] {
                let node = nodes
                    .values()
                    .find(|node| {
                        node.role() == Role::Label
                            && (node.label() == Some(text) || node.value() == Some(text))
                    })
                    .unwrap();
                let b = node.bounds().expect("actual description bounds");
                let rect = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                assert!(
                    viewport.contains_rect(rect),
                    "200% {physical_width}px: description {text} {rect:?} outside {viewport:?}"
                );
            }
            let control = nodes.get(&control_id).unwrap();
            let b = control.bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            assert!(viewport.contains_rect(rect));
            assert!(!control.is_disabled());
            assert_eq!(settings.scale, 200, "focus is not a scale selection");
            assert_eq!(settings.theme, ThemeChoice::System);
            stop(&mut settings);
        }
    }
    #[test]
    fn actual_scale_selection_reveals_selected_row_after_200_percent_relayout() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        for physical_width in [960.0, 800.0] {
            let fixture = Fixture::new();
            let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            let mut nodes = HashMap::<NodeId, Node>::new();
            let draw = |settings: &mut AppearanceSettings, frame: u32, events: Vec<egui::Event>| {
                let mut viewport = egui::Rect::NOTHING;
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(
                                physical_width / ctx.zoom_factor(),
                                640.0 / ctx.zoom_factor(),
                            ),
                        )),
                        focused: true,
                        time: Some(f64::from(frame) * 0.016),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.heading(c::SETTINGS);
                            ui.add_space(12.0);
                            let area = egui::ScrollArea::vertical()
                                .id_salt("actual-settings-focus-context")
                                .max_height(100.0);
                            area.show(ui, |ui| {
                                viewport = ui.clip_rect();
                                settings.show(ui, ctx);
                                ui.add_space(200.0);
                            });
                        });
                    },
                );
                (
                    output
                        .platform_output
                        .accesskit_update
                        .expect("actual settings AccessKit tree"),
                    viewport,
                )
            };
            for (id, node) in draw(&mut settings, 0, vec![]).0.nodes {
                nodes.insert(id, node);
            }
            let label_id = *nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Label
                        && (node.label() == Some(c::SCALE) || node.value() == Some(c::SCALE))
                })
                .unwrap()
                .0;
            let control_id = *nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::ComboBox && node.labelled_by().contains(&label_id)
                })
                .unwrap()
                .0;
            let click = |target| {
                egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Click,
                    target,
                    data: None,
                })
            };
            for (id, node) in draw(&mut settings, 1, vec![click(control_id)]).0.nodes {
                nodes.insert(id, node);
            }
            let choice_id = *nodes
                .iter()
                .find(|(_, node)| {
                    node.supports_action(Action::Click)
                        && (node.label() == Some("200%") || node.value() == Some("200%"))
                })
                .expect("actual 200 percent popup choice")
                .0;
            for (id, node) in draw(&mut settings, 2, vec![click(choice_id)]).0.nodes {
                nodes.insert(id, node);
            }
            assert_eq!(settings.scale, 200, "actual popup selection changed scale");
            let mut viewport = egui::Rect::NOTHING;
            for frame in 3..20 {
                let (update, clip) = draw(&mut settings, frame, vec![]);
                viewport = clip;
                for (id, node) in update.nodes {
                    nodes.insert(id, node);
                }
            }
            for text in [c::SCALE, c::SCALE_HELP] {
                let node = nodes
                    .values()
                    .find(|node| {
                        node.role() == Role::Label
                            && (node.label() == Some(text) || node.value() == Some(text))
                    })
                    .unwrap();
                let b = node.bounds().expect("actual description bounds");
                let rect = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                assert!(
                    viewport.contains_rect(rect),
                    "200% {physical_width}px: description {text} {rect:?} outside {viewport:?}"
                );
            }
            let control = nodes.get(&control_id).unwrap();
            let b = control.bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            assert!(viewport.contains_rect(rect));
            assert!(!control.is_disabled());
            assert_eq!(settings.scale, 200, "selected scale remains unchanged");
            assert_eq!(settings.theme, ThemeChoice::System);
            assert!((ctx.zoom_factor() - 2.0).abs() < 0.01);
            stop(&mut settings);
        }
    }
    #[test]
    fn actual_pointer_scale_selection_keeps_reveal_through_selection_frame_multipass() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        let fixture = Fixture::new();
        let mut settings = AppearanceSettings::load_initial(fixture.0.clone());
        let ctx = egui::Context::default();
        crate::ui_theme::configure_dlpad_theme(&ctx);
        ctx.enable_accesskit();
        let mut nodes = HashMap::<NodeId, Node>::new();
        let release_frame = std::cell::Cell::new(u32::MAX);
        let draw = |settings: &mut AppearanceSettings, frame: u32, events| {
            let mut requested = false;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0 / ctx.zoom_factor(), 600.0 / ctx.zoom_factor()),
                    )),
                    focused: true,
                    time: Some(f64::from(frame) * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("pointer-scale-multipass-owned")
                            .max_height(200.0)
                            .show(ui, |ui| settings.show(ui, ctx));
                    });
                    if frame == release_frame.get() && !requested {
                        requested = true;
                        ctx.request_discard("owned pointer selection multipass regression");
                    }
                },
            )
        };
        let remember = |nodes: &mut HashMap<NodeId, Node>, out: &egui::FullOutput| {
            for (id, node) in &out.platform_output.accesskit_update.as_ref().unwrap().nodes {
                nodes.insert(*id, node.clone());
            }
        };
        remember(&mut nodes, &draw(&mut settings, 0, vec![]));
        let label = *nodes
            .iter()
            .find(|(_, n)| {
                n.role() == Role::Label
                    && (n.label() == Some(c::SCALE) || n.value() == Some(c::SCALE))
            })
            .unwrap()
            .0;
        let combo = *nodes
            .iter()
            .find(|(_, n)| n.role() == Role::ComboBox && n.labelled_by().contains(&label))
            .unwrap()
            .0;
        let focused = draw(
            &mut settings,
            1,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: combo,
                data: None,
            })],
        );
        remember(&mut nodes, &focused);
        let mut settled = None;
        for frame in 2..14 {
            let out = draw(&mut settings, frame, vec![]);
            remember(&mut nodes, &out);
            settled = Some(out);
        }
        let cb = nodes[&combo].bounds().unwrap();
        let control = egui::Rect::from_min_max(
            egui::pos2(cb.x0 as f32, cb.y0 as f32),
            egui::pos2(cb.x1 as f32, cb.y1 as f32),
        );
        fn painted_background(shape: &egui::epaint::Shape, rect: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Rect(s) => {
                    (s.rect.min - rect.min).length() < 0.5 && (s.rect.max - rect.max).length() < 0.5
                }
                egui::epaint::Shape::Vec(items) => {
                    items.iter().any(|s| painted_background(s, rect))
                }
                _ => false,
            }
        }
        assert!(
            settled.unwrap().shapes.iter().any(
                |s| s.clip_rect.contains_rect(control) && painted_background(&s.shape, control)
            ),
            "actual initialScale control focused and fully painted before openingpopup"
        );
        remember(
            &mut nodes,
            &draw(
                &mut settings,
                14,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Click,
                    target: combo,
                    data: None,
                })],
            ),
        );
        remember(&mut nodes, &draw(&mut settings, 15, vec![]));
        let choice_bounds = |nodes: &HashMap<NodeId, Node>| {
            let choice = nodes
                .values()
                .find(|n| {
                    n.supports_action(Action::Click)
                        && (n.label() == Some("200%") || n.value() == Some("200%"))
                })
                .expect("actual popup200 clickable node");
            let b = choice.bounds().unwrap();
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        };
        fn visible_choice(shape: &egui::epaint::Shape, clip: egui::Rect, rect: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Text(t) => {
                    t.galley.text() == "200%"
                        && clip.contains_rect(rect)
                        && rect.contains_rect(t.galley.rect.translate(t.pos.to_vec2()))
                }
                egui::epaint::Shape::Vec(shapes) => {
                    shapes.iter().any(|s| visible_choice(s, clip, rect))
                }
                _ => false,
            }
        }
        fn diagnostics(
            shape: &egui::epaint::Shape,
            clip: egui::Rect,
            rows: &mut Vec<(egui::Rect, egui::Rect)>,
        ) {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == "200%" => {
                    rows.push((clip, t.galley.rect.translate(t.pos.to_vec2())))
                }
                egui::epaint::Shape::Vec(items) => {
                    for s in items {
                        diagnostics(s, clip, rows)
                    }
                }
                _ => {}
            }
        }
        let mut frame = 16;
        let mut popup = draw(&mut settings, frame, vec![]);
        remember(&mut nodes, &popup);
        let mut wheel_steps = 0;
        for _ in 0..12 {
            let rect = choice_bounds(&nodes);
            if popup
                .shapes
                .iter()
                .any(|s| visible_choice(&s.shape, s.clip_rect, rect))
            {
                break;
            }
            let mut painted = Vec::new();
            for shape in &popup.shapes {
                diagnostics(&shape.shape, shape.clip_rect, &mut painted);
            }
            let clip = painted.first().expect("actualpopup200 painter clip").0;
            let anchor = nodes
                .values()
                .filter(|n| n.supports_action(Action::Click) && n.role() != Role::ComboBox)
                .filter(|n| {
                    ["75%", "100%", "125%", "150%", "175%", "200%"]
                        .iter()
                        .any(|label| n.label() == Some(*label))
                })
                .filter_map(|n| n.bounds())
                .map(|b| {
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    )
                })
                .find(|rect| clip.contains_rect(*rect))
                .expect("actual visiblepopup option wheelanchor")
                .center();
            frame += 1;
            let out = draw(
                &mut settings,
                frame,
                vec![egui::Event::PointerMoved(anchor)],
            );
            remember(&mut nodes, &out);
            frame += 1;
            popup = draw(
                &mut settings,
                frame,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -40.0),
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            remember(&mut nodes, &popup);
            wheel_steps += 1;
            for _ in 0..8 {
                frame += 1;
                popup = draw(&mut settings, frame, vec![]);
                remember(&mut nodes, &popup);
            }
        }
        assert!(
            wheel_steps > 0,
            "bounded actual dropdownwheel exercised for clippedlastoption"
        );
        let rect = choice_bounds(&nodes);
        let pos = rect.center();
        frame += 1;
        let hovered = draw(&mut settings, frame, vec![egui::Event::PointerMoved(pos)]);
        remember(&mut nodes, &hovered);
        assert_eq!(
            choice_bounds(&nodes),
            rect,
            "actual popup row settled before pointer press"
        );
        let mut popup_paint = Vec::new();
        for shape in &hovered.shapes {
            diagnostics(&shape.shape, shape.clip_rect, &mut popup_paint);
        }
        assert!(
            hovered
                .shapes
                .iter()
                .any(|s| visible_choice(&s.shape, s.clip_rect, rect)),
            "actual popup200 glyph/row fully painted before pointer: node={rect:?} textclip/glyph={popup_paint:?}"
        );
        frame += 1;
        let press = draw(
            &mut settings,
            frame,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        remember(&mut nodes, &press);
        assert_eq!(
            settings.scale, 100,
            "pointer press alone has not selected row"
        );
        assert_eq!(
            choice_bounds(&nodes),
            rect,
            "popup row remains original through owned press"
        );
        frame += 1;
        release_frame.set(frame);
        let out = draw(
            &mut settings,
            frame,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(
            out.platform_output.num_completed_passes >= 2,
            "actual forced multipass"
        );
        assert_eq!(settings.scale, 200, "actual pointer popup selection");
        assert!(
            settings.pending_scale_reveal,
            "selection-frame multipass must preserve its pending reveal"
        );
        for frame in frame + 1..frame + 10 {
            let _ = draw(&mut settings, frame, vec![]);
        }
        assert!(
            !settings.pending_scale_reveal,
            "bounded request completes on later stable frames"
        );
        stop(&mut settings);
    }
}

#[cfg(test)]
mod stop_notification_tests {
    use super::*;
    fn signals() -> Arc<Signals> {
        Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
        })
    }
    #[test]
    fn idle_notification_wakes_actual_writer_and_ack_without_database_creation() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let data = std::env::temp_dir().join(format!(
            "dlpad-appearance-no-write-{}-{stamp}",
            std::process::id()
        ));
        assert!(!data.exists());
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, _results) = mpsc::sync_channel(1);
        let state = signals();
        let worker = state.clone();
        let target = data.clone();
        let (ready, started) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            worker_entry(worker.clone(), || {
                ready.send(()).unwrap();
                writer(target, rx, tx, &worker);
            })
        });
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        stop_worker(&state, &commands);
        let end = std::time::Instant::now() + Duration::from_secs(2);
        while !state.stopped.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < end);
            thread::sleep(Duration::from_millis(1));
        }
        handle.join().unwrap();
        assert!(!data.exists());
        assert!(!state.failed.load(Ordering::Acquire));
    }
    #[test]
    fn full_stop_notification_preserves_original_accepted_write() {
        let (commands, rx) = mpsc::sync_channel(1);
        let state = signals();
        commands
            .try_send(WorkerCommand::Write(Write {
                revision: 7,
                theme: ThemeChoice::Dark,
            }))
            .ok()
            .unwrap();
        stop_worker(&state, &commands);
        assert!(state.stop.load(Ordering::Acquire));
        match rx.try_recv().unwrap() {
            WorkerCommand::Write(write) => {
                assert_eq!(write.revision, 7);
                assert_eq!(write.theme, ThemeChoice::Dark);
            }
            _ => panic!("accepted write displaced"),
        }
        assert!(rx.try_recv().is_err());
    }
    #[test]
    fn disconnected_stop_notification_is_nonblocking_and_writer_acknowledges() {
        let (commands, rx) = mpsc::sync_channel(1);
        drop(rx);
        let state = signals();
        stop_worker(&state, &commands);
        assert!(state.stop.load(Ordering::Acquire));
        state.stop.store(false, Ordering::Release);
        let (commands, rx) = mpsc::sync_channel(1);
        drop(commands);
        let (tx, _results) = mpsc::sync_channel(1);
        worker_entry(state.clone(), || {
            writer(PathBuf::from("unused-disconnected"), rx, tx, &state)
        });
        assert!(state.stopped.load(Ordering::Acquire));
    }
    // Insert inside existing stop_notification_tests module.
    #[test]
    fn stop_before_wait_aborts_queued_write_without_creating_database_and_acks() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let data = std::env::temp_dir().join(format!(
            "dlpad-appearance-stopped-before-wait-{}-{stamp}",
            std::process::id()
        ));
        assert!(!data.exists());
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, _results) = mpsc::sync_channel(1);
        let state = signals();
        commands
            .try_send(WorkerCommand::Write(Write {
                revision: 9,
                theme: ThemeChoice::Dark,
            }))
            .ok()
            .unwrap();
        // Mirrors immediate abort/Drop, not graceful busy request_stop.
        stop_worker(&state, &commands);
        worker_entry(state.clone(), || writer(data.clone(), rx, tx, &state));
        assert!(state.stopped.load(Ordering::Acquire));
        assert!(!state.failed.load(Ordering::Acquire));
        assert!(!data.exists());
    }
}

/// Settings keep description and control aligned; large app zoom stacks them.
fn setting_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    help: &str,
    control: impl FnOnce(&mut egui::Ui) -> R,
) -> (egui::Id, R, egui::Rect) {
    let wide = ui.available_width() >= 640.0;
    let left_width = (ui.available_width() - 188.0).max(160.0);
    let contents = |ui: &mut egui::Ui| {
        let id = ui
            .allocate_ui_with_layout(
                egui::vec2(
                    if wide {
                        left_width
                    } else {
                        ui.available_width()
                    },
                    0.0,
                ),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if wide {
                        ui.set_min_width(left_width);
                    }
                    let id = ui.label(label).id;
                    ui.weak(help);
                    id
                },
            )
            .inner;
        let value = ui.vertical(control).inner;
        (id, value)
    };
    let result = if wide {
        ui.horizontal_top(contents)
    } else {
        ui.vertical(contents)
    };
    ui.add_space(crate::ui_tokens::LABEL_GAP);
    (result.inner.0, result.inner.1, result.response.rect)
}

/// Keep the input's meaning visible when its complete row can fit the viewport.
fn reveal_setting_context(ui: &mut egui::Ui, row: egui::Rect, response: &egui::Response) {
    if response.gained_focus() {
        reveal_setting_row(ui, row, response);
    }
}

fn reveal_setting_row(ui: &mut egui::Ui, row: egui::Rect, response: &egui::Response) {
    let viewport = ui.clip_rect();
    if row.width() <= viewport.width() && row.height() <= viewport.height() {
        // A fitting row can still exceed egui's implicit two-sided spacing.
        // Center removes that extra margin requirement without clipping meaning.
        ui.scroll_to_rect(row, Some(egui::Align::Center));
    } else {
        response.scroll_to_me(None);
    }
}
