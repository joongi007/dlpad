//! Caller-thread CPU clock. A finite-resolution measurement, not a hard quota.
use std::{io, marker::PhantomData, rc::Rc};

/// This capability is deliberately !Send and !Sync; never transfer a pseudo HANDLE.
/// ```compile_fail
/// use dlpad_engine::thread_cpu::ThreadCpuClock;
/// fn assert_send<T: Send>() {}
/// assert_send::<ThreadCpuClock>();
/// ```
/// ```compile_fail
/// use dlpad_engine::thread_cpu::ThreadCpuClock;
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<ThreadCpuClock>();
/// ```
pub struct ThreadCpuClock {
    #[cfg(windows)]
    thread_id: u32,
    #[cfg(windows)]
    creation: u64,
    #[cfg(windows)]
    last_ticks: u64,
    _caller_thread: PhantomData<Rc<()>>,
}

impl ThreadCpuClock {
    /// Windows only. Unsupported platforms have no native CPU clock.
    pub fn new() -> io::Result<Option<Self>> {
        #[cfg(windows)]
        {
            let (thread_id, creation, ticks) = native_sample()?;
            Ok(Some(Self {
                thread_id,
                creation,
                last_ticks: ticks,
                _caller_thread: PhantomData,
            }))
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }

    /// Absolute cumulative kernel+user CPU usage, in 100ns ticks.
    /// Callers subtract their own initial sample; creation time is not CPU credit.
    pub fn sample(&mut self) -> io::Result<u64> {
        #[cfg(windows)]
        {
            let (thread_id, creation, ticks) = native_sample()?;
            verify_sample(
                self.thread_id,
                self.creation,
                self.last_ticks,
                thread_id,
                creation,
                ticks,
            )?;
            self.last_ticks = ticks;
            Ok(ticks)
        }
        #[cfg(not(windows))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "caller thread CPU clock unavailable",
            ))
        }
    }
}

#[cfg(any(windows, test))]
fn rejected() -> io::Error {
    io::Error::other("caller thread CPU clock rejected")
}
#[cfg(any(windows, test))]
fn verify_sample(
    original_id: u32,
    original_creation: u64,
    last: u64,
    id: u32,
    creation: u64,
    ticks: u64,
) -> io::Result<()> {
    if id == 0 || original_id != id || original_creation != creation || ticks < last {
        return Err(rejected());
    }
    Ok(())
}
#[cfg(any(windows, test))]
fn checked_ticks(kernel: u64, user: u64) -> io::Result<u64> {
    kernel.checked_add(user).ok_or_else(rejected)
}
#[cfg(windows)]
fn native_sample() -> io::Result<(u32, u64, u64)> {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentThread, GetCurrentThreadId, GetThreadTimes},
    };
    fn ticks(value: FILETIME) -> u64 {
        (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
    }
    // SAFETY: only the live caller's pseudo HANDLE is used; four initialized FILETIMEs
    // are valid writable outputs. No CloseHandle or retained pseudo HANDLE is needed.
    let id = unsafe { GetCurrentThreadId() };
    let mut creation = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exit = creation;
    let mut kernel = creation;
    let mut user = creation;
    let ok = unsafe {
        GetThreadTimes(
            GetCurrentThread(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 || id == 0 || unsafe { GetCurrentThreadId() } != id {
        // Fixed error intentionally exposes neither paths nor OS-specific identity.
        return Err(rejected());
    }
    Ok((
        id,
        ticks(creation),
        checked_ticks(ticks(kernel), ticks(user))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_identity_and_monotonic_equal_quantized_ticks_are_required() {
        assert!(verify_sample(7, 11, 20, 7, 11, 20).is_ok());
        assert!(verify_sample(7, 11, 20, 7, 11, 21).is_ok());
        for (id, creation, ticks) in [(0, 11, 20), (8, 11, 20), (7, 12, 20), (7, 11, 19)] {
            assert!(verify_sample(7, 11, 20, id, creation, ticks).is_err());
        }
    }
    #[test]
    fn cumulative_cpu_overflow_is_rejected() {
        assert_eq!(checked_ticks(3, 4).unwrap(), 7);
        assert_eq!(checked_ticks(u64::MAX, 0).unwrap(), u64::MAX);
        assert!(checked_ticks(u64::MAX, 1).is_err());
    }
    #[cfg(not(windows))]
    #[test]
    fn unsupported_platform_does_not_claim_measurement() {
        assert!(ThreadCpuClock::new().unwrap().is_none());
    }
    #[cfg(windows)]
    #[test]
    fn actual_current_thread_counter_is_stable_and_monotonic() {
        let mut clock = ThreadCpuClock::new().unwrap().unwrap();
        let initial = clock.sample().unwrap();
        let original_id = clock.thread_id;
        let original_creation = clock.creation;
        let mut value = 0_u64;
        for n in 0..100_000 {
            value = std::hint::black_box(value.wrapping_add(n));
        }
        std::hint::black_box(value);
        assert!(clock.sample().unwrap() >= initial);
        assert_eq!(clock.thread_id, original_id);
        assert_eq!(clock.creation, original_creation);
        // CPU resolution may produce equal ticks; no forced fake positive delta.
    }
}
