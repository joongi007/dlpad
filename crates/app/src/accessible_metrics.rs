//! Latest-only volume and selected-interface observations; no UI-thread I/O.
use crate::catalog as c;
#[cfg(any(test, all(windows, feature = "first-paint-fixture")))]
#[path = "metrics_worker_phase_probe.rs"]
mod worker_phase_probe;
use dlpad_engine::metrics::{self, NetworkRate};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};
const INTERVAL: Duration = Duration::from_secs(1);
const STALE: Duration = Duration::from_secs(3);
#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkMode {
    Auto,
    Manual,
    Off,
}
impl Default for NetworkMode {
    fn default() -> Self {
        if cfg!(windows) { Self::Auto } else { Self::Off }
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum NetworkState {
    #[default]
    Unavailable,
    Off,
    NoRoute,
    Failed,
    Observing,
}
#[derive(Clone, Default)]
struct NetworkView {
    state: NetworkState,
    alias: Option<String>,
    #[cfg(windows)]
    aliases: Arc<Vec<String>>,
    list_generation: u64,
    list_failed: bool,
}
#[derive(Default)]
struct Discovery {
    #[cfg(windows)]
    initialized: bool,
    #[cfg(windows)]
    refresh: u64,
    #[cfg(windows)]
    aliases: Arc<Vec<String>>,
    #[cfg(windows)]
    list_generation: u64,
    #[cfg(windows)]
    list_failed: bool,
}
impl Discovery {
    fn view(&mut self, selection: &Selection, io: &mut impl MetricsIo) -> NetworkView {
        #[cfg(windows)]
        {
            if selection.mode != NetworkMode::Off
                && (!self.initialized || self.refresh != selection.refresh)
            {
                self.initialized = true;
                self.refresh = selection.refresh;
                match io.aliases() {
                    Ok(names)
                        if names.len() <= 256
                            && names.iter().all(|s| metrics::valid_observed_interface(s)) =>
                    {
                        if self.aliases.as_ref() != &names {
                            self.aliases = Arc::new(names);
                            self.list_generation = self.list_generation.wrapping_add(1);
                        }
                        self.list_failed = false;
                    }
                    _ => {
                        self.list_failed = true;
                    }
                }
            }
            NetworkView {
                aliases: self.aliases.clone(),
                list_generation: self.list_generation,
                list_failed: self.list_failed,
                ..Default::default()
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (selection, io);
            NetworkView::default()
        }
    }
}
#[derive(Clone, Default)]
struct Selection {
    revision: u64,
    root: Option<PathBuf>,
    interface: String,
    mode: NetworkMode,
    #[cfg(windows)]
    refresh: u64,
}
struct Command {
    selection: Selection,
    context: egui::Context,
}
enum WorkerCommand {
    Selection(Command),
    StopWake,
}
fn stop_worker(signals: &Signals, commands: &SyncSender<WorkerCommand>) {
    signals.stop.store(true, Ordering::Release);
    // Full means the accepted selection already wakes recv; never displace it.
    let _ = commands.try_send(WorkerCommand::StopWake);
}
#[derive(Clone)]
struct Sample {
    revision: u64,
    at: Instant,
    free: Option<u64>,
    rate: Option<NetworkRate>,
    observation_failed: (bool, bool),
    network_view: NetworkView,
}
#[derive(Clone, PartialEq, Eq)]
struct DisplayKey {
    revision: u64,
    free: Option<String>,
    rate: Option<(String, String)>,
    observation_failed: (bool, bool),
    network_state: NetworkState,
    selected_alias: Option<String>,
    list_generation: u64,
    list_failed: bool,
}
fn visible_rate(sample: &Sample) -> Option<NetworkRate> {
    if sample.observation_failed.1
        || matches!(
            sample.network_view.state,
            NetworkState::Off | NetworkState::NoRoute | NetworkState::Failed
        )
    {
        None
    } else {
        sample.rate
    }
}
fn display_key(sample: &Sample) -> DisplayKey {
    DisplayKey {
        revision: sample.revision,
        free: sample
            .free
            .map(|bytes| format!("{:.1}", bytes as f64 / 1073741824.0)),
        rate: visible_rate(sample).map(|rate| {
            (
                crate::display_rate::format_bytes_per_second(Some(rate.received_bytes_per_second)),
                crate::display_rate::format_bytes_per_second(Some(rate.sent_bytes_per_second)),
            )
        }),
        observation_failed: sample.observation_failed,
        network_state: sample.network_view.state,
        selected_alias: sample.network_view.alias.clone(),
        list_generation: sample.network_view.list_generation,
        list_failed: sample.network_view.list_failed,
    }
}
#[derive(Clone, PartialEq, Eq)]
enum VisibleKey {
    Fresh(DisplayKey),
    Unknown { revision: u64, failed: bool },
}
#[derive(Default)]
struct Mailbox {
    latest: Option<Sample>,
    key: Option<DisplayKey>,
}
impl Mailbox {
    fn publish(&mut self, sample: Sample) -> bool {
        let key = display_key(&sample);
        let wake = self.key.as_ref() != Some(&key);
        self.latest = Some(sample);
        self.key = Some(key);
        wake
    }
}
#[derive(Default)]
struct Signals {
    stop: AtomicBool,
    stopped: AtomicBool,
    failed: AtomicBool,
    context: Mutex<Option<egui::Context>>,
}
struct Ack(Arc<Signals>);
impl Drop for Ack {
    fn drop(&mut self) {
        self.0.stopped.store(true, Ordering::Release);
        if let Ok(context) = self.0.context.lock()
            && let Some(context) = context.as_ref()
        {
            context.request_repaint();
        }
    }
}
fn supervise(signals: Arc<Signals>, run: impl FnOnce()) {
    let _ack = Ack(signals.clone());
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).is_err() {
        signals.failed.store(true, Ordering::Release);
    }
}
fn path_bytes(path: &std::path::Path) -> usize {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().len()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str().encode_wide().count().saturating_mul(2)
    }
    #[cfg(not(any(unix, windows)))]
    {
        path.as_os_str().len()
    }
}
// Private worker adapter seam; fixtures exercise the production observation logic.
trait MetricsIo {
    fn volume(&mut self, root: &std::path::Path) -> std::io::Result<Option<u64>>;
    #[cfg(windows)]
    fn aliases(&mut self) -> std::io::Result<Vec<String>>;
    fn automatic(&mut self) -> std::io::Result<Option<String>>;
    fn sample(&mut self, alias: &str) -> std::io::Result<Option<metrics::NetworkSample>>;
    fn reset(&mut self);
}
struct NativeMetrics<'a>(&'a mut metrics::NetworkObserver);
impl MetricsIo for NativeMetrics<'_> {
    fn volume(&mut self, root: &std::path::Path) -> std::io::Result<Option<u64>> {
        metrics::volume_free_bytes(root)
    }
    #[cfg(windows)]
    fn aliases(&mut self) -> std::io::Result<Vec<String>> {
        metrics::observed_interfaces()
    }
    fn automatic(&mut self) -> std::io::Result<Option<String>> {
        #[cfg(windows)]
        {
            metrics::automatic_observed_interface()
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    fn sample(&mut self, alias: &str) -> std::io::Result<Option<metrics::NetworkSample>> {
        self.0.sample(alias)
    }
    fn reset(&mut self) {
        self.0.reset();
    }
}
fn observe_selection(
    selection: &Selection,
    previous: &mut Option<metrics::NetworkSample>,
    _sample_due: Instant,
    network: &mut metrics::NetworkObserver,
    discovery: &mut Discovery,
) -> Sample {
    observe_selection_with_io(selection, previous, &mut NativeMetrics(network), discovery)
}
fn observe_selection_with_io(
    selection: &Selection,
    previous: &mut Option<metrics::NetworkSample>,
    io: &mut impl MetricsIo,
    discovery: &mut Discovery,
) -> Sample {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    let free_result = selection.root.as_ref().map(|p| {
        worker_phase_probe::measured(
            worker_phase_probe::Phase::Volume,
            || io.volume(p),
            |r| match r {
                Ok(Some(_)) => 0,
                Ok(None) => 1,
                Err(_) => 2,
            },
        )
    });
    #[cfg(not(all(windows, feature = "first-paint-fixture")))]
    let free_result = selection.root.as_ref().map(|p| io.volume(p));
    let free_failed = free_result.as_ref().is_some_and(Result::is_err);
    let free = free_result.and_then(Result::ok).flatten();
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    let cached_discovery = discovery.initialized && discovery.refresh == selection.refresh;
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    let mut network_view = worker_phase_probe::measured(
        worker_phase_probe::Phase::Discovery,
        || discovery.view(selection, io),
        |view| {
            if selection.mode == NetworkMode::Off {
                1
            } else if view.list_failed {
                2
            } else if cached_discovery {
                3
            } else {
                0
            }
        },
    );
    #[cfg(not(all(windows, feature = "first-paint-fixture")))]
    let mut network_view = discovery.view(selection, io);
    let selected = match selection.mode {
        NetworkMode::Off => Ok(None),
        NetworkMode::Manual => Ok(Some(selection.interface.clone())),
        NetworkMode::Auto => {
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            let selected = worker_phase_probe::measured(
                worker_phase_probe::Phase::Automatic,
                || io.automatic(),
                |r| match r {
                    Ok(Some(_)) => 0,
                    Ok(None) => 1,
                    Err(_) => 2,
                },
            );
            #[cfg(not(all(windows, feature = "first-paint-fixture")))]
            let selected = io.automatic();
            selected
        }
    };
    let network_result = match selected {
        Ok(Some(alias)) => {
            network_view.alias = Some(alias.clone());
            network_view.state = NetworkState::Observing;
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            let sampled = worker_phase_probe::measured(
                worker_phase_probe::Phase::Sample,
                || io.sample(&alias),
                |r| match r {
                    Ok(Some(_)) => 0,
                    Ok(None) => 1,
                    Err(_) => 2,
                },
            );
            #[cfg(not(all(windows, feature = "first-paint-fixture")))]
            let sampled = io.sample(&alias);
            Some(sampled)
        }
        Ok(None) => {
            io.reset();
            network_view.state = if selection.mode == NetworkMode::Off {
                NetworkState::Off
            } else {
                NetworkState::NoRoute
            };
            None
        }
        Err(error) => {
            io.reset();
            Some(Err(error))
        }
    };
    let network_failed = network_result.as_ref().is_some_and(Result::is_err);
    if network_failed {
        network_view.state = NetworkState::Failed;
    }
    let current = network_result.and_then(Result::ok).flatten();
    let rate = current
        .as_ref()
        .zip(previous.as_ref())
        .and_then(|(now, before)| now.rate_since(before));
    *previous = current;
    Sample {
        revision: selection.revision,
        at: Instant::now(),
        free,
        rate,
        observation_failed: (free_failed, network_failed),
        network_view,
    }
}
fn worker_loop(
    requests: mpsc::Receiver<WorkerCommand>,
    worker: &Signals,
    responses: &Mutex<Mailbox>,
    observe: &mut impl FnMut(&Selection, &mut Option<metrics::NetworkSample>, Instant) -> Sample,
) {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    let _probe_owner = worker_phase_probe::Owner::from_environment();
    let mut selection = Selection::default();
    let mut context = None;
    let mut previous = None;
    let mut next = Instant::now();
    loop {
        if worker.stop.load(Ordering::Acquire) {
            break;
        }
        let delay = next.saturating_duration_since(Instant::now());
        match requests.recv_timeout(delay) {
            Ok(WorkerCommand::Selection(command)) => {
                selection = command.selection;
                if let Ok(mut terminal_context) = worker.context.lock() {
                    *terminal_context = Some(command.context.clone());
                }
                context = Some(command.context);
                previous = None;
            }
            Ok(WorkerCommand::StopWake) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if worker.stop.load(Ordering::Acquire) {
            break;
        }
        if Instant::now() < next {
            continue;
        }
        let sample_due = next;
        next = Instant::now() + INTERVAL;
        if context.is_none() {
            continue;
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        worker_phase_probe::begin_tick(sample_due);
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        let mut worker_phase = crate::worker_phase_fixture::Span::metrics_observe();
        let sample = observe(&selection, &mut previous, sample_due);
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        worker_phase.finish(sample.observation_failed.0 || sample.observation_failed.1);
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        let wake = match worker_phase_probe::measured(
            worker_phase_probe::Phase::Mailbox,
            || responses.lock().map(|mut mailbox| mailbox.publish(sample)),
            |r| if r.is_ok() { 0 } else { 2 },
        ) {
            Ok(wake) => wake,
            Err(_) => {
                worker.failed.store(true, Ordering::Release);
                break;
            }
        };
        #[cfg(not(all(windows, feature = "first-paint-fixture")))]
        let wake = match responses.lock() {
            Ok(mut mailbox) => mailbox.publish(sample),
            Err(_) => {
                worker.failed.store(true, Ordering::Release);
                break;
            }
        };
        if wake && let Some(ctx) = &context {
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            worker_phase.display_wake();
            #[cfg(all(windows, feature = "first-paint-fixture"))]
            worker_phase_probe::measured(
                worker_phase_probe::Phase::Repaint,
                || ctx.request_repaint(),
                |_| 0,
            );
            #[cfg(not(all(windows, feature = "first-paint-fixture")))]
            ctx.request_repaint();
        }
        #[cfg(all(windows, feature = "first-paint-fixture"))]
        worker_phase_probe::end_tick();
    }
}

pub(crate) struct MetricsPanel {
    commands: SyncSender<WorkerCommand>,
    results: Arc<Mutex<Mailbox>>,
    signals: Arc<Signals>,
    selection: Selection,
    pending: bool,
    edit: String,
    error: bool,
    latest: Option<Sample>,
    latest_key: Option<DisplayKey>,
    host_freshness_timer: bool,
    pending_visible: Option<VisibleKey>,
    #[cfg_attr(not(any(feature = "cpu", test)), allow(dead_code))]
    presented_visible: Option<VisibleKey>,
}
impl MetricsPanel {
    pub(crate) fn new() -> Self {
        let (commands, requests) = mpsc::sync_channel::<WorkerCommand>(1);
        let results = Arc::new(Mutex::new(Mailbox::default()));
        let responses = results.clone();
        let signals = Arc::new(Signals::default());
        let worker = signals.clone();
        let spawn = thread::Builder::new()
            .name("dlpad-metrics".into())
            .spawn(move || {
                supervise(worker.clone(), || {
                    let mut network = metrics::NetworkObserver::default();
                    let mut revision = None;
                    let mut discovery = Discovery::default();
                    worker_loop(
                        requests,
                        &worker,
                        &responses,
                        &mut |selection, previous, due| {
                            if revision != Some(selection.revision) {
                                network.reset();
                                revision = Some(selection.revision);
                            }
                            observe_selection(
                                selection,
                                previous,
                                due,
                                &mut network,
                                &mut discovery,
                            )
                        },
                    );
                })
            });
        if spawn.is_err() {
            signals.failed.store(true, Ordering::Release);
            signals.stopped.store(true, Ordering::Release);
        }
        Self {
            commands,
            results,
            signals,
            selection: Selection::default(),
            pending: true,
            edit: String::new(),
            error: false,
            latest: None,
            latest_key: None,
            host_freshness_timer: false,
            pending_visible: None,
            presented_visible: None,
        }
    }
    fn changed(&mut self) {
        self.selection.revision = self.selection.revision.wrapping_add(1);
        self.pending = true;
        self.latest = None;
        self.latest_key = None;
    }
    pub(crate) fn set_root(&mut self, root: PathBuf) {
        if self.signals.stop.load(Ordering::Acquire) {
            return;
        }
        let root = (path_bytes(&root) <= 32768 && root.is_absolute()).then_some(root);
        if self.selection.root != root {
            self.selection.root = root;
            self.changed();
        }
    }
    fn apply_interface(&mut self) {
        if self.signals.stop.load(Ordering::Acquire) {
            return;
        }
        #[cfg(windows)]
        let value = self.edit.as_str();
        #[cfg(not(windows))]
        let value = self.edit.trim();
        if !value.is_empty() && !metrics::valid_observed_interface(value) {
            self.error = true;
            return;
        }
        self.error = false;
        let mode = if value.is_empty() {
            NetworkMode::Off
        } else {
            NetworkMode::Manual
        };
        if self.selection.interface != value || self.selection.mode != mode {
            self.selection.interface = value.into();
            self.selection.mode = mode;
            self.changed();
        }
    }
    #[cfg(windows)]
    fn choose_network(&mut self, mode: NetworkMode, alias: Option<String>) {
        if self.signals.stop.load(Ordering::Acquire) || self.stopped() {
            return;
        }
        let interface = if mode == NetworkMode::Manual {
            alias.unwrap_or_default()
        } else {
            String::new()
        };
        if mode == NetworkMode::Manual && !metrics::valid_observed_interface(&interface) {
            self.error = true;
            return;
        }
        self.error = false;
        if self.selection.mode != mode || self.selection.interface != interface {
            self.selection.mode = mode;
            self.selection.interface = interface.clone();
            if mode == NetworkMode::Manual {
                self.edit = interface;
            }
            self.changed();
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        if self.pending
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped()
            && self
                .commands
                .try_send(WorkerCommand::Selection(Command {
                    selection: self.selection.clone(),
                    context: ctx.clone(),
                }))
                .is_ok()
        {
            self.pending = false;
        }
        self.refresh_mailbox();
        if let Some(sample) = &self.latest {
            let age = sample.at.elapsed();
            if age > STALE {
                self.latest = None;
                self.latest_key = None;
            } else if !self.host_freshness_timer {
                ctx.request_repaint_after(STALE.saturating_sub(age) + Duration::from_millis(1));
            }
        }
        if self.pending || (self.signals.stop.load(Ordering::Acquire) && !self.stopped()) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    // Bounded latest-only refresh: no I/O, selection commands, waits or repaint requests.
    // A terminal poison error also notifies the owned worker to stop its idle receive.
    fn refresh_mailbox(&mut self) {
        match self.results.try_lock() {
            Ok(mailbox) => {
                if let Some(sample) = mailbox.latest.as_ref()
                    && sample.revision == self.selection.revision
                {
                    self.latest = Some(sample.clone());
                    self.latest_key = mailbox.key.clone();
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
            Err(std::sync::TryLockError::Poisoned(_)) => {
                self.signals.failed.store(true, Ordering::Release);
                stop_worker(&self.signals, &self.commands);
            }
        }
        if self.signals.failed.load(Ordering::Acquire) {
            self.latest = None;
            self.latest_key = None;
        }
    }
    #[cfg(any(feature = "cpu", test))]
    pub(crate) fn use_host_freshness_timer(&mut self) {
        self.host_freshness_timer = true;
    }
    #[cfg(any(feature = "cpu", test))]
    fn visible_key(&self, now: Instant) -> VisibleKey {
        let failed = self.signals.failed.load(Ordering::Acquire);
        if !failed
            && !self.stopped()
            && let Some(sample) = &self.latest
            && sample.revision == self.selection.revision
            && now.saturating_duration_since(sample.at) <= STALE
        {
            // Production samples arrive with the already formatted mailbox key.
            // Fallback supports direct synthetic samples and remains bounded.
            return VisibleKey::Fresh(
                self.latest_key
                    .clone()
                    .unwrap_or_else(|| display_key(sample)),
            );
        }
        VisibleKey::Unknown {
            revision: self.selection.revision,
            failed,
        }
    }
    #[cfg(any(feature = "cpu", test))]
    pub(crate) fn freshness_deadline(&self, now: Instant) -> Option<Instant> {
        if !self.host_freshness_timer || self.stopped() {
            return None;
        }
        // Existing generic 100ms command/Stop handling remains the owner of these states.
        if self.pending || self.signals.stop.load(Ordering::Acquire) {
            return None;
        }
        if !self.signals.failed.load(Ordering::Acquire)
            && let Some(sample) = &self.latest
            && sample.revision == self.selection.revision
            && now.saturating_duration_since(sample.at) <= STALE
        {
            return sample.at.checked_add(STALE + Duration::from_millis(1));
        }
        // A healthy producer may return with the same display key after a stall.
        now.checked_add(INTERVAL)
    }
    #[cfg(any(feature = "cpu", test))]
    pub(crate) fn check_freshness(&mut self, now: Instant) -> bool {
        self.refresh_mailbox();
        if self.stopped()
            || self
                .latest
                .as_ref()
                .is_some_and(|sample| now.saturating_duration_since(sample.at) > STALE)
        {
            self.latest = None;
            self.latest_key = None;
        }
        self.pending
            || (self.signals.stop.load(Ordering::Acquire) && !self.stopped())
            || self.presented_visible.as_ref() != Some(&self.visible_key(now))
    }
    #[cfg(any(feature = "cpu", test))]
    pub(crate) fn frame_presented(&mut self) {
        if let Some(key) = self.pending_visible.take() {
            self.presented_visible = Some(key);
        }
    }
    #[cfg(all(test, not(any(target_os = "linux", windows))))]
    pub(crate) fn edit_for_test(&self) -> &str {
        &self.edit
    }
    #[cfg(test)]
    pub(crate) fn show_settings(&mut self, ui: &mut egui::Ui, enabled: bool) {
        let _ = self.show_settings_with_critical(ui, enabled);
    }
    pub(crate) fn show_settings_with_critical(
        &mut self,
        ui: &mut egui::Ui,
        enabled: bool,
    ) -> Option<egui::Response> {
        let enabled = enabled
            && cfg!(any(target_os = "linux", target_os = "windows"))
            && !self.signals.stop.load(Ordering::Acquire)
            && !self.stopped();
        // Unsupported platforms explain the state before presenting inert controls.
        // Keep the saved value visible; do not mutate selection or persistence here.
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let critical = {
            let response = ui.label(c::NETWORK_COUNTER_UNAVAILABLE);
            ui.push_id("network-unavailable-details", |ui| {
                ui.collapsing(c::NETWORK_COUNTER_UNAVAILABLE_DETAILS, |ui| {
                    ui.label(c::NETWORK_COUNTER_UNAVAILABLE_HELP);
                });
            });
            Some(response)
        };
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let critical = None;
        #[cfg(windows)]
        {
            let label = ui.label(c::NETWORK_CONNECTION);
            let selected_text = match self.selection.mode {
                NetworkMode::Auto => c::NETWORK_AUTOMATIC.to_owned(),
                NetworkMode::Off => c::NETWORK_OFF.to_owned(),
                NetworkMode::Manual => self.selection.interface.clone(),
            };
            let view = self
                .latest
                .as_ref()
                .map(|s| s.network_view.clone())
                .unwrap_or_default();
            let mut chosen = None;
            let row_width = ui.available_width().max(0.0);
            let refresh_width = (egui::WidgetText::from(c::NETWORK_REFRESH)
                .into_galley(ui, None, f32::INFINITY, egui::TextStyle::Button)
                .size()
                .x
                + 2.0 * crate::ui_tokens::CONTROL_PADDING_X)
                .max(crate::ui_tokens::CONTROL_MIN_WIDTH);
            let alongside_width = row_width - refresh_width - ui.spacing().item_spacing.x;
            let combo_width = if alongside_width >= 160.0 {
                alongside_width.min(360.0)
            } else {
                row_width.min(360.0)
            };
            let mut refresh_clicked = false;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().interact_size.y = ui
                    .spacing()
                    .interact_size
                    .y
                    .max(crate::ui_tokens::CONTROL_HEIGHT);
                ui.add_enabled_ui(enabled, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(combo_width, crate::ui_tokens::CONTROL_HEIGHT),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let connection =
                                egui::ComboBox::from_id_salt("settings-network-choice")
                                    .selected_text(selected_text)
                                    .width(combo_width)
                                    .truncate()
                                    .height(240.0)
                                    .show_ui(ui, |ui| {
                                        if crate::ui_components::choice(
                                            ui,
                                            self.selection.mode == NetworkMode::Auto,
                                            c::NETWORK_AUTOMATIC,
                                        )
                                        .clicked()
                                        {
                                            chosen = Some((NetworkMode::Auto, None));
                                            ui.close();
                                        }
                                        if crate::ui_components::choice(
                                            ui,
                                            self.selection.mode == NetworkMode::Off,
                                            c::NETWORK_OFF,
                                        )
                                        .clicked()
                                        {
                                            chosen = Some((NetworkMode::Off, None));
                                            ui.close();
                                        }
                                        for name in view.aliases.iter() {
                                            if crate::ui_components::choice(
                                                ui,
                                                self.selection.mode == NetworkMode::Manual
                                                    && self.selection.interface == *name,
                                                name,
                                            )
                                            .clicked()
                                            {
                                                chosen =
                                                    Some((NetworkMode::Manual, Some(name.clone())));
                                                ui.close();
                                            }
                                        }
                                    })
                                    .response
                                    .labelled_by(label.id);
                            let connection = crate::ui_components::combo_response(ui, connection);
                            if connection.gained_focus() {
                                connection.scroll_to_me(None);
                            }
                        },
                    );
                });
                let refresh = crate::ui_components::secondary_sized(
                    ui,
                    c::NETWORK_REFRESH,
                    enabled && self.selection.mode != NetworkMode::Off,
                    egui::vec2(
                        refresh_width.min(row_width),
                        crate::ui_tokens::CONTROL_HEIGHT,
                    ),
                );
                if refresh.gained_focus() {
                    refresh.scroll_to_me(None);
                }
                refresh_clicked = refresh.clicked();
            });
            if let Some((mode, alias)) = chosen {
                self.choose_network(mode, alias);
            }
            if refresh_clicked {
                self.selection.refresh = self.selection.refresh.wrapping_add(1);
                self.changed();
            }
            if let Some(alias) = self
                .latest
                .as_ref()
                .and_then(|s| s.network_view.alias.as_ref())
            {
                ui.add(egui::Label::new(format!("{}: {}", c::NETWORK_SELECTED, alias)).wrap());
            }
            if view.list_failed {
                ui.label(c::NETWORK_LIST_FAILED);
            }
            if self.selection.mode == NetworkMode::Manual
                && view.state == NetworkState::Failed
                && self.latest.as_ref().is_some_and(|sample| {
                    sample.revision == self.selection.revision && sample.at.elapsed() <= STALE
                })
            {
                ui.label(c::NETWORK_MANUAL_FAILED);
            }
            ui.label(c::NETWORK_AUTOMATIC_HELP);
        }
        let label = ui.label(c::INTERFACE);
        let width = (ui.available_width().min(360.0)
            - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
            .max(0.0);
        let input = ui
            .add_enabled_ui(enabled, |ui| {
                crate::ui_components::text_input_with_id_and_hint(
                    ui,
                    &mut self.edit,
                    if cfg!(windows) { 256 } else { 64 },
                    width,
                    egui::Id::new("settings-network-interface-input"),
                    c::INTERFACE_EXAMPLE,
                )
                .labelled_by(label.id)
            })
            .inner;
        // Match Apply's syntax policy; adapter existence is a worker observation.
        #[cfg(windows)]
        let value = self.edit.as_str();
        #[cfg(not(windows))]
        let value = self.edit.trim();
        let syntax_invalid = !value.is_empty() && !metrics::valid_observed_interface(value);
        if input.changed() {
            self.error = syntax_invalid;
        }
        ui.ctx().accesskit_node_builder(input.id, |node| {
            if syntax_invalid {
                node.set_invalid(egui::accesskit::Invalid::True);
                node.set_description(c::INVALID_INTERFACE);
            } else {
                node.clear_invalid();
                node.clear_description();
            }
        });
        if syntax_invalid || self.error {
            // Keep actionable feedback adjacent to its input, replacing help rather than
            // pushing the error beyond Apply at larger text sizes.
            let error = ui.add(
                egui::Label::new(
                    egui::RichText::new(c::INVALID_INTERFACE).color(ui.visuals().error_fg_color),
                )
                .wrap(),
            );
            let feedback = label.rect.union(input.rect).union(error.rect);
            if input.changed()
                || input.gained_focus()
                || (input.has_focus() && !ui.clip_rect().contains_rect(feedback))
            {
                ui.scroll_to_rect(feedback, Some(egui::Align::TOP));
            }
        } else {
            #[cfg(target_os = "linux")]
            ui.label(c::INTERFACE_HELP);
            #[cfg(windows)]
            ui.label(c::NETWORK_MANUAL_HELP);
        }
        let apply = ui
            .push_id("settings-network-apply", |ui| {
                ui.scope_builder(
                    egui::UiBuilder::new().id("settings-network-apply-control"),
                    |ui| crate::ui_components::secondary(ui, c::APPLY_INTERFACE, enabled),
                )
                .inner
            })
            .inner;
        if apply.gained_focus() {
            apply.scroll_to_me(None);
        }
        if apply.clicked() {
            self.apply_interface();
        }
        critical
    }
    pub(crate) fn status(&mut self, ui: &mut egui::Ui) {
        // Capture exactly the availability/key used by this status call.
        // A failed/aborted frame must never advance last-presented metrics.
        let now = Instant::now();
        let failed = self.signals.failed.load(Ordering::Acquire);
        let stopped = self.host_freshness_timer && self.stopped();
        let current = self.latest.as_ref().filter(|sample| {
            sample.revision == self.selection.revision
                && now.saturating_duration_since(sample.at) <= STALE
                && !failed
                && !stopped
        });
        self.pending_visible = Some(match current {
            Some(sample) => VisibleKey::Fresh(
                self.latest_key
                    .clone()
                    .unwrap_or_else(|| display_key(sample)),
            ),
            None => VisibleKey::Unknown {
                revision: self.selection.revision,
                failed,
            },
        });
        let free = current
            .and_then(|sample| sample.free)
            .map(|bytes| format!("{:.1} GiB", bytes as f64 / 1073741824.0))
            .unwrap_or_else(|| c::UNKNOWN.into());
        // Freeze the row bound before either metric can expand the wrapping Ui.
        let metric_width = ui.max_rect().intersect(ui.clip_rect()).width().max(1.0);
        let location = save_location_prefix(self.selection.root.as_deref());
        let label = metric_label(
            ui,
            metric_width,
            format!("{}{}: {}", c::FREE_SPACE, location, free),
        );
        label.on_hover_ui(|ui| {
            ui.label(c::FREE_SPACE_HELP);
            if let Some(root) = self.selection.root.as_deref() {
                ui.label(crate::path_picker::display_path(root));
            }
        });
        if let Some(rate) = current.and_then(visible_rate) {
            metric_label(
                ui,
                metric_width,
                format!(
                    "{} {} · {} {}",
                    c::RECEIVED,
                    crate::display_rate::format_bytes_per_second(Some(
                        rate.received_bytes_per_second
                    )),
                    c::SENT,
                    crate::display_rate::format_bytes_per_second(Some(rate.sent_bytes_per_second))
                ),
            );
        } else {
            #[cfg(windows)]
            let unknown = if self.selection.mode == NetworkMode::Off {
                c::NETWORK_OFF_STATUS
            } else {
                match current.map(|s| s.network_view.state) {
                    Some(NetworkState::NoRoute) => c::NETWORK_NO_ACTIVE,
                    Some(NetworkState::Failed) if self.selection.mode == NetworkMode::Auto => {
                        c::NETWORK_AUTOMATIC_FAILED
                    }
                    Some(NetworkState::Observing)
                        if current.is_some_and(|sample| !sample.observation_failed.1)
                            && !self.signals.stop.load(Ordering::Acquire)
                            && !self.stopped() =>
                    {
                        c::NETWORK_PREPARING
                    }
                    None if self.latest.is_none()
                        && !failed
                        && !stopped
                        && !self.signals.stop.load(Ordering::Acquire)
                        && !self.stopped() =>
                    {
                        // No completed sample is still preparation after dispatch; cached stale samples stay unknown.
                        c::NETWORK_PREPARING
                    }
                    _ => c::NETWORK_UNKNOWN,
                }
            };
            #[cfg(not(windows))]
            let unknown = c::NETWORK_UNKNOWN;
            metric_label(ui, metric_width, unknown);
        }
    }
    pub(crate) fn request_stop(&mut self) {
        stop_worker(&self.signals, &self.commands);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.signals.stopped.load(Ordering::Acquire)
    }
}
impl Drop for MetricsPanel {
    fn drop(&mut self) {
        self.request_stop();
    }
}

// A footer metric is allocated as one item in the wrapping parent. Lay out
// against the full row, rather than Label's remaining-width indentation path.
// Text wider than the entire row still wraps within this bounded galley.
// This qualifier identifies the configured save path, not physical volume identity.
// Mounted folders can belong to another volume; the tooltip describes that distinction.
fn save_location_prefix(root: Option<&std::path::Path>) -> String {
    #[cfg(windows)]
    if let Some(std::path::Component::Prefix(prefix)) = root.and_then(|p| p.components().next()) {
        match prefix.kind() {
            std::path::Prefix::Disk(drive) | std::path::Prefix::VerbatimDisk(drive) => {
                return format!(" ({}:)", char::from(drive).to_ascii_uppercase());
            }
            _ => (),
        }
    }
    #[cfg(not(windows))]
    let _ = root;
    String::new()
}

fn metric_label(
    ui: &mut egui::Ui,
    width: f32,
    text: impl Into<egui::WidgetText>,
) -> egui::Response {
    let galley = text.into().into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        width,
        egui::TextStyle::Body,
    );
    ui.add(egui::Label::new(galley))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_footer_metrics_move_as_units_and_wrap_only_beyond_the_full_row() {
        for (logical_width, expect_one_line) in [(400.0, true), (48.0, false)] {
            let mut panel = MetricsPanel::new();
            stop(&mut panel); // No metric query or worker is left alive by assertions.
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_pixels_per_point(2.0); // 800 physical px = 400 logical px.
            // A pending zoom applies using the previous viewport content_rect on
            // its first pass, overriding supplied screen_rect (egui begin_pass).
            // Consume that transition, then establish the actual target viewport.
            for time in [0.0, 0.01] {
                let _ = ctx.run(
                    egui::RawInput {
                        time: Some(time),
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(logical_width, 300.0),
                        )),
                        ..Default::default()
                    },
                    |_| {},
                );
            }
            assert!(
                (ctx.input(|i| i.content_rect().width()) - logical_width).abs() < 0.1,
                "actual viewport must be stabilized before metric assertions"
            );
            assert!((ctx.pixels_per_point() - 2.0).abs() < 0.01);
            ctx.enable_accesskit();
            let mut clip = egui::Rect::NOTHING;
            let mut row_width = 0.0;
            let mut natural_network_width = 0.0;
            let output = ctx.run(
                egui::RawInput {
                    time: Some(0.02),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(logical_width, 300.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.style_mut()
                            .text_styles
                            .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
                        ui.horizontal_wrapped(|ui| {
                            clip = ui.clip_rect();
                            row_width = ui.max_rect().intersect(clip).width();
                            natural_network_width = egui::WidgetText::from(c::NETWORK_UNKNOWN)
                                .into_galley(
                                    ui,
                                    Some(egui::TextWrapMode::Extend),
                                    f32::INFINITY,
                                    egui::TextStyle::Body,
                                )
                                .size()
                                .x;
                            // Leave room for the actual free-space label plus only 20px
                            // for network: free must fit before testing the network break.
                            let free_text = format!("{}: {}", c::FREE_SPACE, c::UNKNOWN);
                            let free_width = egui::WidgetText::from(free_text)
                                .into_galley(
                                    ui,
                                    Some(egui::TextWrapMode::Extend),
                                    f32::INFINITY,
                                    egui::TextStyle::Body,
                                )
                                .size()
                                .x;
                            let width = (ui.available_width()
                                - free_width
                                - 2.0 * ui.spacing().item_spacing.x
                                - 20.0)
                                .max(0.0);
                            ui.allocate_space(egui::vec2(width, 14.0));
                            panel.status(ui);
                        });
                    });
                },
            );
            let mut glyphs = vec![];
            let mut free_glyphs = vec![];
            for shape in &output.shapes {
                network_text_shapes(&shape.shape, c::NETWORK_UNKNOWN, &mut glyphs);
                network_text_shapes(
                    &shape.shape,
                    &format!("{}: {}", c::FREE_SPACE, c::UNKNOWN),
                    &mut free_glyphs,
                );
            }
            assert_eq!(free_glyphs.len(), 1);
            if expect_one_line {
                assert_eq!(
                    free_glyphs[0].galley.rows.len(),
                    1,
                    "free metric fits before the narrow network remainder"
                );
                assert!(
                    clip.right() - (free_glyphs[0].pos.x + free_glyphs[0].galley.size().x) < 40.0,
                    "actual free metric leaves only a fragment-width remainder"
                );
            }
            assert_eq!(glyphs.len(), 1, "one actual network information label");
            assert_eq!(
                natural_network_width <= row_width,
                expect_one_line,
                "fixture must genuinely distinguish fitting vs oversized text: logical={logical_width} row={row_width} natural={natural_network_width} clip={clip:?}"
            );
            let text = &glyphs[0];
            assert_eq!(
                text.galley.rows.len() == 1,
                expect_one_line,
                "remaining-width wrapping must not split a fitting metric: logical={logical_width} row={row_width} natural={natural_network_width} clip={clip:?}"
            );
            let bounds = egui::Rect::from_min_size(text.pos, text.galley.size());
            assert!(
                clip.contains_rect(bounds),
                "entire metric stays in the footer clip"
            );
            let tree = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("AccessKit output");
            assert!(
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.bounds().is_some_and(|b| {
                        (b.x0 as f32 - bounds.left()).abs() < 1.0
                            && (b.x1 as f32 - bounds.right()).abs() < 1.0
                            && (b.y0 as f32 - bounds.top()).abs() < 1.0
                            && (b.y1 as f32 - bounds.bottom()).abs() < 1.0
                    })),
                "actual label accessibility geometry follows the whole metric"
            );
        }
    }
    #[test]
    fn actual_network_controls_expose_platform_availability_and_keep_saved_bytes() {
        let mut panel = MetricsPanel::new();
        panel.edit = "lo".into();
        let original_edit = panel.edit.clone();
        let original_selection = panel.selection.interface.clone();
        let revision = panel.selection.revision;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let first = network_frame(&mut panel, &ctx, 1.0, true, None, vec![]);
        let (input_id, _, input_disabled) = network_input(&first);
        let tree = first.platform_output.accesskit_update.as_ref().unwrap();
        let (apply_id, apply_node) = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::APPLY_INTERFACE)
            })
            .expect("actual Apply button");
        let apply_disabled = apply_node.is_disabled();
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = network_frame(
                &mut panel,
                &ctx,
                2.0,
                true,
                None,
                vec![
                    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: input_id,
                        data: None,
                    }),
                    egui::Event::Text("changed".into()),
                    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target: *apply_id,
                        data: None,
                    }),
                ],
            );
        }
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let _ = (input_id, apply_id);
        let unchanged = panel.edit == original_edit
            && panel.selection.interface == original_selection
            && panel.selection.revision == revision;
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let explanation = painted_metric_text(&first, c::NETWORK_COUNTER_UNAVAILABLE);
        stop(&mut panel);
        assert_eq!(
            input_disabled,
            !cfg!(any(target_os = "linux", target_os = "windows"))
        );
        assert_eq!(
            apply_disabled,
            !cfg!(any(target_os = "linux", target_os = "windows"))
        );
        assert!(
            unchanged,
            "unsupported controls must not edit or dispatch selection"
        );
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        assert!(explanation, "actual UI must explain unavailable counters");
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn painted_metric_text(output: &egui::FullOutput, text: &str) -> bool {
        let mut found = Vec::new();
        for clipped in &output.shapes {
            network_text_shapes(&clipped.shape, text, &mut found);
        }
        !found.is_empty()
    }
    fn network_frame(
        panel: &mut MetricsPanel,
        ctx: &egui::Context,
        time: f64,
        enabled: bool,
        focus: Option<egui::Id>,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(id) = focus {
                        ctx.memory_mut(|m| m.request_focus(id));
                    }
                    panel.show_settings(ui, enabled);
                });
            },
        )
    }
    fn network_input(output: &egui::FullOutput) -> (egui::accesskit::NodeId, egui::Rect, bool) {
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("actual AccessKit tree");
        let nodes: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == egui::accesskit::Role::TextInput)
            .collect();
        assert_eq!(nodes.len(), 1, "actual network settings has one input");
        let (id, node) = nodes[0];
        let b = node.bounds().expect("actual input bounds");
        (
            *id,
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            ),
            node.is_disabled(),
        )
    }
    fn network_text_shapes(
        shape: &egui::epaint::Shape,
        text: &str,
        found: &mut Vec<egui::epaint::TextShape>,
    ) {
        match shape {
            egui::epaint::Shape::Text(s) if s.galley.text() == text => found.push(s.clone()),
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    network_text_shapes(s, text, found);
                }
            }
            _ => {}
        }
    }
    #[test]
    fn actual_network_settings_centers_glyphs_with_product_input_padding() {
        for dark in [false, true] {
            for scale in [1.0, 1.5, 2.0] {
                let mut panel = MetricsPanel::new();
                // Do not apply a network selection: this test causes no interface query.
                panel.edit = "A0_interface".into();
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
                let output = network_frame(&mut panel, &ctx, 1.0, true, None, vec![]);
                let (_, bounds, disabled) = network_input(&output);
                let mut glyphs = vec![];
                for clipped in &output.shapes {
                    network_text_shapes(&clipped.shape, "A0_interface", &mut glyphs);
                }
                stop(&mut panel); // ACK even before expected-RED geometry assertions.
                assert_eq!(
                    disabled,
                    !cfg!(any(target_os = "linux", target_os = "windows"))
                );
                assert_eq!(glyphs.len(), 1);
                let text = &glyphs[0];
                assert!((bounds.height() - 32.0).abs() < 0.5);
                assert!(
                    (text.pos.x - bounds.left() - 12.0).abs() < 0.5,
                    "actual settings glyph origin needs product 12px inset"
                );
                assert!(
                    (text.pos.y + text.galley.size().y * 0.5 - bounds.center().y).abs() < 0.5,
                    "actual settings line must be vertically centered"
                );
            }
        }
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn actual_network_settings_invalid_state_preserves_identity_and_focus_then_disables() {
        let mut panel = MetricsPanel::new();
        panel.edit = "lo".into();
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let first = network_frame(&mut panel, &ctx, 1.0, true, None, vec![]);
        let (original_node, _, _) = network_input(&first);
        let _ = network_frame(
            &mut panel,
            &ctx,
            2.0,
            true,
            None,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target: original_node,
                    data: None,
                },
            )],
        );
        let focus = ctx.memory(|m| m.focused()).expect("actual input focused");
        panel.edit = "../invalid".into();
        panel.apply_interface(); // Validates/rejects before Selection dispatch, no network query.
        assert!(panel.error);
        let invalid = network_frame(&mut panel, &ctx, 3.0, true, None, vec![]);
        assert_eq!(network_input(&invalid).0, original_node);
        assert!(ctx.memory(|m| m.has_focus(focus)));
        // Drive actual TextEdit char_limit via events, not direct 65-char fixture assignment.
        panel.edit.clear();
        let edited = network_frame(
            &mut panel,
            &ctx,
            4.0,
            true,
            Some(focus),
            vec![egui::Event::Text("x".repeat(65))],
        );
        assert_eq!(panel.edit.chars().count(), 64);
        assert_eq!(network_input(&edited).0, original_node);
        let before = panel.edit.clone();
        let disabled = network_frame(
            &mut panel,
            &ctx,
            5.0,
            false,
            None,
            vec![egui::Event::Text("should-not-insert".into())],
        );
        let observed = network_input(&disabled);
        let unchanged = panel.edit == before;
        stop(&mut panel);
        assert_eq!(observed.0, original_node);
        assert!(observed.2);
        assert!(unchanged, "disabled actual input rejects text");
    }

    #[cfg(windows)]
    #[test]
    fn actual_apply_focus_and_node_identity_survive_alias_rows_and_disabled_click() {
        let mut panel = MetricsPanel::new();
        panel.selection.mode = NetworkMode::Manual;
        panel.selection.interface = "선택한 연결".into();
        panel.edit = "편집 중 연결".into();
        let mode = panel.selection.mode;
        let interface = panel.selection.interface.clone();
        let edit = panel.edit.clone();
        let revision = panel.selection.revision;
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let apply_node = |output: &egui::FullOutput| {
            let tree = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("actual AccessKit output");
            let (id, node) = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some(c::APPLY_INTERFACE)
                })
                .expect("actual Apply button node");
            (*id, node.is_disabled())
        };
        let first = network_frame(&mut panel, &ctx, 1.0, true, None, vec![]);
        let original_node = apply_node(&first).0;
        network_frame(
            &mut panel,
            &ctx,
            1.1,
            true,
            None,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target: original_node,
                    data: None,
                },
            )],
        );
        let original_focus = ctx
            .memory(|memory| memory.focused())
            .expect("actual Apply focus action");
        let mut observed = Vec::new();
        for (index, alias) in [
            Some("한글 관측 연결".to_owned()),
            None,
            Some("😀 긴 관측 연결".to_owned()),
        ]
        .into_iter()
        .enumerate()
        {
            panel.latest = Some(Sample {
                revision,
                at: Instant::now(),
                free: None,
                rate: None,
                observation_failed: (false, false),
                network_view: NetworkView {
                    state: NetworkState::Observing,
                    alias,
                    ..Default::default()
                },
            });
            let frame = network_frame(
                &mut panel,
                &ctx,
                1.2 + index as f64 * 0.1,
                true,
                None,
                vec![],
            );
            observed.push((
                apply_node(&frame),
                ctx.memory(|memory| memory.has_focus(original_focus)),
            ));
        }
        let disabled = network_frame(
            &mut panel,
            &ctx,
            1.6,
            false,
            None,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target: original_node,
                    data: None,
                },
            )],
        );
        let disabled_node = apply_node(&disabled);
        let unchanged = panel.selection.mode == mode
            && panel.selection.interface == interface
            && panel.edit == edit
            && panel.selection.revision == revision;
        stop(&mut panel);
        for (node, focused) in observed {
            assert_eq!(
                node,
                (original_node, false),
                "alias arrival/removal must keep the same enabled Apply node"
            );
            assert!(
                focused,
                "alias rows must not move actual Apply keyboard focus"
            );
        }
        assert_eq!(disabled_node, (original_node, true));
        assert!(
            unchanged,
            "disabled actual click cannot apply the manual draft or change the selection"
        );
    }

    #[cfg(windows)]
    #[test]
    fn actual_network_controls_focus_reveals_full_body_without_selecting_or_held_focus_snap() {
        fn frame(
            panel: &mut MetricsPanel,
            ctx: &egui::Context,
            time: f64,
            events: Vec<egui::Event>,
        ) -> (egui::FullOutput, f32, egui::Rect) {
            let mut offset = 0.0;
            let mut body = egui::Rect::NOTHING;
            let output = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::TopBottomPanel::top("network-focus-navigation")
                        .exact_height(40.0)
                        .show(ctx, |ui| {
                            ui.label("상단 탐색");
                        });
                    egui::TopBottomPanel::bottom("network-focus-footer")
                        .exact_height(48.0)
                        .show(ctx, |ui| {
                            ui.label("하단 상태");
                        });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let scroll = egui::ScrollArea::vertical()
                            .id_salt("actual-network-focus-body")
                            .animated(false)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                body = ui.clip_rect();
                                ui.add_space(480.0);
                                panel.show_settings(ui, true);
                                ui.add_space(512.0);
                            });
                        offset = scroll.state.offset.y;
                    });
                },
            );
            (output, offset, body)
        }
        fn target(
            output: &egui::FullOutput,
            index: usize,
        ) -> (egui::accesskit::NodeId, egui::Rect) {
            let tree = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("actual AccessKit tree");
            let (id, node) = tree
                .nodes
                .iter()
                .find(|(_, node)| match index {
                    0 => {
                        node.role() == egui::accesskit::Role::ComboBox
                            && node.value() == Some(c::NETWORK_AUTOMATIC)
                    }
                    1 => {
                        node.role() == egui::accesskit::Role::Button
                            && node.label() == Some(c::NETWORK_REFRESH)
                    }
                    _ => {
                        node.role() == egui::accesskit::Role::Button
                            && node.label() == Some(c::APPLY_INTERFACE)
                    }
                })
                .expect("actual network control");
            let b = node.bounds().expect("actual full control bounds");
            (
                *id,
                egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32),
                    egui::pos2(b.x1 as f32, b.y1 as f32),
                ),
            )
        }
        for index in 0..3 {
            let mut panel = MetricsPanel::new();
            panel.edit = "보존할 한글 연결".into();
            let original = (
                panel.selection.mode,
                panel.selection.interface.clone(),
                panel.selection.revision,
                panel.selection.refresh,
                panel.edit.clone(),
            );
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.enable_accesskit();
            let initial = frame(&mut panel, &ctx, 0.0, vec![]);
            let original_node = target(&initial.0, index).0;
            let _ = frame(
                &mut panel,
                &ctx,
                1.0,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target: original_node,
                        data: None,
                    },
                )],
            );
            let focus = ctx
                .memory(|memory| memory.focused())
                .expect("actual focus action accepted");
            let revealed = frame(&mut panel, &ctx, 2.0, vec![]);
            let revealed_node = target(&revealed.0, index);
            let pointer = revealed.2.center();
            let _ = frame(
                &mut panel,
                &ctx,
                3.0,
                vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        // Point deltas below 8 are already smooth in egui. A larger
                        // delta leaves input interpolation pending even when the
                        // ScrollArea's scroll-to animation is disabled.
                        delta: egui::vec2(0.0, -6.0),
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            );
            let wheel = frame(&mut panel, &ctx, 4.0, vec![]);
            let settled = frame(&mut panel, &ctx, 5.0, vec![]);
            let held = ctx.memory(|memory| memory.has_focus(focus));
            let unchanged = (
                panel.selection.mode,
                panel.selection.interface.clone(),
                panel.selection.revision,
                panel.selection.refresh,
                panel.edit.clone(),
            ) == original;
            stop(&mut panel);
            assert_eq!(initial.1, 0.0);
            assert_eq!(revealed_node.0, original_node);
            assert!(
                revealed.1 > 0.0 && revealed.2.contains_rect(revealed_node.1),
                "focus alone must reveal the whole control inside the fixed body clip: {index} {:?} {:?}",
                revealed.2,
                revealed_node.1
            );
            assert!(
                wheel.1 > revealed.1 + 1.0,
                "held focus must allow a later body wheel"
            );
            assert!(
                (settled.1 - wheel.1).abs() < 0.5,
                "held focus must not snap back on later frames: index={index} revealed={} wheel={} settled={}",
                revealed.1,
                wheel.1,
                settled.1
            );
            assert!(
                held && unchanged,
                "focus/wheel cannot select, refresh or apply the draft"
            );
        }
    }

    fn stop(panel: &mut MetricsPanel) {
        panel.request_stop();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !panel.stopped() {
            panel.poll(&egui::Context::default());
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn invalid_interface_preserves_selection_and_empty_disables() {
        let mut panel = MetricsPanel::new();
        panel.edit = "lo".into();
        panel.apply_interface();
        let revision = panel.selection.revision;
        panel.edit = "invalid\ninterface".into();
        panel.apply_interface();
        assert!(panel.error);
        assert_eq!(panel.selection.interface, "lo");
        assert_eq!(panel.selection.revision, revision);
        panel.edit.clear();
        panel.apply_interface();
        assert!(!panel.error);
        assert!(panel.selection.interface.is_empty());
        assert!(panel.latest.is_none());
        stop(&mut panel);
    }
    #[test]
    fn root_change_clears_evidence_and_bounds_os_bytes() {
        let mut panel = MetricsPanel::new();
        panel.set_root(std::env::temp_dir());
        let revision = panel.selection.revision;
        panel.latest = Some(Sample {
            revision,
            at: Instant::now(),
            free: Some(7),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
        panel.set_root(std::env::temp_dir().join("synthetic-metrics-no-io"));
        assert!(panel.latest.is_none());
        assert_ne!(panel.selection.revision, revision);
        panel.set_root(PathBuf::from(format!("/{}", "x".repeat(32769))));
        assert!(panel.selection.root.is_none());
        stop(&mut panel);
    }
    #[test]
    fn stale_sample_is_not_retained() {
        let mut panel = MetricsPanel::new();
        panel.latest = Some(Sample {
            revision: 0,
            at: Instant::now() - Duration::from_secs(4),
            free: Some(9),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
        panel.poll(&egui::Context::default());
        assert!(panel.latest.is_none());
        stop(&mut panel);
    }
    #[test]
    fn old_revision_result_never_overwrites_new_selection() {
        let mut panel = MetricsPanel::new();
        panel.results = Arc::new(Mutex::new(Mailbox::default()));
        panel.selection.revision = 3;
        panel.results.lock().unwrap().publish(Sample {
            revision: 2,
            at: Instant::now(),
            free: Some(9),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
        panel.poll(&egui::Context::default());
        assert!(panel.latest.is_none());
        stop(&mut panel);
    }
    #[test]
    fn panic_ack_and_stop_block_new_selection() {
        let signals = Arc::new(Signals::default());
        supervise(signals.clone(), || panic!("synthetic metrics panic"));
        assert!(signals.stopped.load(Ordering::Acquire));
        assert!(signals.failed.load(Ordering::Acquire));
        let mut panel = MetricsPanel::new();
        panel.request_stop();
        panel.edit = "lo".into();
        panel.apply_interface();
        panel.set_root(std::env::temp_dir());
        assert!(panel.selection.interface.is_empty());
        assert!(panel.selection.root.is_none());
        stop(&mut panel);
    }

    fn sample(at: Instant) -> Sample {
        Sample {
            revision: 7,
            at,
            free: Some(10 * 1073741824),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        }
    }
    #[test]
    fn identical_display_keeps_freshness_without_wake_and_overwrites_unconsumed() {
        let before = Instant::now() - Duration::from_secs(4);
        let now = Instant::now();
        let mut mailbox = Mailbox::default();
        assert!(mailbox.publish(sample(before)));
        let mut next = sample(now);
        next.free = Some(10 * 1073741824 + 100);
        assert!(!mailbox.publish(next));
        assert_eq!(mailbox.latest.as_ref().unwrap().at, now);
        assert_eq!(
            mailbox.latest.as_ref().unwrap().free,
            Some(10 * 1073741824 + 100)
        );
        let mut panel = MetricsPanel::new();
        panel.selection.revision = 7;
        panel.pending = false;
        panel.latest = Some(sample(before));
        *panel.results.lock().unwrap() = mailbox;
        panel.poll(&egui::Context::default());
        assert_eq!(panel.latest.as_ref().unwrap().at, now);
        stop(&mut panel);
    }
    #[test]
    fn display_revision_unknown_and_failure_changes_each_wake() {
        let now = Instant::now();
        let mut mailbox = Mailbox::default();
        assert!(mailbox.publish(sample(now)));
        let mut next = sample(now);
        next.free = Some(11 * 1073741824);
        assert!(mailbox.publish(next));
        let mut next = sample(now);
        next.revision = 8;
        assert!(mailbox.publish(next));
        let mut next = sample(now);
        next.free = None;
        assert!(mailbox.publish(next.clone()));
        next.observation_failed.0 = true;
        assert!(mailbox.publish(next.clone()));
        next.at = Instant::now();
        assert!(!mailbox.publish(next));
    }

    #[test]
    fn rate_display_rounding_and_network_failure_are_distinct() {
        let mut mailbox = Mailbox::default();
        let mut next = sample(Instant::now());
        next.rate = Some(NetworkRate {
            received_bytes_per_second: 1024.0,
            sent_bytes_per_second: 2048.0,
        });
        assert!(mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second += 1.0;
        assert!(!mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second = 2048.0;
        assert!(mailbox.publish(next.clone()));
        next.observation_failed.1 = true;
        assert!(mailbox.publish(next));
    }
    #[test]
    fn decimal_rate_carry_wakes_only_when_visible_label_changes() {
        let mut mailbox = Mailbox::default();
        let mut next = sample(Instant::now());
        next.rate = Some(NetworkRate {
            received_bytes_per_second: 999_940.0,
            sent_bytes_per_second: 0.0,
        });
        assert!(mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second = 999_949.0;
        assert!(!mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second = 999_950.0;
        assert!(mailbox.publish(next.clone()));
        assert_eq!(
            display_key(&next).rate,
            Some(("1.0 MB/s".to_owned(), "0 B/s".to_owned()))
        );
        next.rate.as_mut().unwrap().received_bytes_per_second = 1_000_010.0;
        assert!(!mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second = f64::NAN;
        assert!(mailbox.publish(next.clone()));
        next.rate.as_mut().unwrap().received_bytes_per_second = f64::INFINITY;
        assert!(!mailbox.publish(next));
    }

    #[test]
    fn panic_and_normal_stop_ack_request_terminal_repaint() {
        for panic in [false, true] {
            let signals = Arc::new(Signals::default());
            let context = egui::Context::default();
            let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let callback = wakes.clone();
            context.set_request_repaint_callback(move |_| {
                callback.fetch_add(1, Ordering::Relaxed);
            });
            *signals.context.lock().unwrap() = Some(context);
            supervise(signals.clone(), || {
                if panic {
                    panic!("owned synthetic metrics panic");
                }
            });
            assert!(signals.stopped.load(Ordering::Acquire));
            assert_eq!(signals.failed.load(Ordering::Acquire), panic);
            assert!(wakes.load(Ordering::Relaxed) > 0);
        }
    }
    #[test]
    fn fresh_metrics_survive_a_paused_ui_consumer() {
        let mut panel = MetricsPanel::new();
        let context = egui::Context::default();
        panel.poll(&context);
        thread::sleep(STALE + INTERVAL + Duration::from_millis(250));
        panel.poll(&context);
        let fresh = panel
            .latest
            .as_ref()
            .is_some_and(|sample| sample.at.elapsed() < INTERVAL);
        stop(&mut panel);
        assert!(
            fresh,
            "latest-only observation must stay fresh while UI pauses"
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    #[test]
    fn actual_unsupported_reason_precedes_inert_input_at_200_percent() {
        let mut panel = MetricsPanel::new();
        panel.edit = "saved-interface".into();
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        ctx.set_pixels_per_point(2.0);
        let mut reason_rect = None;
        let mut input_rect = None;
        let mut reason_clip = None;
        for frame in 0..4 {
            let output = ctx.run(
                egui::RawInput {
                    time: Some(frame as f64 * 0.02),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        panel.show_settings(ui, true);
                    });
                },
            );
            if let Some(tree) = &output.platform_output.accesskit_update
                && let Some((_, node)) = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.role() == egui::accesskit::Role::TextInput)
            {
                assert!(node.is_disabled());
                let bounds = node.bounds().expect("input bounds");
                input_rect = Some(egui::Rect::from_min_max(
                    egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                    egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                ));
            }
            for clipped in &output.shapes {
                let mut found = vec![];
                network_text_shapes(&clipped.shape, c::NETWORK_COUNTER_UNAVAILABLE, &mut found);
                if let Some(shape) = found.first() {
                    reason_rect = Some(egui::Rect::from_min_size(shape.pos, shape.galley.size()));
                    reason_clip = Some(clipped.clip_rect);
                }
            }
        }
        let reason = reason_rect.expect("actual painted platform reason");
        let input = input_rect.unwrap();
        let clip = reason_clip.unwrap();
        let saved = panel.edit.clone();
        stop(&mut panel);
        assert_eq!(ctx.content_rect().width(), 400.0);
        assert_eq!(ctx.pixels_per_point(), 2.0);
        assert!(
            reason.bottom() <= input.top(),
            "reason must precede disabled field"
        );
        assert!(
            clip.contains_rect(reason),
            "reason must fit the actual 200% viewport"
        );
        assert_eq!(saved, "saved-interface");
    }
    // Append inside accessible_metrics.rs existing #[cfg(test)] mod tests.
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    #[test]
    fn actual_unsupported_network_density_disclosure_click_preserves_disabled_saved_controls() {
        let mut panel = MetricsPanel::new();
        panel.edit = "saved-interface".into();
        let saved = panel.edit.clone();
        let selection = panel.selection.interface.clone();
        let revision = panel.selection.revision;
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let first = network_frame(&mut panel, &ctx, 1.0, true, None, vec![]);
        let tree = first.platform_output.accesskit_update.as_ref().unwrap();
        let (detail_id, detail) = tree
            .nodes
            .iter()
            .find(|(_, n)| {
                n.role() == egui::accesskit::Role::Button
                    && n.label() == Some(c::NETWORK_COUNTER_UNAVAILABLE_DETAILS)
            })
            .expect("actual native disclosure action");
        assert!(detail.supports_action(egui::accesskit::Action::Click));
        assert!(!detail.is_disabled());
        let detail_id = *detail_id;
        let click = || {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                target: detail_id,
                action: egui::accesskit::Action::Click,
                data: None,
            })
        };
        network_frame(&mut panel, &ctx, 1.1, true, None, vec![click()]);
        // Settle real egui disclosure animation rather than infer state from click dispatch.
        let expanded = network_frame(&mut panel, &ctx, 1.6, true, None, vec![]);
        network_frame(&mut panel, &ctx, 1.7, true, None, vec![click()]);
        let collapsed = network_frame(&mut panel, &ctx, 2.2, true, None, vec![]);
        let unchanged = panel.edit == saved
            && panel.selection.interface == selection
            && panel.selection.revision == revision;
        stop(&mut panel);
        assert!(
            unchanged,
            "disclosure cannot edit/apply saved interface or advance selection revision"
        );
        for output in [&first, &expanded, &collapsed] {
            assert!(
                painted_metric_text(output, c::NETWORK_COUNTER_UNAVAILABLE),
                "short unavailable status is actually painted in every disclosure state"
            );
            let (_, _, disabled) = network_input(output);
            assert!(disabled, "Windows unsupported input remains disabled");
            let tree = output.platform_output.accesskit_update.as_ref().unwrap();
            let input = &tree
                .nodes
                .iter()
                .find(|(_, n)| n.role() == egui::accesskit::Role::TextInput)
                .unwrap()
                .1;
            assert_eq!(
                input.value(),
                Some(saved.as_str()),
                "actual provider saved-value readback"
            );
            let apply = &tree
                .nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button
                        && n.label() == Some(c::APPLY_INTERFACE)
                })
                .unwrap()
                .1;
            assert!(
                apply.is_disabled(),
                "Apply is still unsupported, not a new implemented function"
            );
            let disclosure = tree
                .nodes
                .iter()
                .find(|(id, n)| {
                    *id == detail_id && n.label() == Some(c::NETWORK_COUNTER_UNAVAILABLE_DETAILS)
                })
                .expect("disclosure native identity stable across actual frames");
            assert!(!disclosure.1.is_disabled());
        }
        assert!(
            !painted_metric_text(&first, c::NETWORK_COUNTER_UNAVAILABLE_HELP),
            "full help is not painted before actual disclosure click"
        );
        assert!(
            painted_metric_text(&expanded, c::NETWORK_COUNTER_UNAVAILABLE_HELP),
            "actual click reveals entire original explanation galley"
        );
        assert!(
            !painted_metric_text(&collapsed, c::NETWORK_COUNTER_UNAVAILABLE_HELP),
            "second actual click collapses full help"
        );
    }
}

#[cfg(test)]
mod host_freshness_tests {
    use super::*;
    fn panel() -> (MetricsPanel, mpsc::Receiver<WorkerCommand>) {
        // Actual panel/mailbox with no worker, external I/O or timing-sensitive producer.
        let (commands, receiver) = mpsc::sync_channel(1);
        let mut value = MetricsPanel {
            commands,
            results: Arc::new(Mutex::new(Mailbox::default())),
            signals: Arc::new(Signals::default()),
            selection: Selection::default(),
            pending: false,
            edit: String::new(),
            error: false,
            latest: None,
            latest_key: None,
            host_freshness_timer: true,
            pending_visible: None,
            presented_visible: None,
        };
        value.use_host_freshness_timer();
        (value, receiver)
    }
    #[cfg(windows)]
    #[test]
    fn actual_footer_priming_preserves_errors_stale_stop_and_valid_rate() {
        fn labels(panel: &mut MetricsPanel) -> Vec<String> {
            let context = egui::Context::default();
            crate::accessible::configure_context(&context, crate::accessible::font_bytes());
            context.enable_accesskit();
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 180.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| panel.status(ui));
                },
            );
            output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == egui::accesskit::Role::Label)
                .filter_map(|(_, node)| node.value().map(str::to_owned))
                .collect()
        }
        fn sample(panel: &MetricsPanel, state: NetworkState) -> Sample {
            Sample {
                revision: panel.selection.revision,
                at: Instant::now(),
                free: None,
                rate: None,
                observation_failed: (false, false),
                network_view: NetworkView {
                    state,
                    ..Default::default()
                },
            }
        }
        let (mut panel, receiver) = panel();
        panel.selection.mode = NetworkMode::Auto;
        panel.pending = true;
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_PREPARING),
            "queued first selection is visibly preparing"
        );
        panel.pending = false;
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_PREPARING),
            "dispatched first selection still awaits its initial sample"
        );
        panel.latest = Some(sample(&panel, NetworkState::Observing));
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_PREPARING),
            "fresh healthy observation without a rate is baseline preparation"
        );
        panel.latest.as_mut().unwrap().observation_failed.1 = true;
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN),
            "even raw Observing must not hide network failure"
        );
        for (mode, state, expected) in [
            (
                NetworkMode::Auto,
                NetworkState::Failed,
                c::NETWORK_AUTOMATIC_FAILED,
            ),
            (
                NetworkMode::Manual,
                NetworkState::Failed,
                c::NETWORK_UNKNOWN,
            ),
            (
                NetworkMode::Auto,
                NetworkState::NoRoute,
                c::NETWORK_NO_ACTIVE,
            ),
            (NetworkMode::Off, NetworkState::Off, c::NETWORK_OFF_STATUS),
            (
                NetworkMode::Auto,
                NetworkState::Unavailable,
                c::NETWORK_UNKNOWN,
            ),
        ] {
            panel.selection.mode = mode;
            panel.latest = Some(sample(&panel, state));
            let actual = labels(&mut panel);
            assert!(actual.iter().any(|text| text == expected));
            assert!(!actual.iter().any(|text| text == c::NETWORK_PREPARING));
        }
        panel.selection.mode = NetworkMode::Auto;
        panel.latest = Some(sample(&panel, NetworkState::Observing));
        panel.latest.as_mut().unwrap().at = Instant::now() - STALE - Duration::from_secs(1);
        panel.pending = true;
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN),
            "stale cached baseline is not pending preparation"
        );
        panel.latest = Some(sample(&panel, NetworkState::Observing));
        panel.latest.as_mut().unwrap().revision = panel.selection.revision.wrapping_add(1);
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN),
            "wrong-revision sample is never shown as preparing"
        );
        panel.latest = None;
        panel.signals.failed.store(true, Ordering::Release);
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN)
        );
        panel.signals.failed.store(false, Ordering::Release);
        panel.signals.stop.store(true, Ordering::Release);
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN)
        );
        panel.signals.stop.store(false, Ordering::Release);
        panel.signals.stopped.store(true, Ordering::Release);
        assert!(
            labels(&mut panel)
                .iter()
                .any(|text| text == c::NETWORK_UNKNOWN)
        );
        panel.signals.stopped.store(false, Ordering::Release);
        panel.pending = false;
        panel.latest = Some(sample(&panel, NetworkState::Observing));
        panel.latest.as_mut().unwrap().rate = Some(NetworkRate {
            received_bytes_per_second: 1024.0,
            sent_bytes_per_second: 2048.0,
        });
        let actual = labels(&mut panel);
        assert!(
            actual
                .iter()
                .any(|text| text == &format!("{} 1.0 KB/s · {} 2.0 KB/s", c::RECEIVED, c::SENT))
        );
        assert!(!actual.iter().any(|text| text == c::NETWORK_PREPARING));
        // Actual production Label/value and repaint key must share the decimal display.
        for (received, sent, expected_received, expected_sent) in [
            (999_940.0, 0.0, "999.9 KB/s", "0 B/s"),
            (999_950.0, 1_000_000_000.0, "1.0 MB/s", "1.0 GB/s"),
            (f64::NAN, -1.0, c::UNKNOWN, c::UNKNOWN),
            (f64::MAX, 0.001, "≥ 1000 GB/s", "< 0.1 B/s"),
        ] {
            let latest = panel.latest.as_mut().unwrap();
            latest.rate = Some(NetworkRate {
                received_bytes_per_second: received,
                sent_bytes_per_second: sent,
            });
            assert_eq!(
                display_key(latest).rate,
                Some((expected_received.to_owned(), expected_sent.to_owned()))
            );
            let actual = labels(&mut panel);
            assert!(
                actual.iter().any(|text| text
                    == &format!(
                        "{} {} · {} {}",
                        c::RECEIVED,
                        expected_received,
                        c::SENT,
                        expected_sent
                    )),
                "actual footer labels: {actual:?}"
            );
        }
        assert!(
            matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "actual footer rendering does not dispatch work"
        );
    }

    fn publish(panel: &MetricsPanel, at: Instant) {
        panel.results.lock().unwrap().publish(Sample {
            revision: panel.selection.revision,
            at,
            free: Some(1_000_000),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
    }
    fn render(panel: &mut MetricsPanel) {
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| panel.status(ui));
        });
    }
    #[test]
    fn same_display_refresh_rearms_without_advancing_rendered_key() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        assert!(p.check_freshness(t));
        render(&mut p);
        p.frame_presented();
        let shown = p.presented_visible.clone();
        publish(&p, t + INTERVAL);
        assert!(!p.check_freshness(t + INTERVAL));
        assert!(p.presented_visible == shown);
        assert_eq!(
            p.freshness_deadline(t + INTERVAL),
            Some(t + INTERVAL + STALE + Duration::from_millis(1))
        );
    }
    #[test]
    fn stall_unknown_and_identical_value_return_require_real_frame_commit() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        assert!(p.check_freshness(t));
        render(&mut p);
        p.frame_presented();
        assert!(p.check_freshness(t + STALE + INTERVAL));
        render(&mut p);
        p.frame_presented();
        assert_eq!(
            p.freshness_deadline(t + STALE + INTERVAL),
            Some(t + STALE + INTERVAL + INTERVAL)
        );
        publish(&p, t + STALE + INTERVAL);
        assert!(p.check_freshness(t + STALE + INTERVAL));
        render(&mut p); // An unsuccessful host frame does not call frame_presented.
        assert!(p.check_freshness(t + STALE + INTERVAL));
        p.frame_presented();
        assert!(!p.check_freshness(t + STALE + INTERVAL));
    }
    #[test]
    fn would_block_does_not_adopt_unread_fresh_timestamp_or_extend_expiry() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        p.check_freshness(t);
        render(&mut p);
        p.frame_presented();
        let mailbox = p.results.clone();
        let mut guard = mailbox.lock().unwrap();
        guard.publish(Sample {
            revision: 0,
            at: t + INTERVAL,
            free: Some(1_000_000),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
        assert!(!p.check_freshness(t + INTERVAL));
        assert_eq!(
            p.freshness_deadline(t + INTERVAL),
            Some(t + STALE + Duration::from_millis(1))
        );
        assert!(p.check_freshness(t + STALE + INTERVAL));
        drop(guard);
    }
    #[test]
    fn poison_failure_and_revision_mismatch_never_keep_fresh_display() {
        let (mut p, rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        p.check_freshness(t);
        render(&mut p);
        p.frame_presented();
        p.selection.revision = 1;
        assert!(p.check_freshness(t));
        assert!(matches!(
            p.visible_key(t),
            VisibleKey::Unknown { revision: 1, .. }
        ));
        let mailbox = p.results.clone();
        let _ = thread::spawn(move || {
            let _lock = mailbox.lock().unwrap();
            panic!("synthetic mailbox poison");
        })
        .join();
        assert!(p.check_freshness(t));
        assert!(p.signals.failed.load(Ordering::Acquire));
        assert!(p.signals.stop.load(Ordering::Acquire));
        assert!(matches!(rx.try_recv(), Ok(WorkerCommand::StopWake)));
        assert!(p.latest.is_none());
    }
    #[test]
    fn pending_stop_is_generic_and_terminal_worker_ends_freshness_retry() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        assert_eq!(p.freshness_deadline(t), Some(t + INTERVAL));
        p.pending = true;
        assert!(p.check_freshness(t));
        assert_eq!(p.freshness_deadline(t), None);
        p.pending = false;
        p.signals.stop.store(true, Ordering::Release);
        assert!(p.check_freshness(t));
        assert_eq!(p.freshness_deadline(t), None);
        p.signals.stopped.store(true, Ordering::Release);
        assert_eq!(p.freshness_deadline(t), None);
    }
    #[test]
    fn actual_status_does_not_commit_without_host_success() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        p.check_freshness(t);
        render(&mut p);
        assert!(p.pending_visible.is_some());
        assert!(p.presented_visible.is_none());
        assert!(p.check_freshness(t));
        p.frame_presented();
        assert!(!p.check_freshness(t));
    }
    #[test]
    fn exact_stale_boundary_is_fresh_then_one_millisecond_later_unknown() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        p.check_freshness(t);
        render(&mut p);
        p.frame_presented();
        assert!(!p.check_freshness(t + STALE));
        assert!(matches!(p.visible_key(t + STALE), VisibleKey::Fresh(_)));
        assert_eq!(
            p.freshness_deadline(t + STALE),
            Some(t + STALE + Duration::from_millis(1))
        );
        assert!(p.check_freshness(t + STALE + Duration::from_millis(1)));
        assert!(matches!(
            p.visible_key(t + STALE + Duration::from_millis(1)),
            VisibleKey::Unknown { .. }
        ));
    }
    #[test]
    fn success_commits_rendered_a_not_mailbox_b_arriving_before_present() {
        let (mut p, _rx) = panel();
        let t = Instant::now();
        publish(&p, t);
        assert!(p.check_freshness(t));
        render(&mut p);
        let rendered_a = p.pending_visible.clone();
        p.results.lock().unwrap().publish(Sample {
            revision: 0,
            at: t + INTERVAL,
            free: Some(1_073_741_824),
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        });
        assert!(p.check_freshness(t + INTERVAL));
        p.frame_presented();
        assert!(p.presented_visible == rendered_a);
        assert!(p.check_freshness(t + INTERVAL));
        render(&mut p);
        p.frame_presented();
        assert!(!p.check_freshness(t + INTERVAL));
    }
    // Insert inside existing host_freshness_tests: reuse its worker-free panel/mailbox.
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn input_semantics_frame(
        panel: &mut MetricsPanel,
        context: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        context.run(
            egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 1000.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |context| {
                egui::CentralPanel::default().show(context, |ui| panel.show_settings(ui, true));
            },
        )
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn input_semantics_node(
        output: &egui::FullOutput,
    ) -> (egui::accesskit::NodeId, &egui::accesskit::Node) {
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("actual provider update");
        let mut inputs = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == egui::accesskit::Role::TextInput);
        let (id, node) = inputs.next().expect("actual Interface Edit");
        assert!(inputs.next().is_none(), "one actual Interface Edit");
        (*id, node)
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn input_select_all() -> egui::Event {
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
        }
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn input_focus(id: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Focus,
            target: id,
            data: None,
        })
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn actual_manual_interface_invalid_edit_refusal_correction_and_empty_clear_semantics() {
        let (mut panel, _receiver) = panel();
        panel.selection.mode = NetworkMode::Manual;
        panel.selection.interface = "lo".into();
        panel.edit = "lo".into();
        let original = (
            panel.selection.interface.clone(),
            panel.selection.mode,
            panel.selection.revision,
        );
        let context = egui::Context::default();
        crate::accessible::configure_context(&context, crate::accessible::font_bytes());
        context.enable_accesskit();
        let first = input_semantics_frame(&mut panel, &context, 0.0, vec![]);
        let input = input_semantics_node(&first).0;
        let apply = first
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::APPLY_INTERFACE)
            })
            .expect("actual stable Apply node")
            .0;
        input_semantics_frame(&mut panel, &context, 0.1, vec![input_focus(input)]);
        let original_focus = context
            .memory(|memory| memory.focused())
            .expect("actual input focus");
        #[cfg(windows)]
        let bad = "😀".repeat(129); // 129 chars fit TextEdit256, but258 UTF16units violate alias syntax.
        #[cfg(not(windows))]
        let bad = "bad/name".to_owned();
        let invalid = input_semantics_frame(
            &mut panel,
            &context,
            0.2,
            vec![input_select_all(), egui::Event::Text(bad.clone())],
        );
        assert_eq!(
            panel.edit, bad,
            "actual Edit must preserve the invalid draft"
        );
        let (id, node) = input_semantics_node(&invalid);
        assert_eq!(id, input);
        assert_eq!(node.invalid(), Some(egui::accesskit::Invalid::True));
        assert_eq!(node.description(), Some(c::INVALID_INTERFACE));
        assert!(panel.error, "live syntax error has visible feedback");
        let tree = invalid.platform_output.accesskit_update.as_ref().unwrap();
        let error = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::INVALID_INTERFACE)
            })
            .expect("actual visible syntax error label")
            .1
            .bounds()
            .unwrap();
        let input_bounds = node.bounds().expect("actual input bounds");
        let apply_bounds = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::APPLY_INTERFACE)
            })
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(
            error.y0 >= input_bounds.y1 - 0.5 && error.y1 <= apply_bounds.y0 + 0.5,
            "actual error is between input and Apply"
        );
        #[cfg(windows)]
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::NETWORK_MANUAL_HELP)),
            "invalid feedback replaces normal help"
        );
        assert!(context.memory(|memory| memory.has_focus(original_focus)));
        input_semantics_frame(
            &mut panel,
            &context,
            0.3,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target: apply,
                    data: None,
                },
            )],
        );
        assert!(panel.error);
        assert!(
            (
                panel.selection.interface.clone(),
                panel.selection.mode,
                panel.selection.revision
            ) == original,
            "actual invalid Apply never dispatches/replaces the accepted selection"
        );
        assert!(!panel.pending);
        input_semantics_frame(&mut panel, &context, 0.4, vec![input_focus(input)]);
        let corrected = input_semantics_frame(
            &mut panel,
            &context,
            0.5,
            vec![input_select_all(), egui::Event::Text("lo".into())],
        );
        let (id, node) = input_semantics_node(&corrected);
        assert_eq!(id, input);
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
        assert!(
            !corrected
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::INVALID_INTERFACE)),
            "correction removes actual visible error"
        );
        #[cfg(windows)]
        assert!(
            corrected
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::NETWORK_MANUAL_HELP)),
            "correction restores normal Windows help"
        );
        assert!(
            !panel.error,
            "correction clears visible error in the same actual edit frame"
        );
        assert!(
            (
                panel.selection.interface.clone(),
                panel.selection.mode,
                panel.selection.revision
            ) == original
        );
        assert!(context.memory(|memory| memory.has_focus(original_focus)));
        let empty = input_semantics_frame(
            &mut panel,
            &context,
            0.6,
            vec![
                input_select_all(),
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert!(panel.edit.is_empty());
        let (id, node) = input_semantics_node(&empty);
        assert!(
            !empty
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::INVALID_INTERFACE)),
            "empty draft removes actual visible error"
        );
        assert_eq!(id, input);
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
        assert!(!panel.error);
        assert!(
            (
                panel.selection.interface.clone(),
                panel.selection.mode,
                panel.selection.revision
            ) == original,
            "clearing the draft does not auto-select Off before explicit Apply"
        );
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn actual_invalid_interface_reveals_input_and_error_in_scroll_viewport() {
        fn frame(
            panel: &mut MetricsPanel,
            context: &egui::Context,
            time: f64,
            events: Vec<egui::Event>,
        ) -> (egui::FullOutput, egui::Rect) {
            let mut viewport = egui::Rect::NOTHING;
            let output = context.run(
                egui::RawInput {
                    time: Some(time),
                    focused: true,
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 125.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("invalid-feedback-scroll")
                            .animated(false)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                viewport = ui.clip_rect();
                                ui.add_space(480.0);
                                panel.show_settings(ui, true);
                                ui.add_space(480.0);
                            });
                    });
                },
            );
            (output, viewport)
        }
        let (mut panel, _receiver) = panel();
        let context = egui::Context::default();
        crate::accessible::configure_context(&context, crate::accessible::font_bytes());
        context.enable_accesskit();
        let first = frame(&mut panel, &context, 0.0, vec![]).0;
        let id = input_semantics_node(&first).0;
        frame(&mut panel, &context, 0.1, vec![input_focus(id)]);
        #[cfg(windows)]
        let bad = "😀".repeat(129);
        #[cfg(not(windows))]
        let bad = "bad/name".to_owned();
        frame(
            &mut panel,
            &context,
            0.2,
            vec![input_select_all(), egui::Event::Text(bad)],
        );
        let (settled, viewport) = frame(&mut panel, &context, 0.3, vec![]);
        let tree = settled.platform_output.accesskit_update.as_ref().unwrap();
        let input = input_semantics_node(&settled).1.bounds().unwrap();
        let error = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Label
                    && node.value() == Some(c::INVALID_INTERFACE)
            })
            .expect("actual visible invalid feedback")
            .1
            .bounds()
            .unwrap();
        let label = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Label && node.value() == Some(c::INTERFACE)
            })
            .expect("actual interface label")
            .1
            .bounds()
            .unwrap();
        assert!(
            label.y0 >= f64::from(viewport.top()) - 1.0
                && input.y0 >= label.y1
                && error.y1 <= f64::from(viewport.bottom()) + 1.0,
            "actual ScrollArea reveals label, input and complete error together"
        );
        assert!(panel.error);
        assert!(
            !panel.pending,
            "render/reveal must not dispatch network selection"
        );
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn actual_syntax_valid_missing_adapter_is_worker_failure_without_edit_invalid_semantics() {
        struct MissingAdapter;
        impl MetricsIo for MissingAdapter {
            fn volume(&mut self, _: &std::path::Path) -> std::io::Result<Option<u64>> {
                panic!("no volume IO authorized");
            }
            #[cfg(windows)]
            fn aliases(&mut self) -> std::io::Result<Vec<String>> {
                Ok(Vec::new())
            }
            fn automatic(&mut self) -> std::io::Result<Option<String>> {
                panic!("manual selection must not query automatic route");
            }
            fn sample(&mut self, alias: &str) -> std::io::Result<Option<metrics::NetworkSample>> {
                assert_eq!(alias, "dlpad-synthetic-missing");
                Err(std::io::ErrorKind::NotFound.into())
            }
            fn reset(&mut self) {}
        }
        let (mut panel, _receiver) = panel();
        panel.edit = "dlpad-synthetic-missing".into();
        let context = egui::Context::default();
        crate::accessible::configure_context(&context, crate::accessible::font_bytes());
        context.enable_accesskit();
        let first = input_semantics_frame(&mut panel, &context, 0.0, vec![]);
        let input = input_semantics_node(&first).0;
        let apply = first
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label() == Some(c::APPLY_INTERFACE)
            })
            .expect("actual Apply")
            .0;
        let clicked = input_semantics_frame(
            &mut panel,
            &context,
            0.1,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target: apply,
                    data: None,
                },
            )],
        );
        assert!(panel.selection.mode == NetworkMode::Manual);
        assert_eq!(panel.selection.interface, "dlpad-synthetic-missing");
        assert!(panel.pending);
        assert!(!panel.error);
        assert_eq!(input_semantics_node(&clicked).0, input);
        let sample = observe_selection_with_io(
            &panel.selection,
            &mut None,
            &mut MissingAdapter,
            &mut Discovery::default(),
        );
        assert!(sample.network_view.state == NetworkState::Failed);
        assert!(sample.observation_failed.1);
        panel.latest = Some(sample);
        let failed = input_semantics_frame(&mut panel, &context, 0.2, vec![]);
        let (id, node) = input_semantics_node(&failed);
        assert_eq!(id, input);
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
        assert!(!panel.error, "worker NotFound cannot become a syntax error");
        assert_eq!(panel.edit, "dlpad-synthetic-missing");
        #[cfg(windows)]
        assert!(
            failed
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| {
                    node.role() == egui::accesskit::Role::Label
                        && node.value() == Some(c::NETWORK_MANUAL_FAILED)
                }),
            "actual Windows manual worker failure remains separate visible feedback"
        );
    }
}

// CPU host tests need real panel/mailbox/ACK state without exposing production internals.
#[cfg(all(test, feature = "cpu"))]
#[allow(
    dead_code,
    reason = "The alternative harness compiles shared metrics, while CPU Host alone consumes this fixture."
)]
pub(crate) mod due_test_fixture {
    use super::*;
    pub(crate) struct Fixture {
        pub(crate) panel: MetricsPanel,
        _receiver: mpsc::Receiver<WorkerCommand>,
        context: egui::Context,
        wakes: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Fixture {
        pub(crate) fn new() -> Self {
            let (commands, receiver) = mpsc::sync_channel(1);
            let context = egui::Context::default();
            let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let sink = wakes.clone();
            context.set_request_repaint_callback(move |_| {
                sink.fetch_add(1, Ordering::Relaxed);
            });
            let signals = Arc::new(Signals::default());
            *signals.context.lock().unwrap() = Some(context.clone());
            let panel = MetricsPanel {
                commands,
                results: Arc::new(Mutex::new(Mailbox::default())),
                signals,
                selection: Selection::default(),
                pending: false,
                edit: String::new(),
                error: false,
                latest: None,
                latest_key: None,
                host_freshness_timer: true,
                pending_visible: None,
                presented_visible: None,
            };
            Self {
                panel,
                _receiver: receiver,
                context,
                wakes,
            }
        }
        pub(crate) fn publish(&self, at: Instant, free: u64) {
            self.panel.results.lock().unwrap().publish(Sample {
                revision: self.panel.selection.revision,
                at,
                free: Some(free),
                rate: None,
                observation_failed: (false, false),
                network_view: NetworkView::default(),
            });
        }
        pub(crate) fn render(&mut self) {
            let _ = self.context.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| self.panel.status(ui));
            });
        }
        pub(crate) fn pump(&mut self) {
            self.panel.poll(&self.context);
        }
        pub(crate) fn acknowledge_worker(&self) {
            drop(Ack(self.panel.signals.clone()));
        }
        pub(crate) fn wake_count(&self) -> usize {
            self.wakes.load(Ordering::Relaxed)
        }
        pub(crate) fn unknown_was_presented(&self) -> bool {
            matches!(
                self.panel.presented_visible.as_ref(),
                Some(VisibleKey::Unknown { .. })
            )
        }
    }
}

#[cfg(test)]
mod stop_wake_worker_tests {
    use super::*;
    fn selection(revision: u64) -> WorkerCommand {
        WorkerCommand::Selection(Command {
            selection: Selection {
                revision,
                ..Selection::default()
            },
            context: egui::Context::default(),
        })
    }
    fn sample(selection: &Selection) -> Sample {
        Sample {
            revision: selection.revision,
            at: Instant::now(),
            free: None,
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        }
    }
    #[test]
    fn stop_empty_queue_notifies_and_full_queue_preserves_accepted_selection() {
        let signals = Signals::default();
        let (tx, rx) = mpsc::sync_channel(1);
        stop_worker(&signals, &tx);
        assert!(signals.stop.load(Ordering::Acquire));
        assert!(matches!(rx.try_recv(), Ok(WorkerCommand::StopWake)));
        tx.try_send(selection(41)).ok().unwrap();
        stop_worker(&signals, &tx);
        assert!(matches!(
            rx.try_recv(),
            Ok(WorkerCommand::Selection(Command {
                selection: Selection { revision: 41, .. },
                ..
            }))
        ));
    }
    #[test]
    fn stop_before_wait_and_disconnected_queue_ack_without_observation() {
        for stopped in [false, true] {
            let signals = Arc::new(Signals::default());
            let (tx, rx) = mpsc::sync_channel(1);
            if stopped {
                tx.try_send(selection(7)).ok().unwrap();
                stop_worker(&signals, &tx);
            } else {
                drop(tx);
            }
            let mailbox = Mutex::new(Mailbox::default());
            let mut calls = 0;
            supervise(signals.clone(), || {
                worker_loop(rx, &signals, &mailbox, &mut |s, _, _sample_due| {
                    calls += 1;
                    sample(s)
                })
            });
            assert_eq!(calls, 0);
            assert!(signals.stopped.load(Ordering::Acquire));
            assert!(!signals.failed.load(Ordering::Acquire));
        }
    }
    #[test]
    fn selection_keeps_sample_deadline_and_idle_stop_ack_is_not_one_second_late() {
        let signals = Arc::new(Signals::default());
        let (tx, rx) = mpsc::sync_channel(1);
        let (observed_tx, observed_rx) = mpsc::sync_channel(4);
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        tx.try_send(selection(1)).ok().unwrap();
        let worker = signals.clone();
        let join = thread::spawn(move || {
            let mailbox = Mutex::new(Mailbox::default());
            supervise(worker.clone(), || {
                worker_loop(rx, &worker, &mailbox, &mut |s, _, sample_due| {
                    let result = sample(s);
                    let _ = observed_tx.try_send((s.revision, sample_due));
                    result
                })
            });
            let _ = done_tx.try_send(());
        });
        // Always stop and join before assertions, including observation failures.
        let first = observed_rx.recv_timeout(Duration::from_secs(3));
        let accepted = tx.try_send(selection(2)).is_ok();
        let early = observed_rx.recv_timeout(Duration::from_millis(50));
        let second = observed_rx.recv_timeout(Duration::from_secs(3));
        let stop_at = Instant::now();
        stop_worker(&signals, &tx);
        let done = done_rx.recv_timeout(Duration::from_millis(500));
        drop(tx);
        join.join().unwrap();
        let first = first.unwrap();
        let second = second.unwrap();
        assert!(accepted);
        assert!(matches!(early, Err(mpsc::RecvTimeoutError::Timeout)));
        assert_eq!((first.0, second.0), (1, 2));
        assert!(second.1.duration_since(first.1) >= INTERVAL);
        assert!(done.is_ok());
        assert!(stop_at.elapsed() < INTERVAL);
        assert!(signals.stopped.load(Ordering::Acquire));
        assert!(!signals.failed.load(Ordering::Acquire));
    }
}

#[cfg(test)]
mod automatic_ui_policy_tests {
    use super::*;
    fn sample() -> Sample {
        Sample {
            revision: 0,
            at: Instant::now(),
            free: None,
            rate: None,
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        }
    }
    #[test]
    fn default_observation_mode_is_windows_auto_and_other_platforms_off() {
        assert!(
            Selection::default().mode
                == if cfg!(windows) {
                    NetworkMode::Auto
                } else {
                    NetworkMode::Off
                }
        );
    }
    #[test]
    fn alias_state_and_list_changes_wake_even_when_rates_are_identical() {
        let mut mailbox = Mailbox::default();
        let mut current = sample();
        assert!(mailbox.publish(current.clone()));
        current.network_view.alias = Some("이더넷 회사망".into());
        assert!(mailbox.publish(current.clone()));
        current.network_view.state = NetworkState::NoRoute;
        assert!(mailbox.publish(current.clone()));
        current.network_view.state = NetworkState::Failed;
        assert!(mailbox.publish(current.clone()));
        current.network_view.state = NetworkState::Off;
        assert!(mailbox.publish(current.clone()));
        current.network_view.list_generation = 1;
        assert!(mailbox.publish(current.clone()));
        current.network_view.list_failed = true;
        assert!(mailbox.publish(current.clone()));
        current.at = Instant::now();
        assert!(!mailbox.publish(current));
    }
    #[test]
    fn off_observation_drops_baseline_without_volume_or_network_io() {
        let mut previous = Some(metrics::NetworkSample {
            interface: "synthetic".into(),
            received_bytes: 10,
            sent_bytes: 10,
            observed_at: Instant::now(),
        });
        let selection = Selection {
            mode: NetworkMode::Off,
            ..Default::default()
        };
        let mut network = metrics::NetworkObserver::default();
        let mut discovery = Discovery::default();
        let current = observe_selection(
            &selection,
            &mut previous,
            Instant::now(),
            &mut network,
            &mut discovery,
        );
        assert!(previous.is_none());
        assert!(current.rate.is_none());
        assert!(current.network_view.state == NetworkState::Off);
        assert_eq!(current.observation_failed, (false, false));
        #[cfg(windows)]
        assert!(
            !discovery.initialized,
            "Off bypasses list discovery as well as counters"
        );
    }
}

#[cfg(test)]
mod injected_worker_regression_tests {
    use super::*;
    use std::collections::VecDeque;
    type NativeSample = metrics::NetworkSample;
    #[derive(Default)]
    struct FakeIo {
        routes: VecDeque<std::io::Result<Option<String>>>,
        samples: VecDeque<std::io::Result<Option<NativeSample>>>,
        #[cfg(windows)]
        names: VecDeque<std::io::Result<Vec<String>>>,
        volume_calls: usize,
        route_calls: usize,
        sample_calls: usize,
        #[cfg(windows)]
        list_calls: usize,
        resets: usize,
    }
    impl MetricsIo for FakeIo {
        fn volume(&mut self, _: &std::path::Path) -> std::io::Result<Option<u64>> {
            self.volume_calls += 1;
            Ok(Some(17))
        }
        #[cfg(windows)]
        fn aliases(&mut self) -> std::io::Result<Vec<String>> {
            self.list_calls += 1;
            self.names.pop_front().unwrap_or_else(|| Ok(Vec::new()))
        }
        fn automatic(&mut self) -> std::io::Result<Option<String>> {
            self.route_calls += 1;
            self.routes.pop_front().expect("unplanned route query")
        }
        fn sample(&mut self, alias: &str) -> std::io::Result<Option<NativeSample>> {
            self.sample_calls += 1;
            let result = self.samples.pop_front().expect("unplanned counter query");
            if let Ok(Some(row)) = &result {
                assert_eq!(row.interface, alias);
            }
            result
        }
        fn reset(&mut self) {
            self.resets += 1;
        }
    }
    fn selection(mode: NetworkMode) -> Selection {
        Selection {
            mode,
            root: Some(PathBuf::from("synthetic-worker-volume")),
            ..Default::default()
        }
    }
    fn raw(alias: &str, at: Instant, bytes: u64) -> NativeSample {
        NativeSample {
            interface: alias.into(),
            observed_at: at,
            received_bytes: bytes,
            sent_bytes: bytes,
        }
    }
    #[test]
    fn automatic_route_change_never_crosses_the_previous_interface_baseline() {
        let t = Instant::now();
        let mut io = FakeIo::default();
        for name in ["A", "A", "B", "B"] {
            io.routes.push_back(Ok(Some(name.into())));
        }
        for row in [
            raw("A", t, 100),
            raw("A", t + INTERVAL, 200),
            raw("B", t + INTERVAL * 2, 10_000),
            raw("B", t + INTERVAL * 3, 10_200),
        ] {
            io.samples.push_back(Ok(Some(row)));
        }
        let selection = selection(NetworkMode::Auto);
        let mut previous = None;
        let mut discovery = Discovery::default();
        let first = observe_selection_with_io(&selection, &mut previous, &mut io, &mut discovery);
        assert!(first.rate.is_none());
        let second = observe_selection_with_io(&selection, &mut previous, &mut io, &mut discovery);
        assert_eq!(second.rate.unwrap().received_bytes_per_second, 100.0);
        let changed = observe_selection_with_io(&selection, &mut previous, &mut io, &mut discovery);
        assert_eq!(changed.network_view.alias.as_deref(), Some("B"));
        assert!(
            changed.rate.is_none(),
            "different aliases cannot create a route-switch spike"
        );
        let next = observe_selection_with_io(&selection, &mut previous, &mut io, &mut discovery);
        assert_eq!(next.rate.unwrap().received_bytes_per_second, 200.0);
        assert_eq!(
            (io.route_calls, io.sample_calls, io.volume_calls),
            (4, 4, 4)
        );
    }
    #[test]
    fn off_does_zero_network_queries_and_keeps_independent_volume_observation() {
        let mut io = FakeIo::default();
        let mut previous = Some(raw("A", Instant::now(), 100));
        let mut discovery = Discovery::default();
        let current = observe_selection_with_io(
            &selection(NetworkMode::Off),
            &mut previous,
            &mut io,
            &mut discovery,
        );
        assert_eq!(
            (io.route_calls, io.sample_calls, io.volume_calls),
            (0, 0, 1)
        );
        #[cfg(windows)]
        assert_eq!(io.list_calls, 0);
        assert_eq!(current.free, Some(17));
        assert_eq!(current.observation_failed, (false, false));
        assert!(current.rate.is_none() && previous.is_none());
        assert!(current.network_view.state == NetworkState::Off);
        assert_eq!(io.resets, 1);
    }
    #[test]
    fn no_route_and_selector_error_clear_baseline_without_discarding_volume() {
        for failed in [false, true] {
            let mut io = FakeIo::default();
            io.routes.push_back(if failed {
                Err(std::io::ErrorKind::PermissionDenied.into())
            } else {
                Ok(None)
            });
            let mut previous = Some(raw("A", Instant::now(), 100));
            let mut discovery = Discovery::default();
            let current = observe_selection_with_io(
                &selection(NetworkMode::Auto),
                &mut previous,
                &mut io,
                &mut discovery,
            );
            assert_eq!(current.free, Some(17));
            assert_eq!(current.observation_failed, (false, failed));
            assert_eq!(
                (io.route_calls, io.sample_calls, io.volume_calls, io.resets),
                (1, 0, 1, 1)
            );
            assert!(previous.is_none() && current.rate.is_none());
            assert!(
                current.network_view.state
                    == if failed {
                        NetworkState::Failed
                    } else {
                        NetworkState::NoRoute
                    }
            );
        }
    }
    #[cfg(windows)]
    #[test]
    fn failed_and_oversized_list_refresh_preserve_last_successful_bounded_names() {
        let mut io = FakeIo::default();
        io.names.push_back(Ok(vec!["이더넷 회사망".into()]));
        io.names
            .push_back(Err(std::io::ErrorKind::PermissionDenied.into()));
        io.names.push_back(Ok(vec!["oversized".into(); 257]));
        let mut discovery = Discovery::default();
        let mut selection = selection(NetworkMode::Auto);
        let first = discovery.view(&selection, &mut io);
        assert!(!first.list_failed);
        assert_eq!(first.aliases.as_ref(), &vec!["이더넷 회사망".to_owned()]);
        for generation in [1, 2] {
            selection.refresh = generation;
            let failed = discovery.view(&selection, &mut io);
            assert!(failed.list_failed);
            assert!(Arc::ptr_eq(&first.aliases, &failed.aliases));
            assert_eq!(failed.list_generation, first.list_generation);
        }
        assert_eq!(io.list_calls, 3);
        let cached = discovery.view(&selection, &mut io);
        assert!(cached.list_failed);
        assert_eq!(
            io.list_calls, 3,
            "unchanged refresh generation does not loop on errors"
        );
        selection.mode = NetworkMode::Off;
        selection.refresh = 3;
        discovery.view(&selection, &mut io);
        assert_eq!(
            io.list_calls, 3,
            "Off suppresses even an explicit pending refresh"
        );
    }
    #[test]
    fn contradictory_failed_or_off_samples_cannot_publish_a_visible_rate() {
        for state in [
            NetworkState::Off,
            NetworkState::NoRoute,
            NetworkState::Failed,
        ] {
            let sample = Sample {
                revision: 0,
                at: Instant::now(),
                free: Some(17),
                rate: Some(NetworkRate {
                    received_bytes_per_second: 1024.0,
                    sent_bytes_per_second: 2048.0,
                }),
                observation_failed: (false, false),
                network_view: NetworkView {
                    state,
                    ..Default::default()
                },
            };
            assert!(visible_rate(&sample).is_none());
            assert!(display_key(&sample).rate.is_none());
        }
        let mut legacy_raw = Sample {
            revision: 0,
            at: Instant::now(),
            free: None,
            rate: Some(NetworkRate {
                received_bytes_per_second: 1.0,
                sent_bytes_per_second: 2.0,
            }),
            observation_failed: (false, false),
            network_view: NetworkView::default(),
        };
        assert!(
            visible_rate(&legacy_raw).is_some(),
            "legacy raw-rate fixtures have no discovery state"
        );
        legacy_raw.observation_failed.1 = true;
        assert!(visible_rate(&legacy_raw).is_none());
    }
}
