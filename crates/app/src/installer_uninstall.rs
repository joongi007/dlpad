//! Explicit current-user uninstall entry and retained-capability native controller.
//! Paths are requests only. Engine inspections/fresh removal grant authority.
use std::{ffi::OsString, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Roots {
    bundle: PathBuf,
    journal: PathBuf,
}
fn parse_roots(args: &[OsString]) -> Result<Roots, ()> {
    if args.len() != 5
        || args[0] != "--uninstall"
        || args[1] != "--bundle-root"
        || args[3] != "--journal-root"
    {
        return Err(());
    }
    let bundle = PathBuf::from(&args[2]);
    let journal = PathBuf::from(&args[4]);
    for path in [&bundle, &journal] {
        let value = path.to_str().ok_or(())?;
        if !path.is_absolute()
            || value.len() > 32760
            || value.contains(['\0', '"'])
            || path.components().any(|part| {
                matches!(
                    part,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return Err(());
        }
    }
    if bundle == journal {
        return Err(());
    }
    Ok(Roots { bundle, journal })
}

/// Called before installer ZIP discovery. Never accepts a positional ZIP/source.
pub(crate) fn maybe_run(args: &[OsString]) -> Option<i32> {
    if !args
        .iter()
        .any(|a| a == "--uninstall" || a == "--uninstall-child")
    {
        return None;
    }
    #[cfg(windows)]
    {
        Some(native::entry(args).unwrap_or_else(|_| {
            native::entry_error();
            2
        }))
    }
    #[cfg(not(windows))]
    {
        let _ = parse_roots(args);
        Some(2)
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use crate::installer_uninstall_catalog as text;
    use dlpad_engine::{app_install as install, file_identity};
    use std::{
        fs::File,
        io::{self, Read, Write},
        mem,
        os::windows::{
            ffi::{OsStrExt, OsStringExt},
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
        ptr,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, SyncSender},
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{JobObjects::*, LibraryLoader::GetModuleHandleW, Threading::*},
        UI::{
            Input::KeyboardAndMouse::{EnableWindow, SetFocus},
            WindowsAndMessaging::*,
        },
    };
    const CHILD_FLAG: &str = "--uninstall-child";
    const CLASS: &str = "DlpadExplicitUninstaller";
    const WAKE: u32 = WM_APP + 51;
    const REMOVE: usize = 101;
    const CLOSE: usize = 102;
    const DEADLINE: Duration = Duration::from_secs(120);
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreatePipe(
            read: *mut HANDLE,
            write: *mut HANDLE,
            attrs: *const windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
            size: u32,
        ) -> i32;
        fn PeekNamedPipe(
            pipe: HANDLE,
            buffer: *mut std::ffi::c_void,
            size: u32,
            read: *mut u32,
            available: *mut u32,
            left: *mut u32,
        ) -> i32;
    }
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(algorithm: HANDLE, buffer: *mut u8, bytes: u32, flags: u32) -> i32;
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQueryInformationProcess(
            process: HANDLE,
            class: u32,
            output: *mut std::ffi::c_void,
            length: u32,
            returned: *mut u32,
        ) -> i32;
    }
    #[repr(C)]
    #[derive(Default)]
    struct BasicProcess {
        reserved1: usize,
        peb: usize,
        reserved2: [usize; 2],
        pid: usize,
        parent_pid: usize,
    }
    fn error() -> io::Error {
        io::ErrorKind::PermissionDenied.into()
    }
    fn handle(raw: HANDLE) -> io::Result<OwnedHandle> {
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful freshly owned API result, transferred exactly once.
        Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
    }
    fn wide(value: &std::ffi::OsStr) -> io::Result<Vec<u16>> {
        let mut units: Vec<_> = value.encode_wide().collect();
        if units.len() > 32760 || units.contains(&0) {
            return Err(error());
        }
        units.push(0);
        Ok(units)
    }
    fn creation(process: HANDLE) -> io::Result<u64> {
        let (mut c, mut e, mut k, mut u) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        if unsafe { GetProcessTimes(process, &mut c, &mut e, &mut k, &mut u) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64)
    }
    fn image(process: HANDLE) -> io::Result<PathBuf> {
        let mut buf = vec![0u16; 32768];
        let mut count = buf.len() as u32;
        if unsafe { QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut count) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(PathBuf::from(OsString::from_wide(&buf[..count as usize])))
    }
    fn same_path(a: &std::path::Path, b: &std::path::Path) -> bool {
        fn normalized(p: &std::path::Path) -> String {
            p.to_string_lossy()
                .trim_start_matches("\\\\?\\")
                .to_lowercase()
        }
        normalized(a) == normalized(b)
    }
    fn duplicate(raw: HANDLE) -> io::Result<OwnedHandle> {
        let mut copy = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                raw,
                GetCurrentProcess(),
                &mut copy,
                0,
                1,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        handle(copy)
    }
    fn active(job: HANDLE) -> io::Result<u32> {
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        if unsafe {
            QueryInformationJobObject(
                job,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                mem::size_of_val(&info) as u32,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(info.ActiveProcesses)
    }
    fn own_job() -> io::Result<OwnedHandle> {
        let job = handle(unsafe { CreateJobObjectW(ptr::null(), ptr::null()) })?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe {
            SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                mem::size_of_val(&info) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(job)
    }
    fn quote(value: &std::ffi::OsStr, output: &mut Vec<u16>) -> io::Result<()> {
        let units: Vec<_> = value.encode_wide().collect();
        if units.contains(&0) {
            return Err(error());
        }
        output.push(34);
        let mut slashes = 0;
        for unit in units {
            if unit == 92 {
                slashes += 1;
                continue;
            }
            if unit == 34 {
                output.extend(std::iter::repeat_n(92, slashes * 2 + 1));
            } else {
                output.extend(std::iter::repeat_n(92, slashes));
            }
            slashes = 0;
            output.push(unit);
        }
        output.extend(std::iter::repeat_n(92, slashes * 2));
        output.push(34);
        Ok(())
    }
    fn nonce() -> io::Result<String> {
        let mut bytes = [0u8; 16];
        if unsafe { BCryptGenRandom(ptr::null_mut(), bytes.as_mut_ptr(), 16, 2) } < 0 {
            return Err(error());
        }
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
    }
    struct AttributeList {
        memory: Vec<usize>,
        initialized: bool,
    }
    impl AttributeList {
        fn new(handles: &mut [HANDLE]) -> io::Result<Self> {
            let mut size = 0;
            unsafe {
                InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut size);
            }
            if size == 0 || size > 65536 {
                return Err(error());
            }
            let mut value = Self {
                memory: vec![0; size.div_ceil(mem::size_of::<usize>())],
                initialized: false,
            };
            if unsafe {
                InitializeProcThreadAttributeList(value.memory.as_mut_ptr().cast(), 1, 0, &mut size)
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            value.initialized = true;
            if unsafe {
                UpdateProcThreadAttribute(
                    value.memory.as_mut_ptr().cast(),
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    handles.as_mut_ptr().cast(),
                    mem::size_of_val(handles),
                    ptr::null_mut(),
                    ptr::null(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(value)
        }
    }
    impl Drop for AttributeList {
        fn drop(&mut self) {
            if self.initialized {
                unsafe {
                    DeleteProcThreadAttributeList(self.memory.as_mut_ptr().cast());
                }
            }
        }
    }
    struct CreatedChild {
        process: OwnedHandle,
        thread: OwnedHandle,
        accepted: bool,
    }
    impl Drop for CreatedChild {
        fn drop(&mut self) {
            if !self.accepted {
                // Exact newly created process HANDLE only. No running app/PID adoption.
                unsafe {
                    TerminateProcess(self.process.as_raw_handle(), 126);
                    WaitForSingleObject(self.process.as_raw_handle(), 30000);
                }
            }
        }
    }
    /// Bootstrap has no HWND. All copy/hash/ACK waiting occurs before UI exists.
    fn handoff(roots: Roots) -> io::Result<()> {
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + DEADLINE;
        let copy = install::prepare_current_uninstall_copy(&cancel, deadline)?;
        copy.verify_current(&cancel, deadline)?;
        let mut parent_raw = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                GetCurrentProcess(),
                GetCurrentProcess(),
                &mut parent_raw,
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                1,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let parent = handle(parent_raw)?;
        let source = duplicate(copy.source().as_raw_handle())?;
        let destination = duplicate(copy.copy().as_raw_handle())?;
        let job = own_job()?;
        let child_job = duplicate(job.as_raw_handle())?;
        let mut read = ptr::null_mut();
        let mut write = ptr::null_mut();
        let sa = windows_sys::Win32::Security::SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<windows_sys::Win32::Security::SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        if unsafe { CreatePipe(&mut read, &mut write, &sa, 128) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut ack = File::from(handle(read)?);
        let ack_writer = handle(write)?;
        if unsafe { SetHandleInformation(ack.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let nonce = nonce()?;
        let parent_creation = creation(parent.as_raw_handle())?;
        let raw = [
            parent.as_raw_handle(),
            child_job.as_raw_handle(),
            source.as_raw_handle(),
            destination.as_raw_handle(),
            ack_writer.as_raw_handle(),
        ];
        let mut inherited = raw;
        let attributes = AttributeList::new(&mut inherited)?;
        let mut args = vec![
            copy.path().as_os_str().to_owned(),
            CHILD_FLAG.into(),
            roots.bundle.into_os_string(),
            roots.journal.into_os_string(),
            nonce.clone().into(),
            parent_creation.to_string().into(),
            copy.sha256().into(),
            copy.current_sid().into(),
        ];
        args.extend(
            raw.iter()
                .map(|h| OsString::from((*h as usize).to_string())),
        );
        let mut command = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                command.push(32);
            }
            quote(arg, &mut command)?;
        }
        command.push(0);
        if command.len() > 32767 {
            return Err(error());
        }
        let application = wide(copy.path().as_os_str())?;
        // Never inherit the registered helper's directory: Windows keeps the
        // current directory open without delete sharing for the child's lifetime.
        // The verified private copy's parent is outside every removal namespace.
        let working_directory = wide(copy.path().parent().ok_or_else(error)?.as_os_str())?;
        let startup = STARTUPINFOEXW {
            StartupInfo: STARTUPINFOW {
                cb: mem::size_of::<STARTUPINFOEXW>() as u32,
                ..Default::default()
            },
            lpAttributeList: attributes.memory.as_ptr() as _,
        };
        let mut pi = PROCESS_INFORMATION::default();
        if unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                1,
                CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT,
                ptr::null(),
                working_directory.as_ptr(),
                &startup.StartupInfo,
                &mut pi,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut child = CreatedChild {
            process: handle(pi.hProcess)?,
            thread: handle(pi.hThread)?,
            accepted: false,
        };
        if unsafe { AssignProcessToJobObject(job.as_raw_handle(), child.process.as_raw_handle()) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        copy.verify_current(&cancel, deadline)?;
        if !same_path(&image(child.process.as_raw_handle())?, copy.path()) {
            return Err(error());
        }
        if unsafe { ResumeThread(child.thread.as_raw_handle()) } == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        drop(ack_writer);
        drop(source);
        drop(destination);
        drop(child_job);
        drop(parent);
        loop {
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            if unsafe { WaitForSingleObject(child.process.as_raw_handle(), 0) } != WAIT_TIMEOUT {
                return Err(error());
            }
            let mut available = 0;
            if unsafe {
                PeekNamedPipe(
                    ack.as_raw_handle(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut available,
                    ptr::null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if available >= 32 {
                let mut bytes = [0u8; 32];
                ack.read_exact(&mut bytes)?;
                if bytes != nonce.as_bytes() {
                    return Err(error());
                }
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        // Child owns an inherited Job HANDLE and waits for this exact original process
        // exit before it can show a confirmation UI. No detach/breakaway/global kill.
        child.accepted = true;
        drop(copy);
        Ok(())
    }
    struct ChildLifetime {
        job: OwnedHandle,
    }
    impl ChildLifetime {
        fn finish(self) -> io::Result<()> {
            let mut member = 0;
            if unsafe { IsProcessInJob(GetCurrentProcess(), self.job.as_raw_handle(), &mut member) }
                == 0
                || member == 0
                || active(self.job.as_raw_handle())? != 1
            {
                return Err(error());
            }
            // Only this verified helper remains. Removing KILL_ON_CLOSE prevents
            // normal last-handle close from killing its own successful final exit.
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            if unsafe {
                QueryInformationJobObject(
                    self.job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&mut info as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    mem::size_of_val(&info) as u32,
                    ptr::null_mut(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if info.BasicLimitInformation.LimitFlags != JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE {
                return Err(error());
            }
            info.BasicLimitInformation.LimitFlags = 0;
            if unsafe {
                SetInformationJobObject(
                    self.job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    mem::size_of_val(&info) as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }
    fn inherited(value: &OsString) -> io::Result<OwnedHandle> {
        let raw = value
            .to_str()
            .ok_or_else(error)?
            .parse::<usize>()
            .map_err(|_| error())?;
        if raw == 0 || raw == usize::MAX {
            return Err(error());
        }
        let mut flags = 0;
        if unsafe { GetHandleInformation(raw as HANDLE, &mut flags) } == 0
            || flags & HANDLE_FLAG_INHERIT == 0
        {
            return Err(error());
        }
        if unsafe { SetHandleInformation(raw as HANDLE, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        handle(raw as HANDLE)
    }
    fn accept_child(args: &[OsString]) -> io::Result<(Roots, ChildLifetime)> {
        if args.len() != 12 || args[0] != CHILD_FLAG {
            return Err(error());
        }
        let roots = parse_roots(&[
            "--uninstall".into(),
            "--bundle-root".into(),
            args[1].clone(),
            "--journal-root".into(),
            args[2].clone(),
        ])
        .map_err(|_| error())?;
        let nonce = args[3].to_str().ok_or_else(error)?;
        let hash = args[5].to_str().ok_or_else(error)?;
        let sid = args[6].to_str().ok_or_else(error)?;
        if nonce.len() != 32
            || !nonce.bytes().all(|b| b.is_ascii_hexdigit())
            || hash.len() != 64
            || !hash.bytes().all(|b| b.is_ascii_hexdigit())
            || sid.len() > 255
        {
            return Err(error());
        }
        let expected_creation = args[4]
            .to_str()
            .ok_or_else(error)?
            .parse::<u64>()
            .map_err(|_| error())?;
        let numbers: Vec<_> = args[7..]
            .iter()
            .map(|a| {
                a.to_str()
                    .ok_or_else(error)?
                    .parse::<usize>()
                    .map_err(|_| error())
            })
            .collect::<io::Result<_>>()?;
        if numbers
            .iter()
            .enumerate()
            .any(|(i, n)| numbers[..i].contains(n))
        {
            return Err(error());
        }
        let parent = inherited(&args[7])?;
        let job = inherited(&args[8])?;
        let source = File::from(inherited(&args[9])?);
        let copy = File::from(inherited(&args[10])?);
        let mut ack = File::from(inherited(&args[11])?);
        let deadline = Instant::now() + DEADLINE;
        let cancel = AtomicBool::new(false);
        let mut basic = BasicProcess::default();
        if unsafe {
            NtQueryInformationProcess(
                GetCurrentProcess(),
                0,
                (&mut basic as *mut BasicProcess).cast(),
                mem::size_of::<BasicProcess>() as u32,
                ptr::null_mut(),
            )
        } < 0
        {
            return Err(error());
        }
        if basic.parent_pid != unsafe { GetProcessId(parent.as_raw_handle()) } as usize
            || creation(parent.as_raw_handle())? != expected_creation
            || unsafe { WaitForSingleObject(parent.as_raw_handle(), 0) } != WAIT_TIMEOUT
        {
            return Err(error());
        }
        let mut member = 0;
        if unsafe { IsProcessInJob(GetCurrentProcess(), job.as_raw_handle(), &mut member) } == 0
            || member == 0
            || active(job.as_raw_handle())? != 1
        {
            return Err(error());
        }
        if !same_path(
            &image(parent.as_raw_handle())?,
            &file_identity::final_path(&source)?,
        ) || !same_path(
            &image(unsafe { GetCurrentProcess() })?,
            &file_identity::final_path(&copy)?,
        ) {
            return Err(error());
        }
        install::verify_uninstall_handoff_files(&source, &copy, hash, sid, &cancel, deadline)?;
        ack.write_all(nonce.as_bytes())?;
        ack.flush()?;
        drop(ack);
        let remaining = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u32::MAX as u128) as u32;
        if unsafe { WaitForSingleObject(parent.as_raw_handle(), remaining) } != WAIT_OBJECT_0 {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let mut exit = 0;
        if unsafe { GetExitCodeProcess(parent.as_raw_handle(), &mut exit) } == 0 || exit != 0 {
            return Err(error());
        }
        // Original source pin must end before registered support removal opens DELETE.
        drop(source);
        drop(copy);
        drop(parent);
        Ok((roots, ChildLifetime { job }))
    }
    #[cfg(test)]
    mod handoff_tests {
        use super::*;
        #[test]
        fn malformed_internal_entry_cannot_adopt_or_wait_on_handles() {
            let root = std::env::temp_dir();
            let mut args: Vec<OsString> = vec![
                CHILD_FLAG.into(),
                root.join("bundle").into_os_string(),
                root.join("journal").into_os_string(),
                "0".repeat(32).into(),
                "1".into(),
                "a".repeat(64).into(),
                "S-1-5-21-1".into(),
            ];
            args.extend((0..5).map(|_| OsString::from("42")));
            assert!(
                accept_child(&args).is_err(),
                "duplicate handles must fail before adoption"
            );
            args[3] = "invalid nonce".into();
            assert!(accept_child(&args).is_err());
            args[3] = "0".repeat(32).into();
            args[5] = "not a digest".into();
            assert!(accept_child(&args).is_err());
            args.push("--force".into());
            assert!(accept_child(&args).is_err());
        }
        #[test]
        fn full_terminal_mailbox_drains_after_window_exit() {
            let (send, commands) = mpsc::sync_channel(1);
            let (events_tx, events) = mpsc::sync_channel(1);
            let (ready, wait) = mpsc::sync_channel(1);
            let cancel = Arc::new(AtomicBool::new(false));
            let stopped = Arc::new(AtomicBool::new(false));
            let stopped_thread = Arc::clone(&stopped);
            let thread = thread::spawn(move || {
                events_tx.send(Phase::Confirm).unwrap();
                ready.send(()).unwrap();
                assert!(matches!(commands.recv(), Ok(Command::Stop)));
                events_tx.send(Phase::Cancelled).unwrap();
                stopped_thread.store(true, Ordering::Release);
            });
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            let mut worker = Worker {
                send,
                events,
                cancel,
                thread: Some(thread),
                stopped,
            };
            assert!(worker.finish_after_window());
            assert!(worker.thread.is_none());
            assert!(worker.cancel.load(Ordering::Acquire));
        }
        #[test]
        fn native_command_line_roundtrips_paths_quotes_and_trailing_slashes() {
            let args = [
                "C:\\owned folder\\dlpad-uninstall.exe",
                "C:\\한글 폴더\\",
                "a\"b",
                "ends\\",
                "",
            ];
            let mut line = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    line.push(32);
                }
                quote(arg.as_ref(), &mut line).unwrap();
            }
            line.push(0);
            let mut count = 0;
            let parsed = unsafe {
                windows_sys::Win32::UI::Shell::CommandLineToArgvW(line.as_ptr(), &mut count)
            };
            assert!(!parsed.is_null());
            let actual: Vec<OsString> = (0..count as usize)
                .map(|i| {
                    let start = unsafe { *parsed.add(i) };
                    let mut len = 0;
                    while unsafe { *start.add(len) } != 0 {
                        len += 1;
                    }
                    OsString::from_wide(unsafe { std::slice::from_raw_parts(start, len) })
                })
                .collect();
            unsafe {
                LocalFree(parsed.cast());
            }
            assert_eq!(
                actual,
                args.into_iter().map(OsString::from).collect::<Vec<_>>()
            );
        }
    }
    pub(super) fn entry(args: &[OsString]) -> io::Result<i32> {
        if args.first().is_some_and(|a| a == CHILD_FLAG) {
            let (roots, lifetime) = accept_child(args)?;
            let outcome = window(roots);
            let finish = lifetime.finish();
            outcome?;
            finish?;
            Ok(0)
        } else {
            handoff(parse_roots(args).map_err(|_| error())?)?;
            Ok(0)
        }
    }
    pub(super) fn entry_error() {
        let title = wide(text::TITLE.as_ref()).unwrap_or_default();
        let message = wide(text::BAD_ENTRY.as_ref()).unwrap_or_default();
        unsafe {
            MessageBoxW(
                ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Phase {
        Checking,
        OtherWindow,
        Confirm,
        Removing,
        Stopping,
        Done,
        Cancelled,
        Failed { partial: bool },
    }
    #[derive(Clone, Copy)]
    enum Command {
        Remove,
        Stop,
    }
    struct Worker {
        send: SyncSender<Command>,
        events: Receiver<Phase>,
        cancel: Arc<AtomicBool>,
        thread: Option<JoinHandle<()>>,
        stopped: Arc<AtomicBool>,
    }
    impl Worker {
        fn new(roots: Roots, hwnd: HWND) -> io::Result<Self> {
            let (send, commands) = mpsc::sync_channel(1);
            let (events_tx, events) = mpsc::sync_channel(1);
            let cancel = Arc::new(AtomicBool::new(false));
            let stopped = Arc::new(AtomicBool::new(false));
            let c = Arc::clone(&cancel);
            let s = Arc::clone(&stopped);
            let window = hwnd as usize;
            let thread = thread::Builder::new()
                .name("dlpad-explicit-uninstall".into())
                .spawn(move || {
                    let publish = |phase| {
                        let _ = events_tx.send(phase);
                        unsafe {
                            PostMessageW(window as HWND, WAKE, 0, 0);
                        }
                    };
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        worker_body(roots, &commands, &c, &publish)
                    }));
                    if result.is_err() {
                        publish(Phase::Failed { partial: true });
                    }
                    s.store(true, Ordering::Release);
                    unsafe {
                        PostMessageW(window as HWND, WAKE, 0, 0);
                    }
                })?;
            Ok(Self {
                send,
                events,
                cancel,
                thread: Some(thread),
                stopped,
            })
        }
        fn stop(&self) {
            self.cancel.store(true, Ordering::Release);
            let _ = self.send.try_send(Command::Stop);
        }
        fn remove(&self) -> bool {
            self.send.try_send(Command::Remove).is_ok()
        }
        fn finished(&self) -> bool {
            self.stopped.load(Ordering::Acquire)
                && self.thread.as_ref().is_none_or(JoinHandle::is_finished)
        }
        fn finish_after_window(&mut self) -> bool {
            self.stop();
            // The HWND/message loop has ended. Keep draining the bounded mailbox:
            // a worker may be publishing its terminal ACK behind an unread sample.
            while self
                .thread
                .as_ref()
                .is_some_and(|worker| !worker.is_finished())
            {
                let _ = self.events.recv_timeout(Duration::from_millis(75));
            }
            self.thread
                .take()
                .is_none_or(|worker| worker.join().is_ok())
                && self.stopped.load(Ordering::Acquire)
        }
        fn join_finished(&mut self) {
            if self.finished()
                && let Some(worker) = self.thread.take()
            {
                let _ = worker.join();
            }
        }
    }
    fn worker_body(
        roots: Roots,
        commands: &Receiver<Command>,
        cancel: &AtomicBool,
        publish: &dyn Fn(Phase),
    ) {
        let request = install::LocalInspectionRequest {
            bundle_root: roots.bundle.clone(),
            journal_root: roots.journal.clone(),
        };
        let shell = loop {
            match install::inspect_local_windows_shell(&request, cancel, Instant::now() + DEADLINE)
            {
                Ok(value) => break value,
                Err(error) => {
                    publish(if error.kind() == io::ErrorKind::WouldBlock {
                        Phase::OtherWindow
                    } else {
                        Phase::Failed { partial: false }
                    });
                    if !matches!(commands.recv(), Ok(Command::Remove))
                        || cancel.load(Ordering::Acquire)
                    {
                        return;
                    }
                }
            }
        };
        publish(Phase::Confirm);
        if !matches!(commands.recv(), Ok(Command::Remove)) || cancel.load(Ordering::Acquire) {
            publish(Phase::Cancelled);
            return;
        }
        publish(Phase::Removing);
        let deadline = Instant::now() + DEADLINE;
        let result = install::authorize_explicit_user_bundle_removal(
            &roots.bundle,
            &roots.journal,
            cancel,
            deadline,
        )
        .and_then(|removal| removal.remove(cancel, deadline));
        let mut failure = match result {
            Ok(value) => {
                if !value.outer_namespace_absence_confirmed {
                    publish(Phase::Failed { partial: true });
                    return;
                }
                finish_shell(shell, commands, cancel, publish);
                return;
            }
            Err(value) => value,
        };
        loop {
            publish(if cancel.load(Ordering::Acquire) {
                Phase::Cancelled
            } else {
                Phase::Failed {
                    partial: failure.mutation_may_have_occurred(),
                }
            });
            if !matches!(commands.recv(), Ok(Command::Remove)) || cancel.load(Ordering::Acquire) {
                return;
            }
            publish(Phase::Removing);
            match failure.retry(cancel, Instant::now() + DEADLINE) {
                Ok(value) => {
                    if !value.outer_namespace_absence_confirmed {
                        publish(Phase::Failed { partial: true });
                        return;
                    }
                    finish_shell(shell, commands, cancel, publish);
                    return;
                }
                Err(value) => failure = value,
            }
        }
    }
    fn finish_shell(
        shell: install::ShellRegistrationInspection,
        commands: &Receiver<Command>,
        cancel: &AtomicBool,
        publish: &dyn Fn(Phase),
    ) {
        let mut outcome = shell.cleanup(cancel, Instant::now() + DEADLINE);
        loop {
            let failure = match outcome {
                Ok(_) => {
                    publish(Phase::Done);
                    return;
                }
                Err(failure) => failure,
            };
            publish(if cancel.load(Ordering::Acquire) {
                Phase::Cancelled
            } else {
                Phase::Failed { partial: true }
            });
            if !matches!(commands.recv(), Ok(Command::Remove)) || cancel.load(Ordering::Acquire) {
                return;
            }
            publish(Phase::Removing);
            outcome = failure.retry(cancel, Instant::now() + DEADLINE);
        }
    }
    struct WindowState {
        hwnd: HWND,
        worker: Worker,
        phase: Phase,
        closing: bool,
        status: HWND,
        remove: HWND,
        close: HWND,
        font: HFONT,
    }
    fn label(window: HWND, value: &str) {
        let value: Vec<_> = value.encode_utf16().chain([0]).collect();
        unsafe {
            SetWindowTextW(window, value.as_ptr());
        }
    }
    fn sync(state: &WindowState) {
        let message = match state.phase {
            Phase::Checking => text::CHECKING,
            Phase::OtherWindow => text::OTHER_WINDOW,
            Phase::Confirm => text::READY,
            Phase::Removing => text::REMOVING,
            Phase::Stopping => text::STOPPING,
            Phase::Done => text::DONE,
            Phase::Cancelled => text::CANCELLED,
            Phase::Failed { partial: false } => text::FAILED,
            Phase::Failed { partial: true } => text::PARTIAL,
        };
        label(state.status, message);
        label(
            state.remove,
            if matches!(state.phase, Phase::Failed { .. } | Phase::OtherWindow) {
                text::RETRY
            } else {
                text::REMOVE
            },
        );
        let busy = matches!(
            state.phase,
            Phase::Checking | Phase::Removing | Phase::Stopping
        );
        if busy || (state.worker.stopped.load(Ordering::Acquire) && !state.worker.finished()) {
            unsafe {
                SetTimer(state.hwnd, 1, 75, None);
            }
        } else {
            unsafe {
                KillTimer(state.hwnd, 1);
            }
        }
        label(
            state.close,
            if busy {
                text::STOP
            } else if state.phase == Phase::Confirm {
                text::CANCEL
            } else {
                text::CLOSE
            },
        );
        unsafe {
            EnableWindow(
                state.remove,
                i32::from(
                    !state.closing
                        && matches!(
                            state.phase,
                            Phase::Confirm | Phase::Failed { .. } | Phase::OtherWindow
                        )
                        && !state.worker.finished(),
                ),
            );
            EnableWindow(state.close, i32::from(state.phase != Phase::Stopping));
        }
    }
    unsafe extern "system" fn procedure(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
        if !pointer.is_null() {
            let state = unsafe { &mut *pointer };
            match message {
                WAKE | WM_TIMER => {
                    while let Ok(phase) = state.worker.events.try_recv() {
                        state.phase = phase;
                    }
                    state.worker.join_finished();
                    sync(state);
                    if state.closing && state.worker.finished() {
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    }
                    return 0;
                }
                WM_COMMAND => {
                    match w & 0xffff {
                        REMOVE => {
                            if matches!(
                                state.phase,
                                Phase::Confirm | Phase::Failed { .. } | Phase::OtherWindow
                            ) && state.worker.remove()
                            {
                                state.phase = Phase::Removing;
                                sync(state);
                            }
                        }
                        CLOSE => unsafe {
                            SendMessageW(hwnd, WM_CLOSE, 0, 0);
                        },
                        _ => {}
                    }
                    return 0;
                }
                WM_CLOSE => {
                    state.closing = true;
                    state.worker.stop();
                    if state.worker.finished() {
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    } else {
                        state.phase = Phase::Stopping;
                        sync(state);
                    }
                    return 0;
                }
                WM_DESTROY => {
                    unsafe {
                        PostQuitMessage(0);
                    }
                    return 0;
                }
                _ => {}
            }
        }
        unsafe { DefWindowProcW(hwnd, message, w, l) }
    }
    fn window(roots: Roots) -> io::Result<()> {
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        let class = wide(CLASS.as_ref())?;
        let title = wide(text::TITLE.as_ref())?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hIcon: unsafe { LoadIconW(instance, 101usize as _) },
            hbrBackground: (COLOR_WINDOW + 1) as _,
            ..Default::default()
        };
        if unsafe { RegisterClassW(&wc) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                700,
                460,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null(),
            )
        };
        if hwnd.is_null() {
            return Err(io::Error::last_os_error());
        }
        let font = unsafe {
            CreateFontW(
                -17,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                0,
                0,
                CLEARTYPE_QUALITY as u32,
                0,
                wide("Segoe UI".as_ref())?.as_ptr(),
            )
        };
        let create = |class: &str,
                      value: &str,
                      id: usize,
                      style: u32,
                      x: i32,
                      y: i32,
                      width: i32,
                      height: i32|
         -> io::Result<HWND> {
            let c = wide(class.as_ref())?;
            let v = wide(value.as_ref())?;
            let control = unsafe {
                CreateWindowExW(
                    0,
                    c.as_ptr(),
                    v.as_ptr(),
                    WS_CHILD | WS_VISIBLE | style,
                    x,
                    y,
                    width,
                    height,
                    hwnd,
                    id as _,
                    instance,
                    ptr::null(),
                )
            };
            if control.is_null() {
                return Err(io::Error::last_os_error());
            }
            unsafe {
                SendMessageW(control, WM_SETFONT, font as usize, 1);
            }
            Ok(control)
        };
        let setup = (|| {
            create("STATIC", text::HEADING, 201, 0, 24, 24, 640, 30)?;
            create("STATIC", text::PRESERVED, 202, 0, 24, 66, 640, 44)?;
            create(
                "EDIT",
                &roots.bundle.to_string_lossy(),
                203,
                ES_READONLY as u32
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | WS_VSCROLL
                    | WS_TABSTOP,
                24,
                122,
                640,
                74,
            )?;
            let status = create("STATIC", text::CHECKING, 204, 0, 24, 218, 640, 64)?;
            create("STATIC", text::TEMP_REMAINS, 205, 0, 24, 294, 640, 36)?;
            let remove = create(
                "BUTTON",
                text::REMOVE,
                REMOVE,
                WS_TABSTOP | BS_PUSHBUTTON as u32,
                404,
                350,
                124,
                36,
            )?;
            let close = create(
                "BUTTON",
                text::CANCEL,
                CLOSE,
                WS_TABSTOP | BS_PUSHBUTTON as u32,
                540,
                350,
                124,
                36,
            )?;
            let worker = Worker::new(roots, hwnd)?;
            Ok(Box::new(WindowState {
                hwnd,
                worker,
                phase: Phase::Checking,
                closing: false,
                status,
                remove,
                close,
                font,
            }))
        })();
        let mut state = match setup {
            Ok(s) => s,
            Err(e) => {
                unsafe {
                    DestroyWindow(hwnd);
                    DeleteObject(font);
                }
                return Err(e);
            }
        };
        unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                (&mut *state as *mut WindowState) as isize,
            );
            ShowWindow(hwnd, SW_SHOW);
            SetFocus(state.close);
        }
        sync(&state);
        let mut msg = MSG::default();
        let failed = loop {
            let value = unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
            if value <= 0 {
                break value < 0;
            }
            if unsafe { IsDialogMessageW(hwnd, &msg) } == 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        };
        let worker_acknowledged = state.worker.finish_after_window();
        unsafe {
            // WM_QUIT/GetMessage failure can end a loop before WM_DESTROY. Never
            // leave a live HWND with a pointer to the soon-to-drop boxed state.
            if IsWindow(hwnd) != 0 {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                DestroyWindow(hwnd);
            }
            DeleteObject(state.font);
        }
        if failed || !worker_acknowledged {
            Err(io::ErrorKind::Other.into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(a: &str, b: &str) -> Vec<OsString> {
        ["--uninstall", "--bundle-root", a, "--journal-root", b]
            .into_iter()
            .map(Into::into)
            .collect()
    }
    #[test]
    fn explicit_roots_are_required_without_zip_discovery() {
        let base = std::env::temp_dir();
        let a = base.join("dlpad-uninstall-bundle");
        let b = base.join("dlpad-uninstall-journal");
        assert_eq!(
            parse_roots(&request(a.to_str().unwrap(), b.to_str().unwrap())).unwrap(),
            Roots {
                bundle: a.clone(),
                journal: b.clone()
            }
        );
        assert!(parse_roots(&request("relative", b.to_str().unwrap())).is_err());
        assert!(parse_roots(&request(a.to_str().unwrap(), a.to_str().unwrap())).is_err());
        let mut args = request(a.to_str().unwrap(), b.to_str().unwrap());
        args.push("--force".into());
        assert!(parse_roots(&args).is_err());
    }
    #[test]
    fn unrelated_installer_arguments_are_not_intercepted() {
        assert_eq!(
            maybe_run(&["--manifest".into(), "bundle.json".into()]),
            None
        );
    }
    #[test]
    fn ambiguous_or_escaping_inputs_are_rejected() {
        let base = std::env::temp_dir();
        let a = base.join("a");
        let b = base.join("b");
        for bad in [
            format!("{}\0other", a.display()),
            format!("{}/../foreign", a.display()),
            format!("{}\"", a.display()),
        ] {
            assert!(parse_roots(&request(&bad, b.to_str().unwrap())).is_err());
        }
    }
}
