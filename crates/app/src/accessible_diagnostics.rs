//! Session-only sanitized diagnostics; local attachment/export I/O is worker-owned.
use crate::{accessible_preview::PreviewPages, catalog as c};
use dlpad_core::diagnostics::{DiagnosticReport, MAX_REPORT_BYTES, MpdNode, MpdNodeKind};
use dlpad_engine::reports::{self, ReportFormat};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};
const MAX_INPUT_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FeedbackScope {
    General,
    Export,
    Mpd,
    LogIndex,
    LogEdit,
}

enum Operation {
    Attach(PathBuf),
    Save(PathBuf, ReportFormat),
}
struct Command {
    report: Box<DiagnosticReport>,
    operation: Operation,
}
struct Response {
    report: Box<DiagnosticReport>,
    outcome: Result<bool, ()>,
}
struct Signals {
    stop: AtomicBool,
    stopped: AtomicBool,
    failed: AtomicBool,
    context: Mutex<Option<egui::Context>>,
    #[cfg(all(windows, feature = "report-closewait-fixture"))]
    barrier: Option<Arc<crate::report_closewait_fixture::ReportSaveBarrier>>,
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
        #[cfg(all(windows, feature = "report-closewait-fixture"))]
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

pub(crate) struct DiagnosticPanel {
    report: Option<Box<DiagnosticReport>>,
    preview: String,
    pages: PreviewPages,
    format: ReportFormat,
    preview_ok: bool,
    log_index: String,
    edited_log: String,
    edit_expanded: bool,
    open_context: Option<(u64, egui::Rect)>,
    mpd_path: String,
    export_path: String,
    status: Option<&'static str>,
    feedback_scope: FeedbackScope,
    commands: SyncSender<Command>,
    results: Receiver<Response>,
    signals: Arc<Signals>,
    pending: bool,
    path_picker: crate::diagnostic_path_picker::PathPicker,
}
impl DiagnosticPanel {
    #[cfg(all(test, windows, feature = "cpu"))]
    pub(crate) fn queued_picker_for_test(path: PathBuf) -> Self {
        let mut panel = Self::new(Box::new(reports::demo_report()));
        panel.export_path = "original draft".into();
        panel.path_picker = crate::diagnostic_path_picker::PathPicker::completed_for_test(
            crate::diagnostic_path_picker::Target::ReportJson,
            path,
        );
        panel
    }
    #[cfg(all(test, windows, feature = "cpu"))]
    pub(crate) fn export_draft_for_test(&self) -> &str {
        &self.export_path
    }
    pub(crate) fn new(report: Box<DiagnosticReport>) -> Self {
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
            #[cfg(all(windows, feature = "report-closewait-fixture"))]
            barrier: crate::report_closewait_fixture::configured(),
        });
        let worker_signals = signals.clone();
        let started = thread::Builder::new()
            .name("dlpad-diagnostics".into())
            .spawn(move || {
                worker_entry(worker_signals.clone(), || worker(rx, tx, &worker_signals));
            });
        let mut panel = Self {
            report: Some(report),
            preview: String::new(),
            pages: PreviewPages::default(),
            format: ReportFormat::Markdown,
            preview_ok: false,
            log_index: "0".into(),
            edited_log: String::new(),
            edit_expanded: false,
            open_context: None,
            mpd_path: String::new(),
            export_path: String::new(),
            status: None,
            feedback_scope: FeedbackScope::General,
            commands,
            results,
            signals,
            pending: false,
            path_picker: crate::diagnostic_path_picker::PathPicker::default(),
        };
        panel.refresh_preview();
        if started.is_err() {
            panel.status = Some(c::DIAGNOSTIC_FAILED);
            panel.signals.stopped.store(true, Ordering::Release);
        }
        panel
    }
    pub(crate) fn busy(&self) -> bool {
        self.pending || self.path_picker.busy()
    }
    #[cfg(all(windows, feature = "cpu"))]
    pub(crate) fn set_picker_owner(&mut self, owner: std::sync::Arc<winit::window::Window>) {
        self.path_picker.set_owner(owner);
    }
    pub(crate) fn picker_busy(&self) -> bool {
        self.path_picker.busy()
    }
    pub(crate) fn cancel_picker(&self) {
        self.path_picker.cancel();
    }
    pub(crate) fn request_stop(&mut self) {
        self.path_picker.cancel();
        self.signals.stop.store(true, Ordering::Release);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire) && !self.path_picker.busy()
    }
    fn refresh_preview(&mut self) {
        let Some(report) = &self.report else {
            return;
        };
        match preview(report, self.format) {
            Ok(text) => {
                self.preview = text;
                self.preview_ok = self.pages.reset(&self.preview);
            }
            Err(()) => {
                self.preview.clear();
                self.pages.reset("");
                self.preview_ok = false;
                self.feedback_scope = FeedbackScope::General;
                self.status = Some(c::PREVIEW_TOO_LARGE);
            }
        }
    }
    fn request(&mut self, operation: Operation) {
        if self.pending || self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        let Some(report) = self.report.take() else {
            return;
        };
        self.feedback_scope = match &operation {
            Operation::Save(..) => FeedbackScope::Export,
            Operation::Attach(..) => FeedbackScope::Mpd,
        };
        match self.commands.try_send(Command { report, operation }) {
            Ok(()) => {
                self.pending = true;
                self.status = None;
            }
            Err(TrySendError::Full(command) | TrySendError::Disconnected(command)) => {
                self.report = Some(command.report);
                self.status = Some(c::DIAGNOSTIC_FAILED);
            }
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        if let Some((target, result)) = self.path_picker.poll(ctx)
            && !self.signals.stop.load(Ordering::Acquire)
        {
            self.feedback_scope = if target == crate::diagnostic_path_picker::Target::Mpd {
                FeedbackScope::Mpd
            } else {
                FeedbackScope::Export
            };
            match result {
                Ok(Some(path)) => {
                    let value = path.to_str().expect("picker validates UTF-8").to_owned();
                    match target {
                        crate::diagnostic_path_picker::Target::Mpd => self.mpd_path = value,
                        _ => self.export_path = value,
                    }
                    self.status = None;
                }
                Ok(None) => {}
                Err(()) => self.status = Some(c::DIAGNOSTIC_PICKER_FAILED),
            }
        }
        *self
            .signals
            .context
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(ctx.clone());
        if self.signals.failed.load(Ordering::Acquire) {
            self.feedback_scope = FeedbackScope::General;
            self.pending = false;
            self.status = Some(c::DIAGNOSTIC_FAILED);
            return;
        }
        if let Ok(response) = self.results.try_recv() {
            self.report = Some(response.report);
            self.pending = false;
            self.status = match response.outcome {
                Ok(true) => Some(c::DIAGNOSTIC_SAVED),
                Ok(false) => None,
                Err(()) => Some(c::DIAGNOSTIC_FAILED),
            };
            self.refresh_preview();
        }
        if self.pending || (self.signals.stop.load(Ordering::Acquire) && !self.stopped()) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn show_feedback(&self, ui: &mut egui::Ui, scope: FeedbackScope) {
        if self.feedback_scope == scope
            && let Some(status) = self.status
        {
            ui.label(status);
        }
    }
    fn selected_log(&self) -> Option<usize> {
        let index = self.log_index.parse::<usize>().ok()?;
        (index < self.report.as_ref()?.logs().len()).then_some(index)
    }
    fn omit_log(&mut self) {
        self.status = None;
        self.feedback_scope = FeedbackScope::LogIndex;
        if let Some(index) = self.selected_log() {
            self.report.as_mut().unwrap().omit_log(index);
            self.refresh_preview();
        } else {
            self.status = Some(c::INVALID_LOG_INDEX);
        }
    }
    fn apply_edit(&mut self) {
        self.status = None;
        self.feedback_scope = FeedbackScope::LogEdit;
        if self.edited_log.len() > MAX_INPUT_BYTES {
            self.status = Some(c::DIAGNOSTIC_INPUT_TOO_LARGE);
            return;
        }
        if let Some(index) = self.selected_log() {
            self.report
                .as_mut()
                .unwrap()
                .edit_log(index, &self.edited_log);
            self.refresh_preview();
        } else {
            self.status = Some(c::INVALID_LOG_INDEX);
        }
    }
    /// Returns a target requiring ancestor visibility after nested scroll areas consume it.
    pub(crate) fn open_context_target(&self, ctx: &egui::Context) -> Option<egui::Rect> {
        self.open_context
            .filter(|(frame, _)| *frame == ctx.cumulative_frame_nr())
            .map(|(_, rect)| rect)
    }
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        can_mutate: bool,
    ) -> Option<egui::Rect> {
        let mut ancestor_target = None;
        // Capture exact requests before widgets consume them. No IDs are guessed from labels.
        let mut scroll_targets = [None; 32];
        ui.input(|input| {
            for (slot, target) in scroll_targets
                .iter_mut()
                .zip(input.events.iter().filter_map(|event| {
                    if let egui::Event::AccessKitActionRequest(request) = event
                        && matches!(
                            request.action,
                            egui::accesskit::Action::ScrollIntoView
                                | egui::accesskit::Action::Focus
                        )
                    {
                        Some(request.target)
                    } else {
                        None
                    }
                }))
            {
                *slot = Some(target);
            }
        });
        let review_label = ui.label(c::REPORT_REVIEW);
        let mut toolbar_rect = review_label.rect;
        if self.path_picker.busy() {
            ui.label(c::DIAGNOSTIC_PICKER_BUSY);
        }
        let active = !self.pending
            && !self.path_picker.busy()
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped()
            && self.report.is_some();
        let mutate = active && can_mutate;
        ui.add_enabled_ui(active, |ui| {
            let old = self.format;
            let toolbar = ui.horizontal_wrapped(|ui| {
                let markdown = crate::ui_components::select(
                    ui,
                    &mut self.format,
                    ReportFormat::Markdown,
                    c::MARKDOWN,
                );
                reveal_control(&markdown, &scroll_targets, &mut ancestor_target);
                let json =
                    crate::ui_components::select(ui, &mut self.format, ReportFormat::Json, c::JSON);
                reveal_control(&json, &scroll_targets, &mut ancestor_target);
                if self.format != old {
                    self.refresh_preview();
                }
                let copy = crate::ui_components::ghost(ui, c::COPY_REPORT, self.preview_ok);
                reveal_control(&copy, &scroll_targets, &mut ancestor_target);
                if copy.clicked() {
                    ctx.copy_text(self.preview.clone());
                }
            });
            toolbar_rect = toolbar_rect.union(toolbar.response.rect);
        });
        self.open_context = Some((ctx.cumulative_frame_nr(), toolbar_rect));
        ui.label(c::REPORT_PREVIEW_HELP);
        ui.horizontal_wrapped(|ui| {
            let previous =
                crate::ui_components::ghost(ui, c::REPORT_PREVIEW_PREVIOUS, self.pages.index() > 0);
            reveal_control(&previous, &scroll_targets, &mut ancestor_target);
            if previous.clicked() {
                self.pages.select(self.pages.index() - 1);
            }
            let label = ui.label(c::REPORT_PREVIEW_PAGE);
            let mut page = self.pages.index() + 1;
            let page_field = ui
                .add(egui::DragValue::new(&mut page).range(1..=self.pages.count()))
                .labelled_by(label.id);
            reveal_control(&page_field, &scroll_targets, &mut ancestor_target);
            self.pages.select(page.saturating_sub(1));
            ui.label(format!("/ {}", self.pages.count()));
            let next = crate::ui_components::ghost(
                ui,
                c::REPORT_PREVIEW_NEXT,
                self.pages.index() + 1 < self.pages.count(),
            );
            reveal_control(&next, &scroll_targets, &mut ancestor_target);
            if next.clicked() {
                self.pages.select(self.pages.index() + 1);
            }
        });
        let preview_label = ui.label(c::REPORT_PREVIEW_CONTENT);
        let preview_id = ui.make_persistent_id(("diagnostic-preview-content", self.pages.index()));
        let preview_scroll_requested = ui.input(|input| {
            input.has_accesskit_action_request(preview_id, egui::accesskit::Action::ScrollIntoView)
        });
        #[cfg(test)]
        let preview_parent_clip = ui.clip_rect();
        let preview_font = egui::FontSelection::Default.resolve(ui.style());
        let preview_row_height = ui.fonts_mut(|fonts| fonts.row_height(&preview_font));
        // A large font excerpt still needs room for the parent reveal spacing.
        // The parent body owns scrolling past the other diagnostic controls.
        let preview_min_height = (8.0 * preview_row_height + 2.0 * crate::ui_tokens::LABEL_GAP)
            .min((ui.clip_rect().height() - 2.0 * ui.spacing().item_spacing.y).max(0.0));
        let preview = egui::ScrollArea::vertical()
            .id_salt(("diagnostic-preview", self.pages.index()))
            // Parent body scrolling may leave almost no available height here.
            // Reserve a readable excerpt; the outer body still exposes following controls.
            .min_scrolled_height(preview_min_height)
            .max_height(320.0)
            .show(ui, |ui| {
                // Borrow only one UTF-8 page: copy/export still use the complete report.
                let mut page = self.pages.current(&self.preview);
                let width = (ui.available_width()
                    - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                    .max(0.0);
                let response = ui
                    .add(
                        egui::TextEdit::multiline(&mut page)
                            .id(preview_id)
                            .interactive(false)
                            .desired_width(width)
                            .margin(egui::Margin::symmetric(
                                crate::ui_tokens::CONTROL_PADDING_X as i8,
                                crate::ui_tokens::LABEL_GAP as i8,
                            ))
                            .desired_rows(12),
                    )
                    .labelled_by(preview_label.id);
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_read_only();
                    node.add_action(egui::accesskit::Action::ScrollIntoView);
                });
            });
        #[cfg(test)]
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("s10-preview-actual-geometry"),
                (preview.inner_rect, preview_parent_clip, preview_row_height),
            )
        });
        if preview_scroll_requested {
            // The full page can exceed its viewport. Reveal the bounded preview area in its parent.
            ui.scroll_to_rect(preview.inner_rect, Some(egui::Align::Center));
            ancestor_target = Some(preview.inner_rect);
        }
        ui.add_enabled_ui(mutate, |ui| {
            let export_label = ui.label(c::EXPORT_PATH);
            let export_id = ui.make_persistent_id("diagnostic-export-path");
            let (export_response, choose) = diagnostic_path_field(
                ui,
                &mut self.export_path,
                export_id,
                export_label.id,
                self.path_picker.available(),
                c::DIAGNOSTIC_BROWSE_EXPORT,
            );
            reveal_control(&export_response, &scroll_targets, &mut ancestor_target);
            reveal_control(&choose, &scroll_targets, &mut ancestor_target);
            if choose.clicked() {
                self.status = None;
                self.feedback_scope = FeedbackScope::Export;
                let target = match self.format {
                    ReportFormat::Markdown => crate::diagnostic_path_picker::Target::ReportMarkdown,
                    ReportFormat::Json => crate::diagnostic_path_picker::Target::ReportJson,
                };
                if self
                    .path_picker
                    .start(target, PathBuf::from(&self.export_path), ctx)
                    .is_err()
                {
                    self.status = Some(c::DIAGNOSTIC_PICKER_FAILED);
                }
            }
            let save = crate::ui_components::secondary(
                ui,
                c::SAVE_REPORT,
                self.preview_ok && valid_path(&self.export_path) && !self.path_picker.busy(),
            );
            reveal_control(&save, &scroll_targets, &mut ancestor_target);
            if save.clicked() {
                self.request(Operation::Save(
                    PathBuf::from(&self.export_path),
                    self.format,
                ));
            }
        });
        self.show_feedback(ui, FeedbackScope::Export);
        // Save transfers the report: every later mutation must recheck its presence.
        let mutate = mutate && self.report.is_some() && !self.path_picker.busy();
        ui.add_enabled_ui(mutate, |ui| {
            let index_label = ui.label(c::LOG_INDEX);
            let index_id = ui.make_persistent_id("diagnostic-log-index");
            let index_width = (ui.available_width()
                - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                .max(0.0);
            let index = crate::ui_components::text_input_with_id(
                ui,
                &mut self.log_index,
                20,
                index_width,
                index_id,
            )
            .labelled_by(index_label.id);
            reveal_control(&index, &scroll_targets, &mut ancestor_target);
            // Clear only the attempted log-index error once real editing makes the index valid.
            // Untouched invalid drafts and unrelated export/editor failures keep their policy.
            if index.changed()
                && self.selected_log().is_some()
                && self.status == Some(c::INVALID_LOG_INDEX)
                && matches!(
                    self.feedback_scope,
                    FeedbackScope::LogIndex | FeedbackScope::LogEdit
                )
            {
                self.status = None;
            }
            ui.horizontal_wrapped(|ui| {
                let edit = crate::ui_components::secondary(ui, c::EDIT_LOG, true);
                reveal_control(&edit, &scroll_targets, &mut ancestor_target);
                if edit.clicked() {
                    self.status = None;
                    self.feedback_scope = FeedbackScope::LogIndex;
                    if let Some(index) = self.selected_log() {
                        self.edited_log = self.report.as_ref().unwrap().logs()[index].clone();
                    } else {
                        self.status = Some(c::INVALID_LOG_INDEX);
                    }
                }
                let omit = crate::ui_components::secondary(ui, c::OMIT_LOG, true);
                reveal_control(&omit, &scroll_targets, &mut ancestor_target);
                if omit.clicked() {
                    self.omit_log();
                }
                let remove = crate::ui_components::danger(ui, c::REMOVE_LOGS, true);
                reveal_control(&remove, &scroll_targets, &mut ancestor_target);
                if remove.clicked() {
                    self.status = None;
                    self.feedback_scope = FeedbackScope::LogIndex;
                    let report = self.report.as_mut().unwrap();
                    while report.omit_log(0) {}
                    self.refresh_preview();
                }
            });
            self.show_feedback(ui, FeedbackScope::LogIndex);
            let edit_label = ui.label(c::EDIT_LOG);
            let edit_id = ui.make_persistent_id("diagnostic-edit-log");
            let edit_requested = scroll_targets.contains(&Some(edit_id.value().into()));
            // Generic egui ScrollIntoView centers the full TextEdit document in
            // the inner scroll area. Reveal our bounded parent instead, retaining
            // the inner offset that TextEdit chose for the actual primary cursor.
            ui.input_mut(|input| {
                input.consume_accesskit_action_requests(edit_id, |request| {
                    request.action == egui::accesskit::Action::ScrollIntoView
                });
            });
            // Retain expansion for this panel so empty blur cannot shift the form.
            self.edit_expanded |= !self.edited_log.is_empty()
                || edit_requested
                || ui.memory(|memory| memory.has_focus(edit_id));
            let editor_background = ui.painter().add(egui::Shape::Noop);
            let editor = egui::ScrollArea::vertical()
                .id_salt("diagnostic-edit-log-scroll")
                .min_scrolled_height(if self.edit_expanded { 120.0 } else { 64.0 })
                .max_height(if self.edit_expanded { 200.0 } else { 64.0 })
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = ui.available_width();
                    let previous_cursor = egui::text_edit::TextEditState::load(ui.ctx(), edit_id)
                        .and_then(|state| state.cursor.char_range());
                    let output = egui::TextEdit::multiline(&mut self.edited_log)
                        .frame(false)
                        .id(edit_id)
                        .char_limit(MAX_INPUT_BYTES / 4)
                        .desired_width(
                            (width - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN).max(0.0),
                        )
                        .desired_rows(if self.edit_expanded { 6 } else { 2 })
                        .margin(egui::Margin::symmetric(
                            crate::ui_tokens::CONTROL_PADDING_X as i8,
                            crate::ui_tokens::LABEL_GAP as i8,
                        ))
                        .show(ui);
                    let cursor_moved = previous_cursor != output.cursor_range;
                    let response = output.response.labelled_by(edit_label.id);
                    let active_cursor =
                        response.has_focus() && (response.changed() || cursor_moved);
                    if response.enabled() {
                        response.ctx.accesskit_node_builder(response.id, |node| {
                            node.add_action(egui::accesskit::Action::ScrollIntoView);
                        });
                    }
                    // TextEdit scrolls its actual primary cursor on edits/movement.
                    // This inner ScrollArea consumes that target without growing the form.
                    (response, active_cursor)
                });
            // Test-only, immediately after actual `let editor = ... .show(...)`:
            #[cfg(test)]
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("actual-diagnostic-editor-viewport"),
                    editor.inner_rect,
                );
            });

            // Paint one stationary viewport frame behind the editable document.
            // Document/accessibility coordinates and cursor scrolling stay unchanged.
            let visuals = ui.style().interact(&editor.inner.0);
            let stroke = if editor.inner.0.enabled() && editor.inner.0.has_focus() {
                egui::Stroke::new(
                    1.5,
                    if ui.visuals().dark_mode {
                        crate::ui_theme::ACCENT
                    } else {
                        crate::ui_theme::ACCENT_SOLID
                    },
                )
            } else {
                visuals.bg_stroke
            };
            ui.painter().set(
                editor_background,
                egui::Shape::rect_filled(
                    editor.inner_rect,
                    visuals.corner_radius,
                    ui.visuals().text_edit_bg_color(),
                ),
            );
            ui.painter().rect_stroke(
                editor.inner_rect,
                visuals.corner_radius,
                stroke,
                egui::StrokeKind::Inside,
            );
            if editor.inner.0.enabled()
                && (editor.inner.0.gained_focus() || edit_requested || editor.inner.1)
            {
                ui.scroll_to_rect(editor.inner_rect, Some(egui::Align::Center));
                ancestor_target = Some(editor.inner_rect);
            }
            let apply = crate::ui_components::secondary(ui, c::APPLY_EDIT, true);
            reveal_control(&apply, &scroll_targets, &mut ancestor_target);
            if apply.clicked() {
                self.apply_edit();
            }
            self.show_feedback(ui, FeedbackScope::LogEdit);
            // Actions above can reject the selected index in either feedback scope.
            // Attach that existing error to the actual input in the same rendered frame.
            let index_error = self.status == Some(c::INVALID_LOG_INDEX)
                && matches!(
                    self.feedback_scope,
                    FeedbackScope::LogIndex | FeedbackScope::LogEdit
                )
                && self.selected_log().is_none();
            ui.ctx().accesskit_node_builder(index.id, |node| {
                if index_error {
                    node.set_invalid(egui::accesskit::Invalid::True);
                    node.set_description(c::INVALID_LOG_INDEX);
                } else {
                    node.clear_invalid();
                    node.clear_description();
                }
            });
        });
        // MPD attachment is optional expert detail; no report operation is started on open.
        let mutate = mutate && self.report.is_some() && !self.path_picker.busy();
        ui.add_enabled_ui(mutate, |ui| {
            let mpd_id = ui.make_persistent_id("diagnostic-mpd-path");
            let section = crate::accessible::collapsing(ui, c::DIAGNOSTIC_MPD_DETAILS, |ui| {
                let attach_label = ui.label(c::MPD_PATH);
                let (mpd_path, choose) = diagnostic_path_field(
                    ui,
                    &mut self.mpd_path,
                    mpd_id,
                    attach_label.id,
                    self.path_picker.available(),
                    c::DIAGNOSTIC_BROWSE_MPD,
                );
                reveal_control(&mpd_path, &scroll_targets, &mut ancestor_target);
                reveal_control(&choose, &scroll_targets, &mut ancestor_target);
                if choose.clicked()
                    && self
                        .path_picker
                        .start(
                            crate::diagnostic_path_picker::Target::Mpd,
                            PathBuf::from(&self.mpd_path),
                            ctx,
                        )
                        .is_err()
                {
                    self.feedback_scope = FeedbackScope::Mpd;
                    self.status = Some(c::DIAGNOSTIC_PICKER_FAILED);
                }
                let attach = crate::ui_components::secondary(
                    ui,
                    c::ATTACH_MPD,
                    valid_path(&self.mpd_path) && !self.path_picker.busy(),
                );
                reveal_control(&attach, &scroll_targets, &mut ancestor_target);
                if attach.clicked() {
                    self.request(Operation::Attach(PathBuf::from(&self.mpd_path)));
                }
                // Attach transfers the report; remaining actions cannot use a stale snapshot.
                if self.report.is_some() {
                    let mut omitted = None;
                    if let Some(report) = &self.report {
                        for (index, attachment) in report.mpd_attachments().iter().enumerate() {
                            let section =
                                crate::accessible::collapsing(ui, format!("MPD {index}"), |ui| {
                                    for node in &attachment.nodes {
                                        ui.label(mpd_node_label(node));
                                    }
                                    let omit_mpd =
                                        crate::ui_components::secondary(ui, c::OMIT_MPD, true);
                                    reveal_control(
                                        &omit_mpd,
                                        &scroll_targets,
                                        &mut ancestor_target,
                                    );
                                    if omit_mpd.clicked() {
                                        omitted = Some(index);
                                    }
                                });
                            reveal_control(
                                &section.header_response,
                                &scroll_targets,
                                &mut ancestor_target,
                            );
                        }
                    }
                    if let Some(index) = omitted {
                        self.report.as_mut().unwrap().omit_mpd_attachment(index);
                        self.refresh_preview();
                    }
                    let omit_all_mpd = crate::ui_components::danger(ui, c::OMIT_ALL_MPD, true);
                    reveal_control(&omit_all_mpd, &scroll_targets, &mut ancestor_target);
                    if omit_all_mpd.clicked() {
                        let report = self.report.as_mut().unwrap();
                        while report.omit_mpd_attachment(0) {}
                        self.refresh_preview();
                    }
                }
                self.show_feedback(ui, FeedbackScope::Mpd);
            });
            reveal_control(
                &section.header_response,
                &scroll_targets,
                &mut ancestor_target,
            );
        });
        self.show_feedback(ui, FeedbackScope::General);
        ancestor_target
    }
}
impl Drop for DiagnosticPanel {
    fn drop(&mut self) {
        self.path_picker.cancel();
        self.signals.stop.store(true, Ordering::Release);
    }
}
fn mpd_node_label(node: &MpdNode) -> String {
    let kind = match node.kind {
        MpdNodeKind::BaseUrl => c::MPD_BASE_URL_NODE,
        MpdNodeKind::SegmentList => c::MPD_SEGMENT_LIST_NODE,
        MpdNodeKind::Initialization => c::MPD_INITIALIZATION_NODE,
        MpdNodeKind::SegmentUrl => c::MPD_SEGMENT_URL_NODE,
    };
    let mut label = String::from(kind);
    for (name, value) in [
        (c::MPD_BYTE_RANGE, node.range.as_deref()),
        (c::MPD_MEDIA_BYTE_RANGE, node.media_range.as_deref()),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            label.push_str(&format!(" · {name}: {value}"));
        }
    }
    label
}

fn diagnostic_path_field(
    ui: &mut egui::Ui,
    value: &mut String,
    id: egui::Id,
    label: egui::Id,
    available: bool,
    caption: &str,
) -> (egui::Response, egui::Response) {
    let inline = ui.available_width() >= 360.0;
    let controls = |ui: &mut egui::Ui| {
        let reserved = if inline {
            112.0 + ui.spacing().item_spacing.x
        } else {
            0.0
        };
        let width =
            (ui.available_width() - reserved - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                .max(0.0);
        let field =
            crate::ui_components::text_input_with_id(ui, value, MAX_INPUT_BYTES / 4, width, id)
                .labelled_by(label)
                .on_hover_text(value.as_str());
        let button = crate::ui_components::secondary_sized(
            ui,
            caption,
            available,
            egui::vec2(112.0, crate::ui_tokens::CONTROL_HEIGHT),
        );
        let button = if !available {
            button.on_disabled_hover_text(c::DIAGNOSTIC_PICKER_UNAVAILABLE)
        } else {
            button
        };
        (field, button)
    };
    let result = if inline {
        ui.horizontal(controls).inner
    } else {
        ui.vertical(controls).inner
    };
    if !available {
        ui.label(c::DIAGNOSTIC_PICKER_UNAVAILABLE);
    }
    result
}

/// A native exact-node request or newly acquired focus reveals the actual response in both
/// its containing ScrollArea and the caller's ancestor. Disabled controls cannot request work.
fn reveal_control(
    response: &egui::Response,
    scroll_targets: &[Option<egui::accesskit::NodeId>; 32],
    ancestor_target: &mut Option<egui::Rect>,
) {
    if response.enabled() {
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.add_action(egui::accesskit::Action::ScrollIntoView);
        });
    }
    let requested = scroll_targets.contains(&Some(response.id.value().into()));
    if response.enabled() && (response.gained_focus() || requested) {
        response.scroll_to_me(Some(egui::Align::Center));
        *ancestor_target = Some(response.rect);
    }
}
fn valid_path(input: &str) -> bool {
    !input.is_empty() && input.len() <= MAX_INPUT_BYTES && PathBuf::from(input).is_absolute()
}
fn preview(report: &DiagnosticReport, format: ReportFormat) -> Result<String, ()> {
    let text = match format {
        ReportFormat::Markdown => report.markdown_preview(),
        ReportFormat::Json => report.json_preview().map_err(|_| ())?,
    };
    if text.len() > MAX_REPORT_BYTES {
        Err(())
    } else {
        Ok(text)
    }
}
fn execute(
    report: &mut DiagnosticReport,
    operation: Operation,
    _signals: &Signals,
) -> Result<bool, ()> {
    match operation {
        Operation::Attach(path) => reports::attach_local_mpd(report, &path)
            .map(|()| false)
            .map_err(|_| ()),
        Operation::Save(path, format) => {
            #[cfg(all(windows, feature = "report-closewait-fixture"))]
            if let Some(barrier) = &_signals.barrier {
                return reports::save_report_after_create(report, &path, format, |file| {
                    barrier.after_create(file)
                })
                .map(|()| true)
                .map_err(|_| ());
            }
            reports::save_report(report, &path, format)
                .map(|()| true)
                .map_err(|_| ())
        }
    }
}
fn worker(rx: Receiver<Command>, tx: SyncSender<Response>, signals: &Signals) {
    loop {
        if signals.stop.load(Ordering::Acquire) {
            return;
        }
        let mut command = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        };
        if signals.stop.load(Ordering::Acquire) {
            return;
        }
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute(&mut command.report, command.operation, signals)
        }))
        .unwrap_or(Err(()));
        let mut response = Response {
            report: command.report,
            outcome,
        };
        loop {
            if signals.stop.load(Ordering::Acquire) {
                return;
            }
            match tx.try_send(response) {
                Ok(()) => {
                    signals.wake();
                    break;
                }
                Err(TrySendError::Disconnected(_)) => return,
                Err(TrySendError::Full(value)) => {
                    response = value;
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
                    "dlpad-diagnostic-panel-{}-{stamp}-{sequence}",
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
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn panel() -> DiagnosticPanel {
        DiagnosticPanel::new(Box::new(reports::demo_report()))
    }
    fn wait(panel: &mut DiagnosticPanel, predicate: impl Fn(&DiagnosticPanel) -> bool) {
        let ctx = egui::Context::default();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            panel.poll(&ctx);
            if predicate(panel) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "bounded diagnostic acknowledgment timed out"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(panel: &mut DiagnosticPanel) {
        panel.request_stop();
        wait(panel, DiagnosticPanel::stopped);
    }
    fn input_frame(
        panel: &mut DiagnosticPanel,
        ctx: &egui::Context,
        enabled: bool,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, Option<egui::Rect>) {
        let mut target = None;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 2000.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    target = panel.show(ui, ctx, enabled);
                });
            },
        );
        (output, target)
    }
    fn expand_mpd_with_actual_action(panel: &mut DiagnosticPanel, ctx: &egui::Context) {
        let (first, _) = input_frame(panel, ctx, true, vec![]);
        let update = first
            .platform_output
            .accesskit_update
            .expect("actual disclosure tree");
        let headers: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::DIAGNOSTIC_MPD_DETAILS)
            })
            .collect();
        assert_eq!(headers.len(), 1, "unique actual MPD disclosure");
        let target = headers[0].0;
        input_frame(
            panel,
            ctx,
            true,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Expand,
                    target,
                    data: None,
                },
            )],
        );
        for _ in 0..16 {
            input_frame(panel, ctx, true, vec![]);
        }
        let (expanded, _) = input_frame(panel, ctx, true, vec![]);
        let nodes = expanded
            .platform_output
            .accesskit_update
            .expect("fresh expanded tree")
            .nodes;
        assert_eq!(
            nodes
                .iter()
                .filter(|(_, n)| n.role() == egui::accesskit::Role::Button
                    && n.label() == Some(c::ATTACH_MPD))
                .count(),
            1,
            "actual Expand makes attachment available"
        );
    }
    fn diagnostic_inputs(
        output: &egui::FullOutput,
    ) -> Vec<(egui::accesskit::NodeId, egui::Rect, bool)> {
        let mut fields: Vec<_> = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == egui::accesskit::Role::TextInput)
            .map(|(id, node)| {
                let b = node.bounds().unwrap();
                (
                    *id,
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    ),
                    node.is_disabled(),
                )
            })
            .collect();
        // Layout order changes; identify each semantic input by its actual label relation.
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let label_for = |id| {
            let node = &nodes.iter().find(|(node_id, _)| *node_id == id).unwrap().1;
            let labels: Vec<_> = node
                .labelled_by()
                .iter()
                .filter_map(|label_id| {
                    nodes
                        .iter()
                        .find(|(id, _)| id == label_id)
                        .and_then(|(_, label)| {
                            assert_eq!(label.role(), egui::accesskit::Role::Label);
                            label.value()
                        })
                })
                .collect();
            assert_eq!(labels.len(), 1, "one actual label per diagnostic input");
            match labels[0] {
                c::LOG_INDEX => 0,
                c::MPD_PATH => 1,
                c::EXPORT_PATH => 2,
                other => panic!("unexpected input label {other}"),
            }
        };
        fields.sort_by_key(|field| label_for(field.0));
        fields
    }
    fn diagnostic_glyphs(
        shape: &egui::epaint::Shape,
        text: &str,
        found: &mut Vec<egui::epaint::TextShape>,
    ) {
        match shape {
            egui::epaint::Shape::Text(s) if s.galley.text() == text => found.push(s.clone()),
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    diagnostic_glyphs(s, text, found);
                }
            }
            _ => {}
        }
    }
    // Actual panel/tree/action fixture, with owned worker cleanup even on assertion failure.
    struct IndexAssociationFixture {
        panel: DiagnosticPanel,
        ctx: egui::Context,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        time: f64,
        shapes: Vec<egui::epaint::ClippedShape>,
    }
    impl IndexAssociationFixture {
        fn new(dark: bool) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            let mut fixture = Self {
                panel: panel(),
                ctx,
                nodes: Default::default(),
                time: 0.0,
                shapes: vec![],
            };
            assert!(
                !fixture.panel.report.as_ref().unwrap().logs().is_empty(),
                "setup needs a real demo log"
            );
            fixture.frame(true, vec![]);
            fixture
        }
        fn frame(&mut self, enabled: bool, events: Vec<egui::Event>) {
            self.time += 1.0 / 60.0;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 2000.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        self.panel.show(ui, ctx, enabled);
                    });
                },
            );
            self.shapes = output.shapes;
            // Preserve unchanged nodes across legitimate incremental AccessKit updates.
            for (id, node) in output
                .platform_output
                .accesskit_update
                .expect("actual panel AccessKit update")
                .nodes
            {
                self.nodes.insert(id, node);
            }
        }
        fn index(&self) -> (egui::accesskit::NodeId, egui::accesskit::Node) {
            let found: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == egui::accesskit::Role::TextInput
                        && node.labelled_by().iter().any(|id| {
                            self.nodes.get(id).is_some_and(|label| {
                                label.role() == egui::accesskit::Role::Label
                                    && label.value() == Some(c::LOG_INDEX)
                            })
                        })
                })
                .collect();
            assert_eq!(
                found.len(),
                1,
                "setup requires one actual labelled diagnostic index input"
            );
            (*found[0].0, found[0].1.clone())
        }
        fn button(&self, label: &str) -> egui::accesskit::NodeId {
            let found: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
                })
                .collect();
            assert_eq!(found.len(), 1, "setup requires one actual action {label}");
            *found[0].0
        }
        fn action(target: egui::accesskit::NodeId, action: egui::accesskit::Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                target,
                action,
                data: None,
            })
        }
        fn edit_index(&mut self, value: &str) -> egui::Id {
            let id = self.index().0;
            self.frame(true, vec![Self::action(id, egui::accesskit::Action::Focus)]);
            let actual_id = self
                .ctx
                .memory(|memory| memory.focused())
                .expect("setup actual index Focus accepted");
            assert_eq!(
                actual_id.value(),
                id.0,
                "focus belongs to actual index response"
            );
            let mut state =
                egui::TextEdit::load_state(&self.ctx, actual_id).expect("actual TextEdit state");
            let length = self.panel.log_index.chars().count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(length),
                    egui::text::CCursor::new(0),
                )));
            state.store(&self.ctx, actual_id);
            let event = if value.is_empty() {
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }
            } else {
                egui::Event::Text(value.into())
            };
            self.frame(true, vec![event]);
            assert_eq!(
                self.panel.log_index, value,
                "setup actual TextEdit readback"
            );
            assert_eq!(self.index().0, id);
            actual_id
        }
        fn mark_selection(&mut self, actual_id: egui::Id) -> Option<egui::text::CCursorRange> {
            let mut state = egui::TextEdit::load_state(&self.ctx, actual_id).unwrap();
            let end = self.panel.log_index.chars().count();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(end),
                    egui::text::CCursor::new(0),
                )));
            state.store(&self.ctx, actual_id);
            self.frame(true, vec![]);
            egui::TextEdit::load_state(&self.ctx, actual_id)
                .unwrap()
                .cursor
                .char_range()
        }
        fn assert_actual_caret_clip(&self, actual_id: egui::Id) {
            let b = self.index().1.bounds().expect("actual input bounds");
            let bounds = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            assert!((bounds.height() - 32.0).abs() < 0.5);
            let cursor = egui::TextEdit::load_state(&self.ctx, actual_id)
                .unwrap()
                .cursor
                .char_range()
                .expect("actual input cursor")
                .primary;
            if self.panel.log_index.is_empty() {
                assert_eq!(cursor.index, 0, "empty input caret remains at zero");
                return;
            }
            let mut found = vec![];
            for shape in &self.shapes {
                let mut glyphs = vec![];
                diagnostic_glyphs(&shape.shape, &self.panel.log_index, &mut glyphs);
                for text in glyphs {
                    if bounds.contains(text.pos) {
                        found.push((text, shape.clip_rect));
                    }
                }
            }
            assert_eq!(
                found.len(),
                1,
                "one actual index galley inside its node bounds"
            );
            let (text, clip) = &found[0];
            let caret = text
                .galley
                .pos_from_cursor(cursor)
                .translate(text.pos.to_vec2());
            assert!(bounds.contains(caret.center_top()) && bounds.contains(caret.center_bottom()));
            assert!(
                clip.contains(caret.center_top()) && clip.contains(caret.center_bottom()),
                "actual primary cursor survives inline error reflow: {caret:?} clip={clip:?}"
            );
        }
        fn click(&mut self, label: &str, enabled: bool) {
            let target = self.button(label);
            self.frame(
                enabled,
                vec![Self::action(target, egui::accesskit::Action::Click)],
            );
            self.frame(enabled, vec![]);
        }
    }
    impl Drop for IndexAssociationFixture {
        fn drop(&mut self) {
            stop(&mut self.panel);
        }
    }

    #[test]
    fn actual_diagnostic_index_attempt_error_association_correction_preserves_selection_and_report()
    {
        for dark in [false, true] {
            for action in [c::EDIT_LOG, c::OMIT_LOG, c::APPLY_EDIT] {
                for draft in ["bad", "999999", ""] {
                    let mut f = IndexAssociationFixture::new(dark);
                    let original = f.panel.report.as_ref().unwrap().logs().to_vec();
                    let preview = f.panel.preview.clone();
                    f.panel.edited_log = "replacement fixture".into();
                    let actual_id = f.edit_index(draft);
                    let node_id = f.index().0;
                    assert_eq!(
                        f.index().1.invalid(),
                        None,
                        "untouched invalid draft must not be blanket flagged"
                    );
                    assert_eq!(f.index().1.description(), None);
                    let selection = f.mark_selection(actual_id);
                    f.click(action, true);
                    assert_eq!(
                        f.panel.status,
                        Some(c::INVALID_LOG_INDEX),
                        "setup/action must reach real rejection before association assertion"
                    );
                    assert_eq!(f.panel.report.as_ref().unwrap().logs(), original.as_slice());
                    assert_eq!(f.panel.preview, preview);
                    assert!(
                        !f.panel.pending,
                        "local rejected edit must not start report I/O"
                    );
                    assert_eq!(f.index().0, node_id);
                    assert_eq!(
                        egui::TextEdit::load_state(&f.ctx, actual_id)
                            .unwrap()
                            .cursor
                            .char_range(),
                        selection
                    );
                    assert_eq!(
                        f.index().1.invalid(),
                        Some(egui::accesskit::Invalid::True),
                        "rejected {action} must annotate actual log-index input, theme={dark} draft={draft:?}"
                    );
                    assert_eq!(f.index().1.description(), Some(c::INVALID_LOG_INDEX));
                    assert_eq!(f.index().1.value(), Some(draft));
                    f.assert_actual_caret_clip(actual_id);
                    // Correction is actual TextEdit input, not assigning the model or invoking policy helpers.
                    assert_eq!(f.edit_index("0"), actual_id);
                    assert_eq!(f.index().0, node_id);
                    assert_eq!(f.index().1.invalid(), None);
                    assert_eq!(f.index().1.description(), None);
                    assert_eq!(
                        f.panel.status, None,
                        "correction removes obsolete index error"
                    );
                    assert_eq!(f.index().1.value(), Some("0"));
                    f.assert_actual_caret_clip(actual_id);
                    assert_eq!(f.panel.report.as_ref().unwrap().logs(), original.as_slice());
                    f.click(action, true);
                    assert_eq!(f.panel.status, None);
                    match action {
                        c::EDIT_LOG => {
                            assert_eq!(f.panel.edited_log, original[0]);
                            assert_eq!(
                                f.panel.report.as_ref().unwrap().logs(),
                                original.as_slice()
                            );
                        }
                        c::OMIT_LOG => {
                            assert_eq!(f.panel.report.as_ref().unwrap().logs(), &original[1..])
                        }
                        c::APPLY_EDIT => {
                            assert_eq!(
                                f.panel.report.as_ref().unwrap().logs()[0],
                                "replacement fixture"
                            );
                            assert_eq!(
                                &f.panel.report.as_ref().unwrap().logs()[1..],
                                &original[1..]
                            );
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    #[test]
    fn actual_diagnostic_index_disabled_actions_and_typing_refuse_without_new_error_or_mutation() {
        for action in [c::EDIT_LOG, c::OMIT_LOG, c::APPLY_EDIT] {
            let mut f = IndexAssociationFixture::new(false);
            f.edit_index("bad");
            let id = f.index().0;
            let original = f.panel.report.as_ref().unwrap().logs().to_vec();
            let preview = f.panel.preview.clone();
            f.frame(false, vec![]);
            assert!(f.index().1.is_disabled(), "setup actual index disabled");
            f.frame(
                false,
                vec![
                    IndexAssociationFixture::action(id, egui::accesskit::Action::Focus),
                    egui::Event::Text("0".into()),
                ],
            );
            f.click(action, false);
            assert_eq!(f.panel.log_index, "bad");
            assert_eq!(
                f.panel.status, None,
                "disabled action is not an invalid attempt"
            );
            assert_eq!(f.index().0, id);
            assert_eq!(f.index().1.invalid(), None);
            assert_eq!(f.index().1.description(), None);
            assert_eq!(f.panel.report.as_ref().unwrap().logs(), original.as_slice());
            assert_eq!(f.panel.preview, preview);
            assert!(!f.panel.pending);
        }
    }

    #[test]
    fn actual_diagnostic_singleline_inputs_have_product_geometry_in_both_themes_and_scales() {
        for dark in [false, true] {
            for scale in [1.0, 1.5, 2.0] {
                let mut panel = panel();
                panel.log_index = "123456".into();
                panel.mpd_path = "attachment_fixture.mpd".into();
                panel.export_path = "report_fixture.json".into();
                let ctx = egui::Context::default();
                crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
                ctx.set_visuals(if dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                crate::ui_theme::configure_dlpad_theme(&ctx);
                ctx.set_pixels_per_point(scale);
                ctx.enable_accesskit();
                expand_mpd_with_actual_action(&mut panel, &ctx);
                let (output, _) = input_frame(&mut panel, &ctx, true, vec![]);
                stop(&mut panel);
                let inputs = diagnostic_inputs(&output);
                assert_eq!(
                    inputs.len(),
                    3,
                    "multiline fields retain their separate role"
                );
                for ((_, bounds, disabled), text) in
                    inputs
                        .iter()
                        .zip(["123456", "attachment_fixture.mpd", "report_fixture.json"])
                {
                    assert!(!disabled);
                    assert!((bounds.height() - 32.0).abs() < 0.5);
                    let mut glyphs = vec![];
                    for clipped in &output.shapes {
                        diagnostic_glyphs(&clipped.shape, text, &mut glyphs);
                    }
                    assert_eq!(glyphs.len(), 1);
                    assert!((glyphs[0].pos.x - bounds.left() - 12.0).abs() < 0.5);
                    assert!(
                        (glyphs[0].pos.y + glyphs[0].galley.size().y * 0.5 - bounds.center().y)
                            .abs()
                            < 0.5,
                        "theme={dark} scale={scale} text={text} glyph={:?} galley={:?} bounds={bounds:?}",
                        glyphs[0].pos,
                        glyphs[0].galley.size()
                    );
                }
            }
        }
    }
    #[test]
    fn actual_diagnostic_singleline_accesskit_focus_scroll_and_disabled_preserve_report() {
        for field in 0..3 {
            for action in [
                egui::accesskit::Action::Focus,
                egui::accesskit::Action::ScrollIntoView,
            ] {
                let mut panel = panel();
                let before = panel.preview.clone();
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                expand_mpd_with_actual_action(&mut panel, &ctx);
                let (output, _) = input_frame(&mut panel, &ctx, true, vec![]);
                let original = diagnostic_inputs(&output);
                assert_eq!(original.len(), 3);
                let event = || {
                    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                        action,
                        target: original[field].0,
                        data: None,
                    })
                };
                let (focused, target) = input_frame(&mut panel, &ctx, true, vec![event()]);
                let focus = ctx.memory(|m| m.focused());
                let (disabled, disabled_target) =
                    input_frame(&mut panel, &ctx, false, vec![event()]);
                stop(&mut panel);
                assert!(target.is_some_and(|r| r.is_finite() && r.height() > 0.0));
                if action == egui::accesskit::Action::Focus {
                    assert!(focus.is_some());
                }
                assert_eq!(diagnostic_inputs(&focused)[field].0, original[field].0);
                let inputs = diagnostic_inputs(&disabled);
                assert_eq!(inputs[field].0, original[field].0);
                assert!(inputs[field].2);
                assert!(disabled_target.is_none());
                assert_eq!(panel.preview, before);
                assert!(!panel.pending);
            }
        }
    }
    #[test]
    fn actual_diagnostic_singleline_click_edit_bounds_and_disabled_rejection() {
        for field in 0..3 {
            let mut panel = panel();
            panel.log_index.clear();
            panel.mpd_path.clear();
            panel.export_path.clear();
            let before = panel.preview.clone();
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            expand_mpd_with_actual_action(&mut panel, &ctx);
            input_frame(&mut panel, &ctx, true, vec![]);
            let (first, _) = input_frame(&mut panel, &ctx, true, vec![]);
            let inputs = diagnostic_inputs(&first);
            let position = inputs[field].1.center();
            let pointer = |pressed| egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            let _ = input_frame(
                &mut panel,
                &ctx,
                true,
                vec![egui::Event::PointerMoved(position), pointer(true)],
            );
            let _ = input_frame(&mut panel, &ctx, true, vec![pointer(false)]);
            assert!(
                ctx.memory(|m| m.focused()).is_some(),
                "actual field {field} receives pointer focus"
            );
            let limit = if field == 0 { 20 } else { MAX_INPUT_BYTES / 4 };
            let (edited, _) = input_frame(
                &mut panel,
                &ctx,
                true,
                vec![egui::Event::Text("😀".repeat(limit + 1))],
            );
            let value = match field {
                0 => &panel.log_index,
                1 => &panel.mpd_path,
                _ => &panel.export_path,
            }
            .clone();
            let (disabled, _) = input_frame(
                &mut panel,
                &ctx,
                false,
                vec![egui::Event::Text("rejected".into())],
            );
            let unchanged = (match field {
                0 => &panel.log_index,
                1 => &panel.mpd_path,
                _ => &panel.export_path,
            }) == &value;
            stop(&mut panel);
            assert_eq!(value.chars().count(), limit);
            assert!(value.len() <= MAX_INPUT_BYTES);
            assert_eq!(diagnostic_inputs(&edited)[field].0, inputs[field].0);
            assert_eq!(diagnostic_inputs(&disabled)[field].0, inputs[field].0);
            assert!(unchanged);
            assert_eq!(panel.preview, before);
            assert!(!panel.pending);
        }
    }
    #[test]
    fn actual_diagnostic_buttons_reveal_exact_scroll_and_focus_without_mutation() {
        for name in [
            c::MARKDOWN,
            c::JSON,
            c::EDIT_LOG,
            c::OMIT_LOG,
            c::REMOVE_LOGS,
            c::APPLY_EDIT,
            c::OMIT_ALL_MPD,
        ] {
            for action in [
                egui::accesskit::Action::Focus,
                egui::accesskit::Action::ScrollIntoView,
            ] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut panel = panel();
                let before = panel.preview.clone();
                if name == c::OMIT_ALL_MPD {
                    expand_mpd_with_actual_action(&mut panel, &ctx);
                }
                let mut target = None;
                let output = ctx.run(egui::RawInput::default(), |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        target = panel.show(ui, ctx, true);
                    });
                });
                assert!(target.is_none());
                let update = output.platform_output.accesskit_update.unwrap();
                let matches: Vec<_> = update
                    .nodes
                    .iter()
                    .filter(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button && node.label() == Some(name)
                    })
                    .collect();
                assert_eq!(matches.len(), 1, "actual diagnostic button {name}");
                let id = matches[0].0;
                let input = egui::RawInput {
                    events: vec![egui::Event::AccessKitActionRequest(
                        egui::accesskit::ActionRequest {
                            action,
                            target: id,
                            data: None,
                        },
                    )],
                    ..Default::default()
                };
                let _ = ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        target = panel.show(ui, ctx, true);
                    });
                });
                assert!(
                    target.is_some_and(|r| r.is_finite() && r.height() > 0.0),
                    "missing ancestor target for {name} {action:?}"
                );
                assert_eq!(
                    panel.preview, before,
                    "focus/scroll must not activate {name}"
                );
                stop(&mut panel);
            }
        }
    }
    #[test]
    fn disabled_diagnostic_control_cannot_request_ancestor_scroll() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut panel = panel();
        let mut target = None;
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                target = panel.show(ui, ctx, false);
            });
        });
        let update = output.platform_output.accesskit_update.unwrap();
        let id = update
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button && node.label() == Some(c::OMIT_LOG)
            })
            .unwrap()
            .0;
        let input = egui::RawInput {
            events: vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::ScrollIntoView,
                    target: id,
                    data: None,
                },
            )],
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                target = panel.show(ui, ctx, false);
            });
        });
        assert!(target.is_none());
        stop(&mut panel);
    }
    #[cfg(all(windows, feature = "report-closewait-fixture"))]
    #[test]
    fn actual_opened_save_blocks_stop_ack_until_original_release() {
        let fixture = Fixture::new();
        let output = fixture.0.join("blocked-report.md");
        let barrier = crate::report_closewait_fixture::ReportSaveBarrier::testing().unwrap();
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
            barrier: Some(barrier.clone()),
        });
        let (command_tx, rx) = mpsc::sync_channel(1);
        let (tx, _response_rx) = mpsc::sync_channel(1);
        let owned_signals = signals.clone();
        let thread = thread::spawn(move || {
            worker_entry(owned_signals.clone(), || worker(rx, tx, &owned_signals))
        });
        command_tx
            .send(Command {
                report: Box::new(reports::demo_report()),
                operation: Operation::Save(output.clone(), ReportFormat::Json),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !barrier.test_started() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(std::fs::metadata(&output).unwrap().len(), 0);
        signals.stop.store(true, Ordering::Release);
        // Actual worker is inside the original file callback, not a synthetic busy flag.
        thread::sleep(Duration::from_millis(50));
        assert!(!signals.stopped.load(Ordering::Acquire));
        assert!(!barrier.test_ack());
        barrier.test_release().unwrap();
        while !signals.stopped.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        thread.join().unwrap();
        assert!(barrier.test_ack());
        assert!(!signals.failed.load(Ordering::Acquire));
        let bytes = std::fs::read(&output).unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&bytes).is_ok());
        assert!(
            !String::from_utf8(bytes)
                .unwrap()
                .contains("signature=fixture")
        );
    }
    #[test]
    fn actual_editor_stationary_frame_focus_blur_disabled_stroke_in_both_themes() {
        fn border(ctx: &egui::Context, output: &egui::FullOutput) -> egui::Stroke {
            let viewport = ctx
                .data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new("actual-diagnostic-editor-viewport"))
                })
                .expect("actual Panel.show viewport");
            let strokes: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::Shape::Rect(rect) = &shape.shape
                        && rect.rect == viewport
                        && rect.stroke.width > 0.0
                    {
                        return Some(rect.stroke);
                    }
                    None
                })
                .collect();
            assert_eq!(
                strokes.len(),
                1,
                "exact stationary viewport has one outline"
            );
            strokes[0]
        }
        for dark in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            let mut panel = panel();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (initial, _) = input_frame(&mut panel, &ctx, true, vec![]);
                let editor = initial
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::MultilineTextInput
                            && node.supports_action(egui::accesskit::Action::Focus)
                    })
                    .map(|(id, _)| *id)
                    .expect("actual editable log node");
                let request = || {
                    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: editor,
                        data: None,
                    })
                };
                input_frame(&mut panel, &ctx, true, vec![request()]);
                let (focused, _) = input_frame(&mut panel, &ctx, true, vec![]);
                assert_eq!(
                    ctx.memory(|memory| memory.focused()).unwrap().value(),
                    editor.0
                );
                let focused_stroke = border(&ctx, &focused);
                assert_eq!(focused_stroke.width, 1.5);
                assert_eq!(
                    focused_stroke.color,
                    if dark {
                        crate::ui_theme::ACCENT
                    } else {
                        crate::ui_theme::ACCENT_SOLID
                    }
                );
                let focused_id = ctx.memory(|memory| memory.focused()).unwrap();
                ctx.memory_mut(|memory| memory.surrender_focus(focused_id));
                let (blurred, _) = input_frame(&mut panel, &ctx, true, vec![]);
                assert_ne!(border(&ctx, &blurred), focused_stroke);
                let before = panel.edited_log.clone();
                let (disabled, _) = input_frame(
                    &mut panel,
                    &ctx,
                    false,
                    vec![request(), egui::Event::Text("blocked".into())],
                );
                assert_eq!(panel.edited_log, before);
                assert_ne!(
                    border(&ctx, &disabled),
                    focused_stroke,
                    "disabled is not a focused accent frame"
                );
            }));
            stop(&mut panel);
            if let Err(payload) = result {
                std::panic::resume_unwind(payload);
            }
        }
    }
    #[test]
    fn readonly_preview_scroll_action_returns_ancestor_target_only_for_exact_request() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut panel = panel();
        let mut preview_id = egui::Id::NULL;
        let mut target = None;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                preview_id =
                    ui.make_persistent_id(("diagnostic-preview-content", panel.pages.index()));
                target = panel.show(ui, ctx, true);
            });
        });
        assert!(target.is_none());
        for (id, action, expected) in [
            (
                preview_id.with("unrelated"),
                egui::accesskit::Action::ScrollIntoView,
                false,
            ),
            (preview_id, egui::accesskit::Action::Focus, false),
            (preview_id, egui::accesskit::Action::ScrollIntoView, true),
        ] {
            let input = egui::RawInput {
                events: vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action,
                        target: id.value().into(),
                        data: None,
                    },
                )],
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    target = panel.show(ui, ctx, true);
                });
            });
            assert_eq!(target.is_some(), expected);
            if let Some(rect) = target {
                assert!(rect.is_finite() && rect.height() > 0.0 && rect.height() <= 320.0);
            }
        }
    }
    #[test]
    fn page_navigation_preserves_full_report_and_mutations_reset_page() {
        let mut panel = panel();
        panel.report = Some(Box::new(crate::accessible_report_fixture::stress_report()));
        panel.refresh_preview();
        let complete = panel.preview.clone();
        assert!(panel.pages.count() > 200);
        panel.pages.select(usize::MAX);
        assert_eq!(panel.preview, complete);
        assert!(panel.pages.current(&panel.preview).len() <= 8192);
        let fixture = Fixture::new();
        let output = fixture.0.join("complete.md");
        panel.request(Operation::Save(output.clone(), ReportFormat::Markdown));
        wait(&mut panel, |panel| !panel.busy());
        assert_eq!(std::fs::read_to_string(output).unwrap(), complete);
        assert_eq!(panel.preview, complete);
        assert_eq!(panel.pages.index(), 0);
        panel.pages.select(usize::MAX);
        panel.format = ReportFormat::Json;
        panel.refresh_preview();
        assert_eq!(panel.pages.index(), 0);
        panel.pages.select(usize::MAX);
        panel.edited_log = "페이지 편집".into();
        panel.apply_edit();
        assert_eq!(panel.pages.index(), 0);
        panel.pages.select(usize::MAX);
        panel.omit_log();
        assert_eq!(panel.pages.index(), 0);
        stop(&mut panel);
    }
    #[test]
    fn existing_export_is_preserved_and_report_returns_from_worker() {
        let fixture = Fixture::new();
        let existing = fixture.0.join("existing.json");
        std::fs::write(&existing, b"existing synthetic user data").unwrap();
        let mut panel = panel();
        let id = panel.report.as_ref().unwrap().report_id().to_owned();
        panel.request(Operation::Save(existing.clone(), ReportFormat::Json));
        assert!(panel.report.is_none());
        assert!(panel.busy());
        wait(&mut panel, |panel| !panel.busy());
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_FAILED));
        assert_eq!(panel.report.as_ref().unwrap().report_id(), id);
        assert_eq!(
            std::fs::read(existing).unwrap(),
            b"existing synthetic user data"
        );
        stop(&mut panel);
    }
    #[test]
    fn explicit_export_is_new_sanitized_file_without_cache_or_other_writes() {
        let fixture = Fixture::new();
        let mut panel = panel();
        let output = fixture.0.join("report.json");
        panel.request(Operation::Save(output.clone(), ReportFormat::Json));
        wait(&mut panel, |panel| !panel.busy());
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_SAVED));
        let text = std::fs::read_to_string(output).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&text).is_ok());
        assert!(!text.contains("signature=fixture"));
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
        stop(&mut panel);
    }
    #[test]
    fn mpd_attachment_is_structure_only_and_oversize_keeps_sources() {
        let fixture = Fixture::new();
        let input = fixture.0.join("input.mpd");
        let bytes = br#"<MPD><BaseURL>https://invalid.test/PRIVATE?token=PRIVATE</BaseURL><SegmentList><Initialization range="0-5"/></SegmentList></MPD>"#;
        std::fs::write(&input, bytes).unwrap();
        let mut panel = panel();
        panel.request(Operation::Attach(input.clone()));
        wait(&mut panel, |panel| !panel.busy());
        assert!(!panel.preview.contains("PRIVATE"));
        assert_eq!(std::fs::read(&input).unwrap(), bytes);
        let count = panel.report.as_ref().unwrap().mpd_attachments().len();
        std::fs::write(&input, vec![b'x'; reports::MAX_MPD_BYTES + 1]).unwrap();
        panel.request(Operation::Attach(input.clone()));
        wait(&mut panel, |panel| !panel.busy());
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_FAILED));
        assert_eq!(
            panel.report.as_ref().unwrap().mpd_attachments().len(),
            count
        );
        assert_eq!(
            std::fs::metadata(input).unwrap().len(),
            (reports::MAX_MPD_BYTES + 1) as u64
        );
        stop(&mut panel);
    }
    #[test]
    fn edited_logs_are_resanitized_and_invalid_index_preserves_report() {
        let mut panel = panel();
        panel.edited_log = "request https://media.invalid/PRIVATE?token=PRIVATE".into();
        panel.apply_edit();
        assert!(!panel.preview.contains("PRIVATE"));
        let previous = panel.preview.clone();
        panel.log_index = usize::MAX.to_string();
        panel.omit_log();
        assert_eq!(panel.status, Some(c::INVALID_LOG_INDEX));
        assert_eq!(panel.preview, previous);
        panel.log_index = "0".into();
        panel.omit_log();
        let report = panel.report.as_mut().unwrap();
        while report.omit_log(0) {}
        while report.omit_mpd_attachment(0) {}
        panel.refresh_preview();
        assert!(panel.report.as_ref().unwrap().logs().is_empty());
        assert!(panel.report.as_ref().unwrap().mpd_attachments().is_empty());
        stop(&mut panel);
    }
    #[test]
    fn preview_and_unicode_input_limits_are_enforced() {
        let mut report = reports::demo_report();
        report.residual_risk = Box::leak("x".repeat(MAX_REPORT_BYTES + 1).into_boxed_str());
        assert!(preview(&report, ReportFormat::Markdown).is_err());
        assert!(preview(&report, ReportFormat::Json).is_err());
        let mut panel = panel();
        let previous = panel.preview.clone();
        panel.edited_log = "한".repeat(MAX_INPUT_BYTES / 3 + 1);
        panel.apply_edit();
        assert_eq!(panel.preview, previous);
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_INPUT_TOO_LARGE));
        assert!(!valid_path("relative.json"));
        assert!(!valid_path(&"x".repeat(MAX_INPUT_BYTES + 1)));
        stop(&mut panel);
    }
    #[test]
    fn stop_ack_is_finite_without_draining_completed_result() {
        let fixture = Fixture::new();
        let mut panel = panel();
        panel.request(Operation::Save(
            fixture.0.join("owned.json"),
            ReportFormat::Json,
        ));
        thread::sleep(Duration::from_millis(100));
        panel.request_stop();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !panel.stopped() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        let previous = panel.preview.clone();
        panel.request(Operation::Attach(fixture.0.join("absent.mpd")));
        assert_eq!(panel.preview, previous);
    }
    #[test]
    fn worker_panic_has_fixed_error_and_stop_ack_without_report_payload() {
        let mut panel = panel();
        stop(&mut panel);
        panel.signals.stopped.store(false, Ordering::Release);
        let signals = panel.signals.clone();
        thread::spawn(move || worker_entry(signals, || panic!("synthetic worker panic")))
            .join()
            .unwrap();
        assert!(panel.stopped());
        panel.poll(&egui::Context::default());
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_FAILED));
        assert!(!panel.busy());
    }
    #[test]
    fn actual_multiline_editor_wraps_bounds_and_reveals_edited_cursor() {
        fn frame(
            panel: &mut DiagnosticPanel,
            ctx: &egui::Context,
            time: &mut f64,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            *time += 1.0 / 30.0;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 2000.0),
                    )),
                    time: Some(*time),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let parent = panel.show(ui, ctx, true);
                        ctx.data_mut(|data| {
                            data.insert_temp(egui::Id::new("observed-diagnostic-ancestor"), parent)
                        });
                    });
                },
            )
        }
        fn text_shape(
            shape: &egui::epaint::Shape,
            text: &str,
            clip: egui::Rect,
            out: &mut Vec<(egui::epaint::TextShape, egui::Rect)>,
        ) {
            match shape {
                egui::epaint::Shape::Text(s) if s.galley.text() == text => {
                    out.push((s.clone(), clip))
                }
                egui::epaint::Shape::Vec(items) => {
                    for item in items {
                        text_shape(item, text, clip, out)
                    }
                }
                _ => {}
            }
        }
        let mut panel = panel();
        panel.edited_log = (0..100)
            .map(|i| format!("row{i:03} {}\n", "x".repeat(45)))
            .collect::<String>()
            + &"wide-line".repeat(80);
        let initial = panel.edited_log.clone();
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut time = 0.0;
        let output = frame(&mut panel, &ctx, &mut time, vec![]);
        let editor = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::MultilineTextInput
                    && node.supports_action(egui::accesskit::Action::Focus)
            })
            .expect("actual editable multiline node")
            .0;
        let initial_bounds = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(id, _)| *id == editor)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(
            initial_bounds.y1 - initial_bounds.y0 > 200.0,
            "AccessKit exposes full editable document extent; it is not viewport geometry"
        );
        let focus = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: editor,
            data: None,
        });
        frame(&mut panel, &ctx, &mut time, vec![focus]);
        let actual_id = ctx.memory(|m| m.focused()).expect("actual editor focus");
        assert_eq!(actual_id.value(), editor.0);
        let command = egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::NONE
        };
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        for (phase, events) in [
            vec![key(egui::Key::End, command), egui::Event::Text("!".into())],
            vec![
                key(egui::Key::Home, command),
                key(egui::Key::Delete, egui::Modifiers::NONE),
            ],
        ]
        .into_iter()
        .enumerate()
        {
            frame(&mut panel, &ctx, &mut time, events);
            let parent = ctx
                .data(|data| {
                    data.get_temp::<Option<egui::Rect>>(egui::Id::new(
                        "observed-diagnostic-ancestor",
                    ))
                })
                .flatten()
                .expect("active cursor edit/movement returns ancestor reveal");
            assert!(
                parent.height() <= 201.0 && parent.width() > 800.0,
                "parent receives bounded viewport on edit/movement, not full document: {parent:?}"
            );

            let mut settled = None;
            for _ in 0..16 {
                settled = Some(frame(&mut panel, &ctx, &mut time, vec![]));
            }
            let output = settled.unwrap();
            let state =
                egui::text_edit::TextEditState::load(&ctx, actual_id).expect("real TextEdit state");
            let cursor = state
                .cursor
                .char_range()
                .expect("actual primary cursor")
                .primary;
            let mut shapes = vec![];
            for shape in &output.shapes {
                text_shape(
                    &shape.shape,
                    &panel.edited_log,
                    shape.clip_rect,
                    &mut shapes,
                );
            }
            assert_eq!(shapes.len(), 1, "actual editor painted galley");
            let (text, clip) = &shapes[0];
            assert!(
                clip.width() > 800.0,
                "editor consumes 900px form available width, not default288: {clip:?}"
            );
            assert!(
                clip.height() >= 110.0 && clip.height() <= 201.0,
                "bounded viewport independent of long log: {clip:?}"
            );
            assert!(
                text.galley.size().x <= clip.width() + 1.0,
                "long paragraph wraps into viewport"
            );
            let caret = text
                .galley
                .pos_from_cursor(cursor)
                .translate(text.pos.to_vec2());
            assert!(
                clip.contains(caret.center_top()) && clip.contains(caret.center_bottom()),
                "actual galley primary cursor is clipped after edit/move: caret={caret:?} clip={clip:?}"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(actual_id));
            if phase == 0 {
                let before = panel.edited_log.clone();
                frame(
                    &mut panel,
                    &ctx,
                    &mut time,
                    vec![egui::Event::AccessKitActionRequest(
                        egui::accesskit::ActionRequest {
                            action: egui::accesskit::Action::ScrollIntoView,
                            target: editor,
                            data: None,
                        },
                    )],
                );
                let parent = ctx
                    .data(|data| {
                        data.get_temp::<Option<egui::Rect>>(egui::Id::new(
                            "observed-diagnostic-ancestor",
                        ))
                    })
                    .flatten()
                    .expect("exact edit ScrollIntoView reveals parent");
                assert!(parent.height() <= 201.0 && parent.width() > 800.0);
                let mut settled = frame(&mut panel, &ctx, &mut time, vec![]);
                for _ in 0..16 {
                    settled = frame(&mut panel, &ctx, &mut time, vec![]);
                }
                let actual_cursor = egui::text_edit::TextEditState::load(&ctx, actual_id)
                    .unwrap()
                    .cursor
                    .char_range()
                    .unwrap()
                    .primary;
                assert_eq!(actual_cursor, cursor, "reveal preserves CtrlEnd caret");
                let mut painted = vec![];
                for shape in &settled.shapes {
                    text_shape(
                        &shape.shape,
                        &panel.edited_log,
                        shape.clip_rect,
                        &mut painted,
                    );
                }
                assert_eq!(painted.len(), 1);
                let (text, clip) = &painted[0];
                let caret = text
                    .galley
                    .pos_from_cursor(actual_cursor)
                    .translate(text.pos.to_vec2());
                assert!(
                    clip.contains(caret.center_top()) && clip.contains(caret.center_bottom()),
                    "ScrollIntoView must retain actual end caret in painted viewport: caret={caret:?} clip={clip:?}"
                );
                assert_eq!(panel.edited_log, before);
                assert_eq!(ctx.memory(|m| m.focused()), Some(actual_id));
            }
        }
        assert!(panel.edited_log.ends_with('!'));
        assert_eq!(
            panel.edited_log,
            initial.chars().skip(1).collect::<String>() + "!"
        );
        frame(
            &mut panel,
            &ctx,
            &mut time,
            vec![egui::Event::Paste("한".repeat(MAX_INPUT_BYTES))],
        );
        assert!(
            panel.edited_log.len() <= MAX_INPUT_BYTES,
            "existing byte bound survives real paste"
        );
        let reveal = egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::ScrollIntoView,
            target: editor,
            data: None,
        });
        let mut parent_target = None;
        time += 1.0 / 30.0;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 2000.0),
                )),
                time: Some(time),
                focused: true,
                events: vec![reveal],
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    parent_target = panel.show(ui, ctx, true);
                });
            },
        );
        let target = parent_target.expect("actual ScrollIntoView returns ancestor reveal");
        assert!(
            target.height() <= 201.0 && target.width() > 800.0,
            "ancestor reveals bounded editor viewport, not full log extent: {target:?}"
        );
        stop(&mut panel);
    }

    // Append inside accessible_diagnostics.rs tests; apply observation-hunk first.
    #[test]
    fn actual_empty_editor_density_focus_type_blur_preserves_report_and_caret() {
        let mut panel = panel();
        let before = panel.preview.clone();
        assert!(panel.edited_log.is_empty());
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut time = 0.0;
        let mut editor_id = None;
        let mut frame = |panel: &mut DiagnosticPanel, events| {
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
            ctx.run(input, |ctx| {
                egui::TopBottomPanel::top("diagnostic-test-navigation")
                    .exact_height(48.0)
                    .show(ctx, |ui| {
                        ui.label(c::DIAGNOSTICS);
                    });
                egui::TopBottomPanel::bottom("diagnostic-test-footer")
                    .exact_height(54.0)
                    .show(ctx, |ui| {
                        ui.label("owned fixed footer");
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("diagnostic-test-download-body")
                        .show(ui, |ui| {
                            ui.allocate_space(egui::vec2(1.0, 96.0));
                            let target = panel.show(ui, ctx, true);
                            if let Some(rect) = target {
                                ui.scroll_to_rect(rect, Some(egui::Align::Center));
                            }
                        });
                });
            })
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..4 {
                let out = frame(&mut panel, vec![]);
                if let Some(update) = out.platform_output.accesskit_update {
                    for (id, node) in update.nodes {
                        if node.role() == egui::accesskit::Role::MultilineTextInput
                            && node.supports_action(egui::accesskit::Action::Focus)
                        {
                            editor_id = Some(id);
                        }
                    }
                }
            }
            let editor = editor_id.expect("actual editable multiline node");
            let compact = ctx
                .data(|d| {
                    d.get_temp::<egui::Rect>(egui::Id::new("actual-diagnostic-editor-viewport"))
                })
                .unwrap();
            assert!(
                compact.height() <= 64.1,
                "unused actual inner viewport compact: {compact:?}"
            );
            frame(
                &mut panel,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: editor,
                        data: None,
                    },
                )],
            );
            let id = ctx.memory(|m| m.focused()).expect("actual focus");
            assert_eq!(id.value(), editor.0);
            let mut focused_out = frame(&mut panel, vec![]);
            for _ in 0..10 {
                focused_out = frame(&mut panel, vec![]);
            }
            let expanded = ctx
                .data(|d| {
                    d.get_temp::<egui::Rect>(egui::Id::new("actual-diagnostic-editor-viewport"))
                })
                .unwrap();
            assert!(
                expanded.height() >= 119.9,
                "empty focused editor expands actual viewport"
            );
            let footer = egui::containers::panel::PanelState::load(
                &ctx,
                egui::Id::new("diagnostic-test-footer"),
            )
            .unwrap()
            .rect;
            assert!(
                expanded.bottom() <= footer.top()
                    && focused_out
                        .shapes
                        .iter()
                        .any(|s| s.clip_rect.contains_rect(expanded)),
                "actual ancestor reveal/fullpaint before footer"
            );
            ctx.memory_mut(|memory| memory.surrender_focus(id));
            for _ in 0..5 {
                frame(&mut panel, vec![]);
            }
            let empty_blur = ctx
                .data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new("actual-diagnostic-editor-viewport"))
                })
                .unwrap();
            assert!(
                empty_blur.height() >= 119.9,
                "empty blur retains expanded form"
            );
            frame(
                &mut panel,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: editor,
                        data: None,
                    },
                )],
            );
            frame(
                &mut panel,
                vec![egui::Event::Text("합성 secret-token".into())],
            );
            assert_eq!(panel.edited_log, "합성 secret-token");
            let cursor = egui::text_edit::TextEditState::load(&ctx, id)
                .unwrap()
                .cursor
                .char_range()
                .expect("actual cursor");
            ctx.memory_mut(|m| m.surrender_focus(id));
            for _ in 0..5 {
                frame(&mut panel, vec![]);
            }
            assert_eq!(
                egui::text_edit::TextEditState::load(&ctx, id)
                    .unwrap()
                    .cursor
                    .char_range(),
                Some(cursor),
                "blur preserves caret/selection and original ID"
            );
            assert_eq!(
                panel.preview, before,
                "editing/focus must not silently apply unsanitized text to report"
            );
        }));
        stop(&mut panel);
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    #[test]
    fn actual_default_report_flow_keeps_mpd_optional_and_preserves_report() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut panel = panel();
        let before = panel.preview.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (output, _) = input_frame(&mut panel, &ctx, true, vec![]);
            let nodes = output.platform_output.accesskit_update.unwrap().nodes;
            let buttons = |name| {
                nodes
                    .iter()
                    .filter(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button && node.label() == Some(name)
                    })
                    .count()
            };
            assert_eq!(buttons(c::COPY_REPORT), 1, "basic sharing stays available");
            assert_eq!(buttons(c::SAVE_REPORT), 1, "basic saving stays available");
            assert_eq!(
                buttons(c::ATTACH_MPD),
                0,
                "advanced structure is initially collapsed"
            );
            assert_eq!(
                buttons(c::OMIT_ALL_MPD),
                0,
                "advanced removal is initially collapsed"
            );
            assert_eq!(
                panel.preview, before,
                "layout does not change report content"
            );
        }));
        stop(&mut panel);
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    #[test]
    fn actual_mpd_disclosure_preserves_ids_draft_and_rejects_stale_hidden_focus() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut panel = panel();
        fn identity(
            output: &egui::FullOutput,
            name: &str,
            role: egui::accesskit::Role,
        ) -> egui::accesskit::NodeId {
            let nodes = &output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            let matches: Vec<_> = nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == role
                        && (node.label() == Some(name)
                            || node.labelled_by().iter().any(|label_id| {
                                nodes.iter().any(|(id, label)| {
                                    id == label_id
                                        && label.role() == egui::accesskit::Role::Label
                                        && label.value() == Some(name)
                                })
                            }))
                })
                .collect();
            assert_eq!(matches.len(), 1, "unique actual field {name}");
            matches[0].0
        }
        let event = |target, action| {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                target,
                action,
                data: None,
            })
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (initial, _) = input_frame(&mut panel, &ctx, true, vec![]);
            let editor_id = identity(
                &initial,
                c::EDIT_LOG,
                egui::accesskit::Role::MultilineTextInput,
            );
            let export_id = identity(&initial, c::EXPORT_PATH, egui::accesskit::Role::TextInput);
            input_frame(
                &mut panel,
                &ctx,
                true,
                vec![event(editor_id, egui::accesskit::Action::Focus)],
            );
            input_frame(
                &mut panel,
                &ctx,
                true,
                vec![egui::Event::Text("유지할 초안".into())],
            );
            let draft = panel.edited_log.clone();
            assert!(!draft.is_empty(), "actual focused typing changes draft");
            let report = panel.preview.clone();
            expand_mpd_with_actual_action(&mut panel, &ctx);
            let (expanded, _) = input_frame(&mut panel, &ctx, true, vec![]);
            let mpd_id = identity(&expanded, c::MPD_PATH, egui::accesskit::Role::TextInput);
            let header_id = identity(
                &expanded,
                c::DIAGNOSTIC_MPD_DETAILS,
                egui::accesskit::Role::Button,
            );
            assert_eq!(
                identity(&expanded, c::EXPORT_PATH, egui::accesskit::Role::TextInput),
                export_id
            );
            assert_eq!(
                identity(
                    &expanded,
                    c::EDIT_LOG,
                    egui::accesskit::Role::MultilineTextInput
                ),
                editor_id
            );
            input_frame(
                &mut panel,
                &ctx,
                true,
                vec![event(header_id, egui::accesskit::Action::Collapse)],
            );
            for _ in 0..16 {
                input_frame(&mut panel, &ctx, true, vec![]);
            }
            let (closed, _) = input_frame(
                &mut panel,
                &ctx,
                true,
                vec![event(mpd_id, egui::accesskit::Action::Focus)],
            );
            assert_eq!(
                identity(&closed, c::EXPORT_PATH, egui::accesskit::Role::TextInput),
                export_id
            );
            assert_eq!(
                identity(
                    &closed,
                    c::EDIT_LOG,
                    egui::accesskit::Role::MultilineTextInput
                ),
                editor_id
            );
            assert!(
                !closed
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(id, _)| *id == mpd_id),
                "closed MPD has no stale input"
            );
            assert_eq!(panel.edited_log, draft);
            assert_eq!(panel.preview, report);
            assert!(!panel.pending);
            assert!(panel.report.is_some());
        }));
        stop(&mut panel);
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn diagnostic_picker_completion_changes_only_target_draft_without_export() {
        use crate::diagnostic_path_picker::{PathPicker, Target};
        for target in [Target::ReportMarkdown, Target::ReportJson, Target::Mpd] {
            let mut panel = panel();
            panel.export_path = "existing-report-draft".into();
            panel.mpd_path = "existing-mpd-draft".into();
            let before = panel.preview.clone();
            let path = PathBuf::from(r"C:\owned-synthetic\한글.json");
            panel.path_picker = PathPicker::completed_for_test(target, path.clone());
            assert!(panel.busy());
            panel.poll(&egui::Context::default());
            assert!(!panel.busy());
            assert_eq!(panel.preview, before);
            assert!(panel.report.is_some());
            assert!(!panel.pending);
            if target == Target::Mpd {
                assert_eq!(panel.mpd_path, path.to_str().unwrap());
                assert_eq!(panel.export_path, "existing-report-draft");
            } else {
                assert_eq!(panel.export_path, path.to_str().unwrap());
                assert_eq!(panel.mpd_path, "existing-mpd-draft");
            }
            stop(&mut panel);
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn diagnostic_picker_close_discards_queued_selection_and_drains_ack() {
        use crate::diagnostic_path_picker::{PathPicker, Target};
        let mut panel = panel();
        panel.export_path = "old-draft".into();
        panel.path_picker = PathPicker::completed_for_test(
            Target::ReportJson,
            PathBuf::from(r"C:\owned-synthetic\new.json"),
        );
        panel.request_stop();
        assert!(!panel.stopped(), "queued picker ACK must still be drained");
        wait(&mut panel, DiagnosticPanel::stopped);
        assert_eq!(panel.export_path, "old-draft");
        assert!(!panel.pending);
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn diagnostic_picker_oversized_result_preserves_existing_draft() {
        use crate::diagnostic_path_picker::{PathPicker, Target};
        let mut panel = panel();
        panel.export_path = "old-draft".into();
        let before = panel.preview.clone();
        panel.path_picker = PathPicker::completed_for_test(
            Target::ReportJson,
            PathBuf::from("x".repeat(MAX_INPUT_BYTES + 1)),
        );
        panel.poll(&egui::Context::default());
        assert_eq!(panel.export_path, "old-draft");
        assert_eq!(panel.preview, before);
        assert_eq!(panel.status, Some(c::DIAGNOSTIC_PICKER_FAILED));
        assert!(!panel.busy());
        stop(&mut panel);
    }
}
