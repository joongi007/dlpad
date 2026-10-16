//! Native filesystem hints. Events coalesce; reconciliation remains authoritative.
use notify::{RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchError {
    /// The backend lost events or failed. Reconcile and recreate the watch if needed.
    BackendFailure,
}

#[derive(Default)]
struct Signals {
    dirty: AtomicBool,
    error: AtomicBool,
}
impl Signals {
    fn record(&self, event: notify::Result<notify::Event>) {
        match event {
            // Reading files while reconciling must not trigger endless re-scans.
            Ok(event) if !matches!(event.kind, notify::EventKind::Access(_)) => {
                self.dirty.store(true, Ordering::Release);
            }
            Ok(_) => (),
            Err(_) => {
                self.error.store(true, Ordering::Release);
                self.dirty.store(true, Ordering::Release);
            }
        }
    }
}

pub struct Watcher {
    // Retaining the owned backend retains its OS watch. Drop stops it.
    _backend: RecommendedWatcher,
    signals: Arc<Signals>,
}
impl Watcher {
    pub fn new(root: &Path) -> notify::Result<Self> {
        let metadata = std::fs::symlink_metadata(root).map_err(notify::Error::io)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(notify::Error::generic(
                "watch root must be a directory, not a symlink",
            ));
        }
        let signals = Arc::new(Signals::default());
        let callback = Arc::clone(&signals);
        let mut backend = notify::recommended_watcher(move |event| callback.record(event))?;
        backend.watch(root, RecursiveMode::Recursive)?;
        Ok(Self {
            _backend: backend,
            signals,
        })
    }

    /// Atomically acknowledges changes. Events arriving afterward remain pending.
    /// Poll from the worker after debounce; this does not perform a filesystem scan.
    pub fn changed(&self) -> bool {
        self.signals.dirty.swap(false, Ordering::AcqRel)
    }

    /// Fixed-size failure marker avoids retaining backend path vectors or logs.
    pub fn take_error(&self) -> Option<WatchError> {
        self.signals
            .error
            .swap(false, Ordering::AcqRel)
            .then_some(WatchError::BackendFailure)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchHealth {
    Active,
    Unavailable,
}
/// Highest-priority reason for an authoritative reconciliation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchCause {
    Periodic,
    ChangeHint,
    Recovery,
}
#[derive(Debug, Default)]
pub struct WatchPoll {
    pub dirty: bool,
    pub cause: Option<WatchCause>,
    pub health: Option<WatchHealth>,
}
#[cfg(unix)]
type RootIdentity = (u64, u64);
#[cfg(not(unix))]
type RootIdentity = Option<std::time::SystemTime>;
#[cfg(test)]
std::thread_local! {
    static ROOT_METADATA_OBSERVATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
fn root_metadata(root: &Path) -> notify::Result<std::fs::Metadata> {
    #[cfg(test)]
    ROOT_METADATA_OBSERVATIONS.with(|count| count.set(count.get() + 1));
    std::fs::symlink_metadata(root).map_err(notify::Error::io)
}
fn root_observation(root: &Path) -> notify::Result<(RootIdentity, Option<std::time::SystemTime>)> {
    let metadata = root_metadata(root)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(notify::Error::generic(
            "watch root must be a directory, not a symlink",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(((metadata.dev(), metadata.ino()), metadata.modified().ok()))
    }
    #[cfg(not(unix))]
    {
        Ok((metadata.created().ok(), metadata.modified().ok()))
    }
}
const ORDINARY_HINT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// A fixed deadline preserves the last hint even when changes never become quiet.
struct OrdinaryHintGate {
    pending: bool,
    next_due: std::time::Instant,
}
impl OrdinaryHintGate {
    fn new(now: std::time::Instant) -> Self {
        Self {
            pending: false,
            next_due: now,
        }
    }
    fn poll(&mut self, now: std::time::Instant, ordinary: bool, urgent: bool) -> bool {
        self.pending |= ordinary;
        if urgent || (self.pending && now >= self.next_due) {
            self.pending = false;
            self.next_due = now + ORDINARY_HINT_INTERVAL;
            true
        } else {
            false
        }
    }
}

/// A single owned native watch with fixed retry spacing, never an event queue.
pub struct WatchSession {
    root: std::path::PathBuf,
    watch: Option<(Watcher, RootIdentity)>,
    health: Option<WatchHealth>,
    retry_at: std::time::Instant,
    root_modified: Option<std::time::SystemTime>,
    reconcile_at: std::time::Instant,
    ordinary_hints: OrdinaryHintGate,
}
impl WatchSession {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            root,
            watch: None,
            health: None,
            retry_at: std::time::Instant::now(),
            root_modified: None,
            reconcile_at: std::time::Instant::now(),
            ordinary_hints: OrdinaryHintGate::new(std::time::Instant::now()),
        }
    }
    pub fn poll(&mut self) -> WatchPoll {
        self.poll_at(std::time::Instant::now())
    }
    fn poll_at(&mut self, now: std::time::Instant) -> WatchPoll {
        let mut result = WatchPoll::default();
        let mut ordinary_hint = false;
        if let Some((watch, identity)) = &self.watch {
            ordinary_hint = watch.changed();
            let observation = root_observation(&self.root);
            let modified = observation
                .as_ref()
                .ok()
                .and_then(|(_, modified)| *modified);
            let periodic = now >= self.reconcile_at;
            if modified != self.root_modified || periodic {
                ordinary_hint |= modified != self.root_modified;
                self.root_modified = modified;
                self.reconcile_at = now + std::time::Duration::from_secs(30);
                result.dirty = periodic;
                result.cause = periodic.then_some(WatchCause::Periodic);
            }
            if watch.take_error().is_some()
                || observation.as_ref().ok().map(|(observed, _)| observed) != Some(identity)
            {
                self.watch = None;
                self.retry_at = now + std::time::Duration::from_secs(2);
                result.dirty = true;
                result.cause = Some(WatchCause::Recovery);
                self.set_health(WatchHealth::Unavailable, &mut result);
            }
        }
        if self.watch.is_none() && now >= self.retry_at {
            let installed = root_observation(&self.root).and_then(|(before, _)| {
                let watcher = Watcher::new(&self.root)?;
                let (after, modified) = root_observation(&self.root)?;
                if after != before {
                    return Err(notify::Error::generic("root changed while attaching watch"));
                }
                Ok(((watcher, after), modified))
            });
            match installed {
                Ok((watch, modified)) => {
                    self.watch = Some(watch);
                    self.root_modified = modified;
                    self.reconcile_at = now + std::time::Duration::from_secs(30);
                    result.dirty = true; // Reconcile changes missed while detached.
                    result.cause = Some(WatchCause::Recovery);
                    self.set_health(WatchHealth::Active, &mut result);
                }
                Err(_) => {
                    self.retry_at = now + std::time::Duration::from_secs(2);
                    self.set_health(WatchHealth::Unavailable, &mut result);
                }
            }
        }
        let pending_hint = self.ordinary_hints.pending || ordinary_hint;
        result.dirty =
            self.ordinary_hints
                .poll(now, ordinary_hint, result.dirty || result.health.is_some());
        result.cause = if result.dirty {
            if result.health.is_some() {
                Some(WatchCause::Recovery)
            } else if pending_hint {
                Some(
                    result
                        .cause
                        .unwrap_or(WatchCause::ChangeHint)
                        .max(WatchCause::ChangeHint),
                )
            } else {
                result.cause
            }
        } else {
            None
        };
        result
    }
    fn set_health(&mut self, health: WatchHealth, result: &mut WatchPoll) {
        if self.health != Some(health) {
            self.health = Some(health);
            result.health = Some(health);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_change_outranks_periodic_and_attach_is_recovery() {
        let directory = tempfile::tempdir().unwrap();
        let mut session = WatchSession::new(directory.path().into());
        let now = std::time::Instant::now();
        assert_eq!(session.poll_at(now).cause, Some(WatchCause::Recovery));
        session.ordinary_hints.pending = true;
        let mixed = session.poll_at(now + std::time::Duration::from_secs(30));
        assert!(mixed.dirty);
        assert_eq!(mixed.cause, Some(WatchCause::ChangeHint));
        assert_eq!(
            session
                .poll_at(now + std::time::Duration::from_secs(60))
                .cause,
            Some(WatchCause::Periodic)
        );
    }

    #[test]
    fn continuous_hints_flush_on_fixed_due_and_preserve_last_pending_hint() {
        let now = std::time::Instant::now();
        let mut gate = OrdinaryHintGate::new(now);
        assert!(gate.poll(now, false, true));
        for ms in [100, 250, 500, 750, 999] {
            assert!(!gate.poll(now + std::time::Duration::from_millis(ms), true, false));
        }
        assert!(gate.poll(now + std::time::Duration::from_secs(1), false, false));
        assert!(!gate.poll(now + std::time::Duration::from_millis(1001), true, false));
        assert!(gate.poll(now + std::time::Duration::from_secs(2), false, false));
        assert!(!gate.poll(now + std::time::Duration::from_secs(3), false, false));
    }
    #[test]
    fn urgent_flush_consumes_pending_and_never_waits_for_ordinary_due() {
        let now = std::time::Instant::now();
        let mut gate = OrdinaryHintGate::new(now);
        assert!(gate.poll(now, false, true));
        assert!(!gate.poll(now + std::time::Duration::from_millis(100), true, false));
        assert!(gate.poll(now + std::time::Duration::from_millis(200), false, true));
        assert!(!gate.poll(now + std::time::Duration::from_secs(1), false, false));
        assert!(!gate.poll(now + std::time::Duration::from_millis(1200), false, false));
        assert!(gate.poll(now + std::time::Duration::from_millis(1201), true, false));
    }
    #[test]
    fn empty_poll_does_not_move_deadline_or_make_busy_dirty() {
        let now = std::time::Instant::now();
        let mut gate = OrdinaryHintGate::new(now);
        assert!(gate.poll(now, false, true));
        for ms in 1..1000 {
            assert!(!gate.poll(now + std::time::Duration::from_millis(ms), false, false));
        }
        assert!(gate.poll(now + std::time::Duration::from_secs(1), true, false));
        assert!(!gate.poll(now + std::time::Duration::from_secs(30), false, false));
        // Periodic is a caller's urgent fact, never lost by lack of native hints.
        assert!(gate.poll(now + std::time::Duration::from_secs(30), false, true));
    }

    #[test]
    fn active_poll_uses_single_root_metadata_observation() {
        let directory = tempfile::tempdir().unwrap();
        let mut session = WatchSession::new(directory.path().into());
        let now = std::time::Instant::now();
        assert_eq!(session.poll_at(now).health, Some(WatchHealth::Active));
        ROOT_METADATA_OBSERVATIONS.with(|count| count.set(0));
        let update = session.poll_at(now + std::time::Duration::from_millis(100));
        assert!(!update.dirty);
        assert_eq!(update.health, None);
        ROOT_METADATA_OBSERVATIONS.with(|count| assert_eq!(count.get(), 1));
    }

    #[test]
    fn root_mtime_hint_is_retained_until_due_without_another_event() {
        let directory = tempfile::tempdir().unwrap();
        let mut session = WatchSession::new(directory.path().into());
        let now = std::time::Instant::now();
        assert!(session.poll_at(now).dirty);
        // Force a stale observation while retaining the real, unchanged namespace.
        session.root_modified = Some(std::time::SystemTime::UNIX_EPOCH);
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_millis(100))
                .dirty
        );
        assert!(
            session
                .poll_at(now + std::time::Duration::from_secs(1))
                .dirty
        );
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_millis(1100))
                .dirty
        );
    }
    #[test]
    fn backend_failure_flushes_pending_before_due_and_reconnect_stays_urgent() {
        let directory = tempfile::tempdir().unwrap();
        let mut session = WatchSession::new(directory.path().into());
        let now = std::time::Instant::now();
        assert!(session.poll_at(now).dirty);
        let signals = session.watch.as_ref().unwrap().0.signals.clone();
        signals.record(Ok(notify::Event::new(notify::EventKind::Any)));
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_millis(100))
                .dirty
        );
        signals.record(Err(notify::Error::generic("fixture backend failure")));
        let failure = session.poll_at(now + std::time::Duration::from_millis(200));
        assert!(failure.dirty);
        assert_eq!(failure.health, Some(WatchHealth::Unavailable));
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_secs(1))
                .dirty
        );
        let recovered = session.poll_at(now + std::time::Duration::from_millis(2200));
        assert!(recovered.dirty);
        assert_eq!(recovered.health, Some(WatchHealth::Active));
    }
    #[test]
    fn periodic_reconciliation_covers_silently_missing_native_events() {
        let directory = tempfile::tempdir().unwrap();
        let mut session = WatchSession::new(directory.path().into());
        let now = std::time::Instant::now();
        assert!(session.poll_at(now).dirty);
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_secs(1))
                .dirty
        );
        assert!(
            session
                .poll_at(now + std::time::Duration::from_secs(30))
                .dirty
        );
        assert!(
            !session
                .poll_at(now + std::time::Duration::from_secs(31))
                .dirty
        );
    }
    #[test]
    fn reconnects_after_missing_root_appears_without_busy_retry() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("戻った 폴더");
        let mut session = WatchSession::new(root.clone());
        let now = std::time::Instant::now();
        assert_eq!(session.poll_at(now).health, Some(WatchHealth::Unavailable));
        std::fs::create_dir(&root).unwrap();
        assert_eq!(
            session
                .poll_at(now + std::time::Duration::from_secs(1))
                .health,
            None
        );
        let recovered = session.poll_at(now + std::time::Duration::from_secs(2));
        assert_eq!(recovered.health, Some(WatchHealth::Active));
        assert!(recovered.dirty);
    }
    #[cfg(unix)]
    #[test]
    fn replacement_directory_reconnects_using_identity() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        std::fs::create_dir(&root).unwrap();
        let mut session = WatchSession::new(root.clone());
        let now = std::time::Instant::now();
        assert_eq!(session.poll_at(now).health, Some(WatchHealth::Active));
        std::fs::rename(&root, directory.path().join("detached")).unwrap();
        std::fs::create_dir(&root).unwrap();
        let failed = session.poll_at(now);
        assert_eq!(failed.health, Some(WatchHealth::Unavailable));
        assert!(failed.dirty);
        let restored = session.poll_at(now + std::time::Duration::from_secs(2));
        assert_eq!(restored.health, Some(WatchHealth::Active));
        assert!(restored.dirty);
    }
    #[test]
    fn missing_root_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Watcher::new(&dir.path().join("missing")).is_err());
    }
    #[test]
    fn bursts_coalesce_and_errors_are_acknowledged() {
        let signals = Signals::default();
        for _ in 0..100_000 {
            signals.record(Ok(notify::Event::new(notify::EventKind::Any)));
        }
        assert!(signals.dirty.swap(false, Ordering::AcqRel));
        assert!(!signals.dirty.swap(false, Ordering::AcqRel));
        signals.record(Err(notify::Error::generic("fixture backend failure")));
        assert!(signals.error.swap(false, Ordering::AcqRel));
        assert!(!signals.error.swap(false, Ordering::AcqRel));
        assert!(signals.dirty.swap(false, Ordering::AcqRel));
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn native_write_notifies_within_two_seconds() {
        let dir = tempfile::tempdir().unwrap();
        let watcher = Watcher::new(dir.path()).unwrap();
        std::fs::write(dir.path().join("합성.mp4"), b"fixture").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if watcher.changed() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "native watcher missed fixture write"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(watcher.take_error(), None);
    }
}
