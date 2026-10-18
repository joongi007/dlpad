//! Original-handle Windows implementation; no shell, breakaway or detached child.
use super::{FRAME_BYTES, Frame, ImageRole, Kind, PROFILE_EPOCH, ParentEvent, UiRequest, rejected};
use crate::{
    app_install::{PreparedUpdateBundle, UserBundleInspection, inspect_user_bundle},
    app_update::activation_journal::InstalledImageEvidence,
    file_identity::{self, NativeFileIdentity},
};
use sha2::{Digest, Sha256};
use std::{
    ffi::{OsStr, OsString},
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::windows::{
        ffi::OsStrExt,
        fs::{FileExt, OpenOptionsExt},
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::Cryptography::*,
    Storage::FileSystem::*,
    System::{JobObjects::*, Pipes::*, Threading::*},
};
const MARKER: &str = "--dlpad-internal-owned-ui-v1";
const CLEANUP: Duration = Duration::from_secs(5);
const IMAGE_LIMIT: u64 = 128 * 1024 * 1024;
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
fn owned(raw: HANDLE) -> io::Result<OwnedHandle> {
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful native constructor transfers exactly one owned HANDLE.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}
fn creation(raw: HANDLE) -> io::Result<u64> {
    let (mut c, mut e, mut k, mut u) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: live process HANDLE; correctly sized output structures.
    if unsafe { GetProcessTimes(raw, &mut c, &mut e, &mut k, &mut u) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64)
}
fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut out: Vec<u16> = value.encode_wide().collect();
    if out.len() > 32760 || out.contains(&0) || out.contains(&34) {
        return Err(rejected());
    }
    out.push(0);
    Ok(out)
}
fn attributes(file: &File) -> io::Result<FILE_ATTRIBUTE_TAG_INFO> {
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: original live file HANDLE and matching output buffer.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(info)
}
struct NamespacePin {
    file: File,
    identity: NativeFileIdentity,
    path: PathBuf,
}
impl NamespacePin {
    fn verify(&self) -> io::Result<()> {
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != self.path
            || attributes(&self.file)?.FileAttributes
                & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY)
                != FILE_ATTRIBUTE_DIRECTORY
        {
            return Err(rejected());
        }
        Ok(())
    }
}
struct Image {
    file: File,
    identity: NativeFileIdentity,
    path: PathBuf,
    bytes: u64,
    sha256: [u8; 32],
    namespace: Vec<NamespacePin>,
    weak: bool,
}
impl Image {
    fn pin(path: &Path, cancel: &AtomicBool, deadline: Instant, weak: bool) -> io::Result<Self> {
        check(cancel, deadline)?;
        let file = OpenOptions::new()
            .read(true)
            .share_mode(if weak {
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            } else {
                FILE_SHARE_READ
            })
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if attributes(&file)?.FileAttributes
            & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
            != 0
        {
            return Err(rejected());
        }
        let path = file_identity::final_path(&file)?;
        let mut namespace = Vec::new();
        for ancestor in path.parent().ok_or_else(rejected)?.ancestors() {
            check(cancel, deadline)?;
            if namespace.len() >= 256 {
                return Err(rejected());
            }
            let f = OpenOptions::new()
                .read(true)
                .share_mode(
                    FILE_SHARE_READ | FILE_SHARE_WRITE | if weak { FILE_SHARE_DELETE } else { 0 },
                )
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(ancestor)?;
            let pin = NamespacePin {
                identity: file_identity::from_file(&f)?,
                path: file_identity::final_path(&f)?,
                file: f,
            };
            pin.verify()?;
            namespace.push(pin);
        }
        let identity = file_identity::from_file(&file)?;
        let bytes = file.metadata()?.len();
        let sha256 = hash(&file, bytes, cancel, deadline)?;
        let image = Self {
            file,
            identity,
            path,
            bytes,
            sha256,
            namespace,
            weak,
        };
        image.verify(cancel, deadline)?;
        Ok(image)
    }
    fn verify(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        check(cancel, deadline)?;
        for pin in &self.namespace {
            check(cancel, deadline)?;
            pin.verify()?;
        }
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != self.path
            || self.file.metadata()?.len() != self.bytes
            || hash(&self.file, self.bytes, cancel, deadline)? != self.sha256
        {
            return Err(rejected());
        }
        check(cancel, deadline)
    }
    // The regular source is reopened from its original HANDLE. Directory
    // ReOpenFile fails on Windows, while OpenFileById does not protect named links.
    // Retain every weak original and acquire named deny-DELETE pins root-to-leaf;
    // adopt each new pin only after full original ID/path/attribute equality.
    fn tighten(&self) -> io::Result<Self> {
        // SAFETY: original retained regular file and exact readonly access/share policy.
        let raw = unsafe {
            ReOpenFile(
                self.file.as_raw_handle(),
                GENERIC_READ,
                FILE_SHARE_READ,
                FILE_FLAG_OPEN_REPARSE_POINT,
            )
        };
        let file = File::from(owned(raw)?);
        let mut namespace = Vec::new();
        for pin in self.namespace.iter().rev() {
            pin.verify()?;
            let file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&pin.path)?;
            let strong = NamespacePin {
                file,
                identity: pin.identity.clone(),
                path: pin.path.clone(),
            };
            strong.verify()?;
            pin.verify()?;
            namespace.push(strong);
        }
        if file_identity::from_file(&file)? != self.identity
            || file_identity::final_path(&file)? != self.path
        {
            return Err(rejected());
        }
        Ok(Self {
            file,
            identity: self.identity.clone(),
            path: self.path.clone(),
            bytes: self.bytes,
            sha256: self.sha256,
            namespace,
            weak: false,
        })
    }
    fn duplicate(&self) -> io::Result<Self> {
        if self.weak {
            return self.tighten();
        }
        let mut namespace = Vec::new();
        for pin in &self.namespace {
            namespace.push(NamespacePin {
                file: pin.file.try_clone()?,
                identity: pin.identity.clone(),
                path: pin.path.clone(),
            });
        }
        Ok(Self {
            file: self.file.try_clone()?,
            identity: self.identity.clone(),
            path: self.path.clone(),
            bytes: self.bytes,
            sha256: self.sha256,
            namespace,
            weak: false,
        })
    }
}
fn hash(file: &File, bytes: u64, cancel: &AtomicBool, deadline: Instant) -> io::Result<[u8; 32]> {
    if !(64..=IMAGE_LIMIT).contains(&bytes) {
        return Err(rejected());
    }
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    let mut offset = 0;
    while offset < bytes {
        check(cancel, deadline)?;
        let size = (bytes - offset).min(buffer.len() as u64) as usize;
        let n = file.seek_read(&mut buffer[..size], offset)?;
        if n == 0 {
            return Err(rejected());
        }
        digest.update(&buffer[..n]);
        offset += n as u64;
    }
    let mut tail = [0];
    if file.seek_read(&mut tail, bytes)? != 0 {
        return Err(rejected());
    }
    Ok(digest.finalize().into())
}
/// Initial authority is exactly this process image, never a supplied path or record.
pub struct SelfImage(Image);
impl SelfImage {
    /// Observation only; this identity is not a path-based launch constructor.
    pub fn identity(&self) -> &NativeFileIdentity {
        &self.0.identity
    }
    pub fn pin_current(cancel: &AtomicBool, deadline: Instant) -> io::Result<Self> {
        Ok(Self(Image::pin(
            &std::env::current_exe()?,
            cancel,
            deadline,
            false,
        )?))
    }
}
/// Original weak readonly pins from a verified fresh B bundle. The fresh bundle must
/// be dropped before spawn can tighten these pins into execution authority.
pub struct PreparedImage(Image);
impl PreparedImage {
    pub fn pin_prepared(
        bundle: &PreparedUpdateBundle,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<Self> {
        bundle.verify_current(cancel, deadline)?;
        let registration = bundle.installation().bundle().registration();
        let expected = &bundle
            .installation()
            .bundle()
            .active()
            .record()
            .selected_executable;
        let cpu = bundle
            .observation()
            .members()
            .iter()
            .find(|m| m.name() == "payload/dlpad.exe")
            .ok_or_else(rejected)?;
        let image = Image::pin(
            &registration.path().join("dlpad.exe"),
            cancel,
            deadline,
            true,
        )?;
        if image.identity != expected.identity
            || image.bytes != expected.bytes
            || image.bytes != cpu.bytes()
            || image.sha256 != *cpu.sha256()
        {
            return Err(rejected());
        }
        // The prepared manifest hash is checked by the bundle; compare its executable too.
        let hex: String = image.sha256.iter().map(|b| format!("{b:02x}")).collect();
        if hex != expected.sha256 {
            return Err(rejected());
        }
        bundle.verify_current(cancel, deadline)?;
        Ok(Self(image))
    }
}
/// Revalidated original image and bundle pins from an authenticated typed journal observation.
/// The journal loader owns signing-key/time/sequence policy; records alone never launch.
pub struct InstalledImage {
    image: Image,
    bundle: UserBundleInspection,
}
impl InstalledImage {
    pub fn pin_committed(
        evidence: &InstalledImageEvidence,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<Self> {
        check(cancel, deadline)?;
        if evidence.profile_epoch() != PROFILE_EPOCH {
            return Err(rejected());
        }
        let bundle = inspect_user_bundle(evidence.bundle_root(), cancel, deadline)?;
        let record = bundle.selection().record();
        if <[u8; 32]>::from(Sha256::digest(record.canonical_bytes()?))
            != *evidence.active_record_sha256()
            || Path::new(&record.active_path) != evidence.active_root()
            || Path::new(&record.registered_path) != evidence.registered_root()
            || Path::new(&record.receipt_path) != evidence.receipt_root()
            || record.selected_executable.identity != *evidence.executable_identity()
            || record.selected_executable.bytes != evidence.executable_bytes()
            || evidence.registered_root().join("dlpad.exe") != evidence.executable_path()
        {
            return Err(rejected());
        }
        let image = Image::pin(evidence.executable_path(), cancel, deadline, false)?;
        if image.identity != *evidence.executable_identity()
            || image.bytes != evidence.executable_bytes()
            || image.sha256 != *evidence.executable_sha256()
            || image.path != evidence.executable_path()
        {
            return Err(rejected());
        }
        let hex: String = image.sha256.iter().map(|b| format!("{b:02x}")).collect();
        if hex != record.selected_executable.sha256 {
            return Err(rejected());
        }
        bundle.verify_current(cancel, deadline)?;
        image.verify(cancel, deadline)?;
        Ok(Self { image, bundle })
    }
}
pub fn spawn_installed_ui(
    image: &InstalledImage,
    profile: &NativeFileIdentity,
    epoch: u32,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<OwnedUi, SpawnFailure> {
    image
        .bundle
        .verify_current(cancel, deadline)
        .map_err(|error| SpawnFailure { error, child: None })?;
    let mut child = spawn(
        &image.image,
        profile,
        epoch,
        ImageRole::Installed,
        cancel,
        deadline,
    )?;
    if let Err(error) = image.bundle.verify_current(cancel, deadline) {
        let _ = child.terminate_and_wait(Instant::now() + CLEANUP);
        return Err(SpawnFailure {
            error,
            child: Some(Box::new(child)),
        });
    }
    Ok(child)
}
fn job() -> io::Result<OwnedHandle> {
    // SAFETY: fresh unnamed noninherited Job.
    let job = owned(unsafe { CreateJobObjectW(ptr::null(), ptr::null()) })?;
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: owned Job and correct native class/structure.
    if unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(job)
}
fn active(job: &OwnedHandle) -> io::Result<u32> {
    let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
    // SAFETY: original retained Job and matching writable output.
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
        return Err(io::Error::last_os_error());
    }
    Ok(info.ActiveProcesses)
}
fn pipe() -> io::Result<(File, File)> {
    let (mut read, mut write) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: writable outputs, default noninheritable security.
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 4096) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let read = File::from(owned(read)?);
    let write = File::from(owned(write)?);
    let mode = PIPE_NOWAIT;
    // SAFETY: owned anonymous pipe writer, bounded nonblocking protocol writes.
    if unsafe { SetNamedPipeHandleState(write.as_raw_handle(), &mode, ptr::null(), ptr::null()) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut actual = 0;
    // SAFETY: original pipe writer and writable state output; anonymous pipes are supported.
    if unsafe {
        GetNamedPipeHandleStateW(
            write.as_raw_handle(),
            &mut actual,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        )
    } == 0
        || actual & PIPE_NOWAIT == 0
    {
        return Err(rejected());
    }
    Ok((read, write))
}
fn inherit(raw: HANDLE, access: u32) -> io::Result<OwnedHandle> {
    let mut copy = ptr::null_mut();
    // SAFETY: original handles remain live; exact rights or SAME_ACCESS, fresh inheritable duplicate.
    if unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            raw,
            GetCurrentProcess(),
            &mut copy,
            access,
            1,
            if access == 0 {
                DUPLICATE_SAME_ACCESS
            } else {
                0
            },
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    owned(copy)
}
struct Attributes {
    storage: Vec<usize>,
    initialized: bool,
}
impl Attributes {
    fn new(handles: &[HANDLE], jobs: &[HANDLE]) -> io::Result<Self> {
        let mut size = 0;
        // SAFETY: query allocation size only.
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut size);
        }
        if size == 0 || size > 65536 {
            return Err(rejected());
        }
        let mut this = Self {
            storage: vec![0; size.div_ceil(std::mem::size_of::<usize>())],
            initialized: false,
        };
        // SAFETY: sized pointer-aligned backing allocation retained through process creation.
        if unsafe { InitializeProcThreadAttributeList(this.pointer(), 2, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        this.initialized = true;
        for (attribute, values) in [
            (PROC_THREAD_ATTRIBUTE_HANDLE_LIST, handles),
            (PROC_THREAD_ATTRIBUTE_JOB_LIST, jobs),
        ] {
            // SAFETY: original allowlist arrays outlive CreateProcess and native attribute list.
            if unsafe {
                UpdateProcThreadAttribute(
                    this.pointer(),
                    0,
                    attribute as usize,
                    values.as_ptr().cast(),
                    std::mem::size_of_val(values),
                    ptr::null_mut(),
                    ptr::null(),
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(this)
    }
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: initialized original allocation remains live.
            unsafe {
                DeleteProcThreadAttributeList(self.pointer());
            }
        }
    }
}
fn write_before(
    file: &mut File,
    bytes: &[u8],
    peer: HANDLE,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        check(cancel, deadline)?;
        alive(peer)?;
        match file.write(&bytes[offset..]) {
            Ok(0) => {}
            Ok(n) => offset += n,
            Err(e) if matches!(e.raw_os_error(),Some(code)if code==ERROR_NO_DATA as i32 || code==ERROR_PIPE_BUSY as i32) =>
                {}
            Err(e) => return Err(e),
        }
        if offset < bytes.len() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}
fn alive(peer: HANDLE) -> io::Result<()> {
    // SAFETY: original live process HANDLE, finite kernel wait.
    if unsafe { WaitForSingleObject(peer, 0) } != WAIT_TIMEOUT {
        return Err(io::ErrorKind::BrokenPipe.into());
    }
    Ok(())
}
fn available(file: &File) -> io::Result<usize> {
    let mut count = 0;
    // SAFETY: original retained pipe reader and live output.
    if unsafe {
        PeekNamedPipe(
            file.as_raw_handle(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            &mut count,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(count as usize)
}
fn read_before(
    file: &mut File,
    peer: HANDLE,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<Frame> {
    let mut bytes = [0; FRAME_BYTES];
    let mut offset = 0;
    while offset < bytes.len() {
        check(cancel, deadline)?;
        alive(peer)?;
        let n = available(file)?.min(bytes.len() - offset);
        if n == 0 {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let n = file.read(&mut bytes[offset..offset + n])?;
        if n == 0 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        offset += n;
    }
    Frame::decode(&bytes)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Spawned,
    Preflight,
    Committed,
    RuntimeReady,
    Failed,
    Terminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiExit {
    pub code: u32,
    pub job_terminal_confirmed: bool,
    /// The leader had already exited while owned descendants remained; only that Job was terminated.
    pub descendants_terminated: bool,
}
/// Original image/process/Job/namespace capabilities survive all wait failures.
pub struct OwnedUi {
    _image: Image,
    process: OwnedHandle,
    thread: Option<OwnedHandle>,
    job: OwnedHandle,
    control: File,
    events: File,
    frame: Frame,
    state: State,
    exit: Option<UiExit>,
    pending: Pending,
    leader_cleanup: Option<Instant>,
}
impl std::fmt::Debug for OwnedUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedUi")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
pub struct SpawnFailure {
    error: io::Error,
    child: Option<Box<OwnedUi>>,
}
impl SpawnFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn retained_child(&mut self) -> Option<&mut OwnedUi> {
        self.child.as_deref_mut()
    }
    pub fn terminal_confirmed(&self) -> bool {
        self.child.as_ref().is_none_or(|c| c.exit.is_some())
    }
}
impl std::fmt::Debug for SpawnFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnFailure")
            .field("kind", &self.kind())
            .field("terminal_confirmed", &self.terminal_confirmed())
            .finish()
    }
}
impl std::fmt::Display for SpawnFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "owned UI startup failed ({:?})", self.kind())
    }
}
impl std::error::Error for SpawnFailure {}
pub fn spawn_ui(
    image: &SelfImage,
    profile: &NativeFileIdentity,
    epoch: u32,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<OwnedUi, SpawnFailure> {
    spawn(
        &image.0,
        profile,
        epoch,
        ImageRole::SelfOriginal,
        cancel,
        deadline,
    )
}
pub fn spawn_prepared_ui(
    image: &PreparedImage,
    profile: &NativeFileIdentity,
    epoch: u32,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<OwnedUi, SpawnFailure> {
    spawn(
        &image.0,
        profile,
        epoch,
        ImageRole::Prepared,
        cancel,
        deadline,
    )
}
fn spawn(
    image: &Image,
    profile: &NativeFileIdentity,
    epoch: u32,
    role: ImageRole,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<OwnedUi, SpawnFailure> {
    let mut retained = None;
    let result = (|| -> io::Result<()> {
        image.verify(cancel, deadline)?;
        let NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } = profile
        else {
            return Err(rejected());
        };
        if epoch != PROFILE_EPOCH || *file_id == [0; 16] {
            return Err(rejected());
        }
        let image = image.duplicate()?;
        image.verify(cancel, deadline)?;
        let job = job()?;
        let (control_reader, control) = pipe()?;
        let (events, events_writer) = pipe()?;
        let caps = [
            inherit(control_reader.as_raw_handle(), 0)?,
            inherit(events_writer.as_raw_handle(), 0)?,
            inherit(
                unsafe { GetCurrentProcess() },
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE,
            )?,
        ];
        let handles = caps
            .iter()
            .map(AsRawHandle::as_raw_handle)
            .collect::<Vec<_>>();
        let jobs = [job.as_raw_handle()];
        let mut attrs = Attributes::new(&handles, &jobs)?;
        let application = wide(image.path.as_os_str())?;
        let suffix = format!(
            "\" {MARKER} {} {} {}",
            handles[0] as usize, handles[1] as usize, handles[2] as usize
        );
        let mut command = vec![34u16];
        command.extend_from_slice(&application[..application.len() - 1]);
        command.extend(suffix.encode_utf16());
        command.push(0);
        if command.len() > 32767 {
            return Err(rejected());
        }
        let mut startup = STARTUPINFOEXW::default();
        startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attrs.pointer();
        let mut pi = PROCESS_INFORMATION::default();
        // SAFETY: exact pinned image and fixed argv; only original three capabilities inherited.
        // Atomic Job attribute covers owner death even before suspended-child adoption.
        if unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                1,
                CREATE_SUSPENDED | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
                ptr::null(),
                ptr::null(),
                &startup.StartupInfo,
                &mut pi,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // Successful CreateProcess guarantees both original handles; adopt before any fallible call.
        let process = unsafe { OwnedHandle::from_raw_handle(pi.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(pi.hThread) };
        let frame = Frame {
            kind: Kind::Hello,
            role,
            epoch,
            parent: unsafe { GetCurrentProcessId() },
            parent_creation: 0,
            child: pi.dwProcessId,
            child_creation: 0,
            volume: *volume_serial,
            file_id: *file_id,
            nonce: [0; 32],
        };
        retained = Some(OwnedUi {
            _image: image,
            process,
            thread: Some(thread),
            job,
            control,
            events,
            frame,
            state: State::Spawned,
            exit: None,
            pending: Pending::default(),
            leader_cleanup: None,
        });
        drop(caps);
        drop(control_reader);
        drop(events_writer);
        let child = retained.as_mut().unwrap();
        child.frame.parent_creation = creation(unsafe { GetCurrentProcess() })?;
        child.frame.child_creation = creation(child.process.as_raw_handle())?;
        // SAFETY: OS RNG writes exactly the fixed nonce buffer.
        if unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                child.frame.nonce.as_mut_ptr(),
                32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
        {
            return Err(rejected());
        }
        let mut in_job = 0;
        // SAFETY: original process and original owned Job; live boolean output.
        if unsafe {
            IsProcessInJob(
                child.process.as_raw_handle(),
                child.job.as_raw_handle(),
                &mut in_job,
            )
        } == 0
            || in_job == 0
        {
            return Err(rejected());
        }
        verify_process_image(child)?;
        child._image.verify(cancel, deadline)?;
        check(cancel, deadline)?;
        // SAFETY: original suspended primary thread remains owned until this one resume.
        if unsafe { ResumeThread(child.thread.as_ref().unwrap().as_raw_handle()) } != 1 {
            return Err(rejected());
        }
        child.thread.take();
        write_before(
            &mut child.control,
            &child.frame.encode(),
            child.process.as_raw_handle(),
            cancel,
            deadline,
        )?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(retained.unwrap()),
        Err(error) => {
            if let Some(child) = retained.as_mut() {
                let _ = child.terminate_and_wait(Instant::now() + CLEANUP);
            }
            Err(SpawnFailure {
                error,
                child: retained.map(Box::new),
            })
        }
    }
}
fn verify_process_image(child: &OwnedUi) -> io::Result<()> {
    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    // SAFETY: original process and bounded writable UTF16 output.
    if unsafe {
        QueryFullProcessImageNameW(
            child.process.as_raw_handle(),
            0,
            buffer.as_mut_ptr(),
            &mut size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    use std::os::windows::ffi::OsStringExt;
    let path = PathBuf::from(OsString::from_wide(&buffer[..size as usize]));
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if file_identity::from_file(&file)? != child._image.identity
        || file_identity::final_path(&file)? != child._image.path
    {
        return Err(rejected());
    }
    Ok(())
}
impl OwnedUi {
    pub fn process_id(&self) -> u32 {
        self.frame.child
    }
    pub fn creation_filetime(&self) -> u64 {
        self.frame.child_creation
    }
    fn receive(
        &mut self,
        from: State,
        to: State,
        kind: Kind,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<()> {
        if self.state != from {
            return Err(rejected());
        }
        self.state = State::Failed;
        let frame = read_before(
            &mut self.events,
            self.process.as_raw_handle(),
            cancel,
            deadline,
        )?;
        self.frame.expect(frame, kind)?;
        self.state = to;
        Ok(())
    }
    pub fn wait_preflight(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        self.receive(
            State::Spawned,
            State::Preflight,
            Kind::Ready,
            cancel,
            deadline,
        )
    }
    pub fn commit(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        if self.state != State::Preflight {
            return Err(rejected());
        }
        self.state = State::Failed;
        let mut frame = self.frame.clone();
        frame.kind = Kind::Commit;
        write_before(
            &mut self.control,
            &frame.encode(),
            self.process.as_raw_handle(),
            cancel,
            deadline,
        )?;
        self.state = State::Committed;
        Ok(())
    }
    pub fn wait_runtime_ready(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        self.receive(
            State::Committed,
            State::RuntimeReady,
            Kind::RuntimeReady,
            cancel,
            deadline,
        )
    }
    pub fn poll_exit(&mut self) -> io::Result<Option<UiExit>> {
        if let Some(exit) = self.exit {
            return Ok(Some(exit));
        }
        if creation(self.process.as_raw_handle())? != self.frame.child_creation {
            return Err(rejected());
        }
        // SAFETY: original process and finite zero-duration wait.
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => return Ok(None),
            WAIT_OBJECT_0 => {}
            _ => return Err(io::Error::last_os_error()),
        }
        if active(&self.job)? != 0 {
            // A live UI is never subject to a runtime timeout. Once its original
            // leader is signaled, however, its owned descendants cannot be abandoned.
            if let Some(started) = self.leader_cleanup {
                if started.elapsed() >= CLEANUP {
                    self.state = State::Failed;
                    return Err(io::ErrorKind::TimedOut.into());
                }
            } else {
                // SAFETY: only the original Job after original leader terminal observation.
                if unsafe { TerminateJobObject(self.job.as_raw_handle(), 126) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                self.leader_cleanup = Some(Instant::now());
            }
            return Ok(None);
        }
        let mut code = 0;
        // SAFETY: signaled original process and writable exit code.
        if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let exit = UiExit {
            code,
            job_terminal_confirmed: true,
            descendants_terminated: self.leader_cleanup.is_some(),
        };
        self.exit = Some(exit);
        self.state = State::Terminal;
        Ok(Some(exit))
    }
    pub fn terminate_and_wait(&mut self, deadline: Instant) -> io::Result<UiExit> {
        if let Some(exit) = self.poll_exit()? {
            return Ok(exit);
        }
        // SAFETY: original owned Job only; no PID lookup or outside process mutation.
        if unsafe { TerminateJobObject(self.job.as_raw_handle(), 126) } == 0 {
            return Err(io::Error::last_os_error());
        }
        loop {
            if let Some(exit) = self.poll_exit()? {
                return Ok(exit);
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for OwnedUi {
    fn drop(&mut self) {
        if self.exit.is_none() {
            // SAFETY: original Job best-effort fallback only, never a terminal proof.
            unsafe {
                TerminateJobObject(self.job.as_raw_handle(), 126);
            }
        }
    }
}
/// Internal mode never accepts a path, environment command or restore IPC request.
pub struct UiChild {
    control: File,
    events: File,
    parent: OwnedHandle,
    frame: Frame,
    state: State,
    pending: Pending,
}
impl UiChild {
    /// Available immediately after authenticated Hello, before any profile I/O.
    pub fn requires_profile_preflight(&self) -> bool {
        self.frame.role != ImageRole::SelfOriginal
    }
    pub fn await_commit(
        &mut self,
        profile: &NativeFileIdentity,
        epoch: u32,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<()> {
        if self.state != State::Spawned || !self.frame.profile_matches(profile, epoch) {
            return Err(rejected());
        }
        self.state = State::Failed;
        let mut frame = self.frame.clone();
        frame.kind = Kind::Ready;
        write_before(
            &mut self.events,
            &frame.encode(),
            self.parent.as_raw_handle(),
            cancel,
            deadline,
        )?;
        self.frame.expect(
            read_before(
                &mut self.control,
                self.parent.as_raw_handle(),
                cancel,
                deadline,
            )?,
            Kind::Commit,
        )?;
        self.state = State::Committed;
        Ok(())
    }
    pub fn runtime_ready(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        if self.state != State::Committed {
            return Err(rejected());
        }
        self.state = State::Failed;
        let mut frame = self.frame.clone();
        frame.kind = Kind::RuntimeReady;
        write_before(
            &mut self.events,
            &frame.encode(),
            self.parent.as_raw_handle(),
            cancel,
            deadline,
        )?;
        self.state = State::RuntimeReady;
        Ok(())
    }
}
fn inherited_handle(text: &OsStr, kind: u32) -> io::Result<HANDLE> {
    let text = text.to_str().ok_or_else(rejected)?;
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(rejected());
    }
    let raw = text.parse::<usize>().map_err(|_| rejected())? as HANDLE;
    let mut flags = 0;
    // SAFETY: queries borrowed process-local handle; no ownership taken until all checks pass.
    if raw.is_null()
        || raw == INVALID_HANDLE_VALUE
        || unsafe { GetHandleInformation(raw, &mut flags) } == 0
        || flags & HANDLE_FLAG_INHERIT == 0
        || (kind != 0 && unsafe { GetFileType(raw) } != kind)
    {
        return Err(rejected());
    }
    Ok(raw)
}
pub fn child_entry() -> io::Result<Option<UiChild>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).take(6).collect();
    if arguments.first().is_none_or(|a| a != MARKER) {
        return Ok(None);
    }
    if arguments.len() != 4 {
        return Err(rejected());
    }
    let handles = [
        inherited_handle(&arguments[1], FILE_TYPE_PIPE)?,
        inherited_handle(&arguments[2], FILE_TYPE_PIPE)?,
        inherited_handle(&arguments[3], 0)?,
    ];
    if handles[0] == handles[1] || handles[0] == handles[2] || handles[1] == handles[2] {
        return Err(rejected());
    }
    for raw in handles {
        // SAFETY: exact validated inherited capabilities; exclude from all later children.
        if unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
    }
    // SAFETY: distinct original inherited handles transfer into one RAII owner each.
    let (mut control, events, parent) = unsafe {
        (
            File::from_raw_handle(handles[0]),
            File::from_raw_handle(handles[1]),
            OwnedHandle::from_raw_handle(handles[2]),
        )
    };
    let cancel = AtomicBool::new(false);
    let frame = read_before(
        &mut control,
        parent.as_raw_handle(),
        &cancel,
        Instant::now() + Duration::from_secs(20),
    )?;
    // SAFETY: original retained parent process and current process pseudo handle.
    if frame.kind != Kind::Hello
        || frame.parent != unsafe { GetProcessId(parent.as_raw_handle()) }
        || frame.parent_creation != creation(parent.as_raw_handle())?
        || frame.child != unsafe { GetCurrentProcessId() }
        || frame.child_creation != creation(unsafe { GetCurrentProcess() })?
    {
        return Err(rejected());
    }
    Ok(Some(UiChild {
        control,
        events,
        parent,
        frame,
        state: State::Spawned,
        pending: Pending::default(),
    }))
}

#[derive(Default)]
struct Pending {
    bytes: Vec<u8>,
    started: Option<Instant>,
}
impl Pending {
    fn read(&mut self, file: &mut File, peer: HANDLE) -> io::Result<Option<[u8; FRAME_BYTES]>> {
        alive(peer)?;
        if self
            .started
            .is_some_and(|start| start.elapsed() >= Duration::from_secs(20))
        {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let count = available(file)?.min(FRAME_BYTES - self.bytes.len());
        if count == 0 {
            return Ok(None);
        }
        self.started.get_or_insert_with(Instant::now);
        let mut buffer = [0; FRAME_BYTES];
        let count = file.read(&mut buffer[..count])?;
        if count == 0 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        self.bytes.extend_from_slice(&buffer[..count]);
        if self.bytes.len() != FRAME_BYTES {
            return Ok(None);
        }
        let mut result = [0; FRAME_BYTES];
        result.copy_from_slice(&self.bytes);
        self.bytes.clear();
        self.started = None;
        Ok(Some(result))
    }
}
impl OwnedUi {
    pub fn try_request(&mut self) -> io::Result<Option<UiRequest>> {
        if self.state != State::RuntimeReady {
            return Err(rejected());
        }
        let result = self
            .pending
            .read(&mut self.events, self.process.as_raw_handle())
            .and_then(|frame| {
                frame
                    .map(|frame| UiRequest::decode(&frame, &self.frame.nonce))
                    .transpose()
            });
        if result.is_err() {
            self.state = State::Failed;
        }
        result
    }
    pub fn send_event(
        &mut self,
        event: &ParentEvent,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<()> {
        if self.state != State::RuntimeReady {
            return Err(rejected());
        }
        let frame = event.encode(&self.frame.nonce)?;
        let result = write_before(
            &mut self.control,
            &frame,
            self.process.as_raw_handle(),
            cancel,
            deadline,
        );
        if result.is_err() {
            self.state = State::Failed;
        }
        result
    }
}
impl UiChild {
    pub fn send_request(
        &mut self,
        request: &UiRequest,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<()> {
        if self.state != State::RuntimeReady {
            return Err(rejected());
        }
        let frame = request.encode(&self.frame.nonce)?;
        let result = write_before(
            &mut self.events,
            &frame,
            self.parent.as_raw_handle(),
            cancel,
            deadline,
        );
        if result.is_err() {
            self.state = State::Failed;
        }
        result
    }
    pub fn try_event(&mut self) -> io::Result<Option<ParentEvent>> {
        if self.state != State::RuntimeReady {
            return Err(rejected());
        }
        let result = self
            .pending
            .read(&mut self.control, self.parent.as_raw_handle())
            .and_then(|frame| {
                frame
                    .map(|frame| ParentEvent::decode(&frame, &self.frame.nonce))
                    .transpose()
            });
        if result.is_err() {
            self.state = State::Failed;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_pipe_writer_obeys_deadline_and_cancel() {
        let (_reader, mut writer) = pipe().unwrap();
        let buffer = [1u8; 4096];
        let mut full = false;
        for _ in 0..4096 {
            match writer.write(&buffer) {
                Ok(0) => {
                    full = true;
                    break;
                }
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(ERROR_NO_DATA as i32) => {
                    full = true;
                    break;
                }
                Err(e) => panic!("pipe fill: {e}"),
            }
        }
        assert!(
            full,
            "bounded pipe must eventually refuse bytes without blocking"
        );
        let start = Instant::now();
        let cancel = AtomicBool::new(false);
        // SAFETY: live current process pseudo-handle, borrowed only.
        let peer = unsafe { GetCurrentProcess() };
        assert_eq!(
            write_before(
                &mut writer,
                &[2; 128],
                peer,
                &cancel,
                start + Duration::from_millis(50)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        cancel.store(true, Ordering::Release);
        assert_eq!(
            write_before(
                &mut writer,
                &[2; 128],
                peer,
                &cancel,
                Instant::now() + Duration::from_secs(1)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Interrupted
        );
    }
    #[test]
    fn empty_pipe_reader_and_partial_runtime_frame_are_bounded() {
        let (mut reader, mut writer) = pipe().unwrap();
        let cancel = AtomicBool::new(false);
        // SAFETY: live current process pseudo-handle, borrowed only.
        let peer = unsafe { GetCurrentProcess() };
        let start = Instant::now();
        assert_eq!(
            read_before(
                &mut reader,
                peer,
                &cancel,
                start + Duration::from_millis(30)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        writer.write_all(&[1]).unwrap();
        let mut pending = Pending::default();
        assert!(pending.read(&mut reader, peer).unwrap().is_none());
        assert_eq!(pending.bytes.len(), 1);
        pending.started = Some(Instant::now() - Duration::from_secs(21));
        assert_eq!(
            pending.read(&mut reader, peer).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }
    #[test]
    fn current_image_cancel_does_not_start_a_process() {
        let cancel = AtomicBool::new(true);
        assert!(
            matches!(SelfImage::pin_current(&cancel,Instant::now()+Duration::from_secs(1)),Err(e)if e.kind()==io::ErrorKind::Interrupted)
        );
    }
}

#[cfg(test)]
mod sharing_tests {
    use super::*;
    #[test]
    fn named_directory_pin_blocks_rename_without_any_open_descendant_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        let moved = temp.path().join("moved");
        std::fs::create_dir(&path).unwrap();
        let weak = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
            .unwrap();
        let identity = file_identity::from_file(&weak).unwrap();
        let final_path = file_identity::final_path(&weak).unwrap();
        let file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&final_path)
            .unwrap();
        let named = NamespacePin {
            file,
            identity: identity.clone(),
            path: final_path.clone(),
        };
        named.verify().unwrap();
        assert!(std::fs::rename(&path, &moved).is_err());
        drop(named);
        std::fs::rename(&path, &moved).unwrap();
        assert_eq!(file_identity::from_file(&weak).unwrap(), identity);
        assert_ne!(file_identity::final_path(&weak).unwrap(), final_path);
    }

    #[test]
    fn weak_prepared_pins_require_writer_and_directory_delete_caps_to_close() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.exe");
        std::fs::write(&path, [3u8; 256]).unwrap();
        let writer = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)
            .unwrap();
        let delete_dir = OpenOptions::new()
            .access_mode(DELETE | FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(dir.path())
            .unwrap();
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        let image = Image::pin(&path, &cancel, deadline, true).unwrap();
        assert!(image.tighten().is_err());
        drop(writer);
        assert!(image.tighten().is_err());
        drop(delete_dir);
        let sealed = image.tighten().unwrap();
        sealed.verify(&cancel, deadline).unwrap();
        assert!(OpenOptions::new().write(true).open(&path).is_err());
    }
    #[test]
    fn modified_weak_original_is_rejected_after_tightening_without_path_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.exe");
        std::fs::write(&path, [3u8; 256]).unwrap();
        let mut writer = OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)
            .unwrap();
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        let image = Image::pin(&path, &cancel, deadline, true).unwrap();
        writer.write_all(&[4; 256]).unwrap();
        writer.sync_all().unwrap();
        drop(writer);
        let sealed = image.tighten().unwrap();
        assert!(sealed.verify(&cancel, deadline).is_err());
    }
}
