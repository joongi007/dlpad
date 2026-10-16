//! Latest-only preview coordinator. Blocking implementation lives in engine.
use dlpad_engine::media_preview::{
    Error, PreviewSession, Product, Request, Tools, execute_with_session,
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
    generation: u64,
    request: Request,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
}
enum Command {
    Run(Job),
    WakeStop,
}
struct Completion {
    generation: u64,
    value: Result<Product, Error>,
}
pub struct PreviewWorker {
    tx: SyncSender<Command>,
    rx: Receiver<Completion>,
    stop: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    uncertain: Arc<AtomicBool>,
    generation: u64,
    active: Option<Arc<AtomicBool>>,
    pending: Option<Request>,
    terminal: bool,
}
// Only the worker performs package/cache/lease I/O. Each request owns one journal
// scope so repeated seeking cannot exhaust the journal's fixed reader slots.
fn execute_scoped(
    tools: &Tools,
    root: &Path,
    request: Request,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Product, Error> {
    if cancel.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(Error::TimedOut);
    }
    let remote = matches!(
        &request,
        Request::Prepare(dlpad_engine::media_preview::Source::Url(_))
    );
    let selected: Vec<&Path> = match &request {
        Request::Prepare(dlpad_engine::media_preview::Source::Url(_)) => {
            std::iter::once(tools.ytdlp.as_path())
                .chain(tools.deno.as_deref())
                .collect()
        }
        Request::Prepare(_) => vec![tools.ffprobe.as_path()],
        _ => vec![tools.ffmpeg.as_path()],
    };
    // A lexical managed path must never silently fall back to an external run if
    // its original root/file has disappeared. Canonical aliases are covered too.
    let canonical_root = root.canonicalize().ok();
    let managed = selected.iter().any(|path| {
        path.starts_with(root)
            || canonical_root
                .as_ref()
                .is_some_and(|root| path.canonicalize().is_ok_and(|path| path.starts_with(root)))
    });
    let manager = if managed {
        if !root.is_dir() {
            return Err(Error::ToolFailed);
        }
        Some(dlpad_engine::tools::ToolManager::new(root).map_err(|_| Error::ToolFailed)?)
    } else {
        None
    };
    #[cfg(windows)]
    let environment = if remote {
        match (manager.as_ref(), tools.deno.as_deref()) {
            (Some(manager), Some(deno)) => manager
                .prepare_managed_deno_environment(deno, cancel)
                .map_err(|_| Error::ToolFailed)?,
            _ => None,
        }
    } else {
        None
    };
    #[cfg(not(windows))]
    let _ = remote;
    let execution = manager
        .as_ref()
        .map(|manager| manager.begin_execution())
        .transpose()
        .map_err(|_| Error::OwnershipUnconfirmed)?;
    let result = execute_with_session(
        tools,
        request,
        cancel,
        deadline,
        PreviewSession {
            managed: execution.as_ref(),
            #[cfg(windows)]
            runtime_environment: environment.as_ref(),
        },
    );
    if matches!(&result, Err(Error::OwnershipUnconfirmed)) {
        // Keep the original pending marker. Cleanup uncertainty is sticky.
        return result;
    }
    if let Some(execution) = execution {
        execution
            .finish()
            .map_err(|_| Error::OwnershipUnconfirmed)?;
    }
    result
}
fn notify(wake: &Arc<dyn Fn() + Send + Sync>) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake()));
}
impl PreviewWorker {
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
        let failure = uncertain.clone();
        let quit = stop.clone();
        let ack = stopped.clone();
        // Tools snapshot never changes under this worker. Reconfigure only after stop ACK.
        thread::Builder::new()
            .name("dlpad-media-preview".into())
            .spawn(move || {
                struct Ack(Arc<AtomicBool>, Arc<dyn Fn() + Send + Sync>);
                impl Drop for Ack {
                    fn drop(&mut self) {
                        self.0.store(true, Ordering::Release);
                        notify(&self.1);
                    }
                }
                let _ack = Ack(ack, wake.clone());
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    while !quit.load(Ordering::Acquire) {
                        // No idle timeout/poll; capacity1 StopWake releases a blocked receiver.
                        let job = match jobs.recv() {
                            Ok(Command::Run(v)) => v,
                            Ok(Command::WakeStop) => break,
                            Err(_) => break,
                        };
                        if quit.load(Ordering::Acquire) {
                            break;
                        }
                        let value = execute_scoped(
                            &tools,
                            &managed_root,
                            job.request,
                            &job.cancel,
                            job.deadline,
                        );
                        let fatal = value
                            .as_ref()
                            .is_err_and(|e| *e == Error::OwnershipUnconfirmed);
                        if fatal {
                            failure.store(true, Ordering::Release);
                        }
                        if done
                            .try_send(Completion {
                                generation: job.generation,
                                value,
                            })
                            .is_err()
                        {
                            break;
                        }
                        notify(&wake);
                        if fatal {
                            break;
                        }
                    }
                }));
                if outcome.is_err() {
                    // A panic may leave an original process lease pending. Stop ACK alone
                    // is not proof that the owned process tree was recovered.
                    failure.store(true, Ordering::Release);
                }
            })?;
        Ok(Self {
            tx,
            rx,
            stop,
            stopped,
            uncertain,
            generation: 0,
            active: None,
            pending: None,
            terminal: false,
        })
    }
    pub fn submit(&mut self, request: Request) -> Result<(), Error> {
        if self.terminal || self.stop.load(Ordering::Acquire) {
            return Err(Error::OwnershipUnconfirmed);
        }
        self.generation = self.generation.checked_add(1).ok_or(Error::OutputLimit)?;
        if let Some(c) = &self.active {
            c.store(true, Ordering::Release)
        }
        self.pending = Some(request);
        Ok(())
    }
    pub fn cancel(&mut self) {
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next
        } else {
            self.terminal = true
        }
        self.pending = None;
        if let Some(c) = &self.active {
            c.store(true, Ordering::Release)
        }
    }
    pub fn poll(&mut self) -> Option<Result<Product, Error>> {
        let mut result = None;
        if let Ok(done) = self.rx.try_recv() {
            self.active = None;
            if done
                .value
                .as_ref()
                .is_err_and(|e| *e == Error::OwnershipUnconfirmed)
            {
                self.terminal = true;
                self.pending = None;
                result = Some(Err(Error::OwnershipUnconfirmed));
            } else if accepts(self.generation, done.generation) {
                result = Some(done.value)
            }
        }
        if self.uncertain.load(Ordering::Acquire) && !self.terminal {
            self.terminal = true;
            self.pending = None;
            result = Some(Err(Error::OwnershipUnconfirmed));
        }
        if self.stopped.load(Ordering::Acquire) {
            self.terminal = true;
            self.pending = None;
            if self.active.take().is_some()
                && !self.stop.load(Ordering::Acquire)
                && result.is_none()
            {
                result = Some(Err(Error::ToolFailed));
            }
        }
        if self.active.is_none()
            && !self.terminal
            && !self.stop.load(Ordering::Acquire)
            && let Some(request) = self.pending.take()
        {
            let cancel = Arc::new(AtomicBool::new(false));
            let job = Job {
                generation: self.generation,
                request,
                cancel: cancel.clone(),
                deadline: Instant::now() + Duration::from_secs(30),
            };
            match self.tx.try_send(Command::Run(job)) {
                Ok(()) => self.active = Some(cancel),
                Err(TrySendError::Full(Command::Run(job))) => self.pending = Some(job.request),
                Err(_) => {
                    self.terminal = true;
                    result = Some(Err(Error::ToolFailed));
                }
            }
        }
        result
    }
    pub fn request_stop(&mut self) {
        self.cancel();
        self.stop.store(true, Ordering::Release);
        // Full already contains Run: receiver wakes on it then sees atomic stop.
        let _ = self.tx.try_send(Command::WakeStop);
    }
    pub fn busy(&self) -> bool {
        self.active.is_some() || self.pending.is_some()
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}
impl Drop for PreviewWorker {
    fn drop(&mut self) {
        self.request_stop();
    }
} // Drop never means stop ACK.
fn accepts(current: u64, completed: u64) -> bool {
    current == completed
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PreviewWorker, SyncSender<Completion>, Receiver<Command>) {
        let (tx, jobs) = mpsc::sync_channel(1);
        let (done, rx) = mpsc::sync_channel(1);
        (
            PreviewWorker {
                tx,
                rx,
                stop: Arc::new(AtomicBool::new(false)),
                stopped: Arc::new(AtomicBool::new(false)),
                uncertain: Arc::new(AtomicBool::new(false)),
                generation: 1,
                active: None,
                pending: None,
                terminal: false,
            },
            done,
            jobs,
        )
    }
    #[test]
    fn panic_uncertainty_survives_cancellation_and_stop_ack() {
        let (mut worker, _, _) = fixture();
        worker.active = Some(Arc::new(AtomicBool::new(false)));
        worker.request_stop();
        worker.uncertain.store(true, Ordering::Release);
        worker.stopped.store(true, Ordering::Release);
        assert!(matches!(
            worker.poll(),
            Some(Err(Error::OwnershipUnconfirmed))
        ));
        assert!(worker.terminal);
        assert!(!worker.busy());
        assert!(matches!(
            worker.submit(Request::Prepare(dlpad_engine::media_preview::Source::Url(
                "https://fixture.invalid/v".into()
            ))),
            Err(Error::OwnershipUnconfirmed)
        ));
        assert!(worker.poll().is_none());
    }
    #[test]
    fn queued_old_completion_cancel_discard() {
        let (mut w, done, _) = fixture();
        w.active = Some(Arc::new(AtomicBool::new(false)));
        assert!(
            done.send(Completion {
                generation: 1,
                value: Err(Error::TimedOut)
            })
            .is_ok()
        );
        w.cancel();
        assert!(w.poll().is_none());
        assert!(w.active.is_none());
    }
    #[test]
    fn full_queue_stop_preserves_job_and_sets_cancel() {
        let (mut w, _, jobs) = fixture();
        let cancel = Arc::new(AtomicBool::new(false));
        w.active = Some(cancel.clone());
        assert!(
            w.tx.try_send(Command::Run(Job {
                generation: 1,
                request: Request::Prepare(dlpad_engine::media_preview::Source::Url(
                    "https://fixture.invalid/v".into()
                )),
                cancel: cancel.clone(),
                deadline: Instant::now()
            }))
            .is_ok()
        );
        w.request_stop();
        assert!(w.stop.load(Ordering::Acquire));
        assert!(cancel.load(Ordering::Acquire));
        assert!(matches!(jobs.try_recv(), Ok(Command::Run(_))));
    }
    #[test]
    fn empty_queue_stop_wakes_receiver() {
        let (mut w, _, jobs) = fixture();
        w.request_stop();
        assert!(matches!(jobs.try_recv(), Ok(Command::WakeStop)));
    }
    #[test]
    fn unconfirmed_cleanup_is_not_discarded_as_stale() {
        let (mut w, done, _) = fixture();
        w.generation = 2;
        assert!(
            done.send(Completion {
                generation: 1,
                value: Err(Error::OwnershipUnconfirmed)
            })
            .is_ok()
        );
        assert!(matches!(w.poll(), Some(Err(Error::OwnershipUnconfirmed))));
        assert!(w.terminal);
    }
}

#[cfg(test)]
mod begin_execution_regression {
    use super::*;
    #[test]
    #[ignore = "creates owned pending managed fixture; run explicitly on accepted project TEMP"]
    fn abandoned_original_scope_rejects_preview_as_ownership_unconfirmed() {
        let parent = PathBuf::from(
            std::env::var_os("DLPAD_PREVIEW_TEST_PARENT")
                .expect("explicit owned project TEMP required"),
        );
        let parent = parent.canonicalize().expect("existing owned project TEMP");
        let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let accepted = project
            .join(".local-tools/windows/tmp")
            .canonicalize()
            .unwrap();
        assert_eq!(parent, accepted, "only original project TEMP accepted");
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = parent.join(format!(
            "dlpad-preview-begin-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&temp).unwrap();
        let root = temp.join("managed");
        let manager = dlpad_engine::tools::ToolManager::new(&root).unwrap();
        drop(manager.begin_execution().unwrap());
        let marker = root.join(".execution-pending");
        let before = std::fs::read(&marker).unwrap();
        let tools = Tools {
            ytdlp: root.join("never-run-ytdlp"),
            deno: None,
            ffprobe: root.join("never-run-ffprobe"),
            ffmpeg: root.join("never-run-ffmpeg"),
        };
        let result = execute_scoped(
            &tools,
            &root,
            Request::Prepare(dlpad_engine::media_preview::Source::Local(
                temp.join("source"),
            )),
            &Arc::new(AtomicBool::new(false)),
            Instant::now() + Duration::from_secs(5),
        );
        assert!(matches!(result, Err(Error::OwnershipUnconfirmed)));
        assert_eq!(std::fs::read(marker).unwrap(), before);
        assert!(matches!(
            manager.begin_execution(),
            Err(dlpad_engine::tools::ToolError::ExecutionPending)
        ));
    }
}
