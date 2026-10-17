//! Read-only app release panel. No package execution, silent network or updater trust bypass.
use crate::{catalog as c, ui_components as controls, ui_tokens as t};
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

pub(crate) struct AppUpdatePanel {
    channel: Channel,
    generation: u64,
    worker: Option<Worker>,
    observation: Option<ReleaseObservation>,
    status: Status,
    cancel_requested: bool,
    stopping: bool,
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
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.worker.is_some()
    }
    pub(crate) fn stopped(&self) -> bool {
        self.worker.is_none()
    }
    pub(crate) fn request_stop(&mut self) {
        self.stopping = true;
        self.cancel_requested = true;
        if let Some(worker) = &self.worker {
            worker.signals.cancel.store(true, Ordering::Release);
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
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
        if self.cancel_requested && self.busy() {
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
                        c::APP_UPDATE_NONE
                    }
                }
                DiscoveryError::InvalidResponse | DiscoveryError::Limit => c::APP_UPDATE_INVALID,
            },
        }
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
                if self.busy()
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
                        ui.hyperlink_to(c::APP_UPDATE_RELEASE_OPEN, observation.url());
                    });
                });
            } else {
                ui.add_enabled_ui(enabled && !self.stopping, |ui| {
                    ui.hyperlink_to(c::APP_UPDATE_ALL_RELEASES, discovery::RELEASES_URL);
                });
            }
            ui.add_space(t::LABEL_GAP);
            ui.weak(c::APP_UPDATE_MANUAL_ONLY);
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
