// Included inside windows_guardian::tests. Native runtime only; not a process entry test.
fn native_fixture_pipe() -> (std::fs::File, std::fs::File) {
    use std::os::windows::io::FromRawHandle;
    let mut read = std::ptr::null_mut();
    let mut write = std::ptr::null_mut();
    // SAFETY: new non-inheritable 4096-byte anonymous pipe handles, transferred once.
    unsafe {
        assert_ne!(
            windows_sys::Win32::System::Pipes::CreatePipe(
                &mut read,
                &mut write,
                std::ptr::null(),
                4096
            ),
            0
        );
        (
            std::fs::File::from_raw_handle(read),
            std::fs::File::from_raw_handle(write),
        )
    }
}
fn native_fixture_read(
    file: &mut std::fs::File,
    amount: usize,
    deadline: std::time::Instant,
) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    use std::os::windows::io::AsRawHandle;
    let mut bytes = Vec::with_capacity(amount);
    while bytes.len() < amount {
        if std::time::Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "fixture pipe deadline",
            ));
        }
        let mut available = 0;
        // SAFETY: owned read pipe and valid count output; no payload buffer.
        if unsafe {
            windows_sys::Win32::System::Pipes::PeekNamedPipe(
                file.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error());
        }
        if available == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        let count = (available as usize).min(amount - bytes.len());
        let mut part = vec![0; count];
        let got = file.read(&mut part)?;
        if got == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "fixture pipe EOF",
            ));
        }
        bytes.extend_from_slice(&part[..got]);
    }
    Ok(bytes)
}
fn native_fixture_frame(file: &mut std::fs::File, deadline: std::time::Instant) -> Frame {
    Frame::decode(&native_fixture_read(file, FRAME_LEN, deadline).unwrap()).unwrap()
}
fn native_fixture_output(file: &mut std::fs::File, deadline: std::time::Instant) -> Vec<u8> {
    use std::io::Read;
    use std::os::windows::io::AsRawHandle;
    let mut output = Vec::new();
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "fixture output deadline"
        );
        let mut available = 0;
        let ok = unsafe {
            windows_sys::Win32::System::Pipes::PeekNamedPipe(
                file.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE as i32)
            );
            return output;
        }
        if available == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        assert!(
            output.len() + available as usize <= 4096,
            "fixture output bound"
        );
        let mut bytes = vec![0; available as usize];
        let got = file.read(&mut bytes).unwrap();
        assert!(got > 0);
        output.extend_from_slice(&bytes[..got]);
    }
}
#[derive(Clone, Copy)]
enum RuntimeFixtureCase {
    Normal,
    Nonzero,
    EofBeforeBegin,
    QueuedCancel,
    WrongBeginDigest,
}
#[test]
fn fixture_tool() {}
fn native_runtime_fixture(case: RuntimeFixtureCase) {
    use crate::cookies::windows_test_fixture::TestFixture;
    use std::io::Write;
    let mut fixture = TestFixture::create();
    let original = fixture.copybytes();
    let receipt = fixture.begin(0, [3; 16]);
    let intent = fixture.record();
    let owner = fixture.owner_path().to_owned();
    let (control, control_writer) = native_fixture_pipe();
    let (mut events, events_writer) = native_fixture_pipe();
    let (mut stdout, stdout_writer) = native_fixture_pipe();
    let (mut stderr, stderr_writer) = native_fixture_pipe();
    let args = if matches!(case, RuntimeFixtureCase::Nonzero) {
        vec!["--dlpad-intentionally-unknown-fixture-option".into()]
    } else {
        vec![
            "--exact".into(),
            "process::windows_guardian::tests::fixture_tool".into(),
            "--nocapture".into(),
        ]
    };
    let spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: None,
        executable: std::env::current_exe().unwrap(),
        args,
        timeout: std::time::Duration::from_secs(3),
        max_log_bytes: 4096,
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let result = std::thread::scope(|scope| {
        let task = scope.spawn(|| {
            run(Request {
                cache: None,
                spec: &spec,
                intent: &intent,
                slot: 0,
                nonce: [3; 16],
                owner_path: &owner,
                receipt,
                control,
                events: events_writer,
                stdout: stdout_writer,
                stderr: stderr_writer,
            })
        });
        // Drop on parent failure closes its only control writer before scoped join.
        let mut writer = Some(control_writer);
        let prepared = native_fixture_frame(&mut events, deadline);
        assert_eq!(prepared.kind(), Kind::Prepared);
        assert_eq!(prepared.slot(), 0);
        assert_eq!(prepared.nonce(), [3; 16]);
        let bound = fixture.bind(0, prepared.guardian_created(), prepared.tool_created());
        fixture.release(0, bound);
        writer
            .as_mut()
            .unwrap()
            .write_all(
                &Frame::new(Kind::Ack, 0, [3; 16], bound.identity_digest, (0, 0), 0)
                    .unwrap()
                    .encode()
                    .unwrap(),
            )
            .unwrap();
        let ack = native_fixture_frame(&mut events, deadline);
        assert_eq!(ack.kind(), Kind::Ack);
        assert_eq!(ack.digest(), bound.identity_digest);
        assert_eq!(ack.nonce(), [3; 16]);
        assert_eq!(ack.slot(), 0);
        match case {
            RuntimeFixtureCase::EofBeforeBegin => {
                drop(writer.take());
            }
            RuntimeFixtureCase::QueuedCancel => {
                let mut queued = Vec::with_capacity(2 * FRAME_LEN);
                queued.extend(
                    Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0)
                        .unwrap()
                        .encode()
                        .unwrap(),
                );
                queued.extend(
                    Frame::new(Kind::Cancel, 0, [3; 16], [0; 32], (0, 0), 0)
                        .unwrap()
                        .encode()
                        .unwrap(),
                );
                assert_eq!(queued.len(), 256);
                assert_eq!(writer.as_mut().unwrap().write(&queued).unwrap(), 256);
            }
            RuntimeFixtureCase::WrongBeginDigest => writer
                .as_mut()
                .unwrap()
                .write_all(
                    &Frame::new(Kind::Begin, 0, [3; 16], [9; 32], (0, 0), 0)
                        .unwrap()
                        .encode()
                        .unwrap(),
                )
                .unwrap(),
            _ => writer
                .as_mut()
                .unwrap()
                .write_all(
                    &Frame::new(Kind::Begin, 0, [3; 16], bound.identity_digest, (0, 0), 0)
                        .unwrap()
                        .encode()
                        .unwrap(),
                )
                .unwrap(),
        }
        let terminal = native_fixture_frame(&mut events, deadline);
        assert_eq!(terminal.kind(), Kind::Terminal);
        assert_eq!(terminal.digest(), bound.identity_digest);
        assert_eq!(terminal.nonce(), [3; 16]);
        assert_eq!(terminal.slot(), 0);
        let result = task.join().unwrap();
        drop(writer);
        match case {
            RuntimeFixtureCase::Normal => {
                assert_eq!(result.as_ref().unwrap(), &0);
                assert_eq!(terminal.code(), 0);
            }
            RuntimeFixtureCase::Nonzero => {
                assert_ne!(*result.as_ref().unwrap(), 0);
                assert_eq!(*result.as_ref().unwrap(), terminal.code());
            }
            _ => {
                assert!(result.is_err());
                assert_ne!(terminal.code(), 0);
            }
        }
        result
    });
    assert!(native_fixture_output(&mut events, deadline).is_empty());
    let output = native_fixture_output(&mut stdout, deadline);
    let error = native_fixture_output(&mut stderr, deadline);
    match case {
        RuntimeFixtureCase::Normal => {
            assert!(String::from_utf8_lossy(&output).contains("1 passed"))
        }
        RuntimeFixtureCase::Nonzero => assert!(!error.is_empty() || !output.is_empty()),
        _ => {
            assert!(output.is_empty(), "pre-Begin tool output was observed");
            assert!(error.is_empty(), "pre-Begin tool stderr was observed");
            assert!(result.is_err());
        }
    }
    assert_eq!(fixture.receipt_bytes(0).len(), 256);
    assert!(fixture.receipt_bytes(1).is_empty());
    fixture.reacquire();
    fixture.resolve(0);
    assert!(fixture.all_receipts_match());
    assert_eq!(fixture.copybytes(), original);
}
#[test]
fn native_runtime_normal_exit_and_actual_receipt() {
    native_runtime_fixture(RuntimeFixtureCase::Normal)
}
#[test]
fn native_runtime_genuine_nonzero_exit_and_actual_receipt() {
    native_runtime_fixture(RuntimeFixtureCase::Nonzero)
}
#[test]
fn native_runtime_helper_ack_then_control_eof_preserves_no_tool_output() {
    native_runtime_fixture(RuntimeFixtureCase::EofBeforeBegin)
}
#[test]
fn native_runtime_queued_begin_cancel_single_write_prevents_resume() {
    native_runtime_fixture(RuntimeFixtureCase::QueuedCancel)
}
#[test]
fn native_runtime_wrong_begin_digest_prevents_resume() {
    native_runtime_fixture(RuntimeFixtureCase::WrongBeginDigest)
}

#[test]
fn owned_cache_environment_fixture() {
    let Some(value) = std::env::var_os("DENO_DIR") else {
        return;
    };
    let path = std::path::PathBuf::from(value);
    if path.file_name().is_none_or(|name| name != "deno")
        || path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .is_none_or(|name| {
                !name
                    .to_string_lossy()
                    .starts_with("dlpad-runtime-cache-helper-")
            })
    {
        return;
    }
    assert_eq!(
        std::env::var_os("DENO_DISABLE_NODE_SHIM").as_deref(),
        Some(std::ffi::OsStr::new("1"))
    );
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join("cache.lock"))
        .unwrap();
    assert!(
        lock.try_lock().is_err(),
        "tool must observe helper-owned cache lock"
    );
    assert!(path.join("owner.json").is_file());
    println!("DLPAD_CACHE_HELPER_ENV_OK");
}
