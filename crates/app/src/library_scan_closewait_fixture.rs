//! App-private explicit developer fixture only; no environment or file-based switches.
//! Observer creates 3 manual-reset events and transfers exact HANDLE_LIST duplicates.
use std::{
    ffi::OsString,
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
use windows_sys::Win32::{
    Foundation::{
        GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::Threading::{ResetEvent, SetEvent, WaitForSingleObject},
};
fn rejected() -> io::Error {
    io::Error::other("library scan close-wait fixture rejected")
}
pub(crate) struct LibraryScanBarrier {
    started: File,
    release: File,
    worker_ack: File,
    entered: AtomicBool,
    periodic: bool,
    committed: Option<File>,
}
static CONFIGURED: OnceLock<Arc<LibraryScanBarrier>> = OnceLock::new();
pub(crate) fn configured() -> Option<Arc<LibraryScanBarrier>> {
    CONFIGURED.get().cloned()
}
pub(crate) fn initialize(arguments: Vec<OsString>) -> io::Result<()> {
    let marker = std::ffi::OsStr::new("--fixture-library-scan-closewait");
    let periodic_marker = std::ffi::OsStr::new("--fixture-library-periodic-scan");
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_os_str() == marker || arg.as_os_str() == periodic_marker)
        .map(|(i, _)| i)
        .collect();
    if positions.is_empty() {
        return Ok(());
    }
    if positions.len() != 1 || !arguments.iter().any(|arg| arg == "--fixture") {
        return Err(rejected());
    }
    let periodic = arguments[positions[0]].as_os_str() == periodic_marker;
    let start = positions[0] + 1;
    if arguments.len() != start + if periodic { 4 } else { 3 } {
        return Err(rejected());
    }
    let barrier = if periodic {
        LibraryScanBarrier::from_capabilities(&arguments[start..], true)?
    } else {
        LibraryScanBarrier::from_inherited(&arguments[start..])?
    };
    CONFIGURED.set(Arc::new(barrier)).map_err(|_| rejected())
}
impl LibraryScanBarrier {
    /// Caller requires --fixture and --fixture-library-scan-closewait plus exactly3 primitive caps.
    /// Must be called before GUI/worker startup. Never searches by object name/PID/path.
    pub(crate) fn from_inherited(arguments: &[OsString]) -> io::Result<Self> {
        Self::from_capabilities(arguments, false)
    }
    fn from_capabilities(arguments: &[OsString], periodic: bool) -> io::Result<Self> {
        if arguments.len() != if periodic { 4 } else { 3 } {
            return Err(rejected());
        }
        let mut raw = Vec::<HANDLE>::with_capacity(3);
        let mut files = Vec::<File>::with_capacity(3);
        for value in arguments {
            let text = value.to_str().ok_or_else(rejected)?;
            if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err(rejected());
            }
            let handle = text.parse::<usize>().map_err(|_| rejected())? as HANDLE;
            if handle.is_null() || handle == INVALID_HANDLE_VALUE || raw.contains(&handle) {
                return Err(rejected());
            }
            let mut flags = 0;
            // SAFETY: original process-local inherited capability query.
            if unsafe { GetHandleInformation(handle, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT == 0
            {
                return Err(rejected());
            }
            raw.push(handle);
            // Take RAII ownership immediately: subsequent partial validation errors close
            // every previously accepted inherited duplicate exactly once.
            files.push(unsafe { File::from_raw_handle(handle) });
            if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
                return Err(rejected());
            }
            // ResetEvent verifies EVENT type, not manual-reset semantics. The observer
            // owns creation and records manual-reset=true plus exact original HANDLE_LIST.
            if unsafe { ResetEvent(handle) } == 0 {
                return Err(rejected());
            }
        }
        let worker_ack = files.pop().ok_or_else(rejected)?;
        let committed = if periodic {
            Some(files.pop().ok_or_else(rejected)?)
        } else {
            None
        };
        let release = files.pop().ok_or_else(rejected)?;
        let started = files.pop().ok_or_else(rejected)?;
        Ok(Self {
            started,
            release,
            worker_ack,
            entered: AtomicBool::new(false),
            periodic,
            committed,
        })
    }
    /// Called only after an actual ordinary-file observation INSERT, within the
    /// original reconciliation transaction. Subsequent scans do not rearm it.
    pub(crate) fn after_first_observation(&self) -> io::Result<()> {
        if self.entered.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        if unsafe { SetEvent(self.started.as_raw_handle()) } == 0 {
            return Err(rejected());
        }
        // A stop request cannot shortcut this original observer release capability.
        self.wait_release(20_000)
    }
    pub(crate) fn periodic(&self) -> bool {
        self.periodic
    }
    pub(crate) fn acknowledge_committed(&self) -> io::Result<()> {
        if let Some(event) = &self.committed
            && (!self.entered.load(Ordering::Acquire)
                || unsafe { SetEvent(event.as_raw_handle()) } == 0)
        {
            return Err(rejected());
        }
        Ok(())
    }
    fn wait_release(&self, milliseconds: u32) -> io::Result<()> {
        match unsafe { WaitForSingleObject(self.release.as_raw_handle(), milliseconds) } {
            WAIT_OBJECT_0 => Ok(()),
            WAIT_TIMEOUT => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "library scan fixture release timed out",
            )),
            _ => Err(rejected()),
        }
    }
    /// Invoke only from StopAck::drop after the worker loop returns, BEFORE
    /// signals.stopped.store(true,Release), so the GUI cannot exit before native ACK.
    /// A scan-completed response alone is not the worker-stopped ACK.
    pub(crate) fn acknowledge_worker_stopped(&self) -> io::Result<()> {
        if !self.entered.load(Ordering::Acquire) {
            return Err(rejected());
        }
        if unsafe { SetEvent(self.worker_ack.as_raw_handle()) } == 0 {
            return Err(rejected());
        }
        Ok(())
    }
}

#[cfg(test)]
impl LibraryScanBarrier {
    pub(crate) fn testing() -> io::Result<Arc<Self>> {
        use windows_sys::Win32::System::Threading::CreateEventW;
        fn event() -> io::Result<File> {
            let h = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
            if h.is_null() {
                return Err(rejected());
            }
            Ok(unsafe { File::from_raw_handle(h) })
        }
        Ok(Arc::new(Self {
            started: event()?,
            release: event()?,
            worker_ack: event()?,
            entered: AtomicBool::new(false),
            periodic: false,
            committed: None,
        }))
    }
    pub(crate) fn test_started(&self) -> bool {
        (unsafe { WaitForSingleObject(self.started.as_raw_handle(), 0) }) == WAIT_OBJECT_0
    }
    pub(crate) fn test_ack(&self) -> bool {
        (unsafe { WaitForSingleObject(self.worker_ack.as_raw_handle(), 0) }) == WAIT_OBJECT_0
    }
    pub(crate) fn test_release(&self) -> io::Result<()> {
        if unsafe { SetEvent(self.release.as_raw_handle()) } == 0 {
            Err(rejected())
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::IntoRawHandle;
    use windows_sys::Win32::{
        Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle},
        System::Threading::{CreateEventW, CreateSemaphoreW, GetCurrentProcess},
    };
    fn original_event() -> File {
        let h = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        assert!(!h.is_null());
        unsafe { File::from_raw_handle(h) }
    }
    fn inherited_copy(file: &File) -> File {
        let mut copy = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    file.as_raw_handle(),
                    GetCurrentProcess(),
                    &mut copy,
                    0,
                    1,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        unsafe { File::from_raw_handle(copy) }
    }
    fn argument(file: File) -> (OsString, HANDLE) {
        let raw = file.into_raw_handle();
        (OsString::from((raw as usize).to_string()), raw)
    }
    fn assert_closed(raw: HANDLE) {
        let mut flags = 0;
        assert_eq!(unsafe { GetHandleInformation(raw, &mut flags) }, 0);
    }
    fn assert_original_alive(file: &File) {
        let mut flags = 0;
        assert_ne!(
            unsafe { GetHandleInformation(file.as_raw_handle(), &mut flags) },
            0
        );
        assert_eq!(
            unsafe { WaitForSingleObject(file.as_raw_handle(), 0) },
            WAIT_TIMEOUT
        );
    }
    #[test]
    fn periodic_commit_requires_entry_and_preserves_four_original_events() {
        let originals = [
            original_event(),
            original_event(),
            original_event(),
            original_event(),
        ];
        let args: Vec<_> = originals
            .iter()
            .map(|f| argument(inherited_copy(f)).0)
            .collect();
        let barrier = LibraryScanBarrier::from_capabilities(&args, true).unwrap();
        assert!(barrier.periodic());
        assert!(barrier.acknowledge_committed().is_err());
        assert_original_alive(&originals[2]);
        assert_ne!(unsafe { SetEvent(originals[1].as_raw_handle()) }, 0);
        barrier.after_first_observation().unwrap();
        assert_eq!(
            unsafe { WaitForSingleObject(originals[0].as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
        assert_original_alive(&originals[2]);
        barrier.acknowledge_committed().unwrap();
        assert_eq!(
            unsafe { WaitForSingleObject(originals[2].as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
        assert_original_alive(&originals[3]);
        barrier.acknowledge_worker_stopped().unwrap();
        assert_eq!(
            unsafe { WaitForSingleObject(originals[3].as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
    }
    #[test]
    fn exact_inherited_event_capabilities_clear_inheritance_and_preserve_originals() {
        let originals = [original_event(), original_event(), original_event()];
        let args: Vec<_> = originals
            .iter()
            .map(|f| argument(inherited_copy(f)).0)
            .collect();
        let barrier = LibraryScanBarrier::from_inherited(&args).unwrap();
        for owned in [&barrier.started, &barrier.release, &barrier.worker_ack] {
            let mut flags = 0;
            assert_ne!(
                unsafe { GetHandleInformation(owned.as_raw_handle(), &mut flags) },
                0
            );
            assert_eq!(flags & HANDLE_FLAG_INHERIT, 0);
        }
        let copies = [
            barrier.started.as_raw_handle(),
            barrier.release.as_raw_handle(),
            barrier.worker_ack.as_raw_handle(),
        ];
        drop(barrier);
        for raw in copies {
            assert_closed(raw);
        }
        for original in &originals {
            assert_original_alive(original);
        }
    }
    #[test]
    fn duplicate_and_invalid_text_close_accepted_copy_not_observer_original() {
        for duplicate in [true, false] {
            let original = original_event();
            let (first, raw) = argument(inherited_copy(&original));
            let second = if duplicate {
                first.clone()
            } else {
                OsString::from("invalid")
            };
            assert!(
                LibraryScanBarrier::from_inherited(&[first, second, OsString::from("0")]).is_err()
            );
            assert_closed(raw);
            assert_original_alive(&original);
        }
    }
    #[test]
    fn wrong_native_type_closes_both_accepted_copies_not_original_objects() {
        let original = original_event();
        let semaphore = unsafe { CreateSemaphoreW(std::ptr::null(), 0, 1, std::ptr::null()) };
        assert!(!semaphore.is_null());
        let wrong = unsafe { File::from_raw_handle(semaphore) };
        let (first, one) = argument(inherited_copy(&original));
        let (second, two) = argument(inherited_copy(&wrong));
        assert!(LibraryScanBarrier::from_inherited(&[first, second, OsString::from("0")]).is_err());
        assert_closed(one);
        assert_closed(two);
        assert_original_alive(&original);
        assert_original_alive(&wrong);
    }
    #[test]
    fn worker_ack_requires_actual_first_observation_entry() {
        let barrier = LibraryScanBarrier::testing().unwrap();
        assert!(barrier.acknowledge_worker_stopped().is_err());
        assert!(!barrier.test_ack());
        barrier.test_release().unwrap();
        barrier.after_first_observation().unwrap();
        assert!(barrier.test_started());
        assert!(!barrier.test_ack());
        barrier.acknowledge_worker_stopped().unwrap();
        assert!(barrier.test_ack());
    }

    #[test]
    fn actual_release_event_timeout_does_not_ack_worker() {
        let barrier = LibraryScanBarrier::testing().unwrap();
        assert_eq!(
            barrier.wait_release(20).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(!barrier.test_started());
        assert!(!barrier.test_ack());
        barrier.test_release().unwrap();
        barrier.wait_release(20).unwrap();
        assert!(!barrier.test_ack());
    }
}
