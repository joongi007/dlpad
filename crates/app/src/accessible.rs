//! Accessible native vertical slice; domain policy and blocking I/O stay in the engine worker.
use crate::catalog as c;
use dlpad_core::{
    diagnostics::{DiagnosticReport, FailureStage, ToolComponent},
    options::DownloadOptions,
};
use dlpad_engine::{
    download::{DownloadError, DownloadRepository, DownloadSession},
    metrics::ProgressFeed,
};
use eframe::egui;
#[cfg(feature = "cpu")]
use std::time::Instant;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// Fresh current-file observations only; 100% does not mark the job downloaded.
fn show_download_progress(
    ui: &mut egui::Ui,
    sample: Option<&dlpad_engine::metrics::DownloadMetrics>,
    processing: bool,
) {
    let fraction = sample.and_then(|sample| sample.transfer_fraction());
    let label = if processing {
        c::STATE_POSTPROCESSING
    } else if sample.is_some_and(|sample| sample.transfer_finished) {
        c::DOWNLOAD_PROGRESS_FINISHING
    } else if fraction.is_some_and(|(_, estimated)| estimated) {
        c::DOWNLOAD_PROGRESS_ESTIMATED
    } else if fraction.is_some() {
        c::DOWNLOAD_PROGRESS
    } else {
        c::DOWNLOAD_PROGRESS_WAITING
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        crate::ui_components::help(
            ui,
            "download-transfer-scope",
            c::DOWNLOAD_PROGRESS,
            c::DOWNLOAD_PROGRESS_HELP,
        );
        if let Some(sample) = sample {
            ui.weak(crate::display_rate::format_bytes_per_second(
                sample.bytes_per_second,
            ));
            if let Some(eta) = sample.eta_seconds {
                ui.weak(format!("{} {eta:.0} {}", c::ETA, c::SECONDS));
            }
        }
    });
    let value = fraction.map_or(0.0, |(value, _)| value);
    let bar = egui::ProgressBar::new(value)
        .desired_width(ui.available_width())
        .desired_height(14.0);
    ui.add(if fraction.is_some() {
        bar.text(format!("{:.1}%", (value * 1000.0).floor() / 10.0))
    } else {
        bar
    });
}

/// Attach the target open state, independently of the body animation fraction.
pub(crate) fn collapsing<R>(
    ui: &mut egui::Ui,
    title: impl Into<egui::WidgetText>,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::CollapsingResponse<R> {
    let response = ui.collapsing(title, body);
    let header = &response.header_response;
    if header.gained_focus() {
        header.scroll_to_me(None);
    }
    if let Some(mut state) = egui::collapsing_header::CollapsingState::load(&header.ctx, header.id)
    {
        if header.enabled() {
            let (expand, collapse) = header.ctx.input(|input| {
                (
                    input.num_accesskit_action_requests(header.id, egui::accesskit::Action::Expand)
                        > 0,
                    input.num_accesskit_action_requests(
                        header.id,
                        egui::accesskit::Action::Collapse,
                    ) > 0,
                )
            });
            if expand || collapse {
                state.set_open(!collapse);
                state.store(&header.ctx);
                header.ctx.request_repaint();
            }
        }
        header.ctx.accesskit_node_builder(header.id, |node| {
            node.set_expanded(state.is_open());
            // accesskit_windows 0.29 exposes Toggle, but has no ExpandCollapse provider.
            // Preserve expanded semantics and offer the equivalent native toggle action.
            node.set_toggled(if state.is_open() {
                egui::accesskit::Toggled::True
            } else {
                egui::accesskit::Toggled::False
            });
            if header.enabled() {
                node.add_action(egui::accesskit::Action::Click);
                node.add_action(egui::accesskit::Action::Expand);
                node.add_action(egui::accesskit::Action::Collapse);
            }
        });
    }
    response
}

/// Reveal a newly opened critical message once, after its animation has settled.
/// Later wheel input never arms another reveal while the section remains open.
#[derive(Default)]
struct NetworkReveal {
    was_open: bool,
    pending: bool,
    scheduled_frame: Option<u64>,
}
impl NetworkReveal {
    fn update(&mut self, response: &egui::CollapsingResponse<Option<egui::Response>>) {
        let header = &response.header_response;
        let Some(state) = egui::collapsing_header::CollapsingState::load(&header.ctx, header.id)
        else {
            return;
        };
        let open = state.is_open();
        if open && !self.was_open {
            self.pending = response.body_returned.as_ref().is_some_and(Option::is_some)
                || !cfg!(target_os = "linux");
        }
        if !open {
            self.pending = false;
            self.scheduled_frame = None;
        }
        self.was_open = open;
        let frame = header.ctx.cumulative_frame_nr();
        // All passes of the scheduling frame repeat the request. A later
        // request_discard cannot consume the pending intent on a discarded pass.
        if self
            .scheduled_frame
            .is_some_and(|scheduled| scheduled < frame)
        {
            self.pending = false;
            self.scheduled_frame = None;
        }
        if self.pending
            && state.openness(&header.ctx) >= 1.0
            && let Some(Some(critical)) = response.body_returned.as_ref()
        {
            critical.scroll_to_me(Some(egui::Align::Min));
            self.scheduled_frame = Some(frame);
        }
    }
}

/// Parent-owned disclosure transition: report replacement never re-arms this intent.
#[derive(Default)]
struct DiagnosticOpenReveal {
    initialized: bool,
    was_open: bool,
    pending: bool,
    scheduled_frame: Option<u64>,
}
impl DiagnosticOpenReveal {
    fn target(
        &mut self,
        response: &egui::CollapsingResponse<(Option<egui::Rect>, Option<egui::Rect>)>,
    ) -> Option<egui::Rect> {
        let ctx = &response.header_response.ctx;
        let state =
            egui::collapsing_header::CollapsingState::load(ctx, response.header_response.id)?;
        let frame = ctx.cumulative_frame_nr();
        let open = state.is_open();
        if !self.initialized {
            self.initialized = true;
            self.was_open = open;
            return None;
        }
        if open && !self.was_open {
            self.pending = true;
        }
        if !open
            || ctx.input(|input| input.raw_scroll_delta != egui::Vec2::ZERO)
            || self
                .scheduled_frame
                .is_some_and(|scheduled| scheduled < frame)
        {
            self.pending = false;
            self.scheduled_frame = None;
        }
        self.was_open = open;
        if self.pending
            && state.openness(ctx) >= 1.0
            && let Some((_, Some(target))) = response.body_returned.as_ref()
        {
            self.scheduled_frame = Some(frame);
            return Some(*target);
        }
        None
    }
}

pub fn run(data: PathBuf) -> Result<(), eframe::Error> {
    // Font I/O happens before a UI thread exists; candidate font discovery remains bounded.
    let font = font_bytes();
    let candidate = Candidate::new(data);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_icon(egui::IconData {
                rgba: crate::app_icon::rgba(32),
                width: 32,
                height: 32,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "dlpad",
        options,
        Box::new(move |cc| {
            configure_context(&cc.egui_ctx, font);
            candidate.apply_appearance(&cc.egui_ctx);
            Ok(Box::new(candidate))
        }),
    )
}
pub(crate) struct StartupFonts {
    korean: Option<Vec<u8>>,
    system_ui: Option<Vec<u8>>,
    symbols: Option<Vec<u8>>,
}

pub(crate) fn configure_context(ctx: &egui::Context, font: StartupFonts) {
    let mut fonts = egui::FontDefinitions::default();
    let mut names = Vec::new();
    if let Some(bytes) = font.system_ui {
        fonts
            .font_data
            .insert("system-ui".into(), egui::FontData::from_owned(bytes).into());
        names.push("system-ui".to_owned());
    }
    if let Some(bytes) = font.korean {
        fonts
            .font_data
            .insert("korean".into(), egui::FontData::from_owned(bytes).into());
        names.push("korean".to_owned());
    }
    // Segoe UI Symbol supplies monochrome outline emoji absent from the
    // bundled Noto snapshot. It follows UI/Korean fonts, preserving typography.
    if let Some(bytes) = font.symbols {
        fonts.font_data.insert(
            "windows-symbols".into(),
            egui::FontData::from_owned(bytes).into(),
        );
        names.push("windows-symbols".to_owned());
        // Retain Hack/Ubuntu first for diagnostic monospace; symbols precede
        // the existing emoji fallback only.
        let monospace = fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default();
        let index = monospace
            .iter()
            .position(|name| name == "NotoEmoji-Regular")
            .unwrap_or(monospace.len());
        monospace.insert(index, "windows-symbols".to_owned());
    }
    if !names.is_empty() {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .splice(0..0, names);
        ctx.set_fonts(fonts);
    }
    crate::ui_theme::configure_dlpad_theme(ctx);
}

/// Fonts are read before the window/event loop starts. No UI-thread font I/O.
pub(crate) fn font_bytes() -> StartupFonts {
    let mut paths = Vec::new();
    let custom = std::env::var_os("DLPAD_TEST_FONT");
    if let Some(path) = &custom {
        paths.push(PathBuf::from(path));
    }
    paths.extend(
        [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "C:/Windows/Fonts/malgun.ttf",
        ]
        .map(PathBuf::from),
    );
    let korean = paths.into_iter().find_map(read_font_path);
    // A fixture override retains its exact single-font semantics.
    let system_ui = if cfg!(windows) && custom.is_none() {
        read_font_path(PathBuf::from("C:/Windows/Fonts/segoeui.ttf"))
    } else {
        None
    };
    let symbols = if cfg!(windows) && custom.is_none() {
        read_font_path(PathBuf::from("C:/Windows/Fonts/seguisym.ttf"))
    } else {
        None
    };
    StartupFonts {
        korean,
        system_ui,
        symbols,
    }
}
fn read_font_path(path: PathBuf) -> Option<Vec<u8>> {
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    read_bounded_font(std::fs::File::open(path).ok()?, metadata.len()).ok()
}

const MAX_FONT_BYTES: u64 = 32 * 1024 * 1024;

fn read_bounded_font(mut reader: impl Read, expected: u64) -> std::io::Result<Vec<u8>> {
    if expected == 0 || expected > MAX_FONT_BYTES {
        return Err(std::io::Error::other("Font size rejected"));
    }
    let size =
        usize::try_from(expected).map_err(|_| std::io::Error::other("Font size rejected"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| std::io::Error::other("Font allocation rejected"))?;
    bytes.resize(size, 0);
    reader.read_exact(&mut bytes)?;
    let mut extra = [0u8; 1];
    loop {
        match reader.read(&mut extra) {
            Ok(0) => return Ok(bytes),
            Ok(_) => return Err(std::io::Error::other("Font changed during read")),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

struct Worker {
    handle: JoinHandle<()>,
    receiver: Receiver<Completion>,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
}
impl Worker {
    // Called only by the UI owner. The first intent wins; pause is visible before cancel.
    fn request_stop(&self, pause: bool) {
        if !self.cancel.load(Ordering::Acquire) {
            self.pause.store(pause, Ordering::Release);
            self.cancel.store(true, Ordering::Release);
        }
    }
}
// Blocking recovery-only helper. Never request/create a default Documents location.
fn restore_saved_library_root(
    data: &std::path::Path,
) -> Result<Option<PathBuf>, dlpad_engine::library::LibraryError> {
    let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite"))?;
    if store.setting("library_root")?.is_none() {
        return Ok(None);
    }
    drop(store);
    // The setting can change between reads. Missing on second read must fail,
    // never fall back to the default provider or create a default media folder.
    dlpad_engine::location::prepare_with_default(
        data,
        dlpad_engine::location::LocationRequest::Restore,
        || Err(dlpad_engine::library::LibraryError::DocumentsUnavailable),
    )
    .map(Some)
}

// Resolve first-run presentation without creating a media directory.
fn initial_library_root(
    data: &std::path::Path,
    default: impl FnOnce() -> Result<PathBuf, dlpad_engine::library::LibraryError>,
) -> Result<Option<PathBuf>, dlpad_engine::library::LibraryError> {
    if let Some(saved) = restore_saved_library_root(data)? {
        return Ok(Some(saved));
    }
    let root = default()?;
    if !root.is_absolute() || root.to_str().is_none_or(|text| text.len() > 32 * 1024) {
        return Err(dlpad_engine::library::LibraryError::UnsupportedPath);
    }
    Ok(Some(root))
}

#[derive(Default)]
struct RecoveryState {
    cookie_pending: bool,
    managed_pending: bool,
    restored_library_root: Option<PathBuf>,
    library_restore_failed: bool,
    restored_tools: Vec<(crate::managed::ToolId, PathBuf)>,
    managed_inspection: Option<crate::accessible_tools::ManagedInspection>,
}
enum Completion {
    Recovery(Result<RecoveryState, String>),
    Job {
        status: String,
        report: Option<Box<DiagnosticReport>>,
        resume: Option<String>,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsSection {
    #[default]
    General,
    Tools,
    SaveLocation,
    AppUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureToolIssue {
    Setup,
    InvalidPath,
    Rejected(&'static str),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FeatureToolNeed {
    tool: crate::managed::ToolId,
    issue: FeatureToolIssue,
}
struct LocationOutputIntent {
    serial: u64,
    text: String,
    original: Option<(String, PathBuf)>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum UpdateRecoveryNotice {
    #[default]
    Unseen,
    Visible,
    Dismissed,
}

pub(crate) struct Candidate {
    scroll_trace_remaining: u16,
    scroll_trace_armed: bool,
    data: PathBuf,
    activation: Option<dlpad_engine::activation::ActivationServer>,
    hidden: bool,
    url: String,
    native_drop_fallback: bool,
    tool: String,
    tool_preferences: crate::accessible_tool_preferences::Controller,
    output: String,
    output_original: Option<(String, PathBuf)>,
    location_output_intent: Option<LocationOutputIntent>,
    ffmpeg: String,
    deno: String,
    cookie_source: String,
    worker: Option<Worker>,
    #[cfg(windows)]
    local_export: crate::local_export_ui::LocalExportUi,
    progress: ProgressFeed,
    ready: bool,
    cookie_recovery_pending: bool,
    managed_recovery_pending: bool,
    status: String,
    report: Option<Box<DiagnosticReport>>,
    diagnostics: Option<crate::accessible_diagnostics::DiagnosticPanel>,
    resume: Option<String>,
    close_prompt: bool,
    close_wait: bool,
    settings: bool,
    settings_section: SettingsSection,
    app_update_panel: Option<crate::accessible_app_update::AppUpdatePanel>,
    update_recovery_notice: UpdateRecoveryNotice,
    network_reveal: NetworkReveal,
    diagnostic_open_reveal: DiagnosticOpenReveal,
    start_condition_reveal_frame: Option<u64>,
    start_condition_error_target: Option<egui::Rect>,
    library_open: bool,
    tools_open: bool,
    tool_return_pending: bool,
    history_open: bool,
    tools: Option<crate::accessible_tools::Panel>,
    history: Option<crate::accessible_history::HistoryPanel>,
    library: Option<crate::accessible_library::LibraryPanel>,
    form: crate::accessible_options::DownloadForm,
    media: crate::media_workspace::Workspace,
    skip_preview: bool,
    appearance: crate::accessible_settings::AppearanceSettings,
    metrics: crate::accessible_metrics::MetricsPanel,
    report_stress: bool,
    plan_preview: String,
    plan_pages: crate::accessible_preview::PreviewPages,
    ime: String,
    path_picker: crate::path_picker::PathPicker,
    tool_path_picker: crate::tool_path_picker::PathPicker,
}
#[cfg(feature = "cpu")]
#[allow(
    dead_code,
    reason = "The eframe host compiles this shared Candidate but keeps egui freshness scheduling."
)]
impl Candidate {
    pub(crate) fn use_host_metrics_freshness_timer(&mut self) {
        self.metrics.use_host_freshness_timer();
    }
    pub(crate) fn metrics_freshness_deadline(&self, now: Instant) -> Option<Instant> {
        self.metrics.freshness_deadline(now)
    }
    pub(crate) fn check_metrics_freshness(&mut self, now: Instant) -> bool {
        self.metrics.check_freshness(now)
    }
    pub(crate) fn metrics_frame_presented(&mut self) {
        self.metrics.frame_presented();
    }
}

impl Candidate {
    #[allow(
        dead_code,
        reason = "The native host publishes runtime readiness after a successful frame"
    )]
    pub(crate) fn update_frame_ready(&self) -> bool {
        self.ready
    }
    fn observe_update_recovery(&mut self, snapshot: &crate::app_update_client::Snapshot) {
        if self.update_recovery_notice == UpdateRecoveryNotice::Unseen
            && snapshot.runtime_ready
            && snapshot.phase == crate::app_update_client::Phase::Failed
            && snapshot.generation == 0
            && snapshot.error == Some(13)
        {
            self.update_recovery_notice = UpdateRecoveryNotice::Visible;
        }
    }
    fn show_update_recovery_notice(&mut self, ctx: &egui::Context) {
        if self.update_recovery_notice != UpdateRecoveryNotice::Visible {
            return;
        }
        egui::TopBottomPanel::top("update-recovery-notice")
            .frame(
                egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(
                    crate::ui_tokens::PAGE_MARGIN as i8,
                    8,
                )),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(c::APP_UPDATE_RECOVERED);
                    if crate::ui_components::secondary(ui, c::APP_UPDATE_RECOVERY_CONFIRM, true)
                        .clicked()
                    {
                        self.update_recovery_notice = UpdateRecoveryNotice::Dismissed;
                    }
                });
            });
    }
    fn app_update_busy(&self) -> bool {
        crate::app_update_client::handle().is_some_and(|client| client.busy())
            || self
                .app_update_panel
                .as_ref()
                .is_some_and(|panel| panel.busy())
    }
    fn app_update_stopped(&self) -> bool {
        crate::app_update_client::handle().is_none_or(|client| client.stopped())
            && self
                .app_update_panel
                .as_ref()
                .is_none_or(|panel| panel.stopped())
    }
    fn update_tasks_idle(&self) -> bool {
        self.worker.is_none()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && !self.tool_preferences.busy()
            && !self.appearance.busy()
            && !self.media.busy()
            && !self.media.closing()
            && !self.media.fatal()
            && !self.local_export_blocks()
            && self.local_export_close_ready()
            && self.tools.as_ref().is_none_or(|panel| !panel.busy())
            && self.library.as_ref().is_none_or(|panel| !panel.busy())
            && self.history.as_ref().is_none_or(|panel| !panel.busy())
            && self.diagnostics.as_ref().is_none_or(|panel| !panel.busy())
    }
    fn stop_app_update(&mut self) {
        // The client is installed before this lazy panel is ever constructed.
        if let Some(client) = crate::app_update_client::handle() {
            client.begin_shutdown();
        }
        if let Some(panel) = &mut self.app_update_panel {
            panel.request_stop();
        }
    }
    fn begin_update_close_wait(&mut self) {
        self.close_wait = true;
        self.path_picker.cancel();
        self.tool_path_picker.cancel();
        self.media.request_stop();
        self.stop_local_export();
        self.stop_app_update();
        self.appearance.request_stop();
        self.tool_preferences.request_stop();
        self.metrics.request_stop();
        if let Some(panel) = &mut self.tools {
            panel.request_stop();
        }
        if let Some(panel) = &mut self.history {
            panel.request_stop();
        }
        if let Some(panel) = &mut self.library {
            panel.request_stop();
        }
        if let Some(panel) = &mut self.diagnostics {
            panel.request_stop();
        }
        if let Some(worker) = &self.worker {
            worker.request_stop(false);
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[allow(dead_code, reason = "Used by the native CPU window host")]
    pub(crate) fn set_folder_picker_owner(&mut self, owner: std::sync::Arc<winit::window::Window>) {
        self.path_picker.set_owner(owner.clone());
        self.tool_path_picker.set_owner(owner.clone());
        if let Some(panel) = &mut self.diagnostics {
            panel.set_picker_owner(owner);
        }
    }
    #[allow(dead_code, reason = "Used by the native CPU window host")]
    pub(crate) fn folder_picker_busy(&self) -> bool {
        (self.path_picker.busy() || self.tool_path_picker.busy())
            || self.diagnostics.as_ref().is_some_and(|p| p.picker_busy())
            || !self.media.stopped()
            || !self.local_export_close_ready()
            || !self.app_update_stopped()
    }
    #[allow(dead_code, reason = "Used by the native CPU window host")]
    pub(crate) fn cancel_folder_picker(&mut self) {
        self.stop_app_update();
        self.path_picker.cancel();
        self.tool_path_picker.cancel();
        if let Some(panel) = &self.diagnostics {
            panel.cancel_picker();
        }
        self.media.request_stop();
        self.stop_local_export();
    }
    pub(crate) fn poll_folder_picker(&mut self, ctx: &egui::Context) {
        if self.media.closing() {
            self.media.poll(
                ctx,
                &self.url,
                &self.tool,
                &self.ffmpeg,
                &self.deno,
                &self.data.join("managed-tools"),
            );
        }
        if self.close_wait || ctx.input(|input| input.viewport().close_requested()) {
            self.path_picker.cancel();
            self.tool_path_picker.cancel();
            if let Some(panel) = &self.diagnostics {
                panel.cancel_picker();
            }
        }
        if let Some(panel) = &mut self.diagnostics {
            panel.poll(ctx);
        }
        if let Some((target, result)) = self.tool_path_picker.poll(ctx)
            && !self.close_wait
            && !self.local_export_blocks()
        {
            match result {
                Ok(Some(path)) => {
                    if let Some(value) = path
                        .to_str()
                        .filter(|value| !value.is_empty() && value.len() <= 32768)
                    {
                        let index = match target {
                            crate::tool_path_picker::Target::YtDlp => {
                                self.tool = value.into();
                                0
                            }
                            crate::tool_path_picker::Target::Ffmpeg => {
                                self.ffmpeg = value.into();
                                1
                            }
                            crate::tool_path_picker::Target::Deno => {
                                self.deno = value.into();
                                2
                            }
                        };
                        self.tool_preferences.edited(index);
                    } else {
                        self.status = c::TOOL_PICKER_FAILED.into();
                    }
                }
                Ok(None) => {}
                Err(()) => self.status = c::TOOL_PICKER_FAILED.into(),
            }
        }
        if let Some((target, result)) = self.path_picker.poll(ctx) {
            if self.close_wait || self.local_export_blocks() {
                return;
            }
            match result {
                Ok(Some(path)) => match target {
                    crate::path_picker::Target::Download => self.set_output_path(path),
                    crate::path_picker::Target::Library => {
                        if let Some(library) = &mut self.library {
                            library.apply_selected_root(path);
                        }
                        self.capture_location_request();
                    }
                    crate::path_picker::Target::LibraryExport { generation } => {
                        if let Some(library) = &mut self.library {
                            library.apply_export_folder(generation, path);
                        }
                    }
                },
                Ok(None) => {}
                Err(()) => self.status = c::FOLDER_PICKER_FAILED.into(),
            }
        }
    }
    fn apply_managed_tool(&mut self, tool: crate::managed::ToolId, path: &Path) {
        if self.close_wait
            || self.worker.is_some()
            || self.local_export_blocks()
            || self.app_update_busy()
        {
            return;
        }
        let Some(path) = path.to_str() else {
            self.status = c::TOOL_FAILED.into();
            return;
        };
        self.tool_preferences.managed_selected(
            tool_preference_index(tool),
            path,
            [&mut self.tool, &mut self.ffmpeg, &mut self.deno],
        );
    }

    fn apply_restored_managed_tools(&mut self, tools: Vec<(crate::managed::ToolId, PathBuf)>) {
        if self.close_wait
            || self.worker.is_some()
            || self.local_export_blocks()
            || self.app_update_busy()
        {
            return;
        }
        for (tool, path) in tools {
            if let Some(path) = path.to_str() {
                self.tool_preferences.managed_restored(
                    tool_preference_index(tool),
                    path,
                    [&mut self.tool, &mut self.ffmpeg, &mut self.deno],
                );
            }
        }
    }
    fn apply_managed_inspection(&mut self, inspection: crate::accessible_tools::ManagedInspection) {
        if self.close_wait {
            return;
        }
        for (index, value) in inspection.slots.into_iter().enumerate() {
            use crate::accessible_tool_preferences::ManagedAvailability;
            use crate::accessible_tools::Availability;
            let value = match value {
                Availability::Checking => ManagedAvailability::Checking,
                Availability::Available(path) => path
                    .to_str()
                    .map(|v| ManagedAvailability::Available(v.to_owned()))
                    .unwrap_or(ManagedAvailability::Unavailable(c::TOOL_AUTO_UNAVAILABLE)),
                Availability::Unavailable(reason) => ManagedAvailability::Unavailable(reason),
            };
            self.tool_preferences.set_managed_availability(index, value);
        }
    }
    pub(crate) fn hidden(&self) -> bool {
        self.hidden
    }
    pub(crate) fn apply_appearance(&self, ctx: &egui::Context) {
        self.appearance.apply_to_context(ctx);
        if let Some(server) = &self.activation {
            let wake = ctx.clone();
            server.set_wake_callback(move || wake.request_repaint());
        }
    }
    pub(crate) fn new(data: PathBuf) -> Self {
        let activation = dlpad_engine::activation::ActivationServer::bind(&data).ok();
        let appearance = crate::accessible_settings::AppearanceSettings::load_initial(data.clone());
        let tool_environment = ["DLPAD_YTDLP", "DLPAD_FFMPEG", "DLPAD_DENO"]
            .map(|key| std::env::var_os(key).map(|value| value.to_string_lossy().into_owned()));
        let tool_preferences = crate::accessible_tool_preferences::Controller::new(
            data.clone(),
            tool_environment.clone(),
        );
        let mut app = Self {
            scroll_trace_armed: false,
            scroll_trace_remaining: if cfg!(all(windows, feature = "first-paint-fixture"))
                && std::env::var_os("DLPAD_TEST_SCROLL_TRACE").is_some_and(|v| v == "1")
            {
                128
            } else {
                0
            },
            data,
            activation,
            hidden: false,
            url: String::new(),
            native_drop_fallback: false,
            tool: tool_environment[0].clone().unwrap_or_default(),
            tool_preferences,
            output: std::env::var("DLPAD_LIBRARY_ROOT").unwrap_or_default(),
            output_original: None,
            location_output_intent: None,
            ffmpeg: tool_environment[1].clone().unwrap_or_default(),
            deno: tool_environment[2].clone().unwrap_or_default(),
            cookie_source: String::new(),
            worker: None,
            #[cfg(windows)]
            local_export: crate::local_export_ui::LocalExportUi::default(),
            progress: ProgressFeed::default(),
            ready: false,
            cookie_recovery_pending: false,
            managed_recovery_pending: false,
            status: c::RECOVERY_CHECKING.into(),
            report: None,
            diagnostics: None,
            resume: None,
            close_prompt: false,
            close_wait: false,
            settings: std::env::args().any(|a| a == "--settings"),
            settings_section: SettingsSection::General,
            app_update_panel: None,
            update_recovery_notice: UpdateRecoveryNotice::Unseen,
            network_reveal: NetworkReveal::default(),
            diagnostic_open_reveal: DiagnosticOpenReveal::default(),
            start_condition_reveal_frame: None,
            start_condition_error_target: None,
            library_open: false,
            tools_open: false,
            tool_return_pending: false,
            history_open: false,
            tools: None,
            history: None,
            library: None,
            form: crate::accessible_options::DownloadForm::default(),
            media: crate::media_workspace::Workspace::default(),
            skip_preview: false,
            appearance,
            metrics: crate::accessible_metrics::MetricsPanel::new(),
            plan_preview: String::new(),
            plan_pages: crate::accessible_preview::PreviewPages::default(),
            report_stress: std::env::args().any(|arg| arg == "--fixture")
                && std::env::var_os("DLPAD_TEST_REPORT_STRESS").is_some_and(|v| v == "1"),
            ime: String::new(),
            path_picker: crate::path_picker::PathPicker::default(),
            tool_path_picker: crate::tool_path_picker::PathPicker::default(),
        };
        app.recover();
        app
    }
    fn spawn(
        &mut self,
        operation: impl FnOnce(Arc<AtomicBool>, Arc<AtomicBool>) -> Completion + Send + 'static,
    ) {
        if self.worker.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let (worker_cancel, worker_pause) = (cancel.clone(), pause.clone());
        let handle = thread::spawn(move || {
            let result = operation(worker_cancel, worker_pause);
            let _ = sender.send(result);
        });
        self.worker = Some(Worker {
            handle,
            receiver,
            cancel,
            pause,
        });
    }
    fn can_recover(&self) -> bool {
        !self.close_wait
            && !self.app_update_busy()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && self.worker.is_none()
            && !self.tools.as_ref().is_some_and(|p| p.busy())
            && !self.history.as_ref().is_some_and(|p| p.busy())
            && !self.library.as_ref().is_some_and(|p| p.busy())
            && !self.diagnostics.as_ref().is_some_and(|p| p.busy())
    }
    fn recover(&mut self) {
        if !self.can_recover() {
            return;
        }
        self.ready = false;
        self.status = c::RECOVERY_CHECKING.into();
        let data = self.data.clone();
        let restore_library_root = self.output.is_empty();
        let restore_tools = [
            (
                crate::managed::ToolId::YtDlp,
                self.tool.is_empty() && std::env::var_os("DLPAD_YTDLP").is_none(),
            ),
            (
                crate::managed::ToolId::Ffmpeg,
                self.ffmpeg.is_empty() && std::env::var_os("DLPAD_FFMPEG").is_none(),
            ),
            (
                crate::managed::ToolId::Deno,
                self.deno.is_empty() && std::env::var_os("DLPAD_DENO").is_none(),
            ),
        ];
        self.spawn(move |cancel, _| {
            Completion::Recovery((|| -> Result<RecoveryState, String> {
                #[cfg(not(any(target_os = "linux", windows)))]
                let pending = false;
                #[cfg(any(target_os = "linux", windows))]
                let pending;
                #[cfg(target_os = "linux")]
                {
                    let cache = data.join("session-cookies");
                    pending = match std::fs::symlink_metadata(&cache) {
                        Ok(_) => {
                            let summary = dlpad_engine::cookies::recover_sessions(&cache)
                                .map_err(|_| c::COOKIE_RECOVERY_FAILED.to_owned())?;
                            summary.deferred > 0 || summary.rejected > 0 || summary.truncated
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                        Err(_) => return Err(c::COOKIE_RECOVERY_FAILED.to_owned()),
                    };
                }
                #[cfg(windows)]
                {
                    let cache = data.join("session-cookies");
                    pending = match std::fs::symlink_metadata(&cache) {
                        Ok(_) => {
                            let summary = dlpad_engine::cookies::recover_sessions(&cache)
                                .map_err(|_| c::COOKIE_RECOVERY_FAILED.to_owned())?;
                            summary.deferred > 0 || summary.rejected > 0 || summary.truncated
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                        Err(_) => return Err(c::COOKIE_RECOVERY_FAILED.to_owned()),
                    };
                }
                #[cfg(not(windows))]
                let managed_pending = false;
                #[cfg(windows)]
                let managed_pending = {
                    let root = data.join("managed-tools");
                    match std::fs::symlink_metadata(&root) {
                        Ok(_) => {
                            dlpad_engine::tools::ToolManager::new(&root)
                                .and_then(|manager| manager.recover_execution())
                                .map_err(|_| c::TOOL_RECOVERY_FAILED.to_owned())?
                                .pending
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                        Err(_) => return Err(c::TOOL_RECOVERY_FAILED.to_owned()),
                    }
                };
                dlpad_engine::download::recover_at_startup(&data)
                    .map_err(|_| c::RECOVERY_FAILED.to_owned())?;
                let restored = if restore_library_root {
                    initial_library_root(&data, || {
                        #[cfg(not(test))]
                        {
                            dlpad_engine::library::default_library_root()
                        }
                        // Unit fixtures never inspect or create user Documents.
                        #[cfg(test)]
                        {
                            Ok(data.parent().unwrap_or(&data).join("default-media"))
                        }
                    })
                    .map_err(|_| ())
                } else {
                    Ok(None)
                };
                let library_restore_failed = restored.is_err();
                let managed_inspection = if managed_pending {
                    crate::accessible_tools::ManagedInspection {
                        slots: std::array::from_fn(|_| {
                            crate::accessible_tools::Availability::Checking
                        }),
                    }
                } else {
                    crate::accessible_tools::inspect_managed(&data, &cancel)?
                };
                // A failed preference read never grants Automatic restoration.
                // The preferences controller separately presents its storage error.
                let saved_auto =
                    crate::accessible_tool_preferences::automatic_modes_for_managed_restore(&data)
                        .unwrap_or([false; 3]);
                let requested = std::array::from_fn(|index| {
                    (
                        restore_tools[index].0,
                        restore_tools[index].1 && saved_auto[index],
                    )
                });
                let restored_tools = if managed_pending {
                    Vec::new()
                } else {
                    crate::accessible_tools::restore_active(&data, &requested, &cancel)?
                };
                Ok(RecoveryState {
                    cookie_pending: pending,
                    managed_pending,
                    restored_library_root: restored.unwrap_or(None),
                    library_restore_failed,
                    restored_tools,
                    managed_inspection: Some(managed_inspection),
                })
            })())
        });
    }
    fn poll(&mut self) {
        if !self.worker.as_ref().is_some_and(|w| w.handle.is_finished()) {
            return;
        }
        let worker = self.worker.take().expect("checked worker");
        let result = worker.receiver.try_recv();
        let joined = worker.handle.join();
        if joined.is_err() {
            self.ready = false;
            self.status = c::FAILED.into();
            return;
        }
        match result {
            Ok(Completion::Recovery(Ok(state))) => {
                if self.output.is_empty()
                    && !self.close_wait
                    && let Some(root) = state.restored_library_root
                {
                    self.set_output_path(root.clone());
                    if let Some(library) = &mut self.library {
                        library.apply_observed_root(root);
                    }
                }
                if !state.managed_pending {
                    if let Some(inspection) = state.managed_inspection {
                        self.apply_managed_inspection(inspection);
                    }
                    self.apply_restored_managed_tools(state.restored_tools);
                }
                self.ready = true;
                self.cookie_recovery_pending = state.cookie_pending;
                self.managed_recovery_pending = state.managed_pending;
                self.status = if state.managed_pending {
                    c::TOOL_EXECUTION_PENDING
                } else if state.library_restore_failed {
                    c::LOCATION_FAILED
                } else {
                    c::RECOVERY_DONE
                }
                .into();
            }
            Ok(Completion::Recovery(Err(message))) => {
                self.status = message;
                self.ready = false;
            }
            Ok(Completion::Job {
                status,
                report,
                resume,
            }) => {
                self.status = status;
                self.report = report;
                self.resume = resume;
                self.progress = ProgressFeed::default();
            }
            Err(_) => {
                self.ready = false;
                self.status = c::FAILED.into();
            }
        }
    }
    fn set_output_path(&mut self, path: PathBuf) {
        self.output = crate::path_picker::display_path(&path);
        self.output_original = Some((self.output.clone(), path));
    }
    fn output_path(&self) -> PathBuf {
        match &self.output_original {
            Some((display, original)) if display == &self.output => original.clone(),
            _ => PathBuf::from(&self.output),
        }
    }
    // Validate the draft for a read-only preview; this grants no execution authority.
    fn draft_options(&self) -> Result<DownloadOptions, &'static str> {
        let output = self.output_path();
        let ffmpeg = (!self.ffmpeg.is_empty()).then(|| PathBuf::from(&self.ffmpeg));
        let mut options = self
            .form
            .build(self.url.trim(), &output, ffmpeg.as_deref())
            .map_err(|e| e.0)?;
        options.deno = (!self.deno.is_empty()).then(|| PathBuf::from(&self.deno));
        options.plan().map_err(|e| e.0)?;
        Ok(options)
    }
    fn options(&self) -> Result<DownloadOptions, &'static str> {
        let options = self.draft_options()?;
        if !PathBuf::from(&self.tool).is_absolute() {
            return Err(c::TOOL_PATH);
        }
        Ok(options)
    }
    fn start(&mut self, resume: bool) {
        if self.app_update_busy() || self.path_picker.busy() || self.tool_path_picker.busy() {
            return;
        }
        if self.media.busy()
            || self.media.fatal()
            || !self.ready
            || self.tool_preferences.busy()
            || self.worker.is_some()
            || self.close_wait
            || self.local_export_blocks()
            || self.tools.as_ref().is_some_and(|p| p.busy())
            || self.history.as_ref().is_some_and(|p| p.busy())
            || self.library.as_ref().is_some_and(|p| p.busy())
            || self.diagnostics.as_ref().is_some_and(|p| p.busy())
        {
            return;
        }
        let options = if resume {
            None
        } else {
            match self.options() {
                Ok(o) => Some(o),
                Err(error) => {
                    self.status = error.into();
                    return;
                }
            }
        };
        let saved_id = if resume {
            match self.resume.clone() {
                Some(id) => Some(id),
                None => return,
            }
        } else {
            None
        };
        self.status = c::DOWNLOADING.into();
        self.report = None;
        self.diagnostics = None;
        self.progress = ProgressFeed::default();
        let (data, tool, feed) = (
            self.data.clone(),
            PathBuf::from(&self.tool),
            self.progress.clone(),
        );
        let cookie_source = std::mem::take(&mut self.cookie_source);
        let request = WorkerData {
            data,
            cookie_source: (!cookie_source.is_empty()).then(|| PathBuf::from(cookie_source)),
        };
        self.spawn(move |cancel, pause| {
            execute(request, tool, options, saved_id, cancel, pause, feed)
        });
    }
}
impl Drop for Candidate {
    fn drop(&mut self) {
        self.stop_app_update();
        self.appearance.request_stop();
        if let Some(panel) = &mut self.app_update_panel {
            panel.request_stop();
        }
        self.tool_preferences.request_stop();
        self.metrics.request_stop();
        if let Some(tools) = &mut self.tools {
            tools.request_stop();
        }
        if let Some(history) = &mut self.history {
            history.request_stop();
        }
        if let Some(worker) = self.worker.take() {
            worker.request_stop(false);
            if worker.handle.is_finished() {
                let _ = worker.handle.join();
            }
        }
    }
}
impl eframe::App for Candidate {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_ui(ctx);
    }
}
impl Candidate {
    fn show_plan(&mut self, ui: &mut egui::Ui) {
        let rendered = self
            .draft_options()
            .map_err(|_| crate::accessible_plan::PreviewError::InvalidOptions)
            .and_then(|options| crate::accessible_plan::preview(&options));
        match rendered {
            Ok(preview) => {
                if self.plan_preview != preview {
                    self.plan_preview = preview;
                    self.plan_pages.reset(&self.plan_preview);
                }
                ui.label(c::PLAN_PREVIEW_HELP);
                ui.horizontal(|ui| {
                    if crate::ui_components::secondary(
                        ui,
                        c::REPORT_PREVIEW_PREVIOUS,
                        self.plan_pages.index() > 0,
                    )
                    .clicked()
                    {
                        self.plan_pages.select(self.plan_pages.index() - 1);
                    }
                    let label = ui.label(c::REPORT_PREVIEW_PAGE);
                    let mut page = self.plan_pages.index() + 1;
                    ui.add(egui::DragValue::new(&mut page).range(1..=self.plan_pages.count()))
                        .labelled_by(label.id);
                    self.plan_pages.select(page.saturating_sub(1));
                    ui.label(format!("/ {}", self.plan_pages.count()));
                    if crate::ui_components::secondary(
                        ui,
                        c::REPORT_PREVIEW_NEXT,
                        self.plan_pages.index() + 1 < self.plan_pages.count(),
                    )
                    .clicked()
                    {
                        self.plan_pages.select(self.plan_pages.index() + 1);
                    }
                });
                let label = ui.label(c::PLAN_PREVIEW_CONTENT);
                let preview_id =
                    ui.make_persistent_id(("execution-plan-content", self.plan_pages.index()));
                let scroll_requested = ui.input(|input| {
                    input.has_accesskit_action_request(
                        preview_id,
                        egui::accesskit::Action::ScrollIntoView,
                    )
                });
                let preview = egui::ScrollArea::vertical()
                    .id_salt(("execution-plan-preview", self.plan_pages.index()))
                    .min_scrolled_height(120.0)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let mut page = self.plan_pages.current(&self.plan_preview);
                        let response = ui
                            .add(
                                egui::TextEdit::multiline(&mut page)
                                    .interactive(false)
                                    .id(preview_id)
                                    .code_editor()
                                    .desired_rows(8),
                            )
                            .labelled_by(label.id);
                        ui.ctx().accesskit_node_builder(response.id, |node| {
                            node.set_read_only();
                            node.add_action(egui::accesskit::Action::ScrollIntoView);
                        });
                        if scroll_requested {
                            response.scroll_to_me(Some(egui::Align::Min));
                        }
                    });
                if scroll_requested {
                    // Readonly content advertises ScrollIntoView, not keyboard
                    // focus. Reveal its full bounded viewport in download-content.
                    ui.scroll_to_rect(preview.inner_rect, Some(egui::Align::Center));
                }
            }
            Err(error) => {
                self.plan_preview.clear();
                self.plan_pages.reset("");
                use crate::accessible_plan::PreviewError;
                ui.label(match error {
                    PreviewError::InvalidOptions => c::PLAN_PREVIEW_INVALID,
                    PreviewError::NonUnicodeArgument => c::PLAN_PREVIEW_NON_UNICODE,
                    PreviewError::TooLarge => c::PLAN_PREVIEW_TOO_LARGE,
                    PreviewError::Serialization => c::PLAN_PREVIEW_FAILED,
                });
            }
        }
    }
    fn show_navigation(&mut self, ui: &mut egui::Ui) {
        if navigation_button(
            ui,
            !self.settings && !self.library_open && !self.tools_open && !self.history_open,
            c::DOWNLOADS,
        )
        .clicked()
        {
            self.tool_return_pending = false;
            self.settings = false;
            self.library_open = false;
            self.tools_open = false;
            self.history_open = false;
        }
        if navigation_button(ui, self.library_open, c::LIBRARY).clicked() && !self.close_wait {
            self.settings = false;
            self.library_open = true;
            self.tools_open = false;
            self.history_open = false;
            if self.library.is_none() {
                let mut library = crate::accessible_library::LibraryPanel::new(
                    self.data.clone(),
                    self.output_path(),
                );
                library.poll(ui.ctx());
                self.library = Some(library);
            }
        }
        if navigation_button(ui, self.history_open, c::HISTORY).clicked() && !self.close_wait {
            self.history_open = true;
            self.tools_open = false;
            self.settings = false;
            self.library_open = false;
            if let Some(history) = &mut self.history {
                history.refresh();
            } else {
                self.history = Some(crate::accessible_history::HistoryPanel::new(
                    self.data.clone(),
                ));
            }
        }
        if navigation_button(ui, self.settings, c::SETTINGS).clicked() {
            self.settings = true;
            self.settings_section = SettingsSection::General;
            self.library_open = false;
            self.tools_open = false;
            self.history_open = false;
        }
    }
    fn open_tools(&mut self) {
        if self.close_wait || self.app_update_busy() {
            return;
        }
        self.tools_open = true;
        self.history_open = false;
        self.settings = true;
        self.settings_section = SettingsSection::Tools;
        self.library_open = false;
        if self.tools.is_none() {
            self.tools = Some(crate::accessible_tools::Panel::new(self.data.clone()));
        }
    }
    fn show_settings_navigation(&mut self, ui: &mut egui::Ui) {
        let update_busy = self.app_update_busy();
        ui.horizontal_wrapped(|ui| {
            for (section, label) in [
                (SettingsSection::General, c::SETTINGS_GENERAL),
                (SettingsSection::Tools, c::TOOLS),
                (SettingsSection::SaveLocation, c::SETTINGS_SAVE_LOCATION),
                (SettingsSection::AppUpdate, c::APP_UPDATE_TITLE),
            ] {
                let key = match section {
                    SettingsSection::General => "general",
                    SettingsSection::Tools => "tools",
                    SettingsSection::SaveLocation => "save-location",
                    SettingsSection::AppUpdate => "app-update",
                };
                let clicked = ui
                    .scope_builder(
                        egui::UiBuilder::new().id(("settings-navigation", key)),
                        |ui| {
                            ui.add_enabled_ui(
                                section != SettingsSection::Tools || !update_busy,
                                |ui| {
                                    crate::ui_components::navigation_choice(
                                        ui,
                                        self.settings_section == section,
                                        label,
                                    )
                                },
                            )
                            .inner
                        },
                    )
                    .inner
                    .clicked();
                if clicked {
                    if section == SettingsSection::Tools {
                        self.open_tools();
                    } else {
                        self.settings_section = section;
                        self.tools_open = false;
                    }
                }
            }
        });
        ui.add_space(8.0);
    }
    fn capture_location_request(&mut self) {
        if let Some(serial) = self
            .library
            .as_mut()
            .and_then(|panel| panel.take_location_request_started())
        {
            self.location_output_intent =
                (self.output.len() <= 32768).then(|| LocationOutputIntent {
                    serial,
                    text: self.output.clone(),
                    original: self.output_original.clone(),
                });
        }
    }
    fn apply_location_output_ack(&mut self) {
        let Some((serial, root)) = self
            .library
            .as_mut()
            .and_then(|panel| panel.take_applied_root())
        else {
            return;
        };
        self.accept_location_output_ack(serial, root);
    }
    fn accept_location_output_ack(&mut self, serial: u64, root: PathBuf) {
        if self
            .location_output_intent
            .as_ref()
            .is_some_and(|intent| intent.serial == serial)
        {
            let intent = self.location_output_intent.take().expect("matched intent");
            if !self.close_wait
                && self.output == intent.text
                && self.output_original == intent.original
            {
                self.set_output_path(root);
            }
        }
    }
    fn show_settings_save_location(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.strong(c::SETTINGS_DEFAULT_SAVE);
        ui.weak(c::SETTINGS_DEFAULT_SAVE_HELP);
        let enabled = self.ready
            && !self.close_wait
            && !self.app_update_busy()
            && self.worker.is_none()
            && !self.local_export_blocks()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && self.tools.as_ref().is_none_or(|panel| !panel.busy())
            && self.history.as_ref().is_none_or(|panel| !panel.busy())
            && self.diagnostics.as_ref().is_none_or(|panel| !panel.busy());
        if self.library.is_none() && enabled {
            self.library = Some(crate::accessible_library::LibraryPanel::new(
                self.data.clone(),
                self.output_path(),
            ));
        }
        let picker_available = self.path_picker.available();
        if let Some(library) = &mut self.library {
            library.show_location_settings(ui, enabled, picker_available);
        }
        self.capture_location_request();
        if let Some(initial) = self
            .library
            .as_mut()
            .and_then(|panel| panel.take_browse_request())
            && self
                .path_picker
                .start(crate::path_picker::Target::Library, initial, ctx)
                .is_err()
        {
            self.status = c::FOLDER_PICKER_FAILED.into();
        }
        ui.add_space(crate::ui_tokens::SECTION_GAP);
        ui.push_id("settings-download-draft-location", |ui| {
            collapsing(ui, c::SETTINGS_DRAFT_SAVE, |ui| {
                self.show_download_location_draft(ui, ctx)
            });
        });
    }
    fn show_download_location_draft(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label(c::SETTINGS_SAVE_LOCATION_HELP);
        let enabled = self.ready
            && !self.close_wait
            && !self.app_update_busy()
            && self.worker.is_none()
            && !self.local_export_blocks()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && self.tools.as_ref().is_none_or(|p| !p.busy())
            && self.library.as_ref().is_none_or(|p| !p.busy())
            && self.history.as_ref().is_none_or(|p| !p.busy())
            && self.diagnostics.as_ref().is_none_or(|p| !p.busy());
        if crate::path_picker::folder_field_with_id(
            ui,
            &mut self.output,
            enabled,
            self.path_picker.available(),
            egui::Id::new("download-save-path-input"),
        ) && enabled
            && !ctx.input(|input| input.viewport().close_requested())
            && self
                .path_picker
                .start(
                    crate::path_picker::Target::Download,
                    self.output_path(),
                    ctx,
                )
                .is_err()
        {
            self.status = c::FOLDER_PICKER_FAILED.into();
        }
    }
    fn feature_tool_needs(&self) -> [Option<FeatureToolNeed>; 3] {
        use crate::accessible_tool_preferences::ManagedAvailability;
        use crate::managed::ToolId;
        let local = self.media.local() || self.media.has_local_intent();
        if self.media.fatal() || self.media.source_unsupported() {
            return [None; 3];
        }
        if !local && dlpad_core::options::SourceUrl::parse(self.url.trim()).is_err() {
            return [None; 3];
        }
        let range_needs_ffmpeg = self.form.range_requested();
        let required = [
            !local,
            local || (!self.skip_preview && self.media.selection().is_none()) || range_needs_ffmpeg,
            !local && !self.deno.is_empty(),
        ];
        let fields = [&self.tool, &self.ffmpeg, &self.deno];
        let tools = [ToolId::YtDlp, ToolId::Ffmpeg, ToolId::Deno];
        std::array::from_fn(|index| {
            if !required[index] {
                return None;
            }
            let text = fields[index];
            let path = Path::new(text);
            let issue = if !text.is_empty()
                && (!path.is_absolute()
                    || text.len() > 32768
                    || text.chars().any(char::is_control)
                    || path
                        .components()
                        .any(|part| matches!(part, std::path::Component::ParentDir)))
            {
                Some(FeatureToolIssue::InvalidPath)
            } else if text.is_empty() {
                match self.tool_preferences.recovery_availability(index) {
                    Some(ManagedAvailability::Unavailable(reason))
                        if *reason != c::TOOL_AUTO_UNAVAILABLE =>
                    {
                        Some(FeatureToolIssue::Rejected(reason))
                    }
                    _ => Some(FeatureToolIssue::Setup),
                }
            } else {
                None
            };
            issue.map(|issue| FeatureToolNeed {
                tool: tools[index],
                issue,
            })
        })
    }
    fn tool_recovery_enabled(&self) -> bool {
        !self.close_wait
            && !self.app_update_busy()
            && !self.local_export_blocks()
            && self.worker.is_none()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && !self.media.busy()
            && !self.tool_preferences.busy()
            && self.tools.as_ref().is_none_or(|p| !p.busy())
            && self.history.as_ref().is_none_or(|p| !p.busy())
            && self.library.as_ref().is_none_or(|p| !p.busy())
            && self.diagnostics.as_ref().is_none_or(|p| !p.busy())
    }
    fn open_feature_tools(&mut self, tool: crate::managed::ToolId) {
        if !self.tool_recovery_enabled() {
            return;
        }
        self.open_tools();
        if let Some(panel) = &mut self.tools {
            panel.select_tool(tool);
        }
        self.tool_return_pending = true;
    }
    fn return_from_feature_tools(&mut self, retry_preview: bool) {
        if self.close_wait {
            return;
        }
        // A return is navigation; retry is a separate, explicitly captioned user action.
        let retry = retry_preview
            && self.tool_recovery_enabled()
            && self.feature_tool_needs().iter().all(Option::is_none)
            && self.media.can_retry_after_tool_setup();
        self.settings = false;
        self.tools_open = false;
        self.library_open = false;
        self.history_open = false;
        self.tool_return_pending = false;
        if retry {
            self.media.retry_after_tool_setup();
        }
    }
    fn show_feature_tool_return(&mut self, ui: &mut egui::Ui) {
        if !self.tool_return_pending {
            return;
        }
        let needs = self.feature_tool_needs();
        let retry = needs.iter().all(Option::is_none)
            && self.tool_recovery_enabled()
            && self.media.can_retry_after_tool_setup();
        let mut choose = None;
        let mut go_back = false;
        let shown = ui.horizontal_wrapped(|ui| {
            ui.label(c::FEATURE_TOOL_DRAFT_PRESERVED);
            if crate::ui_components::secondary(
                ui,
                if retry {
                    c::FEATURE_TOOL_RETURN_RETRY
                } else {
                    c::FEATURE_TOOL_RETURN
                },
                !self.close_wait,
            )
            .clicked()
            {
                go_back = true;
            }
            for need in needs.into_iter().flatten() {
                if self
                    .tools
                    .as_ref()
                    .is_some_and(|panel| panel.selected_tool() == need.tool)
                {
                    continue;
                }
                let caption = match need.tool {
                    crate::managed::ToolId::YtDlp => c::FEATURE_TOOL_NEXT_YTDLP,
                    crate::managed::ToolId::Ffmpeg => c::FEATURE_TOOL_NEXT_FFMPEG,
                    crate::managed::ToolId::Deno => c::FEATURE_TOOL_NEXT_DENO,
                };
                if crate::ui_components::secondary(ui, caption, !self.close_wait).clicked() {
                    choose = Some(need.tool);
                }
            }
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new("feature-tool-return-geometry"),
                (shown.response.rect, ui.clip_rect()),
            )
        });
        #[cfg(not(test))]
        let _ = shown;
        if let Some(tool) = choose
            && let Some(panel) = &mut self.tools
        {
            panel.select_tool(tool);
        }
        if go_back {
            self.return_from_feature_tools(retry);
        }
    }
    fn show_tool_recovery(
        &mut self,
        ui: &mut egui::Ui,
        reason: Option<&'static str>,
    ) -> Option<egui::Rect> {
        use crate::managed::ToolId;
        let needed = self.feature_tool_needs();
        let has_needs = needed.iter().any(Option::is_some);
        let execution_failed = self.media.tool_execution_failed();
        let reason = reason
            .filter(|reason| self.media.tool_error() != Some(*reason))
            .filter(|reason| {
                !has_needs
                    || !matches!(
                        *reason,
                        c::MEDIA_TOOLS
                            | c::MEDIA_TOOL_REQUIRED
                            | c::MEDIA_PREVIEW_FFMPEG_REQUIRED
                            | c::LOCAL_EXPORT_TOOL_REQUIRED
                    )
            })
            .filter(|reason| !(self.media.has_local_intent() && *reason == c::MEDIA_URL_REQUIRED));
        if reason.is_none() && !has_needs && !execution_failed {
            return None;
        }
        let shown = ui.horizontal_wrapped(|ui| {
            if let Some(reason) = reason {
                ui.label(reason);
            }
            if has_needs {
                let ytdlp = needed
                    .iter()
                    .flatten()
                    .any(|need| need.tool == ToolId::YtDlp);
                let ffmpeg = needed
                    .iter()
                    .flatten()
                    .any(|need| need.tool == ToolId::Ffmpeg);
                ui.label(match (ytdlp, ffmpeg) {
                    (true, true) => c::FEATURE_TOOL_REMOTE_REQUIRED,
                    (true, false) => c::FEATURE_TOOL_YTDLP_REQUIRED,
                    (false, true) => c::FEATURE_TOOL_FFMPEG_REQUIRED,
                    _ => c::FEATURE_TOOL_DENO_PATH,
                });
                let first = needed
                    .iter()
                    .flatten()
                    .next()
                    .copied()
                    .expect("nonempty bounded needs");
                for need in needed.iter().flatten() {
                    match need.issue {
                        FeatureToolIssue::InvalidPath => {
                            ui.label(need.tool.label());
                            ui.label(c::FEATURE_TOOL_INVALID_PATH);
                        }
                        FeatureToolIssue::Rejected(reason) => {
                            ui.label(need.tool.label());
                            ui.label(reason);
                        }
                        FeatureToolIssue::Setup => (),
                    }
                }
                let repair = needed
                    .iter()
                    .flatten()
                    .any(|need| !matches!(need.issue, FeatureToolIssue::Setup));
                let caption = if repair {
                    c::FEATURE_TOOL_CONFIGURE
                } else {
                    match (ytdlp, ffmpeg) {
                        (true, true) => c::MEDIA_OPEN_TOOLS,
                        (true, false) => c::MEDIA_OPEN_YTDLP,
                        (false, true) => c::MEDIA_OPEN_FFMPEG,
                        _ => c::FEATURE_TOOL_CONFIGURE,
                    }
                };
                if crate::ui_components::secondary_sized(
                    ui,
                    caption,
                    self.tool_recovery_enabled(),
                    egui::vec2(144.0, 32.0),
                )
                .clicked()
                {
                    self.open_feature_tools(first.tool);
                }
            } else if execution_failed {
                ui.label(c::FEATURE_TOOL_EXECUTION_FAILED);
                if crate::ui_components::secondary_sized(
                    ui,
                    c::FEATURE_TOOL_CONFIGURE,
                    self.tool_recovery_enabled(),
                    egui::vec2(144.0, 32.0),
                )
                .clicked()
                {
                    self.open_feature_tools(
                        if self.media.local() || self.media.has_local_intent() {
                            ToolId::Ffmpeg
                        } else {
                            ToolId::YtDlp
                        },
                    );
                }
            }
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new("feature-tool-recovery-geometry"),
                (shown.response.rect, ui.clip_rect()),
            )
        });
        Some(shown.response.rect)
    }
    fn source_prepare_disabled_reason(&self) -> Option<&'static str> {
        if self.url.trim().is_empty() {
            return Some(c::MEDIA_URL_REQUIRED);
        }
        if dlpad_core::options::SourceUrl::parse(self.url.trim()).is_err() {
            return Some(c::MEDIA_URL_INVALID);
        }
        if self.media.fatal() {
            return Some(c::MEDIA_CLEANUP_FAILED);
        }
        if self.media.busy() || self.media.closing() {
            return Some(c::MEDIA_WAIT);
        }
        None
    }
    fn download_disabled_reason(&self) -> Option<&'static str> {
        if self.app_update_busy() {
            return Some(c::APP_UPDATE_PREPARING);
        }
        if !self.ready {
            return Some(if self.worker.is_some() {
                c::RECOVERY_CHECKING
            } else {
                c::RECOVERY_FAILED
            });
        }
        if self.tool_preferences.busy() {
            return Some(c::TOOL_PREF_LOADING);
        }
        if self.media.fatal() {
            return Some(c::MEDIA_CLEANUP_FAILED);
        }
        if self.media.local() {
            return Some(c::MEDIA_LOCAL_HINT);
        }
        if self.url.trim().is_empty() {
            return Some(c::MEDIA_URL_REQUIRED);
        }
        if dlpad_core::options::SourceUrl::parse(self.url.trim()).is_err() {
            return Some(c::MEDIA_URL_INVALID);
        }
        if self.media.busy() {
            return Some(if self.media.selection().is_some() {
                c::MEDIA_FRAME_WAIT
            } else {
                c::MEDIA_WAIT
            });
        }
        if self.output.is_empty() {
            return Some(c::MEDIA_LOCATION_REQUIRED);
        }
        if !Path::new(&self.tool).is_absolute() {
            return Some(c::MEDIA_TOOL_REQUIRED);
        }
        if self.media.selection().is_none() && !self.skip_preview {
            return Some(c::MEDIA_PREVIEW_REQUIRED);
        }
        if let Some(s) = self.media.selection()
            && self.form.media_range(s.duration_ms()).is_err()
        {
            return Some(c::MEDIA_RANGE_INVALID);
        }
        let ffmpeg = (!self.ffmpeg.is_empty()).then(|| Path::new(&self.ffmpeg));
        self.form
            .build(self.url.trim(), &self.output_path(), ffmpeg)
            .err()
            .map(|e| e.0)
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[allow(dead_code)] // The alternate eframe binary shares Candidate but owns its window host.
    pub(crate) fn set_native_drop_registration(
        &mut self,
        registration: winit::platform::windows::FileDropRegistration,
    ) {
        self.native_drop_fallback = matches!(
            registration,
            winit::platform::windows::FileDropRegistration::NativeFallback(_)
        );
    }

    fn show_media_preview(&mut self, ui: &mut egui::Ui) {
        if self.native_drop_fallback {
            ui.label(c::MEDIA_DROP_FALLBACK);
        }
        let can_open = self.ready
            && self.worker.is_none()
            && !self.close_wait
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && self.tools.as_ref().is_none_or(|p| !p.busy())
            && self.history.as_ref().is_none_or(|p| !p.busy());
        self.media.show_with_open(ui, can_open);
        if self.start_condition_reveal_frame.is_some()
            && self
                .download_disabled_reason()
                .is_some_and(|reason| self.media.tool_error() == Some(reason))
        {
            self.start_condition_error_target = self.media.error_target();
        }
        if let Some(selection) = self.media.take_range_edit() {
            self.form.apply_media_selection(selection);
        }
        // Keep numeric editing next to the visual range. Each field keeps its
        // existing global download-option-input identity across both layouts.
        self.show_media_range_inputs(ui);
    }
    fn show_media_range_inputs(&mut self, ui: &mut egui::Ui) {
        if let Some(selection) = self.media.selection() {
            self.form.show_media_range(ui, selection.duration_ms());
            if let Ok((start, end)) = self.form.media_range(selection.duration_ms()) {
                self.media.synchronize_range(start, end);
            }
        }
    }
    fn local_export_blocks(&self) -> bool {
        #[cfg(windows)]
        {
            self.local_export.blocks_mutations()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
    fn local_export_can_request_stop(&self) -> bool {
        #[cfg(windows)]
        {
            !self.local_export.uncertainty()
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
    fn local_export_close_ready(&self) -> bool {
        #[cfg(windows)]
        {
            self.local_export.close_ready()
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
    fn stop_local_export(&mut self) {
        #[cfg(windows)]
        self.local_export.request_stop();
    }
    fn local_export_disabled_reason(&self) -> Option<&'static str> {
        if self.app_update_busy() {
            return Some(c::APP_UPDATE_PREPARING);
        }
        if self.local_export_blocks()
            || self.close_wait
            || !self.ready
            || self.tool_preferences.busy()
            || self.managed_recovery_pending
        {
            return Some(c::LOCAL_EXPORT_BLOCKED);
        }
        if self.media.busy() {
            return Some(c::LOCAL_EXPORT_PREVIEW_BUSY);
        }
        if self.media.prepared_for_export().is_none() {
            return Some(c::LOCAL_EXPORT_LOCAL_REQUIRED);
        }
        if !self.output_path().is_absolute() {
            return Some(c::MEDIA_LOCATION_REQUIRED);
        }
        if !Path::new(&self.ffmpeg).is_absolute() {
            return Some(c::LOCAL_EXPORT_TOOL_REQUIRED);
        }
        let Some(selection) = self.media.selection() else {
            return Some(c::LOCAL_EXPORT_LOCAL_REQUIRED);
        };
        match self.form.media_range(selection.duration_ms()) {
            Err(_) => return Some(c::MEDIA_RANGE_INVALID),
            Ok((start, end)) if end - start > 10 * 60 * 1000 => {
                return Some(c::LOCAL_EXPORT_RANGE_LIMIT);
            }
            _ => {}
        }
        #[cfg(not(windows))]
        return Some(c::LOCAL_EXPORT_UNAVAILABLE);
        #[cfg(windows)]
        None
    }
    #[cfg(windows)]
    fn local_export_tools_snapshot(&self) -> Option<dlpad_engine::media_preview::Tools> {
        let ffmpeg = PathBuf::from(&self.ffmpeg);
        if !ffmpeg.is_absolute() {
            return None;
        }
        let ffprobe = ffmpeg.parent()?.join("ffprobe.exe");
        Some(dlpad_engine::media_preview::Tools {
            ffmpeg,
            ffprobe,
            ytdlp: PathBuf::new(),
            deno: None,
        })
    }
    #[cfg(windows)]
    fn start_local_export(&mut self, ctx: &egui::Context) {
        if self.local_export_disabled_reason().is_some()
            || self.worker.is_some()
            || (self.path_picker.busy() || self.tool_path_picker.busy())
            || self.tools.as_ref().is_some_and(|p| p.busy())
            || self.history.as_ref().is_some_and(|p| p.busy())
            || self.library.as_ref().is_some_and(|p| p.busy())
            || self.diagnostics.as_ref().is_some_and(|p| p.busy())
        {
            return;
        }
        let Some(prepared) = self.media.prepared_for_export() else {
            return;
        };
        let Some(selection) = self.media.selection() else {
            return;
        };
        let Ok((start_ms, end_ms)) = self.form.media_range(selection.duration_ms()) else {
            return;
        };
        // Root reviews this pure snapshot factory; actual verification is worker-owned.
        let Some(tools) = self.local_export_tools_snapshot() else {
            self.status = c::LOCAL_EXPORT_TOOL_REQUIRED.into();
            return;
        };
        let stamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(stamp) => stamp.as_nanos(),
            Err(_) => {
                self.status = c::LOCAL_EXPORT_START_FAILED.into();
                return;
            }
        };
        let request = dlpad_engine::media_preview::local_export::ExportRequest {
            prepared,
            start_ms,
            end_ms,
            output_parent: self.output_path(),
            folder_name: format!("dlpad-range-{}-{stamp}", std::process::id()),
        };
        self.media.cancel(); // busy==false; pause playback without making Workspace permanently closing.
        if self
            .local_export
            .start(tools, self.data.join("managed-tools"), request, ctx)
            .is_err()
        {
            self.status = c::LOCAL_EXPORT_START_FAILED.into();
        }
    }
    fn show_media_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.media.local() {
            ui.strong(c::MEDIA_INFO);
            ui.weak(c::MEDIA_LOCAL_HINT);
            if let Some(selection) = self.media.selection() {
                ui.label(format!(
                    "{} {}",
                    c::MEDIA_SELECTED_DURATION,
                    crate::media_timeline::time(selection.end_ms() - selection.start_ms())
                ));
            }
        }
        let show_preview_bypass = !self.media.local()
            && self.media.selection().is_none()
            && dlpad_core::options::SourceUrl::parse(self.url.trim()).is_ok();
        if show_preview_bypass {
            ui.push_id("media-preview-bypass", |ui| {
                crate::ui_components::checkbox(ui, &mut self.skip_preview, c::MEDIA_SKIP_PREVIEW);
            });
        }
        if self.media.local() || self.media.selection().is_some() || show_preview_bypass {
            ui.add_space(8.0);
        }
        ui.strong(c::MEDIA_SAVE_SETTINGS);
        if crate::path_picker::folder_field_with_id(
            ui,
            &mut self.output,
            true,
            self.path_picker.available(),
            egui::Id::new("download-save-path-input"),
        ) && self
            .path_picker
            .start(
                crate::path_picker::Target::Download,
                self.output_path(),
                ctx,
            )
            .is_err()
        {
            self.status = c::FOLDER_PICKER_FAILED.into();
        }
        if self.media.local() {
            ui.weak(c::LOCAL_EXPORT_RETAINED_STAGE_EXPLANATION);
            return;
        }
        self.form
            .set_media_prepared(self.media.selection().is_some());
        self.form.show(ui, true);
        if self.tool.is_empty() {
            ui.weak(c::TOOL_SETUP_HINT);
            let enabled = !self.close_wait
                && !self.local_export_blocks()
                && self.worker.is_none()
                && !self.path_picker.busy()
                && !self.tool_path_picker.busy()
                && !self.tool_preferences.busy()
                && self.tools.as_ref().is_none_or(|p| !p.busy());
            if ui
                .scope_builder(
                    egui::UiBuilder::new().id("download-tool-setup-open-tools"),
                    |ui| crate::ui_components::secondary(ui, c::TOOLS, enabled),
                )
                .inner
                .clicked()
            {
                self.open_tools();
            }
        }
    }
    pub(crate) fn show_ui(&mut self, ctx: &egui::Context) {
        // Reserve first, before any panel can leave a transient unpainted region.
        // Resolve color after drawing: appearance actions may update the theme.
        let backdrop = egui::Painter::new(
            ctx.clone(),
            egui::LayerId::background(),
            ctx.viewport_rect(),
        );
        let slot = backdrop.add(egui::Shape::Noop);
        self.show_ui_contents(ctx);
        backdrop.set(
            slot,
            egui::Shape::rect_filled(ctx.viewport_rect(), 0.0, ctx.style().visuals.panel_fill),
        );
    }
    fn show_ui_contents(&mut self, ctx: &egui::Context) {
        if let Some(client) = crate::app_update_client::handle() {
            client.bind_context(ctx);
            if let Some(snapshot) = client.snapshot() {
                self.observe_update_recovery(&snapshot);
            }
        }
        let update_busy = self.app_update_busy();
        if self.close_wait {
            self.stop_app_update();
            self.tool_preferences.request_stop();
        }
        self.tool_preferences
            .poll(ctx, [&mut self.tool, &mut self.ffmpeg, &mut self.deno]);
        #[cfg(windows)]
        self.local_export.poll(ctx);
        if !self.local_export_blocks() && self.media.source_changed(&self.url) {
            self.tool_return_pending = false;
            self.skip_preview = false;
        }
        if !self.close_wait && !update_busy && self.worker.is_none() && !self.local_export_blocks()
        {
            let dropped = ctx.input(|input| input.raw.dropped_files.clone());
            if dropped.len() == 1 {
                if let Some(path) = &dropped[0].path {
                    self.url.clear();
                    self.media.source_changed("");
                    self.media.dropped(path.clone());
                    self.tool_return_pending = false;
                    self.settings = false;
                    self.library_open = false;
                    self.tools_open = false;
                    self.history_open = false;
                }
            } else if dropped.len() > 1 {
                self.status = c::MEDIA_DROP_ONE.into();
            }
        }
        if self.close_wait {
            self.media.request_stop();
        }
        if self.hidden || update_busy || self.worker.is_some() || self.local_export_blocks() {
            self.media.cancel();
        }
        if self.hidden || update_busy || self.worker.is_some() || self.local_export_blocks() {
            self.media.drain();
        } else {
            self.media.poll(
                ctx,
                &self.url,
                &self.tool,
                &self.ffmpeg,
                &self.deno,
                &self.data.join("managed-tools"),
            );
        }
        if self.media.fatal() {
            self.status = c::MEDIA_CLEANUP_FAILED.into();
        }
        self.poll_folder_picker(ctx);
        if self
            .activation
            .as_ref()
            .is_some_and(|server| server.poll_restore())
        {
            if self.hidden() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
            self.hidden = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        self.poll();
        self.appearance.poll(ctx);
        let environment = crate::accessible_app_update::UpdateEnvironment {
            startup_ready: self.ready,
            task_idle: self.update_tasks_idle(),
            closing: self.close_wait,
        };
        let restart_accepted = if let Some(panel) = &mut self.app_update_panel {
            panel.set_environment(environment);
            panel.poll(ctx);
            panel.take_restart_requested()
        } else {
            false
        };
        if restart_accepted {
            self.begin_update_close_wait();
        }
        self.metrics.set_root(self.output_path());
        self.metrics.poll(ctx);
        let tickets = std::array::from_fn(|index| self.tool_preferences.managed_auto_ticket(index));
        let tool_results = self.tools.as_mut().map(|tools| {
            tools.set_automatic_tickets(tickets);
            tools.poll();
            (
                tools.take_inspection(),
                tools.take_selected_path(),
                tools.take_auto_selected_path(),
            )
        });
        if let Some((inspection, selected, automatic)) = tool_results {
            if let Some(inspection) = inspection {
                self.apply_managed_inspection(inspection);
            }
            if !self.close_wait
                && !update_busy
                && self.worker.is_none()
                && !self.local_export_blocks()
            {
                if let Some((tool, path)) = selected {
                    self.apply_managed_tool(tool, &path);
                }
                if let Some((tool, path, ticket)) = automatic
                    && let Some(path) = path.to_str()
                {
                    self.tool_preferences.managed_auto_selected(
                        tool_preference_index(tool),
                        path,
                        ticket,
                        [&mut self.tool, &mut self.ffmpeg, &mut self.deno],
                    );
                }
            }
        }
        if let Some(history) = &mut self.history {
            history.poll();
            if let Some(id) = history.take_resume()
                && !self.close_wait
                && self.ready
                && !update_busy
                && self.worker.is_none()
                && !self.local_export_blocks()
            {
                self.resume = Some(id);
                self.history_open = false;
                self.tools_open = false;
                self.settings = false;
                self.library_open = false;
                self.start(true);
            }
        }
        if self.close_wait {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if let Some(report) = self.report.take() {
            let report = if self.report_stress {
                Box::new(crate::accessible_report_fixture::stress_report())
            } else {
                report
            };
            let panel = crate::accessible_diagnostics::DiagnosticPanel::new(report);
            #[cfg(all(windows, feature = "cpu"))]
            let panel = {
                let mut panel = panel;
                if let Some(owner) = self.path_picker.owner() {
                    panel.set_picker_owner(owner);
                }
                panel
            };
            self.diagnostics = Some(panel);
        }
        if let Some(diagnostics) = &mut self.diagnostics {
            diagnostics.poll(ctx);
        }
        if let Some(library) = &mut self.library {
            library.poll(ctx);
            if let Some(notice) = library.take_export_notice() {
                self.status = notice.into();
            }
        }
        self.capture_location_request();
        self.apply_location_output_ack();
        if ctx.input(|i| i.viewport().close_requested())
            && (!self.tool_preferences.stopped()
                || !self.appearance.stopped()
                || !self.app_update_stopped()
                || !self.metrics.stopped()
                || self.worker.is_some()
                || self.tools.is_some()
                || self.history.is_some()
                || self.library.is_some()
                || self.diagnostics.is_some()
                || !self.media.stopped()
                || !self.local_export_close_ready())
        {
            self.path_picker.cancel();
            self.tool_path_picker.cancel();
            self.media.cancel();
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.worker.is_some()
                || self.app_update_busy()
                || self.tools.as_ref().is_some_and(|p| p.busy())
                || self.history.as_ref().is_some_and(|p| p.busy())
                || self.tool_preferences.busy()
                || self.appearance.busy()
                || self.library.as_ref().is_some_and(|p| p.busy())
                || self.diagnostics.as_ref().is_some_and(|p| p.busy())
                || self.local_export_blocks()
            {
                self.close_prompt = true;
            } else {
                self.close_wait = true;
                self.stop_app_update();
                self.stop_local_export();
                self.appearance.request_stop();
                if let Some(panel) = &mut self.app_update_panel {
                    panel.request_stop();
                }
                self.tool_preferences.request_stop();
                self.metrics.request_stop();
                if let Some(tools) = &mut self.tools {
                    tools.request_stop();
                }
                if let Some(history) = &mut self.history {
                    history.request_stop();
                }
                if let Some(diagnostics) = &mut self.diagnostics {
                    diagnostics.request_stop();
                }
                if let Some(library) = &mut self.library {
                    library.request_stop();
                }
            }
        }
        if self.close_wait
            && self.local_export_close_ready()
            && self.media.stopped()
            && !self.path_picker.busy()
            && !self.tool_path_picker.busy()
            && self.tool_preferences.stopped()
            && self.appearance.stopped()
            && self.app_update_stopped()
            && self.metrics.stopped()
            && self.tools.as_ref().is_none_or(|p| p.stopped())
            && self.history.as_ref().is_none_or(|p| p.stopped())
            && self.worker.is_none()
            && self.library.as_ref().is_none_or(|p| p.stopped())
            && self.diagnostics.as_ref().is_none_or(|p| p.stopped())
        {
            #[cfg(windows)]
            crate::local_export_worker::export_trace(
                6,
                self.local_export_close_ready(),
                self.close_wait,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.worker.is_some() {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        egui::TopBottomPanel::top("navigation")
            .frame(
                egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(
                    crate::ui_tokens::PAGE_MARGIN as i8,
                    8,
                )),
            )
            .show(ctx, |ui| {
                if (self.path_picker.busy() || self.tool_path_picker.busy())
                    || self.local_export_blocks()
                {
                    ui.disable();
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    self.show_navigation(ui);
                });
            });
        if (!self.ready || self.tool_preferences.busy()) && !self.close_wait {
            let checking = self.worker.is_some() || (self.ready && self.tool_preferences.busy());
            let style = ctx.style();
            let detail = if self.ready {
                c::TOOL_PREF_LOADING
            } else if checking {
                c::STARTUP_LOCKED_HELP
            } else {
                &self.status
            }
            .to_owned();
            let detail_width =
                (ctx.content_rect().width() - 2.0 * crate::ui_tokens::PAGE_MARGIN).max(1.0);
            let detail_height = ctx
                .fonts_mut(|fonts| {
                    fonts
                        .layout(
                            detail.clone(),
                            egui::TextStyle::Body.resolve(&style),
                            style.visuals.text_color(),
                            detail_width,
                        )
                        .size()
                        .y
                })
                .min(56.0);
            // Top panels otherwise clip new or newly wrapped content against the previous
            // frame's height. Reserve the measured text before allocating this frame.
            let panel_height = crate::ui_tokens::CONTROL_HEIGHT
                + style.spacing.item_spacing.y
                + detail_height
                + 16.0;
            egui::TopBottomPanel::top("startup-readiness")
                .exact_height(panel_height)
                .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(
                    egui::Margin::symmetric(crate::ui_tokens::PAGE_MARGIN as i8, 8),
                ))
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if checking {
                            ui.spinner();
                        }
                        ui.strong(if checking {
                            c::STARTUP_CHECKING_TITLE
                        } else {
                            c::STARTUP_FAILED_TITLE
                        });
                        if !checking
                            && crate::ui_components::secondary(
                                ui,
                                c::RECOVERY_RETRY,
                                self.can_recover(),
                            )
                            .clicked()
                        {
                            self.recover();
                        }
                    });
                    if checking {
                        ui.label(detail);
                    } else {
                        // Keep exceptional paths/errors readable without allowing an arbitrary
                        // recovery message to consume the whole small-window viewport.
                        egui::ScrollArea::vertical()
                            .id_salt("startup-failure-detail")
                            .max_height(56.0)
                            .show(ui, |ui| {
                                ui.label(&self.status);
                            });
                    }
                });
        }
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(
                    crate::ui_tokens::PAGE_MARGIN as i8,
                    6,
                )),
            )
            .show(ctx, |ui| {
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
                // The status row contains read-only text, not 32px controls.
                ui.spacing_mut().interact_size.y = ui.text_style_height(&egui::TextStyle::Body);
                ui.spacing_mut().item_spacing.y = 4.0;
                let export_status = self
                    .library
                    .as_ref()
                    .and_then(|library| library.background_export_status());
                ui.horizontal_wrapped(|ui| {
                    ui.label(if self.tool_path_picker.busy() {
                        c::TOOL_PICKER_BUSY
                    } else if self.path_picker.busy() {
                        c::FOLDER_PICKER_BUSY
                    } else if !self.close_wait
                        && let Some(status) = export_status.as_deref()
                    {
                        status
                    } else if !self.close_wait
                        && let Some(status) = self
                            .tools
                            .as_ref()
                            .and_then(|tools| tools.background_status())
                    {
                        status
                    } else {
                        &self.status
                    });
                    ui.separator();
                    self.metrics.status(ui);
                });
                if self.worker.is_some()
                    && let Some(sample) = self.progress.processing_snapshot().filter(|s| !s.ended)
                {
                    let speed = sample
                        .speed
                        .map(|v| format!("{v:.2}×"))
                        .unwrap_or_else(|| c::UNKNOWN.into());
                    let eta = sample
                        .eta_seconds
                        .map(|v| format!(" · {} {v:.0} {}", c::ETA, c::SECONDS))
                        .unwrap_or_default();
                    ui.label(format!("{} {}{}", c::PROCESSING_SPEED, speed, eta));
                }
                if let Some(sample) = self.progress.snapshot() {
                    let eta = sample
                        .eta_seconds
                        .map(|value| format!(" · {}: {value:.0} {}", c::ETA, c::SECONDS))
                        .unwrap_or_default();
                    ui.label(format!(
                        "{}: {}{}",
                        c::DOWNLOAD_SPEED,
                        crate::display_rate::format_bytes_per_second(sample.bytes_per_second),
                        eta
                    ));
                }
            });
        if self.settings
            || self.library_open
            || self.tools_open
            || self.history_open
            || self.media.local()
            || self.download_disabled_reason().is_none()
        {
            self.start_condition_reveal_frame = None;
        }
        if !self.settings && !self.library_open && !self.tools_open && !self.history_open {
            egui::TopBottomPanel::bottom("download-actions")
                .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(
                    egui::Margin::symmetric(crate::ui_tokens::PAGE_MARGIN as i8, 10),
                ))
                .show(ctx, |ui| {
                    if self.ready && self.worker.is_some() {
                        show_download_progress(
                            ui,
                            self.progress.snapshot().as_ref(),
                            self.progress
                                .processing_snapshot()
                                .is_some_and(|sample| !sample.ended),
                        );
                        ui.add_space(8.0);
                    }
                    let active = self.worker.is_some()
                        || update_busy
                        || self.tools.as_ref().is_some_and(|p| p.busy())
                        || self.history.as_ref().is_some_and(|p| p.busy())
                        || self.library.as_ref().is_some_and(|p| p.busy())
                        || self.diagnostics.as_ref().is_some_and(|p| p.busy())
                        || (self.path_picker.busy() || self.tool_path_picker.busy());
                    let actions_enabled =
                        !active && !self.close_wait && !self.local_export_blocks();
                    // Keep original control IDs while gating actions independently of explanations.
                    ui.add_enabled_ui(true, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let local = self.media.local();
                            let disabled = if local {
                                self.local_export_disabled_reason()
                            } else {
                                self.download_disabled_reason()
                            };
                            let label = if local {
                                c::LOCAL_EXPORT_SAVE_SELECTION
                            } else {
                                c::START
                            };
                            let mut start = crate::ui_components::primary(
                                ui,
                                label,
                                actions_enabled && disabled.is_none(),
                            );
                            if !local && let Some(reason) = disabled {
                                start = start.on_disabled_hover_text(reason);
                                start.ctx.accesskit_node_builder(start.id, |node| {
                                    node.set_description(reason)
                                });
                                let conditions =
                                    crate::ui_components::secondary(ui, c::START_CONDITIONS, true);
                                conditions
                                    .ctx
                                    .accesskit_node_builder(conditions.id, |node| {
                                        node.set_description(reason)
                                    });
                                if conditions.clicked() {
                                    self.start_condition_reveal_frame =
                                        Some(ctx.cumulative_frame_nr());
                                }
                            }
                            if start.clicked() {
                                if local {
                                    #[cfg(windows)]
                                    self.start_local_export(ctx);
                                } else {
                                    self.media.cancel();
                                    self.start(false);
                                }
                            }
                            if local && let Some(reason) = disabled {
                                ui.weak(reason);
                            }
                            if !local
                                && self.resume.is_some()
                                && crate::ui_components::secondary(
                                    ui,
                                    c::RESUME,
                                    actions_enabled
                                        && self.ready
                                        && !self.media.busy()
                                        && !self.media.fatal(),
                                )
                                .clicked()
                            {
                                self.start(true);
                            }
                        });
                    });
                    #[cfg(windows)]
                    self.local_export.show_status(ui);
                    if self.ready
                        && let Some(worker) = &self.worker
                    {
                        ui.horizontal(|ui| {
                            if crate::ui_components::ghost(
                                ui,
                                c::CANCEL,
                                !worker.cancel.load(Ordering::Acquire),
                            )
                            .clicked()
                            {
                                worker.request_stop(false);
                            }
                            if crate::ui_components::secondary(
                                ui,
                                c::PAUSE,
                                !worker.cancel.load(Ordering::Acquire),
                            )
                            .clicked()
                            {
                                worker.request_stop(true);
                            }
                        });
                    }
                });
        }
        self.show_update_recovery_notice(ctx);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(crate::ui_tokens::PAGE_MARGIN))
            .show(ctx, |ui| {
                if (self.path_picker.busy() || self.tool_path_picker.busy())
                    || self.local_export_blocks()
                {
                    ui.disable();
                }
                if !self.settings && (self.tools_open || self.library_open || self.history_open) {
                    ui.label(egui::RichText::new(if self.tools_open {
                        c::TOOLS
                    } else if self.library_open {
                        c::LIBRARY
                    } else {
                        c::HISTORY
                    }).text_style(egui::TextStyle::Name("screen-title".into())));
                    ui.add_space(12.0);
                }
                if self.settings {
                    self.show_settings_navigation(ui);
                }
                if self.settings && self.tool_return_pending {
                    self.show_feature_tool_return(ui);
                }
                if self.tools_open {
                    egui::ScrollArea::vertical()
                        .id_salt("tools-content")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.managed_recovery_pending {
                                ui.label(c::TOOL_EXECUTION_PENDING);
                                if crate::ui_components::secondary(ui, c::RECOVERY_RETRY, self.can_recover())
                                    .clicked()
                                {
                                    self.recover();
                                }
                            }
                            let tickets = std::array::from_fn(|index| self.tool_preferences.managed_auto_ticket(index));
                            let fields = [&self.tool, &self.ffmpeg, &self.deno];
                            let sources = std::array::from_fn(|index| {
                                if fields[index].is_empty() {
                                    crate::accessible_tools::CurrentSource::Unknown
                                } else if self.tool_preferences.displayed_managed_path_matches(index, fields[index]) {
                                    crate::accessible_tools::CurrentSource::Managed
                                } else {
                                    crate::accessible_tools::CurrentSource::External
                                }
                            });
                            if let Some(tools) = &mut self.tools {
                                tools.set_update_current_sources(sources);
                                tools.set_automatic_tickets(tickets);
                                tools.show(
                                    ui,
                                    self.ready
                                        && !update_busy
                                        && self.worker.is_none()
                                        && !self.close_wait
                                        && self.library.as_ref().is_none_or(|p| !p.busy())
                                        && self.history.as_ref().is_none_or(|p| !p.busy()),
                                );
                            }
                        });
                    return;
                }
                if self.history_open {
                    if let Some(history) = &mut self.history {
                        history.show(
                            ui,
                            self.worker.is_none()
                                && !update_busy
                                && !self.close_wait
                                && self.library.as_ref().is_none_or(|p| !p.busy())
                                && self.tools.as_ref().is_none_or(|p| !p.busy()),
                            self.ready,
                        );
                    }
                    return;
                }
                if self.library_open {
                    if let Some(library) = &mut self.library {
                        library.show(
                            ui,
                            self.ready
                                && !update_busy
                                && self.worker.is_none()
                                && !self.close_wait
                                && !self.path_picker.busy()
                                && !self.tool_path_picker.busy()
                                && self.tools.as_ref().is_none_or(|p| !p.busy())
                                && self.history.as_ref().is_none_or(|p| !p.busy()),
                            self.path_picker.available(),
                        );
                        if let Some(path) = library.take_preview_request() {
                            self.media.dropped(path);
                            self.library_open = false;
                            self.settings = false;
                            self.tools_open = false;
                            self.history_open = false;
                            self.skip_preview = false;
                        }
                        if let Some(initial) = library.take_browse_request()
                            && self
                                .path_picker
                                .start(crate::path_picker::Target::Library, initial, ctx)
                                .is_err()
                        {
                            self.status = c::FOLDER_PICKER_FAILED.into();
                        }
                        if let Some((generation, initial)) = library.take_export_browse_request()
                            && self.path_picker.start(crate::path_picker::Target::LibraryExport { generation }, initial, ctx).is_err()
                        {
                            self.status = c::FOLDER_PICKER_FAILED.into();
                        }
                    }
                    return;
                }
                if self.settings && self.settings_section == SettingsSection::AppUpdate {
                    let enabled = !self.close_wait;
                    let environment = crate::accessible_app_update::UpdateEnvironment {
                        startup_ready: self.ready,
                        task_idle: self.update_tasks_idle(),
                        closing: self.close_wait,
                    };
                    let panel = self.app_update_panel.get_or_insert_with(crate::accessible_app_update::AppUpdatePanel::new);
                    panel.set_environment(environment);
                    egui::ScrollArea::vertical()
                        .id_salt("settings-app-update-content")
                        .show(ui, |ui| panel.ui(ui, enabled));
                    return;
                }
                if self.settings && self.settings_section == SettingsSection::SaveLocation {
                    egui::ScrollArea::vertical()
                        .id_salt("settings-save-location-content")
                        .show(ui, |ui| self.show_settings_save_location(ui, ctx));
                    return;
                }
                if self.settings {
                    egui::ScrollArea::vertical()
                        .id_salt("settings-content")
                        .show(ui, |ui| {
                            self.appearance.show(ui, ctx);
                            ui.add_space(8.0);
                            ui.separator();
                            let network = collapsing(ui, c::NETWORK_OBSERVATION, |ui| {
                                self.metrics.show_settings_with_critical(ui, !self.close_wait)
                            });
                            self.network_reveal.update(&network);
                            collapsing(ui, c::INPUT_CHECK, |ui| {
                                let label = ui.label(c::IME);
                                let ime = crate::ui_components::text_input_with_id(
                                    ui,
                                    &mut self.ime,
                                    8192,
                                    f32::INFINITY,
                                    egui::Id::new("settings-ime-input"),
                                )
                                .labelled_by(label.id);
                                if ime.gained_focus() {
                                    ime.scroll_to_me(None);
                                }
                                ui.label(c::IME_HELP);
                            });
                        });
                    return;
                }
                let (trace_raw_before, trace_smooth_before) = if self.scroll_trace_remaining > 0 { ctx.input(|i| (i.raw_scroll_delta, i.smooth_scroll_delta)) } else { (egui::Vec2::ZERO, egui::Vec2::ZERO) };
                if self.scroll_trace_remaining > 0 && trace_raw_before != egui::Vec2::ZERO { self.scroll_trace_armed = true; }
                let scroll_trace_output = egui::ScrollArea::vertical()
                    .id_salt("download-content")
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| {
                        let mut start_condition_target = None;
                        self.start_condition_error_target = None;
                        if self.start_condition_reveal_frame.is_some_and(|frame| frame < ctx.cumulative_frame_nr())
                            && ctx.input(|input| input.raw_scroll_delta != egui::Vec2::ZERO)
                        {
                            self.start_condition_reveal_frame = None;
                        }
                        let active = self.worker.is_some()
                            || update_busy
                            || self.tools.as_ref().is_some_and(|p| p.busy())
                            || self.history.as_ref().is_some_and(|p| p.busy())
                            || self.library.as_ref().is_some_and(|p| p.busy())
                            || self.diagnostics.as_ref().is_some_and(|p| p.busy())
                            || (self.path_picker.busy() || self.tool_path_picker.busy());
                        ui.add_enabled_ui(
                            !active && !self.close_wait && !self.local_export_blocks(),
                            |ui| {
                                if self.media.local() {
                                    if crate::ui_components::secondary(ui, c::MEDIA_CHANGE_SOURCE, !self.media.fatal())
                                        .clicked()
                                    {
                                        self.media.clear_source();
                                        self.url.clear();
                                    }
                                    if !Path::new(&self.ffmpeg).is_absolute() {
                                        self.show_tool_recovery(ui, Some(c::LOCAL_EXPORT_TOOL_REQUIRED));
                                    }
                                } else {
                                    let label = ui.weak(c::URL);
                                    let mut source_input_target = label.rect;
                                    ui.horizontal(|ui| {
                                        let reserve = 112.0 + ui.spacing().item_spacing.x;
                                        let response = crate::ui_components::text_input_with_id(
                                        ui,
                                        &mut self.url,
                                        8192,
                                        (ui.available_width()
                                            - reserve
                                            - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                                            .max(80.0),
                                        egui::Id::new("download-source-input"),
                                    )
                                    .labelled_by(label.id);
                                        source_input_target = source_input_target.union(response.rect);
                                        let disabled = self.source_prepare_disabled_reason();
                                        let mut prepare = crate::ui_components::secondary_sized(ui, c::MEDIA_PREPARE, disabled.is_none(), egui::vec2(112.0, 32.0));
                                        if let Some(reason) = disabled {
                                            prepare = prepare.on_disabled_hover_text(reason);
                                            prepare.ctx.accesskit_node_builder(prepare.id, |node| {
                                                node.set_description(reason);
                                            });
                                        } else {
                                            prepare.ctx.accesskit_node_builder(prepare.id, |node| {
                                                node.clear_description();
                                            });
                                        }
                                        if disabled.is_none()
                                            && (prepare.clicked()
                                                || (response.lost_focus()
                                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                                        {
                                            self.media.source_changed(&self.url);
                                            self.media.prepare_url(&self.url);
                                        }
                                    });
                                    #[cfg(test)]
                                    ctx.data_mut(|data| data.insert_temp(egui::Id::new("start-reason-source-input"), (source_input_target, ui.clip_rect())));
                                    let reason_target = self.show_tool_recovery(ui, self.download_disabled_reason());
                                    if self.start_condition_reveal_frame.is_some()
                                        && let Some(target) = reason_target
                                    {
                                        start_condition_target = Some(source_input_target.union(target));
                                    }
                                    ui.label(egui::RichText::new(c::MEDIA_SOURCE_HINT).small().weak());
                                }
                                ui.add_space(12.0);
                                let media_workspace = self.media.selection().is_some()
                                    || self.media.busy()
                                    || self.media.closing();
                                if ui.available_width() >= 760.0 && media_workspace {
                                    // Leave room for the save path and browse action on one row.
                                    let right = 400.0;
                                    let left = ui.available_width() - right - 24.0;
                                    ui.horizontal_top(|ui| {
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(left, 0.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| self.show_media_preview(ui),
                                        );
                                        ui.add_space(8.0);
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(right, 0.0),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                egui::Frame::new()
                                                    .fill(if ui.visuals().dark_mode {
                                                        ui.visuals().window_fill
                                                    } else {
                                                        ui.visuals().faint_bg_color
                                                    })
                                                    .corner_radius(6)
                                                    .inner_margin(16)
                                                    .show(ui, |ui| {
                                                        ui.set_width(right - 32.0);
                                                        self.show_media_settings(ui, ctx);
                                                    });
                                            },
                                        );
                                    });
                                } else {
                                    self.show_media_preview(ui);
                                    ui.add(egui::Separator::default().spacing(0.0));
                                    self.show_media_settings(ui, ctx);
                                }
                                ui.add_space(12.0);
                                collapsing(ui, c::DOWNLOAD_ENVIRONMENT, |ui| {
                                    let (edited, browse) = executable_field(
                                        ui,
                                        c::TOOL_PATH,
                                        &mut self.tool,
                                        self.tool_path_picker.available()
                                            && !self.tool_path_picker.busy(),
                                    );
                                    if edited {
                                        self.tool_preferences.edited(0);
                                    }
                                    if browse
                                        && !ctx.input(|input| input.viewport().close_requested())
                                        && self
                                            .tool_path_picker
                                            .start(
                                                crate::tool_path_picker::Target::YtDlp,
                                                PathBuf::from(&self.tool),
                                                ctx,
                                            )
                                            .is_err()
                                    {
                                        self.status = c::TOOL_PICKER_FAILED.into();
                                    }
                                    if self.tool_path_picker.busy() {
                                        ui.disable();
                                    }
                                    self.tool_preferences.show_mode_with_field(
                                        0,
                                        ui,
                                        !active
                                            && !self.close_wait
                                            && !self.path_picker.busy()
                                            && !self.tool_path_picker.busy(),
                                        &mut self.tool,
                                    );
                                    ui.add_space(8.0);
                                    let (edited, browse) = executable_field(
                                        ui,
                                        c::FFMPEG_PATH,
                                        &mut self.ffmpeg,
                                        self.tool_path_picker.available()
                                            && !self.tool_path_picker.busy(),
                                    );
                                    if edited {
                                        self.tool_preferences.edited(1);
                                    }
                                    if browse
                                        && !ctx.input(|input| input.viewport().close_requested())
                                        && self
                                            .tool_path_picker
                                            .start(
                                                crate::tool_path_picker::Target::Ffmpeg,
                                                PathBuf::from(&self.ffmpeg),
                                                ctx,
                                            )
                                            .is_err()
                                    {
                                        self.status = c::TOOL_PICKER_FAILED.into();
                                    }
                                    if self.tool_path_picker.busy() {
                                        ui.disable();
                                    }
                                    self.tool_preferences.show_mode_with_field(
                                        1,
                                        ui,
                                        !active
                                            && !self.close_wait
                                            && !self.path_picker.busy()
                                            && !self.tool_path_picker.busy(),
                                        &mut self.ffmpeg,
                                    );
                                    ui.add_space(8.0);
                                    let (edited, browse) = executable_field(
                                        ui,
                                        c::DENO_PATH,
                                        &mut self.deno,
                                        self.tool_path_picker.available()
                                            && !self.tool_path_picker.busy(),
                                    );
                                    if edited {
                                        self.tool_preferences.edited(2);
                                    }
                                    if browse
                                        && !ctx.input(|input| input.viewport().close_requested())
                                        && self
                                            .tool_path_picker
                                            .start(
                                                crate::tool_path_picker::Target::Deno,
                                                PathBuf::from(&self.deno),
                                                ctx,
                                            )
                                            .is_err()
                                    {
                                        self.status = c::TOOL_PICKER_FAILED.into();
                                    }
                                    if self.tool_path_picker.busy() {
                                        ui.disable();
                                    }
                                    self.tool_preferences.show_mode_with_field(
                                        2,
                                        ui,
                                        !active
                                            && !self.close_wait
                                            && !self.path_picker.busy()
                                            && !self.tool_path_picker.busy(),
                                        &mut self.deno,
                                    );
                                    self.tool_preferences.show_save_status(
                                        ui,
                                        !active
                                            && !self.close_wait
                                            && !self.path_picker.busy()
                                            && !self.tool_path_picker.busy(),
                                        [&mut self.tool, &mut self.ffmpeg, &mut self.deno],
                                    );
                                    field(ui, c::COOKIE_SOURCE, &mut self.cookie_source, 32768);
                                    ui.label(c::COOKIE_SESSION_HELP);
                                    collapsing(ui, c::COOKIE_HELP_TITLE, |ui| {
                                        ui.label(c::COOKIE_DEFAULT_EXAMPLE);
                                        ui.label(c::COOKIE_CONFLICT_VERSION);
                                    });
                                    if self.cookie_recovery_pending {
                                        ui.label(c::COOKIE_RECOVERY_PENDING);
                                    }
                                    if self.managed_recovery_pending {
                                        ui.label(c::TOOL_EXECUTION_PENDING);
                                    }
                                });
                                collapsing(ui, c::PLAN, |ui| self.show_plan(ui));
                            },
                        );
                        if let Some(diagnostics) = &mut self.diagnostics {
                            let section = collapsing(ui, c::DIAGNOSTICS, |ui| {
                                let controls_target = diagnostics.show(ui, ctx, !active && !self.close_wait);
                                (controls_target, diagnostics.open_context_target(ctx))
                            });
                            let open_target = self.diagnostic_open_reveal.target(&section);
                            let controls_target = section.body_returned.as_ref().and_then(|(target, _)| *target);
                            // Explicit control/ancestor requests retain priority over disclosure.
                            if let Some(target) = controls_target {
                                ui.scroll_to_rect(target, Some(egui::Align::Center));
                            } else if let Some(target) = open_target {
                                ui.scroll_to_rect_animation(target, Some(egui::Align::Min), egui::style::ScrollAnimation::none());
                            }
                        }
                        if let Some(target) = self.form.height_reveal_target(ctx) {
                            ui.scroll_to_rect_animation(target, None, egui::style::ScrollAnimation::none());
                        }
                        // Nested preview/diagnostic scroll areas must not consume this request.
                        if let Some(target) = self.start_condition_error_target.or(start_condition_target) {
                            // A prior wheel can still be settling on the click frame.
                            // Complete the navigation once the target is visible in a later frame.
                            if self.start_condition_reveal_frame.is_some_and(|frame| frame < ctx.cumulative_frame_nr())
                                && ui.clip_rect().contains_rect(target)
                                && ctx.input(|input| input.smooth_scroll_delta == egui::Vec2::ZERO)
                            {
                                self.start_condition_reveal_frame = None;
                            }
                            #[cfg(test)]
                            {
                                let frame = ctx.cumulative_frame_nr();
                                ctx.data_mut(|data| data.insert_temp(egui::Id::new("start-reason-target"), (frame, target)));
                            }
                            ui.scroll_to_rect_animation(
                                target,
                                Some(egui::Align::Min),
                                egui::style::ScrollAnimation::none(),
                            );
                        }
                    });
                #[cfg(test)]
                ctx.data_mut(|data| data.insert_temp(egui::Id::new("start-reason-offset"), scroll_trace_output.state.offset));
                #[cfg(test)]
                ctx.data_mut(|data| data.insert_temp(
                    egui::Id::new("feature-tool-download-viewport"),
                    (scroll_trace_output.inner_rect.intersect(ui.clip_rect()), scroll_trace_output.state.offset),
                ));
                if self.scroll_trace_remaining > 0 && self.scroll_trace_armed {
                    self.scroll_trace_remaining -= 1;
                    let (pointer, raw, smooth, focused) = ctx.input(|i| (i.pointer.hover_pos(), i.raw_scroll_delta, i.smooth_scroll_delta, i.focused));
                    let rect = scroll_trace_output.inner_rect;
                    eprintln!("dlpad-scroll-trace stage=download remaining={} pointer={:?} raw_before={:?} smooth_before={:?} raw_after={:?} smooth_after={:?} focused={} offset={:?} content={:?} inner=[{},{},{},{}]", self.scroll_trace_remaining, pointer, trace_raw_before, trace_smooth_before, raw, smooth, focused, scroll_trace_output.state.offset, scroll_trace_output.content_size, rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                }
            });
        if self.close_prompt {
            egui::Window::new(if self.close_wait {
                c::CLOSE_WAIT_TITLE
            } else {
                c::CLOSE_ACTIVE
            })
            .id(egui::Id::new(c::CLOSE_ACTIVE))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if self.close_wait {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(16.0));
                        ui.label(c::CLOSE_WAIT_HELP);
                    });
                } else {
                    ui.label(c::BACKGROUND_RESTORE);
                    if self.activation.is_none() {
                        ui.label(c::BACKGROUND_UNAVAILABLE);
                    }
                }
                ui.horizontal(|ui| {
                    if crate::ui_components::ghost(
                        ui,
                        c::BACKGROUND,
                        !self.close_wait
                            && self.activation.is_some()
                            && self.local_export_close_ready(),
                    )
                    .clicked()
                    {
                        self.hidden = true;
                        self.close_prompt = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    }
                    if crate::ui_components::danger(
                        ui,
                        c::STOP_EXIT,
                        !self.close_wait && self.local_export_can_request_stop(),
                    )
                    .clicked()
                    {
                        self.close_wait = true;
                        self.stop_app_update();
                        self.stop_local_export();
                        self.appearance.request_stop();
                        if let Some(panel) = &mut self.app_update_panel {
                            panel.request_stop();
                        }
                        self.tool_preferences.request_stop();
                        self.metrics.request_stop();
                        if let Some(tools) = &mut self.tools {
                            tools.request_stop();
                        }
                        if let Some(history) = &mut self.history {
                            history.request_stop();
                        }
                        if let Some(diagnostics) = &mut self.diagnostics {
                            diagnostics.request_stop();
                        }
                        if let Some(library) = &mut self.library {
                            library.request_stop();
                        }
                        if let Some(worker) = &self.worker {
                            worker.request_stop(false);
                        }
                    }
                    if crate::ui_components::ghost(ui, c::RETURN, !self.close_wait).clicked() {
                        self.close_prompt = false;
                    }
                });
            });
        }
    }
}
fn navigation_button(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    // Labels retain semantic identity across the new four-purpose order.
    // Historical positional auto IDs are deliberately not claimed byte-identical.
    let key = if label == c::DOWNLOADS {
        "downloads"
    } else if label == c::LIBRARY {
        "library"
    } else if label == c::HISTORY {
        "history"
    } else if label == c::SETTINGS {
        "settings"
    } else {
        label
    };
    ui.scope_builder(egui::UiBuilder::new().id(("main-navigation", key)), |ui| {
        crate::ui_components::navigation_choice(ui, selected, label)
    })
    .inner
}

fn tool_preference_index(tool: crate::managed::ToolId) -> usize {
    match tool {
        crate::managed::ToolId::YtDlp => 0,
        crate::managed::ToolId::Ffmpeg => 1,
        crate::managed::ToolId::Deno => 2,
    }
}
fn executable_field(
    ui: &mut egui::Ui,
    label_text: &str,
    value: &mut String,
    available: bool,
) -> (bool, bool) {
    let id = egui::Id::new(("download-environment-input", label_text));
    let explanation = match label_text {
        c::TOOL_PATH => Some((
            c::DOWNLOAD_ENVIRONMENT,
            std::borrow::Cow::Borrowed(c::TOOL_PREF_HELP),
        )),
        c::FFMPEG_PATH => Some((
            c::FFMPEG_HELP_TITLE,
            std::borrow::Cow::Owned(format!(
                "{}\n\n{}\n{}",
                c::FFMPEG_HELP,
                c::FFMPEG_DEFAULT_EXAMPLE,
                c::FFMPEG_CONFLICT_VERSION
            )),
        )),
        c::DENO_PATH => Some((
            c::DENO_HELP_TITLE,
            std::borrow::Cow::Owned(format!(
                "{}\n\n{}\n{}",
                c::DENO_HELP,
                c::DENO_DEFAULT_EXAMPLE,
                c::DENO_CONFLICT_VERSION
            )),
        )),
        _ => None,
    };
    let (label, help_response) = if let Some((title, text)) = &explanation {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let label = ui.label(egui::RichText::new(label_text).small().weak());
            let help = crate::ui_components::help_button(
                ui,
                ("environment-label-help", label_text),
                title,
                text.as_ref(),
            );
            (label, Some(help))
        })
        .inner
    } else {
        (
            ui.label(egui::RichText::new(label_text).small().weak()),
            None,
        )
    };
    let inline = ui.available_width() >= 232.0;
    let (response, browse_response) = ui
        .with_layout(
            if inline {
                egui::Layout::left_to_right(egui::Align::Center)
            } else {
                egui::Layout::top_down(egui::Align::Min)
            },
            |ui| {
                let width = (ui.available_width()
                    - if inline {
                        112.0 + ui.spacing().item_spacing.x
                    } else {
                        0.0
                    }
                    - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                    .max(0.0);

                let response =
                    crate::ui_components::text_input_with_id(ui, value, 32768, width, id)
                        .labelled_by(label.id);
                let browse_response = crate::ui_components::secondary_sized(
                    ui,
                    match label_text {
                        c::TOOL_PATH => c::TOOL_BROWSE_YTDLP,
                        c::FFMPEG_PATH => c::TOOL_BROWSE_FFMPEG,
                        _ => c::TOOL_BROWSE_DENO,
                    },
                    available,
                    egui::vec2(112.0, 32.0),
                );
                let browse_response =
                    crate::ui_components::context_help(browse_response, c::TOOL_BROWSE_HELP);
                #[cfg(test)]
                ui.ctx().data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new("tool-browser-test-geometry"),
                        (response.rect, browse_response.rect, response.id),
                    )
                });
                (response, browse_response)
            },
        )
        .inner;
    let help_response = help_response.map(|help| {
        let (title, text) = explanation
            .as_ref()
            .expect("allocated help has explanation");
        crate::ui_components::show_help(
            ui,
            help,
            title,
            text.as_ref(),
            Some(response.rect.union(browse_response.rect)),
        )
    });
    let group = label.rect.union(response.rect).union(browse_response.rect);
    let group = help_response
        .as_ref()
        .map_or(group, |help| group.union(help.rect));
    for focused in [&response, &browse_response]
        .into_iter()
        .chain(help_response.as_ref())
    {
        reveal_executable_context(ui, group, focused);
    }
    #[cfg(test)]
    ui.ctx().data_mut(|data| {
        data.insert_temp(
            egui::Id::new("executable-context-geometry"),
            ExecutableContextGeometry {
                label: label.rect,
                help: help_response.as_ref().map(|r| (r.rect, r.id)),
                input: (response.rect, response.id),
                browse: (browse_response.rect, browse_response.id),
                viewport: ui.clip_rect(),
            },
        )
    });
    (response.changed(), browse_response.clicked())
}
fn reveal_executable_context(ui: &mut egui::Ui, group: egui::Rect, response: &egui::Response) {
    if !response.enabled() || !response.gained_focus() {
        return;
    }
    let viewport = ui.clip_rect();
    if group.width() <= viewport.width() && group.height() <= viewport.height() {
        ui.scroll_to_rect(group, Some(egui::Align::Center));
    } else {
        response.scroll_to_me(None);
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
struct ExecutableContextGeometry {
    label: egui::Rect,
    help: Option<(egui::Rect, egui::Id)>,
    input: (egui::Rect, egui::Id),
    browse: (egui::Rect, egui::Id),
    viewport: egui::Rect,
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, limit: usize) -> bool {
    let id = egui::Id::new(("download-environment-input", label));
    let label = ui.label(egui::RichText::new(label).small().weak());
    let response = crate::ui_components::text_input_with_id(ui, value, limit, f32::INFINITY, id)
        .labelled_by(label.id);
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    response.changed()
}
// The selected source is consumed by one worker and never saved in settings.
struct WorkerData {
    data: PathBuf,
    cookie_source: Option<PathBuf>,
}
impl From<PathBuf> for WorkerData {
    fn from(data: PathBuf) -> Self {
        Self {
            data,
            cookie_source: None,
        }
    }
}
fn execute(
    data: impl Into<WorkerData>,
    tool: PathBuf,
    options: Option<DownloadOptions>,
    resume: Option<String>,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    feed: ProgressFeed,
) -> Completion {
    let WorkerData {
        data,
        cookie_source,
    } = data.into();
    let mut cookies = None;
    let mut managed_execution = None;
    #[cfg(windows)]
    let mut runtime_environment = None;
    let id = resume.clone().unwrap_or_else(|| {
        format!(
            "job-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    });
    let mut attempt = 0;
    let mut tool_version = None;
    let mut ffmpeg_version = None;
    let mut deno_version = None;
    let mut job_state = None;
    let mut resumable = false;
    let result = (|| -> Result<_, DownloadError> {
        if let Some(source) = cookie_source {
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                let cache = data.join("session-cookies");
                match std::fs::DirBuilder::new().mode(0o700).create(&cache) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(_) => return Err(DownloadError::Policy(c::COOKIE_IMPORT_FAILED)),
                }
                cookies = Some(
                    dlpad_engine::cookies::import_session(&source, &cache)
                        .map_err(|_| DownloadError::Policy(c::COOKIE_IMPORT_FAILED))?,
                );
            }
            #[cfg(windows)]
            {
                let cache = data.join("session-cookies");
                dlpad_engine::cookies::prepare_session_cache(&cache)
                    .map_err(|_| DownloadError::Policy(c::COOKIE_IMPORT_FAILED))?;
                cookies = Some(
                    dlpad_engine::cookies::import_session(&source, &cache)
                        .map_err(|_| DownloadError::Policy(c::COOKIE_IMPORT_FAILED))?,
                );
            }
            #[cfg(not(any(unix, windows)))]
            {
                let _ = source;
                return Err(DownloadError::Policy(c::COOKIE_UNAVAILABLE));
            }
        }
        let repository = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let managed_root = data.join("managed-tools");
        if managed_root.exists() {
            let saved;
            let selected = if let Some(options) = options.as_ref() {
                options
            } else {
                saved = repository.stored_options(&id)?;
                &saved
            };
            let canonical_root = managed_root
                .canonicalize()
                .unwrap_or_else(|_| managed_root.clone());
            let managed_path = |path: &Path| {
                path.canonicalize().map_or_else(
                    |_| path.starts_with(&managed_root),
                    |canonical| canonical.starts_with(&canonical_root),
                )
            };
            if managed_path(&tool)
                || selected.deno.as_deref().is_some_and(managed_path)
                || selected.ffmpeg.as_deref().is_some_and(managed_path)
            {
                let manager = dlpad_engine::tools::ToolManager::new(&managed_root)
                    .map_err(|_| DownloadError::Policy(c::TOOL_EXECUTION_PENDING))?;
                #[cfg(windows)]
                if let Some(deno) = selected.deno.as_deref() {
                    runtime_environment =
                        manager
                            .prepare_managed_deno_environment(deno, &cancel)
                            .map_err(|_| DownloadError::Policy(c::TOOL_EXECUTION_PENDING))?;
                }
                managed_execution = Some(
                    manager
                        .begin_execution()
                        .map_err(|_| DownloadError::Policy(c::TOOL_EXECUTION_PENDING))?,
                );
            }
        }
        let result = if let Some(mut options) = options {
            options.output_dir = dlpad_engine::location::prepare(
                &data,
                dlpad_engine::location::LocationRequest::Selected(options.output_dir.clone()),
            )
            .map_err(|_| DownloadError::Policy(c::INVALID_ROOT))?;
            repository.create_job(&id, &options)?;
            repository.execute_with_session_naming(
                &id,
                &options,
                &tool,
                DownloadSession {
                    #[cfg(windows)]
                    runtime_environment: runtime_environment.as_ref(),
                    cancel: &cancel,
                    pause: Some(&pause),
                    progress: Some(feed),
                    cookies: cookies.as_ref(),
                    managed: managed_execution.as_ref(),
                },
                options.final_name_style.map(|style| match style {
                    dlpad_core::options::FinalNameStyle::TitleWithJobId => {
                        dlpad_engine::download::FinalNameStyle::TitleWithJobId
                    }
                }),
            )
        } else {
            let saved_style = repository
                .stored_options(&id)?
                .final_name_style
                .map(|style| match style {
                    dlpad_core::options::FinalNameStyle::TitleWithJobId => {
                        dlpad_engine::download::FinalNameStyle::TitleWithJobId
                    }
                });
            repository.resume_with_session_naming(
                &id,
                &tool,
                DownloadSession {
                    #[cfg(windows)]
                    runtime_environment: runtime_environment.as_ref(),
                    cancel: &cancel,
                    pause: Some(&pause),
                    progress: Some(feed),
                    cookies: cookies.as_ref(),
                    managed: managed_execution.as_ref(),
                },
                saved_style,
            )
        };
        attempt = repository
            .job(&id)
            .ok()
            .and_then(|j| j.attempts.last().map(|a| a.number))
            .unwrap_or(0);
        tool_version = repository.tool_version(&id).ok().flatten();
        ffmpeg_version = repository.ffmpeg_version(&id).ok().flatten();
        deno_version = repository.deno_version(&id).ok().flatten();
        job_state = repository.job(&id).ok().map(|j| j.state);
        resumable = matches!(
            job_state,
            Some(
                dlpad_core::jobs::JobState::Paused
                    | dlpad_core::jobs::JobState::Failed
                    | dlpad_core::jobs::JobState::Interrupted
            )
        ) && repository.stored_options(&id).is_ok();
        result
    })();
    // Preserve the private copy and managed marker when cleanup is unconfirmed.
    let cleanup_deferred = matches!(&result, Err(DownloadError::SessionCleanupDeferred));
    let managed_cleanup_pending = managed_execution.is_some() && cleanup_deferred;
    let managed_cleanup_failed =
        if !cleanup_deferred && let Some(execution) = managed_execution.take() {
            execution.finish().is_err()
        } else {
            false
        };
    if cleanup_deferred && let Some(lease) = cookies.as_mut() {
        lease.defer_cleanup();
    }
    let cleanup_failed = cookies
        .as_mut()
        .is_some_and(|lease| lease.cleanup().is_err());
    let mut completion = match result {
        Ok(result) => Completion::Job {
            status: format!(
                "{} · {}",
                c::DOWNLOADED,
                result
                    .job
                    .artifacts
                    .first()
                    .map(|a| a.path.display().to_string())
                    .unwrap_or_default()
            ),
            report: None,
            resume: None,
        },
        Err(DownloadError::Execution { result, .. })
            if result.outcome == dlpad_engine::process::ProcessOutcome::Cancelled =>
        {
            Completion::Job {
                status: if job_state == Some(dlpad_core::jobs::JobState::Paused) {
                    c::STATE_PAUSED
                } else {
                    c::CANCELLED
                }
                .into(),
                report: None,
                resume: resumable.then_some(id),
            }
        }
        Err(error) => {
            let mut report = DiagnosticReport::new(u64::from(attempt), FailureStage::Unknown, None);
            report.add_tool(ToolComponent::Dlpad, env!("CARGO_PKG_VERSION"), false);
            if let Some(version) = tool_version {
                report.add_tool(ToolComponent::YtDlp, &version, false);
            }
            if let Some(version) = ffmpeg_version {
                report.add_tool(ToolComponent::Ffmpeg, &version, false);
            }
            if let Some(version) = deno_version {
                report.add_tool(ToolComponent::JsRuntime, &version, false);
            }
            if let DownloadError::Execution { stage, result } = &error {
                report.stage = match *stage {
                    "metadata" => FailureStage::Metadata,
                    "download" => FailureStage::Download,
                    _ => FailureStage::ToolDiscovery,
                };
                report.exit_code = result.exit_code;
                report.add_log(&result.logs);
                report.truncated |= result.truncated;
            } else {
                if matches!(
                    error,
                    DownloadError::ToolDiscovery(_) | DownloadError::DenoDiscovery(_)
                ) {
                    report.stage = FailureStage::ToolDiscovery;
                }
                report.add_log(&error.to_string());
            }
            Completion::Job {
                status: c::FAILED.into(),
                report: Some(Box::new(report)),
                resume: resumable.then_some(id),
            }
        }
    };
    if (cleanup_failed || cleanup_deferred)
        && let Completion::Job { status, .. } = &mut completion
    {
        status.push_str(" · ");
        status.push_str(c::COOKIE_CLEANUP_FAILED);
    }
    if (managed_cleanup_pending || managed_cleanup_failed)
        && let Completion::Job { status, .. } = &mut completion
    {
        status.push_str(" · ");
        status.push_str(c::TOOL_EXECUTION_PENDING);
    }
    completion
}

#[cfg(test)]
mod tests {
    #[test]
    fn actual_startup_readiness_explains_busy_failure_and_clears_after_ready() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        let (stop_tx, stop_rx) = mpsc::channel();
        let (completion_tx, completion_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let _ = stop_rx.recv_timeout(Duration::from_secs(30));
            drop(completion_tx);
        });
        app.worker = Some(Worker {
            handle,
            receiver: completion_rx,
            cancel: Arc::new(AtomicBool::new(false)),
            pause: Arc::new(AtomicBool::new(false)),
        });
        app.ready = false;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (width, height) in [(800.0, 600.0), (400.0, 300.0)] {
                for dark in [false, true] {
                    ctx.set_theme(if dark {
                        egui::ThemePreference::Dark
                    } else {
                        egui::ThemePreference::Light
                    });
                    let frame = |app: &mut Candidate| {
                        ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, height),
                                )),
                                focused: true,
                                ..Default::default()
                            },
                            |ctx| app.show_ui(ctx),
                        )
                    };
                    assert_eq!(app.download_disabled_reason(), Some(c::RECOVERY_CHECKING));
                    for _ in 0..3 {
                        let output = frame(&mut app);
                        for label in [c::STARTUP_CHECKING_TITLE, c::STARTUP_LOCKED_HELP] {
                            let (glyph, clip) = output
                                .shapes
                                .iter()
                                .find_map(|shape| {
                                    if let egui::epaint::Shape::Text(text) = &shape.shape
                                        && text.galley.text() == label
                                    {
                                        Some((text.visual_bounding_rect(), shape.clip_rect))
                                    } else {
                                        None
                                    }
                                })
                                .expect("startup reason is painted without hovering or clicking");
                            assert!(
                                clip.contains_rect(glyph),
                                "reason fully painted at {width}x{height}: {glyph:?}/{clip:?}"
                            );
                        }
                        assert!(!output.shapes.iter().any(|shape| matches!(&shape.shape,
                            egui::epaint::Shape::Text(text) if [c::CANCEL, c::PAUSE].contains(&text.galley.text()))),
                            "recovery must not expose download-only cancel/pause rows");
                        assert!(
                            ctx.memory(|memory| memory.focused()).is_none(),
                            "readiness must not claim keyboard focus"
                        );
                    }
                }
            }
        }));
        stop_tx.send(()).unwrap();
        app.worker.take().unwrap().handle.join().unwrap();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        app.status = c::RECOVERY_FAILED.into();
        assert_eq!(app.download_disabled_reason(), Some(c::RECOVERY_FAILED));
        for ready in [false, true] {
            app.ready = ready;
            for _ in 0..3 {
                let output = ctx.run(egui::RawInput::default(), |ctx| app.show_ui(ctx));
                let visible = output.shapes.iter().any(|shape| matches!(&shape.shape,
                    egui::epaint::Shape::Text(text) if text.galley.text() == c::STARTUP_FAILED_TITLE));
                assert_eq!(
                    visible, !ready,
                    "failure remains visible until recovery succeeds"
                );
            }
        }
    }

    #[cfg(feature = "cpu")]
    #[test]
    fn actual_candidate_cpu_initial_resize_theme_footer_shrink_scene_is_fully_painted() {
        use egui_software_backend::{BufferMutRef, ColorFieldOrder, EguiSoftwareRender};
        use std::io::Write;
        let fixture = std::mem::ManuallyDrop::new(Fixture::new());
        let owned = fixture.0.join("background-raster");
        std::fs::create_dir(&owned).unwrap();
        let mut records = Vec::new();
        let mut total_holes = 0usize;
        for (case, (dark, zoom)) in [(true, 1.0f32), (false, 1.0), (true, 2.0), (false, 2.0)]
            .into_iter()
            .enumerate()
        {
            let root = owned.join(format!("case-{case}"));
            std::fs::create_dir(&root).unwrap();
            let data = root.join("data");
            let media = root.join("media");
            std::fs::create_dir(&data).unwrap();
            std::fs::create_dir(&media).unwrap();
            let media = media.canonicalize().unwrap();
            let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
            store
                .set_setting("library_root", media.to_str().unwrap())
                .unwrap();
            drop(store);
            std::fs::write(
                data.join("theme.json"),
                if dark { "\"Dark\"" } else { "\"Light\"" },
            )
            .unwrap();
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            let mut app = Candidate::new(data);
            app.use_host_metrics_freshness_timer();
            app.appearance.fixture_initial_scale((zoom * 100.0) as u16);
            app.apply_appearance(&ctx);
            let mut cached = EguiSoftwareRender::new(ColorFieldOrder::Bgra);
            let mut direct = EguiSoftwareRender::new(ColorFieldOrder::Bgra).with_caching(false);
            let mut time = 0.0;
            let mut file_count = 0usize;
            let mut frame = |app: &mut Candidate,
                             phase: &str,
                             physical: [usize; 2],
                             close: bool| {
                time += 1.0 / 60.0;
                let factor = ctx.zoom_factor();
                let mut input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(physical[0] as f32 / factor, physical[1] as f32 / factor),
                    )),
                    focused: false,
                    time: Some(time),
                    ..Default::default()
                };
                let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
                viewport.native_pixels_per_point = Some(1.0);
                if close {
                    viewport.events.push(egui::ViewportEvent::Close);
                }
                let output = ctx.run(input, |ctx| app.show_ui(ctx));
                let viewport = ctx.viewport_rect();
                let expected = egui::vec2(physical[0] as f32, physical[1] as f32);
                eprintln!(
                    "background-raster-viewport case={case} phase={phase} viewport={viewport:?} pp={} target={expected:?}",
                    output.pixels_per_point
                );
                assert_eq!(
                    viewport.size() * output.pixels_per_point,
                    expected,
                    "actual first viewport and CPU target dimensions must agree"
                );
                let jobs = ctx.tessellate(output.shapes.clone(), output.pixels_per_point);
                assert!(jobs.len() <= 8192 && jobs.iter().all(|job| job.clip_rect.is_finite()));
                let footer =
                    egui::containers::panel::PanelState::load(&ctx, egui::Id::new("status"))
                        .map(|state| state.rect);
                for (label, renderer) in [("cached", &mut cached), ("direct", &mut direct)] {
                    let mut pixels = vec![[0u8; 4]; physical[0] * physical[1]];
                    renderer.render(
                        &mut BufferMutRef::new(&mut pixels, physical[0], physical[1]),
                        &jobs,
                        &output.textures_delta,
                        output.pixels_per_point,
                    );
                    let mut holes = 0usize;
                    let mut first = None;
                    for (index, pixel) in pixels.iter().enumerate() {
                        if pixel[3] == 0 {
                            holes += 1;
                            if first.is_none() {
                                first = Some((index % physical[0], index / physical[0]));
                            }
                        }
                    }
                    // RGB0 is legal for intentional dark content; alpha0 marks an unpainted scene pixel.
                    total_holes += holes;
                    records.push(format!("case={case} phase={phase} renderer={label} px={physical:?} zoom={} dark={} footer={footer:?} holes={holes} first={first:?}",output.pixels_per_point,ctx.style().visuals.dark_mode));
                    eprintln!("background-raster {}", records.last().unwrap());
                    if (holes > 0 || phase == "first") && label == "cached" && file_count < 2 {
                        file_count += 1;
                        let path = root.join(format!(
                            "{phase}-{label}-{}x{}.bgra",
                            physical[0], physical[1]
                        ));
                        let mut file = std::fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .open(&path)
                            .unwrap();
                        for row in pixels.chunks(physical[0]) {
                            let bytes: Vec<u8> =
                                row.iter().flat_map(|p| p.iter().copied()).collect();
                            file.write_all(&bytes).unwrap();
                        }
                        file.sync_all().unwrap();
                    }
                }
                footer
            };
            frame(&mut app, "first", [960, 640], false);
            wait_finished(&mut app);
            frame(&mut app, "ready", [960, 640], false);
            let short = frame(&mut app, "short-stable", [960, 640], false).unwrap();
            app.status = "합성 상태 높이 변화 확인 · ".repeat(40);
            frame(&mut app, "long-transition", [960, 640], false);
            let long = frame(&mut app, "long-stable", [960, 640], false).unwrap();
            assert!(
                long.height() > short.height() + 12.0,
                "setup must produce an actual larger status panel: {short:?} -> {long:?}"
            );
            app.status = c::RECOVERY_DONE.into();
            let shrink = frame(&mut app, "shrink-transition", [960, 640], false).unwrap();
            frame(&mut app, "shrink-stable", [960, 640], false);
            assert!(
                shrink.height() < long.height(),
                "setup must actually shrink the painted status panel"
            );
            frame(&mut app, "resize-small", [800, 600], false);
            frame(&mut app, "resize-small-stable", [800, 600], false);
            frame(&mut app, "resize-large", [960, 640], false);
            ctx.set_theme(if dark {
                egui::ThemePreference::Light
            } else {
                egui::ThemePreference::Dark
            });
            frame(&mut app, "theme-transition", [960, 640], false);
            frame(&mut app, "theme-stable", [960, 640], false);
            frame(&mut app, "close-request", [960, 640], true);
            drop(app);
        }
        let text = records.join("\n");
        assert!(text.len() <= 65536);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(owned.join("coverage.txt"))
            .unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file.sync_all().unwrap();
        println!(
            "owned_background_raster={} frame_records={} unpainted_pixels={total_holes} physical_OS_verified=false",
            owned.display(),
            records.len()
        );
        assert_eq!(
            total_holes, 0,
            "actual Candidate CPU scene has unpainted alpha0 pixels; retained owned coverage/raster artifacts"
        );
    }

    fn settings_navigation_owned_app() -> (Fixture, Candidate) {
        let fixture = Fixture::new();
        let data = fixture.0.join("data");
        let media = fixture.0.join("media");
        std::fs::create_dir(&data).unwrap();
        std::fs::create_dir(&media).unwrap();
        let media = media.canonicalize().unwrap();
        let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
        store
            .set_setting("library_root", media.to_str().unwrap())
            .unwrap();
        drop(store);
        let mut app = Candidate::new(data);
        wait_finished(&mut app);
        (fixture, app)
    }
    #[test]
    fn actual_four_navigation_settings_tools_routes_keep_entity_ids_draft_and_close_wait() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        let (_fixture, mut app) = settings_navigation_owned_app();
        let draft = (
            app.url.clone(),
            app.output_path(),
            app.tool.clone(),
            app.ffmpeg.clone(),
            app.deno.clone(),
        );
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut nodes = std::collections::BTreeMap::<NodeId, Node>::new();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, enabled, events| {
            time += 0.05;
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add_enabled_ui(enabled, |ui| {
                            // Adjacent settings status can change the parent auto-ID sequence.
                            // The actual four navigation actions must retain semantic identities.
                            if app.settings && app.tools_open {
                                ui.label(c::TOOLS);
                            }
                            ui.horizontal_wrapped(|ui| app.show_navigation(ui));
                            if app.settings {
                                app.show_settings_navigation(ui);
                            }
                        });
                    });
                },
            );
            nodes.extend(out.platform_output.accesskit_update.unwrap().nodes);
            nodes.clone()
        };
        let exact = |tree: &std::collections::BTreeMap<NodeId, Node>, label| {
            let matches: Vec<_> = tree
                .iter()
                .filter(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
                .collect();
            assert_eq!(matches.len(), 1, "actual unique action {label}");
            *matches[0].0
        };
        let click = |target| {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target,
                data: None,
            })
        };
        let first = frame(&mut app, true, vec![]);
        let labels = [c::DOWNLOADS, c::LIBRARY, c::HISTORY, c::SETTINGS];
        let identities: Vec<_> = labels.iter().map(|label| exact(&first, *label)).collect();
        assert!(
            !first
                .values()
                .any(|node| node.role() == Role::Button && node.label() == Some(c::TOOLS)),
            "no fifth top-level Tools"
        );
        let xs: Vec<_> = identities
            .iter()
            .map(|id| first[id].bounds().unwrap().x0)
            .collect();
        assert!(
            xs.windows(2).all(|w| w[0] < w[1]),
            "actual requested four-purpose order"
        );
        let settings = frame(&mut app, true, vec![click(identities[3])]);
        assert!(
            app.settings && !app.tools_open && app.settings_section == SettingsSection::General
        );
        let update_id = exact(&settings, c::APP_UPDATE_TITLE);
        frame(&mut app, true, vec![click(update_id)]);
        assert!(
            app.settings && !app.tools_open && app.settings_section == SettingsSection::AppUpdate
        );
        assert!(
            app.app_update_panel.is_none(),
            "navigation alone does not start network or workers"
        );
        let tools_id = exact(&settings, c::TOOLS);
        frame(&mut app, true, vec![click(tools_id)]);
        assert!(app.settings && app.tools_open && app.settings_section == SettingsSection::Tools);
        let panel_address = app.tools.as_ref().map(|panel| panel as *const _).unwrap();
        let general = exact(&settings, c::SETTINGS_GENERAL);
        frame(&mut app, true, vec![click(general)]);
        assert!(app.settings && !app.tools_open);
        frame(&mut app, true, vec![click(tools_id)]);
        assert_eq!(
            app.tools.as_ref().map(|panel| panel as *const _).unwrap(),
            panel_address,
            "navigation retains worker/cache owner"
        );
        let after = frame(&mut app, true, vec![]);
        assert_eq!(
            labels
                .iter()
                .map(|label| exact(&after, *label))
                .collect::<Vec<_>>(),
            identities
        );
        assert_eq!(
            (
                app.url.clone(),
                app.output_path(),
                app.tool.clone(),
                app.ffmpeg.clone(),
                app.deno.clone()
            ),
            draft
        );
        frame(&mut app, false, vec![click(identities[0])]);
        assert!(
            app.settings && app.tools_open,
            "disabled actual nav refuses routing"
        );
        frame(&mut app, true, vec![click(general)]);
        app.close_wait = true;
        frame(&mut app, true, vec![click(tools_id)]);
        assert!(
            app.settings && !app.tools_open && app.settings_section == SettingsSection::General,
            "actual Tools intent respects CloseWait"
        );
    }
    #[test]
    fn actual_download_save_draft_reuses_global_editor_and_preserves_library_setting() {
        let (_fixture, mut app) = settings_navigation_owned_app();
        let original = app.data.parent().unwrap().join("한글 현재 저장 위치");
        app.set_output_path(original.clone());
        let store = dlpad_engine::store::Store::open(&app.data.join("theme.sqlite")).unwrap();
        let stored = store.setting("library_root").unwrap();
        drop(store);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let id = egui::Id::new("download-save-path-input");
        let node_id = egui::accesskit::NodeId(id.value());
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, events| {
            time += 0.05;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .show(ctx, |ui| app.show_download_location_draft(ui, ctx));
                },
            )
        };
        frame(&mut app, vec![]);
        assert_eq!(
            app.output_path(),
            original,
            "readonly paint preserves exact output_original"
        );
        frame(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target: node_id,
                    data: None,
                },
            )],
        );
        let replacement = app.data.parent().unwrap().join("한글 다음 저장 위치");
        let typed = replacement.to_str().unwrap().to_owned();
        frame(
            &mut app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..egui::Modifiers::NONE
                    },
                },
                egui::Event::Text(typed.clone()),
            ],
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        assert_eq!(
            app.output_path(),
            replacement,
            "actual shared editor changes draft, not library persistence"
        );
        assert_eq!(
            egui::text_edit::TextEditState::load(&ctx, id)
                .unwrap()
                .cursor
                .char_range()
                .unwrap()
                .primary
                .index,
            typed.chars().count()
        );
        let response = ctx.read_response(id).unwrap();
        assert!((response.rect.height() - 32.0).abs() < 0.1);
        app.close_wait = true;
        frame(&mut app, vec![egui::Event::Text("blocked".into())]);
        assert_eq!(
            app.output_path(),
            replacement,
            "disabled Settings field cannot mutate draft during CloseWait"
        );
        let store = dlpad_engine::store::Store::open(&app.data.join("theme.sqlite")).unwrap();
        assert_eq!(
            store.setting("library_root").unwrap(),
            stored,
            "no fake durable default saved"
        );
    }

    #[test]
    fn tool_browser_fields_fit_320_and_480_content_with_stable_labelled_inputs() {
        for width in [200.0, 320.0, 480.0] {
            let ctx = egui::Context::default();
            let mut value = "draft".to_owned();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 160.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(width);
                            let left = ui.cursor().left();
                            executable_field(ui, c::FFMPEG_PATH, &mut value, false);
                            assert!(
                                ui.min_rect().right() <= left + width + 1.0,
                                "browser overflow at {width}"
                            );
                            assert!(ui.min_rect().height() >= 32.0);
                            let (input, button, identity) = ctx
                                .data(|data| {
                                    data.get_temp::<(egui::Rect, egui::Rect, egui::Id)>(
                                        egui::Id::new("tool-browser-test-geometry"),
                                    )
                                })
                                .unwrap();
                            assert_eq!(
                                identity,
                                egui::Id::new(("download-environment-input", c::FFMPEG_PATH))
                            );
                            if width >= 232.0 {
                                assert!(input.right() <= button.left());
                            } else {
                                assert!(input.bottom() <= button.top());
                            }
                        },
                    );
                });
            });
            assert_eq!(value, "draft");
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn queued_tool_file_close_preserves_draft_and_stored_preference() {
        for closing in [false, true] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.ffmpeg = "original typed draft".into();
            let store = dlpad_engine::store::Store::open(&fixture.0.join("theme.sqlite")).unwrap();
            let previous = store.setting("manual_tool_preferences").unwrap();
            drop(store);
            app.tool_path_picker = crate::tool_path_picker::PathPicker::completed_for_test(
                crate::tool_path_picker::Target::Ffmpeg,
                fixture.0.join("selected.exe"),
            );
            let context = egui::Context::default();
            app.close_wait = closing;
            if !closing {
                app.tool_path_picker.cancel();
            }
            let _ = context.run(egui::RawInput::default(), |ctx| app.poll_folder_picker(ctx));
            assert_eq!(app.ffmpeg, "original typed draft");
            assert!(!app.tool_path_picker.busy());
            let store = dlpad_engine::store::Store::open(&fixture.0.join("theme.sqlite")).unwrap();
            assert_eq!(store.setting("manual_tool_preferences").unwrap(), previous);
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn chosen_tool_changes_only_target_draft_and_never_implicitly_saves() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.tool = "yt draft".into();
        app.ffmpeg = "ff draft".into();
        app.deno = "deno draft".into();
        let store = dlpad_engine::store::Store::open(&fixture.0.join("theme.sqlite")).unwrap();
        let previous = store.setting("manual_tool_preferences").unwrap();
        drop(store);
        let selected = fixture.0.join("selected.exe");
        app.tool_path_picker = crate::tool_path_picker::PathPicker::completed_for_test(
            crate::tool_path_picker::Target::Deno,
            selected.clone(),
        );
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| app.poll_folder_picker(ctx));
        assert_eq!(app.deno, selected.to_str().unwrap());
        assert_eq!(app.tool, "yt draft");
        assert_eq!(app.ffmpeg, "ff draft");
        let store = dlpad_engine::store::Store::open(&fixture.0.join("theme.sqlite")).unwrap();
        assert_eq!(store.setting("manual_tool_preferences").unwrap(), previous);
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn picked_path_display_keeps_original_authority_until_user_edits() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let original = PathBuf::from(r"\\?\C:\한글\original");
        app.set_output_path(original.clone());
        assert_eq!(app.output_path(), original);
        assert_eq!(app.output, crate::path_picker::display_path(&original));
        app.output = fixture.0.to_string_lossy().into_owned();
        assert_eq!(app.output_path(), fixture.0);
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn download_gate_requires_valid_source_location_and_explicit_preview_bypass() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_URL_REQUIRED));
        assert!(!app.output.is_empty(), "first-run save folder is prefilled");
        assert!(
            !app.output_path().exists(),
            "presentation does not create media"
        );
        app.url = "https://fixture.invalid/video".into();
        assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_TOOL_REQUIRED));
        app.output.clear();
        assert_eq!(
            app.download_disabled_reason(),
            Some(c::MEDIA_LOCATION_REQUIRED)
        );
        app.output = fixture.0.to_str().unwrap().into();
        assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_TOOL_REQUIRED));
        app.tool = fixture.0.join("synthetic-tool").to_str().unwrap().into();
        assert_eq!(
            app.download_disabled_reason(),
            Some(c::MEDIA_PREVIEW_REQUIRED)
        );
        app.skip_preview = true;
        assert_eq!(app.download_disabled_reason(), None);
        app.url = "invalid".into();
        assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_URL_INVALID));
    }
    #[test]
    fn close_input_discards_already_queued_folder_selection() {
        for closing in [false, true] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.output = "original draft".into();
            let selected = fixture.0.join("selected");
            app.path_picker = crate::path_picker::PathPicker::completed_for_test(
                crate::path_picker::Target::Download,
                selected.clone(),
            );
            let ctx = egui::Context::default();
            let mut input = egui::RawInput::default();
            if closing {
                input
                    .viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .events
                    .push(egui::ViewportEvent::Close);
            }
            let _ = ctx.run(input, |ctx| app.poll_folder_picker(ctx));
            assert!(!app.path_picker.busy());
            assert_eq!(
                app.output,
                if closing {
                    "original draft".into()
                } else {
                    selected.to_str().unwrap().to_owned()
                }
            );
        }
    }

    #[test]
    fn disclosure_accesskit_expand_collapse_changes_visible_body_and_rejects_disabled_action() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        crate::ui_theme::configure_dlpad_theme(&ctx);
        let frame = |events: Vec<egui::Event>, enabled| {
            let mut visible = false;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add_enabled_ui(enabled, |ui| {
                            collapsing(ui, c::DOWNLOAD_ENVIRONMENT, |ui| {
                                visible = true;
                                ui.label(c::TOOL_PATH);
                            });
                        });
                    });
                },
            );
            (visible, output)
        };
        let (visible, output) = frame(vec![], true);
        assert!(!visible);
        let tree = output.platform_output.accesskit_update.unwrap();
        let (id, node) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(c::DOWNLOAD_ENVIRONMENT))
            .unwrap();
        assert!(node.supports_action(egui::accesskit::Action::Expand));
        assert!(node.supports_action(egui::accesskit::Action::Click));
        assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
        let event = |action| {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action,
                target: *id,
                data: None,
            })
        };
        frame(vec![event(egui::accesskit::Action::Expand)], true);
        assert!(frame(vec![], true).0);
        assert!(frame(vec![event(egui::accesskit::Action::Collapse)], false).0);
        frame(vec![event(egui::accesskit::Action::Collapse)], true);
        assert!(!frame(vec![], true).0);
        assert!(frame(vec![event(egui::accesskit::Action::Click)], true).0);
        assert!(!frame(vec![event(egui::accesskit::Action::Click)], true).0);
    }

    #[test]
    fn restored_managed_paths_preserve_custom_edits_and_close_wait() {
        for (close, edited) in [(false, false), (false, true), (true, false)] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.tool.clear();
            app.ffmpeg = "existing external ffmpeg".into();
            app.tool_preferences.edited(1);
            app.deno.clear();
            app.close_wait = close;
            if edited {
                app.tool = "later external yt-dlp".into();
                app.tool_preferences.edited(0);
            }
            app.apply_restored_managed_tools(vec![
                (crate::managed::ToolId::YtDlp, fixture.0.join("yt-dlp.exe")),
                (crate::managed::ToolId::Ffmpeg, fixture.0.join("ffmpeg.exe")),
                (crate::managed::ToolId::Deno, fixture.0.join("deno.exe")),
            ]);
            assert_eq!(app.ffmpeg, "existing external ffmpeg");
            if close {
                assert!(app.tool.is_empty() && app.deno.is_empty());
            } else {
                assert_eq!(
                    app.tool,
                    if edited {
                        "later external yt-dlp".into()
                    } else {
                        fixture.0.join("yt-dlp.exe").to_string_lossy().into_owned()
                    }
                );
                assert_eq!(app.deno, fixture.0.join("deno.exe").to_string_lossy());
            }
        }
    }

    #[test]
    fn saved_root_completion_preserves_later_edit_and_close_wait() {
        for (close, edited) in [(false, false), (false, true), (true, false)] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.output.clear();
            let selected = fixture.0.join("selected");
            let restored = selected.clone();
            app.spawn(move |_, _| {
                Completion::Recovery(Ok(RecoveryState {
                    restored_library_root: Some(restored),
                    ..RecoveryState::default()
                }))
            });
            app.close_wait = close;
            if edited {
                app.output = "later user edit".into();
            }
            wait_finished(&mut app);
            let expected = if edited {
                "later user edit".into()
            } else if close {
                String::new()
            } else {
                selected.to_string_lossy().into_owned()
            };
            assert_eq!(app.output, expected);
        }
    }
    #[test]
    fn invalid_saved_location_keeps_recovery_ready_for_folder_repair() {
        let fixture = Fixture::new();
        let store = dlpad_engine::store::Store::open(&fixture.0.join("theme.sqlite")).unwrap();
        store.set_setting("library_root", "relative-root").unwrap();
        drop(store);
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready);
        assert_eq!(app.status, c::LOCATION_FAILED);
        assert!(app.output.is_empty());
    }

    // Insert inside accessible.rs existing test module; actual helper, synthetic paths only.
    #[test]
    fn recovery_without_saved_root_has_no_media_default() {
        let temp = Fixture::new();
        let data = temp.0.as_path().join("data");
        std::fs::create_dir(&data).unwrap();
        assert!(restore_saved_library_root(&data).unwrap().is_none());
        assert_eq!(std::fs::read_dir(temp.0.as_path()).unwrap().count(), 1);
        let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
        assert!(store.setting("library_root").unwrap().is_none());
    }
    #[test]
    fn first_run_resolves_default_without_creating_media_or_saving_it() {
        let fixture = Fixture::new();
        let data = fixture.0.join("data");
        let media = fixture.0.join("default-media");
        std::fs::create_dir(&data).unwrap();
        assert_eq!(
            initial_library_root(&data, || Ok(media.clone())).unwrap(),
            Some(media.clone())
        );
        assert!(!media.exists());
        let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
        assert!(store.setting("library_root").unwrap().is_none());
        assert!(initial_library_root(&data, || Ok(PathBuf::from("relative"))).is_err());
        assert!(
            initial_library_root(&data, || Err(
                dlpad_engine::library::LibraryError::DocumentsUnavailable
            ))
            .is_err()
        );
    }

    #[test]
    fn recovery_restores_valid_saved_root_without_default() {
        let temp = Fixture::new();
        let data = temp.0.as_path().join("data");
        let media = temp.0.as_path().join("media");
        std::fs::create_dir(&data).unwrap();
        std::fs::create_dir(&media).unwrap();
        let expected = dlpad_engine::location::prepare_with_default(
            &data,
            dlpad_engine::location::LocationRequest::Selected(media),
            || panic!("no default"),
        )
        .unwrap();
        assert_eq!(restore_saved_library_root(&data).unwrap(), Some(expected));
    }
    #[test]
    fn recovery_invalid_saved_root_is_error_not_default() {
        let temp = Fixture::new();
        let data = temp.0.as_path().join("data");
        std::fs::create_dir(&data).unwrap();
        let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
        store.set_setting("library_root", "relative-root").unwrap();
        drop(store);
        assert!(restore_saved_library_root(&data).is_err());
        assert_eq!(std::fs::read_dir(temp.0.as_path()).unwrap().count(), 1);
    }

    #[test]
    fn collapsing_tree_exposes_target_state_during_animation() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut header_id = None;
        for open in [false, true, false] {
            if let Some(id) = header_id {
                let mut state = egui::collapsing_header::CollapsingState::load(&ctx, id).unwrap();
                state.set_open(open);
                state.store(&ctx);
            }
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    header_id = Some(
                        super::collapsing(ui, "header", |ui| ui.label("body"))
                            .header_response
                            .id,
                    );
                });
            });
            let update = output.platform_output.accesskit_update.unwrap();
            let id = eframe::egui::accesskit::NodeId(header_id.unwrap().value());
            let node = update
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .unwrap();
            assert_eq!(node.1.is_expanded(), Some(open));
        }
    }

    #[test]
    fn location_output_ack_preserves_later_edit_picker_identity_closewait_and_newer_request() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let first = fixture.0.join("original-draft");
        let applied = fixture.0.join("confirmed-root");
        let record = |app: &mut Candidate, serial| {
            app.location_output_intent = Some(LocationOutputIntent {
                serial,
                text: app.output.clone(),
                original: app.output_original.clone(),
            });
        };
        app.set_output_path(first.clone());
        record(&mut app, 7);
        app.output = "later manual draft".into();
        app.accept_location_output_ack(7, applied.clone());
        assert_eq!(app.output, "later manual draft");
        assert!(app.location_output_intent.is_none());
        app.set_output_path(first.clone());
        record(&mut app, 8);
        app.output_original = Some((
            app.output.clone(),
            fixture.0.join("other-original-authority"),
        ));
        let original = app.output_original.clone();
        app.accept_location_output_ack(8, applied.clone());
        assert_eq!(app.output_original, original);
        app.set_output_path(first.clone());
        record(&mut app, 9);
        app.close_wait = true;
        app.accept_location_output_ack(9, applied.clone());
        assert_eq!(app.output_path(), first);
        assert!(app.location_output_intent.is_none());
        app.close_wait = false;
        record(&mut app, 11);
        app.accept_location_output_ack(10, applied.clone());
        assert_eq!(app.location_output_intent.as_ref().unwrap().serial, 11);
        assert_eq!(app.output_path(), first);
        app.accept_location_output_ack(11, applied.clone());
        assert_eq!(app.output_path(), applied);
        assert!(app.location_output_intent.is_none());
        app.accept_location_output_ack(11, first);
        assert_eq!(
            app.output_path(),
            applied,
            "consumed UI intent cannot be replayed"
        );
        assert!(
            !applied.exists(),
            "UI ACK routing creates no filesystem object"
        );
    }

    use super::*;
    use dlpad_core::options::{Quality, SourceUrl};
    static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    #[test]
    fn actual_startup_observed_apply_cannot_mint_output_intent_or_overwrite_later_text() {
        let fixture = Fixture::new();
        let data = fixture.0.join("data");
        let original = fixture.0.join("original-media");
        let restored = fixture.0.join("restored-media");
        for path in [&data, &original, &restored] {
            std::fs::create_dir(path).unwrap();
        }
        let mut app = Candidate::new(data.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        app.library = Some(crate::accessible_library::LibraryPanel::new(
            data.clone(),
            original,
        ));
        let wait_library = |app: &mut Candidate| {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while app.library.as_ref().unwrap().busy() {
                app.library.as_mut().unwrap().poll(&ctx);
                assert!(
                    std::time::Instant::now() < deadline,
                    "owned library worker bounded fixture"
                );
                thread::sleep(Duration::from_millis(5));
            }
        };
        wait_library(&mut app);
        app.output.clear();
        app.output_original = None;
        app.location_output_intent = None;
        let recovered = restored.clone();
        app.spawn(move |_, _| {
            Completion::Recovery(Ok(RecoveryState {
                restored_library_root: Some(recovered),
                ..RecoveryState::default()
            }))
        });
        // Executes the actual production Recovery completion branch, not a fabricated LocationOutputIntent setter.
        wait_finished(&mut app);
        assert_eq!(
            app.output_path(),
            restored,
            "validated startup restoration itself is retained"
        );
        assert!(
            app.library.as_ref().unwrap().busy(),
            "startup worker Apply was genuinely accepted"
        );
        app.capture_location_request();
        assert!(
            app.location_output_intent.is_none(),
            "startup observation must not mint explicit user output intent"
        );
        wait_library(&mut app);
        // The committed Library ACK remains queued, while actual production draft TextEdit changes later.
        let frame = |app: &mut Candidate, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 500.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .show(ctx, |ui| app.show_download_location_draft(ui, ctx));
                },
            )
        };
        let first = frame(&mut app, vec![]);
        let target = egui::accesskit::NodeId(egui::Id::new("download-save-path-input").value());
        assert!(
            first
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(id, node)| *id == target && !node.is_disabled()),
            "actual editable original draft field"
        );
        let _ = frame(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target,
                    data: None,
                },
            )],
        );
        let later = fixture.0.join("later-explicit-draft");
        let text = crate::path_picker::display_path(&later);
        let _ = frame(
            &mut app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Text(text.clone()),
            ],
        );
        assert_eq!(app.output, text, "real Context draft edit was applied");
        let original_authority = app.output_original.clone();
        app.capture_location_request();
        app.apply_location_output_ack();
        assert_eq!(
            app.output, text,
            "committed startup ACK cannot overwrite later user text"
        );
        assert_eq!(app.output_original, original_authority);
        assert!(app.location_output_intent.is_none());
        assert!(
            app.library.as_mut().unwrap().take_applied_root().is_none(),
            "Main consumed observation exactly once without granting draft authority"
        );
        app.library.as_mut().unwrap().request_stop();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !app.library.as_ref().unwrap().stopped() {
            app.library.as_mut().unwrap().poll(&ctx);
            assert!(std::time::Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            for _ in 0..256 {
                let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
                let root = std::env::temp_dir().join(format!(
                    "dlpad-accessible-{}-{}-{sequence}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
                match std::fs::create_dir(&root) {
                    Ok(()) => return Self(root),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("owned fixture creation failed: {error}"),
                }
            }
            panic!("owned fixture identity retry budget exhausted")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn wait_finished(app: &mut Candidate) {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while app.worker.as_ref().is_some_and(|w| !w.handle.is_finished()) {
            assert!(std::time::Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
        app.poll();
        let ctx = egui::Context::default();
        while app.tool_preferences.busy() {
            app.tool_preferences
                .poll(&ctx, [&mut app.tool, &mut app.ffmpeg, &mut app.deno]);
            assert!(std::time::Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn actual_recovered_update_notice_is_global_keyboard_dismissible_and_not_repeated() {
        use egui::accesskit::{Action, ActionRequest, Role};
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let original_status = app.status.clone();
        let mut notice = crate::app_update_client::Snapshot {
            phase: crate::app_update_client::Phase::Failed,
            generation: 1,
            error: Some(13),
            runtime_ready: true,
            ..Default::default()
        };
        app.observe_update_recovery(&notice);
        assert!(app.update_recovery_notice == UpdateRecoveryNotice::Unseen);
        notice.generation = 0;
        app.observe_update_recovery(&notice);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let draw = |app: &mut Candidate, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| app.show_ui(ctx),
            )
        };
        let mut confirm = None;
        for tab in 0..4 {
            app.settings = tab == 3;
            app.library_open = tab == 1;
            app.history_open = tab == 2;
            for _ in 0..3 {
                let out = draw(&mut app, vec![]);
                assert!(out.shapes.iter().any(|shape| matches!(&shape.shape,
                    egui::epaint::Shape::Text(text) if text.galley.text() == c::APP_UPDATE_RECOVERED
                        && shape.clip_rect.contains_rect(text.visual_bounding_rect()))));
                for (id, node) in &out.platform_output.accesskit_update.as_ref().unwrap().nodes {
                    if node.role() == Role::Button
                        && node.label() == Some(c::APP_UPDATE_RECOVERY_CONFIRM)
                    {
                        assert!(node.supports_action(Action::Focus));
                        confirm = Some(*id);
                    }
                }
            }
        }
        draw(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: confirm.expect("actual global confirmation button"),
                data: None,
            })],
        );
        draw(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(app.update_recovery_notice == UpdateRecoveryNotice::Dismissed);
        app.observe_update_recovery(&notice);
        let out = draw(&mut app, vec![]);
        assert!(!out.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::epaint::Shape::Text(text) if text.galley.text() == c::APP_UPDATE_RECOVERED)));
        assert_eq!(
            app.status, original_status,
            "startup recovery status is independent"
        );
        assert!(
            app.app_update_panel.is_none(),
            "global notice keeps update panel lazy"
        );
    }
    #[test]
    fn accepted_update_close_wait_retains_owned_worker_until_ack_and_keeps_panel_lazy() {
        struct Release(Arc<AtomicBool>);
        impl Drop for Release {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.update_frame_ready());
        assert!(app.app_update_panel.is_none());
        let release = Release(Arc::new(AtomicBool::new(false)));
        let worker_release = release.0.clone();
        app.spawn(move |cancel, _| {
            while !cancel.load(Ordering::Acquire) {
                thread::yield_now();
            }
            while !worker_release.load(Ordering::Acquire) {
                thread::yield_now();
            }
            Completion::Recovery(Ok(RecoveryState::default()))
        });
        assert!(!app.update_tasks_idle());
        app.begin_update_close_wait();
        assert!(app.close_wait);
        assert!(!app.can_recover());
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| app.show_ui(ctx));
        assert!(
            app.worker.is_some(),
            "owned worker remains until its completion ACK"
        );
        assert!(
            !output
                .viewport_output
                .values()
                .any(|viewport| viewport.commands.contains(&egui::ViewportCommand::Close))
        );
        assert!(
            app.app_update_panel.is_none(),
            "shutdown does not construct lazy settings UI"
        );
        release.0.store(true, Ordering::Release);
        wait_finished(&mut app);
        assert!(app.worker.is_none());
    }
    #[test]
    fn close_wait_blocks_new_download_even_when_recovery_is_ready() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready);
        app.close_wait = true;
        app.url = "https://fixture.invalid/watch".into();
        app.tool = fixture
            .0
            .join("not-executed-tool")
            .to_string_lossy()
            .into_owned();
        app.output = fixture.0.join("media").to_string_lossy().into_owned();
        app.start(false);
        assert!(app.worker.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[test]
    fn first_stop_intent_is_preserved() {
        for first_pause in [false, true] {
            let (sender, receiver) = mpsc::sync_channel(1);
            let handle = thread::spawn(move || {
                let _ = sender.send(Completion::Recovery(Ok(RecoveryState::default())));
            });
            let worker = Worker {
                handle,
                receiver,
                cancel: Arc::new(AtomicBool::new(false)),
                pause: Arc::new(AtomicBool::new(false)),
            };
            worker.request_stop(first_pause);
            worker.request_stop(!first_pause);
            assert!(worker.cancel.load(Ordering::Acquire));
            assert_eq!(worker.pause.load(Ordering::Acquire), first_pause);
            worker.handle.join().unwrap();
        }
    }
    #[test]
    fn startup_failure_blocks_start_and_retry_recovers() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.0.join("jobs.sqlite")).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(!app.ready);
        assert_eq!(app.status, c::RECOVERY_FAILED);
        app.start(false);
        assert!(app.worker.is_none());
        std::fs::remove_dir(fixture.0.join("jobs.sqlite")).unwrap();
        app.recover();
        wait_finished(&mut app);
        assert!(app.ready);
    }
    #[test]
    fn active_worker_rejects_second_worker_without_ui_join() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let (release, waiting) = mpsc::sync_channel::<()>(1);
        app.spawn(move |_, _| {
            waiting.recv().unwrap();
            Completion::Recovery(Ok(RecoveryState::default()))
        });
        let called = Arc::new(AtomicBool::new(false));
        let cloned = called.clone();
        app.spawn(move |_, _| {
            cloned.store(true, Ordering::Release);
            Completion::Recovery(Ok(RecoveryState::default()))
        });
        app.poll();
        assert!(app.worker.is_some());
        assert!(!called.load(Ordering::Acquire));
        release.send(()).unwrap();
        wait_finished(&mut app);
        assert!(app.worker.is_none());
    }
    #[test]
    fn managed_selection_updates_only_its_tool_and_respects_close_wait() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.tool = "original yt-dlp".into();
        app.deno = "original deno".into();
        app.ffmpeg = "original ffmpeg".into();
        let path = fixture.0.join("한글 관리 tool");
        app.apply_managed_tool(crate::managed::ToolId::Deno, &path);
        assert_eq!(app.deno, path.to_str().unwrap());
        assert_eq!(app.tool, "original yt-dlp");
        assert_eq!(app.ffmpeg, "original ffmpeg");
        app.apply_managed_tool(crate::managed::ToolId::Ffmpeg, &path);
        assert_eq!(app.ffmpeg, path.to_str().unwrap());
        assert_eq!(app.tool, "original yt-dlp");
        app.close_wait = true;
        app.apply_managed_tool(crate::managed::ToolId::YtDlp, &path);
        assert_eq!(app.tool, "original yt-dlp");
    }
    #[test]
    fn managed_worker_confirmed_discovery_failure_clears_pending() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let manager = dlpad_engine::tools::ToolManager::new(&root).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://example.test/synthetic".into();
        app.output = fixture.0.join("media").display().to_string();
        app.ffmpeg.clear();
        app.deno.clear();
        let tool = root.join("missing-managed-tool");
        app.tool = tool.display().to_string();
        let completion = execute(
            fixture.0.clone(),
            tool,
            Some(app.options().unwrap()),
            None,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            app.progress.clone(),
        );
        assert!(matches!(completion, Completion::Job { .. }));
        assert!(!root.join(".execution-pending").exists());
        manager.begin_execution().unwrap().finish().unwrap();
    }
    #[test]
    fn deno_input_reaches_plan_and_invalid_path_never_starts() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://example.org/synthetic".into();
        app.tool = fixture.0.join("yt-dlp").display().to_string();
        app.output = fixture.0.join("media").display().to_string();
        app.ffmpeg.clear();
        let selected = fixture.0.join("한글 Deno 공백");
        app.deno = selected.display().to_string();
        let options = app.options().unwrap();
        assert_eq!(options.deno.as_ref(), Some(&selected));
        let preview = crate::accessible_plan::preview(&options).unwrap();
        assert!(preview.contains("--js-runtimes"));
        app.deno = "relative/deno".into();
        app.start(false);
        assert!(app.worker.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
        app.deno.clear();
        assert!(app.options().unwrap().deno.is_none());
    }
    #[test]
    fn invalid_typed_options_never_start_or_create_journal() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://user:secret@example.test/".into();
        app.tool = "/missing".into();
        app.output = fixture.0.join("media").display().to_string();
        app.start(false);
        assert!(app.worker.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn unsafe_cookie_cache_blocks_startup_without_touching_external_data() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let foreign = fixture.0.join("external-preserved");
        std::fs::create_dir(&foreign).unwrap();
        let sentinel = foreign.join("sentinel");
        std::fs::write(&sentinel, b"owned-test-external-sentinel").unwrap();
        symlink(&foreign, fixture.0.join("session-cookies")).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(!app.ready);
        assert_eq!(app.status, c::COOKIE_RECOVERY_FAILED);
        app.start(false);
        assert!(app.worker.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
        assert_eq!(
            std::fs::read(sentinel).unwrap(),
            b"owned-test-external-sentinel"
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_deferred_cookie_is_preserved_and_startup_reports_pending() {
        let fixture = Fixture::new();
        let source = fixture.0.join("synthetic-source.txt");
        let bytes = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-only\n";
        std::fs::write(&source, bytes).unwrap();
        let cache = fixture.0.join("session-cookies");
        dlpad_engine::cookies::prepare_session_cache(&cache).unwrap();
        let mut lease = dlpad_engine::cookies::import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        lease.defer_cleanup();
        drop(lease);
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready && app.cookie_recovery_pending);
        assert_eq!(std::fs::read(&copy).unwrap(), bytes);
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        assert!(!fixture.0.join("jobs.sqlite").exists());
        assert!(app.cookie_source.is_empty());
    }
    #[cfg(windows)]
    #[test]
    fn windows_non_directory_cookie_cache_blocks_startup_and_preserves_bytes() {
        let fixture = Fixture::new();
        let cache = fixture.0.join("session-cookies");
        std::fs::write(&cache, b"synthetic foreign cache sentinel").unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(!app.ready);
        assert_eq!(app.status, c::COOKIE_RECOVERY_FAILED);
        app.start(false);
        assert!(app.worker.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
        assert_eq!(
            std::fs::read(cache).unwrap(),
            b"synthetic foreign cache sentinel"
        );
    }
    #[test]
    fn cookie_selection_is_consumed_and_invalid_source_never_creates_job() {
        let fixture = Fixture::new();
        let source = fixture.0.join("private-selection.txt");
        let contents = b"invalid-cookie-format-with-synthetic-secret";
        std::fs::write(&source, contents).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://example.test/fixture".into();
        app.tool = fixture.0.join("must-not-run").display().to_string();
        app.output = fixture.0.join("media").display().to_string();
        app.cookie_source = source.display().to_string();
        app.start(false);
        assert!(app.cookie_source.is_empty());
        wait_finished(&mut app);
        assert_eq!(app.status, c::FAILED);
        let report = app.report.as_ref().unwrap().markdown_preview();
        assert!(!report.contains("synthetic-secret"));
        assert!(!report.contains("private-selection.txt"));
        assert!(app.resume.is_none());
        assert!(!fixture.0.join("jobs.sqlite").exists());
        assert_eq!(std::fs::read(&source).unwrap(), contents);
    }
    #[test]
    fn failed_tool_produces_redacted_report_and_no_artifact() {
        let fixture = Fixture::new();
        let data = fixture.0.join("data");
        std::fs::create_dir(&data).unwrap();
        let media = fixture.0.join("media");
        let options = DownloadOptions {
            url: SourceUrl::parse("https://example.test/video?token=secret").unwrap(),
            output_dir: media.clone(),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let result = execute(
            data.clone(),
            fixture.0.join("missing-tool"),
            Some(options),
            None,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            ProgressFeed::default(),
        );
        let Completion::Job {
            status,
            report: Some(report),
            resume,
        } = result
        else {
            panic!("expected diagnostic failure")
        };
        assert_eq!(status, c::FAILED);
        assert!(resume.is_some());
        assert!(!report.markdown_preview().contains("token=secret"));
        let repository = DownloadRepository::open(&data.join("jobs.sqlite")).unwrap();
        let history = repository.history(None).unwrap();
        assert_eq!(history.len(), 1);
        assert!(repository.job(&history[0].id).unwrap().artifacts.is_empty());
    }
    #[cfg(windows)]
    #[test]
    fn windows_startup_without_managed_root_does_not_create_one() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready && !app.managed_recovery_pending);
        assert!(!fixture.0.join("managed-tools").exists());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[cfg(windows)]
    #[test]
    fn windows_live_managed_scope_is_preserved_until_confirmed_finish_and_retry() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let manager = dlpad_engine::tools::ToolManager::new(&root).unwrap();
        let scope = manager.begin_execution().unwrap();
        let marker = root.join(".execution-pending");
        let before = std::fs::read(&marker).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready && app.managed_recovery_pending);
        assert_eq!(std::fs::read(&marker).unwrap(), before);
        assert!(manager.begin_execution().is_err());
        scope.finish().unwrap();
        app.recover();
        wait_finished(&mut app);
        assert!(app.ready && !app.managed_recovery_pending);
        assert!(!marker.exists());
    }
    #[cfg(windows)]
    #[test]
    fn windows_abandoned_pristine_is_pending_and_never_auto_deleted() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let manager = dlpad_engine::tools::ToolManager::new(&root).unwrap();
        drop(manager.begin_execution().unwrap());
        let marker = root.join(".execution-pending");
        let before = std::fs::read(&marker).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(app.ready && app.managed_recovery_pending);
        assert_eq!(std::fs::read(&marker).unwrap(), before);
        assert!(manager.begin_execution().is_err());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[cfg(windows)]
    #[test]
    fn windows_malformed_managed_marker_blocks_startup_and_preserves_bytes() {
        let fixture = Fixture::new();
        let root = fixture.0.join("managed-tools");
        let manager = dlpad_engine::tools::ToolManager::new(&root).unwrap();
        drop(manager.begin_execution().unwrap());
        let marker = root.join(".execution-pending");
        std::fs::write(&marker, b"{partial-owned-record").unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(!app.ready);
        assert_eq!(app.status, c::TOOL_RECOVERY_FAILED);
        app.start(false);
        assert!(app.worker.is_none());
        assert_eq!(std::fs::read(marker).unwrap(), b"{partial-owned-record");
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[test]
    fn close_wait_rejects_recovery_without_starting_another_worker() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let before = app.status.clone();
        assert!(app.can_recover());
        app.close_wait = true;
        assert!(!app.can_recover());
        app.recover();
        assert!(app.worker.is_none());
        assert_eq!(app.status, before);
        assert!(app.ready);
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    #[ignore = "owned private host runs actual Candidate egui pointer state and CPU raster"]
    fn actual_candidate_pointer_hover_click_keyboard_mesh_frames() {
        use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
        use egui_software_backend::{BufferMutRef, ColorFieldOrder, EguiSoftwareRender};
        use std::io::Write;
        struct Harness {
            ctx: egui::Context,
            renderer: EguiSoftwareRender,
            time: f64,
            dir: PathBuf,
            index: usize,
            next_drop: Option<PathBuf>,
            close: bool,
            record: bool,
        }
        impl Harness {
            fn frame(
                &mut self,
                app: &mut Candidate,
                events: Vec<Event>,
            ) -> (egui::FullOutput, Vec<[u8; 4]>) {
                self.time += 1.0 / 60.0;
                let mut input = RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 640.0))),
                    time: Some(self.time),
                    events,
                    focused: true,
                    ..Default::default()
                };
                input
                    .viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .native_pixels_per_point = Some(1.0);
                if let Some(path) = self.next_drop.take() {
                    input.dropped_files.push(egui::DroppedFile {
                        path: Some(path),
                        ..Default::default()
                    });
                }
                if self.close {
                    input
                        .viewports
                        .get_mut(&egui::ViewportId::ROOT)
                        .unwrap()
                        .events
                        .push(egui::ViewportEvent::Close);
                    self.close = false;
                }
                let output = self.ctx.run(input, |ctx| app.show_ui(ctx));
                let jobs = self
                    .ctx
                    .tessellate(output.shapes.clone(), output.pixels_per_point);
                assert!(jobs.len() <= 4096 && jobs.iter().all(|p| p.clip_rect.is_finite()));
                let mut pixels = vec![[0u8; 4]; 960 * 640];
                self.renderer.render(
                    &mut BufferMutRef::new(&mut pixels, 960, 640),
                    &jobs,
                    &output.textures_delta,
                    output.pixels_per_point,
                );
                if self.record {
                    assert!(self.index < 64);
                    let path = self.dir.join(format!("frame-{:02}.rgba", self.index));
                    self.index += 1;
                    let mut f = std::fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(path)
                        .unwrap();
                    for row in pixels.chunks(960) {
                        let raw: Vec<u8> = row.iter().flat_map(|p| p.iter().copied()).collect();
                        f.write_all(&raw).unwrap();
                    }
                    f.sync_all().unwrap();
                }
                (output, pixels)
            }
            fn button(output: &egui::FullOutput, label: &str) -> Rect {
                let update = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .expect("actual accessible tree");
                let matches: Vec<_> = update
                    .nodes
                    .iter()
                    .filter(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
                    })
                    .collect();
                assert_eq!(matches.len(), 1, "actual unique typed button {label}");
                let b = matches[0].1.bounds().expect("actual bounds");
                Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                )
            }
        }
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let dir = fixture.0.join("raw-input-cpu-frames");
        std::fs::create_dir(&dir).unwrap();
        std::mem::forget(fixture);
        println!("owned_rgba_directory={}", dir.display());
        let mut h = Harness {
            ctx: egui::Context::default(),
            renderer: EguiSoftwareRender::new(ColorFieldOrder::Rgba).with_caching(false),
            time: 0.0,
            dir: dir.clone(),
            index: 0,
            next_drop: None,
            close: false,
            record: true,
        };
        configure_context(&h.ctx, font_bytes());
        h.ctx.enable_accesskit();
        // Establish real layout/hit-test data before supplying pointer input.
        for _ in 0..3 {
            h.frame(&mut app, vec![]);
        }
        let (idle, idle_pixels) = h.frame(&mut app, vec![]);
        let settings = Harness::button(&idle, c::SETTINGS);
        let point = settings.center();
        let (_, hover_pixels) = h.frame(&mut app, vec![Event::PointerMoved(point)]);
        assert_eq!(h.ctx.pointer_hover_pos(), Some(point));
        assert_ne!(hover_pixels, idle_pixels, "actual hover raster changes");
        assert!(!app.settings);
        h.frame(
            &mut app,
            vec![Event::PointerButton {
                pos: point,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
        );
        assert!(!app.settings, "down alone does not select");
        h.frame(
            &mut app,
            vec![Event::PointerButton {
                pos: point,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert!(
            app.settings,
            "real pointer release selects actual Candidate settings"
        );
        let (output, _) = h.frame(&mut app, vec![]);
        let download = Harness::button(&output, c::DOWNLOADS);
        let p = download.center();
        h.frame(
            &mut app,
            vec![
                Event::PointerMoved(p),
                Event::PointerButton {
                    pos: p,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        h.frame(
            &mut app,
            vec![Event::PointerButton {
                pos: p,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert!(!app.settings && !app.library_open && !app.tools_open && !app.history_open);
        // Actual egui Tab event must change actual focused widget; no direct state assignment.
        let old = h.ctx.memory(|m| m.focused());
        h.frame(
            &mut app,
            vec![
                Event::Key {
                    key: Key::Tab,
                    physical_key: Some(Key::Tab),
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
                Event::Key {
                    key: Key::Tab,
                    physical_key: Some(Key::Tab),
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        let new = h.ctx.memory(|m| m.focused());
        assert!(new.is_some() && new != old);
        if let Some(path) = std::env::var_os("DLPAD_MEDIA_FIXTURE") {
            let path = PathBuf::from(path);
            let original = std::fs::read(&path).unwrap();
            h.next_drop = Some(path.clone());
            h.frame(&mut app, vec![]);
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            h.record = false;
            while app.media.busy() {
                assert!(std::time::Instant::now() < deadline);
                thread::sleep(Duration::from_millis(20));
                h.frame(&mut app, vec![]);
            }
            h.record = true;
            assert!(app.media.local());
            assert!(app.media.selection().is_some());
            let (prepared, _) = h.frame(&mut app, vec![]);
            let play = Harness::button(&prepared, c::MEDIA_PLAY);
            let p = play.center();
            h.frame(
                &mut app,
                vec![
                    Event::PointerMoved(p),
                    Event::PointerButton {
                        pos: p,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
            h.frame(
                &mut app,
                vec![Event::PointerButton {
                    pos: p,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            h.record = false;
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while app.media.busy() {
                assert!(std::time::Instant::now() < deadline);
                thread::sleep(Duration::from_millis(20));
                h.frame(&mut app, vec![]);
            }
            h.record = true;
            h.frame(&mut app, vec![]);
            thread::sleep(Duration::from_millis(1150));
            let (_, pixels) = h.frame(&mut app, vec![]);
            assert!(app.media.selection().unwrap().playhead_ms() >= 1000);
            assert!(
                pixels
                    .iter()
                    .any(|pixel| pixel[2] > 200 && pixel[0] < 40 && pixel[1] < 40),
                "actual temporal preview becomes blue"
            );
            h.close = true;
            let output = h.frame(&mut app, vec![]).0;
            assert!(app.close_wait);
            if !app.media.stopped() {
                assert!(
                    output
                        .viewport_output
                        .values()
                        .all(|o| !o.commands.contains(&egui::ViewportCommand::Close))
                );
            }
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            h.record = false;
            while !app.media.stopped() {
                assert!(std::time::Instant::now() < deadline);
                thread::sleep(Duration::from_millis(10));
                h.frame(&mut app, vec![]);
            }
            assert_eq!(std::fs::read(path).unwrap(), original);
            h.record = true;
            h.frame(&mut app, vec![]);
        }
        // Render smaller actual viewport is a follow-up parameter; root must preserve
        // the real footer/clipping rect evidence rather than assume no overlap.
        println!(
            "{{\"phase\":\"actual-candidate-rawinput-cpu-frames\",\"rgba_frames\":{},\"width\":960,\"height\":640,\"physical_OS_pointer_verified\":false,\"native_window_event_route_verified\":false}}",
            h.index
        );
        drop(app);
    }
    // Insert inside accessible.rs existing tests module. No production changes.
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    #[ignore = "actual owned synthetic media/CPU raster; explicit private host run required"]
    fn actual_local_ready_geometry_and_change_source_at_three_scales() {
        use egui::{Event, Modifiers, PointerButton, RawInput, Rect};
        use egui_software_backend::{BufferMutRef, ColorFieldOrder, EguiSoftwareRender};
        use std::io::Write;
        fn frame(
            ctx: &egui::Context,
            app: &mut Candidate,
            scale: f32,
            events: Vec<Event>,
        ) -> egui::FullOutput {
            let mut input = RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(960.0 / scale, 640.0 / scale),
                )),
                focused: true,
                events,
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(scale);
            ctx.run(input, |ctx| app.show_ui(ctx))
        }
        fn bounds(
            nodes: &std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
            label: &str,
        ) -> Rect {
            let matches: Vec<_> = nodes
                .values()
                .filter(|n| {
                    n.label() == Some(label)
                        && n.bounds().is_some()
                        && n.role() != egui::accesskit::Role::TextInput
                })
                .collect();
            assert_eq!(matches.len(), 1, "unique actual node {label}");
            let b = matches[0].bounds().unwrap();
            Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        }
        let original_path = PathBuf::from(
            std::env::var_os("DLPAD_MEDIA_FIXTURE")
                .expect("owned pinned 2s synthetic fixture required"),
        );
        assert!(std::fs::metadata(&original_path).unwrap().len() <= 256 * 1024 * 1024);
        let original = std::fs::read(&original_path).unwrap();
        let ffmpeg =
            PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").expect("pinned FFmpeg required"));
        for scale in [1.0, 1.5, 2.0] {
            let fixture = std::mem::ManuallyDrop::new(Fixture::new());
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.ffmpeg = ffmpeg.to_string_lossy().into_owned();
            let media = fixture.0.join("synthetic-source.mp4");
            std::fs::write(&media, &original).unwrap();
            app.media.dropped(media.clone());
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            let mut renderer = EguiSoftwareRender::new(ColorFieldOrder::Rgba).with_caching(false);
            let mut nodes = std::collections::HashMap::new();
            let deadline = std::time::Instant::now() + Duration::from_secs(20);
            let mut stable = 0;
            let output = loop {
                assert!(
                    std::time::Instant::now() < deadline,
                    "actual preview preparation timed out"
                );
                let out = frame(&ctx, &mut app, scale, vec![]);
                if let Some(update) = &out.platform_output.accesskit_update {
                    for (id, node) in &update.nodes {
                        nodes.insert(*id, node.clone());
                    }
                }
                // Consume every texture delta: renderer state must correspond to actual show_ui.
                let jobs = ctx.tessellate(out.shapes.clone(), out.pixels_per_point);
                let mut pixels = vec![[0u8; 4]; 960 * 640];
                renderer.render(
                    &mut BufferMutRef::new(&mut pixels, 960, 640),
                    &jobs,
                    &out.textures_delta,
                    out.pixels_per_point,
                );
                if app.media.local() && !app.media.busy() {
                    stable += 1;
                } else {
                    stable = 0;
                }
                if stable >= 3 {
                    let path = fixture.0.join(format!("local-ready-scale-{scale}.rgba"));
                    let mut file = std::fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(path)
                        .unwrap();
                    for row in pixels.chunks(960) {
                        let raw: Vec<u8> = row.iter().flat_map(|p| p.iter().copied()).collect();
                        file.write_all(&raw).unwrap();
                    }
                    file.sync_all().unwrap();
                    break out;
                }
                thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(output.pixels_per_point, scale);
            let change = bounds(&nodes, c::MEDIA_CHANGE_SOURCE);
            let footer = bounds(&nodes, c::LOCAL_EXPORT_SAVE_SELECTION);
            let start = bounds(&nodes, c::RANGE_START);
            let end = bounds(&nodes, c::RANGE_END);
            let viewport =
                Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(960.0 / scale, 640.0 / scale));
            assert!(viewport.contains_rect(change) && viewport.contains_rect(footer));
            // Raw layout bounds may lie beneath the fixed footer. Drag there must
            // not reach the scroll-clipped timeline; release outside avoids footer clicks.
            for rect in [start, end] {
                let occluded = rect.intersect(footer);
                if occluded.is_positive() {
                    let before = app.media.selection().unwrap();
                    let p = occluded.center();
                    frame(
                        &ctx,
                        &mut app,
                        scale,
                        vec![
                            Event::PointerMoved(p),
                            Event::PointerButton {
                                pos: p,
                                button: PointerButton::Primary,
                                pressed: true,
                                modifiers: Modifiers::NONE,
                            },
                        ],
                    );
                    frame(
                        &ctx,
                        &mut app,
                        scale,
                        vec![Event::PointerMoved(p + egui::vec2(80.0, 0.0))],
                    );
                    frame(
                        &ctx,
                        &mut app,
                        scale,
                        vec![Event::PointerButton {
                            pos: egui::pos2(-10.0, -10.0),
                            button: PointerButton::Primary,
                            pressed: false,
                            modifiers: Modifiers::NONE,
                        }],
                    );
                    let after = app.media.selection().unwrap();
                    assert_eq!(
                        (before.start_ms(), before.end_ms()),
                        (after.start_ms(), after.end_ms()),
                        "clipped endpoint must not receive footer-area drag"
                    );
                }
            }
            for label in [c::RANGE_START, c::RANGE_END] {
                let thumb_id = *nodes
                    .iter()
                    .find(|(_, n)| {
                        n.role() == egui::accesskit::Role::Slider && n.label() == Some(label)
                    })
                    .expect("actual endpoint slider")
                    .0;
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![Event::AccessKitActionRequest(
                        egui::accesskit::ActionRequest {
                            action: egui::accesskit::Action::Focus,
                            target: thumb_id,
                            data: None,
                        },
                    )],
                );
                let mut revealed = false;
                for _ in 0..24 {
                    let out = frame(&ctx, &mut app, scale, vec![]);
                    if let Some(update) = out.platform_output.accesskit_update {
                        for (id, node) in update.nodes {
                            nodes.insert(id, node);
                        }
                    }
                    let rect = bounds(&nodes, label);
                    if viewport.contains_rect(rect) && !rect.intersects(footer) {
                        revealed = true;
                        break;
                    }
                    let out = frame(
                        &ctx,
                        &mut app,
                        scale,
                        vec![
                            Event::PointerMoved(egui::pos2(
                                viewport.center().x,
                                footer.top() - 40.0,
                            )),
                            Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(0.0, -40.0),
                                modifiers: Modifiers::NONE,
                            },
                        ],
                    );
                    if let Some(update) = out.platform_output.accesskit_update {
                        for (id, node) in update.nodes {
                            nodes.insert(id, node);
                        }
                    }
                }
                assert!(
                    revealed,
                    "actual focus/scroll reveals {label} endpoint at {scale}"
                );
            }
            // Stable IDs are product input identities; inspect actual AccessKit nodes,
            // not mirrored allocation math or labels shared with timeline thumbs.
            let mut input_rects = Vec::new();
            for label in [c::RANGE_START, c::RANGE_END] {
                let id = egui::Id::new(("download-option-input", label));
                let node = nodes
                    .get(&egui::accesskit::NodeId(id.value()))
                    .expect("actual numeric input node");
                assert_eq!(node.role(), egui::accesskit::Role::TextInput);
                assert!(node.supports_action(egui::accesskit::Action::Focus));
                let b = node.bounds().expect("numeric input bounds");
                let rect = Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                // Raw bounds need not be visible until the real focus/scroll route.
                assert!(rect.is_finite() && rect.is_positive());
                input_rects.push((id, rect));
            }
            assert!(!input_rects[0].1.intersects(input_rects[1].1));
            for ((id, rect), value) in input_rects.iter().zip(["00:00:00.200", "00:00:01.800"]) {
                let _raw_rect = rect;
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![Event::AccessKitActionRequest(
                        egui::accesskit::ActionRequest {
                            action: egui::accesskit::Action::Focus,
                            target: egui::accesskit::NodeId(id.value()),
                            data: None,
                        },
                    )],
                );
                // A human waits for focus-triggered scrolling to settle before a
                // pointer click. Keep using real frames/time rather than mutating
                // ScrollArea state or clicking a transient animation snapshot.
                for _ in 0..25 {
                    frame(&ctx, &mut app, scale, vec![]);
                    thread::sleep(Duration::from_millis(20));
                }
                let mut visible = None;
                for _ in 0..24 {
                    frame(&ctx, &mut app, scale, vec![]);
                    let response = ctx.read_response(*id).expect("actual numeric Response");
                    let hit = response.interact_rect;
                    if hit.contains(response.rect.center())
                        && viewport.contains_rect(hit)
                        && !hit.intersects(footer)
                    {
                        visible = Some(hit);
                        break;
                    }
                    // Actual wheel input over central content; no direct ScrollArea mutation.
                    frame(
                        &ctx,
                        &mut app,
                        scale,
                        vec![
                            Event::PointerMoved(egui::pos2(
                                viewport.center().x,
                                footer.top() - 40.0,
                            )),
                            Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(0.0, -40.0),
                                modifiers: Modifiers::NONE,
                            },
                        ],
                    );
                }
                visible.expect("focus + bounded real wheel must reveal numeric field");
                // The final real wheel event may still have smooth-scroll distance
                // pending after the field first enters view. Refresh its actual hit
                // rectangle after that motion settles instead of using stale pixels.
                for _ in 0..25 {
                    frame(&ctx, &mut app, scale, vec![]);
                    thread::sleep(Duration::from_millis(20));
                }
                let response = ctx.read_response(*id).unwrap();
                let hit = response.interact_rect;
                assert!(
                    hit.contains(response.rect.center())
                        && viewport.contains_rect(hit)
                        && !hit.intersects(footer)
                );
                let p = hit.center();
                let actual = ctx.read_response(*id).unwrap();
                println!(
                    "numeric_before_click scale={scale} id={id:?} hit={hit:?} rect={:?} enabled={} focus={:?} point={p:?} media_busy={}",
                    actual.rect,
                    actual.enabled(),
                    ctx.memory(|m| m.focused()),
                    app.media.busy()
                );
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![
                        Event::PointerMoved(p),
                        Event::PointerButton {
                            pos: p,
                            button: PointerButton::Primary,
                            pressed: true,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![Event::PointerButton {
                        pos: p,
                        button: PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    }],
                );
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    Some(*id),
                    "actual numeric field pointer focus at scale {scale} id {id:?} hit {hit:?} response {:?}",
                    ctx.read_response(*id)
                );
                let select_all = Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                };
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![
                        Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: select_all,
                        },
                        Event::Text(value.into()),
                    ],
                );
                frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: select_all,
                    }],
                );
            }
            assert_eq!(
                app.form
                    .media_range(app.media.selection().unwrap().duration_ms())
                    .unwrap(),
                (200, 1800)
            );
            let selection = app.media.selection().unwrap();
            assert_eq!(
                (selection.start_ms(), selection.end_ms()),
                (200, 1800),
                "actual typed numeric inputs synchronize production selection"
            );
            for _ in 0..24 {
                let out = frame(
                    &ctx,
                    &mut app,
                    scale,
                    vec![
                        Event::PointerMoved(egui::pos2(viewport.center().x, footer.top() - 40.0)),
                        Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, 120.0),
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
                if let Some(update) = out.platform_output.accesskit_update {
                    for (id, node) in update.nodes {
                        nodes.insert(id, node);
                    }
                }
            }
            for _ in 0..25 {
                let out = frame(&ctx, &mut app, scale, vec![]);
                if let Some(update) = out.platform_output.accesskit_update {
                    for (id, node) in update.nodes {
                        nodes.insert(id, node);
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
            let change = bounds(&nodes, c::MEDIA_CHANGE_SOURCE);
            assert!(viewport.contains_rect(change) && !change.intersects(footer));
            let p = change.center();
            frame(
                &ctx,
                &mut app,
                scale,
                vec![
                    Event::PointerMoved(p),
                    Event::PointerButton {
                        pos: p,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
            frame(
                &ctx,
                &mut app,
                scale,
                vec![Event::PointerButton {
                    pos: p,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            assert!(!app.media.local());
            assert!(app.media.selection().is_none());
            assert!(app.media.prepared_for_export().is_none());
            assert!(app.url.is_empty());
            assert!(!app.media.busy());
            assert_eq!(std::fs::read(&media).unwrap(), original);
            app.media.request_stop();
            let stop_deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !app.media.stopped() {
                assert!(std::time::Instant::now() < stop_deadline);
                frame(&ctx, &mut app, scale, vec![]);
                thread::sleep(Duration::from_millis(10));
            }
            println!("owned_local_ready_evidence={}", fixture.0.display());
            drop(app);
            // ManuallyDrop retains owned evidence even on an assertion failure.
        }
        assert_eq!(std::fs::read(original_path).unwrap(), original);
    }
    // Insert into accessible.rs tests module. Draft; root must compile/integrate.
    #[test]
    fn actual_candidate_diagnostic_field_reveal_paints_full_input_in_parent_clip() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        fn render(
            app: &mut Candidate,
            ctx: &egui::Context,
            time: f64,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 640.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| app.show_ui(ctx),
            )
        }
        fn request(target: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                target,
                action,
                data: None,
            })
        }
        fn store(nodes: &mut HashMap<NodeId, Node>, output: &egui::FullOutput) {
            if let Some(update) = &output.platform_output.accesskit_update {
                for (id, node) in &update.nodes {
                    nodes.insert(*id, node.clone());
                }
            }
        }
        fn exact(nodes: &HashMap<NodeId, Node>, label: &str, input: bool) -> NodeId {
            let matches: Vec<_> = nodes
                .iter()
                .filter(|(_, n)| {
                    (n.label() == Some(label)
                        || n.labelled_by().iter().any(|id| {
                            nodes.get(id).is_some_and(|node| {
                                node.label() == Some(label) || node.value() == Some(label)
                            })
                        }))
                        && (!input || n.role() == Role::TextInput)
                })
                .map(|(id, _)| *id)
                .collect();
            assert_eq!(matches.len(), 1, "actual unique accessible target {label}");
            matches[0]
        }
        fn visible_glyph(shape: &egui::epaint::Shape, clip: egui::Rect, field: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Text(text) => {
                    let glyph = text.galley.rect.translate(text.pos.to_vec2());
                    !text.galley.text().is_empty()
                        && field.contains_rect(glyph)
                        && clip.contains_rect(glyph)
                }
                egui::epaint::Shape::Vec(items) => {
                    items.iter().any(|s| visible_glyph(s, clip, field))
                }
                _ => false,
            }
        }
        fn full_field_background(shape: &egui::epaint::Shape, field: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Rect(background) => {
                    (background.rect.min - field.min).length() < 0.5
                        && (background.rect.max - field.max).length() < 0.5
                }
                egui::epaint::Shape::Vec(items) => {
                    items.iter().any(|s| full_field_background(s, field))
                }
                _ => false,
            }
        }
        for action in [Action::Focus, Action::ScrollIntoView] {
            for label in [c::LOG_INDEX, c::MPD_PATH, c::EXPORT_PATH] {
                let fixture = Fixture::new();
                let mut app = Candidate::new(fixture.0.clone());
                wait_finished(&mut app);
                app.diagnostics = Some(crate::accessible_diagnostics::DiagnosticPanel::new(
                    Box::new(dlpad_engine::reports::demo_report()),
                ));
                let ctx = egui::Context::default();
                configure_context(&ctx, font_bytes());
                ctx.enable_accesskit();
                let mut nodes = HashMap::new();
                let mut tick = 0;
                let initial = render(&mut app, &ctx, 0.0, vec![]);
                store(&mut nodes, &initial);
                let environment = exact(&nodes, c::DOWNLOAD_ENVIRONMENT, false);
                let out = render(
                    &mut app,
                    &ctx,
                    0.05,
                    vec![request(environment, Action::Expand)],
                );
                store(&mut nodes, &out);
                for n in 0..20 {
                    store(
                        &mut nodes,
                        &render(&mut app, &ctx, 0.06 + n as f64 * 0.001, vec![]),
                    );
                }
                let header = exact(&nodes, c::DIAGNOSTICS, false);
                let out = render(
                    &mut app,
                    &ctx,
                    0.1,
                    vec![
                        request(header, Action::Focus),
                        request(header, Action::Expand),
                    ],
                );
                store(&mut nodes, &out);
                for _ in 0..20 {
                    tick += 1;
                    store(
                        &mut nodes,
                        &render(&mut app, &ctx, tick as f64 * 0.1 + 0.2, vec![]),
                    );
                }
                if label == c::MPD_PATH {
                    let mpd_header = exact(&nodes, c::DIAGNOSTIC_MPD_DETAILS, false);
                    let out = render(
                        &mut app,
                        &ctx,
                        2.25,
                        vec![request(mpd_header, Action::Expand)],
                    );
                    store(&mut nodes, &out);
                    for n in 0..20 {
                        store(
                            &mut nodes,
                            &render(&mut app, &ctx, 2.26 + n as f64 * 0.02, vec![]),
                        );
                    }
                }
                let id = exact(&nodes, label, true);
                let out = render(&mut app, &ctx, 3.0, vec![request(id, action)]);
                store(&mut nodes, &out);
                // Settle scroll animation, without reissuing Focus or scroll every frame.
                for n in 0..20 {
                    let settled = render(&mut app, &ctx, 3.1 + n as f64 * 0.1, vec![]);
                    store(&mut nodes, &settled);
                }
                let b = nodes[&id].bounds().expect("actual input bounds");
                let rect = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                assert!(
                    (rect.height() - 32.0).abs() < 0.5,
                    "full actual input height"
                );
                let content =
                    egui::Rect::from_min_max(egui::pos2(0.0, 48.0), egui::pos2(960.0, 544.0));
                assert!(
                    content.contains_rect(rect),
                    "{label} {action:?} fully inside visible content, not 3px sliver: {rect:?}"
                );
                // Focus + type produces actual glyphs; require their painter clip to
                // contain the entire input, not merely its AccessKit layout bounds.
                let painted = render(
                    &mut app,
                    &ctx,
                    5.2,
                    vec![request(id, Action::Focus), egui::Event::Text("Q".into())],
                );
                store(&mut nodes, &painted);
                let b = nodes[&id].bounds().expect("painted input bounds");
                let rect = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                assert!(content.contains_rect(rect), "painted input fully visible");
                assert!(
                    painted
                        .shapes
                        .iter()
                        .any(|s| visible_glyph(&s.shape, s.clip_rect, rect)),
                    "actual input glyph is fully unclipped: {label} {action:?} {rect:?}"
                );
                assert!(
                    painted.shapes.iter().any(|s| {
                        s.clip_rect.contains_rect(rect) && full_field_background(&s.shape, rect)
                    }),
                    "actual 32px input background fully unclipped: {label} {action:?}"
                );
                app.diagnostics.as_mut().unwrap().request_stop();
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                while !app.diagnostics.as_ref().unwrap().stopped() {
                    assert!(std::time::Instant::now() < deadline);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
    #[test]
    fn actual_readonly_plan_scroll_action_reveals_bounded_painted_viewport() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://fixture.invalid/watch".into();
        app.tool = fixture
            .0
            .join("synthetic-tool.exe")
            .to_string_lossy()
            .into_owned();
        app.set_output_path(fixture.0.clone());
        assert!(
            app.options().is_ok(),
            "valid synthetic plan: {:?}",
            app.options()
        );
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let draw = |app: &mut Candidate, time, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 640.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        // Real show_plan, below the same bounded parent viewport. No fake
                        // AccessKit bounds or direct scroll-offset mutation.
                        egui::ScrollArea::vertical()
                            .id_salt("download-content")
                            .max_height(496.0)
                            .show(ui, |ui| {
                                ui.add_space(650.0);
                                app.show_plan(ui);
                            });
                    });
                },
            )
        };
        let output = draw(&mut app, 0.0, vec![]);
        let nodes: std::collections::HashMap<_, _> = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .collect();
        let matches: Vec<_> = nodes
            .iter()
            .filter(|(_, n)| {
                (n.label() == Some(c::PLAN_PREVIEW_CONTENT)
                    || n.labelled_by().iter().any(|id| {
                        nodes.get(id).is_some_and(|label| {
                            label.label() == Some(c::PLAN_PREVIEW_CONTENT)
                                || label.value() == Some(c::PLAN_PREVIEW_CONTENT)
                        })
                    }))
                    && n.role() == egui::accesskit::Role::MultilineTextInput
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "plan nodes: {:?}",
            nodes
                .iter()
                .map(|(id, n)| (id, n.role(), n.label(), n.value(), n.labelled_by()))
                .collect::<Vec<_>>()
        );
        let id = *matches[0].0;
        assert!(
            matches[0]
                .1
                .supports_action(egui::accesskit::Action::ScrollIntoView)
        );
        assert!(
            !matches[0].1.supports_action(egui::accesskit::Action::Focus),
            "readonly plan is not a keyboard input"
        );
        let before = app.plan_preview.clone();
        let action = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::ScrollIntoView,
            target: id,
            data: None,
        });
        let mut settled = draw(&mut app, 0.1, vec![action]);
        for frame in 0..20 {
            settled = draw(&mut app, 0.2 + f64::from(frame) * 0.1, vec![]);
        }
        assert_eq!(
            app.plan_preview, before,
            "scroll action cannot edit readonly argv"
        );
        assert!(
            ctx.memory(|m| m.focused()).is_none(),
            "scroll action does not pretend keyboard focus"
        );
        let visible = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(960.0, 512.0));
        assert!(
            settled.shapes.iter().any(|shape| {
                if let egui::epaint::Shape::Text(text) = &shape.shape {
                    let ink = text.visual_bounding_rect();
                    text.galley.text().contains("--ignore-config")
                        && ink.is_positive()
                        && shape.clip_rect.height() >= 120.0
                        && visible.contains_rect(shape.clip_rect)
                        && shape.clip_rect.intersect(ink).is_positive()
                } else {
                    false
                }
            }),
            "actual plan glyphs paint in fully visible >=120px bounded viewport"
        );
    }
    #[test]
    fn actual_candidate_recovery_failure_still_opens_history_and_allows_read_retry() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        let fixture = Fixture::new();
        let original = b"owned corrupt startup journal";
        std::fs::write(fixture.0.join("jobs.sqlite"), original).unwrap();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        assert!(!app.ready && app.worker.is_none());
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut nodes = HashMap::<NodeId, Node>::new();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, events: Vec<egui::Event>| {
            time += 0.016;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 640.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| app.show_ui(ctx),
            );
            output
                .platform_output
                .accesskit_update
                .expect("actual candidate tree")
        };
        for (id, node) in frame(&mut app, vec![]).nodes {
            nodes.insert(id, node);
        }
        let history_id = *nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some(c::HISTORY))
            .unwrap()
            .0;
        let click = |target| {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target,
                data: None,
            })
        };
        for (id, node) in frame(&mut app, vec![click(history_id)]).nodes {
            nodes.insert(id, node);
        }
        assert!(
            app.history_open,
            "enabled History navigation must work when recovery fails"
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while app.history.as_ref().is_some_and(|history| history.busy()) {
            assert!(std::time::Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
            for (id, node) in frame(&mut app, vec![]).nodes {
                nodes.insert(id, node);
            }
        }
        for (id, node) in frame(&mut app, vec![]).nodes {
            nodes.insert(id, node);
        }
        assert!(nodes.values().any(|node| node.role() == Role::Label
            && (node.label() == Some(c::HISTORY_FAILED)
                || node.value() == Some(c::HISTORY_FAILED))));
        assert!(!nodes.values().any(|node| node.role() == Role::Label
            && (node.label() == Some(c::HISTORY_EMPTY) || node.value() == Some(c::HISTORY_EMPTY))));
        let (refresh_id, refresh) = nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Button && node.label() == Some(c::HISTORY_REFRESH)
            })
            .unwrap();
        assert!(
            !refresh.is_disabled(),
            "read retry is independent of execution readiness"
        );
        let refresh_id = *refresh_id;
        frame(&mut app, vec![click(refresh_id)]);
        assert!(
            app.history.as_ref().unwrap().busy(),
            "actual refresh schedules bounded read worker"
        );
        assert!(!app.ready && app.worker.is_none() && app.resume.is_none());
        app.history_open = false;
        app.close_wait = true;
        frame(&mut app, vec![click(history_id)]);
        assert!(!app.history_open, "close wait still blocks navigation");
        assert_eq!(
            std::fs::read(fixture.0.join("jobs.sqlite")).unwrap(),
            original
        );
    }
    #[test]
    fn actual_candidate_scale_selection_paints_full_control_above_real_wrapping_footer() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::HashMap;
        fn request(id: NodeId) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target: id,
                data: None,
            })
        }
        fn background(shape: &egui::epaint::Shape, rect: egui::Rect) -> bool {
            match shape {
                egui::epaint::Shape::Rect(s) => {
                    (s.rect.min - rect.min).length() < 0.5 && (s.rect.max - rect.max).length() < 0.5
                }
                egui::epaint::Shape::Vec(shapes) => shapes.iter().any(|s| background(s, rect)),
                _ => false,
            }
        }
        for (physical_width, physical_height) in [(800.0, 600.0), (960.0, 640.0)] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.settings = true;
            app.status = c::RECOVERY_DONE.into();
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            let mut nodes = HashMap::<NodeId, Node>::new();
            let draw = |app: &mut Candidate, frame: u32, events| {
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(
                                physical_width / ctx.zoom_factor(),
                                physical_height / ctx.zoom_factor(),
                            ),
                        )),
                        focused: true,
                        time: Some(f64::from(frame) * 0.05),
                        events,
                        ..Default::default()
                    },
                    |ctx| app.show_ui(ctx),
                )
            };
            let remember = |nodes: &mut HashMap<NodeId, Node>, out: &egui::FullOutput| {
                for (id, node) in &out.platform_output.accesskit_update.as_ref().unwrap().nodes {
                    nodes.insert(*id, node.clone());
                }
            };
            for frame in 0..3 {
                remember(&mut nodes, &draw(&mut app, frame, vec![]));
            }
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
            remember(&mut nodes, &draw(&mut app, 3, vec![request(combo)]));
            let choice = *nodes
                .iter()
                .find(|(_, n)| {
                    n.supports_action(Action::Click)
                        && (n.label() == Some("200%") || n.value() == Some("200%"))
                })
                .expect("real popup200 choice")
                .0;
            remember(&mut nodes, &draw(&mut app, 4, vec![request(choice)]));
            let mut last = None;
            for frame in 5..30 {
                let out = draw(&mut app, frame, vec![]);
                remember(&mut nodes, &out);
                last = Some(out);
            }
            assert!(
                (ctx.zoom_factor() - 2.0).abs() < 0.01,
                "actual scale selection"
            );
            let b = nodes[&combo].bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            let out = last.unwrap();
            let clips: Vec<_> = out
                .shapes
                .iter()
                .filter(|s| background(&s.shape, rect))
                .map(|s| s.clip_rect)
                .collect();
            assert!(
                !clips.is_empty(),
                "actual Scale combo painted background found"
            );
            assert!(
                clips.iter().any(|clip| clip.contains_rect(rect)),
                "actual{physical_width}x{physical_height} Scale selected control {rect:?} clipped by real Candidate footer: {clips:?}"
            );
            for text in [c::SCALE, c::SCALE_HELP] {
                let b = nodes
                    .values()
                    .find(|n| {
                        n.role() == Role::Label
                            && (n.label() == Some(text) || n.value() == Some(text))
                    })
                    .unwrap()
                    .bounds()
                    .unwrap();
                let context = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                assert!(
                    clips.iter().any(|clip| clip.contains_rect(context)),
                    "selected Scale context {text} fully inside actual body painter clip"
                );
            }
            assert_eq!(app.status, c::RECOVERY_DONE);
        }
    }
    /// Uses Candidate's actual status panel and ScrollArea, not a fabricated clip.
    struct LateScaleHarness {
        fixture: Fixture,
        app: Candidate,
        ctx: egui::Context,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        frame: u32,
        focused: bool,
        combo: Option<egui::accesskit::NodeId>,
    }
    #[derive(Clone, Debug)]
    struct LateScaleSnapshot {
        control: egui::Rect,
        label: egui::Rect,
        help: egui::Rect,
        clips: Vec<egui::Rect>,
        footer: egui::Rect,
    }
    impl LateScaleSnapshot {
        fn full_context(&self) -> bool {
            self.clips.iter().any(|clip| {
                clip.contains_rect(self.control)
                    && clip.contains_rect(self.label)
                    && clip.contains_rect(self.help)
            })
        }
    }
    impl LateScaleHarness {
        fn request(id: egui::accesskit::NodeId) -> egui::Event {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target: id,
                data: None,
            })
        }
        fn new(dark: bool, focused: bool) -> Self {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            // Remove external metrics IO timing from this source experiment. The
            // actual production status composition still paints its unknown text.
            app.metrics.request_stop();
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !app.metrics.stopped() {
                assert!(std::time::Instant::now() < deadline);
                thread::sleep(Duration::from_millis(5));
            }
            app.settings = true;
            app.status.clear();
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            ctx.enable_accesskit();
            Self {
                fixture,
                app,
                ctx,
                nodes: Default::default(),
                frame: 0,
                focused,
                combo: None,
            }
        }
        fn draw(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.frame += 1;
            let out = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(
                            960.0 / self.ctx.zoom_factor(),
                            640.0 / self.ctx.zoom_factor(),
                        ),
                    )),
                    focused: self.focused,
                    time: Some(f64::from(self.frame) * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| self.app.show_ui(ctx),
            );
            for (id, node) in &out.platform_output.accesskit_update.as_ref().unwrap().nodes {
                self.nodes.insert(*id, node.clone());
            }
            out
        }
        fn text_rect(&self, text: &str) -> egui::Rect {
            let b = self
                .nodes
                .values()
                .find(|node| {
                    node.role() == egui::accesskit::Role::Label
                        && (node.label() == Some(text) || node.value() == Some(text))
                })
                .unwrap()
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        }
        fn snapshot(&self, out: &egui::FullOutput) -> LateScaleSnapshot {
            fn background(shape: &egui::Shape, rect: egui::Rect) -> bool {
                match shape {
                    egui::Shape::Rect(s) => {
                        (s.rect.min - rect.min).length() < 0.5
                            && (s.rect.max - rect.max).length() < 0.5
                    }
                    egui::Shape::Vec(shapes) => shapes.iter().any(|s| background(s, rect)),
                    _ => false,
                }
            }
            let b = self.nodes[&self.combo.unwrap()].bounds().unwrap();
            let control = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            let clips: Vec<_> = out
                .shapes
                .iter()
                .filter(|s| background(&s.shape, control))
                .map(|s| s.clip_rect)
                .collect();
            assert!(!clips.is_empty(), "actual Scale combo background painted");
            LateScaleSnapshot {
                control,
                label: self.text_rect(c::SCALE),
                help: self.text_rect(c::SCALE_HELP),
                clips,
                footer: egui::containers::panel::PanelState::load(
                    &self.ctx,
                    egui::Id::new("status"),
                )
                .unwrap()
                .rect,
            }
        }
        fn select_and_settle(&mut self, lose_window_focus: bool) -> LateScaleSnapshot {
            use egui::accesskit::{Action, Role};
            for _ in 0..3 {
                self.draw(vec![]);
            }
            let label = *self
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Label
                        && (node.label() == Some(c::SCALE) || node.value() == Some(c::SCALE))
                })
                .unwrap()
                .0;
            let combo = *self
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::ComboBox && node.labelled_by().contains(&label)
                })
                .unwrap()
                .0;
            self.combo = Some(combo);
            self.draw(vec![Self::request(combo)]);
            let choice = *self
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.supports_action(Action::Click)
                        && (node.label() == Some("200%") || node.value() == Some("200%"))
                })
                .unwrap()
                .0;
            self.draw(vec![Self::request(choice)]);
            if lose_window_focus {
                self.focused = false;
            }
            let mut last = self.draw(vec![]);
            for _ in 0..24 {
                last = self.draw(vec![]);
            }
            assert!(
                (self.ctx.zoom_factor() - 2.0).abs() < 0.001,
                "actual popup selected200"
            );
            let snapshot = self.snapshot(&last);
            assert!(
                snapshot.full_context(),
                "initial bounded reveal settled: {snapshot:?}"
            );
            snapshot
        }
        fn late_footer(&mut self) -> LateScaleSnapshot {
            // Explicit bounded synthetic status input to the production footer;
            // this models delayed wrapping, not a real duplicate status message.
            // The compact Settings shell has more room; five bounded segments
            // keep this fixture's intentional contraction meaningful.
            let status = std::iter::repeat_n(c::RECOVERY_DONE, 5)
                .collect::<Vec<_>>()
                .join(" · ");
            assert!(status.len() <= 256, "bounded late status fixture");
            self.app.status = status.clone();
            let mut last = self.draw(vec![]);
            for _ in 0..18 {
                last = self.draw(vec![]);
            }
            assert_eq!(self.app.status, status);
            self.snapshot(&last)
        }
    }
    #[test]
    fn actual_candidate_scale_reveals_after_late_real_footer_wrap_without_window_focus() {
        for dark in [false, true] {
            let mut h = LateScaleHarness::new(dark, true);
            let before = h.select_and_settle(true);
            assert!(
                !h.ctx.input(|input| input.focused),
                "no held window-focus assumption"
            );
            let late = h.late_footer();
            assert!(
                late.footer.height() > before.footer.height() + 8.0,
                "actual production footer adds a line after initial reveal: before={before:?} late={late:?}"
            );
            assert!(
                late.clips
                    .iter()
                    .any(|clip| clip.height() < before.clips[0].height() - 8.0),
                "actual body viewport shrank, no artificial clip"
            );
            assert!(
                late.full_context(),
                "late real footer must preserve full Scale label/help/control after selected200: before={before:?} late={late:?}"
            );
            assert!(!h.ctx.input(|input| input.focused));
            assert!(h.fixture.0.exists());
        }
    }
    #[test]
    fn actual_candidate_scale_late_footer_reveal_respects_user_wheel_and_pointer() {
        for wheel in [true, false] {
            let mut h = LateScaleHarness::new(true, true);
            let before = h.select_and_settle(false);
            let clip = before.clips[0];
            let pos = egui::pos2(clip.right() - 64.0, clip.top() + 20.0);
            h.draw(vec![egui::Event::PointerMoved(pos)]);
            if wheel {
                h.draw(vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 12.0),
                    modifiers: egui::Modifiers::NONE,
                }]);
            } else {
                for pressed in [true, false] {
                    h.draw(vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    }]);
                }
            }
            let mut out = h.draw(vec![]);
            for _ in 0..8 {
                out = h.draw(vec![]);
            }
            let manual = h.snapshot(&out);
            assert!(
                manual
                    .clips
                    .iter()
                    .any(|clip| clip.intersects(manual.control)),
                "bounded manual wheel/pointer keeps actual Scale paint intersecting the body: {manual:?}"
            );
            if wheel {
                assert!(
                    manual.control.top() > before.control.top() + 1.0,
                    "actual user wheel moved the body toward its top: before={before:?} manual={manual:?}"
                );
            }
            let late = h.late_footer();
            assert!(
                late.clips.iter().any(|clip| clip.intersects(late.control)),
                "late-footer Scale remains partly painted for the position-preservation assertion: {late:?}"
            );
            assert!(late.footer.height() > before.footer.height() + 8.0);
            assert!(
                (late.control.top() - manual.control.top()).abs() < 0.5,
                "footer update must not undo actual user wheel/pointer intent: wheel={wheel} manual={manual:?} late={late:?}"
            );
            assert!(
                !late.full_context(),
                "intentional manual position is not forcibly revealed"
            );
        }
    }
    #[test]
    fn actual_candidate_scale_late_footer_watch_stops_across_real_settings_page_gap() {
        let mut h = LateScaleHarness::new(true, true);
        let before = h.select_and_settle(true);
        // Draw real Download composition for intervening Context frames; Appearance
        // show is genuinely absent, not merely a forged cumulative-frame number.
        h.app.settings = false;
        for _ in 0..3 {
            h.draw(vec![]);
        }
        h.app.settings = true;
        let late = h.late_footer();
        assert!(late.footer.height() > before.footer.height() + 8.0);
        assert!(
            !late.full_context(),
            "returning to another page does not resume an old Scale selection episode: {late:?}"
        );
        assert!((late.control.top() - before.control.top()).abs() < 0.5);
    }

    #[test]
    fn actual_candidate_scale_late_footer_watch_respects_keyboard_cancel() {
        let mut h = LateScaleHarness::new(true, true);
        let before = h.select_and_settle(false);
        let out = h.draw(vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        let manual = h.snapshot(&out);
        let late = h.late_footer();
        assert!(late.footer.height() > before.footer.height() + 8.0);
        assert!(
            (late.control.top() - manual.control.top()).abs() < 0.5,
            "late footer must not resume Scale reveal after actual Escape input: manual={manual:?} late={late:?}"
        );
        assert!((h.ctx.zoom_factor() - 2.0).abs() < 0.001);
    }

    fn feature_guidance_frame_sized(
        app: &mut Candidate,
        ctx: &egui::Context,
        size: egui::Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let time = ctx.data_mut(|data| {
            let id = egui::Id::new("feature-guidance-test-clock");
            let next = data.get_temp::<f64>(id).unwrap_or(0.0) + 0.05;
            data.insert_temp(id, next);
            next
        });
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                time: Some(time),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| app.show_ui(ctx),
        )
    }
    fn feature_guidance_frame(
        app: &mut Candidate,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        feature_guidance_frame_sized(app, ctx, egui::vec2(960.0, 640.0), events)
    }
    fn feature_guidance_glyph_visible(output: &egui::FullOutput, text: &str) -> bool {
        feature_guidance_glyph_visible_in(output, text, egui::Rect::EVERYTHING)
    }
    fn feature_guidance_glyph_visible_in(
        output: &egui::FullOutput,
        text: &str,
        viewport: egui::Rect,
    ) -> bool {
        fn glyph(shape: &egui::epaint::Shape, clip: egui::Rect, text: &str) -> bool {
            match shape {
                egui::epaint::Shape::Text(value) if value.galley.text() == text => {
                    !value.galley.elided && clip.contains_rect(value.visual_bounding_rect())
                }
                egui::epaint::Shape::Vec(children) => {
                    children.iter().any(|shape| glyph(shape, clip, text))
                }
                _ => false,
            }
        }
        output
            .shapes
            .iter()
            .any(|shape| glyph(&shape.shape, shape.clip_rect.intersect(viewport), text))
    }
    fn feature_guidance_click_sized(
        app: &mut Candidate,
        ctx: &egui::Context,
        size: egui::Vec2,
        output: &egui::FullOutput,
        label: &str,
    ) -> egui::FullOutput {
        use egui::accesskit::{Action, ActionRequest, Role};
        let buttons: Vec<_> = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
            .collect();
        assert_eq!(buttons.len(), 1, "one actual guidance action: {label}");
        assert!(!buttons[0].1.is_disabled());
        assert!(buttons[0].1.bounds().unwrap().height() >= 32.0);
        feature_guidance_frame_sized(
            app,
            ctx,
            size,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target: buttons[0].0,
                data: None,
            })],
        )
    }
    fn feature_guidance_click(
        app: &mut Candidate,
        ctx: &egui::Context,
        output: &egui::FullOutput,
        label: &str,
    ) -> egui::FullOutput {
        feature_guidance_click_sized(app, ctx, egui::vec2(960.0, 640.0), output, label)
    }
    #[test]
    fn actual_feature_tool_remote_ffmpeg_navigation_and_return_preserve_entire_draft() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://fixture.invalid/guidance".into();
        app.output = fixture.0.to_str().unwrap().into();
        app.tool = fixture
            .0
            .join("synthetic-unopened-ytdlp.exe")
            .to_str()
            .unwrap()
            .into();
        app.ffmpeg.clear();
        app.deno.clear();
        let mut selection = dlpad_core::media_selection::MediaSelection::new(10000).unwrap();
        selection.set_range(1500, 6500).unwrap();
        app.form.apply_media_selection(selection);
        let original = (
            app.url.clone(),
            app.output.clone(),
            app.form.media_range(10000).unwrap(),
        );
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let output = feature_guidance_frame(&mut app, &ctx, vec![]);
        feature_guidance_click(&mut app, &ctx, &output, c::MEDIA_OPEN_FFMPEG);
        assert!(app.settings && app.tools_open && app.tool_return_pending);
        assert_eq!(
            app.tools.as_ref().unwrap().selected_tool(),
            crate::managed::ToolId::Ffmpeg
        );
        let output = feature_guidance_frame(&mut app, &ctx, vec![]);
        feature_guidance_click(&mut app, &ctx, &output, c::FEATURE_TOOL_RETURN);
        assert!(!app.settings && !app.tools_open && !app.tool_return_pending);
        assert_eq!(
            (
                app.url.clone(),
                app.output.clone(),
                app.form.media_range(10000).unwrap()
            ),
            original
        );
        assert!(app.worker.is_none(), "navigation cannot start a download");
    }
    #[test]
    fn actual_feature_tool_failed_managed_verification_uses_repair_navigation_and_keeps_draft() {
        use egui::accesskit::Role;
        for (physical, scale) in [
            (egui::vec2(960.0, 640.0), 1.0),
            (egui::vec2(800.0, 600.0), 1.0),
            (egui::vec2(800.0, 600.0), 2.0),
        ] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.url = "https://fixture.invalid/rejected".into();
            app.output = fixture.0.to_str().unwrap().into();
            app.tool = fixture
                .0
                .join("synthetic-ytdlp.exe")
                .to_str()
                .unwrap()
                .into();
            app.ffmpeg.clear();
            app.deno.clear();
            app.tool_preferences.set_managed_availability(
                1,
                crate::accessible_tool_preferences::ManagedAvailability::Unavailable(
                    c::TOOL_HASH_FAILED,
                ),
            );
            let original = (
                app.url.clone(),
                app.output.clone(),
                app.form.media_range(10000).unwrap(),
            );
            assert_eq!(
                app.feature_tool_needs()[1].unwrap().issue,
                FeatureToolIssue::Rejected(c::TOOL_HASH_FAILED)
            );
            let ctx = egui::Context::default();
            // egui applies pending pixels_per_point to the PREVIOUS InputState rect.
            // Seed that actual physical viewport before requesting app zoom; otherwise
            // its default10000px rect can overwrite the first RawInput.screen_rect.
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, physical)),
                    ..Default::default()
                },
                |_| {},
            );
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            ctx.set_pixels_per_point(scale);
            let logical = physical / scale;
            // Actual two app passes settle panels/footer/body before geometry claims.
            for _ in 0..2 {
                let _ = feature_guidance_frame_sized(&mut app, &ctx, logical, vec![]);
            }
            let mut output = feature_guidance_frame_sized(&mut app, &ctx, logical, vec![]);
            assert!((ctx.content_rect().width() - logical.x).abs() < 0.5);
            assert!((ctx.content_rect().height() - logical.y).abs() < 0.5);
            // Distinguish actual lifecycle state, accessibility text semantics and paint.
            assert_eq!(
                app.feature_tool_needs()[1].unwrap().issue,
                FeatureToolIssue::Rejected(c::TOOL_HASH_FAILED),
                "actual show did not replace the known rejection"
            );
            let has_value = |output: &egui::FullOutput| {
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| {
                        node.role() == Role::Label && node.value() == Some(c::TOOL_HASH_FAILED)
                    })
            };
            assert!(has_value(&output), "Role::Label publishes value, not label");
            let mut reason_seen = false;
            let mut action_seen = false;
            let mut jointly_seen = false;
            let mut wheel_frames = 0;
            for step in 0..32 {
                assert_eq!(
                    app.feature_tool_needs()[1].unwrap().issue,
                    FeatureToolIssue::Rejected(c::TOOL_HASH_FAILED)
                );
                let (row, clip) = ctx
                    .data_mut(|data| {
                        data.get_temp::<(egui::Rect, egui::Rect)>(egui::Id::new(
                            "feature-tool-recovery-geometry",
                        ))
                    })
                    .expect("actual recovery layout");
                let (viewport, offset) = ctx
                    .data(|data| {
                        data.get_temp::<(egui::Rect, egui::Vec2)>(egui::Id::new(
                            "feature-tool-download-viewport",
                        ))
                    })
                    .expect("actual ScrollArea inner viewport");
                let paint_clip = clip;
                let clip = paint_clip.intersect(viewport);
                if step == 0 || step == 31 {
                    eprintln!(
                        "feature-tool-viewport physical={physical:?} scale={scale} step={step} row={row:?} paint_clip={paint_clip:?} inner_viewport={viewport:?} effective_viewport={clip:?} offset={offset:?}"
                    );
                }
                let row_fits =
                    row.width() <= clip.width() + 0.5 && row.height() <= clip.height() + 0.5;
                let reason_now =
                    feature_guidance_glyph_visible_in(&output, c::TOOL_HASH_FAILED, clip);
                let action_now =
                    feature_guidance_glyph_visible_in(&output, c::FEATURE_TOOL_CONFIGURE, clip);
                reason_seen |= reason_now;
                action_seen |= action_now;
                jointly_seen |= reason_now && action_now && clip.contains_rect(row);
                if (row_fits && jointly_seen)
                    || (!row_fits && reason_seen && action_seen && action_now)
                {
                    break;
                }
                assert!(
                    step < 31,
                    "required reason/action not reachable with existing user scrolling: physical={physical:?} scale={scale} row={row:?} clip={clip:?} reason={reason_seen} action={action_seen}"
                );
                wheel_frames += 1;
                output = feature_guidance_frame_sized(
                    &mut app,
                    &ctx,
                    logical,
                    vec![
                        egui::Event::PointerMoved(clip.center()),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(
                                0.0,
                                if row_fits && row.top() < clip.top() {
                                    (clip.top() - row.top()).min(30.0)
                                } else if row_fits && row.bottom() > clip.bottom() {
                                    -(row.bottom() - clip.bottom()).min(30.0)
                                } else {
                                    -30.0
                                },
                            ),
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
            }
            assert!(reason_seen && action_seen);
            let (row, clip) = ctx
                .data_mut(|data| {
                    data.get_temp::<(egui::Rect, egui::Rect)>(egui::Id::new(
                        "feature-tool-recovery-geometry",
                    ))
                })
                .expect("current recovery layout");
            let viewport = ctx
                .data(|data| {
                    data.get_temp::<(egui::Rect, egui::Vec2)>(egui::Id::new(
                        "feature-tool-download-viewport",
                    ))
                })
                .expect("actual current viewport")
                .0;
            let clip = clip.intersect(viewport);
            let row_fits = row.width() <= clip.width() + 0.5 && row.height() <= clip.height() + 0.5;
            if row_fits {
                assert!(
                    jointly_seen,
                    "fit row must expose mandatory reason and action together"
                );
            }
            eprintln!(
                "feature-tool-geometry physical={physical:?} scale={scale} logical={logical:?} user_wheel_frames={wheel_frames} row_fits={row_fits} full_row_joint={jointly_seen} reason_glyph={reason_seen} action_glyph={action_seen}"
            );
            let before_action = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && node.label() == Some(c::FEATURE_TOOL_CONFIGURE)
                })
                .unwrap();
            eprintln!(
                "feature-tool-click-before physical={physical:?} scale={scale} viewport={:?} id={:?} disabled={} bounds={:?} recovery_gate={} settings={} tools_open={} close_wait={} media_busy={} tools_busy={}",
                ctx.content_rect(),
                before_action.0,
                before_action.1.is_disabled(),
                before_action.1.bounds(),
                app.tool_recovery_enabled(),
                app.settings,
                app.tools_open,
                app.close_wait,
                app.media.busy(),
                app.tools.as_ref().is_some_and(|panel| panel.busy())
            );
            assert!(
                app.tool_recovery_enabled(),
                "actual explicit settings action admission before Click"
            );
            assert!(!before_action.1.is_disabled());
            assert!(feature_guidance_glyph_visible_in(
                &output,
                c::FEATURE_TOOL_CONFIGURE,
                clip
            ));
            let clicked = feature_guidance_click_sized(
                &mut app,
                &ctx,
                logical,
                &output,
                c::FEATURE_TOOL_CONFIGURE,
            );
            let after_action =
                clicked
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .and_then(|update| {
                        update.nodes.iter().find(|(_, node)| {
                            node.role() == Role::Button
                                && node.label() == Some(c::FEATURE_TOOL_CONFIGURE)
                        })
                    });
            eprintln!(
                "feature-tool-click-after physical={physical:?} scale={scale} viewport={:?} action={:?} settings={} tools_open={} has_tools={} close_wait={} media_busy={} tools_busy={}",
                ctx.content_rect(),
                after_action.map(|(id, node)| (id, node.is_disabled(), node.bounds())),
                app.settings,
                app.tools_open,
                app.tools.is_some(),
                app.close_wait,
                app.media.busy(),
                app.tools.as_ref().is_some_and(|panel| panel.busy())
            );
            assert!(
                app.settings && app.tools_open && app.tools.is_some(),
                "actual settings navigation must happen after visible enabled admitted Click"
            );
            assert_eq!(
                app.tools.as_ref().unwrap().selected_tool(),
                crate::managed::ToolId::Ffmpeg
            );
            let output = feature_guidance_frame_sized(&mut app, &ctx, logical, vec![]);
            let (row, clip) = ctx
                .data_mut(|data| {
                    data.get_temp::<(egui::Rect, egui::Rect)>(egui::Id::new(
                        "feature-tool-return-geometry",
                    ))
                })
                .expect("actual return context");
            assert!(
                row.height() >= 32.0 && clip.contains_rect(row),
                "return row must fit current body: physical={physical:?} scale={scale} row={row:?} clip={clip:?}"
            );
            assert!(feature_guidance_glyph_visible(
                &output,
                c::FEATURE_TOOL_DRAFT_PRESERVED
            ));
            assert!(feature_guidance_glyph_visible(
                &output,
                c::FEATURE_TOOL_RETURN
            ));
            eprintln!(
                "feature-tool-return physical={physical:?} scale={scale} logical_row_height={} logical_clip_height={}",
                row.height(),
                clip.height()
            );
            feature_guidance_click_sized(&mut app, &ctx, logical, &output, c::FEATURE_TOOL_RETURN);
            assert_eq!(
                (
                    app.url.clone(),
                    app.output.clone(),
                    app.form.media_range(10000).unwrap()
                ),
                original
            );
            assert!(
                !app.settings
                    && !app.tools_open
                    && !app.tool_return_pending
                    && app.worker.is_none()
            );
        }
    }
    #[test]
    fn feature_tool_local_pending_intent_opens_ffmpeg_and_returns_without_automatic_retry() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url.clear();
        app.tool = "invalid-yt-path".into();
        app.deno = "invalid-deno-path".into();
        app.ffmpeg.clear();
        let path = fixture.0.join("synthetic-unopened-local.mp4");
        app.media.dropped(path);
        app.media.poll(
            &egui::Context::default(),
            "",
            &app.tool,
            &app.ffmpeg,
            &app.deno,
            &fixture.0.join("managed-tools"),
        );
        let needs = app.feature_tool_needs();
        assert!(needs[0].is_none() && needs[2].is_none());
        assert_eq!(needs[1].unwrap().tool, crate::managed::ToolId::Ffmpeg);
        assert!(!app.media.local());
        assert!(app.media.has_local_intent());
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let output = feature_guidance_frame(&mut app, &ctx, vec![]);
        feature_guidance_click(&mut app, &ctx, &output, c::MEDIA_OPEN_FFMPEG);
        assert_eq!(
            app.tools.as_ref().unwrap().selected_tool(),
            crate::managed::ToolId::Ffmpeg
        );
        // Navigation alone never retries. No configured worker or file pin is created here.
        let output = feature_guidance_frame(&mut app, &ctx, vec![]);
        feature_guidance_click(&mut app, &ctx, &output, c::FEATURE_TOOL_RETURN);
        assert!(app.media.has_local_intent());
        assert!(!app.media.local());
        assert!(!app.media.busy());
        assert!(app.worker.is_none());
    }
    #[test]
    fn feature_tool_guidance_distinguishes_invalid_path_optional_deno_and_close_wait() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://fixture.invalid/configuration".into();
        app.tool = "relative-ytdlp.exe".into();
        app.ffmpeg.clear();
        app.deno.clear();
        assert_eq!(
            app.feature_tool_needs()[0].unwrap().issue,
            FeatureToolIssue::InvalidPath
        );
        assert!(
            app.feature_tool_needs()[2].is_none(),
            "empty Deno is optional, not proof of missing runtime"
        );
        app.tool_preferences.set_managed_availability(
            1,
            crate::accessible_tool_preferences::ManagedAvailability::Unavailable(
                c::TOOL_UNSUPPORTED,
            ),
        );
        assert_eq!(
            app.feature_tool_needs()[1].unwrap().issue,
            FeatureToolIssue::Rejected(c::TOOL_UNSUPPORTED)
        );
        app.deno = "relative-deno.exe".into();
        assert_eq!(
            app.feature_tool_needs()[2].unwrap().issue,
            FeatureToolIssue::InvalidPath
        );
        let original = (app.url.clone(), app.output.clone());
        app.close_wait = true;
        app.open_feature_tools(crate::managed::ToolId::YtDlp);
        assert!(!app.tools_open && !app.tool_return_pending);
        assert_eq!((app.url.clone(), app.output.clone()), original);
    }
    #[test]
    fn actual_missing_tool_recovery_opens_tools_without_changing_download_draft() {
        use egui::accesskit::{Action, ActionRequest, Role};
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://fixture.invalid/video".into();
        app.output = fixture.0.to_str().unwrap().into();
        app.tool.clear();
        app.ffmpeg.clear();
        let original = (app.url.clone(), app.output.clone(), app.skip_preview);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let frame = |app: &mut Candidate, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 640.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| app.show_ui(ctx),
            )
        };
        for (source, reason) in [
            ("", c::MEDIA_URL_REQUIRED),
            ("not a video URL", c::MEDIA_URL_INVALID),
        ] {
            app.url = source.into();
            let output = frame(&mut app, vec![]);
            assert_eq!(app.download_disabled_reason(), Some(reason));
            assert!(
                !output
                    .platform_output
                    .accesskit_update
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Button
                        && [
                            c::MEDIA_OPEN_TOOLS,
                            c::MEDIA_OPEN_YTDLP,
                            c::MEDIA_OPEN_FFMPEG
                        ]
                        .contains(&node.label().unwrap_or(""))),
                "source entry precedes contextual tool installation"
            );
        }
        app.url = original.0.clone();
        let output = frame(&mut app, vec![]);
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let buttons: Vec<_> = nodes
            .iter()
            .filter(|(_, node)| {
                node.role() == Role::Button && node.label() == Some(c::MEDIA_OPEN_TOOLS)
            })
            .collect();
        assert_eq!(buttons.len(), 1);
        assert!(!buttons[0].1.is_disabled());
        assert!(buttons[0].1.bounds().unwrap().height() >= 32.0);
        frame(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target: buttons[0].0,
                data: None,
            })],
        );
        assert!(
            app.tools_open
                && app.settings
                && app.settings_section == SettingsSection::Tools
                && !app.library_open
                && !app.history_open
        );
        assert_eq!(
            (app.url.clone(), app.output.clone(), app.skip_preview),
            original
        );
        assert!(app.worker.is_none());
        app.tools_open = false;
        app.close_wait = true;
        let output = frame(&mut app, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, n)| n.role() == Role::Button && n.label() == Some(c::MEDIA_OPEN_TOOLS))
                .all(|(_, n)| n.is_disabled())
        );
    }

    #[test]
    fn actual_tool_recovery_keyboard_busy_and_explicit_bypass_policy() {
        use egui::accesskit::{Action, ActionRequest, Role};
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.url = "https://fixture.invalid/video".into();
        app.output = fixture.0.to_str().unwrap().into();
        app.tool.clear();
        app.ffmpeg.clear();
        let original = (app.url.clone(), app.output.clone(), app.skip_preview);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let draw = |app: &mut Candidate, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        app.show_tool_recovery(ui, Some(c::MEDIA_TOOL_REQUIRED))
                    });
                },
            )
        };
        let output = draw(&mut app, vec![]);
        let id = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some(c::MEDIA_OPEN_TOOLS))
            .unwrap()
            .0;
        draw(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: id,
                data: None,
            })],
        );
        draw(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        draw(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(app.tools_open, "actual keyboard activation opens Tools");
        assert_eq!(
            (app.url.clone(), app.output.clone(), app.skip_preview),
            original
        );
        app.tools_open = false;
        app.media.prepare_url(&app.url);
        assert!(app.media.busy());
        let output = draw(&mut app, vec![]);
        assert!(
            output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, n)| n.role() == Role::Button && n.label() == Some(c::MEDIA_OPEN_TOOLS))
                .all(|(_, n)| n.is_disabled()),
            "actual pending preview disables recovery action"
        );
        app.media.clear_source();
        app.skip_preview = true;
        app.tool = fixture
            .0
            .join("synthetic-yt-dlp.exe")
            .to_str()
            .unwrap()
            .into();
        let output = draw(&mut app, vec![]);
        assert!(
            !output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.role() == Role::Button
                    && [
                        c::MEDIA_OPEN_TOOLS,
                        c::MEDIA_OPEN_YTDLP,
                        c::MEDIA_OPEN_FFMPEG
                    ]
                    .contains(&n.label().unwrap_or(""))),
            "explicit bypass with yt-dlp configured does not require FFmpeg"
        );
    }
    // Append inside accessible.rs existing tests module; no production changes.
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    #[ignore = "owned pinned media/FFmpeg and private native host required"]
    fn actual_ready_preview_and_inspector_geometry_across_widths() {
        use egui::{
            Event, RawInput, Rect,
            accesskit::{Action, ActionRequest, Node, NodeId, Role},
        };
        use std::collections::{HashMap, HashSet};
        fn store_current_tree(
            output: &egui::FullOutput,
            root: &mut Option<NodeId>,
            nodes: &mut HashMap<NodeId, Node>,
        ) {
            let Some(update) = &output.platform_output.accesskit_update else {
                return;
            };
            if let Some(tree) = &update.tree {
                *root = Some(tree.root);
            }
            for (id, node) in &update.nodes {
                nodes.insert(*id, node.clone());
            }
            let root = root.expect("initial AccessKit root captured during warmup");
            fn visit(
                id: NodeId,
                nodes: &HashMap<NodeId, Node>,
                reached: &mut HashSet<NodeId>,
                depth: usize,
            ) {
                assert!(
                    depth <= 256 && reached.len() < 10000,
                    "bounded current AccessKit tree"
                );
                assert!(
                    reached.insert(id),
                    "AccessKit cycle or multiply-parented node"
                );
                let node = nodes
                    .get(&id)
                    .expect("current root child present in incremental cache");
                for child in node.children() {
                    visit(*child, nodes, reached, depth + 1);
                }
            }
            let mut reached = HashSet::new();
            visit(root, nodes, &mut reached, 0);
            nodes.retain(|id, _| reached.contains(id));
        }
        fn associated(n: &Node, nodes: &HashMap<NodeId, Node>, label: &str) -> bool {
            n.label() == Some(label)
                || n.labelled_by().iter().any(|id| {
                    nodes
                        .get(id)
                        .is_some_and(|p| p.label() == Some(label) || p.value() == Some(label))
                })
        }
        fn node(nodes: &HashMap<NodeId, Node>, label: &str, role: Role) -> (NodeId, Rect) {
            let found: Vec<_> = nodes
                .iter()
                .filter(|(_, n)| n.role() == role && associated(n, nodes, label))
                .collect();
            assert_eq!(found.len(), 1, "unique actual {label} {role:?}");
            let b = found[0].1.bounds().expect("actual bounds");
            (
                *found[0].0,
                Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                ),
            )
        }
        fn image(shape: &egui::epaint::Shape) -> Option<Rect> {
            match shape {
                egui::epaint::Shape::Rect(rect)
                    if rect.fill_texture_id() != egui::TextureId::Managed(0) =>
                {
                    Some(rect.rect)
                }
                egui::epaint::Shape::Mesh(m) if m.texture_id != egui::TextureId::Managed(0) => {
                    Some(m.vertices.iter().fold(Rect::NOTHING, |r, v| {
                        r.union(Rect::from_min_max(v.pos, v.pos))
                    }))
                }
                egui::epaint::Shape::Vec(v) => v
                    .iter()
                    .filter_map(image)
                    .max_by(|a, b| a.area().total_cmp(&b.area())),
                _ => None,
            }
        }
        let source =
            PathBuf::from(std::env::var_os("DLPAD_MEDIA_FIXTURE").expect("owned pinned source"));
        assert!(std::fs::metadata(&source).unwrap().len() <= 256 * 1024 * 1024);
        let original = std::fs::read(&source).unwrap();
        let ffmpeg = std::env::var_os("DLPAD_MEDIA_FFMPEG").expect("pinned FFmpeg");
        for (physical_width, scale) in [(960.0, 1.0), (824.0, 1.0), (800.0, 1.5), (800.0, 2.0)] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.ffmpeg = PathBuf::from(&ffmpeg).to_string_lossy().into_owned();
            app.set_output_path(
                fixture
                    .0
                    .join("한글 저장 폴더와 긴 경로를 읽을 수 있는지 확인하는 합성 경로"),
            );
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            let mut nodes = HashMap::new();
            let mut accesskit_root = None;
            let mut tick = 0.0;
            let mut frame = |app: &mut Candidate, events: Vec<Event>| {
                tick += 0.05;
                let mut input = RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(physical_width / scale, 640.0 / scale),
                    )),
                    time: Some(tick),
                    focused: true,
                    events,
                    ..Default::default()
                };
                input
                    .viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .native_pixels_per_point = Some(scale);
                ctx.run(input, |ctx| app.show_ui(ctx))
            };
            for _ in 0..3 {
                let warm = frame(&mut app, vec![]);
                store_current_tree(&warm, &mut accesskit_root, &mut nodes);
            }
            let (idle_folder, _) = node(&nodes, c::SAVE_ROOT, Role::TextInput);
            let focused_idle = frame(
                &mut app,
                vec![Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target: idle_folder,
                    data: None,
                })],
            );
            store_current_tree(&focused_idle, &mut accesskit_root, &mut nodes);
            let typed_idle = frame(
                &mut app,
                vec![
                    Event::Key {
                        key: egui::Key::End,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                    Event::Text(" 유지할 초안".into()),
                ],
            );
            store_current_tree(&typed_idle, &mut accesskit_root, &mut nodes);
            let path_draft = app.output.clone();
            assert!(
                path_draft.ends_with(" 유지할 초안"),
                "actual idle path typing"
            );
            assert_eq!(
                ctx.memory(|memory| memory.focused())
                    .map(|id| NodeId(id.value())),
                Some(idle_folder),
                "actual idle path focus"
            );
            let copied = fixture.0.join("owned-preview.mp4");
            std::fs::write(&copied, &original).unwrap();
            app.media.dropped(copied.clone());
            let deadline = std::time::Instant::now() + Duration::from_secs(20);
            let mut stable = 0;
            let mut busy_observed = false;
            let mut last_image_geometry = None;
            let ready = loop {
                assert!(
                    std::time::Instant::now() < deadline,
                    "actual owned preview deadline: width={physical_width} scale={scale} last={last_image_geometry:?}"
                );
                let out = frame(&mut app, vec![]);
                busy_observed |= app.media.busy();
                store_current_tree(&out, &mut accesskit_root, &mut nodes);
                last_image_geometry = out
                    .shapes
                    .iter()
                    .find_map(|shape| image(&shape.shape).map(|rect| (rect, shape.clip_rect)));
                let complete_image = out.shapes.iter().any(|shape| {
                    image(&shape.shape).is_some_and(|rect| shape.clip_rect.contains_rect(rect))
                });
                if app.media.local() && !app.media.busy() && complete_image {
                    stable += 1
                } else {
                    stable = 0
                }
                if stable >= 3 {
                    break out;
                }
                thread::sleep(Duration::from_millis(10));
            };
            assert!(
                busy_observed,
                "actual preview preparation busy state observed"
            );
            let (_, footer) = node(&nodes, c::LOCAL_EXPORT_SAVE_SELECTION, Role::Button);
            let (folder, field) = node(&nodes, c::SAVE_ROOT, Role::TextInput);
            assert_eq!(
                folder, idle_folder,
                "path identity survives idle to local prepared workspace"
            );
            assert_eq!(
                app.output, path_draft,
                "path draft survives media preparation"
            );
            assert_eq!(
                ctx.memory(|memory| memory.focused())
                    .map(|id| NodeId(id.value())),
                Some(idle_folder),
                "logical path focus survives preparation"
            );

            let (_, browse) = node(&nodes, c::CHOOSE_FOLDER, Role::Button);
            let painted = ready
                .shapes
                .iter()
                .filter_map(|s| image(&s.shape).map(|r| (r, s.clip_rect)))
                .max_by(|a, b| a.0.area().total_cmp(&b.0.area()))
                .expect("actual prepared video texture paint");
            assert!(
                painted.1.contains_rect(painted.0),
                "complete preview texture unclipped: width={physical_width} scale={scale} image={:?} clip={:?}",
                painted.0,
                painted.1
            );
            assert!(
                painted.0.bottom() < footer.top(),
                "prepared preview above actual fixed actionfooter"
            );
            assert!(
                painted.0.width() >= 160.0 && painted.0.height() >= 90.0,
                "prepared image retains readable >=160x90 source pixels at every logical size: {:?}",
                painted.0
            );
            assert!(
                !painted.0.intersects(field) && !painted.0.intersects(browse),
                "actual image/path/browse do not overlap"
            );
            if physical_width == 960.0 && scale == 1.0 {
                assert!(
                    (field.center().y - browse.center().y).abs() < 1.0,
                    "wide actual folder and browse same row"
                );
                assert!(
                    field.right() < browse.left(),
                    "wide editor/browse nonoverlap"
                );
            }
            // Actual focus request must reveal the entire field, including stacked scales.
            let focus_action = frame(
                &mut app,
                vec![Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target: folder,
                    data: None,
                })],
            );
            store_current_tree(&focus_action, &mut accesskit_root, &mut nodes);
            let mut focused = frame(&mut app, vec![]);
            store_current_tree(&focused, &mut accesskit_root, &mut nodes);
            for _ in 0..20 {
                focused = frame(&mut app, vec![]);
                store_current_tree(&focused, &mut accesskit_root, &mut nodes);
            }
            let (_, field) = node(&nodes, c::SAVE_ROOT, Role::TextInput);
            let (_, footer) = node(&nodes, c::LOCAL_EXPORT_SAVE_SELECTION, Role::Button);
            assert!(
                field.bottom() < footer.top() && field.top() >= 0.0,
                "whole focused folder above fixedfooter"
            );
            assert!(
                focused
                    .shapes
                    .iter()
                    .any(|s| s.clip_rect.contains_rect(field)),
                "actual painter clip contains entire focused32px field"
            );
            // A real missing owned source produces the error state; no private model seam.
            app.media.dropped(fixture.0.join("不存在的 합성 영상.mp4"));
            let deadline = std::time::Instant::now() + Duration::from_secs(20);
            let mut error_frame = frame(&mut app, vec![]);
            store_current_tree(&error_frame, &mut accesskit_root, &mut nodes);
            while app.media.busy() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "actual error ACK deadline"
                );
                thread::sleep(Duration::from_millis(10));
                error_frame = frame(&mut app, vec![]);
                store_current_tree(&error_frame, &mut accesskit_root, &mut nodes);
            }
            assert!(
                app.media.selection().is_none(),
                "failed source cannot retain prepared selection"
            );
            let error_labels = [
                c::MEDIA_FAILED,
                c::MEDIA_INVALID_DATA,
                c::MEDIA_UNSUPPORTED,
                c::MEDIA_TOOL_FAILED,
            ];
            // Settle the actual one-shot scroll after the asynchronous error ACK.
            for _ in 0..8 {
                error_frame = frame(&mut app, vec![]);
                store_current_tree(&error_frame, &mut accesskit_root, &mut nodes);
            }
            let (error_folder, _) = node(&nodes, c::SAVE_ROOT, Role::TextInput);
            assert_eq!(
                error_folder, idle_folder,
                "path identity survives terminal failure stack"
            );
            assert_eq!(
                app.output, path_draft,
                "failed source retains actual path draft"
            );
            assert!(
                error_frame.shapes.iter().any(|s| {
                    if let egui::epaint::Shape::Text(text) = &s.shape {
                        let ink = text.visual_bounding_rect();
                        error_labels
                            .iter()
                            .any(|label| text.galley.text().contains(label))
                            && ink.is_positive()
                            && s.clip_rect.contains_rect(ink)
                    } else {
                        false
                    }
                }),
                "actual missing-source error glyph fully painted, no hidden error fallback"
            );
            app.media.request_stop();
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !app.media.stopped() {
                assert!(std::time::Instant::now() < deadline);
                app.media.drain();
                thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(
                std::fs::read(copied).unwrap(),
                original,
                "owned original preserved"
            );
        }
        assert_eq!(std::fs::read(source).unwrap(), original);
    }

    // Append inside accessible.rs existing cfg(test) module. No native execution yet.
    #[cfg(not(target_os = "linux"))]
    #[test]
    fn actual_network_open_reveals_reason_once_and_later_wheel_stays_put() {
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        ctx.set_pixels_per_point(2.0);
        let mut reveal = NetworkReveal::default();
        let mut header_id = None;
        ctx.options_mut(|options| options.max_passes = std::num::NonZeroUsize::new(2).unwrap());
        let mut discarded = false;
        let mut reason_rect = None;
        let mut clip = None;
        let mut offsets = Vec::new();
        // Warm up zoom before viewport assertions; animate using deterministic time.
        for frame in 0..70 {
            let events = if frame == 6 {
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: egui::accesskit::NodeId(header_id.unwrap()),
                        data: None,
                    },
                )]
            } else if frame == 45 {
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 150.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -90.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ]
            } else {
                vec![]
            };
            let mut final_pass_offset = None;
            let output = ctx.run(
                egui::RawInput {
                    time: Some(frame as f64 * 0.05),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let area = egui::ScrollArea::vertical()
                            .id_salt("network-reveal-regression")
                            .show(ui, |ui| {
                                ui.add_space(180.0);
                                let response = collapsing(ui, c::NETWORK_OBSERVATION, |ui| {
                                    let critical = ui.label(c::NETWORK_COUNTER_UNAVAILABLE);
                                    reason_rect = Some(critical.rect);
                                    clip = Some(ui.clip_rect());
                                    ui.add_space(500.0);
                                    Some(critical)
                                });
                                header_id = Some(response.header_response.id.value());
                                reveal.update(&response);
                                if reveal.scheduled_frame.is_some() && !discarded {
                                    ctx.request_discard("network reveal regression");
                                    discarded = true;
                                }
                            });
                        final_pass_offset = Some(area.state.offset.y);
                    });
                },
            );
            offsets.push(final_pass_offset.expect("actual final pass scroll offset"));
            if frame == 40 {
                let mut painted = false;
                for clipped in &output.shapes {
                    if let egui::epaint::Shape::Text(text) = &clipped.shape
                        && text.galley.text() == c::NETWORK_COUNTER_UNAVAILABLE
                    {
                        let visual = text.visual_bounding_rect();
                        assert!(clipped.clip_rect.contains_rect(visual));
                        assert!(ctx.content_rect().contains_rect(visual));
                        painted = true;
                    }
                }
                assert!(
                    painted && discarded,
                    "final painted reason after a real discarded pass"
                );
                assert_eq!(ctx.content_rect().width(), 400.0);
                assert_eq!(ctx.pixels_per_point(), 2.0);
                assert!(
                    clip.unwrap().contains_rect(reason_rect.unwrap()),
                    "critical reason must be fully painted after actual native-toggle equivalent"
                );
                assert!(!reveal.pending);
            }
        }
        assert!(
            offsets[65] > offsets[40] + 20.0,
            "actual later wheel must scroll away"
        );
        assert!(
            !reveal.pending,
            "an open section must not re-arm from wheel input"
        );
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    #[test]
    fn actual_candidate_settings_open_paints_complete_unsupported_reason() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.settings = true;
        let original_edit = app.metrics.edit_for_test().to_owned();
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        ctx.set_pixels_per_point(2.0);
        ctx.options_mut(|options| options.max_passes = std::num::NonZeroUsize::new(2).unwrap());
        let mut network_id = None;
        let mut painted = false;
        for frame in 0..55 {
            let events = if frame == 8 {
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: network_id.expect("actual Candidate network header"),
                        data: None,
                    },
                )]
            } else {
                vec![]
            };
            let output = ctx.run(
                egui::RawInput {
                    time: Some(frame as f64 * 0.05),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    app.show_ui(ctx);
                    if frame == 20 {
                        ctx.request_discard("Candidate network paint regression");
                    }
                },
            );
            if let Some(tree) = &output.platform_output.accesskit_update {
                for (id, node) in &tree.nodes {
                    if node.label() == Some(c::NETWORK_OBSERVATION)
                        && node.supports_action(egui::accesskit::Action::Click)
                    {
                        network_id = Some(*id);
                    }
                }
            }
            if frame == 50 {
                for clipped in &output.shapes {
                    if let egui::epaint::Shape::Text(text) = &clipped.shape
                        && text.galley.text() == c::NETWORK_COUNTER_UNAVAILABLE
                    {
                        let visual = text.visual_bounding_rect();
                        assert!(
                            clipped.clip_rect.contains_rect(visual),
                            "actual scroll-body paint clip"
                        );
                        assert!(
                            ctx.content_rect().contains_rect(visual),
                            "actual viewport clip"
                        );
                        painted = true;
                    }
                }
            }
        }
        assert!(
            painted,
            "Candidate settings must paint complete platform reason after opening"
        );
        assert_eq!(app.metrics.edit_for_test(), original_edit);
        app.metrics.request_stop();
    }
    // Append inside accessible.rs existing tests. Draft; root native RED first.
    #[test]
    fn actual_tools_install_focus_is_fully_painted_for_three_tabs_at_200_percent() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::{HashMap, HashSet};
        fn reachable(root: NodeId, nodes: &HashMap<NodeId, Node>) -> HashSet<NodeId> {
            let mut live = HashSet::new();
            let mut pending = vec![root];
            while let Some(id) = pending.pop() {
                if live.insert(id)
                    && let Some(node) = nodes.get(&id)
                {
                    pending.extend(node.children());
                }
            }
            live
        }
        fn target(nodes: &HashMap<NodeId, Node>, root: NodeId, label: &str) -> NodeId {
            let live = reachable(root, nodes);
            let found: Vec<_> = nodes
                .iter()
                .filter(|(id, node)| {
                    live.contains(id) && node.role() == Role::Button && node.label() == Some(label)
                })
                .map(|(id, _)| *id)
                .collect();
            assert_eq!(found.len(), 1, "current reachable actual button {label}");
            found[0]
        }
        fn glyphs(
            shape: &egui::epaint::Shape,
            clip: egui::Rect,
            text: &str,
            found: &mut Vec<(egui::Rect, egui::Rect)>,
        ) {
            match shape {
                egui::epaint::Shape::Text(value) if value.galley.text() == text => {
                    found.push((value.visual_bounding_rect(), clip))
                }
                egui::epaint::Shape::Vec(children) => {
                    for child in children {
                        glyphs(child, clip, text, found);
                    }
                }
                _ => {}
            }
        }
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.open_tools();
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        ctx.set_pixels_per_point(2.0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut nodes = HashMap::new();
            let mut root = None;
            let mut time = 0.0;
            let mut frame = |app: &mut Candidate, events: Vec<egui::Event>| {
                time += 0.05;
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 300.0),
                        )),
                        time: Some(time),
                        focused: true,
                        events,
                        ..Default::default()
                    },
                    |ctx| app.show_ui(ctx),
                );
                if let Some(update) = &output.platform_output.accesskit_update {
                    if let Some(tree) = &update.tree {
                        root = Some(tree.root);
                    }
                    for (id, node) in &update.nodes {
                        nodes.insert(*id, node.clone());
                    }
                }
                (
                    output,
                    nodes.clone(),
                    root.expect("actual accessibility root"),
                )
            };
            for _ in 0..5 {
                frame(&mut app, vec![]);
            }
            for (label, install_label) in [
                ("yt-dlp", c::TOOL_INSTALL_YTDLP),
                ("Deno", c::TOOL_INSTALL_DENO),
                ("FFmpeg·ffprobe", c::TOOL_INSTALL_FFMPEG),
            ] {
                let (_, current, root) = frame(&mut app, vec![]);
                let tab = target(&current, root, label);
                frame(
                    &mut app,
                    vec![egui::Event::AccessKitActionRequest(ActionRequest {
                        action: Action::Click,
                        target: tab,
                        data: None,
                    })],
                );
                let deadline = Instant::now() + Duration::from_secs(4);
                while app.tools.as_ref().unwrap().busy() {
                    assert!(
                        Instant::now() < deadline,
                        "owned inventory worker must finish"
                    );
                    thread::sleep(Duration::from_millis(5));
                    frame(&mut app, vec![]);
                }
                let (_, current, root) = frame(&mut app, vec![]);
                let install = target(&current, root, install_label);
                assert!(
                    !current[&install].is_disabled(),
                    "actual install must be enabled"
                );
                frame(
                    &mut app,
                    vec![egui::Event::AccessKitActionRequest(ActionRequest {
                        action: Action::Focus,
                        target: install,
                        data: None,
                    })],
                );
                let mut final_frame = None;
                for _ in 0..12 {
                    final_frame = Some(frame(&mut app, vec![]));
                }
                let (output, current, root) = final_frame.unwrap();
                let install = target(&current, root, install_label);
                let b = current[&install].bounds().unwrap();
                let rect = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                );
                let footer =
                    egui::containers::panel::PanelState::load(&ctx, egui::Id::new("status"))
                        .unwrap()
                        .rect;
                let mut text = vec![];
                for shape in &output.shapes {
                    glyphs(&shape.shape, shape.clip_rect, install_label, &mut text);
                }
                assert_eq!(ctx.content_rect().width(), 400.0);
                assert_eq!(ctx.pixels_per_point(), 2.0);
                assert_eq!(text.len(), 1, "one final painted install caption");
                let (visual, clip) = text[0];
                println!(
                    "tools_focus_clip tab={label} button={rect:?} glyph={visual:?} clip={clip:?} footer={footer:?}"
                );
                assert!(
                    clip.contains_rect(rect) && clip.contains_rect(visual),
                    "full button and glyph must fit actual body paint clip: {label}"
                );
                assert!(
                    rect.bottom() <= footer.top() && visual.bottom() <= footer.top(),
                    "install must stay above actual footer: {label}"
                );
                assert!(
                    ctx.content_rect().contains_rect(rect),
                    "viewport contains entire control"
                );
            }
        }));
        app.tools.as_mut().unwrap().request_stop();
        app.metrics.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !app.tools.as_ref().unwrap().stopped() || !app.metrics.stopped() {
            app.tools.as_mut().unwrap().poll();
            app.metrics.poll(&ctx);
            assert!(Instant::now() < deadline, "original worker stop ACK");
            thread::sleep(Duration::from_millis(5));
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    // Append all items inside accessible.rs existing tests module.
    #[derive(Default)]
    struct NavigationDiagnosticTree {
        root: Option<egui::accesskit::NodeId>,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    }
    impl NavigationDiagnosticTree {
        fn apply(&mut self, out: &egui::FullOutput) {
            if let Some(update) = &out.platform_output.accesskit_update {
                self.merge(update.tree.as_ref().map(|t| t.root), &update.nodes);
            }
        }
        fn merge(
            &mut self,
            root: Option<egui::accesskit::NodeId>,
            changed: &[(egui::accesskit::NodeId, egui::accesskit::Node)],
        ) {
            if let Some(root) = root {
                self.root = Some(root);
            }
            for (id, node) in changed {
                self.nodes.insert(*id, node.clone());
            }
            fn visit(
                id: egui::accesskit::NodeId,
                nodes: &std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
                seen: &mut std::collections::HashSet<egui::accesskit::NodeId>,
                depth: usize,
            ) {
                assert!(
                    depth <= 256 && seen.len() < 10000,
                    "bounded current AccessKit tree"
                );
                assert!(seen.insert(id), "no cycle/multiple parent");
                let node = nodes
                    .get(&id)
                    .expect("current child exists in incremental cache");
                for child in node.children() {
                    visit(*child, nodes, seen, depth + 1);
                }
            }
            let mut seen = std::collections::HashSet::new();
            visit(
                self.root.expect("initial tree root"),
                &self.nodes,
                &mut seen,
                0,
            );
            self.nodes.retain(|id, _| seen.contains(id));
        }
    }
    #[test]
    fn navigation_diagnostic_tree_preserves_omitted_nodes_and_prunes_removed_children() {
        use egui::accesskit::{Node, NodeId, Role, Toggled};
        let root = NodeId(1);
        let target = NodeId(2);
        let mut parent = Node::new(Role::Window);
        parent.set_children(vec![target]);
        let mut selected = Node::new(Role::Button);
        selected.set_toggled(Toggled::True);
        let mut cache = NavigationDiagnosticTree::default();
        cache.merge(Some(root), &[(root, parent.clone()), (target, selected)]);
        cache.merge(None, &[(root, parent)]);
        assert_eq!(
            cache.nodes[&target].toggled(),
            Some(Toggled::True),
            "omitted partial-update node remains selected"
        );
        let removed = Node::new(Role::Window);
        cache.merge(None, &[(root, removed)]);
        assert!(
            !cache.nodes.contains_key(&target),
            "removed target cannot be read from stale cached node"
        );
    }

    // Append inside accessible.rs existing tests module (uses its owned Fixture/wait_finished).
    #[test]
    fn actual_navigation_click_classifies_transient_or_sustained_selection_paint() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, events| {
            time += 0.05;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            app.show_navigation(ui);
                        });
                        // Same conditional used by the actual show body; no invented state mutation.
                        if app.settings {
                            ui.heading(c::SETTINGS);
                        }
                    });
                },
            )
        };
        let mut cache = NavigationDiagnosticTree::default();
        let first = frame(&mut app, vec![]);
        cache.apply(&first);
        let targets: Vec<_> = cache
            .nodes
            .iter()
            .filter(|(_, n)| {
                n.role() == egui::accesskit::Role::Button && n.label() == Some(c::SETTINGS)
            })
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(
            targets.len(),
            1,
            "unique actual current Settings navigation node"
        );
        let target = targets[0];
        let observe = |out: &egui::FullOutput, cache: &mut NavigationDiagnosticTree| {
            let included = out
                .platform_output
                .accesskit_update
                .as_ref()
                .is_some_and(|u| u.nodes.iter().any(|(id, _)| *id == target));
            cache.apply(out);
            let node = cache
                .nodes
                .get(&target)
                .expect("target reachable in current incremental tree");
            assert_eq!(node.role(), egui::accesskit::Role::Button);
            assert_eq!(node.label(), Some(c::SETTINGS));
            let b = node.bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            let fill = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Rect(r) if r.rect == rect => Some(r.fill),
                    _ => None,
                })
                .expect("actual current-bound navigation painter");
            (
                node.toggled() == Some(egui::accesskit::Toggled::True),
                fill,
                rect,
                included,
            )
        };
        // Warmup avoids treating first-paint repaint scheduling as evidence for click.
        cache.apply(&frame(&mut app, vec![]));
        let clicked = frame(
            &mut app,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    target,
                    action: egui::accesskit::Action::Click,
                    data: None,
                },
            )],
        );
        assert!(app.settings, "actual Candidate navigation consumes Click");
        let (click_selected, click_fill, rect, click_node_in_delta) = observe(&clicked, &mut cache);
        assert!(
        clicked
            .shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()==c::SETTINGS && t.pos.y > rect.bottom())),
        "same-frame current page heading painted"
    );
        let delay = clicked
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .unwrap()
            .repaint_delay;
        let settled = frame(&mut app, vec![]);
        let (settled_selected, settled_fill, _, settled_node_in_delta) =
            observe(&settled, &mut cache);
        let second = frame(&mut app, vec![]);
        let (second_selected, second_fill, _, second_node_in_delta) = observe(&second, &mut cache);
        let current_page_is_settings =
            app.settings && !app.library_open && !app.tools_open && !app.history_open;
        let settled_consistent = current_page_is_settings
            && settled_selected
            && second_selected
            && settled_fill != egui::Color32::TRANSPARENT
            && settled_fill == second_fill;
        let click_consistent = click_selected && click_fill == settled_fill;
        let classification = if !settled_consistent {
            "SUSTAINED_SELECTION_OR_PAINT_MISMATCH_IN_FORCED_SETTLED_FRAMES"
        } else if click_consistent {
            "CLICK_AND_SETTLED_SELECTION_MATCH"
        } else if delay == Duration::ZERO {
            "TRANSIENT_CLICK_FRAME_MISMATCH_WITH_ZERO_DELAY_FOLLOWUP"
        } else {
            "TRANSIENT_CLICK_FRAME_MISMATCH_WITHOUT_ZERO_DELAY_FOLLOWUP"
        };
        eprintln!(
            "navigation_diagnostic classification={classification} click_selected={click_selected} click_fill={click_fill:?} settled_selected={settled_selected} settled_fill={settled_fill:?} second_selected={second_selected} second_fill={second_fill:?} current_settings={current_page_is_settings} repaint_delay={delay:?} click_node_in_delta={click_node_in_delta} settled_node_in_delta={settled_node_in_delta} second_node_in_delta={second_node_in_delta}"
        );
        // A diagnostic test need not manufacture a failure for expected pre-transition paint.
        // Sustained mismatch after real next frames remains a product invariant failure.
        assert!(
            settled_consistent,
            "actual Candidate current page, native selection and settled painter must converge"
        );
    }
    #[test]
    fn actual_narrow_status_footer_wrap_has_text_sized_row_gap() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        let expected_network = if cfg!(windows) {
            c::NETWORK_PREPARING
        } else {
            c::NETWORK_UNKNOWN
        };
        let mut tick = 0.0;
        let mut frame = |app: &mut Candidate| {
            tick += 0.05;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(tick),
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(2.0);
            ctx.run(input, |ctx| app.show_ui(ctx))
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..4 {
                frame(&mut app);
            }
            let out = frame(&mut app);
            let footer = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("status"))
                .unwrap()
                .rect;
            let mut free = None;
            let mut network = None;
            for shape in &out.shapes {
                if let egui::epaint::Shape::Text(text) = &shape.shape {
                    if text.galley.text().starts_with(c::FREE_SPACE) {
                        free = Some(text.visual_bounding_rect());
                    }
                    if text.galley.text() == expected_network {
                        network = Some((text.visual_bounding_rect(), shape.clip_rect));
                    }
                }
            }
            let free = free.expect("actual free metric glyph");
            let (network, clip) = network.expect("actual wrapped initial network status glyph");
            assert!(network.top() >= free.bottom(), "actual second metric line");
            assert!(
                network.top() - free.bottom() <= 8.0,
                "text-sized wrapped gap, not global32px row: free={free:?} network={network:?}"
            );
            assert!(
                footer.contains_rect(free)
                    && footer.contains_rect(network)
                    && clip.contains_rect(network),
                "actual entire metrics/footer paint"
            );
        }));
        app.metrics.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !app.metrics.stopped() {
            app.metrics.poll(&ctx);
            assert!(Instant::now() < deadline, "original metric stop ACK");
            thread::sleep(Duration::from_millis(5));
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    // Append inside accessible.rs existing tests module. Production remains untouched.
    #[test]
    fn actual_download_quality_focus_reveals_full_combo_at_200_percent() {
        use egui::accesskit::{Action, ActionRequest, Role};
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut tree = NavigationDiagnosticTree::default();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, events| {
            time += 0.05;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(time),
                focused: true,
                events,
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(2.0);
            ctx.run(input, |ctx| app.show_ui(ctx))
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Real Candidate media/path/source-before-options content and fixed action/footer.
            // No isolated Options viewport or direct ScrollArea offset injection.
            for _ in 0..20 {
                let out = frame(&mut app, vec![]);
                tree.apply(&out);
            }
            assert_eq!(
                ctx.content_rect().size(),
                egui::vec2(400.0, 300.0),
                "settled actual 800x600@200 viewport"
            );
            let matches: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::ComboBox
                        && (node.value() == Some(c::QUALITY_BEST)
                            || node.label() == Some(c::QUALITY_BEST))
                })
                .map(|(id, _)| *id)
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "one actual current quality Combo, no guessed ordinal"
            );
            let target = matches[0];
            let action = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target,
                    data: None,
                })],
            );
            tree.apply(&action);
            let mut final_out = frame(&mut app, vec![]);
            tree.apply(&final_out);
            for _ in 0..20 {
                final_out = frame(&mut app, vec![]);
                tree.apply(&final_out);
            }
            let node = tree
                .nodes
                .get(&target)
                .expect("same current Combo retained after settle");
            let bounds = node.bounds().expect("actual Combo bounds");
            let rect = egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            );
            assert_eq!(
                ctx.memory(|memory| memory.focused().map(|id| id.value())),
                Some(target.0),
                "actual focus reached original Combo"
            );
            let footer = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("status"))
                .expect("actual status footer")
                .rect;
            assert!(
                rect.top() >= 0.0 && rect.bottom() <= footer.top(),
                "full quality Combo above actual footer: rect={rect:?} footer={footer:?}"
            );
            let painted = final_out
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::epaint::Shape::Text(text) = &shape.shape
                        && text.galley.text() == c::QUALITY_BEST
                    {
                        return Some((text.visual_bounding_rect(), shape.clip_rect));
                    }
                    None
                })
                .expect("actual selected Combo glyph paint");
            assert!(
                painted.1.contains_rect(rect)
                    && painted.1.contains_rect(painted.0)
                    && painted.0.bottom() <= footer.top(),
                "whole Combo and selected glyph in actual paint clip: rect={rect:?} glyph={:?} clip={:?}",
                painted.0,
                painted.1
            );
        }));
        app.metrics.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !app.metrics.stopped() {
            app.metrics.poll(&ctx);
            assert!(Instant::now() < deadline, "original metrics stop ACK");
            thread::sleep(Duration::from_millis(5));
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    // Append in accessible.rs tests, using actual Candidate and current-tree cache.
    #[test]
    fn actual_disabled_start_conditions_click_reveals_reason_once_at_200_percent() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut tree = NavigationDiagnosticTree::default();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate, events| {
            time += 0.05;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(time),
                focused: true,
                events,
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(2.0);
            ctx.run(input, |ctx| app.show_ui(ctx))
        };
        let mut owned_worker_stop = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_URL_REQUIRED));
            for _ in 0..10 {
                let out = frame(&mut app, vec![]);
                tree.apply(&out);
            }
            let rows: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button
                        && n.label() == Some(c::START_CONDITIONS)
                })
                .collect();
            assert_eq!(rows.len(), 1, "one actual condition action");
            let target = *rows[0].0;
            assert!(!rows[0].1.is_disabled());
            assert_eq!(rows[0].1.description(), Some(c::MEDIA_URL_REQUIRED));
            // Actual wheel over body first moves source information away; never inject an offset.
            let out = frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 120.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -240.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            tree.apply(&out);
            let out = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target,
                        data: None,
                    },
                )],
            );
            tree.apply(&out);
            let mut out = frame(&mut app, vec![]);
            for _ in 0..20 {
                out = frame(&mut app, vec![]);
                tree.apply(&out);
            }
            let reason = out
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::epaint::Shape::Text(text) = &shape.shape
                        && text.galley.text() == c::MEDIA_URL_REQUIRED
                    {
                        return Some((text.visual_bounding_rect(), shape.clip_rect));
                    }
                    None
                })
                .unwrap_or_else(|| {
                    let text: Vec<_> = out.shapes.iter().filter_map(|shape| {
                        if let egui::epaint::Shape::Text(text) = &shape.shape {
                            Some((text.galley.text().to_owned(), text.visual_bounding_rect(), shape.clip_rect))
                        } else { None }
                    }).take(40).collect();
                    panic!("missing reason: gate={:?}, media_error={:?}, reveal={:?}, frame={}, target={:?}, offset={:?}, text={text:?}", app.download_disabled_reason(), app.media.tool_error(), app.start_condition_reveal_frame, ctx.cumulative_frame_nr(), ctx.data(|d| d.get_temp::<(u64, egui::Rect)>(egui::Id::new("start-reason-target"))), ctx.data(|d| d.get_temp::<egui::Vec2>(egui::Id::new("start-reason-offset"))));
                });
            let footer =
                egui::containers::panel::PanelState::load(&ctx, egui::Id::new("download-actions"))
                    .unwrap()
                    .rect;
            assert!(
                reason.1.contains_rect(reason.0) && reason.0.bottom() <= footer.top(),
                "revealed actual reason glyph fully painted before action footer"
            );

            let (source_input, source_clip) = ctx
                .data(|data| {
                    data.get_temp::<(egui::Rect, egui::Rect)>(egui::Id::new(
                        "start-reason-source-input",
                    ))
                })
                .unwrap();
            assert!(
                source_clip.contains_rect(source_input),
                "URL label and actual input remain fully visible with reason: {source_input:?} / {source_clip:?}"
            );
            let revealed_offset = ctx
                .data(|data| data.get_temp::<egui::Vec2>(egui::Id::new("start-reason-offset")))
                .unwrap();
            assert!(
                app.start_condition_reveal_frame.is_none(),
                "navigation completes after visible target and settled wheel"
            );
            frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 120.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -120.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            for _ in 0..10 {
                frame(&mut app, vec![]);
            }

            let later_offset = ctx
                .data(|data| data.get_temp::<egui::Vec2>(egui::Id::new("start-reason-offset")))
                .unwrap();
            assert!(
                later_offset.y > revealed_offset.y + 40.0,
                "new wheel remains effective instead of snapping back: {revealed_offset:?} -> {later_offset:?}"
            );
            assert!(
                app.start_condition_reveal_frame.is_none(),
                "later wheel does not rearm explanation navigation"
            );
            assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_URL_REQUIRED));
            assert!(app.worker.is_none(), "explanation never starts download");
            let conditions = *tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::START_CONDITIONS)
                })
                .unwrap()
                .0;
            let out = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: conditions,
                        data: None,
                    },
                )],
            );
            tree.apply(&out);
            assert!(
                app.start_condition_reveal_frame.is_some(),
                "actual click arms navigation before settlement"
            );
            let settings = *tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::SETTINGS)
                })
                .unwrap()
                .0;
            let out = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: settings,
                        data: None,
                    },
                )],
            );
            tree.apply(&out);
            assert!(app.settings, "actual settings navigation occurred");
            assert!(
                app.start_condition_reveal_frame.is_none(),
                "leaving download retires pending explanation navigation"
            );
            let download = *tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::DOWNLOADS)
                })
                .unwrap()
                .0;
            let out = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: download,
                        data: None,
                    },
                )],
            );
            tree.apply(&out);
            assert!(!app.settings);
            assert!(
                app.start_condition_reveal_frame.is_none(),
                "return does not resurrect prior explanation request"
            );
            // Public preview preflight, with missing yt-dlp: no tool is spawned.
            app.url = "https://fixture.invalid/video".into();
            app.output = fixture.0.display().to_string();
            app.output_original = None;
            app.tool.clear();
            app.ffmpeg = fixture.0.join("synthetic-ffmpeg.exe").display().to_string();
            let out = frame(&mut app, vec![]);
            tree.apply(&out);
            app.media.prepare_url(&app.url);
            let out = frame(&mut app, vec![]);
            tree.apply(&out);
            assert_eq!(app.media.tool_error(), Some(c::MEDIA_TOOL_REQUIRED));
            assert_eq!(app.download_disabled_reason(), Some(c::MEDIA_TOOL_REQUIRED));
            let conditions = *tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::START_CONDITIONS)
                })
                .unwrap()
                .0;
            let out = frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: conditions,
                        data: None,
                    },
                )],
            );
            tree.apply(&out);
            let mut out = frame(&mut app, vec![]);
            for _ in 0..20 {
                out = frame(&mut app, vec![]);
                tree.apply(&out);
            }
            let (glyph, clip) = out
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::epaint::Shape::Text(text) = &shape.shape
                        && text.galley.text() == c::MEDIA_TOOL_REQUIRED
                    {
                        Some((text.visual_bounding_rect(), shape.clip_rect))
                    } else {
                        None
                    }
                })
                .expect("actual deduplicated media error glyph must be painted");
            assert!(
                clip.contains_rect(glyph),
                "actual media error fully visible, not only recovery button: {glyph:?} / {clip:?}"
            );
            let (stop_tx, stop_rx) = mpsc::channel();
            let (completion_tx, completion_rx) = mpsc::channel();
            let (started_tx, started_rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                started_tx.send(()).unwrap();
                let _ = stop_rx.recv_timeout(Duration::from_secs(4));
                drop(completion_tx);
            });
            owned_worker_stop = Some(stop_tx);
            app.worker = Some(Worker {
                handle,
                receiver: completion_rx,
                cancel: Arc::new(AtomicBool::new(false)),
                pause: Arc::new(AtomicBool::new(false)),
            });
            started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            let out = frame(&mut app, vec![]);
            tree.apply(&out);
            let condition = tree
                .nodes
                .values()
                .find(|node| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::START_CONDITIONS)
                })
                .unwrap();
            assert!(
                !condition.is_disabled(),
                "read-only explanation remains available with actual owned worker alive"
            );
            assert!(
                tree.nodes
                    .values()
                    .find(|node| node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::START))
                    .unwrap()
                    .is_disabled(),
                "original start gate stays closed while worker is alive"
            );
        }));
        if let Some(stop) = owned_worker_stop {
            let _ = stop.send(());
        }
        if let Some(worker) = app.worker.take() {
            worker.handle.join().unwrap();
        }
        app.metrics.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !app.metrics.stopped() {
            app.metrics.poll(&ctx);
            assert!(Instant::now() < deadline, "original metrics stop ACK");
            thread::sleep(Duration::from_millis(5));
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
    // Append inside existing accessible.rs tests module; root runs actual Context test.
    #[test]
    fn actual_source_prepare_blank_invalid_valid_and_error_retry_affordance() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut tree = NavigationDiagnosticTree::default();
        let mut time = 0.0;
        let mut frame = |app: &mut Candidate| {
            time += 0.01;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    time: Some(time),
                    focused: true,
                    ..Default::default()
                },
                |ctx| app.show_ui(ctx),
            )
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (url, expected) in [
                ("", Some(c::MEDIA_URL_REQUIRED)),
                ("file:///owned/missing.mp4", Some(c::MEDIA_URL_INVALID)),
                ("https://fixture.invalid/video", None),
            ] {
                // Display state only: never click/Enter a remote source in this Context test.
                app.url = url.into();
                let out = frame(&mut app);
                tree.apply(&out);
                let buttons: Vec<_> = tree
                    .nodes
                    .values()
                    .filter(|n| {
                        n.role() == egui::accesskit::Role::Button
                            && n.label() == Some(c::MEDIA_PREPARE)
                    })
                    .collect();
                assert_eq!(buttons.len(), 1);
                assert_eq!(buttons[0].is_disabled(), expected.is_some());
                assert_eq!(buttons[0].description(), expected);
                assert_eq!(app.url, url, "unchanged URL draft and parser policy");
            }
            // A production API invalid-source failure is synchronous and never queues network I/O.
            app.media.prepare_url("file:///owned/not-remote");
            let out = frame(&mut app);
            tree.apply(&out);
            let prepare = tree
                .nodes
                .values()
                .find(|n| {
                    n.role() == egui::accesskit::Role::Button && n.label() == Some(c::MEDIA_PREPARE)
                })
                .unwrap();
            assert!(
                !prepare.is_disabled(),
                "valid URL retry is available despite prior nonfatal prepare error"
            );
            // Real pending API gate sampled before rendering; cancel immediately to avoid remote work.
            app.media.prepare_url(&app.url);
            assert!(app.media.busy());
            assert_eq!(app.source_prepare_disabled_reason(), Some(c::MEDIA_WAIT));
            app.media.cancel();
        }));
        app.media.request_stop();
        app.metrics.request_stop();
        let deadline = Instant::now() + Duration::from_secs(4);
        while !app.metrics.stopped() || !app.media.stopped() {
            app.media.drain();
            app.metrics.poll(&ctx);
            assert!(Instant::now() < deadline, "original metric stop ACK");
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            app.media.stopped(),
            "no queued remote worker in display-only fixture"
        );
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }
    // Append inside accessible.rs existing cfg(test) module. Uncompiled actual Candidate regression.
    #[test]
    fn actual_main_diagnostic_preview_keeps_six_painted_lines_and_later_controls_reachable() {
        use egui::accesskit::{Action, ActionRequest, NodeId, Role};
        fn event(id: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                target: id,
                action,
                data: None,
            })
        }
        fn painted_rows(shape: &egui::Shape, clip: egui::Rect, expected: &str) -> usize {
            match shape {
                egui::Shape::Text(text) if text.galley.text() == expected => text
                    .galley
                    .rows
                    .iter()
                    .filter(|row| {
                        !row.glyphs.is_empty()
                            && clip.contains_rect(row.rect().translate(text.pos.to_vec2()))
                    })
                    .count(),
                egui::Shape::Vec(items) => {
                    items.iter().map(|s| painted_rows(s, clip, expected)).sum()
                }
                _ => 0,
            }
        }
        for scale in [1.0_f32, 2.0] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.diagnostics = Some(crate::accessible_diagnostics::DiagnosticPanel::new(
                Box::new(dlpad_engine::reports::demo_report()),
            ));
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            let mut nodes = std::collections::HashMap::<NodeId, egui::accesskit::Node>::new();
            let mut now = 0.0;
            let mut frame =
                |app: &mut Candidate,
                 nodes: &mut std::collections::HashMap<NodeId, egui::accesskit::Node>,
                 events| {
                    now += 0.05;
                    let mut raw = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(960.0 / scale, 640.0 / scale),
                        )),
                        time: Some(now),
                        focused: true,
                        events,
                        ..Default::default()
                    };
                    raw.viewports
                        .get_mut(&egui::ViewportId::ROOT)
                        .unwrap()
                        .native_pixels_per_point = Some(scale);
                    let output = ctx.run(raw, |ctx| app.show_ui(ctx));
                    if let Some(update) = &output.platform_output.accesskit_update {
                        for (id, node) in &update.nodes {
                            nodes.insert(*id, node.clone());
                        }
                    }
                    output
                };
            let node = |nodes: &std::collections::HashMap<NodeId, egui::accesskit::Node>,
                        label: &str,
                        role| {
                let found: Vec<_> = nodes
                    .iter()
                    .filter(|(_, n)| {
                        n.role() == role
                            && (n.label() == Some(label)
                                || n.labelled_by().iter().any(|label_id| {
                                    nodes.get(label_id).is_some_and(|label_node| {
                                        label_node.role() == Role::Label
                                            && label_node.value() == Some(label)
                                    })
                                }))
                    })
                    .map(|(id, _)| *id)
                    .collect();
                assert_eq!(found.len(), 1, "unique actual node {label}");
                found[0]
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for _ in 0..12 {
                    frame(&mut app, &mut nodes, vec![]);
                }
                let header = node(&nodes, c::DIAGNOSTICS, Role::Button);
                frame(&mut app, &mut nodes, vec![event(header, Action::Expand)]);
                for _ in 0..30 {
                    frame(&mut app, &mut nodes, vec![]);
                }
                let json = node(&nodes, c::JSON, Role::Button);
                frame(&mut app, &mut nodes, vec![event(json, Action::Click)]);
                for _ in 0..20 {
                    frame(&mut app, &mut nodes, vec![]);
                }
                let preview = node(&nodes, c::REPORT_PREVIEW_CONTENT, Role::MultilineTextInput);
                let before_focus = ctx.memory(|memory| memory.focused());
                frame(
                    &mut app,
                    &mut nodes,
                    vec![event(preview, Action::ScrollIntoView)],
                );
                let mut output = frame(&mut app, &mut nodes, vec![]);
                for _ in 0..40 {
                    output = frame(&mut app, &mut nodes, vec![]);
                }
                let (inner, parent, row_height) = ctx
                    .data(|data| {
                        data.get_temp::<(egui::Rect, egui::Rect, f32)>(egui::Id::new(
                            "s10-preview-actual-geometry",
                        ))
                    })
                    .unwrap();
                let visible = inner.intersect(parent);
                let value = nodes[&preview]
                    .value()
                    .expect("actual readonly preview value");
                let count: usize = output
                    .shapes
                    .iter()
                    .map(|shape| {
                        painted_rows(&shape.shape, shape.clip_rect.intersect(visible), value)
                    })
                    .sum();
                println!(
                    "S10_PREVIEW_GEOMETRY scale={scale} inner={inner:?} parent={parent:?} visible={visible:?} row_height={row_height} complete_painted_nonempty_rows={count}"
                );
                assert!(
                    inner.width() > 100.0 && parent.contains_rect(inner),
                    "actual preview viewport fully fits Main body at {scale}: {inner:?} {parent:?}"
                );
                assert!(
                    visible.height() >= 6.0 * row_height + 2.0 * crate::ui_tokens::LABEL_GAP,
                    "six complete readable rows plus original padding at {scale}: {visible:?}"
                );
                assert!(
                    count >= 6,
                    "at least six actual nonempty galley rows fully inside actual paint clip, got {count} at {scale}"
                );
                assert_eq!(
                    ctx.memory(|memory| memory.focused()),
                    before_focus,
                    "preview reveal does not steal focus"
                );
                // The same parent body must still reveal the original later inputs using actual focus actions.
                for (label, role) in [
                    (c::EXPORT_PATH, Role::TextInput),
                    (c::EDIT_LOG, Role::MultilineTextInput),
                ] {
                    let id = node(&nodes, label, role);
                    frame(&mut app, &mut nodes, vec![event(id, Action::Focus)]);
                    for _ in 0..40 {
                        frame(&mut app, &mut nodes, vec![]);
                    }
                    let b = nodes[&id].bounds().expect("actual later control bounds");
                    let logical = egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    );
                    assert!(
                        logical.height() >= crate::ui_tokens::CONTROL_HEIGHT,
                        "original control height preserved"
                    );
                    assert_eq!(
                        ctx.memory(|memory| memory.focused())
                            .map(|id| NodeId(id.value())),
                        Some(id),
                        "exact original input focused: {label}"
                    );
                    let navigation = egui::containers::panel::PanelState::load(
                        &ctx,
                        egui::Id::new("navigation"),
                    )
                    .unwrap()
                    .rect;
                    let footer = egui::containers::panel::PanelState::load(
                        &ctx,
                        egui::Id::new("download-actions"),
                    )
                    .unwrap()
                    .rect;
                    assert!(
                        logical.top() >= navigation.bottom() && logical.bottom() <= footer.top(),
                        "later control stays between actual fixed navigation/footer: {label} {logical:?} {navigation:?} {footer:?}"
                    );
                }
                assert!(
                    !fixture.0.join("current-s10.json").exists(),
                    "display/focus regression never exports"
                );
            }));
            app.diagnostics.as_mut().unwrap().request_stop();
            app.metrics.request_stop();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !app.diagnostics.as_ref().unwrap().stopped() || !app.metrics.stopped() {
                app.diagnostics.as_mut().unwrap().poll(&ctx);
                app.metrics.poll(&ctx);
                assert!(
                    Instant::now() < deadline,
                    "original diagnostic/metrics stop ACK"
                );
                thread::sleep(Duration::from_millis(5));
            }
            if let Err(error) = result {
                std::panic::resume_unwind(error);
            }
        }
    }

    #[test]
    fn actual_diagnostic_open_reveals_toolbar_without_focus_steal_or_wheel_resnap() {
        use egui::accesskit::{Action, ActionRequest, NodeId, Role};
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.diagnostics = Some(crate::accessible_diagnostics::DiagnosticPanel::new(
            Box::new(dlpad_engine::reports::demo_report()),
        ));
        let ctx = egui::Context::default();
        configure_context(&ctx, font_bytes());
        ctx.enable_accesskit();
        let mut nodes = std::collections::HashMap::new();
        let mut time = 0.0;
        let mut discarded_open_frame = false;
        let mut frame = |app: &mut Candidate,
                         nodes: &mut std::collections::HashMap<NodeId, egui::accesskit::Node>,
                         events| {
            time += 0.05;
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(time),
                focused: true,
                events,
                ..Default::default()
            };
            raw.viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(2.0);
            let output = ctx.run(raw, |ctx| {
                app.show_ui(ctx);
                if !discarded_open_frame
                    && app.diagnostic_open_reveal.scheduled_frame == Some(ctx.cumulative_frame_nr())
                {
                    ctx.request_discard("actual diagnostic opening regression");
                    discarded_open_frame = true;
                }
                let target = app
                    .diagnostics
                    .as_ref()
                    .and_then(|panel| panel.open_context_target(ctx));
                ctx.data_mut(|data| {
                    data.insert_temp(egui::Id::new("actual-open-toolbar-target"), target)
                });
            });
            if output.platform_output.num_completed_passes == 2 {
                ctx.data_mut(|data| {
                    data.insert_temp(egui::Id::new("actual-open-discard-observed"), true)
                });
            }
            if let Some(update) = &output.platform_output.accesskit_update {
                for (id, node) in &update.nodes {
                    nodes.insert(*id, node.clone());
                }
            }
            output
        };
        fn request(id: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target: id,
                data: None,
            })
        }
        fn glyphs(
            shape: &egui::Shape,
            clip: egui::Rect,
            label: &str,
            out: &mut Vec<(egui::Rect, egui::Rect)>,
        ) {
            match shape {
                egui::Shape::Text(text) if text.galley.text() == label => {
                    out.push((text.visual_bounding_rect(), clip))
                }
                egui::Shape::Vec(items) => {
                    for shape in items {
                        glyphs(shape, clip, label, out);
                    }
                }
                _ => {}
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..10 {
                frame(&mut app, &mut nodes, vec![]);
            }
            // Root fixture must locate current disclosure via actual reachable child graph;
            // this closure intentionally does not guess a logical widget ID.
            let environment = nodes
                .iter()
                .find(|(_, n)| n.label() == Some(c::DOWNLOAD_ENVIRONMENT))
                .map(|(id, _)| *id)
                .unwrap();
            frame(
                &mut app,
                &mut nodes,
                vec![request(environment, Action::Expand)],
            );
            for _ in 0..20 {
                frame(&mut app, &mut nodes, vec![]);
            }
            let header = nodes
                .iter()
                .find(|(_, n)| n.label() == Some(c::DIAGNOSTICS))
                .map(|(id, _)| *id)
                .unwrap();
            frame(
                &mut app,
                &mut nodes,
                vec![
                    request(header, Action::Focus),
                    request(header, Action::Expand),
                ],
            );
            let focus = ctx.memory(|memory| memory.focused());
            let mut last = frame(&mut app, &mut nodes, vec![]);
            for _ in 0..40 {
                last = frame(&mut app, &mut nodes, vec![]);
            }
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                focus,
                "opening never requests a new control focus"
            );
            assert!(
                ctx.data(
                    |data| data.get_temp::<bool>(egui::Id::new("actual-open-discard-observed"))
                )
                .unwrap_or(false),
                "opening was checked after an actual discarded pass"
            );
            let mut painted = Vec::new();
            for shape in &last.shapes {
                glyphs(&shape.shape, shape.clip_rect, c::COPY_REPORT, &mut painted);
            }
            assert_eq!(painted.len(), 1, "actual Copy glyph painted once");
            assert!(
                painted[0].1.contains_rect(painted[0].0),
                "actual Copy glyph wholly in body paint clip: glyph={:?} clip={:?} toolbar={:?}",
                painted[0].0,
                painted[0].1,
                ctx.data(|data| data
                    .get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target")))
                    .flatten()
            );
            assert!(nodes.values().any(|node|node.role()==Role::Button&&node.label()==Some(c::COPY_REPORT)));
            let baseline = ctx
                .data(|data| {
                    data.get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target"))
                })
                .flatten()
                .unwrap();
            frame(
                &mut app,
                &mut nodes,
                vec![
                    egui::Event::PointerMoved(egui::pos2(180.0, 100.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -100.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            for _ in 0..30 {
                frame(&mut app, &mut nodes, vec![]);
            }
            let wheel = ctx
                .data(|data| {
                    data.get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target"))
                })
                .flatten()
                .unwrap();
            assert!(
                (wheel.top() - baseline.top()).abs() > 20.0,
                "actual later wheel moves the body"
            );
            for _ in 0..10 {
                frame(&mut app, &mut nodes, vec![]);
            }
            let idle = ctx
                .data(|data| {
                    data.get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target"))
                })
                .flatten()
                .unwrap();
            assert!(
                (idle.top() - wheel.top()).abs() < 1.0,
                "later idle never snaps back"
            );
            let old_panel = app.diagnostics.as_mut().unwrap();
            old_panel.request_stop();
            let replacement_deadline = Instant::now() + Duration::from_secs(5);
            while !old_panel.stopped() {
                assert!(
                    Instant::now() < replacement_deadline,
                    "old report worker stop ACK"
                );
                thread::sleep(Duration::from_millis(5));
            }
            app.diagnostics = Some(crate::accessible_diagnostics::DiagnosticPanel::new(
                Box::new(dlpad_engine::reports::demo_report()),
            ));
            let replacement_focus = ctx.memory(|memory| memory.focused());
            for _ in 0..40 {
                frame(&mut app, &mut nodes, vec![]);
            }
            let replacement_target = ctx
                .data(|data| {
                    data.get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target"))
                })
                .flatten()
                .unwrap();
            assert!(
                (replacement_target.top() - idle.top()).abs() < 1.0,
                "replacing a report while open does not rearm disclosure scroll"
            );
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                replacement_focus,
                "report replacement does not steal focus"
            );
            frame(
                &mut app,
                &mut nodes,
                vec![request(header, Action::Collapse)],
            );
            for _ in 0..20 {
                frame(&mut app, &mut nodes, vec![]);
            }
            assert!(
                ctx.data(|data| data
                    .get_temp::<Option<egui::Rect>>(egui::Id::new("actual-open-toolbar-target")))
                    .flatten()
                    .is_none(),
                "closed body has no current toolbar"
            );
            let reopen_focus = ctx.memory(|memory| memory.focused());
            frame(&mut app, &mut nodes, vec![request(header, Action::Expand)]);
            let mut reopened = frame(&mut app, &mut nodes, vec![]);
            for _ in 0..40 {
                reopened = frame(&mut app, &mut nodes, vec![]);
            }
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                reopen_focus,
                "reopening does not steal focus"
            );
            let mut reopened_glyphs = Vec::new();
            for shape in &reopened.shapes {
                glyphs(
                    &shape.shape,
                    shape.clip_rect,
                    c::COPY_REPORT,
                    &mut reopened_glyphs,
                );
            }
            assert_eq!(reopened_glyphs.len(), 1, "actual reopened Copy glyph");
            assert!(
                reopened_glyphs[0].1.contains_rect(reopened_glyphs[0].0),
                "reopening reveals the full Copy glyph again"
            );
        }));
        // Explicit original panel stop ACK even when glyph assertion is RED.
        let panel = app.diagnostics.as_mut().unwrap();
        panel.request_stop();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !panel.stopped() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(panel.stopped(), "original diagnostic worker ACK");
        app.metrics.request_stop();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !app.metrics.stopped() {
            app.metrics.poll(&ctx);
            assert!(
                std::time::Instant::now() < deadline,
                "original metrics stop ACK"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn host_error_diagnostic_picker_cancel_poll_drains_completion_without_rendering() {
        let fixture = Fixture::new();
        let mut app = Candidate::new(fixture.0.clone());
        wait_finished(&mut app);
        app.diagnostics = Some(
            crate::accessible_diagnostics::DiagnosticPanel::queued_picker_for_test(
                fixture.0.join("uncreated-report.json"),
            ),
        );
        let ctx = egui::Context::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert!(app.diagnostics.as_ref().unwrap().picker_busy());
            app.cancel_folder_picker();
            app.poll_folder_picker(&ctx);
            let panel = app.diagnostics.as_ref().unwrap();
            assert!(
                !panel.picker_busy(),
                "renderer error polling must drain diagnostic picker ACK"
            );
            assert_eq!(panel.export_draft_for_test(), "original draft");
            assert!(!fixture.0.join("uncreated-report.json").exists());
        }));
        app.media.drain();
        app.metrics.request_stop();
        app.diagnostics.as_mut().unwrap().request_stop();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !app.diagnostics.as_ref().unwrap().stopped() || !app.metrics.stopped() {
            app.diagnostics.as_mut().unwrap().poll(&ctx);
            app.metrics.poll(&ctx);
            assert!(
                Instant::now() < deadline,
                "original diagnostic/metrics cleanup ACK"
            );
            thread::sleep(Duration::from_millis(5));
        }
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    #[test]
    fn actual_candidate_missing_tool_previews_valid_redacted_draft_but_cannot_start() {
        use egui::accesskit::{Action, ActionRequest, Invalid, NodeId, Role};
        for tool in ["", "relative-yt-dlp.exe"] {
            let fixture = Fixture::new();
            let mut app = Candidate::new(fixture.0.clone());
            wait_finished(&mut app);
            app.url =
                "https://fixture.invalid/watch?v=private-preview-token#private-fragment".into();
            app.tool = tool.into();
            app.ffmpeg.clear();
            app.deno.clear();
            app.set_output_path(fixture.0.clone());
            assert_eq!(app.options().unwrap_err(), c::TOOL_PATH);
            let expected = crate::accessible_plan::preview(&app.draft_options().unwrap()).unwrap();
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            ctx.enable_accesskit();
            let mut time = 0.0;
            let mut frame = |app: &mut Candidate, events| {
                time += 0.05;
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(640.0, 4000.0),
                        )),
                        time: Some(time),
                        focused: true,
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            app.form.show(ui, true);
                            app.show_plan(ui);
                        });
                    },
                )
            };
            let initial = frame(&mut app, vec![]);
            let nodes = &initial
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            let plans: Vec<_> = nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::MultilineTextInput)
                .collect();
            assert_eq!(
                plans.len(),
                1,
                "actual readonly plan with an unselected tool"
            );
            let (plan_id, plan_node) = plans[0];
            assert!(plan_node.is_read_only());
            assert!(!plan_node.supports_action(Action::Focus));
            assert!(plan_node.supports_action(Action::ScrollIntoView));
            let provider_text: String = plan_node
                .children()
                .iter()
                .map(|child| {
                    let node = &nodes.iter().find(|(id, _)| id == child).unwrap().1;
                    assert_eq!(
                        node.role(),
                        Role::TextRun,
                        "actual readonly provider text child"
                    );
                    node.value().unwrap()
                })
                .collect();
            assert_eq!(
                provider_text, expected,
                "actual readonly provider renders redacted argv"
            );
            assert_eq!(app.plan_preview, expected);
            let preview_args: Vec<String> = serde_json::from_str(&app.plan_preview).unwrap();
            assert_eq!(
                preview_args.last().map(String::as_str),
                Some(c::OMITTED_SOURCE)
            );
            assert!(!app.plan_preview.contains("private-preview-token"));
            assert!(!app.plan_preview.contains("private-fragment"));
            assert!(!app.plan_preview.contains("fixture.invalid"));
            app.start(false);
            assert!(
                app.worker.is_none(),
                "preview cannot authorize missing-tool execution"
            );
            assert_eq!(app.options().unwrap_err(), c::TOOL_PATH);
            assert!(
                !fixture.0.join("jobs.sqlite").exists(),
                "preview/start rejection creates no job journal"
            );
            let advanced = nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && node.label() == Some(c::ADVANCED_OPEN)
                })
                .unwrap()
                .0;
            frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    target: advanced,
                    action: Action::Click,
                    data: None,
                })],
            );
            for _ in 0..3 {
                frame(&mut app, vec![]);
            }
            let rate_id = NodeId(
                egui::Id::new((
                    "download-option-input",
                    dlpad_core::options::CATALOG[0].label,
                ))
                .value(),
            );
            frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    target: rate_id,
                    action: Action::Focus,
                    data: None,
                })],
            );
            let invalid = frame(&mut app, vec![egui::Event::Text("0".into())]);
            let invalid_nodes = &invalid
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            assert_eq!(
                invalid_nodes
                    .iter()
                    .find(|(id, _)| *id == rate_id)
                    .unwrap()
                    .1
                    .invalid(),
                Some(Invalid::True)
            );
            assert!(
                invalid_nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Label
                        && node.value() == Some(c::PLAN_PREVIEW_INVALID))
            );
            assert!(
                !invalid_nodes
                    .iter()
                    .any(|(id, node)| *id == *plan_id || node.role() == Role::MultilineTextInput),
                "invalid draft removes readonly plan provider"
            );
            assert!(app.plan_preview.is_empty());
            assert!(app.draft_options().is_err());
            assert!(app.options().is_err());
            app.start(false);
            assert!(
                app.worker.is_none(),
                "invalid rate never starts even when preview ignores tool selection"
            );
            assert!(!fixture.0.join("jobs.sqlite").exists());
        }
    }
    #[cfg(feature = "cpu")]
    mod copy_progress_context_audit {
        use super::*;
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::{HashMap, HashSet};
        struct Scene {
            app: Candidate,
            ctx: egui::Context,
            nodes: HashMap<NodeId, Node>,
            tree_root: Option<NodeId>,
            clock: f64,
            size: [usize; 2],
            renderer: egui_software_backend::EguiSoftwareRender,
            _fixture: Option<Fixture>,
            closed: bool,
            source: PathBuf,
        }
        impl Scene {
            fn new(width: usize, zoom: u16, dark: bool) -> Self {
                let fixture = Fixture::new();
                let data = fixture.0.join("data");
                let media = fixture.0.join("media");
                std::fs::create_dir(&data).unwrap();
                std::fs::create_dir(&media).unwrap();
                let store = dlpad_engine::store::Store::open(&data.join("theme.sqlite")).unwrap();
                store
                    .set_setting("library_root", media.to_str().unwrap())
                    .unwrap();
                drop(store);
                std::fs::write(
                    data.join("theme.json"),
                    if dark { "\"Dark\"" } else { "\"Light\"" },
                )
                .unwrap();
                let source = media.join("원본.mp4");
                std::fs::write(&source, b"original").unwrap();
                std::fs::write(media.join("completed-copy.mp4"), b"original").unwrap();
                let ctx = egui::Context::default();
                configure_context(&ctx, font_bytes());
                ctx.enable_accesskit();
                let mut app = Candidate::new(data);
                wait_finished(&mut app);
                app.use_host_metrics_freshness_timer();
                app.appearance.fixture_initial_scale(zoom);
                app.apply_appearance(&ctx);
                app.library = Some(
                    crate::accessible_library::LibraryPanel::copy_context_fixture(
                        media,
                        source.clone(),
                    ),
                );
                Self {
                    app,
                    ctx,
                    nodes: HashMap::new(),
                    tree_root: None,
                    clock: 0.0,
                    size: [width, 640],
                    renderer: egui_software_backend::EguiSoftwareRender::new(
                        egui_software_backend::ColorFieldOrder::Bgra,
                    ),
                    _fixture: Some(fixture),
                    closed: false,
                    source,
                }
            }
            fn event(id: NodeId, action: Action) -> egui::Event {
                egui::Event::AccessKitActionRequest(ActionRequest {
                    action,
                    target: id,
                    data: None,
                })
            }
            fn frame(&mut self, events: Vec<egui::Event>, close: bool) -> egui::FullOutput {
                self.clock += 0.05;
                let zoom = self.ctx.zoom_factor();
                let mut input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(self.size[0] as f32 / zoom, self.size[1] as f32 / zoom),
                    )),
                    focused: true,
                    time: Some(self.clock),
                    events,
                    ..Default::default()
                };
                input
                    .viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .native_pixels_per_point = Some(1.0);
                if close {
                    input
                        .viewports
                        .get_mut(&egui::ViewportId::ROOT)
                        .unwrap()
                        .events
                        .push(egui::ViewportEvent::Close);
                }
                let out = self.ctx.run(input, |ctx| self.app.show_ui(ctx));
                self.closed |= out.viewport_output.values().any(|viewport| {
                    viewport
                        .commands
                        .iter()
                        .any(|command| matches!(command, egui::ViewportCommand::Close))
                });
                let update = out
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .expect("actual AccessKit update");
                if let Some(tree) = &update.tree {
                    self.tree_root = Some(tree.root);
                }
                self.nodes.extend(update.nodes.iter().cloned());
                let mut live = HashSet::new();
                let mut work = vec![self.tree_root.expect("actual tree root")];
                while let Some(id) = work.pop() {
                    if live.insert(id) {
                        work.extend(self.nodes[&id].children().iter().copied());
                    }
                }
                self.nodes.retain(|id, _| live.contains(id));
                let jobs = self
                    .ctx
                    .tessellate(out.shapes.clone(), out.pixels_per_point);
                assert!(jobs.len() <= 8192 && jobs.iter().all(|job| job.clip_rect.is_finite()));
                assert_eq!(
                    self.ctx.viewport_rect().size() * out.pixels_per_point,
                    egui::vec2(self.size[0] as f32, self.size[1] as f32)
                );
                let mut pixels = vec![[0u8; 4]; self.size[0] * self.size[1]];
                self.renderer.render(
                    &mut egui_software_backend::BufferMutRef::new(
                        &mut pixels,
                        self.size[0],
                        self.size[1],
                    ),
                    &jobs,
                    &out.textures_delta,
                    out.pixels_per_point,
                );
                assert!(
                    pixels.iter().all(|p| p[3] != 0),
                    "actual CPU scene has unpainted pixels"
                );
                assert_eq!(std::fs::read(&self.source).unwrap(), b"original");
                out
            }
            fn button(&self, caption: &str) -> NodeId {
                let hits: Vec<_> = self
                    .nodes
                    .iter()
                    .filter(|(_, n)| n.role() == Role::Button && n.label() == Some(caption))
                    .collect();
                assert_eq!(hits.len(), 1, "unique current button {caption}");
                *hits[0].0
            }
            fn route(&mut self, caption: &str) {
                let id = self.button(caption);
                assert!(!self.nodes[&id].is_disabled());
                self.frame(vec![Self::event(id, Action::Click)], false);
                self.frame(vec![], false);
            }
            fn footer(&mut self, expected: &str) {
                let out = self.frame(vec![], false);
                let footer =
                    egui::containers::panel::PanelState::load(&self.ctx, egui::Id::new("status"))
                        .unwrap()
                        .rect;
                let painted: Vec<_> = out
                    .shapes
                    .iter()
                    .filter_map(|p| {
                        if let egui::Shape::Text(text) = &p.shape
                            && text.galley.text() == expected
                        {
                            return Some((
                                p.clip_rect,
                                egui::Rect::from_min_size(text.pos, text.galley.size()),
                            ));
                        }
                        None
                    })
                    .collect();
                assert_eq!(painted.len(), 1, "one actual footer galley {expected}");
                assert!(
                    painted[0].0.contains_rect(painted[0].1)
                        && footer.contains_rect(painted[0].1)
                        && self.ctx.viewport_rect().contains_rect(painted[0].1),
                    "full footer text clips: {painted:?} footer={footer:?}"
                );
                assert!(
                    self.nodes
                        .values()
                        .any(|n| n.label() == Some(expected) || n.value() == Some(expected)),
                    "current reachable accessibility text"
                );
            }
            fn finish(&mut self) {
                if self.closed {
                    return;
                }
                // No worker exists behind this Library test transport. End only that fixture
                // transport; real Candidate workers must satisfy original Close gates.
                if let Some(panel) = &self.app.library {
                    panel.copy_context_worker_ack();
                }
                self.frame(vec![], true);
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                while !self.closed && std::time::Instant::now() < deadline {
                    let stop = self
                        .nodes
                        .iter()
                        .find(|(_, n)| {
                            n.role() == Role::Button
                                && n.label() == Some(c::STOP_EXIT)
                                && !n.is_disabled()
                        })
                        .map(|(id, _)| *id);
                    self.frame(
                        stop.map(|id| vec![Self::event(id, Action::Click)])
                            .unwrap_or_default(),
                        false,
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
                assert!(
                    self.closed,
                    "actual Candidate Close gates did not acknowledge owned fixture cleanup within 5s"
                );
            }
            fn reveal_stop(&mut self) -> NodeId {
                let id = self.button(c::LIBRARY_EXPORT_STOP);
                assert!(!self.nodes[&id].is_disabled());
                self.frame(vec![Self::event(id, Action::Focus)], false);
                for _ in 0..32 {
                    let out = self.frame(vec![], false);
                    let visible = out.shapes.iter().any(|p| {
                        if let egui::Shape::Text(t) = &p.shape {
                            t.galley.text() == c::LIBRARY_EXPORT_STOP
                                && p.clip_rect.contains_rect(egui::Rect::from_min_size(
                                    t.pos,
                                    t.galley.size(),
                                ))
                        } else {
                            false
                        }
                    });
                    if visible {
                        let bounds = self.nodes[&id].bounds().unwrap();
                        let control = egui::Rect::from_min_max(
                            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                        );
                        let footer = egui::containers::panel::PanelState::load(
                            &self.ctx,
                            egui::Id::new("status"),
                        )
                        .unwrap()
                        .rect;
                        let full = out.shapes.iter().any(|p| {
                            if let egui::Shape::Text(t) = &p.shape {
                                t.galley.text() == c::LIBRARY_EXPORT_STOP
                                    && p.clip_rect.contains_rect(control)
                            } else {
                                false
                            }
                        });
                        if full
                            && self.ctx.viewport_rect().contains_rect(control)
                            && control.bottom() <= footer.top()
                        {
                            assert!(control.height() >= 32.0);
                            return id;
                        }
                    }
                }
                panic!("setup failed: actual Stop cannot be revealed in 32 idle frames");
            }
        }
        impl Drop for Scene {
            fn drop(&mut self) {
                // Preserve the original assertion and retain files if cleanup is unknown.
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.finish()));
                if !self.closed {
                    if let Some(fixture) = self._fixture.take() {
                        std::mem::forget(fixture);
                    }
                    eprintln!("copy-context owned fixture retained: Close ACK unconfirmed");
                }
            }
        }
        #[test]
        fn actual_copy_progress_cross_tab_stop_late_receipt_notice_and_retained_evidence_matrix() {
            for width in [800, 960] {
                for zoom in [100, 200] {
                    for dark in [false, true] {
                        let mut s = Scene::new(width, zoom, dark);
                        s.frame(vec![], false);
                        s.footer(c::LIBRARY_EXPORT_PREPARING);
                        s.route(c::SETTINGS);
                        assert!(s.app.settings);
                        s.footer(c::LIBRARY_EXPORT_PREPARING);
                        s.route(c::LIBRARY);
                        assert!(s.app.library_open);
                        let refresh = s.button(c::REFRESH);
                        assert!(
                            s.nodes[&refresh].is_disabled(),
                            "accepted copy blocks Library refresh"
                        );
                        let stop = s.reveal_stop();
                        s.frame(vec![Scene::event(stop, Action::Click)], false);
                        assert!(s.app.library.as_ref().unwrap().copy_context_cancelled());
                        assert!(
                            s.app.library.as_ref().unwrap().busy(),
                            "Stop is request, not terminal ACK"
                        );
                        s.frame(vec![], false);
                        assert!(s.nodes[&stop].is_disabled());
                        s.frame(vec![Scene::event(stop, Action::Click)], false);
                        assert!(
                            s.app.library.as_ref().unwrap().busy(),
                            "disabled repeated Stop must not complete copy"
                        );
                        s.route(c::DOWNLOADS);
                        s.footer(c::LIBRARY_EXPORT_STOPPING);
                        s.app
                            .library
                            .as_mut()
                            .unwrap()
                            .copy_context_ack(true, false);
                        s.frame(vec![], false);
                        assert!(
                            s.app.library.as_ref().unwrap().busy(),
                            "mismatched receipt cannot publish"
                        );
                        s.app
                            .library
                            .as_mut()
                            .unwrap()
                            .copy_context_ack(false, false);
                        s.frame(vec![], false);
                        assert!(!s.app.library.as_ref().unwrap().busy());
                        assert!(s.app.library.as_ref().unwrap().copy_context_retained());
                        s.footer(c::LIBRARY_EXPORT_RETAINED_NOTICE);
                        assert!(
                            s.app
                                .library
                                .as_mut()
                                .unwrap()
                                .take_export_notice()
                                .is_none(),
                            "Main consumed one notice"
                        );
                        s.app.status = "distinct later unrelated status".into();
                        s.frame(vec![], false);
                        assert_eq!(
                            s.app.status, "distinct later unrelated status",
                            "same receipt cannot overwrite later status"
                        );
                        s.route(c::LIBRARY);
                        s.frame(vec![], false);
                        assert!(s.app.library.as_ref().unwrap().copy_context_retained());
                        let retained_path = crate::path_picker::display_path(
                            &s.source
                                .parent()
                                .unwrap()
                                .join(".dlpad-export-owned.partial"),
                        );
                        assert!(
                            s.nodes
                                .values()
                                .any(|n| n.label() == Some(retained_path.as_str())
                                    || n.value() == Some(retained_path.as_str())),
                            "retained raw path stays in the current Library accessibility tree; not a delete capability"
                        );
                        assert_eq!(std::fs::read(&s.source).unwrap(), b"original");
                        s.finish();
                    }
                }
            }
        }
        #[test]
        fn actual_copy_close_wait_retains_owner_until_worker_ack_and_completion_notice_is_once() {
            let mut s = Scene::new(800, 200, true);
            s.frame(vec![], false);
            s.app
                .library
                .as_mut()
                .unwrap()
                .copy_context_ack(false, true);
            s.frame(vec![], false);
            s.footer(c::LIBRARY_EXPORT_DONE);
            assert!(
                s.app
                    .library
                    .as_mut()
                    .unwrap()
                    .take_export_notice()
                    .is_none()
            );
            s.app.status = "completion already consumed".into();
            s.frame(vec![], false);
            assert_eq!(s.app.status, "completion already consumed");
            let root = s.source.parent().unwrap().to_owned();
            s.app.library = Some(
                crate::accessible_library::LibraryPanel::copy_context_fixture(
                    root,
                    s.source.clone(),
                ),
            );
            s.frame(vec![], true);
            assert!(s.app.close_prompt && !s.app.close_wait);
            let stop_exit = s.button(c::STOP_EXIT);
            s.frame(vec![Scene::event(stop_exit, Action::Click)], false);
            assert!(s.app.close_wait);
            assert!(s.app.library.as_ref().unwrap().copy_context_cancelled());
            s.frame(vec![], false);
            assert!(
                s.app.library.is_some() && !s.app.library.as_ref().unwrap().stopped(),
                "owner retained before ACK"
            );
            s.app
                .library
                .as_mut()
                .unwrap()
                .copy_context_ack(false, false);
            s.frame(vec![], false);
            assert!(s.app.library.as_ref().unwrap().copy_context_retained());
            assert!(
                !s.app.library.as_ref().unwrap().stopped(),
                "terminal receipt alone is not StopAck"
            );
            s.app.library.as_ref().unwrap().copy_context_worker_ack();
            s.frame(vec![], false);
            assert!(s.app.library.as_ref().unwrap().stopped());
            // Other workers' readiness and real native natural exit are deliberately not fabricated.
            s.finish();
        }
    }
}

#[cfg(test)]
mod bounded_font_tests {
    use super::*;
    use std::io::{self, Cursor};

    struct TinyReads(Cursor<Vec<u8>>);
    impl Read for TinyReads {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output.len().min(2);
            self.0.read(&mut output[..count])
        }
    }

    #[test]
    fn short_chunks_preserve_complete_font() {
        let expected = b"synthetic font bytes";
        let bytes = read_bounded_font(
            TinyReads(Cursor::new(expected.to_vec())),
            expected.len() as u64,
        )
        .unwrap();
        assert_eq!(bytes, expected);
    }

    #[test]
    fn changed_length_rejects_growth_and_truncation() {
        assert!(read_bounded_font(Cursor::new(b"longer"), 3).is_err());
        assert_eq!(
            read_bounded_font(Cursor::new(b"short"), 9)
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn declared_oversize_rejects_before_reading() {
        struct MustNotRead;
        impl Read for MustNotRead {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("rejected size must not read");
            }
        }
        assert!(read_bounded_font(MustNotRead, MAX_FONT_BYTES + 1).is_err());
        assert!(read_bounded_font(MustNotRead, 0).is_err());
    }
}

#[cfg(all(test, windows))]
mod windows_symbol_font_tests {
    use super::*;
    // Append inside existing accessible tests; root Windows-only actual run.
    #[test]
    #[cfg(windows)]
    fn actual_windows_symbol_font_rasterizes_smile_without_replacement() {
        let bytes = read_font_path(PathBuf::from("C:/Windows/Fonts/seguisym.ttf"))
            .expect("owned Windows test requires installed Symbol font");
        let ctx = egui::Context::default();
        configure_context(
            &ctx,
            StartupFonts {
                korean: None,
                system_ui: None,
                symbols: Some(bytes),
            },
        );
        for _ in 0..2 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("🙂");
                });
            });
        }
        ctx.fonts_mut(|fonts| {
            for font in [
                egui::FontId::proportional(13.0),
                egui::FontId::monospace(13.0),
            ] {
                assert!(
                    fonts.has_glyph(&font, '🙂'),
                    "real fallback must not use missing-glyph substitution"
                );
                let galley = fonts.layout_no_wrap("🙂".into(), font, egui::Color32::WHITE);
                assert!(!galley.elided);
                assert_eq!(galley.rows[0].glyphs[0].chr, '🙂');
                assert!(
                    !galley.rows[0].visuals.mesh.vertices.is_empty(),
                    "actual outline produces a raster mesh"
                );
            }
        });
    }
}

#[cfg(test)]
mod download_progress_ui_tests {
    use super::*;
    #[test]
    fn actual_progress_scope_unknown_transfer_and_processing_at_small_viewports() {
        let cases = [
            (None, false, c::DOWNLOAD_PROGRESS_WAITING, None),
            (
                Some(r#"{"speed":1500000,"downloaded":25,"total":100}"#),
                false,
                c::DOWNLOAD_PROGRESS,
                Some("25.0%"),
            ),
            (
                Some(r#"{"downloaded":25,"estimated_total":100}"#),
                false,
                c::DOWNLOAD_PROGRESS_ESTIMATED,
                Some("25.0%"),
            ),
            (
                Some(r#"{"downloaded":25,"estimated_total":100.5}"#),
                false,
                c::DOWNLOAD_PROGRESS_ESTIMATED,
                Some("24.8%"),
            ),
            (
                Some(r#"{"downloaded":1,"estimated_total":1.5}"#),
                false,
                c::DOWNLOAD_PROGRESS_ESTIMATED,
                Some("66.6%"),
            ),
            (
                Some(r#"{"downloaded":100,"total":100,"status":"finished"}"#),
                false,
                c::DOWNLOAD_PROGRESS_FINISHING,
                Some("100.0%"),
            ),
            (
                Some(r#"{"downloaded":18446744073709551614,"total":18446744073709551615}"#),
                false,
                c::DOWNLOAD_PROGRESS,
                Some("99.9%"),
            ),
            (None, true, c::STATE_POSTPROCESSING, None),
        ];
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                for width in [800.0, 960.0] {
                    for (json, processing, label, percent) in cases {
                        let sample = json
                            .map(|json| dlpad_engine::metrics::parse_ytdlp_progress(json).unwrap());
                        let ctx = egui::Context::default();
                        ctx.set_visuals(if dark {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        });
                        crate::ui_theme::configure_dlpad_theme(&ctx);
                        ctx.set_zoom_factor(zoom);
                        let mut last_output = None;
                        for _ in 0..2 {
                            let output = ctx.run(
                                egui::RawInput {
                                    screen_rect: Some(egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        egui::vec2(width, 600.0),
                                    )),
                                    ..Default::default()
                                },
                                |ctx| {
                                    egui::TopBottomPanel::bottom("progress-actions-fixture").show(
                                        ctx,
                                        |ui| {
                                            show_download_progress(ui, sample.as_ref(), processing);
                                            crate::ui_components::secondary(ui, c::CANCEL, true);
                                        },
                                    );
                                },
                            );
                            last_output = Some(output);
                        }
                        let output = last_output.unwrap();
                        let texts = output
                            .shapes
                            .iter()
                            .filter_map(|shape| match &shape.shape {
                                egui::epaint::Shape::Text(text) => Some(text.galley.text()),
                                _ => None,
                            })
                            .collect::<Vec<_>>();
                        assert!(texts.contains(&label), "{texts:?}");
                        assert!(texts.contains(&c::CANCEL), "{texts:?}");
                        if let Some(percent) = percent {
                            assert!(texts.contains(&percent), "{texts:?}");
                        } else {
                            assert!(!texts.iter().any(|text| text.ends_with('%')));
                        }
                        assert!(!texts.contains(&c::DOWNLOADED));
                        if sample
                            .as_ref()
                            .is_some_and(|sample| sample.bytes_per_second == Some(1500000.0))
                        {
                            assert!(texts.contains(&"1.5 MB/s"));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod executable_context_reveal_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
    use std::collections::{HashMap, HashSet};
    struct Harness {
        ctx: egui::Context,
        value: String,
        unrelated_value: String,
        width: f32,
        height: f32,
        time: f64,
        enabled: bool,
        available: bool,
        nodes: HashMap<NodeId, Node>,
        root: Option<NodeId>,
        offset: f32,
        geometry: Option<ExecutableContextGeometry>,
    }
    impl Harness {
        fn new(width: f32, height: f32, zoom: f32) -> Self {
            let ctx = egui::Context::default();
            configure_context(&ctx, font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.enable_accesskit();
            let mut h = Self {
                ctx,
                value: "/synthetic/한글.exe".into(),
                unrelated_value: "unrelated field".into(),
                width,
                height,
                time: 0.0,
                enabled: true,
                available: true,
                nodes: Default::default(),
                root: None,
                offset: 0.0,
                geometry: None,
            };
            h.frame(vec![]);
            h
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += 0.05;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.unrelated_value)
                                .id(egui::Id::new("executable-context-unrelated-input")),
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(self.width, self.height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(self.width);
                                let output = egui::ScrollArea::vertical()
                                    .id_salt("executable-field-context-body")
                                    .max_height(self.height)
                                    .min_scrolled_height(0.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.add_space(160.0);
                                        let returned = ui
                                            .add_enabled_ui(self.enabled, |ui| {
                                                executable_field(
                                                    ui,
                                                    c::FFMPEG_PATH,
                                                    &mut self.value,
                                                    self.available,
                                                )
                                            })
                                            .inner;
                                        assert_eq!(
                                            returned,
                                            (false, false),
                                            "focus and scroll cannot edit or dispatch browse"
                                        );
                                        ui.add_space(240.0);
                                    });
                                assert!(
                                    (output.inner_rect.height() - self.height).abs() < 0.01,
                                    "actual bounded ScrollArea viewport before interaction"
                                );
                                self.offset = output.state.offset.y;
                            },
                        );
                    });
                },
            );
            self.geometry = self
                .ctx
                .data(|data| data.get_temp(egui::Id::new("executable-context-geometry")));
            let update = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("actual tree");
            if let Some(tree) = &update.tree {
                self.root = Some(tree.root);
            }
            self.nodes.extend(update.nodes.iter().cloned());
            let mut live = HashSet::new();
            let mut stack = vec![self.root.unwrap()];
            while let Some(id) = stack.pop() {
                assert!(live.len() < 256);
                if live.insert(id) {
                    stack.extend(self.nodes[&id].children().iter().copied());
                }
            }
            self.nodes.retain(|id, _| live.contains(id));
            output
        }
        fn action(&self, target: usize) -> egui::Event {
            let mut found = self.nodes.iter().filter(|(id, node)| match target {
                0 => node.role() == Role::Button && node.label() == Some(c::FFMPEG_HELP_TITLE),
                1 => {
                    node.role() == Role::TextInput
                        && **id
                            == NodeId(
                                egui::Id::new(("download-environment-input", c::FFMPEG_PATH))
                                    .value(),
                            )
                }
                _ => node.role() == Role::Button && node.label() == Some(c::TOOL_BROWSE_FFMPEG),
            });
            let (id, _) = found.next().expect("actual current focus target");
            assert!(found.next().is_none());
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: *id,
                data: None,
            })
        }
    }
    fn group(g: ExecutableContextGeometry) -> egui::Rect {
        g.label
            .union(g.input.0)
            .union(g.browse.0)
            .union(g.help.unwrap().0)
    }
    fn target(g: ExecutableContextGeometry, index: usize) -> (egui::Rect, egui::Id) {
        match index {
            0 => g.help.unwrap(),
            1 => g.input,
            _ => g.browse,
        }
    }
    #[test]
    fn actual_executable_focus_reveals_fitting_label_help_input_browse_group_once() {
        for zoom in [1.0, 2.0] {
            for focused in 0..3 {
                let mut h = Harness::new(320.0, 96.0, zoom);
                let before = h.geometry.unwrap();
                assert!(
                    !before.viewport.contains_rect(group(before)),
                    "actual initially clipped context"
                );
                let event = h.action(focused);
                h.frame(vec![event]);
                let mut output = h.frame(vec![]);
                for _ in 0..32 {
                    output = h.frame(vec![]);
                }
                let after = h.geometry.unwrap();
                assert!(
                    after.viewport.contains_rect(group(after)),
                    "fitting group must be fully exposed for help, input and browse focus; target={focused} zoom={zoom} group={:?} viewport={:?}",
                    group(after),
                    after.viewport
                );
                assert_eq!(after.input.1, before.input.1);
                assert_eq!(after.browse.1, before.browse.1);
                assert_eq!(after.help.unwrap().1, before.help.unwrap().1);
                assert_eq!(
                    after.input.1,
                    egui::Id::new(("download-environment-input", c::FFMPEG_PATH))
                );
                assert!(
                    h.ctx
                        .memory(|memory| memory.has_focus(target(after, focused).1))
                );
                let input = h.nodes.get(&NodeId(after.input.1.value())).unwrap();
                assert!(
                    !input.labelled_by().is_empty(),
                    "real accessible label association retained"
                );
                assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text()==c::FFMPEG_PATH && shape.clip_rect.contains_rect(text.visual_bounding_rect()))), "actual Korean purpose label painter is fully clipped in body");
                let revealed = h.offset;
                // Native pointer motion and wheel are distinct dispatches. Settle
                // hover on the real editable control so a focus-help tooltip can
                // yield before the wheel is delivered to the surrounding body.
                let point = after.input.0.center();
                for _ in 0..2 {
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                }
                let input_response = h.ctx.read_response(after.input.1).unwrap();
                let actual_pointer = h.ctx.input(|input| input.pointer.interact_pos());
                let top_layer = h.ctx.layer_id_at(point);
                let body_hit = h
                    .ctx
                    .rect_contains_pointer(input_response.layer_id, after.viewport);
                eprintln!(
                    "executable-wheel-before zoom={zoom} focus={focused} point={point:?} pointer={actual_pointer:?} input_layer={:?} top_layer={top_layer:?} body_hit={body_hit} offset={} revealed={revealed}",
                    input_response.layer_id, h.offset
                );
                assert_eq!(
                    actual_pointer,
                    Some(point),
                    "real pointer arrival before wheel"
                );
                assert!(
                    body_hit,
                    "actual body layer must receive pointer before wheel; focus={focused} zoom={zoom}"
                );
                assert!(
                    (h.offset - revealed).abs() < 0.01,
                    "hover transport itself cannot scroll or refocus the body"
                );
                h.frame(vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 6.0),
                    modifiers: egui::Modifiers::NONE,
                }]);
                let manual = h.offset;
                eprintln!(
                    "executable-wheel-after zoom={zoom} focus={focused} offset={manual} raw={:?} smooth_remaining={:?} pointer={:?} top_layer={:?}",
                    h.ctx.input(|input| input.raw_scroll_delta),
                    h.ctx.input(|input| input.smooth_scroll_delta),
                    h.ctx.input(|input| input.pointer.interact_pos()),
                    h.ctx.layer_id_at(point)
                );
                assert_eq!(
                    h.ctx.input(|input| input.raw_scroll_delta),
                    egui::vec2(0.0, 6.0),
                    "same six-point wheel actually reached Context"
                );
                assert!(
                    manual < revealed - 1.0,
                    "real wheel must remain usable after context reveal"
                );
                for _ in 0..4 {
                    h.frame(vec![]);
                }
                assert!(
                    (h.offset - manual).abs() < 0.01,
                    "held focus does not repeatedly snap to group"
                );
                assert_eq!(h.value, "/synthetic/한글.exe");
                let unrelated = egui::Id::new("executable-context-unrelated-input");
                let unrelated_node = NodeId(unrelated.value());
                assert_eq!(h.nodes[&unrelated_node].role(), Role::TextInput);
                assert!(h.nodes[&unrelated_node].supports_action(Action::Focus));
                h.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target: unrelated_node,
                    data: None,
                })]);
                for _ in 0..4 {
                    h.frame(vec![]);
                }
                assert!(h.ctx.memory(|memory| memory.has_focus(unrelated)));
                assert!(
                    (h.offset - manual).abs() < 0.01,
                    "moving focus to another real field must not snap this scroll body"
                );
                assert_eq!(h.unrelated_value, "unrelated field");
            }
        }
    }
    #[test]
    fn actual_executable_oversized_group_reveals_only_focused_control_and_refuses_disabled() {
        for zoom in [1.0, 2.0] {
            for focused in 0..3 {
                let mut h = Harness::new(200.0, 56.0, zoom);
                let event = h.action(focused);
                h.frame(vec![event]);
                for _ in 0..32 {
                    h.frame(vec![]);
                }
                let after = h.geometry.unwrap();
                assert!(
                    group(after).height() > after.viewport.height()
                        || group(after).width() > after.viewport.width()
                );
                assert!(
                    after.viewport.contains_rect(target(after, focused).0),
                    "oversized context still exposes entire focused control; target={focused} zoom={zoom}"
                );
                assert!(
                    h.ctx
                        .memory(|memory| memory.has_focus(target(after, focused).1))
                );
                assert_eq!(h.value, "/synthetic/한글.exe");
            }
        }
        let mut h = Harness::new(320.0, 96.0, 1.0);
        h.enabled = false;
        h.frame(vec![]);
        let offset = h.offset;
        let event = h.action(1);
        h.frame(vec![event]);
        for _ in 0..4 {
            h.frame(vec![]);
        }
        assert_eq!(h.offset, offset);
        assert!(
            !h.ctx
                .memory(|memory| memory.has_focus(h.geometry.unwrap().input.1))
        );
        assert_eq!(h.value, "/synthetic/한글.exe");
        let mut h = Harness::new(320.0, 96.0, 1.0);
        h.available = false;
        h.frame(vec![]);
        let offset = h.offset;
        let event = h.action(2);
        h.frame(vec![event]);
        for _ in 0..4 {
            h.frame(vec![]);
        }
        assert_eq!(h.offset, offset);
        assert!(
            !h.ctx
                .memory(|memory| memory.has_focus(h.geometry.unwrap().browse.1))
        );
    }
}
