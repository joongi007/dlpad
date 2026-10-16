//! Synthetic session authentication only: no accounts, network, or user cookie data.
#![cfg(target_os = "linux")]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    cookies::{CookieImportError, SessionCookieLease, import_session},
    download::{DownloadError, DownloadRepository, DownloadSession},
    process::{self, ProcessOutcome, ProcessSpec},
};
use std::{
    fs,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const SECRET: &str = "synthetic-cookie-only-never-real";
const COOKIES: &[u8] = b"# Netscape HTTP Cookie File\n# literal fixture comment\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-cookie-only-never-real\n";
const TOOL: &str = r#"#!/usr/bin/python3
import json,os,pathlib,sys,time
root=pathlib.Path(__file__).parent;a=sys.argv[1:];mode=(root/'mode').read_text()
stage='version' if '--version' in a else 'metadata' if '--dump-single-json' in a else 'download'
path=pathlib.Path(a[a.index('--cookies')+1]) if '--cookies' in a else None
with (root/'calls.jsonl').open('a') as f:f.write(json.dumps({'stage':stage,'cookie_path':str(path) if path else None,'literal':path.read_text() if path else None})+'\n')
if stage=='version':
 assert path is None;print('2026.10.04');sys.exit(0)
assert path is not None
secret='synthetic-cookie-only-never-real'
if stage=='metadata':
 if mode=='metadata-failure':print('cookie='+secret+' path='+str(path),file=sys.stderr);sys.exit(7)
 duration=secret if mode=='duration-string' else {'cookie':secret,'path':str(path)} if mode=='duration-object' else 10
 print(json.dumps({'id':'synthetic-auth','duration':duration,'title':secret,'cookie_echo':str(path),'formats':[{'url':'https://fixture.invalid/'+secret}]}))
 print('metadata stderr '+secret+' '+str(path),file=sys.stderr);sys.exit(0)
print('download stdout '+secret+' '+str(path),flush=True)
print('download stderr '+secret+' '+str(path),file=sys.stderr,flush=True)
if mode=='download-failure':sys.exit(9)
media=pathlib.Path(a[a.index('--paths')+1]);target=media/a[a.index('--output')+1].replace('%(ext)s','mp4')
if mode in ['cancel','guardian-kill']:
 target.with_suffix('.part').write_bytes(b'owned partial')
 if mode=='guardian-kill':
  def ident(pid):return {'pid':pid,'start_ticks':int(pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(')',1)[1].split()[19])}
  tmp=root/'owned-identities.tmp';tmp.write_text(json.dumps({'worker':ident(os.getpid()),'guardian':ident(os.getppid())}));tmp.replace(root/'owned-identities.json')
 (root/'worker-ready').write_text('owned worker ready');time.sleep(30)
target.write_bytes(b'synthetic authenticated media')
print('DLPAD_ARTIFACT:'+json.dumps(str(target)),flush=True)
"#;
struct Fixture {
    temp: tempfile::TempDir,
    root: PathBuf,
    source: PathBuf,
    cache: PathBuf,
    tool: PathBuf,
    options: DownloadOptions,
}
impl Fixture {
    fn new(mode: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("external-source.cookies");
        fs::write(&source, COOKIES).unwrap();
        let cache = root.join("private-cache");
        fs::create_dir(&cache).unwrap();
        fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            cache.join("unrelated-sentinel"),
            b"keep unrelated cache file",
        )
        .unwrap();
        let tool = root.join("synthetic-ytdlp");
        fs::write(&tool, TOOL).unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(root.join("mode"), mode).unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/authenticated-source").unwrap(),
            output_dir: root.join("media 한글"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        Self {
            temp,
            root,
            source,
            cache,
            tool,
            options,
        }
    }
    fn lease(&self) -> SessionCookieLease {
        import_session(&self.source, &self.cache).unwrap()
    }
    fn repo(&self) -> DownloadRepository {
        let repo = DownloadRepository::open(&self.root.join("jobs.sqlite")).unwrap();
        repo.create_job("auth-fixture", &self.options).unwrap();
        repo
    }
    fn calls(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.root.join("calls.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn preserve(&self) {
        assert_eq!(fs::read(&self.source).unwrap(), COOKIES);
        assert_eq!(
            fs::read(self.cache.join("unrelated-sentinel")).unwrap(),
            b"keep unrelated cache file"
        );
        assert!(self.temp.path().exists());
    }
    fn no_persistence(&self, repo: &DownloadRepository, path: &Path) {
        let private = path.to_str().unwrap();
        let job = serde_json::to_string(&repo.job("auth-fixture").unwrap()).unwrap();
        let options = repo
            .stored_options("auth-fixture")
            .unwrap()
            .to_stored_json()
            .unwrap();
        for text in [job, options] {
            assert!(!text.contains(SECRET));
            assert!(!text.contains(private));
            assert!(!text.contains("--cookies"));
        }
        for entry in fs::read_dir(&self.root).unwrap().flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("jobs.sqlite")
            {
                let bytes = fs::read(entry.path()).unwrap();
                assert!(!bytes.windows(SECRET.len()).any(|x| x == SECRET.as_bytes()));
                assert!(
                    !bytes
                        .windows(private.len())
                        .any(|x| x == private.as_bytes())
                );
            }
        }
    }
    fn verify_calls(&self, path: &Path) {
        let calls = self.calls();
        assert!(calls.iter().any(|x| x["stage"] == "version"));
        for call in calls {
            if call["stage"] == "version" {
                assert!(call["cookie_path"].is_null());
            } else {
                assert_eq!(call["cookie_path"], path.to_str().unwrap());
                assert_eq!(call["literal"], std::str::from_utf8(COOKIES).unwrap());
            }
        }
    }
}
fn session<'a>(cancel: &'a Arc<AtomicBool>, lease: &'a SessionCookieLease) -> DownloadSession<'a> {
    DownloadSession {
        #[cfg(windows)]
        runtime_environment: None,
        cancel,
        pause: None,
        progress: None,
        cookies: Some(lease),
        managed: None,
    }
}
fn private_text(text: &str, path: &Path) {
    assert!(!text.contains(SECRET));
    assert!(!text.contains(path.to_str().unwrap()));
    assert!(!text.contains("download stdout"));
    assert!(!text.contains("download stderr"));
    assert!(!text.contains("metadata stderr"));
}
fn confirm_reader_available(lease: &SessionCookieLease) {
    lease.begin_reader().unwrap().confirm();
}

#[test]
fn authenticated_success_shares_literal_private_copy_without_log_metadata_or_db_persistence() {
    let f = Fixture::new("success");
    let repo = f.repo();
    let mut lease = f.lease();
    let path = lease.path_for_tool().to_owned();
    let cancel = Arc::new(AtomicBool::new(false));
    let result = repo
        .execute_with_session(
            "auth-fixture",
            &f.options,
            &f.tool,
            session(&cancel, &lease),
        )
        .unwrap();
    assert_eq!(result.job.state, JobState::Downloaded);
    assert_eq!(result.execution.exit_code, Some(0));
    assert_eq!(
        fs::read(&result.job.artifacts[0].path).unwrap(),
        b"synthetic authenticated media"
    );
    private_text(&result.execution.logs, &path);
    private_text(&serde_json::to_string(&result.metadata).unwrap(), &path);
    assert!(result.metadata.get("cookie_echo").is_none());
    f.verify_calls(&path);
    f.no_persistence(&repo, &path);
    assert!(path.exists());
    confirm_reader_available(&lease);
    lease.cleanup().unwrap();
    assert!(!path.exists());
    f.preserve();
}
#[test]
fn metadata_and_download_failures_omit_private_output_and_cleanup_only_owned_session() {
    for mode in ["metadata-failure", "download-failure"] {
        let f = Fixture::new(mode);
        let repo = f.repo();
        let mut lease = f.lease();
        let path = lease.path_for_tool().to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let error = repo
            .execute_with_session(
                "auth-fixture",
                &f.options,
                &f.tool,
                session(&cancel, &lease),
            )
            .err()
            .unwrap();
        private_text(&format!("{error:?}"), &path);
        assert!(matches!(error, DownloadError::Execution { .. }));
        f.verify_calls(&path);
        f.no_persistence(&repo, &path);
        confirm_reader_available(&lease);
        lease.cleanup().unwrap();
        assert!(!path.exists());
        f.preserve();
    }
}
#[test]
fn actual_cancel_returns_before_cookie_copy_cleanup_and_preserves_source_and_sentinel() {
    let f = Fixture::new("cancel");
    let repo = f.repo();
    let mut lease = f.lease();
    let path = lease.path_for_tool().to_owned();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let ready = f.root.join("worker-ready");
    let trigger = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.exists() {
            assert!(
                Instant::now() < deadline,
                "synthetic worker readiness timed out"
            );
            thread::sleep(Duration::from_millis(10));
        }
        flag.store(true, Ordering::Release);
    });
    let error = repo
        .execute_with_session(
            "auth-fixture",
            &f.options,
            &f.tool,
            session(&cancel, &lease),
        )
        .err()
        .unwrap();
    trigger.join().unwrap();
    private_text(&format!("{error:?}"), &path);
    assert!(
        matches!(error,DownloadError::Execution {result,..} if result.outcome==ProcessOutcome::Cancelled)
    );
    f.no_persistence(&repo, &path);
    confirm_reader_available(&lease);
    lease.cleanup().unwrap();
    assert!(!path.exists());
    f.preserve();
}
#[test]
fn cleaned_or_replaced_lease_fails_before_any_external_tool_use() {
    for replaced in [false, true] {
        let f = Fixture::new("success");
        let repo = f.repo();
        let mut lease = f.lease();
        let path = lease.path_for_tool().to_owned();
        if replaced {
            fs::remove_file(&path).unwrap();
            fs::write(&path, b"unrelated replacement sentinel").unwrap();
        } else {
            lease.cleanup().unwrap();
        }
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(
            repo.execute_with_session(
                "auth-fixture",
                &f.options,
                &f.tool,
                session(&cancel, &lease)
            )
            .is_err()
        );
        assert!(!f.root.join("calls.jsonl").exists());
        f.no_persistence(&repo, &path);
        if replaced {
            assert!(lease.cleanup().is_err());
            assert_eq!(fs::read(&path).unwrap(), b"unrelated replacement sentinel");
        }
        f.preserve();
    }
}
#[test]
fn failed_resume_uses_a_fresh_session_copy_without_persisting_either_path() {
    let f = Fixture::new("download-failure");
    let repo = f.repo();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut first = f.lease();
    let old = first.path_for_tool().to_owned();
    assert!(
        repo.execute_with_session(
            "auth-fixture",
            &f.options,
            &f.tool,
            session(&cancel, &first)
        )
        .is_err()
    );
    confirm_reader_available(&first);
    first.cleanup().unwrap();
    fs::write(f.root.join("mode"), "success").unwrap();
    let mut second = f.lease();
    let new = second.path_for_tool().to_owned();
    assert_ne!(old, new);
    let result = repo
        .resume_with_session("auth-fixture", &f.tool, session(&cancel, &second))
        .unwrap();
    assert_eq!(result.job.state, JobState::Downloaded);
    assert_eq!(result.job.attempts.len(), 2);
    private_text(&result.execution.logs, &new);
    f.no_persistence(&repo, &old);
    f.no_persistence(&repo, &new);
    let calls = f.calls();
    let metadata: Vec<_> = calls.iter().filter(|x| x["stage"] == "metadata").collect();
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0]["cookie_path"], old.to_str().unwrap());
    assert_eq!(metadata[1]["cookie_path"], new.to_str().unwrap());
    confirm_reader_available(&second);
    second.cleanup().unwrap();
    f.preserve();
}

#[test]
fn malformed_sensitive_duration_fails_with_fixed_error_before_download() {
    for mode in ["duration-string", "duration-object"] {
        let f = Fixture::new(mode);
        let repo = f.repo();
        let mut lease = f.lease();
        let path = lease.path_for_tool().to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let error = repo
            .execute_with_session(
                "auth-fixture",
                &f.options,
                &f.tool,
                session(&cancel, &lease),
            )
            .err()
            .unwrap();
        private_text(&format!("{error:?}"), &path);
        assert!(matches!(
            error,
            DownloadError::Policy("authenticated tool evidence invalid; details omitted")
        ));
        let calls = f.calls();
        assert!(calls.iter().any(|call| call["stage"] == "metadata"));
        assert!(!calls.iter().any(|call| call["stage"] == "download"));
        assert_eq!(repo.job("auth-fixture").unwrap().state, JobState::Failed);
        f.verify_calls(&path);
        f.no_persistence(&repo, &path);
        confirm_reader_available(&lease);
        lease.cleanup().unwrap();
        assert!(!path.exists());
        f.preserve();
    }
}

#[derive(Clone, Copy, serde::Deserialize)]
struct OwnedIdentity {
    pid: i32,
    start_ticks: u64,
}
impl OwnedIdentity {
    fn matches(self) -> bool {
        fs::read_to_string(format!("/proc/{}/stat", self.pid))
            .ok()
            .and_then(|text| {
                text.rsplit_once(')')?
                    .1
                    .split_whitespace()
                    .nth(19)?
                    .parse::<u64>()
                    .ok()
            })
            == Some(self.start_ticks)
    }
    fn pin(self) -> PinnedFixture {
        assert!(self.pid > 1 && self.matches());
        // SAFETY: opens a new descriptor for the already verified fixture PID;
        // the second /proc comparison rejects a reuse race before any signal.
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, self.pid, 0) };
        assert!(fd >= 0, "owned fixture pidfd unavailable");
        let descriptor = unsafe { OwnedFd::from_raw_fd(fd as i32) };
        assert!(self.matches(), "fixture changed while pinning");
        PinnedFixture { descriptor }
    }
}
struct PinnedFixture {
    descriptor: OwnedFd,
}
impl PinnedFixture {
    fn exited(&self) -> bool {
        let mut event = libc::pollfd {
            fd: self.descriptor.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid pollfd, nonblocking; never signals a numeric PID.
        unsafe { libc::poll(&mut event, 1, 0) == 1 && event.revents & libc::POLLIN != 0 }
    }
    fn kill(&self) {
        if !self.exited() {
            // SAFETY: descriptor pins only the verified synthetic fixture.
            assert_eq!(
                unsafe {
                    libc::syscall(
                        libc::SYS_pidfd_send_signal,
                        self.descriptor.as_raw_fd(),
                        libc::SIGKILL,
                        std::ptr::null::<libc::siginfo_t>(),
                        0,
                    )
                },
                0
            );
        }
    }
}
impl Drop for PinnedFixture {
    fn drop(&mut self) {
        if !self.exited() {
            // SAFETY: best-effort cleanup targets only this owned pinned identity.
            unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    self.descriptor.as_raw_fd(),
                    libc::SIGKILL,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                );
            }
        }
    }
}

#[test]
fn guardian_sigkill_defers_cookie_cleanup_and_invalidates_lease_without_private_evidence() {
    let f = Fixture::new("guardian-kill");
    let repo = f.repo();
    let mut lease = f.lease();
    let path = lease.path_for_tool().to_owned();
    let directory = path.parent().unwrap().to_owned();
    let ready = f.root.join("owned-identities.json");
    let tool = f.tool.clone();
    let trigger = thread::spawn(move || {
        #[derive(serde::Deserialize)]
        struct Identities {
            worker: OwnedIdentity,
            guardian: OwnedIdentity,
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            assert!(
                Instant::now() < deadline,
                "owned download did not become ready"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let identities: Identities = serde_json::from_slice(&fs::read(ready).unwrap()).unwrap();
        let worker_command = fs::read(format!("/proc/{}/cmdline", identities.worker.pid)).unwrap();
        assert!(
            worker_command
                .split(|byte| *byte == 0)
                .any(|arg| arg == tool.as_os_str().as_encoded_bytes())
        );
        let worker_stat =
            fs::read_to_string(format!("/proc/{}/stat", identities.worker.pid)).unwrap();
        let parent: i32 = worker_stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(parent, identities.guardian.pid);
        let guardian = identities.guardian.pin();
        let worker = identities.worker.pin();
        guardian.kill();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !guardian.exited() {
            assert!(Instant::now() < deadline, "owned guardian did not exit");
            thread::sleep(Duration::from_millis(10));
        }
        // Direct-fixture guardian death may leave the worker alive. This is an
        // independent test cleanup, not adapter proof of confirmed termination.
        thread::sleep(Duration::from_millis(300));
        worker.kill();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !worker.exited() || identities.worker.matches() || identities.guardian.matches() {
            assert!(
                Instant::now() < deadline,
                "owned fixture was not killed/reaped"
            );
            thread::sleep(Duration::from_millis(10));
        }
    });
    let cancel = Arc::new(AtomicBool::new(false));
    let result = repo.execute_with_session(
        "auth-fixture",
        &f.options,
        &f.tool,
        session(&cancel, &lease),
    );
    trigger.join().unwrap();
    let error = result.err().unwrap();
    assert!(matches!(error, DownloadError::SessionCleanupDeferred));
    private_text(&format!("{error:?}"), &path);
    assert_ne!(
        repo.job("auth-fixture").unwrap().state,
        JobState::Downloaded
    );
    f.verify_calls(&path);
    f.no_persistence(&repo, &path);
    assert!(matches!(
        lease.begin_reader(),
        Err(CookieImportError::ReaderUnconfirmed)
    ));
    lease.defer_cleanup();
    assert!(lease.validate_for_tool().is_err());
    assert_eq!(fs::read(&path).unwrap(), COOKIES);
    for name in ["recovery.json", "owner.marker"] {
        assert!(directory.join(name).is_file());
    }
    for entry in fs::read_dir("/proc/self/fd").unwrap().flatten() {
        if let Ok(target) = fs::read_link(entry.path()) {
            assert!(
                !target.starts_with(&directory),
                "deferred lease retained owned FD"
            );
        }
    }
    drop(lease);
    assert!(path.exists() && directory.join("recovery.json").exists());
    f.preserve();
}

#[test]
fn actual_tool_terminal_without_confirm_then_worker_unwind_preserves_cookie_records() {
    let f = Fixture::new("metadata-failure");
    let repo = f.repo();
    let mut lease = f.lease();
    let path = lease.path_for_tool().to_owned();
    let directory = path.parent().unwrap().to_owned();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let cancel = Arc::new(AtomicBool::new(false));
            let version = process::run(
                &ProcessSpec {
                    #[cfg(windows)]
                    runtime_environment: None,
                    executable: f.tool.clone(),
                    args: vec!["--version".into()],
                    timeout: Duration::from_secs(5),
                    max_log_bytes: process::MAX_LOG_BYTES,
                },
                &cancel,
            )
            .unwrap();
            assert_eq!(version.exit_code, Some(0));
            let _reader = lease.begin_reader().unwrap();
            let spec = ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: f.tool.clone(),
                args: vec![
                    "--dump-single-json".into(),
                    "--cookies".into(),
                    path.clone().into_os_string(),
                    "--".into(),
                    f.options.url.as_str().into(),
                ],
                timeout: Duration::from_secs(5),
                max_log_bytes: process::MAX_LOG_BYTES,
            };
            // This returns only after the actual direct-fixture guardian/tool
            // cleanup. Deliberately panic before giving the reader its receipt.
            let execution = process::run(&spec, &cancel).unwrap();
            assert_eq!(execution.outcome, ProcessOutcome::Exited);
            assert_eq!(execution.exit_code, Some(7));
            // Raw synthetic tool evidence is never logged or persisted here;
            // authenticated omission is covered by execute_with_session tests.
            drop(execution);
            panic!("synthetic worker unwound before reader confirmation");
        });
        assert!(worker.join().is_err());
    });
    assert!(matches!(
        lease.begin_reader(),
        Err(CookieImportError::ReaderUnconfirmed)
    ));
    assert_eq!(lease.cleanup(), Err(CookieImportError::ReaderUnconfirmed));
    assert!(lease.validate_for_tool().is_err());
    assert_ne!(
        repo.job("auth-fixture").unwrap().state,
        JobState::Downloaded
    );
    f.verify_calls(&path);
    f.no_persistence(&repo, &path);
    for entry in fs::read_dir("/proc/self/fd").unwrap().flatten() {
        if let Ok(target) = fs::read_link(entry.path()) {
            assert!(
                !target.starts_with(&directory),
                "unwound lease retained owned FD"
            );
        }
    }
    drop(lease);
    assert_eq!(fs::read(&path).unwrap(), COOKIES);
    assert!(directory.join("recovery.json").is_file());
    assert!(directory.join("owner.marker").is_file());
    f.preserve();
}
