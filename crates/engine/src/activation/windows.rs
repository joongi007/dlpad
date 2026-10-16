//! Local, same-user, nonce-authenticated restore-only Windows IPC.
use super::{ActivationError, RestoreSignal};
use crate::file_identity::{self, NativeFileIdentity};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, PathBuf},
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, Cryptography::*, *},
    Storage::FileSystem::*,
    System::{IO::*, Pipes::*, Threading::*},
};
const MARKER: &[u8] = b"dlpad-user-data-v1\n";
const PREFIX: &[u8] = b"DLPAD_RESTORE_V1";
const ACK: &[u8] = b"DLPAD_RESTORE_QUEUED_V1\n";
const RECEIPT: &[u8] = b"DLPAD_RESTORE_RECEIVED_V1\n";
const TOKEN_BYTES: usize = 32;
const LIMIT: usize = 64;
const IO_TIMEOUT: Duration = Duration::from_millis(250);

fn owned(handle: HANDLE) -> Result<OwnedHandle, ActivationError> {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error().into());
    }
    // SAFETY: fresh valid handle is transferred once.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}
struct Local(*mut std::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn sid_string(sid: PSID) -> Result<String, ActivationError> {
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(ActivationError::UnsafeData);
    }
    let mut text = ptr::null_mut();
    // SAFETY: validated SID and writable pointer; Windows allocates string.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let _text = Local(text.cast());
    let mut length = 0;
    // Windows SID strings have a small documented finite layout; impose own cap.
    while length < 256 && unsafe { *text.add(length) } != 0 {
        length += 1;
    }
    if length == 256 {
        return Err(ActivationError::UnsafeData);
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(|_| ActivationError::UnsafeData)
}
fn process_sid(handle: HANDLE) -> Result<String, ActivationError> {
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let token = owned(raw)?;
    let mut length = 0;
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            ptr::null_mut(),
            0,
            &mut length,
        );
    }
    if length == 0 || length > 4096 {
        return Err(ActivationError::UnsafeData);
    }
    let mut storage = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            length,
            &mut length,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    // SAFETY: TokenUser output is pointer-aligned and SID is live in storage.
    sid_string(unsafe { (*storage.as_ptr().cast::<TOKEN_USER>()).User.Sid })
}
fn current_sid() -> Result<String, ActivationError> {
    process_sid(unsafe { GetCurrentProcess() })
}
struct Descriptor(Local);
impl Descriptor {
    fn private(sid: &str) -> Result<Self, ActivationError> {
        let text: Vec<u16> = format!("O:{sid}D:P(A;;GA;;;{sid})")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error().into());
        }
        Ok(Self(Local(descriptor)))
    }
    fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.0,
            bInheritHandle: 0,
        }
    }
}
fn acl(file: &File, sid: &str, strict: bool) -> Result<(), ActivationError> {
    let mut owner = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut descriptor = ptr::null_mut();
    let error = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if error != 0 {
        return Err(io::Error::from_raw_os_error(error as i32).into());
    }
    let descriptor = Local(descriptor);
    if sid_string(owner)? != sid || dacl.is_null() {
        return Err(ActivationError::UnsafeData);
    }
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let mut info = ACL_SIZE_INFORMATION::default();
    if unsafe {
        GetAclInformation(
            dacl,
            (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
            AclSizeInformation,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    if info.AceCount > 128 || (strict && (info.AceCount != 1 || control & SE_DACL_PROTECTED == 0)) {
        return Err(ActivationError::UnsafeData);
    }
    let mut user_allowed = false;
    for index in 0..info.AceCount {
        let mut ace = ptr::null_mut();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            return Err(io::Error::last_os_error().into());
        }
        let header = unsafe { &*ace.cast::<ACE_HEADER>() };
        // Reject unrecognized object/callback ACEs instead of approximating rights.
        if header.AceType != 0
            || (header.AceSize as usize) < std::mem::size_of::<ACCESS_ALLOWED_ACE>()
        {
            return Err(ActivationError::UnsafeData);
        }
        let entry = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
        let principal = sid_string((&entry.SidStart as *const u32).cast_mut().cast())?;
        if principal == sid {
            user_allowed = true;
        } else if strict
            || (principal != "S-1-5-18"
                && principal != "S-1-5-32-544"
                && entry.Mask
                    & (0x40000000
                        | 0x10000000
                        | FILE_WRITE_DATA
                        | FILE_APPEND_DATA
                        | FILE_WRITE_EA
                        | FILE_WRITE_ATTRIBUTES
                        | FILE_DELETE_CHILD
                        | DELETE
                        | WRITE_DAC
                        | WRITE_OWNER)
                    != 0)
        {
            return Err(ActivationError::UnsafeData);
        }
    }
    if !user_allowed {
        return Err(ActivationError::UnsafeData);
    }
    Ok(())
}
fn open_checked(path: &Path, directory: bool, pin: bool) -> Result<File, ActivationError> {
    let file = OpenOptions::new()
        .read(true)
        .access_mode(FILE_READ_DATA | FILE_READ_ATTRIBUTES | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | if pin { 0 } else { FILE_SHARE_DELETE })
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    if info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
    {
        return Err(ActivationError::UnsafeData);
    }
    Ok(file)
}
struct Root {
    path: PathBuf,
    identity: NativeFileIdentity,
    _pin: File,
}
fn root(selected: &Path, sid: &str) -> Result<Root, ActivationError> {
    if !selected.is_absolute()
        || selected
            .components()
            .any(|p| matches!(p, Component::ParentDir))
    {
        return Err(ActivationError::UnsafeData);
    }
    let pin = open_checked(selected, true, true)?;
    acl(&pin, sid, false)?;
    let identity = file_identity::from_file(&pin)?;
    let path = file_identity::final_path(&pin)?;
    let marker = open_checked(&path.join(".dlpad-data-owner"), false, false)?;
    acl(&marker, sid, false)?;
    if marker.metadata()?.len() != MARKER.len() as u64 {
        return Err(ActivationError::UnsafeData);
    }
    let mut bytes = [0; MARKER.len()];
    (&marker).read_exact(&mut bytes)?;
    if bytes != MARKER {
        return Err(ActivationError::UnsafeData);
    }
    Ok(Root {
        path,
        identity,
        _pin: pin,
    })
}
fn pipe_name(root: &Root, sid: &str) -> Vec<u16> {
    let mut hash = Sha256::new();
    hash.update(sid.as_bytes());
    hash.update([0]);
    for unit in root.path.as_os_str().encode_wide() {
        hash.update(unit.to_le_bytes());
    }
    format!(r"\\.\pipe\dlpad-restore-v1-{:x}", hash.finalize())
        .encode_utf16()
        .chain([0])
        .collect()
}
fn request(token: &[u8; TOKEN_BYTES]) -> [u8; 49] {
    let mut message = [0; 49];
    message[..PREFIX.len()].copy_from_slice(PREFIX);
    message[PREFIX.len()..48].copy_from_slice(token);
    message[48] = b'\n';
    message
}
fn token_read(
    path: &Path,
    sid: &str,
) -> Result<([u8; TOKEN_BYTES], NativeFileIdentity), ActivationError> {
    let mut file = open_checked(path, false, false)?;
    acl(&file, sid, true)?;
    if file.metadata()?.len() != TOKEN_BYTES as u64 {
        return Err(ActivationError::UnsafeData);
    }
    let identity = file_identity::from_file(&file)?;
    let mut bytes = [0; TOKEN_BYTES];
    file.read_exact(&mut bytes)?;
    if !bytes
        .iter()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
    {
        return Err(ActivationError::UnsafeData);
    }
    Ok((bytes, identity))
}
// Failure cleanup never deletes an unknown/replaced identity. As with final cleanup,
// this is a checked path operation, not a claim of atomic ancestor TOCTOU exclusion.
fn remove_provisional(root_pin: &Root, path: &Path, sid: &str, identity: &NativeFileIdentity) {
    if root(&root_pin.path, sid).is_ok_and(|root| root.identity == root_pin.identity)
        && open_checked(path, false, false).is_ok_and(|file| {
            acl(&file, sid, true).is_ok()
                && file_identity::from_file(&file).is_ok_and(|observed| &observed == identity)
        })
    {
        let _ = fs::remove_file(path);
    }
}
struct Token {
    root: Arc<Root>,
    path: PathBuf,
    sid: String,
    bytes: [u8; TOKEN_BYTES],
    identity: NativeFileIdentity,
}
impl Token {
    fn create(root: Arc<Root>, sid: &str, security: &Descriptor) -> Result<Self, ActivationError> {
        let mut random = [0; 16];
        if unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                random.as_mut_ptr(),
                16,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
        {
            return Err(io::Error::other("secure activation nonce unavailable").into());
        }
        let mut bytes = [0; TOKEN_BYTES];
        let hex = b"0123456789abcdef";
        for (index, byte) in random.into_iter().enumerate() {
            bytes[index * 2] = hex[(byte >> 4) as usize];
            bytes[index * 2 + 1] = hex[(byte & 15) as usize];
        }
        let path = root.path.join("activation.token");
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let attrs = security.attributes();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ,
                &attrs,
                CREATE_NEW,
                FILE_FLAG_OPEN_REPARSE_POINT,
                ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::AlreadyExists {
                return Err(ActivationError::ExistingEndpoint);
            }
            return Err(error.into());
        }
        let mut file = File::from(owned(handle)?);
        // Establish native ownership before writes. Unknown identity preserves the file.
        let identity = file_identity::from_file(&file)?;
        let result = (|| {
            file.write_all(&bytes)?;
            file.sync_all()?;
            acl(&file, sid, true)?;
            Ok::<(), ActivationError>(())
        })();
        drop(file);
        if let Err(error) = result {
            remove_provisional(&root, &path, sid, &identity);
            return Err(error);
        }
        Ok(Self {
            root,
            path,
            sid: sid.into(),
            bytes,
            identity,
        })
    }
    fn matches(&self) -> bool {
        root(&self.root.path, &self.sid).is_ok_and(|r| r.identity == self.root.identity)
            && token_read(&self.path, &self.sid)
                .is_ok_and(|(bytes, id)| bytes == self.bytes && id == self.identity)
    }
}
impl Drop for Token {
    fn drop(&mut self) {
        if self.matches() {
            let _ = fs::remove_file(&self.path);
        }
    }
}
struct State {
    restore: RestoreSignal,
    stop: AtomicBool,
    stop_event: OwnedHandle,
}
struct Pending<'a> {
    handle: &'a OwnedHandle,
    event: OwnedHandle,
    ov: Box<OVERLAPPED>,
    active: bool,
    #[cfg(test)]
    waits: Arc<std::sync::atomic::AtomicUsize>,
}
impl<'a> Pending<'a> {
    fn new(handle: &'a OwnedHandle) -> Result<Self, ActivationError> {
        let event = owned(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) })?;
        let mut ov = Box::<OVERLAPPED>::default();
        ov.hEvent = event.as_raw_handle();
        Ok(Self {
            handle,
            event,
            ov,
            active: false,
            #[cfg(test)]
            waits: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }
    fn finish_until_stop(
        &mut self,
        deadline: Instant,
        stop: &AtomicBool,
        stop_event: &OwnedHandle,
    ) -> Result<u32, ActivationError> {
        // Both original events stay alive through the wait and pending-I/O drain.
        // Index zero gives cancellation priority when completion races with stop.
        let handles = [stop_event.as_raw_handle(), self.event.as_raw_handle()];
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if stop.load(Ordering::Acquire) || remaining.is_zero() {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }
            let timeout = remaining
                .as_nanos()
                .div_ceil(1_000_000)
                .min((INFINITE - 1) as u128) as u32;
            #[cfg(test)]
            self.waits.fetch_add(1, Ordering::AcqRel);
            match unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, timeout) } {
                WAIT_OBJECT_0 => return Err(io::Error::from(io::ErrorKind::TimedOut).into()),
                value if value == WAIT_OBJECT_0 + 1 => {
                    if stop.load(Ordering::Acquire) {
                        return Err(io::Error::from(io::ErrorKind::TimedOut).into());
                    }
                    let mut bytes = 0;
                    let result = unsafe {
                        GetOverlappedResult(self.handle.as_raw_handle(), &*self.ov, &mut bytes, 0)
                    };
                    self.active = false;
                    if result == 0 {
                        return Err(io::Error::last_os_error().into());
                    }
                    return Ok(bytes);
                }
                WAIT_TIMEOUT => {}
                _ => return Err(io::Error::last_os_error().into()),
            }
        }
    }
    fn finish(&mut self, deadline: Instant, stop: &AtomicBool) -> Result<u32, ActivationError> {
        loop {
            if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }
            #[cfg(test)]
            self.waits.fetch_add(1, Ordering::AcqRel);
            match unsafe { WaitForSingleObject(self.event.as_raw_handle(), 20) } {
                WAIT_OBJECT_0 => {
                    let mut bytes = 0;
                    let result = unsafe {
                        GetOverlappedResult(self.handle.as_raw_handle(), &*self.ov, &mut bytes, 0)
                    };
                    self.active = false;
                    if result == 0 {
                        return Err(io::Error::last_os_error().into());
                    }
                    return Ok(bytes);
                }
                WAIT_TIMEOUT => {}
                _ => return Err(io::Error::last_os_error().into()),
            }
        }
    }
}
impl Drop for Pending<'_> {
    fn drop(&mut self) {
        if self.active {
            unsafe {
                CancelIoEx(self.handle.as_raw_handle(), &*self.ov);
                let mut bytes = 0;
                GetOverlappedResult(self.handle.as_raw_handle(), &*self.ov, &mut bytes, 1);
            }
        }
    }
}
fn transfer(
    handle: &OwnedHandle,
    buffer: &mut [u8],
    write: bool,
    deadline: Instant,
    stop: &AtomicBool,
) -> Result<usize, ActivationError> {
    let mut pending = Pending::new(handle)?;
    let mut count = 0;
    let result = unsafe {
        if write {
            WriteFile(
                handle.as_raw_handle(),
                buffer.as_ptr(),
                buffer.len() as u32,
                &mut count,
                &mut *pending.ov,
            )
        } else {
            ReadFile(
                handle.as_raw_handle(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                &mut count,
                &mut *pending.ov,
            )
        }
    };
    if result != 0 {
        return Ok(count as usize);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
        return Err(error.into());
    }
    pending.active = true;
    pending.finish(deadline, stop).map(|bytes| bytes as usize)
}
fn peer(handle: &OwnedHandle, sid: &str, client: bool) -> Result<(), ActivationError> {
    let mut pid = 0;
    let ok = unsafe {
        if client {
            GetNamedPipeClientProcessId(handle.as_raw_handle(), &mut pid)
        } else {
            GetNamedPipeServerProcessId(handle.as_raw_handle(), &mut pid)
        }
    };
    if ok == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let process = owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) })?;
    if process_sid(process.as_raw_handle())? != sid {
        return Err(ActivationError::UnsafeData);
    }
    Ok(())
}
fn create_pipe(name: &[u16], security: &Descriptor) -> Result<OwnedHandle, ActivationError> {
    let attrs = security.attributes();
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            LIMIT as u32,
            LIMIT as u32,
            250,
            &attrs,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let error = io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(5 | 231)) {
            return Err(ActivationError::ExistingEndpoint);
        }
        return Err(error.into());
    }
    owned(handle)
}
pub struct ActivationServer {
    token: Token,
    state: Arc<State>,
    pipe: Arc<OwnedHandle>,
    worker: Option<JoinHandle<()>>,
}
impl ActivationServer {
    /// InstanceGuard must outlive this server. Never starts or controls a job.
    pub fn bind(data: &Path) -> Result<Self, ActivationError> {
        let sid = current_sid()?;
        let root = Arc::new(root(data, &sid)?);
        let name = pipe_name(&root, &sid);
        let security = Descriptor::private(&sid)?;
        // Claim exclusive pipe first: a foreign server never causes creation of our token.
        let pipe = Arc::new(create_pipe(&name, &security)?);
        let token = Token::create(root, &sid, &security)?;
        let state = Arc::new(State {
            restore: RestoreSignal::default(),
            stop: AtomicBool::new(false),
            stop_event: owned(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) })?,
        });
        let owned = pipe.clone();
        let signals = state.clone();
        let nonce = token.bytes;
        let worker = thread::Builder::new()
            .name("dlpad-restore".into())
            .spawn(move || {
                while !signals.stop.load(Ordering::Acquire) {
                    let Ok(mut pending) = Pending::new(&owned) else {
                        break;
                    };
                    let ready =
                        unsafe { ConnectNamedPipe(owned.as_raw_handle(), &mut *pending.ov) };
                    let connected = if ready != 0 {
                        true
                    } else {
                        let error = io::Error::last_os_error();
                        if error.raw_os_error() == Some(ERROR_PIPE_CONNECTED as i32) {
                            true
                        } else if error.raw_os_error() == Some(ERROR_IO_PENDING as i32) {
                            pending.active = true;
                            pending
                                .finish_until_stop(
                                    Instant::now() + Duration::from_secs(3600),
                                    &signals.stop,
                                    &signals.stop_event,
                                )
                                .is_ok()
                        } else {
                            false
                        }
                    };
                    drop(pending);
                    if signals.stop.load(Ordering::Acquire) {
                        break;
                    }
                    if connected {
                        let deadline = Instant::now() + IO_TIMEOUT;
                        let mut message = [0; LIMIT];
                        if peer(&owned, &sid, true).is_ok()
                            && transfer(&owned, &mut message, false, deadline, &signals.stop)
                                .is_ok_and(|count| message[..count] == request(&nonce))
                            && !signals.stop.load(Ordering::Acquire)
                        {
                            signals.restore.notify();
                            if transfer(&owned, &mut ACK.to_vec(), true, deadline, &signals.stop)
                                .is_ok()
                            {
                                // DisconnectNamedPipe discards unread output. A bounded receipt
                                // proves the client consumed ACK without unbounded FlushFileBuffers.
                                let mut receipt = [0; LIMIT];
                                let _ =
                                    transfer(&owned, &mut receipt, false, deadline, &signals.stop);
                            }
                        }
                    }
                    unsafe {
                        DisconnectNamedPipe(owned.as_raw_handle());
                    }
                    if !connected {
                        thread::sleep(Duration::from_millis(20));
                    }
                }
            })?;
        Ok(Self {
            token,
            state,
            pipe,
            worker: Some(worker),
        })
    }
    /// Caller supplies a nonblocking UI wake hint; pending requests are retained.
    pub fn set_wake_callback(&self, callback: impl Fn() + Send + Sync + 'static) {
        self.state.restore.set(callback);
    }
    pub fn poll_restore(&self) -> bool {
        self.state.restore.poll()
    }
}
impl Drop for ActivationServer {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        unsafe {
            SetEvent(self.state.stop_event.as_raw_handle());
            CancelIoEx(self.pipe.as_raw_handle(), ptr::null());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        // Token guard removes only matching nonce identity. Pipe disappears with owned handles.
        let _ = &self.token;
    }
}
fn connect(name: &[u16]) -> Result<OwnedHandle, ActivationError> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let raw = unsafe {
            CreateFileW(
                name.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                0,
                ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED | SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                ptr::null_mut(),
            )
        };
        if raw != INVALID_HANDLE_VALUE {
            let handle = owned(raw)?;
            let mode = PIPE_READMODE_MESSAGE;
            if unsafe {
                SetNamedPipeHandleState(handle.as_raw_handle(), &mode, ptr::null(), ptr::null())
            } == 0
            {
                return Err(io::Error::last_os_error().into());
            }
            return Ok(handle);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_PIPE_BUSY as i32) {
            return Err(error.into());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::from(io::ErrorKind::TimedOut).into());
        }
        unsafe {
            WaitNamedPipeW(name.as_ptr(), 20);
        }
    }
}
pub fn send_restore(data: &Path) -> Result<(), ActivationError> {
    let sid = current_sid()?;
    let root = root(data, &sid)?;
    let (token, _) = token_read(&root.path.join("activation.token"), &sid)?;
    let pipe = connect(&pipe_name(&root, &sid))?;
    peer(&pipe, &sid, false)?;
    let stop = AtomicBool::new(false);
    let deadline = Instant::now() + IO_TIMEOUT;
    let mut message = request(&token);
    if transfer(&pipe, &mut message, true, deadline, &stop)? != message.len() {
        return Err(ActivationError::InvalidRequest);
    }
    let mut ack = [0; LIMIT];
    let count = transfer(&pipe, &mut ack, false, deadline, &stop)?;
    if ack[..count] != *ACK {
        return Err(ActivationError::InvalidRequest);
    }
    let mut receipt = RECEIPT.to_vec();
    if transfer(&pipe, &mut receipt, true, deadline, &stop)? != receipt.len() {
        return Err(ActivationError::InvalidRequest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        _guard: crate::instance::InstanceGuard,
        directory: tempfile::TempDir,
    }
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let handle = OpenOptions::new()
            .read(true)
            .access_mode(READ_CONTROL | WRITE_DAC | FILE_READ_ATTRIBUTES)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(directory.path())
            .unwrap();
        let descriptor = Descriptor::private(&current_sid().unwrap()).unwrap();
        let mut present = 0;
        let mut defaulted = 0;
        let mut dacl = ptr::null_mut();
        unsafe {
            assert_ne!(
                GetSecurityDescriptorDacl(descriptor.0.0, &mut present, &mut dacl, &mut defaulted),
                0
            );
            assert_eq!(
                SetSecurityInfo(
                    handle.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    dacl,
                    ptr::null()
                ),
                0
            );
        }
        drop(handle);
        let guard = crate::instance::InstanceGuard::acquire(directory.path()).unwrap();
        Fixture {
            _guard: guard,
            directory,
        }
    }
    fn await_restore(server: &ActivationServer) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(1) {
            if server.poll_restore() {
                return true;
            }
            thread::sleep(Duration::from_millis(5));
        }
        false
    }
    #[test]
    fn same_user_ack_coalescing_drop_and_rebind() {
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        assert!(!server.poll_restore());
        for _ in 0..3 {
            send_restore(f.directory.path()).unwrap();
        }
        assert!(await_restore(&server));
        assert!(!server.poll_restore());
        assert!(f.directory.path().join("activation.token").exists());
        let start = Instant::now();
        drop(server);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(!f.directory.path().join("activation.token").exists());
        assert!(f.directory.path().join("instance.lock").exists());
        let again = ActivationServer::bind(f.directory.path()).unwrap();
        send_restore(f.directory.path()).unwrap();
        assert!(await_restore(&again));
    }
    #[test]
    fn invalid_nonce_extra_commands_and_oversized_message_never_restore() {
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        let sid = current_sid().unwrap();
        let r = root(f.directory.path(), &sid).unwrap();
        let name = pipe_name(&r, &sid);
        for bytes in [
            b"arbitrary-download-path\n".to_vec(),
            request(&[b'0'; 32]).to_vec(),
            vec![b'x'; LIMIT + 1],
        ] {
            let client = connect(&name).unwrap();
            let stop = AtomicBool::new(false);
            let mut bytes = bytes;
            let _ = transfer(
                &client,
                &mut bytes,
                true,
                Instant::now() + IO_TIMEOUT,
                &stop,
            );
            drop(client);
            thread::sleep(Duration::from_millis(30));
            assert!(!server.poll_restore());
        }
        send_restore(f.directory.path()).unwrap();
        assert!(await_restore(&server));
    }
    #[test]
    fn idle_and_partial_client_are_bounded_and_drop_cancels_pending_read() {
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        let sid = current_sid().unwrap();
        let r = root(f.directory.path(), &sid).unwrap();
        let name = pipe_name(&r, &sid);
        let idle = connect(&name).unwrap();
        thread::sleep(IO_TIMEOUT + Duration::from_millis(75));
        drop(idle);
        assert!(!server.poll_restore());
        send_restore(f.directory.path()).unwrap();
        assert!(await_restore(&server));
        let partial = connect(&name).unwrap();
        let mut byte = [b'D'];
        let _ = transfer(
            &partial,
            &mut byte,
            true,
            Instant::now() + IO_TIMEOUT,
            &AtomicBool::new(false),
        );
        thread::sleep(Duration::from_millis(25));
        assert!(!server.poll_restore());
        drop(partial);
        let idle = connect(&name).unwrap();
        thread::sleep(Duration::from_millis(25));
        let start = Instant::now();
        drop(server);
        assert!(start.elapsed() < Duration::from_secs(1));
        drop(idle);
    }
    #[test]
    fn foreign_token_and_changed_marker_are_preserved() {
        let f = fixture();
        let path = f.directory.path().join("activation.token");
        fs::write(&path, b"foreign").unwrap();
        assert!(matches!(
            ActivationServer::bind(f.directory.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        assert_eq!(fs::read(&path).unwrap(), b"foreign");
        fs::remove_file(&path).unwrap();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        fs::write(
            f.directory.path().join(".dlpad-data-owner"),
            b"changed marker",
        )
        .unwrap();
        let token = fs::read(&path).unwrap();
        assert!(send_restore(f.directory.path()).is_err());
        drop(server);
        assert_eq!(fs::read(path).unwrap(), token);
    }
    #[test]
    fn foreign_pipe_public_token_and_replaced_nonce_are_rejected_preserved() {
        let f = fixture();
        let sid = current_sid().unwrap();
        let r = root(f.directory.path(), &sid).unwrap();
        let name = pipe_name(&r, &sid);
        let descriptor = Descriptor::private(&sid).unwrap();
        let foreign = create_pipe(&name, &descriptor).unwrap();
        assert!(matches!(
            ActivationServer::bind(f.directory.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        assert!(!f.directory.path().join("activation.token").exists());
        drop(foreign);
        drop(r);
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        let path = f.directory.path().join("activation.token");
        let bytes = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        fs::write(&path, &bytes).unwrap();
        // Default replacement file lacks protected single-user DACL; never trusted.
        assert!(send_restore(f.directory.path()).is_err());
        drop(server);
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    #[test]
    fn unsafe_missing_root_or_marker_and_changed_nonce_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        assert!(ActivationServer::bind(temp.path()).is_err());
        assert!(matches!(
            ActivationServer::bind(Path::new("relative")),
            Err(ActivationError::UnsafeData)
        ));
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        let path = f.directory.path().join("activation.token");
        fs::write(&path, [b'0'; TOKEN_BYTES]).unwrap();
        // Correct ACL but wrong nonce cannot queue a restore.
        assert!(send_restore(f.directory.path()).is_err());
        assert!(!server.poll_restore());
        drop(server);
        assert_eq!(fs::read(path).unwrap(), [b'0'; TOKEN_BYTES]);
    }
    #[test]
    fn missing_or_malformed_receipt_remains_bounded_and_restore_is_coalesced() {
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        assert!(matches!(
            ActivationServer::bind(f.directory.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        let sid = current_sid().unwrap();
        let root = root(f.directory.path(), &sid).unwrap();
        let name = pipe_name(&root, &sid);
        let nonce = token_read(&root.path.join("activation.token"), &sid)
            .unwrap()
            .0;
        let stop = AtomicBool::new(false);
        for malformed in [false, true] {
            let client = connect(&name).unwrap();
            let deadline = Instant::now() + IO_TIMEOUT;
            transfer(
                &client,
                &mut request(&nonce).to_vec(),
                true,
                deadline,
                &stop,
            )
            .unwrap();
            let mut ack = [0; LIMIT];
            let count = transfer(&client, &mut ack, false, deadline, &stop).unwrap();
            assert_eq!(&ack[..count], ACK);
            if malformed {
                transfer(
                    &client,
                    &mut b"arbitrary command".to_vec(),
                    true,
                    deadline,
                    &stop,
                )
                .unwrap();
            }
            thread::sleep(Duration::from_millis(325));
            assert!(server.poll_restore());
            assert!(!server.poll_restore());
            drop(client);
            send_restore(f.directory.path()).unwrap();
            assert!(server.poll_restore());
        }
    }
    #[test]
    fn provisional_cleanup_requires_native_identity_and_preserves_replacement() {
        let f = fixture();
        let sid = current_sid().unwrap();
        let root = Arc::new(root(f.directory.path(), &sid).unwrap());
        let descriptor = Descriptor::private(&sid).unwrap();
        let token = Token::create(root.clone(), &sid, &descriptor).unwrap();
        let path = token.path.clone();
        let identity = token.identity.clone();
        remove_provisional(&root, &path, &sid, &identity);
        assert!(!path.exists());
        drop(token);
        let replacement = Token::create(root.clone(), &sid, &descriptor).unwrap();
        remove_provisional(&root, &path, &sid, &identity);
        assert!(path.exists());
        drop(replacement);
        assert!(!path.exists());
    }
    #[test]
    fn native_client_process_entry() {
        if let Some(path) = std::env::var_os("DLPAD_ACTIVATION_FIXTURE") {
            send_restore(Path::new(&path)).unwrap();
        }
    }
    #[test]
    fn independent_same_user_process_receives_ack_without_console() {
        use std::os::windows::process::CommandExt;
        let f = fixture();
        let server = ActivationServer::bind(f.directory.path()).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "activation::windows::tests::native_client_process_entry",
            ])
            .env("DLPAD_ACTIVATION_FIXTURE", f.directory.path())
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("owned fixture client timed out");
            }
            thread::sleep(Duration::from_millis(10));
        };
        assert!(status.success());
        assert!(server.poll_restore());
        assert!(!server.poll_restore());
    }
    #[test]
    fn idle_connect_waits_once_and_stop_event_drains_original_io() {
        let fixture = fixture();
        let sid = current_sid().unwrap();
        let root = root(fixture.directory.path(), &sid).unwrap();
        let security = Descriptor::private(&sid).unwrap();
        let pipe = create_pipe(&pipe_name(&root, &sid), &security).unwrap();
        let stop_event =
            Arc::new(owned(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) }).unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let mut pending = Pending::new(&pipe).unwrap();
        assert_eq!(
            unsafe { ConnectNamedPipe(pipe.as_raw_handle(), &mut *pending.ov) },
            0
        );
        assert_eq!(
            io::Error::last_os_error().raw_os_error(),
            Some(ERROR_IO_PENDING as i32)
        );
        pending.active = true;
        let waits = pending.waits.clone();
        let observed_waits = waits.clone();
        let worker_stop = stop.clone();
        let event = stop_event.clone();
        let stopper = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            while observed_waits.load(Ordering::Acquire) == 0 {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
            thread::sleep(Duration::from_millis(120));
            worker_stop.store(true, Ordering::Release);
            assert_ne!(unsafe { SetEvent(event.as_raw_handle()) }, 0);
        });
        let started = Instant::now();
        assert!(
            pending
                .finish_until_stop(started + Duration::from_secs(3), &stop, &stop_event)
                .is_err()
        );
        drop(pending);
        stopper.join().unwrap();
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(
            waits.load(Ordering::Acquire),
            1,
            "idle listener must wait on completion or stop instead of polling"
        );
    }
    #[test]
    fn missing_and_malformed_server_ack_fail_with_finite_deadline() {
        for wrong_ack in [false, true] {
            let f = fixture();
            let sid = current_sid().unwrap();
            let root = Arc::new(root(f.directory.path(), &sid).unwrap());
            let descriptor = Descriptor::private(&sid).unwrap();
            let token = Token::create(root.clone(), &sid, &descriptor).unwrap();
            let pipe = create_pipe(&pipe_name(&root, &sid), &descriptor).unwrap();
            let worker = thread::spawn(move || {
                let stop = AtomicBool::new(false);
                let mut pending = Pending::new(&pipe).unwrap();
                if unsafe { ConnectNamedPipe(pipe.as_raw_handle(), &mut *pending.ov) } == 0 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() == Some(ERROR_IO_PENDING as i32) {
                        pending.active = true;
                        pending
                            .finish(Instant::now() + Duration::from_secs(1), &stop)
                            .unwrap();
                    } else {
                        assert_eq!(error.raw_os_error(), Some(ERROR_PIPE_CONNECTED as i32));
                    }
                }
                drop(pending);
                let mut request = [0; LIMIT];
                transfer(
                    &pipe,
                    &mut request,
                    false,
                    Instant::now() + IO_TIMEOUT,
                    &stop,
                )
                .unwrap();
                if wrong_ack {
                    transfer(
                        &pipe,
                        &mut b"invalid ack".to_vec(),
                        true,
                        Instant::now() + IO_TIMEOUT,
                        &stop,
                    )
                    .unwrap();
                }
                thread::sleep(Duration::from_millis(325));
            });
            let started = Instant::now();
            assert!(send_restore(f.directory.path()).is_err());
            assert!(started.elapsed() < Duration::from_secs(1));
            worker.join().unwrap();
            drop(token);
        }
    }
    #[test]
    fn nonprivileged_writable_root_acl_is_rejected_without_mutation() {
        let f = fixture();
        let marker = f.directory.path().join(".dlpad-data-owner");
        let original = fs::read(&marker).unwrap();
        let sid = current_sid().unwrap();
        let text: Vec<u16> = format!("O:{sid}D:P(A;;GA;;;{sid})(A;;GA;;;WD)")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut raw = ptr::null_mut();
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    text.as_ptr(),
                    SDDL_REVISION_1,
                    &mut raw,
                    ptr::null_mut(),
                )
            },
            0
        );
        let descriptor = Local(raw);
        let mut present = 0;
        let mut defaulted = 0;
        let mut dacl = ptr::null_mut();
        let file = OpenOptions::new()
            .access_mode(READ_CONTROL | WRITE_DAC)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(f.directory.path())
            .unwrap();
        unsafe {
            assert_ne!(
                GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut dacl, &mut defaulted),
                0
            );
            assert_eq!(
                SetSecurityInfo(
                    file.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    dacl,
                    ptr::null()
                ),
                0
            );
        }
        assert!(matches!(
            ActivationServer::bind(f.directory.path()),
            Err(ActivationError::UnsafeData)
        ));
        assert_eq!(fs::read(marker).unwrap(), original);
        assert!(!f.directory.path().join("activation.token").exists());
    }
}
