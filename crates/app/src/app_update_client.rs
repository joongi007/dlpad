//! Bounded UI intents for the parent-owned updater. Only this actor performs IPC.
use dlpad_engine::app_update::Channel;
use eframe::egui;
use std::{
    io,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const POLL: Duration = Duration::from_millis(100);
const RPC_BUDGET: Duration = Duration::from_secs(10);
static CLIENT: OnceLock<ClientHandle> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Requesting,
    Verifying,
    Downloading,
    Staging,
    Preparing,
    Prepared,
    RestartPending,
    RestartAccepted,
    Cancelling,
    Failed,
    Stopped,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub phase: Phase,
    pub version: Option<String>,
    pub generation: u64,
    pub received: u64,
    pub total: u64,
    pub error: Option<u16>,
    pub runtime_ready: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            version: None,
            generation: 0,
            received: 0,
            total: 0,
            error: None,
            runtime_ready: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestError {
    Busy,
    Stopped,
    Invalid,
    Unavailable,
}
#[derive(Clone, Debug)]
enum Command {
    Prepare { version: String, channel: Channel },
    Restart { generation: u64 },
}
#[derive(Debug)]
enum Event {
    Progress {
        phase: u8,
        received: u64,
        total: u64,
        generation: u64,
    },
    Prepared {
        version: String,
        generation: u64,
    },
    Failed {
        code: u16,
        generation: u64,
    },
    RestartAccepted {
        generation: u64,
    },
}
trait Peer: Send + 'static {
    fn runtime_ready(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()>;
    fn send(&mut self, command: Command, cancel: &AtomicBool, deadline: Instant) -> io::Result<()>;
    fn cancel(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()>;
    fn event(&mut self) -> io::Result<Option<Event>>;
}
struct Shared {
    commands: SyncSender<Command>,
    state: Mutex<Snapshot>,
    context: Mutex<Option<egui::Context>>,
    cancel: AtomicBool,
    shutdown: AtomicBool,
    ready: AtomicBool,
    busy: AtomicBool,
    stopped: AtomicBool,
    thread: Mutex<Option<JoinHandle<()>>>,
}
#[derive(Clone)]
pub(crate) struct ClientHandle {
    shared: Arc<Shared>,
}
impl ClientHandle {
    /// Contention is not an idle observation. The caller retains its last snapshot on None.
    pub(crate) fn snapshot(&self) -> Option<Snapshot> {
        self.shared.state.try_lock().ok().map(|state| state.clone())
    }
    pub(crate) fn bind_context(&self, context: &egui::Context) {
        if let Ok(mut value) = self.shared.context.try_lock() {
            *value = Some(context.clone());
        }
    }
    fn dispatch(&self, command: Command) -> Result<(), RequestError> {
        if self.shared.shutdown.load(Ordering::Acquire) || self.stopped() {
            return Err(RequestError::Stopped);
        }
        if self
            .shared
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(RequestError::Busy);
        }
        self.shared.cancel.store(false, Ordering::Release);
        match self.shared.commands.try_send(command) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.shared.busy.store(false, Ordering::Release);
                Err(match error {
                    TrySendError::Full(_) => RequestError::Busy,
                    TrySendError::Disconnected(_) => RequestError::Unavailable,
                })
            }
        }
    }
    pub(crate) fn prepare(&self, version: &str, channel: Channel) -> Result<(), RequestError> {
        if !valid_version(version) {
            return Err(RequestError::Invalid);
        }
        if !self.snapshot().is_some_and(|state| state.runtime_ready) {
            return Err(RequestError::Unavailable);
        }
        self.dispatch(Command::Prepare {
            version: version.to_owned(),
            channel,
        })
    }
    pub(crate) fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
    }
    pub(crate) fn request_restart(&self, generation: u64) -> Result<(), RequestError> {
        let state = self.snapshot().ok_or(RequestError::Busy)?;
        if state.phase != Phase::Prepared || state.generation != generation || generation == 0 {
            return Err(RequestError::Invalid);
        }
        self.dispatch(Command::Restart { generation })
    }
    pub(crate) fn notify_runtime_ready(&self) {
        self.shared.ready.store(true, Ordering::Release);
    }
    pub(crate) fn begin_shutdown(&self) {
        self.shared.shutdown.store(true, Ordering::Release);
        self.shared.cancel.store(true, Ordering::Release);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.shared.stopped.load(Ordering::Acquire)
    }
    pub(crate) fn busy(&self) -> bool {
        self.shared.busy.load(Ordering::Acquire)
    }
}
pub(crate) fn handle() -> Option<ClientHandle> {
    CLIENT.get().cloned()
}
pub(crate) fn notify_runtime_ready() {
    if let Some(client) = handle() {
        client.notify_runtime_ready();
    }
}
fn valid_version(value: &str) -> bool {
    if value.len() > 32 {
        return false;
    }
    let mut parts = value.split('.');
    let valid = (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            !part.is_empty()
                && part.len() <= 9
                && (part == "0" || !part.starts_with('0'))
                && part.bytes().all(|b| b.is_ascii_digit())
        })
    });
    valid && parts.next().is_none()
}
fn publish(shared: &Shared, state: &Snapshot) {
    let changed = if let Ok(mut current) = shared.state.lock() {
        if *current == *state {
            false
        } else {
            *current = state.clone();
            true
        }
    } else {
        false
    };
    if changed
        && let Ok(context) = shared.context.lock()
        && let Some(context) = &*context
    {
        context.request_repaint();
    }
}
fn start(peer: impl Peer) -> io::Result<ClientHandle> {
    let (commands, receiver) = mpsc::sync_channel(1);
    let shared = Arc::new(Shared {
        commands,
        state: Mutex::new(Snapshot::default()),
        context: Mutex::new(None),
        cancel: AtomicBool::new(false),
        shutdown: AtomicBool::new(false),
        ready: AtomicBool::new(false),
        busy: AtomicBool::new(false),
        stopped: AtomicBool::new(false),
        thread: Mutex::new(None),
    });
    let owned = shared.clone();
    let thread = thread::Builder::new()
        .name("dlpad-update-client".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                actor(peer, receiver, &owned)
            }));
            if result.is_err() {
                let mut state = owned.state.lock().map(|s| s.clone()).unwrap_or_default();
                state.phase = Phase::Failed;
                state.error = Some(11);
                publish(&owned, &state);
            }
            owned.busy.store(false, Ordering::Release);
            owned.stopped.store(true, Ordering::Release);
            if let Ok(context) = owned.context.lock()
                && let Some(context) = &*context
            {
                context.request_repaint();
            }
        })?;
    *shared
        .thread
        .lock()
        .map_err(|_| io::Error::other("updater client lock"))? = Some(thread);
    Ok(ClientHandle { shared })
}
fn actor(mut peer: impl Peer, receiver: Receiver<Command>, shared: &Shared) {
    let mut state = Snapshot::default();
    let mut preparing = false;
    let mut cancel_sent = false;
    let mut previous_generation = 0;
    loop {
        let shutdown = shared.shutdown.load(Ordering::Acquire);
        if shutdown && !preparing {
            break;
        }
        if !state.runtime_ready && shared.ready.load(Ordering::Acquire) {
            if peer
                .runtime_ready(&shared.shutdown, Instant::now() + RPC_BUDGET)
                .is_err()
            {
                state.phase = Phase::Failed;
                state.error = Some(10);
                publish(shared, &state);
                break;
            }
            state.runtime_ready = true;
            publish(shared, &state);
        }
        if preparing && shared.cancel.load(Ordering::Acquire) && !cancel_sent {
            // A fresh token allows the cancellation request itself to be written.
            if peer
                .cancel(&AtomicBool::new(false), Instant::now() + RPC_BUDGET)
                .is_err()
            {
                state.phase = Phase::Failed;
                state.error = Some(10);
                publish(shared, &state);
                break;
            }
            cancel_sent = true;
            state.phase = Phase::Cancelling;
            publish(shared, &state);
        }
        match if state.runtime_ready {
            peer.event()
        } else {
            Ok(None)
        } {
            Ok(Some(event)) => {
                match event {
                    Event::Progress {
                        phase,
                        received,
                        total,
                        generation,
                    } => {
                        if !preparing
                            || generation == 0
                            || generation <= previous_generation
                            || (state.generation > previous_generation
                                && state.generation != generation)
                        {
                            continue;
                        }
                        if phase > 4 || received > total || total > 256 * 1024 * 1024 {
                            state.phase = Phase::Failed;
                            state.error = Some(10);
                            publish(shared, &state);
                            break;
                        }
                        state.generation = generation;
                        state.received = received;
                        state.total = total;
                        if !cancel_sent {
                            state.phase = match phase {
                                0 => Phase::Verifying,
                                1 => Phase::Downloading,
                                2 => Phase::Staging,
                                3 => Phase::Preparing,
                                _ => Phase::Preparing,
                            };
                        }
                    }
                    Event::Prepared {
                        version,
                        generation,
                    } => {
                        if !preparing
                            || generation == 0
                            || generation <= previous_generation
                            || (state.generation > previous_generation
                                && state.generation != generation)
                        {
                            continue;
                        }
                        if !valid_version(&version) || state.version.as_deref() != Some(&version) {
                            state.phase = Phase::Failed;
                            state.error = Some(10);
                            publish(shared, &state);
                            break;
                        }
                        state.generation = generation;
                        if cancel_sent {
                            // Preparation won the computation race, but the queued cancellation
                            // still belongs to the parent. Wait for its operation terminal response.
                            state.phase = Phase::Cancelling;
                        } else {
                            state.phase = Phase::Prepared;
                            state.error = None;
                            preparing = false;
                            shared.busy.store(false, Ordering::Release);
                        }
                    }
                    Event::Failed { code, generation } => {
                        if code == 13 {
                            if generation == 0
                                && state.runtime_ready
                                && state.phase == Phase::Idle
                                && !preparing
                                && !shared.busy.load(Ordering::Acquire)
                            {
                                state.phase = Phase::Failed;
                                state.error = Some(13);
                                publish(shared, &state);
                            }
                            continue;
                        }
                        if !preparing && state.phase != Phase::RestartPending {
                            continue;
                        }
                        if generation == 0
                            || generation < state.generation
                            || (preparing && generation <= previous_generation)
                        {
                            continue;
                        }
                        if code == 9
                            && state.phase == Phase::RestartPending
                            && generation == state.generation
                        {
                            state.phase = Phase::Prepared;
                            state.error = Some(code);
                        } else {
                            if generation > state.generation {
                                state.generation = generation;
                            }
                            state.phase = Phase::Failed;
                            state.error = Some(if (1..=12).contains(&code) { code } else { 10 });
                        }
                        preparing = false;
                        cancel_sent = false;
                        shared.busy.store(false, Ordering::Release);
                    }
                    Event::RestartAccepted { generation } => {
                        if state.phase != Phase::RestartPending
                            || generation != state.generation
                            || generation == 0
                        {
                            continue;
                        }
                        state.phase = Phase::RestartAccepted;
                        state.error = None;
                        shared.busy.store(false, Ordering::Release);
                    }
                }
                publish(shared, &state);
            }
            Ok(None) => (),
            Err(_) => {
                state.phase = Phase::Failed;
                state.error = Some(10);
                publish(shared, &state);
                break;
            }
        }
        match receiver.recv_timeout(POLL) {
            Ok(command) => {
                if shared.shutdown.load(Ordering::Acquire) {
                    shared.busy.store(false, Ordering::Release);
                    continue;
                }
                match &command {
                    Command::Prepare { version, .. } => {
                        previous_generation = state.generation;
                        state.version = Some(version.clone());
                        state.phase = Phase::Requesting;
                        state.error = None;
                        state.received = 0;
                        state.total = 0;
                        preparing = true;
                        cancel_sent = false;
                    }
                    Command::Restart { generation } => {
                        if state.phase != Phase::Prepared || *generation != state.generation {
                            shared.busy.store(false, Ordering::Release);
                            continue;
                        }
                        state.phase = Phase::RestartPending;
                        state.error = None;
                    }
                }
                publish(shared, &state);
                if peer
                    .send(command, &shared.shutdown, Instant::now() + RPC_BUDGET)
                    .is_err()
                {
                    state.phase = Phase::Failed;
                    state.error = Some(10);
                    publish(shared, &state);
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    // The peer closes before the stopped ACK is published by the outer owner.
    drop(peer);
    if shared.shutdown.load(Ordering::Acquire) {
        state.phase = Phase::Stopped;
        publish(shared, &state);
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use dlpad_engine::app_update::bootstrap::{ParentEvent, UiChild, UiRequest};
    impl Peer for UiChild {
        fn runtime_ready(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
            UiChild::runtime_ready(self, cancel, deadline)
        }
        fn send(
            &mut self,
            command: Command,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> io::Result<()> {
            let command = match command {
                Command::Prepare { version, channel } => UiRequest::Prepare { version, channel },
                Command::Restart { generation } => UiRequest::RestartPrepared { generation },
            };
            self.send_request(&command, cancel, deadline)
        }
        fn cancel(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
            self.send_request(&UiRequest::CancelPrepare, cancel, deadline)
        }
        fn event(&mut self) -> io::Result<Option<Event>> {
            Ok(self.try_event()?.map(|event| match event {
                ParentEvent::Progress {
                    phase,
                    received,
                    total,
                    generation,
                } => Event::Progress {
                    phase,
                    received,
                    total,
                    generation,
                },
                ParentEvent::Prepared {
                    version,
                    generation,
                } => Event::Prepared {
                    version,
                    generation,
                },
                ParentEvent::Failed { code, generation } => Event::Failed { code, generation },
                ParentEvent::RestartAccepted { generation } => {
                    Event::RestartAccepted { generation }
                }
            }))
        }
    }
    pub(crate) fn install(peer: UiChild) -> io::Result<()> {
        if CLIENT.get().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "updater client installed",
            ));
        }
        let client = start(peer)?;
        if let Err(client) = CLIENT.set(client) {
            client.begin_shutdown();
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "updater client installed",
            ));
        }
        Ok(())
    }
}
#[cfg(windows)]
pub(crate) fn install(peer: dlpad_engine::app_update::bootstrap::UiChild) -> io::Result<()> {
    windows::install(peer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    enum Seen {
        Prepare(String),
        Cancel,
        Restart(u64),
        Ready,
    }
    struct FakePeer {
        events: Receiver<Event>,
        seen: SyncSender<Seen>,
    }
    impl Peer for FakePeer {
        fn runtime_ready(&mut self, _: &AtomicBool, _: Instant) -> io::Result<()> {
            self.seen.try_send(Seen::Ready).map_err(io::Error::other)
        }
        fn send(&mut self, command: Command, _: &AtomicBool, _: Instant) -> io::Result<()> {
            self.seen
                .try_send(match command {
                    Command::Prepare { version, .. } => Seen::Prepare(version),
                    Command::Restart { generation } => Seen::Restart(generation),
                })
                .map_err(io::Error::other)
        }
        fn cancel(&mut self, _: &AtomicBool, _: Instant) -> io::Result<()> {
            self.seen.try_send(Seen::Cancel).map_err(io::Error::other)
        }
        fn event(&mut self) -> io::Result<Option<Event>> {
            match self.events.try_recv() {
                Ok(event) => Ok(Some(event)),
                Err(mpsc::TryRecvError::Empty) => Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "fixture disconnect",
                )),
            }
        }
    }
    struct Harness {
        client: ClientHandle,
        events: SyncSender<Event>,
        seen: Receiver<Seen>,
    }
    impl Harness {
        fn new() -> Self {
            let (events, receiver) = mpsc::sync_channel(16);
            let (sender, seen) = mpsc::sync_channel(16);
            let client = start(FakePeer {
                events: receiver,
                seen: sender,
            })
            .unwrap();
            let harness = Self {
                client,
                events,
                seen,
            };
            harness.client.notify_runtime_ready();
            assert!(matches!(
                harness.seen.recv_timeout(Duration::from_secs(2)).unwrap(),
                Seen::Ready
            ));
            harness.state(|state| state.runtime_ready);
            harness
        }
        fn state(&self, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
            let deadline = Instant::now() + Duration::from_secs(4);
            loop {
                if let Some(state) = self.client.snapshot()
                    && predicate(&state)
                {
                    return state;
                }
                assert!(Instant::now() < deadline, "bounded fixture actor response");
                thread::sleep(Duration::from_millis(2));
            }
        }
        fn prepare(&self) {
            self.client.prepare("0.2.0", Channel::Stable).unwrap();
            assert!(
                matches!(self.seen.recv_timeout(Duration::from_secs(2)).unwrap(), Seen::Prepare(version) if version == "0.2.0")
            );
        }
        fn prepared(&self) {
            self.prepare();
            self.events
                .send(Event::Prepared {
                    version: "0.2.0".into(),
                    generation: 1,
                })
                .unwrap();
            self.state(|state| state.phase == Phase::Prepared);
        }
    }
    impl Drop for Harness {
        fn drop(&mut self) {
            self.client.begin_shutdown();
            let generation = self
                .client
                .snapshot()
                .map(|s| s.generation.max(1))
                .unwrap_or(1);
            let _ = self.events.try_send(Event::Failed {
                code: 1,
                generation,
            });
            let deadline = Instant::now() + Duration::from_secs(4);
            while !self.client.stopped() {
                assert!(
                    Instant::now() < deadline,
                    "fixture owns actor through stopped ACK"
                );
                thread::sleep(Duration::from_millis(2));
            }
            if let Some(thread) = self.client.shared.thread.lock().unwrap().take() {
                thread.join().unwrap();
            }
        }
    }
    #[test]
    fn invalid_and_duplicate_intents_do_not_reach_the_peer() {
        let h = Harness::new();
        assert_eq!(
            h.client.prepare("0.02.0", Channel::Stable),
            Err(RequestError::Invalid)
        );
        h.prepare();
        assert_eq!(
            h.client.prepare("0.3.0", Channel::Stable),
            Err(RequestError::Busy)
        );
        assert_eq!(h.client.request_restart(1), Err(RequestError::Invalid));
        assert!(h.seen.try_recv().is_err());
        h.events
            .send(Event::Failed {
                code: 3,
                generation: 1,
            })
            .unwrap();
        h.state(|state| state.phase == Phase::Failed);
    }
    #[test]
    fn readiness_is_worker_owned_and_once_only() {
        let h = Harness::new();
        h.client.notify_runtime_ready();
        h.client.notify_runtime_ready();
        h.state(|state| state.runtime_ready);
        assert!(h.seen.try_recv().is_err());
    }
    #[test]
    fn shutdown_waits_for_prepare_terminal_ack_and_preserves_cancel_result() {
        let h = Harness::new();
        h.prepare();
        h.client.begin_shutdown();
        assert!(matches!(
            h.seen.recv_timeout(Duration::from_secs(2)).unwrap(),
            Seen::Cancel
        ));
        assert!(!h.client.stopped());
        assert_eq!(
            h.client.prepare("0.3.0", Channel::Stable),
            Err(RequestError::Stopped)
        );
        h.events
            .send(Event::Failed {
                code: 1,
                generation: 1,
            })
            .unwrap();
        let state = h.state(|state| state.phase == Phase::Stopped);
        assert_eq!(state.error, Some(1));
    }
    #[test]
    fn restart_requires_matching_prepared_generation_and_parent_acceptance() {
        let h = Harness::new();
        h.prepared();
        assert!(!h.client.busy());
        assert_eq!(h.client.request_restart(2), Err(RequestError::Invalid));
        h.client.request_restart(1).unwrap();
        assert!(matches!(
            h.seen.recv_timeout(Duration::from_secs(2)).unwrap(),
            Seen::Restart(1)
        ));
        h.events
            .send(Event::RestartAccepted { generation: 2 })
            .unwrap();
        h.events
            .send(Event::Failed {
                code: 9,
                generation: 1,
            })
            .unwrap();
        let state = h.state(|state| state.phase == Phase::Prepared && state.error == Some(9));
        assert_eq!(state.version.as_deref(), Some("0.2.0"));
        h.client.request_restart(1).unwrap();
        assert!(matches!(
            h.seen.recv_timeout(Duration::from_secs(2)).unwrap(),
            Seen::Restart(1)
        ));
        h.events
            .send(Event::RestartAccepted { generation: 1 })
            .unwrap();
        h.state(|state| state.phase == Phase::RestartAccepted);
    }
    #[test]
    fn stale_events_cannot_replace_new_attempt_and_progress_is_bounded() {
        let h = Harness::new();
        h.prepared();
        h.client.prepare("0.3.0", Channel::Preview).unwrap();
        assert!(
            matches!(h.seen.recv_timeout(Duration::from_secs(2)).unwrap(), Seen::Prepare(version) if version == "0.3.0")
        );
        h.events
            .send(Event::Prepared {
                version: "0.2.0".into(),
                generation: 1,
            })
            .unwrap();
        h.events
            .send(Event::Progress {
                phase: 1,
                received: 5,
                total: 10,
                generation: 2,
            })
            .unwrap();
        let state = h.state(|state| state.generation == 2 && state.phase == Phase::Downloading);
        assert_eq!((state.received, state.total), (5, 10));
        h.events
            .send(Event::Progress {
                phase: 1,
                received: 11,
                total: 10,
                generation: 2,
            })
            .unwrap();
        h.state(|state| state.phase == Phase::Failed && state.error == Some(10));
    }
    #[test]
    fn cancelled_preparation_does_not_adopt_racing_prepared_before_parent_terminal() {
        for code in [1, 4] {
            let h = Harness::new();
            h.prepare();
            h.client.cancel();
            assert!(matches!(
                h.seen.recv_timeout(Duration::from_secs(2)).unwrap(),
                Seen::Cancel
            ));
            h.events
                .send(Event::Prepared {
                    version: "0.2.0".into(),
                    generation: 1,
                })
                .unwrap();
            h.state(|state| state.phase == Phase::Cancelling && state.generation == 1);
            assert!(
                h.client.busy(),
                "Prepared is not the cancellation terminal response"
            );
            assert_eq!(h.client.request_restart(1), Err(RequestError::Invalid));
            h.events
                .send(Event::Failed {
                    code,
                    generation: 1,
                })
                .unwrap();
            h.state(|state| state.phase == Phase::Failed && state.error == Some(code));
            assert!(
                !h.client.busy(),
                "both cancellation ACK and a raced real failure are terminal"
            );
        }
    }
    #[test]
    fn cold_recovery_notice_is_only_accepted_before_any_request_at_idle_generation_zero() {
        let h = Harness::new();
        h.events
            .send(Event::Failed {
                code: 13,
                generation: 0,
            })
            .unwrap();
        h.state(|state| state.phase == Phase::Failed && state.error == Some(13));
        assert!(!h.client.busy());
        h.prepare();
        h.events
            .send(Event::Failed {
                code: 13,
                generation: 0,
            })
            .unwrap();
        h.events
            .send(Event::Progress {
                phase: 1,
                received: 1,
                total: 10,
                generation: 1,
            })
            .unwrap();
        let state = h.state(|state| state.phase == Phase::Downloading && state.generation == 1);
        assert_eq!(
            state.error, None,
            "startup notice cannot replace a newer operation"
        );
        h.events
            .send(Event::Failed {
                code: 3,
                generation: 1,
            })
            .unwrap();
        h.state(|state| state.phase == Phase::Failed && state.error == Some(3));
    }
    #[test]
    fn ui_snapshot_never_waits_for_the_actor_state_lock() {
        let h = Harness::new();
        let _locked = h.client.shared.state.lock().unwrap();
        assert!(h.client.snapshot().is_none());
    }
}
