//! Actual native Windows child/grandchild tests, run under the private CLI host.
#![cfg(windows)]
use dlpad_engine::process::{self, ProcessOutcome, ProcessSpec};
use std::{
    fs, io,
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::WAIT_OBJECT_0,
    System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
};

static TEST_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn fixture(test: &str, root: &std::path::Path, case: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env("DLPAD_WINDOWS_FIXTURE_ROOT", root)
        .env("DLPAD_WINDOWS_FIXTURE_CASE", case)
        .creation_flags(0x08000000)
        .stdin(Stdio::null());
    command
}
fn root() -> Option<std::path::PathBuf> {
    std::env::var_os("DLPAD_WINDOWS_FIXTURE_ROOT").map(Into::into)
}
fn await_file(path: &std::path::Path) {
    let started = Instant::now();
    while !path.exists() {
        assert!(
            started.elapsed() < Duration::from_secs(8),
            "fixture gate absent: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn leaf_entry() {
    if root().is_none() {
        return;
    }
    thread::sleep(Duration::from_secs(60));
}
#[test]
fn root_entry() {
    let Some(root) = root() else {
        return;
    };
    let mut leaf = fixture("leaf_entry", &root, "leaf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // No delay before spawning: this used to be able to escape assignment.
    fs::write(
        root.join("pids.tmp"),
        format!("{} {}", std::process::id(), leaf.id()),
    )
    .unwrap();
    fs::rename(root.join("pids.tmp"), root.join("pids")).unwrap();
    await_file(&root.join("release"));
    let case = std::env::var("DLPAD_WINDOWS_FIXTURE_CASE").unwrap();
    if case == "normal" {
        // Deliberately leave the leaf alive so the product Job must clean it up.
        std::process::exit(0);
    }
    let _ = leaf.wait();
}
#[test]
fn worker_entry() {
    let Some(root) = root() else {
        return;
    };
    let case = std::env::var("DLPAD_WINDOWS_FIXTURE_CASE").unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    if case == "cancel" {
        let flag = cancel.clone();
        let gate = root.join("cancel");
        thread::spawn(move || {
            await_file(&gate);
            flag.store(true, Ordering::Release);
        });
    }
    let spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: None,
        executable: std::env::current_exe().unwrap(),
        args: ["--exact", "root_entry", "--nocapture", "--test-threads=1"]
            .into_iter()
            .map(Into::into)
            .collect(),
        timeout: Duration::from_secs(3),
        max_log_bytes: 8192,
    };
    let result = process::run(&spec, &cancel).unwrap();
    let expected = match case.as_str() {
        "normal" => ProcessOutcome::Exited,
        "cancel" => ProcessOutcome::Cancelled,
        "timeout" => ProcessOutcome::TimedOut,
        "crash" => panic!("crash worker should be killed externally"),
        _ => panic!("unknown fixture mode"),
    };
    assert_eq!(result.outcome, expected);
}
struct OwnedFixture(Child);
impl Drop for OwnedFixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn observed_process(id: u32) -> io::Result<OwnedHandle> {
    // Only observe IDs written by our synthetic fixture; never terminate by PID.
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, id) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: fresh process handle becomes owned and pins identity during observation.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}
#[test]
fn immediate_descendants_normal_cancel_timeout_and_parent_crash() {
    if root().is_some() {
        return;
    }
    let _gate = TEST_GATE.lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let mut outsider = OwnedFixture(
        fixture("leaf_entry", temp.path(), "outsider")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    for case in ["normal", "cancel", "timeout", "crash"] {
        let directory = temp.path().join(case);
        fs::create_dir(&directory).unwrap();
        let mut worker = OwnedFixture(
            fixture("worker_entry", &directory, case)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        await_file(&directory.join("pids"));
        let ids: Vec<u32> = fs::read_to_string(directory.join("pids"))
            .unwrap()
            .split_whitespace()
            .map(|id| id.parse().unwrap())
            .collect();
        assert_eq!(ids.len(), 2);
        let handles: Vec<_> = ids
            .into_iter()
            .map(|id| observed_process(id).unwrap())
            .collect();
        fs::write(directory.join("release"), b"go").unwrap();
        match case {
            "cancel" => fs::write(directory.join("cancel"), b"cancel").unwrap(),
            "crash" => worker.0.kill().unwrap(),
            _ => {}
        }
        let status = worker.0.wait().unwrap();
        if case != "crash" {
            assert!(status.success(), "{case}: worker failed: {status}");
        }
        for handle in handles {
            // SAFETY: handles were pinned before termination; no PID reuse inference.
            assert_eq!(
                unsafe { WaitForSingleObject(handle.as_raw_handle(), 3000) },
                WAIT_OBJECT_0,
                "{case}: owned descendant survived"
            );
        }
        assert!(
            outsider.0.try_wait().unwrap().is_none(),
            "{case}: unrelated fixture was terminated"
        );
    }
}

#[test]
fn argv_entry() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(2).is_none_or(|arg| arg != "argv_entry") {
        return;
    }
    println!("argv-evidence={}", serde_json::to_string(&args).unwrap());
}
#[test]
fn literal_unicode_argv_and_failed_spawn_release_handles() {
    if root().is_some() {
        return;
    }
    let _gate = TEST_GATE.lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("한글 공백 fixture.exe");
    fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
    let payloads = [
        "",
        "한글 공백",
        "quote\"tail",
        "trailing\\",
        "\\\"mixed\\",
        "& | $(); %PATH%",
        "😀",
    ];
    let mut args: Vec<std::ffi::OsString> = ["--exact", "argv_entry", "--nocapture"]
        .into_iter()
        .map(Into::into)
        .collect();
    for payload in payloads {
        args.push(payload.into());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let mut spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: None,
        executable,
        args,
        timeout: Duration::from_secs(3),
        max_log_bytes: 8192,
    };
    let output = process::run_metadata(&spec, &cancel).unwrap();
    assert_eq!(output.execution.exit_code, Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    let line = text
        .lines()
        .find_map(|line| line.split_once("argv-evidence=").map(|(_, value)| value))
        .unwrap();
    let observed: Vec<String> = serde_json::from_str(line).unwrap();
    assert_eq!(
        observed[1..],
        spec.args
            .iter()
            .map(|arg| arg.to_str().unwrap())
            .collect::<Vec<_>>()
    );
    // CreateProcess failure occurs after Job and pipe creation. Repeat to catch leaks.
    spec.executable = temp.path().join("absent.exe");
    // Warm up Windows loader/error-path caches before measuring repeated failures.
    for _ in 0..16 {
        assert!(process::run(&spec, &cancel).is_err());
    }
    let mut before = 0;
    unsafe {
        assert_ne!(
            windows_sys::Win32::System::Threading::GetProcessHandleCount(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut before
            ),
            0
        );
    }
    for _ in 0..64 {
        assert!(process::run(&spec, &cancel).is_err());
    }
    let mut after = 0;
    unsafe {
        assert_ne!(
            windows_sys::Win32::System::Threading::GetProcessHandleCount(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut after
            ),
            0
        );
    }
    println!("failed-spawn-handles: {before} -> {after}");
    // Windows/harness may own transient handles; persistent per-failure growth fails.
    assert!(
        after <= before + 4,
        "failed spawn leaked handles: {before} -> {after}"
    );
    spec.args = vec![std::ffi::OsString::from("NUL\0bad")];
    assert_eq!(
        process::run(&spec, &cancel).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    spec.args = vec![std::ffi::OsString::from("a".repeat(32767))];
    assert_eq!(
        process::run(&spec, &cancel).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
}
