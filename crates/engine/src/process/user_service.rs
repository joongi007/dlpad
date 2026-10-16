//! Linux user-manager transport. No shell or privileged-manager fallback.
use libc::{c_char, c_int, c_void};
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    fs::File,
    io::{self, Read},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::{
            fs::{FileTypeExt, MetadataExt},
            process::ExitStatusExt,
        },
    },
    path::{Component, Path, PathBuf},
    process::ExitStatus,
    ptr, thread,
    time::{Duration, Instant},
};

const CALL_US: u64 = 2_000_000;
const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_INPUT_COUNT: usize = 4096;
const MAX_PROPERTY_BYTES: usize = 4096;
const CLEANUP_GRACE: Duration = Duration::from_secs(5);

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn failure(message: &'static str) -> io::Error {
    io::Error::other(message)
}
fn check(code: c_int) -> io::Result<()> {
    if code < 0 {
        Err(io::Error::from_raw_os_error(
            code.checked_neg().unwrap_or(libc::EIO),
        ))
    } else {
        Ok(())
    }
}
struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            libc::dlclose(self.0);
        }
    }
}

#[repr(C)]
struct RawError {
    name: *const c_char,
    message: *const c_char,
    need_free: c_int,
}
macro_rules! api {
    ($($name:ident: $signature:ty),+ $(,)?) => {
        struct Api { _library: Library, $($name: $signature),+ }
        impl Api {
            fn load() -> io::Result<Self> {
                let raw = unsafe { libc::dlopen(c"libsystemd.so.0".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
                if raw.is_null() { return Err(failure("user-service runtime library unavailable")); }
                let library = Library(raw);
                $(let $name = {
                    let symbol = unsafe { libc::dlsym(raw, concat!(stringify!($name), "\0").as_ptr().cast()) };
                    if symbol.is_null() { return Err(failure("user-service runtime symbol unavailable")); }
                    unsafe { std::mem::transmute::<*mut c_void, $signature>(symbol) }
                };)+
                Ok(Self { _library: library, $($name),+ })
            }
        }
    }
}
api! {
    sd_bus_new: unsafe extern "C" fn(*mut *mut c_void) -> c_int,
    sd_bus_set_address: unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    sd_bus_set_bus_client: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    sd_bus_negotiate_fds: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    sd_bus_start: unsafe extern "C" fn(*mut c_void) -> c_int,
    sd_bus_unref: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    sd_bus_message_new_method_call: unsafe extern "C" fn(*mut c_void, *mut *mut c_void, *const c_char, *const c_char, *const c_char, *const c_char) -> c_int,
    sd_bus_message_set_allow_interactive_authorization: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    sd_bus_message_open_container: unsafe extern "C" fn(*mut c_void, c_char, *const c_char) -> c_int,
    sd_bus_message_close_container: unsafe extern "C" fn(*mut c_void) -> c_int,
    sd_bus_message_append_basic: unsafe extern "C" fn(*mut c_void, c_char, *const c_void) -> c_int,
    sd_bus_call: unsafe extern "C" fn(*mut c_void, *mut c_void, u64, *mut RawError, *mut *mut c_void) -> c_int,
    sd_bus_error_free: unsafe extern "C" fn(*mut RawError),
    sd_bus_message_unref: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    sd_bus_message_read_basic: unsafe extern "C" fn(*mut c_void, c_char, *mut c_void) -> c_int,
    sd_bus_message_enter_container: unsafe extern "C" fn(*mut c_void, c_char, *const c_char) -> c_int,
    sd_bus_message_read_array: unsafe extern "C" fn(*mut c_void, c_char, *mut *const c_void, *mut usize) -> c_int,
}
struct Bus<'a> {
    api: &'a Api,
    raw: *mut c_void,
}
impl Drop for Bus<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.sd_bus_unref)(self.raw);
        }
    }
}
struct Message<'a> {
    api: &'a Api,
    raw: *mut c_void,
}
impl Drop for Message<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.sd_bus_message_unref)(self.raw);
        }
    }
}
struct Error<'a> {
    api: &'a Api,
    raw: RawError,
}
impl Drop for Error<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.sd_bus_error_free)(&mut self.raw);
        }
    }
}
impl<'a> Bus<'a> {
    fn connect(api: &'a Api, path: &Path) -> io::Result<Self> {
        let mut raw = ptr::null_mut();
        check(unsafe { (api.sd_bus_new)(&mut raw) })?;
        if raw.is_null() {
            return Err(failure("user-service bus allocation failed"));
        }
        let bus = Self { api, raw };
        let address = bus_address(path)?;
        check(unsafe { (api.sd_bus_set_address)(raw, address.as_ptr()) })?;
        check(unsafe { (api.sd_bus_set_bus_client)(raw, 1) })?;
        check(unsafe { (api.sd_bus_negotiate_fds)(raw, 1) })?;
        check(unsafe { (api.sd_bus_start)(raw) })?;
        Ok(bus)
    }
    fn method(&self, path: &str, interface: &str, method: &str) -> io::Result<Message<'a>> {
        let path = CString::new(path).map_err(|_| invalid("user-service object path invalid"))?;
        let interface =
            CString::new(interface).map_err(|_| invalid("user-service interface invalid"))?;
        let method = CString::new(method).map_err(|_| invalid("user-service method invalid"))?;
        let mut raw = ptr::null_mut();
        check(unsafe {
            (self.api.sd_bus_message_new_method_call)(
                self.raw,
                &mut raw,
                c"org.freedesktop.systemd1".as_ptr(),
                path.as_ptr(),
                interface.as_ptr(),
                method.as_ptr(),
            )
        })?;
        if raw.is_null() {
            return Err(failure("user-service message allocation failed"));
        }
        let message = Message { api: self.api, raw };
        check(unsafe { (self.api.sd_bus_message_set_allow_interactive_authorization)(raw, 0) })?;
        Ok(message)
    }
    fn call(&self, message: &Message<'_>) -> io::Result<Message<'a>> {
        let mut error = Error {
            api: self.api,
            raw: RawError {
                name: ptr::null(),
                message: ptr::null(),
                need_free: 0,
            },
        };
        let mut raw = ptr::null_mut();
        let result = unsafe {
            (self.api.sd_bus_call)(self.raw, message.raw, CALL_US, &mut error.raw, &mut raw)
        };
        // Even a failed call may have allocated a reply; own it before checking.
        let reply = Message { api: self.api, raw };
        check(result)?;
        if raw.is_null() {
            return Err(failure("user-service reply missing"));
        }
        Ok(reply)
    }
    fn property(
        &self,
        path: &str,
        interface: &str,
        name: &str,
        signature: &CStr,
    ) -> io::Result<Message<'a>> {
        let request = self.method(path, "org.freedesktop.DBus.Properties", "Get")?;
        request.string(interface)?;
        request.string(name)?;
        let reply = self.call(&request)?;
        let result = unsafe {
            (self.api.sd_bus_message_enter_container)(reply.raw, b'v' as c_char, signature.as_ptr())
        };
        check(result)?;
        if result == 0 {
            return Err(failure("user-service property variant missing"));
        }
        Ok(reply)
    }
    fn text(&self, path: &str, interface: &str, name: &str) -> io::Result<String> {
        self.property(path, interface, name, c"s")?
            .read_string(b's')
    }
    fn number_u(&self, path: &str, name: &str) -> io::Result<u32> {
        let message = self.property(path, "org.freedesktop.systemd1.Service", name, c"u")?;
        let mut value = 0u32;
        message.read(b'u', (&mut value as *mut u32).cast())?;
        Ok(value)
    }
    fn number_i(&self, path: &str, name: &str) -> io::Result<i32> {
        let message = self.property(path, "org.freedesktop.systemd1.Service", name, c"i")?;
        let mut value = 0i32;
        message.read(b'i', (&mut value as *mut i32).cast())?;
        Ok(value)
    }
    fn boolean(&self, path: &str, interface: &str, name: &str) -> io::Result<bool> {
        let message = self.property(path, interface, name, c"b")?;
        let mut value: c_int = 0;
        message.read(b'b', (&mut value as *mut c_int).cast())?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(failure("user-service boolean invalid")),
        }
    }
    fn invocation(&self, path: &str) -> io::Result<[u8; 16]> {
        self.starting_invocation(path)?
            .ok_or_else(|| failure("user-service invocation identity missing"))
    }
    fn starting_invocation(&self, path: &str) -> io::Result<Option<[u8; 16]>> {
        let message =
            self.property(path, "org.freedesktop.systemd1.Unit", "InvocationID", c"ay")?;
        let mut raw = ptr::null();
        let mut size = 0usize;
        let result = unsafe {
            (self.api.sd_bus_message_read_array)(message.raw, b'y' as c_char, &mut raw, &mut size)
        };
        check(result)?;
        if result > 0 && size == 0 {
            return Ok(None);
        }
        if result == 0 || size != 16 || raw.is_null() {
            return Err(failure("user-service invocation identity invalid"));
        }
        let mut identity = [0u8; 16];
        identity.copy_from_slice(unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), 16) });
        if identity == [0; 16] {
            return Ok(None);
        }
        Ok(Some(identity))
    }
}
impl Message<'_> {
    fn open(&self, kind: u8, contents: &CStr) -> io::Result<()> {
        check(unsafe {
            (self.api.sd_bus_message_open_container)(self.raw, kind as c_char, contents.as_ptr())
        })
    }
    fn close(&self) -> io::Result<()> {
        check(unsafe { (self.api.sd_bus_message_close_container)(self.raw) })
    }
    fn basic(&self, kind: u8, pointer: *const c_void) -> io::Result<()> {
        check(unsafe { (self.api.sd_bus_message_append_basic)(self.raw, kind as c_char, pointer) })
    }
    fn string(&self, value: &str) -> io::Result<()> {
        let value = CString::new(value).map_err(|_| invalid("user-service string contains NUL"))?;
        self.basic(b's', value.as_ptr().cast())
    }
    fn strings(&self, values: &[CString]) -> io::Result<()> {
        self.open(b'a', c"s")?;
        for value in values {
            self.basic(b's', value.as_ptr().cast())?;
        }
        self.close()
    }
    fn scalar_property(
        &self,
        name: &str,
        kind: u8,
        signature: &CStr,
        value: *const c_void,
    ) -> io::Result<()> {
        self.open(b'r', c"sv")?;
        self.string(name)?;
        self.open(b'v', signature)?;
        self.basic(kind, value)?;
        self.close()?;
        self.close()
    }
    fn text_property(&self, name: &str, value: &CStr) -> io::Result<()> {
        self.scalar_property(name, b's', c"s", value.as_ptr().cast())
    }
    fn read(&self, kind: u8, pointer: *mut c_void) -> io::Result<()> {
        let result =
            unsafe { (self.api.sd_bus_message_read_basic)(self.raw, kind as c_char, pointer) };
        check(result)?;
        if result == 0 {
            return Err(failure("user-service property value missing"));
        }
        Ok(())
    }
    fn read_string(&self, kind: u8) -> io::Result<String> {
        let mut raw: *const c_char = ptr::null();
        self.read(kind, (&mut raw as *mut *const c_char).cast())?;
        if raw.is_null() {
            return Err(failure("user-service property string missing"));
        }
        let bytes = unsafe { CStr::from_ptr(raw) }.to_bytes();
        if bytes.len() > MAX_PROPERTY_BYTES {
            return Err(failure("user-service property exceeds limit"));
        }
        Ok(std::str::from_utf8(bytes)
            .map_err(|_| failure("user-service property is not UTF-8"))?
            .to_owned())
    }
}

fn strings<V: AsRef<OsStr>>(values: impl IntoIterator<Item = V>) -> io::Result<Vec<CString>> {
    let mut result = Vec::new();
    let mut bytes = 0usize;
    for value in values {
        if result.len() >= MAX_INPUT_COUNT {
            return Err(invalid("user-service input count exceeds limit"));
        }
        let text = value
            .as_ref()
            .to_str()
            .ok_or_else(|| invalid("user-service input is not UTF-8"))?;
        bytes = bytes
            .checked_add(text.len().saturating_add(1))
            .ok_or_else(|| invalid("user-service input size overflow"))?;
        if bytes > MAX_INPUT_BYTES {
            return Err(invalid("user-service input exceeds limit"));
        }
        result.push(CString::new(text).map_err(|_| invalid("user-service input contains NUL"))?);
    }
    Ok(result)
}
fn environment() -> io::Result<Vec<CString>> {
    // Cargo exposes executable paths under names with '-' during integration
    // tests. These build-harness hints are not tool runtime configuration and
    // systemd rejects their names. Other invalid input still fails closed.
    environment_entries(std::env::vars_os().filter(|(key, _)| {
        !key.to_str()
            .is_some_and(|name| name.starts_with("CARGO_BIN_EXE_") && !environment_key_valid(name))
    }))
}
fn environment_key_valid(key: &str) -> bool {
    key.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
fn environment_entries(
    values: impl IntoIterator<Item = (OsString, OsString)>,
) -> io::Result<Vec<CString>> {
    let mut entries = Vec::new();
    let mut bytes = 0usize;
    for (key, value) in values {
        if entries.len() >= MAX_INPUT_COUNT {
            return Err(invalid("user-service environment count exceeds limit"));
        }
        let key = key
            .to_str()
            .ok_or_else(|| invalid("user-service environment key is not UTF-8"))?;
        let value = value
            .to_str()
            .ok_or_else(|| invalid("user-service environment value is not UTF-8"))?;
        if !environment_key_valid(key) || value.contains('\0') {
            return Err(invalid("user-service environment invalid"));
        }
        bytes = bytes
            .checked_add(key.len())
            .and_then(|n| n.checked_add(value.len()))
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| invalid("user-service environment size overflow"))?;
        if bytes > MAX_INPUT_BYTES {
            return Err(invalid("user-service environment exceeds limit"));
        }
        entries.push(
            CString::new(format!("{key}={value}"))
                .map_err(|_| invalid("user-service environment invalid"))?,
        );
    }
    Ok(entries)
}
fn validate_control(control: &OwnedFd) -> io::Result<()> {
    for (option, expected) in [
        (libc::SO_TYPE, libc::SOCK_STREAM),
        (libc::SO_DOMAIN, libc::AF_UNIX),
    ] {
        let mut value = 0 as c_int;
        let mut length = std::mem::size_of::<c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                control.as_raw_fd(),
                libc::SOL_SOCKET,
                option,
                (&mut value as *mut c_int).cast(),
                &mut length,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        if value != expected || length as usize != std::mem::size_of::<c_int>() {
            return Err(invalid("user-service control socket invalid"));
        }
    }
    let flags = unsafe { libc::fcntl(control.as_raw_fd(), libc::F_GETFD) };
    if flags < 0
        || unsafe { libc::fcntl(control.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn bus_address(path: &Path) -> io::Result<CString> {
    let text = path
        .to_str()
        .ok_or_else(|| invalid("user-service bus path is not UTF-8"))?;
    if text.len() > MAX_PROPERTY_BYTES {
        return Err(invalid("user-service bus path exceeds limit"));
    }
    let mut address = String::from("unix:path=");
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-_.".contains(&byte) {
            address.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(&mut address, "%{byte:02X}")
                .map_err(|_| failure("user-service address encoding failed"))?;
        }
    }
    CString::new(address).map_err(|_| invalid("user-service bus address invalid"))
}

fn bus_path() -> io::Result<PathBuf> {
    let uid = unsafe { libc::getuid() };
    #[cfg(debug_assertions)]
    if let Some(value) = std::env::var_os("DLPAD_TEST_SYSTEMD_BUS_PATH") {
        let path = PathBuf::from(value);
        if !path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
        {
            return Err(invalid("user-service test bus path invalid"));
        }
        validate_socket(&path, uid)?;
        return Ok(path);
    }
    let directory = PathBuf::from(format!("/run/user/{uid}"));
    let metadata = std::fs::symlink_metadata(&directory)?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err(failure("user-service runtime directory unsafe"));
    }
    let path = directory.join("bus");
    validate_socket(&path, uid)?;
    Ok(path)
}
fn validate_socket(path: &Path, uid: u32) -> io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() || metadata.uid() != uid {
        return Err(failure("user-service socket ownership invalid"));
    }
    Ok(())
}
fn nonce_unit() -> io::Result<String> {
    let mut nonce = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut nonce)?;
    let mut name = String::from("dlpad-job-");
    for byte in nonce {
        use std::fmt::Write;
        write!(&mut name, "{byte:02x}")
            .map_err(|_| failure("user-service identity formatting failed"))?;
    }
    name.push_str(".service");
    Ok(name)
}
fn gated_nonce_unit() -> io::Result<String> {
    #[cfg(debug_assertions)]
    if let Some(value) = std::env::var_os("DLPAD_TEST_COOKIE_UNIT_NONCE") {
        return test_nonce_unit(&value);
    }
    nonce_unit()
}
#[cfg(any(debug_assertions, test))]
fn test_nonce_unit(value: &OsStr) -> io::Result<String> {
    let value = value
        .to_str()
        .ok_or_else(|| invalid("synthetic reader nonce invalid"))?;
    if value.len() != 32
        || !value.bytes().all(|b| b.is_ascii_hexdigit())
        || value.bytes().all(|b| b == b'0')
    {
        return Err(invalid("synthetic reader nonce invalid"));
    }
    Ok(format!("dlpad-job-{}.service", value.to_ascii_lowercase()))
}
#[cfg(debug_assertions)]
const RECEIPT_BARRIER_ENV: &str = "DLPAD_TEST_COOKIE_RECEIPT_BARRIER_FD";
#[cfg(any(debug_assertions, test))]
fn receipt_barrier_fd(value: &OsStr) -> io::Result<c_int> {
    let value = value
        .to_str()
        .ok_or_else(|| invalid("receipt barrier descriptor invalid"))?;
    if value.is_empty() || value.len() > 10 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("receipt barrier descriptor invalid"));
    }
    let fd: c_int = value
        .parse()
        .map_err(|_| invalid("receipt barrier descriptor invalid"))?;
    if fd < 3 {
        return Err(invalid("receipt barrier descriptor invalid"));
    }
    Ok(fd)
}
#[cfg(any(debug_assertions, test))]
fn socket_stat(fd: c_int) -> io::Result<libc::stat> {
    let mut value = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: fstat initializes the provided native struct on success only.
    if unsafe { libc::fstat(fd, value.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { value.assume_init() })
}
#[cfg(any(debug_assertions, test))]
fn validate_receipt_barrier(fd: c_int, control: c_int) -> io::Result<()> {
    if fd < 3 || fd == control {
        return Err(invalid("receipt barrier descriptor alias"));
    }
    let metadata = socket_stat(fd)?;
    let owner = unsafe { libc::geteuid() };
    if metadata.st_mode & libc::S_IFMT != libc::S_IFSOCK || metadata.st_uid != owner {
        return Err(invalid("receipt barrier socket ownership invalid"));
    }
    let other = socket_stat(control)?;
    if (metadata.st_dev, metadata.st_ino) == (other.st_dev, other.st_ino) {
        return Err(invalid("receipt barrier control socket alias"));
    }
    for (option, expected) in [
        (libc::SO_DOMAIN, libc::AF_UNIX),
        (libc::SO_TYPE, libc::SOCK_STREAM),
        (libc::SO_ACCEPTCONN, 0),
    ] {
        let mut value: c_int = -1;
        let mut size = std::mem::size_of::<c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                option,
                (&mut value as *mut c_int).cast(),
                &mut size,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        if size as usize != std::mem::size_of::<c_int>() || value != expected {
            return Err(invalid("receipt barrier socket type invalid"));
        }
    }
    let mut peer = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            peer.as_mut_ptr().cast(),
            &mut size,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    if size as usize != std::mem::size_of::<libc::ucred>() {
        return Err(invalid("receipt barrier peer invalid"));
    }
    let peer = unsafe { peer.assume_init() };
    if peer.uid != owner || peer.pid <= 0 {
        return Err(invalid("receipt barrier peer ownership invalid"));
    }
    let mut address = std::mem::MaybeUninit::<libc::sockaddr_storage>::zeroed();
    let mut size = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    if unsafe { libc::getpeername(fd, address.as_mut_ptr().cast(), &mut size) } != 0 {
        return Err(io::Error::last_os_error());
    }
    if size < std::mem::size_of::<libc::sa_family_t>() as libc::socklen_t
        || unsafe { address.assume_init() }.ss_family as c_int != libc::AF_UNIX
    {
        return Err(invalid("receipt barrier connection invalid"));
    }
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let after = socket_stat(fd)?;
    if (after.st_dev, after.st_ino, after.st_uid, after.st_mode)
        != (
            metadata.st_dev,
            metadata.st_ino,
            metadata.st_uid,
            metadata.st_mode,
        )
    {
        return Err(invalid("receipt barrier socket identity changed"));
    }
    Ok(())
}
#[cfg(debug_assertions)]
fn capture_receipt_barrier(control: &OwnedFd) -> io::Result<Option<OwnedFd>> {
    let Some(value) = std::env::var_os(RECEIPT_BARRIER_ENV) else {
        return Ok(None);
    };
    let fd = receipt_barrier_fd(&value)?;
    validate_receipt_barrier(fd, control.as_raw_fd())?;
    // The explicit fixture delivery contract transfers this validated unique FD.
    Ok(Some(unsafe { OwnedFd::from_raw_fd(fd) }))
}
fn gated_environment() -> io::Result<Vec<CString>> {
    let values = environment()?;
    let values = values
        .into_iter()
        .filter(|value| {
            !value
                .as_bytes()
                .starts_with(b"DLPAD_TEST_COOKIE_RECEIPT_BARRIER_FD=")
        })
        .collect();
    Ok(values)
}
#[cfg(any(debug_assertions, test))]
fn barrier_poll(fd: c_int, events: libc::c_short, deadline: Instant) -> io::Result<()> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "receipt barrier deadline",
            ));
        }
        let millis = remaining
            .as_millis()
            .saturating_add(1)
            .min(i32::MAX as u128) as c_int;
        let mut event = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let count = unsafe { libc::poll(&mut event, 1, millis) };
        if count < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if count == 0 {
            continue;
        }
        if event.revents & (libc::POLLNVAL | libc::POLLERR) != 0 {
            return Err(failure("receipt barrier socket failed"));
        }
        // Read queued bytes before considering HUP; recv establishes actual EOF.
        if event.revents & (events | libc::POLLHUP) != 0 {
            return Ok(());
        }
    }
}
#[cfg(any(debug_assertions, test))]
fn receipt_barrier_exchange(socket: &OwnedFd) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    let fd = socket.as_raw_fd();
    loop {
        barrier_poll(fd, libc::POLLOUT, deadline)?;
        let count = unsafe {
            libc::send(
                fd,
                b"K".as_ptr().cast(),
                1,
                libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
            )
        };
        if count == 1 {
            break;
        }
        let error = io::Error::last_os_error();
        if count < 0
            && matches!(
                error.kind(),
                io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
            )
        {
            continue;
        }
        return Err(failure("receipt barrier readiness failed"));
    }
    let mut byte = 0_u8;
    loop {
        barrier_poll(fd, libc::POLLIN, deadline)?;
        let count = unsafe { libc::recv(fd, (&mut byte as *mut u8).cast(), 1, libc::MSG_DONTWAIT) };
        if count == 1 {
            if byte != b'B' {
                return Err(invalid("receipt barrier acknowledgement invalid"));
            }
            break;
        }
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "receipt barrier acknowledgement missing",
            ));
        }
        let error = io::Error::last_os_error();
        if matches!(
            error.kind(),
            io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
        ) {
            continue;
        }
        return Err(error);
    }
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "receipt barrier deadline",
        ));
    }
    loop {
        let count = unsafe {
            libc::recv(
                fd,
                (&mut byte as *mut u8).cast(),
                1,
                libc::MSG_PEEK | libc::MSG_DONTWAIT,
            )
        };
        if count > 0 {
            return Err(invalid("receipt barrier extra acknowledgement"));
        }
        if count == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::WouldBlock {
            return Ok(());
        }
        if error.kind() == io::ErrorKind::Interrupted && Instant::now() < deadline {
            continue;
        }
        return Err(error);
    }
}
struct GatedStart<'a> {
    slice: &'a str,
    keeper: &'a str,
    keeper_command: &'a [CString],
    keeper_input: &'a OwnedFd,
}
fn names_property(message: &Message<'_>, name: &str, values: &[CString]) -> io::Result<()> {
    message.open(b'r', c"sv")?;
    message.string(name)?;
    message.open(b'v', c"as")?;
    message.strings(values)?;
    message.close()?;
    message.close()
}
fn start(
    bus: &Bus<'_>,
    unit: &str,
    command: &[CString],
    env: &[CString],
    control: &OwnedFd,
    gated: Option<&GatedStart<'_>>,
) -> io::Result<()> {
    let slice = gated.map(|g| g.slice);
    let message = bus.method(
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "StartTransientUnit",
    )?;
    message.string(unit)?;
    message.string("fail")?;
    message.open(b'a', c"(sv)")?;
    for (name, value) in [
        ("Type", c"exec"),
        ("KillMode", c"control-group"),
        ("Restart", c"no"),
        ("CollectMode", c"inactive-or-failed"),
    ] {
        message.text_property(name, value)?;
    }
    if let Some(slice) = slice {
        message.text_property(
            "Slice",
            &CString::new(slice).map_err(|_| invalid("reader container name invalid"))?,
        )?;
        message
            .scalar_property("Delegate", b'b', c"b", (&0_i32 as *const i32).cast())
            .map_err(|e| io::Error::new(e.kind(), format!("reader Delegate encoding: {e}")))?;
    }
    if let Some(gated) = gated {
        let values = strings([gated.keeper])?;
        names_property(&message, "Wants", &values)?;
        names_property(&message, "After", &values)?;
    }
    for (name, value) in [
        ("AddRef", 1 as c_int),
        ("SendSIGKILL", 1 as c_int),
        ("RemainAfterExit", 0 as c_int),
    ] {
        message.scalar_property(name, b'b', c"b", (&value as *const c_int).cast())?;
    }
    for (name, value) in [
        ("TimeoutStartUSec", 10_000_000u64),
        ("TimeoutStopUSec", 2_000_000u64),
    ] {
        message.scalar_property(name, b't', c"t", (&value as *const u64).cast())?;
    }
    for (name, value) in [
        ("StandardInputFileDescriptor", control.as_raw_fd()),
        ("StandardOutputFileDescriptor", 1),
        ("StandardErrorFileDescriptor", 2),
    ] {
        message.scalar_property(name, b'h', c"h", (&value as *const c_int).cast())?;
    }
    message.open(b'r', c"sv")?;
    message.string("Environment")?;
    message.open(b'v', c"as")?;
    message.strings(env)?;
    message.close()?;
    message.close()?;
    message.open(b'r', c"sv")?;
    message.string("ExecStartEx")?;
    message.open(b'v', c"a(sasas)")?;
    message.open(b'a', c"(sasas)")?;
    message.open(b'r', c"sasas")?;
    message.basic(b's', command[0].as_ptr().cast())?;
    message.strings(command)?;
    message.open(b'a', c"s")?;
    message.string("no-env-expand")?;
    message.close()?;
    for _ in 0..4 {
        message.close()?;
    }
    message.close()?;
    message.open(b'a', c"(sa(sv))")?;
    if let Some(slice) = slice {
        message.open(b'r', c"sa(sv)")?;
        message.string(slice)?;
        message.open(b'a', c"(sv)")?;
        message.text_property("CollectMode", c"inactive-or-failed")?;
        for (name, value) in [("AddRef", 1_i32), ("StopWhenUnneeded", 1_i32)] {
            message.scalar_property(name, b'b', c"b", (&value as *const i32).cast())?;
        }
        message.close()?;
        message.close()?;
    }
    if let Some(gated) = gated {
        let null = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;
        message.open(b'r', c"sa(sv)")?;
        message.string(gated.keeper)?;
        message.open(b'a', c"(sv)")?;
        for (name, value) in [
            ("Type", c"exec"),
            ("Slice", c"app.slice"),
            ("KillMode", c"control-group"),
            ("Restart", c"no"),
            ("CollectMode", c"inactive-or-failed"),
        ] {
            message.text_property(name, value)?;
        }
        for (name, value) in [
            ("AddRef", 1_i32),
            ("RemainAfterExit", 0_i32),
            ("Delegate", 0_i32),
        ] {
            message.scalar_property(name, b'b', c"b", (&value as *const i32).cast())?;
        }
        for (name, value) in [
            ("TimeoutStartUSec", 10_000_000_u64),
            ("TimeoutStopUSec", 2_000_000_u64),
        ] {
            message.scalar_property(name, b't', c"t", (&value as *const u64).cast())?;
        }
        for (name, value) in [
            (
                "StandardInputFileDescriptor",
                gated.keeper_input.as_raw_fd(),
            ),
            ("StandardOutputFileDescriptor", null.as_raw_fd()),
            ("StandardErrorFileDescriptor", null.as_raw_fd()),
        ] {
            message.scalar_property(name, b'h', c"h", (&value as *const c_int).cast())?;
        }
        let values = strings([gated.slice])?;
        names_property(&message, "Requires", &values)?;
        names_property(&message, "After", &values)?;
        message.open(b'r', c"sv")?;
        message.string("ExecStartEx")?;
        message.open(b'v', c"a(sasas)")?;
        message.open(b'a', c"(sasas)")?;
        message.open(b'r', c"sasas")?;
        message.basic(b's', gated.keeper_command[0].as_ptr().cast())?;
        message.strings(gated.keeper_command)?;
        message.open(b'a', c"s")?;
        message.string("no-env-expand")?;
        message.close()?;
        for _ in 0..4 {
            message.close()?;
        }
        message.close()?;
        message.close()?;
    }
    message.close()?;
    let reply = bus
        .call(&message)
        .map_err(|e| io::Error::new(e.kind(), format!("service StartTransientUnit call: {e}")))?;
    let job = reply.read_string(b'o')?;
    if !job.starts_with("/org/freedesktop/systemd1/job/") {
        return Err(failure("user-service start job identity invalid"));
    }
    // message and reply are dropped here, including their copied Unix FDs.
    Ok(())
}
fn get_unit(bus: &Bus<'_>, unit: &str) -> io::Result<String> {
    let request = bus.method(
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "GetUnit",
    )?;
    request.string(unit)?;
    let path = bus.call(&request)?.read_string(b'o')?;
    if !path.starts_with("/org/freedesktop/systemd1/unit/") {
        return Err(failure("user-service object identity invalid"));
    }
    Ok(path)
}
fn group_path(group: &str, unit: &str) -> io::Result<PathBuf> {
    let path = Path::new(group);
    if group.len() > MAX_PROPERTY_BYTES
        || !path.is_absolute()
        || !group.ends_with(&format!("/{unit}"))
        || path
            .components()
            .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
    {
        return Err(failure("user-service control group identity invalid"));
    }
    Ok(Path::new("/sys/fs/cgroup").join(group.trim_start_matches('/')))
}
fn bounded_file(path: &Path, limit: usize) -> io::Result<String> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(failure("user-service identity file exceeds limit"));
    }
    String::from_utf8(bytes).map_err(|_| failure("user-service identity file invalid"))
}
fn empty_group(path: &Path) -> io::Result<bool> {
    match bounded_file(&path.join("cgroup.events"), 4096) {
        Ok(events) => {
            if !events.lines().any(|line| line == "populated 0") {
                return Ok(false);
            }
            Ok(bounded_file(&path.join("cgroup.procs"), 65536)?
                .trim()
                .is_empty())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            match std::fs::symlink_metadata(path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
                _ => Err(failure("user-service cgroup evidence missing")),
            }
        }
        Err(error) => Err(error),
    }
}
fn status(code: i32, value: i32) -> io::Result<ExitStatus> {
    match code {
        libc::CLD_EXITED if (0..=255).contains(&value) => Ok(ExitStatus::from_raw(value << 8)),
        libc::CLD_KILLED if (1..=64).contains(&value) => Ok(ExitStatus::from_raw(value)),
        libc::CLD_DUMPED if (1..=64).contains(&value) => Ok(ExitStatus::from_raw(value | 0x80)),
        _ => Err(failure("user-service terminal process status invalid")),
    }
}
fn stop_owned(bus: &Bus<'_>, unit: &str, path: &str, identity: &[u8; 16]) -> io::Result<()> {
    if &bus.invocation(path)? != identity {
        return Err(failure("user-service invocation identity changed"));
    }
    let message = bus.method(
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "StopUnit",
    )?;
    message.string(unit)?;
    message.string("replace")?;
    let _reply = bus.call(&message)?;
    Ok(())
}
fn wait_cleanup(bus: &Bus<'_>, unit: &str, path: &str, identity: &[u8; 16]) -> io::Result<()> {
    let started = Instant::now();
    loop {
        if &bus.invocation(path)? != identity {
            return Err(failure("user-service cleanup invocation changed"));
        }
        let active = bus.text(path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
        if active == "inactive" || active == "failed" {
            let group = bus.text(path, "org.freedesktop.systemd1.Service", "ControlGroup")?;
            if group.is_empty() || empty_group(&group_path(&group, unit)?)? {
                return Ok(());
            }
        }
        if started.elapsed() >= CLEANUP_GRACE {
            return Err(failure("user-service cleanup timed out"));
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn unref(bus: &Bus<'_>, path: &str) -> io::Result<()> {
    let message = bus.method(path, "org.freedesktop.systemd1.Unit", "Unref")?;
    let _reply = bus.call(&message)?;
    Ok(())
}
// A numeric MainPID is never sufficient to identify a live process.
struct MainIdentity {
    pid: u32,
    start_ticks: u64,
    descriptor: OwnedFd,
}
fn start_ticks_from_stat(value: &str) -> io::Result<u64> {
    let (_, fields) = value
        .rsplit_once(')')
        .ok_or_else(|| failure("guardian stat framing invalid"))?;
    fields
        .split_whitespace()
        .nth(19)
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| failure("guardian start ticks invalid"))
}
fn start_ticks(pid: u32) -> io::Result<u64> {
    start_ticks_from_stat(&bounded_file(
        &PathBuf::from(format!("/proc/{pid}/stat")),
        8192,
    )?)
}
impl MainIdentity {
    fn capture(pid: u32) -> io::Result<Self> {
        let before = start_ticks(pid)?;
        let descriptor = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as c_int, 0u32) };
        if descriptor < 0 {
            let error = io::Error::last_os_error();
            return Err(if error.raw_os_error() == Some(libc::ESRCH) {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "guardian exited during identity capture",
                )
            } else {
                error
            });
        }
        // pidfd_open returned a fresh descriptor owned only by this object.
        let descriptor = unsafe { OwnedFd::from_raw_fd(descriptor as c_int) };
        let identity = Self {
            pid,
            start_ticks: before,
            descriptor,
        };
        if !identity.matches_live()? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "guardian exited during identity capture",
            ));
        }
        Ok(identity)
    }
    fn matches_live(&self) -> io::Result<bool> {
        let mut event = libc::pollfd {
            fd: self.descriptor.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let code = unsafe { libc::poll(&mut event, 1, 0) };
        if code < 0 {
            return Err(io::Error::last_os_error());
        }
        if event.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err(failure("guardian pidfd validation failed"));
        }
        if event.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
            return Ok(false);
        }
        match start_ticks(self.pid) {
            Ok(value) if value == self.start_ticks => Ok(true),
            Ok(_) => Err(failure("guardian process identity changed")),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}
fn missing_guardian(since: &mut Option<Instant>) -> io::Result<()> {
    if since.get_or_insert_with(Instant::now).elapsed() >= CLEANUP_GRACE {
        return Err(failure("user-service guardian identity unavailable"));
    }
    thread::sleep(Duration::from_millis(20));
    Ok(())
}
fn wait_unit(bus: &Bus<'_>, unit: &str, path: &str, identity: [u8; 16]) -> io::Result<ExitStatus> {
    wait_unit_seeded(bus, unit, path, identity, None, None, false)
}
fn wait_unit_seeded(
    bus: &Bus<'_>,
    unit: &str,
    path: &str,
    identity: [u8; 16],
    mut known_group: Option<PathBuf>,
    mut known_pid: Option<MainIdentity>,
    gated: bool,
) -> io::Result<ExitStatus> {
    let mut transition_deadline = None;
    let mut missing_pid_since = None;
    loop {
        if bus.invocation(path)? != identity {
            gated_trace("user-service-invocation-identity-changed", None);
            return Err(failure("user-service invocation identity changed"));
        }
        let active = bus.text(path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
        let group = bus.text(path, "org.freedesktop.systemd1.Service", "ControlGroup")?;
        if !group.is_empty() {
            let actual = group_path(&group, unit)?;
            if known_group
                .as_ref()
                .is_some_and(|previous| previous != &actual)
            {
                gated_trace("user-service-control-group-changed", None);
                return Err(failure("user-service control group changed"));
            }
            known_group = Some(actual);
        }
        let exited = gated
            && active == "active"
            && bus.text(path, "org.freedesktop.systemd1.Unit", "SubState")? == "exited"
            && bus.number_u(path, "MainPID")? == 0;
        if active == "inactive" || active == "failed" || exited {
            if gated {
                let main = known_pid
                    .as_ref()
                    .ok_or_else(|| failure("reader terminal guardian identity missing"))?;
                if main.matches_live()? {
                    gated_trace("reader-terminal-guardian-still-live", None);
                    return Err(failure("reader terminal guardian still live"));
                }
            }
            let result = bus.text(path, "org.freedesktop.systemd1.Service", "Result")?;
            if !matches!(
                result.as_str(),
                "success" | "exit-code" | "signal" | "core-dump"
            ) {
                gated_trace("user-service-execution-protection-failed", None);
                return Err(failure("user-service execution protection failed"));
            }
            let terminal_pid = bus.number_u(path, "ExecMainPID")?;
            if terminal_pid <= 1
                || known_pid
                    .as_ref()
                    .is_some_and(|previous| previous.pid != terminal_pid)
            {
                gated_trace("user-service-terminal-guardian-identity-missing", None);
                return Err(failure("user-service terminal guardian identity missing"));
            }
            let status = status(
                bus.number_i(path, "ExecMainCode")?,
                bus.number_i(path, "ExecMainStatus")?,
            )?;
            let group = known_group
                .as_ref()
                .ok_or_else(|| failure("user-service cgroup identity never observed"))?;
            let started = Instant::now();
            while !empty_group(group)? {
                if started.elapsed() >= CLEANUP_GRACE {
                    gated_trace("user-service-cgroup-cleanup-incomplete", None);
                    return Err(failure("user-service cgroup cleanup incomplete"));
                }
                thread::sleep(Duration::from_millis(20));
            }
            if bus.invocation(path)? != identity {
                gated_trace("user-service-terminal-invocation-changed", None);
                return Err(failure("user-service terminal invocation changed"));
            }
            return Ok(status);
        }
        if !matches!(active.as_str(), "activating" | "active" | "deactivating") {
            gated_trace("user-service-lifecycle-state-invalid", None);
            return Err(failure("user-service lifecycle state invalid"));
        }
        if active != "active" {
            let deadline = transition_deadline.get_or_insert_with(Instant::now);
            if deadline.elapsed() > Duration::from_secs(15) {
                gated_trace("user-service-lifecycle-transition-timed-out", None);
                return Err(failure("user-service lifecycle transition timed out"));
            }
        } else {
            transition_deadline = None;
        }
        let pid = bus.number_u(path, "MainPID")?;
        if active == "active" {
            if pid <= 1 {
                missing_guardian(&mut missing_pid_since)?;
                continue;
            }
            if known_pid
                .as_ref()
                .is_some_and(|previous| previous.pid != pid)
            {
                gated_trace("user-service-guardian-main-PID-changed", None);
                return Err(failure("user-service guardian main PID changed"));
            }
            if known_pid.is_none() {
                match MainIdentity::capture(pid) {
                    Ok(value) => known_pid = Some(value),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        missing_guardian(&mut missing_pid_since)?;
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            let identity = known_pid
                .as_ref()
                .ok_or_else(|| failure("guardian identity missing"))?;
            if !identity.matches_live()? {
                missing_guardian(&mut missing_pid_since)?;
                continue;
            }
            let membership = bounded_file(&PathBuf::from(format!("/proc/{pid}/cgroup")), 4096);
            match membership {
                Ok(value)
                    if value
                        .lines()
                        .any(|line| line.strip_prefix("0::") == Some(group.as_str())) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    missing_guardian(&mut missing_pid_since)?;
                    continue;
                }
                _ => return Err(failure("user-service guardian cgroup membership invalid")),
            }
            if !identity.matches_live()? {
                missing_guardian(&mut missing_pid_since)?;
                continue;
            }
            // Reset only after the complete identity + membership check succeeded.
            missing_pid_since = None;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

// Dedicated synthetic fixtures can fail at an accepted-unit startup boundary.
// Release builds never consult this environment variable.
fn startup_fault(stage: &str) -> io::Result<()> {
    #[cfg(debug_assertions)]
    if startup_fault_selected(
        std::env::var_os("DLPAD_TEST_SYSTEMD_STARTUP_FAULT").as_deref(),
        stage,
    ) {
        // Give the isolated fixture time to capture the real unit/process identity.
        thread::sleep(Duration::from_secs(1));
        return Err(failure("user-service synthetic startup failure"));
    }
    #[cfg(not(debug_assertions))]
    let _ = stage;
    Ok(())
}
#[cfg(any(debug_assertions, test))]
fn startup_fault_selected(value: Option<&OsStr>, stage: &str) -> bool {
    value.is_some_and(|value| value == OsStr::new(stage))
}

pub(super) fn run(control: OwnedFd, guardian: &Path, args: &[OsString]) -> io::Result<ExitStatus> {
    if control.as_raw_fd() < 3 || !guardian.is_absolute() || args.is_empty() {
        return Err(invalid("user-service launch framing invalid"));
    }
    validate_control(&control)?;
    let path = bus_path()?;
    if args.len() > MAX_INPUT_COUNT - 3 {
        return Err(invalid("user-service argument count exceeds limit"));
    }
    let prefix = [
        guardian.as_os_str(),
        OsStr::new("--dlpad-internal-user-guardian"),
        OsStr::new("--"),
    ];
    let command = strings(
        prefix
            .into_iter()
            .chain(args.iter().map(OsString::as_os_str)),
    )?;
    let env = environment()?;
    let api = Api::load()?;
    let bus = Bus::connect(&api, &path)?;
    let unit = nonce_unit()?;
    start(&bus, &unit, &command, &env, &control, None)?;
    drop(control);
    startup_fault("after-start")?;
    let path = get_unit(&bus, &unit)?;
    startup_fault("after-get-unit")?;
    let identity = bus.invocation(&path)?;
    startup_fault("after-invocation")?;
    let result = wait_unit(&bus, &unit, &path, identity);
    let cleanup = if result.is_err() {
        // Never signal a PID or stop a reused invocation. Cleanup failure remains an error.
        stop_owned(&bus, &unit, &path, &identity)
            .and_then(|_| wait_cleanup(&bus, &unit, &path, &identity))
    } else {
        Ok(())
    };
    let released = if bus.invocation(&path)? == identity {
        unref(&bus, &path)
    } else {
        Err(failure("user-service release invocation changed"))
    };
    if cleanup.is_err() {
        return Err(failure("user-service protection cleanup failed"));
    }
    match (result, released) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

// A top-level owned slice preserves the original cgroup after the service exits.
// Its identity is never inferred from a missing service cgroup or reused unit name.
struct ReaderContainer<'a, 'b> {
    bus: &'a Bus<'b>,
    unit: String,
    path: String,
    invocation: [u8; 16],
    released: bool,
}
impl<'a, 'b> ReaderContainer<'a, 'b> {
    fn capture(bus: &'a Bus<'b>, unit: String) -> io::Result<Self> {
        let path = get_unit(bus, &unit)?;
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let active = bus.text(&path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
            if active == "active" {
                return Ok(Self {
                    bus,
                    unit,
                    invocation: bus.invocation(&path)?,
                    path,
                    released: false,
                });
            }
            if Instant::now() >= deadline || !matches!(active.as_str(), "activating" | "inactive") {
                return Err(failure("reader container activation unavailable"));
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    fn group(&self) -> io::Result<String> {
        if self.bus.invocation(&self.path)? != self.invocation {
            gated_trace("parent-invocation", None);
            return Err(failure("reader container invocation changed"));
        }
        if self
            .bus
            .text(&self.path, "org.freedesktop.systemd1.Unit", "ActiveState")?
            != "active"
        {
            gated_trace("parent-not-active", None);
            return Err(failure("reader container no longer active"));
        }
        let group = self
            .bus
            .text(&self.path, "org.freedesktop.systemd1.Slice", "ControlGroup")?;
        group_path(&group, &self.unit)?;
        Ok(group)
    }
    fn release(&mut self) -> io::Result<()> {
        if self.bus.invocation(&self.path)? != self.invocation {
            return Err(failure("reader container release identity changed"));
        }
        // The manager stops only when no active or queued dependency needs this slice.
        // Never send a parent Stop after a sampled empty observation.
        unref(self.bus, &self.path)?;
        self.released = true;
        Ok(())
    }
}
impl Drop for ReaderContainer<'_, '_> {
    fn drop(&mut self) {
        if !self.released && self.bus.invocation(&self.path).ok() == Some(self.invocation) {
            let _ = unref(self.bus, &self.path);
        }
    }
}
// Cleanup authority is the exact service Invocation, not every child of its slice.
struct ReaderService<'a, 'b> {
    bus: &'a Bus<'b>,
    unit: &'a str,
    path: &'a str,
    invocation: [u8; 16],
    released: bool,
}
impl ReaderService<'_, '_> {
    fn release(&mut self) -> io::Result<()> {
        if self.bus.invocation(self.path)? != self.invocation {
            return Err(failure("reader service release identity changed"));
        }
        stop_owned(self.bus, self.unit, self.path, &self.invocation)?;
        wait_cleanup(self.bus, self.unit, self.path, &self.invocation)?;
        unref(self.bus, self.path)?;
        self.released = true;
        Ok(())
    }
}
impl Drop for ReaderService<'_, '_> {
    fn drop(&mut self) {
        if !self.released && self.bus.invocation(self.path).ok() == Some(self.invocation) {
            let _ = stop_owned(self.bus, self.unit, self.path, &self.invocation)
                .and_then(|_| wait_cleanup(self.bus, self.unit, self.path, &self.invocation));
            if self.bus.invocation(self.path).ok() == Some(self.invocation) {
                let _ = unref(self.bus, self.path);
            }
        }
    }
}
fn keeper_alive(
    service: &ReaderService<'_, '_>,
    main: &MainIdentity,
    group: &str,
    pin: &super::reader_cgroup::PinnedCgroup,
) -> io::Result<()> {
    if service.bus.invocation(service.path)? != service.invocation
        || service
            .bus
            .text(service.path, "org.freedesktop.systemd1.Unit", "ActiveState")?
            != "active"
        || service.bus.number_u(service.path, "MainPID")? != main.pid
        || service.bus.text(
            service.path,
            "org.freedesktop.systemd1.Service",
            "ControlGroup",
        )? != group
        || service
            .bus
            .text(service.path, "org.freedesktop.systemd1.Service", "Slice")?
            != "app.slice"
        || service
            .bus
            .boolean(service.path, "org.freedesktop.systemd1.Service", "Delegate")?
        || !main.matches_live()?
    {
        return Err(failure("keeper live identity changed"));
    }
    let membership = bounded_file(
        &PathBuf::from(format!("/proc/{}/cgroup", main.pid)),
        MAX_PROPERTY_BYTES,
    )?;
    if membership
        .lines()
        .filter_map(|line| line.strip_prefix("0::"))
        .collect::<Vec<_>>()
        != [group]
        || super::reader_cgroup::PinnedCgroup::open(&group_path(group, service.unit)?)?.identity()
            != pin.identity()
        || !main.matches_live()?
    {
        return Err(failure("keeper native membership changed"));
    }
    Ok(())
}
fn capture_starting_invocation(bus: &Bus<'_>, unit: &str, path: &str) -> io::Result<[u8; 16]> {
    // This caller holds the original AddRef from a successful fail-mode creation.
    // A queued start has no InvocationID yet; that is never terminal evidence.
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if get_unit(bus, unit)? != path {
            return Err(failure("reader starting object changed"));
        }
        let active = bus.text(path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
        if !matches!(active.as_str(), "inactive" | "activating" | "active") {
            return Err(failure("reader starting state invalid"));
        }
        if let Some(identity) = bus.starting_invocation(path)? {
            return Ok(identity);
        }
        if Instant::now() >= deadline {
            return Err(failure("reader starting invocation timed out"));
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn nonce_bytes(nonce: &str) -> io::Result<[u8; 16]> {
    if nonce.len() != 32 {
        return Err(invalid("reader unit nonce invalid"));
    }
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&nonce[index * 2..index * 2 + 2], 16)
            .map_err(|_| invalid("reader unit nonce invalid"))?;
    }
    if bytes == [0; 16] {
        return Err(invalid("reader unit nonce invalid"));
    }
    Ok(bytes)
}
// Debug-only fixed diagnostics: no unit names, paths, command arguments or credentials.
fn gated_trace(stage: &'static str, error: Option<&io::Error>) {
    #[cfg(debug_assertions)]
    if std::env::var_os("DLPAD_TEST_CLEANUP_STAGE_TRACE").as_deref() == Some(OsStr::new("1")) {
        use std::io::Write;
        // Losing the app also closes its log pipe. Tracing must not interrupt cleanup.
        if let Some(error) = error {
            let _ = writeln!(
                io::stderr(),
                "dlpad-reader-service stage={stage} kind={:?} errno={:?}",
                error.kind(),
                error.raw_os_error()
            );
        } else {
            let _ = writeln!(io::stderr(), "dlpad-reader-service stage={stage}");
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (stage, error);
}

pub(super) fn run_gated(
    control: OwnedFd,
    guardian: &Path,
    args: &[OsString],
    capability: super::reader_protocol::Capability,
    mut receipts: super::reader_receipt::ReceiptHandles,
) -> io::Result<ExitStatus> {
    use super::{
        reader_cgroup::PinnedCgroup,
        reader_protocol::{CleanupReceipt, ReaderIdentity},
    };
    use std::os::unix::net::UnixStream;
    if control.as_raw_fd() < 3
        || !guardian.is_absolute()
        || args.is_empty()
        || args.len() > MAX_INPUT_COUNT - 5
    {
        return Err(invalid("cookie service launch framing invalid"));
    }
    validate_control(&control)?;
    #[cfg(debug_assertions)]
    let receipt_barrier = capture_receipt_barrier(&control)?;
    let api = Api::load()?;
    let bus = Bus::connect(&api, &bus_path()?)?;
    let unit = gated_nonce_unit()?;
    let nonce = unit
        .strip_prefix("dlpad-job-")
        .and_then(|v| v.strip_suffix(".service"))
        .ok_or_else(|| invalid("reader unit framing invalid"))?;
    let container_unit = format!("dlpadcookie{nonce}.slice");
    let index = capability.index.to_string();
    let begin_nonce: String = capability
        .nonce
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let prefix = [
        guardian.as_os_str(),
        OsStr::new("--dlpad-internal-cookie-user-guardian"),
        OsStr::new(&index),
        OsStr::new(&begin_nonce),
        OsStr::new("--"),
    ];
    let command = strings(
        prefix
            .into_iter()
            .chain(args.iter().map(OsString::as_os_str)),
    )?;
    let mut writer = UnixStream::from(control);
    let passed: OwnedFd = writer.try_clone()?.into();
    let mut pipe = [-1; 2];
    // Fresh anonymous pipe: the write end is never passed to the manager or a tool.
    if unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let keeper_input = unsafe { OwnedFd::from_raw_fd(pipe[0]) };
    let keeper_writer = unsafe { OwnedFd::from_raw_fd(pipe[1]) };
    let keeper_unit = format!("dlpadkeeper{nonce}.service");
    let keeper_command = strings([
        guardian.as_os_str(),
        OsStr::new("--dlpad-internal-cookie-keeper"),
    ])?;
    let gated = GatedStart {
        slice: &container_unit,
        keeper: &keeper_unit,
        keeper_command: &keeper_command,
        keeper_input: &keeper_input,
    };
    start(
        &bus,
        &unit,
        &command,
        &gated_environment()?,
        &passed,
        Some(&gated),
    )
    .map_err(|error| io::Error::new(error.kind(), format!("reader service setup: {error}")))?;
    drop(passed);
    drop(keeper_input);
    startup_fault("after-start")?;
    let path = get_unit(&bus, &unit)?;
    startup_fault("after-get-unit")?;
    let invocation = capture_starting_invocation(&bus, &unit, &path)?;
    let mut service = ReaderService {
        bus: &bus,
        unit: &unit,
        path: &path,
        invocation,
        released: false,
    };
    startup_fault("after-invocation")?;
    let keeper_path = get_unit(&bus, &keeper_unit)?;
    let keeper_invocation = capture_starting_invocation(&bus, &keeper_unit, &keeper_path)?;
    let mut keeper_service = ReaderService {
        bus: &bus,
        unit: &keeper_unit,
        path: &keeper_path,
        invocation: keeper_invocation,
        released: false,
    };
    let keeper_deadline = Instant::now() + Duration::from_secs(3);
    let (keeper_main, keeper_group, keeper_pin) = loop {
        if bus.invocation(&keeper_path)? != keeper_invocation {
            return Err(failure("keeper invocation changed"));
        }
        let active = bus.text(&keeper_path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
        let pid = bus.number_u(&keeper_path, "MainPID")?;
        let group = bus.text(
            &keeper_path,
            "org.freedesktop.systemd1.Service",
            "ControlGroup",
        )?;
        if active == "active" && pid > 1 && !group.is_empty() {
            let main = MainIdentity::capture(pid)?;
            let pin = PinnedCgroup::open(&group_path(&group, &keeper_unit)?)?;
            keeper_alive(&keeper_service, &main, &group, &pin)?;
            break (main, group, pin);
        }
        if Instant::now() >= keeper_deadline {
            return Err(failure("keeper native identity unavailable"));
        }
        thread::sleep(Duration::from_millis(20));
    };
    let mut container = ReaderContainer::capture(&bus, container_unit).map_err(|error| {
        io::Error::new(error.kind(), format!("reader container capture: {error}"))
    })?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let (main, service_group, parent_group, pinned) = loop {
        if bus.invocation(&path)? != invocation {
            return Err(failure("cookie service invocation changed"));
        }
        let pid = bus.number_u(&path, "MainPID")?;
        let group = bus.text(&path, "org.freedesktop.systemd1.Service", "ControlGroup")?;
        if pid > 1 && !group.is_empty() {
            let parent = container.group()?;
            if group != format!("{parent}/{unit}") {
                return Err(failure("reader container membership changed"));
            }
            let main = MainIdentity::capture(pid)?;
            let membership = bounded_file(
                &PathBuf::from(format!("/proc/{pid}/cgroup")),
                MAX_PROPERTY_BYTES,
            )?;
            if membership
                .lines()
                .filter_map(|line| line.strip_prefix("0::"))
                .collect::<Vec<_>>()
                != [group.as_str()]
            {
                return Err(failure("reader native membership changed"));
            }
            let pinned = PinnedCgroup::open(&group_path(&parent, &container.unit)?)?;
            if !main.matches_live()?
                || bus.invocation(&path)? != invocation
                || container.group()? != parent
            {
                return Err(failure("reader capture identity changed"));
            }
            break (main, group, parent, pinned);
        }
        if Instant::now() >= deadline {
            return Err(failure("reader native identity unavailable"));
        }
        thread::sleep(Duration::from_millis(20));
    };
    if keeper_group == parent_group || keeper_group.starts_with(&format!("{parent_group}/")) {
        return Err(failure("keeper unexpectedly inside reader parent"));
    }
    keeper_alive(&keeper_service, &keeper_main, &keeper_group, &keeper_pin)?;
    let native = pinned.identity();
    let mut identity = ReaderIdentity {
        capability,
        identity_digest: [1; 32],
        unit: nonce_bytes(nonce)?,
        invocation,
        cgroup_mount: native.mount_id,
        cgroup_dev: native.device,
        cgroup_ino: native.inode,
        path: parent_group.clone(),
    };
    identity.identity_digest = crate::cookies::reader_identity_digest(&identity)
        .map_err(|_| failure("reader identity digest unavailable"))?;
    let seed = MainIdentity {
        pid: main.pid,
        start_ticks: main.start_ticks,
        descriptor: main.descriptor.try_clone()?,
    };
    super::reader_gate::send_identity_only(
        &identity,
        &mut writer,
        Instant::now() + Duration::from_secs(3),
        None,
    )
    .map_err(|_| failure("reader identity exchange unavailable"))?;
    // The launcher never reads B or tool control. Only the service guardian reads them.
    drop(writer);
    let result = wait_unit_seeded(
        &bus,
        &unit,
        &path,
        invocation,
        Some(group_path(&service_group, &unit)?),
        Some(seed),
        true,
    );
    if let Err(error) = &result {
        gated_trace("wait-unit-error", Some(error));
        stop_owned(&bus, &unit, &path, &invocation)
            .inspect_err(|e| gated_trace("wait-error-stop", Some(e)))?;
        wait_cleanup(&bus, &unit, &path, &invocation)
            .inspect_err(|e| gated_trace("wait-error-cleanup", Some(e)))?;
    } else {
        gated_trace("wait-unit-terminal", None);
    }
    if main.matches_live()?
        || bus.invocation(&path)? != invocation
        || container.group()? != parent_group
    {
        return Err(failure("reader terminal native identity changed"));
    }
    pinned.confirm_empty()?;
    if main.matches_live()?
        || bus.invocation(&path)? != invocation
        || container.group()? != parent_group
    {
        return Err(failure("reader empty proof identity changed"));
    }
    keeper_alive(&keeper_service, &keeper_main, &keeper_group, &keeper_pin)?;
    receipts
        .write_receipt(&CleanupReceipt::from_identity(&identity))
        .map_err(io::Error::other)?;
    if bus.invocation(&path)? != invocation || container.group()? != parent_group {
        return Err(failure("reader receipt release identity changed"));
    }
    keeper_alive(&keeper_service, &keeper_main, &keeper_group, &keeper_pin)?;
    #[cfg(debug_assertions)]
    if let Some(barrier) = &receipt_barrier {
        receipt_barrier_exchange(barrier)?;
    }
    drop(keeper_writer);
    let keeper_eof_deadline = Instant::now() + CLEANUP_GRACE;
    loop {
        if bus.invocation(&keeper_path)? != keeper_invocation {
            return Err(failure("keeper EOF invocation changed"));
        }
        let active = bus.text(&keeper_path, "org.freedesktop.systemd1.Unit", "ActiveState")?;
        if matches!(active.as_str(), "inactive" | "failed") && !keeper_main.matches_live()? {
            break;
        }
        if Instant::now() >= keeper_eof_deadline {
            return Err(failure("keeper EOF termination timed out"));
        }
        thread::sleep(Duration::from_millis(20));
    }
    let keeper_seed = MainIdentity {
        pid: keeper_main.pid,
        start_ticks: keeper_main.start_ticks,
        descriptor: keeper_main.descriptor.try_clone()?,
    };
    let keeper_status = wait_unit_seeded(
        &bus,
        &keeper_unit,
        &keeper_path,
        keeper_invocation,
        Some(group_path(&keeper_group, &keeper_unit)?),
        Some(keeper_seed),
        false,
    )?;
    if !keeper_status.success() || keeper_main.matches_live()? {
        return Err(failure("keeper EOF termination unconfirmed"));
    }
    if bus.invocation(&keeper_path)? != keeper_invocation {
        return Err(failure("keeper release identity changed"));
    }
    unref(&bus, &keeper_path)?;
    keeper_service.released = true;
    service.release()?;
    container.release()?;
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn receipt_barrier_parser_and_native_alias_are_fail_closed() {
        use std::os::unix::net::UnixStream;
        assert_eq!(receipt_barrier_fd(OsStr::new("3")).unwrap(), 3);
        for value in [
            "",
            "0",
            "1",
            "2",
            "-3",
            "+3",
            " 3",
            "3x",
            "2147483648",
            "99999999999",
        ] {
            assert!(receipt_barrier_fd(OsStr::new(value)).is_err());
        }
        assert!(receipt_barrier_fd(&OsString::from_vec(vec![0xff])).is_err());
        let (control, _control_peer) = UnixStream::pair().unwrap();
        let (barrier, _barrier_peer) = UnixStream::pair().unwrap();
        validate_receipt_barrier(barrier.as_raw_fd(), control.as_raw_fd()).unwrap();
        assert_ne!(
            unsafe { libc::fcntl(barrier.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
            0
        );
        assert!(validate_receipt_barrier(control.as_raw_fd(), control.as_raw_fd()).is_err());
        let alias = control.try_clone().unwrap();
        assert!(validate_receipt_barrier(alias.as_raw_fd(), control.as_raw_fd()).is_err());
        let file = File::open("/dev/null").unwrap();
        assert!(validate_receipt_barrier(file.as_raw_fd(), control.as_raw_fd()).is_err());
        let mut pipe = [-1; 2];
        assert_eq!(
            unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) },
            0
        );
        let read = unsafe { OwnedFd::from_raw_fd(pipe[0]) };
        let _write = unsafe { OwnedFd::from_raw_fd(pipe[1]) };
        assert!(validate_receipt_barrier(read.as_raw_fd(), control.as_raw_fd()).is_err());
        assert!(validate_receipt_barrier(-1, control.as_raw_fd()).is_err());
    }
    #[test]
    fn receipt_barrier_one_byte_protocol_rejects_eof_wrong_and_extra() {
        use std::io::Write;
        use std::os::unix::net::UnixStream;
        for reply in [
            b"B".as_slice(),
            b"X".as_slice(),
            b"BB".as_slice(),
            b"".as_slice(),
        ] {
            let (socket, mut peer) = UnixStream::pair().unwrap();
            let socket: OwnedFd = socket.into();
            let reply = reply.to_vec();
            let valid = reply == b"B";
            let controller = thread::spawn(move || {
                peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut ready = [0_u8; 1];
                peer.read_exact(&mut ready).unwrap();
                assert_eq!(ready, *b"K");
                peer.write_all(&reply).unwrap();
            });
            assert_eq!(receipt_barrier_exchange(&socket).is_ok(), valid);
            controller.join().unwrap();
        }
    }
    #[test]
    fn receipt_barrier_missing_ack_has_absolute_two_second_budget() {
        use std::os::unix::net::UnixStream;
        let (socket, mut peer) = UnixStream::pair().unwrap();
        let socket: OwnedFd = socket.into();
        let controller = thread::spawn(move || {
            let mut ready = [0_u8; 1];
            peer.read_exact(&mut ready).unwrap();
            assert_eq!(ready, *b"K");
            // Hold the peer without adding a thread sleep or arbitrary timeout tolerance.
            let mut byte = [0_u8; 1];
            peer.read(&mut byte).unwrap()
        });
        let start = Instant::now();
        assert_eq!(
            receipt_barrier_exchange(&socket).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() >= Duration::from_secs(2));
        drop(socket);
        assert_eq!(controller.join().unwrap(), 0);
    }
    #[test]
    fn gated_debug_nonce_is_exact_bounded_and_path_free() {
        let value = "1234567890abcdef1234567890ABCDEF";
        assert_eq!(
            test_nonce_unit(OsStr::new(value)).unwrap(),
            "dlpad-job-1234567890abcdef1234567890abcdef.service"
        );
        for invalid in [
            "",
            "00000000000000000000000000000000",
            "../1234567890abcdef1234567890abcde",
            "1234567890abcdef1234567890abcdef0",
            "1234567890abcdef1234567890abcdef000",
            "1234567890abcdef1234567890abcdef0z",
        ] {
            assert!(test_nonce_unit(OsStr::new(invalid)).is_err());
        }
        assert!(test_nonce_unit(&OsString::from_vec(vec![0xff; 32])).is_err());
    }
    #[test]
    fn synthetic_startup_failure_selection_is_exact() {
        for stage in ["after-start", "after-get-unit", "after-invocation"] {
            assert!(startup_fault_selected(Some(OsStr::new(stage)), stage));
            assert!(!startup_fault_selected(None, stage));
            assert!(!startup_fault_selected(Some(OsStr::new("unknown")), stage));
            assert!(!startup_fault_selected(
                Some(OsStr::new("after-start ")),
                stage
            ));
        }
        assert!(!startup_fault_selected(
            Some(OsStr::new("after-start")),
            "after-get-unit"
        ));
    }

    use super::*;
    use std::os::unix::ffi::OsStringExt;
    #[test]
    fn literal_and_invalid_argv_are_bounded() {
        let values = vec![OsString::from("$VAR ${VAR} %n 한글 quote'\"literal")];
        assert_eq!(
            strings(values.clone()).unwrap()[0].to_bytes(),
            values[0].to_str().unwrap().as_bytes()
        );
        for value in [
            OsString::from_vec(vec![0xff]),
            OsString::from("NUL\0suffix"),
            OsString::from("x".repeat(MAX_INPUT_BYTES + 1)),
        ] {
            assert!(strings([value]).is_err());
        }
        assert!(strings(vec![OsString::from("x"); MAX_INPUT_COUNT + 1]).is_err());
    }
    #[test]
    fn environment_and_bus_address_reject_lossy_or_injectable_data() {
        let values = [(
            OsString::from("DLPAD_LITERAL"),
            OsString::from("$VAR %n 한글 secret-fixture"),
        )];
        assert_eq!(
            environment_entries(values).unwrap()[0].to_bytes(),
            "DLPAD_LITERAL=$VAR %n 한글 secret-fixture".as_bytes()
        );
        for (key, value) in [
            (OsString::from("bad=key"), OsString::from("fixture-secret")),
            (
                OsString::from("CARGO_BIN_EXE_dlpad-process-guard"),
                OsString::from("fixture-secret"),
            ),
            (OsString::from("1INVALID"), OsString::from("fixture-secret")),
            (OsString::from("x"), OsString::from_vec(vec![0xff])),
            (OsString::from("x"), OsString::from("NUL\0")),
        ] {
            let error = environment_entries([(key, value)]).unwrap_err();
            assert!(!error.to_string().contains("fixture-secret"));
        }
        assert_eq!(
            bus_address(Path::new("/tmp/comma,semi;equals=percent%/bus"))
                .unwrap()
                .to_bytes(),
            b"unix:path=/tmp/comma%2Csemi%3Bequals%3Dpercent%25/bus"
        );
    }
    #[test]
    fn control_fd_must_be_unix_stream_and_private() {
        let (left, _right) = std::os::unix::net::UnixStream::pair().unwrap();
        let fd: OwnedFd = left.into();
        validate_control(&fd).unwrap();
        assert_ne!(
            unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
            0
        );
        let file: OwnedFd = File::open("/dev/null").unwrap().into();
        assert!(validate_control(&file).is_err());
    }
    #[test]
    fn terminal_status_preserves_exit_and_signal_without_fake_success() {
        assert_eq!(status(libc::CLD_EXITED, 17).unwrap().code(), Some(17));
        assert_eq!(
            status(libc::CLD_KILLED, libc::SIGTERM).unwrap().signal(),
            Some(libc::SIGTERM)
        );
        assert!(
            status(libc::CLD_DUMPED, libc::SIGSEGV)
                .unwrap()
                .core_dumped()
        );
        for pair in [
            (0, 0),
            (libc::CLD_EXITED, 256),
            (libc::CLD_KILLED, 0),
            (libc::CLD_KILLED, 65),
        ] {
            assert!(status(pair.0, pair.1).is_err());
        }
    }
    #[test]
    fn control_group_identity_cannot_escape_owned_unit() {
        let unit = "dlpad-job-test.service";
        assert!(
            group_path(
                "/user.slice/user-1000.slice/user@1000.service/app.slice/dlpad-job-test.service",
                unit
            )
            .is_ok()
        );
        for path in [
            "/",
            "relative/dlpad-job-test.service",
            "/user.slice/../dlpad-job-test.service",
            "/user.slice/foreign.service",
        ] {
            assert!(group_path(path, unit).is_err());
        }
    }
}
