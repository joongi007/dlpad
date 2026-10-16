//! Paired managed/cookie Windows capture through the independently gated helper.
use super::windows_paired_guardian_binding::PairedGuardianBinding;
use super::windows_paired_guardian_config::PairedGuardianConfig;
use super::windows_paired_guardian_protocol::{FRAME_LEN, Frame, Kind};
use super::windows_runtime_environment_config::RuntimeEnvironmentConfig;
use super::{
    Evidence, ExecutionControl, ExecutionResult, MetadataResult, OwnershipCleanupError,
    ProcessOutcome, ProcessSpec, StopDrains, drain, drain_metadata,
};
use crate::{
    cookies::SessionCookieLease,
    metrics::ProgressFeed,
    tools::{ManagedJournal, windows_managed_records::ReaderStage},
};
use std::{
    fs::File,
    io::{self, Read},
    os::windows::{ffi::OsStrExt, io::AsRawHandle},
    path::Path,
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
// Kept aligned with the process wrapper's explicit capture/context contract.
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    spec: &ProcessSpec,
    cancel: &Arc<AtomicBool>,
    metadata: bool,
    progress: Option<ProgressFeed>,
    journal: &mut ManagedJournal,
    root_path: &Path,
    index: u8,
    stage: ReaderStage,
    cookies: &SessionCookieLease,
) -> io::Result<MetadataResult> {
    let deadline = Instant::now()
        .checked_add(spec.timeout)
        .ok_or_else(unconfirmed)?;
    let execution = ExecutionControl { cancel, deadline };
    if let Some(outcome) = execution.outcome() {
        return Ok(super::interrupted_result(outcome));
    }
    let cookie_index = match (metadata, stage) {
        (true, ReaderStage::Metadata) => 0,
        (false, ReaderStage::Download) => 1,
        _ => return Err(unconfirmed()),
    };
    // Authority stays alive and its cache mutex is held for the entire capture.
    let authority = match spec.runtime_environment.as_ref() {
        None => None,
        Some(policy) => {
            policy.validate().map_err(|_| unconfirmed())?;
            policy
                .verify_original(cancel, deadline)
                .map_err(|_| unconfirmed())?;
            let binding = policy.cache_binding().ok_or_else(unconfirmed)?;
            let anchor = journal.anchors().root;
            if binding.root_path().map_err(|_| unconfirmed())? != root_path
                || binding.root
                    != (crate::file_identity::NativeFileIdentity::Windows {
                        volume_serial: anchor.volume_serial,
                        file_id: anchor.file_id,
                    })
            {
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
    let mut nonce = [0u8; 16];
    // SAFETY: fixed stack output buffer and the system-preferred cryptographic RNG.
    if unsafe {
        windows_sys::Win32::Security::Cryptography::BCryptGenRandom(
            std::ptr::null_mut(),
            nonce.as_mut_ptr(),
            nonce.len() as u32,
            windows_sys::Win32::Security::Cryptography::BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } < 0
        || nonce == [0; 16]
    {
        return Err(unconfirmed());
    }
    let writer = journal
        .begin(index, stage, nonce)
        .map_err(|_| unconfirmed())?;
    let intent = journal.record().map_err(|_| unconfirmed())?.clone();
    if index > 7
        || writer.index != index
        || intent.anchors().receipts[index as usize] != writer.anchor
    {
        return Err(unconfirmed());
    }
    let mut cookie_reader = cookies
        .begin_windows_gated_reader(cookie_index)
        .map_err(|_| unconfirmed())?;
    let cookie_launch = cookie_reader.take_launch().map_err(|_| unconfirmed())?;
    if cookie_launch.slot != cookie_index
        || cookie_launch.intent.anchors().receipts[cookie_index as usize]
            != cookie_launch.receipt_anchor
    {
        return Err(unconfirmed());
    }
    let config = PairedGuardianConfig {
        domain: "dlpad-windows-paired-guardian-config-v1".into(),
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
        managed_root_path: root_path.as_os_str().encode_wide().collect(),
        cookie_owner_path: cookie_launch.owner_path.as_os_str().encode_wide().collect(),
        managed_intent: intent,
        managed_index: index,
        managed_nonce: nonce,
        cookie_intent: cookie_launch.intent,
        cookie_index,
        cookie_nonce: cookie_launch.nonce,
        timeout_secs: spec.timeout.as_secs(),
        timeout_nanos: spec.timeout.subsec_nanos(),
        max_log_bytes: spec.max_log_bytes,
    };
    let cache_lock = cache_guard
        .as_ref()
        .map(|cache| cache.lock_file())
        .transpose()
        .map_err(|_| unconfirmed())?;
    let launched = super::windows_helper_launch::launch_paired_with_cache(
        &helper,
        &config,
        &writer.file,
        &cookie_launch.receipt,
        cache_lock,
    );
    drop(writer.file);
    drop(cookie_launch.receipt);
    let mut launch = launched.map_err(|_| unconfirmed())?;
    let stop = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));
    let _stop = StopDrains(stop.clone());
    let out = super::windows::InterruptiblePipe::new(
        launch.stdout.try_clone().map_err(|_| unconfirmed())?,
        stop.clone(),
        aborted.clone(),
    )
    .map_err(|_| unconfirmed())?;
    let err = super::windows::InterruptiblePipe::new(
        launch.stderr.try_clone().map_err(|_| unconfirmed())?,
        stop.clone(),
        aborted.clone(),
    )
    .map_err(|_| unconfirmed())?;
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
            config.managed_index,
            config.managed_nonce,
            [0; 32],
        )?;
        if prepared.guardian_created() != launch.child.creation_time().map_err(|_| unconfirmed())? {
            return Err(unconfirmed());
        }
        let bound = journal
            .bind(index, prepared.guardian_created(), prepared.tool_created())
            .map_err(|_| unconfirmed())?;
        let cookie_bound = cookie_reader
            .bind_identity(prepared.guardian_created(), prepared.tool_created())
            .map_err(|_| unconfirmed())?;
        let mut cookie_record = config.cookie_intent.clone();
        let reconstructed = cookie_record
            .bind(
                cookie_index,
                prepared.guardian_created(),
                prepared.tool_created(),
            )
            .map_err(|_| unconfirmed())?;
        if reconstructed != cookie_bound {
            return Err(unconfirmed());
        }
        let pair = PairedGuardianBinding::new(
            journal.record().map_err(|_| unconfirmed())?,
            index,
            &cookie_record,
            cookie_index,
        )
        .map_err(|_| unconfirmed())?;
        if pair.managed_identity() != bound || pair.cookie_identity() != cookie_bound {
            return Err(unconfirmed());
        }
        let digest = pair.digest();
        journal
            .release_for_guardian(index, bound)
            .map_err(|_| unconfirmed())?;
        cookie_reader
            .release_for_guardian()
            .map_err(|_| unconfirmed())?;
        if let Some(cache) = cache_guard.as_mut() {
            cache.release_for_guardian().map_err(|_| unconfirmed())?;
        }
        let ack = Frame::new(
            Kind::Ack,
            config.managed_index,
            config.managed_nonce,
            digest,
            (0, 0),
            0,
        )
        .map_err(|_| unconfirmed())?;
        launch
            .send_paired_frame(&ack, &execution)
            .map_err(|_| unconfirmed())?;
        let ack = events.wait(&mut launch.events, &execution)?;
        expected(
            &ack,
            Kind::Ack,
            config.managed_index,
            config.managed_nonce,
            digest,
        )?;
        if execution.outcome().is_some() {
            return Err(unconfirmed());
        }
        let begin = Frame::new(
            Kind::Begin,
            config.managed_index,
            config.managed_nonce,
            digest,
            (0, 0),
            0,
        )
        .map_err(|_| unconfirmed())?;
        launch
            .send_paired_frame(&begin, &execution)
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
                        .send_paired_frame(
                            &Frame::new(
                                kind,
                                config.managed_index,
                                config.managed_nonce,
                                [0; 32],
                                (0, 0),
                                0,
                            )
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
                        config.managed_index,
                        config.managed_nonce,
                        digest,
                    )?;
                    if terminal.replace(frame.code()).is_some() {
                        return Err(unconfirmed());
                    }
                }
                Event::Eof => {
                    let code = terminal.ok_or_else(unconfirmed)?;
                    let status = launch.child.wait().map_err(|_| unconfirmed())?;
                    if status.code().ok_or_else(unconfirmed)? as u32 != code {
                        return Err(unconfirmed());
                    }
                    journal.reacquire().map_err(|_| unconfirmed())?;
                    if let Some(cache) = cache_guard.as_mut() {
                        // Original strict helper proof has completed; pending journals are not cleared yet. A failed try-lock
                        // preserves cache authority and is never promoted to cleanup proof.
                        cache.reacquire().map_err(|_| unconfirmed())?;
                    }
                    journal.resolve(index).map_err(|_| unconfirmed())?;
                    cookie_reader
                        .confirm_terminal()
                        .map_err(|_| unconfirmed())?;
                    terminal_confirmed = true;
                    return Ok((code, outcome));
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    })();
    // Any failed protocol leaves journal pending. Close control promptly; drop owned helper on error.
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
        if let Ok(frame) = Frame::new(
            kind,
            config.managed_index,
            config.managed_nonce,
            [0; 32],
            (0, 0),
            0,
        ) {
            let _ = launch.send_paired_frame(&frame, &cleanup);
        }
        // Do not grant terminal authority to an arbitrary error-path frame.
        // Give the real helper time to empty its Job and write its original receipt;
        // journal stays pending unless the normal strict terminal path succeeded.
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
