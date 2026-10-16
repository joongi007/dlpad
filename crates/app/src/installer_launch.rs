//! Explicit verified launch with retained supervision. No permanent detach.
//! The caller closes its HWND only after Started, then keeps main alive in finish().
use dlpad_engine::app_install::{
    OwnedSelectedProcess, SelectedExitReason, SelectedLaunchFailure, SelectedProcessExit,
    SelectedWaitFailure, inspect_active_selection, inspect_install_journal, inspect_user_bundle,
    launch_explicit_selection,
};
use std::{
    io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub(crate) struct Request {
    pub(crate) bundle_root: PathBuf,
    pub(crate) journal_root: PathBuf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Started {
    pub(crate) process_id: u32,
    pub(crate) creation_filetime: u64,
}
pub(crate) enum Completion {
    Exited(SelectedProcessExit),
    InspectionFailed(io::ErrorKind),
    LaunchFailed(Box<SelectedLaunchFailure>),
    WaitFailed {
        error: SelectedWaitFailure,
        // Retain original owner on an unconfirmed wait, rather than claiming terminal.
        owner: Box<OwnedSelectedProcess>,
    },
    OwnerFailure {
        kind: io::ErrorKind,
        owner: Box<OwnedSelectedProcess>,
    },
    WorkerPanicked,
}
impl Completion {
    pub(crate) fn kind(&self) -> Option<io::ErrorKind> {
        match self {
            Self::Exited(exit) => match exit.reason {
                SelectedExitReason::Cancelled => Some(io::ErrorKind::Interrupted),
                SelectedExitReason::TimedOut => Some(io::ErrorKind::TimedOut),
                SelectedExitReason::Natural if exit.code == 0 => None,
                SelectedExitReason::Natural => Some(io::ErrorKind::Other),
            },
            Self::InspectionFailed(kind) | Self::OwnerFailure { kind, .. } => Some(*kind),
            Self::LaunchFailed(error) => Some(error.kind()),
            Self::WaitFailed { error, .. } => Some(error.kind()),
            Self::WorkerPanicked => Some(io::ErrorKind::Other),
        }
    }
    pub(crate) fn terminal_confirmed(&self) -> bool {
        match self {
            Self::Exited(exit) => exit.job_terminal_confirmed,
            Self::InspectionFailed(_) => true, // Inspection never creates a child.
            Self::LaunchFailed(error) => !error.child_created() || error.terminal_confirmed(),
            Self::WaitFailed { .. } | Self::OwnerFailure { .. } | Self::WorkerPanicked => false,
        }
    }
    pub(crate) fn was_cancelled(&self) -> bool {
        matches!(self, Self::Exited(exit) if exit.reason == SelectedExitReason::Cancelled)
            || self.kind() == Some(io::ErrorKind::Interrupted)
    }
    pub(crate) fn may_have_executed(&self) -> bool {
        match self {
            Self::Exited(_) => true,
            Self::WaitFailed { owner, .. } | Self::OwnerFailure { owner, .. } => {
                owner.may_have_executed()
            }
            Self::LaunchFailed(error) => error.may_have_executed(),
            Self::InspectionFailed(_) => false,
            Self::WorkerPanicked => true, // Cannot infer how far an unwind progressed.
        }
    }
}
/// Main's receipt policy: failed terminal outcomes remain observable until an
/// explicitly accepted retry. Unknown owners can never be discarded by retry.
pub(crate) fn retain_completion(receipt: &mut Option<Completion>, outcome: Completion) -> bool {
    let success = outcome.kind().is_none() && outcome.terminal_confirmed();
    if !success {
        *receipt = Some(outcome);
    }
    success
}
pub(crate) fn clear_confirmed_failure_for_retry(receipt: &mut Option<Completion>) -> bool {
    if receipt
        .as_ref()
        .is_some_and(|outcome| !outcome.terminal_confirmed())
    {
        return false;
    }
    *receipt = None;
    true
}
pub(crate) fn final_receipt_success(receipt: &Option<Completion>) -> bool {
    receipt
        .as_ref()
        .is_none_or(|outcome| outcome.kind().is_none() && outcome.terminal_confirmed())
}

pub(crate) enum Event {
    Pending,
    Started(Started),
    Complete(Completion),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClosePolicy {
    CancelStartup,
    CloseWindowRetainSupervisor,
}

pub(crate) struct Supervisor {
    cancel: Arc<AtomicBool>,
    started: Receiver<Started>,
    worker: Option<JoinHandle<Completion>>,
    accepted_started: bool,
}
impl Supervisor {
    pub(crate) fn start(request: Request, startup_timeout: Duration) -> io::Result<Self> {
        if !(Duration::from_secs(1)..=Duration::from_secs(600)).contains(&startup_timeout)
            || !request.bundle_root.is_absolute()
            || !request.journal_root.is_absolute()
        {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Self::spawn(move |cancel, started| launch(request, cancel, startup_timeout, started))
    }
    fn spawn(
        run: impl FnOnce(Arc<AtomicBool>, SyncSender<Started>) -> Completion + Send + 'static,
    ) -> io::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));
        let (started_tx, started) = mpsc::sync_channel(1);
        let worker_cancel = Arc::clone(&cancel);
        let worker = thread::Builder::new()
            .name("dlpad-installed-app-owner".into())
            .spawn(move || run(worker_cancel, started_tx))?;
        Ok(Self {
            cancel,
            started,
            worker: Some(worker),
            accepted_started: false,
        })
    }
    pub(crate) fn poll(&mut self) -> Event {
        // Preserve an already available terminal failure instead of hiding the window
        // for a stale resume notification from a child that has already exited.
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            return Event::Complete(self.join_worker());
        }
        if !self.accepted_started {
            match self.started.try_recv() {
                Ok(started) => {
                    if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
                        return Event::Complete(self.join_worker());
                    }
                    self.accepted_started = true;
                    return Event::Started(started);
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => {}
            }
        }
        Event::Pending
    }
    pub(crate) fn close_policy(&self) -> ClosePolicy {
        if self.accepted_started && !self.is_cancelled() {
            ClosePolicy::CloseWindowRetainSupervisor
        } else {
            ClosePolicy::CancelStartup
        }
    }
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
    pub(crate) fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    /// Called AFTER window destruction, while main/the owner process still exists.
    /// Normal window close after Started deliberately does not cancel the child.
    pub(crate) fn finish(mut self, normal_window_close: bool) -> Completion {
        if !normal_window_close || self.close_policy() == ClosePolicy::CancelStartup {
            self.cancel();
        }
        self.join_worker()
    }
    fn join_worker(&mut self) -> Completion {
        self.worker
            .take()
            .map_or(Completion::WorkerPanicked, |worker| {
                worker.join().unwrap_or(Completion::WorkerPanicked)
            })
    }
}
impl Drop for Supervisor {
    fn drop(&mut self) {
        if self.worker.is_some() {
            // Abandoning the controller is explicit cancellation, never orphan/detach.
            self.cancel();
            let _completion = self.join_worker();
        }
    }
}

fn launch(
    request: Request,
    cancel: Arc<AtomicBool>,
    timeout: Duration,
    started: SyncSender<Started>,
) -> Completion {
    let deadline = Instant::now() + timeout;
    let inspected = (|| {
        let bundle = inspect_user_bundle(&request.bundle_root, &cancel, deadline)?;
        let journal = inspect_install_journal(&request.journal_root, &cancel, deadline)?;
        journal.verify_active_bundle(&bundle, &cancel, deadline)?;
        // Fixed explicit children; no executable/path discovery from persisted markers.
        // Keep bundle/journal originals live, since bundle exposes only borrowed selection.
        let selection = inspect_active_selection(
            &request.bundle_root.join("active"),
            &request.bundle_root.join("registered"),
            &request.bundle_root.join("receipt"),
            &cancel,
            deadline,
        )?;
        let original = bundle.selection().record().canonical_bytes()?;
        let selected = selection.record().canonical_bytes()?;
        if original.len() > 65536 || selected.len() > 65536 || original != selected {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        journal.verify_active_bundle(&bundle, &cancel, deadline)?;
        Ok((bundle, journal, selection))
    })();
    let (_bundle, _journal, selection) = match inspected {
        Ok(value) => value,
        Err(error) => return Completion::InspectionFailed(error.kind()),
    };
    let mut owner = match launch_explicit_selection(selection, &cancel, deadline) {
        Ok(owner) => owner,
        Err(error) => return Completion::LaunchFailed(Box::new(error)),
    };
    let process_id = match owner.process_id() {
        Ok(id) => id,
        Err(error) => {
            return Completion::OwnerFailure {
                kind: error.kind(),
                owner: Box::new(owner),
            };
        }
    };
    let message = Started {
        process_id,
        creation_filetime: owner.creation_filetime(),
    };
    if started.try_send(message).is_err() {
        return Completion::OwnerFailure {
            kind: io::ErrorKind::BrokenPipe,
            owner: Box::new(owner),
        };
    }
    match owner.wait_cancelled(&cancel) {
        Ok(exit) => Completion::Exited(exit),
        Err(error) => Completion::WaitFailed {
            error,
            owner: Box::new(owner),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn exited() -> Completion {
        Completion::Exited(SelectedProcessExit {
            code: 0,
            reason: SelectedExitReason::Natural,
            job_terminal_confirmed: true,
        })
    }
    #[test]
    fn main_receipt_retains_confirmed_nonzero_exit() {
        let mut receipt = None;
        assert!(!retain_completion(
            &mut receipt,
            Completion::Exited(SelectedProcessExit {
                code: 17,
                reason: SelectedExitReason::Natural,
                job_terminal_confirmed: true,
            })
        ));
        assert!(matches!(&receipt, Some(Completion::Exited(exit)) if exit.code == 17));
        assert!(!final_receipt_success(&receipt));
    }
    #[test]
    fn main_receipt_unknown_cannot_be_cleared_by_retry() {
        let mut receipt = None;
        assert!(!retain_completion(&mut receipt, Completion::WorkerPanicked));
        assert!(!clear_confirmed_failure_for_retry(&mut receipt));
        assert!(matches!(receipt, Some(Completion::WorkerPanicked)));
        assert!(!final_receipt_success(&receipt));
    }
    #[test]
    fn main_receipt_only_explicit_accepted_retry_clears_confirmed_failure() {
        let mut receipt = Some(Completion::InspectionFailed(io::ErrorKind::InvalidData));
        assert!(!final_receipt_success(&receipt));
        assert!(clear_confirmed_failure_for_retry(&mut receipt));
        assert!(receipt.is_none());
    }
    #[test]
    fn main_receipt_successful_retry_has_successful_final_policy() {
        let mut receipt = Some(Completion::InspectionFailed(io::ErrorKind::Other));
        assert!(clear_confirmed_failure_for_retry(&mut receipt));
        assert!(retain_completion(&mut receipt, exited()));
        assert!(final_receipt_success(&receipt));
    }
    #[test]
    fn already_completed_nonzero_exit_wins_over_queued_started_message() {
        let mut supervisor = Supervisor::spawn(|_cancel, started| {
            started
                .send(Started {
                    process_id: 7,
                    creation_filetime: 9,
                })
                .unwrap();
            Completion::Exited(SelectedProcessExit {
                code: 17,
                reason: SelectedExitReason::Natural,
                job_terminal_confirmed: true,
            })
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !supervisor.worker.as_ref().unwrap().is_finished() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        let Event::Complete(outcome) = supervisor.poll() else {
            panic!("completed failure must precede queued startup notification");
        };
        assert_eq!(outcome.kind(), Some(io::ErrorKind::Other));
        assert!(outcome.terminal_confirmed());
        let Completion::Exited(exit) = outcome else {
            panic!("original exit receipt");
        };
        assert_eq!(exit.code, 17);
        assert_eq!(
            supervisor.close_policy(),
            ClosePolicy::CancelStartup,
            "failure must not activate close-window-retain policy"
        );
    }
    #[test]
    fn cancelled_started_owner_keeps_startup_close_policy() {
        let (release, wait) = mpsc::sync_channel::<()>(1);
        let mut supervisor = Supervisor::spawn(move |_cancel, started| {
            started
                .send(Started {
                    process_id: 7,
                    creation_filetime: 9,
                })
                .unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            exited()
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Event::Started(_) = supervisor.poll() {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        supervisor.cancel();
        assert!(supervisor.is_cancelled());
        assert_eq!(supervisor.close_policy(), ClosePolicy::CancelStartup);
        release.send(()).unwrap();
        supervisor.finish(true);
    }
    #[test]
    fn invalid_startup_request_never_spawns_native_or_worker() {
        for timeout in [Duration::ZERO, Duration::from_secs(601)] {
            let result = Supervisor::start(
                Request {
                    bundle_root: PathBuf::from("relative"),
                    journal_root: PathBuf::from("relative-journal"),
                },
                timeout,
            );
            assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::InvalidInput));
        }
        assert!(
            Supervisor::start(
                Request {
                    bundle_root: PathBuf::from("relative"),
                    journal_root: PathBuf::from("relative-journal"),
                },
                Duration::from_secs(1)
            )
            .is_err()
        );
    }
    #[test]
    fn completed_worker_delivers_failure_without_started_or_hidden_owner() {
        let mut supervisor = Supervisor::spawn(|_cancel, _started| {
            Completion::InspectionFailed(io::ErrorKind::InvalidData)
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match supervisor.poll() {
                Event::Complete(outcome) => {
                    assert_eq!(outcome.kind(), Some(io::ErrorKind::InvalidData));
                    assert!(outcome.terminal_confirmed());
                    break;
                }
                Event::Started(_) => panic!("invalid inspection cannot announce launch Started"),
                Event::Pending => {
                    assert!(Instant::now() < deadline);
                    thread::yield_now();
                }
            }
        }
    }
    #[test]
    fn window_close_before_started_cancels_startup_and_joins() {
        let mut supervisor = Supervisor::spawn(|cancel, _started| {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !cancel.load(Ordering::Acquire) && Instant::now() < deadline {
                thread::yield_now();
            }
            assert!(
                cancel.load(Ordering::Acquire),
                "close must reach original worker"
            );
            Completion::InspectionFailed(io::ErrorKind::Interrupted)
        })
        .unwrap();
        assert_eq!(supervisor.close_policy(), ClosePolicy::CancelStartup);
        assert!(matches!(supervisor.poll(), Event::Pending));
        let outcome = supervisor.finish(true);
        assert_eq!(outcome.kind(), Some(io::ErrorKind::Interrupted));
        assert!(outcome.terminal_confirmed());
        assert!(!outcome.may_have_executed());
    }
    #[test]
    fn started_window_close_retains_owner_until_natural_completion() {
        let (release, wait) = mpsc::sync_channel::<()>(1);
        let (observed, observation) = mpsc::sync_channel(1);
        let mut supervisor = Supervisor::spawn(move |cancel, started| {
            started
                .send(Started {
                    process_id: 7,
                    creation_filetime: 9,
                })
                .unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            observed.send(cancel.load(Ordering::Acquire)).unwrap();
            exited()
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match supervisor.poll() {
                Event::Started(started) => {
                    assert_eq!(started.process_id, 7);
                    assert_eq!(started.creation_filetime, 9);
                    break;
                }
                Event::Pending => {
                    assert!(Instant::now() < deadline);
                    thread::yield_now();
                }
                Event::Complete(_) => panic!("worker must await release after Started"),
            }
        }
        assert_eq!(
            supervisor.close_policy(),
            ClosePolicy::CloseWindowRetainSupervisor
        );
        let owner = thread::spawn(move || supervisor.finish(true));
        assert!(
            !owner.is_finished(),
            "window close does not end the owner/process wait"
        );
        release.send(()).unwrap();
        assert!(
            !observation.recv_timeout(Duration::from_secs(2)).unwrap(),
            "normal post-Started UI close must not cancel the app"
        );
        assert!(owner.join().unwrap().terminal_confirmed());
    }
    #[test]
    fn abnormal_message_loop_exit_after_started_cancels_owner() {
        let (release, wait) = mpsc::sync_channel::<()>(1);
        let mut supervisor = Supervisor::spawn(move |cancel, started| {
            started
                .send(Started {
                    process_id: 7,
                    creation_filetime: 9,
                })
                .unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while !cancel.load(Ordering::Acquire) && Instant::now() < deadline {
                thread::yield_now();
            }
            assert!(cancel.load(Ordering::Acquire));
            exited()
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match supervisor.poll() {
                Event::Started(started) => {
                    assert_eq!(started.process_id, 7);
                    assert_eq!(started.creation_filetime, 9);
                    break;
                }
                Event::Pending => {
                    assert!(Instant::now() < deadline);
                    thread::yield_now();
                }
                Event::Complete(_) => panic!("worker must await release after Started"),
            }
        }
        release.send(()).unwrap();
        assert!(supervisor.finish(false).terminal_confirmed());
    }
}
