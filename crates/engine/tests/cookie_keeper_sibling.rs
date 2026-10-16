//! Product keeper release preserves separately owned live/pending siblings.
#![cfg(target_os = "linux")]
#[path = "../src/process/reader_protocol.rs"]
mod reader_protocol;
mod process {
    pub(crate) use crate::reader_protocol;
}
#[path = "../src/cookies/reader_records.rs"]
mod reader_records;
mod cookies {
    pub(crate) use crate::reader_records;
}
#[path = "../src/process/reader_cgroup.rs"]
mod reader_cgroup;
#[path = "../src/cookies/reader_journal.rs"]
mod reader_journal;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    net::Shutdown,
    os::{
        fd::AsRawFd,
        unix::{
            fs::{MetadataExt, PermissionsExt},
            net::UnixStream,
            process::CommandExt,
        },
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
const TOOL: &str = r#"import os,stat,sys
from pathlib import Path
receipt_dev,receipt_ino,lock_dev,lock_ino=map(int,sys.argv[2:6])
assert sys.stdin.read()==''
assert 'DLPAD_TEST_COOKIE_RECEIPT_BARRIER_FD' not in os.environ
for entry in os.listdir('/proc/self/fd'):
 if not entry.isdigit() or int(entry)<=2:continue
 try:info=os.fstat(int(entry))
 except OSError:continue
 assert not stat.S_ISSOCK(info.st_mode)
 assert (info.st_dev,info.st_ino) not in [(receipt_dev,receipt_ino),(lock_dev,lock_ino)]
assert Path(sys.argv[6]).read_bytes()==b'# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tFALSE\t0\tsynthetic\tfixture-only\n'
Path(sys.argv[1]).write_text('synthetic tool started; capability FDs absent')
"#;
const COOKIE: &[u8] =
    b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tFALSE\t0\tsynthetic\tfixture-only\n";
struct OwnedHelper {
    child: Child,
    shutdown: UnixStream,
}
impl OwnedHelper {
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "owned launcher terminal deadline exceeded"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for OwnedHelper {
    fn drop(&mut self) {
        let _ = self.shutdown.shutdown(Shutdown::Write);
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // Only this exact unreaped Child is signalled; unknown service identities are never stopped here.
        let _ = self.child.kill();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        eprintln!("owned synthetic launcher reap unconfirmed");
    }
}
fn bounded_output(command: &mut Command) -> (std::process::ExitStatus, Vec<u8>, Vec<u8>) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    fn drain(mut pipe: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
        std::thread::spawn(move || {
            let mut retained = Vec::new();
            let mut block = [0; 4096];
            let mut oversized = false;
            loop {
                let count = pipe.read(&mut block).unwrap();
                if count == 0 {
                    break;
                }
                let keep = count.min(65536_usize.saturating_sub(retained.len()));
                oversized |= keep != count;
                retained.extend_from_slice(&block[..keep]);
            }
            assert!(!oversized, "manager query output exceeded bound");
            retained
        })
    }
    let stdout = drain(child.stdout.take().unwrap());
    let stderr = drain(child.stderr.take().unwrap());
    let deadline = Instant::now() + Duration::from_secs(3);
    let (status, expired) = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break (status, false);
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            break (child.wait().unwrap(), true);
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = stdout.join().unwrap();
    let error = stderr.join().unwrap();
    assert!(!expired, "owned manager query deadline exceeded");
    (status, output, error)
}
struct NativeProof {
    service: String,
    parent: String,
    pid: u32,
    start: u64,
    pidfd: File,
    _container: reader_cgroup::PinnedCgroup,
}
impl NativeProof {
    fn terminal(&self) {
        assert_ne!(
            native_start(self.pid),
            Some(self.start),
            "owned guardian still present"
        );
        let mut event = libc::pollfd {
            fd: self.pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: poll addresses one live held pidfd and does not wait.
        assert_eq!(unsafe { libc::poll(&mut event, 1, 0) }, 1);
        wait_unit_gone(&self.service);
        wait_unit_gone(&self.parent);
    }
}
fn manager_env(command: &mut Command) {
    let uid = unsafe { libc::geteuid() };
    let runtime = format!("/run/user/{uid}");
    for name in ["bus", "systemd/private"] {
        let metadata = fs::symlink_metadata(format!("{runtime}/{name}")).unwrap();
        assert_eq!(metadata.uid(), uid);
        use std::os::unix::fs::FileTypeExt;
        assert!(metadata.file_type().is_socket());
    }
    command
        .env("XDG_RUNTIME_DIR", &runtime)
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={runtime}/bus"),
        )
        .env("LC_ALL", "C");
}
fn properties(unit: &str) -> HashMap<String, String> {
    let mut command = Command::new("/usr/bin/systemctl");
    command.args(["--user", "--no-ask-password", "--no-pager", "show", unit]);
    for key in [
        "InvocationID",
        "MainPID",
        "Slice",
        "ControlGroup",
        "Delegate",
        "ActiveState",
        "SubState",
        "Job",
        "ExecMainCode",
        "ExecMainStatus",
        "Result",
    ] {
        command.arg(format!("--property={key}"));
    }
    manager_env(&mut command);
    let (status, output, error) = bounded_output(&mut command);
    assert!(status.success() && output.len() <= 65536 && error.len() <= 4096);
    String::from_utf8(output)
        .unwrap()
        .lines()
        .filter_map(|line| {
            line.split_once('=')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
        })
        .collect()
}
fn native_start(pid: u32) -> Option<u64> {
    let text = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    assert!(text.len() <= 4096);
    text.rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn boot() -> [u8; 16] {
    let text = fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap();
    let compact = text.trim().replace('-', "");
    assert_eq!(compact.len(), 32);
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&compact[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
fn receive_identity(app: &mut UnixStream) -> reader_protocol::ReaderIdentity {
    let mut header = [0; 8];
    let mut used = 0;
    while used < header.len() {
        match app.read(&mut header[used..]) {
            Ok(0) => panic!("native I header EOF after {used} bytes; no valid identity received"),
            Ok(count) => {
                used += count;
                let fixed = match &header[..used.min(2)] {
                    b"LU" => Some("LU"),
                    b"GU" => Some("GU"),
                    b"EP" => Some("EP"),
                    b"GC" => Some("GC"),
                    b"LC" => Some("LC"),
                    _ => None,
                };
                if let Some(frame) = fixed {
                    panic!(
                        "native I header rejected early fixed frame {frame}; no valid identity received"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => panic!(
                "native I header read failed after {used} bytes; kind={:?}",
                error.kind()
            ),
        }
    }
    let length = u16::from_le_bytes([header[6], header[7]]) as usize;
    assert!(length <= reader_protocol::MAX_PACKET - 8);
    let mut packet = header.to_vec();
    packet.resize(8 + length, 0);
    app.read_exact(&mut packet[8..]).unwrap();
    reader_protocol::decode_identity(&packet).unwrap()
}
fn verify_identity(
    identity: &reader_protocol::ReaderIdentity,
    expected: &reader_protocol::Capability,
) -> NativeProof {
    assert_eq!(&identity.capability, expected);
    assert_eq!(
        identity.identity_digest,
        reader_records::identity_digest(identity).unwrap()
    );
    let service = format!("dlpad-job-{}.service", hex(&identity.unit));
    let parent = Path::new(&identity.path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        parent.starts_with("dlpadcookie") && parent.ends_with(".slice") && !parent.contains('-')
    );
    let service_props = properties(&service);
    let parent_props = properties(&parent);
    assert_eq!(service_props["InvocationID"], hex(&identity.invocation));
    assert_eq!(service_props["Slice"], parent);
    assert_eq!(service_props["Delegate"], "no");
    assert_eq!(parent_props["ActiveState"], "active");
    assert_eq!(parent_props["ControlGroup"], identity.path);
    let parent_invocation = parent_props["InvocationID"].clone();
    assert_eq!(parent_invocation.len(), 32);
    let pid: u32 = service_props["MainPID"].parse().unwrap();
    assert!(pid > 1);
    let start = native_start(pid).expect("service guardian not live at I");
    let group = &service_props["ControlGroup"];
    assert_eq!(group, &format!("{}/{}", identity.path, service));
    let membership = fs::read_to_string(format!("/proc/{pid}/cgroup")).unwrap();
    assert!(membership.len() <= 4096);
    assert!(membership.lines().any(|line| line == format!("0::{group}")));
    // SAFETY: pidfd_open addresses only the live PID captured from this owned service.
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0_u32) } as i32;
    assert!(fd >= 0);
    use std::os::fd::FromRawFd;
    // SAFETY: syscall returned a fresh descriptor uniquely owned here.
    let pin = unsafe { File::from_raw_fd(fd) };
    let mut event = libc::pollfd {
        fd: pin.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: one live pollfd is passed with no wait.
    assert_eq!(unsafe { libc::poll(&mut event, 1, 0) }, 0);
    let pinned = reader_cgroup::PinnedCgroup::open(&PathBuf::from(format!(
        "/sys/fs/cgroup{}",
        identity.path
    )))
    .unwrap();
    let actual = pinned.identity();
    assert_eq!(
        (actual.mount_id, actual.device, actual.inode),
        (
            identity.cgroup_mount,
            identity.cgroup_dev,
            identity.cgroup_ino
        )
    );
    assert_eq!(actual.relative_path, identity.path);
    assert!(
        pinned.confirm_empty().is_err(),
        "live guardian descendant was mistaken for empty container"
    );
    assert_eq!(native_start(pid), Some(start));
    assert_eq!(
        properties(&service)["InvocationID"],
        hex(&identity.invocation)
    );
    assert_eq!(properties(&parent)["InvocationID"], parent_invocation);
    println!(
        "NATIVE_COOKIE_GATE_IDENTITY service={service} container={parent} pid={pid} start={start}"
    );
    NativeProof {
        service,
        parent,
        pid,
        start,
        pidfd: pin,
        _container: pinned,
    }
}
fn wait_unit_gone(unit: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let uid = unsafe { libc::geteuid() };
        let mut command = Command::new("/usr/bin/busctl");
        command
            .args([
                format!("--address=unix:path=/run/user/{uid}/bus"),
                "--timeout=2".into(),
                "call".into(),
                "org.freedesktop.systemd1".into(),
                "/org/freedesktop/systemd1".into(),
                "org.freedesktop.systemd1.Manager".into(),
                "GetUnit".into(),
                "s".into(),
                unit.into(),
            ])
            .env("LC_ALL", "C");
        let (status, output, stderr) = bounded_output(&mut command);
        assert!(output.len() <= 4096 && stderr.len() <= 4096);
        let error = String::from_utf8_lossy(&stderr);
        if !status.success() && error.contains("not loaded") {
            return;
        }
        assert!(Instant::now() < deadline, "owned unit GC not observed");
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn spawn(
    folder: &Path,
    launch: reader_journal::ReaderLaunch,
    marker: &Path,
    nonce: &str,
    fault: Option<&str>,
    barrier: &UnixStream,
) -> (OwnedHelper, UnixStream, reader_protocol::Capability) {
    let (app, peer) = UnixStream::pair().unwrap();
    let descriptors = [
        peer.as_raw_fd(),
        launch.writer.as_raw_fd(),
        launch.lock.as_raw_fd(),
        barrier.as_raw_fd(),
    ];
    assert!(descriptors.iter().all(|fd| *fd >= 3));
    assert_ne!(descriptors[0], descriptors[1]);
    assert_ne!(descriptors[0], descriptors[2]);
    assert_ne!(descriptors[1], descriptors[2]);
    let lock = launch.lock.metadata().unwrap();
    let capability = launch.capability.clone();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dlpad-process-guard"));
    command.arg("--dlpad-internal-cookie-user-launcher");
    for fd in &descriptors[..3] {
        command.arg(fd.to_string());
    }
    command
        .args([
            lock.dev().to_string(),
            lock.ino().to_string(),
            "--".into(),
            "/usr/bin/python3".into(),
            "-c".into(),
            TOOL.into(),
        ])
        .arg(marker)
        .args([
            capability.receipt_dev.to_string(),
            capability.receipt_ino.to_string(),
            lock.dev().to_string(),
            lock.ino().to_string(),
        ])
        .arg(folder.join("copy.cookies"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        // Synthetic-only helper: root captures bounded native logs; retain fixed setup errors.
        .stderr(Stdio::inherit());
    manager_env(&mut command);
    command.env("DLPAD_TEST_COOKIE_UNIT_NONCE", nonce);
    command.env(
        "DLPAD_TEST_COOKIE_RECEIPT_BARRIER_FD",
        barrier.as_raw_fd().to_string(),
    );
    if let Some(fault) = fault {
        command.env("DLPAD_TEST_SYSTEMD_STARTUP_FAULT", fault);
    } else {
        command.env_remove("DLPAD_TEST_SYSTEMD_STARTUP_FAULT");
    }
    // SAFETY: three live owned FDs only; no allocation or filesystem activity in pre_exec.
    unsafe {
        command.pre_exec(move || {
            for fd in descriptors {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    let child = command.spawn().unwrap();
    drop(peer);
    drop(launch);
    app.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    app.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut app = app;
    app.write_all(&reader_protocol::encode_capability(&capability).unwrap())
        .unwrap();
    let helper = OwnedHelper {
        child,
        shutdown: app.try_clone().unwrap(),
    };
    (helper, app, capability)
}
fn fixture() -> (tempfile::TempDir, reader_journal::ReaderJournal) {
    let folder = tempfile::tempdir().unwrap();
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o700)).unwrap();
    for name in ["source.cookies", "copy.cookies"] {
        fs::write(folder.path().join(name), COOKIE).unwrap();
        fs::set_permissions(folder.path().join(name), fs::Permissions::from_mode(0o600)).unwrap();
    }
    fs::write(folder.path().join("sentinel"), b"owned synthetic sentinel").unwrap();
    let journal =
        reader_journal::ReaderJournal::create(File::open(folder.path()).unwrap()).unwrap();
    (folder, journal)
}
fn preserved(folder: &Path) {
    assert_eq!(fs::read(folder.join("source.cookies")).unwrap(), COOKIE);
    assert_eq!(fs::read(folder.join("copy.cookies")).unwrap(), COOKIE);
    assert_eq!(
        fs::read(folder.join("sentinel")).unwrap(),
        b"owned synthetic sentinel"
    );
}

fn random_nonce() -> String {
    let mut bytes = [0; 16];
    File::open("/dev/urandom")
        .unwrap()
        .read_exact(&mut bytes)
        .unwrap();
    assert!(bytes.iter().any(|b| *b != 0));
    hex(&bytes)
}
fn unit_names(nonce: &str) -> [String; 3] {
    [
        format!("dlpad-job-{nonce}.service"),
        format!("dlpadkeeper{nonce}.service"),
        format!("dlpadcookie{nonce}.slice"),
    ]
}
fn preexisting_absent(unit: &str) {
    let uid = unsafe { libc::geteuid() };
    let mut command = Command::new("/usr/bin/busctl");
    command
        .args([
            format!("--address=unix:path=/run/user/{uid}/bus"),
            "--timeout=2".into(),
            "call".into(),
            "org.freedesktop.systemd1".into(),
            "/org/freedesktop/systemd1".into(),
            "org.freedesktop.systemd1.Manager".into(),
            "GetUnit".into(),
            "s".into(),
            unit.into(),
        ])
        .env("LC_ALL", "C");
    let (status, output, error) = bounded_output(&mut command);
    assert!(
        !status.success()
            && output.len() <= 4096
            && error.len() <= 4096
            && String::from_utf8_lossy(&error).contains("not loaded"),
        "fresh owned nonce unit was not confirmed absent"
    );
}

struct Sibling {
    child: Child,
    writer: Option<File>,
    unit: String,
    invocation: String,
    job: String,
    pid: u32,
    start: u64,
    pidfd: File,
}
impl Drop for Sibling {
    fn drop(&mut self) {
        self.writer.take();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // Only the exact unreaped systemd-run controller, never an inferred service PID.
        let _ = self.child.kill();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        eprintln!("owned sibling controller reap unconfirmed");
    }
}
fn sibling_prepare(parent: &str, pending: bool) -> (Command, File, String) {
    use std::os::fd::FromRawFd;
    let unit = format!("dlpad-sibling-{}.service", random_nonce());
    preexisting_absent(&unit);
    let mut fds = [-1; 2];
    assert_eq!(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
    let input = unsafe { File::from_raw_fd(fds[0]) };
    let writer = unsafe { File::from_raw_fd(fds[1]) };
    let mut command = Command::new("/usr/bin/systemd-run");
    command
        .args([
            "--user",
            "--no-ask-password",
            "--quiet",
            "--pipe",
            "--wait",
            "--collect",
        ])
        .arg(format!("--unit={unit}"))
        .arg(format!("--slice={parent}"))
        .arg(format!(
            "--property=Type={}",
            if pending { "oneshot" } else { "exec" }
        ))
        .args([
            "--property=KillMode=control-group",
            "--property=Restart=no",
            "--property=Delegate=no",
            "--property=TimeoutStartSec=10",
            "--property=TimeoutStopSec=2",
            "/usr/bin/python3",
            "-c",
            "import sys;assert sys.stdin.buffer.read()==b''",
        ])
        .stdin(Stdio::from(input))
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    manager_env(&mut command);
    (command, writer, unit)
}
fn sibling_start(
    prepared: (Command, File, String),
    parent: &str,
    group: &str,
    pending: bool,
) -> Sibling {
    use std::os::fd::FromRawFd;
    let (mut command, writer, unit) = prepared;
    let child = command.spawn().unwrap();
    let mut owned = Sibling {
        child,
        writer: Some(writer),
        unit: unit.clone(),
        invocation: String::new(),
        job: String::new(),
        pid: 0,
        start: 0,
        pidfd: File::open("/dev/null").unwrap(),
    };
    let deadline = Instant::now() + Duration::from_secs(1);
    let value = loop {
        let value = properties(&unit);
        if value.get("ActiveState").map(String::as_str)
            == Some(if pending { "activating" } else { "active" })
            && value
                .get("MainPID")
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|pid| pid > 1)
        {
            break value;
        }
        assert!(
            Instant::now() < deadline,
            "sibling actual state unavailable within barrier budget"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(value["Slice"], parent);
    assert_eq!(value["ControlGroup"], format!("{group}/{unit}"));
    assert_eq!(value["Delegate"], "no");
    let invocation = value["InvocationID"].clone();
    assert_eq!(invocation.len(), 32);
    let job = value.get("Job").cloned().unwrap_or_default();
    if pending {
        assert!(!job.is_empty() && job.split_whitespace().next() != Some("0"));
    }
    let pid = value["MainPID"].parse().unwrap();
    let start = native_start(pid).unwrap();
    assert!(
        fs::read_to_string(format!("/proc/{pid}/cgroup"))
            .unwrap()
            .lines()
            .any(|line| line == format!("0::{}", value["ControlGroup"]))
    );
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0_u32) } as i32;
    assert!(fd >= 0);
    let pidfd = unsafe { File::from_raw_fd(fd) };
    println!(
        "NATIVE_COOKIE_SIBLING unit={unit} invocation={invocation} pid={pid} start={start} job={job}"
    );
    owned.invocation = invocation;
    owned.job = job;
    owned.pid = pid;
    owned.start = start;
    owned.pidfd = pidfd;
    owned
}
// Fixture-only persistent bus Ref: an ephemeral busctl connection cannot own this lifetime.
#[repr(C)]
struct BusError {
    name: *const libc::c_char,
    message: *const libc::c_char,
    need_free: libc::c_int,
}
struct NativeLibrary(*mut libc::c_void);
impl Drop for NativeLibrary {
    fn drop(&mut self) {
        unsafe {
            libc::dlclose(self.0);
        }
    }
}
impl NativeLibrary {
    unsafe fn symbol<T: Copy>(&self, name: &std::ffi::CStr) -> T {
        let pointer = unsafe { libc::dlsym(self.0, name.as_ptr()) };
        assert!(!pointer.is_null(), "fixture native bus symbol missing");
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of_val(&pointer));
        unsafe { std::mem::transmute_copy(&pointer) }
    }
}
type BusCall = unsafe extern "C" fn(
    *mut libc::c_void,
    *const libc::c_char,
    *const libc::c_char,
    *const libc::c_char,
    *const libc::c_char,
    *mut BusError,
    *mut *mut libc::c_void,
    *const libc::c_char,
    ...
) -> libc::c_int;
struct RefBus {
    _library: NativeLibrary,
    bus: *mut libc::c_void,
    unref: unsafe extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    message_unref: unsafe extern "C" fn(*mut libc::c_void) -> *mut libc::c_void,
    error_free: unsafe extern "C" fn(*mut BusError),
    call: BusCall,
    read: unsafe extern "C" fn(*mut libc::c_void, libc::c_char, *mut libc::c_void) -> libc::c_int,
}
impl Drop for RefBus {
    fn drop(&mut self) {
        unsafe {
            (self.unref)(self.bus);
        }
    }
}
impl RefBus {
    fn connect() -> Self {
        let uid = unsafe { libc::geteuid() };
        let metadata = fs::symlink_metadata(format!("/run/user/{uid}/bus")).unwrap();
        use std::os::unix::fs::FileTypeExt;
        assert!(metadata.file_type().is_socket());
        assert_eq!(metadata.uid(), uid);
        let raw = unsafe {
            libc::dlopen(
                c"libsystemd.so.0".as_ptr(),
                libc::RTLD_NOW | libc::RTLD_LOCAL,
            )
        };
        assert!(!raw.is_null());
        let library = NativeLibrary(raw);
        let mut bus = std::ptr::null_mut();
        unsafe {
            let new: unsafe extern "C" fn(*mut *mut libc::c_void) -> libc::c_int =
                library.symbol(c"sd_bus_new");
            assert!(new(&mut bus) >= 0);
            let owned = Self {
                unref: library.symbol(c"sd_bus_unref"),
                message_unref: library.symbol(c"sd_bus_message_unref"),
                error_free: library.symbol(c"sd_bus_error_free"),
                call: library.symbol(c"sd_bus_call_method"),
                read: library.symbol(c"sd_bus_message_read_basic"),
                _library: library,
                bus,
            };
            let address: unsafe extern "C" fn(
                *mut libc::c_void,
                *const libc::c_char,
            ) -> libc::c_int = owned._library.symbol(c"sd_bus_set_address");
            let client: unsafe extern "C" fn(*mut libc::c_void, libc::c_int) -> libc::c_int =
                owned._library.symbol(c"sd_bus_set_bus_client");
            let start: unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int =
                owned._library.symbol(c"sd_bus_start");
            let timeout: unsafe extern "C" fn(*mut libc::c_void, u64) -> libc::c_int =
                owned._library.symbol(c"sd_bus_set_method_call_timeout");
            let value = std::ffi::CString::new(format!("unix:path=/run/user/{uid}/bus")).unwrap();
            assert!(address(owned.bus, value.as_ptr()) >= 0);
            assert!(client(owned.bus, 1) >= 0);
            assert!(timeout(owned.bus, 2_000_000) >= 0);
            assert!(start(owned.bus) >= 0);
            owned
        }
    }
    fn unit_path(&self, unit: &str) -> String {
        let unit = std::ffi::CString::new(unit).unwrap();
        let mut error = BusError {
            name: std::ptr::null(),
            message: std::ptr::null(),
            need_free: 0,
        };
        let mut reply = std::ptr::null_mut();
        let result = unsafe {
            (self.call)(
                self.bus,
                c"org.freedesktop.systemd1".as_ptr(),
                c"/org/freedesktop/systemd1".as_ptr(),
                c"org.freedesktop.systemd1.Manager".as_ptr(),
                c"GetUnit".as_ptr(),
                &mut error,
                &mut reply,
                c"s".as_ptr(),
                unit.as_ptr(),
            )
        };
        unsafe {
            (self.error_free)(&mut error);
        }
        if result < 0 {
            if !reply.is_null() {
                unsafe {
                    (self.message_unref)(reply);
                }
            }
            panic!("fixture bus GetUnit errno={}", -result);
        }
        let mut text: *const libc::c_char = std::ptr::null();
        let result = unsafe {
            (self.read)(
                reply,
                b'o' as libc::c_char,
                (&mut text as *mut *const libc::c_char).cast(),
            )
        };
        let path = if result > 0 && !text.is_null() {
            let length = unsafe { libc::strnlen(text, 1025) };
            if length <= 1024 {
                Some(
                    unsafe { std::ffi::CStr::from_ptr(text) }
                        .to_str()
                        .unwrap()
                        .to_owned(),
                )
            } else {
                None
            }
        } else {
            None
        };
        unsafe {
            (self.message_unref)(reply);
        }
        let path = path.expect("fixture unit path unavailable");
        assert!(path.starts_with("/org/freedesktop/systemd1/unit/"));
        path
    }
    fn reference(&self, path: &str, method: &std::ffi::CStr) {
        let path = std::ffi::CString::new(path).unwrap();
        let mut error = BusError {
            name: std::ptr::null(),
            message: std::ptr::null(),
            need_free: 0,
        };
        let mut reply = std::ptr::null_mut();
        let result = unsafe {
            (self.call)(
                self.bus,
                c"org.freedesktop.systemd1".as_ptr(),
                path.as_ptr(),
                c"org.freedesktop.systemd1.Unit".as_ptr(),
                method.as_ptr(),
                &mut error,
                &mut reply,
                c"".as_ptr(),
            )
        };
        unsafe {
            (self.error_free)(&mut error);
            if !reply.is_null() {
                (self.message_unref)(reply);
            }
        }
        assert!(
            result >= 0,
            "fixture exact unit reference errno={}",
            -result
        );
    }
}
fn run_sibling(pending: bool) {
    let nonce = random_nonce();
    let units = unit_names(&nonce);
    for unit in &units {
        preexisting_absent(unit);
    }
    let sibling_bus = RefBus::connect();
    let prepared_sibling = sibling_prepare(&units[2], pending);
    let (folder, mut journal) = fixture();
    let marker = folder.path().join("tool-started");
    let launch = journal.begin(0, [1; 16], [2; 32], boot()).unwrap();
    let (mut barrier, delivery) = UnixStream::pair().unwrap();
    barrier
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    barrier
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let (mut helper, mut app, capability) =
        spawn(folder.path(), launch, &marker, &nonce, None, &delivery);
    drop(delivery);
    let identity = receive_identity(&mut app);
    let native = verify_identity(&identity, &capability);
    let parent_inv = properties(&units[2])["InvocationID"].clone();
    let keeper = properties(&units[1]);
    let keeper_invocation = keeper["InvocationID"].clone();
    let keeper_pid: u32 = keeper["MainPID"].parse().unwrap();
    assert!(keeper_pid > 1);
    assert!(!keeper["ControlGroup"].starts_with(&format!("{}/", identity.path)));
    let keeper_start = native_start(keeper_pid).unwrap();
    let keeper_fd = unsafe { libc::syscall(libc::SYS_pidfd_open, keeper_pid, 0_u32) } as i32;
    assert!(keeper_fd >= 0);
    use std::os::fd::FromRawFd;
    let keeper_pin = unsafe { File::from_raw_fd(keeper_fd) };
    assert_eq!(properties(&units[1])["InvocationID"], keeper_invocation);
    journal.bind(identity.clone()).unwrap();
    app.write_all(
        &reader_protocol::encode_begin(&reader_protocol::Begin {
            index: 0,
            nonce: capability.nonce,
        })
        .unwrap(),
    )
    .unwrap();
    let mut ready = [0];
    barrier.read_exact(&mut ready).unwrap();
    assert_eq!(ready, *b"K");
    let ack_deadline = Instant::now() + Duration::from_secs(2);
    let proof = fs::read(folder.path().join("reader0.receipt")).unwrap();
    assert_eq!(proof.len(), 256);
    reader_protocol::decode_cleanup_receipt(&proof)
        .unwrap()
        .matches(&identity)
        .unwrap();
    // Receipt is already synced; this sibling is intentionally inserted after strict empty proof.
    let mut sibling = sibling_start(prepared_sibling, &units[2], &identity.path, pending);
    assert!(
        Instant::now() < ack_deadline,
        "sibling proof exceeded barrier ack budget"
    );
    let sibling_path = sibling_bus.unit_path(&sibling.unit);
    assert_eq!(
        properties(&sibling.unit)["InvocationID"],
        sibling.invocation
    );
    sibling_bus.reference(&sibling_path, c"Ref");
    assert_eq!(
        properties(&sibling.unit)["InvocationID"],
        sibling.invocation
    );
    assert!(
        Instant::now() < ack_deadline,
        "persistent sibling Ref exceeded barrier ack budget"
    );
    barrier.write_all(b"B").unwrap();
    let mut frames = Vec::new();
    (&mut app).take(7).read_to_end(&mut frames).unwrap();
    assert_eq!(frames, b"RGCLC");
    assert!(helper.wait().success());
    journal.resolve(0).unwrap();
    assert!(journal.cleanup_eligible().unwrap());
    assert!(marker.is_file());
    wait_unit_gone(&units[0]);
    wait_unit_gone(&units[1]);
    assert_ne!(native_start(keeper_pid), Some(keeper_start));
    let mut keeper_event = libc::pollfd {
        fd: keeper_pin.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert_eq!(unsafe { libc::poll(&mut keeper_event, 1, 0) }, 1);
    assert_ne!(native_start(native.pid), Some(native.start));
    let parent = properties(&units[2]);
    assert_eq!(parent["InvocationID"], parent_inv);
    assert_eq!(parent["ActiveState"], "active");
    assert_eq!(parent["ControlGroup"], identity.path);
    let pin = reader_cgroup::PinnedCgroup::open(&PathBuf::from(format!(
        "/sys/fs/cgroup{}",
        identity.path
    )))
    .unwrap();
    assert_eq!(pin.identity(), native._container.identity());
    assert!(pin.confirm_empty().is_err());
    let after = properties(&sibling.unit);
    assert_eq!(after["InvocationID"], sibling.invocation);
    assert_eq!(native_start(sibling.pid), Some(sibling.start));
    assert_eq!(
        after["ActiveState"],
        if pending { "activating" } else { "active" }
    );
    if pending {
        assert_eq!(after["Job"], sibling.job);
    }
    let mut event = libc::pollfd {
        fd: sibling.pidfd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert_eq!(unsafe { libc::poll(&mut event, 1, 0) }, 0);
    sibling.writer.take();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = sibling.child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    let terminal = properties(&sibling.unit);
    assert_eq!(terminal["InvocationID"], sibling.invocation);
    assert_eq!(terminal["ExecMainCode"], "1");
    assert_eq!(terminal["ExecMainStatus"], "0");
    assert_ne!(native_start(sibling.pid), Some(sibling.start));
    assert_eq!(unsafe { libc::poll(&mut event, 1, 0) }, 1);
    assert_eq!(
        properties(&sibling.unit)["InvocationID"],
        sibling.invocation
    );
    assert_eq!(sibling_bus.unit_path(&sibling.unit), sibling_path);
    sibling_bus.reference(&sibling_path, c"Unref");
    wait_unit_gone(&sibling.unit);
    native.terminal();
    wait_unit_gone(&units[2]);
    preserved(folder.path());
}
#[test]
#[ignore = "actual user-manager live sibling preservation; root explicit execution"]
fn actual_cookie_keeper_live_sibling_preserved_after_receipt() {
    run_sibling(false);
}
#[test]
#[ignore = "actual user-manager pending sibling preservation; root explicit execution"]
fn actual_cookie_keeper_pending_job_preserved_after_receipt() {
    run_sibling(true);
}
