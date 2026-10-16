//! Explicit selected app launch; no discovery, argv override, trust or file deletion.
use super::{ActiveSelectionInspection, PreparedEntry, windows_caps::pin_source};
use crate::file_identity::{self, NativeFileIdentity};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::{FileExt, OpenOptionsExt},
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::*,
    System::{JobObjects::*, Threading::*},
};
const CLEANUP: Duration = Duration::from_secs(2);
fn rejected() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "selected app launch rejected")
}
fn unconfirmed() -> io::Error {
    io::Error::other("selected app terminal unconfirmed")
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "selected app cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "selected app deadline",
        ))
    } else {
        Ok(())
    }
}
fn wide(path: &Path) -> io::Result<Vec<u16>> {
    let mut v: Vec<_> = path.as_os_str().encode_wide().take(32768).collect();
    if v.is_empty() || v.len() > 32766 || v.contains(&0) {
        return Err(rejected());
    }
    v.push(0);
    Ok(v)
}
fn identity(file: &File, expected: &NativeFileIdentity) -> io::Result<()> {
    if file_identity::from_file(file)? != *expected {
        return Err(rejected());
    }
    Ok(())
}
fn verify_exe(
    file: &File,
    entry: &PreparedEntry,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    check(cancel, deadline)?;
    identity(file, &entry.identity)?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: live disk File and correctly sized writable native output.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
        || info.nNumberOfLinks != 1
        || file.metadata()?.len() != entry.bytes
        || entry.bytes > 128 * 1024 * 1024
    {
        return Err(rejected());
    }
    let mut buffer = [0u8; 65536];
    let mut at = 0;
    let mut hash = Sha256::new();
    while at < entry.bytes {
        check(cancel, deadline)?;
        let cap = (entry.bytes - at).min(buffer.len() as u64) as usize;
        let n = file.seek_read(&mut buffer[..cap], at)?;
        if n == 0 {
            return Err(rejected());
        }
        at += n as u64;
        hash.update(&buffer[..n]);
    }
    if file.seek_read(&mut [0], entry.bytes)? != 0
        || file.metadata()?.len() != entry.bytes
        || format!("{:x}", hash.finalize()) != entry.sha256
    {
        return Err(rejected());
    }
    identity(file, &entry.identity)?;
    check(cancel, deadline)
}
struct NamespacePin {
    file: File,
    path: PathBuf,
    final_path: PathBuf,
    id: NativeFileIdentity,
}
impl NamespacePin {
    fn verify(&self) -> io::Result<()> {
        identity(&self.file, &self.id)?;
        if file_identity::final_path(&self.file)? != self.final_path {
            return Err(rejected());
        }
        let current = directory(&self.path)?;
        identity(&current, &self.id)?;
        if file_identity::final_path(&current)? != self.final_path {
            return Err(rejected());
        }
        Ok(())
    }
}
fn directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: borrowed File and native attribute structure matching its class.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut attrs as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&attrs) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if attrs.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || attrs.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(rejected());
    }
    Ok(file)
}
fn pin_namespace(
    exe: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<Vec<NamespacePin>> {
    let parent = exe.parent().ok_or_else(rejected)?;
    let mut paths = parent.ancestors().take(257).collect::<Vec<_>>();
    if paths.len() > 256 {
        return Err(rejected());
    }
    paths.reverse();
    let mut pins = Vec::with_capacity(paths.len());
    for path in paths {
        check(cancel, deadline)?;
        let file = directory(path)?;
        check(cancel, deadline)?;
        let id = file_identity::from_file(&file)?;
        let final_path = file_identity::final_path(&file)?;
        let pin = NamespacePin {
            file,
            path: path.to_owned(),
            final_path,
            id,
        };
        check(cancel, deadline)?;
        pin.verify()?;
        check(cancel, deadline)?;
        pins.push(pin);
    }
    Ok(pins)
}
fn owned(raw: HANDLE) -> io::Result<OwnedHandle> {
    if raw.is_null() || raw as isize == -1 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: caller passes a successful newly owned native handle exactly once.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}
fn job() -> io::Result<OwnedHandle> {
    // SAFETY: no security attributes, unnamed noninherited fresh Job.
    let h = owned(unsafe { CreateJobObjectW(ptr::null(), ptr::null()) })?;
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: owned Job and matching native class/structure.
    if unsafe {
        SetInformationJobObject(
            h.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(h)
}
fn job_active_count(job: &OwnedHandle) -> io::Result<u32> {
    let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
    // SAFETY: original owned Job, correctly sized native writable output.
    if unsafe {
        QueryInformationJobObject(
            job.as_raw_handle(),
            JobObjectBasicAccountingInformation,
            (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(unconfirmed());
    }
    Ok(info.ActiveProcesses)
}
fn job_empty(job: &OwnedHandle) -> io::Result<bool> {
    Ok(job_active_count(job)? == 0)
}
fn creation(process: &OwnedHandle) -> io::Result<u64> {
    let mut c = FILETIME::default();
    let mut e = FILETIME::default();
    let mut k = FILETIME::default();
    let mut u = FILETIME::default();
    // SAFETY: original process capability and four writable FILETIMEs.
    if unsafe { GetProcessTimes(process.as_raw_handle(), &mut c, &mut e, &mut k, &mut u) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let value = ((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64;
    if value == 0 {
        return Err(rejected());
    }
    Ok(value)
}
fn image(process: &OwnedHandle) -> io::Result<PathBuf> {
    let mut bytes = vec![0u16; 32768];
    let mut count = bytes.len() as u32;
    // SAFETY: original suspended process and bounded writable UTF16 buffer.
    if unsafe {
        QueryFullProcessImageNameW(process.as_raw_handle(), 0, bytes.as_mut_ptr(), &mut count)
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if count == 0 || count as usize >= bytes.len() || bytes[..count as usize].contains(&0) {
        return Err(rejected());
    }
    bytes.truncate(count as usize);
    Ok(PathBuf::from(std::ffi::OsString::from_wide(&bytes)))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedExitReason {
    Natural,
    Cancelled,
    TimedOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedProcessExit {
    pub code: u32,
    pub reason: SelectedExitReason,
    pub job_terminal_confirmed: bool,
}
/// Original Job/leader terminal could not both be confirmed; caller still owns process.
pub struct SelectedWaitFailure {
    error: io::Error,
    terminal_unconfirmed: bool,
}
impl SelectedWaitFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn terminal_unconfirmed(&self) -> bool {
        self.terminal_unconfirmed
    }
}
impl std::fmt::Debug for SelectedWaitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectedWaitFailure")
            .field("terminal_unconfirmed", &self.terminal_unconfirmed)
            .finish()
    }
}
impl std::fmt::Display for SelectedWaitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("selected app terminal unconfirmed")
    }
}
impl std::error::Error for SelectedWaitFailure {}
pub struct OwnedSelectedProcess {
    selection: Box<ActiveSelectionInspection>,
    _exe: File,
    namespace: Vec<NamespacePin>,
    job: OwnedHandle,
    process: OwnedHandle,
    _thread: OwnedHandle,
    creation: u64,
    bound: bool,
    resumed: bool,
    resume_attempted: bool,
    may_run: bool,
    terminal: bool,
}
impl std::fmt::Debug for OwnedSelectedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedSelectedProcess")
            .field("resumed", &self.resumed)
            .field("terminal_confirmed", &self.terminal)
            .finish()
    }
}
impl OwnedSelectedProcess {
    pub fn resume_attempted(&self) -> bool {
        self.resume_attempted
    }
    pub fn may_have_executed(&self) -> bool {
        self.may_run
    }
    pub fn creation_filetime(&self) -> u64 {
        self.creation
    }
    /// Observation of the original retained process; this grants no PID-based ownership.
    pub fn process_id(&self) -> io::Result<u32> {
        // SAFETY: the original process HANDLE remains owned by this value.
        let id = unsafe { GetProcessId(self.process.as_raw_handle()) };
        if id == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(id)
        }
    }
    /// Snapshot of the original retained Job. A count is not terminal proof or PID ownership.
    pub fn active_process_count(&self) -> io::Result<u32> {
        if creation(&self.process)? != self.creation {
            return Err(unconfirmed());
        }
        job_active_count(&self.job)
    }
    fn confirm(&mut self, end: Instant) -> io::Result<u32> {
        loop {
            // SAFETY: original process handle remains owned for entire bounded wait.
            let state = unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) };
            if state != WAIT_OBJECT_0 && state != WAIT_TIMEOUT {
                return Err(unconfirmed());
            }
            if state == WAIT_OBJECT_0 && job_empty(&self.job)? {
                let mut code = 0;
                // SAFETY: signaled original process and writable exit code.
                if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
                    return Err(unconfirmed());
                }
                self.terminal = true;
                return Ok(code);
            }
            let remaining = end.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(unconfirmed());
            }
            std::thread::sleep(remaining.min(Duration::from_millis(20)));
        }
    }
    fn terminate_confirm(&mut self) -> io::Result<u32> {
        // SAFETY: only the original owned Job and, before assignment, original process.
        let signalled = unsafe {
            if self.bound {
                TerminateJobObject(self.job.as_raw_handle(), 126)
            } else {
                TerminateProcess(self.process.as_raw_handle(), 126)
            }
        };
        if signalled == 0 {
            return Err(unconfirmed());
        }
        self.confirm(Instant::now() + CLEANUP)
    }
    pub fn wait(
        &mut self,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<SelectedProcessExit, SelectedWaitFailure> {
        self.wait_inner(cancel, Some(deadline), 20)
            .map_err(|error| SelectedWaitFailure {
                error,
                terminal_unconfirmed: !self.terminal,
            })
    }
    /// Retains the original process, Job and selection until cancellation or terminal proof.
    /// This deliberately has no runtime deadline; startup still uses the launch deadline.
    pub fn wait_cancelled(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<SelectedProcessExit, SelectedWaitFailure> {
        self.wait_inner(cancel, None, 100)
            .map_err(|error| SelectedWaitFailure {
                error,
                terminal_unconfirmed: !self.terminal,
            })
    }
    fn wait_inner(
        &mut self,
        cancel: &AtomicBool,
        deadline: Option<Instant>,
        poll_ms: u32,
    ) -> io::Result<SelectedProcessExit> {
        if self.terminal {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "selected app already terminal",
            ));
        }
        loop {
            if creation(&self.process)? != self.creation {
                return Err(unconfirmed());
            }
            let reason = if cancel.load(Ordering::Acquire) {
                Some(SelectedExitReason::Cancelled)
            } else if deadline.is_some_and(|end| Instant::now() >= end) {
                Some(SelectedExitReason::TimedOut)
            } else {
                None
            };
            if let Some(reason) = reason {
                let code = self.terminate_confirm()?;
                return Ok(SelectedProcessExit {
                    code,
                    reason,
                    job_terminal_confirmed: true,
                });
            }
            // SAFETY: original retained process; every kernel wait is finite.
            let state = unsafe { WaitForSingleObject(self.process.as_raw_handle(), poll_ms) };
            if state == WAIT_OBJECT_0 && job_empty(&self.job)? {
                let code = self.confirm(Instant::now() + CLEANUP)?;
                return Ok(SelectedProcessExit {
                    code,
                    reason: SelectedExitReason::Natural,
                    job_terminal_confirmed: true,
                });
            }
            if state != WAIT_OBJECT_0 && state != WAIT_TIMEOUT {
                return Err(unconfirmed());
            }
            if state == WAIT_OBJECT_0 {
                // A signaled leader with live owned descendants must not busy-spin.
                std::thread::sleep(
                    deadline
                        .map(|end| end.saturating_duration_since(Instant::now()))
                        .unwrap_or(Duration::from_millis(u64::from(poll_ms)))
                        .min(Duration::from_millis(u64::from(poll_ms))),
                );
            }
        }
    }
}
impl Drop for OwnedSelectedProcess {
    fn drop(&mut self) {
        if !self.terminal {
            // SAFETY: best-effort safeguard on original capabilities only; no terminal claim.
            unsafe {
                if self.bound {
                    TerminateJobObject(self.job.as_raw_handle(), 126);
                } else {
                    TerminateProcess(self.process.as_raw_handle(), 126);
                }
            }
        }
    }
}
// Guard adopts all known-valid raw outputs together. Malformed success cannot
// lose original process provenance or close it before bounded termination.
struct CreatedHandleGuard {
    process: Option<OwnedHandle>,
    thread: Option<OwnedHandle>,
    parts: Option<(OwnedHandle, File, Vec<NamespacePin>)>,
    terminal: bool,
}
impl CreatedHandleGuard {
    fn adopt(
        pi: PROCESS_INFORMATION,
        job: OwnedHandle,
        exe: File,
        namespace: Vec<NamespacePin>,
    ) -> Self {
        let valid = |h: HANDLE| !h.is_null() && h as isize != -1;
        // SAFETY: successful CreateProcess transfers ownership of noninvalid
        // output handles. Equal malformed outputs are adopted once, never twice.
        let process = if valid(pi.hProcess) {
            Some(unsafe { OwnedHandle::from_raw_handle(pi.hProcess) })
        } else {
            None
        };
        let thread = if valid(pi.hThread) && pi.hThread != pi.hProcess {
            Some(unsafe { OwnedHandle::from_raw_handle(pi.hThread) })
        } else {
            None
        };
        Self {
            process,
            thread,
            parts: Some((job, exe, namespace)),
            terminal: false,
        }
    }
    fn terminate_confirm(&mut self) -> io::Result<()> {
        let process = self.process.as_ref().ok_or_else(unconfirmed)?;
        // SAFETY: original successful native process output only; not PID reopen.
        if unsafe { TerminateProcess(process.as_raw_handle(), 126) } == 0 {
            return Err(unconfirmed());
        }
        let end = Instant::now() + CLEANUP;
        loop {
            // SAFETY: retained original process, no handle closes during loop.
            let state = unsafe { WaitForSingleObject(process.as_raw_handle(), 0) };
            if state == WAIT_OBJECT_0 && job_empty(&self.parts.as_ref().ok_or_else(unconfirmed)?.0)?
            {
                let mut code = 0;
                // SAFETY: signaled original process and writable exit-code output.
                if unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) } == 0 {
                    return Err(unconfirmed());
                }
                self.terminal = true;
                return Ok(());
            }
            if state != WAIT_OBJECT_0 && state != WAIT_TIMEOUT {
                return Err(unconfirmed());
            }
            let remaining = end.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(unconfirmed());
            }
            std::thread::sleep(remaining.min(Duration::from_millis(20)));
        }
    }
}
impl Drop for CreatedHandleGuard {
    fn drop(&mut self) {
        if !self.terminal
            && let Some(process) = &self.process
        {
            // SAFETY: original retained orphan, best effort, no terminal claim.
            unsafe {
                TerminateProcess(process.as_raw_handle(), 126);
            }
        }
    }
}
pub struct SelectedLaunchFailure {
    error: io::Error,
    selection: Option<Box<ActiveSelectionInspection>>,
    child: Option<Box<OwnedSelectedProcess>>,
    orphan: Option<Box<CreatedHandleGuard>>,
    created: bool,
    resumed: bool,
    resume_attempted: bool,
    may_run: bool,
    terminal: bool,
}
impl SelectedLaunchFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn child_created(&self) -> bool {
        self.created
    }
    pub fn child_resumed(&self) -> bool {
        self.resumed
    }
    pub fn resume_attempted(&self) -> bool {
        self.resume_attempted
    }
    pub fn may_have_executed(&self) -> bool {
        self.may_run
    }
    pub fn terminal_confirmed(&self) -> bool {
        self.terminal
    }
    pub fn cleanup_unconfirmed(&self) -> bool {
        self.created && !self.terminal
    }
    pub fn retained_child(&mut self) -> Option<&mut OwnedSelectedProcess> {
        self.child.as_deref_mut()
    }
    pub fn retained_orphan_handle_count(&self) -> usize {
        self.orphan.as_ref().map_or(0, |g| {
            usize::from(g.process.is_some()) + usize::from(g.thread.is_some())
        })
    }
    pub fn selection_retained(&self) -> bool {
        self.selection.is_some() || self.child.is_some()
    }
}
impl std::fmt::Debug for SelectedLaunchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectedLaunchFailure")
            .field("kind", &self.kind())
            .field("child_created", &self.created)
            .field("child_resumed", &self.resumed)
            .field("terminal_confirmed", &self.terminal)
            .finish()
    }
}
impl std::fmt::Display for SelectedLaunchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("selected app launch failed; original capabilities preserved")
    }
}
impl std::error::Error for SelectedLaunchFailure {}
#[derive(Clone, Copy)]
enum Gate {
    CreatedSuspended,
    BeforeResume,
}
pub fn launch_explicit_selection(
    selection: ActiveSelectionInspection,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<OwnedSelectedProcess, SelectedLaunchFailure> {
    launch_inner(selection, cancel, deadline, &mut |_| Ok(()))
}
fn launch_inner(
    selection: ActiveSelectionInspection,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(Gate) -> io::Result<()>,
) -> Result<OwnedSelectedProcess, SelectedLaunchFailure> {
    let mut selection = Some(Box::new(selection));
    let mut child: Option<OwnedSelectedProcess> = None;
    let mut orphan: Option<CreatedHandleGuard> = None;
    let mut created = false;
    let outcome = (|| -> io::Result<()> {
        check(cancel, deadline)?;
        let inspect = selection.as_ref().ok_or_else(rejected)?;
        inspect.verify_current(cancel, deadline)?;
        let (source, path) = inspect.launch_inputs()?;
        let expected = inspect.record().selected_executable.clone();
        let namespace = pin_namespace(&path, cancel, deadline)?;
        let exe = pin_source(source)?;
        verify_exe(&exe, &expected, cancel, deadline)?;
        if file_identity::final_path(&exe)? != path {
            return Err(rejected());
        }
        for pin in &namespace {
            check(cancel, deadline)?;
            pin.verify()?;
            check(cancel, deadline)?;
        }
        inspect.verify_current(cancel, deadline)?;
        check(cancel, deadline)?;
        let job = job()?;
        let application = wide(&path)?;
        // The sole argv0 is the exact validated executable; Windows paths cannot contain quotes.
        let mut command = vec![34u16];
        command.extend_from_slice(&application[..application.len() - 1]);
        command.extend_from_slice(&[34, 0]);
        if command.len() > 32767 {
            return Err(rejected());
        }
        let startup = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        // SAFETY: fixed argv0, live bounded UTF16 buffers, no inherited handles,
        // untouched ambient environment, suspended original child, no shell/std pipes.
        if unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                0,
                CREATE_SUSPENDED | CREATE_NO_WINDOW,
                ptr::null(),
                ptr::null(),
                &startup,
                &mut pi,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        created = true;
        // Immediately protect both successful PI outputs. No fallible conversion
        // may discard an unbound suspended original process.
        let mut guard = CreatedHandleGuard::adopt(pi, job, exe, namespace);
        if guard.process.is_none() || guard.thread.is_none() {
            orphan = Some(guard);
            return Err(rejected());
        }
        let process = guard.process.take().expect("checked original process");
        let thread = guard
            .thread
            .take()
            .expect("checked original primary thread");
        let (job, exe, namespace) = guard.parts.take().expect("original startup pins");
        child = Some(OwnedSelectedProcess {
            selection: selection.take().ok_or_else(rejected)?,
            _exe: exe,
            namespace,
            job,
            process,
            _thread: thread,
            creation: 0,
            bound: false,
            resumed: false,
            resume_attempted: false,
            may_run: false,
            terminal: false,
        });
        let owned = child.as_mut().ok_or_else(rejected)?;
        owned.creation = creation(&owned.process)?;
        hook(Gate::CreatedSuspended)?;
        // SAFETY: original suspended child assigned to fresh kill-on-close Job before resume.
        if unsafe {
            AssignProcessToJobObject(owned.job.as_raw_handle(), owned.process.as_raw_handle())
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        owned.bound = true;
        let actual = image(&owned.process)?;
        let image_file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&actual)?;
        verify_exe(&image_file, &expected, cancel, deadline)?;
        if file_identity::final_path(&image_file)? != path {
            return Err(rejected());
        }
        owned.selection.verify_current(cancel, deadline)?;
        for pin in &owned.namespace {
            check(cancel, deadline)?;
            pin.verify()?;
            check(cancel, deadline)?;
        }
        identity(&owned._exe, &expected.identity)?;
        hook(Gate::BeforeResume)?;
        check(cancel, deadline)?;
        // SAFETY: original primary thread is suspended once and owned until terminal.
        owned.resume_attempted = true;
        let previous = unsafe { ResumeThread(owned._thread.as_raw_handle()) };
        if previous == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        owned.may_run = previous <= 1;
        if previous != 1 {
            return Err(rejected());
        }
        owned.resumed = true;
        Ok(())
    })();
    match outcome {
        Ok(()) => Ok(child.expect("successful original child")),
        Err(mut error) => {
            let resumed = child.as_ref().is_some_and(|v| v.resumed);
            let resume_attempted = child.as_ref().is_some_and(|v| v.resume_attempted);
            let may_run = child.as_ref().is_some_and(|v| v.may_run);
            let mut terminal = false;
            if let Some(c) = child.as_mut() {
                match c.terminate_confirm() {
                    Ok(_) => terminal = true,
                    Err(e) => error = e,
                }
            }
            if let Some(guard) = orphan.as_mut() {
                match guard.terminate_confirm() {
                    Ok(()) => terminal = true,
                    Err(e) => error = e,
                }
            }
            Err(SelectedLaunchFailure {
                error,
                selection,
                child: child.map(Box::new),
                orphan: orphan.map(Box::new),
                created,
                resumed,
                resume_attempted,
                may_run,
                terminal,
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::super::{
        create_registration_receipt, inspect_active_selection, inspect_registered_with_receipt,
        manifest::{BundleEntry, BundleManifest},
        prepare_new, register_fresh, select_registered_pair,
    };
    use super::*;
    use std::io::Read;
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(60)
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let source = File::open(std::env::current_exe().unwrap()).unwrap();
        let mut read = source.try_clone().unwrap();
        let mut h = Sha256::new();
        let mut b = [0u8; 65536];
        let mut bytes = 0;
        loop {
            let n = read.read(&mut b).unwrap();
            if n == 0 {
                break;
            }
            bytes += n as u64;
            assert!(bytes <= 128 * 1024 * 1024);
            h.update(&b[..n]);
        }
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes,
                sha256: format!("{:x}", h.finalize()),
            }],
        };
        let c = AtomicBool::new(false);
        let reg = dir.path().join("registered");
        let receipt = dir.path().join("receipt");
        let active = dir.path().join("active");
        let prepared =
            prepare_new(&manifest, &[("dlpad.exe", &source)], &reg, &c, later()).unwrap();
        let registered = register_fresh(prepared, &c, later()).unwrap();
        let external = create_registration_receipt(&registered, &receipt, &c, later()).unwrap();
        drop(external);
        drop(registered);
        let pair = inspect_registered_with_receipt(&reg, &receipt, &c, later()).unwrap();
        let selected = select_registered_pair(pair, &active, &c, later()).unwrap();
        drop(selected);
        (dir, reg, receipt, active)
    }
    fn inspect(reg: &Path, receipt: &Path, active: &Path) -> ActiveSelectionInspection {
        inspect_active_selection(active, reg, receipt, &AtomicBool::new(false), later()).unwrap()
    }
    #[test]
    fn pre_cancel_deadline_never_create_and_preserve_selection() {
        let (_dir, reg, receipt, active) = fixture();
        for (cancel, deadline, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let fail = launch_explicit_selection(
                inspect(&reg, &receipt, &active),
                &AtomicBool::new(cancel),
                deadline,
            )
            .unwrap_err();
            assert_eq!(fail.kind(), kind);
            assert!(!fail.child_created());
            assert!(!fail.child_resumed());
            assert!(fail.selection_retained());
            drop(fail);
            inspect(&reg, &receipt, &active)
                .verify_current(&AtomicBool::new(false), later())
                .unwrap();
        }
    }
    #[test]
    fn original_suspended_child_failure_and_pre_resume_cancel_are_terminal_without_execution() {
        let (_dir, reg, receipt, active) = fixture();
        // Real current test PE is copied, but never resumed. No GUI/tests child execution.
        let c = AtomicBool::new(false);
        let fail =
            launch_inner(
                inspect(&reg, &receipt, &active),
                &c,
                later(),
                &mut |gate| match gate {
                    Gate::CreatedSuspended => Err(io::Error::other("fixture binding gate failure")),
                    Gate::BeforeResume => Ok(()),
                },
            )
            .unwrap_err();
        assert!(fail.child_created());
        assert!(!fail.child_resumed());
        assert!(!fail.resume_attempted());
        assert!(!fail.may_have_executed());
        assert!(fail.terminal_confirmed());
        assert!(fail.selection_retained());
        drop(fail);
        let c = AtomicBool::new(false);
        let fail = launch_inner(inspect(&reg, &receipt, &active), &c, later(), &mut |gate| {
            if matches!(gate, Gate::BeforeResume) {
                c.store(true, Ordering::Release);
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(fail.kind(), io::ErrorKind::Interrupted);
        assert!(fail.child_created());
        assert!(!fail.child_resumed());
        assert!(!fail.resume_attempted());
        assert!(!fail.may_have_executed());
        assert!(fail.terminal_confirmed());
        drop(fail);
        inspect(&reg, &receipt, &active)
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
    }
}
