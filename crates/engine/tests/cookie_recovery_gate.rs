//! 합성 쿠키만 사용하는 명시적 same-boot native crash 회귀.
#![cfg(target_os = "linux")]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    cookies::{import_session, recover_sessions},
    download::{DownloadRepository, DownloadSession},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    thread,
    time::{Duration, Instant},
};
const COOKIE: &[u8] = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-native-cookie-never-real\n";
const TOOL: &str = r#"#!/usr/bin/python3
import json,pathlib,sys,time
r=pathlib.Path(__file__).parent;a=sys.argv[1:];m=(r/'mode').read_text()
s='version' if '--version' in a else 'metadata' if '--dump-single-json' in a else 'download'
p=pathlib.Path(a[a.index('--cookies')+1]) if '--cookies' in a else None
with (r/'calls').open('a') as f:f.write(json.dumps({'stage':s,'path':str(p) if p else None,'literal':p.read_text() if p else None})+'\n')
if s=='version':assert p is None;print('2026.10.04');sys.exit(0)
assert p is not None
secret='synthetic-native-cookie-never-real'
if s=='metadata':
 if m=='metadata-failure':print(secret+' '+str(p),file=sys.stderr);sys.exit(7)
 print(json.dumps({'id':'native-cookie','duration':10,'title':secret,'cookie_echo':str(p)}));sys.exit(0)
print(secret+' '+str(p),flush=True);print(secret+' '+str(p),file=sys.stderr,flush=True)
if m=='download-failure':sys.exit(9)
if m=='cancel':(r/'ready').write_text('synthetic ready');time.sleep(15);sys.exit(11)
media=pathlib.Path(a[a.index('--paths')+1]);target=media/a[a.index('--output')+1].replace('%(ext)s','mp4')
target.write_bytes(b'synthetic-native-artifact');print('DLPAD_ARTIFACT:'+json.dumps(str(target)),flush=True)
"#;

fn identity(pid: u32) -> u64 {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    stat.rsplit_once(')')
        .unwrap()
        .1
        .split_whitespace()
        .nth(19)
        .unwrap()
        .parse()
        .unwrap()
}
fn preserved(root: &Path) {
    assert_eq!(fs::read(root.join("synthetic.cookies")).unwrap(), COOKIE);
    assert_eq!(
        fs::read(root.join("cache/foreign-sentinel")).unwrap(),
        b"preserve unrelated fixture"
    );
}
struct Worker(std::process::Child);
impl Drop for Worker {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            self.0.kill().unwrap();
            self.0.wait().unwrap();
        }
    }
}
fn native_case(crash: bool) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(root.join("synthetic.cookies"), COOKIE).unwrap();
    fs::create_dir(root.join("cache")).unwrap();
    fs::set_permissions(root.join("cache"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        root.join("cache/foreign-sentinel"),
        b"preserve unrelated fixture",
    )
    .unwrap();
    fs::write(root.join("fixture-tool"), TOOL).unwrap();
    fs::set_permissions(root.join("fixture-tool"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(root.join("mode"), if crash { "cancel" } else { "success" }).unwrap();
    let baseline = recover_sessions(&root.join("cache")).unwrap();
    assert_eq!(
        (baseline.recovered, baseline.deferred, baseline.rejected),
        (0, 0, 1)
    );
    assert!(!baseline.truncated);
    preserved(&root);
    let started = Instant::now();
    let mut worker = Worker(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "sameboot_recovery_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("DLPAD_RECOVERY_FIXTURE_ROOT", &root)
            .env("DLPAD_GUARDIAN_BACKEND", "user-systemd")
            .env_remove("DLPAD_TEST_SYSTEMD_STARTUP_FAULT")
            .env_remove("DLPAD_TEST_SYSTEMD_BUS_PATH")
            .spawn()
            .unwrap(),
    );
    let start_ticks = identity(worker.0.id());
    if crash {
        while !root.join("ready").exists() {
            assert!(
                worker.0.try_wait().unwrap().is_none(),
                "worker exited before actual tool readiness"
            );
            assert!(
                started.elapsed() < Duration::from_secs(12),
                "actual tool readiness deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
        // Actual tool validated its literal private copy before writing this marker.
        assert_eq!(fs::read(root.join("ready")).unwrap(), b"synthetic ready");
        assert_eq!(identity(worker.0.id()), start_ticks);
        worker.0.kill().unwrap();
        assert!(!worker.0.wait().unwrap().success());
    } else {
        loop {
            if let Some(status) = worker.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(15),
                "success worker deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            fs::read(root.join("success")).unwrap(),
            b"actual execution success"
        );
        assert_eq!(
            fs::read(root.join("artifact")).unwrap(),
            b"synthetic-native-artifact"
        );
    }
    let private = PathBuf::from(fs::read_to_string(root.join("private-path")).unwrap());
    assert!(private.starts_with(root.join("cache")) && private.exists());
    let owner = private.parent().unwrap().to_owned();
    let mut recovered = 0;
    let deadline = Instant::now() + Duration::from_secs(10);
    let first = recover_sessions(&root.join("cache")).unwrap();
    eprintln!(
        "sameboot first recovery: recovered={} deferred={} rejected={}",
        first.recovered, first.deferred, first.rejected
    );
    assert_eq!(first.rejected, baseline.rejected);
    assert!(!first.truncated);
    recovered += first.recovered;
    if crash {
        assert!(first.recovered == 1 || first.deferred == 1);
    }
    while recovered == 0 {
        assert!(
            Instant::now() < deadline,
            "confirmed native receipt recovery deadline"
        );
        thread::sleep(Duration::from_millis(20));
        let next = recover_sessions(&root.join("cache")).unwrap();
        assert_eq!(next.rejected, baseline.rejected);
        assert!(!next.truncated);
        recovered += next.recovered;
    }
    assert_eq!(recovered, 1);
    assert!(!private.exists() && !owner.exists());
    preserved(&root);
    assert_eq!(fs::read_dir(root.join("cache")).unwrap().count(), 1);
    if !crash {
        assert_eq!(
            fs::read(root.join("artifact")).unwrap(),
            b"synthetic-native-artifact"
        );
    }
    assert!(started.elapsed() < Duration::from_secs(30));
}
#[test]
#[ignore = "actual user manager and latest guardian required"]
fn sameboot_success_without_destructors_recovers() {
    native_case(false);
}
#[test]
#[ignore = "actual user manager and latest guardian required"]
fn sameboot_killed_worker_recovers_only_after_terminal_receipt() {
    native_case(true);
}
#[test]
#[ignore = "internal owned worker; use sameboot_* parent tests"]
fn sameboot_recovery_worker() {
    let Ok(raw) = std::env::var("DLPAD_RECOVERY_FIXTURE_ROOT") else {
        return;
    };
    let root = PathBuf::from(raw);
    assert!(root.is_absolute() && root.starts_with("/tmp"));
    assert_eq!(
        fs::symlink_metadata(&root).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert_eq!(root.canonicalize().unwrap(), root);
    let options = DownloadOptions {
        url: SourceUrl::parse("https://fixture.invalid/native-cookie").unwrap(),
        output_dir: root.join("media"),
        quality: Quality::Best,
        time_range: None,
        advanced: vec![],
        ffmpeg: None,
        deno: None,
        final_name_style: None,
        subtitles: None,
    };
    let repo = DownloadRepository::open(&root.join("jobs.sqlite")).unwrap();
    repo.create_job("native-cookie", &options).unwrap();
    let lease = import_session(&root.join("synthetic.cookies"), &root.join("cache")).unwrap();
    fs::write(
        root.join("private-path"),
        lease.path_for_tool().as_os_str().as_encoded_bytes(),
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let result = repo
        .execute_with_session(
            "native-cookie",
            &options,
            &root.join("fixture-tool"),
            DownloadSession {
                #[cfg(windows)]
                runtime_environment: None,
                cancel: &cancel,
                pause: None,
                progress: None,
                cookies: Some(&lease),
                managed: None,
            },
        )
        .unwrap();
    assert_eq!(result.job.state, JobState::Downloaded);
    let artifact = fs::read(&result.job.artifacts[0].path).unwrap();
    assert_eq!(artifact, b"synthetic-native-artifact");
    fs::write(root.join("artifact"), artifact).unwrap();
    fs::write(root.join("success"), b"actual execution success").unwrap();
    // Intentionally skip lease, repository and all other Rust destructors.
    std::process::exit(0);
}
