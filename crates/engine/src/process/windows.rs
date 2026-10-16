//! Start suspended, bind ownership, then resume: no unowned child can execute.
use super::{OwnershipCleanupError, ProcessSpec};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::ExitStatusExt,
    },
    process::ExitStatus,
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        DUPLICATE_SAME_ACCESS, DuplicateHandle, ERROR_BROKEN_PIPE, FILETIME, HANDLE,
        HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, SetHandleInformation, WAIT_OBJECT_0,
        WAIT_TIMEOUT,
    },
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    },
    System::{
        JobObjects::*,
        Pipes::{CreatePipe, PeekNamedPipe},
        Threading::*,
    },
};

const TERMINAL_GRACE: Duration = Duration::from_secs(2);
const TERMINAL_POLL: Duration = Duration::from_millis(20);

fn unconfirmed() -> io::Error {
    io::Error::other(OwnershipCleanupError::TerminalUnconfirmed)
}

/// Exclusive synchronous pipe reader; readiness prevents waiting for unavailable bytes.
pub(super) struct InterruptiblePipe<T: Read + AsRawHandle> {
    pipe: T,
    stop: Arc<AtomicBool>,
    aborted: Arc<AtomicBool>,
}
impl<T: Read + AsRawHandle> InterruptiblePipe<T> {
    pub fn new(pipe: T, stop: Arc<AtomicBool>, aborted: Arc<AtomicBool>) -> io::Result<Self> {
        let this = Self {
            pipe,
            stop,
            aborted,
        };
        this.available()?;
        Ok(this)
    }
    fn available(&self) -> io::Result<Option<u32>> {
        let mut available = 0;
        // SAFETY: exclusively owned pipe handle; no buffer is passed and only byte count is written.
        if unsafe {
            PeekNamedPipe(
                self.pipe.as_raw_handle(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut available,
                ptr::null_mut(),
            )
        } == 0
        {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                return Ok(None);
            }
            return Err(error);
        }
        Ok(Some(available))
    }
}
impl<T: Read + AsRawHandle> Read for InterruptiblePipe<T> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        loop {
            if self.stop.load(Ordering::Acquire) {
                self.aborted.store(true, Ordering::Release);
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "process output drain interrupted",
                ));
            }
            match self.available()? {
                None => return Ok(0),
                Some(0) => std::thread::sleep(TERMINAL_POLL),
                Some(available) => {
                    let count = buffer.len().min(available as usize);
                    return self.pipe.read(&mut buffer[..count]);
                }
            }
        }
    }
}

pub(super) struct Child {
    process: OwnedHandle,
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}
impl Child {
    #[allow(dead_code)]
    pub fn creation_time(&self) -> io::Result<u64> {
        let mut times = [FILETIME::default(); 4];
        // SAFETY: original retained process handle and live FILETIME outputs.
        if unsafe {
            GetProcessTimes(
                self.process.as_raw_handle(),
                &mut times[0],
                &mut times[1],
                &mut times[2],
                &mut times[3],
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let value = (u64::from(times[0].dwHighDateTime) << 32) | u64::from(times[0].dwLowDateTime);
        if value == 0 {
            return Err(io::Error::other("missing process creation identity"));
        }
        Ok(value)
    }
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        // SAFETY: process handle stays owned throughout the wait.
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => self.status().map(Some),
            _ => Err(io::Error::last_os_error()),
        }
    }
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        // SAFETY: process handle is owned, not a process ID looked up later.
        if unsafe {
            WaitForSingleObject(
                self.process.as_raw_handle(),
                TERMINAL_GRACE.as_millis() as u32,
            )
        } != WAIT_OBJECT_0
        {
            return Err(unconfirmed());
        }
        self.status().map_err(|_| unconfirmed())
    }
    fn status(&self) -> io::Result<ExitStatus> {
        let mut code = 0;
        // SAFETY: valid owned process handle and writable output.
        if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ExitStatus::from_raw(code))
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        // Covers errors before or after Job assignment. Never signal by PID.
        // SAFETY: this exclusively owned process handle cannot be reused while live.
        unsafe {
            if WaitForSingleObject(self.process.as_raw_handle(), 0) == WAIT_TIMEOUT
                && TerminateProcess(self.process.as_raw_handle(), 1) != 0
            {
                // Drop cannot return confirmation; closing the Job remains kill-on-close.
                WaitForSingleObject(
                    self.process.as_raw_handle(),
                    TERMINAL_GRACE.as_millis() as u32,
                );
            }
        }
    }
}

pub(super) struct Ownership(OwnedHandle);
impl Ownership {
    fn new() -> io::Result<Self> {
        // SAFETY: unnamed, non-inheritable Job; all returned handles are checked.
        unsafe {
            let job = owned(CreateJobObjectW(ptr::null(), ptr::null()))?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(Self(job))
        }
    }
    pub fn confirm_terminal(&self) -> io::Result<()> {
        self.confirm_terminal_with_grace(TERMINAL_GRACE)
    }
    fn confirm_terminal_with_grace(&self, grace: Duration) -> io::Result<()> {
        let deadline = Instant::now() + grace;
        loop {
            let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
            // SAFETY: owned non-inherited Job handle and correctly sized writable accounting.
            if unsafe {
                QueryInformationJobObject(
                    self.0.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    &mut info as *mut _ as _,
                    std::mem::size_of_val(&info) as u32,
                    ptr::null_mut(),
                )
            } == 0
            {
                return Err(unconfirmed());
            }
            if info.ActiveProcesses == 0 {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(unconfirmed());
            }
            std::thread::sleep(remaining.min(TERMINAL_POLL));
        }
    }
    pub fn signal(&self, _: bool) -> io::Result<()> {
        // SAFETY: Job handle remains owned and was never inherited by a child.
        if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

fn owned(handle: HANDLE) -> io::Result<OwnedHandle> {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: freshly returned valid handle is transferred exactly once.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}
fn pipe(attributes: &SECURITY_ATTRIBUTES) -> io::Result<(File, OwnedHandle)> {
    let mut read = ptr::null_mut();
    let mut write = ptr::null_mut();
    // SAFETY: valid writable outputs; attributes live through the call.
    if unsafe { CreatePipe(&mut read, &mut write, attributes, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let reader = owned(read)?;
    let writer = owned(write)?;
    // SAFETY: only writer may be inherited, never reader.
    if unsafe { SetHandleInformation(reader.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((File::from(reader), writer))
}
struct Attributes {
    storage: Vec<usize>,
    initialized: bool,
}
impl Attributes {
    fn new(handles: &[HANDLE], jobs: &[HANDLE]) -> io::Result<Self> {
        let mut size = 0;
        let count = if jobs.is_empty() { 1 } else { 2 };
        // First call obtains required size. Storage uses pointer alignment.
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), count, 0, &mut size);
        }
        if size == 0 || size > 64 * 1024 {
            return Err(io::Error::other("invalid process attribute size"));
        }
        let mut this = Self {
            storage: vec![0; size.div_ceil(std::mem::size_of::<usize>())],
            initialized: false,
        };
        // SAFETY: sufficient pointer-aligned storage; outputs remain live until Drop.
        if unsafe { InitializeProcThreadAttributeList(this.pointer(), count, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        this.initialized = true;
        // SAFETY: handle slice remains live until CreateProcess; exactly stdio handles.
        if unsafe {
            UpdateProcThreadAttribute(
                this.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                std::mem::size_of_val(handles),
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // Atomic Job assignment eliminates even the suspended-child crash window.
        // SAFETY: job slice remains live until CreateProcess, all jobs are owned.
        if !jobs.is_empty()
            && unsafe {
                UpdateProcThreadAttribute(
                    this.pointer(),
                    0,
                    PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                    jobs.as_ptr().cast(),
                    std::mem::size_of_val(jobs),
                    ptr::null_mut(),
                    ptr::null(),
                )
            } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(this)
    }
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: only initialized lists reach this destructor after creation.
        if self.initialized {
            unsafe {
                DeleteProcThreadAttributeList(self.pointer());
            }
        }
    }
}
fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut encoded = Vec::new();
    for unit in value.encode_wide() {
        if encoded.len() >= 32766 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Windows process string exceeds limit",
            ));
        }
        encoded.push(unit);
    }
    let mut value = encoded;
    if value.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "NUL in process argument",
        ));
    }
    value.push(0);
    Ok(value)
}
fn command_line(executable: &OsStr, args: &[OsString]) -> io::Result<Vec<u16>> {
    let mut result = Vec::new();
    for arg in std::iter::once(executable).chain(args.iter().map(OsString::as_os_str)) {
        if !result.is_empty() {
            result.push(b' ' as u16);
        }
        let value = wide(arg)?;
        result.push(b'"' as u16);
        let mut slashes = 0;
        for &unit in &value[..value.len() - 1] {
            if unit == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            if unit == b'"' as u16 {
                result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2 + 1));
            } else {
                result.extend(std::iter::repeat_n(b'\\' as u16, slashes));
            }
            result.push(unit);
            slashes = 0;
        }
        result.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        result.push(b'"' as u16);
        if result.len() >= 32767 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Windows command line exceeds limit",
            ));
        }
    }
    result.push(0);
    Ok(result)
}

fn suspended(
    spec: &ProcessSpec,
    ownership: Ownership,
    jobs: &[HANDLE],
) -> io::Result<(Child, OwnedHandle, Ownership)> {
    suspended_with_stdio(spec, ownership, jobs, None)
}

fn suspended_with_stdio(
    spec: &ProcessSpec,
    ownership: Ownership,
    jobs: &[HANDLE],
    output: Option<(&File, &File)>,
) -> io::Result<(Child, OwnedHandle, Ownership)> {
    let environment = spec
        .runtime_environment
        .as_ref()
        .map(|policy| policy.encode(std::env::vars_os()))
        .transpose()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "managed runtime environment rejected",
            )
        })?;
    let application = wide(spec.executable.as_os_str())?;
    let mut command = command_line(spec.executable.as_os_str(), &spec.args)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    fn inherited_writer(file: &File) -> io::Result<OwnedHandle> {
        let mut duplicate = ptr::null_mut();
        // SAFETY: duplicate an original borrowed handle in the current process;
        // only the fresh duplicate is inheritable, with no mutation of the source.
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                file.as_raw_handle(),
                GetCurrentProcess(),
                &mut duplicate,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        owned(duplicate)
    }
    let (stdout, out_writer, stderr, err_writer) = if let Some((out, err)) = output {
        (None, inherited_writer(out)?, None, inherited_writer(err)?)
    } else {
        let (out, out_writer) = pipe(&attributes)?;
        let (err, err_writer) = pipe(&attributes)?;
        (Some(out), out_writer, Some(err), err_writer)
    };
    let null = [b'N' as u16, b'U' as u16, b'L' as u16, 0];
    // SAFETY: NUL is a system device, not existing user data.
    let stdin = owned(unsafe {
        CreateFileW(
            null.as_ptr(),
            FILE_GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &attributes,
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    })?;
    let handles = [
        stdin.as_raw_handle(),
        out_writer.as_raw_handle(),
        err_writer.as_raw_handle(),
    ];
    let mut handle_list = Attributes::new(&handles, jobs)?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = handles[0];
    startup.StartupInfo.hStdOutput = handles[1];
    startup.StartupInfo.hStdError = handles[2];
    startup.lpAttributeList = handle_list.pointer();
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: strings, attribute list, stdio handles and outputs remain live.
    // No shell; primary thread cannot execute until the Job owns this process.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            CREATE_SUSPENDED
                | CREATE_NO_WINDOW
                | EXTENDED_STARTUPINFO_PRESENT
                | if environment.is_some() {
                    CREATE_UNICODE_ENVIRONMENT
                } else {
                    0
                },
            environment
                .as_ref()
                .map_or(ptr::null(), |block| block.as_ptr().cast()),
            ptr::null(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let child = Child {
        process: owned(info.hProcess)?,
        stdout,
        stderr,
    };
    let primary = owned(info.hThread)?;
    // SAFETY: CreateProcess atomically assigned this non-inherited Job. Verify
    // membership before executing any child code; unsupported systems fail closed.
    let mut member = 0;
    if unsafe {
        IsProcessInJob(
            child.process.as_raw_handle(),
            ownership.0.as_raw_handle(),
            &mut member,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if member == 0 {
        return Err(io::Error::other("suspended child is not in owned Job"));
    }
    Ok((child, primary, ownership))
}

/// Starts the independent helper outside the tool Job; never used to start tools.
/// Caller retains exactly five original duplicated capabilities through CreateProcess.
pub(super) fn spawn_independent_helper(
    helper: &std::path::Path,
    args: &[OsString],
    handles: &[HANDLE],
) -> io::Result<Child> {
    let base_count = if args.first().is_some_and(|marker| {
        marker == OsStr::new(super::windows_guardian_entry::MARKER)
            || marker == OsStr::new(super::windows_managed_guardian_entry::MARKER)
    }) {
        5
    } else if args
        .first()
        .is_some_and(|marker| marker == OsStr::new(super::windows_paired_guardian_entry::MARKER))
    {
        6
    } else {
        return Err(io::Error::other("invalid helper launch contract"));
    };
    if args.len() != handles.len() + 1 || !(base_count..=base_count + 1).contains(&handles.len()) {
        return Err(io::Error::other("invalid helper launch contract"));
    }
    let application = wide(helper.as_os_str())?;
    let mut command = command_line(helper.as_os_str(), args)?;
    let mut list = Attributes::new(handles, &[])?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = handles[0];
    startup.StartupInfo.hStdOutput = handles[base_count - 2];
    startup.StartupInfo.hStdError = handles[base_count - 1];
    startup.lpAttributeList = list.pointer();
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: original capabilities, UTF-16 buffers, attributes and outputs remain live.
    // Tool code remains gated by the helper; no breakaway or unowned tool fallback.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
            ptr::null(),
            ptr::null(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let process = owned(info.hProcess)?;
    let _primary = owned(info.hThread)?;
    Ok(Child {
        process,
        stdout: None,
        stderr: None,
    })
}

pub(super) enum SpawnOutcome {
    Ready(Child, Ownership),
    Interrupted(super::ProcessOutcome),
}

fn resume_controlled(
    mut child: Child,
    primary: OwnedHandle,
    ownership: Ownership,
    execution: &super::ExecutionControl<'_>,
) -> io::Result<SpawnOutcome> {
    if let Some(outcome) = execution.outcome() {
        ownership.signal(true).map_err(|_| unconfirmed())?;
        child.wait().map_err(|_| unconfirmed())?;
        ownership.confirm_terminal()?;
        return Ok(SpawnOutcome::Interrupted(outcome));
    }
    // SAFETY: this is the primary thread returned by CreateProcess, not a lookup.
    if unsafe { ResumeThread(primary.as_raw_handle()) } == u32::MAX {
        let error = io::Error::last_os_error();
        let _ = ownership.signal(true);
        return Err(error);
    }
    Ok(SpawnOutcome::Ready(child, ownership))
}

/// Retains the original suspended process, primary thread and unnamed Job.
/// Dropping before Begin closes the kill-on-close Job and never runs tool code.
pub(super) struct PreparedChild {
    child: Child,
    primary: OwnedHandle,
    ownership: Ownership,
}
impl PreparedChild {
    pub fn prepare(spec: &ProcessSpec) -> io::Result<Self> {
        let ownership = Ownership::new()?;
        let jobs = [ownership.0.as_raw_handle()];
        let (child, primary, ownership) = suspended(spec, ownership, &jobs)?;
        Ok(Self {
            child,
            primary,
            ownership,
        })
    }
    /// Only fresh duplicates of these writers and NUL stdin enter HANDLE_LIST.
    /// Control, journal, receipt and Job handles remain excluded.
    #[allow(dead_code)]
    pub fn prepare_with_stdio(
        spec: &ProcessSpec,
        stdout: &File,
        stderr: &File,
    ) -> io::Result<Self> {
        let ownership = Ownership::new()?;
        let jobs = [ownership.0.as_raw_handle()];
        let (child, primary, ownership) =
            suspended_with_stdio(spec, ownership, &jobs, Some((stdout, stderr)))?;
        Ok(Self {
            child,
            primary,
            ownership,
        })
    }
    // Creation times bind journal identity, but never independently authorize deletion.
    #[allow(dead_code)]
    pub fn creation_times(&self) -> io::Result<(u64, u64)> {
        fn creation(process: HANDLE) -> io::Result<u64> {
            let mut times = [FILETIME::default(); 4];
            // SAFETY: original live process handle or current-process pseudo handle;
            // all four writable FILETIME outputs remain live throughout the call.
            if unsafe {
                GetProcessTimes(
                    process,
                    &mut times[0],
                    &mut times[1],
                    &mut times[2],
                    &mut times[3],
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            let value =
                (u64::from(times[0].dwHighDateTime) << 32) | u64::from(times[0].dwLowDateTime);
            if value == 0 {
                return Err(io::Error::other("missing process creation identity"));
            }
            Ok(value)
        }
        // SAFETY: current-process pseudo handle requires no ownership or close.
        Ok((
            creation(unsafe { GetCurrentProcess() })?,
            creation(self.child.process.as_raw_handle())?,
        ))
    }
    pub fn resume(self, execution: &super::ExecutionControl<'_>) -> io::Result<SpawnOutcome> {
        resume_controlled(self.child, self.primary, self.ownership, execution)
    }
    #[allow(dead_code)]
    pub fn cancel(self) -> io::Result<()> {
        self.cancel_status().map(|_| ())
    }
    #[allow(dead_code)]
    pub fn cancel_status(mut self) -> io::Result<ExitStatus> {
        self.ownership.signal(true).map_err(|_| unconfirmed())?;
        let status = self.child.wait().map_err(|_| unconfirmed())?;
        self.ownership.confirm_terminal()?;
        Ok(status)
    }
}

pub(super) fn spawn(
    spec: &ProcessSpec,
    execution: &super::ExecutionControl<'_>,
) -> io::Result<SpawnOutcome> {
    if let Some(outcome) = execution.outcome() {
        return Ok(SpawnOutcome::Interrupted(outcome));
    }
    // Writer handles are dropped before returning, so root/descendant exit yields EOF.
    PreparedChild::prepare(spec)?.resume(execution)
}

#[cfg(test)]
impl Child {
    pub(super) fn duplicate_to_process(&self, target: HANDLE) -> io::Result<usize> {
        let mut copy = ptr::null_mut();
        // SAFETY: original owned process handle and explicit narrowed target process capability.
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.process.as_raw_handle(),
                target,
                &mut copy,
                0,
                0,
                windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(copy as usize)
    }
    pub(super) fn terminate_owned_for_test(&mut self) -> io::Result<()> {
        // SAFETY: exclusively retained original process handle; not a PID lookup or Job termination.
        if unsafe { TerminateProcess(self.process.as_raw_handle(), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
#[cfg(test)]
pub(super) fn spawn_test_producer(
    exe: &std::path::Path,
    args: &[OsString],
    observer_cap: HANDLE,
    base: &std::path::Path,
) -> io::Result<Child> {
    let executable = wide(exe.as_os_str())?;
    let mut command = command_line(exe.as_os_str(), args)?;
    let attr = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    let null = [b'N' as u16, b'U' as u16, b'L' as u16, 0];
    let stdout = owned(unsafe {
        CreateFileW(
            null.as_ptr(),
            windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &attr,
            OPEN_EXISTING,
            windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    })?;
    let stderr = owned(unsafe {
        CreateFileW(
            null.as_ptr(),
            windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &attr,
            OPEN_EXISTING,
            windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    })?;
    let handles = [observer_cap, stdout.as_raw_handle(), stderr.as_raw_handle()];
    let mut attributes = Attributes::new(&handles, &[])?;
    // Minimal explicit test environment; no global set_var or credential environment copying.
    let system_root = std::env::var_os("SystemRoot")
        .ok_or_else(|| io::Error::other("test SystemRoot unavailable"))?;
    let mut environment = Vec::new();
    for (name, value) in [
        ("DLPAD_APP_KILL_FIXTURE_BASE", base.as_os_str()),
        ("SystemRoot", system_root.as_os_str()),
    ] {
        environment.extend(name.encode_utf16());
        environment.push(b'=' as u16);
        let units: Vec<u16> = value.encode_wide().collect();
        if units.contains(&0) {
            return Err(io::Error::other("test environment rejected"));
        }
        environment.extend(units);
        environment.push(0);
    }
    environment.push(0);
    if environment.len() > 32767 {
        return Err(io::Error::other("test environment bound"));
    }
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = std::mem::size_of_val(&startup) as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = handles[0];
    startup.StartupInfo.hStdOutput = handles[1];
    startup.StartupInfo.hStdError = handles[2];
    startup.lpAttributeList = attributes.pointer();
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: exact original inherited capabilities, environment and buffers remain live.
    // No producer Job and no breakaway. Parent's external hidden-host Job stays alive.
    if unsafe {
        CreateProcessW(
            executable.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr().cast(),
            ptr::null(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let process = owned(info.hProcess)?;
    let _primary = owned(info.hThread)?;
    Ok(Child {
        process,
        stdout: None,
        stderr: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn prepared_identity_is_native_and_cancel_never_runs_tool() {
        let mut prepared = PreparedChild::prepare(&fixture_spec()).unwrap();
        let first = prepared.creation_times().unwrap();
        assert!(first.0 > 0 && first.1 > 0);
        assert_eq!(first, prepared.creation_times().unwrap());
        assert!(prepared.child.try_wait().unwrap().is_none());
        let stdout = prepared.child.stdout.take().unwrap();
        let mut available = 0;
        // SAFETY: original owned pipe and valid byte-count output.
        assert_ne!(
            unsafe {
                PeekNamedPipe(
                    stdout.as_raw_handle(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut available,
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(available, 0);
        prepared.cancel().unwrap();
        let mut stdout = stdout;
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        assert!(bytes.is_empty());
    }
    #[test]
    fn prepared_direct_stdio_reaches_original_pipe_after_resume() {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let (mut out, out_writer) = pipe(&attributes).unwrap();
        let (mut err, err_writer) = pipe(&attributes).unwrap();
        let out_writer = File::from(out_writer);
        let err_writer = File::from(err_writer);
        let prepared =
            PreparedChild::prepare_with_stdio(&fixture_spec(), &out_writer, &err_writer).unwrap();
        assert!(prepared.child.stdout.is_none() && prepared.child.stderr.is_none());
        let cancel = AtomicBool::new(false);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(3),
        };
        let SpawnOutcome::Ready(mut child, ownership) = prepared.resume(&execution).unwrap() else {
            panic!("unexpected interruption");
        };
        drop(out_writer);
        drop(err_writer);
        // --list is bounded and finishes without requiring any external fixture.
        let mut bytes = Vec::new();
        out.read_to_end(&mut bytes).unwrap();
        assert!(!bytes.is_empty());
        assert!(child.wait().unwrap().success());
        ownership.confirm_terminal().unwrap();
        let mut errors = Vec::new();
        err.read_to_end(&mut errors).unwrap();
        assert!(errors.is_empty());
    }
    fn fixture_spec() -> ProcessSpec {
        ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: std::env::current_exe().unwrap(),
            args: vec!["--list".into()],
            timeout: Duration::from_secs(3),
            max_log_bytes: 1024,
        }
    }
    #[test]
    fn closing_job_before_resume_kills_suspended_child_without_execution() {
        let ownership = Ownership::new().unwrap();
        let jobs = [ownership.0.as_raw_handle()];
        let (mut child, _primary, ownership) =
            suspended(&fixture_spec(), ownership, &jobs).unwrap();
        assert!(child.try_wait().unwrap().is_none());
        drop(ownership);
        let start = std::time::Instant::now();
        while child.try_wait().unwrap().is_none() {
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "Job close did not reap suspended child"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        // Kill-on-close may report exit code 0: termination, not a chosen code, is the contract.
        let _ = child.wait().unwrap();
        let mut output = Vec::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut output)
            .unwrap();
        assert!(
            output.is_empty(),
            "suspended fixture ran before ownership was established"
        );
    }
    fn anonymous_pipe() -> (File, File) {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 0,
        };
        let (reader, writer) = pipe(&attributes).unwrap();
        (reader, File::from(writer))
    }
    #[test]
    fn interruptible_pipe_preserves_bytes_then_eof() {
        let (reader, mut writer) = anonymous_pipe();
        let stop = Arc::new(AtomicBool::new(false));
        let aborted = Arc::new(AtomicBool::new(false));
        let mut reader = InterruptiblePipe::new(reader, stop, aborted.clone()).unwrap();
        writer
            .write_all(b"literal \x00 UTF-8 \xe9\x9f\x93")
            .unwrap();
        drop(writer);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"literal \x00 UTF-8 \xe9\x9f\x93");
        assert!(!aborted.load(Ordering::Acquire));
    }
    #[test]
    fn interruptible_pipe_empty_live_writer_stops() {
        let (reader, _writer) = anonymous_pipe();
        let stop = Arc::new(AtomicBool::new(false));
        let aborted = Arc::new(AtomicBool::new(false));
        let mut reader = InterruptiblePipe::new(reader, stop.clone(), aborted.clone()).unwrap();
        let stopper = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            stop.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let error = reader.read(&mut [0; 16]).unwrap_err();
        stopper.join().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(aborted.load(Ordering::Acquire));
        assert!(start.elapsed() < Duration::from_secs(1));
    }
    #[test]
    fn interruptible_pipe_non_pipe_handle_is_rejected() {
        // SAFETY: owned unnamed event is valid, but cannot be used as a pipe.
        let event = owned(unsafe { CreateEventW(ptr::null(), 0, 0, ptr::null()) }).unwrap();
        struct NonPipe(OwnedHandle);
        impl AsRawHandle for NonPipe {
            fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
                self.0.as_raw_handle()
            }
        }
        impl Read for NonPipe {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("invalid handle must fail before read")
            }
        }
        assert!(
            InterruptiblePipe::new(
                NonPipe(event),
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
    }
    #[test]
    fn suspended_cancel_and_deadline_do_not_resume_user_code() {
        for cancelled in [true, false] {
            let cancel = AtomicBool::new(cancelled);
            let execution = super::super::ExecutionControl {
                cancel: &cancel,
                deadline: if cancelled {
                    Instant::now() + Duration::from_secs(10)
                } else {
                    Instant::now()
                },
            };
            let ownership = Ownership::new().unwrap();
            let jobs = [ownership.0.as_raw_handle()];
            let (child, primary, ownership) = suspended(&fixture_spec(), ownership, &jobs).unwrap();
            let observer = Child {
                process: owned({
                    let mut duplicate = ptr::null_mut();
                    // SAFETY: duplicate only this owned process for post-cleanup observation.
                    assert_ne!(
                        unsafe {
                            windows_sys::Win32::Foundation::DuplicateHandle(
                                GetCurrentProcess(),
                                child.process.as_raw_handle(),
                                GetCurrentProcess(),
                                &mut duplicate,
                                0,
                                0,
                                windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS,
                            )
                        },
                        0
                    );
                    duplicate
                })
                .unwrap(),
                stdout: None,
                stderr: None,
            };
            let mut output = child.stdout.as_ref().unwrap().try_clone().unwrap();
            let outcome = resume_controlled(child, primary, ownership, &execution).unwrap();
            assert!(
                matches!(outcome, SpawnOutcome::Interrupted(value) if value == if cancelled { super::super::ProcessOutcome::Cancelled } else { super::super::ProcessOutcome::TimedOut })
            );
            let mut bytes = Vec::new();
            output.read_to_end(&mut bytes).unwrap();
            assert!(bytes.is_empty());
            assert_ne!(
                unsafe { WaitForSingleObject(observer.process.as_raw_handle(), 0) },
                WAIT_TIMEOUT
            );
        }
    }
    #[test]
    fn empty_job_confirms_terminal() {
        Ownership::new().unwrap().confirm_terminal().unwrap();
    }
    #[test]
    fn live_suspended_job_terminal_timeout_is_typed() {
        let ownership = Ownership::new().unwrap();
        let jobs = [ownership.0.as_raw_handle()];
        let (_child, _primary, ownership) = suspended(&fixture_spec(), ownership, &jobs).unwrap();
        let started = Instant::now();
        let error = ownership
            .confirm_terminal_with_grace(Duration::from_millis(30))
            .unwrap_err();
        assert!(super::super::cleanup_is_unconfirmed(&error));
        assert!(started.elapsed() < Duration::from_secs(1));
        ownership.signal(true).unwrap();
        ownership.confirm_terminal().unwrap();
    }
    #[test]
    fn non_job_owned_handle_query_failure_is_typed() {
        // SAFETY: new unnamed event is owned and never used as a process or signalled.
        let event = owned(unsafe { CreateEventW(ptr::null(), 0, 0, ptr::null()) }).unwrap();
        let ownership = Ownership(event);
        let error = ownership.confirm_terminal().unwrap_err();
        assert!(super::super::cleanup_is_unconfirmed(&error));
    }
    #[test]
    fn invalid_job_assignment_fails_without_resuming_child() {
        let result = suspended(
            &fixture_spec(),
            Ownership::new().unwrap(),
            &[INVALID_HANDLE_VALUE],
        );
        assert!(result.is_err());
    }
    #[test]
    fn runtime_environment_child_probe() {
        let Some(cache) = std::env::var_os("DENO_DIR") else {
            return;
        };
        let cache = std::path::PathBuf::from(cache);
        if cache.file_name().is_none_or(|v| v != "한글 cache")
            || !cache
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|v| v.to_string_lossy().starts_with("dlpad-runtime-env-owned-"))
        {
            return;
        }
        assert_eq!(
            std::env::var_os("DENO_DISABLE_NODE_SHIM").as_deref(),
            Some(std::ffi::OsStr::new("1"))
        );
        assert!(cache.is_dir());
        println!("DLPAD_DENO_ENV_OK");
    }
    #[test]
    fn actual_owned_child_receives_fixed_unicode_runtime_environment() {
        let directory = tempfile::Builder::new()
            .prefix("dlpad-runtime-env-owned-")
            .tempdir()
            .unwrap();
        let cache = directory.path().join("한글 cache");
        std::fs::create_dir(&cache).unwrap();
        let spec = ProcessSpec {
            executable: std::env::current_exe().unwrap(),
            args: vec![
                "--exact".into(),
                "process::windows::tests::runtime_environment_child_probe".into(),
                "--nocapture".into(),
            ],
            timeout: Duration::from_secs(10),
            max_log_bytes: 4096,
            runtime_environment: Some(
                crate::runtime_environment::WindowsManagedDenoEnvironment::new(cache).unwrap(),
            ),
        };
        let mut prepared = PreparedChild::prepare(&spec).unwrap();
        let mut stdout = prepared.child.stdout.take().unwrap();
        let mut stderr = prepared.child.stderr.take().unwrap();
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + spec.timeout;
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        let SpawnOutcome::Ready(mut child, ownership) = prepared.resume(&execution).unwrap() else {
            panic!("unexpected interruption");
        };
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "owned child deadline");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success());
        ownership.confirm_terminal().unwrap();
        let mut bytes = Vec::new();
        (&mut stdout).take(4097).read_to_end(&mut bytes).unwrap();
        assert!(bytes.len() <= 4096);
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .contains("DLPAD_DENO_ENV_OK"),
            "actual child must observe both fixed values"
        );
        let mut errors = Vec::new();
        (&mut stderr).take(4097).read_to_end(&mut errors).unwrap();
        assert!(errors.is_empty());
    }
}

// Test-only original process capabilities; production helper contract unchanged.
