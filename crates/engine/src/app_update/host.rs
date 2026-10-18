//! Stable parent for one owned UI and one bounded update worker. No network work at idle.
use std::{io, path::Path};
#[cfg(any(windows, test))]
#[derive(Default)]
struct Generations {
    counter: u64,
    worker: Option<u64>,
    prepared: Option<u64>,
}
#[cfg(any(windows, test))]
impl Generations {
    fn begin(&mut self) -> io::Result<u64> {
        if self.worker.is_some() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let next = self
            .counter
            .checked_add(1)
            .ok_or(io::ErrorKind::InvalidData)?;
        self.counter = next;
        self.worker = Some(next);
        self.prepared = None;
        Ok(next)
    }
    fn complete(&mut self, generation: u64, success: bool) -> io::Result<()> {
        if self.worker != Some(generation) {
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.worker = None;
        self.prepared = success.then_some(generation);
        Ok(())
    }
    fn restart(&self, generation: u64) -> io::Result<()> {
        if self.worker.is_some() || self.prepared != Some(generation) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(())
    }
}
#[cfg(not(windows))]
pub fn run(
    _root: &Path,
    _keys: &[super::TrustedKey<'_>],
    _key_id: &str,
    _version: &str,
) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::{
        app_install::{FreshRoot, UpdatePreparationRequest},
        app_update::{
            Channel, Policy, StageRequest, TrustedKey,
            activation_journal::{CapturedImage, Journal, JournalBinding, SignedImageInput},
            bootstrap::{
                self, InstalledImage, OwnedUi, ParentEvent, PreparedImage, SelfImage, SpawnFailure,
                UiRequest,
            },
            control::FailureCode,
            download::{self, DownloadFailure, DownloadRequest, Phase, PreparedDownload, Progress},
        },
        file_identity, windows_private_security,
    };
    use ring::rand::{SecureRandom, SystemRandom};
    use std::{
        fs::{File, OpenOptions},
        os::windows::fs::OpenOptionsExt,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };
    const STARTUP: Duration = Duration::from_secs(120);
    const TRANSFER: Duration = Duration::from_secs(600);
    const CLEANUP: Duration = Duration::from_secs(10);
    const POLL: Duration = Duration::from_millis(100);
    fn error(kind: io::ErrorKind) -> io::Error {
        io::Error::new(kind, "app update owner could not complete the transition")
    }
    fn now() -> io::Result<u64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|v| v.as_secs())
            .map_err(|_| error(io::ErrorKind::InvalidData))
    }
    fn directory(path: &Path) -> io::Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if !file.metadata()?.is_dir() || file_identity::final_path(&file)? != path {
            return Err(error(io::ErrorKind::InvalidData));
        }
        Ok(file)
    }
    fn private_directory(path: &Path) -> io::Result<File> {
        match std::fs::symlink_metadata(path) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => (),
            Ok(_) => return Err(error(io::ErrorKind::InvalidData)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let fresh = FreshRoot::create_new(path)?;
                fresh.verify()?;
                let file = directory(&std::fs::canonicalize(path)?)?;
                if file_identity::from_file(&file)? != *fresh.identity() {
                    return Err(error(io::ErrorKind::InvalidData));
                }
                drop(fresh);
                windows_private_security::verify_acl(
                    &file,
                    &windows_private_security::current_sid()?,
                )?;
                return Ok(file);
            }
            Err(e) => return Err(e),
        };
        let file = directory(path)?;
        windows_private_security::verify_acl(&file, &windows_private_security::current_sid()?)?;
        Ok(file)
    }
    fn startup(mut child: OwnedUi) -> Result<OwnedUi, StartFailure> {
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + STARTUP;
        let result = child
            .wait_preflight(&cancel, deadline)
            .and_then(|_| child.commit(&cancel, deadline))
            .and_then(|_| child.wait_runtime_ready(&cancel, deadline));
        match result {
            Ok(()) => Ok(child),
            Err(cause) => {
                let terminal = child
                    .terminate_and_wait(Instant::now() + CLEANUP)
                    .is_ok_and(|v| v.job_terminal_confirmed);
                Err(StartFailure {
                    kind: cause.kind(),
                    terminal,
                })
            }
        }
    }
    struct StartFailure {
        kind: io::ErrorKind,
        terminal: bool,
    }
    fn spawned(result: Result<OwnedUi, SpawnFailure>) -> Result<OwnedUi, StartFailure> {
        match result {
            Ok(child) => startup(child),
            Err(mut failure) => {
                let kind = failure.kind();
                let terminal = if let Some(child) = failure.retained_child() {
                    child
                        .terminate_and_wait(Instant::now() + CLEANUP)
                        .is_ok_and(|v| v.job_terminal_confirmed)
                } else {
                    failure.terminal_confirmed()
                };
                Err(StartFailure { kind, terminal })
            }
        }
    }
    fn start_selected(
        image: &SelfImage,
        journal: Option<&Journal<'_>>,
        profile: &crate::file_identity::NativeFileIdentity,
    ) -> Result<OwnedUi, StartFailure> {
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + STARTUP;
        if let Some(evidence) = journal.and_then(|v| v.head().active()) {
            let installed =
                InstalledImage::pin_committed(&evidence, &cancel, deadline).map_err(|e| {
                    StartFailure {
                        kind: e.kind(),
                        terminal: true,
                    }
                })?;
            spawned(bootstrap::spawn_installed_ui(
                &installed,
                profile,
                bootstrap::PROFILE_EPOCH,
                &cancel,
                deadline,
            ))
        } else {
            spawned(bootstrap::spawn_ui(
                image,
                profile,
                bootstrap::PROFILE_EPOCH,
                &cancel,
                deadline,
            ))
        }
    }
    fn event(child: &mut OwnedUi, event: ParentEvent) -> io::Result<()> {
        child.send_event(
            &event,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(2),
        )
    }
    fn failed(child: &mut OwnedUi, code: FailureCode, generation: u64) -> io::Result<()> {
        event(
            child,
            ParentEvent::Failed {
                code: code as u16,
                generation,
            },
        )
    }
    struct OwnedKey {
        id: String,
        key: [u8; 32],
    }
    struct WorkerFailure {
        code: FailureCode,
        _download: Option<DownloadFailure>,
        _session: Option<Box<FreshRoot>>,
    }
    struct Worker {
        generation: u64,
        cancel: Arc<AtomicBool>,
        progress: Arc<Mutex<Option<Progress>>>,
        join: Option<JoinHandle<Result<PreparedDownload, WorkerFailure>>>,
    }
    fn setup_failure(kind: io::ErrorKind) -> WorkerFailure {
        WorkerFailure {
            code: if kind == io::ErrorKind::Interrupted {
                FailureCode::Cancelled
            } else {
                FailureCode::Disk
            },
            _download: None,
            _session: None,
        }
    }
    fn transfer_code(error: &DownloadFailure) -> FailureCode {
        #[cfg(feature = "app-update-fixture")]
        {
            let stage = error.stage_failure();
            eprintln!(
                "UPDATE_FIXTURE_DOWNLOAD kind={:?} creation={:?} publication={:?} cleanup={:?} package={} bundle={}",
                error.kind(),
                stage.map(|v| v.creation),
                stage.map(|v| v.publication),
                stage.map(|v| v.cleanup),
                error.package().is_some(),
                error.bundle().is_some()
            );
        }
        transfer_kind(error.kind())
    }
    pub(super) fn transfer_kind(error: &download::FailureKind) -> FailureCode {
        use crate::app_update::{Error as Validation, StageFailureKind, transport};
        use download::FailureKind;
        match error {
            FailureKind::Io(io::ErrorKind::Interrupted) => FailureCode::Cancelled,
            FailureKind::Transport(transport::Error::Cancelled) => FailureCode::Cancelled,
            FailureKind::Transport(transport::Error::Validation(Validation::Policy)) => {
                // Policy also covers replay/channel/version mismatch, not just expiry.
                FailureCode::Verification
            }
            FailureKind::Transport(transport::Error::Network | transport::Error::TimedOut) => {
                FailureCode::Network
            }
            FailureKind::Transport(_) => FailureCode::Verification,
            FailureKind::Stage(StageFailureKind::Io(io::ErrorKind::Interrupted)) => {
                FailureCode::Cancelled
            }
            FailureKind::Stage(StageFailureKind::Io(io::ErrorKind::TimedOut)) => {
                FailureCode::Network
            }
            FailureKind::Stage(StageFailureKind::Validation(Validation::Cancelled)) => {
                FailureCode::Cancelled
            }
            FailureKind::Stage(StageFailureKind::Validation(Validation::TimedOut)) => {
                FailureCode::Network
            }
            FailureKind::Stage(StageFailureKind::Validation(_)) => FailureCode::Verification,
            FailureKind::Stage(_) | FailureKind::Io(_) => FailureCode::Disk,
            FailureKind::Preparation(crate::app_install::UpdatePreparationError::Io(
                io::ErrorKind::Interrupted,
            ))
            | FailureKind::Preparation(crate::app_install::UpdatePreparationError::Bundle(
                crate::app_update::bundle::BundleError::Cancelled,
            )) => FailureCode::Cancelled,
            FailureKind::Preparation(_) => FailureCode::Prepare,
        }
    }
    struct WorkerRequest<'a> {
        root: &'a Path,
        keys: &'a [TrustedKey<'a>],
        key_id: &'a str,
        installed_version: String,
        sequence: u64,
        version: String,
        channel: Channel,
        generation: u64,
    }
    fn begin_worker<T, F>(request: WorkerRequest<'_>, factory: Arc<F>) -> io::Result<Worker>
    where
        T: crate::app_update::transport::Transport + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let WorkerRequest {
            root,
            keys,
            key_id,
            installed_version,
            sequence,
            version,
            channel,
            generation,
        } = request;
        let root = root.to_path_buf();
        let keys: Vec<_> = keys
            .iter()
            .map(|v| OwnedKey {
                id: v.id.into(),
                key: *v.public_key,
            })
            .collect();
        let key_id = key_id.to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = cancel.clone();
        let progress = Arc::new(Mutex::new(None));
        let latest = progress.clone();
        let join = thread::Builder::new()
            .name("app-update-download".into())
            .spawn(move || {
                let deadline = Instant::now() + TRANSFER;
                if signal.load(Ordering::Acquire) {
                    return Err(setup_failure(io::ErrorKind::Interrupted));
                }
                let parent_path = root.join("app-update-work");
                let parent =
                    private_directory(&parent_path).map_err(|e| setup_failure(e.kind()))?;
                let mut count = 0;
                for entry in std::fs::read_dir(&parent_path).map_err(|e| setup_failure(e.kind()))? {
                    let entry = entry.map_err(|e| setup_failure(e.kind()))?;
                    count += 1;
                    if count >= 32
                        || !entry
                            .file_type()
                            .map_err(|e| setup_failure(e.kind()))?
                            .is_dir()
                        || !entry
                            .file_name()
                            .to_str()
                            .is_some_and(|v| v.starts_with("session-"))
                    {
                        return Err(setup_failure(io::ErrorKind::InvalidData));
                    }
                }
                drop(parent);
                let mut nonce = [0; 16];
                SystemRandom::new()
                    .fill(&mut nonce)
                    .map_err(|_| setup_failure(io::ErrorKind::Other))?;
                let nonce: String = nonce.iter().map(|v| format!("{v:02x}")).collect();
                let session_path = parent_path.join(format!("session-{nonce}"));
                let session =
                    FreshRoot::create_new(&session_path).map_err(|e| setup_failure(e.kind()))?;
                let operation = (|| -> Result<PreparedDownload, WorkerFailure> {
                    let session_path = std::fs::canonicalize(&session_path)
                        .map_err(|e| setup_failure(e.kind()))?;
                    let stage_parent =
                        directory(&session_path).map_err(|e| setup_failure(e.kind()))?;
                    let pinned: Vec<_> = keys
                        .iter()
                        .map(|v| TrustedKey {
                            id: &v.id,
                            public_key: &v.key,
                        })
                        .collect();
                    let extraction = session_path.join("extracted");
                    let bundle = session_path.join("bundle");
                    let journal = session_path.join("installation-journal");
                    let request = DownloadRequest {
                        version: &version,
                        key_id_hint: &key_id,
                        keys: &pinned,
                        policy: Policy {
                            installed_version: &installed_version,
                            installed_sequence: sequence,
                            channel,
                            now_unix: now().map_err(|e| setup_failure(e.kind()))?,
                        },
                        stage: StageRequest {
                            parent: &stage_parent,
                            parent_path: &session_path,
                            nonce: &nonce,
                        },
                        prepare: UpdatePreparationRequest {
                            extraction_root: &extraction,
                            bundle_root: &bundle,
                            journal_root: &journal,
                        },
                    };
                    download::prepare_download(
                        &mut factory(),
                        &request,
                        signal,
                        deadline,
                        &mut |sample| {
                            if let Ok(mut slot) = latest.lock() {
                                *slot = Some(sample);
                            }
                        },
                    )
                    .map_err(|failure| WorkerFailure {
                        code: transfer_code(&failure),
                        _download: Some(failure),
                        _session: None,
                    })
                })();
                match operation {
                    Ok(value) => {
                        drop(session);
                        Ok(value)
                    }
                    Err(mut failure) => {
                        failure._session = Some(Box::new(session));
                        Err(failure)
                    }
                }
            })?;
        Ok(Worker {
            generation,
            cancel,
            progress,
            join: Some(join),
        })
    }
    impl Worker {
        fn finished(&self) -> bool {
            self.join.as_ref().is_none_or(|v| v.is_finished())
        }
    }
    impl Drop for Worker {
        fn drop(&mut self) {
            self.cancel.store(true, Ordering::Release);
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }
    fn finish_worker(mut worker: Worker) -> Result<PreparedDownload, WorkerFailure> {
        worker
            .join
            .take()
            .expect("owned worker join")
            .join()
            .unwrap_or(Err(WorkerFailure {
                code: FailureCode::Worker,
                _download: None,
                _session: None,
            }))
    }
    fn wait_terminal(child: &mut OwnedUi) -> io::Result<()> {
        // A live UI has no runtime deadline. Its own CloseWait drains owned workers.
        // poll_exit bounds cleanup only after the original leader has exited.
        loop {
            if let Some(exit) = child.poll_exit()? {
                if !exit.job_terminal_confirmed {
                    return Err(error(io::ErrorKind::Other));
                }
                return Ok(());
            }
            thread::park_timeout(POLL);
        }
    }
    fn progress_phase(value: Phase) -> u8 {
        match value {
            Phase::Verifying => 0,
            Phase::Downloading => 1,
            Phase::Staging => 2,
            Phase::Preparing => 3,
            Phase::Prepared => 4,
        }
    }
    fn record_and_revert(writer: &mut Journal<'_>, sequence: u64) -> io::Result<()> {
        let cancel = AtomicBool::new(false);
        let previous = writer.head().clone();
        let rollback = previous.active().is_some_and(|v| v.sequence() == sequence);
        let failed = writer
            .record_failed(&previous, sequence, &cancel, Instant::now() + STARTUP)
            .map_err(io::Error::other)?;
        if rollback {
            writer
                .rollback(&failed, &cancel, Instant::now() + STARTUP)
                .map_err(io::Error::other)?;
        }
        Ok(())
    }
    pub fn run(
        root: &Path,
        keys: &[TrustedKey<'_>],
        key_id: &str,
        version: &str,
    ) -> io::Result<()> {
        run_with_transport(root, keys, key_id, version, || {
            crate::app_update::transport::OfficialHttps
        })
    }
    /// Explicit library/test seam. Production run always supplies the fixed official HTTPS adapter.
    /// There is no environment, URL, command-line or candidate-supplied transport/key override.
    pub fn run_with_transport<T, F>(
        root: &Path,
        keys: &[TrustedKey<'_>],
        key_id: &str,
        version: &str,
        factory: F,
    ) -> io::Result<()>
    where
        T: crate::app_update::transport::Transport + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let factory = Arc::new(factory);
        let root = std::fs::canonicalize(root)?;
        let profile_root = directory(&root)?;
        let profile_id = file_identity::from_file(&profile_root)?;
        let lock_path = root.join("updater-host.lock");
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&lock_path)?;
        file_identity::ensure_regular_under(&profile_root, &lease)?;
        if file_identity::final_path(&lease)? != lock_path || lease.metadata()?.len() != 0 {
            return Err(error(io::ErrorKind::InvalidData));
        }
        fs2::FileExt::try_lock_exclusive(&lease)?;
        let cancel = AtomicBool::new(false);
        let image = SelfImage::pin_current(&cancel, Instant::now() + STARTUP)?;
        let bootstrap = file_identity::open_regular_no_reparse(&std::env::current_exe()?)?;
        if file_identity::from_file(&bootstrap)? != *image.identity() {
            return Err(error(io::ErrorKind::InvalidData));
        }
        let mut visited = 0usize;
        let mut namespaces = 0usize;
        for entry in std::fs::read_dir(&root)? {
            visited += 1;
            if visited > 4096 {
                return Err(error(io::ErrorKind::InvalidData));
            }
            let entry = entry?;
            if entry
                .file_name()
                .to_str()
                .is_some_and(|v| v.starts_with("app-update-selection-"))
            {
                namespaces += 1;
                if namespaces > 32 {
                    return Err(error(io::ErrorKind::InvalidData));
                }
            }
        }
        use sha2::{Digest, Sha256};
        let namespace_bytes =
            serde_json::to_vec(&(image.identity(), version)).map_err(io::Error::other)?;
        let suffix = format!("{:x}", Sha256::digest(namespace_bytes));
        let journal_path = root.join(format!("app-update-selection-{suffix}"));
        if namespaces >= 32
            && std::fs::symlink_metadata(&journal_path)
                .is_err_and(|e| e.kind() == io::ErrorKind::NotFound)
        {
            return Err(error(io::ErrorKind::InvalidData));
        }
        let journal_root = if keys.is_empty() {
            match std::fs::symlink_metadata(&journal_path) {
                Ok(m) => {
                    if !m.is_dir() || m.file_type().is_symlink() {
                        return Err(error(io::ErrorKind::InvalidData));
                    }
                    let file = directory(&journal_path)?;
                    windows_private_security::verify_acl(
                        &file,
                        &windows_private_security::current_sid()?,
                    )?;
                    let mut count = 0;
                    for entry in std::fs::read_dir(&journal_path)? {
                        count += 1;
                        if count > 1 || entry?.file_name() != "writer.lock" {
                            return Err(error(io::ErrorKind::PermissionDenied));
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            }
            None
        } else {
            Some(private_directory(&journal_path)?)
        };
        let mut journal = journal_root
            .as_ref()
            .map(|file| {
                Journal::open(
                    JournalBinding {
                        root: file,
                        root_path: &journal_path,
                        bootstrap: &bootstrap,
                        bootstrap_version: version,
                        profile_epoch: bootstrap::PROFILE_EPOCH,
                    },
                    keys,
                    &cancel,
                    Instant::now() + STARTUP,
                )
            })
            .transpose()
            .map_err(io::Error::other)?;
        let mut recovered = false;
        let mut protocol_lost = false;
        let mut child = match start_selected(&image, journal.as_ref(), &profile_id) {
            Ok(child) => child,
            Err(failure) => {
                if !failure.terminal {
                    return Err(error(failure.kind));
                }
                let Some(writer) = journal.as_mut() else {
                    return Err(error(failure.kind));
                };
                let Some(active) = writer.head().active() else {
                    return Err(error(failure.kind));
                };
                record_and_revert(writer, active.sequence())?;
                recovered = true;
                let mut prior =
                    start_selected(&image, Some(writer), &profile_id).map_err(|e| error(e.kind))?;
                if failed(&mut prior, FailureCode::Recovered, 0).is_err() {
                    protocol_lost = true;
                }
                prior
            }
        };
        let mut generations = Generations::default();
        let mut worker: Option<Worker> = None;
        let mut prepared: Option<PreparedDownload> = None;
        let mut retained_failure: Option<WorkerFailure> = None;
        let mut busy_rejection = false;
        loop {
            if let Some(exit) = child.poll_exit()? {
                if let Some(pending) = worker.take() {
                    pending.cancel.store(true, Ordering::Release);
                    let _retained = finish_worker(pending);
                }
                prepared = None;
                if !exit.job_terminal_confirmed {
                    return Err(error(io::ErrorKind::Other));
                }
                if exit.code != 0
                    && !recovered
                    && let Some(writer) = journal.as_mut()
                    && let Some(active) = writer.head().active()
                {
                    record_and_revert(writer, active.sequence())?;
                    drop(child);
                    child = start_selected(&image, Some(writer), &profile_id)
                        .map_err(|e| error(e.kind))?;
                    recovered = true;
                    protocol_lost = failed(&mut child, FailureCode::Recovered, 0).is_err();
                    continue;
                }
                return if exit.code == 0 {
                    Ok(())
                } else {
                    Err(error(io::ErrorKind::Other))
                };
            }
            if worker.as_ref().is_some_and(Worker::finished) {
                let completed = worker.take().expect("single worker");
                let generation = completed.generation;
                let outcome = finish_worker(completed);
                if busy_rejection {
                    drop(outcome);
                    generations.complete(generation, false)?;
                    let rejection = generations.begin()?;
                    generations.complete(rejection, false)?;
                    busy_rejection = false;
                    prepared = None;
                    if !protocol_lost && failed(&mut child, FailureCode::Busy, rejection).is_err() {
                        protocol_lost = true;
                    }
                } else {
                    match outcome {
                        Ok(value) => {
                            if protocol_lost {
                                generations.complete(generation, false)?;
                                drop(value);
                            } else {
                                let version = value.candidate().version().to_owned();
                                generations.complete(generation, true)?;
                                prepared = Some(value);
                                if event(
                                    &mut child,
                                    ParentEvent::Prepared {
                                        version,
                                        generation,
                                    },
                                )
                                .is_err()
                                {
                                    protocol_lost = true;
                                    prepared = None;
                                    generations.prepared = None;
                                }
                            }
                        }
                        Err(failure) => {
                            generations.complete(generation, false)?;
                            let code = failure.code;
                            retained_failure = Some(failure);
                            if !protocol_lost && failed(&mut child, code, generation).is_err() {
                                protocol_lost = true;
                            }
                        }
                    }
                }
            }
            if protocol_lost {
                if let Some(pending) = worker.as_ref() {
                    pending.cancel.store(true, Ordering::Release);
                }
                thread::park_timeout(POLL);
                continue;
            }
            if !busy_rejection
                && let Some(pending) = worker.as_ref()
                && let Some(sample) = pending
                    .progress
                    .lock()
                    .map_err(|_| error(io::ErrorKind::Other))?
                    .take()
                && event(
                    &mut child,
                    ParentEvent::Progress {
                        phase: progress_phase(sample.phase),
                        received: sample.received,
                        total: sample.total,
                        generation: pending.generation,
                    },
                )
                .is_err()
            {
                protocol_lost = true;
                pending.cancel.store(true, Ordering::Release);
                continue;
            }
            let request = match child.try_request() {
                Ok(request) => request,
                Err(_) => {
                    protocol_lost = true;
                    if let Some(pending) = worker.as_ref() {
                        pending.cancel.store(true, Ordering::Release);
                    }
                    continue;
                }
            };
            if let Some(request) = request {
                match request {
                    UiRequest::Prepare {
                        version: requested,
                        channel,
                    } => {
                        if keys.is_empty() {
                            let generation = generations.begin()?;
                            generations.complete(generation, false)?;
                            if failed(&mut child, FailureCode::TrustUnavailable, generation)
                                .is_err()
                            {
                                protocol_lost = true;
                            }
                            continue;
                        }
                        if let Some(pending) = worker.as_ref() {
                            pending.cancel.store(true, Ordering::Release);
                            busy_rejection = true;
                            prepared = None;
                            generations.prepared = None;
                            continue;
                        }
                        let generation = generations.begin()?;
                        prepared = None;
                        retained_failure = None;
                        let installed = journal
                            .as_ref()
                            .and_then(|v| v.head().active())
                            .map_or_else(|| version.to_owned(), |v| v.version().to_owned());
                        let sequence = journal.as_ref().map_or(0, |v| v.head().highest_sequence());
                        match begin_worker(
                            WorkerRequest {
                                root: &root,
                                keys,
                                key_id,
                                installed_version: installed,
                                sequence,
                                version: requested,
                                channel,
                                generation,
                            },
                            factory.clone(),
                        ) {
                            Ok(value) => worker = Some(value),
                            Err(_) => {
                                generations.complete(generation, false)?;
                                if failed(&mut child, FailureCode::Worker, generation).is_err() {
                                    protocol_lost = true;
                                }
                            }
                        }
                    }
                    UiRequest::CancelPrepare => {
                        busy_rejection = false;
                        if let Some(pending) = &worker {
                            pending.cancel.store(true, Ordering::Release);
                        } else {
                            prepared = None;
                            generations.prepared = None;
                            if failed(&mut child, FailureCode::Cancelled, generations.counter)
                                .is_err()
                            {
                                protocol_lost = true;
                            }
                        }
                    }
                    UiRequest::RestartPrepared { generation } => {
                        if generations.restart(generation).is_err() {
                            if failed(&mut child, FailureCode::RestartRefused, generation).is_err()
                            {
                                protocol_lost = true;
                            }
                            continue;
                        }
                        let Some(mut download) = prepared.take() else {
                            if failed(&mut child, FailureCode::RestartRefused, generation).is_err()
                            {
                                protocol_lost = true;
                            }
                            continue;
                        };
                        let Some(writer) = journal.as_mut() else {
                            if failed(&mut child, FailureCode::TrustUnavailable, generation)
                                .is_err()
                            {
                                protocol_lost = true;
                            }
                            continue;
                        };
                        let deadline = Instant::now() + STARTUP;
                        let captured = (|| -> io::Result<(CapturedImage, PreparedImage)> {
                            download.verify_current(&cancel, deadline)?;
                            let timestamp = now()?;
                            let capture = writer
                                .capture(
                                    SignedImageInput {
                                        candidate: download.candidate(),
                                        prepared: download.bundle(),
                                        verified_at_unix: timestamp,
                                    },
                                    timestamp,
                                    &cancel,
                                    deadline,
                                )
                                .map_err(io::Error::other)?;
                            let pin =
                                PreparedImage::pin_prepared(download.bundle(), &cancel, deadline)?;
                            Ok((capture, pin))
                        })();
                        let sequence = download.candidate().sequence();
                        drop(download);
                        generations.prepared = None;
                        let (captured, pin) = match captured {
                            Ok(v) => v,
                            Err(_) => {
                                if failed(&mut child, FailureCode::Verification, generation)
                                    .is_err()
                                {
                                    protocol_lost = true;
                                }
                                continue;
                            }
                        };
                        let expected = writer.head().clone();
                        if event(&mut child, ParentEvent::RestartAccepted { generation }).is_err() {
                            drop(pin);
                            protocol_lost = true;
                            continue;
                        }
                        // Original owner and its entire Job must be terminal before B can execute.
                        wait_terminal(&mut child)?;
                        drop(child);
                        let result = spawned(bootstrap::spawn_prepared_ui(
                            &pin,
                            &profile_id,
                            bootstrap::PROFILE_EPOCH,
                            &cancel,
                            Instant::now() + STARTUP,
                        ));
                        drop(pin);
                        match result {
                            Ok(mut next) => match writer.commit(
                                &expected,
                                &captured,
                                now()?,
                                &cancel,
                                Instant::now() + STARTUP,
                            ) {
                                Ok(_) => {
                                    child = next;
                                    recovered = false;
                                }
                                Err(failure) => {
                                    let terminal = next
                                        .terminate_and_wait(Instant::now() + CLEANUP)?
                                        .job_terminal_confirmed;
                                    if !terminal || failure.mutation_may_have_occurred() {
                                        return Err(error(io::ErrorKind::InvalidData));
                                    }
                                    record_and_revert(writer, sequence)?;
                                    child = start_selected(&image, Some(writer), &profile_id)
                                        .map_err(|e| error(e.kind))?;
                                    recovered = true;
                                    if failed(&mut child, FailureCode::Recovered, 0).is_err() {
                                        protocol_lost = true;
                                    }
                                }
                            },
                            Err(failure) => {
                                if !failure.terminal {
                                    return Err(error(failure.kind));
                                }
                                record_and_revert(writer, sequence)?;
                                child = start_selected(&image, Some(writer), &profile_id)
                                    .map_err(|e| error(e.kind))?;
                                recovered = true;
                                if failed(&mut child, FailureCode::Recovered, 0).is_err() {
                                    protocol_lost = true;
                                }
                            }
                        }
                    }
                }
            }
            // Failure capabilities live until a later explicit request. Drop never deletes them.
            let _ = &retained_failure;
            thread::park_timeout(POLL);
        }
    }
}
#[cfg(windows)]
pub use windows::{run, run_with_transport};
#[cfg(test)]
mod tests;
