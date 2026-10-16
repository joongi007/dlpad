#[allow(dead_code)] // Shared with the standalone renderer comparison.
mod catalog;
mod fixture_trace;
mod managed;
mod managed_candidates;

use catalog as c;
use dlpad_core::{
    diagnostics::{DiagnosticReport, FailureStage, ToolComponent},
    options::{
        AdvancedOption, DownloadOptions, Quality, SourceUrl, SubtitleLanguage, SubtitleOptions,
        TimeRange,
    },
};
use dlpad_engine::download::{DownloadError, DownloadRepository, HISTORY_PAGE_SIZE, JobSummary};
use dlpad_engine::metrics::ProgressFeed;
use dlpad_engine::{
    library::LibraryEntry,
    reports::{ReportFormat, demo_report, save_report},
    store::Store,
};
use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Element, Length, Task, Theme};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}
impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::System => c::SYSTEM,
            Self::Light => c::LIGHT,
            Self::Dark => c::DARK,
        })
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ExportFormat {
    #[default]
    Markdown,
    Json,
}
impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Markdown => c::MARKDOWN,
            Self::Json => c::JSON,
        })
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum CutMode {
    #[default]
    Fast,
    Accurate,
}
impl std::fmt::Display for CutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Fast => c::FAST_CUT,
            Self::Accurate => c::ACCURATE_CUT,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Downloads,
    Library,
    History,
    Tools,
    Settings,
    Diagnostics,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum RecoveryState {
    Checking,
    Ready(usize),
    Failed,
}
#[derive(Debug, Clone)]
enum Message {
    FixtureEvent(fixture_trace::EventTag),
    MoveFocus(bool),
    Managed(Box<managed::Message>),
    Url(String),
    Recovered(Result<usize, String>),
    RetryRecovery,
    Ime(String),
    Theme(ThemeChoice),
    Persisted(Result<(), String>),
    ThemeSaved(Result<(), String>),
    Page(Page),
    Scroll(f32),
    Scale(u16),
    Root(String),
    PrepareLocation(bool),
    LocationPrepared(Result<PathBuf, String>),
    Scan(bool),
    Scanned(Result<Vec<LibraryEntry>, String>),
    ExportPath(String),
    SaveReport,
    CopyReport,
    RemoveLogs,
    ExportFormat(ExportFormat),
    FreeSpace(Option<u64>),
    ProgressTick,
    InterfaceInput(String),
    ApplyInterface,
    NetworkSample(u64, Option<dlpad_engine::metrics::NetworkRate>),
    Dirty,
    WatchHealth(dlpad_engine::watch::WatchHealth),
    LogIndex(String),
    EditText(String),
    ApplyEdit,
    OmitLog,
    AttachmentPath(String),
    Attach,
    AttachmentDone((Arc<DiagnosticReport>, Option<String>)),
    Tool(String),
    Ffmpeg(String),
    Deno(String),
    Quality(String),
    PreviewPlan,
    RangeStart(String),
    RangeEnd(String),
    CutMode(CutMode),
    SubtitleLanguage(Option<SubtitleLanguage>),
    SubtitleAuto(bool),
    ToggleAdvanced,
    AdvancedInput(usize, String),
    LoadHistory(bool),
    HistoryLoaded(Result<Vec<JobSummary>, String>),
    Resume(String),
    Start,
    Pause,
    Cancel,
    Finished(WorkerOutcome),
    Close,
    WindowClose(iced::window::Id),
    Background,
    ActivationTick,
    StopExit,
    Return,
}
#[derive(Debug, Clone)]
struct WorkerOutcome {
    status: String,
    report: Option<Arc<DiagnosticReport>>,
}
fn managed_message(message: managed::Message) -> Message {
    Message::Managed(Box::new(message))
}
struct App {
    fixture_trace: Option<fixture_trace::FixtureTrace>,
    activation: Option<Arc<dlpad_engine::activation::ActivationServer>>,
    close_window: Option<iced::window::Id>,
    hidden: bool,
    managed: managed::Panel,
    url: String,
    ime: String,
    theme: ThemeChoice,
    theme_writes: ThemeWrites,
    page: Page,
    first: usize,
    visible: usize,
    fixture: bool,
    scale: u16,
    error: Option<String>,
    root: String,
    entries: Vec<LibraryEntry>,
    history: Vec<JobSummary>,
    history_loading: bool,
    history_error: Option<String>,
    busy: bool,
    report: Option<Arc<DiagnosticReport>>,
    export_path: String,
    export_format: ExportFormat,
    free_space: Option<u64>,
    progress: ProgressFeed,
    interface_input: String,
    interface: String,
    interface_revision: u64,
    interface_error: Option<String>,
    network: Option<dlpad_engine::metrics::NetworkRate>,
    watch_root: String,
    watch_failed: bool,
    pending_refresh: bool,
    log_index: String,
    edit_text: String,
    attachment_path: String,
    tool: String,
    ffmpeg: String,
    deno: String,
    quality: String,
    plan_preview: String,
    range_start: String,
    range_end: String,
    cut_mode: CutMode,
    subtitles: Option<SubtitleOptions>,
    advanced_open: bool,
    advanced_values: [String; 3],
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    running: bool,
    status: String,
    recovery: RecoveryState,
    closing: bool,
    exit_after_stop: bool,
}
impl App {
    fn boot() -> (Self, Task<Message>) {
        let fixture = std::env::args().any(|arg| arg == "--fixture");
        let theme = load_theme().unwrap_or_default();
        let interface = std::env::var("DLPAD_TEST_INTERFACE")
            .ok()
            .filter(|name| dlpad_engine::metrics::valid_interface(name))
            .unwrap_or_default();
        let recovery_data = data_path();
        let mut app = Self {
            fixture_trace: None,
            activation: None,
            close_window: None,
            hidden: false,
            managed: managed::Panel::default(),
            url: String::new(),
            ime: String::new(),
            theme,
            theme_writes: ThemeWrites::default(),
            page: if std::env::args().any(|a| a == "--diagnostics") {
                Page::Diagnostics
            } else if std::env::args().any(|a| a == "--settings") {
                Page::Settings
            } else if std::env::args().any(|arg| arg == "--downloads" || arg == "--advanced") {
                Page::Downloads
            } else if std::env::args().any(|arg| arg == "--tools") {
                Page::Tools
            } else if std::env::args().any(|arg| arg == "--history") {
                Page::History
            } else if fixture || std::env::args().any(|arg| arg == "--library") {
                Page::Library
            } else {
                Page::Downloads
            },
            first: 0,
            visible: 40,
            fixture,
            scale: 100,
            error: None,
            root: std::env::var("DLPAD_LIBRARY_ROOT").unwrap_or_default(),
            entries: vec![],
            history: vec![],
            history_loading: false,
            history_error: None,
            busy: false,
            report: fixture.then(|| Arc::new(demo_report())),
            export_path: String::new(),
            export_format: ExportFormat::default(),
            free_space: None,
            progress: ProgressFeed::default(),
            interface_input: interface.clone(),
            interface,
            interface_revision: 0,
            interface_error: None,
            network: None,
            watch_root: String::new(),
            watch_failed: false,
            pending_refresh: false,
            log_index: "0".into(),
            edit_text: String::new(),
            attachment_path: String::new(),
            tool: std::env::var("DLPAD_YTDLP").unwrap_or_default(),
            ffmpeg: std::env::var("DLPAD_FFMPEG").unwrap_or_default(),
            deno: std::env::var("DLPAD_DENO").unwrap_or_default(),
            quality: "0".into(),
            cut_mode: CutMode::Fast,
            subtitles: None,
            advanced_open: std::env::args().any(|arg| arg == "--advanced"),
            advanced_values: std::array::from_fn(|_| String::new()),
            plan_preview: String::new(),
            range_start: String::new(),
            range_end: String::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            pause: Arc::new(AtomicBool::new(false)),
            running: false,
            status: c::STATUS.into(),
            recovery: if recovery_data.is_some() {
                RecoveryState::Checking
            } else {
                RecoveryState::Ready(0)
            },
            closing: false,
            exit_after_stop: false,
        };
        let task = if !app.fixture {
            app.update(Message::PrepareLocation(false))
        } else {
            Task::none()
        };
        let recovery = recovery_data.map(recovery_task).unwrap_or_else(Task::none);
        let tools = if app.page == Page::Tools {
            app.update(managed_message(managed::Message::Refresh))
        } else {
            Task::none()
        };
        (app, Task::batch([task, recovery, tools]))
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MoveFocus(previous) => {
                return if previous {
                    iced::widget::operation::focus_previous()
                } else {
                    iced::widget::operation::focus_next()
                };
            }
            Message::FixtureEvent(tag) => {
                if let Some(trace) = &mut self.fixture_trace {
                    trace.observe(tag);
                }
            }
            Message::Managed(message) => {
                let message = *message;
                if let managed::Message::Finished(Err(error))
                | managed::Message::UseVerified(Err(error)) = &message
                    && error != c::TOOL_CANCELLED
                {
                    let mut report = DiagnosticReport::new(0, FailureStage::ToolDiscovery, None);
                    report.add_tool(ToolComponent::Dlpad, env!("CARGO_PKG_VERSION"), false);
                    report.add_log(error);
                    self.report = Some(Arc::new(report));
                }
                let verified = match &message {
                    managed::Message::UseVerified(Ok(path)) => Some(path.clone()),
                    _ => None,
                };
                let finished = matches!(
                    message,
                    managed::Message::Finished(_) | managed::Message::UseVerified(_)
                );
                let task = self
                    .managed
                    .update(message, self.running, data_path())
                    .map(managed_message);
                if let Some(path) = verified
                    && !self.running
                {
                    self.tool = path.to_string_lossy().into_owned();
                }
                if finished && self.exit_after_stop && !self.running {
                    return iced::exit();
                }
                return task;
            }
            Message::Recovered(result) => {
                self.recovery = match result {
                    Ok(count) => {
                        if matches!(
                            self.error.as_deref(),
                            Some(c::RECOVERY_CHECKING | c::RECOVERY_FAILED)
                        ) {
                            self.error = None;
                        }
                        RecoveryState::Ready(count)
                    }
                    Err(_) => RecoveryState::Failed,
                };
                if self.page == Page::History {
                    return self.update(Message::LoadHistory(true));
                }
            }
            Message::RetryRecovery => {
                if self.running || self.recovery == RecoveryState::Checking {
                    return Task::none();
                }
                self.recovery = RecoveryState::Checking;
                return data_path()
                    .map(recovery_task)
                    .unwrap_or_else(|| Task::done(Message::Recovered(Ok(0))));
            }
            Message::Url(value) => {
                if !self.running {
                    self.url = value;
                    self.plan_preview.clear();
                }
            }
            Message::Tool(value) => {
                if !self.running && !self.managed.busy {
                    self.tool = value;
                    self.plan_preview.clear();
                }
            }
            Message::Ffmpeg(value) => {
                if !self.running {
                    self.ffmpeg = value.chars().take(32768).collect();
                    self.plan_preview.clear();
                }
            }
            Message::Deno(value) => {
                if !self.running {
                    self.deno = value.chars().take(32768).collect();
                    self.plan_preview.clear();
                }
            }
            Message::Quality(value) => {
                if !self.running {
                    self.quality = value;
                    self.plan_preview.clear();
                }
            }
            Message::PreviewPlan => match self
                .options()
                .and_then(|o| o.plan().map_err(|e| e.0.to_owned()))
            {
                Ok(plan) => {
                    let mut args: Vec<String> = plan
                        .args
                        .iter()
                        .map(|a| a.to_string_lossy().into_owned())
                        .collect();
                    if let Some(url) = args.last_mut() {
                        *url = c::OMITTED_SOURCE.into();
                    }
                    self.plan_preview = serde_json::to_string_pretty(&args).unwrap_or_default();
                }
                Err(error) => self.error = Some(error),
            },
            Message::RangeStart(value) => {
                if !self.running {
                    self.range_start = value;
                    self.plan_preview.clear();
                }
            }
            Message::RangeEnd(value) => {
                if !self.running {
                    self.range_end = value;
                    self.plan_preview.clear();
                }
            }
            Message::CutMode(value) => {
                if !self.running {
                    self.cut_mode = value;
                    self.plan_preview.clear();
                }
            }
            Message::SubtitleLanguage(language) => {
                if !self.running {
                    let auto_generated = self
                        .subtitles
                        .is_some_and(|selected| selected.auto_generated);
                    self.subtitles = language.map(|language| SubtitleOptions {
                        language,
                        auto_generated,
                    });
                    self.plan_preview.clear();
                }
            }
            Message::SubtitleAuto(value) => {
                if !self.running
                    && let Some(selected) = &mut self.subtitles
                {
                    selected.auto_generated = value;
                    self.plan_preview.clear();
                }
            }
            Message::ToggleAdvanced => self.advanced_open = !self.advanced_open,
            Message::AdvancedInput(index, value) => {
                if !self.running
                    && let Some(field) = self.advanced_values.get_mut(index)
                {
                    *field = value.chars().take(64).collect();
                    self.plan_preview.clear();
                }
            }
            Message::Pause => {
                if self.running && !self.cancel.load(Ordering::Acquire) {
                    self.pause.store(true, Ordering::Release);
                    self.cancel.store(true, Ordering::Release);
                }
            }
            Message::Cancel => {
                if self.running && !self.cancel.load(Ordering::Acquire) {
                    self.cancel.store(true, Ordering::Release);
                }
            }
            Message::WindowClose(id) => {
                self.close_window = Some(id);
                return self.update(Message::Close);
            }
            Message::Background => {
                if self.closing
                    && self.activation.is_some()
                    && !self.exit_after_stop
                    && let Some(id) = self.close_window
                {
                    self.hidden = true;
                    self.closing = false;
                    return iced::window::set_mode(id, iced::window::Mode::Hidden);
                }
            }
            Message::ActivationTick => {
                if self
                    .activation
                    .as_ref()
                    .is_some_and(|server| server.poll_restore())
                {
                    self.hidden = false;
                    self.closing = false;
                    if let Some(id) = self.close_window {
                        return iced::window::set_mode(id, iced::window::Mode::Windowed);
                    }
                }
            }
            Message::Return => {
                if !self.exit_after_stop {
                    self.closing = false;
                }
            }
            Message::Close => {
                if self.running || self.managed.busy {
                    self.closing = true;
                } else {
                    return iced::exit();
                }
            }
            Message::StopExit => {
                self.exit_after_stop = true;
                self.cancel.store(true, Ordering::Release);
                return self
                    .managed
                    .update(managed::Message::Cancel, self.running, data_path())
                    .map(managed_message);
            }
            Message::Finished(outcome) => {
                self.running = false;
                self.status = outcome.status;
                self.report = outcome.report;
                if self.exit_after_stop && !self.managed.busy {
                    return iced::exit();
                }
                if self.page == Page::History {
                    return self.update(Message::LoadHistory(true));
                }
            }
            Message::LoadHistory(reset) => {
                if self.history_loading || !matches!(self.recovery, RecoveryState::Ready(_)) {
                    return Task::none();
                }
                let after = if reset {
                    None
                } else {
                    self.history.last().map(|job| job.id.clone())
                };
                self.history_loading = true;
                self.history_error = None;
                return Task::perform(async move { load_history(after) }, Message::HistoryLoaded);
            }
            Message::HistoryLoaded(result) => {
                self.history_loading = false;
                match result {
                    Ok(jobs) => self.history = jobs,
                    Err(_) => self.history_error = Some(c::HISTORY_FAILED.into()),
                }
            }
            Message::Resume(id) => {
                if self.running
                    || !matches!(self.recovery, RecoveryState::Ready(_))
                    || !self.history.iter().any(|job| job.id == id && job.resumable)
                {
                    return Task::none();
                }
                self.cancel = Arc::new(AtomicBool::new(false));
                self.pause = Arc::new(AtomicBool::new(false));
                self.progress = ProgressFeed::default();
                let cancel = self.cancel.clone();
                let pause = self.pause.clone();
                let progress = self.progress.clone();
                let tool = PathBuf::from(&self.tool);
                self.running = true;
                self.status = c::DOWNLOADING.into();
                self.error = None;
                return Task::perform(
                    async move { execute_job(JobRequest::Resume(id), tool, cancel, pause, progress) },
                    Message::Finished,
                );
            }
            Message::Start => {
                if self.managed.busy {
                    self.error = Some(c::TOOL_BUSY.into());
                    return Task::none();
                }
                if self.running {
                    return Task::none();
                }
                if !matches!(self.recovery, RecoveryState::Ready(_)) {
                    self.error = Some(
                        if self.recovery == RecoveryState::Checking {
                            c::RECOVERY_CHECKING
                        } else {
                            c::RECOVERY_FAILED
                        }
                        .into(),
                    );
                    return Task::none();
                }
                let options = self.options();
                let options = match options {
                    Ok(options) => options,
                    Err(error) => {
                        self.error = Some(error);
                        return Task::none();
                    }
                };
                self.error = None;
                self.cancel = Arc::new(AtomicBool::new(false));
                self.pause = Arc::new(AtomicBool::new(false));
                let cancel = self.cancel.clone();
                let pause = self.pause.clone();
                self.progress = ProgressFeed::default();
                let progress = self.progress.clone();
                let tool = PathBuf::from(&self.tool);
                self.running = true;
                self.status = c::DOWNLOADING.into();
                let root = options.output_dir.clone();
                return Task::batch([
                    Task::perform(
                        async move {
                            execute_job(JobRequest::New(options), tool, cancel, pause, progress)
                        },
                        Message::Finished,
                    ),
                    free_space_task(root),
                ]);
            }
            Message::Ime(value) => self.ime = value,
            Message::Page(page) => {
                self.page = page;
                self.first = 0;
                if page == Page::Tools {
                    return self.update(managed_message(managed::Message::Refresh));
                }
                if page == Page::History {
                    return self.update(Message::LoadHistory(true));
                }
                if page == Page::Library && !self.fixture && !self.root.is_empty() {
                    return self.update(Message::Scan(false));
                }
            }
            Message::Scroll(offset) => self.first = (offset / 36.0).floor() as usize,
            Message::Scale(value) => self.scale = value,
            Message::Persisted(result) => {
                self.busy = false;
                self.error = result.err();
            }
            Message::PrepareLocation(default) => {
                if self.busy || self.running {
                    return Task::none();
                }
                self.busy = true;
                let root = self.root.clone();
                return Task::perform(
                    async move { prepare_location(root, default) },
                    Message::LocationPrepared,
                );
            }
            Message::LocationPrepared(result) => {
                self.busy = false;
                match result {
                    Ok(root) => {
                        self.root = root.to_string_lossy().into_owned();
                        self.watch_root.clear();
                        self.watch_failed = false;
                        self.entries.clear();
                        self.pending_refresh = false;
                        self.error = None;
                        return self.update(Message::Scan(false));
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Message::Root(root) => {
                self.root = root;
                self.free_space = None;
                self.watch_root.clear();
                self.watch_failed = false;
                self.entries.clear();
            }
            Message::ExportFormat(format) => self.export_format = format,
            Message::FreeSpace(bytes) => self.free_space = bytes,
            Message::ProgressTick => {}
            Message::InterfaceInput(value) => self.interface_input = value,
            Message::ApplyInterface => {
                if !self.interface_input.is_empty()
                    && !dlpad_engine::metrics::valid_interface(&self.interface_input)
                {
                    self.interface_error = Some(c::INVALID_INTERFACE.into());
                } else {
                    self.interface_error = None;
                    if self.interface == self.interface_input {
                        return Task::none();
                    }
                    self.interface = self.interface_input.clone();
                    self.interface_revision = self.interface_revision.wrapping_add(1);
                    self.network = None;
                }
            }
            Message::NetworkSample(revision, sample) => {
                if revision == self.interface_revision {
                    self.network = sample;
                }
            }
            Message::LogIndex(value) => self.log_index = value,
            Message::EditText(value) => self.edit_text = value,
            Message::AttachmentPath(value) => self.attachment_path = value,
            Message::ApplyEdit | Message::OmitLog => {
                let edit = matches!(message, Message::ApplyEdit);
                let changed = self
                    .log_index
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| {
                        self.report.as_mut().and_then(Arc::get_mut).map(|report| {
                            if edit {
                                report.edit_log(index, &self.edit_text)
                            } else {
                                report.omit_log(index)
                            }
                        })
                    })
                    .unwrap_or(false);
                self.error = (!changed).then(|| c::INVALID_LOG_INDEX.into());
            }
            Message::AttachmentDone((report, error)) => {
                self.busy = false;
                self.report = Some(report);
                self.error = error;
            }
            Message::Attach => {
                if self.busy {
                    return Task::none();
                }
                let Some(report) = self.report.take() else {
                    return Task::none();
                };
                let mut report = match Arc::try_unwrap(report) {
                    Ok(report) => report,
                    Err(report) => {
                        self.report = Some(report);
                        return Task::none();
                    }
                };
                let path = PathBuf::from(&self.attachment_path);
                self.busy = true;
                return Task::perform(
                    async move {
                        let error = dlpad_engine::reports::attach_local_mpd(&mut report, &path)
                            .err()
                            .map(|e| e.to_string());
                        (Arc::new(report), error)
                    },
                    Message::AttachmentDone,
                );
            }
            Message::WatchHealth(health) => {
                self.watch_failed = health == dlpad_engine::watch::WatchHealth::Unavailable
            }
            Message::Dirty => {
                if self.busy {
                    self.pending_refresh = true;
                } else {
                    return self.update(Message::Scan(false));
                }
            }
            Message::ExportPath(path) => self.export_path = path,
            Message::Scanned(result) => {
                self.busy = false;
                match result {
                    Ok(entries) => self.entries = entries,
                    Err(error) => self.error = Some(error),
                }
                if self.pending_refresh {
                    self.pending_refresh = false;
                    return self.update(Message::Scan(false));
                }
            }
            Message::Scan(next) => {
                if self.busy {
                    return Task::none();
                }
                self.busy = true;
                self.watch_root = self.root.clone();
                let root = PathBuf::from(&self.root);
                let after = if next {
                    self.entries.last().map(|e| e.path.clone())
                } else {
                    None
                };
                let space_task = free_space_task(root.clone());
                return Task::batch([
                    Task::perform(async move { scan_page(root, after) }, Message::Scanned),
                    space_task,
                ]);
            }
            Message::CopyReport => {
                if let Some(report) = &self.report {
                    return iced::clipboard::write(self.report_preview(report));
                }
            }
            Message::RemoveLogs => {
                if let Some(report) = self.report.as_mut().and_then(Arc::get_mut) {
                    while report.omit_log(0) {}
                }
            }
            Message::SaveReport => {
                if self.busy {
                    return Task::none();
                }
                let Some(report) = self.report.clone() else {
                    return Task::none();
                };
                let path = PathBuf::from(&self.export_path);
                let format = match self.export_format {
                    ExportFormat::Markdown => ReportFormat::Markdown,
                    ExportFormat::Json => ReportFormat::Json,
                };
                self.busy = true;
                return Task::perform(
                    async move { save_report(&report, &path, format).map_err(|e| e.to_string()) },
                    Message::Persisted,
                );
            }
            Message::Theme(choice) => {
                self.theme = choice;
                if let Some(choice) = self.theme_writes.request(choice) {
                    return Task::perform(
                        async move { persist_theme(choice) },
                        Message::ThemeSaved,
                    );
                }
            }
            Message::ThemeSaved(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                }
                if let Some(choice) = self.theme_writes.finished() {
                    return Task::perform(
                        async move { persist_theme(choice) },
                        Message::ThemeSaved,
                    );
                }
            }
        }
        Task::none()
    }
    fn report_preview(&self, report: &DiagnosticReport) -> String {
        match self.export_format {
            ExportFormat::Markdown => report.markdown_preview(),
            ExportFormat::Json => report
                .json_preview()
                .unwrap_or_else(|_| c::FAILED.to_owned()),
        }
    }
    fn metrics_label(&self) -> String {
        let free = self
            .free_space
            .map(|b| format!("{:.2} GiB", b as f64 / (1024.0 * 1024.0 * 1024.0)))
            .unwrap_or_else(|| c::UNKNOWN.to_owned());
        let sample = self.running.then(|| self.progress.snapshot()).flatten();
        let speed = sample
            .as_ref()
            .and_then(|s| s.bytes_per_second)
            .map(|bytes| format!("{:.1} KiB/s", bytes / 1024.0))
            .unwrap_or_else(|| c::UNKNOWN.into());
        let eta = sample
            .and_then(|s| s.eta_seconds)
            .map(|seconds| format!(" · {} {:.0} {}", c::ETA, seconds, c::SECONDS))
            .unwrap_or_default();
        let processing = self
            .running
            .then(|| self.progress.processing_snapshot())
            .flatten();
        let media = processing
            .filter(|sample| !sample.ended)
            .map(|sample| {
                let speed = sample
                    .speed
                    .map(|speed| format!("{speed:.2}×"))
                    .unwrap_or_else(|| c::UNKNOWN.into());
                let eta = sample
                    .eta_seconds
                    .map(|eta| format!(" · {} {:.0} {}", c::ETA, eta, c::SECONDS))
                    .unwrap_or_default();
                format!("{} {}{}", c::PROCESSING_SPEED, speed, eta)
            })
            .unwrap_or_else(|| c::MEDIA_UNKNOWN.into());
        let network = self
            .network
            .map(|sample| {
                format!(
                    "{} {} · {} {:.1} KiB/s · {} {:.1} KiB/s",
                    c::INTERFACE,
                    self.interface,
                    c::RECEIVED,
                    sample.received_bytes_per_second / 1024.0,
                    c::SENT,
                    sample.sent_bytes_per_second / 1024.0
                )
            })
            .unwrap_or_else(|| c::NETWORK_UNKNOWN.into());
        let watch = if self.watch_failed {
            format!(" · {}", c::WATCH_FAILED)
        } else {
            String::new()
        };
        format!(
            "{} {}{} · {} · {} · {} {}{}",
            c::DOWNLOAD_SPEED,
            speed,
            eta,
            network,
            media,
            c::FREE_SPACE,
            free,
            watch
        )
    }
    fn options(&self) -> Result<DownloadOptions, String> {
        let url = SourceUrl::parse(&self.url).map_err(|e| e.0.to_owned())?;
        let h: u16 = self.quality.parse().map_err(|_| c::QUALITY_NUMBER)?;
        let range = if self.range_start.is_empty() && self.range_end.is_empty() {
            None
        } else {
            Some(
                TimeRange::new(
                    TimeRange::parse_time(&self.range_start).map_err(|e| e.0)?,
                    TimeRange::parse_time(&self.range_end).map_err(|e| e.0)?,
                    self.cut_mode == CutMode::Accurate,
                )
                .map_err(|e| e.0)?,
            )
        };
        let options = DownloadOptions {
            url,
            output_dir: PathBuf::from(&self.root),
            quality: if h == 0 {
                Quality::Best
            } else {
                Quality::Height(h)
            },
            time_range: range,
            ffmpeg: (!self.ffmpeg.is_empty()).then(|| PathBuf::from(&self.ffmpeg)),
            deno: (!self.deno.is_empty()).then(|| PathBuf::from(&self.deno)),
            final_name_style: None,
            subtitles: self.subtitles,
            advanced: self
                .advanced_values
                .iter()
                .enumerate()
                .filter(|(_, value)| !value.is_empty())
                .map(|(index, value)| AdvancedOption {
                    key: dlpad_core::options::CATALOG[index].id.into(),
                    value: value.clone(),
                })
                .collect(),
        };
        options.plan().map_err(|e| e.0)?;
        Ok(options)
    }
    fn recovery_controls(&self) -> Element<'_, Message> {
        match self.recovery {
            RecoveryState::Checking => text(c::RECOVERY_CHECKING).size(13).into(),
            RecoveryState::Failed => column![
                text(c::RECOVERY_FAILED).size(13),
                button(c::RECOVERY_RETRY).on_press(Message::RetryRecovery)
            ]
            .spacing(8)
            .into(),
            RecoveryState::Ready(0) => Space::new().height(0).into(),
            RecoveryState::Ready(count) => text(format!(
                "{}: {} · {}",
                c::RECOVERY_DONE,
                count,
                c::RECOVERY_NO_AUTOSTART
            ))
            .size(13)
            .into(),
        }
    }
    fn cut_control(&self) -> Element<'_, Message> {
        if self.running {
            text(self.cut_mode.to_string()).into()
        } else {
            pick_list(
                [CutMode::Fast, CutMode::Accurate],
                Some(self.cut_mode),
                Message::CutMode,
            )
            .into()
        }
    }
    fn subtitle_controls(&self) -> Element<'_, Message> {
        let help = &dlpad_core::options::SUBTITLE_HELP;
        let mut choices = row![
            button(c::SUBTITLE_NONE)
                .on_press_maybe((!self.running).then_some(Message::SubtitleLanguage(None)))
        ]
        .spacing(8);
        for language in SubtitleLanguage::ALL {
            choices = choices.push(button(language.label()).on_press_maybe(
                (!self.running).then_some(Message::SubtitleLanguage(Some(language))),
            ));
        }
        let selected = self
            .subtitles
            .map_or(c::SUBTITLE_NONE, |value| value.language.label());
        let automatic = self.subtitles.is_some_and(|value| value.auto_generated);
        column![
            text(help.label),
            choices,
            text(format!("{}: {}", c::SUBTITLE_SELECTED, selected)),
            button(if automatic {
                c::SUBTITLE_AUTO_ALLOWED
            } else {
                c::SUBTITLE_AUTO_DENIED
            })
            .on_press_maybe(
                (!self.running && self.subtitles.is_some())
                    .then_some(Message::SubtitleAuto(!automatic))
            ),
            text(help.summary).size(13),
            text(format!(
                "{}: {} · {}: {}",
                c::HELP_DEFAULT,
                help.default,
                c::HELP_EXAMPLE,
                help.example
            ))
            .size(13),
            text(help.tradeoff).size(13),
            text(format!(
                "{}: {} · {}: {}",
                c::HELP_CONFLICT,
                help.conflicts,
                c::HELP_VERSION,
                help.version
            ))
            .size(13)
        ]
        .spacing(8)
        .into()
    }
    fn advanced_controls(&self) -> Element<'_, Message> {
        let mut controls = column![
            button(if self.advanced_open {
                c::ADVANCED_CLOSE
            } else {
                c::ADVANCED_OPEN
            })
            .on_press(Message::ToggleAdvanced)
        ]
        .spacing(8);
        if self.advanced_open {
            controls = controls.push(text(c::ADVANCED_HELP).size(13));
            for (index, help) in dlpad_core::options::CATALOG.iter().enumerate() {
                let value = &self.advanced_values[index];
                let input = text_input(help.example, value)
                    .on_input_maybe(
                        (!self.running)
                            .then_some(move |value| Message::AdvancedInput(index, value)),
                    )
                    .padding(8);
                controls = controls
                    .push(text(help.label))
                    .push(input)
                    .push(text(help.summary).size(13))
                    .push(
                        text(format!(
                            "{}: {} · {}: {}",
                            c::HELP_DEFAULT,
                            help.default,
                            c::HELP_EXAMPLE,
                            help.example
                        ))
                        .size(13),
                    )
                    .push(text(help.tradeoff).size(13))
                    .push(
                        text(format!(
                            "{}: {} · {}: {}",
                            c::HELP_CONFLICT,
                            help.conflicts,
                            c::HELP_VERSION,
                            help.version
                        ))
                        .size(13),
                    );
                if !value.is_empty()
                    && let Err(error) = (AdvancedOption {
                        key: help.id.into(),
                        value: value.clone(),
                    })
                    .validate()
                {
                    controls =
                        controls.push(text(error.0).color(iced::Color::from_rgb8(229, 72, 77)));
                }
            }
        }
        controls.into()
    }
    fn theme(&self) -> Option<Theme> {
        match self.theme {
            ThemeChoice::System => None,
            ThemeChoice::Light => Some(palette(false)),
            ThemeChoice::Dark => Some(palette(true)),
        }
    }
    fn view(&self) -> Element<'_, Message> {
        let nav = row![
            button(c::DOWNLOADS).on_press(Message::Page(Page::Downloads)),
            button(c::LIBRARY).on_press(Message::Page(Page::Library)),
            button(c::HISTORY).on_press(Message::Page(Page::History)),
            button(c::DIAGNOSTICS).on_press(Message::Page(Page::Diagnostics)),
            button(c::TOOLS).on_press(Message::Page(Page::Tools)),
            Space::new().width(Length::Fill),
            button(c::SETTINGS).on_press(Message::Page(Page::Settings))
        ]
        .spacing(8);
        let header = column![text("dlpad").size(22), nav].spacing(12);
        let content: Element<'_, Message> = match self.page {
            Page::Downloads => scrollable(
                column![
                    self.recovery_controls(),
                    text(c::URL).size(16),
                    text_input(c::PLACEHOLDER, &self.url)
                        .on_input_maybe((!self.running).then_some(Message::Url))
                        .padding(10),
                    text(c::SAVE_ROOT),
                    text_input(c::SAVE_ROOT, &self.root)
                        .on_input_maybe((!self.busy && !self.running).then_some(Message::Root))
                        .padding(8),
                    row![
                        button(c::APPLY_ROOT).on_press_maybe(
                            (!self.busy && !self.running)
                                .then_some(Message::PrepareLocation(false))
                        ),
                        button(c::DEFAULT_ROOT).on_press_maybe(
                            (!self.busy && !self.running).then_some(Message::PrepareLocation(true))
                        )
                    ]
                    .spacing(8),
                    text(c::TOOL_PATH),
                    text_input(c::TOOL_PATH, &self.tool)
                        .on_input_maybe(
                            (!self.running && !self.managed.busy).then_some(Message::Tool)
                        )
                        .padding(8),
                    text(c::FFMPEG_PATH),
                    text_input(c::FFMPEG_PATH, &self.ffmpeg)
                        .on_input_maybe((!self.running).then_some(Message::Ffmpeg))
                        .padding(8),
                    text(c::FFMPEG_HELP).size(13),
                    text(c::FFMPEG_DEFAULT_EXAMPLE).size(13),
                    text(c::FFMPEG_CONFLICT_VERSION).size(13),
                    text(c::DENO_PATH),
                    text_input(c::DENO_PATH, &self.deno)
                        .on_input_maybe((!self.running).then_some(Message::Deno))
                        .padding(8),
                    text(c::DENO_HELP).size(13),
                    text(c::DENO_DEFAULT_EXAMPLE).size(13),
                    text(c::DENO_CONFLICT_VERSION).size(13),
                    text(c::QUALITY),
                    text_input(c::QUALITY, &self.quality)
                        .on_input_maybe((!self.running).then_some(Message::Quality))
                        .padding(8),
                    text(c::DEFAULT_QUALITY_HELP).size(13),
                    text(c::DEFAULT_QUALITY_DEFAULT_EXAMPLE).size(13),
                    text(c::DEFAULT_QUALITY_CONFLICT_VERSION).size(13),
                    text(c::RANGE_START),
                    text_input("00:00:00.000", &self.range_start)
                        .on_input_maybe((!self.running).then_some(Message::RangeStart))
                        .padding(8),
                    text(c::RANGE_END),
                    text_input("08:30:00.000", &self.range_end)
                        .on_input_maybe((!self.running).then_some(Message::RangeEnd))
                        .padding(8),
                    text(c::RANGE_HELP).size(13),
                    text(c::RANGE_DEFAULT_EXAMPLE).size(13),
                    text(c::RANGE_CONFLICT_VERSION).size(13),
                    text(c::CUT_MODE),
                    self.cut_control(),
                    text(c::CUT_COST).size(13),
                    text(c::CUT_DEFAULT_EXAMPLE).size(13),
                    text(c::CUT_CONFLICT_VERSION).size(13),
                    self.subtitle_controls(),
                    self.advanced_controls(),
                    row![
                        button(c::START).on_press_maybe(
                            (!self.running
                                && !self.managed.busy
                                && matches!(self.recovery, RecoveryState::Ready(_)))
                            .then_some(Message::Start)
                        ),
                        button(c::PAUSE).on_press_maybe(
                            (self.running && !self.cancel.load(Ordering::Acquire))
                                .then_some(Message::Pause)
                        ),
                        button(c::CANCEL).on_press_maybe(
                            (self.running && !self.cancel.load(Ordering::Acquire))
                                .then_some(Message::Cancel)
                        )
                    ]
                    .spacing(8),
                    text(c::PAUSE_HELP).size(13),
                    text(&self.status),
                    button(c::PLAN).on_press(Message::PreviewPlan),
                    text(&self.plan_preview).size(13)
                ]
                .spacing(8),
            )
            .height(Length::Fill)
            .into(),
            Page::Tools => self.managed.view(self.running).map(managed_message),
            Page::History => {
                let mut list = column![
                    self.recovery_controls(),
                    text(c::HISTORY).size(20),
                    text(c::HISTORY_HELP),
                    row![
                        button(c::HISTORY_REFRESH).on_press_maybe(
                            (!self.history_loading).then_some(Message::LoadHistory(true))
                        ),
                        button(c::HISTORY_NEXT).on_press_maybe(
                            (!self.history_loading && self.history.len() == HISTORY_PAGE_SIZE)
                                .then_some(Message::LoadHistory(false))
                        )
                    ]
                    .spacing(8)
                ]
                .spacing(12);
                if self.history_loading {
                    list = list.push(text(c::HISTORY_LOADING));
                }
                if let Some(error) = &self.history_error {
                    list = list.push(text(error));
                }
                if self.history.is_empty() && !self.history_loading {
                    list = list.push(text(c::HISTORY_EMPTY));
                }
                for job in &self.history {
                    list = list.push(
                        row![
                            text(&job.id).width(Length::Fill),
                            text(job_state_label(job.state)),
                            text(format!("{} {}", c::ATTEMPTS, job.attempts)),
                            button(c::RESUME).on_press_maybe(
                                (job.resumable
                                    && !self.running
                                    && matches!(self.recovery, RecoveryState::Ready(_)))
                                .then(|| Message::Resume(job.id.clone()))
                            )
                        ]
                        .spacing(12),
                    );
                }
                scrollable(list).into()
            }
            Page::Library => {
                if !self.fixture {
                    let mut rows = column![
                        text(c::SAVE_ROOT),
                        text_input(c::SAVE_ROOT, &self.root)
                            .on_input_maybe((!self.busy && !self.running).then_some(Message::Root))
                            .padding(8),
                        row![
                            button(c::APPLY_ROOT).on_press_maybe(
                                (!self.busy && !self.running)
                                    .then_some(Message::PrepareLocation(false))
                            ),
                            button(c::DEFAULT_ROOT).on_press_maybe(
                                (!self.busy && !self.running)
                                    .then_some(Message::PrepareLocation(true))
                            ),
                            button(c::REFRESH)
                                .on_press_maybe((!self.busy).then_some(Message::Scan(false))),
                            button(c::NEXT).on_press_maybe(
                                (!self.busy && self.entries.len() == 128)
                                    .then_some(Message::Scan(true))
                            )
                        ]
                        .spacing(8)
                    ]
                    .spacing(12);
                    if let Some(error) = &self.error {
                        rows = rows.push(text(error));
                    }
                    for entry in &self.entries {
                        rows = rows.push(text(format!(
                            "{} · {} bytes · {}",
                            entry.path.file_name().unwrap_or_default().to_string_lossy(),
                            entry.size,
                            c::LOCAL
                        )));
                    }
                    return column![
                        header,
                        scrollable(rows).height(Length::Fill),
                        text(self.metrics_label()).size(12)
                    ]
                    .spacing(20)
                    .padding(20)
                    .height(Length::Fill)
                    .into();
                }
                let total: usize = 10_000;
                let first = self.first.min(total.saturating_sub(self.visible));
                let end = (first + self.visible).min(total);
                let mut rows = column![Space::new().height(first as f32 * 36.0)].spacing(0);
                for index in first..end {
                    rows = rows.push(
                        container(row![
                            text(format!("{:05}  한글 영상 · 08:30:00", index + 1))
                                .width(Length::Fill),
                            text(c::LOCAL)
                        ])
                        .height(36)
                        .padding(8),
                    );
                }
                rows = rows.push(Space::new().height((total - end) as f32 * 36.0));
                column![
                    text(if self.fixture { c::FIXTURE } else { c::EMPTY }).size(16),
                    scrollable(rows)
                        .height(Length::Fill)
                        .on_scroll(|v| Message::Scroll(v.absolute_offset().y))
                ]
                .spacing(12)
                .height(Length::Fill)
                .into()
            }
            Page::Diagnostics => {
                let preview = self
                    .report
                    .as_ref()
                    .map(|r| self.report_preview(r))
                    .unwrap_or_else(|| c::NO_REPORT.to_owned());
                scrollable(
                    column![
                        text(c::REPORT_REVIEW),
                        pick_list(
                            [ExportFormat::Markdown, ExportFormat::Json],
                            Some(self.export_format),
                            Message::ExportFormat
                        ),
                        row![
                            button(c::COPY_REPORT).on_press(Message::CopyReport),
                            button(c::REMOVE_LOGS).on_press(Message::RemoveLogs)
                        ]
                        .spacing(8),
                        text(c::LOG_INDEX),
                        text_input(c::LOG_INDEX, &self.log_index)
                            .on_input(Message::LogIndex)
                            .padding(8),
                        text(c::EDIT_LOG),
                        text_input(c::EDIT_LOG, &self.edit_text)
                            .on_input(Message::EditText)
                            .padding(8),
                        row![
                            button(c::APPLY_EDIT)
                                .on_press_maybe((!self.busy).then_some(Message::ApplyEdit)),
                            button(c::OMIT_LOG)
                                .on_press_maybe((!self.busy).then_some(Message::OmitLog))
                        ]
                        .spacing(8),
                        text(c::MPD_PATH),
                        text_input(c::MPD_PATH, &self.attachment_path)
                            .on_input(Message::AttachmentPath)
                            .padding(8),
                        button(c::ATTACH_MPD).on_press_maybe(
                            (!self.busy && self.report.is_some()).then_some(Message::Attach)
                        ),
                        text(c::EXPORT_PATH),
                        text_input(c::EXPORT_PATH, &self.export_path)
                            .on_input(Message::ExportPath)
                            .padding(8),
                        button(c::SAVE_REPORT).on_press_maybe(
                            (!self.busy && self.report.is_some()).then_some(Message::SaveReport)
                        ),
                        text(preview).size(13)
                    ]
                    .spacing(12),
                )
                .height(Length::Fill)
                .into()
            }
            Page::Settings => scrollable(
                column![
                    text(c::THEME).size(16),
                    pick_list(
                        [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark],
                        Some(self.theme),
                        Message::Theme
                    ),
                    text(c::SCALE),
                    pick_list([100u16, 150, 200], Some(self.scale), Message::Scale),
                    text(c::IME).size(16),
                    text_input(c::IME_HELP, &self.ime)
                        .on_input(Message::Ime)
                        .padding(10),
                    text(&self.ime),
                    text(c::INTERFACE).size(16),
                    text(c::INTERFACE_HELP).size(13),
                    row![
                        text_input(c::INTERFACE_EXAMPLE, &self.interface_input)
                            .on_input(Message::InterfaceInput)
                            .padding(8),
                        button(c::APPLY_INTERFACE).on_press(Message::ApplyInterface)
                    ]
                    .spacing(8),
                    text(self.interface_error.as_deref().unwrap_or("")).size(13),
                ]
                .spacing(12),
            )
            .height(Length::Fill)
            .into(),
        };
        let mut body = column![
            header,
            container(content).height(Length::Fill),
            text(self.metrics_label()).size(12)
        ]
        .spacing(20)
        .padding(20)
        .height(Length::Fill);
        if let Some(error) = &self.error {
            body = body.push(text(error));
        }
        if self.closing {
            body = body.push(column![
                text(c::CLOSE_ACTIVE),
                text(if self.activation.is_some() {
                    c::BACKGROUND_RESTORE
                } else {
                    c::BACKGROUND_UNAVAILABLE
                })
                .size(12),
                row![
                    button(c::STOP_EXIT)
                        .on_press_maybe((!self.exit_after_stop).then_some(Message::StopExit)),
                    button(c::RETURN)
                        .on_press_maybe((!self.exit_after_stop).then_some(Message::Return)),
                    button(c::BACKGROUND).on_press_maybe(
                        self.activation
                            .as_ref()
                            .filter(|_| !self.exit_after_stop)
                            .map(|_| Message::Background)
                    )
                ]
                .spacing(8)
            ]);
        }
        body.into()
    }
}
fn palette(dark: bool) -> Theme {
    use iced::Color;
    let hex = |r, g, b| Color::from_rgb8(r, g, b);
    Theme::custom(
        if dark { "dlpad Dark" } else { "dlpad Light" },
        iced::theme::Palette {
            background: if dark {
                hex(30, 30, 30)
            } else {
                hex(255, 255, 255)
            },
            text: if dark {
                hex(245, 245, 245)
            } else {
                hex(0, 0, 0)
            },
            primary: hex(0, 111, 190),
            success: hex(34, 197, 94),
            warning: hex(250, 204, 21),
            danger: hex(229, 72, 77),
        },
    )
}
fn read_theme_file(path: PathBuf) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(4097)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() <= 4096).then_some(bytes)
}
fn data_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("DLPAD_TEST_DATA") {
        return Some(path.into());
    }
    if cfg!(test) || std::env::args().any(|arg| arg == "--fixture") {
        return None;
    }
    dlpad_engine::location::default_data_dir().ok()
}
fn prepare_location(root: String, default: bool) -> Result<PathBuf, String> {
    use dlpad_engine::location::{self, LocationRequest};
    let data = data_path().ok_or(c::DATA_REQUIRED)?;
    if (cfg!(test) || std::env::args().any(|arg| arg == "--fixture"))
        && (default || root.is_empty())
        && std::env::var_os("DLPAD_TEST_DOCUMENTS").is_none()
    {
        return Err(c::FIXTURE_DOCUMENTS.into());
    }
    let request = if default {
        LocationRequest::Default
    } else if root.is_empty() {
        LocationRequest::Restore
    } else {
        LocationRequest::Selected(root.into())
    };
    if let Some(documents) = std::env::var_os("DLPAD_TEST_DOCUMENTS") {
        location::prepare_with_default(
            &data,
            request,
            || Ok(PathBuf::from(documents).join("dlpad")),
        )
    } else {
        location::prepare(&data, request)
    }
    .map_err(|error| match error {
        dlpad_engine::library::LibraryError::DocumentsUnavailable => {
            c::DOCUMENTS_UNAVAILABLE.into()
        }
        dlpad_engine::library::LibraryError::UnsupportedPath => c::INVALID_ROOT.into(),
        _ => c::LOCATION_FAILED.into(),
    })
}
fn settings_path() -> Option<PathBuf> {
    // S00 uses an explicit sandbox directory; no Documents or production settings touched.
    data_path().map(|path| path.join("theme.json"))
}
#[derive(Default)]
struct ThemeWrites {
    active: bool,
    pending: Option<ThemeChoice>,
}
impl ThemeWrites {
    fn request(&mut self, choice: ThemeChoice) -> Option<ThemeChoice> {
        if self.active {
            self.pending = Some(choice);
            None
        } else {
            self.active = true;
            Some(choice)
        }
    }
    fn finished(&mut self) -> Option<ThemeChoice> {
        let next = self.pending.take();
        self.active = next.is_some();
        next
    }
}
fn load_theme() -> Option<ThemeChoice> {
    let path = settings_path()?;
    let database = path.with_extension("sqlite");
    if database.exists()
        && let Some(value) = Store::open(&database).ok()?.setting("theme").ok()?
    {
        return (value.len() <= 4096)
            .then(|| serde_json::from_str(&value).ok())
            .flatten();
    }
    read_theme_file(path).and_then(|bytes| serde_json::from_slice(&bytes).ok())
}
fn persist_theme(theme: ThemeChoice) -> Result<(), String> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    let parent = path.parent().ok_or(c::FAILED)?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let value = serde_json::to_string(&theme).map_err(|e| e.to_string())?;
    Store::open(&path.with_extension("sqlite"))
        .and_then(|store| store.set_setting("theme", &value))
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod theme_tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn measured_speed_and_eta_are_hidden_when_job_finishes() {
        let (mut app, _) = App::boot();
        assert!(!app.metrics_label().contains(c::ETA));
        let spec = dlpad_engine::process::ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: "python3".into(),
            args: vec![
                "-c".into(),
                r#"print('DLPAD_PROGRESS:{"speed":4096,"eta":4}')"#.into(),
            ],
            timeout: std::time::Duration::from_secs(5),
            max_log_bytes: 4096,
        };
        dlpad_engine::process::run_with_progress(
            &spec,
            &Arc::new(AtomicBool::new(false)),
            app.progress.clone(),
        )
        .unwrap();
        app.running = true;
        assert!(app.metrics_label().contains("4.0 KiB/s"));
        assert!(app.metrics_label().contains(c::ETA));
        app.running = false;
        assert!(!app.metrics_label().contains("4.0 KiB/s"));
        assert!(!app.metrics_label().contains(c::ETA));
    }
    #[test]
    fn advanced_inputs_reach_plan_and_invalid_values_do_not_start() {
        let (mut app, _) = App::boot();
        app.url = "https://example.org/synthetic".into();
        app.root = "/synthetic-only".into();
        for (index, value) in [(0, "1048576"), (1, "0"), (2, "16")] {
            let _ = app.update(Message::AdvancedInput(index, value.into()));
        }
        let options = app.options().unwrap();
        let args = options.plan().unwrap().args;
        assert_eq!(args.iter().filter(|arg| *arg == "--retries").count(), 1);
        assert_eq!(args[5], "0");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--limit-rate", "1048576"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--concurrent-fragments", "16"])
        );
        let _ = app.update(Message::AdvancedInput(2, "17".into()));
        let _ = app.update(Message::Start);
        assert!(!app.running);
        assert!(app.error.as_deref().unwrap().contains("1~16"));
        let _ = app.update(Message::AdvancedInput(2, "1".into()));
        let _ = app.update(Message::AdvancedInput(0, "9".repeat(10000)));
        assert_eq!(app.advanced_values[0].len(), 64);
        assert!(app.options().is_err());
        for index in 0..3 {
            let _ = app.update(Message::AdvancedInput(index, String::new()));
        }
        assert!(app.options().unwrap().advanced.is_empty());
        assert_eq!(app.options().unwrap().plan().unwrap().args[5], "3");
    }
    #[test]
    fn ffmpeg_selection_reaches_snapshot_and_is_frozen_during_execution() {
        let (mut app, _) = App::boot();
        app.url = "https://example.org/synthetic".into();
        app.root = std::env::temp_dir()
            .join("synthetic-only")
            .to_string_lossy()
            .into_owned();
        let selected = std::env::temp_dir().join("한글 FFmpeg 공백").join("ffmpeg");
        let _ = app.update(Message::Ffmpeg(selected.to_string_lossy().into_owned()));
        assert_eq!(app.options().unwrap().ffmpeg.as_ref(), Some(&selected));
        app.running = true;
        let _ = app.update(Message::Ffmpeg(String::new()));
        assert_eq!(app.options().unwrap().ffmpeg.as_ref(), Some(&selected));
        app.running = false;
        let _ = app.update(Message::Ffmpeg("relative/ffmpeg".into()));
        let _ = app.update(Message::Start);
        assert!(!app.running);
        assert!(app.error.as_deref().unwrap().contains("절대 경로"));
        let _ = app.update(Message::Ffmpeg(String::new()));
        assert!(app.options().unwrap().ffmpeg.is_none());
    }
    #[test]
    fn deno_selection_is_literal_and_frozen_while_running() {
        let (mut app, _) = App::boot();
        app.url = "https://example.org/synthetic".into();
        app.root = std::env::temp_dir()
            .join("synthetic-deno-only")
            .display()
            .to_string();
        app.ffmpeg.clear();
        let selected = std::env::temp_dir().join("한글 Deno 공백").join("deno");
        let _ = app.update(Message::Deno(selected.display().to_string()));
        let options = app.options().unwrap();
        assert_eq!(options.deno.as_ref(), Some(&selected));
        let args = options.plan().unwrap().args;
        let index = args.iter().position(|arg| arg == "--js-runtimes").unwrap();
        assert_eq!(
            args[index + 1],
            std::ffi::OsString::from(format!("deno:{}", selected.display()))
        );
        app.running = true;
        let _ = app.update(Message::Deno(String::new()));
        assert_eq!(app.options().unwrap().deno.as_ref(), Some(&selected));
        app.running = false;
        let _ = app.update(Message::Deno("relative/deno".into()));
        assert!(app.options().is_err());
        let _ = app.update(Message::Deno(String::new()));
        assert!(app.options().unwrap().deno.is_none());
    }
    #[test]
    fn subtitle_selection_reaches_snapshot_conflicts_and_running_changes_are_blocked() {
        let (mut app, _) = App::boot();
        app.url = "https://example.org/synthetic".into();
        app.root = std::env::temp_dir()
            .join("synthetic-subtitle-options")
            .display()
            .to_string();
        let _ = app.update(Message::SubtitleAuto(true));
        assert!(app.subtitles.is_none());
        let _ = app.update(Message::SubtitleLanguage(Some(
            SubtitleLanguage::KoreanAndEnglish,
        )));
        let _ = app.update(Message::SubtitleAuto(true));
        let options = app.options().unwrap();
        assert!(
            options
                .plan()
                .unwrap()
                .args
                .iter()
                .any(|arg| arg == "--write-auto-subs")
        );
        assert_eq!(
            DownloadOptions::from_stored_json(&options.to_stored_json().unwrap())
                .unwrap()
                .subtitles,
            app.subtitles
        );
        app.running = true;
        let _ = app.update(Message::SubtitleLanguage(None));
        let _ = app.update(Message::SubtitleAuto(false));
        assert_eq!(app.subtitles, options.subtitles);
        app.running = false;
        let _ = app.update(Message::RangeStart("0".into()));
        let _ = app.update(Message::RangeEnd("1".into()));
        assert!(app.options().is_err());
        let _ = app.update(Message::SubtitleLanguage(None));
        assert!(app.options().unwrap().plan().is_ok());
    }
    #[test]
    fn unavailable_restore_never_hides_and_return_preserves_worker() {
        let (mut app, _) = App::boot();
        app.running = true;
        let id = iced::window::Id::unique();
        let _ = app.update(Message::WindowClose(id));
        assert!(app.closing);
        let _ = app.update(Message::Background);
        assert!(!app.hidden);
        assert!(app.closing);
        assert!(!app.cancel.load(Ordering::Acquire));
        let _ = app.update(Message::Return);
        assert!(!app.closing);
        assert!(app.running);
        assert!(!app.cancel.load(Ordering::Acquire));
    }
    #[test]
    fn accurate_cut_is_explicit_and_running_settings_are_frozen() {
        let (mut app, _) = App::boot();
        app.url = "https://example.org/synthetic".into();
        app.root = "/synthetic-only".into();
        let _ = app.update(Message::RangeStart("0.200".into()));
        let _ = app.update(Message::RangeEnd("1.200".into()));
        let _ = app.update(Message::CutMode(CutMode::Accurate));
        assert!(
            app.options()
                .unwrap()
                .plan()
                .unwrap()
                .args
                .iter()
                .any(|arg| arg == "--force-keyframes-at-cuts")
        );
        app.running = true;
        let _ = app.update(Message::CutMode(CutMode::Fast));
        let _ = app.update(Message::AdvancedInput(1, "20".into()));
        let _ = app.update(Message::Url("https://changed.invalid".into()));
        assert_eq!(app.cut_mode, CutMode::Accurate);
        assert!(app.advanced_values[1].is_empty());
        assert_eq!(app.url, "https://example.org/synthetic");
        app.running = false;
        let _ = app.update(Message::RangeStart(String::new()));
        let _ = app.update(Message::RangeEnd(String::new()));
        assert!(
            !app.options()
                .unwrap()
                .plan()
                .unwrap()
                .args
                .iter()
                .any(|arg| arg == "--force-keyframes-at-cuts")
        );
    }
    #[test]
    fn managed_install_blocks_download_and_close_waits_for_owned_operation() {
        let (mut app, _) = App::boot();
        app.managed.busy = true;
        let _ = app.update(Message::Start);
        assert!(!app.running);
        assert_eq!(app.error.as_deref(), Some(c::TOOL_BUSY));
        let previous = app.tool.clone();
        let _ = app.update(Message::Tool("changed".into()));
        assert_eq!(app.tool, previous);
        let _ = app.update(Message::Close);
        assert!(app.closing);
        let _ = app.update(Message::StopExit);
        assert!(app.exit_after_stop);
        assert!(app.managed.busy);
        let _ = app.update(managed_message(managed::Message::Finished(Err(
            c::TOOL_FAILED.into(),
        ))));
        assert!(!app.managed.busy);
    }
    #[test]
    fn pause_and_cancel_intents_are_distinct_and_frozen_after_request() {
        let (mut app, _) = App::boot();
        let _ = app.update(Message::Pause);
        assert!(!app.pause.load(Ordering::Acquire));
        app.running = true;
        let _ = app.update(Message::Pause);
        assert!(app.pause.load(Ordering::Acquire));
        assert!(app.cancel.load(Ordering::Acquire));
        let _ = app.update(Message::Cancel);
        assert!(app.pause.load(Ordering::Acquire));
        app.cancel = Arc::new(AtomicBool::new(false));
        app.pause = Arc::new(AtomicBool::new(false));
        let _ = app.update(Message::Cancel);
        assert!(!app.pause.load(Ordering::Acquire));
        let _ = app.update(Message::Pause);
        assert!(!app.pause.load(Ordering::Acquire));
    }
    #[test]
    fn history_resume_requires_valid_row_and_ready_idle_state() {
        let (mut app, _) = App::boot();
        app.history = vec![JobSummary {
            id: "fixture".into(),
            state: dlpad_core::jobs::JobState::Interrupted,
            attempts: 1,
            resumable: true,
            tool_version: None,
        }];
        app.recovery = RecoveryState::Checking;
        let _ = app.update(Message::Resume("fixture".into()));
        assert!(!app.running);
        app.recovery = RecoveryState::Ready(1);
        let _ = app.update(Message::Resume("unknown".into()));
        assert!(!app.running);
        app.history[0].resumable = false;
        let _ = app.update(Message::Resume("fixture".into()));
        assert!(!app.running);
        app.history[0].resumable = true;
        let _ = app.update(Message::Resume("fixture".into()));
        assert!(app.running);
        let previous = app.cancel.clone();
        let _ = app.update(Message::Resume("fixture".into()));
        assert!(Arc::ptr_eq(&previous, &app.cancel));
        app.page = Page::History;
        let _ = app.update(Message::Finished(WorkerOutcome {
            status: c::DOWNLOADED.into(),
            report: None,
        }));
        assert!(!app.running);
        assert!(app.history_loading);
    }
    #[test]
    fn startup_recovery_gate_and_notice_are_independent_of_active_job() {
        let (mut app, _) = App::boot();
        app.recovery = RecoveryState::Checking;
        let _ = app.update(Message::Start);
        assert!(!app.running);
        assert_eq!(app.error.as_deref(), Some(c::RECOVERY_CHECKING));
        let _ = app.update(Message::Recovered(Err("synthetic failure".into())));
        let _ = app.update(Message::Start);
        assert!(!app.running);
        assert_eq!(app.error.as_deref(), Some(c::RECOVERY_FAILED));
        let _ = app.update(Message::RetryRecovery);
        assert_eq!(app.recovery, RecoveryState::Checking);
        app.status = "current worker status".into();
        let _ = app.update(Message::Recovered(Ok(2)));
        assert_eq!(app.recovery, RecoveryState::Ready(2));
        assert!(!app.running);
        assert_eq!(app.status, "current worker status");
        assert!(app.error.is_none());
    }
    #[test]
    fn interface_selection_rejects_invalid_and_obsolete_samples() {
        let (mut app, _) = App::boot();
        let _ = app.update(Message::InterfaceInput("../invalid".into()));
        let _ = app.update(Message::ApplyInterface);
        assert!(app.interface_error.is_some());
        assert!(app.interface.is_empty());
        let _ = app.update(Message::InterfaceInput("lo".into()));
        let _ = app.update(Message::ApplyInterface);
        let rate = dlpad_engine::metrics::NetworkRate {
            received_bytes_per_second: 0.0,
            sent_bytes_per_second: 1.0,
        };
        let _ = app.update(Message::NetworkSample(app.interface_revision, Some(rate)));
        assert_eq!(app.network, Some(rate));
        let old = app.interface_revision;
        let _ = app.update(Message::InterfaceInput(String::new()));
        let _ = app.update(Message::ApplyInterface);
        let _ = app.update(Message::NetworkSample(old, Some(rate)));
        assert!(app.network.is_none());
        assert!(app.interface_error.is_none());
    }
    #[test]
    fn rapid_changes_keep_latest_selection() {
        let mut writes = ThemeWrites::default();
        assert_eq!(writes.request(ThemeChoice::Light), Some(ThemeChoice::Light));
        assert_eq!(writes.request(ThemeChoice::Dark), None);
        assert_eq!(writes.request(ThemeChoice::System), None);
        assert_eq!(writes.finished(), Some(ThemeChoice::System));
        assert_eq!(writes.finished(), None);
        assert_eq!(writes.request(ThemeChoice::Dark), Some(ThemeChoice::Dark));
    }
    #[test]
    fn failed_theme_write_preserves_other_operation_and_drains_pending() {
        let (mut app, _) = App::boot();
        app.busy = true;
        app.theme_writes.request(ThemeChoice::Light);
        app.theme_writes.request(ThemeChoice::Dark);
        let _ = app.update(Message::ThemeSaved(Err("fixture failure".into())));
        assert!(app.busy);
        assert!(app.theme_writes.active);
        assert!(app.theme_writes.pending.is_none());
        assert_eq!(app.error.as_deref(), Some("fixture failure"));
    }
}
#[derive(Clone, Hash)]
struct NetworkSubscription {
    interface: String,
    revision: u64,
}
fn network_subscription(
    input: &NetworkSubscription,
) -> impl iced::futures::Stream<Item = Message> + use<> {
    let input = input.clone();
    iced::stream::channel(1, async move |mut output| {
        if input.interface.is_empty() {
            return;
        }
        std::thread::spawn(move || {
            let mut previous = None;
            while !output.is_closed() {
                let current = dlpad_engine::metrics::sample_network_interface(&input.interface)
                    .ok()
                    .flatten();
                let rate = current
                    .as_ref()
                    .zip(previous.as_ref())
                    .and_then(|(current, previous)| current.rate_since(previous));
                previous = current;
                let _ = output.try_send(Message::NetworkSample(input.revision, rate));
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    })
}
fn activation_subscription(enabled: &bool) -> impl iced::futures::Stream<Item = Message> + use<> {
    let enabled = *enabled;
    iced::stream::channel(1, async move |mut output| {
        if enabled {
            std::thread::spawn(move || {
                while !output.is_closed() {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    let _ = output.try_send(Message::ActivationTick);
                }
            });
        }
    })
}
#[derive(Clone, Hash)]
struct ProgressSubscription {
    feed: ProgressFeed,
    active: bool,
}
fn progress_subscription(
    input: &ProgressSubscription,
) -> impl iced::futures::Stream<Item = Message> + use<> {
    let active = input.active;
    iced::stream::channel(1, async move |mut output| {
        if !active {
            return;
        }
        std::thread::spawn(move || {
            while !output.is_closed() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let _ = output.try_send(Message::ProgressTick);
            }
        });
    })
}
#[derive(Clone, Hash)]
struct WatchRoot(String);
fn watch_subscription(root: &WatchRoot) -> impl iced::futures::Stream<Item = Message> + use<> {
    let root = root.0.clone();
    iced::stream::channel(1, async move |mut output| {
        if root.is_empty() {
            return;
        }
        std::thread::spawn(move || {
            let mut session = dlpad_engine::watch::WatchSession::new(PathBuf::from(root));
            let mut dirty = false;
            let mut health = None;
            while !output.is_closed() {
                std::thread::sleep(std::time::Duration::from_millis(250));
                let sample = session.poll();
                dirty |= sample.dirty;
                if sample.health.is_some() {
                    health = sample.health;
                }
                if let Some(value) = health
                    && output.try_send(Message::WatchHealth(value)).is_ok()
                {
                    health = None;
                }
                if dirty && output.try_send(Message::Dirty).is_ok() {
                    dirty = false;
                }
            }
        });
    })
}
fn free_space_task(root: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            dlpad_engine::metrics::volume_free_bytes(&root)
                .ok()
                .flatten()
        },
        Message::FreeSpace,
    )
}
fn scan_page(root: PathBuf, after: Option<PathBuf>) -> Result<Vec<LibraryEntry>, String> {
    if root.as_os_str().is_empty() {
        return Err(c::INVALID_ROOT.into());
    }
    let data = data_path().ok_or(c::DATA_REQUIRED)?;
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let actual_root = root.canonicalize().map_err(|e| e.to_string())?;
    let actual_data = data.canonicalize().map_err(|e| e.to_string())?;
    if actual_root.starts_with(&actual_data) || actual_data.starts_with(&actual_root) {
        return Err(c::SEPARATE_DATA.into());
    }
    let mut store = Store::open(&data.join("library.sqlite")).map_err(|e| e.to_string())?;
    store.reconcile_library(&root).map_err(|e| e.to_string())?;
    store
        .library_page(&root, after.as_deref(), 128)
        .map_err(|e| e.to_string())
}
fn recovery_task(data: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            dlpad_engine::download::recover_at_startup(&data)
                .map_err(|_| c::RECOVERY_FAILED.to_owned())
        },
        Message::Recovered,
    )
}
enum JobRequest {
    New(DownloadOptions),
    Resume(String),
}
fn job_state_label(state: dlpad_core::jobs::JobState) -> &'static str {
    use dlpad_core::jobs::JobState::*;
    match state {
        Downloaded => c::DOWNLOADED,
        Failed => c::FAILED,
        Cancelled => c::CANCELLED,
        Interrupted => c::STATE_INTERRUPTED,
        Paused => c::STATE_PAUSED,
        Queued => c::STATE_QUEUED,
        Preparing => c::STATE_PREPARING,
        Downloading => c::DOWNLOADING,
        PostProcessing => c::STATE_POSTPROCESSING,
        Verifying => c::STATE_VERIFYING,
        Stopping => c::STATE_STOPPING,
        RetryWaiting => c::STATE_RETRY,
    }
}
fn load_history(after: Option<String>) -> Result<Vec<JobSummary>, String> {
    let data = data_path().ok_or(c::DATA_REQUIRED)?;
    let path = data.join("jobs.sqlite");
    if !path.exists() {
        return Ok(vec![]);
    }
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| c::HISTORY_FAILED)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(c::HISTORY_FAILED.into());
    }
    DownloadRepository::open(&path)
        .and_then(|repo| repo.history(after.as_deref()))
        .map_err(|_| c::HISTORY_FAILED.into())
}
fn execute_job(
    request: JobRequest,
    tool: PathBuf,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    progress: ProgressFeed,
) -> WorkerOutcome {
    let mut failed_version = None;
    let mut failed_ffmpeg_version = None;
    let mut deno_version = None;
    let mut attempt_number = 0;
    let run = || -> Result<_, DownloadError> {
        let data = data_path().ok_or(DownloadError::Policy(c::DATA_REQUIRED))?;
        let resume_id = match &request {
            JobRequest::Resume(id) => Some(id.clone()),
            _ => None,
        };
        std::fs::create_dir_all(&data)?;
        let repository = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let mut options = match request {
            JobRequest::New(options) => options,
            JobRequest::Resume(ref id) => repository.stored_options(id)?,
        };
        options.output_dir = dlpad_engine::location::prepare(
            &data,
            dlpad_engine::location::LocationRequest::Selected(options.output_dir.clone()),
        )
        .map_err(|_| DownloadError::Policy(c::INVALID_ROOT))?;
        let id = resume_id.clone().unwrap_or_else(|| {
            format!(
                "job-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            )
        });
        if resume_id.is_none() {
            repository.create_job(&id, &options)?;
        }
        let result = if resume_id.is_some() {
            repository.resume_with_control(&id, &tool, &cancel, &pause, progress.clone())
        } else {
            repository.execute_with_control(&id, &options, &tool, &cancel, &pause, progress.clone())
        };
        failed_version = repository.tool_version(&id).ok().flatten();
        failed_ffmpeg_version = repository.ffmpeg_version(&id).ok().flatten();
        deno_version = repository.deno_version(&id).ok().flatten();
        attempt_number = repository
            .job(&id)
            .ok()
            .and_then(|j| j.attempts.last().map(|a| a.number))
            .unwrap_or(0);
        result
    };
    match run() {
        Ok(result) => WorkerOutcome {
            status: format!(
                "{} · {}",
                c::DOWNLOADED,
                result.job.artifacts[0].path.display()
            ),
            report: None,
        },
        Err(DownloadError::Execution { result, .. })
            if result.outcome == dlpad_engine::process::ProcessOutcome::Cancelled =>
        {
            WorkerOutcome {
                status: if pause.load(Ordering::Acquire) {
                    c::STATE_PAUSED
                } else {
                    c::CANCELLED
                }
                .into(),
                report: None,
            }
        }
        Err(error) => {
            let mut report =
                DiagnosticReport::new(u64::from(attempt_number), FailureStage::Unknown, None);
            report.add_tool(ToolComponent::Dlpad, env!("CARGO_PKG_VERSION"), false);
            if let Some(version) = failed_version {
                report.add_tool(ToolComponent::YtDlp, &version, false);
            }
            if let Some(version) = failed_ffmpeg_version {
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
            WorkerOutcome {
                status: c::FAILED.into(),
                report: Some(Arc::new(report)),
            }
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlpad_engine::process::guardian_entry()?;
    dlpad_engine::process::register_self_guardian()?;
    let _instance = match data_path() {
        Some(root) => match dlpad_engine::instance::InstanceGuard::acquire(&root) {
            Ok(guard) => Some(guard),
            Err(dlpad_engine::instance::InstanceError::AlreadyRunning) => {
                dlpad_engine::activation::send_restore(&root)?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        },
        None => None,
    };
    let activation = _instance.as_ref().and_then(|guard| {
        dlpad_engine::activation::ActivationServer::bind(guard.root())
            .ok()
            .map(Arc::new)
    });
    let fixture_trace_root = if std::env::args().any(|arg| arg == "--fixture")
        && std::env::args().any(|arg| arg == "--fixture-events")
    {
        _instance.as_ref().map(|guard| guard.root().to_path_buf())
    } else {
        None
    };
    let mut app = iced::application(
        move || {
            let (mut app, task) = App::boot();
            app.activation = activation.clone();
            app.fixture_trace = fixture_trace_root
                .clone()
                .map(fixture_trace::FixtureTrace::start);
            (app, task)
        },
        App::update,
        App::view,
    )
    .title(c::TITLE)
    .theme(App::theme)
    .scale_factor(|app: &App| f32::from(app.scale) / 100.0)
    .window_size((960.0, 640.0))
    .centered()
    .exit_on_close_request(false)
    .subscription(|app: &App| {
        iced::Subscription::batch([
            iced::keyboard::listen().filter_map(|event| {
                if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event
                    && key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab)
                    && !modifiers.control()
                    && !modifiers.alt()
                    && !modifiers.logo()
                {
                    Some(Message::MoveFocus(modifiers.shift()))
                } else {
                    None
                }
            }),
            if app.fixture_trace.is_some() {
                iced::event::listen_with(|event, status, _| {
                    fixture_trace::classify(event, status).map(Message::FixtureEvent)
                })
            } else {
                iced::Subscription::none()
            },
            iced::window::close_requests().map(Message::WindowClose),
            iced::Subscription::run_with(app.activation.is_some(), activation_subscription),
            iced::Subscription::run_with(
                NetworkSubscription {
                    interface: app.interface.clone(),
                    revision: app.interface_revision,
                },
                network_subscription,
            ),
            iced::Subscription::run_with(
                ProgressSubscription {
                    feed: app.progress.clone(),
                    active: app.running,
                },
                progress_subscription,
            ),
            iced::Subscription::run_with(WatchRoot(app.watch_root.clone()), watch_subscription),
        ])
    });
    if let Some(path) = std::env::var_os("DLPAD_TEST_FONT")
        && std::fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() <= 32 * 1024 * 1024)
        && let Ok(font) = std::fs::read(path)
    {
        app = app.font(font);
    }
    app.run().map_err(Into::into)
}
