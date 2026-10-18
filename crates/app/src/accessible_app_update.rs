//! App release observations and bounded intents for the parent-owned updater.
use crate::{app_update_client as client, catalog as c, ui_components as controls, ui_tokens as t};
use dlpad_engine::app_update::{
    Channel,
    discovery::{self, DiscoveryError, ReleaseObservation},
};
use eframe::egui;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
enum PanelError {
    Discovery(DiscoveryError),
    Worker,
}
#[derive(Clone, Copy, Debug)]
enum Status {
    Idle,
    Checking,
    Finished,
    Cancelled,
    Failed(PanelError),
}
#[derive(Default)]
struct Signals {
    cancel: AtomicBool,
    stopped: AtomicBool,
}
struct Ack {
    signals: Arc<Signals>,
    context: egui::Context,
}
impl Drop for Ack {
    fn drop(&mut self) {
        self.signals.stopped.store(true, Ordering::Release);
        self.context.request_repaint();
    }
}
struct Completion {
    generation: u64,
    channel: Channel,
    outcome: Result<ReleaseObservation, PanelError>,
}
struct Worker {
    signals: Arc<Signals>,
    results: Receiver<Completion>,
    thread: JoinHandle<()>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UpdateEnvironment {
    pub startup_ready: bool,
    pub task_idle: bool,
    pub closing: bool,
}
pub(crate) struct AppUpdatePanel {
    channel: Channel,
    generation: u64,
    worker: Option<Worker>,
    observation: Option<ReleaseObservation>,
    status: Status,
    cancel_requested: bool,
    stopping: bool,
    environment: UpdateEnvironment,
    client: Option<client::ClientHandle>,
    client_state: client::Snapshot,
    restart_requested: bool,
    accepted_generation: Option<u64>,
    action_error: Option<&'static str>,
}
impl AppUpdatePanel {
    pub(crate) fn new() -> Self {
        Self {
            channel: Channel::Stable,
            generation: 0,
            worker: None,
            observation: None,
            status: Status::Idle,
            cancel_requested: false,
            stopping: false,
            environment: UpdateEnvironment::default(),
            client: client::handle(),
            client_state: client::Snapshot::default(),
            restart_requested: false,
            accepted_generation: None,
            action_error: None,
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.worker.is_some() || self.client.as_ref().is_some_and(client::ClientHandle::busy)
    }
    pub(crate) fn stopped(&self) -> bool {
        self.worker.is_none()
            && self
                .client
                .as_ref()
                .is_none_or(client::ClientHandle::stopped)
    }
    pub(crate) fn request_stop(&mut self) {
        self.stopping = true;
        self.cancel_requested = true;
        if let Some(client) = &self.client {
            client.begin_shutdown();
        }
        if let Some(worker) = &self.worker {
            worker.signals.cancel.store(true, Ordering::Release);
        }
    }
    pub(crate) fn set_environment(&mut self, environment: UpdateEnvironment) {
        self.environment = environment;
    }
    pub(crate) fn take_restart_requested(&mut self) -> bool {
        std::mem::take(&mut self.restart_requested)
    }
    fn poll_client(&mut self, ctx: &egui::Context) {
        if self.client.is_none() {
            self.client = client::handle();
        }
        if let Some(client) = &self.client {
            client.bind_context(ctx);
            if let Some(state) = client.snapshot() {
                self.apply_client_snapshot(state);
            }
        }
    }
    fn apply_client_snapshot(&mut self, state: client::Snapshot) {
        if state.phase == client::Phase::RestartAccepted
            && state.generation != 0
            && self.accepted_generation != Some(state.generation)
            && !self.stopping
        {
            self.accepted_generation = Some(state.generation);
            self.restart_requested = true;
        }
        self.client_state = state;
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        self.poll_client(ctx);
        let Some(worker) = &self.worker else {
            return;
        };
        if !worker.thread.is_finished() {
            // Active task only; there is no idle timer or idle worker.
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }
        let worker = self.worker.take().expect("finished worker remains owned");
        let joined = worker.thread.join();
        let completion = worker.results.try_recv();
        self.cancel_requested = false;
        if self.stopping {
            self.status = Status::Cancelled;
            return;
        }
        if joined.is_err() || !worker.signals.stopped.load(Ordering::Acquire) {
            self.status = Status::Failed(PanelError::Worker);
            return;
        }
        let Ok(result) = completion else {
            self.status = Status::Failed(PanelError::Worker);
            return;
        };
        if result.generation != self.generation || result.channel != self.channel {
            self.status = Status::Failed(PanelError::Worker);
            return;
        }
        match result.outcome {
            Ok(observation) => {
                self.observation = Some(observation);
                self.status = Status::Finished;
            }
            Err(PanelError::Discovery(DiscoveryError::Cancelled)) => {
                self.status = Status::Cancelled
            }
            Err(error) => self.status = Status::Failed(error),
        }
    }
    fn start_with(
        &mut self,
        ctx: &egui::Context,
        lookup: impl FnOnce(Channel, &AtomicBool, Instant) -> Result<ReleaseObservation, DiscoveryError>
        + Send
        + 'static,
    ) {
        if self.stopping || self.busy() {
            return;
        }
        let Some(generation) = self.generation.checked_add(1) else {
            self.status = Status::Failed(PanelError::Worker);
            return;
        };
        self.generation = generation;
        let channel = self.channel;
        let signals = Arc::new(Signals::default());
        let owned = signals.clone();
        let (responses, results) = mpsc::sync_channel(1);
        let context = ctx.clone();
        let spawned = thread::Builder::new()
            .name("dlpad-app-update".into())
            .spawn(move || {
                let _ack = Ack {
                    signals: owned.clone(),
                    context,
                };
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    lookup(
                        channel,
                        &owned.cancel,
                        Instant::now() + Duration::from_secs(20),
                    )
                }))
                .map_err(|_| PanelError::Worker)
                .and_then(|value| value.map_err(PanelError::Discovery));
                let _ = responses.try_send(Completion {
                    generation,
                    channel,
                    outcome,
                });
            });
        match spawned {
            Ok(thread) => {
                self.worker = Some(Worker {
                    signals,
                    results,
                    thread,
                });
                self.status = Status::Checking;
                self.cancel_requested = false;
            }
            Err(_) => self.status = Status::Failed(PanelError::Worker),
        }
    }
    fn start(&mut self, ctx: &egui::Context) {
        self.start_with(ctx, |channel, cancel, deadline| {
            discovery::discover_release(channel, env!("CARGO_PKG_VERSION"), cancel, deadline)
        });
    }
    fn status_text(&self) -> &'static str {
        if self.cancel_requested && self.worker.is_some() {
            return c::APP_UPDATE_CANCELLING;
        }
        match self.status {
            Status::Idle => c::APP_UPDATE_INITIAL,
            Status::Checking => c::APP_UPDATE_CHECKING,
            Status::Finished => {
                if self
                    .observation
                    .as_ref()
                    .is_some_and(ReleaseObservation::is_newer)
                {
                    c::APP_UPDATE_AVAILABLE
                } else {
                    c::APP_UPDATE_CURRENT
                }
            }
            Status::Cancelled => c::APP_UPDATE_CANCELLED,
            Status::Failed(PanelError::Worker) => c::APP_UPDATE_WORKER_FAILED,
            Status::Failed(PanelError::Discovery(error)) => match error {
                DiscoveryError::Cancelled => c::APP_UPDATE_CANCELLED,
                DiscoveryError::TimedOut => c::APP_UPDATE_TIMEOUT,
                DiscoveryError::Offline => c::APP_UPDATE_OFFLINE,
                DiscoveryError::RateLimited => c::APP_UPDATE_RATE_LIMIT,
                DiscoveryError::NoRelease => {
                    if self.channel == Channel::Preview {
                        c::APP_UPDATE_NONE_PREVIEW
                    } else {
                        c::APP_UPDATE_NONE_STABLE
                    }
                }
                DiscoveryError::InvalidResponse | DiscoveryError::Limit => c::APP_UPDATE_INVALID,
            },
        }
    }
    fn update_blocked_reason(&self) -> Option<&'static str> {
        if self.stopping || self.environment.closing {
            Some(c::APP_UPDATE_STOPPING)
        } else if self.client.is_none() || !cfg!(all(windows, target_arch = "x86_64")) {
            Some(c::APP_UPDATE_SUPPORT_MANUAL)
        } else if self
            .client
            .as_ref()
            .is_some_and(client::ClientHandle::stopped)
        {
            Some(c::APP_UPDATE_PROTOCOL_FAILED)
        } else if !self.environment.startup_ready || !self.client_state.runtime_ready {
            Some(c::APP_UPDATE_STARTUP_WAIT)
        } else if !self.environment.task_idle {
            Some(c::APP_UPDATE_TASK_WAIT)
        } else {
            None
        }
    }
    fn update_error(code: u16) -> &'static str {
        match code {
            1 => c::APP_UPDATE_UPDATE_CANCELLED,
            2 => c::APP_UPDATE_TRUST_UNAVAILABLE,
            3 => c::APP_UPDATE_OFFLINE,
            4 => c::APP_UPDATE_VERIFY_FAILED,
            5 => c::APP_UPDATE_DISK_FAILED,
            6 => c::APP_UPDATE_PREPARE_FAILED,
            7 => c::APP_UPDATE_TASK_WAIT,
            8 => c::APP_UPDATE_EXPIRED,
            9 => c::APP_UPDATE_RESTART_REFUSED,
            11 => c::APP_UPDATE_UPDATE_WORKER_FAILED,
            12 => c::APP_UPDATE_PROFILE_FAILED,
            13 => c::APP_UPDATE_RECOVERED,
            _ => c::APP_UPDATE_PROTOCOL_FAILED,
        }
    }
    fn prepare_requested(&mut self) {
        if self.update_blocked_reason().is_some() || self.busy() {
            return;
        }
        let Some(observation) = self
            .observation
            .as_ref()
            .filter(|value| value.is_newer() && value.signed_metadata_available())
        else {
            return;
        };
        let Some(client) = &self.client else {
            return;
        };
        self.action_error = client
            .prepare(observation.version(), self.channel)
            .err()
            .map(|error| match error {
                client::RequestError::Busy => c::APP_UPDATE_TASK_WAIT,
                _ => c::APP_UPDATE_PROTOCOL_FAILED,
            });
    }
    fn restart_requested(&mut self) {
        if self.update_blocked_reason().is_some()
            || self.busy()
            || self.client_state.phase != client::Phase::Prepared
        {
            return;
        }
        let Some(client) = &self.client else {
            return;
        };
        self.action_error = client
            .request_restart(self.client_state.generation)
            .err()
            .map(|_| c::APP_UPDATE_RESTART_REFUSED);
    }
    fn show_update_actions(&mut self, ui: &mut egui::Ui, enabled: bool) {
        use client::Phase;
        let phase = self.client_state.phase;
        let label = match phase {
            Phase::Idle | Phase::Stopped => None,
            Phase::Requesting | Phase::Verifying => Some(c::APP_UPDATE_VERIFYING),
            Phase::Downloading => Some(c::APP_UPDATE_DOWNLOADING),
            Phase::Staging => Some(c::APP_UPDATE_STAGING),
            Phase::Preparing => Some(c::APP_UPDATE_PREPARING),
            Phase::Prepared => Some(c::APP_UPDATE_PREPARED),
            Phase::RestartPending | Phase::RestartAccepted => Some(c::APP_UPDATE_RESTARTING),
            Phase::Cancelling => Some(c::APP_UPDATE_STOPPING),
            Phase::Failed => self.client_state.error.map(Self::update_error),
        };
        if let Some(label) = label {
            ui.add_space(t::CONTROL_GAP);
            ui.label(label);
            if phase == Phase::Prepared
                && let Some(version) = &self.client_state.version
            {
                ui.strong(format!("v{version}"));
            }
            if phase == Phase::Downloading && self.client_state.total > 0 {
                let received = self.client_state.received.min(self.client_state.total);
                let fraction = received as f64 / self.client_state.total as f64;
                ui.add(egui::ProgressBar::new(fraction as f32).show_percentage());
                // Binary units are size labels, not download-rate observations.
                ui.weak(format!(
                    "{:.1} / {:.1} MiB",
                    received as f64 / 1_048_576.0,
                    self.client_state.total as f64 / 1_048_576.0
                ));
            }
            if phase == Phase::Prepared
                && let Some(code) = self.client_state.error
            {
                ui.label(Self::update_error(code));
            }
        }
        if let Some(error) = self.action_error {
            ui.label(error);
        }
        let preparing = matches!(
            phase,
            Phase::Requesting
                | Phase::Verifying
                | Phase::Downloading
                | Phase::Staging
                | Phase::Preparing
                | Phase::Cancelling
        );
        let observed = self
            .observation
            .as_ref()
            .is_some_and(|value| value.is_newer() && value.signed_metadata_available());
        if !preparing && phase != Phase::Prepared && !observed {
            return;
        }
        let blocked = self.update_blocked_reason();
        ui.horizontal_wrapped(|ui| {
            if phase == Phase::Prepared {
                if controls::primary(
                    ui,
                    c::APP_UPDATE_RESTART,
                    enabled && blocked.is_none() && !self.busy(),
                )
                .clicked()
                {
                    self.restart_requested();
                }
            } else if preparing {
                if controls::ghost(
                    ui,
                    c::APP_UPDATE_STOP,
                    enabled && !self.stopping && phase != Phase::Cancelling,
                )
                .clicked()
                    && let Some(client) = &self.client
                {
                    client.cancel();
                }
            } else if controls::primary(
                ui,
                c::APP_UPDATE_PREPARE,
                enabled && blocked.is_none() && !self.busy(),
            )
            .clicked()
            {
                self.prepare_requested();
            }
        });
        if let Some(reason) = blocked
            && !preparing
        {
            ui.label(reason);
        }
    }
    fn show_notes(ui: &mut egui::Ui, notes: &str, truncated: bool) {
        ui.add_space(t::CONTROL_GAP);
        egui::CollapsingHeader::new(c::APP_UPDATE_NOTES)
            .id_salt("app-update-notes")
            .default_open(true)
            .show(ui, |ui| {
                if notes.trim().is_empty() {
                    ui.weak(c::APP_UPDATE_NOTES_EMPTY);
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("app-update-notes-text")
                        .max_height(180.0)
                        .show(ui, |ui| {
                            // Release text remains inert: no Markdown, URL opening or external images.
                            ui.add(egui::Label::new(notes).wrap());
                        });
                }
                if truncated {
                    ui.label(c::APP_UPDATE_NOTES_TRUNCATED);
                }
            });
    }
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui, enabled: bool) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        ui.push_id("app-update-panel", |ui| {
            ui.heading(c::APP_UPDATE_TITLE);
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{} {}",
                    c::APP_UPDATE_INSTALLED,
                    env!("CARGO_PKG_VERSION")
                ));
                controls::help(
                    ui,
                    "app-update-version",
                    c::APP_UPDATE_INSTALLED,
                    c::APP_UPDATE_VERSION_HELP,
                );
                if cfg!(all(windows, target_arch = "x86_64")) {
                    ui.weak(c::APP_UPDATE_WINDOWS);
                }
            });
            ui.add_space(t::LABEL_GAP);
            if !cfg!(all(windows, target_arch = "x86_64")) {
                ui.label(c::APP_UPDATE_OTHER_PLATFORM);
            }
            ui.add_enabled_ui(enabled && !self.stopping && !self.busy(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(c::APP_UPDATE_CHANNEL);
                    let stable =
                        controls::choice(ui, self.channel == Channel::Stable, c::APP_UPDATE_STABLE)
                            .clicked();
                    let preview = controls::choice(
                        ui,
                        self.channel == Channel::Preview,
                        c::APP_UPDATE_PREVIEW,
                    )
                    .clicked();
                    let channel = if stable {
                        Some(Channel::Stable)
                    } else if preview {
                        Some(Channel::Preview)
                    } else {
                        None
                    };
                    if let Some(channel) = channel.filter(|channel| *channel != self.channel) {
                        self.channel = channel;
                        self.observation = None;
                        self.status = Status::Idle;
                    }
                });
            });
            ui.add_space(t::CONTROL_GAP);
            ui.horizontal_wrapped(|ui| {
                if controls::secondary(
                    ui,
                    c::APP_UPDATE_CHECK,
                    enabled
                        && !self.stopping
                        && !self.busy()
                        && cfg!(all(windows, target_arch = "x86_64")),
                )
                .clicked()
                {
                    self.start(&ctx);
                }
                if self.worker.is_some()
                    && controls::ghost(ui, c::APP_UPDATE_CANCEL, enabled && !self.cancel_requested)
                        .clicked()
                {
                    self.cancel_requested = true;
                    if let Some(worker) = &self.worker {
                        worker.signals.cancel.store(true, Ordering::Release);
                    }
                }
            });
            ui.add_space(t::LABEL_GAP);
            ui.label(self.status_text());
            if let Some(observation) = &self.observation {
                ui.horizontal_wrapped(|ui| {
                    if !matches!(self.status, Status::Finished) {
                        ui.weak(c::APP_UPDATE_CACHED);
                    }
                    ui.strong(format!("v{}", observation.version()));
                    ui.add_enabled_ui(enabled && !self.stopping, |ui| {
                        ui.hyperlink_to(
                            if observation.is_newer() {
                                c::APP_UPDATE_RELEASE_OPEN
                            } else {
                                c::APP_UPDATE_RELEASE_VIEW
                            },
                            observation.url(),
                        );
                    });
                });
                if observation.is_newer() && !observation.signed_metadata_available() {
                    ui.label(c::APP_UPDATE_SIGNED_METADATA_MISSING);
                }
            } else {
                ui.add_enabled_ui(enabled && !self.stopping, |ui| {
                    ui.hyperlink_to(c::APP_UPDATE_ALL_RELEASES, discovery::RELEASES_URL);
                });
            }
            self.show_update_actions(ui, enabled);
            if let Some(observation) = &self.observation {
                Self::show_notes(ui, observation.notes(), observation.notes_truncated());
            }
            ui.add_space(t::LABEL_GAP);
            if self.client.is_none() {
                ui.weak(c::APP_UPDATE_MANUAL_ONLY);
            }
        });
    }
}
impl Drop for AppUpdatePanel {
    fn drop(&mut self) {
        self.request_stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn finish(panel: &mut AppUpdatePanel, ctx: &egui::Context) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while panel.busy() {
            panel.poll(ctx);
            assert!(Instant::now() < deadline, "bounded worker ACK");
            thread::yield_now();
        }
    }
    #[test]
    fn only_parent_restart_acceptance_produces_one_host_exit_intent() {
        let mut panel = AppUpdatePanel::new();
        for phase in [client::Phase::Prepared, client::Phase::RestartPending] {
            panel.apply_client_snapshot(client::Snapshot {
                phase,
                generation: 1,
                ..Default::default()
            });
            assert!(!panel.take_restart_requested());
        }
        let accepted = client::Snapshot {
            phase: client::Phase::RestartAccepted,
            generation: 1,
            ..Default::default()
        };
        panel.apply_client_snapshot(accepted.clone());
        assert!(panel.take_restart_requested());
        panel.apply_client_snapshot(accepted);
        assert!(!panel.take_restart_requested());
        panel.request_stop();
        panel.apply_client_snapshot(client::Snapshot {
            phase: client::Phase::RestartAccepted,
            generation: 2,
            ..Default::default()
        });
        assert!(
            !panel.take_restart_requested(),
            "normal close cannot mint a fresh restart intent"
        );
    }
    #[test]
    fn restart_refusal_preserves_prepared_version_and_visible_reason() {
        let ctx = egui::Context::default();
        let mut panel = AppUpdatePanel::new();
        panel.apply_client_snapshot(client::Snapshot {
            phase: client::Phase::Prepared,
            version: Some("0.2.0".into()),
            generation: 1,
            error: Some(9),
            ..Default::default()
        });
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| panel.show_update_actions(ui, true));
        });
        fn text(shape: &egui::Shape, out: &mut String) {
            match shape {
                egui::Shape::Text(value) => out.push_str(value.galley.text()),
                egui::Shape::Vec(values) => {
                    for value in values {
                        text(value, out);
                    }
                }
                _ => (),
            }
        }
        let mut painted = String::new();
        for shape in output.shapes {
            text(&shape.shape, &mut painted);
        }
        assert!(painted.contains("v0.2.0"));
        assert!(painted.contains(c::APP_UPDATE_RESTART_REFUSED));
        assert!(!panel.take_restart_requested());
        assert!(!panel.busy());
    }
    #[test]
    fn no_release_message_identifies_the_selected_channel() {
        let ctx = egui::Context::default();
        let mut panel = AppUpdatePanel::new();
        panel.start_with(&ctx, |_, _, _| Err(DiscoveryError::NoRelease));
        finish(&mut panel, &ctx);
        assert_eq!(panel.status_text(), c::APP_UPDATE_NONE_STABLE);
        panel.channel = Channel::Preview;
        panel.start_with(&ctx, |_, _, _| Err(DiscoveryError::NoRelease));
        finish(&mut panel, &ctx);
        assert_eq!(panel.status_text(), c::APP_UPDATE_NONE_PREVIEW);
    }
    #[test]
    fn release_notes_remain_literal_text_and_truncation_is_visible() {
        fn text(shape: &egui::Shape, out: &mut String) {
            match shape {
                egui::Shape::Text(value) => out.push_str(value.galley.text()),
                egui::Shape::Vec(values) => {
                    for value in values {
                        text(value, out);
                    }
                }
                _ => (),
            }
        }
        for (notes, expected, truncated) in [
            ("", c::APP_UPDATE_NOTES_EMPTY, false),
            (
                "<script>fixture</script> [link](https://invalid.test) 한글",
                "<script>fixture</script>",
                true,
            ),
        ] {
            let ctx = egui::Context::default();
            ctx.style_mut(|style| style.animation_time = 0.0);
            let mut output = None;
            for _ in 0..2 {
                output = Some(ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 600.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| AppUpdatePanel::show_notes(ui, notes, truncated));
                    },
                ));
            }
            let mut painted = String::new();
            for shape in output.unwrap().shapes {
                text(&shape.shape, &mut painted);
            }
            assert!(painted.contains(c::APP_UPDATE_NOTES));
            assert!(painted.contains(expected), "actual notes: {painted}");
            assert_eq!(painted.contains(c::APP_UPDATE_NOTES_TRUNCATED), truncated);
        }
    }
    #[test]
    fn new_is_idle_without_network_and_stop_prevents_dispatch() {
        let mut p = AppUpdatePanel::new();
        assert!(!p.busy());
        assert!(p.stopped());
        p.request_stop();
        p.start_with(&egui::Context::default(), |_, _, _| {
            panic!("must not dispatch")
        });
        assert!(p.stopped());
    }
    #[test]
    fn panic_and_failure_finish_with_ack_and_no_raw_error() {
        let ctx = egui::Context::default();
        let mut p = AppUpdatePanel::new();
        p.start_with(&ctx, |_, _, _| panic!("fixture only"));
        finish(&mut p, &ctx);
        assert_eq!(p.status_text(), c::APP_UPDATE_WORKER_FAILED);
        assert!(p.stopped());
        p.start_with(&ctx, |_, _, _| Err(DiscoveryError::Offline));
        finish(&mut p, &ctx);
        assert_eq!(p.status_text(), c::APP_UPDATE_OFFLINE);
    }
    #[test]
    fn stop_during_owned_worker_waits_for_ack_and_blocks_duplicate() {
        let ctx = egui::Context::default();
        let mut p = AppUpdatePanel::new();
        p.start_with(&ctx, |_, cancel, deadline| {
            while !cancel.load(Ordering::Acquire) {
                assert!(Instant::now() < deadline);
                thread::yield_now();
            }
            Err(DiscoveryError::Cancelled)
        });
        p.start_with(&ctx, |_, _, _| panic!("duplicate"));
        p.request_stop();
        finish(&mut p, &ctx);
        assert_eq!(p.status_text(), c::APP_UPDATE_CANCELLED);
        assert!(p.stopped());
    }
    #[test]
    fn bounded_egui_context_can_draw_100_and_200_percent_without_network() {
        for zoom in [1.0, 2.0] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(zoom);
            let mut p = AppUpdatePanel::new();
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| p.ui(ui, true));
                },
            );
            assert!(!p.busy());
        }
    }
    #[test]
    fn stale_worker_generation_is_rejected_without_publishing_an_error_for_new_request() {
        let ctx = egui::Context::default();
        let mut p = AppUpdatePanel::new();
        p.start_with(&ctx, |_, _, _| Err(DiscoveryError::Offline));
        p.generation += 1;
        finish(&mut p, &ctx);
        assert_eq!(p.status_text(), c::APP_UPDATE_WORKER_FAILED);
        assert!(p.observation.is_none());
    }
    #[test]
    fn manual_release_notice_is_painted_without_automatic_install_claim() {
        fn text(shape: &egui::Shape, out: &mut String) {
            match shape {
                egui::Shape::Text(value) => out.push_str(value.galley.text()),
                egui::Shape::Vec(values) => {
                    for value in values {
                        text(value, out)
                    }
                }
                _ => (),
            }
        }
        let ctx = egui::Context::default();
        let mut p = AppUpdatePanel::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| p.ui(ui, false));
            },
        );
        let mut painted = String::new();
        for shape in output.shapes {
            text(&shape.shape, &mut painted);
        }
        assert!(painted.contains(c::APP_UPDATE_MANUAL_ONLY));
        assert!(painted.contains(c::APP_UPDATE_INSTALLED));
        assert!(!p.busy());
    }
}
