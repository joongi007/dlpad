//! Linux job-scoped subreaper. Control socket EOF is loss of the owning app.
use std::{
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            net::UnixStream,
            process::{CommandExt, ExitStatusExt},
        },
    },
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
pub const MARKER: &str = "--dlpad-internal-guardian";
pub const USER_MARKER: &str = "--dlpad-internal-user-guardian";
const GATED_MARKER: &str = "--dlpad-internal-cookie-guardian";
const GATED_USER_MARKER: &str = "--dlpad-internal-cookie-user-guardian";
const COOKIE_LAUNCHER_MARKER: &str = "--dlpad-internal-cookie-user-launcher";
const COOKIE_KEEPER_MARKER: &str = "--dlpad-internal-cookie-keeper";
const LAUNCHER_MARKER: &str = "--dlpad-internal-user-service";
static SELF_GUARDIAN: OnceLock<PathBuf> = OnceLock::new();
const CHILDREN_BYTES: u64 = 64 * 1024;
static SIGNAL_REQUEST: AtomicU8 = AtomicU8::new(0);

extern "C" fn request_shutdown(_: libc::c_int) {
    // AtomicU8 is lock-free on supported Linux targets. No allocation, locks,
    // logging, or process operations are performed in the signal handler.
    SIGNAL_REQUEST.store(2, Ordering::Relaxed);
}
fn install_shutdown_handlers() -> io::Result<()> {
    for signal in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP, libc::SIGQUIT] {
        // SAFETY: initialized sigaction with an async-signal-safe handler and
        // empty signal mask. Only this dedicated headless guardian is affected.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = request_shutdown as *const () as usize;
            libc::sigemptyset(&mut action.sa_mask);
            if libc::sigaction(signal, &action, std::ptr::null_mut()) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}

/// A Linux-owned pipe reader that can be stopped even if escaped descendants
/// retain a writer after the guardian exits. This stops reading, not processes.
pub(super) struct InterruptiblePipe<T> {
    pipe: T,
    stop: Arc<AtomicBool>,
    aborted: Arc<AtomicBool>,
}
impl<T: Read + AsRawFd> InterruptiblePipe<T> {
    pub(super) fn new(
        pipe: T,
        stop: Arc<AtomicBool>,
        aborted: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        let descriptor = pipe.as_raw_fd();
        // SAFETY: this reader exclusively owns the pipe descriptor. Preserve
        // existing flags and CLOEXEC; no other reader shares this file description.
        let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            pipe,
            stop,
            aborted,
        })
    }
}
impl<T: Read + AsRawFd> Read for InterruptiblePipe<T> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            if self.stop.load(Ordering::Acquire) {
                self.aborted.store(true, Ordering::Release);
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "owned output drain interrupted",
                ));
            }
            match self.pipe.read(bytes) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                result => return result,
            }
            let mut event = libc::pollfd {
                fd: self.pipe.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: poll writes this live descriptor observation only, with a
            // short timeout so a stop request never depends on writer-side EOF.
            let ready = unsafe { libc::poll(&mut event, 1, 20) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if event.revents & libc::POLLNVAL != 0 {
                return Err(io::Error::other("owned output descriptor invalid"));
            }
            // POLLHUP is deliberately read again: pending bytes precede EOF.
        }
    }
}

pub fn register() -> io::Result<()> {
    let _ = SELF_GUARDIAN.set(std::env::current_exe()?);
    Ok(())
}
pub fn executable() -> io::Result<PathBuf> {
    if let Some(path) = SELF_GUARDIAN.get() {
        return Ok(path.clone());
    }
    let current = std::env::current_exe()?;
    let directory = current
        .parent()
        .ok_or_else(|| io::Error::other("missing executable directory"))?;
    let adjacent = directory.join("dlpad-process-guard");
    if adjacent.is_file() {
        return Ok(adjacent);
    }
    if directory.file_name().is_some_and(|name| name == "deps") {
        let test_helper = directory
            .parent()
            .ok_or_else(|| io::Error::other("missing build directory"))?
            .join("dlpad-process-guard");
        if test_helper.is_file() {
            return Ok(test_helper);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Linux guardian missing: build workspace or register the app guardian entry",
    ))
}
fn backend_marker(
    selected: Option<&std::ffi::OsStr>,
    registered_app: bool,
) -> io::Result<&'static str> {
    match selected {
        None if registered_app => Ok(LAUNCHER_MARKER),
        None => Ok(MARKER), // Standalone development helper keeps manager-independent fixtures.
        Some(value) if value == "user-systemd" => Ok(LAUNCHER_MARKER),
        Some(value) if value == "direct" && !registered_app => Ok(MARKER),
        Some(_) => Err(io::Error::other("unsupported Linux guardian backend")),
    }
}

pub fn command(spec: &super::ProcessSpec) -> io::Result<(Command, UnixStream, UnixStream, bool)> {
    let selected = std::env::var_os("DLPAD_GUARDIAN_BACKEND");
    let marker = backend_marker(selected.as_deref(), SELF_GUARDIAN.get().is_some())?;
    let (app, guard) = UnixStream::pair()?;
    let descriptor = guard.as_raw_fd();
    let mut command = Command::new(executable()?);
    command
        .arg(marker)
        .arg(descriptor.to_string())
        .arg("--")
        .arg(&spec.executable)
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: this live socket descriptor is owned across spawn; clear only its
    // child-side CLOEXEC flag. No fixed fd can collide with Rust's exec-error pipe.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(descriptor, libc::F_SETFD, 0) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok((command, app, guard, marker == LAUNCHER_MARKER))
}
/// Three explicitly owned descriptors are inherited by the launcher only.
pub(super) fn cookie_command(
    spec: &super::ProcessSpec,
    launch: &crate::cookies::ReaderLaunch,
    anchor: &crate::cookies::JournalAnchor,
) -> io::Result<(Command, UnixStream, UnixStream, bool)> {
    let (app, guard) = UnixStream::pair()?;
    let descriptors = [
        guard.as_raw_fd(),
        launch.writer.as_raw_fd(),
        launch.lock.as_raw_fd(),
    ];
    let mut command = Command::new(executable()?);
    command.arg(COOKIE_LAUNCHER_MARKER);
    for descriptor in descriptors {
        command.arg(descriptor.to_string());
    }
    command
        .arg(anchor.record_dev.to_string())
        .arg(anchor.record_ino.to_string())
        .arg("--")
        .arg(&spec.executable)
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: live distinct owned descriptors remain open through spawn. No
    // allocations or service/filesystem operations occur in this child hook.
    unsafe {
        command.pre_exec(move || {
            for descriptor in descriptors {
                let flags = libc::fcntl(descriptor, libc::F_GETFD);
                if flags < 0
                    || libc::fcntl(descriptor, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    Ok((command, app, guard, true))
}
pub(super) fn read_readiness(control: &mut impl Read) -> io::Result<()> {
    let mut marker = [0u8; 1];
    control.read_exact(&mut marker)?;
    match marker[0] {
        b'R' => Ok(()),
        b'G' | b'L' => {
            let mut code = [0u8; 1];
            control.read_exact(&mut code)?;
            if code[0] == b'U' {
                Err(io::Error::other(
                    super::OwnershipCleanupError::TerminalUnconfirmed,
                ))
            } else {
                Err(io::Error::other("invalid guardian readiness"))
            }
        }
        b'E' => {
            let mut code = [0u8; 1];
            control.read_exact(&mut code)?;
            match code[0] {
                b'P' => Err(io::Error::other(
                    "Linux user-service protection setup failed",
                )),
                _ => Err(io::Error::other("invalid guardian readiness")),
            }
        }
        _ => Err(io::Error::other("invalid guardian readiness")),
    }
}

pub fn entry() -> io::Result<bool> {
    let marker = std::env::args_os().nth(1);
    if marker.as_deref() == Some(std::ffi::OsStr::new(COOKIE_KEEPER_MARKER)) {
        if std::env::args_os().nth(2).is_some() {
            return Err(io::Error::other("invalid cookie keeper arguments"));
        }
        wait_cookie_keeper(&mut std::io::stdin(), 0)?;
        std::process::exit(0);
    }
    let gated = matches!(
        marker.as_deref().and_then(|value| value.to_str()),
        Some(GATED_MARKER | GATED_USER_MARKER)
    );
    let user_service = matches!(
        marker.as_deref().and_then(|value| value.to_str()),
        Some(USER_MARKER | GATED_USER_MARKER)
    );
    let cookie_launcher = marker.as_deref() == Some(std::ffi::OsStr::new(COOKIE_LAUNCHER_MARKER));
    let launcher =
        cookie_launcher || marker.as_deref() == Some(std::ffi::OsStr::new(LAUNCHER_MARKER));
    if !gated
        && !user_service
        && !launcher
        && marker.as_deref() != Some(std::ffi::OsStr::new(MARKER))
    {
        return Ok(false);
    }
    let result = if launcher {
        launch_service(cookie_launcher)
    } else {
        supervise(user_service, gated)
    };
    match result {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
            if let Some(signal) = status.signal() {
                // SAFETY: restoring and delivering the child's terminating signal to ourselves.
                unsafe {
                    libc::signal(signal, libc::SIG_DFL);
                    libc::kill(libc::getpid(), signal);
                }
            }
            std::process::exit(126);
        }
        Err(error) => {
            eprintln!("dlpad guardian: {error}");
            std::process::exit(126);
        }
    }
}
fn wait_cookie_keeper(input: &mut impl Read, descriptor: libc::c_int) -> io::Result<()> {
    // The sole writer stays in the launcher; no cookie or receipt FD is accepted.
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
    if flags < 0 || unsafe { libc::fstat(descriptor, metadata.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let metadata = unsafe { metadata.assume_init() };
    if flags & libc::O_ACCMODE != libc::O_RDONLY
        || metadata.st_mode & libc::S_IFMT != libc::S_IFIFO
        || metadata.st_uid != unsafe { libc::geteuid() }
    {
        return Err(io::Error::other("invalid cookie keeper pipe"));
    }
    let mut byte = [0; 1];
    loop {
        match input.read(&mut byte) {
            Ok(0) => return Ok(()),
            Ok(_) => return Err(io::Error::other("invalid cookie keeper data")),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}
fn launch_service(cookie_launcher: bool) -> io::Result<ExitStatus> {
    let mut args = std::env::args_os().skip(2);
    let descriptor = args
        .next()
        .and_then(|value| value.to_str().and_then(|text| text.parse::<i32>().ok()))
        .filter(|value| *value >= 3)
        .ok_or_else(|| io::Error::other("invalid guardian descriptor"))?;
    let control = control_socket(descriptor)?;
    // SAFETY: the launcher exclusively owns this validated control descriptor.
    if unsafe { libc::fcntl(descriptor, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let receipt_fds = if cookie_launcher {
        let writer = parse_internal_number(&mut args)?;
        let lock = parse_internal_number(&mut args)?;
        let record_dev = parse_internal_number(&mut args)?;
        let record_ino = parse_internal_number(&mut args)?;
        let writer =
            i32::try_from(writer).map_err(|_| io::Error::other("invalid receipt descriptor"))?;
        let lock =
            i32::try_from(lock).map_err(|_| io::Error::other("invalid receipt descriptor"))?;
        if writer < 3 || lock < 3 || writer == lock || writer == descriptor || lock == descriptor {
            return Err(io::Error::other("invalid receipt descriptor"));
        }
        for fd in [writer, lock] {
            // Restore CLOEXEC before capability I/O or any service launch.
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
            {
                return Err(io::Error::other("invalid receipt descriptor"));
            }
        }
        // SAFETY: disjoint explicitly inherited descriptors are owned by this dedicated launcher.
        Some((
            unsafe { std::os::fd::OwnedFd::from_raw_fd(writer) },
            unsafe { std::os::fd::OwnedFd::from_raw_fd(lock) },
            record_dev,
            record_ino,
        ))
    } else {
        None
    };
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--")) {
        return Err(io::Error::other("invalid guardian framing"));
    }
    let args: Vec<_> = args.collect();
    if args.is_empty() {
        return Err(io::Error::other("guardian tool missing"));
    }
    // The launcher runs before register() in its new process: reuse this exact
    // executable rather than depending on an adjacent development helper.
    // Keep a CLOEXEC alias until the launch result is known. A service can have
    // duplicated this endpoint before GetUnit/InvocationID validation fails.
    // Dropping our descriptor alone would not make that guardian observe EOF.
    let mut startup_control = control.try_clone()?;
    let result = if let Some((writer, lock, dev, ino)) = receipt_fds {
        let mut control = control;
        let capability = super::reader_gate::receive_capability_only(
            &mut control,
            Instant::now() + Duration::from_secs(10),
            None,
        )
        .map_err(|_| io::Error::other("cookie launcher capability unavailable"));
        match capability {
            Ok(capability) => {
                super::reader_receipt::ReceiptHandles::validate(writer, lock, &capability, dev, ino)
                    .map_err(io::Error::other)
                    .and_then(|receipts| {
                        super::user_service::run_gated(
                            control.into(),
                            &std::env::current_exe()?,
                            &args,
                            capability,
                            receipts,
                        )
                    })
            }
            Err(error) => Err(error),
        }
    } else {
        super::user_service::run(control.into(), &std::env::current_exe()?, &args)
    };
    // The launcher is the final service/cgroup authority. A tool exit126 is still Ok.
    let frame = if result.is_ok() { b"LC" } else { b"LU" };
    let written = write_terminal_frame(&mut startup_control, frame);
    if result.is_err() {
        // Preserve the existing setup-failure notification and owned endpoint EOF.
        let _ = startup_control.write_all(b"EP");
        let _ = startup_control.shutdown(std::net::Shutdown::Both);
    }
    written?;
    result
}

fn parse_internal_number(args: &mut impl Iterator<Item = std::ffi::OsString>) -> io::Result<u64> {
    args.next()
        .and_then(|value| {
            value.to_str().and_then(|text| {
                (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
                    .then(|| text.parse::<u64>().ok())
                    .flatten()
            })
        })
        .ok_or_else(|| io::Error::other("invalid cookie launcher framing"))
}

fn write_terminal_frame(socket: &mut UnixStream, frame: &[u8; 2]) -> io::Result<()> {
    socket.set_write_timeout(Some(Duration::from_secs(1)))?;
    socket.write_all(frame)
}

fn kill_group(pid: i32, signal: i32) -> io::Result<()> {
    // SAFETY: pid belongs to our unreaped child and was assigned its own group.
    if unsafe { libc::kill(-pid, signal) } == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}
fn leader_exited(pid: i32) -> io::Result<bool> {
    // SAFETY: waitid writes the live siginfo struct; WNOWAIT pins the child's PID.
    unsafe {
        let mut info: libc::siginfo_t = std::mem::zeroed();
        if libc::waitid(
            libc::P_PID,
            pid as libc::id_t,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(info.si_pid() != 0)
    }
}
fn reap_descendants() -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    // Only this main thread creates/waits children. The stdin reader cannot reap
    // them, so each listed child identity remains pinned until the wait below.
    let path = format!("/proc/self/task/{}/children", std::process::id());
    loop {
        let mut text = String::new();
        std::fs::File::open(&path)?
            .take(CHILDREN_BYTES + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > CHILDREN_BYTES {
            return Err(io::Error::other("guardian child list limit"));
        }
        for value in text.split_ascii_whitespace() {
            let pid = value
                .parse::<i32>()
                .map_err(|_| io::Error::other("invalid guardian child identity"))?;
            if pid <= 1 {
                return Err(io::Error::other("invalid guardian child identity"));
            }
            // SAFETY: direct, unreaped children of this guardian only; no global PID search.
            unsafe {
                libc::kill(pid, libc::SIGKILL);
            }
        }
        loop {
            let mut status = 0;
            // SAFETY: waits only this guardian's children, which we own.
            let result = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
            if result > 0 {
                continue;
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::ECHILD) {
                    return Ok(());
                }
                if error.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(error);
            }
            break;
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other("guardian cleanup timed out"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}
/// Validate the descriptor before taking ownership; never treat a file/pipe as control.
fn control_socket(descriptor: i32) -> io::Result<UnixStream> {
    for (option, expected) in [
        (libc::SO_TYPE, libc::SOCK_STREAM),
        (libc::SO_DOMAIN, libc::AF_UNIX),
    ] {
        let mut actual: libc::c_int = 0;
        let mut length = std::mem::size_of_val(&actual) as libc::socklen_t;
        // SAFETY: getsockopt writes one integer to a correctly sized live buffer.
        if unsafe {
            libc::getsockopt(
                descriptor,
                libc::SOL_SOCKET,
                option,
                (&mut actual as *mut libc::c_int).cast(),
                &mut length,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        if length as usize != std::mem::size_of_val(&actual) || actual != expected {
            return Err(io::Error::other("invalid guardian control socket"));
        }
    }
    // SAFETY: the internal invocation transfers this validated socket descriptor.
    Ok(unsafe { UnixStream::from_raw_fd(descriptor) })
}

const CONTROL_READER_STEP: Duration = Duration::from_millis(20);
const CONTROL_READER_FINISH: Duration = Duration::from_secs(1);

struct ControlReader {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<io::Result<()>>>,
}
impl ControlReader {
    fn start(mut reader: UnixStream, input: Arc<AtomicU8>) -> io::Result<Self> {
        reader.set_read_timeout(Some(CONTROL_READER_STEP))?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let worker = thread::Builder::new()
            .name("dlpad-guardian-control".into())
            .spawn(move || {
                let mut bytes = [0u8; 16];
                let mut closing = None;
                loop {
                    if stopped.load(Ordering::Acquire) && closing.is_none() {
                        closing = Some(Instant::now());
                    }
                    if closing
                        .is_some_and(|start: Instant| start.elapsed() >= CONTROL_READER_FINISH)
                    {
                        return Err(io::Error::other("guardian control drain deadline exceeded"));
                    }
                    match reader.read(&mut bytes) {
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(error)
                            if matches!(
                                error.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                            ) =>
                        {
                            if stopped.load(Ordering::Acquire) {
                                return Ok(());
                            }
                        }
                        Ok(0) => {
                            input.store(2, Ordering::Release);
                            return Ok(());
                        }
                        Err(_) => {
                            input.store(2, Ordering::Release);
                            return Err(io::Error::other("guardian control reader failed"));
                        }
                        Ok(count) => {
                            for byte in &bytes[..count] {
                                input
                                    .fetch_max(if *byte == b'T' { 1 } else { 2 }, Ordering::AcqRel);
                            }
                        }
                    }
                }
            })?;
        Ok(Self {
            stop,
            thread: Some(worker),
        })
    }
    fn finish(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        let deadline = Instant::now() + CONTROL_READER_FINISH;
        if let Some(worker) = &self.thread {
            while !worker.is_finished() {
                if Instant::now() >= deadline {
                    return Err(io::Error::other(
                        "guardian control reader termination unconfirmed",
                    ));
                }
                thread::sleep(CONTROL_READER_STEP);
            }
        }
        match self.thread.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("guardian control reader panicked"))?,
            None => Ok(()),
        }
    }
}
impl Drop for ControlReader {
    fn drop(&mut self) {
        // Unwind/error still stops the same bounded worker; no unconditional blocking join.
        self.stop.store(true, Ordering::Release);
    }
}

fn gate_identity(index: &std::ffi::OsStr, nonce: &std::ffi::OsStr) -> io::Result<(u8, [u8; 16])> {
    let index = match index.to_str() {
        Some("0") => 0,
        Some("1") => 1,
        _ => return Err(io::Error::other("invalid cookie guardian gate")),
    };
    let text = nonce
        .to_str()
        .filter(|value| value.len() == 32)
        .ok_or_else(|| io::Error::other("invalid cookie guardian gate"))?;
    if !text
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(io::Error::other("invalid cookie guardian gate"));
    }
    let mut nonce = [0; 16];
    for (slot, pair) in nonce.iter_mut().zip(text.as_bytes().chunks_exact(2)) {
        let pair = std::str::from_utf8(pair)
            .map_err(|_| io::Error::other("invalid cookie guardian gate"))?;
        *slot = u8::from_str_radix(pair, 16)
            .map_err(|_| io::Error::other("invalid cookie guardian gate"))?;
    }
    if nonce == [0; 16] {
        return Err(io::Error::other("invalid cookie guardian gate"));
    }
    Ok((index, nonce))
}

fn supervise(user_service: bool, gated: bool) -> io::Result<ExitStatus> {
    let mut args = std::env::args_os().skip(2);
    let mut service_stdin = None;
    let mut socket = if user_service {
        // FD0 arrives from the user manager. Clone to a private descriptor so
        // external tools retain their existing null-stdin contract.
        let input = control_socket(0)?;
        // SAFETY: duplicate this live socket above stdio and keep it CLOEXEC.
        let duplicate = unsafe { libc::fcntl(input.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
        if duplicate < 0 {
            return Err(io::Error::last_os_error());
        }
        // Keep FD0 occupied until Command has installed the tool's null stdin.
        // Closing it early can make a newly opened CLOEXEC null FD land on 0.
        service_stdin = Some(input);
        // SAFETY: fcntl returned a new owned socket descriptor.
        unsafe { UnixStream::from_raw_fd(duplicate) }
    } else {
        let descriptor = args
            .next()
            .and_then(|value| value.to_str().and_then(|text| text.parse::<i32>().ok()))
            .filter(|value| *value >= 3)
            .ok_or_else(|| io::Error::other("invalid guardian descriptor"))?;
        control_socket(descriptor)?
    };
    let result = (|| -> io::Result<ExitStatus> {
        let descriptor = socket.as_raw_fd();
        // SAFETY: prevent the control socket from reaching any external tool exec.
        if unsafe { libc::fcntl(descriptor, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let reader = socket.try_clone()?;
        // SAFETY: explicitly keep the reader clone private even across a tool exec.
        if unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let gate = if gated {
            let index = args
                .next()
                .ok_or_else(|| io::Error::other("invalid cookie guardian gate"))?;
            let nonce = args
                .next()
                .ok_or_else(|| io::Error::other("invalid cookie guardian gate"))?;
            Some(gate_identity(&index, &nonce)?)
        } else {
            None
        };
        if args.next().as_deref() != Some(std::ffi::OsStr::new("--")) {
            return Err(io::Error::other("invalid guardian framing"));
        }
        let tool = args
            .next()
            .ok_or_else(|| io::Error::other("guardian tool missing"))?;
        // SAFETY: Linux process-local subreaper setting, before any tool is created.
        if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) } != 0 {
            return Err(io::Error::last_os_error());
        }
        install_shutdown_handlers()?;
        if let Some((index, nonce)) = gate {
            // This is the sole Begin reader. No drain worker or external tool exists yet.
            super::reader_gate::wait_begin_only(
                index,
                nonce,
                &mut socket,
                std::time::Instant::now() + Duration::from_secs(10),
                None,
            )
            .map_err(|_| io::Error::other("cookie guardian begin unconfirmed"))?;
            if SIGNAL_REQUEST.load(Ordering::Acquire) != 0 {
                return Err(io::Error::other("cookie guardian begin interrupted"));
            }
        }
        let control = Arc::new(AtomicU8::new(0));
        let mut control_reader = ControlReader::start(reader, control.clone())?;
        let result = (|| -> io::Result<ExitStatus> {
            let mut command = Command::new(tool);
            command
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            // SAFETY: no allocation or locks between fork/exec. New group and direct
            // parent-death protection also cover guardian failure before exec.
            unsafe {
                command.pre_exec(|| {
                    if libc::setpgid(0, 0) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    let parent = libc::getppid();
                    if parent == 1 || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    if libc::getppid() != parent {
                        return Err(io::Error::other("guardian disappeared before exec"));
                    }
                    Ok(())
                });
            }
            let mut child = command.spawn()?;
            drop(service_stdin);
            let pid = child.id() as i32;
            let mut term_at = None;
            let run = (|| -> io::Result<()> {
                socket.write_all(b"R")?;
                loop {
                    let request = control
                        .load(Ordering::Acquire)
                        .max(SIGNAL_REQUEST.load(Ordering::Relaxed));
                    if request >= 2 {
                        break;
                    }
                    if request == 1 && term_at.is_none() {
                        kill_group(pid, libc::SIGTERM)?;
                        term_at = Some(Instant::now());
                    }
                    if term_at.is_some_and(|time| time.elapsed() >= Duration::from_millis(200))
                        || leader_exited(pid)?
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(())
            })();
            // Always clean up even if observing or signalling failed. Group cleanup
            // happens before leader reaping, so the group ID cannot be recycled here.
            let killed = kill_group(pid, libc::SIGKILL);
            // A leader can move to another group. It is still our unreaped direct
            // child, so its pinned positive PID is safe even when the original group vanished.
            // SAFETY: this PID has not been reaped by any thread and cannot be reused.
            unsafe {
                libc::kill(pid, libc::SIGKILL);
            }
            let status = child.wait();
            let reaped = reap_descendants();
            run?;
            killed?;
            reaped?;
            status
        })();
        let reader_finished = control_reader.finish();
        result.and_then(|status| reader_finished.map(|()| status))
    })();
    // Keep the private socket through every observation/kill/reap/status result.
    let frame = if result.is_ok() { b"GC" } else { b"GU" };
    write_terminal_frame(&mut socket, frame)?;
    result
}

#[cfg(test)]
mod pipe_tests {
    use super::*;
    #[test]
    fn cookie_keeper_accepts_only_owned_readonly_pipe_eof() {
        if super::super::reader_protocol::run_isolated_lock_test() {
            return;
        }
        for data in [None, Some(b"x".as_slice())] {
            let mut descriptors = [-1; 2];
            // SAFETY: two exclusive output slots; descriptors are adopted once below.
            assert_eq!(
                unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
                0
            );
            let mut input = unsafe { std::fs::File::from_raw_fd(descriptors[0]) };
            let mut output = unsafe { std::fs::File::from_raw_fd(descriptors[1]) };
            assert!(wait_cookie_keeper(&mut io::Cursor::new([]), output.as_raw_fd()).is_err());
            if let Some(bytes) = data {
                output.write_all(bytes).unwrap();
            }
            drop(output);
            let descriptor = input.as_raw_fd();
            assert_eq!(
                wait_cookie_keeper(&mut input, descriptor).is_ok(),
                data.is_none()
            );
        }
        let file = tempfile::tempfile().unwrap();
        assert!(wait_cookie_keeper(&mut io::Cursor::new([]), file.as_raw_fd()).is_err());
        assert!(wait_cookie_keeper(&mut io::Cursor::new([]), -1).is_err());
    }
    #[test]
    fn control_reader_finishes_with_live_writer_without_waiting_for_eof() {
        let (_app, peer) = UnixStream::pair().unwrap();
        let mut reader = ControlReader::start(peer, Arc::new(AtomicU8::new(0))).unwrap();
        let start = Instant::now();
        reader.finish().unwrap();
        assert!(start.elapsed() < Duration::from_millis(500));
        assert!(reader.thread.is_none());
    }
    #[test]
    fn control_reader_drains_stop_bytes_then_terminal_has_real_eof() {
        for service in [false, true] {
            let (mut app, mut peer) = UnixStream::pair().unwrap();
            let launcher = if service {
                Some(peer.try_clone().unwrap())
            } else {
                None
            };
            let input = Arc::new(AtomicU8::new(0));
            let mut reader =
                ControlReader::start(peer.try_clone().unwrap(), input.clone()).unwrap();
            app.write_all(b"TKK").unwrap();
            app.shutdown(std::net::Shutdown::Write).unwrap();
            reader.finish().unwrap();
            assert_eq!(input.load(Ordering::Acquire), 2);
            write_terminal_frame(&mut peer, b"GC").unwrap();
            drop(peer);
            if let Some(mut launcher) = launcher {
                write_terminal_frame(&mut launcher, b"LC").unwrap();
            }
            super::super::terminal::confirm(&mut app, service).unwrap();
        }
    }
    #[test]
    fn startup_terminal_failure_is_typed_before_readiness() {
        for frame in [b"GU", b"LU"] {
            let error = read_readiness(&mut &frame[..]).unwrap_err();
            assert!(super::super::cleanup_is_unconfirmed(&error));
            assert_eq!(
                error
                    .get_ref()
                    .unwrap()
                    .downcast_ref::<super::super::OwnershipCleanupError>(),
                Some(&super::super::OwnershipCleanupError::TerminalUnconfirmed)
            );
        }
        assert!(read_readiness(&mut &b"GC"[..]).is_err());
        assert!(
            !read_readiness(&mut &b"LU-secret"[..])
                .unwrap_err()
                .to_string()
                .contains("secret")
        );
    }
    #[test]
    fn terminal_ack_frames_are_fixed_and_bounded() {
        for frame in [b"GC", b"GU", b"LC", b"LU"] {
            let (mut writer, mut reader) = UnixStream::pair().unwrap();
            write_terminal_frame(&mut writer, frame).unwrap();
            assert_eq!(
                writer.write_timeout().unwrap(),
                Some(Duration::from_secs(1))
            );
            let mut actual = [0u8; 2];
            reader.read_exact(&mut actual).unwrap();
            assert_eq!(&actual, frame);
            assert_ne!(
                unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
                0
            );
        }
    }
    #[test]
    fn missing_terminal_peer_returns_error() {
        let (mut writer, reader) = UnixStream::pair().unwrap();
        drop(reader);
        assert!(write_terminal_frame(&mut writer, b"GC").is_err());
    }
    #[test]
    fn idle_writer_does_not_prevent_read_interruption() {
        let (reader, _writer) = UnixStream::pair().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let aborted = Arc::new(AtomicBool::new(false));
        let mut pipe = InterruptiblePipe::new(reader, stop.clone(), aborted.clone()).unwrap();
        let worker = thread::spawn(move || pipe.read(&mut [0; 8]));
        thread::sleep(Duration::from_millis(50));
        let start = Instant::now();
        stop.store(true, Ordering::Release);
        let error = worker.join().unwrap().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(aborted.load(Ordering::Acquire));
        assert!(start.elapsed() < Duration::from_secs(1));
    }
    #[test]
    fn normal_data_and_eof_are_not_forced_interruption() {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let aborted = Arc::new(AtomicBool::new(false));
        let mut pipe = InterruptiblePipe::new(reader, stop, aborted.clone()).unwrap();
        writer.write_all(b"bounded evidence").unwrap();
        drop(writer);
        let mut actual = Vec::new();
        pipe.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, b"bounded evidence");
        assert!(!aborted.load(Ordering::Acquire));
    }
    #[test]
    fn stop_has_priority_even_when_writer_has_pending_data() {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(b"pending").unwrap();
        let stop = Arc::new(AtomicBool::new(true));
        let aborted = Arc::new(AtomicBool::new(false));
        let mut pipe = InterruptiblePipe::new(reader, stop, aborted.clone()).unwrap();
        assert_eq!(
            pipe.read(&mut [0; 8]).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(aborted.load(Ordering::Acquire));
    }
}

#[cfg(test)]
mod control_descriptor_tests {
    use super::*;
    use std::os::fd::IntoRawFd;

    #[test]
    fn startup_shutdown_reaches_the_service_endpoint_duplicate() {
        let (mut app, guard) = UnixStream::pair().unwrap();
        let mut alias = guard.try_clone().unwrap();
        let mut service = guard.try_clone().unwrap();
        app.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        service
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        drop(guard);
        // A descriptor drop cannot stop an already duplicated service endpoint.
        app.write_all(b"T").unwrap();
        let mut byte = [0];
        service.read_exact(&mut byte).unwrap();
        assert_eq!(&byte, b"T");
        alias.write_all(b"EP").unwrap();
        alias.shutdown(std::net::Shutdown::Both).unwrap();
        assert_eq!(service.read(&mut byte).unwrap(), 0);
        let mut failure = [0; 2];
        app.read_exact(&mut failure).unwrap();
        assert_eq!(&failure, b"EP");
        assert_eq!(app.read(&mut byte).unwrap(), 0);
    }

    #[test]
    fn readiness_has_bounded_fixed_failure_frames() {
        assert!(read_readiness(&mut &b"R"[..]).is_ok());
        let error = read_readiness(&mut &b"EP"[..]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(
            error.to_string(),
            "Linux user-service protection setup failed"
        );
        for frame in [&b""[..], &b"E"[..]] {
            assert_eq!(
                read_readiness(&mut &*frame).unwrap_err().kind(),
                io::ErrorKind::UnexpectedEof
            );
        }
        for frame in [&b"EX"[..], &b"X"[..]] {
            assert_eq!(
                read_readiness(&mut &*frame).unwrap_err().to_string(),
                "invalid guardian readiness"
            );
        }
    }

    #[test]
    fn registered_app_requires_user_manager_without_weak_fallback() {
        use std::ffi::OsStr;
        assert_eq!(backend_marker(None, true).unwrap(), LAUNCHER_MARKER);
        assert_eq!(backend_marker(None, false).unwrap(), MARKER);
        assert!(backend_marker(Some(OsStr::new("direct")), true).is_err());
        assert_eq!(
            backend_marker(Some(OsStr::new("direct")), false).unwrap(),
            MARKER
        );
        for registered in [false, true] {
            assert_eq!(
                backend_marker(Some(OsStr::new("user-systemd")), registered).unwrap(),
                LAUNCHER_MARKER
            );
            assert!(backend_marker(Some(OsStr::new("unknown")), registered).is_err());
        }
    }

    #[test]
    fn accepts_only_owned_unix_stream_control() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let mut control = control_socket(stream.into_raw_fd()).unwrap();
        peer.write_all(b"R").unwrap();
        let mut ready = [0u8];
        control.read_exact(&mut ready).unwrap();
        assert_eq!(ready, *b"R");
    }

    #[test]
    fn rejects_file_without_taking_its_ownership() {
        let mut file = tempfile::tempfile().unwrap();
        assert!(control_socket(file.as_raw_fd()).is_err());
        file.write_all(b"preserved").unwrap();
    }

    #[test]
    fn rejects_datagram_without_taking_its_ownership() {
        let (socket, peer) = std::os::unix::net::UnixDatagram::pair().unwrap();
        assert!(control_socket(socket.as_raw_fd()).is_err());
        assert_eq!(socket.send(b"preserved").unwrap(), 9);
        let mut bytes = [0u8; 9];
        peer.recv(&mut bytes).unwrap();
        assert_eq!(&bytes, b"preserved");
    }
}
