//! Explicit native Windows authenticated execution; synthetic private fixtures only.
#![cfg(windows)]
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
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const SECRET: &str = "windows-download-fixture-only";
const COOKIE: &[u8] = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-download-fixture-only\n";
fn native_case(case: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "cookie_gate_worker", "--ignored", "--nocapture"])
        .env("DLPAD_COOKIE_GATE_CASE", case)
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
#[ignore = "fresh native Windows guardian and session_cookie_fixture_tool.exe required"]
fn native_authenticated_success() {
    native_case("success");
}
#[test]
#[ignore = "fresh native Windows guardian and session_cookie_fixture_tool.exe required"]
fn native_authenticated_metadata_failure() {
    native_case("metadata-failure");
}
#[test]
#[ignore = "fresh native Windows guardian and session_cookie_fixture_tool.exe required"]
fn native_authenticated_download_failure() {
    native_case("download-failure");
}
#[test]
#[ignore = "fresh native Windows guardian and session_cookie_fixture_tool.exe required"]
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
    assert!(std::path::Path::new(env!("CARGO_BIN_EXE_dlpad-process-guard")).is_file());
    let temp = tempfile::Builder::new()
        .prefix("dlpad-windows-cookie-download-owned-")
        .tempdir()
        .unwrap();
    let root = temp.path().canonicalize().unwrap();
    let source = root.join("source.cookies");
    fs::write(&source, COOKIE).unwrap();
    let cache = root.join("private-cache");
    dlpad_engine::cookies::prepare_session_cache(&cache).unwrap();
    let sentinel = cache.join("foreign-sentinel");
    fs::write(&sentinel, b"preserve unrelated fixture").unwrap();
    let built_tool = std::path::PathBuf::from(
        std::env::var_os("DLPAD_WINDOWS_COOKIE_FIXTURE_EXE")
            .expect("fresh example executable required"),
    );
    assert!(built_tool.is_absolute() && built_tool.is_file());
    let tool = root.join("fixture-tool.exe");
    fs::copy(&built_tool, &tool).unwrap();
    fs::write(
        root.join("fixture.cap"),
        b"dlpad-windows-session-cookie-synthetic-fixture-v1",
    )
    .unwrap();
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
        let ready = root.join("worker-ready");
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
                b"synthetic authenticated Windows artifact"
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
    let calls: Vec<serde_json::Value> = fs::read_to_string(root.join("calls.jsonl"))
        .unwrap()
        .lines()
        .map(|x| serde_json::from_str(x).unwrap())
        .collect();
    assert_eq!(calls.len(), if case == "metadata-failure" { 2 } else { 3 });
    for call in calls {
        if call["stage"] == "version" {
            assert!(call["cookie_path"].is_null());
        } else {
            assert_eq!(call["cookie_path"], private.to_str().unwrap());
            assert_eq!(
                call["literal_sha256"],
                format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(COOKIE))
            );
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
                .windows(private.to_str().unwrap().len())
                .any(|w| w == private.to_str().unwrap().as_bytes())
        );
    }
    // Close the original mandatory journal lock before independent file inspection.
    lease.defer_cleanup();
    let owner = private.parent().unwrap();
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(owner.join("reader-state.json")).unwrap()).unwrap();
    assert_eq!(state["slots"][0]["state"], "Resolved");
    assert_eq!(
        fs::metadata(owner.join("reader0.receipt")).unwrap().len(),
        256
    );
    if case == "metadata-failure" {
        assert_eq!(state["slots"][1]["state"], "Empty");
        assert_eq!(
            fs::metadata(owner.join("reader1.receipt")).unwrap().len(),
            0
        );
    } else {
        assert_eq!(state["slots"][1]["state"], "Resolved");
        assert_eq!(
            fs::metadata(owner.join("reader1.receipt")).unwrap().len(),
            256
        );
    }
    assert!(private.exists());
    let recovered = dlpad_engine::cookies::recover_sessions(&cache).unwrap();
    assert_eq!(recovered.recovered, 1);
    assert_eq!(recovered.deferred, 0);
    assert!(!private.exists());
    assert_eq!(fs::read(source).unwrap(), COOKIE);
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve unrelated fixture");
}
