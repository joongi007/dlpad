//! Owned argv-only subprocesses, bounded evidence, and concurrent pipe draining.
use std::{
    ffi::OsString,
    io::{self, Read},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(not(windows))]
use std::process::Command;
#[cfg(not(any(target_os = "linux", windows)))]
use std::process::Stdio;
#[cfg(target_os = "linux")]
mod guardian;
#[cfg(target_os = "linux")]
mod terminal;
#[cfg(target_os = "linux")]
mod user_service;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
mod windows_cookie_capture;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_guardian;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_guardian_config;
// Managed handoff stays separate from cookie execution authority.
#[cfg(windows)]
mod windows_guardian_entry;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_guardian_protocol;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_guardian_registration;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_helper_launch;
#[cfg(windows)]
mod windows_managed_capture;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_managed_guardian;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_managed_guardian_config;
#[cfg(windows)]
mod windows_managed_guardian_entry;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_managed_guardian_protocol;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_paired_capture;
#[cfg(windows)]
#[allow(dead_code)]
mod windows_paired_guardian;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_paired_guardian_binding;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_paired_guardian_config;
#[cfg(windows)]
mod windows_paired_guardian_entry;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_paired_guardian_protocol;
#[cfg(any(windows, test))]
mod windows_runtime_environment_config;
// Mirrors the typed capture boundary while keeping native journal private to tools.
#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_managed_tool(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<crate::metrics::ProgressFeed>,
    journal: &mut crate::tools::ManagedJournal,
    root: &std::path::Path,
    index: u8,
    stage: crate::tools::ManagedReaderStage,
) -> io::Result<MetadataResult> {
    windows_managed_capture::run(
        spec, cancel, metadata, progress, journal, root, index, stage,
    )
}

// One typed boundary for both original journal capabilities.
#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_paired_tool(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<crate::metrics::ProgressFeed>,
    journal: &mut crate::tools::ManagedJournal,
    root: &std::path::Path,
    index: u8,
    stage: crate::tools::ManagedReaderStage,
    cookies: &crate::cookies::SessionCookieLease,
) -> io::Result<MetadataResult> {
    windows_paired_capture::run(
        spec, cancel, metadata, progress, journal, root, index, stage, cookies,
    )
}

/// Ownership may remain live after a bounded cleanup attempt.
/// This marker carries no tool output, path, credential, or numeric process identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipCleanupError {
    StartupDeferred,
    StartupUnconfirmed,
    OutputDrainInterrupted,
    TerminalUnconfirmed,
}
impl std::fmt::Display for OwnershipCleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::StartupDeferred => "guardian startup failed; child cleanup deferred",
            Self::StartupUnconfirmed => {
                "guardian startup failed; child cleanup could not be confirmed"
            }
            Self::TerminalUnconfirmed => "process terminal ownership confirmation missing",
            Self::OutputDrainInterrupted => {
                "owned output drain interrupted; process ownership protection incomplete"
            }
        })
    }
}
impl std::error::Error for OwnershipCleanupError {}
/// Classifies only an ownership marker emitted by this process adapter.
pub fn cleanup_is_unconfirmed(error: &io::Error) -> bool {
    error
        .get_ref()
        .is_some_and(|inner| inner.is::<OwnershipCleanupError>())
}

/// Must run before GUI/DB initialization; never starts a visible window.
pub fn guardian_entry() -> io::Result<bool> {
    #[cfg(target_os = "linux")]
    {
        guardian::entry()
    }
    #[cfg(windows)]
    {
        if windows_paired_guardian_entry::entry()? {
            return Ok(true);
        }
        if windows_managed_guardian_entry::entry()? {
            return Ok(true);
        }
        windows_guardian_entry::entry()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        Ok(false)
    }
}
pub fn register_self_guardian() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        guardian::register()
    }
    #[cfg(windows)]
    {
        windows_guardian_registration::register()
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        Ok(())
    }
}
pub const MAX_LINE_BYTES: usize = 64 * 1024;
pub const MAX_LOG_BYTES: usize = 256 * 1024;
pub const MAX_METADATA_BYTES: usize = 8 * 1024 * 1024;
#[derive(Debug)]
pub struct MetadataResult {
    pub execution: ExecutionResult,
    pub stdout: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ProcessSpec {
    #[cfg(windows)]
    pub runtime_environment: Option<crate::runtime_environment::WindowsManagedDenoEnvironment>,
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub timeout: Duration,
    pub max_log_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOutcome {
    Exited,
    Cancelled,
    TimedOut,
    OutputLimit,
}
#[derive(Debug)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub logs: String,
    pub truncated: bool,
    pub outcome: ProcessOutcome,
}
#[derive(Default)]
struct Evidence {
    bytes: std::collections::VecDeque<u8>,
    truncated: bool,
}
fn drain(
    mut pipe: impl Read,
    evidence: Arc<Mutex<Evidence>>,
    limit: usize,
    oversized: Arc<AtomicBool>,
    progress: Option<crate::metrics::ProgressFeed>,
    processing: bool,
) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    let mut line = 0usize;
    let mut current = Vec::new();
    loop {
        let n = pipe.read(&mut buf)?;
        if n == 0 {
            if line <= MAX_LINE_BYTES
                && let Some(feed) = &progress
                && let Ok(text) = std::str::from_utf8(&current)
            {
                feed.observe(text.trim_end_matches('\r'));
                if processing {
                    feed.observe_processing(text.trim_end_matches('\r'));
                }
            }
            return Ok(());
        }
        for byte in &buf[..n] {
            if *byte == b'\n' {
                if line <= MAX_LINE_BYTES
                    && let Some(feed) = &progress
                    && let Ok(text) = std::str::from_utf8(&current)
                {
                    feed.observe(text.trim_end_matches('\r'));
                    if processing {
                        feed.observe_processing(text.trim_end_matches('\r'));
                    }
                }
                current.clear();
                line = 0;
            } else {
                line = line.saturating_add(1);
                if progress.is_some() && line <= MAX_LINE_BYTES {
                    current.push(*byte);
                }
                if line > MAX_LINE_BYTES {
                    oversized.store(true, Ordering::Release);
                }
            }
        }
        let mut retained = evidence
            .lock()
            .map_err(|_| io::Error::other("evidence mutex poisoned"))?;
        let keep = n.min(limit);
        let excess = retained
            .bytes
            .len()
            .saturating_add(keep)
            .saturating_sub(limit);
        retained.truncated |= excess > 0 || n > keep;
        retained.bytes.drain(..excess);
        retained.bytes.extend(&buf[n - keep..n]);
    }
}
fn drain_metadata(
    mut pipe: impl Read,
    capture: Arc<Mutex<Vec<u8>>>,
    oversized: Arc<AtomicBool>,
) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    loop {
        let n = pipe.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        let mut bytes = capture
            .lock()
            .map_err(|_| io::Error::other("metadata mutex poisoned"))?;
        let keep = n.min(MAX_METADATA_BYTES.saturating_sub(bytes.len()));
        bytes.extend_from_slice(&buf[..keep]);
        if keep < n {
            oversized.store(true, Ordering::Release);
        }
    }
}
#[cfg(unix)]
fn configure(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}
#[cfg(not(any(unix, windows)))]
fn configure(_: &mut Command) {}
#[cfg(all(unix, not(target_os = "linux")))]
struct Ownership {
    pid: i32,
}
#[cfg(all(unix, not(target_os = "linux")))]
impl Ownership {
    fn attach(child: &mut std::process::Child) -> io::Result<Self> {
        Ok(Self {
            pid: child.id() as i32,
        })
    }
    fn signal(&self, force: bool) -> io::Result<()> {
        let signal = if force { libc::SIGKILL } else { libc::SIGTERM }; // SAFETY: negative owned group ID never targets the caller group.
        if unsafe { libc::kill(-self.pid, signal) } != 0 {
            let e = io::Error::last_os_error();
            if e.raw_os_error() != Some(libc::ESRCH) {
                return Err(e);
            }
        }
        Ok(())
    }
}
#[cfg(target_os = "linux")]
const STARTUP_REAPER_SLOTS: usize = 8;
#[cfg(target_os = "linux")]
const STARTUP_ADMISSION_GRACE: Duration = Duration::from_secs(1);
#[cfg(target_os = "linux")]
const STARTUP_NATURAL_GRACE: Duration = Duration::from_secs(5);
#[cfg(target_os = "linux")]
const STARTUP_KILL_GRACE: Duration = Duration::from_secs(2);
#[cfg(target_os = "linux")]
struct StartupQueue {
    reserved: usize,
    pending: std::collections::VecDeque<std::process::Child>,
}
#[cfg(target_os = "linux")]
struct StartupReaper {
    queue: Mutex<StartupQueue>,
    wake: std::sync::Condvar,
}
#[cfg(target_os = "linux")]
struct StartupPermit {
    reaper: Arc<StartupReaper>,
    held: bool,
}
#[cfg(target_os = "linux")]
impl Drop for StartupPermit {
    fn drop(&mut self) {
        if self.held {
            let mut queue = self
                .reaper
                .queue
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            queue.reserved -= 1;
            self.reaper.wake.notify_all();
        }
    }
}
#[cfg(target_os = "linux")]
impl StartupPermit {
    fn reserve(control: &ExecutionControl<'_>) -> io::Result<Self> {
        static REAPER: std::sync::OnceLock<Result<Arc<StartupReaper>, ()>> =
            std::sync::OnceLock::new();
        let reaper = REAPER
            .get_or_init(|| {
                let reaper = Arc::new(StartupReaper {
                    queue: Mutex::new(StartupQueue {
                        reserved: 0,
                        pending: std::collections::VecDeque::with_capacity(STARTUP_REAPER_SLOTS),
                    }),
                    wake: std::sync::Condvar::new(),
                });
                let worker = reaper.clone();
                thread::Builder::new()
                    .name("dlpad-startup-reaper".into())
                    .stack_size(128 * 1024)
                    .spawn(move || startup_reaper_worker(worker))
                    .map_err(|_| ())?;
                Ok(reaper)
            })
            .as_ref()
            .map_err(|_| io::Error::other("guardian startup reaper unavailable"))?
            .clone();
        Self::reserve_on_controlled(reaper, Some(control))
    }
    #[cfg(test)]
    fn reserve_on(reaper: Arc<StartupReaper>) -> io::Result<Self> {
        Self::reserve_on_controlled(reaper, None)
    }
    fn reserve_on_controlled(
        reaper: Arc<StartupReaper>,
        control: Option<&ExecutionControl<'_>>,
    ) -> io::Result<Self> {
        {
            let mut queue = reaper
                .queue
                .lock()
                .map_err(|_| io::Error::other("guardian startup reaper unavailable"))?;
            let deadline = Instant::now() + STARTUP_ADMISSION_GRACE;
            while queue.reserved >= STARTUP_REAPER_SLOTS {
                if let Some(control) = control {
                    control.checkpoint()?;
                }
                let mut remaining = deadline.saturating_duration_since(Instant::now());
                if let Some(control) = control {
                    remaining = remaining
                        .min(control.deadline.saturating_duration_since(Instant::now()))
                        .min(Duration::from_millis(20));
                }
                if remaining.is_zero() {
                    return Err(io::Error::other("guardian startup admission limit reached"));
                }
                let (next, _) = reaper
                    .wake
                    .wait_timeout(queue, remaining)
                    .map_err(|_| io::Error::other("guardian startup reaper unavailable"))?;
                queue = next;
            }
            if let Some(control) = control {
                control.checkpoint()?;
            }
            queue.reserved += 1;
        }
        Ok(Self { reaper, held: true })
    }
    fn transfer(mut self, child: std::process::Child) {
        let mut queue = self
            .reaper
            .queue
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // This child already owns one of eight reserved slots. No overflow path.
        queue.pending.push_back(child);
        self.held = false;
        self.reaper.wake.notify_all();
    }
}
#[cfg(target_os = "linux")]
fn startup_reaper_worker(reaper: Arc<StartupReaper>) {
    let mut active = Vec::with_capacity(STARTUP_REAPER_SLOTS);
    loop {
        let mut queue = reaper
            .queue
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while queue.pending.is_empty() && active.is_empty() {
            queue = reaper
                .wake
                .wait(queue)
                .unwrap_or_else(|error| error.into_inner());
        }
        active.extend(queue.pending.drain(..));
        drop(queue);
        let mut index = 0;
        while index < active.len() {
            let finished = match active[index].try_wait() {
                Ok(Some(_)) => true,
                Err(error) if error.raw_os_error() == Some(libc::ECHILD) => true,
                _ => false,
            };
            if finished {
                drop(active.swap_remove(index));
                let mut queue = reaper
                    .queue
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                queue.reserved -= 1;
                reaper.wake.notify_all();
            } else {
                index += 1;
            }
        }
        if !active.is_empty() {
            let queue = reaper
                .queue
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            // Poll only owned deferred children; an empty worker blocks indefinitely.
            drop(
                reaper
                    .wake
                    .wait_timeout(queue, Duration::from_millis(20))
                    .unwrap_or_else(|error| error.into_inner()),
            );
        }
    }
}
#[cfg(target_os = "linux")]
trait StartupChild {
    fn exited(&mut self) -> io::Result<bool>;
    fn kill_owned(&mut self) -> io::Result<()>;
}
#[cfg(target_os = "linux")]
impl StartupChild for std::process::Child {
    fn exited(&mut self) -> io::Result<bool> {
        Ok(self.try_wait()?.is_some())
    }
    fn kill_owned(&mut self) -> io::Result<()> {
        self.kill()
    }
}
#[cfg(target_os = "linux")]
fn wait_startup_child(child: &mut impl StartupChild, grace: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + grace;
    loop {
        match child.exited() {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(
            Duration::from_millis(20).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}
#[cfg(target_os = "linux")]
fn cleanup_startup_child(
    child: &mut impl StartupChild,
    natural: Duration,
    killed: Duration,
) -> io::Result<bool> {
    if wait_startup_child(child, natural)? {
        return Ok(true);
    }
    // The preceding try_wait still owns an unreaped Child. No other reaper may consume it.
    if let Err(error) = child.kill_owned() {
        if child.exited()? {
            return Ok(true);
        }
        return Err(error);
    }
    wait_startup_child(child, killed)
}
#[cfg(target_os = "linux")]
struct Ownership {
    control: Mutex<std::os::unix::net::UnixStream>,
    expects_service: bool,
}
#[cfg(target_os = "linux")]
impl Ownership {
    #[cfg(test)]
    fn attach(
        child: &std::process::Child,
        control: std::os::unix::net::UnixStream,
        expects_service: bool,
    ) -> io::Result<Self> {
        let cancel = AtomicBool::new(false);
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
        };
        Self::attach_controlled(child, control, expects_service, &execution)
    }
    fn attach_controlled(
        _child: &std::process::Child,
        mut control: std::os::unix::net::UnixStream,
        expects_service: bool,
        execution: &ExecutionControl<'_>,
    ) -> io::Result<Self> {
        let checked = read_startup_controlled(&mut control, execution);
        if let Err(error) = checked {
            drop(control); // EOF makes a started guardian clean up its tool tree.
            return Err(error);
        }
        control.set_read_timeout(None)?;
        control.set_write_timeout(Some(Duration::from_secs(1)))?;
        Ok(Self {
            control: Mutex::new(control),
            expects_service,
        })
    }
    fn confirm_terminal(&self) -> io::Result<()> {
        let mut control = self
            .control
            .lock()
            .map_err(|_| io::Error::other(OwnershipCleanupError::TerminalUnconfirmed))?;
        terminal::confirm(&mut control, self.expects_service)
    }
    fn finish_commands(&self) -> io::Result<()> {
        self.control
            .lock()
            .map_err(|_| io::Error::other("guardian control poisoned"))?
            .shutdown(std::net::Shutdown::Write)
    }
    fn signal(&self, force: bool) -> io::Result<()> {
        use std::io::Write;
        match self
            .control
            .lock()
            .map_err(|_| io::Error::other("guardian control poisoned"))?
            .write_all(if force { b"K" } else { b"T" })
        {
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            result => result,
        }
    }
}
#[cfg(target_os = "linux")]
struct OwnedChildLifetime {
    child: Option<std::process::Child>,
    permit: Option<StartupPermit>,
    // Exclusive alias permits EOF shutdown during unwind without the control mutex.
    shutdown: std::os::unix::net::UnixStream,
}
#[cfg(target_os = "linux")]
impl OwnedChildLifetime {
    fn new(
        child: std::process::Child,
        permit: StartupPermit,
        shutdown: std::os::unix::net::UnixStream,
    ) -> Self {
        Self {
            child: Some(child),
            permit: Some(permit),
            shutdown,
        }
    }
    fn shutdown_peer(&self) {
        let _ = self.shutdown.shutdown(std::net::Shutdown::Both);
    }
    fn defer(&mut self) {
        if let Some(child) = self.child.take() {
            self.permit
                .take()
                .expect("owned child has reserved reaper slot")
                .transfer(child);
        }
    }
    fn cleanup_error(&mut self) -> io::Result<bool> {
        self.shutdown_peer();
        if self.child.is_none() {
            return Ok(false);
        }
        let result = cleanup_startup_child(self, STARTUP_NATURAL_GRACE, STARTUP_KILL_GRACE);
        if !matches!(result, Ok(true)) {
            self.defer();
        }
        result
    }
    fn wait_cancelled(&mut self) -> io::Result<std::process::ExitStatus> {
        let result = (|| {
            if wait_startup_child(self, STARTUP_NATURAL_GRACE)? {
                return self
                    .try_wait()?
                    .ok_or_else(|| io::Error::other(OwnershipCleanupError::TerminalUnconfirmed));
            }
            self.shutdown_peer();
            if cleanup_startup_child(self, Duration::ZERO, STARTUP_KILL_GRACE)? {
                self.try_wait()?
                    .ok_or_else(|| io::Error::other(OwnershipCleanupError::TerminalUnconfirmed))
            } else {
                Err(io::Error::other(OwnershipCleanupError::TerminalUnconfirmed))
            }
        })();
        if result.is_err() {
            self.shutdown_peer();
            self.defer();
        }
        result
    }
}
#[cfg(target_os = "linux")]
impl std::ops::Deref for OwnedChildLifetime {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        self.child.as_ref().expect("live owned child")
    }
}
#[cfg(target_os = "linux")]
impl std::ops::DerefMut for OwnedChildLifetime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.child.as_mut().expect("live owned child")
    }
}
#[cfg(target_os = "linux")]
impl StartupChild for OwnedChildLifetime {
    fn exited(&mut self) -> io::Result<bool> {
        Ok(self.try_wait()?.is_some())
    }
    fn kill_owned(&mut self) -> io::Result<()> {
        self.kill()
    }
}
#[cfg(target_os = "linux")]
impl Drop for OwnedChildLifetime {
    fn drop(&mut self) {
        self.shutdown_peer();
        let Some(child) = self.child.as_mut() else {
            return;
        };
        match child.try_wait() {
            Ok(Some(_)) => {} // Original child already reaped; no PID operation remains.
            Ok(None) => {
                let _ = child.kill(); // Only this still-unreaped Child, never a cached numeric PID.
                self.defer();
            }
            Err(_) => self.defer(), // Identity/status unknown: do not guess a signal target.
        }
        // A retained permit drops here; transferred permits release only after reaping.
    }
}
#[cfg(not(any(unix, windows)))]
struct Ownership;
#[cfg(not(any(unix, windows)))]
impl Ownership {
    fn attach(_: &std::process::Child) -> io::Result<Self> {
        Err(io::Error::other("owned process trees unsupported"))
    }
    fn signal(&self, _: bool) -> io::Result<()> {
        Ok(())
    }
}

pub fn run(spec: &ProcessSpec, cancel: &Arc<AtomicBool>) -> io::Result<ExecutionResult> {
    Ok(run_capture(spec, cancel, false, None)?.execution)
}
pub fn run_metadata(spec: &ProcessSpec, cancel: &Arc<AtomicBool>) -> io::Result<MetadataResult> {
    run_capture(spec, cancel, true, None)
}
pub fn run_with_progress(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    progress: crate::metrics::ProgressFeed,
) -> io::Result<ExecutionResult> {
    Ok(run_capture(spec, cancel, false, Some(progress))?.execution)
}
#[cfg(any(target_os = "linux", windows))]
struct StopDrains(Arc<AtomicBool>);
#[cfg(any(target_os = "linux", windows))]
impl Drop for StopDrains {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
#[cfg(target_os = "linux")]
fn read_startup_controlled(
    control: &mut std::os::unix::net::UnixStream,
    execution: &ExecutionControl<'_>,
) -> io::Result<()> {
    let startup_deadline = Instant::now() + Duration::from_secs(10);
    let mut frame = [0u8; 2];
    let mut length = 0;
    loop {
        execution.checkpoint()?;
        let remaining = startup_deadline
            .min(execution.deadline)
            .saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "guardian startup readiness timed out",
            ));
        }
        control.set_read_timeout(Some(remaining.min(Duration::from_millis(20))))?;
        match control.read(&mut frame[length..length + 1]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "guardian readiness ended",
                ));
            }
            Ok(_) => {
                length += 1;
                if length == 2 || !matches!(frame[0], b'E' | b'G' | b'L') {
                    let result = guardian::read_readiness(&mut &frame[..length]);
                    execution.checkpoint()?;
                    return result;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error),
        }
    }
}

struct ExecutionControl<'a> {
    cancel: &'a AtomicBool,
    deadline: Instant,
}
impl ExecutionControl<'_> {
    fn outcome(&self) -> Option<ProcessOutcome> {
        if self.cancel.load(Ordering::Acquire) {
            Some(ProcessOutcome::Cancelled)
        } else if Instant::now() >= self.deadline {
            Some(ProcessOutcome::TimedOut)
        } else {
            None
        }
    }
    #[cfg(target_os = "linux")]
    fn checkpoint(&self) -> io::Result<()> {
        match self.outcome() {
            Some(ProcessOutcome::Cancelled) => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "process startup cancelled",
            )),
            Some(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "process startup deadline exceeded",
            )),
            None => Ok(()),
        }
    }
}
fn interrupted_result(outcome: ProcessOutcome) -> MetadataResult {
    MetadataResult {
        stdout: vec![],
        execution: ExecutionResult {
            exit_code: None,
            logs: String::new(),
            truncated: false,
            outcome,
        },
    }
}

#[cfg(any(target_os = "linux", windows))]
pub(crate) fn run_metadata_with_cookie(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    lease: &crate::cookies::SessionCookieLease,
) -> io::Result<MetadataResult> {
    run_capture_inner(spec, cancel, true, None, Some((lease, 0)))
}
#[cfg(any(target_os = "linux", windows))]
pub(crate) fn run_with_cookie(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    progress: Option<crate::metrics::ProgressFeed>,
    lease: &crate::cookies::SessionCookieLease,
) -> io::Result<ExecutionResult> {
    Ok(run_capture_inner(spec, cancel, false, progress, Some((lease, 1)))?.execution)
}
fn run_capture(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<crate::metrics::ProgressFeed>,
) -> io::Result<MetadataResult> {
    run_capture_inner(spec, cancel, metadata, progress, None)
}
fn run_capture_inner(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<crate::metrics::ProgressFeed>,
    cookie: Option<(&crate::cookies::SessionCookieLease, u8)>,
) -> io::Result<MetadataResult> {
    #[cfg(not(target_os = "linux"))]
    let _ = cookie;
    if spec.timeout.is_zero() || spec.max_log_bytes > MAX_LOG_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid process bounds",
        ));
    }
    let execution = ExecutionControl {
        cancel,
        deadline: Instant::now().checked_add(spec.timeout).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid process deadline")
        })?,
    };
    if let Some(outcome) = execution.outcome() {
        return Ok(interrupted_result(outcome));
    }
    #[cfg(windows)]
    if let Some((lease, index)) = cookie {
        return windows_cookie_capture::run(spec, cancel, metadata, progress, lease, index);
    }
    #[cfg(target_os = "linux")]
    let mut cookie_reader = cookie
        .map(|(lease, index)| lease.begin_gated_reader(index))
        .transpose()
        .map_err(|_| io::Error::other(OwnershipCleanupError::StartupUnconfirmed))?;
    #[cfg(target_os = "linux")]
    let cookie_launch = cookie_reader
        .as_mut()
        .map(|reader| reader.take_launch())
        .transpose()
        .map_err(|_| io::Error::other(OwnershipCleanupError::StartupUnconfirmed))?;
    #[cfg(target_os = "linux")]
    let (mut command, mut control, guard_side, expects_service) = match &cookie_launch {
        Some((launch, anchor)) => guardian::cookie_command(spec, launch, anchor)?,
        None => guardian::command(spec)?,
    };
    #[cfg(not(any(target_os = "linux", windows)))]
    let mut command = {
        let mut command = Command::new(&spec.executable);
        command
            .args(&spec.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    };
    #[cfg(not(windows))]
    configure(&mut command);
    #[cfg(target_os = "linux")]
    let startup_permit = match StartupPermit::reserve(&execution) {
        Ok(permit) => permit,
        Err(error) => {
            if let Some(outcome) = execution.outcome() {
                return Ok(interrupted_result(outcome));
            }
            return Err(error);
        }
    };
    #[cfg(target_os = "linux")]
    if let Some(outcome) = execution.outcome() {
        return Ok(interrupted_result(outcome));
    }
    #[cfg(target_os = "linux")]
    let shutdown_control = control.try_clone()?;
    #[cfg(target_os = "linux")]
    if let Some(outcome) = execution.outcome() {
        return Ok(interrupted_result(outcome));
    }
    #[cfg(target_os = "linux")]
    let mut child = OwnedChildLifetime::new(command.spawn()?, startup_permit, shutdown_control);
    #[cfg(not(any(target_os = "linux", windows)))]
    let mut child = command.spawn()?;
    #[cfg(windows)]
    let (mut child, ownership) = match windows::spawn(spec, &execution)
        .map_err(|error| io::Error::new(error.kind(), OwnershipCleanupError::StartupUnconfirmed))?
    {
        windows::SpawnOutcome::Ready(child, ownership) => (child, ownership),
        windows::SpawnOutcome::Interrupted(outcome) => return Ok(interrupted_result(outcome)),
    };
    #[cfg(target_os = "linux")]
    drop(guard_side);
    #[cfg(target_os = "linux")]
    let attached = (|| {
        // The original app holds the journal lock; inherited writer/lock copies
        // remain only in the dedicated launcher after this drop.
        let capability = cookie_launch.map(|(launch, _)| launch.capability);
        if let (Some(capability), Some(reader)) = (capability, cookie_reader.as_mut()) {
            let deadline = execution
                .deadline
                .min(Instant::now() + Duration::from_secs(10));
            reader_gate::send_capability_only(&capability, &mut control, deadline, Some(cancel))
                .map_err(|_| io::Error::other(OwnershipCleanupError::StartupUnconfirmed))?;
            reader_gate::receive_identity_then_begin(
                &mut control,
                deadline,
                Some(cancel),
                |identity| reader.bind_identity(identity),
            )
            .map_err(|_| io::Error::other(OwnershipCleanupError::StartupUnconfirmed))?;
        }
        Ownership::attach_controlled(&child, control, expects_service, &execution)
    })();
    #[cfg(not(any(target_os = "linux", windows)))]
    let attached = Ownership::attach(&mut child);
    #[cfg(not(windows))]
    let ownership = match attached {
        Ok(ownership) => ownership,
        Err(error) => {
            #[cfg(not(target_os = "linux"))]
            {
                let _ = child.kill();
            }
            #[cfg(not(target_os = "linux"))]
            let _ = child.wait();
            #[cfg(target_os = "linux")]
            {
                // The lifetime alias is still open: descriptor drop alone cannot deliver EOF.
                child.shutdown_peer();
                match cleanup_startup_child(&mut child, STARTUP_NATURAL_GRACE, STARTUP_KILL_GRACE) {
                    Ok(true) => {
                        // Reaping the launcher/helper is not a terminal acknowledgment
                        // from all cookie readers, including the direct fixture backend.
                        return Err(io::Error::new(
                            error.kind(),
                            OwnershipCleanupError::StartupUnconfirmed,
                        ));
                    }
                    Ok(false) => {
                        child.defer();
                        return Err(io::Error::other(OwnershipCleanupError::StartupDeferred));
                    }
                    Err(_) => {
                        child.defer();
                        return Err(io::Error::other(OwnershipCleanupError::StartupUnconfirmed));
                    }
                }
            }
            #[cfg(not(target_os = "linux"))]
            return Err(error);
        }
    };
    #[cfg(any(target_os = "linux", windows))]
    let mut terminal_confirmed = false;
    let result = (|| -> io::Result<MetadataResult> {
        let evidence = Arc::new(Mutex::new(Evidence::default()));
        let oversized = Arc::new(AtomicBool::new(false));
        let out = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("stdout pipe missing"))?;
        let err = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("stderr pipe missing"))?;
        #[cfg(any(target_os = "linux", windows))]
        let stop = Arc::new(AtomicBool::new(false));
        #[cfg(any(target_os = "linux", windows))]
        let aborted = Arc::new(AtomicBool::new(false));
        #[cfg(any(target_os = "linux", windows))]
        let _stop_drains = StopDrains(stop.clone());
        #[cfg(target_os = "linux")]
        let out = guardian::InterruptiblePipe::new(out, stop.clone(), aborted.clone())?;
        #[cfg(target_os = "linux")]
        let err = guardian::InterruptiblePipe::new(err, stop.clone(), aborted.clone())?;
        #[cfg(windows)]
        let out = windows::InterruptiblePipe::new(out, stop.clone(), aborted.clone())?;
        #[cfg(windows)]
        let err = windows::InterruptiblePipe::new(err, stop.clone(), aborted.clone())?;
        let limit = spec.max_log_bytes;
        // Keep machine stdout evidence from being evicted by unrelated stderr floods.
        let stdout_limit = if metadata { 0 } else { limit / 2 };
        let stderr_limit = limit - stdout_limit;
        let stderr_evidence = Arc::new(Mutex::new(Evidence::default()));
        let ev = evidence.clone();
        let flag = oversized.clone();
        let capture = Arc::new(Mutex::new(Vec::new()));
        let captured = capture.clone();
        let stdout_progress = progress.clone();
        let stdout = thread::spawn(move || {
            if metadata {
                drain_metadata(out, captured, flag)
            } else {
                drain(out, ev, stdout_limit, flag, stdout_progress, false)
            }
        });
        let ev = stderr_evidence.clone();
        let flag = oversized.clone();
        let stderr = thread::spawn(move || drain(err, ev, stderr_limit, flag, progress, true));
        let mut outcome = ProcessOutcome::Exited;
        let status = loop {
            if oversized.load(Ordering::Acquire) {
                outcome = ProcessOutcome::OutputLimit;
            } else if cancel.load(Ordering::Acquire) {
                outcome = ProcessOutcome::Cancelled;
            } else if Instant::now() >= execution.deadline {
                outcome = ProcessOutcome::TimedOut;
            }
            if outcome != ProcessOutcome::Exited {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                let _ = ownership.signal(false);
                let grace = Instant::now();
                let mut reaped = None;
                while grace.elapsed() < Duration::from_millis(200) {
                    if let Some(status) = child.try_wait()? {
                        reaped = Some(status);
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                if reaped.is_none() {
                    ownership
                        .signal(true)
                        .map_err(|error| cleanup_stage_error("force-command", error))?;
                }
                #[cfg(target_os = "linux")]
                {
                    // No more commands are valid after the final stop request.
                    // Keep the read direction open for exact GC/LC and EOF proof.
                    ownership
                        .finish_commands()
                        .map_err(|error| cleanup_stage_error("command-half-close", error))?;
                    break match reaped {
                        Some(status) => status,
                        None => child
                            .wait_cancelled()
                            .map_err(|error| cleanup_stage_error("owned-child-wait", error))?,
                    };
                }
                #[cfg(not(target_os = "linux"))]
                {
                    break child.wait()?;
                }
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            thread::sleep(Duration::from_millis(10));
        };
        // Linux has reaped the guardian/launcher, which already owns descendant
        // cleanup. A late command can reset its queued terminal acknowledgment.
        // Other adapters still need their owned group/Job cleanup before drains.
        #[cfg(not(target_os = "linux"))]
        ownership.signal(true)?;
        #[cfg(any(target_os = "linux", windows))]
        {
            // Bound the drain wait even when ownership cleanup cannot close all writers.
            // Reading stops without signalling unproven process IDs.
            let grace = Instant::now();
            while !(stdout.is_finished() && stderr.is_finished())
                && grace.elapsed() < Duration::from_secs(1)
            {
                thread::sleep(Duration::from_millis(10));
            }
            if !(stdout.is_finished() && stderr.is_finished()) {
                stop.store(true, Ordering::Release);
            }
        }
        let drain_result = stdout
            .join()
            .map_err(|_| io::Error::other("stdout drain panicked"))?;
        let stderr_result = stderr
            .join()
            .map_err(|_| io::Error::other("stderr drain panicked"))?;
        #[cfg(any(target_os = "linux", windows))]
        if aborted.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                OwnershipCleanupError::OutputDrainInterrupted,
            ));
        }
        drain_result?;
        stderr_result?;
        #[cfg(any(target_os = "linux", windows))]
        {
            ownership
                .confirm_terminal()
                .map_err(|error| cleanup_stage_error("terminal-frame-eof", error))?;
            terminal_confirmed = true;
        }
        #[cfg(target_os = "linux")]
        if let Some(reader) = cookie_reader.as_mut() {
            reader
                .confirm_terminal()
                .map_err(|_| io::Error::other(OwnershipCleanupError::TerminalUnconfirmed))?;
        }
        if oversized.load(Ordering::Acquire) && outcome == ProcessOutcome::Exited {
            outcome = ProcessOutcome::OutputLimit;
        }
        let mut ev = evidence
            .lock()
            .map_err(|_| io::Error::other("evidence mutex poisoned"))?;
        let mut stderr_ev = stderr_evidence
            .lock()
            .map_err(|_| io::Error::other("stderr evidence mutex poisoned"))?;
        ev.truncated |= stderr_ev.truncated;
        // Preserve stream boundaries. Combined memory remains within the original budget.
        let mut logs = String::from_utf8_lossy(ev.bytes.make_contiguous()).into_owned();
        if !logs.is_empty() && !logs.ends_with('\n') && !stderr_ev.bytes.is_empty() {
            logs.push('\n');
        }
        logs.push_str(&String::from_utf8_lossy(stderr_ev.bytes.make_contiguous()));
        // Lossy conversion can expand bytes; truncate on UTF-8 boundary after conversion.
        if logs.len() > limit {
            let mut end = limit;
            while !logs.is_char_boundary(end) {
                end -= 1;
            }
            logs.truncate(end);
            ev.truncated = true;
        }
        let stdout = std::mem::take(
            &mut *capture
                .lock()
                .map_err(|_| io::Error::other("metadata mutex poisoned"))?,
        );
        Ok(MetadataResult {
            stdout,
            execution: ExecutionResult {
                exit_code: status.code(),
                logs,
                truncated: ev.truncated,
                outcome,
            },
        })
    })();
    #[cfg(target_os = "linux")]
    if result.is_err() {
        let _ = child.cleanup_error();
    }
    #[cfg(any(target_os = "linux", windows))]
    {
        result.map_err(|error| classify_ready_error(error, terminal_confirmed))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    result
}

#[cfg(any(target_os = "linux", windows))]
fn classify_ready_error(error: io::Error, terminal_confirmed: bool) -> io::Error {
    if terminal_confirmed || cleanup_is_unconfirmed(&error) {
        error
    } else {
        io::Error::new(error.kind(), OwnershipCleanupError::TerminalUnconfirmed)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn processing_records_are_stderr_only_and_independent_from_download_speed() {
        let feed = crate::metrics::ProgressFeed::default();
        feed.set_processing_duration(Some(10_000_000));
        let result = run_with_progress(
            &spec(
                r#"import sys
print('frame=1\nout_time_us=9000000\nspeed=99x\nprogress=continue',flush=True)
print('DLPAD_PROGRESS:{"speed":1000,"eta":10}',flush=True)
sys.stderr.write('frame=2\nout_time_us=2000000\nspeed=2x\nprogress=continue\n');sys.stderr.flush()
"#,
            ),
            &Arc::new(AtomicBool::new(false)),
            feed.clone(),
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(1000.0));
        let processing = feed.processing_snapshot().unwrap();
        assert_eq!(processing.output_time_us, Some(2_000_000));
        assert_eq!(processing.speed, Some(2.0));
        assert_eq!(processing.eta_seconds, Some(4.0));
    }
    #[test]
    fn progress_fragments_both_pipes_and_unterminated_line() {
        let progress = crate::metrics::ProgressFeed::default();
        let result = run_with_progress(
            &spec(
                r#"
import sys, time
sys.stdout.write('DLPAD_PRO'); sys.stdout.flush(); time.sleep(0.02)
sys.stdout.write('GRESS:{"speed":10,"eta":4}\n'); sys.stdout.flush()
time.sleep(0.02)
sys.stderr.write('DLPAD_PROGRESS:{"speed":20,"eta":2}'); sys.stderr.flush()
"#,
            ),
            &Arc::new(AtomicBool::new(false)),
            progress.clone(),
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert!(matches!(
            progress.snapshot().unwrap().bytes_per_second,
            Some(10.0) | Some(20.0)
        ));
        let result = run_with_progress(
            &spec(r#"import sys; sys.stderr.write('DLPAD_PROGRESS:{"speed":20,"eta":2}')"#),
            &Arc::new(AtomicBool::new(false)),
            progress.clone(),
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(progress.snapshot().unwrap().bytes_per_second, Some(20.0));
    }
    fn spec(script: &str) -> ProcessSpec {
        ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: PathBuf::from("python3"),
            args: vec!["-c".into(), script.into()],
            timeout: Duration::from_secs(3),
            max_log_bytes: 4096,
        }
    }
    #[test]
    fn captures_both_streams_and_nonzero() {
        let r = run(
            &spec("import sys;print('한글');print('error',file=sys.stderr);sys.exit(7)"),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(r.exit_code, Some(7));
        assert_eq!(r.outcome, ProcessOutcome::Exited);
        assert!(r.logs.contains("한글") && r.logs.contains("error"));
    }
    #[test]
    fn stderr_flood_cannot_evict_stdout_completion_evidence() {
        let result = run(
            &spec("import sys,time;print('DLPAD_ARTIFACT:completion-fixture',flush=True);time.sleep(.1);[print('stderr-'+('x'*100),file=sys.stderr) for _ in range(5000)]"),
            &Arc::new(AtomicBool::new(false)),
        ).unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert!(result.truncated);
        assert!(result.logs.len() <= 4096);
        assert!(
            result
                .logs
                .lines()
                .any(|line| line == "DLPAD_ARTIFACT:completion-fixture")
        );
        assert!(result.logs.contains("stderr-"));
    }
    #[test]
    fn flood_is_bounded() {
        let r=run(&spec("import sys; [(sys.stdout.write('x'*100+'\\n'),sys.stderr.write('y'*100+'\\n')) for _ in range(10000)]"),&Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(r.exit_code, Some(0));
        assert!(r.truncated);
        assert!(r.logs.len() <= 4096);
    }
    #[test]
    fn oversized_line_stops() {
        let r = run(
            &spec("import sys,time;sys.stdout.write('x'*100000);sys.stdout.flush();time.sleep(2)"),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(r.outcome, ProcessOutcome::OutputLimit);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn timeout_owns_nested_children() {
        fn identity(pid: u32) -> Option<(u32, u64, char)> {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
            let fields: Vec<_> = stat.rsplit_once(')')?.1.split_whitespace().collect();
            Some((
                fields.get(1)?.parse().ok()?,
                fields.get(19)?.parse().ok()?,
                fields.first()?.chars().next()?,
            ))
        }
        let folder = tempfile::tempdir().unwrap();
        let marker = folder.path().join("nested-started");
        let mut s = spec(
            r#"import os,sys,subprocess,time
child = subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(30)'])
def started(pid):
    with open('/proc/%d/stat' % pid) as f:
        return f.read().rsplit(')', 1)[1].split()[19]
with open(sys.argv[1] + '.partial', 'x') as f:
    f.write('%d %s %d %s' % (os.getpid(), started(os.getpid()), child.pid, started(child.pid)))
os.rename(sys.argv[1] + '.partial', sys.argv[1])
time.sleep(30)
"#,
        );
        s.args.push(marker.as_os_str().to_owned());
        let budget = s.timeout;
        let now = Instant::now();
        let observed = thread::spawn(move || {
            let deadline = now + Duration::from_secs(2);
            loop {
                if let Ok(bytes) = std::fs::read(&marker) {
                    assert!(bytes.len() <= 128, "fixture marker exceeded bound");
                    let text = std::str::from_utf8(&bytes).unwrap();
                    let fields: Vec<_> = text.split_whitespace().collect();
                    assert_eq!(fields.len(), 4);
                    let parent: u32 = fields[0].parse().unwrap();
                    let parent_start: u64 = fields[1].parse().unwrap();
                    let child: u32 = fields[2].parse().unwrap();
                    let child_start: u64 = fields[3].parse().unwrap();
                    let parent_live = identity(parent).expect("parent missing before timeout");
                    let child_live = identity(child).expect("nested child missing before timeout");
                    assert_eq!(parent_live.1, parent_start);
                    assert_eq!((child_live.0, child_live.1), (parent, child_start));
                    assert!(!matches!(parent_live.2, 'Z' | 'X'));
                    assert!(!matches!(child_live.2, 'Z' | 'X'));
                    assert!(
                        Instant::now() < deadline,
                        "nested start exceeded startup proof budget"
                    );
                    return [(parent, parent_start), (child, child_start)];
                }
                assert!(
                    Instant::now() < deadline,
                    "nested start marker not observed"
                );
                thread::sleep(Duration::from_millis(5));
            }
        });
        let result = run(&s, &Arc::new(AtomicBool::new(false)));
        let identities = observed.join().unwrap();
        let r = result.unwrap();
        assert_eq!(r.outcome, ProcessOutcome::TimedOut);
        assert!(now.elapsed() >= budget, "whole deadline ended prematurely");
        assert!(
            now.elapsed() < budget + Duration::from_secs(2),
            "post-deadline cleanup exceeded bound"
        );
        for (pid, start) in identities {
            assert!(
                identity(pid).is_none_or(
                    |(_, current, state)| current != start || matches!(state, 'Z' | 'X')
                ),
                "owned identity still running after timeout"
            );
        }
    }
    fn cancel_after_observed_progress(
        feed: crate::metrics::ProgressFeed,
        cancel: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            while feed.snapshot().is_none() {
                assert!(
                    Instant::now() < deadline,
                    "tool progress was not observed after readiness"
                );
                thread::sleep(Duration::from_millis(5));
            }
            cancel.store(true, Ordering::Release);
        })
    }
    #[test]
    fn cancellation() {
        let cancel = Arc::new(AtomicBool::new(false));
        let feed = crate::metrics::ProgressFeed::default();
        let t = cancel_after_observed_progress(feed.clone(), cancel.clone());
        let r = run_with_progress(
            &spec("import time;print('DLPAD_PROGRESS:{\"speed\":1,\"eta\":2}',flush=True);time.sleep(30)"),
            &cancel,
            feed,
        ).unwrap();
        t.join().unwrap();
        assert_eq!(r.outcome, ProcessOutcome::Cancelled);
    }
    #[test]
    fn metadata_cancellation() {
        let folder = tempfile::tempdir().unwrap();
        let marker = folder.path().join("synthetic-tool-started");
        let cancel = Arc::new(AtomicBool::new(false));
        let trigger = cancel.clone();
        let observed = marker.clone();
        let t = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            while !observed.is_file() {
                assert!(Instant::now() < deadline, "metadata fixture did not start");
                thread::sleep(Duration::from_millis(5));
            }
            trigger.store(true, Ordering::Release);
        });
        let mut fixture = spec(
            "import sys,time,pathlib;sys.stdout.write('x'*100000);sys.stdout.flush();pathlib.Path(sys.argv[1]).write_bytes(b'started');time.sleep(30)",
        );
        fixture.args.push(marker.into_os_string());
        let result = run_metadata(&fixture, &cancel).unwrap();
        t.join().unwrap();
        assert_eq!(result.execution.outcome, ProcessOutcome::Cancelled);
        assert!(result.stdout.len() <= MAX_METADATA_BYTES);
        assert!(result.execution.logs.is_empty());
    }
    #[test]
    fn observed_output_limit_and_cancellation_remain_bounded() {
        let feed = crate::metrics::ProgressFeed::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let oversized = run_with_progress(
            &spec("import sys;sys.stdout.write('DLPAD_PROGRESS:'+'x'*100000);sys.stdout.flush()"),
            &cancel,
            feed.clone(),
        )
        .unwrap();
        assert_eq!(oversized.outcome, ProcessOutcome::OutputLimit);
        assert!(oversized.logs.len() <= MAX_LOG_BYTES);
        assert_eq!(feed.snapshot(), None);
        let stopper = cancel_after_observed_progress(feed.clone(), cancel.clone());
        let stopped = run_with_progress(&spec("import time;print('DLPAD_PROGRESS:{\"speed\":1,\"eta\":2}',flush=True);time.sleep(20)"), &cancel, feed).unwrap();
        stopper.join().unwrap();
        assert_eq!(stopped.outcome, ProcessOutcome::Cancelled);
    }
    #[test]
    fn literal_argv_and_start_failure() {
        let mut s = spec("import sys;print(sys.argv[1])");
        s.args.push("한글 '$HOME' $(literal)".into());
        let r = run(&s, &Arc::new(AtomicBool::new(false))).unwrap();
        assert!(r.logs.contains("한글 '$HOME' $(literal)"));
        s.executable = "/does/not/exist/dlpad-test".into();
        assert!(run(&s, &Arc::new(AtomicBool::new(false))).is_err());
    }
}

#[cfg(all(test, target_os = "linux"))]
mod startup_wait_tests {
    use super::*;
    use std::os::{
        fd::AsRawFd,
        unix::{net::UnixStream, process::CommandExt},
    };

    struct FakeChild {
        checks: usize,
        killed: bool,
        after_kill: bool,
        interrupted: bool,
    }
    impl StartupChild for FakeChild {
        fn exited(&mut self) -> io::Result<bool> {
            self.checks += 1;
            if self.interrupted {
                self.interrupted = false;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            Ok(self.killed && self.after_kill)
        }
        fn kill_owned(&mut self) -> io::Result<()> {
            self.killed = true;
            Ok(())
        }
    }
    #[test]
    fn startup_cleanup_adapter_has_finite_kill_and_defer_boundaries() {
        let mut stuck = FakeChild {
            checks: 0,
            killed: false,
            after_kill: false,
            interrupted: true,
        };
        assert!(!cleanup_startup_child(&mut stuck, Duration::ZERO, Duration::ZERO).unwrap());
        assert!(stuck.killed);
        assert_eq!(stuck.checks, 2);
        let mut exits = FakeChild {
            checks: 0,
            killed: false,
            after_kill: true,
            interrupted: false,
        };
        assert!(cleanup_startup_child(&mut exits, Duration::ZERO, Duration::ZERO).unwrap());
        assert!(exits.killed);
    }
    #[test]
    fn startup_partial_frames_preserve_protocol_and_deadline() {
        use std::io::Write;
        let cancel = AtomicBool::new(false);
        for frame in [b"EP", b"GU", b"LU"] {
            let (mut app, mut peer) = UnixStream::pair().unwrap();
            let execution = ExecutionControl {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(1),
            };
            let bytes = *frame;
            let sender = thread::spawn(move || {
                peer.write_all(&bytes[..1]).unwrap();
                thread::sleep(Duration::from_millis(30));
                peer.write_all(&bytes[1..]).unwrap();
            });
            let error = read_startup_controlled(&mut app, &execution).unwrap_err();
            assert_eq!(cleanup_is_unconfirmed(&error), frame != b"EP");
            sender.join().unwrap();
        }
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        peer.write_all(b"E").unwrap();
        let start = Instant::now();
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: start + Duration::from_millis(50),
        };
        assert_eq!(
            read_startup_controlled(&mut app, &execution)
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(start.elapsed() < Duration::from_millis(500));
    }
    #[test]
    fn startup_partial_readiness_observes_cancel_and_cancel_wins_deadline() {
        use std::io::Write;
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        peer.write_all(b"G").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let requested = cancel.clone();
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            requested.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: start + Duration::from_secs(1),
        };
        assert_eq!(
            read_startup_controlled(&mut app, &execution)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
        sender.join().unwrap();
        assert!(start.elapsed() < Duration::from_millis(500));
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: Instant::now(),
        };
        assert_eq!(execution.outcome(), Some(ProcessOutcome::Cancelled));
    }
    #[test]
    fn startup_saturated_admission_observes_deadline_and_cancel_without_reserving() {
        let reaper = Arc::new(StartupReaper {
            queue: Mutex::new(StartupQueue {
                reserved: STARTUP_REAPER_SLOTS,
                pending: std::collections::VecDeque::with_capacity(STARTUP_REAPER_SLOTS),
            }),
            wake: std::sync::Condvar::new(),
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let start = Instant::now();
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: start + Duration::from_millis(50),
        };
        let error = StartupPermit::reserve_on_controlled(reaper.clone(), Some(&execution))
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_millis(500));
        let requested = cancel.clone();
        let sender = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            requested.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: start + Duration::from_secs(1),
        };
        let error = StartupPermit::reserve_on_controlled(reaper.clone(), Some(&execution))
            .err()
            .unwrap();
        sender.join().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert!(start.elapsed() < Duration::from_millis(500));
        assert_eq!(reaper.queue.lock().unwrap().reserved, STARTUP_REAPER_SLOTS);
    }
    fn test_startup_permit() -> StartupPermit {
        let cancel = AtomicBool::new(false);
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(30),
        };
        StartupPermit::reserve(&execution).unwrap()
    }
    fn fake_launcher(mode: &str) -> (std::process::Child, UnixStream) {
        let (app, peer) = UnixStream::pair().unwrap();
        let fd = peer.as_raw_fd();
        let mut command = Command::new("python3");
        command.args(["-c", "import socket,sys,time; s=socket.socket(fileno=int(sys.argv[1])); mode=sys.argv[2]; s.sendall(b'EP') if mode in ('EP','EP-EOF') else None; s.sendall(b'E') if mode=='E-only' else None; s.close() if mode=='EOF' else None; sys.exit(0 if s.recv(1)==b'' else 12) if mode=='EP-EOF' else None; time.sleep(0 if mode=='EOF' else 30)"])
            .arg(fd.to_string()).arg(mode)
            .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        unsafe {
            command.pre_exec(move || {
                if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn().unwrap();
        drop(peer);
        (child, app)
    }
    #[test]
    fn startup_ep_sleep_is_killed_and_reaped_without_blocking_wait() {
        let (mut child, control) = fake_launcher("EP");
        let started = Instant::now();
        let error = Ownership::attach(&child, control, false).err().unwrap();
        assert!(error.to_string().contains("protection setup failed"));
        assert!(
            cleanup_startup_child(&mut child, STARTUP_NATURAL_GRACE, STARTUP_KILL_GRACE).unwrap()
        );
        assert!(started.elapsed() < Duration::from_secs(8));
        assert!(child.try_wait().unwrap().is_some());
    }
    #[test]
    fn startup_no_readiness_timeout_and_eof_exit_use_same_cleanup() {
        let (mut child, mut control) = fake_launcher("noR");
        control
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        assert!(guardian::read_readiness(&mut control).is_err());
        drop(control);
        assert!(
            cleanup_startup_child(
                &mut child,
                Duration::from_millis(30),
                Duration::from_secs(2)
            )
            .unwrap()
        );
        let (mut child, control) = fake_launcher("EOF");
        assert!(Ownership::attach(&child, control, false).is_err());
        assert!(
            cleanup_startup_child(&mut child, Duration::from_secs(2), Duration::from_secs(2))
                .unwrap()
        );
        assert!(child.try_wait().unwrap().is_some());
    }
    #[test]
    fn startup_partial_cancel_keeps_owned_cleanup_unconfirmed_after_reap() {
        let (child, control) = fake_launcher("E-only");
        let alias = control.try_clone().unwrap();
        let mut owned = OwnedChildLifetime::new(child, test_startup_permit(), alias);
        let cancel = Arc::new(AtomicBool::new(false));
        let requested = cancel.clone();
        let stopper = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            requested.store(true, Ordering::Release);
        });
        let execution = ExecutionControl {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(1),
        };
        let error = Ownership::attach_controlled(&owned, control, false, &execution)
            .err()
            .unwrap();
        stopper.join().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        owned.shutdown_peer();
        assert!(cleanup_startup_child(&mut owned, Duration::ZERO, STARTUP_KILL_GRACE).unwrap());
        assert!(owned.try_wait().unwrap().is_some());
        // Reaping this helper supplies no GC/LC from possible reader descendants.
        let error = io::Error::new(error.kind(), OwnershipCleanupError::StartupUnconfirmed);
        assert!(cleanup_is_unconfirmed(&error));
    }
    #[test]
    fn startup_failure_shutdown_reaches_peer_with_lifetime_alias_alive() {
        let (child, control) = fake_launcher("EP-EOF");
        let shutdown = control.try_clone().unwrap();
        let permit = test_startup_permit();
        let mut owned = OwnedChildLifetime::new(child, permit, shutdown);
        assert!(Ownership::attach(&owned, control, false).is_err());
        // attach dropped its descriptor, but the lifetime alias keeps this same peer alive.
        assert!(owned.try_wait().unwrap().is_none());
        owned.shutdown_peer();
        assert!(
            cleanup_startup_child(&mut owned, Duration::from_secs(2), STARTUP_KILL_GRACE).unwrap()
        );
        // The fixture exits successfully only after receiving EOF, rather than being killed.
        assert!(owned.try_wait().unwrap().unwrap().success());
    }
    #[test]
    fn owned_child_unwind_closes_peer_and_reaps_exact_child() {
        let permit = test_startup_permit();
        let child = Command::new("python3")
            .args(["-c", "import time; time.sleep(30)"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let path = PathBuf::from(format!("/proc/{}/stat", child.id()));
        let original = std::fs::read_to_string(&path).unwrap();
        let ticks = original
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .to_owned();
        let (control, mut peer) = UnixStream::pair().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = OwnedChildLifetime::new(child, permit, control);
            panic!("synthetic owned worker unwind");
        }));
        assert!(result.is_err());
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        assert_eq!(peer.read(&mut [0u8; 1]).unwrap(), 0);
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match std::fs::read_to_string(&path) {
                // procfs can report ESRCH when the held task disappears during read.
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH) =>
                {
                    break;
                }
                Ok(value) => {
                    if value
                        .rsplit_once(')')
                        .unwrap()
                        .1
                        .split_whitespace()
                        .nth(19)
                        .unwrap()
                        != ticks
                    {
                        break;
                    }
                }
                Err(error) => panic!("owned identity observation failed: {error}"),
            }
            assert!(Instant::now() < deadline, "unwind child was not reaped");
            thread::sleep(Duration::from_millis(20));
        }
    }
    #[test]
    fn owned_reaped_child_drop_releases_its_lifetime_reservation() {
        let reaper = Arc::new(StartupReaper {
            queue: Mutex::new(StartupQueue {
                reserved: 0,
                pending: std::collections::VecDeque::with_capacity(STARTUP_REAPER_SLOTS),
            }),
            wake: std::sync::Condvar::new(),
        });
        let permit = StartupPermit::reserve_on(reaper.clone()).unwrap();
        let mut child = Command::new("python3")
            .args(["-c", "pass"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        assert!(child.wait().unwrap().success());
        let (control, _peer) = UnixStream::pair().unwrap();
        let guard = OwnedChildLifetime::new(child, permit, control);
        assert_eq!(reaper.queue.lock().unwrap().reserved, 1);
        drop(guard);
        let queue = reaper.queue.lock().unwrap();
        assert_eq!(queue.reserved, 0);
        assert!(queue.pending.is_empty());
    }
    #[test]
    fn startup_deferred_real_child_is_reaped_by_singleton() {
        let permit = test_startup_permit();
        let child = Command::new("python3")
            .args(["-c", "import time; time.sleep(0.1)"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let pid = child.id();
        let stat_path = PathBuf::from(format!("/proc/{pid}/stat"));
        let original = std::fs::read_to_string(&stat_path).unwrap();
        let ticks = original
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .to_owned();
        permit.transfer(child);
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match std::fs::read_to_string(&stat_path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => break,
                Ok(value) => {
                    let current = value
                        .rsplit_once(')')
                        .unwrap()
                        .1
                        .split_whitespace()
                        .nth(19)
                        .unwrap();
                    if current != ticks {
                        break;
                    } // The original identity was reaped; a PID reuse is unrelated.
                }
                Err(error) => panic!("owned child identity observation failed: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "singleton did not reap its transferred child"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
    #[test]
    fn startup_admission_waiter_wakes_when_reserved_slot_releases() {
        let reaper = Arc::new(StartupReaper {
            queue: Mutex::new(StartupQueue {
                reserved: 0,
                pending: std::collections::VecDeque::with_capacity(STARTUP_REAPER_SLOTS),
            }),
            wake: std::sync::Condvar::new(),
        });
        let mut permits = Vec::new();
        for _ in 0..STARTUP_REAPER_SLOTS {
            permits.push(StartupPermit::reserve_on(reaper.clone()).unwrap());
        }
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let waiter = thread::spawn(move || {
            started_tx.send(()).unwrap();
            let permit = StartupPermit::reserve_on(reaper).unwrap();
            done_tx.send(()).unwrap();
            drop(permit);
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
        drop(permits.pop());
        done_rx.recv_timeout(Duration::from_millis(500)).unwrap();
        waiter.join().unwrap();
    }
    #[test]
    fn startup_reservation_has_exact_capacity_and_releases_on_drop() {
        let reaper = Arc::new(StartupReaper {
            queue: Mutex::new(StartupQueue {
                reserved: 0,
                pending: std::collections::VecDeque::with_capacity(STARTUP_REAPER_SLOTS),
            }),
            wake: std::sync::Condvar::new(),
        });
        let mut permits = Vec::new();
        for _ in 0..STARTUP_REAPER_SLOTS {
            permits.push(StartupPermit::reserve_on(reaper.clone()).unwrap());
        }
        let saturated = Instant::now();
        assert!(StartupPermit::reserve_on(reaper.clone()).is_err());
        assert!(saturated.elapsed() >= STARTUP_ADMISSION_GRACE);
        assert!(saturated.elapsed() < Duration::from_secs(2));
        drop(permits.pop());
        let recovered = StartupPermit::reserve_on(reaper.clone()).unwrap();
        drop(recovered);
        drop(permits);
        assert_eq!(reaper.queue.lock().unwrap().reserved, 0);
    }
}

// Opt-in development tracing contains only fixed stage names and numeric OS errors.
// It never includes tool output, argv, credentials, paths or process identities.
fn cleanup_stage_error(stage: &'static str, error: io::Error) -> io::Error {
    #[cfg(debug_assertions)]
    if std::env::var_os("DLPAD_TEST_CLEANUP_STAGE_TRACE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        use std::io::Write;
        let _ = writeln!(
            io::stderr(),
            "dlpad-cleanup-stage stage={stage} kind={:?} errno={:?}",
            error.kind(),
            error.raw_os_error()
        );
    }
    #[cfg(not(debug_assertions))]
    let _ = stage;
    error
}

#[cfg(test)]
mod cleanup_marker_tests {
    use super::*;
    #[cfg(target_os = "linux")]
    #[test]
    fn ready_error_requires_terminal_confirmation_without_exposing_tool_text() {
        let before = classify_ready_error(
            io::Error::new(io::ErrorKind::InvalidData, "synthetic-private-text"),
            false,
        );
        assert!(cleanup_is_unconfirmed(&before));
        assert_eq!(before.kind(), io::ErrorKind::InvalidData);
        assert!(!before.to_string().contains("synthetic-private-text"));
        let after = classify_ready_error(
            io::Error::new(io::ErrorKind::InvalidData, "confirmed evidence failure"),
            true,
        );
        assert!(!cleanup_is_unconfirmed(&after));
        assert_eq!(after.kind(), io::ErrorKind::InvalidData);
        assert_eq!(after.to_string(), "confirmed evidence failure");
        let typed = classify_ready_error(
            io::Error::new(
                io::ErrorKind::TimedOut,
                OwnershipCleanupError::OutputDrainInterrupted,
            ),
            false,
        );
        assert_eq!(typed.kind(), io::ErrorKind::TimedOut);
        assert!(typed.get_ref().unwrap().is::<OwnershipCleanupError>());
        assert_eq!(
            typed.to_string(),
            OwnershipCleanupError::OutputDrainInterrupted.to_string()
        );
    }
    #[test]
    fn cleanup_marker_retains_kind_without_classifying_tool_text() {
        for marker in [
            OwnershipCleanupError::StartupDeferred,
            OwnershipCleanupError::StartupUnconfirmed,
            OwnershipCleanupError::OutputDrainInterrupted,
            OwnershipCleanupError::TerminalUnconfirmed,
        ] {
            let typed = io::Error::new(io::ErrorKind::TimedOut, marker);
            assert!(cleanup_is_unconfirmed(&typed));
            assert_eq!(typed.kind(), io::ErrorKind::TimedOut);
            let ordinary = io::Error::new(io::ErrorKind::TimedOut, marker.to_string());
            assert!(!cleanup_is_unconfirmed(&ordinary));
        }
        assert!(!cleanup_is_unconfirmed(&io::Error::other("tool failed")));
    }
}

// Pending durable-cookie launch wiring; codec is also compiled by cookie journal.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) mod reader_protocol;

#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod reader_gate;

#[cfg(target_os = "linux")]
mod reader_cgroup;
#[cfg(target_os = "linux")]
mod reader_receipt;
