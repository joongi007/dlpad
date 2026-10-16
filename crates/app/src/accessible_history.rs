//! Bounded history UI; the owned worker performs every SQLite operation.
use crate::catalog as c;
use dlpad_core::jobs::JobState;
use dlpad_engine::download::{DownloadRepository, HISTORY_PAGE_SIZE, JobSummary};
use eframe::egui;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};

#[derive(Debug)]
enum Command {
    Page(Option<String>),
    Search(crate::list_search::Request, Option<String>),
    Resume(String),
}
#[derive(Debug)]
enum Outcome {
    Page(Result<Vec<JobSummary>, ()>),
    Search(crate::list_search::Request, Result<Vec<JobSummary>, ()>),
    Resume(Result<String, ()>),
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

pub(crate) struct HistoryPanel {
    rows: Vec<JobSummary>,
    search: crate::list_search::Search,
    error: bool,
    pending: bool,
    pending_search: Option<u64>,
    resume: Option<String>,
    commands: SyncSender<Command>,
    results: Receiver<Outcome>,
    signals: Arc<Signals>,
}
impl HistoryPanel {
    pub(crate) fn new(data: PathBuf) -> Self {
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::sync_channel(1);
        let signals = Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
        });
        let worker = signals.clone();
        let started = thread::Builder::new()
            .name("dlpad-history".into())
            .spawn(move || worker_entry(&worker, || run_worker(&data, rx, tx, &worker)));
        let mut panel = Self {
            rows: Vec::new(),
            search: Default::default(),
            error: started.is_err(),
            pending: false,
            pending_search: None,
            resume: None,
            commands,
            results,
            signals,
        };
        if started.is_err() {
            panel.signals.stopped.store(true, Ordering::Release);
        } else {
            panel.request(Command::Page(None));
        }
        panel
    }
    fn request(&mut self, command: Command) {
        self.search.cancel_result_reveal();
        if self.pending
            || self.resume.is_some()
            || self.signals.stop.load(Ordering::Acquire)
            || self.stopped()
        {
            return;
        }
        let search_generation = match &command {
            Command::Search(request, _) => Some(request.generation),
            _ => None,
        };
        match self.commands.try_send(command) {
            Ok(()) => {
                self.pending = true;
                self.pending_search = search_generation;
                self.error = false;
            }
            Err(_) => self.error = true,
        }
    }
    pub(crate) fn poll(&mut self) {
        if self.signals.failed.load(Ordering::Acquire) {
            self.search.cancel_result_reveal();
            self.pending = false;
            self.error = true;
        }
        if let Ok(outcome) = self.results.try_recv() {
            let expected = match &outcome {
                Outcome::Search(request, _) => self.pending_search == Some(request.generation),
                _ => self.pending_search.is_none(),
            };
            if !expected {
                return;
            }
            self.pending = false;
            self.pending_search = None;
            match outcome {
                Outcome::Page(Ok(rows)) if self.search.current.generation == 0 => {
                    self.rows = rows;
                    self.error = false;
                }
                Outcome::Search(request, rows) => {
                    if self.search.accepts(&request) {
                        match rows {
                            Ok(rows) => {
                                self.rows = rows;
                                self.error = false;
                                self.search.published(&request);
                                if request.query.is_empty() {
                                    self.search.cancel_result_reveal();
                                }
                            }
                            Err(()) => {
                                self.search.cancel_result_reveal();
                                self.error = true;
                            }
                        }
                    }
                }
                Outcome::Page(_) if self.search.current.generation != 0 => {}
                Outcome::Resume(Ok(id)) if !self.signals.stop.load(Ordering::Acquire) => {
                    self.resume = Some(id)
                }
                _ => self.error = true,
            }
        }
        self.dispatch_search();
    }
    fn dispatch_search(&mut self) {
        if !self.pending
            && self.resume.is_none()
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped()
            && let Some(request) = self.search.queued.take()
        {
            self.request(Command::Search(request, None));
        }
    }
    pub(crate) fn refresh(&mut self) {
        if self.search.current.generation == 0 {
            self.request(Command::Page(None));
        } else {
            let query = self.search.current.query.clone();
            self.search.enqueue(&query);
            self.dispatch_search();
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.pending
    }
    pub(crate) fn request_stop(&mut self) {
        self.resume = None;
        self.signals.stop.store(true, Ordering::Release);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire)
    }
    pub(crate) fn take_resume(&mut self) -> Option<String> {
        self.resume.take()
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, read_enabled: bool, resume_enabled: bool) {
        *self
            .signals
            .context
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(ui.ctx().clone());
        let active = read_enabled
            && !self.pending
            && self.resume.is_none()
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped();
        if !read_enabled || self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            self.search.cancel_result_reveal();
        }
        // One body scroll keeps the search toolbar and nonfocusable result Labels reachable.
        // Existing entity-scoped Resume IDs and worker dispatch remain authoritative.
        let history_body = egui::ScrollArea::vertical()
            .id_salt("history-body")
            .min_scrolled_height(0.0)
            .show_viewport(ui, |ui, viewport| {
                // Paint clips include stroke overscan. A result's complete logical
                // controls must fit the real scroll viewport, without that overscan.
                let visible_viewport = egui::Rect::from_min_size(
                    ui.max_rect().min + viewport.min.to_vec2(),
                    viewport.size(),
                )
                .intersect(ui.clip_rect());
                crate::accessible::collapsing(ui, c::HISTORY_RESUME_DETAILS, |ui| {
                    ui.label(c::HISTORY_HELP);
                });
                let search_enabled = read_enabled
                    && self.resume.is_none()
                    && !self.signals.stop.load(Ordering::Acquire)
                    && !self.stopped();
                self.search.show(
                    ui,
                    "history-search",
                    c::HISTORY_SEARCH_LABEL,
                    search_enabled,
                );
                self.dispatch_search();
                ui.horizontal_wrapped(|ui| {
                    if crate::ui_components::secondary(ui, c::HISTORY_REFRESH, active).clicked() {
                        self.refresh();
                    }
                    if crate::ui_components::ghost(
                        ui,
                        c::HISTORY_NEXT,
                        active && !self.search.stale() && self.rows.len() == HISTORY_PAGE_SIZE,
                    )
                    .clicked()
                        && let Some(last) = self.rows.last()
                    {
                        let after = Some(last.id.clone());
                        if self.search.current.generation == 0 {
                            self.request(Command::Page(after));
                        } else {
                            self.request(Command::Search(self.search.current.clone(), after));
                        }
                    }
                });
                if self.pending {
                    ui.label(c::HISTORY_LOADING);
                } else if self.error {
                    ui.colored_label(ui.visuals().error_fg_color, c::HISTORY_FAILED);
                    ui.weak(c::HISTORY_RETRY_HELP);
                } else if self.rows.is_empty() {
                    if self.search.current.query.is_empty() {
                        ui.label(c::HISTORY_EMPTY);
                        ui.weak(c::HISTORY_EMPTY_HELP);
                    } else {
                        let empty = ui.label(c::LIST_SEARCH_EMPTY);
                        self.search.reveal_result(ui, &empty);
                    }
                }
                // Keep resume limitations visible beside the affected action, even when
                // the general help is collapsed. Existing row eligibility is authoritative.
                if self.rows.iter().any(|row| row.resumable) {
                    ui.weak(c::HISTORY_HELP);
                }
                let mut resume = None;
                for (index, row) in self.rows.iter().enumerate() {
                    let row_group = ui.horizontal_wrapped(|ui| {
                        let result = ui.label(&row.id);
                        ui.label(state_label(row.state));
                        ui.label(format!("{} {}", c::ATTEMPTS, row.attempts));
                        if let Some(version) = &row.tool_version {
                            ui.label(format!("{} {}", c::HELP_VERSION, version));
                        }
                        if row.resumable {
                            ui.weak(c::HISTORY_RESUME_INSTRUCTION);
                        }
                        let resume_action = ui
                            .allocate_ui_with_layout(
                                egui::vec2(
                                    crate::ui_tokens::CONTROL_MIN_WIDTH,
                                    crate::ui_tokens::CONTROL_HEIGHT,
                                ),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.scope_builder(
                                        egui::UiBuilder::new()
                                            .id(egui::Id::new(("history-job-resume", &row.id))),
                                        |ui| {
                                            crate::ui_components::secondary(
                                                ui,
                                                c::RESUME,
                                                active && resume_enabled && row.resumable,
                                            )
                                        },
                                    )
                                    .inner
                                },
                            )
                            .inner;
                        if resume_action.clicked() {
                            resume = Some(row.id.clone());
                        }
                        result
                    });
                    // Keep status consequences in the body, separate from the scan label.
                    // Retain the original Label ID/description and entity-scoped Resume ID.
                    let result = row_group.inner;
                    let mut group_rect = row_group.response.rect;
                    let note = match row.state {
                        JobState::Cancelled => Some(c::HISTORY_CANCELLED_NOTE),
                        JobState::Downloaded => Some(c::HISTORY_DOWNLOADED_NOTE),
                        JobState::Failed => Some(c::HISTORY_FAILED_NOTE),
                        _ => None,
                    };
                    if let Some(note) = note {
                        group_rect = group_rect.union(ui.label(note).rect);
                    }
                    if !row.resumable
                        && matches!(
                            row.state,
                            JobState::Failed | JobState::Paused | JobState::Interrupted
                        )
                    {
                        group_rect =
                            group_rect.union(ui.label(c::HISTORY_OPTIONS_UNAVAILABLE_NOTE).rect);
                    }
                    #[cfg(test)]
                    ui.ctx().data_mut(|data| {
                        data.insert_temp(
                            egui::Id::new("history-note-reveal-group-test"),
                            group_rect,
                        )
                    });
                    if index == 0 && !self.pending && !self.error {
                        let revealed = self.search.reveal_result(ui, &result);
                        let clip = visible_viewport;
                        if revealed
                            && group_rect.width() <= clip.width()
                            && group_rect.height() <= clip.height()
                            && !clip.contains_rect(group_rect)
                        {
                            // egui 0.33 centers against content-origin + paint-clip
                            // height, which includes stroke overscan. Project the
                            // scroll request into that coordinate convention so the
                            // actual group centers in the real visible viewport.
                            // This changes only the request, never any Label/Resume rect.
                            let engine_center = ui.min_rect().top()
                                + viewport.min.y
                                + ui.clip_rect().height() * 0.5;
                            let correction = engine_center - clip.center().y;
                            let scroll_request = group_rect.translate(egui::vec2(0.0, correction));
                            #[cfg(test)]
                            eprintln!("history-center-projection actual_group={group_rect:?} actual_visible={clip:?} paint_clip={:?} content_origin={:?} offset={:?} engine_center={engine_center} correction={correction} request={scroll_request:?}", ui.clip_rect(), ui.min_rect().min, viewport.min);
                            // Replaces the ID reveal's earlier target; Ui::scroll_to_rect
                            // stores one target per axis, so these requests do not add.
                            ui.scroll_to_rect(scroll_request, Some(egui::Align::Center));
                        }
                    }
                }
                if let Some(id) = resume {
                    self.request(Command::Resume(id));
                }
            });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new("history-note-reveal-body-test"),
                (history_body.inner_rect, history_body.state.offset),
            );
            data.insert_temp(
                egui::Id::new("history-note-reveal-content-size-test"),
                history_body.content_size,
            )
        });
        #[cfg(not(test))]
        let _ = history_body;
        if self.pending || (self.signals.stop.load(Ordering::Acquire) && !self.stopped()) {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}
impl Drop for HistoryPanel {
    fn drop(&mut self) {
        self.signals.stop.store(true, Ordering::Release);
    }
}
fn state_label(state: JobState) -> &'static str {
    use JobState::*;
    match state {
        Downloaded => c::HISTORY_STATE_DOWNLOADED,
        Failed => c::HISTORY_STATE_FAILED,
        Cancelled => c::HISTORY_STATE_CANCELLED,
        Interrupted => c::STATE_INTERRUPTED,
        Paused => c::STATE_PAUSED,
        Queued => c::STATE_QUEUED,
        Preparing => c::STATE_PREPARING,
        Downloading => c::HISTORY_STATE_DOWNLOADING,
        PostProcessing => c::STATE_POSTPROCESSING,
        Verifying => c::STATE_VERIFYING,
        Stopping => c::STATE_STOPPING,
        RetryWaiting => c::STATE_RETRY,
    }
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn repository(data: &Path) -> Result<Option<DownloadRepository>, ()> {
    let path = data.join("jobs.sqlite");
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            DownloadRepository::open(&path).map(Some).map_err(|_| ())
        }
        _ => Err(()),
    }
}
fn page(data: &Path, after: Option<&str>) -> Result<Vec<JobSummary>, ()> {
    search_page(data, after, "")
}
fn search_page(data: &Path, after: Option<&str>, query: &str) -> Result<Vec<JobSummary>, ()> {
    if after.is_some_and(|id| !valid_id(id)) {
        return Err(());
    }
    let path = data.join("jobs.sqlite");
    let repo = match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            DownloadRepository::open_readonly(&path).map_err(|_| ())?
        }
        _ => return Err(()),
    };
    let rows = repo.history_search(after, query).map_err(|_| ())?;
    if rows.len() > HISTORY_PAGE_SIZE || rows.iter().any(|row| !valid_id(&row.id)) {
        return Err(());
    }
    Ok(rows)
}
fn validate_resume(data: &Path, id: &str) -> Result<String, ()> {
    if !valid_id(id) {
        return Err(());
    }
    let repo = repository(data)?.ok_or(())?;
    repo.validated_resume(id).map_err(|_| ())?;
    Ok(id.to_owned())
}
fn worker_entry(signals: &Arc<Signals>, run: impl FnOnce()) {
    let _ack = StopAck(signals.clone());
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).is_err() {
        signals.failed.store(true, Ordering::Release);
    }
}
fn run_worker(data: &Path, rx: Receiver<Command>, tx: SyncSender<Outcome>, signals: &Signals) {
    while !signals.stop.load(Ordering::Acquire) {
        let command = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(value) => value,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        };
        if signals.stop.load(Ordering::Acquire) {
            break;
        }
        let mut outcome = match command {
            Command::Page(after) => Outcome::Page(page(data, after.as_deref())),
            Command::Search(request, after) => {
                let rows = search_page(data, after.as_deref(), &request.query);
                Outcome::Search(request, rows)
            }
            Command::Resume(id) => Outcome::Resume(validate_resume(data, &id)),
        };
        loop {
            if signals.stop.load(Ordering::Acquire) {
                return;
            }
            match tx.try_send(outcome) {
                Ok(()) => {
                    signals.wake();
                    break;
                }
                Err(TrySendError::Disconnected(_)) => return,
                Err(TrySendError::Full(value)) => {
                    outcome = value;
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "dlpad-history-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn signals() -> Arc<Signals> {
        Arc::new(Signals {
            stop: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            context: Mutex::new(None),
        })
    }
    #[test]
    fn missing_journal_is_empty_and_resume_never_creates_it() {
        let fixture = Fixture::new();
        assert!(page(&fixture.0, None).unwrap().is_empty());
        assert!(validate_resume(&fixture.0, "job-1").is_err());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[test]
    fn invalid_ids_fail_closed_without_journal_creation() {
        let fixture = Fixture::new();
        for id in ["", "../job", "한글", &"a".repeat(81)] {
            assert!(validate_resume(&fixture.0, id).is_err());
        }
        assert!(page(&fixture.0, Some("../job")).is_err());
        assert!(!fixture.0.join("jobs.sqlite").exists());
    }
    #[test]
    fn corrupt_or_nonregular_journal_is_preserved() {
        let fixture = Fixture::new();
        let path = fixture.0.join("jobs.sqlite");
        std::fs::write(&path, b"owned invalid journal").unwrap();
        assert!(page(&fixture.0, None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"owned invalid journal");
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(page(&fixture.0, None).is_err());
        assert!(path.is_dir());
    }
    #[cfg(unix)]
    #[test]
    fn selected_journal_symlink_preserves_owned_target() {
        let fixture = Fixture::new();
        let target = fixture.0.join("target");
        std::fs::write(&target, b"owned target").unwrap();
        std::os::unix::fs::symlink(&target, fixture.0.join("jobs.sqlite")).unwrap();
        assert!(page(&fixture.0, None).is_err());
        assert_eq!(std::fs::read(target).unwrap(), b"owned target");
    }
    #[test]
    fn full_outcome_channel_can_stop_without_ui_consumption() {
        let fixture = Fixture::new();
        let signals = signals();
        let worker = signals.clone();
        let (commands, rx) = mpsc::sync_channel(1);
        let (tx, results) = mpsc::sync_channel(1);
        tx.send(Outcome::Page(Ok(Vec::new()))).unwrap();
        commands.send(Command::Page(None)).unwrap();
        let handle = thread::spawn(move || {
            let _ack = StopAck(worker.clone());
            run_worker(&fixture.0, rx, tx, &worker);
        });
        thread::sleep(Duration::from_millis(20));
        signals.stop.store(true, Ordering::Release);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !signals.stopped.load(Ordering::Acquire) && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(signals.stopped.load(Ordering::Acquire));
        handle.join().unwrap();
        drop(results);
    }
    #[test]
    fn stop_discards_pending_resume_delivery() {
        let fixture = Fixture::new();
        let mut panel = HistoryPanel::new(fixture.0.clone());
        panel.resume = Some("job-1".into());
        panel.request_stop();
        assert!(panel.take_resume().is_none());
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !panel.stopped() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(panel.stopped());
    }
    #[test]
    fn failed_page_preserves_previous_rows() {
        let fixture = Fixture::new();
        let mut panel = HistoryPanel::new(fixture.0.clone());
        panel.rows.push(JobSummary {
            id: "job-1".into(),
            state: JobState::Failed,
            attempts: 1,
            resumable: false,
            tool_version: None,
        });
        let (tx, rx) = mpsc::sync_channel(1);
        panel.results = rx;
        tx.send(Outcome::Page(Err(()))).unwrap();
        panel.poll();
        assert_eq!(panel.rows.len(), 1);
        assert_eq!(panel.rows[0].id, "job-1");
        assert!(panel.error);
        panel.request_stop();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !panel.stopped() && std::time::Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(panel.stopped());
    }
    #[test]
    fn panic_releases_owned_resources_before_stop_ack() {
        struct Resource(Arc<AtomicBool>);
        impl Drop for Resource {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let signals = signals();
        let released = Arc::new(AtomicBool::new(false));
        worker_entry(&signals, || {
            let _resource = Resource(released.clone());
            panic!("synthetic history worker panic");
        });
        assert!(released.load(Ordering::Acquire));
        assert!(signals.failed.load(Ordering::Acquire));
        assert!(signals.stopped.load(Ordering::Acquire));
    }
    // Render the production panel after actual page outcomes; no worker or user data.
    fn render_panel(panel: &mut HistoryPanel, enabled: bool) -> Vec<egui::accesskit::Node> {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, enabled, enabled));
            },
        );
        output
            .platform_output
            .accesskit_update
            .expect("actual history tree")
            .nodes
            .into_iter()
            .map(|(_, node)| node)
            .collect()
    }
    fn displayed(nodes: &[egui::accesskit::Node], text: &str) -> bool {
        nodes.iter().any(|node| {
            node.role() == egui::accesskit::Role::Label
                && (node.label() == Some(text) || node.value() == Some(text))
        })
    }
    fn button_disabled(nodes: &[egui::accesskit::Node], text: &str) -> bool {
        let mut matching = nodes.iter().filter(|node| {
            node.role() == egui::accesskit::Role::Button && node.label() == Some(text)
        });
        let node = matching.next().expect("actual history action");
        assert!(matching.next().is_none(), "unique history action");
        node.is_disabled()
    }
    fn render_fixture() -> (HistoryPanel, Receiver<Command>, SyncSender<Outcome>) {
        let (commands, receive) = mpsc::sync_channel(1);
        let (send, results) = mpsc::sync_channel(1);
        (
            HistoryPanel {
                rows: Vec::new(),
                search: Default::default(),
                error: false,
                pending: false,
                pending_search: None,
                resume: None,
                commands,
                results,
                signals: signals(),
            },
            receive,
            send,
        )
    }
    #[derive(Default)]
    struct HistoryActionTree {
        root: Option<egui::accesskit::NodeId>,
        focus: Option<egui::accesskit::NodeId>,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
    }
    impl HistoryActionTree {
        fn apply(&mut self, output: &egui::FullOutput) {
            let update = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("actual history update");
            if let Some(tree) = &update.tree {
                self.root = Some(tree.root);
            }
            self.focus = Some(update.focus);
            for (id, node) in &update.nodes {
                self.nodes.insert(*id, node.clone());
            }
            let mut live = std::collections::HashSet::new();
            let mut pending = vec![self.root.expect("actual history root")];
            while let Some(id) = pending.pop() {
                assert!(live.len() < 256, "bounded current history tree");
                if live.insert(id) {
                    pending.extend(self.nodes[&id].children().iter().copied());
                }
            }
            self.nodes.retain(|id, _| live.contains(id));
        }
    }
    fn identity_row(id: &str) -> JobSummary {
        JobSummary {
            id: id.into(),
            state: JobState::Failed,
            attempts: 1,
            resumable: true,
            tool_version: None,
        }
    }
    fn history_identity_frame(
        ctx: &egui::Context,
        panel: &mut HistoryPanel,
        tree: &mut HistoryActionTree,
        events: Vec<egui::Event>,
    ) -> std::collections::BTreeMap<String, egui::accesskit::NodeId> {
        use egui::accesskit::Role;
        let output = ctx.run(
            egui::RawInput {
                focused: true,
                time: Some(ctx.cumulative_frame_nr() as f64 * 0.1),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, true, true));
            },
        );
        tree.apply(&output);
        // Associate actual painted row labels with actual action bounds, not a recreated ID hash.
        let mut actions = std::collections::BTreeMap::new();
        for row in &panel.rows {
            let labels: Vec<_> = tree
                .nodes
                .values()
                .filter(|n| {
                    n.role() == Role::Label
                        && (n.label() == Some(row.id.as_str())
                            || n.value() == Some(row.id.as_str()))
                })
                .collect();
            assert_eq!(labels.len(), 1, "one current job label");
            let label = labels[0].bounds().expect("actual job label bounds");
            // A horizontal_wrapped row can legitimately put Resume on its next line.
            // Bind to the current job's vertical row band, never a larger center tolerance.
            let next_job_top = panel
                .rows
                .iter()
                .filter(|other| other.id != row.id)
                .filter_map(|other| {
                    tree.nodes
                        .values()
                        .find(|n| {
                            n.role() == Role::Label
                                && (n.label() == Some(other.id.as_str())
                                    || n.value() == Some(other.id.as_str()))
                        })
                        .and_then(|n| n.bounds())
                        .map(|b| b.y0)
                })
                .filter(|top| *top > label.y0)
                .min_by(f64::total_cmp)
                .unwrap_or(600.0);
            let matches: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, n)| {
                    n.role() == Role::Button
                        && n.label() == Some(c::RESUME)
                        && n.bounds()
                            .is_some_and(|b| b.y0 >= label.y0 && b.y1 <= next_job_top)
                })
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "one current Resume inside this job's actual wrapped row band"
            );
            let bounds = matches[0].1.bounds().unwrap();
            let rect = egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            );
            assert!(
                (rect.height() - crate::ui_tokens::CONTROL_HEIGHT).abs() < 0.01,
                "actual original32px Resume height: {rect:?}"
            );
            assert!(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))
                    .contains_rect(rect),
                "whole current Resume remains within800 viewport: {rect:?}"
            );
            assert!(output.shapes.iter().any(|paint| matches!(&paint.shape, egui::Shape::Rect(background) if background.rect == rect && background.fill != egui::Color32::TRANSPARENT && paint.clip_rect.contains_rect(rect))), "whole current Resume background paints inside its ownclip");
            actions.insert(row.id.clone(), *matches[0].0);
        }
        assert_eq!(actions.len(), panel.rows.len());
        actions
    }
    fn history_identity_action(
        id: egui::accesskit::NodeId,
        action: egui::accesskit::Action,
    ) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action,
            target: id,
            data: None,
        })
    }
    fn history_identity_context() -> egui::Context {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        ctx
    }
    #[test]
    fn actual_history_resume_identity_and_focus_survive_list_changes() {
        use egui::accesskit::Action;
        let (mut panel, commands, outcomes) = render_fixture();
        outcomes
            .send(Outcome::Page(Ok(vec![
                identity_row("history-a"),
                identity_row("history-b"),
            ])))
            .unwrap();
        panel.poll();
        let ctx = history_identity_context();
        let mut tree = HistoryActionTree::default();
        let original = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
        assert_ne!(original["history-a"], original["history-b"]);
        history_identity_frame(
            &ctx,
            &mut panel,
            &mut tree,
            vec![history_identity_action(
                original["history-b"],
                Action::Focus,
            )],
        );
        assert_eq!(
            tree.focus,
            Some(original["history-b"]),
            "actual Focus action acknowledged"
        );
        for rows in [
            vec![identity_row("history-b"), identity_row("history-a")],
            vec![
                identity_row("history-c"),
                identity_row("history-a"),
                identity_row("history-b"),
            ],
            vec![identity_row("history-b")],
        ] {
            outcomes.send(Outcome::Page(Ok(rows))).unwrap();
            panel.poll();
            let current = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
            assert_eq!(
                current["history-b"], original["history-b"],
                "same actual entity action after reorder/prepend/remove"
            );
            assert_eq!(
                tree.focus,
                Some(original["history-b"]),
                "focus remains on the same live job"
            );
            if let Some(a) = current.get("history-a") {
                assert_eq!(*a, original["history-a"]);
            }
        }
        // Metadata changes also must not renumber the focused action.
        panel.rows[0].tool_version = Some("v1".into());
        let current = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
        assert_eq!(current["history-b"], original["history-b"]);
        assert_eq!(tree.focus, Some(original["history-b"]));
        assert!(
            commands.try_recv().is_err(),
            "list/focus changes are not resume commands"
        );
    }
    #[test]
    fn actual_history_original_action_routes_to_same_job_after_reorder_prepend_and_remove() {
        use egui::accesskit::Action;
        for rows in [
            vec![identity_row("history-b"), identity_row("history-a")],
            vec![
                identity_row("history-c"),
                identity_row("history-a"),
                identity_row("history-b"),
            ],
            vec![identity_row("history-b")],
        ] {
            let (mut panel, commands, outcomes) = render_fixture();
            outcomes
                .send(Outcome::Page(Ok(vec![
                    identity_row("history-a"),
                    identity_row("history-b"),
                ])))
                .unwrap();
            panel.poll();
            let ctx = history_identity_context();
            let mut tree = HistoryActionTree::default();
            let original = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
            outcomes.send(Outcome::Page(Ok(rows))).unwrap();
            panel.poll();
            history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
            history_identity_frame(
                &ctx,
                &mut panel,
                &mut tree,
                vec![history_identity_action(
                    original["history-b"],
                    Action::Click,
                )],
            );
            match commands.try_recv() {
                Ok(Command::Resume(id)) => assert_eq!(
                    id, "history-b",
                    "actual original node routes to the original entity"
                ),
                other => panic!("expected Resume(history-b), observed {other:?}"),
            }
            assert!(panel.pending);
        }
    }
    #[test]
    fn actual_history_removed_action_cannot_resume_a_replacement_job() {
        use egui::accesskit::Action;
        let (mut panel, commands, outcomes) = render_fixture();
        outcomes
            .send(Outcome::Page(Ok(vec![
                identity_row("history-a"),
                identity_row("history-b"),
            ])))
            .unwrap();
        panel.poll();
        let ctx = history_identity_context();
        let mut tree = HistoryActionTree::default();
        let original = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
        outcomes
            .send(Outcome::Page(Ok(vec![
                identity_row("history-a"),
                identity_row("history-c"),
            ])))
            .unwrap();
        panel.poll();
        let current = history_identity_frame(&ctx, &mut panel, &mut tree, vec![]);
        assert!(
            !tree.nodes.contains_key(&original["history-b"]),
            "removed action is absent from the current rooted tree"
        );
        assert_ne!(current["history-c"], original["history-b"]);
        history_identity_frame(
            &ctx,
            &mut panel,
            &mut tree,
            vec![history_identity_action(
                original["history-b"],
                Action::Click,
            )],
        );
        assert!(
            commands.try_recv().is_err(),
            "removed entity action is refused instead of reused"
        );
        assert!(!panel.pending);
        history_identity_frame(
            &ctx,
            &mut panel,
            &mut tree,
            vec![history_identity_action(current["history-c"], Action::Click)],
        );
        assert!(
            matches!(commands.try_recv(), Ok(Command::Resume(id)) if id == "history-c"),
            "the live replacement action still works"
        );
    }

    #[test]
    fn actual_history_single_metadata_resume_paints_inside_original_800_viewport() {
        use egui::accesskit::{Action, ActionRequest, Role};
        let (mut panel, commands, _outcomes) = render_fixture();
        panel.rows = vec![JobSummary {
            id: "history-b".into(),
            state: JobState::Failed,
            attempts: 1,
            resumable: true,
            tool_version: None,
        }];
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut nodes = std::collections::HashMap::new();
        let mut root = None;
        let mut target = None;
        for frame in 0..8 {
            if frame == 4 {
                panel.rows[0].tool_version = Some("v1".into());
            }
            let events = if frame == 3 {
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target: target.expect("initial actual Resume node"),
                    data: None,
                })]
            } else {
                vec![]
            };
            let output = ctx.run(
                egui::RawInput {
                    focused: true,
                    time: Some(f64::from(frame) * 0.1),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, true, true));
                },
            );
            let update = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("actual geometry tree");
            if let Some(tree) = &update.tree {
                root = Some(tree.root);
            }
            for (id, node) in &update.nodes {
                nodes.insert(*id, node.clone());
            }
            let mut seen = std::collections::HashSet::new();
            let mut pending = vec![root.expect("current root")];
            while let Some(id) = pending.pop() {
                assert!(seen.len() < 256);
                if seen.insert(id) {
                    pending.extend(nodes[&id].children().iter().copied());
                }
            }
            nodes.retain(|id, _| seen.contains(id));
            let buttons: Vec<_> = nodes
                .iter()
                .filter(|(_, n)| n.role() == Role::Button && n.label() == Some(c::RESUME))
                .collect();
            assert_eq!(buttons.len(), 1, "one actual Resume");
            target = Some(*buttons[0].0);
            let b = buttons[0].1.bounds().expect("actual button bounds");
            let rect = egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            );
            let label = nodes
                .values()
                .find(|n| {
                    n.role() == Role::Label
                        && (n.label() == Some("history-b") || n.value() == Some("history-b"))
                })
                .unwrap()
                .bounds()
                .unwrap();
            let paints: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Rect(r)
                        if r.rect == rect && r.fill != egui::Color32::TRANSPARENT =>
                    {
                        Some((r.rect, s.clip_rect))
                    }
                    _ => None,
                })
                .collect();
            let glyphs: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == c::RESUME => {
                        Some((t.galley.rect.translate(t.pos.to_vec2()), s.clip_rect))
                    }
                    _ => None,
                })
                .collect();
            eprintln!(
                "history_metadata_geometry frame={frame} version={:?} node={:?} focus={:?} job_label={label:?} Resume={rect:?} paints={paints:?} glyphs={glyphs:?}",
                panel.rows[0].tool_version, target, update.focus
            );
            assert!(
                (rect.height() - crate::ui_tokens::CONTROL_HEIGHT).abs() < 0.01,
                "original32px Resume height: frame={frame}, actual={rect:?}"
            );
            assert!(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))
                    .contains_rect(rect),
                "actual whole Resume remains inside800 viewport: {rect:?}"
            );
            assert!(
                !paints.is_empty() && paints.iter().all(|(r, c)| c.contains_rect(*r)),
                "full actual Resume background painted within ownclip"
            );
            assert!(
                !glyphs.is_empty() && glyphs.iter().all(|(r, c)| c.contains_rect(*r)),
                "full actual Resume glyphs within ownclip"
            );
        }
        assert!(
            commands.try_recv().is_err(),
            "read/focus/metadata diagnostic sends no command"
        );
    }

    #[test]
    fn actual_history_empty_failure_pending_labels_are_exclusive() {
        let fixture = Fixture::new();
        let (mut panel, _commands, outcomes) = render_fixture();
        outcomes
            .send(Outcome::Page(page(&fixture.0, None)))
            .unwrap();
        panel.poll();
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, c::HISTORY_EMPTY));
        assert!(displayed(&nodes, c::HISTORY_EMPTY_HELP));
        assert!(!displayed(&nodes, c::HISTORY_FAILED));
        assert!(!displayed(&nodes, c::HISTORY_LOADING));
        assert!(
            !displayed(&nodes, c::HISTORY_HELP),
            "general resume guidance starts collapsed"
        );
        assert!(!fixture.0.join("jobs.sqlite").exists());
        let corrupt = b"owned corrupt history fixture";
        std::fs::write(fixture.0.join("jobs.sqlite"), corrupt).unwrap();
        outcomes
            .send(Outcome::Page(page(&fixture.0, None)))
            .unwrap();
        panel.poll();
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, c::HISTORY_FAILED));
        assert!(displayed(&nodes, c::HISTORY_RETRY_HELP));
        assert!(!displayed(&nodes, c::HISTORY_EMPTY));
        assert!(!displayed(&nodes, c::HISTORY_EMPTY_HELP));
        assert!(!displayed(&nodes, c::HISTORY_LOADING));
        panel.refresh();
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, c::HISTORY_LOADING));
        assert!(!displayed(&nodes, c::HISTORY_FAILED));
        assert!(!displayed(&nodes, c::HISTORY_RETRY_HELP));
        assert!(!displayed(&nodes, c::HISTORY_EMPTY));
        assert!(button_disabled(&nodes, c::HISTORY_REFRESH));
        assert!(button_disabled(&nodes, c::HISTORY_NEXT));
        assert_eq!(
            std::fs::read(fixture.0.join("jobs.sqlite")).unwrap(),
            corrupt
        );
    }
    #[test]
    fn actual_history_failure_preserves_rows_and_pending_disables_resume() {
        let (mut panel, _commands, outcomes) = render_fixture();
        panel.rows.push(JobSummary {
            id: "owned-resumable-job".into(),
            state: JobState::Failed,
            attempts: 2,
            resumable: true,
            tool_version: Some("fixture-version".into()),
        });
        outcomes.send(Outcome::Page(Err(()))).unwrap();
        panel.poll();
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, "owned-resumable-job"));
        assert!(displayed(&nodes, c::HISTORY_FAILED));
        assert!(!displayed(&nodes, c::HISTORY_EMPTY));
        assert!(
            displayed(&nodes, c::HISTORY_HELP),
            "resumable row retains full limitations"
        );
        assert!(displayed(&nodes, c::HISTORY_RESUME_INSTRUCTION));
        assert!(!button_disabled(&nodes, c::RESUME));
        panel.refresh();
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, "owned-resumable-job"));
        assert!(displayed(&nodes, c::HISTORY_LOADING));
        assert!(displayed(&nodes, c::HISTORY_HELP));
        assert!(button_disabled(&nodes, c::RESUME));
        assert_eq!(panel.rows.len(), 1);
    }
    #[test]
    fn actual_history_nonresumable_rows_and_external_gates_disable_actions() {
        let (mut panel, _commands, _outcomes) = render_fixture();
        panel.rows.push(JobSummary {
            id: "owned-completed-job".into(),
            state: JobState::Downloaded,
            attempts: 1,
            resumable: false,
            tool_version: None,
        });
        let nodes = render_panel(&mut panel, true);
        assert!(displayed(&nodes, "owned-completed-job"));
        assert!(!displayed(&nodes, c::HISTORY_HELP));
        assert!(!displayed(&nodes, c::HISTORY_RESUME_INSTRUCTION));
        assert!(button_disabled(&nodes, c::RESUME));
        assert!(!button_disabled(&nodes, c::HISTORY_REFRESH));
        assert!(button_disabled(&nodes, c::HISTORY_NEXT));
        panel.rows.clear();
        for index in 0..HISTORY_PAGE_SIZE {
            panel.rows.push(JobSummary {
                id: format!("owned-{index}"),
                state: JobState::Failed,
                attempts: 1,
                resumable: false,
                tool_version: None,
            });
        }
        let nodes = render_panel(&mut panel, true);
        assert!(!button_disabled(&nodes, c::HISTORY_NEXT));
        for enabled in [false, true] {
            panel.signals.stop.store(enabled, Ordering::Release);
            let nodes = render_panel(&mut panel, enabled);
            assert!(button_disabled(&nodes, c::HISTORY_REFRESH));
            assert!(button_disabled(&nodes, c::HISTORY_NEXT));
        }
    }
    #[test]
    fn actual_history_read_retry_and_next_do_not_enable_resume_without_readiness() {
        let (mut panel, _commands, _outcomes) = render_fixture();
        for index in 0..HISTORY_PAGE_SIZE {
            panel.rows.push(JobSummary {
                id: format!("owned-ready-{index}"),
                state: JobState::Failed,
                attempts: 1,
                resumable: index == 0,
                tool_version: None,
            });
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| panel.show(ui, true, false));
            },
        );
        let nodes: Vec<_> = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .map(|(_, node)| node)
            .collect();
        assert!(!button_disabled(&nodes, c::HISTORY_REFRESH));
        assert!(!button_disabled(&nodes, c::HISTORY_NEXT));
        let resumes: Vec<_> = nodes
            .iter()
            .filter(|node| {
                node.role() == egui::accesskit::Role::Button && node.label() == Some(c::RESUME)
            })
            .collect();
        assert!(!resumes.is_empty() && resumes.iter().all(|node| node.is_disabled()));
        assert!(displayed(&nodes, c::HISTORY_HELP));
    }
}

#[cfg(test)]
mod search_interaction_tests {
    use super::*;
    use crate::list_search::interaction_tests::{Fixture, enter};
    #[test]
    fn actual_history_compact_status_and_visible_limits_fit_searched_row_at_200_percent() {
        use egui::accesskit::{Action, Role};
        for body_height in [60.0, 125.0] {
            for zoom in [1.0, 2.0] {
                for (state, expected, notes) in [
                    (
                        JobState::Cancelled,
                        c::HISTORY_STATE_CANCELLED,
                        vec![c::HISTORY_CANCELLED_NOTE],
                    ),
                    (
                        JobState::Downloaded,
                        c::HISTORY_STATE_DOWNLOADED,
                        vec![c::HISTORY_DOWNLOADED_NOTE],
                    ),
                    (
                        JobState::Failed,
                        c::HISTORY_STATE_FAILED,
                        vec![c::HISTORY_FAILED_NOTE, c::HISTORY_OPTIONS_UNAVAILABLE_NOTE],
                    ),
                    (JobState::Downloading, c::HISTORY_STATE_DOWNLOADING, vec![]),
                ] {
                    let (commands, receive) = mpsc::sync_channel(1);
                    let (send, results) = mpsc::sync_channel(1);
                    let mut panel = HistoryPanel {
                        rows: Vec::new(),
                        search: Default::default(),
                        error: false,
                        pending: false,
                        pending_search: None,
                        resume: None,
                        commands,
                        results,
                        signals: Arc::new(Signals {
                            stop: AtomicBool::new(false),
                            stopped: AtomicBool::new(false),
                            failed: AtomicBool::new(false),
                            context: Mutex::new(None),
                        }),
                    };
                    let mut f = Fixture::new();
                    f.ctx.options_mut(|options| options.zoom_factor = zoom);
                    let render = |f: &mut Fixture, panel: &mut HistoryPanel, events| {
                        f.frame(events, |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), body_height),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_max_height(body_height);
                                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                                    panel.show(ui, true, true);
                                },
                            );
                        });
                    };
                    render(&mut f, &mut panel, vec![]);
                    let focus = f.action(Role::TextInput, None, Action::Focus);
                    render(&mut f, &mut panel, vec![focus]);
                    render(
                        &mut f,
                        &mut panel,
                        vec![egui::Event::Text("history".into())],
                    );
                    render(&mut f, &mut panel, vec![enter()]);
                    let Command::Search(request, None) =
                        receive.try_recv().expect("real Enter search")
                    else {
                        panic!("read-only History search");
                    };
                    // Enter may surrender focus and pending controls may be disabled.
                    // Observe that real state; publication must not acquire new focus.
                    let original_focus = f.ctx.memory(|memory| memory.focused());
                    send.send(Outcome::Search(
                        request,
                        Ok(vec![JobSummary {
                            id: "history-12345678-1234-1234-1234567890ab".into(),
                            state,
                            attempts: 1,
                            resumable: false,
                            tool_version: Some("2026.09.24".into()),
                        }]),
                    ))
                    .unwrap();
                    panel.poll();
                    for _ in 0..32 {
                        render(&mut f, &mut panel, vec![]);
                    }
                    assert!(
                        f.fully_painted(expected),
                        "compact {state:?} status fully painted at zoom={zoom}"
                    );
                    assert!(
                        f.is_nonfocusable_label(expected),
                        "state is readable without pointer hover"
                    );
                    if body_height >= 125.0 {
                        for note in notes {
                            assert!(
                                f.fully_painted(note),
                                "consequence/restriction remains fully painted at zoom={zoom}: {note}"
                            );
                            assert!(
                                f.is_nonfocusable_label(note),
                                "important reason is a body Label, not hover-only"
                            );
                        }
                    }
                    assert!(
                        f.fully_painted("history-12345678-1234-1234-1234567890ab"),
                        "oversized context preserves original ID visibility"
                    );
                    assert_eq!(
                        f.ctx.memory(|memory| memory.focused()),
                        original_focus,
                        "result publication preserves the actual post-Enter focus state"
                    );
                    // Failure-path injection, not an enabled OS Invoke claim.
                    let disabled_resume = f.disabled_button_click(c::RESUME);
                    render(&mut f, &mut panel, vec![disabled_resume]);
                    assert!(
                        receive.try_recv().is_err(),
                        "compact status never enables nonresumable action"
                    );
                }
            }
        }
    }
    #[test]
    fn actual_history_result_reveal_rejects_stale_draft_old_query_failure_and_stop() {
        for case in 0..4 {
            let (commands, receive) = mpsc::sync_channel(1);
            let (send, results) = mpsc::sync_channel(1);
            let mut panel = HistoryPanel {
                rows: Vec::new(),
                search: Default::default(),
                error: false,
                pending: false,
                pending_search: None,
                resume: None,
                commands,
                results,
                signals: Arc::new(Signals {
                    stop: AtomicBool::new(false),
                    stopped: AtomicBool::new(false),
                    failed: AtomicBool::new(false),
                    context: Mutex::new(None),
                }),
            };
            let mut f = Fixture::new();
            let render = |f: &mut Fixture, panel: &mut HistoryPanel, events| {
                f.frame(events, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 125.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_height(125.0);
                            ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                            assert!(
                                (ui.clip_rect().height() - 125.0).abs() < 0.01,
                                "actual fresh History component body clip before panel"
                            );
                            panel.show(ui, true, false);
                        },
                    );
                });
            };
            render(&mut f, &mut panel, vec![]);
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            render(&mut f, &mut panel, vec![focus]);
            render(
                &mut f,
                &mut panel,
                vec![egui::Event::Text("첫-query".into())],
            );
            render(&mut f, &mut panel, vec![enter()]);
            let Command::Search(request, None) = receive.try_recv().unwrap() else {
                panic!("actual Search dispatch");
            };
            let focus = f.action(
                egui::accesskit::Role::TextInput,
                None,
                egui::accesskit::Action::Focus,
            );
            render(&mut f, &mut panel, vec![focus]);
            if case == 0 {
                render(
                    &mut f,
                    &mut panel,
                    vec![egui::Event::Text("draft-changed".into())],
                );
            }
            if case == 1 {
                let clear = f.action(
                    egui::accesskit::Role::Button,
                    Some(c::LIST_SEARCH_CLEAR),
                    egui::accesskit::Action::Click,
                );
                render(&mut f, &mut panel, vec![clear]);
            }
            if case == 3 {
                panel.request_stop();
            }
            let response = if case == 2 {
                Err(())
            } else {
                Ok(vec![JobSummary {
                    id: "retained-query-record".into(),
                    state: JobState::Cancelled,
                    attempts: 0,
                    resumable: false,
                    tool_version: None,
                }])
            };
            send.send(Outcome::Search(request, response)).unwrap();
            panel.poll();
            for _ in 0..32 {
                render(&mut f, &mut panel, vec![]);
            }
            if case == 0 {
                assert_eq!(
                    panel.rows[0].id, "retained-query-record",
                    "matching previous query can publish under explicit stale-query label"
                );
                assert!(
                    !f.fully_painted("retained-query-record"),
                    "stale draft must not auto-scroll to old query results"
                );
            }
            if case == 1 || case == 2 {
                assert!(
                    panel.rows.is_empty(),
                    "old reply and failure preserve prior empty page"
                );
            }
            assert!(
                !f.fully_painted(c::LIST_SEARCH_EMPTY),
                "failure/Stop must not fabricate a successful empty-result reveal"
            );
        }
    }
    #[test]
    fn actual_history_search_result_label_reveals_in_single_body_without_focus_theft() {
        for zoom in [1.0, 2.0] {
            for empty in [false, true] {
                let (commands, receive) = mpsc::sync_channel(1);
                let (send, results) = mpsc::sync_channel(1);
                let mut panel = HistoryPanel {
                    rows: Vec::new(),
                    search: Default::default(),
                    error: false,
                    pending: false,
                    pending_search: None,
                    resume: None,
                    commands,
                    results,
                    signals: Arc::new(Signals {
                        stop: AtomicBool::new(false),
                        stopped: AtomicBool::new(false),
                        failed: AtomicBool::new(false),
                        context: Mutex::new(None),
                    }),
                };
                let mut f = Fixture::new();
                f.ctx.options_mut(|options| options.zoom_factor = zoom);
                let render = |f: &mut Fixture, panel: &mut HistoryPanel, events| {
                    f.frame(events, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 125.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_max_height(125.0);
                                ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                                assert!(
                                    (ui.clip_rect().height() - 125.0).abs() < 0.01,
                                    "actual fresh History component body clip before panel"
                                );
                                panel.show(ui, true, false);
                            },
                        );
                    });
                };
                render(&mut f, &mut panel, vec![]);
                let focus = f.action(
                    egui::accesskit::Role::TextInput,
                    None,
                    egui::accesskit::Action::Focus,
                );
                render(&mut f, &mut panel, vec![focus]);
                render(
                    &mut f,
                    &mut panel,
                    vec![egui::Event::Text("한글-query".into())],
                );
                render(&mut f, &mut panel, vec![enter()]);
                let Command::Search(request, None) =
                    receive.try_recv().expect("real History Enter dispatch")
                else {
                    panic!("readonly search");
                };
                let focus = f.action(
                    egui::accesskit::Role::TextInput,
                    None,
                    egui::accesskit::Action::Focus,
                );
                render(&mut f, &mut panel, vec![focus]);
                let original_focus = f.ctx.memory(|memory| memory.focused());
                assert!(original_focus.is_some());
                let rows = if empty {
                    Vec::new()
                } else {
                    vec![JobSummary {
                        id: "query-record".into(),
                        state: JobState::Cancelled,
                        attempts: 0,
                        resumable: false,
                        tool_version: None,
                    }]
                };
                send.send(Outcome::Search(request, Ok(rows))).unwrap();
                panel.poll();
                for _ in 0..32 {
                    render(&mut f, &mut panel, vec![]);
                }
                let expected = if empty {
                    c::LIST_SEARCH_EMPTY
                } else {
                    "query-record"
                };
                assert!(
                    f.fully_painted(expected),
                    "actual nonfocusable history result fully painted inside125logical body at100/200"
                );
                assert!(f.is_nonfocusable_label(expected));
                assert_eq!(f.ctx.memory(|memory| memory.focused()), original_focus);
            }
        }
    }
    #[test]
    fn actual_history_search_latest_request_and_failed_reply_preserve_previous_rows() {
        let (commands, receive) = mpsc::sync_channel(1);
        let (send, results) = mpsc::sync_channel(1);
        let mut panel = HistoryPanel {
            rows: vec![JobSummary {
                id: "prior".into(),
                state: JobState::Queued,
                attempts: 0,
                resumable: false,
                tool_version: None,
            }],
            search: Default::default(),
            error: false,
            pending: false,
            pending_search: None,
            resume: None,
            commands,
            results,
            signals: Arc::new(Signals {
                stop: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
                failed: AtomicBool::new(false),
                context: Mutex::new(None),
            }),
        };
        let mut f = Fixture::new();
        f.frame(vec![], |ui| panel.show(ui, true, false));
        let focus = f.action(
            egui::accesskit::Role::TextInput,
            None,
            egui::accesskit::Action::Focus,
        );
        f.frame(vec![focus], |ui| panel.show(ui, true, false));
        f.frame(vec![egui::Event::Text(" 한글A ".into())], |ui| {
            panel.show(ui, true, false)
        });
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
        f.frame(vec![enter()], |ui| panel.show(ui, true, false));
        let Command::Search(first, None) = receive.try_recv().expect("actual first search command")
        else {
            panic!("search must be first page")
        };
        assert_eq!(first.query, "한글A");
        let clear = f.action(
            egui::accesskit::Role::Button,
            Some(c::LIST_SEARCH_CLEAR),
            egui::accesskit::Action::Click,
        );
        f.frame(vec![clear], |ui| panel.show(ui, true, false));
        let focus = f.action(
            egui::accesskit::Role::TextInput,
            None,
            egui::accesskit::Action::Focus,
        );
        f.frame(vec![focus], |ui| panel.show(ui, true, false));
        f.frame(vec![egui::Event::Text("한글B".into())], |ui| {
            panel.show(ui, true, false)
        });
        f.frame(vec![enter()], |ui| panel.show(ui, true, false));
        assert!(
            matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "one in-flight worker request"
        );
        send.send(Outcome::Search(first.clone(), Ok(Vec::new())))
            .unwrap();
        panel.poll();
        assert_eq!(
            panel.rows[0].id, "prior",
            "late first results do not replace displayed rows"
        );
        let Command::Search(latest, None) = receive.try_recv().expect("latest-only queued request")
        else {
            panic!("search first page")
        };
        assert_eq!(latest.query, "한글B");
        assert!(
            latest.generation > first.generation + 1,
            "queued clear was replaced, not separately queried"
        );
        send.send(Outcome::Search(first, Ok(Vec::new()))).unwrap();
        panel.poll();
        assert!(
            panel.pending,
            "mismatched late reply cannot acknowledge the new request"
        );
        send.send(Outcome::Search(latest, Err(()))).unwrap();
        panel.poll();
        assert!(!panel.pending && panel.error && panel.search.stale());
        assert_eq!(panel.rows[0].id, "prior");
        f.frame(vec![], |ui| panel.show(ui, true, false));
        panel.request_stop();
        let clear = f.action(
            egui::accesskit::Role::Button,
            Some(c::LIST_SEARCH_CLEAR),
            egui::accesskit::Action::Click,
        );
        f.frame(vec![clear], |ui| panel.show(ui, false, false));
        assert!(
            matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "CloseWait refuses new database requests"
        );
    }
}

#[cfg(test)]
mod searched_note_viewport_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
    use std::collections::{HashMap, HashSet};
    const JOB: &str = "history-failed-12345678-1234-1234";
    struct Harness {
        ctx: egui::Context,
        panel: HistoryPanel,
        receive: mpsc::Receiver<Command>,
        send: mpsc::SyncSender<Outcome>,
        nodes: HashMap<NodeId, Node>,
        root: Option<NodeId>,
        height: f32,
        time: f64,
        viewport: egui::Rect,
        group: egui::Rect,
        offset: egui::Vec2,
    }
    impl Harness {
        fn new(zoom: f32, height: f32) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.enable_accesskit();
            let (commands, receive) = mpsc::sync_channel(1);
            let (send, results) = mpsc::sync_channel(1);
            Self {
                ctx,
                panel: HistoryPanel {
                    rows: Vec::new(),
                    search: Default::default(),
                    error: false,
                    pending: false,
                    pending_search: None,
                    resume: None,
                    commands,
                    results,
                    signals: Arc::new(Signals {
                        stop: AtomicBool::new(false),
                        stopped: AtomicBool::new(false),
                        failed: AtomicBool::new(false),
                        context: Mutex::new(None),
                    }),
                },
                receive,
                send,
                nodes: Default::default(),
                root: None,
                height,
                time: 0.0,
                viewport: egui::Rect::NOTHING,
                group: egui::Rect::NOTHING,
                offset: egui::Vec2::ZERO,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += 0.05;
            let out = self.ctx.run(
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
                        ui.allocate_ui_with_layout(
                            egui::vec2(352.0, self.height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(352.0);
                                ui.set_max_height(self.height);
                                // Match Main: the paint clip is wider than the content max_rect.
                                // Do not erase egui's stroke overscan by forcing clip=max_rect.
                                assert!(
                                    ui.clip_rect().height() > self.height + 6.0,
                                    "setup: parent paint clip includes room outside the content box"
                                );
                                self.panel.show(ui, true, true);
                            },
                        );
                    });
                },
            );
            (self.viewport, self.offset) = self
                .ctx
                .data(|data| data.get_temp(egui::Id::new("history-note-reveal-body-test")))
                .unwrap();
            self.group = self
                .ctx
                .data(|data| data.get_temp(egui::Id::new("history-note-reveal-group-test")))
                .unwrap_or(egui::Rect::NOTHING);
            assert!(
                (self.viewport.height() - self.height).abs() < 0.01,
                "setup: actual inner viewport is bounded, not minimum-height expanded"
            );
            let update = out.platform_output.accesskit_update.as_ref().unwrap();
            if let Some(tree) = &update.tree {
                self.root = Some(tree.root);
            }
            self.nodes.extend(update.nodes.iter().cloned());
            let mut live = HashSet::new();
            let mut pending = vec![self.root.unwrap()];
            while let Some(id) = pending.pop() {
                assert!(live.len() < 256);
                if live.insert(id) {
                    pending.extend(self.nodes[&id].children().iter().copied());
                }
            }
            self.nodes.retain(|id, _| live.contains(id));
            out
        }
        fn exact(&self, role: Role, label: &str) -> (NodeId, &Node) {
            let mut found = self.nodes.iter().filter(|(_, node)| {
                node.role() == role
                    && (node.label() == Some(label)
                        || node.value() == Some(label)
                        || node.labelled_by().iter().any(|id| {
                            self.nodes.get(id).is_some_and(|caption| {
                                caption.label() == Some(label) || caption.value() == Some(label)
                            })
                        }))
            });
            let (id, node) = found.next().expect("actual named node");
            assert!(found.next().is_none(), "unique real node");
            (*id, node)
        }
        fn action(&mut self, role: Role, label: &str, action: Action) {
            let (id, node) = self.exact(role, label);
            assert!(!node.is_disabled());
            assert!(node.supports_action(action));
            self.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target: id,
                data: None,
            })]);
        }
        fn bounds(node: &Node) -> egui::Rect {
            let b = node.bounds().unwrap();
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        }
    }
    #[test]
    fn actual_searched_resumable_failed_note_and_resume_fit_inner_viewport_without_overscan() {
        for zoom in [1.0, 2.0] {
            for height in [117.5, 125.0] {
                for version in [None, Some("2026.09.24".to_owned())] {
                    let mut h = Harness::new(zoom, height);
                    h.frame(vec![]);
                    h.action(Role::TextInput, c::HISTORY_SEARCH_LABEL, Action::Focus);
                    h.frame(vec![egui::Event::Text(JOB.into())]);
                    h.action(Role::Button, c::LIST_SEARCH, Action::Focus);
                    h.action(Role::Button, c::LIST_SEARCH, Action::Click);
                    let Command::Search(request, None) =
                        h.receive.try_recv().expect("actual search dispatch")
                    else {
                        panic!("read-only search expected")
                    };
                    let original_focus = h.ctx.memory(|memory| memory.focused());
                    assert!(original_focus.is_some());
                    h.send
                        .send(Outcome::Search(
                            request,
                            Ok(vec![JobSummary {
                                id: JOB.into(),
                                state: JobState::Failed,
                                attempts: 1,
                                resumable: true,
                                tool_version: version.clone(),
                            }]),
                        ))
                        .unwrap();
                    h.panel.poll();
                    let mut out = h.frame(vec![]);
                    for _ in 0..32 {
                        out = h.frame(vec![]);
                    }
                    eprintln!(
                        "history-note-inner zoom={zoom} height={height} version={version:?} actual_inner={:?} group={:?} offset={:?}",
                        h.viewport, h.group, h.offset
                    );
                    assert!(
                        h.group.width() <= h.viewport.width()
                            && h.group.height() <= h.viewport.height(),
                        "setup: complete failed/resumable group can fit the actual viewport"
                    );
                    let mut identities = Vec::new();
                    for label in [
                        JOB,
                        c::HISTORY_STATE_FAILED,
                        c::HISTORY_FAILED_NOTE,
                        c::HISTORY_RESUME_INSTRUCTION,
                    ] {
                        let (id, node) = h.exact(Role::Label, label);
                        let bounds = Harness::bounds(node);
                        eprintln!(
                            "history-note-label label={label:?} actual_bounds={bounds:?} viewport={:?}",
                            h.viewport
                        );
                        assert!(
                            !node.supports_action(Action::Focus),
                            "original pure Label remains nonfocusable"
                        );
                        assert!(
                            h.viewport.contains_rect(bounds),
                            "actual original label bounds must be entirely inside inner viewport, not stroke overscan: {label} {bounds:?} {:?}",
                            h.viewport
                        );
                        assert!(out.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text)if text.galley.text()==label&&shape.clip_rect.contains_rect(text.visual_bounding_rect()))),"original label glyphs must be fully painted: {label}");
                        identities.push((label, id));
                    }
                    let (resume, node) = h.exact(Role::Button, c::RESUME);
                    let bounds = Harness::bounds(node);
                    assert!(!node.is_disabled() && node.supports_action(Action::Click));
                    assert!(
                        (bounds.height() - 32.0).abs() < 0.01,
                        "original Resume control remains32logical"
                    );
                    assert!(
                        h.viewport.contains_rect(bounds),
                        "actual complete Resume rect must fit viewport"
                    );
                    assert_eq!(
                        h.ctx.memory(|memory| memory.focused()),
                        original_focus,
                        "publication cannot steal actual search action focus"
                    );
                    let revealed = h.offset.y;
                    let point = h.viewport.center();
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    h.frame(vec![egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, 6.0),
                        modifiers: egui::Modifiers::NONE,
                    }]);
                    let manual = h.offset.y;
                    assert!(
                        manual < revealed - 1.0,
                        "actual wheel must move after one-shot group reveal"
                    );
                    for _ in 0..4 {
                        h.frame(vec![]);
                    }
                    assert!(
                        (h.offset.y - manual).abs() < 0.01,
                        "held search focus cannot repeatedly snap to result"
                    );
                    for (label, id) in identities {
                        assert_eq!(h.exact(Role::Label, label).0, id);
                    }
                    assert_eq!(h.exact(Role::Button, c::RESUME).0, resume);
                    assert!(
                        h.receive.try_recv().is_err(),
                        "geometry/wheel/focus do not dispatch Resume"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod native_history_sequence_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
    use std::collections::{HashMap, HashSet};
    const JOB: &str = "history-failed-12345678-1234-1234";
    struct Harness {
        ctx: egui::Context,
        panel: HistoryPanel,
        receive: mpsc::Receiver<Command>,
        send: mpsc::SyncSender<Outcome>,
        nodes: HashMap<NodeId, Node>,
        root: Option<NodeId>,
        zoom: f32,
        time: f64,
        viewport: egui::Rect,
        group: egui::Rect,
        offset: egui::Vec2,
    }
    impl Harness {
        fn new(zoom: f32) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.enable_accesskit();
            let (commands, receive) = mpsc::sync_channel(1);
            let (send, results) = mpsc::sync_channel(1);
            Self {
                ctx,
                panel: HistoryPanel {
                    rows: Vec::new(),
                    search: Default::default(),
                    error: false,
                    pending: false,
                    pending_search: None,
                    resume: None,
                    commands,
                    results,
                    signals: Arc::new(Signals {
                        stop: AtomicBool::new(false),
                        stopped: AtomicBool::new(false),
                        failed: AtomicBool::new(false),
                        context: Mutex::new(None),
                    }),
                },
                receive,
                send,
                nodes: Default::default(),
                root: None,
                zoom,
                time: 0.0,
                viewport: egui::Rect::NOTHING,
                group: egui::Rect::NOTHING,
                offset: egui::Vec2::ZERO,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += 0.05;
            let out = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0 / self.zoom, 600.0 / self.zoom),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    // Reproduce Main's real panel hierarchy rather than constraining
                    // a child Ui to an observer-derived body height.
                    egui::TopBottomPanel::top("navigation")
                        .frame(
                            egui::Frame::side_top_panel(&ctx.style())
                                .inner_margin(egui::Margin::symmetric(24, 8)),
                        )
                        .show(ctx, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                for label in [c::DOWNLOADS, c::LIBRARY, c::HISTORY, c::SETTINGS] {
                                    let _ = crate::ui_components::navigation_choice(
                                        ui,
                                        label == c::HISTORY,
                                        label,
                                    );
                                }
                            });
                        });
                    egui::TopBottomPanel::bottom("status")
                        .frame(
                            egui::Frame::side_top_panel(&ctx.style())
                                .inner_margin(egui::Margin::symmetric(24, 6)),
                        )
                        .show(ctx, |ui| {
                            ui.style_mut()
                                .text_styles
                                .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
                            ui.spacing_mut().interact_size.y =
                                ui.text_style_height(&egui::TextStyle::Body);
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.horizontal_wrapped(|ui| {
                                ui.label(c::RECOVERY_DONE);
                                ui.separator();
                                let width =
                                    ui.max_rect().intersect(ui.clip_rect()).width().max(1.0);
                                // Fixed synthetic samples reproduce the raw native footer's
                                // width without starting an OS metrics worker.
                                for label in [
                                    "저장 폴더 여유 (D:): 2.1 GiB",
                                    "수신 302.8 KB/s · 송신 502.3 KB/s",
                                ] {
                                    let galley = egui::WidgetText::from(label).into_galley(
                                        ui,
                                        Some(egui::TextWrapMode::Wrap),
                                        width,
                                        egui::TextStyle::Body,
                                    );
                                    ui.add(egui::Label::new(galley));
                                }
                            });
                        });
                    egui::CentralPanel::default()
                        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(24))
                        .show(ctx, |ui| {
                            ui.label(
                                egui::RichText::new(c::HISTORY)
                                    .text_style(egui::TextStyle::Name("screen-title".into())),
                            );
                            ui.add_space(12.0);
                            self.panel.show(ui, true, true);
                        });
                },
            );
            (self.viewport, self.offset) = self
                .ctx
                .data(|data| data.get_temp(egui::Id::new("history-note-reveal-body-test")))
                .unwrap();
            self.group = self
                .ctx
                .data(|data| data.get_temp(egui::Id::new("history-note-reveal-group-test")))
                .unwrap_or(egui::Rect::NOTHING);
            let update = out.platform_output.accesskit_update.as_ref().unwrap();
            if let Some(tree) = &update.tree {
                self.root = Some(tree.root);
            }
            self.nodes.extend(update.nodes.iter().cloned());
            let mut live = HashSet::new();
            let mut pending = vec![self.root.unwrap()];
            while let Some(id) = pending.pop() {
                assert!(live.len() < 256);
                if live.insert(id) {
                    pending.extend(self.nodes[&id].children().iter().copied());
                }
            }
            self.nodes.retain(|id, _| live.contains(id));
            out
        }
        fn exact(&self, role: Role, label: &str) -> (NodeId, &Node) {
            let mut found = self.nodes.iter().filter(|(_, node)| {
                node.role() == role
                    && (node.label() == Some(label)
                        || node.value() == Some(label)
                        || node.labelled_by().iter().any(|id| {
                            self.nodes.get(id).is_some_and(|caption| {
                                caption.label() == Some(label) || caption.value() == Some(label)
                            })
                        }))
            });
            let (id, node) = found.next().expect("actual named node");
            assert!(found.next().is_none(), "unique real node");
            (*id, node)
        }
        fn action(&mut self, role: Role, label: &str, action: Action) {
            let (id, node) = self.exact(role, label);
            assert!(!node.is_disabled());
            assert!(node.supports_action(action));
            self.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target: id,
                data: None,
            })]);
        }
        fn bounds(node: &Node) -> egui::Rect {
            let b = node.bounds().unwrap();
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        }
    }
    #[test]
    fn actual_main_shell_cancelled_id_url_then_failed_id_preserves_complete_note() {
        for zoom in [1.0, 2.0] {
            let mut h = Harness::new(zoom);
            h.panel.rows = vec![
                JobSummary {
                    id: "history-cancelled-12345678-1234-1234".into(),
                    state: JobState::Cancelled,
                    attempts: 1,
                    resumable: false,
                    tool_version: Some("2026.09.24".into()),
                },
                JobSummary {
                    id: JOB.into(),
                    state: JobState::Failed,
                    attempts: 1,
                    resumable: true,
                    tool_version: Some("2026.09.24".into()),
                },
                JobSummary {
                    id: "history-downloaded-12345678-1234-1234".into(),
                    state: JobState::Downloaded,
                    attempts: 1,
                    resumable: false,
                    tool_version: Some("2026.09.24".into()),
                },
            ];
            h.frame(vec![]);
            for (stage, query, state, id) in [
                (
                    "cancelled-id",
                    "history-cancelled-12345678-1234-1234",
                    JobState::Cancelled,
                    "history-cancelled-12345678-1234-1234",
                ),
                (
                    "cancelled-url",
                    "한글-cancelled-target",
                    JobState::Cancelled,
                    "history-cancelled-12345678-1234-1234",
                ),
                ("failed-id", JOB, JobState::Failed, JOB),
            ] {
                h.action(Role::TextInput, c::HISTORY_SEARCH_LABEL, Action::Focus);
                h.frame(vec![egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                }]);
                h.frame(vec![egui::Event::Text(query.into())]);
                h.action(Role::Button, c::LIST_SEARCH, Action::Focus);
                h.action(Role::Button, c::LIST_SEARCH, Action::Click);
                let Command::Search(request, None) = h
                    .receive
                    .try_recv()
                    .expect("actual explicit query dispatch")
                else {
                    panic!("read-only query expected")
                };
                assert_eq!(
                    request.query, query,
                    "real edit replacement and publication query"
                );
                let focus = h.ctx.memory(|m| m.focused());
                h.send
                    .send(Outcome::Search(
                        request,
                        Ok(vec![JobSummary {
                            id: id.into(),
                            state,
                            attempts: 1,
                            resumable: state == JobState::Failed,
                            tool_version: Some("2026.09.24".into()),
                        }]),
                    ))
                    .unwrap();
                h.panel.poll();
                let mut out = h.frame(vec![]);
                for _ in 0..32 {
                    out = h.frame(vec![]);
                }
                let title = Harness::bounds(h.exact(Role::Label, c::HISTORY).1);
                let footer = Harness::bounds(h.exact(Role::Label, c::RECOVERY_DONE).1);
                let metric =
                    Harness::bounds(h.exact(Role::Label, "저장 폴더 여유 (D:): 2.1 GiB").1);
                eprintln!(
                    "history-native-main-fences zoom={zoom} stage={stage} title={title:?} status={footer:?} metric={metric:?} helper_top={} helper_bottom={} actual_inner={:?}",
                    title.bottom() + 12.0,
                    footer.top().min(metric.top()) - 30.0,
                    h.viewport
                );
                eprintln!(
                    "history-native-sequence zoom={zoom} stage={stage} inner={:?} group={:?} offset={:?}",
                    h.viewport, h.group, h.offset
                );
                assert_eq!(h.ctx.memory(|m| m.focused()), focus);
                let labels: Vec<&str> = if state == JobState::Failed {
                    vec![
                        JOB,
                        c::HISTORY_STATE_FAILED,
                        c::HISTORY_FAILED_NOTE,
                        c::HISTORY_RESUME_INSTRUCTION,
                    ]
                } else {
                    vec![id, c::HISTORY_STATE_CANCELLED, c::HISTORY_CANCELLED_NOTE]
                };
                assert!(
                    h.group.height() <= h.viewport.height(),
                    "setup complete group can fit actual Main viewport"
                );
                for label in labels {
                    let (_, node) = h.exact(Role::Label, label);
                    let bounds = Harness::bounds(node);
                    eprintln!(
                        "history-native-label zoom={zoom} stage={stage} label={label:?} bounds={bounds:?} inner={:?}",
                        h.viewport
                    );
                    assert!(!node.supports_action(Action::Focus));
                    assert!(
                        h.viewport.contains_rect(bounds),
                        "original complete label inside actual Main body: {stage} {label} {bounds:?} {:?}",
                        h.viewport
                    );
                    assert!(out.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text)if text.galley.text()==label&&shape.clip_rect.contains_rect(text.visual_bounding_rect()))));
                }
                let (_, resume) = h.exact(Role::Button, c::RESUME);
                let bounds = Harness::bounds(resume);
                assert!((bounds.height() - 32.0).abs() < 0.01);
                assert_eq!(!resume.is_disabled(), state == JobState::Failed);
                assert!(h.viewport.contains_rect(bounds));
                assert!(
                    h.receive.try_recv().is_err(),
                    "no Resume dispatch by passive geometry"
                );
                if state == JobState::Failed {
                    let original_resume = h.exact(Role::Button, c::RESUME).0;
                    let original_label = h.exact(Role::Label, JOB).0;
                    let revealed = h.offset.y;
                    let point = h.viewport.center();
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    h.frame(vec![egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, 6.0),
                        modifiers: egui::Modifiers::NONE,
                    }]);
                    let manual = h.offset.y;
                    let content_size: egui::Vec2 = h
                        .ctx
                        .data(|data| {
                            data.get_temp(egui::Id::new("history-note-reveal-content-size-test"))
                        })
                        .unwrap();
                    let overflow = content_size.y > h.viewport.height();
                    eprintln!(
                        "history-native-manual-wheel zoom={zoom} content={content_size:?} actual_viewport={:?} true_overflow={overflow} revealed={revealed} manual={manual}",
                        h.viewport
                    );
                    if zoom == 2.0 {
                        assert!(
                            overflow,
                            "actual native200 Main fixture must retain real content overflow"
                        );
                    }
                    if overflow {
                        assert!(
                            manual < revealed - 1.0,
                            "actual Main wheel remains usable after one-shot reveal when the real content overflows"
                        );
                    } else {
                        assert!(
                            (manual - revealed).abs() < 0.01,
                            "naturally fitting100 content stays fixed instead of requiring an impossible scroll"
                        );
                    }
                    for _ in 0..6 {
                        h.frame(vec![]);
                    }
                    assert!(
                        (h.offset.y - manual).abs() < 0.01,
                        "same search focus cannot repeatedly snap the actual Main viewport"
                    );
                    assert_eq!(h.exact(Role::Button, c::RESUME).0, original_resume);
                    assert_eq!(h.exact(Role::Label, JOB).0, original_label);
                    assert_eq!(h.ctx.memory(|m| m.focused()), focus);
                    assert!(h.receive.try_recv().is_err());
                }
            }
        }
    }
}
