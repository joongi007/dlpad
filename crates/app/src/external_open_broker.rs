//! Explicit Explorer handoff candidate; never owns the user's viewer/Explorer.
//! Worker-only; cfg(test), --fixture and non-Default desktops never invoke native COM.
use std::{
    path::{Component, Path},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    NoAssociation,
    Unavailable,
    Identity,
    AutomationBlocked,
    Cancelled,
    TimedOut,
    Dispatch,
}
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
struct Request<'a> {
    file: &'a str,
    directory: &'a str,
    verb: &'static str,
    kind: crate::library_open::Kind,
}
impl<'a> Request<'a> {
    fn new(
        file: &'a Path,
        directory: &'a Path,
        kind: crate::library_open::Kind,
    ) -> Result<Self, Error> {
        if kind != crate::library_open::Kind::Folder && !crate::library_open::supports_file(file) {
            return Err(Error::Identity);
        }
        fn text(p: &Path) -> Result<&str, Error> {
            if !p.is_absolute()
                || p.components()
                    .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
            {
                return Err(Error::Identity);
            }
            #[cfg(windows)]
            if !matches!(p.components().next(),Some(Component::Prefix(x)) if matches!(x.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
            {
                return Err(Error::Identity);
            }
            let text = p.to_str().ok_or(Error::Identity)?;
            if text.is_empty()
                || text.encode_utf16().count() > 32766
                || text.chars().any(char::is_control)
            {
                return Err(Error::Identity);
            }
            Ok(text)
        }
        if kind == crate::library_open::Kind::RevealFile && file.parent() != Some(directory) {
            return Err(Error::Identity);
        }
        Ok(Self {
            kind,
            file: text(file)?,
            directory: text(directory)?,
            verb: if kind == crate::library_open::Kind::ChooseApplication {
                "openas"
            } else {
                "open"
            },
        })
    }
}
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
fn checkpoint(stop: &AtomicBool, deadline: Instant) -> Result<(), Error> {
    if stop.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Error::TimedOut)
    } else {
        Ok(())
    }
}
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
trait Provider {
    type Bound;
    fn capture(&mut self) -> Result<Self::Bound, Error>;
    fn verify(&mut self, bound: &Self::Bound) -> Result<(), Error>;
    fn invoke(
        &mut self,
        bound: &Self::Bound,
        request: &Request<'_>,
        stop: &AtomicBool,
        deadline: Instant,
    ) -> Result<(), Error>;
}
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
fn handoff_with(
    provider: &mut impl Provider,
    request: &Request<'_>,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<(), Error> {
    checkpoint(stop, deadline)?;
    let bound = provider.capture()?;
    checkpoint(stop, deadline)?;
    provider.verify(&bound)?;
    checkpoint(stop, deadline)?;
    // Success means the COM call accepted a handoff, not a new process/first-paint ACK.
    // After-dispatch cancel/deadline/source changes remain caller's existing Report policy.
    provider.invoke(&bound, request, stop, deadline)
}
#[derive(Clone, Copy)]
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
struct Eligibility {
    same_user: bool,
    same_session: bool,
    outside_job: bool,
    alive: bool,
    window_matches: bool,
}
#[cfg_attr(not(all(windows, feature = "cpu")), allow(dead_code))]
impl Eligibility {
    fn verify(self) -> Result<(), Error> {
        if self.same_user
            && self.same_session
            && self.outside_job
            && self.alive
            && self.window_matches
        {
            Ok(())
        } else {
            Err(Error::Identity)
        }
    }
}

pub(crate) fn dispatch(
    file: &Path,
    directory: &Path,
    kind: crate::library_open::Kind,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<(), Error> {
    if cfg!(test) || std::env::args_os().any(|a| a == "--fixture") {
        return Err(Error::AutomationBlocked);
    }
    let request = Request::new(file, directory, kind)?;
    #[cfg(all(windows, feature = "cpu"))]
    {
        native::dispatch(&request, stop, deadline)
    }
    #[cfg(not(all(windows, feature = "cpu")))]
    {
        let _ = (request, stop, deadline);
        Err(Error::Unavailable)
    }
}

#[cfg(all(windows, feature = "cpu"))]
mod native {
    use super::*;
    use std::{
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows::{
        Win32::{
            System::{
                Com::{CLSCTX_LOCAL_SERVER, CoCreateInstance, IDispatch, IServiceProvider},
                Variant::VARIANT,
            },
            UI::Shell::{
                IShellBrowser, IShellDispatch2, IShellFolderViewDual, IShellWindows,
                SID_STopLevelBrowser, SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH,
                ShellWindows,
            },
        },
        core::{BSTR, Interface},
    };
    use windows_sys::Win32::{
        Foundation::{FILETIME, HANDLE, HWND, WAIT_TIMEOUT},
        Security::{
            GetLengthSid, GetTokenInformation, IsValidSid, TOKEN_QUERY, TOKEN_USER, TokenSessionId,
            TokenUser,
        },
        System::{
            Com::{
                COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
            },
            JobObjects::IsProcessInJob,
            StationsAndDesktops::{
                GetProcessWindowStation, GetThreadDesktop, GetUserObjectInformationW, UOI_NAME,
            },
            SystemInformation::GetWindowsDirectoryW,
            Threading::{
                GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId, GetProcessId,
                GetProcessTimes, OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32,
                PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, QueryFullProcessImageNameW,
                WaitForSingleObject,
            },
        },
        UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId, IsWindow},
    };
    struct Bound {
        process: OwnedHandle,
        hwnd: HWND,
        pid: u32,
        tid: u32,
        creation: u64,
        sid: Vec<u8>,
        session: u32,
        dispatch: IShellDispatch2,
    }
    struct Native;
    fn owned(h: HANDLE) -> Result<OwnedHandle, Error> {
        if h.is_null() || h == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(Error::Unavailable);
        }
        // SAFETY: newly returned real process/token HANDLE; exactly one owner, not a pseudo-handle.
        Ok(unsafe { OwnedHandle::from_raw_handle(h) })
    }
    fn default_desktop() -> Result<(), Error> {
        let mut station = [0u16; 256];
        let mut station_bytes = 0;
        if unsafe {
            GetUserObjectInformationW(
                GetProcessWindowStation(),
                UOI_NAME,
                station.as_mut_ptr().cast(),
                512,
                &mut station_bytes,
            )
        } == 0
            || !(2..=512).contains(&station_bytes)
        {
            return Err(Error::AutomationBlocked);
        }
        let end = station
            .iter()
            .position(|x| *x == 0)
            .ok_or(Error::AutomationBlocked)?;
        if String::from_utf16(&station[..end]).ok().as_deref() != Some("WinSta0") {
            return Err(Error::AutomationBlocked);
        }
        let mut name = [0u16; 256];
        let mut needed = 0;
        if unsafe {
            GetUserObjectInformationW(
                GetThreadDesktop(GetCurrentThreadId()),
                UOI_NAME,
                name.as_mut_ptr().cast(),
                512,
                &mut needed,
            )
        } == 0
            || !(2..=512).contains(&needed)
        {
            return Err(Error::AutomationBlocked);
        }
        let end = name
            .iter()
            .position(|x| *x == 0)
            .ok_or(Error::AutomationBlocked)?;
        if String::from_utf16(&name[..end]).ok().as_deref() != Some("Default") {
            return Err(Error::AutomationBlocked);
        }
        Ok(())
    }
    fn token_owner(process: HANDLE) -> Result<(Vec<u8>, u32), Error> {
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(Error::Identity);
        }
        let token = owned(raw)?;
        let mut bytes = 0;
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut bytes,
            );
        }
        if bytes < std::mem::size_of::<TOKEN_USER>() as u32 || bytes > 4096 {
            return Err(Error::Identity);
        }
        let capacity = bytes;
        let mut storage = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                storage.as_mut_ptr().cast(),
                capacity,
                &mut bytes,
            )
        } == 0
            || bytes > capacity
            || bytes < std::mem::size_of::<TOKEN_USER>() as u32
        {
            return Err(Error::Identity);
        }
        // SAFETY: aligned complete TOKEN_USER header lives in bounded initialized storage.
        let sid = unsafe { (*storage.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let start = storage.as_ptr() as usize;
        let end = start.checked_add(bytes as usize).ok_or(Error::Identity)?;
        let address = sid as usize;
        if address < start || address.checked_add(8).is_none_or(|v| v > end) {
            return Err(Error::Identity);
        }
        // SID variable body has a bounded SubAuthorityCount; establish full buffer range
        // before either native SID validator can inspect that body.
        let count = unsafe { *sid.cast::<u8>().add(1) } as usize;
        let length = 8usize
            .checked_add(count.checked_mul(4).ok_or(Error::Identity)?)
            .ok_or(Error::Identity)?;
        if length > 256
            || address.checked_add(length).is_none_or(|v| v > end)
            || unsafe { IsValidSid(sid) } == 0
            || unsafe { GetLengthSid(sid) } as usize != length
        {
            return Err(Error::Identity);
        }
        let sid = unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }.to_vec();
        let mut session = 0u32;
        let mut read = 0;
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenSessionId,
                (&mut session as *mut u32).cast(),
                4,
                &mut read,
            )
        } == 0
            || read != 4
        {
            return Err(Error::Identity);
        }
        Ok((sid, session))
    }
    fn creation(process: HANDLE) -> Result<u64, Error> {
        let mut t = [FILETIME::default(); 4];
        if unsafe { GetProcessTimes(process, &mut t[0], &mut t[1], &mut t[2], &mut t[3]) } == 0 {
            return Err(Error::Identity);
        }
        let time = (u64::from(t[0].dwHighDateTime) << 32) | u64::from(t[0].dwLowDateTime);
        if time == 0 {
            Err(Error::Identity)
        } else {
            Ok(time)
        }
    }
    fn explorer_image(process: HANDLE) -> Result<(), Error> {
        let mut base = [0u16; 32768];
        let n = unsafe { GetWindowsDirectoryW(base.as_mut_ptr(), base.len() as u32) } as usize;
        if n == 0 || n >= base.len() {
            return Err(Error::Identity);
        }
        let mut actual = [0u16; 32768];
        let mut length = actual.len() as u32;
        if unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                actual.as_mut_ptr(),
                &mut length,
            )
        } == 0
            || length == 0
            || length as usize >= actual.len()
        {
            return Err(Error::Identity);
        }
        let expected = Path::new(&String::from_utf16(&base[..n]).map_err(|_| Error::Identity)?)
            .join("explorer.exe");
        let image = String::from_utf16(&actual[..length as usize]).map_err(|_| Error::Identity)?;
        if !expected
            .to_str()
            .is_some_and(|e| e.eq_ignore_ascii_case(&image))
        {
            return Err(Error::Identity);
        }
        Ok(())
    }
    fn membership(process: HANDLE) -> Result<bool, Error> {
        let mut value = 0;
        if unsafe { IsProcessInJob(process, ptr::null_mut(), &mut value) } == 0 {
            return Err(Error::Identity);
        }
        Ok(value != 0)
    }
    fn window_owner(hwnd: HWND) -> Result<(u32, u32), Error> {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if hwnd.is_null() || unsafe { IsWindow(hwnd) } == 0 || pid == 0 || tid == 0 {
            return Err(Error::Identity);
        }
        Ok((pid, tid))
    }
    fn verify(bound: &Bound) -> Result<(), Error> {
        default_desktop()?;
        let owner = token_owner(bound.process.as_raw_handle())?;
        let current = token_owner(unsafe { GetCurrentProcess() })?;
        let window = window_owner(bound.hwnd)?;
        Eligibility {
            same_user: owner.0 == current.0 && owner.0 == bound.sid,
            same_session: owner.1 == current.1 && owner.1 == bound.session,
            outside_job: !membership(bound.process.as_raw_handle())?,
            alive: unsafe { WaitForSingleObject(bound.process.as_raw_handle(), 0) } == WAIT_TIMEOUT,
            window_matches: unsafe { GetShellWindow() } == bound.hwnd
                && window == (bound.pid, bound.tid)
                && unsafe { GetProcessId(bound.process.as_raw_handle()) } == bound.pid
                && creation(bound.process.as_raw_handle())? == bound.creation,
        }
        .verify()?;
        explorer_image(bound.process.as_raw_handle())
    }
    impl Provider for Native {
        type Bound = Bound;
        fn capture(&mut self) -> Result<Bound, Error> {
            default_desktop()?;
            let hwnd = unsafe { GetShellWindow() };
            if hwnd.is_null() {
                return Err(Error::Unavailable);
            }
            let (pid, tid) = window_owner(hwnd)?;
            if pid == unsafe { GetCurrentProcessId() } {
                return Err(Error::Identity);
            }
            let process = owned(unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    pid,
                )
            })?;
            let (sid, session) = token_owner(process.as_raw_handle())?;
            let current = token_owner(unsafe { GetCurrentProcess() })?;
            Eligibility {
                same_user: sid == current.0,
                same_session: session == current.1,
                outside_job: !membership(process.as_raw_handle())?,
                alive: unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } == WAIT_TIMEOUT,
                window_matches: unsafe { GetShellWindow() } == hwnd
                    && window_owner(hwnd)? == (pid, tid)
                    && unsafe { GetProcessId(process.as_raw_handle()) } == pid,
            }
            .verify()?;
            explorer_image(process.as_raw_handle())?;
            let creation = creation(process.as_raw_handle())?;
            // Official ExecInExplorer desktop service chain, existing local-server context.
            // Every COM query failure is refusal; never direct Shell fallback from a Job.
            let windows: IShellWindows =
                unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER) }
                    .map_err(|_| Error::Unavailable)?;
            let empty = VARIANT::default();
            let mut desktop = 0i32;
            let dispatch = unsafe {
                windows.FindWindowSW(&empty, &empty, SWC_DESKTOP, &mut desktop, SWFO_NEEDDISPATCH)
            }
            .map_err(|_| Error::Unavailable)?;
            // HWND returned as LONG by legacy Automation API; sign-extend Windows handle spelling.
            if desktop == 0 || desktop as isize as HWND != hwnd {
                return Err(Error::Identity);
            }
            let service: IServiceProvider = dispatch.cast().map_err(|_| Error::Dispatch)?;
            let browser: IShellBrowser = unsafe { service.QueryService(&SID_STopLevelBrowser) }
                .map_err(|_| Error::Dispatch)?;
            let view = unsafe { browser.QueryActiveShellView() }.map_err(|_| Error::Dispatch)?;
            for actual in [unsafe { browser.GetWindow() }, unsafe { view.GetWindow() }] {
                let actual = actual.map_err(|_| Error::Identity)?;
                if window_owner(actual.0)?.0 != pid {
                    return Err(Error::Identity);
                }
            }
            let background: IDispatch =
                unsafe { view.GetItemObject(SVGIO_BACKGROUND) }.map_err(|_| Error::Dispatch)?;
            let folder: IShellFolderViewDual = background.cast().map_err(|_| Error::Dispatch)?;
            let application = unsafe { folder.Application() }.map_err(|_| Error::Dispatch)?;
            let dispatch: IShellDispatch2 = application.cast().map_err(|_| Error::Dispatch)?;
            let bound = Bound {
                process,
                hwnd,
                pid,
                tid,
                creation,
                sid,
                session,
                dispatch,
            };
            verify(&bound)?;
            Ok(bound)
        }
        fn verify(&mut self, bound: &Bound) -> Result<(), Error> {
            verify(bound)
        }
        fn invoke(
            &mut self,
            bound: &Bound,
            request: &Request<'_>,
            stop: &AtomicBool,
            deadline: Instant,
        ) -> Result<(), Error> {
            if request.kind == crate::library_open::Kind::RevealFile {
                // Retain the original outside-Job Explorer identity gate. This API asks
                // Shell to select an item; it is not a remote call on bound.dispatch.
                verify(bound)?;
                return reveal_file(request.file, stop, deadline);
            }
            let file = BSTR::from(request.file);
            let args = VARIANT::from("");
            let directory = VARIANT::from(request.directory);
            let verb = VARIANT::from(request.verb);
            let show = VARIANT::from(1i32);
            // SAFETY: original desktop-chain COM interface, owned BSTR/VARIANT lifetimes.
            // No command text, executable construction, arbitrary verb/arguments, or process ownership.
            unsafe {
                bound
                    .dispatch
                    .ShellExecute(&file, &args, &directory, &verb, &show)
            }
            .map_err(|error| {
                // Preserve only the exact HRESULT_FROM_WIN32(ERROR_NO_ASSOCIATION).
                // Generic COM errors and successful-but-asynchronous dispatch are not evidence.
                if error.code().0 as u32 == 0x80070483 {
                    Error::NoAssociation
                } else {
                    Error::Dispatch
                }
            })
        }
    }
    struct Pidl(*mut std::ffi::c_void);
    impl Drop for Pidl {
        fn drop(&mut self) {
            // SAFETY: owned PIDL returned by SHParseDisplayName, freed exactly once.
            unsafe { windows_sys::Win32::System::Com::CoTaskMemFree(self.0) }
        }
    }
    impl Pidl {
        fn parse(path: &str) -> Result<Self, Error> {
            use std::ffi::c_void;
            #[link(name = "shell32")]
            unsafe extern "system" {
                fn SHParseDisplayName(
                    path: *const u16,
                    context: *const c_void,
                    pidl: *mut *mut c_void,
                    attributes: u32,
                    observed: *mut u32,
                ) -> i32;
            }
            let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
            let mut raw = ptr::null_mut();
            // SAFETY: bounded validated local UTF-16 spelling; COM is initialized on
            // this worker. Parsing resolves metadata only; no Explorer window is opened.
            let result = unsafe {
                SHParseDisplayName(path.as_ptr(), ptr::null(), &mut raw, 0, ptr::null_mut())
            };
            if result < 0 || raw.is_null() {
                if !raw.is_null() {
                    drop(Self(raw));
                }
                return Err(Error::Dispatch);
            }
            Ok(Self(raw))
        }
    }
    /// One absolute file PIDL, no parsed command line, process spawn or edit-mode flags.
    /// The existing Explorer must already have passed capture/verify; never create
    /// our own Explorer or retry through a different desktop when Shell refuses.
    fn reveal_file(path: &str, stop: &AtomicBool, deadline: Instant) -> Result<(), Error> {
        use std::ffi::c_void;
        #[link(name = "shell32")]
        unsafe extern "system" {
            fn SHOpenFolderAndSelectItems(
                pidl: *const c_void,
                count: u32,
                children: *const *const c_void,
                flags: u32,
            ) -> i32;
        }
        checkpoint(stop, deadline)?;
        let pidl = Pidl::parse(path)?;
        checkpoint(stop, deadline)?;
        // SAFETY: cidl=0 selects the complete item PIDL in its parent folder;
        // null child list is the documented form. Flags=0 never renames the file.
        let result = unsafe { SHOpenFolderAndSelectItems(pidl.0, 0, ptr::null(), 0) };
        if result < 0 {
            Err(Error::Dispatch)
        } else {
            Ok(())
        }
    }
    #[cfg(test)]
    #[test]
    fn reveal_native_parser_roundtrips_short_and_long_unicode_without_opening_windows() {
        use std::os::windows::ffi::OsStringExt;
        #[link(name = "shell32")]
        unsafe extern "system" {
            fn SHGetPathFromIDListEx(
                pidl: *const std::ffi::c_void,
                path: *mut u16,
                count: u32,
                flags: u32,
            ) -> i32;
        }
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe { CoUninitialize() }
            }
        }
        assert!(
            unsafe {
                CoInitializeEx(
                    ptr::null(),
                    (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
                )
            } >= 0
        );
        let _apartment = Apartment;
        let root = std::env::temp_dir().join(format!(
            "dlpad-shell-pidl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        struct Owned(std::path::PathBuf);
        impl Drop for Owned {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let owned = Owned(root);
        let mut directory = std::fs::canonicalize(&owned.0).unwrap();
        for long in [false, true] {
            if long {
                for index in 0..6 {
                    directory.push(format!("{}-{index}", "가".repeat(40)));
                    std::fs::create_dir(&directory).unwrap();
                }
            }
            let file = directory.join("한글 영상 & literal.mp4");
            std::fs::write(&file, b"synthetic media").unwrap();
            let shell = crate::path_picker::display_path(&file);
            assert_eq!(shell.encode_utf16().count() > 260, long);
            let pidl = Pidl::parse(&shell)
                .expect("actual production Shell parser accepts normalized input");
            let mut text = vec![0u16; 32768];
            assert_ne!(
                unsafe { SHGetPathFromIDListEx(pidl.0, text.as_mut_ptr(), text.len() as u32, 0) },
                0
            );
            let length = text.iter().position(|u| *u == 0).unwrap();
            let roundtrip =
                std::path::PathBuf::from(std::ffi::OsString::from_wide(&text[..length]));
            assert_eq!(
                std::fs::canonicalize(roundtrip).unwrap(),
                std::fs::canonicalize(file).unwrap()
            );
        }
    }
    pub(super) fn dispatch(
        request: &Request<'_>,
        stop: &AtomicBool,
        deadline: Instant,
    ) -> Result<(), Error> {
        default_desktop()?;
        checkpoint(stop, deadline)?;
        if unsafe {
            CoInitializeEx(
                ptr::null(),
                (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
            )
        } < 0
        {
            return Err(Error::Unavailable);
        }
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe { CoUninitialize() }
            }
        }
        let _apartment = Apartment;
        handoff_with(&mut Native, request, stop, deadline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    struct Fake {
        eligibility: Eligibility,
        capture_error: Option<Error>,
        change_after_query: bool,
        invocations: Vec<(String, String)>,
        kinds: Vec<crate::library_open::Kind>,
        stop_after: bool,
        cancel_after_capture: bool,
        stop: std::sync::Arc<AtomicBool>,
    }
    impl Provider for Fake {
        type Bound = Eligibility;
        fn capture(&mut self) -> Result<Eligibility, Error> {
            if let Some(e) = self.capture_error {
                return Err(e);
            }
            self.eligibility.verify()?;
            if self.cancel_after_capture {
                self.stop.store(true, Ordering::Release);
            }
            Ok(self.eligibility)
        }
        fn verify(&mut self, bound: &Eligibility) -> Result<(), Error> {
            let mut current = *bound;
            if self.change_after_query {
                current.window_matches = false;
            }
            current.verify()
        }
        fn invoke(
            &mut self,
            _: &Eligibility,
            request: &Request<'_>,
            _: &AtomicBool,
            _: Instant,
        ) -> Result<(), Error> {
            self.invocations
                .push((request.file.into(), request.directory.into()));
            self.kinds.push(request.kind);
            if self.stop_after {
                self.stop.store(true, Ordering::Release);
            }
            Ok(())
        }
    }
    fn eligible() -> Eligibility {
        Eligibility {
            same_user: true,
            same_session: true,
            outside_job: true,
            alive: true,
            window_matches: true,
        }
    }
    fn fake() -> Fake {
        Fake {
            eligibility: eligible(),
            capture_error: None,
            change_after_query: false,
            invocations: vec![],
            kinds: vec![],
            stop_after: false,
            cancel_after_capture: false,
            stop: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }
    fn paths() -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir();
        (root.join("한글 영상 & literal.mp4"), root)
    }
    #[test]
    fn mismatched_owner_session_job_and_window_never_invoke() {
        for field in 0..5 {
            let mut provider = fake();
            match field {
                0 => provider.eligibility.same_user = false,
                1 => provider.eligibility.same_session = false,
                2 => provider.eligibility.outside_job = false,
                3 => provider.eligibility.alive = false,
                _ => provider.eligibility.window_matches = false,
            };
            let (file, dir) = paths();
            let request = Request::new(&file, &dir, crate::library_open::Kind::File).unwrap();
            assert_eq!(
                handoff_with(
                    &mut provider,
                    &request,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(1)
                ),
                Err(Error::Identity)
            );
            assert!(provider.invocations.is_empty());
        }
    }
    #[test]
    fn no_explorer_com_failure_replaced_window_cancel_and_deadline_never_invoke() {
        for error in [
            Error::Unavailable,
            Error::Dispatch,
            Error::Identity,
            Error::AutomationBlocked,
        ] {
            let mut provider = fake();
            provider.capture_error = Some(error);
            let (file, dir) = paths();
            let request = Request::new(&file, &dir, crate::library_open::Kind::File).unwrap();
            assert_eq!(
                handoff_with(
                    &mut provider,
                    &request,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(1)
                ),
                Err(error)
            );
            assert!(provider.invocations.is_empty());
        }
        let (file, dir) = paths();
        let request = Request::new(&file, &dir, crate::library_open::Kind::File).unwrap();
        let mut provider = fake();
        provider.change_after_query = true;
        assert_eq!(
            handoff_with(
                &mut provider,
                &request,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Identity)
        );
        assert!(provider.invocations.is_empty());
        provider.change_after_query = false;
        provider.cancel_after_capture = true;
        let during_query_stop = provider.stop.clone();
        assert_eq!(
            handoff_with(
                &mut provider,
                &request,
                &during_query_stop,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Cancelled)
        );
        assert!(
            provider.invocations.is_empty(),
            "cancellation during COM query never reaches invocation"
        );
        provider.cancel_after_capture = false;
        assert_eq!(
            handoff_with(
                &mut provider,
                &request,
                &AtomicBool::new(true),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Cancelled)
        );
        assert_eq!(
            handoff_with(
                &mut provider,
                &request,
                &AtomicBool::new(false),
                Instant::now() - Duration::from_millis(1)
            ),
            Err(Error::TimedOut)
        );
        assert!(provider.invocations.is_empty());
    }
    #[test]
    fn literal_path_one_explicit_handoff_and_late_cancel_not_false_refusal() {
        let (file, dir) = paths();
        let request = Request::new(&file, &dir, crate::library_open::Kind::File).unwrap();
        let mut provider = fake();
        provider.stop_after = true;
        let stop = provider.stop.clone();
        assert_eq!(
            handoff_with(
                &mut provider,
                &request,
                &stop,
                Instant::now() + Duration::from_secs(1)
            ),
            Ok(())
        );
        assert!(stop.load(Ordering::Acquire));
        assert_eq!(
            provider.invocations,
            vec![(file.to_str().unwrap().into(), dir.to_str().unwrap().into())]
        );
    }
    #[test]
    fn reveal_is_one_literal_file_selection_and_wrong_parent_is_refused() {
        let (file, dir) = paths();
        let request = Request::new(&file, &dir, crate::library_open::Kind::RevealFile).unwrap();
        let mut provider = fake();
        handoff_with(
            &mut provider,
            &request,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(1),
        )
        .unwrap();
        assert_eq!(provider.kinds, vec![crate::library_open::Kind::RevealFile]);
        assert_eq!(
            provider.invocations,
            vec![(file.to_str().unwrap().into(), dir.to_str().unwrap().into())]
        );
        assert!(
            Request::new(
                &file,
                &dir.join("other"),
                crate::library_open::Kind::RevealFile
            )
            .is_err()
        );
        assert_eq!(
            dispatch(
                &file,
                &dir,
                crate::library_open::Kind::RevealFile,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::AutomationBlocked)
        );
        for field in 0..5 {
            let mut provider = fake();
            match field {
                0 => provider.eligibility.same_user = false,
                1 => provider.eligibility.same_session = false,
                2 => provider.eligibility.outside_job = false,
                3 => provider.eligibility.alive = false,
                _ => provider.eligibility.window_matches = false,
            }
            assert_eq!(
                handoff_with(
                    &mut provider,
                    &request,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(1)
                ),
                Err(Error::Identity)
            );
            assert!(provider.invocations.is_empty());
        }
    }
    #[test]
    fn native_dispatch_is_automatically_blocked_and_invalid_targets_have_no_capability() {
        let (file, dir) = paths();
        assert_eq!(
            dispatch(
                &file,
                &dir,
                crate::library_open::Kind::File,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::AutomationBlocked)
        );
        assert!(
            Request::new(
                &dir.join("script.exe"),
                &dir,
                crate::library_open::Kind::File
            )
            .is_err()
        );
        assert!(
            Request::new(
                Path::new("relative.mp4"),
                &dir,
                crate::library_open::Kind::File
            )
            .is_err()
        );
        assert!(
            Request::new(
                &dir.join("../other.mp4"),
                &dir,
                crate::library_open::Kind::File
            )
            .is_err()
        );
        assert!(
            Request::new(
                &dir.join("bad\n.mp4"),
                &dir,
                crate::library_open::Kind::File
            )
            .is_err()
        );
    }
}
