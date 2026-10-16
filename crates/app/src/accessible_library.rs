//! Bounded library panel; the single worker owns filesystem/SQLite/watch I/O.
use crate::catalog as c;
use dlpad_engine::{
    file_export::{self, ProgressFeed},
    library::{LibraryEntry, LibraryError},
    location::{self, LocationRequest},
    store::Store,
    watch::{WatchCause, WatchHealth, WatchSession},
};
use eframe::egui;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::{Duration, Instant},
};
const PAGE_SIZE: usize = 128;
// Commands wake this wait immediately; scan checkpoints retain their 100ms cap.
const IDLE_WATCH_POLL: Duration = Duration::from_millis(500);
// Root metadata polling has a separate cadence from command/CPU checkpoints.
const SCAN_WATCH_POLL: Duration = Duration::from_millis(500);
const MAX_ROOT_BYTES: usize = 32 * 1024;

#[derive(Debug)]
enum Action {
    Export {
        root: PathBuf,
        row: LibraryEntry,
        parent: PathBuf,
        name: String,
        progress: ProgressFeed,
        cancel: Arc<AtomicBool>,
    },
    Refresh,
    Next(PathBuf),
    Search(crate::list_search::Request, Option<PathBuf>),
    Apply(PathBuf),
    Default,
    StopWake,
    SystemOpen {
        root: PathBuf,
        row: LibraryEntry,
        kind: crate::library_open::Kind,
    },
}
struct Command {
    serial: u64,
    action: Action,
}
fn signal_stop(stop: &AtomicBool, commands: &SyncSender<Command>, serial: u64) {
    // Publish cancellation before waking an idle receiver. A full channel already
    // has work to wake it; disconnected receivers require no extra wake.
    stop.store(true, Ordering::Release);
    let _ = commands.try_send(Command {
        serial,
        action: Action::StopWake,
    });
}
struct Snapshot {
    search: crate::list_search::Request,
    opened: Option<Result<crate::library_open::Report, crate::library_open::Error>>,
    serial: Option<u64>,
    root: Option<PathBuf>,
    rows: Result<Arc<Vec<LibraryEntry>>, String>,
    health: Option<WatchHealth>,
}
// Shares the bounded page with the mailbox/UI; never duplicates path payloads.
struct DeliveredState {
    search: crate::list_search::Request,
    root: PathBuf,
    rows: Result<Arc<Vec<LibraryEntry>>, String>,
    health: Option<WatchHealth>,
}
type ExportResult = Result<file_export::Exported, file_export::Failure>;
type ExportTerminal = (u64, ExportResult);
struct Signals {
    stop: AtomicBool,
    stopped: AtomicBool,
    failed: AtomicBool,
    context: Mutex<Option<egui::Context>>,
    export_terminal: Mutex<Option<ExportTerminal>>,
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
    barrier: Option<Arc<crate::library_scan_closewait_fixture::LibraryScanBarrier>>,
}
impl Signals {
    fn wake(&self) {
        if let Some(ctx) = self
            .context
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            ctx.request_repaint();
        }
    }
}
pub(crate) struct LibraryPanel {
    root: PathBuf,
    edit: String,
    search: crate::list_search::Search,
    rows: Arc<Vec<LibraryEntry>>,
    error: Option<String>,
    watch_failed: bool,
    commands: SyncSender<Command>,
    results: Receiver<Snapshot>,
    signals: Arc<Signals>,
    pending: Option<u64>,
    serial: u64,
    pending_refresh: bool,
    applied: Option<(u64, PathBuf)>,
    location_request_started: Option<u64>,
    browse_requested: bool,
    selected_media: Option<PathBuf>,
    preview_requested: Option<PathBuf>,
    system_open_status: Option<crate::library_open::Report>,
    last_focused_row: Option<egui::Id>,
    selected_row_id: Option<egui::Id>,
    path_detail: Option<PathDetail>,
    library_offset: f32,
    selected_row_offset: f32,
    detail_restore: Option<(f32, egui::Id)>,
    export_draft: Option<ExportDraft>,
    export_generation: u64,
    export_browse: Option<(u64, PathBuf)>,
    export_progress: Option<ProgressFeed>,
    export_cancel: Option<Arc<AtomicBool>>,
    export_result: Option<ExportResult>,
    export_result_notified: bool,
}
impl LibraryPanel {
    pub(crate) fn new(data: PathBuf, root: PathBuf) -> Self {
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
            export_terminal: Mutex::new(None),
            #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
            barrier: crate::library_scan_closewait_fixture::configured(),
        });
        let worker_signals = signals.clone();
        let worker_root = root.clone();
        // No join in UI or Drop: stop is acknowledged after any in-flight scan ends.
        let started = thread::Builder::new()
            .name("dlpad-library".into())
            .spawn(move || {
                worker_entry(worker_signals.clone(), || {
                    run_worker(data, worker_root, rx, tx, worker_signals);
                });
            });
        let mut panel = Self {
            edit: crate::path_picker::display_path(&root),
            root,
            rows: Arc::default(),
            error: None,
            watch_failed: false,
            commands,
            results,
            signals,
            pending: None,
            serial: 0,
            pending_refresh: false,
            search: Default::default(),
            applied: None,
            location_request_started: None,
            browse_requested: false,
            selected_media: None,
            preview_requested: None,
            system_open_status: None,
            last_focused_row: None,
            selected_row_id: None,
            path_detail: None,
            library_offset: 0.0,
            selected_row_offset: 0.0,
            detail_restore: None,
            export_draft: None,
            export_generation: 0,
            export_browse: None,
            export_progress: None,
            export_cancel: None,
            export_result: None,
            export_result_notified: false,
        };
        if started.is_err() {
            panel.error = Some(c::LOCATION_FAILED.into());
            panel.signals.stopped.store(true, Ordering::Release);
        } else if !panel.root.as_os_str().is_empty() {
            panel.request(Action::Refresh);
        }
        panel
    }
    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub(crate) fn background_export_status(&self) -> Option<String> {
        let cancel = self.export_cancel.as_ref()?;
        if !self.busy() {
            return None;
        }
        if cancel.load(Ordering::Acquire) {
            return Some(c::LIBRARY_EXPORT_STOPPING.to_owned());
        }
        Some(
            self.export_progress
                .as_ref()
                .and_then(ProgressFeed::latest)
                .map_or_else(
                    || c::LIBRARY_EXPORT_PREPARING.to_owned(),
                    |p| {
                        let percent = if p.total == 0 {
                            0.0
                        } else {
                            100.0 * p.bytes.min(p.total) as f64 / p.total as f64
                        };
                        format!("{} · {:.0}%", export_phase(p.phase), percent)
                    },
                ),
        )
    }
    pub(crate) fn take_export_notice(&mut self) -> Option<&'static str> {
        if self.export_result_notified {
            return None;
        }
        let notice = match self.export_result.as_ref()? {
            Ok(_) => c::LIBRARY_EXPORT_DONE,
            Err(failure) => match export_failure_text(failure) {
                c::LIBRARY_EXPORT_RETAINED => c::LIBRARY_EXPORT_RETAINED_NOTICE,
                notice => notice,
            },
        };
        self.export_result_notified = true;
        Some(notice)
    }
    pub(crate) fn request_stop(&mut self) {
        self.pending_refresh = false;
        if let Some(cancel) = &self.export_cancel {
            cancel.store(true, Ordering::Release);
        }
        self.export_browse = None;
        self.export_draft = None;
        signal_stop(&self.signals.stop, &self.commands, self.serial);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire)
    }
    fn request(&mut self, action: Action) {
        self.search.cancel_result_reveal();
        if self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        if self.busy() {
            if matches!(action, Action::Refresh) {
                self.pending_refresh = true;
            }
            return;
        }
        let Some(serial) = self.serial.checked_add(1) else {
            self.error = Some(c::LOCATION_FAILED.into());
            return;
        };
        let location_change = matches!(&action, Action::Apply(_) | Action::Default);
        match self.commands.try_send(Command { serial, action }) {
            Ok(()) => {
                self.serial = serial;
                self.pending = Some(serial);
                if location_change {
                    self.location_request_started = Some(serial);
                }
            }
            Err(TrySendError::Full(_)) => self.pending_refresh = true,
            Err(TrySendError::Disconnected(_)) => self.error = Some(c::LOCATION_FAILED.into()),
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        *self
            .signals
            .context
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(ctx.clone());
        let terminal = self
            .signals
            .export_terminal
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some((serial, result)) = terminal
            && self.pending == Some(serial)
        {
            // Preserve cancellation/uncertainty receipt even after Stop; no scan mailbox wait.
            self.pending = None;
            self.export_progress = None;
            self.export_cancel = None;
            self.export_result = Some(result);
            self.export_result_notified = false;
        }
        if self.signals.failed.load(Ordering::Acquire) {
            self.search.cancel_result_reveal();
            self.pending = None;
            self.pending_refresh = false;
            self.error = Some(c::LOCATION_FAILED.into());
            return;
        }
        // Capacity one: drain boundedly, never accumulate watch events or snapshots.
        if let Ok(snapshot) = self.results.try_recv()
            && (snapshot.serial.is_none() || snapshot.serial == self.pending)
        {
            if snapshot.serial.is_some() {
                self.pending = None;
            }
            if let Some(result) = snapshot.opened {
                // Never route an OS handoff to internal preview or overwrite current rows/root.
                match result {
                    Ok(report) => {
                        self.system_open_status = Some(report);
                        self.error = None;
                    }
                    Err(error) => {
                        self.system_open_status = None;
                        self.error = Some(system_open_error(error).into());
                    }
                }
                self.dispatch_search();
                if !self.busy() && std::mem::take(&mut self.pending_refresh) {
                    self.request(Action::Refresh);
                }
                return;
            }
            if let Some(root) = snapshot.root {
                self.root = root.clone();
                self.edit = crate::path_picker::display_path(&root);
                if let Some(serial) = snapshot.serial {
                    self.applied = Some((serial, root));
                }
                self.export_draft = None;
                self.export_browse = None;
                self.rows = Arc::default();
                self.selected_media = None;
                self.preview_requested = None;
                self.system_open_status = None;
            }
            if let Some(health) = snapshot.health {
                self.watch_failed = health == WatchHealth::Unavailable;
            }
            if self.search.accepts(&snapshot.search) {
                match snapshot.rows {
                    Ok(rows) => {
                        if self.selected_media.as_ref().is_some_and(|selected| {
                            !rows.iter().any(|entry| &entry.path == selected)
                        }) {
                            self.selected_media = None;
                            self.preview_requested = None;
                        }
                        self.rows = rows;
                        self.error = None;
                        self.search.published(&snapshot.search);
                        if !self.rows.is_empty() {
                            self.search.cancel_result_reveal();
                        }
                    }
                    Err(error) => {
                        self.search.cancel_result_reveal();
                        self.error = Some(error);
                    }
                }
            }
        }
        self.dispatch_search();
        if !self.busy() && std::mem::take(&mut self.pending_refresh) {
            self.request(Action::Refresh);
        }
        if self.busy() || (self.signals.stop.load(Ordering::Acquire) && !self.stopped()) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn dispatch_search(&mut self) {
        if !self.busy()
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped()
            && let Some(request) = self.search.queued.take()
        {
            self.request(Action::Search(request, None));
        }
    }
    fn preview_allowed(&self, enabled: bool) -> bool {
        enabled && !self.busy() && !self.stopped() && !self.signals.stop.load(Ordering::Acquire)
    }
    fn request_preview(&mut self, path: PathBuf, enabled: bool) {
        if self.preview_allowed(enabled) && self.rows.iter().any(|row| row.path == path) {
            self.preview_requested = Some(path);
        }
    }
    fn request_system_open(&mut self, kind: crate::library_open::Kind, enabled: bool) {
        if !self.preview_allowed(enabled) {
            return;
        }
        let Some(row) = self
            .selected_media
            .as_ref()
            .and_then(|path| self.rows.iter().find(|row| &row.path == path))
            .cloned()
        else {
            return;
        };
        if kind == crate::library_open::Kind::File && !crate::library_open::supports_file(&row.path)
        {
            return;
        }
        self.system_open_status = None;
        self.request(Action::SystemOpen {
            root: self.root.clone(),
            row,
            kind,
        });
    }
    fn begin_export(&mut self, enabled: bool) {
        if !self.preview_allowed(enabled) {
            return;
        }
        let Some(row) = self
            .selected_media
            .as_ref()
            .and_then(|p| self.rows.iter().find(|row| &row.path == p))
            .cloned()
        else {
            return;
        };
        let Some(parent) = row.path.parent().map(Path::to_path_buf) else {
            return;
        };
        let Some(generation) = self.export_generation.checked_add(1) else {
            return;
        };
        self.export_generation = generation;
        let stem = row
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(c::LIBRARY_EXPORT_FALLBACK);
        let extension = row.path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let name = format!("{stem} {}.{extension}", c::LIBRARY_EXPORT_COPY_SUFFIX);
        self.export_draft = Some(ExportDraft {
            root: self.root.clone(),
            row,
            parent,
            name,
            generation,
        });
        self.export_browse = None;
        self.export_result = None;
    }
    pub(crate) fn take_export_browse_request(&mut self) -> Option<(u64, PathBuf)> {
        self.export_browse.take()
    }
    pub(crate) fn apply_export_folder(&mut self, generation: u64, folder: PathBuf) {
        if !self.preview_allowed(true) {
            return;
        }
        let current = self
            .export_draft
            .as_ref()
            .is_some_and(|d| d.generation == generation && self.export_draft_current(d));
        if current && let Some(draft) = self.export_draft.as_mut() {
            draft.parent = folder;
        }
    }
    fn export_draft_current(&self, draft: &ExportDraft) -> bool {
        self.root == draft.root
            && self.selected_media.as_ref() == Some(&draft.row.path)
            && self.rows.iter().any(|row| row == &draft.row)
    }
    fn submit_export(&mut self, enabled: bool) {
        if !self.preview_allowed(enabled) {
            return;
        }
        let Some(draft) = self
            .export_draft
            .as_ref()
            .filter(|d| self.export_draft_current(d) && d.valid_name())
        else {
            return;
        };
        let progress = ProgressFeed::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let action = Action::Export {
            root: draft.root.clone(),
            row: draft.row.clone(),
            parent: draft.parent.clone(),
            name: draft.name.clone(),
            progress: progress.clone(),
            cancel: cancel.clone(),
        };
        self.request(action);
        if self.busy() {
            self.export_progress = Some(progress);
            self.export_cancel = Some(cancel);
            self.export_result = None;
            self.export_draft = None;
            self.export_browse = None;
        }
    }
    fn show_export(&mut self, ui: &mut egui::Ui, enabled: bool, picker_available: bool) {
        let current = self
            .export_draft
            .as_ref()
            .is_some_and(|d| self.export_draft_current(d));
        let active = self.preview_allowed(enabled) && current;
        let idle = !self.busy();
        let mut submit = false;
        let mut cancel = false;
        if let Some(draft) = &mut self.export_draft {
            ui.separator();
            ui.label(c::LIBRARY_EXPORT_TITLE);
            ui.weak(c::LIBRARY_EXPORT_PRESERVE);
            if !current {
                ui.label(c::LIBRARY_EXPORT_STALE);
            }
            let label = ui.label(c::LIBRARY_EXPORT_NAME);
            let width = (ui.available_width() - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                .max(32.0);
            let _ = ui
                .add_enabled_ui(active, |ui| {
                    crate::ui_components::text_input_with_id(
                        ui,
                        &mut draft.name,
                        1024,
                        width,
                        egui::Id::new("library-export-name"),
                    )
                })
                .inner
                .labelled_by(label.id);
            if !draft.valid_name() {
                ui.label(c::LIBRARY_EXPORT_INVALID_NAME);
            }
            ui.label(c::LIBRARY_EXPORT_FOLDER);
            ui.add(egui::Label::new(crate::path_picker::display_path(&draft.parent)).wrap());
            if crate::ui_components::secondary(ui, c::CHOOSE_FOLDER, active && picker_available)
                .clicked()
            {
                self.export_browse = Some((draft.generation, draft.parent.clone()));
            }
            ui.horizontal_wrapped(|ui| {
                submit = crate::ui_components::primary(
                    ui,
                    c::LIBRARY_EXPORT_SUBMIT,
                    active && draft.valid_name(),
                )
                .clicked();
                cancel = crate::ui_components::ghost(ui, c::LIBRARY_EXPORT_CANCEL, idle).clicked();
            });
        }
        if cancel {
            self.export_draft = None;
            self.export_browse = None;
        }
        if submit {
            self.submit_export(enabled);
        }
        if let Some(progress) = self.export_progress.as_ref().and_then(ProgressFeed::latest) {
            ui.label(export_phase(progress.phase));
            ui.add(egui::ProgressBar::new(if progress.total == 0 {
                0.0
            } else {
                (progress.bytes as f64 / progress.total as f64) as f32
            }));
            ui.label(format!("{} / {} bytes", progress.bytes, progress.total));
        } else if self.export_progress.is_some() {
            ui.label(c::LIBRARY_EXPORT_PREPARING);
        }
        if let Some(cancel) = &self.export_cancel {
            if crate::ui_components::secondary(
                ui,
                c::LIBRARY_EXPORT_STOP,
                !cancel.load(Ordering::Acquire),
            )
            .clicked()
            {
                cancel.store(true, Ordering::Release);
            }
            if cancel.load(Ordering::Acquire) {
                ui.weak(c::LIBRARY_EXPORT_STOPPING);
            }
        }
        if let Some(result) = &self.export_result {
            match result {
                Ok(report) => {
                    ui.label(c::LIBRARY_EXPORT_DONE);
                    ui.add(egui::Label::new(crate::path_picker::display_path(&report.path)).wrap());
                }
                Err(failure) => {
                    ui.label(export_failure_text(failure));
                    // Names are observations, not delete capabilities. No cleanup button here.
                    if let Some(path) = &failure.stage_path {
                        ui.add(egui::Label::new(crate::path_picker::display_path(path)).wrap());
                    }
                    if let Some(path) = &failure.target_path {
                        ui.add(egui::Label::new(crate::path_picker::display_path(path)).wrap());
                    }
                }
            }
        }
    }
    pub(crate) fn take_preview_request(&mut self) -> Option<PathBuf> {
        self.preview_requested.take()
    }
    pub(crate) fn take_browse_request(&mut self) -> Option<PathBuf> {
        std::mem::take(&mut self.browse_requested).then(|| self.root.clone())
    }
    pub(crate) fn apply_selected_root(&mut self, root: PathBuf) {
        self.request(Action::Apply(root));
    }
    /// Restore an observed root without granting a new user output intent.
    pub(crate) fn apply_observed_root(&mut self, root: PathBuf) {
        let explicit_started = self.location_request_started;
        self.request(Action::Apply(root));
        self.location_request_started = explicit_started;
    }
    /// Render only the existing worker-backed location editor. Main owns the stable settings scope.
    pub(crate) fn show_location_settings(
        &mut self,
        ui: &mut egui::Ui,
        can_change_root: bool,
        picker_available: bool,
    ) {
        let active = !self.signals.stop.load(Ordering::Acquire) && !self.stopped();
        let editable = can_change_root && !self.busy() && active;
        if crate::path_picker::folder_field(ui, &mut self.edit, editable, picker_available) {
            self.browse_requested = true;
        }
        ui.horizontal_wrapped(|ui| {
            if crate::ui_components::secondary(
                ui,
                c::APPLY_ROOT,
                editable && self.edit.len() <= MAX_ROOT_BYTES,
            )
            .clicked()
            {
                let path = if self.edit == crate::path_picker::display_path(&self.root) {
                    self.root.clone()
                } else {
                    PathBuf::from(&self.edit)
                };
                self.request(Action::Apply(path));
            }
            if crate::ui_components::ghost(ui, c::DEFAULT_ROOT, editable).clicked() {
                self.request(Action::Default);
            }
        });
        if self.busy() {
            ui.weak(c::LIBRARY_LOADING);
        } else if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
            ui.weak(c::LIBRARY_RETRY);
        }
    }
    /// Consume a committed worker acknowledgement regardless of which screen is rendered.
    pub(crate) fn take_applied_root(&mut self) -> Option<(u64, PathBuf)> {
        self.applied.take()
    }
    /// Only an accepted Apply/Default enqueue grants a request-start observation.
    pub(crate) fn take_location_request_started(&mut self) -> Option<u64> {
        self.location_request_started.take()
    }
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        can_change_root: bool,
        picker_available: bool,
    ) {
        let mut scroll = egui::ScrollArea::vertical()
            .id_salt("library-page")
            .min_scrolled_height(0.0);
        if let Some((offset, _)) = self.detail_restore {
            scroll = scroll.vertical_scroll_offset(offset);
        }
        let output = scroll.show_viewport(ui, |ui, viewport| {
            let page_origin = ui.min_rect().top();
            let active = !self.signals.stop.load(Ordering::Acquire) && !self.stopped();
            if !active {
                self.search.cancel_result_reveal();
            }
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(c::SAVE_ROOT).small().weak());
                let full = crate::path_picker::display_path(&self.root);
                let location = ui
                    .add(egui::Label::new(&full).truncate())
                    .on_hover_text(&full);
                ui.ctx().accesskit_node_builder(location.id, |node| {
                    node.set_description(full.as_str())
                });
            });
            // Empty-folder guidance precedes search/action controls in the same scroll body.
            // Small viewports still teach where files come from before optional browsing controls.
            let empty_hint = active
                && !self.busy()
                && self.error.is_none()
                && self.rows.is_empty()
                && self.search.current.query.is_empty();
            if empty_hint {
                let empty = ui.label(c::LIBRARY_EMPTY);
                self.search.reveal_result(ui, &empty);
                ui.weak(c::LIBRARY_EMPTY_HELP);
            }
            self.search
                .show(ui, "library-search", c::LIBRARY_SEARCH_LABEL, active);
            self.dispatch_search();
            ui.horizontal_wrapped(|ui| {
                if crate::ui_components::secondary(ui, c::REFRESH, active && !self.busy()).clicked()
                {
                    self.request(Action::Refresh);
                }
                if crate::ui_components::ghost(
                    ui,
                    c::NEXT,
                    active && !self.busy() && !self.search.stale() && self.rows.len() == PAGE_SIZE,
                )
                .clicked()
                    && let Some(last) = self.rows.last()
                {
                    self.request(Action::Next(last.path.clone()));
                }
            });
            if self.watch_failed {
                ui.label(c::WATCH_FAILED);
            }
            ui.add_space(crate::ui_tokens::CONTROL_GAP);
            ui.separator();
            self.show_export(ui, can_change_root, picker_available);
            if self.busy() {
                if self.export_progress.is_none() {
                    ui.weak(c::LIBRARY_LOADING);
                }
            } else if let Some(error) = &self.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
                ui.weak(c::LIBRARY_RETRY);
            } else if self.rows.is_empty() && active && !empty_hint {
                if self.search.current.query.is_empty() {
                    ui.label(c::LIBRARY_EMPTY);
                    ui.weak(c::LIBRARY_EMPTY_HELP);
                } else {
                    let empty = ui.label(c::LIST_SEARCH_EMPTY);
                    self.search.reveal_result(ui, &empty);
                }
            }
            // A confirmed empty result has no preview target. Keep the
            // selection instruction for populated, loading and error states.
            let confirmed_empty = active
                && !self.busy()
                && self.error.is_none()
                && self.rows.is_empty()
                && self.selected_media.is_none();
            if !confirmed_empty {
                ui.horizontal_wrapped(|ui| {
                    if crate::ui_components::secondary(
                        ui,
                        c::LIBRARY_PREVIEW,
                        active && can_change_root && !self.busy() && self.selected_media.is_some(),
                    )
                    .clicked()
                        && let Some(path) = self.selected_media.clone()
                    {
                        self.request_preview(path, can_change_root);
                    }
                    if crate::ui_components::secondary(
                        ui,
                        c::LIBRARY_PATH_DETAIL_ACTION,
                        active
                            && !self.busy()
                            && self.selected_media.is_some()
                            && self.selected_row_id.is_some(),
                    )
                    .clicked()
                        && let (Some(path), Some(row)) =
                            (self.selected_media.clone(), self.selected_row_id)
                    {
                        self.path_detail =
                            Some(PathDetail::new(path, row, self.selected_row_offset));
                    }
                    if crate::ui_components::secondary(
                        ui,
                        c::LIBRARY_EXPORT_ACTION,
                        self.preview_allowed(can_change_root)
                            && self.selected_media.is_some()
                            && cfg!(windows),
                    )
                    .clicked()
                    {
                        self.begin_export(can_change_root);
                    }
                    let selected = self
                        .selected_media
                        .as_ref()
                        .and_then(|path| self.rows.iter().find(|row| &row.path == path));
                    let file_available =
                        selected.is_some_and(|row| crate::library_open::supports_file(&row.path));
                    let enabled = self.preview_allowed(can_change_root)
                        && selected.is_some()
                        && crate::library_open::available();
                    if crate::ui_components::secondary(
                        ui,
                        c::LIBRARY_OPEN_DEFAULT,
                        enabled && file_available,
                    )
                    .on_disabled_hover_text(if !crate::library_open::available() {
                        c::LIBRARY_OPEN_UNSUPPORTED
                    } else if selected.is_some() && !file_available {
                        c::LIBRARY_OPEN_EXTENSION
                    } else {
                        c::LIBRARY_SELECT_HELP
                    })
                    .clicked()
                    {
                        self.request_system_open(crate::library_open::Kind::File, can_change_root);
                    }
                    if crate::ui_components::secondary(ui, c::LIBRARY_OPEN_FOLDER, enabled)
                        .clicked()
                    {
                        self.request_system_open(
                            crate::library_open::Kind::Folder,
                            can_change_root,
                        );
                    }
                });
                ui.weak(c::LIBRARY_SELECT_HELP);
                let selected = self
                    .selected_media
                    .as_ref()
                    .and_then(|path| self.rows.iter().find(|row| &row.path == path));
                selection_summary(ui, selected);
                if !crate::library_open::available() {
                    ui.weak(c::LIBRARY_OPEN_UNSUPPORTED);
                }
                ui.weak(c::LIBRARY_FILE_ACTION_HELP);
                if let Some(report) = self.system_open_status {
                    ui.label(system_open_report(report));
                }
            }
            let row_origin = ui.cursor().top();
            let stride = crate::ui_tokens::CONTROL_HEIGHT + ui.spacing().item_spacing.y;
            let offset = row_origin - page_origin;
            let min_row = ((viewport.min.y - offset).max(0.0) / stride).floor() as usize;
            let max_row = ((viewport.max.y - offset).max(0.0) / stride).ceil() as usize + 1;
            let range = min_row.min(self.rows.len())..max_row.min(self.rows.len());
            let height = (stride * self.rows.len() as f32 - ui.spacing().item_spacing.y).max(0.0);
            let rect = egui::Rect::from_min_max(
                egui::pos2(
                    ui.max_rect().left(),
                    row_origin + range.start as f32 * stride,
                ),
                egui::pos2(
                    ui.max_rect().right(),
                    row_origin + range.end as f32 * stride,
                ),
            );
            let previous_focused_row = self.last_focused_row;
            let mut current_focused_row = None;
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                for entry in &self.rows[range] {
                    let selected = self.selected_media.as_ref() == Some(&entry.path);
                    let response = ui
                        .scope_builder(
                            // push_id's auto seed includes the visible-loop ordinal;
                            // a fixed child scope keeps this path's Button ID stable.
                            egui::UiBuilder::new()
                                .id(ui.make_persistent_id(("library-media-row", &entry.path))),
                            |ui| {
                                ui.with_layout(
                                    egui::Layout::top_down_justified(egui::Align::LEFT),
                                    |ui| {
                                        // Retain fixed virtual-row height and the complete reader label.
                                        ui.style_mut().wrap_mode =
                                            Some(egui::TextWrapMode::Truncate);
                                        ui.add_enabled_ui(
                                            active && can_change_root && !self.busy(),
                                            |ui| {
                                                crate::ui_components::choice(
                                                    ui,
                                                    selected,
                                                    &format!(
                                                        "{} · {} bytes · {}",
                                                        entry
                                                            .path
                                                            .file_name()
                                                            .unwrap_or_default()
                                                            .to_string_lossy(),
                                                        entry.size,
                                                        c::LOCAL
                                                    ),
                                                )
                                            },
                                        )
                                        .inner
                                    },
                                )
                                .inner
                            },
                        )
                        .inner
                        .on_hover_text(entry.path.to_string_lossy());
                    let response = if response.enabled() {
                        response.on_hover_cursor(egui::CursorIcon::PointingHand)
                    } else {
                        response
                    };
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        // Reader label remains basename + size + LOCAL; full path is secondary context.
                        node.set_description(entry.path.to_string_lossy());
                        node.set_toggled(if selected {
                            egui::accesskit::Toggled::True
                        } else {
                            egui::accesskit::Toggled::False
                        });
                    });
                    if self.detail_restore.is_some_and(|(_, id)| id == response.id) {
                        response.request_focus();
                        self.detail_restore = None;
                    }
                    if response.has_focus() {
                        if self.selected_row_id == Some(response.id) {
                            self.selected_row_offset = self.library_offset;
                        }
                        current_focused_row = Some(response.id);
                    }
                    // Cardinal focus changes at egui end_pass, so gained_focus
                    // alone cannot observe Arrow navigation on the next frame.
                    // Reveal a newly observed focused path once; a later wheel
                    // gesture with the same focus must not be pulled back.
                    if response.gained_focus()
                        || (response.has_focus() && previous_focused_row != Some(response.id))
                    {
                        response.scroll_to_me(None);
                    }
                    if response.clicked() {
                        self.selected_media = Some(entry.path.clone());
                        self.selected_row_id = Some(response.id);
                        self.selected_row_offset = self.library_offset;
                        response.request_focus();
                    }
                    if response.enabled()
                        && self.preview_allowed(can_change_root)
                        && (response.double_clicked()
                            || (response.has_focus()
                                && ui.input(|input| input.key_pressed(egui::Key::Enter))))
                    {
                        self.selected_media = Some(entry.path.clone());
                        self.selected_row_id = Some(response.id);
                        self.selected_row_offset = self.library_offset;
                        // Mutate only the existing request field while the immutable rows slice is borrowed.
                        self.preview_requested = Some(entry.path.clone());
                    }
                }
            });
            self.last_focused_row = current_focused_row;
            ui.set_min_height((row_origin - page_origin + height).max(0.0));
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new("search-library-body-observation"),
                (output.inner_rect, output.content_size, output.state.offset),
            )
        });
        self.library_offset = output.state.offset.y;
        if self
            .path_detail
            .as_mut()
            .is_some_and(|detail| detail.show(ui.ctx()))
            && let Some(detail) = self.path_detail.take()
            && self.rows.iter().any(|row| row.path == detail.path)
        {
            self.detail_restore = Some((detail.return_offset, detail.return_focus));
        }
    }
}
struct ExportDraft {
    root: PathBuf,
    row: LibraryEntry,
    parent: PathBuf,
    name: String,
    generation: u64,
}
impl ExportDraft {
    fn valid_name(&self) -> bool {
        self.parent.is_absolute()
            && self
                .row
                .path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|extension| file_export::validate_name(&self.name, extension).is_ok())
    }
}
fn export_phase(phase: file_export::Phase) -> &'static str {
    match phase {
        file_export::Phase::Copy => c::LIBRARY_EXPORT_COPYING,
        file_export::Phase::VerifyStage => c::LIBRARY_EXPORT_VERIFY_STAGE,
        file_export::Phase::VerifySource => c::LIBRARY_EXPORT_VERIFY_SOURCE,
        file_export::Phase::VerifyFinal => c::LIBRARY_EXPORT_VERIFY_FINAL,
        file_export::Phase::Complete => c::LIBRARY_EXPORT_VERIFY_FINAL,
    }
}
fn export_failure_text(failure: &file_export::Failure) -> &'static str {
    use file_export::{Cleanup, Creation, Publication};
    if matches!(
        failure.publication,
        Publication::Unconfirmed | Publication::PublishedUnverified
    ) || matches!(failure.creation, Creation::AttemptedUnconfirmed)
        || matches!(
            failure.cleanup,
            Cleanup::Unconfirmed | Cleanup::OriginalRetained
        )
    {
        return c::LIBRARY_EXPORT_RETAINED;
    }
    match failure.error.kind() {
        std::io::ErrorKind::Interrupted => c::LIBRARY_EXPORT_CANCELLED,
        std::io::ErrorKind::TimedOut => c::LIBRARY_EXPORT_TIMEOUT,
        std::io::ErrorKind::AlreadyExists => c::LIBRARY_EXPORT_COLLISION,
        _ => c::LIBRARY_EXPORT_FAILED,
    }
}

/// One fixed row in both states so first selection cannot move the click target.
fn selection_summary(ui: &mut egui::Ui, selected: Option<&LibraryEntry>) {
    let width = ui.available_width().max(1.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, crate::ui_tokens::CONTROL_HEIGHT),
        egui::Sense::hover(),
    );
    let (name, metadata, description) = if let Some(entry) = selected {
        let basename = entry.path.file_name().unwrap_or_default().to_string_lossy();
        let metadata = format!(
            "{} bytes · {} {}",
            entry.size,
            c::LIBRARY_MODIFIED,
            modified_time(entry.modified_ms)
        );
        (
            format!("{} {}", c::LIBRARY_SELECTED_FILE, basename),
            metadata.clone(),
            format!("{} · {}", entry.path.to_string_lossy(), metadata),
        )
    } else {
        (" ".to_owned(), " ".to_owned(), String::new())
    };
    let metadata_width = if selected.is_some() {
        ui.painter()
            .layout_no_wrap(
                metadata.clone(),
                egui::TextStyle::Small.resolve(ui.style()),
                ui.visuals().weak_text_color(),
            )
            .size()
            .x
            .min(width * 0.45)
    } else {
        0.0
    };
    let name_width = (width - metadata_width - crate::ui_tokens::CONTROL_GAP).max(0.0);
    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .id(ui.make_persistent_id("library-selection-summary"))
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    content.set_clip_rect(rect.intersect(ui.clip_rect()));
    content.spacing_mut().item_spacing.x = crate::ui_tokens::CONTROL_GAP;
    let cell_layout = egui::Layout::left_to_right(egui::Align::Center)
        .with_main_align(egui::Align::Min)
        .with_main_justify(true)
        .with_cross_justify(true);
    let name = content
        .allocate_ui_with_layout(
            egui::vec2(name_width, crate::ui_tokens::CONTROL_HEIGHT),
            cell_layout,
            |cell| cell.add(egui::Label::new(name).truncate().halign(egui::Align::Min)),
        )
        .inner;
    let metadata_text = metadata.clone();
    let metadata = content
        .allocate_ui_with_layout(
            egui::vec2(metadata_width, crate::ui_tokens::CONTROL_HEIGHT),
            cell_layout,
            |cell| {
                cell.add(
                    egui::Label::new(egui::RichText::new(metadata).small().weak())
                        .truncate()
                        .halign(egui::Align::Min),
                )
            },
        )
        .inner;
    if selected.is_some() {
        let name = name.on_hover_text(&description);
        let metadata = metadata.on_hover_text(&description);
        ui.ctx().accesskit_node_builder(name.id, |node| {
            node.set_description(description.as_str());
        });
        ui.ctx().accesskit_node_builder(metadata.id, |node| {
            node.set_description(metadata_text.as_str());
        });
    }
}

fn modified_time(value: Option<i64>) -> String {
    let Some(ms) = value.filter(|ms| *ms >= 0) else {
        return c::LIBRARY_MODIFIED_UNKNOWN.to_owned();
    };
    let before = |year: i64| {
        let y = year - 1;
        365 * y + y / 4 - y / 100 + y / 400
    };
    let absolute = ms / 86_400_000 + before(1970);
    if absolute >= before(10000) {
        return c::LIBRARY_MODIFIED_UNKNOWN.to_owned();
    }
    let (mut low, mut high) = (1970, 10000);
    while high - low > 1 {
        let middle = (low + high) / 2;
        if before(middle) <= absolute {
            low = middle
        } else {
            high = middle
        }
    }
    let year = low;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut day = absolute - before(year);
    let mut month = 1;
    for length in months {
        if day < length {
            break;
        }
        day -= length;
        month += 1;
    }
    let seconds = (ms / 1000) % 86_400;
    format!(
        "{year:04}-{month:02}-{:02} {:02}:{:02} UTC",
        day + 1,
        seconds / 3600,
        seconds / 60 % 60
    )
}

// UI-only candidate; integrate privately in accessible_library.rs.
struct PathDetail {
    path: std::path::PathBuf,
    return_focus: egui::Id,
    return_offset: f32,
    offset: f32,
    max_offset: f32,
    page_height: f32,
}
impl PathDetail {
    fn new(path: std::path::PathBuf, return_focus: egui::Id, return_offset: f32) -> Self {
        Self {
            path,
            return_focus,
            return_offset,
            offset: 0.0,
            max_offset: 0.0,
            page_height: 0.0,
        }
    }
    // true means closed; caller removes state AFTER this returns.
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let screen = ctx.content_rect();
        let width = (screen.width() - 64.0).clamp(1.0, 560.0);
        let height = (screen.height() - 140.0).clamp(1.0, 360.0);
        let requested = ctx.input_mut(|input| {
            if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                Some(0.0)
            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                Some(self.max_offset)
            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::PageDown) {
                Some((self.offset + self.page_height * 0.8).min(self.max_offset))
            } else if input.consume_key(egui::Modifiers::NONE, egui::Key::PageUp) {
                Some((self.offset - self.page_height * 0.8).max(0.0))
            } else {
                None
            }
        });
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::new("library-path-detail")).show(ctx, |ui| {
            ui.set_width(width);
            ui.label(c::LIBRARY_PATH_DETAIL_TITLE);
            ui.weak(c::LIBRARY_PATH_DETAIL_KEYS);
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt("library-path-detail-text")
                .max_height(height)
                .min_scrolled_height(0.0);
            if let Some(offset) = requested {
                scroll = scroll.vertical_scroll_offset(offset);
            }
            let output = scroll.show(ui, |ui| {
                ui.set_max_width(width);
                // No truncation/editing/selection mutation; all path bytes are retained.
                ui.add(egui::Label::new(self.path.to_string_lossy()).wrap());
            });
            self.offset = output.state.offset.y;
            self.page_height = output.inner_rect.height();
            self.max_offset = (output.content_size.y - self.page_height).max(0.0);
            close = crate::ui_components::ghost(ui, c::LIBRARY_PATH_DETAIL_CLOSE, true).clicked();
        });
        close || modal.should_close()
    }
}

impl Drop for LibraryPanel {
    fn drop(&mut self) {
        self.request_stop();
    }
}
fn scan(
    data: &Path,
    root: &Path,
    after: Option<&Path>,
    cancel: &AtomicBool,
) -> Result<Vec<LibraryEntry>, String> {
    scan_inner(
        data,
        root,
        after,
        cancel,
        #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
        None,
    )
}
// Page navigation reads the last committed snapshot; it does not request a scan.
fn read_page(
    data: &Path,
    root: &Path,
    after: &Path,
    cancel: &AtomicBool,
) -> Result<Vec<LibraryEntry>, String> {
    read_search_page(data, root, Some(after), "", cancel)
}
fn read_search_page(
    data: &Path,
    root: &Path,
    after: Option<&Path>,
    query: &str,
    cancel: &AtomicBool,
) -> Result<Vec<LibraryEntry>, String> {
    if cancel.load(Ordering::Acquire) {
        return Err(c::LOCATION_FAILED.into());
    }
    validate_library_paths(data, root)?;
    let rows = Store::read_library_search_page_readonly(
        &data.join("library.sqlite"),
        root,
        after,
        query,
        PAGE_SIZE,
    )
    .map_err(|_| c::LOCATION_FAILED.to_owned())?;
    if cancel.load(Ordering::Acquire) {
        return Err(c::LOCATION_FAILED.into());
    }
    Ok(rows)
}
fn search_after_scan(
    rows: Result<Vec<LibraryEntry>, String>,
    data: &Path,
    root: &Path,
    query: &str,
    cancel: &AtomicBool,
) -> Result<Vec<LibraryEntry>, String> {
    let rows = rows?;
    if query.is_empty() {
        Ok(rows)
    } else {
        read_search_page(data, root, None, query, cancel)
    }
}
fn scan_with_signals(
    data: &Path,
    root: &Path,
    after: Option<&Path>,
    signals: &Signals,
) -> Result<Vec<LibraryEntry>, String> {
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
    if let Some(barrier) = &signals.barrier
        && !barrier.periodic()
    {
        return scan_inner(data, root, after, &signals.stop, Some(barrier));
    }
    scan(data, root, after, &signals.stop)
}
fn scan_inner(
    data: &Path,
    root: &Path,
    after: Option<&Path>,
    cancel: &AtomicBool,
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))] barrier: Option<
        &crate::library_scan_closewait_fixture::LibraryScanBarrier,
    >,
) -> Result<Vec<LibraryEntry>, String> {
    let mut store = open_library_store(data, root)?;
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
    let reconciled = if let Some(barrier) = barrier {
        store.reconcile_library_after_first_observation(root, cancel, || {
            barrier
                .after_first_observation()
                .map_err(dlpad_engine::library::LibraryError::Io)
        })
    } else {
        store.reconcile_library_with_cancel(root, cancel)
    };
    #[cfg(not(all(windows, feature = "library-scan-closewait-fixture")))]
    let reconciled = store.reconcile_library_with_cancel(root, cancel);
    reconciled.map_err(|_| c::LOCATION_FAILED.to_owned())?;
    store
        .library_page(root, after, PAGE_SIZE)
        .map_err(|_| c::LOCATION_FAILED.to_owned())
}
fn open_library_store(data: &Path, root: &Path) -> Result<Store, String> {
    validate_library_paths(data, root)?;
    Store::open(&data.join("library.sqlite")).map_err(|_| c::LOCATION_FAILED.to_owned())
}
fn validate_library_paths(data: &Path, root: &Path) -> Result<(), String> {
    if !root.is_absolute() {
        return Err(c::INVALID_ROOT.into());
    }
    let actual_data = data
        .canonicalize()
        .map_err(|_| c::DATA_REQUIRED.to_owned())?;
    let metadata = std::fs::symlink_metadata(root).map_err(|_| c::LOCATION_FAILED.to_owned())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(c::LOCATION_FAILED.into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(c::LOCATION_FAILED.into());
        }
    }
    let actual_root = root
        .canonicalize()
        .map_err(|_| c::LOCATION_FAILED.to_owned())?;
    if actual_root.starts_with(&actual_data) || actual_data.starts_with(&actual_root) {
        return Err(c::SEPARATE_DATA.into());
    }
    Ok(())
}
fn prepare(data: &Path, action: Action) -> Result<PathBuf, String> {
    let request = match action {
        Action::Apply(root) => LocationRequest::Selected(root),
        Action::Default => LocationRequest::Default,
        _ => unreachable!("only explicit location commands prepare paths"),
    };
    let fixture = cfg!(test) || std::env::args().any(|arg| arg == "--fixture");
    let test_documents = std::env::var_os("DLPAD_TEST_DOCUMENTS");
    if fixture && matches!(request, LocationRequest::Default) && test_documents.is_none() {
        return Err(c::FIXTURE_DOCUMENTS.into());
    }
    let result = if let Some(documents) = test_documents {
        location::prepare_with_default(data, request, || Ok(PathBuf::from(documents).join("dlpad")))
    } else {
        location::prepare(data, request)
    };
    result.map_err(|_| c::LOCATION_FAILED.into())
}
fn send(tx: &SyncSender<Snapshot>, mut value: Snapshot, signals: &Signals) -> bool {
    loop {
        if signals.stop.load(Ordering::Acquire) {
            return false;
        }
        match tx.try_send(value) {
            Ok(()) => {
                signals.wake();
                return true;
            }
            Err(TrySendError::Disconnected(_)) => return false,
            Err(TrySendError::Full(pending)) => {
                value = pending;
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}
fn send_if_changed(
    tx: &SyncSender<Snapshot>,
    value: Snapshot,
    actual_root: &Path,
    last: &mut Option<DeliveredState>,
    signals: &Signals,
) -> bool {
    if signals.stop.load(Ordering::Acquire) {
        return false;
    }
    // Explicit responses must acknowledge the command even when values match.
    if value.serial.is_none()
        && value.root.is_none()
        && last.as_ref().is_some_and(|prior| {
            prior.root.as_path() == actual_root
                && prior.search == value.search
                && prior.rows == value.rows
                && prior.health == value.health
        })
    {
        return true;
    }
    let delivered = DeliveredState {
        search: value.search.clone(),
        root: actual_root.to_path_buf(),
        rows: value.rows.clone(),
        health: value.health,
    };
    if !send(tx, value, signals) {
        return false;
    }
    // A full/stopped/disconnected mailbox never advances the delivered key.
    *last = Some(delivered);
    true
}
// The acknowledgment cannot depend on a successful filesystem/SQLite loop.
// Unwinding drops the worker's owned resources before the stop acknowledgment.
struct StopAck(Arc<Signals>);
impl Drop for StopAck {
    fn drop(&mut self) {
        #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
        if let Some(barrier) = &self.0.barrier
            && barrier.acknowledge_worker_stopped().is_err()
        {
            self.0.failed.store(true, Ordering::Release);
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanInterrupt {
    Stop,
    Disconnected,
    Command,
    Priority,
    Deadline,
}
#[derive(Default)]
struct PendingScan {
    command: Option<Command>,
    cause: Option<WatchCause>,
    health: Option<WatchHealth>,
}
impl PendingScan {
    fn observe(&mut self, update: dlpad_engine::watch::WatchPoll) {
        if let Some(health) = update.health {
            self.health = Some(health);
        }
        if let Some(cause) = update.cause {
            self.cause = Some(self.cause.map_or(cause, |prior| prior.max(cause)));
        }
    }
}
struct PeriodicOutcome {
    rows: Result<Vec<LibraryEntry>, String>,
    interrupt: Option<ScanInterrupt>,
}
fn scan_periodic(
    data: &Path,
    root: &Path,
    watch: &mut WatchSession,
    rx: &Receiver<Command>,
    signals: &Signals,
    pending: &mut PendingScan,
) -> PeriodicOutcome {
    use dlpad_core::library_budget::{Budget, CpuBudget, CpuDecision, Decision};
    use dlpad_engine::thread_cpu::ThreadCpuClock;
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    let mut worker_phase = crate::worker_phase_fixture::Span::library_periodic();
    let mut interrupt = None;
    let result = (|| {
        let now = Instant::now();
        let mut budget = Budget::new(now);
        let mut clock = ThreadCpuClock::new().map_err(|_| c::LOCATION_FAILED.to_owned())?;
        let mut cpu_budget = match clock.as_mut() {
            Some(clock) => Some(CpuBudget::new(
                now,
                clock.sample().map_err(|_| c::LOCATION_FAILED.to_owned())?,
            )),
            None => None,
        };
        let mut cpu_due = now;
        let mut cpu_pause_until = now;
        let mut counter_failed = false;
        let mut store = open_library_store(data, root)?;
        let mut watch_due = now + SCAN_WATCH_POLL;
        let mut checkpoint = || {
            loop {
                if signals.stop.load(Ordering::Acquire) {
                    interrupt = Some(ScanInterrupt::Stop);
                    return true;
                }
                if pending.command.is_none() {
                    match rx.try_recv() {
                        Ok(command) => pending.command = Some(command),
                        Err(mpsc::TryRecvError::Disconnected) => {
                            interrupt = Some(ScanInterrupt::Disconnected);
                            return true;
                        }
                        Err(mpsc::TryRecvError::Empty) => (),
                    }
                }
                if pending.command.is_some() {
                    interrupt = Some(ScanInterrupt::Command);
                    return true;
                }
                let now = Instant::now();
                if now >= watch_due {
                    #[cfg(all(windows, feature = "first-paint-fixture"))]
                    worker_phase.watch_poll();
                    pending.observe(watch.poll());
                    watch_due = now + SCAN_WATCH_POLL;
                }
                if pending
                    .cause
                    .is_some_and(|cause| cause > WatchCause::Periodic)
                {
                    interrupt = Some(ScanInterrupt::Priority);
                    return true;
                }
                if budget.remaining(now).is_zero() {
                    interrupt = Some(ScanInterrupt::Deadline);
                    return true;
                }
                if now >= cpu_due {
                    cpu_due = now + Duration::from_millis(100);
                    if let (Some(clock), Some(cpu_budget)) = (&mut clock, &mut cpu_budget) {
                        let decision = clock
                            .sample()
                            .ok()
                            .and_then(|ticks| cpu_budget.at(now, ticks).ok());
                        match decision {
                            Some(CpuDecision::Continue) => cpu_pause_until = now,
                            Some(CpuDecision::Wait(duration)) => {
                                #[cfg(all(windows, feature = "first-paint-fixture"))]
                                worker_phase.cpu_debt(duration);
                                cpu_pause_until = now + duration;
                            }
                            None => {
                                counter_failed = true;
                                return true;
                            }
                        }
                    }
                }
                let cpu_wait = cpu_pause_until.saturating_duration_since(now);
                let cpu = if cpu_wait.is_zero() {
                    CpuDecision::Continue
                } else {
                    CpuDecision::Wait(cpu_wait)
                };
                let duration = match budget.at_with_cpu(now, cpu) {
                    Decision::Continue => return false,
                    Decision::Deadline => {
                        interrupt = Some(ScanInterrupt::Deadline);
                        return true;
                    }
                    Decision::Wait(duration) => duration,
                };
                match rx.recv_timeout(duration) {
                    Ok(command) => pending.command = Some(command),
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        interrupt = Some(ScanInterrupt::Disconnected);
                        return true;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                }
            }
        };
        #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
        let reconciled = if let Some(barrier) = &signals.barrier
            && barrier.periodic()
        {
            store.reconcile_library_with_checkpoint_after_first(
                root,
                &signals.stop,
                &mut checkpoint,
                || barrier.after_first_observation().map_err(LibraryError::Io),
            )
        } else {
            store.reconcile_library_with_checkpoint(root, &signals.stop, &mut checkpoint)
        };
        #[cfg(not(all(windows, feature = "library-scan-closewait-fixture")))]
        let reconciled =
            store.reconcile_library_with_checkpoint(root, &signals.stop, &mut checkpoint);
        if counter_failed {
            // A rejected native observation is a real failure, never expected preemption.
            interrupt = None;
            return Err(c::LOCATION_FAILED.to_owned());
        }
        match reconciled {
            Ok(_) => {
                #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
                if let Some(barrier) = &signals.barrier
                    && barrier.periodic()
                {
                    barrier
                        .acknowledge_committed()
                        .map_err(|_| c::LOCATION_FAILED.to_owned())?;
                }
                store
                    .library_page(root, None, PAGE_SIZE)
                    .map_err(|_| c::LOCATION_FAILED.to_owned())
            }
            Err(LibraryError::Cancelled) => {
                // Atomic cancellation can short-circuit the policy callback.
                if signals.stop.load(Ordering::Acquire) {
                    interrupt = Some(ScanInterrupt::Stop);
                }
                Err(c::LOCATION_FAILED.to_owned())
            }
            Err(_) => {
                // Genuine I/O/DB failures never masquerade as expected preemption.
                interrupt = None;
                Err(c::LOCATION_FAILED.to_owned())
            }
        }
    })();
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    worker_phase.finish(
        result.is_err()
            && !matches!(
                interrupt,
                Some(
                    ScanInterrupt::Stop
                        | ScanInterrupt::Disconnected
                        | ScanInterrupt::Command
                        | ScanInterrupt::Priority
                )
            ),
    );
    PeriodicOutcome {
        rows: result,
        interrupt,
    }
}

fn system_open_error(error: crate::library_open::Error) -> &'static str {
    use crate::library_open::Error;
    match error {
        Error::NoAssociation => c::LIBRARY_OPEN_NO_ASSOCIATION,
        Error::Unsupported => c::LIBRARY_OPEN_UNSUPPORTED,
        Error::Cancelled => c::LIBRARY_OPEN_CANCELLED,
        Error::TimedOut => c::LIBRARY_OPEN_TIMEOUT,
        Error::AutomationBlocked => c::LIBRARY_OPEN_AUTOMATION_BLOCKED,
        Error::Stale | Error::Unsafe => c::LIBRARY_OPEN_CHANGED,
        Error::Dispatch => c::LIBRARY_OPEN_FAILED,
    }
}
fn system_open_report(report: crate::library_open::Report) -> &'static str {
    use crate::library_open::Report;
    match report {
        Report::HandedOff => c::LIBRARY_OPEN_HANDED_OFF,
        Report::AfterCancellation | Report::AfterDeadline => c::LIBRARY_OPEN_HANDED_OFF_LATE,
        Report::SourceChanged => c::LIBRARY_OPEN_HANDED_OFF_CHANGED,
    }
}
fn current_open_row(
    current_root: &Path,
    requested_root: &Path,
    row: &LibraryEntry,
    delivered: Option<&DeliveredState>,
) -> bool {
    current_root == requested_root
        && delivered.is_some_and(|state| {
            state.root == current_root
                && state
                    .rows
                    .as_ref()
                    .is_ok_and(|rows| rows.iter().any(|current| current == row))
        })
}
fn dispatch_system_open(
    data: &Path,
    current_root: &Path,
    requested_root: &Path,
    row: &LibraryEntry,
    kind: crate::library_open::Kind,
    delivered: Option<&DeliveredState>,
    stop: &AtomicBool,
) -> Result<crate::library_open::Report, crate::library_open::Error> {
    if !current_open_row(current_root, requested_root, row, delivered) {
        return Err(crate::library_open::Error::Stale);
    }
    validate_library_paths(data, current_root).map_err(|_| crate::library_open::Error::Unsafe)?;
    crate::library_open::dispatch(current_root, row, kind, stop)
}

fn run_worker(
    data: PathBuf,
    mut root: PathBuf,
    rx: Receiver<Command>,
    tx: SyncSender<Snapshot>,
    signals: Arc<Signals>,
) {
    let mut watch = (!root.as_os_str().is_empty()).then(|| WatchSession::new(root.clone()));
    let mut health = None;
    let mut delivered = None;
    let mut pending = PendingScan::default();
    let mut search = crate::list_search::Request::default();
    loop {
        if signals.stop.load(Ordering::Acquire) {
            break;
        }
        let command = if let Some(command) = pending.command.take() {
            Some(command)
        } else if pending.cause.is_some() {
            match rx.try_recv() {
                Ok(command) => Some(command),
                Err(mpsc::TryRecvError::Disconnected) => break,
                Err(mpsc::TryRecvError::Empty) => None,
            }
        } else {
            match rx.recv_timeout(IDLE_WATCH_POLL) {
                Ok(command) => Some(command),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => None,
            }
        };
        if pending.health.is_some() {
            health = pending.health.take();
        }
        if signals.stop.load(Ordering::Acquire) {
            break;
        }
        if let Some(command) = command {
            let command = match command.action {
                Action::Export {
                    root: requested_root,
                    row,
                    parent,
                    name,
                    progress,
                    cancel,
                } => {
                    let result =
                        if !current_open_row(&root, &requested_root, &row, delivered.as_ref())
                            || validate_library_paths(&data, &root).is_err()
                        {
                            Err(file_export::Failure {
                                error: std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    "stale library export source",
                                ),
                                creation: file_export::Creation::NotAttempted,
                                publication: file_export::Publication::NotAttempted,
                                cleanup: file_export::Cleanup::NotAttempted,
                                stage_path: None,
                                target_path: None,
                                created_identity: None,
                            })
                        } else {
                            if signals.stop.load(Ordering::Acquire) {
                                cancel.store(true, Ordering::Release);
                            }
                            file_export::export(
                                file_export::Request {
                                    source: row,
                                    output_parent: parent,
                                    output_name: name,
                                },
                                &cancel,
                                Instant::now() + Duration::from_secs(300),
                                &progress,
                            )
                        };
                    *signals
                        .export_terminal
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some((command.serial, result));
                    signals.wake();
                    continue;
                }
                Action::SystemOpen {
                    root: requested_root,
                    row,
                    kind,
                } => {
                    let result = dispatch_system_open(
                        &data,
                        &root,
                        &requested_root,
                        &row,
                        kind,
                        delivered.as_ref(),
                        &signals.stop,
                    );

                    let snapshot = Snapshot {
                        search: search.clone(),
                        opened: Some(result),
                        serial: Some(command.serial),
                        root: None,
                        rows: Ok(Arc::default()),
                        health: None,
                    };
                    if !send(&tx, snapshot, &signals) {
                        break;
                    }
                    continue;
                }
                action => Command {
                    serial: command.serial,
                    action,
                },
            };
            let mut selected = None;
            let rows = match command.action {
                // Internal wake-only commands never scan, select paths or ACK a
                // user serial. Normally the Stop bit above has already ended us.
                Action::StopWake => continue,
                Action::Export { .. } | Action::SystemOpen { .. } => {
                    unreachable!("handled before scan command")
                }
                Action::Refresh => search_after_scan(
                    scan_with_signals(&data, &root, None, &signals),
                    &data,
                    &root,
                    &search.query,
                    &signals.stop,
                ),
                Action::Next(after) => {
                    if search.query.is_empty() {
                        read_page(&data, &root, &after, &signals.stop)
                    } else {
                        read_search_page(&data, &root, Some(&after), &search.query, &signals.stop)
                    }
                }
                Action::Search(request, after) => {
                    search = request;
                    read_search_page(&data, &root, after.as_deref(), &search.query, &signals.stop)
                }
                action @ (Action::Apply(_) | Action::Default) => match prepare(&data, action) {
                    Ok(next) => {
                        root = next;
                        selected = Some(root.clone());
                        watch = Some(WatchSession::new(root.clone()));
                        health = None;
                        pending.cause = None;
                        pending.health = None;
                        search_after_scan(
                            scan_with_signals(&data, &root, None, &signals),
                            &data,
                            &root,
                            &search.query,
                            &signals.stop,
                        )
                    }
                    Err(error) => Err(error),
                },
            };
            let snapshot = Snapshot {
                search: search.clone(),
                opened: None,
                serial: Some(command.serial),
                root: selected,
                rows: rows.map(Arc::new),
                health,
            };
            if !send_if_changed(&tx, snapshot, &root, &mut delivered, &signals) {
                break;
            }
        } else if let Some(watch) = &mut watch {
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            crate::worker_phase_fixture::library_watch_poll();
            pending.observe(watch.poll());
            if pending.health.is_some() {
                health = pending.health.take();
            }
            let cause = pending.cause.take();
            if cause.is_some() {
                let rows = if cause == Some(WatchCause::Periodic) {
                    let outcome = scan_periodic(&data, &root, watch, &rx, &signals, &mut pending);
                    if pending.health.is_some() {
                        health = pending.health.take();
                    }
                    match outcome.interrupt {
                        Some(ScanInterrupt::Stop | ScanInterrupt::Disconnected) => break,
                        Some(ScanInterrupt::Command | ScanInterrupt::Priority) => {
                            pending.cause = Some(
                                pending
                                    .cause
                                    .unwrap_or(WatchCause::Periodic)
                                    .max(WatchCause::Periodic),
                            );
                            continue;
                        }
                        Some(ScanInterrupt::Deadline) | None => outcome.rows,
                    }
                } else {
                    scan_with_signals(&data, &root, None, &signals)
                };
                let rows = search_after_scan(rows, &data, &root, &search.query, &signals.stop);
                let snapshot = Snapshot {
                    search: search.clone(),
                    opened: None,
                    serial: None,
                    root: None,
                    rows: rows.map(Arc::new),
                    health,
                };
                if !send_if_changed(&tx, snapshot, &root, &mut delivered, &signals) {
                    break;
                }
            }
        }
    }
    drop(watch);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_wake_sets_cancellation_before_queueing() {
        let (tx, rx) = mpsc::sync_channel(1);
        let stop = AtomicBool::new(false);
        signal_stop(&stop, &tx, 7);
        let command = rx.try_recv().unwrap();
        assert!(stop.load(Ordering::Acquire));
        assert_eq!(command.serial, 7);
        assert!(matches!(command.action, Action::StopWake));
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
    #[test]
    fn stop_wake_preserves_full_channel_and_never_waits_for_space() {
        let (tx, rx) = mpsc::sync_channel(1);
        tx.try_send(Command {
            serial: 9,
            action: Action::Next(PathBuf::from("owned-page-key")),
        })
        .unwrap();
        let stop = AtomicBool::new(false);
        signal_stop(&stop, &tx, 10);
        assert!(stop.load(Ordering::Acquire));
        let original = rx.try_recv().unwrap();
        assert_eq!(original.serial, 9);
        let Action::Next(key) = original.action else {
            panic!("queued command replaced")
        };
        assert_eq!(key, PathBuf::from("owned-page-key"));
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
    #[test]
    fn stop_wake_with_disconnected_worker_still_cancels() {
        let (tx, rx) = mpsc::sync_channel(1);
        drop(rx);
        let stop = AtomicBool::new(false);
        signal_stop(&stop, &tx, 11);
        assert!(stop.load(Ordering::Acquire));
    }
    #[test]
    fn pending_priority_and_health_survive_coalesced_periodic() {
        let mut pending = PendingScan::default();
        pending.observe(dlpad_engine::watch::WatchPoll {
            dirty: true,
            cause: Some(WatchCause::ChangeHint),
            health: None,
        });
        pending.observe(dlpad_engine::watch::WatchPoll {
            dirty: true,
            cause: Some(WatchCause::Periodic),
            health: None,
        });
        assert_eq!(pending.cause, Some(WatchCause::ChangeHint));
        pending.observe(dlpad_engine::watch::WatchPoll {
            dirty: true,
            cause: Some(WatchCause::Recovery),
            health: Some(WatchHealth::Unavailable),
        });
        pending.observe(dlpad_engine::watch::WatchPoll::default());
        assert_eq!(pending.cause, Some(WatchCause::Recovery));
        assert_eq!(pending.health, Some(WatchHealth::Unavailable));
    }
    #[test]
    fn periodic_preemption_retains_next_serial_and_committed_rows() {
        let fixture = Fixture::new();
        let (data, root) = fixture.paths();
        std::fs::write(root.join("a.mp4"), b"old").unwrap();
        let cancel = AtomicBool::new(false);
        let prior = scan(&data, &root, None, &cancel).unwrap();
        std::fs::write(root.join("b.mp4"), b"new").unwrap();
        let mut watch = WatchSession::new(root.clone());
        watch.poll();
        let (sender, receiver) = mpsc::sync_channel(1);
        sender
            .send(Command {
                serial: 41,
                action: Action::Next(prior[0].path.clone()),
            })
            .unwrap();
        let mut pending = PendingScan::default();
        let outcome = scan_periodic(
            &data,
            &root,
            &mut watch,
            &receiver,
            &delivery_signals(),
            &mut pending,
        );
        assert_eq!(outcome.interrupt, Some(ScanInterrupt::Command));
        let command = pending.command.take().unwrap();
        assert_eq!(command.serial, 41);
        let Action::Next(after) = command.action else {
            panic!("next command lost")
        };
        assert!(read_page(&data, &root, &after, &cancel).unwrap().is_empty());
        assert_eq!(
            Store::read_library_page_readonly(&data.join("library.sqlite"), &root, None, 128)
                .unwrap(),
            prior
        );
        assert_eq!(std::fs::read(root.join("b.mp4")).unwrap(), b"new");
    }
    fn delivery_signals() -> Signals {
        Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
            export_terminal: Mutex::new(None),
            #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
            barrier: None,
        }
    }
    fn delivery_snapshot(
        serial: Option<u64>,
        root: Option<PathBuf>,
        rows: Result<Arc<Vec<LibraryEntry>>, String>,
        health: Option<WatchHealth>,
    ) -> Snapshot {
        Snapshot {
            search: Default::default(),
            opened: None,
            serial,
            root,
            rows,
            health,
        }
    }
    fn delivery_rows(name: &str) -> Arc<Vec<LibraryEntry>> {
        Arc::new(vec![LibraryEntry {
            path: PathBuf::from(name),
            size: 3,
            modified_ms: Some(7),
            file_identity: Some("synthetic-original-identity".into()),
        }])
    }
    #[test]
    fn autonomous_equal_content_does_not_queue_another_snapshot_or_request_repaint() {
        let (tx, rx) = mpsc::sync_channel(1);
        let signals = delivery_signals();
        let mut last = None;
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let context = egui::Context::default();
        let copied = counter.clone();
        context.set_request_repaint_callback(move |_| {
            copied.fetch_add(1, Ordering::SeqCst);
        });
        *signals.context.lock().unwrap() = Some(context);
        let root = Path::new("/owned");
        assert!(send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("a")), None),
            root,
            &mut last,
            &signals
        ));
        let first = rx.try_recv().unwrap();
        assert_eq!(first.rows.unwrap()[0].path, PathBuf::from("a"));
        let before = counter.load(Ordering::SeqCst);
        // Fresh Arc with exact equal page content, not pointer equality.
        assert!(send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("a")), None),
            root,
            &mut last,
            &signals
        ));
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        assert_eq!(counter.load(Ordering::SeqCst), before);
    }
    #[test]
    fn changed_rows_root_health_error_and_identical_command_ack_are_delivered() {
        let (tx, rx) = mpsc::sync_channel(1);
        let signals = delivery_signals();
        let mut last = None;
        let root = Path::new("/owned");
        for (actual, serial, root_ack, rows, health) in [
            (root, None, None, Ok(delivery_rows("a")), None),
            (root, None, None, Ok(delivery_rows("b")), None),
            (
                Path::new("/other"),
                None,
                None,
                Ok(delivery_rows("b")),
                None,
            ),
            (
                Path::new("/other"),
                None,
                None,
                Ok(delivery_rows("b")),
                Some(WatchHealth::Unavailable),
            ),
            (
                Path::new("/other"),
                None,
                None,
                Err("fixed failure".into()),
                Some(WatchHealth::Unavailable),
            ),
            (
                Path::new("/other"),
                Some(9),
                None,
                Err("fixed failure".into()),
                Some(WatchHealth::Unavailable),
            ),
            (
                Path::new("/other"),
                None,
                Some(PathBuf::from("/other")),
                Err("fixed failure".into()),
                Some(WatchHealth::Unavailable),
            ),
        ] {
            assert!(send_if_changed(
                &tx,
                delivery_snapshot(serial, root_ack.clone(), rows, health),
                actual,
                &mut last,
                &signals
            ));
            let received = rx.try_recv().unwrap();
            assert_eq!(received.serial, serial);
            assert_eq!(received.root, root_ack);
        }
    }
    #[test]
    fn same_path_metadata_changes_and_error_recovery_are_delivered() {
        let (tx, rx) = mpsc::sync_channel(1);
        let signals = delivery_signals();
        let mut last = None;
        let root = Path::new("/owned");
        let original = delivery_rows("a");
        for index in 0..5 {
            let mut rows = (*original).clone();
            match index {
                1 => rows[0].size += 1,
                2 => rows[0].modified_ms = Some(8),
                3 => rows[0].file_identity = Some("replacement".into()),
                4 => rows[0].modified_ms = None,
                _ => {}
            }
            let rows = Arc::new(rows);
            assert!(send_if_changed(
                &tx,
                delivery_snapshot(None, None, Ok(rows.clone()), Some(WatchHealth::Active)),
                root,
                &mut last,
                &signals
            ));
            assert_eq!(rx.try_recv().unwrap().rows, Ok(rows));
        }
        for (rows, health) in [
            (Err("scan failed".into()), WatchHealth::Unavailable),
            (Ok(original.clone()), WatchHealth::Unavailable),
            (Ok(original), WatchHealth::Active),
        ] {
            assert!(send_if_changed(
                &tx,
                delivery_snapshot(None, None, rows.clone(), Some(health)),
                root,
                &mut last,
                &signals
            ));
            let received = rx.try_recv().unwrap();
            assert_eq!(received.rows, rows);
            assert_eq!(received.health, Some(health));
        }
    }
    #[test]
    fn stopped_full_or_disconnected_delivery_never_replaces_last_success_key() {
        let (tx, rx) = mpsc::sync_channel(1);
        let signals = Arc::new(delivery_signals());
        let mut last = None;
        let root = Path::new("/owned");
        assert!(send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("a")), None),
            root,
            &mut last,
            &signals
        ));
        let first = rx.try_recv().unwrap();
        // Fill queue without altering last delivery key; stop is set by one owned helper.
        tx.try_send(first).ok().unwrap();
        let copied = signals.clone();
        let stopper = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            copied.stop.store(true, Ordering::Release);
        });
        assert!(!send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("b")), None),
            root,
            &mut last,
            &signals
        ));
        stopper.join().unwrap();
        rx.try_recv().unwrap();
        signals.stop.store(false, Ordering::Release);
        // Original a must still be duplicate; if failed b polluted last, a would enqueue.
        assert!(send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("a")), None),
            root,
            &mut last,
            &signals
        ));
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        drop(rx);
        assert!(!send_if_changed(
            &tx,
            delivery_snapshot(None, None, Ok(delivery_rows("b")), None),
            root,
            &mut last,
            &signals
        ));
        let (fresh_tx, fresh_rx) = mpsc::sync_channel(1);
        assert!(send_if_changed(
            &fresh_tx,
            delivery_snapshot(None, None, Ok(delivery_rows("a")), None),
            root,
            &mut last,
            &signals
        ));
        assert!(matches!(
            fresh_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }

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
                    "dlpad-panel-{}-{stamp}-{sequence}",
                    std::process::id()
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => return Self(path.canonicalize().unwrap()),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("fixture create failed: {error}"),
                }
            }
            panic!("fixture collision retry budget exceeded");
        }
        fn paths(&self) -> (PathBuf, PathBuf) {
            let data = self.0.join("data");
            let media = self.0.join("media");
            std::fs::create_dir(&data).unwrap();
            std::fs::create_dir(&media).unwrap();
            (data, media)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn wait(panel: &mut LibraryPanel, predicate: impl Fn(&LibraryPanel) -> bool) {
        let ctx = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            panel.poll(&ctx);
            if predicate(panel) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "bounded panel acknowledgment timed out"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(panel: &mut LibraryPanel) {
        panel.request_stop();
        wait(panel, LibraryPanel::stopped);
    }
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
    #[test]
    fn actual_scan_worker_waits_release_then_rolls_back_before_ack() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        std::fs::write(media.join("old.mp4"), b"old").unwrap();
        let baseline = scan(&data, &media, None, &AtomicBool::new(false)).unwrap();
        std::fs::write(media.join("new.mp4"), b"new").unwrap();
        let barrier = crate::library_scan_closewait_fixture::LibraryScanBarrier::testing().unwrap();
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
            export_terminal: Mutex::new(None),
            barrier: Some(barrier.clone()),
        });
        let work = signals.clone();
        let data_copy = data.clone();
        let media_copy = media.clone();
        let worker = thread::spawn(move || {
            worker_entry(work.clone(), || {
                assert!(scan_with_signals(&data_copy, &media_copy, None, &work).is_err());
            })
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !barrier.test_started() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        signals.stop.store(true, Ordering::Release);
        assert!(!signals.stopped.load(Ordering::Acquire));
        assert!(!barrier.test_ack());
        barrier.test_release().unwrap();
        while !signals.stopped.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        worker.join().unwrap();
        assert!(barrier.test_ack());
        assert!(!signals.failed.load(Ordering::Acquire));
        let db = Store::open(&data.join("library.sqlite")).unwrap();
        assert_eq!(db.library_page(&media, None, PAGE_SIZE).unwrap(), baseline);
        assert_eq!(std::fs::read(media.join("new.mp4")).unwrap(), b"new");
    }

    // Append inside accessible_library.rs's existing tests module.
    // Uses existing Fixture, scan, read_page and Store; no new dependency/schema.
    // Draft only: production root validates cancellation before Store::open and after page read.
    fn page_only_all_rows(data: &Path, root: &Path) -> Vec<LibraryEntry> {
        let store = Store::open(&data.join("library.sqlite")).unwrap();
        let mut rows = Vec::new();
        let mut after: Option<PathBuf> = None;
        for _ in 0..4 {
            let page = store
                .library_page(root, after.as_deref(), PAGE_SIZE)
                .unwrap();
            if page.is_empty() {
                return rows;
            }
            after = Some(page.last().unwrap().path.clone());
            rows.extend(page);
        }
        panic!("bounded synthetic snapshot exceeded four pages");
    }

    #[test]
    fn page_only_260_rows_have_exact_keysets_and_preserve_entire_snapshot() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let cancel = AtomicBool::new(false);
        for id in 0..260 {
            std::fs::write(media.join(format!("{id:04}.mp4")), b"synthetic").unwrap();
        }
        let first = scan(&data, &media, None, &cancel).unwrap();
        let baseline = page_only_all_rows(&data, &media);
        assert_eq!(baseline.len(), 260);
        // A fresh scan would observe both this new row and changed metadata.
        std::fs::write(media.join("9999.mp4"), b"new").unwrap();
        std::fs::write(&baseline[129].path, b"changed-length").unwrap();
        let second = read_page(&data, &media, &first.last().unwrap().path, &cancel).unwrap();
        let third = read_page(&data, &media, &second.last().unwrap().path, &cancel).unwrap();
        let end = read_page(&data, &media, &third.last().unwrap().path, &cancel).unwrap();
        assert_eq!(
            (first.len(), second.len(), third.len(), end.len()),
            (128, 128, 4, 0)
        );
        let traversed: Vec<_> = first.into_iter().chain(second).chain(third).collect();
        assert_eq!(traversed, baseline);
        assert!(traversed.windows(2).all(|pair| pair[0].path < pair[1].path));
        assert_eq!(page_only_all_rows(&data, &media), baseline);
        scan(&data, &media, None, &cancel).unwrap();
        let refreshed = page_only_all_rows(&data, &media);
        assert_eq!(refreshed.len(), 261);
        assert_eq!(refreshed[129].size, 14);
        assert!(refreshed.iter().any(|row| row.path.ends_with("9999.mp4")));
        assert_eq!(std::fs::read(media.join("9999.mp4")).unwrap(), b"new");
    }

    #[test]
    fn page_only_pre_stop_preserves_rows_and_does_not_create_missing_database() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        std::fs::write(media.join("a.mp4"), b"original").unwrap();
        let baseline = scan(&data, &media, None, &AtomicBool::new(false)).unwrap();
        assert!(read_page(&data, &media, &baseline[0].path, &AtomicBool::new(true)).is_err());
        assert_eq!(page_only_all_rows(&data, &media), baseline);
        let fresh_data = fixture.0.join("fresh-data");
        std::fs::create_dir(&fresh_data).unwrap();
        assert!(
            read_page(
                &fresh_data,
                &media,
                &baseline[0].path,
                &AtomicBool::new(true)
            )
            .is_err()
        );
        assert!(!fresh_data.join("library.sqlite").exists());
        assert_eq!(std::fs::read(media.join("a.mp4")).unwrap(), b"original");
    }

    #[test]
    fn page_only_corrupt_database_failure_preserves_original_bytes_and_media() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let db = data.join("library.sqlite");
        let corrupt = b"synthetic malformed sqlite, preserve this original";
        std::fs::write(&db, corrupt).unwrap();
        let original = media.join("sentinel.mp4");
        std::fs::write(&original, b"owned synthetic sentinel").unwrap();
        assert!(read_page(&data, &media, &original, &AtomicBool::new(false)).is_err());
        assert_eq!(std::fs::read(db).unwrap(), corrupt);
        assert_eq!(
            std::fs::read(original).unwrap(),
            b"owned synthetic sentinel"
        );
    }

    #[test]
    fn page_only_rejects_both_overlap_directions_before_database_creation() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let nested = data.join("nested-media");
        std::fs::create_dir(&nested).unwrap();
        let sentinel = nested.join("sentinel.mp4");
        std::fs::write(&sentinel, b"keep").unwrap();
        let cancel = AtomicBool::new(false);
        assert_eq!(
            read_page(&data, &nested, &sentinel, &cancel),
            Err(c::SEPARATE_DATA.into())
        );
        assert_eq!(
            read_page(&nested, &data, &sentinel, &cancel),
            Err(c::SEPARATE_DATA.into())
        );
        assert!(!data.join("library.sqlite").exists());
        assert!(!nested.join("library.sqlite").exists());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"keep");
        assert!(media.is_dir());
    }

    #[test]
    fn page_only_missing_and_empty_database_never_creates_or_migrates() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let sentinel = media.join("sentinel.mp4");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let cancel = AtomicBool::new(false);
        let db = data.join("library.sqlite");
        assert!(read_page(&data, &media, &sentinel, &cancel).is_err());
        assert!(!db.exists());
        std::fs::write(&db, b"").unwrap();
        assert!(read_page(&data, &media, &sentinel, &cancel).is_err());
        assert_eq!(std::fs::read(&db).unwrap(), b"");
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"preserve");
    }

    #[test]
    fn page_navigation_keeps_committed_snapshot_until_refresh() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let cancel = AtomicBool::new(false);
        for id in 0..140 {
            std::fs::write(media.join(format!("{id:04}.mp4")), b"synthetic").unwrap();
        }
        let first = scan(&data, &media, None, &cancel).unwrap();
        let after = &first.last().unwrap().path;
        let before = Store::open(&data.join("library.sqlite"))
            .unwrap()
            .library_page(&media, Some(after), PAGE_SIZE)
            .unwrap();
        assert_eq!(before.len(), 12);
        std::fs::write(media.join("9999.mp4"), b"new").unwrap();
        std::fs::write(&before[0].path, b"changed-length").unwrap();
        let page = read_page(&data, &media, after, &cancel).unwrap();
        assert_eq!(page, before);
        assert_eq!(
            Store::open(&data.join("library.sqlite"))
                .unwrap()
                .library_page(&media, Some(after), PAGE_SIZE)
                .unwrap(),
            before
        );
        let refreshed = scan(&data, &media, Some(after), &cancel).unwrap();
        assert_eq!(refreshed.len(), 13);
        assert_eq!(refreshed[0].size, 14);
        assert!(refreshed.iter().any(|row| row.path.ends_with("9999.mp4")));
        cancel.store(true, Ordering::Release);
        assert!(read_page(&data, &media, after, &cancel).is_err());
        assert_eq!(std::fs::read(media.join("9999.mp4")).unwrap(), b"new");
    }

    #[test]
    fn synthetic_pages_are_bounded_and_keyset_has_no_duplicates() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        for id in 0..260 {
            std::fs::write(media.join(format!("{id:04}.mp4")), b"synthetic").unwrap();
        }
        let first = scan(&data, &media, None, &AtomicBool::new(false)).unwrap();
        assert_eq!(first.len(), PAGE_SIZE);
        let next = read_page(
            &data,
            &media,
            &first.last().unwrap().path,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(next.len(), PAGE_SIZE);
        assert!(
            first
                .iter()
                .all(|entry| next.iter().all(|other| entry.path != other.path))
        );
        assert_eq!(
            read_page(
                &data,
                &media,
                &next.last().unwrap().path,
                &AtomicBool::new(false)
            )
            .unwrap()
            .len(),
            4
        );
    }
    #[test]
    fn failed_scan_preserves_visible_rows_and_database_snapshot() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        std::fs::write(media.join("합성.mp4"), b"synthetic").unwrap();
        let mut panel = LibraryPanel::new(data.clone(), media.clone());
        wait(&mut panel, |panel| !panel.busy() && panel.rows.len() == 1);
        let previous = panel.rows.clone();
        let disconnected = fixture.0.join("disconnected");
        std::fs::rename(&media, &disconnected).unwrap();
        panel.request(Action::Refresh);
        wait(&mut panel, |panel| !panel.busy() && panel.error.is_some());
        assert_eq!(panel.rows, previous);
        let db = Store::open(&data.join("library.sqlite")).unwrap();
        assert_eq!(db.library_page(&media, None, PAGE_SIZE).unwrap(), *previous);
        stop(&mut panel);
    }
    #[test]
    fn rejected_apply_preserves_root_rows_and_existing_media() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        std::fs::write(media.join("owned-fixture.mp4"), b"synthetic").unwrap();
        let mut panel = LibraryPanel::new(data.clone(), media.clone());
        wait(&mut panel, |panel| !panel.busy() && panel.rows.len() == 1);
        let previous = panel.rows.clone();
        let nested = data.join("must-not-create");
        panel.request(Action::Apply(nested.clone()));
        wait(&mut panel, |panel| !panel.busy() && panel.error.is_some());
        assert_eq!(panel.root, media);
        assert_eq!(panel.rows, previous);
        assert!(!nested.exists());
        assert_eq!(
            std::fs::read(panel.rows[0].path.clone()).unwrap(),
            b"synthetic"
        );
        stop(&mut panel);
    }
    #[test]
    fn successful_apply_commits_root_and_blocks_commands_after_stop() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let mut panel = LibraryPanel::new(data, media);
        wait(&mut panel, |panel| !panel.busy());
        let selected = fixture.0.join("새 위치");
        panel.request(Action::Apply(selected.clone()));
        wait(&mut panel, |panel| !panel.busy() && panel.applied.is_some());
        assert_eq!(panel.root, selected.canonicalize().unwrap());
        assert!(panel.rows.is_empty());
        stop(&mut panel);
        let serial = panel.serial;
        panel.request(Action::Refresh);
        assert_eq!(panel.serial, serial);
    }
    #[test]
    fn stop_ack_does_not_require_ui_to_drain_full_snapshot_channel() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let mut panel = LibraryPanel::new(data, media);
        thread::sleep(Duration::from_millis(200));
        panel.request_stop();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !panel.stopped() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    fn default_fixture_never_falls_back_to_user_documents() {
        // Environment variables are not changed by this parallel test.
        if std::env::var_os("DLPAD_TEST_DOCUMENTS").is_some() {
            return;
        }
        let fixture = Fixture::new();
        let (data, _) = fixture.paths();
        assert_eq!(
            prepare(&data, Action::Default).unwrap_err(),
            c::FIXTURE_DOCUMENTS
        );
    }
    #[test]
    fn scan_rejects_both_data_overlap_directions_before_indexing() {
        let fixture = Fixture::new();
        let (data, _) = fixture.paths();
        assert_eq!(
            scan(&data, &data, None, &AtomicBool::new(false)).unwrap_err(),
            c::SEPARATE_DATA
        );
        assert_eq!(
            scan(&data, &fixture.0, None, &AtomicBool::new(false)).unwrap_err(),
            c::SEPARATE_DATA
        );
    }
    #[test]
    fn worker_panic_acknowledges_stop_and_reports_bounded_failure() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        let mut panel = LibraryPanel::new(data, media);
        stop(&mut panel);
        panel.signals.stopped.store(false, Ordering::Release);
        let signals = panel.signals.clone();
        let handle =
            thread::spawn(move || worker_entry(signals, || panic!("synthetic worker fault")));
        handle.join().unwrap();
        assert!(panel.stopped());
        panel.poll(&egui::Context::default());
        assert_eq!(panel.error.as_deref(), Some(c::LOCATION_FAILED));
        assert!(!panel.busy());
    }
    #[test]
    fn cancelled_scan_keeps_prior_database_observations() {
        let fixture = Fixture::new();
        let (data, media) = fixture.paths();
        std::fs::write(media.join("before.mp4"), b"synthetic").unwrap();
        let previous = scan(&data, &media, None, &AtomicBool::new(false)).unwrap();
        std::fs::write(media.join("after.mp4"), b"synthetic").unwrap();
        assert!(scan(&data, &media, None, &AtomicBool::new(true)).is_err());
        let store = Store::open(&data.join("library.sqlite")).unwrap();
        assert_eq!(
            store.library_page(&media, None, PAGE_SIZE).unwrap(),
            previous
        );
    }
    // No worker or native window: exercise the production panel with a bounded page.
    #[test]
    fn actual_library_search_context_latest_serial_and_failure_preserve_page() {
        use crate::list_search::interaction_tests::{Fixture, enter};
        let root = PathBuf::from("/synthetic/search-owned-media");
        let mut panel = preview_test_panel(root.clone(), root.join("prior.mp4"), 7);
        let prior = panel.rows.clone();
        let (commands, receive) = mpsc::sync_channel(1);
        let (send, results) = mpsc::sync_channel(1);
        panel.commands = commands;
        panel.results = results;
        let mut f = Fixture::new();
        f.frame(vec![], |ui| panel.show(ui, true, false));
        let focus = f.action(
            egui::accesskit::Role::TextInput,
            None,
            egui::accesskit::Action::Focus,
        );
        f.frame(vec![focus], |ui| panel.show(ui, true, false));
        f.frame(vec![egui::Event::Text("  한글%_  ".into())], |ui| {
            panel.show(ui, true, false)
        });
        assert!(
            matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "draft edits do not query or scan"
        );
        f.frame(vec![enter()], |ui| panel.show(ui, true, false));
        let first = receive.try_recv().expect("real library Enter command");
        let Action::Search(first_request, None) = first.action else {
            panic!("readonly first search")
        };
        assert_eq!(first_request.query, "한글%_");
        let clear = f.action(
            egui::accesskit::Role::Button,
            Some(c::LIST_SEARCH_CLEAR),
            egui::accesskit::Action::Click,
        );
        f.frame(vec![clear], |ui| panel.show(ui, true, false));
        assert!(
            matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "bounded one in-flight request"
        );
        let focus = f.action(
            egui::accesskit::Role::TextInput,
            None,
            egui::accesskit::Action::Focus,
        );
        f.frame(vec![focus], |ui| panel.show(ui, true, false));
        f.frame(vec![egui::Event::Text("새 검색".into())], |ui| {
            panel.show(ui, true, false)
        });
        f.frame(vec![enter()], |ui| panel.show(ui, true, false));
        let mut old = delivery_snapshot(Some(first.serial), None, Ok(Arc::default()), None);
        old.search = first_request.clone();
        send.send(old).unwrap();
        panel.poll(&f.ctx);
        assert!(
            Arc::ptr_eq(&panel.rows, &prior),
            "old search cannot publish over the latest draft/request"
        );
        let latest = receive.try_recv().expect("latest-only queued search");
        let Action::Search(request, None) = latest.action else {
            panic!("search first page")
        };
        assert_eq!(request.query, "새 검색");
        let mut old = delivery_snapshot(Some(first.serial), None, Ok(Arc::default()), None);
        old.search = first_request;
        send.send(old).unwrap();
        panel.poll(&f.ctx);
        assert_eq!(
            panel.pending,
            Some(latest.serial),
            "late serial cannot clear matching new request"
        );
        let mut failed = delivery_snapshot(
            Some(latest.serial),
            None,
            Err(c::LOCATION_FAILED.into()),
            None,
        );
        failed.search = request;
        send.send(failed).unwrap();
        panel.poll(&f.ctx);
        assert!(!panel.busy() && panel.error.is_some() && panel.search.stale());
        assert!(
            Arc::ptr_eq(&panel.rows, &prior),
            "database error retains the successful page"
        );
        panel.request_stop();
        assert!(matches!(
            receive.try_recv().expect("owned stop wake").action,
            Action::StopWake
        ));
        let clear = f.action(
            egui::accesskit::Role::Button,
            Some(c::LIST_SEARCH_CLEAR),
            egui::accesskit::Action::Click,
        );
        f.frame(vec![clear], |ui| panel.show(ui, false, false));
        assert!(
            matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "Stop refuses all queued search dispatch"
        );
    }
    fn compact_search_frame(
        f: &mut crate::list_search::interaction_tests::Fixture,
        panel: &mut LibraryPanel,
        events: Vec<egui::Event>,
    ) {
        f.frame(events, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 125.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_max_height(125.0);
                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                    assert!(
                        (ui.clip_rect().height() - 125.0).abs() < 0.01,
                        "actual fresh component body clip before panel"
                    );
                    panel.show(ui, true, false);
                },
            );
        });
        let (inner, content, offset) = f
            .ctx
            .data(|data| {
                data.get_temp::<(egui::Rect, egui::Vec2, egui::Vec2)>(egui::Id::new(
                    "search-library-body-observation",
                ))
            })
            .expect("actual library ScrollArea output");
        assert!(
            (inner.height() - 125.0).abs() < 0.01,
            "actual library viewport must be125logical, inner={inner:?} content={content:?} offset={offset:?}"
        );
    }
    #[test]
    fn actual_library_empty_search_reveals_once_without_focus_or_wheel_snap() {
        use crate::list_search::interaction_tests::{Fixture, enter};
        for zoom in [1.0, 2.0] {
            let root = PathBuf::from("/synthetic/search-owned-media");
            let mut panel = preview_test_panel(root.clone(), root.join("prior.mp4"), 7);
            let (commands, receive) = mpsc::sync_channel(1);
            let (send, results) = mpsc::sync_channel(1);
            panel.commands = commands;
            panel.results = results;
            let mut f = Fixture::new();
            f.ctx.options_mut(|options| options.zoom_factor = zoom);
            compact_search_frame(&mut f, &mut panel, vec![]);
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            compact_search_frame(&mut f, &mut panel, vec![focus]);
            compact_search_frame(
                &mut f,
                &mut panel,
                vec![egui::Event::Text("없는 검색".into())],
            );
            compact_search_frame(&mut f, &mut panel, vec![enter()]);
            let command = receive.try_recv().expect("actual Enter search dispatch");
            let Action::Search(request, None) = command.action else {
                panic!("first-page readonly search");
            };
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            compact_search_frame(&mut f, &mut panel, vec![focus]);
            let original_focus = f.ctx.memory(|memory| memory.focused());
            assert!(original_focus.is_some());
            let mut delivered =
                delivery_snapshot(Some(command.serial), None, Ok(Arc::default()), None);
            delivered.search = request;
            send.send(delivered).unwrap();
            panel.poll(&f.ctx);
            for _ in 0..32 {
                compact_search_frame(&mut f, &mut panel, vec![]);
            }
            assert!(
                f.fully_painted(c::LIST_SEARCH_EMPTY),
                "actual no-result text must be completely painted within125logical body at both scales"
            );
            assert!(f.is_nonfocusable_label(c::LIST_SEARCH_EMPTY));
            assert_eq!(f.ctx.memory(|memory| memory.focused()), original_focus);
            let revealed = panel.library_offset;
            assert!(
                revealed > 1.0,
                "real ScrollArea offset changed to reveal the formerly clipped result: zoom={zoom} offset={revealed}"
            );
            compact_search_frame(
                &mut f,
                &mut panel,
                vec![
                    egui::Event::PointerMoved(egui::pos2(80.0, 80.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, 6.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            let moved = panel.library_offset;
            assert!(
                moved < revealed - 1.0,
                "actual user wheel can move after one-time reveal"
            );
            for _ in 0..4 {
                compact_search_frame(&mut f, &mut panel, vec![]);
            }
            assert!(
                (panel.library_offset - moved).abs() < 0.01,
                "no later-frame reveal snap while focus is held"
            );
            assert_eq!(f.ctx.memory(|memory| memory.focused()), original_focus);
        }
    }
    #[test]
    fn actual_library_empty_search_stale_draft_old_reply_failure_and_stop_never_reveal() {
        use crate::list_search::interaction_tests::{Fixture, enter};
        for case in 0..4 {
            let root = PathBuf::from("/synthetic/search-owned-media");
            let mut panel = preview_test_panel(root.clone(), root.join("prior.mp4"), 7);
            let prior = panel.rows.clone();
            let (commands, receive) = mpsc::sync_channel(1);
            let (send, results) = mpsc::sync_channel(1);
            panel.commands = commands;
            panel.results = results;
            let mut f = Fixture::new();
            compact_search_frame(&mut f, &mut panel, vec![]);
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            compact_search_frame(&mut f, &mut panel, vec![focus]);
            compact_search_frame(
                &mut f,
                &mut panel,
                vec![egui::Event::Text("첫 검색".into())],
            );
            compact_search_frame(&mut f, &mut panel, vec![enter()]);
            let command = receive.try_recv().unwrap();
            let Action::Search(request, None) = command.action else {
                panic!("actual search");
            };
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            compact_search_frame(&mut f, &mut panel, vec![focus]);
            if case == 0 {
                compact_search_frame(&mut f, &mut panel, vec![egui::Event::Text(" 수정".into())]);
            }
            if case == 1 {
                let clear = f.action(
                    egui::accesskit::Role::Button,
                    Some(c::LIST_SEARCH_CLEAR),
                    egui::accesskit::Action::Click,
                );
                compact_search_frame(&mut f, &mut panel, vec![clear]);
            }
            if case == 3 {
                panel.request_stop();
            }
            let offset = panel.library_offset;
            let outcome = if case == 2 {
                Err(c::LOCATION_FAILED.into())
            } else {
                Ok(Arc::default())
            };
            let mut delivered = delivery_snapshot(Some(command.serial), None, outcome, None);
            delivered.search = request;
            send.send(delivered).unwrap();
            panel.poll(&f.ctx);
            for _ in 0..32 {
                compact_search_frame(&mut f, &mut panel, vec![]);
            }
            assert!(
                (panel.library_offset - offset).abs() < 0.01,
                "stale/old/failure/Stop cannot scroll to old no-result feedback; case={case}"
            );
            if case == 1 || case == 2 {
                assert!(
                    Arc::ptr_eq(&prior, &panel.rows),
                    "old generation and failure retain previous rows"
                );
            }
        }
    }
    fn preview_test_panel(root: PathBuf, source: PathBuf, size: u64) -> LibraryPanel {
        let (commands, _) = mpsc::sync_channel(1);
        let (_, results) = mpsc::sync_channel(1);
        LibraryPanel {
            edit: crate::path_picker::display_path(&root),
            root,
            rows: Arc::new(vec![LibraryEntry {
                path: source,
                size,
                modified_ms: Some(7),
                file_identity: Some("fixture-source".into()),
            }]),
            error: None,
            watch_failed: false,
            commands,
            results,
            signals: Arc::new(delivery_signals()),
            pending: None,
            serial: 0,
            pending_refresh: false,
            search: Default::default(),
            applied: None,
            location_request_started: None,
            browse_requested: false,
            selected_media: None,
            preview_requested: None,
            system_open_status: None,
            last_focused_row: None,
            selected_row_id: None,
            path_detail: None,
            library_offset: 0.0,
            selected_row_offset: 0.0,
            detail_restore: None,
            export_draft: None,
            export_generation: 0,
            export_browse: None,
            export_progress: None,
            export_cancel: None,
            export_result: None,
            export_result_notified: false,
        }
    }
    fn preview_panel_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        events: Vec<egui::Event>,
        can_change: bool,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(700.0, 500.0),
                )),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    panel.show(ui, can_change, false);
                });
            },
        )
    }
    fn preview_node(
        output: egui::FullOutput,
        label: &str,
    ) -> (egui::accesskit::NodeId, egui::accesskit::Rect) {
        let tree = output.platform_output.accesskit_update.unwrap();
        let (id, node) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap();
        (*id, node.bounds().unwrap())
    }
    fn preview_accesskit_click(id: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target: id,
            data: None,
        })
    }
    fn preview_enter() -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: Some(egui::Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn preview_focus_row(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        bounds: egui::accesskit::Rect,
    ) {
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        let event = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        preview_panel_frame(
            ctx,
            panel,
            vec![egui::Event::PointerMoved(pos), event(true)],
            true,
        );
        preview_panel_frame(ctx, panel, vec![event(false)], true);
    }
    #[test]
    fn real_library_row_accesskit_button_and_focused_enter_request_preview_once_without_writing_source()
     {
        let fixture = Fixture::new();
        let (_, media) = fixture.paths();
        let source = media.join("preview-original.mp4");
        let bytes = b"synthetic source must remain byte identical";
        std::fs::write(&source, bytes).unwrap();
        let mut panel = preview_test_panel(media, source.clone(), bytes.len() as u64);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let row_label = format!(
            "preview-original.mp4 · {} bytes · {}",
            bytes.len(),
            c::LOCAL
        );
        preview_panel_frame(&ctx, &mut panel, vec![], true);
        let (row, bounds) = preview_node(
            preview_panel_frame(&ctx, &mut panel, vec![], true),
            &row_label,
        );
        preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(row)], true);
        assert_eq!(panel.selected_media.as_ref(), Some(&source));
        assert_eq!(
            panel.take_preview_request(),
            None,
            "selection alone does not prepare media"
        );
        let (button, _) = preview_node(
            preview_panel_frame(&ctx, &mut panel, vec![], true),
            c::LIBRARY_PREVIEW,
        );
        preview_panel_frame(
            &ctx,
            &mut panel,
            vec![preview_accesskit_click(button)],
            true,
        );
        assert_eq!(panel.take_preview_request(), Some(source.clone()));
        assert_eq!(panel.take_preview_request(), None);
        preview_focus_row(&ctx, &mut panel, bounds);
        preview_panel_frame(&ctx, &mut panel, vec![preview_enter()], true);
        assert_eq!(panel.take_preview_request(), Some(source.clone()));
        assert_eq!(panel.take_preview_request(), None);
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        assert_eq!(panel.rows.len(), 1);
        assert!(!fixture.0.join("data/library.sqlite").exists());
    }
    #[test]
    fn real_library_preview_disabled_busy_and_closewait_reject_pointer_keyboard_and_accesskit_requests()
     {
        for mode in 0..3 {
            let fixture = Fixture::new();
            let (_, media) = fixture.paths();
            let source = media.join("blocked-original.mp4");
            let bytes = b"preserve blocked source";
            std::fs::write(&source, bytes).unwrap();
            let mut panel = preview_test_panel(media, source.clone(), bytes.len() as u64);
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let label = format!(
                "blocked-original.mp4 · {} bytes · {}",
                bytes.len(),
                c::LOCAL
            );
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let (row, bounds) =
                preview_node(preview_panel_frame(&ctx, &mut panel, vec![], true), &label);
            preview_focus_row(&ctx, &mut panel, bounds);
            assert_eq!(panel.selected_media.as_ref(), Some(&source));
            let (button, button_bounds) = preview_node(
                preview_panel_frame(&ctx, &mut panel, vec![], true),
                c::LIBRARY_PREVIEW,
            );
            if mode == 1 {
                panel.pending = Some(1);
            }
            if mode == 2 {
                panel.signals.stop.store(true, Ordering::Release);
                assert!(!panel.stopped());
            }
            let can_change = mode != 0;
            preview_panel_frame(&ctx, &mut panel, vec![], can_change);
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![
                    preview_enter(),
                    preview_accesskit_click(row),
                    preview_accesskit_click(button),
                ],
                can_change,
            );
            let pos = egui::pos2(
                ((button_bounds.x0 + button_bounds.x1) / 2.0) as f32,
                ((button_bounds.y0 + button_bounds.y1) / 2.0) as f32,
            );
            let click = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![egui::Event::PointerMoved(pos), click(true)],
                can_change,
            );
            preview_panel_frame(&ctx, &mut panel, vec![click(false)], can_change);
            assert_eq!(panel.take_preview_request(), None);
            assert_eq!(panel.selected_media.as_ref(), Some(&source));
            assert_eq!(std::fs::read(&source).unwrap(), bytes);
        }
    }
    fn narrow_library_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let time = ctx.input(|input| input.time) + 1.0;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(360.0, 1100.0),
                )),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    panel.show(ui, true, false);
                });
            },
        )
    }
    fn painted_text(shapes: &[egui::epaint::ClippedShape]) -> String {
        fn append(shape: &egui::epaint::Shape, text: &mut String) {
            match shape {
                egui::epaint::Shape::Text(shape) => {
                    text.push_str(shape.galley.text());
                    text.push('\n');
                }
                egui::epaint::Shape::Vec(shapes) => {
                    for shape in shapes {
                        append(shape, text);
                    }
                }
                _ => {}
            }
        }
        let mut text = String::new();
        for shape in shapes {
            append(&shape.shape, &mut text);
        }
        text
    }
    #[test]
    fn real_long_unicode_library_rows_fit_virtual_height_at_narrow_width_and_all_scales() {
        for scale in [1.0, 1.5, 2.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.set_zoom_factor(scale);
            let root = PathBuf::from("synthetic-library-root");
            let long_name = format!("{}-original.mp4", "긴영상이름".repeat(35));
            let original = root.join(&long_name);
            let mut panel = preview_test_panel(root.clone(), original.clone(), 17);
            let mut rows = (*panel.rows).clone();
            for i in 1..64 {
                rows.push(LibraryEntry {
                    path: root.join(format!("{i:02}-{}", long_name)),
                    size: 17,
                    modified_ms: Some(7),
                    file_identity: Some(format!("original-{i}")),
                });
            }
            panel.rows = Arc::new(rows);
            let originals = panel.rows.clone();
            narrow_library_frame(&ctx, &mut panel, vec![]);
            let output = narrow_library_frame(&ctx, &mut panel, vec![]);
            assert!((output.pixels_per_point - scale).abs() < 0.01);
            let tree = output.platform_output.accesskit_update.unwrap();
            let mut actual = Vec::new();
            for entry in panel.rows.iter() {
                let label = format!(
                    "{} · 17 bytes · {}",
                    entry.path.file_name().unwrap().to_string_lossy(),
                    c::LOCAL
                );
                if let Some((id, node)) = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label.as_str()))
                {
                    actual.push((*id, node.bounds().unwrap()));
                }
            }
            assert!(
                actual.len() >= 3,
                "narrow virtual list should expose adjacent visible rows"
            );
            for (_, bounds) in &actual {
                assert!(
                    (bounds.y1 - bounds.y0 - 32.0).abs() < 0.1,
                    "actual row height must fit the virtual list stride"
                );
                assert!(
                    bounds.x1 - bounds.x0 <= 360.0,
                    "long name must not widen the row beyond the viewport"
                );
            }
            for pair in actual.windows(2) {
                assert!(
                    pair[0].1.y1 <= pair[1].1.y0,
                    "adjacent painted/hit rows must never overlap"
                );
            }
            let first_id = actual[0].0;
            narrow_library_frame(&ctx, &mut panel, vec![preview_accesskit_click(first_id)]);
            assert_eq!(panel.selected_media.as_ref(), Some(&original));
            assert_eq!(panel.take_preview_request(), None);
            let full_label = format!("{} · 17 bytes · {}", long_name, c::LOCAL);
            let (same_id, bounds) =
                preview_node(narrow_library_frame(&ctx, &mut panel, vec![]), &full_label);
            assert_eq!(
                same_id, first_id,
                "truncation and selection keep the full accessible label and logical row identity"
            );
            let pos = egui::pos2(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            );
            narrow_library_frame(&ctx, &mut panel, vec![egui::Event::PointerMoved(pos)]);
            narrow_library_frame(&ctx, &mut panel, vec![]);
            let hover = narrow_library_frame(&ctx, &mut panel, vec![]);
            assert!(
                painted_text(&hover.shapes).contains(original.to_string_lossy().as_ref()),
                "hover must retain the full original path"
            );
            narrow_library_frame(
                &ctx,
                &mut panel,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -220.0),
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            let scroll = narrow_library_frame(&ctx, &mut panel, vec![]);
            let scroll_tree = scroll.platform_output.accesskit_update.unwrap();
            let mut scrolled = Vec::new();
            for entry in panel.rows.iter() {
                let label = format!(
                    "{} · 17 bytes · {}",
                    entry.path.file_name().unwrap().to_string_lossy(),
                    c::LOCAL
                );
                if let Some((_, node)) = scroll_tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label.as_str()))
                {
                    let bounds = node.bounds().unwrap();
                    assert!((bounds.y1 - bounds.y0 - 32.0).abs() < 0.1);
                    scrolled.push(bounds);
                }
            }
            assert!(scrolled.len() >= 3);
            assert!(
                scrolled[0].y0 != actual[0].1.y0
                    || !scroll_tree.nodes.iter().any(|(id, _)| *id == first_id),
                "wheel must actually move the virtual viewport"
            );
            for pair in scrolled.windows(2) {
                assert!(
                    pair[0].y1 <= pair[1].y0,
                    "scroll must preserve adjacent row separation"
                );
            }
            assert_eq!(panel.rows, originals);
            assert_eq!(panel.selected_media.as_ref(), Some(&original));
        }
    }
    fn zoom_library_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let time = ctx.input(|input| input.time) + 1.0;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::TopBottomPanel::top("fixture-navigation")
                    .exact_height(48.0)
                    .show(ctx, |ui| {
                        ui.label(c::LIBRARY);
                    });
                egui::TopBottomPanel::bottom("fixture-footer")
                    .exact_height(54.0)
                    .show(ctx, |ui| {
                        ui.label("fixture footer");
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(c::LIBRARY);
                    ui.add_space(12.0);
                    panel.show(ui, true, false);
                });
            },
        )
    }
    fn fully_visible_text(output: &egui::FullOutput, needle: &str) -> bool {
        output.shapes.iter().any(|shape| {
            if let egui::epaint::Shape::Text(text) = &shape.shape {
                let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                text.galley.text().contains(needle)
                    && shape.clip_rect.contains_rect(rect)
                    && rect.bottom() <= 246.0
            } else {
                false
            }
        })
    }
    #[test]
    fn empty_library_instruction_is_reachable_by_actual_wheel_with_fixed_footer() {
        let mut panel = preview_test_panel(
            PathBuf::from("fixture-owned-root"),
            PathBuf::from("fixture-owned-root/source.mp4"),
            3,
        );
        panel.rows = Arc::new(Vec::new());
        let ctx = egui::Context::default();
        // Let egui converge its viewport/bar geometry before simulating scroll.
        for _ in 0..3 {
            zoom_library_frame(&ctx, &mut panel, vec![]);
        }
        let mut current = zoom_library_frame(&ctx, &mut panel, vec![]);
        assert!(
            fully_visible_text(&current, c::LIBRARY_EMPTY_HELP),
            "compact read-only location leaves the empty instruction immediately visible"
        );
        for _ in 0..32 {
            current = zoom_library_frame(
                &ctx,
                &mut panel,
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 180.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -24.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            if fully_visible_text(&current, c::LIBRARY_EMPTY_HELP) {
                break;
            }
        }
        assert!(
            fully_visible_text(&current, c::LIBRARY_EMPTY_HELP),
            "actual incremental wheel must leave the complete empty instruction visible above fixed footer"
        );
        assert!(panel.rows.is_empty());
    }
    #[test]
    fn actual_library_page_render_is_bounded_and_failed_refresh_retains_rows() {
        let mut panel = preview_test_panel(
            PathBuf::from("fixture-owned-root"),
            PathBuf::from("fixture-owned-root/source.mp4"),
            3,
        );
        panel.rows = Arc::new(
            (0..10_000)
                .map(|i| LibraryEntry {
                    path: PathBuf::from(format!("fixture-owned-root/unique-visible-row-{i}.mp4")),
                    size: 3,
                    modified_ms: Some(7),
                    file_identity: None,
                })
                .collect(),
        );
        panel.error = Some("fixture failed refresh".into());
        let originals = panel.rows.clone();
        let ctx = egui::Context::default();
        for _ in 0..4 {
            let output = zoom_library_frame(
                &ctx,
                &mut panel,
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 180.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -180.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            let text = painted_text(&output.shapes);
            let rendered = text.matches("unique-visible-row-").count();
            assert!(
                rendered < 16,
                "viewport row materialization must stay bounded, got {rendered}"
            );
            assert!(Arc::ptr_eq(&panel.rows, &originals));
            assert!(panel.error.is_some());
        }
    }
    #[derive(Default)]
    struct TraversalTree {
        root: Option<egui::accesskit::NodeId>,
        focus: Option<egui::accesskit::NodeId>,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    }
    impl TraversalTree {
        fn merge(&mut self, output: &egui::FullOutput) {
            if let Some(update) = output.platform_output.accesskit_update.as_ref() {
                if let Some(tree) = update.tree.as_ref() {
                    self.root = Some(tree.root);
                }
                self.focus = Some(update.focus);
                for (id, node) in &update.nodes {
                    self.nodes.insert(*id, node.clone());
                }
                // Removed nodes are not explicitly listed in TreeUpdate: retain only
                // those reachable through the current original root child graph.
                let mut live = std::collections::HashSet::new();
                let mut pending = vec![self.root.expect("original AccessKit root")];
                while let Some(id) = pending.pop() {
                    if !live.insert(id) {
                        continue;
                    }
                    let node = self
                        .nodes
                        .get(&id)
                        .expect("reachable actual child must exist");
                    pending.extend(node.children().iter().copied());
                }
                self.nodes.retain(|id, _| live.contains(id));
            }
        }
        fn row(&self, label: &str) -> Option<(egui::accesskit::NodeId, &egui::accesskit::Node)> {
            self.nodes
                .iter()
                .find(|(_, n)| n.label() == Some(label))
                .map(|(id, n)| (*id, n))
        }
    }
    fn traversal_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        tree: &mut TraversalTree,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let output = zoom_library_frame(ctx, panel, events);
        tree.merge(&output);
        output
    }
    // Append inside accessible_library.rs's existing tests module.
    fn traversal_key(key: egui::Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn traversal_label(index: usize) -> String {
        format!("keyboard-row-{index:03}.mp4 · 3 bytes · {}", c::LOCAL)
    }
    #[test]
    fn actual_virtual_library_tab_reaches_all_128_paths_then_enter_preview() {
        virtual_library_keyboard_traversal(egui::Key::Tab);
    }
    #[test]
    fn actual_virtual_library_arrow_reaches_all_128_paths_then_enter_preview() {
        virtual_library_keyboard_traversal(egui::Key::ArrowDown);
    }
    fn virtual_library_keyboard_traversal(key: egui::Key) {
        {
            let root = PathBuf::from("keyboard-fixture-owned-root");
            let mut panel = preview_test_panel(root.clone(), root.join("keyboard-row-000.mp4"), 3);
            panel.rows = Arc::new(
                (0..128)
                    .map(|i| LibraryEntry {
                        path: root.join(format!("keyboard-row-{i:03}.mp4")),
                        size: 3,
                        modified_ms: Some(7),
                        file_identity: Some(format!("keyboard-native-fixture-{i}")),
                    })
                    .collect(),
            );
            let originals = panel.rows.clone();
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_pixels_per_point(2.0);
            ctx.enable_accesskit();
            let mut tree = TraversalTree::default();
            for _ in 0..3 {
                traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
            }
            assert!((ctx.input(|i| i.content_rect().width()) - 400.0).abs() < 0.1);
            assert!((ctx.pixels_per_point() - 2.0).abs() < 0.01);
            let mut output = traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
            // Bootstrap only: genuine wheel reveals first row, without touching
            // scroll state. This isolates virtual-boundary keyboard traversal;
            // it does not certify keyboard-only entry from the page toolbar.
            for _ in 0..32 {
                if fully_visible_text(&output, &traversal_label(0)) {
                    break;
                }
                output = traversal_frame(
                    &ctx,
                    &mut panel,
                    &mut tree,
                    vec![
                        egui::Event::PointerMoved(egui::pos2(200.0, 180.0)),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, -24.0),
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
            }
            assert!(
                fully_visible_text(&output, &traversal_label(0)),
                "bootstrap first row"
            );
            let first = tree
                .row(&traversal_label(0))
                .map(|(id, _)| id)
                .expect("actual reachable first path Button");
            traversal_frame(
                &ctx,
                &mut panel,
                &mut tree,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: first,
                        data: None,
                    },
                )],
            );
            for _ in 0..3 {
                output = traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
            }
            let mut seen = std::collections::HashSet::new();
            for index in 0..128 {
                if index > 0 {
                    traversal_frame(&ctx, &mut panel, &mut tree, vec![traversal_key(key, true)]);
                    traversal_frame(&ctx, &mut panel, &mut tree, vec![traversal_key(key, false)]);
                    for _ in 0..3 {
                        output = traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
                    }
                }
                let label = traversal_label(index);
                let (id, node) = tree.row(&label).unwrap_or_else(|| {
                    panic!(
                        "{key:?} next path {index} is not reachable/materialized; focus={:?}",
                        tree.focus
                    )
                });
                assert_eq!(node.role(), egui::accesskit::Role::Button);
                let actual_focus = tree.focus.and_then(|focused| {
                    tree.nodes
                        .get(&focused)
                        .map(|n| (focused, n.role(), n.label().map(str::to_owned), n.bounds()))
                });
                let live_rows: Vec<_> = tree
                    .nodes
                    .iter()
                    .filter(|(_, n)| {
                        n.label()
                            .is_some_and(|label| label.starts_with("keyboard-row-"))
                    })
                    .map(|(id, n)| (*id, n.label().map(str::to_owned), n.bounds()))
                    .collect();
                assert_eq!(
                    tree.focus,
                    Some(id),
                    "{key:?} must focus immediate next path {index}; actual_focus={actual_focus:?}; target_bounds={:?}; reachable_rows={live_rows:?}",
                    node.bounds()
                );
                assert!(
                    seen.insert(id),
                    "different original paths need different stable IDs"
                );
                assert!(
                    fully_visible_text(&output, &label),
                    "{key:?} row {index} full glyph clip"
                );
                let bounds = node.bounds().expect("original row geometry");
                assert!((bounds.y1 - bounds.y0 - 32.0).abs() < 0.5);
                assert!(
                    bounds.y0 >= 48.0 && bounds.y1 <= 246.0,
                    "fixed navigation/footer clip"
                );
                assert!(Arc::ptr_eq(&panel.rows, &originals));
                assert!(
                    panel.take_preview_request().is_none(),
                    "navigation must not launch preview"
                );
            }
            traversal_frame(&ctx, &mut panel, &mut tree, vec![preview_enter()]);
            assert_eq!(
                panel.take_preview_request(),
                Some(root.join("keyboard-row-127.mp4"))
            );
            assert_eq!(
                panel.selected_media,
                Some(root.join("keyboard-row-127.mp4"))
            );
            traversal_frame(
                &ctx,
                &mut panel,
                &mut tree,
                vec![traversal_key(egui::Key::Enter, false)],
            );
            assert!(
                panel.take_preview_request().is_none(),
                "release never duplicates preview"
            );
            assert!(Arc::ptr_eq(&panel.rows, &originals));
        }
    }
    #[test]
    fn actual_library_empty_action_context_and_preview_states() {
        for state in ["empty", "unselected", "loading", "error", "selected"] {
            let root = PathBuf::from("fixture-owned-root");
            let source = root.join("source.mp4");
            let mut panel = preview_test_panel(root, source.clone(), 17);
            match state {
                "empty" => panel.rows = Arc::new(vec![]),
                "loading" => {
                    panel.rows = Arc::new(vec![]);
                    panel.pending = Some(1);
                }
                "error" => {
                    panel.rows = Arc::new(vec![]);
                    panel.error = Some("fixture failed refresh".into());
                }
                "selected" => panel.selected_media = Some(source.clone()),
                "unselected" => {}
                _ => unreachable!(),
            }
            let original_rows = panel.rows.clone();
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let output = preview_panel_frame(&ctx, &mut panel, vec![], true);
            let text = painted_text(&output.shapes);
            let tree = output.platform_output.accesskit_update.as_ref().unwrap();
            let preview = tree.nodes.iter().find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::LIBRARY_PREVIEW)
            });
            if state == "empty" {
                assert!(text.contains(c::LIBRARY_EMPTY), "{state}");
                assert!(text.contains(c::LIBRARY_EMPTY_HELP), "{state}");
                assert!(!text.contains(c::LIBRARY_SELECT_HELP), "{state}");
                assert!(preview.is_none(), "empty has no preview target");
            } else {
                assert!(!text.contains(c::LIBRARY_EMPTY_HELP), "{state}");
                assert!(text.contains(c::LIBRARY_SELECT_HELP), "{state}");
                let (_, node) = preview.expect("actual preview action retained");
                assert_eq!(node.is_disabled(), state != "selected", "{state}");
            }
            for label in [c::REFRESH, c::NEXT] {
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(label)),
                    "{state}: {label}"
                );
            }
            if state == "loading" {
                assert!(text.contains(c::LIBRARY_LOADING));
            }
            if state == "error" {
                assert!(text.contains("fixture failed refresh"));
                assert!(text.contains(c::LIBRARY_RETRY));
            }
            if let Some((id, _)) = preview {
                preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(*id)], true);
                assert_eq!(
                    panel.take_preview_request(),
                    (state == "selected").then_some(source)
                );
            }
            assert!(Arc::ptr_eq(&panel.rows, &original_rows));
        }
    }
    // Append to existing library tests. Reuses current root-pruned TraversalTree.
    fn detail_focus(id: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        })
    }
    fn detail_press(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        tree: &mut TraversalTree,
        key: egui::Key,
    ) -> egui::FullOutput {
        traversal_frame(ctx, panel, tree, vec![traversal_key(key, true)]);
        traversal_frame(ctx, panel, tree, vec![traversal_key(key, false)]);
        let mut out = traversal_frame(ctx, panel, tree, vec![]);
        for _ in 0..3 {
            out = traversal_frame(ctx, panel, tree, vec![]);
        }
        out
    }
    fn detail_endpoint_painted(output: &egui::FullOutput, complete: &str, fragment: &str) -> bool {
        fn visit(
            shape: &egui::epaint::Shape,
            clip: egui::Rect,
            complete: &str,
            fragment: &str,
        ) -> bool {
            match shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == complete => {
                    !text.galley.elided
                        && text.galley.rows.iter().any(|row| {
                            let value: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                            let rect = row.rect().translate(text.pos.to_vec2());
                            value.contains(fragment)
                                && clip.contains_rect(rect)
                                && egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(400.0, 300.0),
                                )
                                .contains_rect(rect)
                        })
                }
                egui::epaint::Shape::Vec(parts) => {
                    parts.iter().any(|p| visit(p, clip, complete, fragment))
                }
                _ => false,
            }
        }
        output
            .shapes
            .iter()
            .any(|s| visit(&s.shape, s.clip_rect, complete, fragment))
    }
    fn detail_panel_setup() -> (
        egui::Context,
        LibraryPanel,
        TraversalTree,
        PathBuf,
        String,
        egui::accesskit::NodeId,
    ) {
        let root = PathBuf::from(format!(
            "DETAIL_START/{}/資料",
            "長いUnicode🙂/".repeat(100)
        ));
        let source = root.join("row000-終.mp4");
        let mut panel = preview_test_panel(root.clone(), source.clone(), 3);
        panel.rows = Arc::new(
            (0..128)
                .map(|i| LibraryEntry {
                    path: if i == 0 {
                        source.clone()
                    } else {
                        root.join(format!("row{i:03}.mp4"))
                    },
                    size: 3,
                    modified_ms: Some(7),
                    file_identity: Some(format!("original-{i}")),
                })
                .collect(),
        );
        let ctx = egui::Context::default();
        crate::ui_theme::configure_dlpad_theme(&ctx);
        ctx.set_pixels_per_point(2.0);
        ctx.enable_accesskit();
        let mut tree = TraversalTree::default();
        for _ in 0..4 {
            traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
        }
        let label = format!("row000-終.mp4 · 3 bytes · {}", c::LOCAL);
        let mut output = traversal_frame(&ctx, &mut panel, &mut tree, vec![]);
        // Same existing bootstrap as 128-row tests; detail interaction is pointer-free.
        for _ in 0..32 {
            if fully_visible_text(&output, &label) {
                break;
            }
            output = traversal_frame(
                &ctx,
                &mut panel,
                &mut tree,
                vec![
                    egui::Event::PointerMoved(egui::pos2(200.0, 180.0)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -24.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(fully_visible_text(&output, &label));
        let row = tree.row(&label).unwrap().0;
        assert_eq!(
            tree.nodes[&row].toggled(),
            Some(egui::accesskit::Toggled::False),
            "actual unselected row state"
        );
        traversal_frame(
            &ctx,
            &mut panel,
            &mut tree,
            vec![egui::Event::PointerGone, detail_focus(row)],
        );
        detail_press(&ctx, &mut panel, &mut tree, egui::Key::Space);
        assert_eq!(panel.selected_media.as_ref(), Some(&source));
        assert_eq!(
            tree.nodes[&row].toggled(),
            Some(egui::accesskit::Toggled::True),
            "actual Space-selected row state"
        );
        assert!(
            panel.take_preview_request().is_none(),
            "Space selects; never previews"
        );
        (ctx, panel, tree, source, label, row)
    }
    fn detail_open_from_actual_action(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        tree: &mut TraversalTree,
    ) {
        let action = tree
            .row(c::LIBRARY_PATH_DETAIL_ACTION)
            .expect("single real external detail action")
            .0;
        assert_eq!(
            tree.nodes
                .values()
                .filter(|n| n.label() == Some(c::LIBRARY_PATH_DETAIL_ACTION))
                .count(),
            1
        );
        traversal_frame(ctx, panel, tree, vec![detail_focus(action)]);
        let mut output = traversal_frame(ctx, panel, tree, vec![]);
        for _ in 0..4 {
            output = traversal_frame(ctx, panel, tree, vec![]);
        }
        assert!(
            fully_visible_text(&output, c::LIBRARY_PATH_DETAIL_ACTION),
            "focused action must be genuinely reachable, not just in provider tree"
        );
        detail_press(ctx, panel, tree, egui::Key::Space);
        assert!(
            panel.path_detail.is_some(),
            "keyboard action opens actual Panel modal"
        );
    }
    #[test]
    fn actual_panel_detail_keyboard_endpoints_close_restore_original_row_and_routes() {
        for close_button in [false, true] {
            let (ctx, mut panel, mut tree, source, row_label, row_id) = detail_panel_setup();
            let originals = panel.rows.clone();
            detail_open_from_actual_action(&ctx, &mut panel, &mut tree);
            let full = source.to_string_lossy().into_owned();
            detail_press(&ctx, &mut panel, &mut tree, egui::Key::PageDown);
            assert!(panel.path_detail.as_ref().unwrap().offset > 0.0);
            assert!(panel.take_preview_request().is_none());
            let mut out = detail_press(&ctx, &mut panel, &mut tree, egui::Key::End);
            assert!(
                detail_endpoint_painted(&out, &full, "終.mp4"),
                "last Unicode path glyph row is actually unclipped"
            );
            out = detail_press(&ctx, &mut panel, &mut tree, egui::Key::Home);
            assert!(detail_endpoint_painted(&out, &full, "DETAIL_START"));
            assert!(
                out.textures_delta
                    .set
                    .iter()
                    .all(|(_, d)| d.image.size().iter().all(|n| *n > 0)),
                "real texture uploads are valid; no fake painter output"
            );
            if close_button {
                let close = tree.row(c::LIBRARY_PATH_DETAIL_CLOSE).unwrap().0;
                traversal_frame(&ctx, &mut panel, &mut tree, vec![detail_focus(close)]);
                out = detail_press(&ctx, &mut panel, &mut tree, egui::Key::Space);
            } else {
                out = detail_press(&ctx, &mut panel, &mut tree, egui::Key::Escape);
            }
            assert!(panel.path_detail.is_none());
            assert_eq!(tree.focus, Some(row_id));
            assert_eq!(tree.row(&row_label).unwrap().0, row_id);
            assert!(fully_visible_text(&out, &row_label));
            assert_eq!(panel.selected_media.as_ref(), Some(&source));
            assert!(panel.take_preview_request().is_none());
            assert!(Arc::ptr_eq(&panel.rows, &originals));
            detail_press(&ctx, &mut panel, &mut tree, egui::Key::Home);
            detail_press(&ctx, &mut panel, &mut tree, egui::Key::End);
            assert!(
                panel.path_detail.is_none(),
                "closed modal cannot consume row keys or reopen"
            );
            assert_eq!(
                tree.focus,
                Some(row_id),
                "closed modal leaves original row Home/End route intact"
            );
            assert!(panel.take_preview_request().is_none());
            // Existing stable Arrow/Tab routes immediately after restoration.
            detail_press(&ctx, &mut panel, &mut tree, egui::Key::ArrowDown);
            let second = tree
                .row(&format!("row001.mp4 · 3 bytes · {}", c::LOCAL))
                .unwrap()
                .0;
            assert_eq!(tree.focus, Some(second));
            detail_press(&ctx, &mut panel, &mut tree, egui::Key::Tab);
            let third = tree
                .row(&format!("row002.mp4 · 3 bytes · {}", c::LOCAL))
                .unwrap()
                .0;
            assert_eq!(tree.focus, Some(third));
        }
    }

    fn detail_resize_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        events: Vec<egui::Event>,
        size: egui::Vec2,
    ) -> egui::FullOutput {
        let time = ctx.input(|input| input.time) + 1.0;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::TopBottomPanel::top("fixture-navigation")
                    .exact_height(48.0)
                    .show(ctx, |ui| {
                        ui.label(c::LIBRARY);
                    });
                egui::TopBottomPanel::bottom("fixture-footer")
                    .exact_height(54.0)
                    .show(ctx, |ui| {
                        ui.label("fixture footer");
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(c::LIBRARY);
                    ui.add_space(12.0);
                    panel.show(ui, true, false);
                });
            },
        )
    }

    #[test]
    fn actual_panel_detail_rescan_page_replacement_and_resize_never_restore_stale_row() {
        for replacement in ["removed", "new-page", "resize"] {
            let (ctx, mut panel, mut tree, source, label, row) = detail_panel_setup();
            detail_open_from_actual_action(&ctx, &mut panel, &mut tree);
            // Inject the already-delivered snapshot seam, not a fake worker/DB scan.
            if replacement != "resize" {
                panel.rows = Arc::new(if replacement == "removed" {
                    vec![]
                } else {
                    vec![LibraryEntry {
                        path: PathBuf::from("new-page/new-original.mp4"),
                        size: 3,
                        modified_ms: Some(7),
                        file_identity: Some("replacement-native".into()),
                    }]
                });
            }
            let size = if replacement == "resize" {
                egui::vec2(480.0, 340.0)
            } else {
                egui::vec2(400.0, 300.0)
            };
            let output = detail_resize_frame(
                &ctx,
                &mut panel,
                vec![traversal_key(egui::Key::Escape, true)],
                size,
            );
            tree.merge(&output);
            let mut output = detail_resize_frame(
                &ctx,
                &mut panel,
                vec![traversal_key(egui::Key::Escape, false)],
                size,
            );
            tree.merge(&output);
            for _ in 0..4 {
                output = detail_resize_frame(&ctx, &mut panel, vec![], size);
                tree.merge(&output);
            }
            assert!(panel.path_detail.is_none());
            assert!(panel.take_preview_request().is_none());
            if replacement == "resize" {
                assert_eq!(tree.focus, Some(row));
                assert_eq!(tree.row(&label).unwrap().0, row);
                assert!(fully_visible_text(&output, &label));
                assert_eq!(panel.selected_media.as_ref(), Some(&source));
            } else {
                assert_ne!(
                    tree.focus,
                    Some(row),
                    "removed original row cannot regain focus"
                );
                assert!(
                    !tree.nodes.contains_key(&row),
                    "root-pruned tree removes stale row"
                );
                assert!(
                    panel.detail_restore.is_none(),
                    "no repeated stale restoration scroll"
                );
            }
        }
    }
    fn consistency_row<'a>(
        output: &'a egui::FullOutput,
        label: &str,
    ) -> (
        egui::accesskit::NodeId,
        &'a egui::accesskit::Node,
        egui::Rect,
    ) {
        let tree = output.platform_output.accesskit_update.as_ref().unwrap();
        let (id, node) = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap();
        let b = node.bounds().unwrap();
        (
            *id,
            node,
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            ),
        )
    }
    fn consistency_row_fill(output: &egui::FullOutput, rect: egui::Rect) -> egui::Color32 {
        output
            .shapes
            .iter()
            .find_map(|paint| match &paint.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.fill),
                _ => None,
            })
            .expect("actual row background shape")
    }
    #[test]
    fn actual_library_selected_row_hover_press_focus_disabled_remain_distinct_and_preserve_action_policy()
     {
        for dark in [false, true] {
            let fixture = Fixture::new();
            let (_, media) = fixture.paths();
            let name = format!("row-selection-{}.mp4", "long".repeat(35));
            let source = media.join(&name);
            let bytes = b"unchanged owned original media";
            std::fs::write(&source, bytes).unwrap();
            let mut panel = preview_test_panel(media, source.clone(), bytes.len() as u64);
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            ctx.enable_accesskit();
            let label = format!("{name} · {} bytes · {}", bytes.len(), c::LOCAL);
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let output = preview_panel_frame(&ctx, &mut panel, vec![], true);
            let (id, node, rect) = consistency_row(&output, &label);
            assert_eq!(node.role(), egui::accesskit::Role::Button);
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
            assert_eq!(rect.height(), crate::ui_tokens::CONTROL_HEIGHT);
            assert!(
                rect.width() > 600.0,
                "original full-width virtual row retained"
            );
            preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(id)], true);
            let idle = preview_panel_frame(&ctx, &mut panel, vec![egui::Event::PointerGone], true);
            let (selected_id, node, selected_rect) = consistency_row(&idle, &label);
            assert_eq!(selected_id, id);
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
            assert_eq!(selected_rect, rect);
            let selected_fill = consistency_row_fill(&idle, rect);
            assert_ne!(selected_fill, egui::Color32::TRANSPARENT);
            assert_eq!(
                panel.take_preview_request(),
                None,
                "selecting does not prepare or launch media"
            );
            let focus = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target: id,
                data: None,
            });
            preview_panel_frame(&ctx, &mut panel, vec![focus], true);
            let focused = preview_panel_frame(&ctx, &mut panel, vec![], true);
            assert_eq!(
                focused
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .focus,
                id
            );
            assert_eq!(
                consistency_row_fill(&focused, rect),
                selected_fill,
                "keyboard focus must not replace persistent selection fill"
            );
            let pos = rect.center();
            let hover =
                preview_panel_frame(&ctx, &mut panel, vec![egui::Event::PointerMoved(pos)], true);
            let hover_fill = consistency_row_fill(&hover, rect);
            assert_ne!(
                hover_fill, selected_fill,
                "selected row hover needs observable feedback"
            );
            let pointer = |pressed, pos| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            let press = preview_panel_frame(&ctx, &mut panel, vec![pointer(true, pos)], true);
            assert_ne!(
                consistency_row_fill(&press, rect),
                hover_fill,
                "held press must differ from selected hover"
            );
            assert_eq!(consistency_row(&press, &label).0, id);
            assert_eq!(panel.selected_media.as_ref(), Some(&source));
            assert_eq!(panel.take_preview_request(), None);
            let outside = egui::pos2(rect.right() + 5.0, rect.bottom() + 5.0);
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![egui::Event::PointerMoved(outside), pointer(false, outside)],
                true,
            );
            let disabled =
                preview_panel_frame(&ctx, &mut panel, vec![egui::Event::PointerGone], false);
            let (disabled_id, node, disabled_rect) = consistency_row(&disabled, &label);
            assert_eq!(disabled_id, id);
            assert_eq!(disabled_rect, rect);
            assert!(node.is_disabled());
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
            let disabled_fill = consistency_row_fill(&disabled, rect);
            assert_ne!(
                disabled_fill,
                egui::Color32::TRANSPARENT,
                "worker lock must retain selected location marker"
            );
            let blocked = preview_panel_frame(
                &ctx,
                &mut panel,
                vec![
                    egui::Event::PointerMoved(pos),
                    pointer(true, pos),
                    preview_accesskit_click(id),
                    preview_enter(),
                ],
                false,
            );
            assert_eq!(
                consistency_row_fill(&blocked, rect),
                disabled_fill,
                "disabled hover/press must not become active feedback"
            );
            preview_panel_frame(&ctx, &mut panel, vec![pointer(false, pos)], false);
            assert_eq!(panel.selected_media.as_ref(), Some(&source));
            assert_eq!(
                panel.take_preview_request(),
                None,
                "disabled pointer, Enter and AccessKit actions are refused"
            );
            assert_eq!(std::fs::read(&source).unwrap(), bytes);
            assert!(!fixture.0.join("data/library.sqlite").exists());
        }
    }
    #[test]
    fn actual_library_double_click_previews_once_and_all_existing_gates_refuse_it() {
        for mode in 0..4 {
            let fixture = Fixture::new();
            let (_, media) = fixture.paths();
            let path = media.join("double-click-original.mp4");
            std::fs::write(&path, b"same original bytes").unwrap();
            let mut panel = preview_test_panel(media, path.clone(), 19);
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let label = format!("double-click-original.mp4 · 19 bytes · {}", c::LOCAL);
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let (id, bounds) =
                preview_node(preview_panel_frame(&ctx, &mut panel, vec![], true), &label);
            preview_focus_row(&ctx, &mut panel, bounds);
            assert!(
                panel.take_preview_request().is_none(),
                "single click selects only"
            );
            let (current_id, bounds) =
                preview_node(preview_panel_frame(&ctx, &mut panel, vec![], true), &label);
            assert_eq!(current_id, id);
            if mode == 1 {
                panel.pending = Some(9)
            }
            if mode == 2 {
                panel.signals.stop.store(true, Ordering::Release)
            }
            let enabled = mode != 3;
            let pos = egui::pos2(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            );
            let button = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![egui::Event::PointerMoved(pos), button(true)],
                enabled,
            );
            preview_panel_frame(&ctx, &mut panel, vec![button(false)], enabled);
            assert_eq!(
                panel.take_preview_request(),
                (mode == 0).then(|| path.clone())
            );
            assert!(panel.take_preview_request().is_none());
            preview_panel_frame(&ctx, &mut panel, vec![preview_enter()], enabled);
            assert_eq!(
                panel.take_preview_request(),
                (mode == 0).then(|| path.clone()),
                "Enter obeys exact same gate"
            );
            assert_eq!(std::fs::read(path).unwrap(), b"same original bytes");
            assert!(!fixture.0.join("data/library.sqlite").exists());
        }
    }
    #[test]
    fn actual_library_explicit_system_buttons_queue_typed_original_row_and_never_preview() {
        if !crate::library_open::available() {
            return;
        }
        for kind in [
            crate::library_open::Kind::File,
            crate::library_open::Kind::Folder,
        ] {
            let fixture = Fixture::new();
            let (_, media) = fixture.paths();
            let path = media.join("original.mp4");
            std::fs::write(&path, b"original").unwrap();
            let mut panel = preview_test_panel(media.clone(), path.clone(), 8);
            let (commands, rx) = mpsc::sync_channel(1);
            panel.commands = commands;
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let row_label = format!("original.mp4 · 8 bytes · {}", c::LOCAL);
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let (row, _) = preview_node(
                preview_panel_frame(&ctx, &mut panel, vec![], true),
                &row_label,
            );
            preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(row)], true);
            let action = if kind == crate::library_open::Kind::File {
                c::LIBRARY_OPEN_DEFAULT
            } else {
                c::LIBRARY_OPEN_FOLDER
            };
            let (button, _) =
                preview_node(preview_panel_frame(&ctx, &mut panel, vec![], true), action);
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![preview_accesskit_click(button)],
                true,
            );
            let command = rx.try_recv().unwrap();
            let Action::SystemOpen {
                root,
                row,
                kind: actual,
            } = command.action
            else {
                panic!("typed original handoff command")
            };
            assert_eq!(actual, kind);
            assert_eq!(root, media);
            assert_eq!(row.path, path);
            assert_eq!(row.size, 8);
            assert_eq!(row.file_identity.as_deref(), Some("fixture-source"));
            assert!(panel.busy());
            assert!(panel.take_preview_request().is_none());
            assert_eq!(std::fs::read(&path).unwrap(), b"original");
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![preview_accesskit_click(button)],
                true,
            );
            assert!(rx.try_recv().is_err(), "cap1 busy gate prevents repeat");
        }
    }
    #[test]
    fn worker_row_snapshot_and_actual_poll_handoff_do_not_rebind_root_selection_or_rows() {
        let root = PathBuf::from("selected-root");
        let row = LibraryEntry {
            path: root.join("original.mp4"),
            size: 3,
            modified_ms: Some(1),
            file_identity: Some("original-id".into()),
        };
        let delivered = DeliveredState {
            search: Default::default(),
            root: root.clone(),
            rows: Ok(Arc::new(vec![row.clone()])),
            health: None,
        };
        assert!(current_open_row(&root, &root, &row, Some(&delivered)));
        let mut stale = row.clone();
        stale.file_identity = Some("replacement-id".into());
        assert!(!current_open_row(&root, &root, &stale, Some(&delivered)));
        assert!(!current_open_row(
            &root,
            &PathBuf::from("another-root"),
            &row,
            Some(&delivered)
        ));
        assert!(!current_open_row(&root, &root, &row, None));
        let mut panel = preview_test_panel(root.clone(), row.path.clone(), 3);
        let original_rows = panel.rows.clone();
        panel.selected_media = Some(row.path.clone());
        panel.pending = Some(5);
        let (tx, rx) = mpsc::sync_channel(1);
        panel.results = rx;
        tx.send(Snapshot {
            search: Default::default(),
            opened: Some(Ok(crate::library_open::Report::AfterCancellation)),
            serial: Some(5),
            root: None,
            rows: Ok(Arc::default()),
            health: None,
        })
        .unwrap();
        panel.poll(&egui::Context::default());
        assert!(!panel.busy());
        assert_eq!(panel.root, root);
        assert_eq!(panel.selected_media.as_ref(), Some(&row.path));
        assert!(Arc::ptr_eq(&original_rows, &panel.rows));
        assert_eq!(
            panel.system_open_status,
            Some(crate::library_open::Report::AfterCancellation)
        );
        assert!(panel.take_preview_request().is_none());
        assert!(panel.applied.is_none());
    }
    #[test]
    fn modified_metadata_utc_epoch_leap_day_and_unknown_bounds_are_explicit() {
        assert_eq!(modified_time(Some(0)), "1970-01-01 00:00 UTC");
        assert_eq!(modified_time(Some(951_782_400_000)), "2000-02-29 00:00 UTC");
        assert_eq!(modified_time(Some(1709210340000)), "2024-02-29 12:39 UTC");
        assert_eq!(modified_time(None), c::LIBRARY_MODIFIED_UNKNOWN);
        assert_eq!(modified_time(Some(-1)), c::LIBRARY_MODIFIED_UNKNOWN);
        assert_eq!(modified_time(Some(i64::MAX)), c::LIBRARY_MODIFIED_UNKNOWN);
    }
    #[test]
    fn actual_selected_metadata_and_secondary_reader_path_preserve_original_row_identity() {
        let fixture = Fixture::new();
        let (_, media) = fixture.paths();
        let path = media.join("한글 selected.mp4");
        std::fs::write(&path, b"original").unwrap();
        let mut panel = preview_test_panel(media, path.clone(), 8);
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let label = format!("한글 selected.mp4 · 8 bytes · {}", c::LOCAL);
        preview_panel_frame(&ctx, &mut panel, vec![], true);
        let output = preview_panel_frame(&ctx, &mut panel, vec![], true);
        let (id, node, _) = consistency_row(&output, &label);
        assert_eq!(node.description(), Some(path.to_string_lossy().as_ref()));
        preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(id)], true);
        let output = preview_panel_frame(&ctx, &mut panel, vec![], true);
        let (selected_id, node, _) = consistency_row(&output, &label);
        assert_eq!(selected_id, id);
        assert_eq!(node.label(), Some(label.as_str()));
        assert_eq!(node.description(), Some(path.to_string_lossy().as_ref()));
        assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
        let visible = |needle: &str| {
            output.shapes.iter().any(|paint| {
                if let egui::Shape::Text(text) = &paint.shape {
                    let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                    text.galley.text().contains(needle)
                        && paint.clip_rect.contains_rect(rect)
                        && ctx.content_rect().contains_rect(rect)
                } else {
                    false
                }
            })
        };
        assert!(visible("8 bytes"));
        assert!(visible(&format!(
            "{} 1970-01-01 00:00 UTC",
            c::LIBRARY_MODIFIED
        )));
        assert!(panel.take_preview_request().is_none());
        assert_eq!(std::fs::read(path).unwrap(), b"original");
    }
    #[test]
    fn export_context_invalid_name_does_not_emit_and_valid_submit_binds_original_row() {
        let fixture = Fixture::new();
        let (_, root) = fixture.paths();
        let source = root.join("원본.mp4");
        let mut panel = preview_test_panel(root.clone(), source.clone(), 7);
        panel.selected_media = Some(source);
        let originals = panel.rows.clone();
        let (commands, rx) = mpsc::sync_channel(1);
        panel.commands = commands;
        panel.begin_export(true);
        panel.export_draft.as_mut().unwrap().name = "CON.mp4".into();
        let ctx = egui::Context::default();
        let output = preview_panel_frame(&ctx, &mut panel, vec![], true);
        assert!(painted_text(&output.shapes).contains(c::LIBRARY_EXPORT_INVALID_NAME));
        panel.submit_export(true);
        assert!(rx.try_recv().is_err());
        panel.export_draft.as_mut().unwrap().name = "새 영상 🎬.mp4".into();
        panel.submit_export(true);
        let command = rx.try_recv().unwrap();
        let Action::Export {
            root: requested,
            row,
            parent,
            name,
            ..
        } = command.action
        else {
            panic!("exact export command")
        };
        assert_eq!(requested, root);
        assert_eq!(parent, root);
        assert_eq!(row, originals[0]);
        assert_eq!(name, "새 영상 🎬.mp4");
        assert_eq!(panel.pending, Some(command.serial));
        assert!(Arc::ptr_eq(&panel.rows, &originals));
    }
    #[test]
    fn export_picker_generation_root_and_changed_row_are_not_authority() {
        let fixture = Fixture::new();
        let (_, root) = fixture.paths();
        let mut panel = preview_test_panel(root.clone(), root.join("source.mp4"), 7);
        panel.selected_media = Some(root.join("source.mp4"));
        panel.begin_export(true);
        let old_generation = panel.export_draft.as_ref().unwrap().generation;
        panel.begin_export(true);
        panel.apply_export_folder(old_generation, root.join("wrong"));
        assert_eq!(panel.export_draft.as_ref().unwrap().parent, root);
        let generation = panel.export_draft.as_ref().unwrap().generation;
        panel.root = root.join("changed-root");
        panel.apply_export_folder(generation, root.join("wrong"));
        assert_eq!(panel.export_draft.as_ref().unwrap().parent, root);
        let (commands, rx) = mpsc::sync_channel(1);
        panel.commands = commands;
        panel.submit_export(true);
        assert!(rx.try_recv().is_err());
        panel.root = root.clone();
        let mut replaced = panel.rows[0].clone();
        replaced.file_identity = Some("new-id".into());
        panel.rows = Arc::new(vec![replaced]);
        panel.submit_export(true);
        assert!(rx.try_recv().is_err());
        panel.apply_export_folder(generation, root.join("wrong"));
        assert_eq!(panel.export_draft.as_ref().unwrap().parent, root);
        panel.request_stop();
        panel.apply_export_folder(generation, root.join("wrong"));
        assert!(panel.export_draft.is_none());
    }
    #[test]
    fn export_stop_retains_uncertain_terminal_receipt_without_reselecting_original() {
        let fixture = Fixture::new();
        let (_, root) = fixture.paths();
        let mut panel = preview_test_panel(root.clone(), root.join("source.mp4"), 7);
        panel.pending = Some(4);
        panel.export_progress = Some(ProgressFeed::default());
        let cancel = Arc::new(AtomicBool::new(false));
        panel.export_cancel = Some(cancel.clone());
        panel.selected_media = Some(root.join("other-choice.mp4"));
        assert_eq!(
            panel.background_export_status().as_deref(),
            Some(c::LIBRARY_EXPORT_PREPARING)
        );
        panel.request_stop();
        assert!(cancel.load(Ordering::Acquire));
        assert_eq!(
            panel.background_export_status().as_deref(),
            Some(c::LIBRARY_EXPORT_STOPPING)
        );
        let attempted = root.join(".dlpad-export-owned.partial");
        *panel.signals.export_terminal.lock().unwrap() = Some((
            4,
            Err(file_export::Failure {
                error: std::io::ErrorKind::Interrupted.into(),
                creation: file_export::Creation::AttemptedUnconfirmed,
                publication: file_export::Publication::NotAttempted,
                cleanup: file_export::Cleanup::Unconfirmed,
                stage_path: Some(attempted.clone()),
                target_path: None,
                created_identity: None,
            }),
        ));
        panel.signals.stopped.store(true, Ordering::Release);
        panel.poll(&egui::Context::default());
        assert!(!panel.busy());
        assert!(panel.export_progress.is_none());
        let failure = panel.export_result.as_ref().unwrap().as_ref().unwrap_err();
        assert_eq!(failure.stage_path.as_ref(), Some(&attempted));
        assert_eq!(export_failure_text(failure), c::LIBRARY_EXPORT_RETAINED);
        assert_eq!(panel.selected_media, Some(root.join("other-choice.mp4")));
        assert!(panel.take_preview_request().is_none());
        assert!(panel.background_export_status().is_none());
        assert_eq!(
            panel.take_export_notice(),
            Some(c::LIBRARY_EXPORT_RETAINED_NOTICE)
        );
        assert!(panel.take_export_notice().is_none());
        assert!(
            panel.export_result.as_ref().unwrap().is_err(),
            "notification preserves full uncertain evidence"
        );
    }

    // Candidate-only tests reuse the original pure panel/AccessKit fixtures, never new()'s I/O worker.
    fn location_panel() -> (LibraryPanel, Receiver<Command>, SyncSender<Snapshot>) {
        let root = std::env::temp_dir().join("dlpad-location-context-original");
        let mut panel = preview_test_panel(root.clone(), root.join("original.mp4"), 3);
        let (commands, receiver) = mpsc::sync_channel(1);
        let (sender, results) = mpsc::sync_channel(1);
        panel.commands = commands;
        panel.results = results;
        (panel, receiver, sender)
    }
    fn location_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        tree: &mut TraversalTree,
        settings: bool,
        can_change: bool,
        picker_available: bool,
        events: Vec<egui::Event>,
    ) {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(320.0, 560.0),
                )),
                focused: true,
                time: Some(ctx.input(|i| i.time) + 0.1),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if settings {
                        panel.show_location_settings(ui, can_change, picker_available);
                    } else {
                        panel.show(ui, can_change, picker_available);
                    }
                });
            },
        );
        tree.merge(&output);
    }
    fn location_context(theme: egui::Theme, scale: f32) -> egui::Context {
        let ctx = egui::Context::default();
        crate::ui_theme::configure_dlpad_theme(&ctx);
        ctx.set_theme(theme);
        ctx.set_pixels_per_point(scale);
        ctx.enable_accesskit();
        ctx
    }
    #[test]
    fn actual_location_editor_apply_uses_original_queue_serial_and_disabled_gate() {
        use egui::accesskit::{Action as UiAction, ActionRequest, Role};
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            for scale in [1.0, 2.0] {
                let (mut panel, commands, results) = location_panel();
                let original = panel.root.clone();
                let ctx = location_context(theme, scale);
                let mut tree = TraversalTree::default();
                location_frame(&ctx, &mut panel, &mut tree, true, true, true, vec![]);
                let fields: Vec<_> = tree
                    .nodes
                    .iter()
                    .filter(|(_, n)| {
                        n.role() == Role::TextInput && n.value() == Some(panel.edit.as_str())
                    })
                    .collect();
                assert_eq!(fields.len(), 1, "actual unique original root editor");
                let field = *fields[0].0;
                location_frame(
                    &ctx,
                    &mut panel,
                    &mut tree,
                    true,
                    true,
                    true,
                    vec![egui::Event::AccessKitActionRequest(ActionRequest {
                        action: UiAction::Focus,
                        target: field,
                        data: None,
                    })],
                );
                let wanted = std::env::temp_dir().join("dlpad-location-context-edited");
                let text = crate::path_picker::display_path(&wanted);
                location_frame(
                    &ctx,
                    &mut panel,
                    &mut tree,
                    true,
                    true,
                    true,
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
                assert_eq!(panel.edit, text, "actual text editing");
                assert_eq!(panel.root, original, "draft is not committed location");
                assert!(commands.try_recv().is_err());
                let apply = tree.row(c::APPLY_ROOT).unwrap().0;
                location_frame(
                    &ctx,
                    &mut panel,
                    &mut tree,
                    true,
                    true,
                    true,
                    vec![preview_accesskit_click(apply)],
                );
                let command = commands.try_recv().unwrap();
                assert!(matches!(&command.action,Action::Apply(path) if path==&wanted));
                assert_eq!(panel.pending, Some(command.serial));
                assert_eq!(panel.take_location_request_started(), Some(command.serial));
                assert!(panel.take_location_request_started().is_none());
                assert!(panel.take_applied_root().is_none());
                location_frame(&ctx, &mut panel, &mut tree, true, true, true, vec![]);
                assert!(tree.row(c::APPLY_ROOT).unwrap().1.is_disabled());
                assert!(tree.nodes[&field].is_disabled());
                let saved = panel.edit.clone();
                location_frame(
                    &ctx,
                    &mut panel,
                    &mut tree,
                    true,
                    true,
                    true,
                    vec![
                        preview_accesskit_click(apply),
                        egui::Event::Text("forged edit".into()),
                    ],
                );
                assert_eq!(panel.edit, saved);
                assert!(commands.try_recv().is_err());
                results
                    .send(delivery_snapshot(
                        Some(command.serial),
                        Some(wanted.clone()),
                        Ok(Arc::default()),
                        None,
                    ))
                    .unwrap();
                panel.poll(&ctx);
                assert_eq!(panel.root, wanted);
                assert!(!panel.busy());
                // Rendering either screen must not consume the committed ACK; Main can take it once while hidden.
                location_frame(&ctx, &mut panel, &mut tree, false, true, true, vec![]);
                assert_eq!(panel.take_applied_root(), Some((command.serial, wanted)));
                assert!(panel.take_applied_root().is_none());
                assert!(
                    !tree.nodes.contains_key(&field),
                    "readonly list removes the original root editor node"
                );
                let inputs = tree
                    .nodes
                    .values()
                    .filter(|node| node.role() == Role::TextInput)
                    .collect::<Vec<_>>();
                assert_eq!(
                    inputs.len(),
                    1,
                    "readonly list retains only its search input"
                );
                assert_eq!(
                    inputs[0].description(),
                    Some(c::LIST_SEARCH_HELP),
                    "actual remaining TextInput is the search field"
                );
                assert!(tree.row(c::APPLY_ROOT).is_none());
                assert!(tree.row(c::DEFAULT_ROOT).is_none());
                assert!(tree.row(c::CHOOSE_FOLDER).is_none());
            }
        }
    }
    #[test]
    fn actual_location_default_picker_and_closewait_keep_original_actions() {
        for action in [c::DEFAULT_ROOT, c::CHOOSE_FOLDER] {
            let (mut panel, commands, _results) = location_panel();
            let root = panel.root.clone();
            let ctx = location_context(egui::Theme::Light, 2.0);
            let mut tree = TraversalTree::default();
            location_frame(&ctx, &mut panel, &mut tree, true, false, true, vec![]);
            let id = tree.row(action).unwrap().0;
            assert!(tree.row(action).unwrap().1.is_disabled());
            let edit = panel.edit.clone();
            location_frame(
                &ctx,
                &mut panel,
                &mut tree,
                true,
                false,
                true,
                vec![preview_accesskit_click(id)],
            );
            assert!(commands.try_recv().is_err());
            assert!(panel.take_browse_request().is_none());
            assert!(panel.take_location_request_started().is_none());
            assert_eq!(panel.edit, edit);
            location_frame(&ctx, &mut panel, &mut tree, true, true, true, vec![]);
            assert_eq!(
                tree.row(action).unwrap().0,
                id,
                "re-enable retains original control ID"
            );
            location_frame(
                &ctx,
                &mut panel,
                &mut tree,
                true,
                true,
                true,
                vec![preview_accesskit_click(id)],
            );
            if action == c::DEFAULT_ROOT {
                let command = commands.try_recv().unwrap();
                assert!(matches!(command.action, Action::Default));
                assert_eq!(panel.take_location_request_started(), Some(command.serial));
            } else {
                assert!(commands.try_recv().is_err());
                assert_eq!(panel.take_browse_request(), Some(root.clone()));
                assert!(panel.take_browse_request().is_none());
                let selected = root.join("picked");
                panel.apply_selected_root(selected.clone());
                let command = commands.try_recv().unwrap();
                assert!(matches!(command.action,Action::Apply(ref path) if path==&selected));
                assert_eq!(panel.take_location_request_started(), Some(command.serial));
            }
            assert_eq!(
                panel.root, root,
                "accepted commands still require worker commit"
            );
        }
        let (mut panel, commands, _results) = location_panel();
        let ctx = location_context(egui::Theme::Dark, 1.0);
        let mut tree = TraversalTree::default();
        location_frame(&ctx, &mut panel, &mut tree, true, true, false, vec![]);
        assert!(tree.row(c::CHOOSE_FOLDER).unwrap().1.is_disabled());
        panel.signals.stop.store(true, Ordering::Release);
        location_frame(&ctx, &mut panel, &mut tree, true, true, true, vec![]);
        assert!(tree.row(c::APPLY_ROOT).unwrap().1.is_disabled());
        panel.apply_selected_root(panel.root.join("rejected"));
        assert!(commands.try_recv().is_err());
        assert!(panel.take_location_request_started().is_none());
    }
    #[test]
    fn location_watch_and_wrong_serial_are_not_main_output_authority() {
        let (mut panel, commands, results) = location_panel();
        let ctx = location_context(egui::Theme::Light, 1.0);
        let root = panel.root.clone();
        panel.apply_selected_root(root.join("new"));
        let command = commands.try_recv().unwrap();
        assert_eq!(panel.take_location_request_started(), Some(command.serial));
        results
            .send(delivery_snapshot(
                Some(command.serial + 1),
                Some(root.join("stale")),
                Ok(Arc::default()),
                None,
            ))
            .unwrap();
        panel.poll(&ctx);
        assert!(panel.take_applied_root().is_none());
        assert_eq!(panel.pending, Some(command.serial));
        assert_eq!(panel.root, root);
        results
            .send(delivery_snapshot(None, None, Ok(Arc::default()), None))
            .unwrap();
        panel.poll(&ctx);
        assert!(panel.take_applied_root().is_none());
        assert!(panel.take_location_request_started().is_none());
        assert_eq!(panel.pending, Some(command.serial));
        let expected = root.join("new");
        results
            .send(delivery_snapshot(
                Some(command.serial),
                Some(expected.clone()),
                Ok(Arc::default()),
                None,
            ))
            .unwrap();
        panel.poll(&ctx);
        assert_eq!(panel.take_applied_root(), Some((command.serial, expected)));
        assert!(panel.take_applied_root().is_none());
        panel.request(Action::Refresh);
        let refresh = commands.try_recv().unwrap();
        assert!(matches!(refresh.action, Action::Refresh));
        assert!(panel.take_location_request_started().is_none());
    }
    #[test]
    fn observed_restore_uses_original_apply_without_minting_or_losing_explicit_started() {
        let (mut panel, commands, results) = location_panel();
        let ctx = location_context(egui::Theme::Light, 1.0);
        let restored = panel.root.join("restored");
        panel.apply_observed_root(restored.clone());
        let command = commands.try_recv().unwrap();
        assert!(matches!(&command.action,Action::Apply(path) if path==&restored));
        assert_eq!(panel.pending, Some(command.serial));
        assert!(panel.take_location_request_started().is_none());
        results
            .send(delivery_snapshot(
                Some(command.serial),
                Some(restored.clone()),
                Ok(Arc::default()),
                None,
            ))
            .unwrap();
        panel.poll(&ctx);
        assert_eq!(
            panel.take_applied_root(),
            Some((command.serial, restored.clone()))
        );
        assert!(panel.take_applied_root().is_none());
        // A rejected startup restore must not consume an earlier real explicit Apply observation.
        panel.apply_selected_root(restored.join("explicit"));
        let explicit = commands.try_recv().unwrap();
        panel.apply_observed_root(restored.join("busy-rejected"));
        assert!(commands.try_recv().is_err());
        assert_eq!(panel.pending, Some(explicit.serial));
        assert_eq!(panel.take_location_request_started(), Some(explicit.serial));
        assert!(panel.take_location_request_started().is_none());
    }

    // Test-only transport for Main's actual panel renderer. No worker is created.
    impl LibraryPanel {
        pub(crate) fn copy_context_fixture(root: PathBuf, source: PathBuf) -> Self {
            let mut panel = preview_test_panel(root.clone(), source.clone(), 8);
            let (commands, receiver) = mpsc::sync_channel(1);
            panel.commands = commands;
            panel.selected_media = Some(source.clone());
            panel.begin_export(true);
            panel.submit_export(true);
            let command = receiver
                .try_recv()
                .expect("real production export dispatch");
            let Action::Export {
                root: requested,
                row,
                parent,
                progress,
                cancel,
                ..
            } = command.action
            else {
                panic!("real exact Export command")
            };
            assert_eq!(requested, root);
            assert_eq!(parent, root);
            assert_eq!(row.path, source);
            assert!(Arc::ptr_eq(&cancel, panel.export_cancel.as_ref().unwrap()));
            assert!(progress.latest().is_none());
            assert_eq!(panel.pending, Some(command.serial));
            panel
        }
        pub(crate) fn copy_context_cancelled(&self) -> bool {
            self.export_cancel
                .as_ref()
                .is_some_and(|flag| flag.load(Ordering::Acquire))
        }
        pub(crate) fn copy_context_ack(&mut self, stale: bool, success: bool) {
            let serial = self
                .pending
                .expect("accepted export still owns worker lifetime");
            let result = if success {
                let source = std::fs::File::open(&self.rows[0].path).unwrap();
                let path = self.root.join("completed-copy.mp4");
                let target = std::fs::File::open(&path).unwrap();
                Ok(file_export::Exported {
                    path,
                    bytes: 8,
                    sha256: "test-terminal-observation-not-core-verification".into(),
                    source_identity: dlpad_engine::file_identity::from_file(&source).unwrap(),
                    target_identity: dlpad_engine::file_identity::from_file(&target).unwrap(),
                })
            } else {
                Err(file_export::Failure {
                    error: std::io::ErrorKind::Interrupted.into(),
                    creation: file_export::Creation::AttemptedUnconfirmed,
                    publication: file_export::Publication::NotAttempted,
                    cleanup: file_export::Cleanup::Unconfirmed,
                    stage_path: Some(self.root.join(".dlpad-export-owned.partial")),
                    target_path: None,
                    created_identity: None,
                })
            };
            *self.signals.export_terminal.lock().unwrap() =
                Some((if stale { serial + 1 } else { serial }, result));
        }
        pub(crate) fn copy_context_retained(&self) -> bool {
            self.export_result.as_ref().is_some_and(|result| {
                result.as_ref().is_err_and(|failure| {
                    failure.stage_path.as_ref()
                        == Some(&self.root.join(".dlpad-export-owned.partial"))
                        && matches!(failure.cleanup, file_export::Cleanup::Unconfirmed)
                        && matches!(
                            failure.creation,
                            file_export::Creation::AttemptedUnconfirmed
                        )
                })
            })
        }
        pub(crate) fn copy_context_worker_ack(&self) {
            self.signals.stopped.store(true, Ordering::Release);
        }
    }
    // Production panel/virtual rows; no worker/native file-open or preview execution.
    fn selection_summary_frame(
        ctx: &egui::Context,
        panel: &mut LibraryPanel,
        physical_width: f32,
        scale: f32,
        events: Vec<egui::Event>,
        can_change: bool,
    ) -> egui::FullOutput {
        let time = ctx.input(|i| i.time) + 0.01;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(physical_width / scale, 600.0 / scale),
                )),
                time: Some(time),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::TopBottomPanel::top("selection-fixture-navigation")
                    .exact_height(48.0)
                    .show(ctx, |ui| {
                        ui.label(c::LIBRARY);
                    });
                egui::TopBottomPanel::bottom("selection-fixture-footer")
                    .exact_height(32.0)
                    .show(ctx, |ui| {
                        ui.weak("fixture footer");
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(c::LIBRARY);
                    ui.add_space(crate::ui_tokens::CONTROL_GAP);
                    panel.show(ui, can_change, false);
                });
            },
        )
    }
    // Diagnostic only: preserve the exact label matcher and all acceptance guards.
    fn summary_diagnostic_row<'a>(
        output: &'a egui::FullOutput,
        label: &str,
        ctx: &egui::Context,
        scene: &str,
        stage: &str,
    ) -> (
        egui::accesskit::NodeId,
        &'a egui::accesskit::Node,
        egui::Rect,
    ) {
        let tree = output.platform_output.accesskit_update.as_ref();
        let found = tree.is_some_and(|tree| {
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some(label))
        });
        if !found {
            eprintln!(
                "SUMMARY_MISSING scene={scene} stage={stage} expected={label:?} frame={} content={:?} focused={}",
                ctx.cumulative_frame_nr(),
                ctx.content_rect(),
                ctx.input(|i| i.focused)
            );
            if let Some(tree) = tree {
                eprintln!("SUMMARY_UPDATE nodes={}", tree.nodes.len());
                for (id, node) in tree
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.label().is_some() || node.value().is_some())
                    .take(32)
                {
                    eprintln!(
                        "SUMMARY_NODE id={id:?} label={:?} value={:?} bounds={:?} role={:?}",
                        node.label(),
                        node.value(),
                        node.bounds(),
                        node.role()
                    );
                }
            } else {
                eprintln!("SUMMARY_UPDATE absent");
            }
            fn rects(shapes: &[egui::epaint::ClippedShape], count: &mut usize) {
                fn shape(current: &egui::epaint::Shape, clip: egui::Rect, count: &mut usize) {
                    if *count >= 16 {
                        return;
                    }
                    match current {
                        egui::epaint::Shape::Rect(rect) => {
                            eprintln!("SUMMARY_RECT rect={:?} clip={clip:?}", rect.rect);
                            *count += 1;
                        }
                        egui::epaint::Shape::Vec(items) => {
                            for item in items {
                                shape(item, clip, count);
                            }
                        }
                        _ => {}
                    }
                }
                for item in shapes {
                    shape(&item.shape, item.clip_rect, count);
                }
            }
            rects(&output.shapes, &mut 0);
        }
        consistency_row(output, label)
    }
    // egui response.rs publishes readonly Label text as value, never button label.
    fn summary_diagnostic_text<'a>(
        output: &'a egui::FullOutput,
        value: &str,
        ctx: &egui::Context,
        scene: &str,
        stage: &str,
    ) -> (
        egui::accesskit::NodeId,
        &'a egui::accesskit::Node,
        egui::Rect,
    ) {
        let tree = output.platform_output.accesskit_update.as_ref().unwrap();
        let found = tree.nodes.iter().find(|(_, node)| {
            node.role() == egui::accesskit::Role::Label && node.value() == Some(value)
        });
        if found.is_none() {
            eprintln!(
                "SUMMARY_TEXT_MISSING scene={scene} stage={stage} expected={value:?} frame={} content={:?} focused={}",
                ctx.cumulative_frame_nr(),
                ctx.content_rect(),
                ctx.input(|i| i.focused)
            );
            for (id, node) in tree
                .nodes
                .iter()
                .filter(|(_, node)| node.label().is_some() || node.value().is_some())
                .take(32)
            {
                eprintln!(
                    "SUMMARY_TEXT_NODE id={id:?} label={:?} value={:?} role={:?} bounds={:?}",
                    node.label(),
                    node.value(),
                    node.role(),
                    node.bounds()
                );
            }
        }
        let (id, node) = found.expect("actual readonly Role::Label must publish exact value");
        let b = node.bounds().unwrap();
        (
            *id,
            node,
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            ),
        )
    }
    fn summary_slot_height(
        output: &egui::FullOutput,
        spacing: f32,
        ctx: &egui::Context,
        scene: &str,
    ) -> f32 {
        let select = summary_diagnostic_text(
            output,
            c::LIBRARY_SELECT_HELP,
            ctx,
            scene,
            "footprint-select-help",
        )
        .2;
        let actions = summary_diagnostic_text(
            output,
            c::LIBRARY_FILE_ACTION_HELP,
            ctx,
            scene,
            "footprint-file-help",
        )
        .2;
        let unsupported = if crate::library_open::available() {
            0.0
        } else {
            summary_diagnostic_text(
                output,
                c::LIBRARY_OPEN_UNSUPPORTED,
                ctx,
                scene,
                "footprint-unsupported",
            )
            .2
            .height()
                + spacing
        };
        actions.top() - select.bottom() - 2.0 * spacing - unsupported
    }
    #[test]
    fn actual_selection_summary_is_one_fixed_row_and_first_selection_preserves_double_click_target()
    {
        for physical_width in [800.0, 960.0] {
            for scale in [1.0, 1.5, 2.0] {
                for dark in [false, true] {
                    let scene = format!("width={physical_width} scale={scale} dark={dark}");
                    let fixture = Fixture::new();
                    let (_, media) = fixture.paths();
                    // Bounded Windows-safe leaf; Context does not read/decode it.
                    let basename = format!("{}.mp4", "한글🙂".repeat(35));
                    let path = media.join(&basename);
                    let mut panel = preview_test_panel(media, path.clone(), 3);
                    let ctx = egui::Context::default();
                    crate::ui_theme::configure_dlpad_theme(&ctx);
                    ctx.set_visuals(if dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });
                    ctx.enable_accesskit();
                    ctx.set_zoom_factor(scale);
                    let frame = |panel: &mut LibraryPanel, events| {
                        selection_summary_frame(&ctx, panel, physical_width, scale, events, true)
                    };
                    frame(&mut panel, vec![]);
                    let initial = frame(&mut panel, vec![]);
                    let row_label = format!("{basename} · 3 bytes · {}", c::LOCAL);
                    let (row, _, _) =
                        summary_diagnostic_row(&initial, &row_label, &ctx, &scene, "initial-row");
                    // Reveal through the actual production row focus route, before first click.
                    frame(
                        &mut panel,
                        vec![egui::Event::AccessKitActionRequest(
                            egui::accesskit::ActionRequest {
                                action: egui::accesskit::Action::Focus,
                                target: row,
                                data: None,
                            },
                        )],
                    );
                    for _ in 0..30 {
                        frame(&mut panel, vec![]);
                    }
                    let before = frame(&mut panel, vec![]);
                    let (same, node, rect) = summary_diagnostic_row(
                        &before,
                        &row_label,
                        &ctx,
                        &scene,
                        "revealed-before-row",
                    );
                    assert_eq!(same, row);
                    assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
                    assert!(
                        ctx.content_rect().contains_rect(rect),
                        "test uses an actually revealed row, not an offscreen click"
                    );
                    let spacing = ctx.style().spacing.item_spacing.y;
                    assert!(
                        (summary_slot_height(&before, spacing, &ctx, &scene)
                            - crate::ui_tokens::CONTROL_HEIGHT)
                            .abs()
                            < 0.1,
                        "actual production summary footprint must be one32px row"
                    );
                    let pos = rect.center();
                    let pointer = |pressed| egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    };
                    frame(
                        &mut panel,
                        vec![egui::Event::PointerMoved(pos), pointer(true)],
                    );
                    frame(&mut panel, vec![pointer(false)]);
                    assert_eq!(panel.selected_media.as_ref(), Some(&path));
                    assert!(panel.take_preview_request().is_none());
                    let selected = frame(&mut panel, vec![]);
                    let (selected_id, node, selected_rect) =
                        summary_diagnostic_row(&selected, &row_label, &ctx, &scene, "selected-row");
                    assert_eq!(selected_id, row);
                    assert_eq!(
                        selected_rect, rect,
                        "selection metadata must not move the second-click target"
                    );
                    assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
                    assert!(
                        (summary_slot_height(&selected, spacing, &ctx, &scene)
                            - crate::ui_tokens::CONTROL_HEIGHT)
                            .abs()
                            < 0.1
                    );
                    let summary = format!("{} {}", c::LIBRARY_SELECTED_FILE, basename);
                    let (_, summary_node, summary_rect) = summary_diagnostic_text(
                        &selected,
                        &summary,
                        &ctx,
                        &scene,
                        "selected-summary",
                    );
                    let metadata =
                        format!("3 bytes · {} 1970-01-01 00:00 UTC", c::LIBRARY_MODIFIED);
                    let (_, metadata_node, metadata_rect) = summary_diagnostic_text(
                        &selected,
                        &metadata,
                        &ctx,
                        &scene,
                        "selected-metadata",
                    );
                    assert_eq!(summary_rect.height(), crate::ui_tokens::CONTROL_HEIGHT);
                    assert_eq!(metadata_rect.height(), crate::ui_tokens::CONTROL_HEIGHT);
                    assert!(
                        summary_rect.right() <= metadata_rect.left(),
                        "long title cannot push metadata"
                    );
                    assert!(
                        ctx.content_rect().contains_rect(summary_rect)
                            && ctx.content_rect().contains_rect(metadata_rect)
                    );
                    assert!(
                        summary_node
                            .description()
                            .unwrap()
                            .contains(path.to_string_lossy().as_ref())
                    );
                    assert!(
                        metadata_node
                            .description()
                            .unwrap()
                            .contains("1970-01-01 00:00 UTC")
                    );
                    // Same original position, not newly recomputed selected coordinates.
                    frame(&mut panel, vec![pointer(true)]);
                    frame(&mut panel, vec![pointer(false)]);
                    assert_eq!(panel.take_preview_request(), Some(path.clone()));
                    assert!(panel.take_preview_request().is_none());
                    frame(&mut panel, vec![preview_enter()]);
                    assert_eq!(panel.take_preview_request(), Some(path));
                    assert!(panel.take_preview_request().is_none());
                    assert!(panel.pending.is_none());
                    assert!(!fixture.0.join("data/library.sqlite").exists());
                }
            }
        }
    }
    #[test]
    fn actual_fixed_selection_summary_keeps_existing_disabled_busy_and_closewait_preview_gates() {
        for mode in 0..3 {
            let fixture = Fixture::new();
            let (_, media) = fixture.paths();
            let path = media.join("gate-original.mp4");
            let mut panel = preview_test_panel(media, path.clone(), 3);
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            let row_label = format!("gate-original.mp4 · 3 bytes · {}", c::LOCAL);
            preview_panel_frame(&ctx, &mut panel, vec![], true);
            let initial = preview_panel_frame(&ctx, &mut panel, vec![], true);
            let (id, _, rect) = consistency_row(&initial, &row_label);
            preview_panel_frame(&ctx, &mut panel, vec![preview_accesskit_click(id)], true);
            if mode == 1 {
                panel.pending = Some(9);
            }
            if mode == 2 {
                panel.signals.stop.store(true, Ordering::Release);
            }
            let enabled = mode != 0;
            let pos = rect.center();
            let pointer = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![
                    egui::Event::PointerMoved(pos),
                    pointer(true),
                    preview_accesskit_click(id),
                    preview_enter(),
                ],
                enabled,
            );
            preview_panel_frame(
                &ctx,
                &mut panel,
                vec![pointer(false), preview_enter()],
                enabled,
            );
            assert_eq!(panel.selected_media.as_ref(), Some(&path));
            assert!(panel.take_preview_request().is_none());
            assert_eq!(panel.pending, (mode == 1).then_some(9));
            assert!(!fixture.0.join("data/library.sqlite").exists());
        }
    }
    #[test]
    fn actual_short_selection_summary_paint_starts_at_accessible_cell_left() {
        fn paint_positions(
            shapes: &[egui::epaint::ClippedShape],
            expected: &str,
        ) -> Vec<(egui::Pos2, egui::Rect)> {
            fn visit(
                shape: &egui::epaint::Shape,
                clip: egui::Rect,
                expected: &str,
                found: &mut Vec<(egui::Pos2, egui::Rect)>,
            ) {
                match shape {
                    egui::epaint::Shape::Text(text) if text.galley.text() == expected => {
                        found.push((text.pos, clip));
                    }
                    egui::epaint::Shape::Vec(items) => {
                        for item in items {
                            visit(item, clip, expected, found);
                        }
                    }
                    _ => {}
                }
            }
            let mut found = Vec::new();
            for shape in shapes {
                visit(&shape.shape, shape.clip_rect, expected, &mut found);
            }
            found
        }
        for physical_width in [800.0, 960.0] {
            for scale in [1.0, 1.5, 2.0] {
                for dark in [false, true] {
                    let scene = format!("width={physical_width} scale={scale} dark={dark}");
                    let entry = LibraryEntry {
                        path: PathBuf::from("synthetic").join("한글.mp4"),
                        size: 3,
                        modified_ms: Some(7),
                        file_identity: Some("context-only-source".into()),
                    };
                    let ctx = egui::Context::default();
                    crate::ui_theme::configure_dlpad_theme(&ctx);
                    ctx.set_visuals(if dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });
                    ctx.enable_accesskit();
                    ctx.set_zoom_factor(scale);
                    let name = format!("{} 한글.mp4", c::LIBRARY_SELECTED_FILE);
                    let metadata =
                        format!("3 bytes · {} 1970-01-01 00:00 UTC", c::LIBRARY_MODIFIED);
                    // Wait for actual painted galleys and exact readonly nodes,
                    // rather than assuming a particular initialization frame.
                    let mut observed = None;
                    for _ in 0..8 {
                        let time = ctx.input(|i| i.time) + 0.01;
                        let output = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(physical_width / scale, 200.0 / scale),
                                )),
                                focused: true,
                                time: Some(time),
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    selection_summary(ui, Some(&entry));
                                });
                            },
                        );
                        let name_paint = paint_positions(&output.shapes, &name);
                        let metadata_paint = paint_positions(&output.shapes, &metadata);
                        let exact_nodes = output
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .is_some_and(|tree| {
                                [&name, &metadata].iter().all(|value| {
                                    tree.nodes.iter().any(|(_, node)| {
                                        node.role() == egui::accesskit::Role::Label
                                            && node.value() == Some(value.as_str())
                                    })
                                })
                            });
                        if name_paint.len() == 1 && metadata_paint.len() == 1 && exact_nodes {
                            observed = Some((output, name_paint[0], metadata_paint[0]));
                            break;
                        }
                    }
                    let (output, (name_pos, name_clip), (metadata_pos, metadata_clip)) =
                        observed.expect("actual product summary must paint both exact text galleys and readonly nodes");
                    let (_, _, name_rect) =
                        summary_diagnostic_text(&output, &name, &ctx, &scene, "left-aligned-name");
                    let (_, _, metadata_rect) = summary_diagnostic_text(
                        &output,
                        &metadata,
                        &ctx,
                        &scene,
                        "left-aligned-metadata",
                    );
                    assert!(
                        (name_pos.x - name_rect.left()).abs() < 0.1,
                        "short filename paint origin must align with its actual accessible cell left: {scene} paint={name_pos:?} node={name_rect:?}"
                    );
                    assert!(
                        (metadata_pos.x - metadata_rect.left()).abs() < 0.1,
                        "metadata paint origin must align with its actual accessible cell left: {scene} paint={metadata_pos:?} node={metadata_rect:?}"
                    );
                    assert!(
                        name_rect.width() > metadata_rect.width(),
                        "filename must remain the leading width allocation even in the narrow scaled scene: {scene} name={name_rect:?} metadata={metadata_rect:?}"
                    );
                    assert!(
                        name_clip.contains(name_pos) && metadata_clip.contains(metadata_pos),
                        "measured origins must be within actual paint clips: {scene}"
                    );
                    assert_eq!(name_rect.height(), crate::ui_tokens::CONTROL_HEIGHT);
                    assert_eq!(metadata_rect.height(), crate::ui_tokens::CONTROL_HEIGHT);
                    assert!(name_rect.right() <= metadata_rect.left());
                }
            }
        }
    }
}
