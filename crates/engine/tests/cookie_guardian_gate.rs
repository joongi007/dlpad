//! Actual helper gate regression with owned temporary files and no user credentials.
#![cfg(target_os = "linux")]
use std::{
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{net::UnixStream, process::CommandExt},
    },
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
#[path = "../src/process/reader_protocol.rs"]
mod reader_protocol;
struct OwnedHelper(Child);
impl OwnedHelper {
    fn wait(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "owned helper did not terminate");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for OwnedHelper {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                if self.0.try_wait().ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
fn spawn(marker: &std::path::Path) -> (OwnedHelper, UnixStream) {
    let (app, peer) = UnixStream::pair().unwrap();
    let descriptor = peer.as_raw_fd();
    let mut command = Command::new(env!("CARGO_BIN_EXE_dlpad-process-guard"));
    command.args(["--dlpad-internal-cookie-guardian", &descriptor.to_string(), "0", &"01".repeat(16), "--", "/usr/bin/python3", "-c",
        "import os,stat,sys; from pathlib import Path; assert sys.stdin.read()==''; assert not any(stat.S_ISSOCK(os.fstat(int(n)).st_mode) for n in os.listdir('/proc/self/fd') if n.isdigit() and int(n)>2 and os.path.exists('/proc/self/fd/'+n)); Path(sys.argv[1]).write_text('owned synthetic tool started')"])
        .arg(marker).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    // SAFETY: this live socket remains owned through spawn; only child-side CLOEXEC is changed.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(descriptor, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = command.spawn().unwrap();
    drop(peer);
    app.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    (OwnedHelper(child), app)
}
#[test]
fn actual_begin_is_required_before_tool_spawn_and_control_never_reaches_tool() {
    let folder = tempfile::tempdir().unwrap();
    let marker = folder.path().join("tool-started");
    let (mut helper, mut app) = spawn(&marker);
    std::thread::sleep(Duration::from_millis(100));
    assert!(helper.0.try_wait().unwrap().is_none());
    assert!(!marker.exists());
    let packet = reader_protocol::encode_begin(&reader_protocol::Begin {
        index: 0,
        nonce: [1; 16],
    })
    .unwrap();
    app.write_all(&packet).unwrap();
    let mut frame = Vec::new();
    app.read_to_end(&mut frame).unwrap();
    assert_eq!(frame, b"RGC");
    assert!(helper.wait().success());
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "owned synthetic tool started"
    );
}
#[test]
fn actual_wrong_nonce_and_partial_eof_never_spawn_tool() {
    for partial in [false, true] {
        let folder = tempfile::tempdir().unwrap();
        let marker = folder.path().join("tool-started");
        let (mut helper, mut app) = spawn(&marker);
        let packet = reader_protocol::encode_begin(&reader_protocol::Begin {
            index: 0,
            nonce: [2; 16],
        })
        .unwrap();
        app.write_all(if partial { &packet[..9] } else { &packet })
            .unwrap();
        app.shutdown(std::net::Shutdown::Write).unwrap();
        let mut frame = Vec::new();
        app.read_to_end(&mut frame).unwrap();
        assert_eq!(frame, b"GU");
        assert_eq!(helper.wait().code(), Some(126));
        assert!(!marker.exists());
    }
}
