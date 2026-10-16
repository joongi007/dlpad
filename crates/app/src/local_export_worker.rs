//! Single-request Windows local export coordinator; publication is never discarded.
//! No UI-thread filesystem work, JoinHandle wait or generation-based result filtering.
use dlpad_engine::media_preview::{
    Error, PreviewSession, Tools,
    local_export::{ExportFailure, ExportRequest, Exported, export_local_range},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::{Duration, Instant},
};
struct Job {
    request: ExportRequest,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
}
enum Command {
    Run(Job),
    Stop,
}
struct Active {
    cancel: Arc<AtomicBool>,
    requested_directory: PathBuf,
}
/// `result=None` means panic/transport loss: mutation facts are unknown, not absent.
/// An Ok file result plus cleanup_uncertain still describes a published file, but
/// must be shown as cleanup uncertainty and blocks all further exports.
pub struct ExportCompletion {
    pub result: Option<Result<Exported, ExportFailure>>,
    pub requested_directory: PathBuf,
    pub cleanup_uncertain: bool,
    pub panicked: bool,
}
pub enum SubmitError {
    Busy,
    Stopped,
    OwnershipUnconfirmed,
    Transport,
}
pub struct LocalExportWorker {
    tx: SyncSender<Command>,
    rx: Receiver<ExportCompletion>,
    stop: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    uncertain: Arc<AtomicBool>,
    active: Option<Active>,
}
fn notify(wake: &Arc<dyn Fn() + Send + Sync>) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake()));
}
fn before_mutation(error: Error) -> ExportFailure {
    ExportFailure {
        error,
        fresh_directory: None,
        stage_preserved: false,
        final_created: false,
    }
}
fn execute_scoped(tools: &Tools, root: &Path, job: Job) -> (Result<Exported, ExportFailure>, bool) {
    if job.cancel.load(Ordering::Acquire) {
        return (Err(before_mutation(Error::Cancelled)), false);
    }
    if Instant::now() >= job.deadline {
        return (Err(before_mutation(Error::TimedOut)), false);
    }
    // Both encode/decode and ffprobe are part of this same per-request scope.
    let canonical_root = root.canonicalize().ok();
    let managed = [tools.ffmpeg.as_path(), tools.ffprobe.as_path()]
        .iter()
        .any(|path| {
            path.starts_with(root)
                || canonical_root.as_ref().is_some_and(|root| {
                    path.canonicalize().is_ok_and(|path| path.starts_with(root))
                })
        });
    let manager = if managed {
        if !root.is_dir() {
            return (Err(before_mutation(Error::ToolFailed)), false);
        }
        match dlpad_engine::tools::ToolManager::new(root) {
            Ok(manager) => Some(manager),
            Err(_) => return (Err(before_mutation(Error::ToolFailed)), false),
        }
    } else {
        None
    };
    let execution = match manager
        .as_ref()
        .map(|manager| manager.begin_execution())
        .transpose()
    {
        Ok(execution) => execution,
        // begin_execution can leave a newly created durable marker on failure.
        Err(_) => return (Err(before_mutation(Error::ToolFailed)), true),
    };
    let result = export_local_range(
        tools,
        job.request,
        &PreviewSession {
            managed: execution.as_ref(),
            runtime_environment: None,
        },
        &job.cancel,
        job.deadline,
    );
    let uncertain = result
        .as_ref()
        .is_err_and(|failure| failure.error == Error::OwnershipUnconfirmed);
    if uncertain {
        return (result, true);
    } // original marker survives Drop.
    let cleanup_uncertain = execution.is_some_and(|execution| execution.finish().is_err());
    // Never replace a published Exported or ExportFailure's mutation facts with a
    // generic finish error. The separate flag records cleanup failure faithfully.
    (result, cleanup_uncertain)
}
impl LocalExportWorker {
    pub fn new(
        tools: Tools,
        managed_root: PathBuf,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> std::io::Result<Self> {
        let (tx, jobs) = mpsc::sync_channel::<Command>(1);
        let (done, rx) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let uncertain = Arc::new(AtomicBool::new(false));
        let quit = stop.clone();
        let ack = stopped.clone();
        let sticky = uncertain.clone();
        thread::Builder::new()
            .name("dlpad-local-export".into())
            .spawn(move || {
                struct Ack(Arc<AtomicBool>, Arc<dyn Fn() + Send + Sync>);
                impl Drop for Ack {
                    fn drop(&mut self) {
                        self.0.store(true, Ordering::Release);
                        export_trace(4, true, false);
                        notify(&self.1);
                    }
                }
                let _ack = Ack(ack, wake.clone());
                while let Ok(command) = jobs.recv() {
                    let job = match command {
                        Command::Stop => break,
                        Command::Run(job) => job,
                    };
                    // A queued Run accepted before Stop still produces a cancellation
                    // result. Never skip it and lose publication/mutation accountability.
                    if quit.load(Ordering::Acquire) {
                        job.cancel.store(true, Ordering::Release);
                    }
                    let requested_directory =
                        job.request.output_parent.join(&job.request.folder_name);
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        execute_scoped(&tools, &managed_root, job)
                    }));
                    let completion = match outcome {
                        Ok((result, cleanup_uncertain)) => ExportCompletion {
                            result: Some(result),
                            requested_directory,
                            cleanup_uncertain,
                            panicked: false,
                        },
                        Err(_) => ExportCompletion {
                            result: None,
                            requested_directory,
                            cleanup_uncertain: true,
                            panicked: true,
                        },
                    };
                    let terminal = completion.cleanup_uncertain;
                    if terminal {
                        sticky.store(true, Ordering::Release);
                    }
                    export_trace(2, terminal, false);
                    if done.try_send(completion).is_err() {
                        sticky.store(true, Ordering::Release);
                        break;
                    }
                    notify(&wake);
                    if terminal || quit.load(Ordering::Acquire) {
                        break;
                    }
                }
            })?;
        Ok(Self {
            tx,
            rx,
            stop,
            stopped,
            uncertain,
            active: None,
        })
    }
    /// The request is returned on rejection; a Busy submit never cancels/replaces an export.
    pub fn submit(&mut self, request: ExportRequest) -> Result<(), (SubmitError, ExportRequest)> {
        if self.uncertain.load(Ordering::Acquire) {
            return Err((SubmitError::OwnershipUnconfirmed, request));
        }
        if self.stop.load(Ordering::Acquire) || self.stopped.load(Ordering::Acquire) {
            return Err((SubmitError::Stopped, request));
        }
        if self.active.is_some() {
            return Err((SubmitError::Busy, request));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let requested_directory = request.output_parent.join(&request.folder_name);
        let job = Job {
            request,
            cancel: cancel.clone(),
            deadline: Instant::now() + Duration::from_secs(180),
        };
        match self.tx.try_send(Command::Run(job)) {
            Ok(()) => {
                self.active = Some(Active {
                    cancel,
                    requested_directory,
                });
                Ok(())
            }
            Err(TrySendError::Full(Command::Run(job))) => Err((SubmitError::Busy, job.request)),
            Err(TrySendError::Disconnected(Command::Run(job))) => {
                Err((SubmitError::Transport, job.request))
            }
            Err(_) => unreachable!("submission sends only Run"),
        }
    }
    pub fn cancel(&mut self) {
        if let Some(active) = &self.active {
            active.cancel.store(true, Ordering::Release);
        }
    }
    pub fn request_stop(&mut self) {
        export_trace(1, self.busy(), self.stopped());
        self.cancel();
        self.stop.store(true, Ordering::Release);
        let _ = self.tx.try_send(Command::Stop);
    }
    /// Poll remains valid after stop/ACK: accepted publication facts must be consumed.
    pub fn poll(&mut self) -> Option<ExportCompletion> {
        if let Ok(mut completion) = self.rx.try_recv() {
            export_trace(3, self.stopped(), completion.cleanup_uncertain);
            self.active = None;
            completion.cleanup_uncertain |= self.uncertain.load(Ordering::Acquire);
            return Some(completion);
        }
        if self.stopped.load(Ordering::Acquire) {
            // Re-read after acquiring ACK: the first try_recv may have raced the
            // completion send that necessarily precedes the worker's stop ACK.
            if let Ok(mut completion) = self.rx.try_recv() {
                export_trace(3, self.stopped(), completion.cleanup_uncertain);
                self.active = None;
                completion.cleanup_uncertain |= self.uncertain.load(Ordering::Acquire);
                return Some(completion);
            }
            if let Some(active) = self.active.take() {
                self.uncertain.store(true, Ordering::Release);
                return Some(ExportCompletion {
                    result: None,
                    requested_directory: active.requested_directory,
                    cleanup_uncertain: true,
                    panicked: false,
                });
            }
        }
        None
    }
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
    pub fn ownership_unconfirmed(&self) -> bool {
        self.uncertain.load(Ordering::Acquire)
    }
}
impl Drop for LocalExportWorker {
    fn drop(&mut self) {
        self.request_stop();
    }
} // Drop is not an ACK.
/// Diagnostic-only numeric phases; opt-in is evaluated once, never per frame.
pub(super) fn export_trace(phase: u8, a: bool, b: bool) {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    {
        static BUDGET: std::sync::OnceLock<std::sync::atomic::AtomicUsize> =
            std::sync::OnceLock::new();
        let budget = BUDGET.get_or_init(|| {
            std::sync::atomic::AtomicUsize::new(
                if std::env::var_os("DLPAD_TEST_EXPORT_ACK_TRACE").is_some_and(|v| v == "1") {
                    128
                } else {
                    0
                },
            )
        });
        if let Ok(before) =
            budget.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_sub(1))
        {
            eprintln!(
                "dlpad-export-ack-trace seq={} phase={} a={} b={}",
                129 - before,
                phase,
                u8::from(a),
                u8::from(b)
            );
        }
    }
    #[cfg(not(all(windows, feature = "first-paint-fixture")))]
    let _ = (phase, a, b);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        LocalExportWorker,
        SyncSender<ExportCompletion>,
        Receiver<Command>,
    ) {
        let (tx, jobs) = mpsc::sync_channel(1);
        let (done, rx) = mpsc::sync_channel(1);
        (
            LocalExportWorker {
                tx,
                rx,
                stop: Arc::new(AtomicBool::new(false)),
                stopped: Arc::new(AtomicBool::new(false)),
                uncertain: Arc::new(AtomicBool::new(false)),
                active: None,
            },
            done,
            jobs,
        )
    }
    fn active(worker: &mut LocalExportWorker) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        worker.active = Some(Active {
            cancel: cancel.clone(),
            requested_directory: PathBuf::from("synthetic-output"),
        });
        cancel
    }
    #[test]
    fn cancel_and_stop_preserve_already_published_completion() {
        let (mut worker, done, _) = fixture();
        let cancel = active(&mut worker);
        assert!(
            done.send(ExportCompletion {
                result: Some(Ok(Exported {
                    path: PathBuf::from("synthetic-output/clip.mp4"),
                    bytes: 123,
                    duration_ms: 1000,
                    retained_stage_path: PathBuf::from("synthetic-output/stage.mp4")
                })),
                requested_directory: PathBuf::from("synthetic-output"),
                cleanup_uncertain: false,
                panicked: false
            })
            .is_ok()
        );
        worker.cancel();
        worker.request_stop();
        assert!(cancel.load(Ordering::Acquire));
        let completion = worker.poll().unwrap();
        let exported = completion.result.unwrap().ok().unwrap();
        assert_eq!(exported.bytes, 123);
        assert!(!completion.cleanup_uncertain);
        assert!(!worker.busy());
    }
    #[test]
    fn finish_failure_keeps_actual_final_and_stage_facts() {
        let (mut worker, done, _) = fixture();
        active(&mut worker);
        worker.uncertain.store(true, Ordering::Release);
        assert!(
            done.send(ExportCompletion {
                result: Some(Err(ExportFailure {
                    error: Error::InvalidMetadata,
                    fresh_directory: Some(PathBuf::from("synthetic-output")),
                    stage_preserved: true,
                    final_created: true
                })),
                requested_directory: PathBuf::from("synthetic-output"),
                cleanup_uncertain: true,
                panicked: false
            })
            .is_ok()
        );
        let failure = worker.poll().unwrap().result.unwrap().err().unwrap();
        assert!(failure.stage_preserved && failure.final_created);
        assert!(worker.ownership_unconfirmed());
    }
    #[test]
    fn ack_without_completion_never_claims_no_mutation() {
        let (mut worker, _, _) = fixture();
        active(&mut worker);
        worker.stopped.store(true, Ordering::Release);
        let completion = worker.poll().unwrap();
        assert!(completion.result.is_none() && completion.cleanup_uncertain);
        assert_eq!(
            completion.requested_directory,
            PathBuf::from("synthetic-output")
        );
        assert!(worker.ownership_unconfirmed());
    }
    #[test]
    fn idle_stop_wakes_without_ui_thread_join() {
        let (mut worker, _, jobs) = fixture();
        worker.request_stop();
        assert!(matches!(jobs.try_recv(), Ok(Command::Stop)));
        assert!(!worker.stopped());
    }
}
