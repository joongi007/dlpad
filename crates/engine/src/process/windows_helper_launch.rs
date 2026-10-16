//! Exact helper capability launch. Configuration transmission is caller-owned bounded I/O.
use super::windows::{self, Child};
use super::windows_guardian_config::{GuardianConfig, MAX_CONFIG_BYTES};
use std::{
    ffi::OsString,
    fs::File,
    io::{self, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    ptr,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE},
    System::{
        Pipes::{CreatePipe, PIPE_NOWAIT, SetNamedPipeHandleState},
        Threading::GetCurrentProcess,
    },
};

pub(super) const MARKER: &str = "--dlpad-internal-windows-cookie-guardian";
pub(super) struct Launch {
    pub child: Child,
    pub control: File,
    pub events: File,
    pub stdout: File,
    pub stderr: File,
    /// Four-byte little-endian JSON length followed by at most 256 KiB validated JSON.
    /// Caller must use cancellation/deadline-aware transmission, never an unbounded write_all.
    pub config_bytes: Vec<u8>,
    config_attempted: bool,
    config_sent: bool,
}
fn rejected() -> io::Error {
    io::Error::other("Windows helper launch failed")
}
fn pipe() -> io::Result<(File, File)> {
    let mut read = ptr::null_mut();
    let mut write = ptr::null_mut();
    // SAFETY: null security attributes make both fresh handles non-inheritable.
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 4096) } == 0 {
        return Err(rejected());
    }
    // SAFETY: CreatePipe returned two distinct newly owned handles.
    Ok(unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) })
}
fn inherited(source: &File) -> io::Result<OwnedHandle> {
    let mut duplicate = ptr::null_mut();
    // SAFETY: live source, current-process pseudo handles, and new owned output.
    // Only the duplicate is inheritable; source flags and original receipt are untouched.
    if unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            source.as_raw_handle(),
            GetCurrentProcess(),
            &mut duplicate,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    } == 0
    {
        return Err(rejected());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(duplicate) })
}

pub(super) fn launch(helper: &Path, config: &GuardianConfig, receipt: &File) -> io::Result<Launch> {
    launch_with_cache(helper, config, receipt, None)
}
pub(super) fn launch_with_cache(
    helper: &Path,
    config: &GuardianConfig,
    receipt: &File,
    cache_lock: Option<&File>,
) -> io::Result<Launch> {
    let json = config.encode().map_err(|_| rejected())?;
    if config
        .runtime_environment
        .as_ref()
        .and_then(|policy| policy.cache_binding())
        .is_some()
        != cache_lock.is_some()
    {
        return Err(rejected());
    }
    launch_encoded(helper, MARKER, &json, receipt, None, cache_lock)
}
pub(super) fn launch_managed(
    helper: &Path,
    config: &super::windows_managed_guardian_config::ManagedGuardianConfig,
    receipt: &File,
) -> io::Result<Launch> {
    launch_managed_with_cache(helper, config, receipt, None)
}
pub(super) fn launch_managed_with_cache(
    helper: &Path,
    config: &super::windows_managed_guardian_config::ManagedGuardianConfig,
    receipt: &File,
    cache_lock: Option<&File>,
) -> io::Result<Launch> {
    let json = config.encode().map_err(|_| rejected())?;
    if config
        .runtime_environment
        .as_ref()
        .and_then(|policy| policy.cache_binding())
        .is_some()
        != cache_lock.is_some()
    {
        return Err(rejected());
    }
    launch_encoded(
        helper,
        super::windows_managed_guardian_entry::MARKER,
        &json,
        receipt,
        None,
        cache_lock,
    )
}
pub(super) fn launch_paired(
    helper: &Path,
    config: &super::windows_paired_guardian_config::PairedGuardianConfig,
    managed_receipt: &File,
    cookie_receipt: &File,
) -> io::Result<Launch> {
    launch_paired_with_cache(helper, config, managed_receipt, cookie_receipt, None)
}
pub(super) fn launch_paired_with_cache(
    helper: &Path,
    config: &super::windows_paired_guardian_config::PairedGuardianConfig,
    managed_receipt: &File,
    cookie_receipt: &File,
    cache_lock: Option<&File>,
) -> io::Result<Launch> {
    let json = config.encode().map_err(|_| rejected())?;
    if config
        .runtime_environment
        .as_ref()
        .and_then(|policy| policy.cache_binding())
        .is_some()
        != cache_lock.is_some()
    {
        return Err(rejected());
    }
    launch_encoded(
        helper,
        super::windows_paired_guardian_entry::MARKER,
        &json,
        managed_receipt,
        Some(cookie_receipt),
        cache_lock,
    )
}

fn launch_encoded(
    helper: &Path,
    marker: &str,
    json: &[u8],
    receipt: &File,
    extra_receipt: Option<&File>,
    cache_lock: Option<&File>,
) -> io::Result<Launch> {
    if json.len() > MAX_CONFIG_BYTES {
        return Err(rejected());
    }
    let length = u32::try_from(json.len()).map_err(|_| rejected())?;
    let mut config_bytes = Vec::with_capacity(4 + json.len());
    config_bytes.extend_from_slice(&length.to_le_bytes());
    config_bytes.extend_from_slice(json);
    let (control_reader, control) = pipe()?;
    let mode = PIPE_NOWAIT;
    // SAFETY: original app-owned anonymous pipe writer; no external handle is changed.
    // Immediate writes plus explicit polling are synchronous bounded I/O, not overlapped I/O.
    if unsafe { SetNamedPipeHandleState(control.as_raw_handle(), &mode, ptr::null(), ptr::null()) }
        == 0
    {
        return Err(rejected());
    }
    let (events, events_writer) = pipe()?;
    let (stdout, stdout_writer) = pipe()?;
    let (stderr, stderr_writer) = pipe()?;
    let mut caps = vec![
        inherited(&control_reader)?,
        inherited(&events_writer)?,
        inherited(receipt)?,
        inherited(&stdout_writer)?,
        inherited(&stderr_writer)?,
    ];
    if let Some(extra) = extra_receipt {
        caps.insert(3, inherited(extra)?);
    }
    if let Some(cache) = cache_lock {
        caps.push(inherited(cache)?);
    }
    let handles: Vec<HANDLE> = caps.iter().map(AsRawHandle::as_raw_handle).collect();
    let mut args = vec![OsString::from(marker)];
    args.extend(
        handles
            .iter()
            .map(|handle| OsString::from((*handle as usize).to_string())),
    );
    let child =
        windows::spawn_independent_helper(helper, &args, &handles).map_err(|_| rejected())?;
    // Drop all helper-end handles immediately. Only parent directions remain in Launch;
    // retaining writers here would suppress events/stdout/stderr EOF on helper exit.
    drop(caps);
    drop(control_reader);
    drop(events_writer);
    drop(stdout_writer);
    drop(stderr_writer);
    Ok(Launch {
        child,
        control,
        events,
        stdout,
        stderr,
        config_bytes,
        config_attempted: false,
        config_sent: false,
    })
}

impl Launch {
    pub(super) fn transmit_config(
        &mut self,
        execution: &super::ExecutionControl<'_>,
    ) -> io::Result<()> {
        if self.config_attempted {
            return Err(rejected());
        }
        self.config_attempted = true;
        let bytes = std::mem::take(&mut self.config_bytes);
        self.write_before(&bytes, execution)?;
        self.config_sent = true;
        Ok(())
    }
    fn write_before(
        &mut self,
        bytes: &[u8],
        execution: &super::ExecutionControl<'_>,
    ) -> io::Result<()> {
        let mut offset = 0;
        while offset < bytes.len() {
            if let Some(outcome) = execution.outcome() {
                return Err(io::Error::new(
                    if outcome == super::ProcessOutcome::Cancelled {
                        io::ErrorKind::Interrupted
                    } else {
                        io::ErrorKind::TimedOut
                    },
                    "Windows helper control interrupted",
                ));
            }
            if self.child.try_wait().map_err(|_| rejected())?.is_some() {
                return Err(rejected());
            }
            let end = bytes.len().min(offset + 4096);
            match self.control.write(&bytes[offset..end]) {
                Ok(0) => std::thread::sleep(Duration::from_millis(10)),
                Ok(written) => offset += written,
                Err(_) => return Err(rejected()),
            }
        }
        Ok(())
    }
    pub(super) fn send_paired_frame(
        &mut self,
        frame: &super::windows_paired_guardian_protocol::Frame,
        execution: &super::ExecutionControl<'_>,
    ) -> io::Result<()> {
        if !self.config_sent || !self.config_bytes.is_empty() {
            return Err(rejected());
        }
        self.write_before(&frame.encode().map_err(|_| rejected())?, execution)
    }
    pub(super) fn send_managed_frame(
        &mut self,
        frame: &super::windows_managed_guardian_protocol::Frame,
        execution: &super::ExecutionControl<'_>,
    ) -> io::Result<()> {
        if !self.config_sent || !self.config_bytes.is_empty() {
            return Err(rejected());
        }
        self.write_before(&frame.encode().map_err(|_| rejected())?, execution)
    }
    pub(super) fn send_frame(
        &mut self,
        frame: &super::windows_guardian_protocol::Frame,
        execution: &super::ExecutionControl<'_>,
    ) -> io::Result<()> {
        if !self.config_sent || !self.config_bytes.is_empty() {
            return Err(rejected());
        }
        self.write_before(&frame.encode().map_err(|_| rejected())?, execution)
    }
}

#[cfg(test)]
mod tests {
    use super::super::windows_guardian_protocol::{FRAME_LEN, Frame, Kind};
    use super::*;
    use std::{io::Read, os::windows::ffi::OsStrExt, sync::atomic::AtomicBool, time::Instant};
    fn read_frame(file: &mut File, deadline: Instant) -> Frame {
        let mut bytes = [0; FRAME_LEN];
        let mut offset = 0;
        while offset < FRAME_LEN {
            assert!(Instant::now() < deadline, "helper frame deadline");
            let mut available = 0;
            // SAFETY: retained original read pipe and byte-count output.
            assert_ne!(
                unsafe {
                    windows_sys::Win32::System::Pipes::PeekNamedPipe(
                        file.as_raw_handle(),
                        ptr::null_mut(),
                        0,
                        ptr::null_mut(),
                        &mut available,
                        ptr::null_mut(),
                    )
                },
                0
            );
            if available == 0 {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            let count = (FRAME_LEN - offset).min(available as usize);
            offset += file.read(&mut bytes[offset..offset + count]).unwrap();
        }
        Frame::decode(&bytes).unwrap()
    }
    #[test]
    fn independent_helper_normal_exit_original_creation_and_actual_receipt() {
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(
            helper.is_file(),
            "build native process guard before this acceptance"
        );
        let mut fixture = crate::cookies::windows_test_fixture::TestFixture::create();
        let receipt = fixture.begin(0, [3; 16]);
        let config = GuardianConfig {
            runtime_environment: None,
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_guardian::tests::fixture_tool"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 3,
            timeout_nanos: 0,
            max_log_bytes: 1024,
        };
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(5);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        let mut launch = launch(&helper, &config, &receipt).unwrap();
        launch.transmit_config(&execution).unwrap();
        assert!(launch.transmit_config(&execution).is_err());
        let prepared = read_frame(&mut launch.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(
            prepared.guardian_created(),
            launch.child.creation_time().unwrap()
        );
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        launch
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launch.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        launch
            .send_frame(
                &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let terminal = read_frame(&mut launch.events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.code(), 0);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert!(launch.child.wait().unwrap().success());
        let mut output = Vec::new();
        (&mut launch.stdout)
            .take(4097)
            .read_to_end(&mut output)
            .unwrap();
        assert!(output.len() <= 4096);
        assert!(String::from_utf8(output).unwrap().contains("1 passed"));
        assert_eq!(fixture.receipt_bytes(0).len(), 256);
        fixture.reacquire();
        fixture.resolve(0);
        assert!(fixture.all_receipts_match());
    }

    // Fragment for windows_helper_launch::tests. No product API/schema changes.
    fn helper_failure_setup() -> (crate::cookies::windows_test_fixture::TestFixture, Launch) {
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(
            helper.is_file(),
            "build native process guard before this acceptance"
        );
        let mut fixture = crate::cookies::windows_test_fixture::TestFixture::create();
        let receipt = fixture.begin(0, [3; 16]);
        let config = GuardianConfig {
            runtime_environment: None,
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_guardian::tests::fixture_tool"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 3,
            timeout_nanos: 0,
            max_log_bytes: 1024,
        };
        let launched = launch(&helper, &config, &receipt).unwrap();
        (fixture, launched)
    }
    fn helper_failure_output(file: &mut File, deadline: Instant) -> Vec<u8> {
        let mut output = Vec::new();
        loop {
            assert!(Instant::now() < deadline, "helper output deadline");
            let mut available = 0;
            // SAFETY: original owned pipe reader and live byte-count output, no payload buffer.
            if unsafe {
                windows_sys::Win32::System::Pipes::PeekNamedPipe(
                    file.as_raw_handle(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut available,
                    ptr::null_mut(),
                )
            } == 0
            {
                assert_eq!(
                    io::Error::last_os_error().raw_os_error(),
                    Some(windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE as i32)
                );
                return output;
            }
            if available == 0 {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            assert!(
                output.len() + available as usize <= 4096,
                "helper output byte bound"
            );
            let mut bytes = vec![0; available as usize];
            let got = file.read(&mut bytes).unwrap();
            assert!(got > 0);
            output.extend_from_slice(&bytes[..got]);
        }
    }
    #[test]
    fn independent_helper_ack_then_control_eof_confirms_actual_receipt() {
        let (mut fixture, mut launched) = helper_failure_setup();
        let original = fixture.copybytes();
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(5);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        launched.transmit_config(&execution).unwrap();
        assert!(launched.config_sent);
        let prepared = read_frame(&mut launched.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(
            prepared.guardian_created(),
            launched.child.creation_time().unwrap()
        );
        assert_eq!(prepared.slot(), 0);
        assert_eq!(prepared.nonce(), [3; 16]);
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        launched
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launched.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        assert_eq!(ack.slot(), 0);
        assert_eq!(ack.nonce(), [3; 16]);
        // Retain original process and read endpoints; EOF affects only app control ownership.
        let Launch {
            mut child,
            control,
            mut events,
            mut stdout,
            mut stderr,
            ..
        } = launched;
        drop(control);
        let terminal = read_frame(&mut events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_ne!(terminal.code(), 0);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert_eq!(terminal.slot(), 0);
        assert_eq!(terminal.nonce(), [3; 16]);
        let status = child.wait().unwrap();
        assert!(!status.success());
        assert_ne!(status.code(), Some(0));
        assert!(helper_failure_output(&mut stdout, deadline).is_empty());
        let _bounded_stderr = helper_failure_output(&mut stderr, deadline);
        assert!(helper_failure_output(&mut events, deadline).is_empty());
        assert_eq!(fixture.receipt_bytes(0).len(), 256);
        assert!(fixture.receipt_bytes(1).is_empty());
        fixture.reacquire();
        fixture.resolve(0);
        assert!(fixture.all_receipts_match());
        assert_eq!(fixture.copybytes(), original);
    }
    fn independent_helper_config_interrupted_before_any_send(cancelled: bool) {
        let (fixture, mut launched) = helper_failure_setup();
        let original = fixture.copybytes();
        let before = fixture.record().encode().unwrap();
        let cancel = AtomicBool::new(cancelled);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline: if cancelled {
                Instant::now() + Duration::from_secs(3)
            } else {
                Instant::now()
            },
        };
        let error = launched.transmit_config(&execution).unwrap_err();
        assert_eq!(
            error.kind(),
            if cancelled {
                io::ErrorKind::Interrupted
            } else {
                io::ErrorKind::TimedOut
            }
        );
        assert!(!launched.config_sent);
        assert!(launched.config_attempted);
        let begin = Frame::new(Kind::Begin, 0, [3; 16], [9; 32], (0, 0), 0).unwrap();
        assert!(launched.send_frame(&begin, &execution).is_err());
        assert!(!launched.config_sent);
        let Launch {
            mut child,
            control,
            mut events,
            mut stdout,
            mut stderr,
            ..
        } = launched;
        drop(control);
        let status = child.wait().unwrap();
        assert_eq!(status.code(), Some(126));
        let deadline = Instant::now() + Duration::from_secs(3);
        assert!(helper_failure_output(&mut stdout, deadline).is_empty());
        let _bounded_stderr = helper_failure_output(&mut stderr, deadline);
        assert!(helper_failure_output(&mut events, deadline).is_empty());
        assert!(fixture.receipt_bytes(0).is_empty());
        assert!(fixture.receipt_bytes(1).is_empty());
        assert_eq!(fixture.record().encode().unwrap(), before);
        assert!(!fixture.all_receipts_match());
        assert_eq!(fixture.copybytes(), original);
    }
    #[test]
    fn independent_helper_config_pre_cancel_preserves_intent_no_tool() {
        independent_helper_config_interrupted_before_any_send(true)
    }
    #[test]
    fn independent_helper_config_expired_deadline_preserves_intent_no_tool() {
        independent_helper_config_interrupted_before_any_send(false)
    }

    // Fragment for windows_helper_launch::tests; uses existing read_frame and fixture APIs.
    #[test]
    fn independent_helper_large_config_streams_through_original_pipe_and_receipt() {
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(
            helper.is_file(),
            "build native process guard before this acceptance"
        );
        let mut fixture = crate::cookies::windows_test_fixture::TestFixture::create();
        let receipt = fixture.begin(0, [3; 16]);
        let config = GuardianConfig {
            runtime_environment: None,
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_guardian::tests::fixture_tool"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
                "--skip".encode_utf16().collect(),
                format!("synthetic-never-matching-filter-{}", "x".repeat(8192))
                    .encode_utf16()
                    .collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 8,
            timeout_nanos: 0,
            max_log_bytes: 1024,
        };
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        let mut launch = launch(&helper, &config, &receipt).unwrap();
        // Actual JSON includes u16 arrays, so this exercises multiple 4 KiB writes.
        let wire_bytes = launch.config_bytes.len();
        assert!(wire_bytes > 32 * 1024);
        assert!(wire_bytes <= MAX_CONFIG_BYTES + 4);
        let started = Instant::now();
        launch.transmit_config(&execution).unwrap();
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(launch.config_bytes.is_empty());
        assert!(launch.config_sent);
        assert!(launch.transmit_config(&execution).is_err());
        let prepared = read_frame(&mut launch.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(
            prepared.guardian_created(),
            launch.child.creation_time().unwrap()
        );
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        launch
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launch.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        launch
            .send_frame(
                &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let terminal = read_frame(&mut launch.events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.code(), 0);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert!(launch.child.wait().unwrap().success());
        let mut output = Vec::new();
        (&mut launch.stdout)
            .take(4097)
            .read_to_end(&mut output)
            .unwrap();
        assert!(output.len() <= 4096);
        assert!(String::from_utf8(output).unwrap().contains("1 passed"));
        assert_eq!(fixture.receipt_bytes(0).len(), 256);
        fixture.reacquire();
        fixture.resolve(0);
        assert!(fixture.all_receipts_match());
    }
    #[test]
    fn independent_helper_actual_receipt_drives_public_product_recovery() {
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(
            helper.is_file(),
            "build native process guard before this acceptance"
        );
        let mut fixture = crate::cookies::windows_test_fixture::ProductFixture::create();
        let receipt = fixture.begin(0, [3; 16]);
        let config = GuardianConfig {
            runtime_environment: None,
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_guardian::tests::fixture_tool"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 3,
            timeout_nanos: 0,
            max_log_bytes: 1024,
        };
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(5);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        let mut launch = launch(&helper, &config, &receipt).unwrap();
        // Parent retains no receipt writer after capability transfer.
        drop(receipt);
        launch.transmit_config(&execution).unwrap();
        assert!(launch.transmit_config(&execution).is_err());
        let prepared = read_frame(&mut launch.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(
            prepared.guardian_created(),
            launch.child.creation_time().unwrap()
        );
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        launch
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launch.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        launch
            .send_frame(
                &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let terminal = read_frame(&mut launch.events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.code(), 0);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert!(launch.child.wait().unwrap().success());
        let mut output = Vec::new();
        (&mut launch.stdout)
            .take(4097)
            .read_to_end(&mut output)
            .unwrap();
        assert!(output.len() <= 4096);
        assert!(String::from_utf8(output).unwrap().contains("1 passed"));
        assert_eq!(fixture.receipt_bytes(0).len(), 256);
        fixture.defer_and_recover();
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetStdHandle"]
        fn app_kill_get_std_handle(which: u32) -> windows_sys::Win32::Foundation::HANDLE;
    }
    const APP_KILL_BASE_ENV: &str = "DLPAD_APP_KILL_FIXTURE_BASE";
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct AppKillReady {
        producer_creation: u64,
        observer_creation: u64,
        helper_creation: u64,
        // Target-process handles minted from original producer-owned capabilities.
        helper: u64,
        events: u64,
        stdout: u64,
        stderr: u64,
        bound_record: crate::cookies::windows_records::WindowsReaderRecord,
    }
    fn app_kill_creation(handle: windows_sys::Win32::Foundation::HANDLE) -> u64 {
        let mut times = [windows_sys::Win32::Foundation::FILETIME::default(); 4];
        // SAFETY: original process capability and live four FILETIME outputs.
        assert_ne!(
            unsafe {
                windows_sys::Win32::System::Threading::GetProcessTimes(
                    handle,
                    &mut times[0],
                    &mut times[1],
                    &mut times[2],
                    &mut times[3],
                )
            },
            0
        );
        let value = (u64::from(times[0].dwHighDateTime) << 32) | u64::from(times[0].dwLowDateTime);
        assert_ne!(value, 0);
        value
    }
    fn app_kill_publish(base: &Path, name: &str, bytes: &[u8]) {
        assert!(bytes.len() <= 8192);
        let pending = base.join(format!("{name}.pending"));
        let final_path = base.join(name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
            .unwrap();
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
        drop(file);
        assert!(!final_path.exists());
        std::fs::rename(pending, final_path).unwrap();
    }
    fn app_kill_file_before(path: &Path, deadline: Instant) -> Vec<u8> {
        loop {
            assert!(Instant::now() < deadline, "owned app-kill marker deadline");
            match File::open(path) {
                Ok(file) => {
                    let mut bytes = Vec::new();
                    file.take(8193).read_to_end(&mut bytes).unwrap();
                    assert!(bytes.len() <= 8192);
                    return bytes;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(_) => panic!("owned app-kill marker read"),
            }
        }
    }
    fn app_kill_duplicate_file(file: &File, target: windows_sys::Win32::Foundation::HANDLE) -> u64 {
        let mut duplicate = ptr::null_mut();
        // SAFETY: original source File and exact observer process capability, returned handle belongs to observer.
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    file.as_raw_handle(),
                    target,
                    &mut duplicate,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        assert!(!duplicate.is_null());
        duplicate as usize as u64
    }
    // Roll back only exact target handles created by this producer if READY is not published.
    struct AppKillTransfers {
        target: windows_sys::Win32::Foundation::HANDLE,
        values: Vec<u64>,
        published: bool,
    }
    impl AppKillTransfers {
        fn remember(&mut self, value: u64) -> u64 {
            self.values.push(value);
            value
        }
        fn file(&mut self, file: &File) -> u64 {
            let value = app_kill_duplicate_file(file, self.target);
            self.remember(value)
        }
        fn publish(&mut self, base: &Path, ready: &AppKillReady) {
            let bytes = serde_json::to_vec(ready).unwrap();
            assert!(bytes.len() <= 8192);
            let pending = base.join("producer-ready.json.pending");
            let path = base.join("producer-ready.json");
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pending)
                .unwrap();
            file.write_all(&bytes).unwrap();
            file.sync_all().unwrap();
            drop(file);
            assert!(!path.exists());
            self.published = true;
            if std::fs::rename(&pending, &path).is_err() {
                self.published = false;
                panic!("owned READY publication failed")
            }
        }
    }
    impl Drop for AppKillTransfers {
        fn drop(&mut self) {
            if self.published {
                return;
            }
            for value in &self.values {
                let mut returned = ptr::null_mut();
                // SAFETY: close only original target duplicates just minted by this guard, not foreign handles.
                if unsafe {
                    DuplicateHandle(
                        self.target,
                        *value as usize as _,
                        GetCurrentProcess(),
                        &mut returned,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS
                            | windows_sys::Win32::Foundation::DUPLICATE_CLOSE_SOURCE,
                    )
                } != 0
                {
                    drop(unsafe { OwnedHandle::from_raw_handle(returned) });
                }
            }
        }
    }
    #[test]
    fn app_kill_running_tool() {
        let Some(base) = std::env::var_os(APP_KILL_BASE_ENV) else {
            return;
        };
        let base = std::path::PathBuf::from(base);
        let expected =
            b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-only\n";
        assert_eq!(
            std::fs::read(
                base.join("private-cache")
                    .join("synthetic-owner")
                    .join("cookies.txt")
            )
            .unwrap(),
            expected
        );
        println!("DLPAD_SYNTHETIC_TOOL_STARTED");
        std::io::stdout().flush().unwrap();
        app_kill_publish(&base, "tool-started", b"R");
        std::thread::sleep(Duration::from_secs(10));
    }
    #[test]
    fn app_kill_producer() {
        let Some(base) = std::env::var_os(APP_KILL_BASE_ENV) else {
            return;
        };
        let base = std::path::PathBuf::from(base);
        // Exact inherited observer process capability in stdin, not a PID lookup.
        let observer = unsafe { app_kill_get_std_handle((-10i32) as u32) };
        assert!(!observer.is_null() && observer as isize != -1);
        let expected_observer = app_kill_file_before(
            &base.join("observer-creation"),
            Instant::now() + Duration::from_secs(2),
        );
        assert_eq!(expected_observer.len(), 8);
        let observer_creation = u64::from_le_bytes(expected_observer.try_into().unwrap());
        assert_eq!(app_kill_creation(observer), observer_creation);
        let mut fixture = crate::cookies::windows_test_fixture::TestFixture::create_in(&base);
        let receipt = fixture.begin(0, [3; 16]);
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(helper.is_file());
        let config = GuardianConfig {
            runtime_environment: None,
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_helper_launch::tests::app_kill_running_tool"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 5,
            timeout_nanos: 0,
            max_log_bytes: 1024,
        };
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(4);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        let mut launched = launch(&helper, &config, &receipt).unwrap();
        launched.transmit_config(&execution).unwrap();
        let prepared = read_frame(&mut launched.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(prepared.slot(), 0);
        assert_eq!(prepared.nonce(), [3; 16]);
        assert_eq!(
            prepared.guardian_created(),
            launched.child.creation_time().unwrap()
        );
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        let bound_record = fixture.record();
        fixture.release(0, bound);
        launched
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launched.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.slot(), 0);
        assert_eq!(ack.nonce(), [3; 16]);
        assert_eq!(ack.digest(), bound.identity_digest);
        launched
            .send_frame(
                &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        assert_eq!(
            app_kill_file_before(&base.join("tool-started"), deadline),
            b"R"
        );
        let mut transfers = AppKillTransfers {
            target: observer,
            values: Vec::with_capacity(4),
            published: false,
        };
        let helper_cap =
            transfers.remember(launched.child.duplicate_to_process(observer).unwrap() as u64);
        let event_cap = transfers.file(&launched.events);
        let output_cap = transfers.file(&launched.stdout);
        let error_cap = transfers.file(&launched.stderr);
        let ready = AppKillReady {
            producer_creation: app_kill_creation(unsafe { GetCurrentProcess() }),
            observer_creation,
            helper_creation: prepared.guardian_created(),
            helper: helper_cap,
            events: event_cap,
            stdout: output_cap,
            stderr: error_cap,
            bound_record,
        };
        transfers.publish(&base, &ready);
        // Stay alive, retaining control writer/Launch/fixture until parent terminates this original process.
        let stop = Instant::now() + Duration::from_secs(10);
        while Instant::now() < stop {
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("observer did not terminate producer");
    }
    struct AppKillHelper {
        process: OwnedHandle,
    }
    impl Drop for AppKillHelper {
        fn drop(&mut self) {
            let handle = self.process.as_raw_handle();
            // SAFETY: original duplicated process handle; cleanup never uses a PID.
            unsafe {
                if windows_sys::Win32::System::Threading::WaitForSingleObject(handle, 0)
                    == windows_sys::Win32::Foundation::WAIT_TIMEOUT
                {
                    windows_sys::Win32::System::Threading::TerminateProcess(handle, 126);
                    windows_sys::Win32::System::Threading::WaitForSingleObject(handle, 2000);
                }
            }
        }
    }
    #[test]
    fn actual_app_like_process_kill_leaves_helper_owned_cleanup_receipt() {
        use std::os::windows::io::FromRawHandle;
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();
        app_kill_publish(
            base,
            "foreign-sentinel",
            b"preserve foreign fixture sentinel",
        );
        let observer_creation = app_kill_creation(unsafe { GetCurrentProcess() });
        app_kill_publish(base, "observer-creation", &observer_creation.to_le_bytes());
        let mut observer_raw = ptr::null_mut();
        // SAFETY: narrowed inheritable current-process capability, never external process lookup.
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    GetCurrentProcess(),
                    GetCurrentProcess(),
                    &mut observer_raw,
                    windows_sys::Win32::System::Threading::PROCESS_DUP_HANDLE
                        | windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION,
                    1,
                    0,
                )
            },
            0
        );
        let observer_cap = unsafe { OwnedHandle::from_raw_handle(observer_raw) };
        let current = std::env::current_exe().unwrap();
        let args = vec![
            OsString::from("--exact"),
            OsString::from("process::windows_helper_launch::tests::app_kill_producer"),
            OsString::from("--nocapture"),
        ];
        let mut producer =
            windows::spawn_test_producer(&current, &args, observer_cap.as_raw_handle(), base)
                .unwrap();
        let original_producer_creation = producer.creation_time().unwrap();
        let ready_bytes = app_kill_file_before(
            &base.join("producer-ready.json"),
            Instant::now() + Duration::from_secs(4),
        );
        let ready: AppKillReady = serde_json::from_slice(&ready_bytes).unwrap();
        assert_eq!(ready.producer_creation, original_producer_creation);
        assert_eq!(ready.observer_creation, observer_creation);
        for handle in [ready.helper, ready.events, ready.stdout, ready.stderr] {
            assert!(handle != 0 && handle as isize != -1);
        }
        let unique: std::collections::BTreeSet<_> =
            [ready.helper, ready.events, ready.stdout, ready.stderr]
                .into_iter()
                .collect();
        assert_eq!(unique.len(), 4);
        // All target handle values are from this producer's fresh private manifest and direct native duplication.
        let helper = AppKillHelper {
            process: unsafe { OwnedHandle::from_raw_handle(ready.helper as usize as _) },
        };
        let mut events = unsafe { File::from_raw_handle(ready.events as usize as _) };
        let mut output = unsafe { File::from_raw_handle(ready.stdout as usize as _) };
        let mut error = unsafe { File::from_raw_handle(ready.stderr as usize as _) };
        for file in [&events, &output, &error] {
            assert_eq!(
                unsafe {
                    windows_sys::Win32::Storage::FileSystem::GetFileType(file.as_raw_handle())
                },
                windows_sys::Win32::Storage::FileSystem::FILE_TYPE_PIPE
            );
        }
        assert_eq!(
            app_kill_creation(helper.process.as_raw_handle()),
            ready.helper_creation
        );
        let expected = crate::cookies::windows_records::WindowsReaderRecord::parse(
            &ready.bound_record.encode().unwrap(),
        )
        .unwrap();
        let bound = expected.pending_bound_identity(0).unwrap();
        assert_eq!(bound.guardian_created, ready.helper_creation);
        let observer =
            crate::cookies::windows_test_fixture::AppKillObserver::pin_original(base, expected);
        assert!(observer.receipt_empty());
        assert_eq!(
            app_kill_file_before(
                &base.join("tool-started"),
                Instant::now() + Duration::from_secs(1)
            ),
            b"R"
        );
        let mut available = 0;
        assert_ne!(
            unsafe {
                windows_sys::Win32::System::Pipes::PeekNamedPipe(
                    events.as_raw_handle(),
                    ptr::null_mut(),
                    0,
                    ptr::null_mut(),
                    &mut available,
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(available, 0, "helper already terminal before app kill");
        assert_eq!(
            unsafe {
                windows_sys::Win32::System::Threading::WaitForSingleObject(
                    helper.process.as_raw_handle(),
                    0,
                )
            },
            windows_sys::Win32::Foundation::WAIT_TIMEOUT
        );
        assert!(producer.try_wait().unwrap().is_none());
        producer.terminate_owned_for_test().unwrap();
        assert!(!producer.wait().unwrap().success());
        let deadline = Instant::now() + Duration::from_secs(5);
        let terminal = read_frame(&mut events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.slot(), 0);
        assert_eq!(terminal.nonce(), [3; 16]);
        assert_ne!(terminal.code(), 0);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert_eq!(
            unsafe {
                windows_sys::Win32::System::Threading::WaitForSingleObject(
                    helper.process.as_raw_handle(),
                    2000,
                )
            },
            windows_sys::Win32::Foundation::WAIT_OBJECT_0
        );
        let mut helper_code = 0;
        assert_ne!(
            unsafe {
                windows_sys::Win32::System::Threading::GetExitCodeProcess(
                    helper.process.as_raw_handle(),
                    &mut helper_code,
                )
            },
            0
        );
        assert_eq!(helper_code, terminal.code());
        let bytes = helper_failure_output(&mut output, deadline);
        assert!(String::from_utf8_lossy(&bytes).contains("DLPAD_SYNTHETIC_TOOL_STARTED"));
        let _bounded_error = helper_failure_output(&mut error, deadline);
        assert!(helper_failure_output(&mut events, deadline).is_empty());
        observer.recover_original();
        assert_eq!(
            std::fs::read(base.join("foreign-sentinel")).unwrap(),
            b"preserve foreign fixture sentinel"
        );
    }
    #[test]
    fn actual_cache_handoff_helper_lock_survives_original_app_handles_close() {
        use super::super::windows_runtime_environment_config::RuntimeEnvironmentConfig;
        use crate::windows_owned_deno_cache::OwnedDenoCache;
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        let temp = tempfile::Builder::new()
            .prefix("dlpad-runtime-cache-helper-")
            .tempdir()
            .unwrap();
        let root = std::fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(temp.path())
            .unwrap();
        let mut cache = OwnedDenoCache::create_or_reopen(temp.path(), &root).unwrap();
        let owner_bytes = std::fs::read(cache.path().join("owner.json")).unwrap();
        let cache_path = cache.path().to_owned();
        let lock_path = cache_path.join("cache.lock");
        let binding = cache.binding().unwrap();
        let mut fixture = crate::cookies::windows_test_fixture::TestFixture::create();
        let receipt = fixture.begin(0, [3; 16]);
        let current = std::env::current_exe().unwrap();
        let helper = current
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("dlpad-process-guard.exe");
        assert!(helper.is_file());
        let config = GuardianConfig {
            runtime_environment: Some(
                RuntimeEnvironmentConfig::new(cache_path.as_os_str().encode_wide().collect(), 1)
                    .unwrap()
                    .with_binding(binding)
                    .unwrap(),
            ),
            version: 1,
            executable: current.as_os_str().encode_wide().collect(),
            args: vec![
                "--exact".encode_utf16().collect(),
                "process::windows_guardian::tests::owned_cache_environment_fixture"
                    .encode_utf16()
                    .collect(),
                "--nocapture".encode_utf16().collect(),
            ],
            owner_path: fixture.owner_path().as_os_str().encode_wide().collect(),
            intent: fixture.record(),
            slot: 0,
            nonce: [3; 16],
            timeout_secs: 10,
            timeout_nanos: 0,
            max_log_bytes: 4096,
        };
        assert!(
            launch(&helper, &config, &receipt).is_err(),
            "binding without original capability must be rejected before spawn"
        );
        let mut launch =
            launch_with_cache(&helper, &config, &receipt, Some(cache.lock_file().unwrap()))
                .unwrap();
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        let execution = super::super::ExecutionControl {
            cancel: &cancel,
            deadline,
        };
        launch.transmit_config(&execution).unwrap();
        let prepared = read_frame(&mut launch.events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(
            prepared.guardian_created(),
            launch.child.creation_time().unwrap()
        );
        let observer = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&lock_path)
            .unwrap();
        assert!(observer.try_lock().is_err());
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        cache.release_for_guardian().unwrap();
        launch
            .send_frame(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let ack = read_frame(&mut launch.events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        drop(cache);
        drop(root);
        assert!(
            observer.try_lock().is_err(),
            "independent helper must retain its own lock after original app handles close"
        );
        launch
            .send_frame(
                &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0).unwrap(),
                &execution,
            )
            .unwrap();
        let terminal = read_frame(&mut launch.events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert_eq!(terminal.code(), 0);
        assert!(launch.child.wait().unwrap().success());
        observer.try_lock().unwrap();
        observer.unlock().unwrap();
        let mut output = Vec::new();
        (&mut launch.stdout)
            .take(4097)
            .read_to_end(&mut output)
            .unwrap();
        assert!(output.len() <= 4096);
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("DLPAD_CACHE_HELPER_ENV_OK")
        );
        assert_eq!(
            std::fs::read(cache_path.join("owner.json")).unwrap(),
            owner_bytes
        );
        fixture.reacquire();
        fixture.resolve(0);
        assert!(fixture.all_receipts_match());
    }
}
