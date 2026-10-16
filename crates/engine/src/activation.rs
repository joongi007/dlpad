//! Fixed restore-only IPC. Never transports job requests, paths or executable commands.
use std::io;
#[cfg(not(any(target_os = "linux", windows)))]
use std::path::Path;

// A callback is a bounded wake hint; the atomic pending flag remains authoritative.
#[cfg(any(target_os = "linux", windows))]
#[derive(Default)]
struct RestoreSignal {
    pending: std::sync::atomic::AtomicBool,
    callback: std::sync::Mutex<Option<std::sync::Arc<dyn Fn() + Send + Sync>>>,
}
#[cfg(any(target_os = "linux", windows))]
impl RestoreSignal {
    fn wake(&self) {
        let callback = self
            .callback
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(callback) = callback {
            // A faulty notifier must not break authenticated IPC or lose pending state.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback()));
        }
    }
    fn notify(&self) {
        if !self.pending.swap(true, std::sync::atomic::Ordering::AcqRel) {
            self.wake();
        }
    }
    fn set(&self, callback: impl Fn() + Send + Sync + 'static) {
        *self.callback.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(std::sync::Arc::new(callback));
        if self.pending.load(std::sync::atomic::Ordering::Acquire) {
            self.wake();
        }
    }
    fn poll(&self) -> bool {
        self.pending
            .swap(false, std::sync::atomic::Ordering::AcqRel)
    }
}

#[derive(Debug)]
pub enum ActivationError {
    Unsupported,
    UnsafeData,
    ExistingEndpoint,
    InvalidRequest,
    Io(io::Error),
}
impl std::fmt::Display for ActivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("restore IPC is unsupported on this platform"),
            Self::UnsafeData => f.write_str("private owned data and endpoint are required"),
            Self::ExistingEndpoint => f.write_str("an existing restore endpoint is preserved"),
            Self::InvalidRequest => f.write_str("invalid restore-only IPC request"),
            Self::Io(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ActivationError {}
impl From<io::Error> for ActivationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{
        fs::{self, File, Metadata, OpenOptions},
        io::{Read, Write},
        os::{
            fd::{AsRawFd, FromRawFd, OwnedFd},
            unix::{
                ffi::OsStrExt,
                fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
                net::{UnixListener, UnixStream},
            },
        },
        path::{Component, Path, PathBuf},
        sync::{
            Arc, Condvar, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };
    const MARKER: &[u8] = b"dlpad-user-data-v1\n";
    const REQUEST_PREFIX: &[u8] = b"DLPAD_RESTORE_V1";
    const TOKEN_BYTES: usize = 32;
    const ACK: &[u8] = b"DLPAD_RESTORE_QUEUED_V1\n";
    const MESSAGE_LIMIT: usize = 64;
    const CONNECTION_TIMEOUT: Duration = Duration::from_millis(250);
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
    fn uid() -> u32 {
        // SAFETY: geteuid has no arguments or memory access obligations.
        unsafe { libc::geteuid() }
    }
    fn identity(metadata: &Metadata) -> (u64, u64) {
        (metadata.dev(), metadata.ino())
    }
    fn private_root(selected: &Path) -> Result<PathBuf, ActivationError> {
        if selected
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(ActivationError::UnsafeData);
        }
        let initial = fs::symlink_metadata(selected)?;
        if !initial.is_dir()
            || initial.file_type().is_symlink()
            || initial.uid() != uid()
            || initial.mode() & 0o077 != 0
        {
            return Err(ActivationError::UnsafeData);
        }
        let root = selected.canonicalize()?;
        let current = fs::symlink_metadata(&root)?;
        if identity(&initial) != identity(&current) {
            return Err(ActivationError::UnsafeData);
        }
        let marker = root.join(".dlpad-data-owner");
        let metadata = fs::symlink_metadata(&marker)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != uid()
            || metadata.mode() & 0o022 != 0
            || metadata.len() != MARKER.len() as u64
        {
            return Err(ActivationError::UnsafeData);
        }
        let mut file = File::open(marker)?;
        if identity(&file.metadata()?) != identity(&metadata) {
            return Err(ActivationError::UnsafeData);
        }
        let mut bytes = [0u8; MARKER.len()];
        file.read_exact(&mut bytes)?;
        if bytes.as_slice() != MARKER {
            return Err(ActivationError::UnsafeData);
        }
        Ok(root)
    }
    fn peer_owned(stream: &UnixStream) -> Result<(), ActivationError> {
        let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
        let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        // SAFETY: credential buffer and size are valid writable storage for SO_PEERCRED.
        if unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut credentials as *mut libc::ucred).cast(),
                &mut size,
            )
        } != 0
        {
            return Err(io::Error::last_os_error().into());
        }
        if size as usize != std::mem::size_of::<libc::ucred>() || credentials.uid != uid() {
            return Err(ActivationError::UnsafeData);
        }
        Ok(())
    }
    fn message(stream: &mut UnixStream, token: &[u8; TOKEN_BYTES]) -> Result<(), ActivationError> {
        peer_owned(stream)?;
        stream.set_read_timeout(Some(CONNECTION_TIMEOUT))?;
        stream.set_write_timeout(Some(CONNECTION_TIMEOUT))?;
        let mut bytes = [0u8; MESSAGE_LIMIT];
        let mut count = 0;
        let deadline = Instant::now() + CONNECTION_TIMEOUT;
        loop {
            if count == bytes.len() {
                return Err(ActivationError::InvalidRequest);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }
            stream.set_read_timeout(Some(remaining))?;
            let read = stream.read(&mut bytes[count..])?;
            if read == 0 {
                break;
            }
            count += read;
            if bytes[..count].contains(&b'\n') {
                break;
            }
        }
        if bytes[..count] != request(token) {
            return Err(ActivationError::InvalidRequest);
        }
        Ok(())
    }
    fn request(token: &[u8; TOKEN_BYTES]) -> [u8; 49] {
        let mut bytes = [0u8; 49];
        bytes[..REQUEST_PREFIX.len()].copy_from_slice(REQUEST_PREFIX);
        bytes[REQUEST_PREFIX.len()..48].copy_from_slice(token);
        bytes[48] = b'\n';
        bytes
    }
    fn read_token(path: &Path) -> Result<([u8; TOKEN_BYTES], (u64, u64)), ActivationError> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != uid()
            || metadata.mode() & 0o077 != 0
            || metadata.len() != TOKEN_BYTES as u64
        {
            return Err(ActivationError::UnsafeData);
        }
        let mut file = File::open(path)?;
        if identity(&file.metadata()?) != identity(&metadata) {
            return Err(ActivationError::UnsafeData);
        }
        let mut bytes = [0u8; TOKEN_BYTES];
        file.read_exact(&mut bytes)?;
        if !bytes
            .iter()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || file.metadata()?.len() != TOKEN_BYTES as u64
        {
            return Err(ActivationError::UnsafeData);
        }
        Ok((bytes, identity(&metadata)))
    }
    struct OwnedToken {
        root: PathBuf,
        root_identity: (u64, u64),
        path: PathBuf,
        file_identity: (u64, u64),
        bytes: [u8; TOKEN_BYTES],
    }
    impl OwnedToken {
        fn create(root: &Path) -> Result<Self, ActivationError> {
            let mut random = [0u8; 16];
            File::open("/dev/urandom")?.read_exact(&mut random)?;
            let mut bytes = [0u8; TOKEN_BYTES];
            const HEX: &[u8] = b"0123456789abcdef";
            for (index, byte) in random.into_iter().enumerate() {
                bytes[index * 2] = HEX[(byte >> 4) as usize];
                bytes[index * 2 + 1] = HEX[(byte & 15) as usize];
            }
            let path = root.join("activation.token");
            let mut file = match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    return Err(ActivationError::ExistingEndpoint);
                }
                Err(error) => return Err(error.into()),
            };
            file.write_all(&bytes)?;
            file.sync_all()?;
            Ok(Self {
                root: root.to_owned(),
                root_identity: identity(&fs::symlink_metadata(root)?),
                path,
                file_identity: identity(&file.metadata()?),
                bytes,
            })
        }
        fn matches(&self) -> bool {
            private_root(&self.root).is_ok()
                && fs::symlink_metadata(&self.root)
                    .is_ok_and(|metadata| identity(&metadata) == self.root_identity)
                && read_token(&self.path)
                    .is_ok_and(|(bytes, id)| bytes == self.bytes && id == self.file_identity)
        }
    }
    impl Drop for OwnedToken {
        fn drop(&mut self) {
            if self.matches() {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
    struct State {
        restore: RestoreSignal,
        stop: AtomicBool,
        wake: Condvar,
        mutex: Mutex<()>,
    }
    pub struct ActivationServer {
        root: PathBuf,
        root_identity: (u64, u64),
        endpoint: PathBuf,
        endpoint_identity: (u64, u64),
        token: OwnedToken,
        state: Arc<State>,
        worker: Option<JoinHandle<()>>,
    }
    impl ActivationServer {
        /// Caller must hold InstanceGuard for this data until the server has been dropped.
        pub fn bind(private_data: &Path) -> Result<Self, ActivationError> {
            let root = private_root(private_data)?;
            let root_identity = identity(&fs::symlink_metadata(&root)?);
            let endpoint = root.join("activation.sock");
            match fs::symlink_metadata(&endpoint) {
                Ok(_) => return Err(ActivationError::ExistingEndpoint),
                Err(error) if error.kind() == io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
            let token = OwnedToken::create(&root)?;
            let listener = UnixListener::bind(&endpoint)?;
            let metadata = fs::symlink_metadata(&endpoint)?;
            let endpoint_identity = identity(&metadata);
            let cleanup_on_error = || {
                if fs::symlink_metadata(&endpoint).is_ok_and(|current| {
                    identity(&current) == endpoint_identity
                        && current.file_type().is_socket()
                        && current.uid() == uid()
                }) {
                    let _ = fs::remove_file(&endpoint);
                }
            };
            if let Err(error) = fs::set_permissions(&endpoint, fs::Permissions::from_mode(0o600))
                .and_then(|_| listener.set_nonblocking(true))
            {
                cleanup_on_error();
                return Err(error.into());
            }
            let state = Arc::new(State {
                restore: RestoreSignal::default(),
                stop: AtomicBool::new(false),
                wake: Condvar::new(),
                mutex: Mutex::new(()),
            });
            let owned = state.clone();
            let secret = token.bytes;
            let worker =
                match thread::Builder::new()
                    .name("dlpad-restore".into())
                    .spawn(move || {
                        while !owned.stop.load(Ordering::Acquire) {
                            for _ in 0..8 {
                                if owned.stop.load(Ordering::Acquire) {
                                    break;
                                }
                                match listener.accept() {
                                    Ok((mut stream, _)) => {
                                        if message(&mut stream, &secret).is_ok()
                                            && !owned.stop.load(Ordering::Acquire)
                                        {
                                            owned.restore.notify();
                                            let _ = stream.write_all(ACK);
                                        }
                                    }
                                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                        break;
                                    }
                                    Err(_) => break,
                                }
                            }
                            let guard = owned
                                .mutex
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner());
                            let _ = owned.wake.wait_timeout_while(
                                guard,
                                Duration::from_millis(25),
                                |_| !owned.stop.load(Ordering::Acquire),
                            );
                        }
                    }) {
                    Ok(worker) => worker,
                    Err(error) => {
                        cleanup_on_error();
                        return Err(error.into());
                    }
                };
            Ok(Self {
                root,
                root_identity,
                endpoint,
                endpoint_identity,
                token,
                state,
                worker: Some(worker),
            })
        }
        /// Install a nonblocking wake hint; registration also wakes already pending restore.
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
            self.state.wake.notify_all();
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
            // No recursive delete or stale filename authority. Same-user path races remain a gate.
            if self.token.matches()
                && private_root(&self.root).is_ok()
                && fs::symlink_metadata(&self.root)
                    .is_ok_and(|m| identity(&m) == self.root_identity)
                && fs::symlink_metadata(&self.endpoint).is_ok_and(|m| {
                    identity(&m) == self.endpoint_identity
                        && m.file_type().is_socket()
                        && m.uid() == uid()
                })
            {
                let _ = fs::remove_file(&self.endpoint);
            }
        }
    }
    fn connect(endpoint: &Path) -> Result<UnixStream, ActivationError> {
        let bytes = endpoint.as_os_str().as_bytes();
        let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
        if bytes.len() >= address.sun_path.len() {
            return Err(ActivationError::UnsafeData);
        }
        address.sun_family = libc::AF_UNIX as libc::sa_family_t;
        for (slot, byte) in address.sun_path.iter_mut().zip(bytes) {
            *slot = *byte as libc::c_char;
        }
        // SAFETY: socket has no pointer inputs. Successful fd is exclusively owned immediately.
        let raw = unsafe {
            libc::socket(
                libc::AF_UNIX,
                libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                0,
            )
        };
        if raw < 0 {
            return Err(io::Error::last_os_error().into());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        let size = (std::mem::offset_of!(libc::sockaddr_un, sun_path) + bytes.len() + 1)
            as libc::socklen_t;
        // SAFETY: address storage contains AF_UNIX and a bounded NUL-terminated pathname.
        let connected = unsafe {
            libc::connect(
                fd.as_raw_fd(),
                (&address as *const libc::sockaddr_un).cast(),
                size,
            )
        };
        if connected != 0 {
            let error = io::Error::last_os_error();
            if !matches!(error.raw_os_error(), Some(libc::EINPROGRESS | libc::EAGAIN)) {
                return Err(error.into());
            }
            let end = Instant::now() + CONNECT_TIMEOUT;
            loop {
                let mut descriptor = libc::pollfd {
                    fd: fd.as_raw_fd(),
                    events: libc::POLLOUT,
                    revents: 0,
                };
                let remaining = end.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(io::Error::from(io::ErrorKind::TimedOut).into());
                }
                // SAFETY: single pollfd storage is valid for the duration of the call.
                let result = unsafe {
                    libc::poll(
                        &mut descriptor,
                        1,
                        remaining.as_millis().min(i32::MAX as u128) as i32,
                    )
                };
                if result < 0 {
                    let error = io::Error::last_os_error();
                    if error.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    return Err(error.into());
                }
                if result == 0 {
                    return Err(io::Error::from(io::ErrorKind::TimedOut).into());
                }
                let mut failure: libc::c_int = 0;
                let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
                // SAFETY: SO_ERROR writes into the initialized integer and valid length.
                if unsafe {
                    libc::getsockopt(
                        fd.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_ERROR,
                        (&mut failure as *mut libc::c_int).cast(),
                        &mut length,
                    )
                } != 0
                {
                    return Err(io::Error::last_os_error().into());
                }
                if failure != 0 {
                    return Err(io::Error::from_raw_os_error(failure).into());
                }
                break;
            }
        }
        let stream = UnixStream::from(fd);
        stream.set_nonblocking(false)?;
        Ok(stream)
    }
    pub fn send_restore(private_data: &Path) -> Result<(), ActivationError> {
        let root = private_root(private_data)?;
        let endpoint = root.join("activation.sock");
        let metadata = fs::symlink_metadata(&endpoint)?;
        if !metadata.file_type().is_socket()
            || metadata.uid() != uid()
            || metadata.mode() & 0o077 != 0
        {
            return Err(ActivationError::UnsafeData);
        }
        let (token, _) = read_token(&root.join("activation.token"))?;
        let mut stream = connect(&endpoint)?;
        peer_owned(&stream)?;
        if !fs::symlink_metadata(&endpoint)
            .is_ok_and(|current| identity(&current) == identity(&metadata))
        {
            return Err(ActivationError::UnsafeData);
        }
        stream.set_read_timeout(Some(CONNECTION_TIMEOUT))?;
        stream.set_write_timeout(Some(CONNECTION_TIMEOUT))?;
        stream.write_all(&request(&token))?;
        let mut ack = [0u8; ACK.len()];
        let deadline = Instant::now() + CONNECTION_TIMEOUT;
        let mut count = 0;
        while count < ack.len() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::Error::from(io::ErrorKind::TimedOut).into());
            }
            stream.set_read_timeout(Some(remaining))?;
            let read = stream.read(&mut ack[count..])?;
            if read == 0 {
                return Err(ActivationError::InvalidRequest);
            }
            count += read;
        }
        if ack.as_slice() != ACK {
            return Err(ActivationError::InvalidRequest);
        }
        Ok(())
    }
}
#[cfg(target_os = "linux")]
pub use linux::{ActivationServer, send_restore};
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{ActivationServer, send_restore};
#[cfg(not(any(target_os = "linux", windows)))]
pub struct ActivationServer;
#[cfg(not(any(target_os = "linux", windows)))]
impl ActivationServer {
    pub fn bind(_: &Path) -> Result<Self, ActivationError> {
        Err(ActivationError::Unsupported)
    }
    pub fn set_wake_callback(&self, _: impl Fn() + Send + Sync + 'static) {}
    pub fn poll_restore(&self) -> bool {
        false
    }
}
#[cfg(not(any(target_os = "linux", windows)))]
pub fn send_restore(_: &Path) -> Result<(), ActivationError> {
    Err(ActivationError::Unsupported)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{
        fs,
        io::{Read, Write},
        os::unix::{
            fs::{PermissionsExt, symlink},
            net::{UnixListener, UnixStream},
        },
        thread,
        time::{Duration, Instant},
    };
    fn owned() -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            temp.path().join(".dlpad-data-owner"),
            b"dlpad-user-data-v1\n",
        )
        .unwrap();
        temp
    }
    #[test]
    fn authenticated_restore_wakes_late_registration_and_coalesces() {
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        send_restore(temp.path()).unwrap();
        let (tx, rx) = std::sync::mpsc::sync_channel(4);
        server.set_wake_callback(move || {
            let _ = tx.try_send(());
        });
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        send_restore(temp.path()).unwrap();
        assert!(rx.try_recv().is_err()); // same undrained pending restore
        assert!(server.poll_restore());
        assert!(!server.poll_restore());
        send_restore(temp.path()).unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(server.poll_restore());
        server.set_wake_callback(|| panic!("owned notifier failure"));
        send_restore(temp.path()).unwrap();
        assert!(server.poll_restore()); // notifier panic does not lose IPC state
    }
    #[test]
    fn restores_coalesce_and_drop_removes_only_owned_endpoint() {
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        send_restore(temp.path()).unwrap();
        send_restore(temp.path()).unwrap();
        assert!(server.poll_restore());
        assert!(!server.poll_restore());
        let endpoint = temp.path().join("activation.sock");
        assert_eq!(
            fs::metadata(&endpoint).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(server);
        assert!(!endpoint.exists());
        assert_eq!(
            fs::read(temp.path().join(".dlpad-data-owner")).unwrap(),
            b"dlpad-user-data-v1\n"
        );
    }
    #[test]
    fn nonprivate_wrong_marker_and_symlinks_fail_without_touching_external_bytes() {
        let temp = owned();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(temp.path().join(".dlpad-data-owner"), b"foreign bytes").unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        assert_eq!(
            fs::read(temp.path().join(".dlpad-data-owner")).unwrap(),
            b"foreign bytes"
        );
        let external = tempfile::tempdir().unwrap();
        let original = external.path().join("original");
        fs::write(&original, b"dlpad-user-data-v1\n").unwrap();
        fs::remove_file(temp.path().join(".dlpad-data-owner")).unwrap();
        symlink(&original, temp.path().join(".dlpad-data-owner")).unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        assert_eq!(fs::read(&original).unwrap(), b"dlpad-user-data-v1\n");
        let alias = external.path().join("alias");
        symlink(temp.path(), &alias).unwrap();
        assert!(matches!(
            ActivationServer::bind(&alias),
            Err(ActivationError::UnsafeData)
        ));
    }
    #[test]
    fn stale_foreign_and_replaced_endpoints_have_no_filename_cleanup_authority() {
        let temp = owned();
        let path = temp.path().join("activation.sock");
        fs::write(&path, b"foreign original").unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        assert_eq!(fs::read(&path).unwrap(), b"foreign original");
        fs::remove_file(&path).unwrap();
        drop(UnixListener::bind(&path).unwrap());
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        assert!(path.exists());
        fs::remove_file(&path).unwrap();
        let server = ActivationServer::bind(temp.path()).unwrap();
        fs::rename(&path, temp.path().join("owned-renamed.sock")).unwrap();
        fs::write(&path, b"foreign replacement").unwrap();
        drop(server);
        assert_eq!(fs::read(&path).unwrap(), b"foreign replacement");
        assert!(temp.path().join("owned-renamed.sock").exists());
    }
    #[test]
    fn invalid_and_oversized_messages_never_queue_restore() {
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        for bytes in [
            b"DLPAD_EXEC_V1\n".to_vec(),
            vec![b'x'; 129],
            b"DLPAD_RESTORE_V1\nextra\n".to_vec(),
        ] {
            let mut stream = UnixStream::connect(temp.path().join("activation.sock")).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            stream.write_all(&bytes).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut ack = [0u8; 64];
            assert!(!matches!(stream.read(&mut ack), Ok(n) if n>0));
            assert!(!server.poll_restore());
        }
        send_restore(temp.path()).unwrap();
        assert!(server.poll_restore());
    }
    #[test]
    fn orphan_or_changed_token_is_never_filename_cleanup_authority() {
        let temp = owned();
        let token = temp.path().join("activation.token");
        fs::write(&token, b"foreign orphan").unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
        assert_eq!(fs::read(&token).unwrap(), b"foreign orphan");
        assert!(!temp.path().join("activation.sock").exists());
        fs::remove_file(&token).unwrap();
        let server = ActivationServer::bind(temp.path()).unwrap();
        let original = fs::read(&token).unwrap();
        let mut changed = original.clone();
        changed[0] = if changed[0] == b'0' { b'1' } else { b'0' };
        fs::write(&token, &changed).unwrap();
        assert!(send_restore(temp.path()).is_err());
        assert!(!server.poll_restore());
        drop(server);
        assert_eq!(fs::read(&token).unwrap(), changed);
        assert!(temp.path().join("activation.sock").exists());
    }
    #[test]
    fn symlink_or_public_token_rejects_restore_and_preserves_foreign_original() {
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        let token = temp.path().join("activation.token");
        fs::set_permissions(&token, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            send_restore(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        fs::rename(&token, temp.path().join("owned-original-token")).unwrap();
        let external = tempfile::tempdir().unwrap();
        let original = external.path().join("original");
        fs::write(&original, b"original external token").unwrap();
        symlink(&original, &token).unwrap();
        assert!(matches!(
            send_restore(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        drop(server);
        assert!(token.is_symlink());
        assert_eq!(fs::read(original).unwrap(), b"original external token");
        assert!(temp.path().join("activation.sock").exists());
    }
    #[test]
    fn idle_and_drip_connections_have_a_whole_request_deadline_and_drop_is_bounded() {
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        let mut idle = UnixStream::connect(temp.path().join("activation.sock")).unwrap();
        idle.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let start = Instant::now();
        let mut byte = [0];
        assert_eq!(idle.read(&mut byte).unwrap(), 0);
        assert!(start.elapsed() < Duration::from_secs(1));
        let mut drip = UnixStream::connect(temp.path().join("activation.sock")).unwrap();
        let sender = thread::spawn(move || {
            for _ in 0..8 {
                if drip.write_all(b"D").is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(80));
            }
        });
        thread::sleep(Duration::from_millis(30));
        let start = Instant::now();
        drop(server);
        assert!(start.elapsed() < Duration::from_secs(1));
        sender.join().unwrap();
        assert!(!temp.path().join("activation.sock").exists());
    }
    #[test]
    fn sender_ack_has_a_whole_deadline_and_invalid_ack_is_rejected() {
        let temp = owned();
        fs::write(
            temp.path().join("activation.token"),
            b"00000000000000000000000000000000",
        )
        .unwrap();
        fs::set_permissions(
            temp.path().join("activation.token"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        let path = temp.path().join("activation.sock");
        let listener = UnixListener::bind(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let peer = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 49];
            stream.read_exact(&mut request).unwrap();
            thread::sleep(Duration::from_millis(400));
            let _ = stream.write_all(b"invalid acknowledgement");
        });
        let start = Instant::now();
        assert!(send_restore(temp.path()).is_err());
        assert!(start.elapsed() < Duration::from_secs(1));
        peer.join().unwrap();
        fs::remove_file(&path).unwrap();
        let listener = UnixListener::bind(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let peer = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 49];
            stream.read_exact(&mut request).unwrap();
            stream.write_all(b"XXXXXXXXXXXXXXXXXXXXXXXX").unwrap();
        });
        assert!(matches!(
            send_restore(temp.path()),
            Err(ActivationError::InvalidRequest)
        ));
        peer.join().unwrap();
    }
    #[test]
    fn actual_cleanup_permission_failure_preserves_owned_socket_token_and_marker() {
        // Root bypasses DAC; do not report a root run as a permission denial test.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let temp = owned();
        let server = ActivationServer::bind(temp.path()).unwrap();
        let token = fs::read(temp.path().join("activation.token")).unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o500)).unwrap();
        drop(server);
        assert!(temp.path().join("activation.sock").exists());
        assert_eq!(
            fs::read(temp.path().join("activation.token")).unwrap(),
            token
        );
        assert_eq!(
            fs::read(temp.path().join(".dlpad-data-owner")).unwrap(),
            b"dlpad-user-data-v1\n"
        );
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(
            ActivationServer::bind(temp.path()),
            Err(ActivationError::ExistingEndpoint)
        ));
    }
    #[test]
    fn missing_or_public_socket_and_changed_root_marker_fail_closed() {
        let temp = owned();
        assert!(send_restore(temp.path()).is_err());
        let server = ActivationServer::bind(temp.path()).unwrap();
        fs::set_permissions(
            temp.path().join("activation.sock"),
            fs::Permissions::from_mode(0o666),
        )
        .unwrap();
        assert!(matches!(
            send_restore(temp.path()),
            Err(ActivationError::UnsafeData)
        ));
        fs::write(
            temp.path().join(".dlpad-data-owner"),
            b"changed ownership evidence",
        )
        .unwrap();
        drop(server);
        assert!(temp.path().join("activation.sock").exists());
        assert_eq!(
            fs::read(temp.path().join(".dlpad-data-owner")).unwrap(),
            b"changed ownership evidence"
        );
    }
}
#[cfg(all(test, not(any(target_os = "linux", windows))))]
mod unsupported_tests {
    use super::*;
    #[test]
    fn restore_has_no_silent_platform_fallback() {
        assert!(matches!(
            ActivationServer::bind(Path::new("unused")),
            Err(ActivationError::Unsupported)
        ));
        assert!(matches!(
            send_restore(Path::new("unused")),
            Err(ActivationError::Unsupported)
        ));
    }
}
