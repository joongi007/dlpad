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
    Storage::FileSystem::*,
    System::Threading::{ResetEvent, SetEvent, WaitForSingleObject},
};
fn rejected() -> io::Error {
    io::Error::other("report close-wait fixture rejected")
}
pub(crate) struct ReportSaveBarrier {
    started: File,
    release: File,
    worker_ack: File,
    entered: AtomicBool,
}
static CONFIGURED: OnceLock<Arc<ReportSaveBarrier>> = OnceLock::new();
pub(crate) fn configured() -> Option<Arc<ReportSaveBarrier>> {
    CONFIGURED.get().cloned()
}
pub(crate) fn initialize(arguments: Vec<OsString>) -> io::Result<()> {
    let marker = std::ffi::OsStr::new("--fixture-report-closewait");
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_os_str() == marker)
        .map(|(i, _)| i)
        .collect();
    if positions.is_empty() {
        return Ok(());
    }
    if positions.len() != 1 || !arguments.iter().any(|arg| arg == "--fixture") {
        return Err(rejected());
    }
    let start = positions[0] + 1;
    if arguments.len() != start + 3 {
        return Err(rejected());
    }
    let barrier = ReportSaveBarrier::from_inherited(&arguments[start..])?;
    CONFIGURED.set(Arc::new(barrier)).map_err(|_| rejected())
}
impl ReportSaveBarrier {
    /// Caller requires --fixture and --fixture-report-closewait plus exactly3 primitive caps.
    /// Must be called before GUI/worker startup. Never searches by object name/PID/path.
    pub(crate) fn from_inherited(arguments: &[OsString]) -> io::Result<Self> {
        if arguments.len() != 3 {
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
        let release = files.pop().ok_or_else(rejected)?;
        let started = files.pop().ok_or_else(rejected)?;
        Ok(Self {
            started,
            release,
            worker_ack,
            entered: AtomicBool::new(false),
        })
    }
    pub(crate) fn after_create(&self, file: &File) -> io::Result<()> {
        if self.entered.swap(true, Ordering::AcqRel) {
            return Err(rejected());
        }
        let h = file.as_raw_handle();
        let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
        let mut standard = FILE_STANDARD_INFO::default();
        // SAFETY: original save_report create_new file and correctly sized stack outputs.
        if unsafe { GetFileType(h) } != FILE_TYPE_DISK
            || unsafe {
                GetFileInformationByHandleEx(
                    h,
                    FileAttributeTagInfo,
                    (&mut attrs as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                    std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
                )
            } == 0
            || unsafe {
                GetFileInformationByHandleEx(
                    h,
                    FileStandardInfo,
                    (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                    std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
                )
            } == 0
            || attrs.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
            || standard.NumberOfLinks != 1
            || standard.DeletePending
            || standard.Directory
            || standard.EndOfFile != 0
        {
            return Err(rejected());
        }
        if unsafe { SetEvent(self.started.as_raw_handle()) } == 0 {
            return Err(rejected());
        }
        // In-flight production file writes are not canceled by request_stop. This fixture
        // models that boundary and never grants stop an early-success escape from the barrier.
        self.wait_release(20_000)
    }
    fn wait_release(&self, milliseconds: u32) -> io::Result<()> {
        match unsafe { WaitForSingleObject(self.release.as_raw_handle(), milliseconds) } {
            WAIT_OBJECT_0 => Ok(()),
            WAIT_TIMEOUT => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "report fixture release timed out",
            )),
            _ => Err(rejected()),
        }
    }
    /// Invoke only from StopAck::drop after the worker loop returns, BEFORE
    /// signals.stopped.store(true,Release), so the GUI cannot exit before native ACK.
    /// A save-completed response alone is not the worker-stopped ACK.
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
impl ReportSaveBarrier {
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
    #[test]
    fn actual_release_event_timeout_does_not_ack_worker() {
        let barrier = ReportSaveBarrier::testing().unwrap();
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
