//! Explicit actual user-manager gate tests; ordinary tests do not require a manager.
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
        let _ = self.child.wait();
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
    let deadline = Instant::now() + Duration::from_secs(5);
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
) -> (OwnedHelper, UnixStream, reader_protocol::Capability) {
    let (app, peer) = UnixStream::pair().unwrap();
    let descriptors = [
        peer.as_raw_fd(),
        launch.writer.as_raw_fd(),
        launch.lock.as_raw_fd(),
    ];
    assert!(descriptors.iter().all(|fd| *fd >= 3));
    assert_ne!(descriptors[0], descriptors[1]);
    assert_ne!(descriptors[0], descriptors[2]);
    assert_ne!(descriptors[1], descriptors[2]);
    let lock = launch.lock.metadata().unwrap();
    let capability = launch.capability.clone();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dlpad-process-guard"));
    command.arg("--dlpad-internal-cookie-user-launcher");
    for fd in descriptors {
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
#[test]
#[ignore = "actual user-manager capability test; root explicit native invocation only"]
fn actual_service_gate_two_slots_bind_receipt_and_reopen() {
    let (folder, mut journal) = fixture();
    let anchor = journal.anchor();
    for index in 0..2 {
        let marker = folder.path().join(format!("tool-started-{index}"));
        let launch = journal
            .begin(index, [index + 1; 16], [index + 2; 32], boot())
            .unwrap();
        let (mut helper, mut app, capability) = spawn(folder.path(), launch, &marker);
        let identity = receive_identity(&mut app);
        let native = verify_identity(&identity, &capability);
        assert!(!marker.exists());
        journal.bind(identity.clone()).unwrap();
        assert!(!marker.exists());
        app.write_all(
            &reader_protocol::encode_begin(&reader_protocol::Begin {
                index,
                nonce: capability.nonce,
            })
            .unwrap(),
        )
        .unwrap();
        let mut terminal = Vec::new();
        (&mut app).take(7).read_to_end(&mut terminal).unwrap();
        assert_eq!(terminal, b"RGCLC");
        assert!(helper.wait().success());
        assert_eq!(
            fs::read_to_string(&marker).unwrap(),
            "synthetic tool started; capability FDs absent"
        );
        let bytes = fs::read(folder.path().join(format!("reader{index}.receipt"))).unwrap();
        assert_eq!(bytes.len(), reader_protocol::RECEIPT_SIZE);
        reader_protocol::decode_cleanup_receipt(&bytes)
            .unwrap()
            .matches(&identity)
            .unwrap();
        journal.resolve(index).unwrap();
        native.terminal();
        preserved(folder.path());
    }
    assert!(journal.cleanup_eligible().unwrap());
    drop(journal);
    let reopened =
        reader_journal::ReaderJournal::reopen(File::open(folder.path()).unwrap(), anchor).unwrap();
    assert!(reopened.cleanup_eligible().unwrap());
    preserved(folder.path());
}
#[test]
#[ignore = "actual user-manager capability test; root explicit native invocation only"]
fn actual_before_begin_endpoint_close_preserves_intent_without_tool() {
    let (folder, mut journal) = fixture();
    let marker = folder.path().join("tool-started");
    let launch = journal.begin(0, [1; 16], [2; 32], boot()).unwrap();
    let (mut helper, mut app, capability) = spawn(folder.path(), launch, &marker);
    let identity = receive_identity(&mut app);
    let native = verify_identity(&identity, &capability);
    assert!(!marker.exists());
    app.shutdown(Shutdown::Both).unwrap();
    drop(app);
    let _status = helper.wait();
    assert!(!marker.exists());
    assert!(!journal.cleanup_eligible().unwrap());
    preserved(folder.path());
    native.terminal();
}
#[test]
#[ignore = "actual user-manager capability test; root explicit native invocation only"]
fn actual_wrong_begin_never_promotes_cleanup_receipt_to_execution_success() {
    let (folder, mut journal) = fixture();
    let marker = folder.path().join("tool-started");
    let launch = journal.begin(0, [1; 16], [2; 32], boot()).unwrap();
    let (mut helper, mut app, capability) = spawn(folder.path(), launch, &marker);
    let identity = receive_identity(&mut app);
    let native = verify_identity(&identity, &capability);
    journal.bind(identity.clone()).unwrap();
    app.write_all(
        &reader_protocol::encode_begin(&reader_protocol::Begin {
            index: 0,
            nonce: [9; 16],
        })
        .unwrap(),
    )
    .unwrap();
    app.shutdown(Shutdown::Write).unwrap();
    let mut terminal = Vec::new();
    (&mut app).take(7).read_to_end(&mut terminal).unwrap();
    assert!(terminal == b"GULC" || terminal == b"GULU");
    assert!(!terminal.contains(&b'R'));
    assert_ne!(terminal, b"RGCLC");
    assert_eq!(helper.wait().code(), Some(126));
    assert!(!marker.exists());
    let bytes = fs::read(folder.path().join("reader0.receipt")).unwrap();
    if !bytes.is_empty() {
        assert_eq!(bytes.len(), reader_protocol::RECEIPT_SIZE);
        reader_protocol::decode_cleanup_receipt(&bytes)
            .unwrap()
            .matches(&identity)
            .unwrap();
    }
    // No resolve: cleanup-only receipt is not an execution success acknowledgment.
    assert!(!journal.cleanup_eligible().unwrap());
    preserved(folder.path());
    native.terminal();
}
