//! Bounded native Windows writer112 SOFTWARE INJECTION; never fills any disk.
#![cfg(windows)]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    download::{DownloadError, DownloadRepository},
    process::ProcessOutcome,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
fn hash(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let size = file.metadata().unwrap().len();
    assert!(size <= 128 * 1024 * 1024);
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        count += n as u64;
        assert!(count <= size);
        digest.update(&buffer[..n]);
    }
    assert_eq!(count, size);
    assert_eq!(file.metadata().unwrap().len(), size);
    format!("{:x}", digest.finalize())
}
fn plain(path: &Path) {
    for node in path.ancestors() {
        if node.exists() {
            assert_eq!(
                fs::symlink_metadata(node).unwrap().file_attributes() & 0x400,
                0,
                "no reparse fixture path"
            );
        }
    }
}
fn new(path: &Path, bytes: &[u8]) {
    plain(path);
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}
struct Deadline {
    cancel: Arc<AtomicBool>,
    done: Arc<(Mutex<bool>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Deadline {
    fn new() -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let done = Arc::new((Mutex::new(false), Condvar::new()));
        let c = cancel.clone();
        let d = done.clone();
        let worker = thread::spawn(move || {
            let (end, _) =
                d.1.wait_timeout_while(d.0.lock().unwrap(), Duration::from_secs(30), |v| !*v)
                    .unwrap();
            if !*end {
                c.store(true, Ordering::Release);
            }
        });
        Self {
            cancel,
            done,
            worker: Some(worker),
        }
    }
}
impl Drop for Deadline {
    fn drop(&mut self) {
        *self.done.0.lock().unwrap() = true;
        self.done.1.notify_all();
        self.worker.take().unwrap().join().unwrap();
    }
}
#[test]
#[ignore = "root builds original pinned native fixture and guardian; explicit owned project TEMP and private host required"]
fn windows_download_writer_failure_preserves_partial_and_resumes() {
    assert!(Path::new(env!("CARGO_BIN_EXE_dlpad-process-guard")).is_file());
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let temp = project
        .join(".local-tools/windows/tmp")
        .canonicalize()
        .unwrap();
    plain(&temp);
    let original = PathBuf::from(
        std::env::var_os("DLPAD_WINDOWS_WRITER_FIXTURE_EXE")
            .expect("original built native example"),
    );
    assert!(original.is_absolute());
    plain(&original);
    assert!(
        original
            .canonicalize()
            .unwrap()
            .starts_with(project.join(".local-tools").canonicalize().unwrap())
    );
    let expected =
        std::env::var("DLPAD_WINDOWS_WRITER_FIXTURE_SHA256").expect("original fixture SHA pin");
    assert!(
        expected.len() == 64
            && expected
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    assert_eq!(hash(&original), expected);
    let _original_pin = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&original)
        .unwrap();
    let root = tempfile::Builder::new()
        .prefix("dlpad-writer-failure-owned-")
        .tempdir_in(&temp)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    plain(&root);
    let tool = root.join("fixture-tool.exe");
    let mut reader = File::open(&original).unwrap();
    let mut writer = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tool)
        .unwrap();
    std::io::copy(&mut reader, &mut writer).unwrap();
    writer.sync_all().unwrap();
    drop(writer);
    assert_eq!(hash(&tool), expected);
    let _tool_pin = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&tool)
        .unwrap();
    new(
        &root.join("fixture.cap"),
        b"dlpad-windows-writer-injected112-fixture-v1",
    );
    new(&root.join("mode"), b"injected112");
    let output = root.join("media 한글");
    fs::create_dir(&output).unwrap();
    let sentinel = output.join("foreign-sentinel.mp4");
    new(&sentinel, b"preserve unrelated owned fixture");
    let sentinel_sha = hash(&sentinel);
    let options = DownloadOptions {
        url: SourceUrl::parse("https://fixture.invalid/writer-injected112").unwrap(),
        output_dir: output.clone(),
        quality: Quality::Best,
        time_range: None,
        advanced: vec![],
        ffmpeg: None,
        deno: None,
        final_name_style: None,
        subtitles: None,
    };
    let db = root.join("jobs.sqlite");
    let repo = DownloadRepository::open(&db).unwrap();
    repo.create_job("writer-native-job", &options).unwrap();
    let deadline = Deadline::new();
    let start = Instant::now();
    let error = repo
        .execute("writer-native-job", &options, &tool, &deadline.cancel)
        .err()
        .expect("injected writer failure must fail");
    let DownloadError::Execution { stage, result } = error else {
        panic!("expected download execution error")
    };
    assert_eq!(stage, "download");
    assert_eq!(result.outcome, ProcessOutcome::Exited);
    assert_eq!(result.exit_code, Some(7));
    let marker = result
        .logs
        .lines()
        .find_map(|s| s.strip_prefix("DLPAD_WRITER_ERROR:"))
        .expect("injected writer evidence");
    let marker: serde_json::Value = serde_json::from_str(marker).unwrap();
    assert_eq!(marker["win32_error"], 112);
    assert_eq!(marker["injected"], true);
    assert_eq!(marker["partial_bytes"], 65536);
    assert!(start.elapsed() < Duration::from_secs(30));
    assert!(!deadline.cancel.load(Ordering::Acquire));
    drop(deadline);
    let failed = repo.job("writer-native-job").unwrap();
    assert_eq!(failed.state, JobState::Failed);
    assert_eq!(failed.attempts.len(), 1);
    assert!(failed.artifacts.is_empty());
    let partial = output.join(".dlpad-partials/writer-native-job/writer-native-job.mp4.part");
    assert_eq!(fs::read(&partial).unwrap(), vec![b'p'; 65536]);
    let partial_sha = hash(&partial);
    let partial_pin = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&partial)
        .unwrap();
    let partial_id = dlpad_engine::file_identity::from_file(&partial_pin).unwrap();
    assert!(!output.join("writer-native-job.mp4").exists());
    drop(repo);
    let repo = DownloadRepository::open(&db).unwrap();
    assert_eq!(
        repo.job("writer-native-job").unwrap().state,
        JobState::Failed
    );
    assert_eq!(
        repo.stored_options("writer-native-job")
            .unwrap()
            .to_stored_json()
            .unwrap(),
        options.to_stored_json().unwrap()
    );
    // Only this owned mode switch is overwritten; no source/user file or partial is modified.
    fs::write(root.join("mode"), b"success").unwrap();
    let deadline = Deadline::new();
    let resumed = repo
        .resume("writer-native-job", &tool, &deadline.cancel)
        .unwrap();
    assert!(!deadline.cancel.load(Ordering::Acquire));
    drop(deadline);
    assert_eq!(resumed.job.id, failed.id);
    assert_eq!(resumed.job.state, JobState::Downloaded);
    assert_eq!(resumed.job.attempts.len(), 2);
    assert_eq!(resumed.job.attempts[0], failed.attempts[0]);
    assert_eq!(resumed.job.attempts[1].number, 2);
    assert_ne!(resumed.job.attempts[0].id, resumed.job.attempts[1].id);
    assert_eq!(resumed.job.artifacts.len(), 1);
    assert_eq!(
        fs::read(&resumed.job.artifacts[0].path).unwrap(),
        b"synthetic recovered writer bytes; not a real video"
    );
    assert_eq!(hash(&partial), partial_sha);
    assert_eq!(
        dlpad_engine::file_identity::from_file(&File::open(&partial).unwrap()).unwrap(),
        partial_id
    );
    assert_eq!(hash(&sentinel), sentinel_sha);
    assert_eq!(
        fs::read_to_string(root.join("extractions.txt"))
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![options.url.as_str(); 2]
    );
    drop(repo);
    let repo = DownloadRepository::open(&db).unwrap();
    assert_eq!(repo.recover().unwrap(), 0);
    let final_job = repo.job("writer-native-job").unwrap();
    assert_eq!(final_job.state, JobState::Downloaded);
    assert_eq!(final_job.attempts.len(), 2);
    assert_eq!(final_job.artifacts, resumed.job.artifacts);
    assert_eq!(hash(&original), expected);
    assert_eq!(hash(&tool), expected);
    let integrity =
        rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let integrity: String = integrity
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let receipt = serde_json::json!({"schema":1,"status":"REVIEW_REQUIRED","injected_win32_error":112,"physical_disk_full_proven":false,"owned_root":root,"fixture_sha256":expected,"partial_bytes":65536,"partial_sha256":partial_sha,"sentinel_sha256":sentinel_sha,"job_id":final_job.id,"attempts":2,"state":"Downloaded","artifact_count":1,"db_reopen_verified":true,"integrity_check":"ok","partial_native_identity_preserved":true,"same_job_resume_verified":true,"original_files_preserved":true});
    new(
        &root.join("receipt.json"),
        &serde_json::to_vec_pretty(&receipt).unwrap(),
    );
    println!("DLPAD_WRITER_FAILURE_RECEIPT:{receipt}");
}
