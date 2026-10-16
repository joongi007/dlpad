//! Explicit native Linux capability tests; all credentials and artifacts are synthetic.
#![cfg(target_os = "linux")]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    cookies::import_session,
    download::{DownloadError, DownloadRepository, DownloadSession},
    process::ProcessOutcome,
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const SECRET: &str = "synthetic-native-cookie-never-real";
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
if m=='cancel':(r/'ready').write_text('synthetic ready');time.sleep(10);sys.exit(11)
media=pathlib.Path(a[a.index('--paths')+1]);target=media/a[a.index('--output')+1].replace('%(ext)s','mp4')
target.write_bytes(b'synthetic-native-artifact');print('DLPAD_ARTIFACT:'+json.dumps(str(target)),flush=True)
"#;
fn native_case(case: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cookie_gate_worker", "--ignored", "--nocapture"])
        .env("DLPAD_COOKIE_GATE_CASE", case)
        .env("DLPAD_GUARDIAN_BACKEND", "user-systemd")
        .env_remove("DLPAD_TEST_SYSTEMD_STARTUP_FAULT")
        .env_remove("DLPAD_TEST_SYSTEMD_BUS_PATH")
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "native case {case} failed");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("native case deadline: {case}");
        }
        thread::sleep(Duration::from_millis(20));
    }
}
#[test]
#[ignore = "actual user manager and latest guardian helper required"]
fn native_authenticated_success() {
    native_case("success");
}
#[test]
#[ignore = "actual user manager and latest guardian helper required"]
fn native_authenticated_metadata_failure() {
    native_case("metadata-failure");
}
#[test]
#[ignore = "actual user manager and latest guardian helper required"]
fn native_authenticated_download_failure() {
    native_case("download-failure");
}
#[test]
#[ignore = "actual user manager and latest guardian helper required"]
fn native_authenticated_cancel() {
    native_case("cancel");
}
#[test]
#[ignore = "internal subprocess driver; run native_authenticated_* tests"]
fn cookie_gate_worker() {
    let Ok(case) = std::env::var("DLPAD_COOKIE_GATE_CASE") else {
        return;
    };
    assert!(["success", "metadata-failure", "download-failure", "cancel"].contains(&case.as_str()));
    assert_eq!(
        std::env::var("DLPAD_GUARDIAN_BACKEND").unwrap(),
        "user-systemd"
    );
    assert!(std::path::Path::new(env!("CARGO_BIN_EXE_dlpad-process-guard")).is_file());
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let source = root.join("synthetic.cookies");
    fs::write(&source, COOKIE).unwrap();
    let cache = root.join("cache");
    fs::create_dir(&cache).unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
    let sentinel = cache.join("foreign-sentinel");
    fs::write(&sentinel, b"preserve unrelated fixture").unwrap();
    let tool = root.join("fixture-tool");
    fs::write(&tool, TOOL).unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(root.join("mode"), &case).unwrap();
    let options = DownloadOptions {
        url: SourceUrl::parse("https://fixture.invalid/native-cookie").unwrap(),
        output_dir: root.join("media 한글"),
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
    let mut lease = import_session(&source, &cache).unwrap();
    let private = lease.path_for_tool().to_owned();
    let cancel = Arc::new(AtomicBool::new(false));
    let trigger = if case == "cancel" {
        let flag = cancel.clone();
        let ready = root.join("ready");
        Some(thread::spawn(move || {
            let end = Instant::now() + Duration::from_secs(8);
            while !ready.exists() && Instant::now() < end {
                thread::sleep(Duration::from_millis(10));
            }
            let observed = ready.exists();
            flag.store(true, Ordering::Release);
            observed
        }))
    } else {
        None
    };
    let started = Instant::now();
    let result = repo.execute_with_session(
        "native-cookie",
        &options,
        &tool,
        DownloadSession {
            #[cfg(windows)]
            runtime_environment: None,
            cancel: &cancel,
            pause: None,
            progress: None,
            cookies: Some(&lease),
            managed: None,
        },
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    if let Some(trigger) = trigger {
        assert!(trigger.join().unwrap(), "cancel worker never started");
    }
    let text = match result {
        Ok(result) => {
            assert_eq!(case, "success");
            assert_eq!(result.job.state, JobState::Downloaded);
            assert_eq!(
                fs::read(&result.job.artifacts[0].path).unwrap(),
                b"synthetic-native-artifact"
            );
            assert!(result.metadata.get("cookie_echo").is_none());
            format!("{} {}", result.execution.logs, result.metadata)
        }
        Err(error) => {
            match &error {
                DownloadError::Execution { stage, result } => {
                    assert_eq!(
                        *stage,
                        if case == "metadata-failure" {
                            "metadata"
                        } else {
                            "download"
                        }
                    );
                    if case == "cancel" {
                        assert_eq!(result.outcome, ProcessOutcome::Cancelled);
                    } else {
                        assert_eq!(
                            result.exit_code,
                            Some(if case == "metadata-failure" { 7 } else { 9 })
                        );
                    }
                }
                _ => panic!("native gate did not confirm cleanup: {error:?}"),
            };
            assert_ne!(
                repo.job("native-cookie").unwrap().state,
                JobState::Downloaded
            );
            format!("{error:?}")
        }
    };
    assert!(!text.contains(SECRET));
    assert!(!text.contains(private.to_str().unwrap()));
    let calls: Vec<serde_json::Value> = fs::read_to_string(root.join("calls"))
        .unwrap()
        .lines()
        .map(|x| serde_json::from_str(x).unwrap())
        .collect();
    assert_eq!(calls.len(), if case == "metadata-failure" { 2 } else { 3 });
    for call in calls {
        if call["stage"] == "version" {
            assert!(call["path"].is_null());
        } else {
            assert_eq!(call["path"], private.to_str().unwrap());
            assert_eq!(call["literal"], std::str::from_utf8(COOKIE).unwrap());
        }
    }
    let stored = serde_json::to_string(&repo.job("native-cookie").unwrap()).unwrap()
        + &repo
            .stored_options("native-cookie")
            .unwrap()
            .to_stored_json()
            .unwrap();
    assert!(!stored.contains(SECRET));
    assert!(!stored.contains(private.to_str().unwrap()));
    for entry in fs::read_dir(&root)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("jobs.sqlite"))
    {
        let bytes = fs::read(entry.path()).unwrap();
        assert!(!bytes.windows(SECRET.len()).any(|w| w == SECRET.as_bytes()));
        assert!(
            !bytes
                .windows(private.as_os_str().len())
                .any(|w| w == private.to_str().unwrap().as_bytes())
        );
    }
    assert!(private.exists());
    lease.cleanup().unwrap();
    assert!(!private.exists());
    assert_eq!(fs::read(source).unwrap(), COOKIE);
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve unrelated fixture");
}
