//! Default-OFF source-operation counters. No worker CPU/TID/time attribution.
//! Only FrameTrace's verified owner activates/publishes; workers use atomics only.
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
const COUNT: usize = 10;
const COUNT_LIMIT: u64 = 1_000_000;
const WAIT_US_LIMIT: u64 = 300_000_000;
const VALUE_MASK: u64 = u32::MAX as u64;
const ACTIVE: u64 = 1 << 63;
const EPOCH_MASK: u64 = (1 << 31) - 1;
const RESERVED: u64 = u64::MAX;
const RETRIES: usize = 16;
// Fixed128B process storage; no allocations, file writes, locks or extra timer.
#[repr(align(64))]
struct Registry {
    enabled: AtomicU64,
    next_epoch: AtomicU64,
    counters: [AtomicU64; COUNT],
    overflow: AtomicU64,
    published_frame: AtomicU64,
    last_event_frame: AtomicU64,
}
static REGISTRY: Registry = Registry::new();
fn packed(epoch: u64, value: u64) -> u64 {
    ACTIVE | (epoch << 32) | value
}
fn belongs(word: u64, epoch: u64) -> bool {
    word & ACTIVE != 0 && ((word >> 32) & EPOCH_MASK) == epoch
}
impl Registry {
    const fn new() -> Self {
        Self {
            enabled: AtomicU64::new(0),
            next_epoch: AtomicU64::new(0),
            counters: [const { AtomicU64::new(0) }; COUNT],
            overflow: AtomicU64::new(0),
            published_frame: AtomicU64::new(0),
            last_event_frame: AtomicU64::new(0),
        }
    }
    fn epoch(&self) -> Option<u64> {
        // The sole inactive hotpath operation. No counter reads/RMW while inactive.
        let epoch = self.enabled.load(Ordering::Acquire);
        (epoch != 0 && epoch <= EPOCH_MASK).then_some(epoch)
    }
    fn flag(&self, epoch: u64) {
        let mut old = self.overflow.load(Ordering::Acquire);
        for _ in 0..RETRIES {
            if !belongs(old, epoch) || old & VALUE_MASK == 1 {
                return;
            }
            match self.overflow.compare_exchange(
                old,
                packed(epoch, 1),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(now) => old = now,
            }
        }
    }
    fn add(&self, epoch: u64, index: usize, amount: u64) -> bool {
        let Some(counter) = self.counters.get(index) else {
            self.flag(epoch);
            return false;
        };
        let limit = if index == 5 {
            WAIT_US_LIMIT
        } else {
            COUNT_LIMIT
        };
        let mut old = counter.load(Ordering::Acquire);
        for _ in 0..RETRIES {
            if !belongs(old, epoch) {
                return false;
            }
            let Some(value) = (old & VALUE_MASK)
                .checked_add(amount)
                .filter(|v| *v <= limit)
            else {
                self.flag(epoch);
                return false;
            };
            match counter.compare_exchange(
                old,
                packed(epoch, value),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.stamp(epoch);
                    return true;
                }
                Err(now) => old = now,
            }
        }
        self.flag(epoch);
        false
    }
    fn stamp(&self, epoch: u64) {
        let observed = self.published_frame.load(Ordering::Acquire);
        if !belongs(observed, epoch) {
            return;
        }
        self.maximum(&self.last_event_frame, epoch, observed & VALUE_MASK);
    }
    fn maximum(&self, target: &AtomicU64, epoch: u64, value: u64) {
        let mut old = target.load(Ordering::Acquire);
        for _ in 0..RETRIES {
            if !belongs(old, epoch) || old & VALUE_MASK >= value {
                return;
            }
            match target.compare_exchange(
                old,
                packed(epoch, value),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(now) => old = now,
            }
        }
        self.flag(epoch);
    }
    fn load(&self, counter: &AtomicU64, epoch: u64) -> u64 {
        let word = counter.load(Ordering::Acquire);
        if belongs(word, epoch) {
            word & VALUE_MASK
        } else {
            0
        }
    }
    fn watch_poll(&self) {
        if let Some(epoch) = self.epoch() {
            self.add(epoch, 0, 1);
        }
    }
    fn span(&self, kind: Kind) -> Span<'_> {
        let epoch = self
            .epoch()
            .filter(|epoch| self.add(*epoch, kind.begin(), 1));
        Span {
            registry: self,
            epoch,
            kind,
            finished: false,
        }
    }
}
// GPU Main compiles the worker hotpath but has no verified FrameTrace owner.
// Only these owner/publication APIs are intentionally unused in that target.
#[allow(dead_code)]
pub(crate) mod owner {
    use super::*;
    pub(crate) const NAMES: [&str; 16] = [
        "DlpadDiagLibraryWatchPolls",
        "DlpadDiagLibraryPeriodicBegins",
        "DlpadDiagLibraryPeriodicEnds",
        "DlpadDiagLibraryPeriodicFailures",
        "DlpadDiagLibraryCpuDebtWaits",
        "DlpadDiagLibraryCpuDebtWaitUs",
        "DlpadDiagMetricsObserveBegins",
        "DlpadDiagMetricsObserveEnds",
        "DlpadDiagMetricsObserveFailures",
        "DlpadDiagMetricsDisplayWakes",
        "DlpadDiagWorkerOverflow",
        "DlpadDiagWorkerEpoch",
        "DlpadDiagWorkerSnapshotFrame",
        "DlpadDiagWorkerLastEventFrame",
        "DlpadDiagWorkerFrameLag",
        "DlpadDiagWorkerSnapshotWeak",
    ];
    pub(super) fn activate_local(registry: &Registry) -> Option<LocalOwner<'_>> {
        registry
            .enabled
            .compare_exchange(0, RESERVED, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        let epoch =
            match registry
                .next_epoch
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                    old.checked_add(1).filter(|next| *next <= EPOCH_MASK)
                }) {
                Ok(old) => old + 1,
                Err(_) => {
                    registry.enabled.store(0, Ordering::Release);
                    return None;
                }
            };
        for counter in &registry.counters {
            counter.store(packed(epoch, 0), Ordering::Release);
        }
        registry.overflow.store(packed(epoch, 0), Ordering::Release);
        registry
            .published_frame
            .store(packed(epoch, 0), Ordering::Release);
        registry
            .last_event_frame
            .store(packed(epoch, 0), Ordering::Release);
        registry.enabled.store(epoch, Ordering::Release);
        Some(LocalOwner { registry, epoch })
    }
    pub(super) struct LocalOwner<'a> {
        registry: &'a Registry,
        pub(super) epoch: u64,
    }
    impl LocalOwner<'_> {
        pub(super) fn snapshot(&self, frame: u64) -> Snapshot {
            let values = std::array::from_fn(|index| {
                self.registry
                    .load(&self.registry.counters[index], self.epoch)
            });
            let last = self
                .registry
                .load(&self.registry.last_event_frame, self.epoch);
            Snapshot {
                values,
                overflow: self.registry.load(&self.registry.overflow, self.epoch) != 0,
                epoch: self.epoch,
                frame,
                last_event_frame: last,
            }
        }
        pub(super) fn published(&self, frame: u64) {
            // Called only AFTER successful existing stable generation publication.
            if frame > COUNT_LIMIT {
                self.registry.flag(self.epoch);
                return;
            }
            self.registry
                .maximum(&self.registry.published_frame, self.epoch, frame);
        }
    }
    impl Drop for LocalOwner<'_> {
        fn drop(&mut self) {
            // Block new calls, then invalidate every old packed-CAS token before release.
            // Racing CAS can commit before its slot is invalidated, never after close returns.
            if self
                .registry
                .enabled
                .compare_exchange(self.epoch, RESERVED, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return;
            }
            for counter in &self.registry.counters {
                counter.fetch_and(!ACTIVE, Ordering::AcqRel);
            }
            self.registry.overflow.fetch_and(!ACTIVE, Ordering::AcqRel);
            self.registry
                .published_frame
                .fetch_and(!ACTIVE, Ordering::AcqRel);
            self.registry
                .last_event_frame
                .fetch_and(!ACTIVE, Ordering::AcqRel);
            self.registry.enabled.store(0, Ordering::Release);
        }
    }
    pub(crate) struct Owner(LocalOwner<'static>, PhantomData<Rc<()>>);
    impl Owner {
        /// Only after FrameTrace's existing64bit/owner/private desktop/clock gates.
        pub(crate) fn activate() -> Option<Self> {
            activate_local(&REGISTRY).map(|owner| Self(owner, PhantomData))
        }
        pub(crate) fn snapshot(&self, frame: u64) -> Snapshot {
            self.0.snapshot(frame)
        }
        pub(crate) fn published(&self, frame: u64) {
            self.0.published(frame);
        }
    }
    pub(crate) struct Snapshot {
        pub(super) values: [u64; COUNT],
        pub(crate) overflow: bool,
        pub(super) epoch: u64,
        frame: u64,
        last_event_frame: u64,
    }
    impl Snapshot {
        pub(crate) fn values(&self) -> [u64; 16] {
            let mut result = [0; 16];
            result[..COUNT].copy_from_slice(&self.values);
            result[10] = u64::from(self.overflow);
            result[11] = self.epoch;
            result[12] = self.frame;
            result[13] = self.last_event_frame;
            result[14] = self.frame.saturating_sub(self.last_event_frame);
            result[15] = 1;
            result
        }
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Library,
    Metrics,
}
impl Kind {
    fn begin(self) -> usize {
        match self {
            Self::Library => 1,
            Self::Metrics => 6,
        }
    }
    fn end(self) -> usize {
        match self {
            Self::Library => 2,
            Self::Metrics => 7,
        }
    }
    fn failure(self) -> usize {
        match self {
            Self::Library => 3,
            Self::Metrics => 8,
        }
    }
}
pub(crate) struct Span<'a> {
    registry: &'a Registry,
    epoch: Option<u64>,
    kind: Kind,
    finished: bool,
}
impl Span<'_> {
    pub(crate) fn library_periodic() -> Span<'static> {
        REGISTRY.span(Kind::Library)
    }
    pub(crate) fn metrics_observe() -> Span<'static> {
        REGISTRY.span(Kind::Metrics)
    }
    pub(crate) fn watch_poll(&self) {
        if let Some(epoch) = self.epoch {
            self.registry.add(epoch, 0, 1);
        }
    }
    pub(crate) fn cpu_debt(&self, duration: Duration) {
        if let Some(epoch) = self.epoch {
            self.registry.add(epoch, 4, 1);
            match u64::try_from(duration.as_micros()) {
                Ok(value) => {
                    self.registry.add(epoch, 5, value);
                }
                Err(_) => self.registry.flag(epoch),
            }
        }
    }
    pub(crate) fn display_wake(&self) {
        if let Some(epoch) = self.epoch {
            self.registry.add(epoch, 9, 1);
        }
    }
    pub(crate) fn finish(&mut self, failed: bool) {
        if self.finished {
            return;
        }
        self.finished = true;
        if let Some(epoch) = self.epoch {
            self.registry.add(epoch, self.kind.end(), 1);
            if failed {
                self.registry.add(epoch, self.kind.failure(), 1);
            }
        }
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        self.finish(true);
    }
}
pub(crate) fn library_watch_poll() {
    REGISTRY.watch_poll();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_and_preowner_span_end_are_noops() {
        let registry = Registry::new();
        let mut before = registry.span(Kind::Metrics);
        registry.watch_poll();
        assert_eq!(registry.counters[0].load(Ordering::Relaxed), 0);
        let owner = owner::activate_local(&registry).unwrap();
        before.finish(true);
        assert_eq!(owner.snapshot(0).values, [0; COUNT]);
    }
    #[test]
    fn old_span_discarded_after_close_and_new_epoch() {
        let registry = Registry::new();
        let first = owner::activate_local(&registry).unwrap();
        let mut old = registry.span(Kind::Library);
        let epoch = first.epoch;
        drop(first);
        old.cpu_debt(Duration::from_millis(1));
        old.finish(true);
        let next = owner::activate_local(&registry).unwrap();
        assert_ne!(next.epoch, epoch);
        old.display_wake();
        assert_eq!(next.snapshot(0).values, [0; COUNT]);
        assert!(!next.snapshot(0).overflow);
    }
    #[test]
    fn real_source_operations_and_lag_are_separate() {
        let registry = Registry::new();
        let owner = owner::activate_local(&registry).unwrap();
        owner.published(7);
        registry.watch_poll();
        let mut library = registry.span(Kind::Library);
        library.watch_poll();
        library.cpu_debt(Duration::from_micros(250));
        library.finish(false);
        let mut metrics = registry.span(Kind::Metrics);
        metrics.finish(true);
        metrics.display_wake();
        let snap = owner.snapshot(9);
        assert_eq!(snap.values, [2, 1, 1, 0, 1, 250, 1, 1, 1, 1]);
        assert_eq!(&snap.values()[12..], [9, 7, 2, 1]);
    }
    #[test]
    fn unclosed_span_is_failure_and_overflow_is_sticky_bounded() {
        let registry = Registry::new();
        let owner = owner::activate_local(&registry).unwrap();
        drop(registry.span(Kind::Library));
        registry.counters[0].store(packed(owner.epoch, COUNT_LIMIT), Ordering::Release);
        registry.watch_poll();
        assert_eq!(owner.snapshot(0).values[0], COUNT_LIMIT);
        assert!(owner.snapshot(0).overflow);
        let work = registry.span(Kind::Library);
        work.cpu_debt(Duration::MAX);
        assert!(owner.snapshot(0).overflow);
    }
    #[test]
    fn owner_exclusive_and_epoch_bound_fail_without_panic() {
        let registry = Registry::new();
        let owner = owner::activate_local(&registry).unwrap();
        assert!(owner::activate_local(&registry).is_none());
        drop(owner);
        registry.next_epoch.store(EPOCH_MASK, Ordering::Release);
        assert!(owner::activate_local(&registry).is_none());
        assert!(registry.epoch().is_none());
        assert!((128..=256).contains(&std::mem::size_of::<Registry>()));
        assert_eq!(owner::NAMES.len(), 16);
    }
    #[test]
    fn captured_worker_span_cannot_accumulate_after_owner_close_returns() {
        let registry = Registry::new();
        let owner = owner::activate_local(&registry).unwrap();
        let barrier = std::sync::Barrier::new(2);
        let (before,) = std::thread::scope(|scope| {
            scope.spawn(|| {
                let mut span = registry.span(Kind::Library);
                barrier.wait();
                barrier.wait();
                for _ in 0..2000 {
                    span.watch_poll();
                    span.cpu_debt(Duration::from_micros(1));
                    registry.watch_poll();
                }
                span.finish(true);
            });
            barrier.wait();
            drop(owner);
            let before: [u64; COUNT] =
                std::array::from_fn(|index| registry.counters[index].load(Ordering::Acquire));
            barrier.wait();
            // Join all scope children before testing final words below.
            (before,)
        });
        assert!(registry.epoch().is_none());
        assert_eq!(
            before,
            std::array::from_fn(|index| registry.counters[index].load(Ordering::Acquire))
        );
        assert_eq!(registry.counters[0].load(Ordering::Acquire) & VALUE_MASK, 0);
        assert_eq!(registry.counters[1].load(Ordering::Acquire) & VALUE_MASK, 1);
        assert_eq!(registry.counters[2].load(Ordering::Acquire) & VALUE_MASK, 0);
        assert_eq!(registry.counters[4].load(Ordering::Acquire) & VALUE_MASK, 0);
        assert_eq!(registry.counters[5].load(Ordering::Acquire) & VALUE_MASK, 0);
    }
    #[test]
    fn local_concurrent_writers_are_bounded_and_stop_before_new_owner() {
        let registry = Registry::new();
        let owner = owner::activate_local(&registry).unwrap();
        std::thread::scope(|scope| {
            for _ in 0..2 {
                scope.spawn(|| {
                    for _ in 0..2000 {
                        registry.watch_poll();
                    }
                });
            }
        });
        let snap = owner.snapshot(0);
        assert!(snap.values[0] <= 4000);
        assert!(snap.values[0] == 4000 || snap.overflow);
        drop(owner);
        let before = registry.counters[0].load(Ordering::Acquire);
        registry.watch_poll();
        assert_eq!(registry.counters[0].load(Ordering::Acquire), before);
    }
}
