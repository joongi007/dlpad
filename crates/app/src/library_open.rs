//! Explicit user handoff. Worker-only fresh validation; never owns/kills the associated viewer.
use dlpad_engine::{
    file_identity::{self, NativeFileIdentity},
    library::LibraryEntry,
};
use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant, UNIX_EPOCH},
};
pub(crate) const DISPATCH_BUDGET: Duration = Duration::from_secs(10);
const MAX_PATH_UNITS: usize = 32_768;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    File,
    ChooseApplication,
    RevealFile,
    Folder,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    Stale,
    Unsafe,
    Unsupported,
    Cancelled,
    TimedOut,
    NoAssociation,
    Dispatch,
    AutomationBlocked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Report {
    HandedOff,
    AfterCancellation,
    AfterDeadline,
    SourceChanged,
}
/// This deliberately excludes executable/script/shortcut/URL extensions. It is not a content/trust claim.
pub(crate) fn supports_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "mp4"
                | "mkv"
                | "webm"
                | "mov"
                | "avi"
                | "m4v"
                | "mp3"
                | "m4a"
                | "aac"
                | "flac"
                | "ogg"
                | "opus"
                | "wav"
        )
    })
}
pub(crate) fn available() -> bool {
    cfg!(all(windows, feature = "cpu"))
}
fn checkpoint(stop: &AtomicBool, deadline: Instant) -> Result<(), Error> {
    if stop.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Error::TimedOut)
    } else {
        Ok(())
    }
}
fn safe(path: &Path, directory: bool) -> Result<(), Error> {
    let m = fs::symlink_metadata(path).map_err(|_| Error::Stale)?;
    if m.file_type().is_symlink() || (directory && !m.is_dir()) || (!directory && !m.is_file()) {
        return Err(Error::Unsafe);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(Error::Unsafe);
        }
    }
    Ok(())
}
fn local_path(path: &Path) -> Result<(), Error> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(Error::Unsafe);
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::path::Prefix;
        let units: Vec<u16> = path.as_os_str().encode_wide().collect();
        if units.len() >= MAX_PATH_UNITS
            || units.contains(&0)
            || char::decode_utf16(units.iter().copied()).any(|c| c.is_err())
        {
            return Err(Error::Unsafe);
        }
        if !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(Error::Unsafe);
        }
        #[cfg(feature = "cpu")]
        {
            use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
            use windows_sys::Win32::System::WindowsProgramming::{
                DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOVABLE,
            };
            let drive = match path.components().next() {
                Some(Component::Prefix(p)) => match p.kind() {
                    Prefix::Disk(d) | Prefix::VerbatimDisk(d) => d,
                    _ => return Err(Error::Unsafe),
                },
                _ => return Err(Error::Unsafe),
            };
            let root = [u16::from(drive), b':' as u16, b'\\' as u16, 0];
            // SAFETY: exact bounded drive-root UTF16 buffer; reject mapped/remote/unknown drives.
            if !matches!(
                unsafe { GetDriveTypeW(root.as_ptr()) },
                DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_RAMDISK
            ) {
                return Err(Error::Unsafe);
            }
        }
        if path
            .components()
            .any(|c| matches!(c,Component::Normal(v) if v.to_string_lossy().contains(':')))
        {
            return Err(Error::Unsafe);
        }
    }
    #[cfg(not(windows))]
    {
        let Some(text) = path.to_str() else {
            return Err(Error::Unsafe);
        };
        if text.len() >= MAX_PATH_UNITS || text.contains('\0') {
            return Err(Error::Unsafe);
        }
    }
    Ok(())
}
fn pin(path: &Path, directory: bool) -> Result<File, Error> {
    safe(path, directory)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(all(windows, feature = "cpu"))]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        options
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(
                FILE_FLAG_OPEN_REPARSE_POINT
                    | if directory {
                        FILE_FLAG_BACKUP_SEMANTICS
                    } else {
                        0
                    },
            );
    }
    let file = options.open(path).map_err(|_| Error::Stale)?;
    safe(path, directory)?;
    if file.metadata().map_err(|_| Error::Stale)?.is_dir() != directory {
        return Err(Error::Unsafe);
    }
    Ok(file)
}
fn row_identity(file: &File) -> Result<String, Error> {
    match file_identity::from_file(file).map_err(|_| Error::Unsupported)? {
        NativeFileIdentity::Unix { device, inode } => Ok(format!("unix:{device}:{inode}")),
        NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } => Ok(format!(
            "windows:v1:{volume_serial}:{}",
            file_id
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )),
    }
}
fn observed(file: &File) -> Result<(u64, Option<i64>), Error> {
    let m = file.metadata().map_err(|_| Error::Stale)?;
    Ok((
        m.len(),
        m.modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|n| i64::try_from(n.as_millis()).ok()),
    ))
}
/// Pins live during the handoff; paths are checked again immediately before and after provider dispatch.
pub(crate) struct Validated {
    root: PathBuf,
    path: PathBuf,
    parent: PathBuf,
    root_pin: File,
    file_pin: File,
    parent_pin: File,
    row: LibraryEntry,
    kind: Kind,
}
impl Validated {
    fn current(&self) -> Result<(), Error> {
        let root = pin(&self.root, true)?;
        let file = pin(&self.path, false)?;
        let parent = pin(&self.parent, true)?;
        if file_identity::from_file(&root).map_err(|_| Error::Unsupported)?
            != file_identity::from_file(&self.root_pin).map_err(|_| Error::Unsupported)?
            || file_identity::from_file(&parent).map_err(|_| Error::Unsupported)?
                != file_identity::from_file(&self.parent_pin).map_err(|_| Error::Unsupported)?
            || row_identity(&file)? != row_identity(&self.file_pin)?
            || observed(&file)? != (self.row.size, self.row.modified_ms)
        {
            return Err(Error::Stale);
        }
        #[cfg(windows)]
        {
            file_identity::ensure_regular_under(&self.root_pin, &file)
                .map_err(|_| Error::Unsafe)?;
            if file_identity::final_path(&file).map_err(|_| Error::Unsafe)? != self.path {
                return Err(Error::Stale);
            }
        }
        if fs::canonicalize(&self.root).map_err(|_| Error::Stale)? != self.root
            || fs::canonicalize(&self.path).map_err(|_| Error::Stale)? != self.path
            || fs::canonicalize(&self.parent).map_err(|_| Error::Stale)? != self.parent
        {
            return Err(Error::Stale);
        }
        Ok(())
    }
    #[cfg(all(windows, feature = "cpu"))]
    fn handoff_paths(&self) -> Result<(PathBuf, PathBuf), Error> {
        #[cfg(all(windows, feature = "cpu"))]
        if self.kind == Kind::RevealFile {
            self.current()?;
            // Shell namespace parsers reject the verbatim spelling used by our file pins.
            // Changing spelling must never change either the file or its directory entry.
            let file = PathBuf::from(crate::path_picker::display_path(&self.path));
            let parent = PathBuf::from(crate::path_picker::display_path(&self.parent));
            let file_alias = pin(&file, false)?;
            let parent_alias = pin(&parent, true)?;
            if file_identity::from_file(&file_alias).map_err(|_| Error::Unsafe)?
                != file_identity::from_file(&self.file_pin).map_err(|_| Error::Unsafe)?
                || file_identity::from_file(&parent_alias).map_err(|_| Error::Unsafe)?
                    != file_identity::from_file(&self.parent_pin).map_err(|_| Error::Unsafe)?
                || fs::canonicalize(&file).map_err(|_| Error::Unsafe)? != self.path
                || fs::canonicalize(&parent).map_err(|_| Error::Unsafe)? != self.parent
            {
                return Err(Error::Unsafe);
            }
            return Ok((file, parent));
        }
        Ok((self.target().to_owned(), self.parent.clone()))
    }
    fn target(&self) -> &Path {
        if self.kind != Kind::Folder {
            &self.path
        } else {
            &self.parent
        }
    }
}
fn validate(
    root: &Path,
    row: &LibraryEntry,
    kind: Kind,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<Validated, Error> {
    checkpoint(stop, deadline)?;
    local_path(root)?;
    local_path(&row.path)?;
    // Check original spelling before canonicalize so normalization cannot hide an ancestor link.
    for (path, last_directory) in [(root, true), (row.path.as_path(), false)] {
        let mut at = PathBuf::new();
        let mut count = 0usize;
        for part in path.components() {
            at.push(part);
            if matches!(part, Component::Normal(_)) {
                count += 1;
                if count > 256 {
                    return Err(Error::Unsafe);
                }
                safe(&at, last_directory || at != path)?;
            }
        }
    }
    if kind != Kind::Folder && !supports_file(&row.path) {
        return Err(Error::Unsupported);
    }
    if row.file_identity.is_none() {
        return Err(Error::Stale);
    }
    let root = fs::canonicalize(root).map_err(|_| Error::Stale)?;
    let path = fs::canonicalize(&row.path).map_err(|_| Error::Stale)?;
    local_path(&root)?;
    local_path(&path)?;
    if !path.starts_with(&root) || path == root {
        return Err(Error::Unsafe);
    }
    // Every descendant component is checked; no reparse/symlink ancestor or ADS/protocol path.
    let mut at = root.clone();
    safe(&root, true)?;
    let relative = path.strip_prefix(&root).map_err(|_| Error::Unsafe)?;
    for component in relative.components() {
        at.push(component);
        safe(&at, at != path)?;
    }
    let parent = path.parent().ok_or(Error::Unsafe)?.to_owned();
    let root_pin = pin(&root, true)?;
    let parent_pin = pin(&parent, true)?;
    let file_pin = pin(&path, false)?;
    if Some(row_identity(&file_pin)?) != row.file_identity
        || observed(&file_pin)? != (row.size, row.modified_ms)
    {
        return Err(Error::Stale);
    }
    let validated = Validated {
        root,
        path,
        parent,
        root_pin,
        file_pin,
        parent_pin,
        row: row.clone(),
        kind,
    };
    validated.current()?;
    checkpoint(stop, deadline)?;
    Ok(validated)
}
/// Fake providers are injected into this exact worker path in tests; automated captures never run Native.
trait Provider {
    fn dispatch(
        &mut self,
        selection: &Validated,
        stop: &AtomicBool,
        deadline: Instant,
    ) -> Result<(), Error>;
}
fn dispatch_with(
    root: &Path,
    row: &LibraryEntry,
    kind: Kind,
    stop: &AtomicBool,
    deadline: Instant,
    provider: &mut dyn Provider,
) -> Result<Report, Error> {
    let verified = validate(root, row, kind, stop, deadline)?;
    verified.current()?;
    checkpoint(stop, deadline)?;
    provider.dispatch(&verified, stop, deadline)?;
    // A successful user handoff is irreversible here. Late cancel/deadline MUST NOT mean "didn't open".
    if verified.current().is_err() {
        return Ok(Report::SourceChanged);
    }
    if stop.load(Ordering::Acquire) {
        return Ok(Report::AfterCancellation);
    }
    if Instant::now() >= deadline {
        return Ok(Report::AfterDeadline);
    }
    Ok(Report::HandedOff)
}
pub(crate) fn dispatch(
    root: &Path,
    row: &LibraryEntry,
    kind: Kind,
    stop: &AtomicBool,
) -> Result<Report, Error> {
    let deadline = Instant::now()
        .checked_add(DISPATCH_BUDGET)
        .ok_or(Error::TimedOut)?;
    dispatch_with(root, row, kind, stop, deadline, &mut Native)
}
/// Retains the preview's original file pin while using the exact existing handoff validator.
/// Dropped media, library previews and locally prepared export files share this path.
pub(crate) fn dispatch_prepared(
    prepared: &dlpad_engine::media_preview::Prepared,
    kind: Kind,
    stop: &AtomicBool,
) -> Result<Report, Error> {
    let deadline = Instant::now()
        .checked_add(DISPATCH_BUDGET)
        .ok_or(Error::TimedOut)?;
    checkpoint(stop, deadline)?;
    let (path, file) = prepared.verified_local_file().map_err(|_| Error::Stale)?;
    let root = path.parent().ok_or(Error::Unsafe)?;
    let (size, modified_ms) = observed(file)?;
    let row = LibraryEntry {
        path: path.to_owned(),
        size,
        modified_ms,
        file_identity: Some(row_identity(file)?),
    };
    if kind == Kind::File {
        dispatch_default_with(root, &row, stop, deadline, &mut Native)
    } else {
        dispatch_with(root, &row, kind, stop, deadline, &mut Native)
    }
}
/// Exactly one fallback only after an observed no-association refusal. The original
/// Prepared is still retained by the caller; both attempts share a single deadline.
fn dispatch_default_with(
    root: &Path,
    row: &LibraryEntry,
    stop: &AtomicBool,
    deadline: Instant,
    provider: &mut impl Provider,
) -> Result<Report, Error> {
    match dispatch_with(root, row, Kind::File, stop, deadline, provider) {
        Err(Error::NoAssociation) => {
            checkpoint(stop, deadline)?;
            dispatch_with(root, row, Kind::ChooseApplication, stop, deadline, provider)
        }
        outcome => outcome,
    }
}

struct Native;
impl Provider for Native {
    fn dispatch(
        &mut self,
        selection: &Validated,
        stop: &AtomicBool,
        deadline: Instant,
    ) -> Result<(), Error> {
        if cfg!(test) || std::env::args_os().any(|a| a == "--fixture") {
            return Err(Error::AutomationBlocked);
        }
        #[cfg(all(windows, feature = "cpu"))]
        {
            native_dispatch(selection, stop, deadline)
        }
        #[cfg(not(all(windows, feature = "cpu")))]
        {
            let _ = (selection, stop, deadline);
            Err(Error::Unsupported)
        }
    }
}

// Job membership is observed only for an explicit worker request. Do not change the
// install supervisor or managed-tool Job policy to make an external viewer escape.
#[cfg(any(test, all(windows, feature = "cpu")))]
fn outside_job_handoff(
    query: impl FnOnce() -> Result<bool, Error>,
    handoff: impl FnOnce() -> Result<(), Error>,
) -> Result<(), Error> {
    if query()? {
        return Err(Error::Dispatch);
    }
    handoff()
}

// A chooser is a user-owned Explorer handoff even when this app is outside a Job.
// Never create an uncancellable application-owned SHOpenWithDialog modal loop.
#[cfg(any(test, all(windows, feature = "cpu")))]
fn broker_required(in_job: bool, kind: Kind) -> bool {
    in_job || matches!(kind, Kind::ChooseApplication | Kind::RevealFile)
}

#[cfg(all(windows, feature = "cpu"))]
fn current_process_in_any_job() -> Result<bool, Error> {
    use windows_sys::Win32::System::{JobObjects::IsProcessInJob, Threading::GetCurrentProcess};
    let mut in_job = 0;
    // SAFETY: current-process pseudo-handle is borrowed, NULL means any Job,
    // and the fixed initialized BOOL output lives through the synchronous call.
    if unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut in_job) } == 0 {
        return Err(Error::Dispatch);
    }
    Ok(in_job != 0)
}

#[cfg(all(windows, feature = "cpu"))]
fn native_dispatch(
    selection: &Validated,
    stop: &AtomicBool,
    deadline: Instant,
) -> Result<(), Error> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::{
        System::{
            Com::{
                COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
            },
            Threading::GetCurrentThreadId,
        },
        UI::{
            Shell::{SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW},
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    };
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetThreadDesktop(thread: u32) -> *mut c_void;
        fn GetUserObjectInformationW(
            handle: *mut c_void,
            index: i32,
            buffer: *mut c_void,
            length: u32,
            needed: *mut u32,
        ) -> i32;
    }
    let mut name = [0u16; 256];
    let mut needed = 0;
    // SAFETY: borrowed current thread desktop and fixed initialized output bounded to its byte size.
    if unsafe {
        GetUserObjectInformationW(
            GetThreadDesktop(GetCurrentThreadId()),
            2,
            name.as_mut_ptr().cast(),
            512,
            &mut needed,
        )
    } == 0
        || !(2..=512).contains(&needed)
    {
        return Err(Error::AutomationBlocked);
    }
    let Some(end) = name.iter().position(|u| *u == 0) else {
        return Err(Error::AutomationBlocked);
    };
    if String::from_utf16(&name[..end]).ok().as_deref() != Some("Default") {
        return Err(Error::AutomationBlocked);
    }
    checkpoint(stop, deadline)?;
    if broker_required(current_process_in_any_job()?, selection.kind) {
        selection.current()?;
        let (file, directory) = selection.handoff_paths()?;
        return crate::external_open_broker::dispatch(
            &file,
            &directory,
            selection.kind,
            stop,
            deadline,
        )
        .map_err(|error| match error {
            crate::external_open_broker::Error::Unavailable => Error::Unsupported,
            crate::external_open_broker::Error::Identity => Error::Unsafe,
            crate::external_open_broker::Error::AutomationBlocked => Error::AutomationBlocked,
            crate::external_open_broker::Error::Cancelled => Error::Cancelled,
            crate::external_open_broker::Error::TimedOut => Error::TimedOut,
            crate::external_open_broker::Error::Dispatch => Error::Dispatch,
            crate::external_open_broker::Error::NoAssociation => Error::NoAssociation,
        });
    }
    // SAFETY: initialize COM only on this existing worker, balance successful apartment init on Drop.
    if unsafe {
        CoInitializeEx(
            std::ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        )
    } < 0
    {
        return Err(Error::Dispatch);
    }
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() }
        }
    }
    let _apartment = Apartment;
    selection.current()?;
    checkpoint(stop, deadline)?;
    let file: Vec<u16> = selection
        .target()
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let directory: Vec<u16> = selection
        .parent
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let verb = [b'o' as u16, b'p' as u16, b'e' as u16, b'n' as u16, 0];
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
        lpVerb: verb.as_ptr(),
        lpFile: file.as_ptr(),
        lpDirectory: directory.as_ptr(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    // SAFETY: synchronous API invocation borrows initialized UTF16 buffers; no process handle requested.
    // Worker has no Windows message loop: finish DDE before balanced STA teardown.
    // NOASYNC can block: before/after deadline is not a hard wall and StopAck must wait.
    outside_job_handoff(current_process_in_any_job, || {
        if unsafe { ShellExecuteExW(&mut info) } == 0 {
            use windows_sys::Win32::Foundation::GetLastError;
            let error = unsafe { GetLastError() };
            return Err(if error == 1155 {
                Error::NoAssociation
            } else {
                Error::Dispatch
            });
        }
        Ok(())
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "dlpad-library-open-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn row(path: &Path) -> LibraryEntry {
        let file = pin(path, false).unwrap();
        let (size, modified_ms) = observed(&file).unwrap();
        LibraryEntry {
            path: path.to_owned(),
            size,
            modified_ms,
            file_identity: Some(row_identity(&file).unwrap()),
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn reveal_handoff_uses_shell_spelling_of_the_same_retained_file() {
        for long in [false, true] {
            let fixture = Fixture::new();
            let mut directory = fs::canonicalize(&fixture.0).unwrap();
            if long {
                for index in 0..6 {
                    directory.push(format!("{}-{index}", "가".repeat(40)));
                    fs::create_dir(&directory).unwrap();
                }
            }
            let path = directory.join("한글 영상 & literal.mp4");
            fs::write(&path, b"synthetic media").unwrap();
            let selected = validate(
                &fixture.0,
                &row(&path),
                Kind::RevealFile,
                &AtomicBool::new(false),
                Instant::now() + DISPATCH_BUDGET,
            )
            .unwrap();
            assert!(selected.path.to_str().unwrap().starts_with(r"\\?\"));
            let (shell_file, shell_parent) = selected.handoff_paths().unwrap();
            assert!(
                !shell_file.to_str().unwrap().starts_with(r"\\?\"),
                "actual native broker input must be Shell-compatible, not the canonical Win32 spelling"
            );
            assert_eq!(shell_file.parent(), Some(shell_parent.as_path()));
            assert_eq!(
                row_identity(&pin(&shell_file, false).unwrap()).unwrap(),
                row_identity(&selected.file_pin).unwrap()
            );
            assert_eq!(fs::canonicalize(&shell_file).unwrap(), selected.path);
            selected.current().unwrap();
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn reveal_handoff_refuses_a_shell_alias_that_names_a_different_directory_entry() {
        for ambiguous in ["ambiguous.", "ambiguous "] {
            let fixture = Fixture::new();
            let root = fs::canonicalize(&fixture.0).unwrap();
            let ordinary = root.join("ambiguous");
            let literal = root.join(ambiguous);
            fs::create_dir(&ordinary).unwrap();
            fs::create_dir(&literal).unwrap();
            fs::write(ordinary.join("video.mp4"), b"different file").unwrap();
            let path = literal.join("video.mp4");
            fs::write(&path, b"original file").unwrap();
            let selected = validate(
                &fixture.0,
                &row(&path),
                Kind::RevealFile,
                &AtomicBool::new(false),
                Instant::now() + DISPATCH_BUDGET,
            )
            .unwrap();
            assert!(
                matches!(selected.handoff_paths(), Err(Error::Unsafe | Error::Stale)),
                "a display spelling is not authority for a different Windows-normalized entry"
            );
            assert_eq!(fs::read(path).unwrap(), b"original file");
        }
    }
    struct Fake {
        calls: Vec<(PathBuf, Kind)>,
        error: Option<Error>,
        cancel_after: bool,
    }
    impl Provider for Fake {
        fn dispatch(
            &mut self,
            selected: &Validated,
            stop: &AtomicBool,
            _: Instant,
        ) -> Result<(), Error> {
            self.calls
                .push((selected.target().to_owned(), selected.kind));
            if self.cancel_after {
                stop.store(true, Ordering::Release)
            }
            self.error.map_or(Ok(()), Err)
        }
    }
    #[test]
    fn actual_original_identity_file_folder_and_late_cancel_handoff_are_distinct() {
        let f = Fixture::new();
        let path = f.0.join("한글 영상.mp4");
        fs::write(&path, b"original").unwrap();
        let original = row(&path);
        let stop = AtomicBool::new(false);
        let mut fake = Fake {
            calls: vec![],
            error: None,
            cancel_after: false,
        };
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Ok(Report::HandedOff)
        );
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::Folder,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Ok(Report::HandedOff)
        );
        assert_eq!(
            fake.calls[0],
            (fs::canonicalize(&path).unwrap(), Kind::File)
        );
        assert_eq!(
            fake.calls[1],
            (fs::canonicalize(&f.0).unwrap(), Kind::Folder)
        );
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::RevealFile,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Ok(Report::HandedOff)
        );
        assert_eq!(
            fake.calls[2],
            (fs::canonicalize(&path).unwrap(), Kind::RevealFile)
        );
        fake.cancel_after = true;
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Ok(Report::AfterCancellation)
        );
        assert_eq!(fs::read(path).unwrap(), b"original");
    }
    #[test]
    fn changed_original_outside_root_missing_identity_script_cancel_and_deadline_never_reach_provider()
     {
        let f = Fixture::new();
        let path = f.0.join("original.mp4");
        fs::write(&path, b"first").unwrap();
        let original = row(&path);
        let stop = AtomicBool::new(false);
        let mut fake = Fake {
            calls: vec![],
            error: None,
            cancel_after: false,
        };
        fs::write(&path, b"replacement longer").unwrap();
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Stale)
        );
        let fresh = row(&path);
        let outside = Fixture::new();
        assert_eq!(
            dispatch_with(
                &outside.0,
                &fresh,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Unsafe)
        );
        let mut no_identity = fresh.clone();
        no_identity.file_identity = None;
        assert_eq!(
            dispatch_with(
                &f.0,
                &no_identity,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Stale)
        );
        let script = f.0.join("bad.cmd");
        fs::write(&script, b"do not execute").unwrap();
        assert_eq!(
            dispatch_with(
                &f.0,
                &row(&script),
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Unsupported)
        );
        stop.store(true, Ordering::Release);
        assert_eq!(
            dispatch_with(
                &f.0,
                &fresh,
                Kind::File,
                &stop,
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Cancelled)
        );
        stop.store(false, Ordering::Release);
        assert_eq!(
            dispatch_with(&f.0, &fresh, Kind::File, &stop, Instant::now(), &mut fake),
            Err(Error::TimedOut)
        );
        assert!(fake.calls.is_empty());
    }
    #[test]
    fn reveal_changed_original_cancel_deadline_and_automation_never_open_explorer() {
        let f = Fixture::new();
        let path = f.0.join("한글 & literal.mp4");
        fs::write(&path, b"original").unwrap();
        let original = row(&path);
        let mut fake = Fake {
            calls: vec![],
            error: None,
            cancel_after: false,
        };
        for (stop, deadline, error) in [
            (true, Instant::now() + DISPATCH_BUDGET, Error::Cancelled),
            (false, Instant::now(), Error::TimedOut),
        ] {
            assert_eq!(
                dispatch_with(
                    &f.0,
                    &original,
                    Kind::RevealFile,
                    &AtomicBool::new(stop),
                    deadline,
                    &mut fake
                ),
                Err(error)
            );
        }
        assert_eq!(
            dispatch(&f.0, &original, Kind::RevealFile, &AtomicBool::new(false)),
            Err(Error::AutomationBlocked)
        );
        fs::write(&path, b"changed original with different size").unwrap();
        assert_eq!(
            dispatch_with(
                &f.0,
                &original,
                Kind::RevealFile,
                &AtomicBool::new(false),
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Stale)
        );
        assert!(fake.calls.is_empty());
    }
    #[test]
    fn chooser_always_hands_off_to_explorer_and_never_uses_an_app_modal_loop() {
        assert!(broker_required(false, Kind::ChooseApplication));
        assert!(broker_required(true, Kind::ChooseApplication));
        assert!(broker_required(false, Kind::RevealFile));
        assert!(broker_required(true, Kind::RevealFile));
        for kind in [Kind::File, Kind::Folder] {
            assert!(broker_required(true, kind));
            assert!(!broker_required(false, kind));
        }
    }
    #[test]
    fn detail_no_association_fallback_is_exact_single_and_preserves_cancel_deadline() {
        struct Sequence {
            first: Option<Error>,
            calls: Vec<Kind>,
            cancel: bool,
        }
        impl Provider for Sequence {
            fn dispatch(
                &mut self,
                selected: &Validated,
                stop: &AtomicBool,
                _: Instant,
            ) -> Result<(), Error> {
                self.calls.push(selected.kind);
                if self.calls.len() == 1 {
                    if self.cancel {
                        stop.store(true, Ordering::Release);
                    }
                    if let Some(error) = self.first {
                        return Err(error);
                    }
                }
                Ok(())
            }
        }
        let fixture = Fixture::new();
        let path = fixture.0.join("synthetic.mp4");
        fs::write(&path, b"original").unwrap();
        let original = row(&path);
        let deadline = Instant::now() + DISPATCH_BUDGET;
        for error in [
            None,
            Some(Error::NoAssociation),
            Some(Error::Dispatch),
            Some(Error::Unsafe),
            Some(Error::AutomationBlocked),
            Some(Error::TimedOut),
        ] {
            let mut provider = Sequence {
                first: error,
                calls: vec![],
                cancel: false,
            };
            let result = dispatch_default_with(
                &fixture.0,
                &original,
                &AtomicBool::new(false),
                deadline,
                &mut provider,
            );
            assert_eq!(
                provider.calls,
                if error == Some(Error::NoAssociation) {
                    vec![Kind::File, Kind::ChooseApplication]
                } else {
                    vec![Kind::File]
                }
            );
            assert_eq!(
                result,
                match error {
                    None | Some(Error::NoAssociation) => Ok(Report::HandedOff),
                    Some(e) => Err(e),
                }
            );
        }
        let mut provider = Sequence {
            first: Some(Error::NoAssociation),
            calls: vec![],
            cancel: true,
        };
        assert_eq!(
            dispatch_default_with(
                &fixture.0,
                &original,
                &AtomicBool::new(false),
                deadline,
                &mut provider
            ),
            Err(Error::Cancelled)
        );
        assert_eq!(provider.calls, vec![Kind::File]);
        let mut provider = Sequence {
            first: Some(Error::NoAssociation),
            calls: vec![],
            cancel: false,
        };
        assert_eq!(
            dispatch_default_with(
                &fixture.0,
                &original,
                &AtomicBool::new(false),
                Instant::now(),
                &mut provider
            ),
            Err(Error::TimedOut)
        );
        assert!(provider.calls.is_empty());
        assert_eq!(fs::read(path).unwrap(), b"original");
    }
    #[test]
    fn missing_association_is_a_failure_not_file_deletion_or_preview() {
        let f = Fixture::new();
        let p = f.0.join("original.mp4");
        fs::write(&p, b"preserve").unwrap();
        let mut fake = Fake {
            calls: vec![],
            error: Some(Error::NoAssociation),
            cancel_after: false,
        };
        assert_eq!(
            dispatch_with(
                &f.0,
                &row(&p),
                Kind::File,
                &AtomicBool::new(false),
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::NoAssociation)
        );
        assert_eq!(fake.calls.len(), 1);
        assert_eq!(fs::read(p).unwrap(), b"preserve");
    }
    #[test]
    fn replacement_same_metadata_is_rejected_by_actual_native_identity_and_automation_never_runs_shell()
     {
        let f = Fixture::new();
        let source = f.0.join("original.mp4");
        let preserved = f.0.join("preserved.mp4");
        fs::write(&source, b"same bytes").unwrap();
        let mut old = row(&source);
        fs::rename(&source, &preserved).unwrap();
        fs::write(&source, b"same bytes").unwrap();
        let replacement = row(&source);
        old.modified_ms = replacement.modified_ms;
        old.size = replacement.size;
        assert_ne!(old.file_identity, replacement.file_identity);
        let mut fake = Fake {
            calls: vec![],
            error: None,
            cancel_after: false,
        };
        assert_eq!(
            dispatch_with(
                &f.0,
                &old,
                Kind::File,
                &AtomicBool::new(false),
                Instant::now() + DISPATCH_BUDGET,
                &mut fake
            ),
            Err(Error::Stale)
        );
        assert!(fake.calls.is_empty());
        assert_eq!(
            dispatch(&f.0, &replacement, Kind::File, &AtomicBool::new(false)),
            Err(Error::AutomationBlocked)
        );
        assert_eq!(fs::read(source).unwrap(), b"same bytes");
        assert_eq!(fs::read(preserved).unwrap(), b"same bytes");
    }
}

#[cfg(test)]
mod external_job_boundary_tests {
    use super::*;

    #[test]
    fn uncertain_or_present_job_never_enters_handoff_and_no_job_preserves_result() {
        let mut calls = 0;
        for query in [Ok(true), Err(Error::Dispatch)] {
            let result = outside_job_handoff(
                || query,
                || {
                    calls += 1;
                    Ok(())
                },
            );
            assert_eq!(result, Err(Error::Dispatch));
            assert_eq!(calls, 0, "query denial cannot reach the external provider");
        }
        let result = outside_job_handoff(
            || Ok(false),
            || {
                calls += 1;
                Err(Error::NoAssociation)
            },
        );
        assert_eq!(calls, 1);
        assert_eq!(
            result,
            Err(Error::NoAssociation),
            "original provider failure is not replaced by success"
        );
    }

    // Invoked only as the original suspended/assigned child by the fixture below.
    // No Shell call, association lookup, desktop/input operation or external viewer.
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    #[ignore = "owned subprocess probe; run only through owned_kill_on_close_job_denies_handoff"]
    fn owned_job_probe() {
        assert!(
            current_process_in_any_job().unwrap(),
            "actual child Job membership"
        );
        let mut calls = 0;
        assert_eq!(
            outside_job_handoff(current_process_in_any_job, || {
                calls += 1;
                Ok(())
            }),
            Err(Error::Dispatch)
        );
        assert_eq!(calls, 0, "actual Job membership suppresses fake handoff");
        // Distinct natural-exit ACK proves the exact probe executed; a zero-test
        // harness success must not masquerade as a verified child observation.
        std::process::exit(73);
    }

    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    #[ignore = "creates only an owned synthetic test process; Root explicit native fixture"]
    fn owned_kill_on_close_job_denies_handoff() {
        use std::{
            os::windows::{
                ffi::OsStrExt,
                io::{AsRawHandle, FromRawHandle, OwnedHandle},
            },
            ptr,
        };
        use windows_sys::Win32::{
            Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
            System::{JobObjects::*, Threading::*},
        };
        fn own(h: windows_sys::Win32::Foundation::HANDLE) -> OwnedHandle {
            assert!(!h.is_null(), "actual owned HANDLE");
            // SAFETY: fresh successful Win32 return; exactly one owner adopts it.
            unsafe { OwnedHandle::from_raw_handle(h) }
        }
        let job = own(unsafe { CreateJobObjectW(ptr::null(), ptr::null()) });
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        assert_ne!(
            unsafe {
                SetInformationJobObject(
                    job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    std::mem::size_of_val(&info) as u32,
                )
            },
            0
        );
        let executable = std::env::current_exe().unwrap();
        assert!(executable.is_absolute());
        assert!(
            !executable
                .as_os_str()
                .encode_wide()
                .any(|u| matches!(u, 0 | 34))
        );
        let application: Vec<u16> = executable
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut command: Vec<u16> = format!("\"{}\" --exact library_open::external_job_boundary_tests::owned_job_probe --ignored --nocapture",executable.display()).encode_utf16().chain(Some(0)).collect();
        assert!(command.len() <= 32767);
        let startup = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        // SAFETY: exact own test executable/fixed test argv, no inherited handles,
        // suspended/no-window original child. Existing ambient private desktop unchanged.
        assert_ne!(
            unsafe {
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
            },
            0
        );
        struct Child {
            process: OwnedHandle,
            thread: OwnedHandle,
            job: OwnedHandle,
            bound: bool,
            terminal: bool,
        }
        impl Drop for Child {
            fn drop(&mut self) {
                if !self.terminal {
                    // SAFETY: only this fixture's original owned Job/process, never a PID lookup.
                    unsafe {
                        if self.bound {
                            TerminateJobObject(self.job.as_raw_handle(), 126);
                        } else {
                            TerminateProcess(self.process.as_raw_handle(), 126);
                        }
                        WaitForSingleObject(self.process.as_raw_handle(), 5000);
                    }
                }
            }
        }
        let mut child = Child {
            process: own(pi.hProcess),
            thread: own(pi.hThread),
            job,
            bound: false,
            terminal: false,
        };
        assert_ne!(
            unsafe {
                AssignProcessToJobObject(child.job.as_raw_handle(), child.process.as_raw_handle())
            },
            0
        );
        child.bound = true;
        let mut present = 0;
        assert_ne!(
            unsafe {
                IsProcessInJob(
                    child.process.as_raw_handle(),
                    child.job.as_raw_handle(),
                    &mut present,
                )
            },
            0
        );
        assert_ne!(
            present, 0,
            "child belongs to exactly the original owned fixture Job before resume"
        );
        let resume = unsafe { ResumeThread(child.thread.as_raw_handle()) };
        assert_eq!(resume, 1, "single original suspension is released");
        let state = unsafe { WaitForSingleObject(child.process.as_raw_handle(), 15000) };
        assert_ne!(
            state, WAIT_TIMEOUT,
            "bounded fixture deadline; Drop terminates owned fixture only on failure"
        );
        assert_eq!(state, WAIT_OBJECT_0, "actual child terminal");
        child.terminal = true;
        let mut code = 0;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle(), &mut code) },
            0
        );
        assert_eq!(
            code, 73,
            "exact child probe ACK: observed Job and declined fake provider"
        );
        let mut counts = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        assert_ne!(
            unsafe {
                QueryInformationJobObject(
                    child.job.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    (&mut counts as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    std::mem::size_of_val(&counts) as u32,
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(
            counts.TotalProcesses, 1,
            "no external descendant was created"
        );
        assert_eq!(
            counts.ActiveProcesses, 0,
            "original fixture Job naturally empty"
        );
    }
}
