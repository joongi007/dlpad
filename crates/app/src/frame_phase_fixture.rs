//! Default-OFF, numeric-only owner-thread frame diagnosis. Not a performance quota.
use dlpad_engine::thread_cpu::ThreadCpuClock;
use std::{cell::RefCell, ffi::c_void, io, rc::Rc, sync::Arc, time::Instant};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, GetCurrentThread, GetCurrentThreadId,
};
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};
type Native = *mut c_void;
#[link(name = "user32")]
unsafe extern "system" {
    fn GetThreadDesktop(thread: u32) -> Native;
    fn OpenInputDesktop(flags: u32, inherit: i32, access: u32) -> Native;
    fn CloseDesktop(desktop: Native) -> i32;
    fn GetUserObjectInformationW(
        object: Native,
        index: i32,
        out: *mut c_void,
        len: u32,
        needed: *mut u32,
    ) -> i32;
    fn SetPropW(window: Native, name: *const u16, value: Native) -> i32;
    fn RemovePropW(window: Native, name: *const u16) -> Native;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryThreadCycleTime(thread: Native, cycles: *mut u64) -> i32;
}
const NAMES: [&str; 36] = [
    "DlpadDiagFrameGen",
    "DlpadDiagFrames",
    "DlpadDiagCacheHits",
    "DlpadDiagHiddenFrames",
    "DlpadDiagFrameWallUs",
    "DlpadDiagFrameCpu100ns",
    "DlpadDiagUiCpu100ns",
    "DlpadDiagTessCpu100ns",
    "DlpadDiagPaintCpu100ns",
    "DlpadDiagPresentCpu100ns",
    "DlpadDiagQueryFailed",
    "DlpadDiagOverflow",
    "DlpadDiagUiWallUs",
    "DlpadDiagTessWallUs",
    "DlpadDiagPaintWallUs",
    "DlpadDiagPresentWallUs",
    "DlpadDiagFrameCycles",
    "DlpadDiagUiCycles",
    "DlpadDiagTessCycles",
    "DlpadDiagPaintCycles",
    "DlpadDiagPresentCycles",
    "DlpadDiagRootScheduled",
    "DlpadDiagCallbackScheduled",
    "DlpadDiagActionScheduled",
    "DlpadDiagRootDue",
    "DlpadDiagCallbackDue",
    "DlpadDiagActionDue",
    "DlpadDiagRepaintEvents",
    "DlpadDiagWindowRepaintEvents",
    "DlpadDiagAccessInitialEvents",
    "DlpadDiagAccessActionEvents",
    "DlpadDiagCloseEvents",
    "DlpadDiagRedrawEvents",
    "DlpadDiagMetricsChecks",
    "DlpadDiagMetricsUnchanged",
    "DlpadDiagMetricsRedraws",
];
// Host scheduling provenance only: not the semantic caller of an egui callback.
#[derive(Clone, Copy)]
pub(crate) enum TimerSource {
    Root,
    Callback,
    Action,
}
impl TimerSource {
    fn index(self) -> usize {
        match self {
            Self::Root => 0,
            Self::Callback => 1,
            Self::Action => 2,
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum WakeEvent {
    Repaint,
    WindowRepaint,
    AccessInitial,
    AccessAction,
    Close,
    Redraw,
}
impl WakeEvent {
    fn index(self) -> usize {
        match self {
            Self::Repaint => 6,
            Self::WindowRepaint => 7,
            Self::AccessInitial => 8,
            Self::AccessAction => 9,
            Self::Close => 10,
            Self::Redraw => 11,
        }
    }
}
#[derive(Default)]
struct WakeCounters {
    values: [u64; 15],
    deadline: Option<(Instant, u8)>,
}
impl WakeCounters {
    fn increment(&mut self, indices: &[usize]) -> bool {
        let mut next = self.values;
        for &i in indices {
            let Some(value) = next[i].checked_add(1).filter(|v| *v <= 1_000_000) else {
                return false;
            };
            next[i] = value;
        }
        self.values = next;
        true
    }
    fn schedule(&mut self, when: Instant, source: TimerSource) -> bool {
        let i = source.index();
        if !self.increment(&[i]) {
            return false;
        }
        let bit = 1u8 << i;
        // Host viewport actions replace next_repaint with now; other sources merge minimum.
        if matches!(source, TimerSource::Action) {
            self.deadline = Some((when, bit));
            return true;
        }
        self.deadline = Some(match self.deadline {
            Some((old, bits)) if old < when => (old, bits),
            Some((old, bits)) if old == when => (old, bits | bit),
            _ => (when, bit),
        });
        true
    }
    fn due(&mut self) -> bool {
        let Some((_, bits)) = self.deadline else {
            return true;
        };
        let mut indices = [0usize; 3];
        let mut n = 0;
        for i in 0..3 {
            if bits & (1 << i) != 0 {
                indices[n] = i + 3;
                n += 1;
            }
        }
        if !self.increment(&indices[..n]) {
            return false;
        }
        self.deadline = None;
        true
    }
}

const CYCLES: u64 = 10_000_000_000_000;
const COUNT: u64 = 10000;
const CPU: u64 = 3_000_000_000;
const WALL: u64 = 300_000_000;
#[derive(Default, Clone)]
struct Counters {
    frames: u64,
    hits: u64,
    hidden: u64,
    wall: u64,
    cpu: u64,
    phases: [u64; 4],
    phase_walls: [u64; 4],
    cycles: u64,
    phase_cycles: [u64; 4],
    failed: bool,
    overflow: bool,
}
impl Counters {
    fn add(&mut self, wall: u64, cpu: u64, phases: [u64; 4], hit: bool) -> bool {
        let next = (|| {
            Some((
                self.frames.checked_add(1)?,
                self.hits.checked_add(u64::from(hit))?,
                self.wall.checked_add(wall)?,
                self.cpu.checked_add(cpu)?,
                [
                    self.phases[0].checked_add(phases[0])?,
                    self.phases[1].checked_add(phases[1])?,
                    self.phases[2].checked_add(phases[2])?,
                    self.phases[3].checked_add(phases[3])?,
                ],
            ))
        })();
        if let Some((frames, hits, wall, cpu, phases)) = next
            && frames <= COUNT
            && hits <= frames
            && wall <= WALL
            && cpu <= CPU
            && phases.iter().all(|v| *v <= CPU)
            && phases
                .iter()
                .try_fold(0u64, |a, v| a.checked_add(*v))
                .is_some_and(|sum| sum <= cpu)
        {
            self.frames = frames;
            self.hits = hits;
            self.wall = wall;
            self.cpu = cpu;
            self.phases = phases;
            true
        } else {
            self.overflow = true;
            false
        }
    }
    // Fixed diagnostic sample: four independent bounded phase vectors, no variable user input.
    #[allow(clippy::too_many_arguments)]
    fn add_complete(
        &mut self,
        wall: u64,
        cpu: u64,
        phases: [u64; 4],
        hit: bool,
        phase_walls: [u64; 4],
        cycles: u64,
        phase_cycles: [u64; 4],
    ) -> bool {
        let mut next = self.clone();
        let valid = (|| -> Option<()> {
            if !next.add(wall, cpu, phases, hit) {
                return None;
            }
            next.cycles = next.cycles.checked_add(cycles)?;
            for i in 0..4 {
                next.phase_walls[i] = next.phase_walls[i].checked_add(phase_walls[i])?;
                next.phase_cycles[i] = next.phase_cycles[i].checked_add(phase_cycles[i])?;
            }
            if next.cycles > CYCLES
                || next.phase_walls.iter().any(|v| *v > WALL)
                || next.phase_cycles.iter().any(|v| *v > CYCLES)
                || next
                    .phase_walls
                    .iter()
                    .try_fold(0u64, |a, v| a.checked_add(*v))?
                    > next.wall
                || next
                    .phase_cycles
                    .iter()
                    .try_fold(0u64, |a, v| a.checked_add(*v))?
                    > next.cycles
            {
                return None;
            }
            Some(())
        })()
        .is_some();
        if valid {
            *self = next;
            true
        } else {
            self.overflow = true;
            false
        }
    }
    fn abort(&mut self) {
        if self.hidden < COUNT {
            self.hidden += 1
        } else {
            self.overflow = true
        }
    }
}
struct State {
    _window: Arc<Window>,
    hwnd: Native,
    thread: u32,
    generation: u64,
    names: Vec<Vec<u16>>,
    clock: ThreadCpuClock,
    last_cycle_sample: Option<u64>,
    counters: Counters,
    wake: WakeCounters,
    worker_phase: Option<crate::worker_phase_fixture::owner::Owner>,
}
impl State {
    fn sample(&mut self) -> io::Result<(u64, u64)> {
        let result = (|| {
            let cpu = self.clock.sample()?;
            let mut cycles = 0;
            // SAFETY: same caller thread pseudo HANDLE; bounded initialized u64 output. No HANDLE transfer.
            if unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut cycles) } == 0 {
                return Err(io::Error::last_os_error());
            }
            verify_cycle_sample(self.last_cycle_sample, cycles)?;
            self.last_cycle_sample = Some(cycles);
            Ok((cpu, cycles))
        })();
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                self.counters.failed = true;
                self.publish()?;
                Err(error)
            }
        }
    }
    fn publish(&mut self) -> io::Result<()> {
        owner(self.hwnd, self.thread)?;
        let even = self.generation.checked_add(2).ok_or_else(rejected)?;
        let worker = self
            .worker_phase
            .as_ref()
            .ok_or_else(rejected)?
            .snapshot(self.counters.frames);
        self.counters.overflow |= worker.overflow;
        let values = [
            even - 1,
            self.counters.frames + 1,
            self.counters.hits + 1,
            self.counters.hidden + 1,
            self.counters.wall + 1,
            self.counters.cpu + 1,
            self.counters.phases[0] + 1,
            self.counters.phases[1] + 1,
            self.counters.phases[2] + 1,
            self.counters.phases[3] + 1,
            u64::from(self.counters.failed) + 1,
            u64::from(self.counters.overflow) + 1,
            self.counters.phase_walls[0] + 1,
            self.counters.phase_walls[1] + 1,
            self.counters.phase_walls[2] + 1,
            self.counters.phase_walls[3] + 1,
            self.counters.cycles + 1,
            self.counters.phase_cycles[0] + 1,
            self.counters.phase_cycles[1] + 1,
            self.counters.phase_cycles[2] + 1,
            self.counters.phase_cycles[3] + 1,
            self.wake.values[0] + 1,
            self.wake.values[1] + 1,
            self.wake.values[2] + 1,
            self.wake.values[3] + 1,
            self.wake.values[4] + 1,
            self.wake.values[5] + 1,
            self.wake.values[6] + 1,
            self.wake.values[7] + 1,
            self.wake.values[8] + 1,
            self.wake.values[9] + 1,
            self.wake.values[10] + 1,
            self.wake.values[11] + 1,
            self.wake.values[12] + 1,
            self.wake.values[13] + 1,
            self.wake.values[14] + 1,
        ];
        for (name, value) in self.names.iter().zip(
            values
                .into_iter()
                .chain(worker.values().map(|value| value + 1)),
        ) {
            let value = usize::try_from(value).map_err(|_| rejected())?;
            // SAFETY: retained owner HWND; terminated fixed property name; numeric value only.
            if unsafe { SetPropW(self.hwnd, name.as_ptr(), value as Native) } == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        // SAFETY: publish stable generation only after all scalar writes succeeded.
        if unsafe {
            SetPropW(
                self.hwnd,
                self.names[0].as_ptr(),
                usize::try_from(even).map_err(|_| rejected())? as Native,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        self.generation = even;
        self.worker_phase
            .as_ref()
            .ok_or_else(rejected)?
            .published(self.counters.frames);
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        drop(self.worker_phase.take()); // disable old worker epochs BEFORE native property removal
        if owner(self.hwnd, self.thread).is_ok() {
            for name in self.names.iter() {
                // SAFETY: only our fixed properties on retained original owner HWND.
                unsafe {
                    RemovePropW(self.hwnd, name.as_ptr());
                }
            }
        }
    }
}
pub(crate) struct Probe(Rc<RefCell<State>>);
fn verify_cycle_sample(previous: Option<u64>, now: u64) -> io::Result<()> {
    if previous.is_some_and(|before| now < before) {
        Err(rejected())
    } else {
        Ok(())
    }
}
fn difference(now: u64, before: u64) -> io::Result<u64> {
    now.checked_sub(before).ok_or_else(rejected)
}
fn rejected() -> io::Error {
    io::Error::other("frame phase fixture rejected")
}
fn owner(hwnd: Native, thread: u32) -> io::Result<()> {
    let mut pid = 0;
    // SAFETY: original HWND observation and valid output pointer.
    let actual = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    // SAFETY: caller-local identity queries have no pointer inputs or ownership transfer.
    let current_thread = unsafe { GetCurrentThreadId() };
    // SAFETY: current process identity only.
    let current_process = unsafe { GetCurrentProcessId() };
    if actual != thread || actual != current_thread || pid != current_process {
        return Err(rejected());
    }
    Ok(())
}
fn desktop_name(handle: Native) -> io::Result<Vec<u16>> {
    let mut name = [0u16; 256];
    let mut needed = 0;
    // SAFETY: bounded UTF16 output, borrowed valid desktop HANDLE.
    if handle.is_null()
        || unsafe {
            GetUserObjectInformationW(handle, 2, name.as_mut_ptr().cast(), 512, &mut needed)
        } == 0
        || needed == 0
        || needed > 512
    {
        return Err(rejected());
    }
    let end = name.iter().position(|x| *x == 0).ok_or_else(rejected)?;
    Ok(name[..end].to_vec())
}
impl Probe {
    pub(crate) fn new(window: Arc<Window>) -> io::Result<Option<Self>> {
        let Some(value) = std::env::var_os("DLPAD_FRAME_PHASE_DIAGNOSTIC") else {
            return Ok(None);
        };
        if value != "owned-private-hwnd"
            || !std::env::args_os().any(|x| x == "--fixture")
            || usize::BITS != 64
        {
            return Err(rejected());
        }
        // SAFETY: caller-local identity query.
        let thread = unsafe { GetCurrentThreadId() };
        // SAFETY: current thread desktop borrowed; input desktop query-only acquired/closed below.
        let current = desktop_name(unsafe { GetThreadDesktop(thread) })?;
        let input = unsafe { OpenInputDesktop(0, 0, 1) };
        if input.is_null() {
            return Err(rejected());
        }
        let input_name = desktop_name(input);
        // SAFETY: close exactly the query-only desktop HANDLE acquired above.
        if unsafe { CloseDesktop(input) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if current == input_name?
            || !String::from_utf16(&current)
                .map_err(|_| rejected())?
                .starts_with("dlpad-")
        {
            return Err(rejected());
        }
        let RawWindowHandle::Win32(raw) = window.window_handle().map_err(|_| rejected())?.as_raw()
        else {
            return Err(rejected());
        };
        let hwnd = raw.hwnd.get() as Native;
        owner(hwnd, thread)?;
        let clock = ThreadCpuClock::new()?.ok_or_else(rejected)?;
        let mut state = State {
            _window: window,
            hwnd,
            thread,
            generation: 0,
            names: NAMES
                .iter()
                .chain(crate::worker_phase_fixture::owner::NAMES.iter())
                .map(|s| s.encode_utf16().chain([0]).collect())
                .collect(),
            clock,
            last_cycle_sample: None,
            counters: Counters::default(),
            wake: WakeCounters::default(),
            worker_phase: Some(
                crate::worker_phase_fixture::owner::Owner::activate().ok_or_else(rejected)?,
            ),
        };
        state.publish()?;
        Ok(Some(Self(Rc::new(RefCell::new(state)))))
    }
    fn wake_update(&self, op: impl FnOnce(&mut WakeCounters) -> bool) -> io::Result<()> {
        let mut state = self.0.borrow_mut();
        owner(state.hwnd, state.thread)?;
        if !op(&mut state.wake) {
            state.counters.overflow = true;
        }
        state.publish()?;
        if state.counters.overflow {
            Err(rejected())
        } else {
            Ok(())
        }
    }
    pub(crate) fn event(&self, event: WakeEvent) -> io::Result<()> {
        self.wake_update(|wake| wake.increment(&[event.index()]))
    }
    pub(crate) fn schedule(&self, when: Instant, source: TimerSource) -> io::Result<()> {
        self.wake_update(|wake| wake.schedule(when, source))
    }
    pub(crate) fn timer_due(&self) -> io::Result<()> {
        self.wake_update(WakeCounters::due)
    }
    pub(crate) fn record_metrics_check(&self, redraw: bool) -> io::Result<()> {
        // Both counters commit together before a single generation publication.
        self.wake_update(|wake| wake.increment(&[12, if redraw { 14 } else { 13 }]))
    }
    pub(crate) fn begin(&self) -> io::Result<Frame> {
        let mut state = self.0.borrow_mut();
        owner(state.hwnd, state.thread)?;
        let (tick, cycle) = state.sample()?;
        let start = Instant::now();
        Ok(Frame {
            state: self.0.clone(),
            start,
            last_wall: start,
            first_cycle: cycle,
            last_cycle: cycle,
            phase_walls: [0; 4],
            phase_cycles: [0; 4],
            first: tick,
            last: tick,
            phases: [0; 4],
            finished: false,
        })
    }
}
pub(crate) struct Frame {
    state: Rc<RefCell<State>>,
    start: Instant,
    last_wall: Instant,
    first_cycle: u64,
    last_cycle: u64,
    phase_walls: [u64; 4],
    phase_cycles: [u64; 4],
    first: u64,
    last: u64,
    phases: [u64; 4],
    finished: bool,
}
impl Frame {
    pub(crate) fn phase(&mut self, index: usize) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        if index >= 4 {
            return Err(rejected());
        }
        let (now, cycles) = state.sample()?;
        let wall = Instant::now();
        let delta = (|| -> io::Result<(u64, u64, u64)> {
            Ok((
                difference(now, self.last)?,
                difference(cycles, self.last_cycle)?,
                u64::try_from(
                    wall.checked_duration_since(self.last_wall)
                        .ok_or_else(rejected)?
                        .as_micros(),
                )
                .map_err(|_| rejected())?,
            ))
        })();
        let (cpu_delta, cycles_delta, wall_delta) = match delta {
            Ok(value) => value,
            Err(error) => {
                state.counters.failed = true;
                state.publish()?;
                return Err(error);
            }
        };
        self.phases[index] = cpu_delta;
        self.phase_cycles[index] = cycles_delta;
        self.phase_walls[index] = wall_delta;
        self.last = now;
        self.last_cycle = cycles;
        self.last_wall = wall;
        Ok(())
    }
    pub(crate) fn finish(mut self, hit: bool) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        let cpu = self.last.checked_sub(self.first).ok_or_else(rejected)?;
        let wall = u64::try_from(self.start.elapsed().as_micros()).map_err(|_| rejected())?;
        let cycles = difference(self.last_cycle, self.first_cycle)?;
        state.counters.add_complete(
            wall,
            cpu,
            self.phases,
            hit,
            self.phase_walls,
            cycles,
            self.phase_cycles,
        );
        self.finished = true;
        state.publish()
    }
}
impl Drop for Frame {
    fn drop(&mut self) {
        if !self.finished {
            let mut state = self.state.borrow_mut();
            state.counters.abort();
            if state.publish().is_err() {
                state.counters.failed = true;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_total_does_not_publish_partial_success() {
        let mut c = Counters::default();
        assert!(c.add(1, 2, [1, 1, 0, 0], true));
        assert!(!c.add(WALL, 1, [0; 4], false));
        assert_eq!((c.frames, c.hits, c.wall, c.cpu), (1, 1, 1, 2));
        assert!(c.overflow);
    }
    #[test]
    fn hidden_and_failed_remain_separate_from_success() {
        let mut c = Counters::default();
        c.abort();
        c.failed = true;
        assert!(c.add(2, 4, [1; 4], false));
        assert_eq!((c.frames, c.hidden, c.hits), (1, 1, 0));
        assert!(c.failed);
    }
    #[test]
    fn inconsistent_phase_total_is_rejected() {
        let mut c = Counters::default();
        assert!(!c.add(1, 1, [1; 4], false));
        assert_eq!(c.frames, 0);
        assert!(c.overflow);
    }
    #[test]
    fn frame_bound_and_hidden_bound_fail_closed() {
        let mut c = Counters {
            frames: COUNT,
            hidden: COUNT,
            ..Default::default()
        };
        assert!(!c.add(1, 1, [0; 4], false));
        c.abort();
        assert_eq!(c.hidden, COUNT);
        assert!(c.overflow);
    }
    #[test]
    fn arithmetic_overflow_is_sticky() {
        let mut c = Counters {
            cpu: u64::MAX,
            ..Default::default()
        };
        assert!(!c.add(1, 1, [0; 4], false));
        assert!(c.overflow);
        c.abort();
        assert!(c.overflow);
    }
    #[test]
    fn cycles_overflow_preserves_all_previous_counters() {
        let mut c = Counters::default();
        assert!(c.add_complete(4, 4, [1; 4], true, [1; 4], 4, [1; 4]));
        assert!(!c.add_complete(4, 4, [1; 4], false, [1; 4], CYCLES, [0; 4]));
        assert_eq!((c.frames, c.hits, c.wall, c.cpu, c.cycles), (1, 1, 4, 4, 4));
        assert!(c.overflow);
    }
    #[test]
    fn backwards_cycles_is_rejected() {
        assert!(difference(9, 10).is_err());
        assert_eq!(difference(10, 10).unwrap(), 0);
        assert_eq!(difference(13, 10).unwrap(), 3);
    }
    #[test]
    fn phase_wall_and_cycles_sum_must_fit_total() {
        let mut c = Counters::default();
        assert!(!c.add_complete(1, 4, [1; 4], false, [1; 4], 4, [1; 4]));
        assert_eq!(c.frames, 0);
        let mut c = Counters::default();
        assert!(!c.add_complete(4, 4, [1; 4], false, [1; 4], 1, [1; 4]));
        assert_eq!(c.frames, 0);
    }
    #[test]
    fn phase_wall_growth_is_transactional() {
        let mut c = Counters::default();
        assert!(c.add_complete(4, 0, [0; 4], false, [1; 4], 0, [0; 4]));
        assert!(!c.add_complete(WALL, 0, [0; 4], false, [WALL; 4], 0, [0; 4]));
        assert_eq!(c.phase_walls, [1; 4]);
        assert_eq!(c.frames, 1);
    }

    #[test]
    fn cycles_continuity_is_checked_across_frame_boundaries() {
        assert!(verify_cycle_sample(None, 0).is_ok());
        assert!(verify_cycle_sample(Some(100), 100).is_ok());
        assert!(verify_cycle_sample(Some(100), 101).is_ok());
        assert!(verify_cycle_sample(Some(100), 99).is_err());
    }
}

#[cfg(test)]
mod wake_tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn earliest_deadline_replaces_reason_and_ties_preserve_both() {
        let t = Instant::now();
        let mut c = WakeCounters::default();
        assert!(c.schedule(t + Duration::from_secs(3), TimerSource::Root));
        assert!(c.schedule(t + Duration::from_secs(2), TimerSource::Callback));
        assert!(c.schedule(t + Duration::from_secs(2), TimerSource::Root));
        assert!(c.schedule(t + Duration::from_secs(4), TimerSource::Root));
        assert!(c.due());
        assert_eq!(&c.values[..6], &[3, 1, 0, 1, 1, 0]);
        assert!(c.deadline.is_none());
        assert!(c.due());
        assert_eq!(&c.values[3..6], &[1, 1, 0]);
    }
    #[test]
    fn immediate_action_replaces_even_an_expired_prior_deadline() {
        let t = Instant::now();
        let mut c = WakeCounters::default();
        assert!(c.schedule(t, TimerSource::Root));
        assert!(c.schedule(t + Duration::from_millis(1), TimerSource::Action));
        assert_eq!(c.deadline, Some((t + Duration::from_millis(1), 4)));
        assert!(c.due());
        assert_eq!(&c.values[3..6], &[0, 0, 1]);
    }
    #[test]
    fn multi_reason_overflow_preserves_all_counts_and_pending_deadline() {
        let t = Instant::now();
        let mut c = WakeCounters::default();
        assert!(c.schedule(t, TimerSource::Root));
        assert!(c.schedule(t, TimerSource::Callback));
        c.values[4] = 1_000_000;
        let before = c.values;
        assert!(!c.due());
        assert_eq!(c.values, before);
        assert_eq!(c.deadline, Some((t, 3)));
    }
}
