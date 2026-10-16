//! Kill only this test's synthetic worker; pin cleanup identities with pidfds.
#![cfg(target_os = "linux")]
use dlpad_engine::process::{self, ProcessSpec};
use std::{
    fs,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, atomic::AtomicBool},
    thread,
    time::{Duration, Instant},
};
struct Fixture {
    worker: Child,
    identities: Vec<OwnedFd>,
    control: Option<std::os::unix::net::UnixStream>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.control.take();
        let _ = self.worker.kill();
        let _ = self.worker.wait();
        for identity in &self.identities {
            // SAFETY: pidfds pin exactly the test-owned processes, never recycled PIDs.
            unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    identity.as_raw_fd(),
                    libc::SIGKILL,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                );
            }
        }
        // Failure paths must not leave synthetic orphan zombies. A pidfd's
        // fdinfo identifies only that pinned fixture; waitpid never signals PIDs.
        for identity in &self.identities {
            if let Ok(info) =
                fs::read_to_string(format!("/proc/self/fdinfo/{}", identity.as_raw_fd()))
                && let Some(pid) = info.lines().find_map(|line| {
                    line.strip_prefix("Pid:")
                        .and_then(|value| value.trim().parse::<i32>().ok())
                })
                && pid > 1
            {
                unsafe {
                    libc::waitpid(pid, std::ptr::null_mut(), 0);
                }
            }
        }
    }
}
fn pin(pid: i32) -> OwnedFd {
    // SAFETY: syscall returns a new owned descriptor for the fixture identity.
    let descriptor = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
    assert!(descriptor >= 0, "fixture identity must still be alive");
    unsafe { OwnedFd::from_raw_fd(descriptor as i32) }
}
fn exited(identity: &OwnedFd) -> bool {
    let mut event = libc::pollfd {
        fd: identity.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: poll writes one live pollfd, without blocking.
    unsafe { libc::poll(&mut event, 1, 0) == 1 && event.revents & libc::POLLIN != 0 }
}
fn read_ids(path: &Path) -> Vec<i32> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(bytes) = fs::read(path)
            && let Ok(ids) = serde_json::from_slice::<Vec<i32>>(&bytes)
        {
            return ids;
        }
        assert!(
            Instant::now() < deadline,
            "synthetic fixture startup timed out"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn crash_fixture_worker() {
    let Some(directory) = std::env::var_os("DLPAD_GUARDIAN_FIXTURE") else {
        return;
    };
    let directory = PathBuf::from(directory);
    let result = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: "python3".into(),
            args: vec![
                directory.join("root.py").into_os_string(),
                directory.into_os_string(),
            ],
            timeout: if std::env::var_os("DLPAD_GUARDIAN_DRAIN_EXPECTED").is_some() {
                Duration::from_millis(300)
            } else {
                Duration::from_secs(30)
            },
            max_log_bytes: 4096,
        },
        &Arc::new(AtomicBool::new(false)),
    );
    if std::env::var_os("DLPAD_GUARDIAN_DRAIN_EXPECTED").is_some() {
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut);
    } else if std::env::var_os("DLPAD_GUARDIAN_EXPECT_MISSING").is_some() {
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
#[test]
fn app_sigkill_reaps_nested_and_detached_descendants_and_preserves_unrelated_process() {
    assert!(Path::new(env!("CARGO_BIN_EXE_dlpad-process-guard")).is_file());
    // SAFETY: this dedicated test process adopts only children of its fixtures;
    // waits below always name an observed fixture PID, never arbitrary children.
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) }, 0);
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("root.py"), r#"
import json, os, pathlib, subprocess, sys, time
folder=pathlib.Path(sys.argv[1])
middle=subprocess.Popen([sys.executable,str(folder/'middle.py'),str(folder)])
detached=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'],start_new_session=True)
(folder/'root.json').write_text(json.dumps([os.getpid(),os.getppid(),middle.pid,detached.pid]))
time.sleep(30)
"#).unwrap();
    fs::write(
        directory.path().join("middle.py"),
        r#"
import json, os, pathlib, subprocess, sys, time
nested=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'],start_new_session=True)
(pathlib.Path(sys.argv[1])/'nested.json').write_text(json.dumps([nested.pid]))
time.sleep(30)
"#,
    )
    .unwrap();
    let unrelated = Command::new("python3")
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut outsider = Fixture {
        worker: unrelated,
        identities: vec![],
        control: None,
    };
    let worker = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_fixture_worker", "--nocapture"])
        .env("DLPAD_GUARDIAN_FIXTURE", directory.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut fixture = Fixture {
        worker,
        identities: vec![],
        control: None,
    };
    let mut ids = read_ids(&directory.path().join("root.json"));
    let guardian_pid = ids[1];
    ids.extend(read_ids(&directory.path().join("nested.json")));
    fixture.identities = ids.iter().map(|pid| pin(*pid)).collect();
    fixture.worker.kill().unwrap();
    fixture.worker.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !fixture.identities.iter().all(exited) {
        assert!(
            Instant::now() < deadline,
            "guardian left a running fixture descendant"
        );
        thread::sleep(Duration::from_millis(20));
    }
    // Reap the guardian now adopted by this test. Its children must already have
    // been reaped by its subreaper, including new sessions outside the original PGID.
    let mut status = 0;
    assert_eq!(
        unsafe { libc::waitpid(guardian_pid, &mut status, 0) },
        guardian_pid
    );
    for pid in ids {
        assert!(
            !PathBuf::from(format!("/proc/{pid}")).exists(),
            "fixture zombie remains: {pid}"
        );
    }
    assert!(
        outsider.worker.try_wait().unwrap().is_none(),
        "unrelated process was touched"
    );
}

#[test]
fn missing_guardian_fails_before_starting_external_tool() {
    let directory = tempfile::tempdir().unwrap();
    let worker = directory.path().join("isolated-test-worker");
    fs::copy(std::env::current_exe().unwrap(), &worker).unwrap();
    fs::write(
        directory.path().join("root.py"),
        "raise RuntimeError('external tool must never start')",
    )
    .unwrap();
    let status = Command::new(worker)
        .args(["--exact", "crash_fixture_worker", "--nocapture"])
        .env("DLPAD_GUARDIAN_FIXTURE", directory.path())
        .env("DLPAD_GUARDIAN_EXPECT_MISSING", "1")
        .stdin(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}
#[test]
fn normal_leader_exit_preserves_status_and_reaps_detached_pipe_holder() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("detached.json");
    let script = r#"
import json, pathlib, subprocess, sys
child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'], start_new_session=True)
pathlib.Path(sys.argv[1]).write_text(json.dumps([child.pid]))
print('bounded evidence')
sys.exit(7)
"#;
    let start = Instant::now();
    let result = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: "python3".into(),
            args: vec!["-c".into(), script.into(), path.clone().into_os_string()],
            timeout: Duration::from_secs(5),
            max_log_bytes: 4096,
        },
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(result.exit_code, Some(7));
    assert!(result.logs.contains("bounded evidence"));
    assert!(start.elapsed() < Duration::from_secs(3));
    for pid in read_ids(&path) {
        assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
    }
}

#[test]
fn leader_that_changes_group_is_still_owned_and_reaped() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("moved.json");
    let result = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: "python3".into(),
            args: vec![
                "-c".into(),
                r#"import json,os,pathlib,time
os.setpgid(0, os.getpgid(os.getppid()))
pathlib.Path(__import__('sys').argv[1]).write_text(json.dumps([os.getpid()]))
time.sleep(30)
"#
                .into(),
                path.clone().into_os_string(),
            ],
            timeout: Duration::from_millis(300),
            max_log_bytes: 4096,
        },
        &Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(result.outcome, process::ProcessOutcome::TimedOut);
    for pid in read_ids(&path) {
        assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
    }
}

fn guardian_fault_fixture(folder: &Path) -> Fixture {
    use std::io::Read;
    use std::os::unix::{net::UnixStream, process::CommandExt};
    fs::write(
        folder.join("fault-root.py"),
        r#"
import json, os, pathlib, subprocess, sys, time
folder=pathlib.Path(sys.argv[1])
child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'], start_new_session=True)
(folder/'fault.json').write_text(json.dumps([os.getpid(),child.pid]))
time.sleep(30)
"#,
    )
    .unwrap();
    let (mut owner, control) = UnixStream::pair().unwrap();
    let descriptor = control.as_raw_fd();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dlpad-process-guard"));
    command
        .args([
            "--dlpad-internal-guardian",
            &descriptor.to_string(),
            "--",
            "python3",
        ])
        .arg(folder.join("fault-root.py"))
        .arg(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: pass only this fixture-owned control socket to the guardian.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(descriptor, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let worker = command.spawn().unwrap();
    drop(control);
    owner
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut ready = [0];
    owner.read_exact(&mut ready).unwrap();
    assert_eq!(ready, [b'R']);
    let identities = read_ids(&folder.join("fault.json"))
        .into_iter()
        .map(pin)
        .collect();
    // Keep the anonymous socket alive without introducing production interfaces.
    // The fixture's Python root holds no socket descriptor across exec.
    Fixture {
        worker,
        identities,
        control: Some(owner),
    }
}

#[test]
fn guardian_shutdown_signals_clean_detached_descendants() {
    for signal in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP, libc::SIGQUIT] {
        let folder = tempfile::tempdir().unwrap();
        let mut fixture = guardian_fault_fixture(folder.path());
        let guardian = pin(fixture.worker.id() as i32);
        // SAFETY: pidfd targets this exact fixture guardian, not a recycled PID.
        assert_eq!(
            unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    guardian.as_raw_fd(),
                    signal,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                )
            },
            0
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if fixture.worker.try_wait().unwrap().is_some() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "guardian signal cleanup timed out"
            );
            thread::sleep(Duration::from_millis(20));
        }
        assert!(fixture.identities.iter().all(exited));
        for pid in read_ids(&folder.path().join("fault.json")) {
            assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
        }
    }
}

#[test]
fn guardian_sigkill_exposes_detached_descendant_limit() {
    // SAFETY: only this test's named synthetic descendants are waited below.
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) }, 0);
    let folder = tempfile::tempdir().unwrap();
    let mut fixture = guardian_fault_fixture(folder.path());
    fixture.worker.kill().unwrap();
    fixture.worker.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !exited(&fixture.identities[0]) {
        assert!(
            Instant::now() < deadline,
            "PDEATHSIG failed to stop direct leader"
        );
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !exited(&fixture.identities[1]),
        "update protection contract: detached child unexpectedly protected"
    );
    // Demonstrate the limitation, then clean exactly these test-owned pidfds.
    for identity in &fixture.identities {
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                identity.as_raw_fd(),
                libc::SIGKILL,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            );
        }
    }
    for pid in read_ids(&folder.path().join("fault.json")) {
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    }
}

#[test]
fn guardian_sigkill_does_not_block_worker_on_escaped_pipe_holder() {
    // SAFETY: this process adopts only synthetic descendants named below.
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) }, 0);
    let unrelated = Command::new("python3")
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut outsider = Fixture {
        worker: unrelated,
        identities: vec![],
        control: None,
    };
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("root.py"),
        r#"
import json, os, pathlib, subprocess, sys, time
folder=pathlib.Path(sys.argv[1])
child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'], start_new_session=True)
(folder/'root.json').write_text(json.dumps([os.getpid(),os.getppid(),child.pid]))
time.sleep(30)
"#,
    )
    .unwrap();
    let worker = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_fixture_worker", "--nocapture"])
        .env("DLPAD_GUARDIAN_FIXTURE", directory.path())
        .env("DLPAD_GUARDIAN_DRAIN_EXPECTED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut fixture = Fixture {
        worker,
        identities: vec![],
        control: None,
    };
    let ids = read_ids(&directory.path().join("root.json"));
    fixture.identities = ids.iter().map(|pid| pin(*pid)).collect();
    // Kill the exact guardian while leaving its detached stdout/stderr holder alive.
    assert_eq!(
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                fixture.identities[1].as_raw_fd(),
                libc::SIGKILL,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            )
        },
        0
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = fixture.worker.try_wait().unwrap() {
            assert!(status.success(), "worker must report drain timeout error");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "worker blocked beyond process timeout on escaped pipe holder"
        );
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        outsider.worker.try_wait().unwrap().is_none(),
        "unrelated process was touched"
    );
    assert!(
        !exited(&fixture.identities[2]),
        "drain bound is not descendant protection"
    );
    for identity in &fixture.identities {
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                identity.as_raw_fd(),
                libc::SIGKILL,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            );
        }
    }
    // The worker has already reaped its guardian. Reap only its two synthetic orphans.
    for index in [0, 2] {
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(ids[index], &mut status, 0) },
            ids[index]
        );
    }
}

#[test]
fn user_guardian_entry_preserves_null_stdin_and_literal_arguments() {
    use std::{io::Read, os::unix::net::UnixStream};
    let (mut app, service) = UnixStream::pair().unwrap();
    app.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_dlpad-process-guard"))
        .args([
            "--dlpad-internal-user-guardian", "--", "/usr/bin/python3", "-c",
            "import sys,json;print(json.dumps({'args':sys.argv[1:],'stdin_empty':sys.stdin.read()==''},ensure_ascii=False),flush=True);sys.exit(42)",
            "$VARIABLE", "literal space", "한글",
        ])
        .stdin(Stdio::from(OwnedFd::from(service)))
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut fixture = Fixture {
        worker: child,
        identities: vec![],
        control: None,
    };
    let mut ready = [0u8];
    app.read_exact(&mut ready).unwrap();
    assert_eq!(ready, *b"R");
    fixture.control = Some(app);
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = fixture.worker.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "tool stdin or pipe EOF did not terminate"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.code(), Some(42));
    let mut output = String::new();
    fixture
        .worker
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    let actual: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        actual,
        serde_json::json!({"args":["$VARIABLE","literal space","한글"],"stdin_empty":true})
    );
}
