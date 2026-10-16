//! Authenticated Windows capture through the independently gated helper.
use super::windows_guardian_config::GuardianConfig;
use super::windows_guardian_protocol::{FRAME_LEN, Frame, Kind};
use super::windows_runtime_environment_config::RuntimeEnvironmentConfig;
use super::{
    Evidence, ExecutionControl, ExecutionResult, MetadataResult, OwnershipCleanupError,
    ProcessOutcome, ProcessSpec, StopDrains, drain, drain_metadata,
};
use crate::{cookies::SessionCookieLease, metrics::ProgressFeed};
use std::{
    fs::File,
    io::{self, Read},
    os::windows::{ffi::OsStrExt, io::AsRawHandle},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::ERROR_BROKEN_PIPE, System::Pipes::PeekNamedPipe};
fn unconfirmed() -> io::Error {
    io::Error::other(OwnershipCleanupError::TerminalUnconfirmed)
}
struct Events {
    bytes: [u8; FRAME_LEN],
    filled: usize,
}
enum Event {
    Pending,
    Frame(Frame),
    Eof,
}
impl Events {
    fn poll(&mut self, file: &mut File) -> io::Result<Event> {
        let mut available = 0;
        // SAFETY: original retained pipe reader and valid count output.
        if unsafe {
            PeekNamedPipe(
                file.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            if io::Error::last_os_error().raw_os_error() == Some(ERROR_BROKEN_PIPE as i32)
                && self.filled == 0
            {
                return Ok(Event::Eof);
            }
            return Err(unconfirmed());
        }
        if available == 0 {
            return Ok(Event::Pending);
        }
        let count = (available as usize).min(FRAME_LEN - self.filled);
        let got = file
            .read(&mut self.bytes[self.filled..self.filled + count])
            .map_err(|_| unconfirmed())?;
        if got == 0 {
            return Err(unconfirmed());
        }
        self.filled += got;
        if self.filled < FRAME_LEN {
            return Ok(Event::Pending);
        }
        let frame = Frame::decode(&self.bytes).map_err(|_| unconfirmed())?;
        self.filled = 0;
        Ok(Event::Frame(frame))
    }
    fn wait(&mut self, file: &mut File, execution: &ExecutionControl<'_>) -> io::Result<Frame> {
        loop {
            if execution.outcome().is_some() {
                return Err(unconfirmed());
            }
            match self.poll(file)? {
                Event::Frame(value) => return Ok(value),
                Event::Eof => return Err(unconfirmed()),
                Event::Pending => thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}
fn expected(
    frame: &Frame,
    kind: Kind,
    slot: u8,
    nonce: [u8; 16],
    digest: [u8; 32],
) -> io::Result<()> {
    if frame.kind() == kind
        && frame.slot() == slot
        && frame.nonce() == nonce
        && frame.digest() == digest
    {
        Ok(())
    } else {
        Err(unconfirmed())
    }
}
pub(super) fn run(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<ProgressFeed>,
    lease: &SessionCookieLease,
    index: u8,
) -> io::Result<MetadataResult> {
    #[cfg(test)]
    let result = run_inner(spec, cancel, metadata, progress, lease, index, None);
    #[cfg(not(test))]
    let result = run_inner(spec, cancel, metadata, progress, lease, index);
    result
}
fn run_inner(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<ProgressFeed>,
    lease: &SessionCookieLease,
    index: u8,
    #[cfg(test)] on_terminal: Option<&dyn Fn()>,
) -> io::Result<MetadataResult> {
    let deadline = Instant::now()
        .checked_add(spec.timeout)
        .ok_or_else(unconfirmed)?;
    let execution = ExecutionControl { cancel, deadline };
    if let Some(outcome) = execution.outcome() {
        return Ok(super::interrupted_result(outcome));
    }
    // Authority stays alive and its cache mutex is held for the entire capture.
    let authority = match spec.runtime_environment.as_ref() {
        None => None,
        Some(policy) => {
            policy.validate().map_err(|_| unconfirmed())?;
            policy
                .verify_original(cancel, deadline)
                .map_err(|_| unconfirmed())?;
            if policy.cache_binding().is_none() {
                return Err(unconfirmed());
            }
            Some(policy.authority().ok_or_else(unconfirmed)?.clone())
        }
    };
    let mut cache_guard = match authority.as_ref() {
        None => None,
        Some(authority) => {
            let guard = loop {
                if let Some(outcome) = execution.outcome() {
                    return Ok(super::interrupted_result(outcome));
                }
                match authority.cache.try_lock() {
                    Ok(guard) => break guard,
                    Err(std::sync::TryLockError::WouldBlock) => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(std::sync::TryLockError::Poisoned(_)) => return Err(unconfirmed()),
                }
            };
            guard.verify().map_err(|_| unconfirmed())?;
            let expected = spec
                .runtime_environment
                .as_ref()
                .and_then(|policy| policy.cache_binding())
                .ok_or_else(unconfirmed)?;
            if guard.binding().map_err(|_| unconfirmed())? != *expected {
                return Err(unconfirmed());
            }
            Some(guard)
        }
    };
    if let Some(outcome) = execution.outcome() {
        return Ok(super::interrupted_result(outcome));
    }
    let helper = super::windows_guardian_registration::executable()?;
    let mut reader = lease
        .begin_windows_gated_reader(index)
        .map_err(|_| unconfirmed())?;
    let request = reader.take_launch().map_err(|_| unconfirmed())?;
    if request.slot > 1
        || request.intent.anchors().receipts[request.slot as usize] != request.receipt_anchor
    {
        return Err(unconfirmed());
    }
    let config = GuardianConfig {
        version: 1,
        runtime_environment: spec
            .runtime_environment
            .as_ref()
            .map(RuntimeEnvironmentConfig::from_native)
            .transpose()
            .map_err(|_| unconfirmed())?,
        executable: spec.executable.as_os_str().encode_wide().collect(),
        args: spec
            .args
            .iter()
            .map(|arg| arg.encode_wide().collect())
            .collect(),
        owner_path: request.owner_path.as_os_str().encode_wide().collect(),
        intent: request.intent,
        slot: request.slot,
        nonce: request.nonce,
        timeout_secs: spec.timeout.as_secs(),
        timeout_nanos: spec.timeout.subsec_nanos(),
        max_log_bytes: spec.max_log_bytes,
    };
    let cache_lock = cache_guard
        .as_ref()
        .map(|cache| cache.lock_file())
        .transpose()
        .map_err(|_| unconfirmed())?;
    let launched = super::windows_helper_launch::launch_with_cache(
        &helper,
        &config,
        &request.receipt,
        cache_lock,
    );
    drop(request.receipt);
    let mut launch = launched.map_err(|_| unconfirmed())?;
    let stop = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));
    let _stop = StopDrains(stop.clone());
    let out = super::windows::InterruptiblePipe::new(
        launch.stdout.try_clone()?,
        stop.clone(),
        aborted.clone(),
    )?;
    let err = super::windows::InterruptiblePipe::new(
        launch.stderr.try_clone()?,
        stop.clone(),
        aborted.clone(),
    )?;
    let evidence = Arc::new(Mutex::new(Evidence::default()));
    let stderr_evidence = Arc::new(Mutex::new(Evidence::default()));
    let oversized = Arc::new(AtomicBool::new(false));
    let capture = Arc::new(Mutex::new(Vec::new()));
    let limit = spec.max_log_bytes;
    let stdout_limit = if metadata { 0 } else { limit / 2 };
    let stderr_limit = limit - stdout_limit;
    let ev = evidence.clone();
    let flag = oversized.clone();
    let captured = capture.clone();
    let feed = progress.clone();
    let stdout = thread::spawn(move || {
        if metadata {
            drain_metadata(out, captured, flag)
        } else {
            drain(out, ev, stdout_limit, flag, feed, false)
        }
    });
    let ev = stderr_evidence.clone();
    let flag = oversized.clone();
    let stderr = thread::spawn(move || drain(err, ev, stderr_limit, flag, progress, true));
    let mut terminal_confirmed = false;
    let result = (|| -> io::Result<(u32, ProcessOutcome)> {
        launch
            .transmit_config(&execution)
            .map_err(|_| unconfirmed())?;
        let mut events = Events {
            bytes: [0; FRAME_LEN],
            filled: 0,
        };
        let prepared = events.wait(&mut launch.events, &execution)?;
        expected(
            &prepared,
            Kind::Prepared,
            config.slot,
            config.nonce,
            [0; 32],
        )?;
        if prepared.guardian_created() != launch.child.creation_time().map_err(|_| unconfirmed())? {
            return Err(unconfirmed());
        }
        let bound = reader
            .bind_identity(prepared.guardian_created(), prepared.tool_created())
            .map_err(|_| unconfirmed())?;
        reader.release_for_guardian().map_err(|_| unconfirmed())?;
        if let Some(cache) = cache_guard.as_mut() {
            cache.release_for_guardian().map_err(|_| unconfirmed())?;
        }
        let ack = Frame::new(
            Kind::Ack,
            config.slot,
            config.nonce,
            bound.identity_digest,
            (0, 0),
            0,
        )
        .map_err(|_| unconfirmed())?;
        launch
            .send_frame(&ack, &execution)
            .map_err(|_| unconfirmed())?;
        let ack = events.wait(&mut launch.events, &execution)?;
        expected(
            &ack,
            Kind::Ack,
            config.slot,
            config.nonce,
            bound.identity_digest,
        )?;
        if execution.outcome().is_some() {
            return Err(unconfirmed());
        }
        let begin = Frame::new(
            Kind::Begin,
            config.slot,
            config.nonce,
            bound.identity_digest,
            (0, 0),
            0,
        )
        .map_err(|_| unconfirmed())?;
        launch
            .send_frame(&begin, &execution)
            .map_err(|_| unconfirmed())?;
        let mut outcome = ProcessOutcome::Exited;
        let mut cleanup_deadline = None;
        let mut terminal = None;
        loop {
            if terminal.is_none() && cleanup_deadline.is_none() {
                outcome = if oversized.load(Ordering::Acquire) {
                    ProcessOutcome::OutputLimit
                } else {
                    execution.outcome().unwrap_or(ProcessOutcome::Exited)
                };
                if outcome != ProcessOutcome::Exited {
                    let fresh = AtomicBool::new(false);
                    let end = Instant::now() + Duration::from_secs(2);
                    let cleanup = ExecutionControl {
                        cancel: &fresh,
                        deadline: end,
                    };
                    let kind = if outcome == ProcessOutcome::TimedOut {
                        Kind::TimedOut
                    } else {
                        Kind::Cancel
                    };
                    launch
                        .send_frame(
                            &Frame::new(kind, config.slot, config.nonce, [0; 32], (0, 0), 0)
                                .map_err(|_| unconfirmed())?,
                            &cleanup,
                        )
                        .map_err(|_| unconfirmed())?;
                    cleanup_deadline = Some(end);
                }
            }
            if Instant::now() >= cleanup_deadline.unwrap_or(deadline + Duration::from_secs(2)) {
                return Err(unconfirmed());
            }
            match events.poll(&mut launch.events)? {
                Event::Pending => {}
                Event::Frame(frame) => {
                    expected(
                        &frame,
                        Kind::Terminal,
                        config.slot,
                        config.nonce,
                        bound.identity_digest,
                    )?;
                    if terminal.replace(frame.code()).is_some() {
                        return Err(unconfirmed());
                    }
                    #[cfg(test)]
                    if let Some(hook) = on_terminal {
                        hook();
                    }
                }
                Event::Eof => {
                    let code = terminal.ok_or_else(unconfirmed)?;
                    let status = launch.child.wait().map_err(|_| unconfirmed())?;
                    if status.code().ok_or_else(unconfirmed)? as u32 != code {
                        return Err(unconfirmed());
                    }
                    if let Some(cache) = cache_guard.as_mut() {
                        // Original strict helper proof has completed; pending journals are not cleared yet. A failed try-lock
                        // preserves cache authority and is never promoted to cleanup proof.
                        cache.reacquire().map_err(|_| unconfirmed())?;
                    }
                    reader.confirm_terminal().map_err(|_| unconfirmed())?;
                    terminal_confirmed = true;
                    return Ok((code, outcome));
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    })();
    // Any failed protocol leaves reader pending. Close control promptly; drop owned helper on error.
    if result.is_err() {
        // Startup cancellation and protocol failure must request owned helper cleanup
        // with a fresh, non-cancelled budget. Any uncertain receipt remains pending.
        let fresh = AtomicBool::new(false);
        let cleanup_deadline = Instant::now() + Duration::from_secs(2);
        let cleanup = ExecutionControl {
            cancel: &fresh,
            deadline: cleanup_deadline,
        };
        let kind = if execution.outcome() == Some(ProcessOutcome::TimedOut) {
            Kind::TimedOut
        } else {
            Kind::Cancel
        };
        if let Ok(frame) = Frame::new(kind, config.slot, config.nonce, [0; 32], (0, 0), 0) {
            let _ = launch.send_frame(&frame, &cleanup);
        }
        // Do not grant terminal authority to an arbitrary error-path frame.
        // Give the real helper time to empty its Job and write its original receipt;
        // reader stays pending unless the normal strict terminal path succeeded.
        while Instant::now() < cleanup_deadline {
            match launch.child.try_wait() {
                Ok(Some(_)) | Err(_) => break,
                Ok(None) => thread::sleep(Duration::from_millis(10)),
            }
        }
        drop(launch);
    }
    let drain_deadline = Instant::now() + Duration::from_secs(1);
    while !(stdout.is_finished() && stderr.is_finished()) && Instant::now() < drain_deadline {
        thread::sleep(Duration::from_millis(10))
    }
    if !(stdout.is_finished() && stderr.is_finished()) {
        stop.store(true, Ordering::Release)
    }
    let out_result = stdout.join().map_err(|_| unconfirmed());
    let err_result = stderr.join().map_err(|_| unconfirmed());
    let (code, mut outcome) = result?;
    out_result??;
    err_result??;
    if aborted.load(Ordering::Acquire) {
        return Err(io::Error::other(
            OwnershipCleanupError::OutputDrainInterrupted,
        ));
    }
    if !terminal_confirmed {
        return Err(unconfirmed());
    }
    if oversized.load(Ordering::Acquire) && outcome == ProcessOutcome::Exited {
        outcome = ProcessOutcome::OutputLimit
    }
    let mut ev = evidence.lock().map_err(|_| unconfirmed())?;
    let mut err = stderr_evidence.lock().map_err(|_| unconfirmed())?;
    ev.truncated |= err.truncated;
    let mut logs = String::from_utf8_lossy(ev.bytes.make_contiguous()).into_owned();
    if !logs.is_empty() && !logs.ends_with('\n') && !err.bytes.is_empty() {
        logs.push('\n')
    }
    logs.push_str(&String::from_utf8_lossy(err.bytes.make_contiguous()));
    if logs.len() > limit {
        let mut end = limit;
        while !logs.is_char_boundary(end) {
            end -= 1
        }
        logs.truncate(end);
        ev.truncated = true;
    }
    let stdout = std::mem::take(&mut *capture.lock().map_err(|_| unconfirmed())?);
    Ok(MetadataResult {
        stdout,
        execution: ExecutionResult {
            exit_code: Some(code as i32),
            logs,
            truncated: ev.truncated,
            outcome,
        },
    })
}

#[cfg(test)]
mod tests {
    // Fragment for windows_cookie_capture::tests. Root adds cfg(test)-only run_inner hook:
    // run_inner(..., on_terminal: Option<&dyn Fn()>), invoked once immediately AFTER
    // expected Terminal parsing and terminal.replace(code), BEFORE the next loop/EOF.
    // Normal product run passes None; no environment/debug-release hook is introduced.
    #[test]
    #[ignore = "requires freshly built native helper and DLPAD_WINDOWS_COOKIE_FIXTURE_EXE"]
    fn actual_cookie_terminal_then_cancel_does_not_send_late_command() {
        use std::{
            ffi::OsString,
            fs,
            sync::{
                Arc,
                atomic::{AtomicBool, Ordering},
            },
            time::Duration,
        };
        const COOKIE:&[u8]=b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-download-fixture-only\n";
        let temp = tempfile::Builder::new()
            .prefix("dlpad-windows-cookie-download-owned-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("source.cookies");
        fs::write(&source, COOKIE).unwrap();
        fs::write(
            root.join("fixture.cap"),
            b"dlpad-windows-session-cookie-synthetic-fixture-v1",
        )
        .unwrap();
        fs::write(root.join("mode"), b"success").unwrap();
        let built = std::path::PathBuf::from(
            std::env::var_os("DLPAD_WINDOWS_COOKIE_FIXTURE_EXE")
                .expect("native synthetic example required"),
        );
        assert!(built.is_absolute() && built.is_file());
        let tool = root.join("fixture-tool.exe");
        fs::copy(built, &tool).unwrap();
        let cache = root.join("private-cache");
        crate::cookies::prepare_session_cache(&cache).unwrap();
        let sentinel = cache.join("foreign-sentinel");
        fs::write(&sentinel, b"preserve").unwrap();
        let mut lease = crate::cookies::import_session(&source, &cache).unwrap();
        let private = lease.path_for_tool().to_path_buf();
        let owner = private.parent().unwrap().to_path_buf();
        let cancel = Arc::new(AtomicBool::new(false));
        let hook_called = AtomicBool::new(false);
        let hook = || {
            assert!(!hook_called.swap(true, Ordering::AcqRel));
            cancel.store(true, Ordering::Release);
        };
        let spec = super::super::ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: tool,
            args: vec![
                OsString::from("--dump-single-json"),
                OsString::from("--cookies"),
                private.as_os_str().to_owned(),
                OsString::from("--"),
                OsString::from("https://fixture.invalid/native-cookie"),
            ],
            timeout: Duration::from_secs(8),
            max_log_bytes: 1024,
        };
        let result = super::run_inner(&spec, &cancel, true, None, &lease, 0, Some(&hook)).unwrap();
        assert!(hook_called.load(Ordering::Acquire));
        assert_eq!(
            result.execution.outcome,
            super::super::ProcessOutcome::Exited
        );
        assert_eq!(result.execution.exit_code, Some(0));
        assert!(!result.stdout.is_empty());
        assert_eq!(fs::read(owner.join("reader0.receipt")).unwrap().len(), 256);
        // This uses actual lease journal/receipt eligibility, not a synthesized resolved record.
        lease.validate_for_tool().unwrap();
        lease.cleanup().unwrap();
        assert!(!owner.exists());
        assert_eq!(fs::read(source).unwrap(), COOKIE);
        assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
    }
}
