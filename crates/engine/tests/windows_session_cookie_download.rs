//! Explicitly invoked native Windows authentication adapter; synthetic values only.
#![cfg(windows)]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    cookies::{import_session, prepare_session_cache},
    download::{DownloadError, DownloadRepository, DownloadSession},
    process::ProcessOutcome,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
const SECRET: &str = "windows-download-fixture-only";
const COOKIES:&[u8]=b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-download-fixture-only\n";
fn private(text: &str, copy: &Path, source: &Path) {
    assert!(!text.contains(SECRET));
    for path in [copy, source] {
        let literal = path.to_str().unwrap();
        assert!(!text.contains(literal));
        let json = serde_json::to_string(literal).unwrap();
        assert!(!text.contains(&json[1..json.len() - 1]));
    }
}
#[test]
#[ignore = "requires explicitly built native DLPAD_WINDOWS_COOKIE_FIXTURE_TOOL; no actual accounts"]
fn native_authenticated_download_success_failure_and_cancel() {
    let original = std::path::PathBuf::from(
        std::env::var_os("DLPAD_WINDOWS_COOKIE_FIXTURE_TOOL")
            .expect("explicit synthetic example exe required"),
    );
    assert!(original.is_absolute() && original.is_file());
    for mode in ["success", "metadata-failure", "download-failure", "cancel"] {
        let temp = tempfile::Builder::new()
            .prefix("dlpad-windows-cookie-download-owned-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("source.cookies");
        let cache = root.join("private-cache");
        let sentinel = root.join("unrelated-sentinel");
        let tool = root.join("synthetic-ytdlp.exe");
        let database = root.join("jobs.sqlite");
        fs::write(
            root.join("fixture.cap"),
            b"dlpad-windows-session-cookie-synthetic-fixture-v1",
        )
        .unwrap();
        fs::write(root.join("mode"), mode).unwrap();
        fs::write(&source, COOKIES).unwrap();
        fs::write(&sentinel, b"preserve unrelated synthetic file").unwrap();
        fs::copy(&original, &tool).unwrap();
        prepare_session_cache(&cache).unwrap();
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        assert!(copy.starts_with(&cache));
        assert_eq!(fs::read(&copy).unwrap(), COOKIES);
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/windows-authenticated-source").unwrap(),
            output_dir: root.join("media 한글"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let repo = DownloadRepository::open(&database).unwrap();
        repo.create_job("windows-auth-fixture", &options).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let trigger = if mode == "cancel" {
            let flag = cancel.clone();
            let ready = root.join("worker-ready");
            Some(thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(10);
                while !ready.exists() {
                    assert!(
                        Instant::now() < deadline,
                        "synthetic native tool readiness timed out"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                flag.store(true, Ordering::Release);
            }))
        } else {
            None
        };
        let result = repo.execute_with_session(
            "windows-auth-fixture",
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
        if let Some(trigger) = trigger {
            trigger.join().unwrap();
        }
        if mode == "success" {
            let completed = result.unwrap();
            assert_eq!(completed.job.state, JobState::Downloaded);
            assert_eq!(completed.execution.exit_code, Some(0));
            let artifact = &completed.job.artifacts[0].path;
            assert!(artifact.starts_with(&options.output_dir));
            assert_eq!(
                fs::read(artifact).unwrap(),
                b"synthetic authenticated Windows artifact"
            );
            private(&completed.execution.logs, &copy, &source);
            private(
                &serde_json::to_string(&completed.metadata).unwrap(),
                &copy,
                &source,
            );
            assert!(completed.metadata.get("cookie_echo").is_none());
            assert!(completed.metadata.get("formats").is_none());
            assert!(completed.metadata.get("title").is_none());
        } else {
            let error = result.err().unwrap();
            private(&format!("{error:?}"), &copy, &source);
            match error {
                DownloadError::Execution { stage, result } => {
                    assert_eq!(
                        stage,
                        if mode == "metadata-failure" {
                            "metadata"
                        } else {
                            "download"
                        }
                    );
                    if mode == "cancel" {
                        assert_eq!(result.outcome, ProcessOutcome::Cancelled);
                    } else {
                        assert_eq!(
                            result.exit_code,
                            Some(if mode == "metadata-failure" { 7 } else { 9 })
                        );
                    }
                    private(&result.logs, &copy, &source);
                }
                _ => panic!("authenticated synthetic failure classification changed"),
            }
            assert_eq!(
                repo.job("windows-auth-fixture").unwrap().state,
                if mode == "cancel" {
                    JobState::Cancelled
                } else {
                    JobState::Failed
                }
            );
        }
        let call_bytes = fs::read(root.join("calls.jsonl")).unwrap();
        assert!(call_bytes.len() <= 32768);
        let calls: Vec<serde_json::Value> = std::str::from_utf8(&call_bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(calls.len(), if mode == "metadata-failure" { 2 } else { 3 });
        assert_eq!(calls[0]["stage"], "version");
        assert!(calls[0]["cookie_path"].is_null());
        for call in calls.iter().skip(1) {
            assert_eq!(call["cookie_path"], copy.to_str().unwrap());
            assert_eq!(
                call["literal_sha256"],
                format!("{:x}", Sha256::digest(COOKIES))
            );
        }
        assert_eq!(calls[1]["stage"], "metadata");
        if mode != "metadata-failure" {
            assert_eq!(calls[2]["stage"], "download");
        }
        private(
            &serde_json::to_string(&repo.job("windows-auth-fixture").unwrap()).unwrap(),
            &copy,
            &source,
        );
        private(
            &repo
                .stored_options("windows-auth-fixture")
                .unwrap()
                .to_stored_json()
                .unwrap(),
            &copy,
            &source,
        );
        drop(repo);
        let reopened = DownloadRepository::open(&database).unwrap();
        let job = reopened.job("windows-auth-fixture").unwrap();
        assert_eq!(job.attempts.len(), 1);
        private(&serde_json::to_string(&job).unwrap(), &copy, &source);
        drop(reopened);
        for entry in fs::read_dir(&root).unwrap().flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("jobs.sqlite")
            {
                let bytes = fs::read(entry.path()).unwrap();
                assert!(bytes.len() <= 2097152);
                private(&String::from_utf8_lossy(&bytes), &copy, &source);
            }
        }
        lease.begin_reader().unwrap().confirm();
        lease.cleanup().unwrap();
        assert!(!copy.exists() && !copy.parent().unwrap().exists());
        assert_eq!(fs::read(&source).unwrap(), COOKIES);
        assert_eq!(
            fs::read(&sentinel).unwrap(),
            b"preserve unrelated synthetic file"
        );
        eprintln!("synthetic Windows authenticated case PASS: {mode}");
    }
}
