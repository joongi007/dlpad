//! Direct yt-dlp executable installs. Archive extraction is deliberately a separate contract.
use super::*;
use crate::process::{self, ProcessOutcome, ProcessSpec};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

pub const MAX_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;
const INSTALL_TIMEOUT: Duration = Duration::from_secs(120);
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) trait DownloadTransport {
    fn open(&mut self, url: &str, timeout: Duration) -> Result<Box<dyn Read>, ToolError>;
}
pub(super) struct TransferLock(File);
impl TransferLock {
    #[cfg(windows)]
    pub(super) fn original_file(&self) -> &File {
        &self.0
    }
}
impl Drop for TransferLock {
    fn drop(&mut self) {
        // An inherited/duplicated descriptor can outlive this installer; close alone
        // leaves flock held by that descriptor's shared open-file description.
        let _ = fs2::FileExt::unlock(&self.0);
    }
}
pub(super) struct HttpsTransport;
impl DownloadTransport for HttpsTransport {
    fn open(&mut self, url: &str, timeout: Duration) -> Result<Box<dyn Read>, ToolError> {
        let config = ureq::Agent::config_builder()
            .https_only(true)
            .proxy(None)
            .max_redirects(5)
            .max_response_header_size(64 * 1024)
            .input_buffer_size(64 * 1024)
            .output_buffer_size(16 * 1024)
            .max_idle_connections(0)
            .timeout_global(Some(timeout))
            .timeout_resolve(Some(Duration::from_secs(10).min(timeout)))
            .timeout_connect(Some(Duration::from_secs(10).min(timeout)))
            .timeout_recv_response(Some(Duration::from_secs(10).min(timeout)))
            .build();
        let response = ureq::Agent::new_with_config(config)
            .get(url)
            .call()
            .map_err(|_| ToolError::Http)?;
        if response.status().as_u16() != 200 {
            return Err(ToolError::Http);
        }
        Ok(Box::new(response.into_body().into_reader()))
    }
}

pub(super) fn source_policy(url: &str) -> Result<(), ToolError> {
    if url.len() > 8192
        || url.contains(['?', '#'])
        || url.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(ToolError::InvalidManifest);
    }
    let uri: ureq::http::Uri = url.parse().map_err(|_| ToolError::InvalidManifest)?;
    if uri.scheme_str() != Some("https")
        || uri
            .authority()
            .is_none_or(|a| a.as_str().contains(['@', '%']) || a.host().is_empty())
    {
        return Err(ToolError::InvalidManifest);
    }
    Ok(())
}
pub(super) fn checkpoint(
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Duration, ToolError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ToolError::Cancelled);
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or(ToolError::TimedOut)
}

/// Tracks only newly created files by open-handle identity. Never recursively deletes.
pub(super) struct OwnedDirectory {
    pub(super) path: PathBuf,
    pub(super) identity: fs::Metadata,
    directory_pin: File,
    files: Vec<(PathBuf, File)>,
    pub(super) keep: bool,
}
impl OwnedDirectory {
    pub(super) fn create(path: PathBuf) -> Result<Self, ToolError> {
        safe_components(&path)?;
        #[cfg(unix)]
        let mut builder = fs::DirBuilder::new();
        #[cfg(not(unix))]
        let builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path)?;
        let identity = fs::symlink_metadata(&path)?;
        let directory_pin = super::native_files::open_directory(&path)?;
        Ok(Self {
            path,
            identity,
            directory_pin,
            files: Vec::new(),
            keep: false,
        })
    }
    pub(super) fn check(&self) -> Result<(), ToolError> {
        safe_components(&self.path)?;
        let current = fs::symlink_metadata(&self.path)?;
        if !super::native_files::same_directory(&self.path, &self.directory_pin) {
            return Err(ToolError::UnsafePath);
        }
        if !current.is_dir() {
            return Err(ToolError::UnsafePath);
        }
        Ok(())
    }
    pub(super) fn file(&mut self, name: &str) -> Result<File, ToolError> {
        self.check()?;
        let path = self.path.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        self.files.push((path, file.try_clone()?));
        Ok(file)
    }
    pub(super) fn seal_executable(&mut self, name: &str) -> Result<(), ToolError> {
        self.check()?;
        let path = self.path.join(name);
        let (_, owned) = self
            .files
            .iter_mut()
            .find(|(p, _)| *p == path)
            .ok_or(ToolError::UnsafePath)?;
        let readonly = File::open(&path)?;
        if !super::native_files::same_file(&path, owned) {
            return Err(ToolError::UnsafePath);
        }
        // execve rejects a file while any write handle remains open (ETXTBSY).
        *owned = readonly;
        Ok(())
    }
    pub(super) fn link(&mut self, source: &Path, name: &str) -> Result<(), ToolError> {
        self.check()?;
        safe_components(source)?;
        let path = self.path.join(name);
        fs::hard_link(source, &path)?; // create-only; no rename-overwrite window
        self.files.push((path.clone(), File::open(path)?));
        Ok(())
    }
}
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if self.keep {
            return;
        }
        if super::native_files::same_directory(&self.path, &self.directory_pin) {
            // Drop each original file pin before removing the directory on Windows.
            for (path, file) in std::mem::take(&mut self.files) {
                if fs::symlink_metadata(&path).is_ok_and(|m| m.is_file())
                    && super::native_files::same_file(&path, &file)
                {
                    let _ = fs::remove_file(&path);
                }
            }
            let _ = fs::remove_dir(&self.path);
        }
    }
}

impl ToolManager {
    pub(super) fn acquire_install_lock(&self) -> Result<TransferLock, ToolError> {
        self.verify_root()?;
        let lock_path = self.root.join(".install-transfer-lock");
        safe_components(&lock_path)?;
        let transfer_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;
        if !transfer_lock.metadata()?.is_file() {
            return Err(ToolError::UnsafePath);
        }
        fs2::FileExt::try_lock_exclusive(&transfer_lock)?;
        if super::execution_marker::pending(&self.root)? {
            return Err(ToolError::ExecutionPending);
        }
        Ok(TransferLock(transfer_lock))
    }
}

impl ToolManager {
    /// Worker-only HTTPS download of a pinned direct yt-dlp executable.
    /// The 120-second budget covers transfer, version checking and pre-activation work.
    /// Blocking HTTP cancellation is checked between reads and can wait for that budget.
    /// Archives and other tool version conventions are not accepted by this API.
    pub fn download_and_install(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledTool, ToolError> {
        self.install_download(
            manifest,
            expected_bytes,
            cancel,
            &mut HttpsTransport,
            INSTALL_TIMEOUT,
        )
    }
    /// Install verified bytes without changing the user's active version.
    pub fn download_and_install_inactive(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledTool, ToolError> {
        let deadline = Instant::now()
            .checked_add(INSTALL_TIMEOUT)
            .ok_or(ToolError::TimedOut)?;
        self.install_download_until_with_activation(
            manifest,
            expected_bytes,
            cancel,
            &mut HttpsTransport,
            deadline,
            false,
        )
    }
    /// Inactive installation bounded by the caller's absolute deadline.
    pub fn download_and_install_inactive_until(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<InstalledTool, ToolError> {
        let limit = Instant::now()
            .checked_add(INSTALL_TIMEOUT)
            .ok_or(ToolError::TimedOut)?;
        self.install_download_until_with_activation(
            manifest,
            expected_bytes,
            cancel,
            &mut HttpsTransport,
            deadline.min(limit),
            false,
        )
    }
    /// Same pinned transfer policy, bounded by the caller's absolute suite deadline.
    pub fn download_and_install_until(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<InstalledTool, ToolError> {
        let limit = Instant::now()
            .checked_add(INSTALL_TIMEOUT)
            .ok_or(ToolError::TimedOut)?;
        self.install_download_until(
            manifest,
            expected_bytes,
            cancel,
            &mut HttpsTransport,
            deadline.min(limit),
        )
    }
    fn install_download(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
        transport: &mut dyn DownloadTransport,
        timeout: Duration,
    ) -> Result<InstalledTool, ToolError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or(ToolError::TimedOut)?;
        self.install_download_until(manifest, expected_bytes, cancel, transport, deadline)
    }
    fn install_download_until(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
        transport: &mut dyn DownloadTransport,
        deadline: Instant,
    ) -> Result<InstalledTool, ToolError> {
        self.install_download_until_with_activation(
            manifest,
            expected_bytes,
            cancel,
            transport,
            deadline,
            true,
        )
    }
    fn install_download_until_with_activation(
        &mut self,
        manifest: InstallManifest,
        expected_bytes: u64,
        cancel: &Arc<AtomicBool>,
        transport: &mut dyn DownloadTransport,
        deadline: Instant,
        activate: bool,
    ) -> Result<InstalledTool, ToolError> {
        let destination = self.install_dir(&manifest)?;
        source_policy(&manifest.source_url)?;
        if manifest.tool != "yt-dlp" {
            return Err(ToolError::InvalidManifest);
        }
        if expected_bytes == 0 || expected_bytes > MAX_DOWNLOAD_BYTES {
            return Err(ToolError::DownloadLimit);
        }
        checkpoint(cancel, deadline)?;
        // Serialize capacity reservation/transfer per owned root, including other managers.
        let _transfer_lock = self.acquire_install_lock()?;
        self.verify_install_inventory_capacity()?;
        self.reserve_staging(
            expected_bytes
                .checked_add(MANIFEST_LIMIT)
                .ok_or(ToolError::DownloadLimit)?,
        )?;
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists).into());
        }
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut staging = OwnedDirectory::create(
            self.root
                .join(format!(".staging-{}-{sequence}", std::process::id())),
        )?;
        let executable = staging.path.join(&manifest.executable);
        let mut output = staging.file(&manifest.executable)?;
        self.record_staging(
            &staging.path,
            &staging.identity,
            &manifest.executable,
            vec![super::staging_recovery::file_evidence(
                &manifest.executable,
                &output,
                0,
                format!("{:x}", Sha256::digest([])),
            )?],
        )?;
        let mut input = transport.open(&manifest.source_url, checkpoint(cancel, deadline)?)?;
        transfer_to_owned_file(
            &mut *input,
            &mut output,
            expected_bytes,
            &manifest.sha256,
            cancel,
            deadline,
            |file, bytes, digest| {
                self.record_staging(
                    &staging.path,
                    &staging.identity,
                    &manifest.executable,
                    vec![super::staging_recovery::file_evidence(
                        &manifest.executable,
                        file,
                        bytes,
                        digest,
                    )?],
                )
            },
        )?;
        let bytes = expected_bytes; // transfer helper proved exact length.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            output.set_permissions(fs::Permissions::from_mode(0o700))?;
        }
        drop(output);
        drop(input);
        staging.seal_executable(&manifest.executable)?;
        staging.check()?;
        let result = process::run_metadata(
            &ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: executable.clone(),
                args: vec![
                    "--ignore-config".into(),
                    "--no-plugin-dirs".into(),
                    "--version".into(),
                ],
                timeout: Duration::from_secs(10).min(checkpoint(cancel, deadline)?),
                max_log_bytes: 4096,
            },
            cancel,
        )
        .map_err(|error| {
            if error.kind() == io::ErrorKind::TimedOut {
                ToolError::TimedOut
            } else {
                ToolError::Io(error)
            }
        })?;
        checkpoint(cancel, deadline)?;
        if result.execution.outcome == ProcessOutcome::Cancelled {
            return Err(ToolError::Cancelled);
        }
        if result.execution.outcome == ProcessOutcome::TimedOut {
            return Err(ToolError::TimedOut);
        }
        if result.execution.outcome != ProcessOutcome::Exited
            || result.execution.exit_code != Some(0)
            || super::selection::verified_ytdlp_version(&result.stdout, &manifest.version).is_err()
        {
            return Err(ToolError::VersionMismatch);
        }
        // A verified program can modify its own file while reporting its version.
        // Recheck both handle identity and the pinned bytes before publication.
        staging.seal_executable(&manifest.executable)?;
        let mut verified = File::open(&executable)?;
        let mut final_hash = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        let mut final_bytes = 0u64;
        loop {
            checkpoint(cancel, deadline)?;
            let limit = buffer
                .len()
                .min(expected_bytes.saturating_sub(final_bytes).saturating_add(1) as usize);
            let n = verified.read(&mut buffer[..limit])?;
            if n == 0 {
                break;
            }
            final_bytes += n as u64;
            if final_bytes > expected_bytes {
                return Err(ToolError::DownloadLimit);
            }
            final_hash.update(&buffer[..n]);
        }
        if final_bytes != expected_bytes {
            return Err(ToolError::UnexpectedSize);
        }
        if format!("{:x}", final_hash.finalize()) != manifest.sha256.to_ascii_lowercase() {
            return Err(ToolError::HashMismatch);
        }
        let record = serde_json::to_vec(&manifest)?;
        let mut record_file = staging.file("manifest.json")?;
        record_file.write_all(&record)?;
        record_file.sync_all()?;
        self.record_staging(
            &staging.path,
            &staging.identity,
            &manifest.executable,
            vec![
                super::staging_recovery::file_evidence(
                    &manifest.executable,
                    &verified,
                    bytes,
                    manifest.sha256.to_ascii_lowercase(),
                )?,
                super::staging_recovery::file_evidence(
                    "manifest.json",
                    &record_file,
                    record.len() as u64,
                    format!("{:x}", Sha256::digest(&record)),
                )?,
            ],
        )?;
        drop(record_file);
        self.verify_root()?;
        staging.check()?;
        let parent = destination.parent().ok_or(ToolError::UnsafePath)?;
        safe_components(parent)?;
        fs::create_dir_all(parent)?;
        safe_components(parent)?;
        let mut published = OwnedDirectory::create(destination.clone())?;
        published.link(&executable, &manifest.executable)?;
        published.link(&staging.path.join("manifest.json"), "manifest.json")?;
        #[cfg(unix)]
        {
            File::open(&destination)?.sync_all()?;
            File::open(parent)?.sync_all()?;
        }
        checkpoint(cancel, deadline)?;
        self.verify_root()?;
        published.check()?;
        if activate {
            let tx = self.db.transaction()?;
            tx.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![manifest.tool,manifest.version,manifest.target])?;
            checkpoint(cancel, deadline)?;
            tx.commit()?;
        } else {
            checkpoint(cancel, deadline)?;
        }
        published.keep = true;
        // Let in-session handle-identity cleanup run before removing persisted evidence.
        let stage_path = staging.path.clone();
        drop(staging);
        if !stage_path.exists() {
            let _ = self.forget_staging(&stage_path); // Committed activation must remain a success.
        }
        Ok(InstalledTool {
            manifest,
            executable: destination.join(executable.file_name().ok_or(ToolError::UnsafePath)?),
            bytes,
        })
    }
}

/// Shared bounded streaming transfer; caller retains source/output ownership.
pub(super) fn transfer_to_owned_file(
    input: &mut dyn Read,
    output: &mut File,
    expected_bytes: u64,
    expected_digest: &str,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    mut journal: impl FnMut(&File, u64, String) -> Result<(), ToolError>,
) -> Result<(), ToolError> {
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut journaled = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        checkpoint(cancel, deadline)?;
        let remaining = expected_bytes.saturating_sub(bytes);
        let limit = buffer.len().min(remaining.saturating_add(1) as usize);
        let n = input.read(&mut buffer[..limit])?;
        checkpoint(cancel, deadline)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if bytes > expected_bytes {
            return Err(ToolError::DownloadLimit);
        }
        hash.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
        if bytes.saturating_sub(journaled) >= 1024 * 1024 {
            output.sync_all()?;
            journal(output, bytes, format!("{:x}", hash.clone().finalize()))?;
            journaled = bytes;
        }
    }
    if bytes != expected_bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", hash.finalize()) != expected_digest.to_ascii_lowercase() {
        return Err(ToolError::HashMismatch);
    }
    output.sync_all()?;
    journal(output, bytes, expected_digest.to_ascii_lowercase())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn manifest(bytes: &[u8], version: &str) -> InstallManifest {
        InstallManifest {
            tool: "yt-dlp".into(),
            version: version.into(),
            target: "synthetic".into(),
            source_url: "https://supply.invalid/releases/fixture".into(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            license: "synthetic fixture only".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp".into(),
        }
    }
    struct MemoryTransport(Vec<u8>);
    impl DownloadTransport for MemoryTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            Ok(Box::new(Cursor::new(self.0.clone())))
        }
    }
    #[test]
    fn original_directory_pin_preserves_replacement_directory() {
        let root = tempfile::tempdir().unwrap();
        let stage = root.path().join(".staging-original-pin");
        let owned = OwnedDirectory::create(stage.clone()).unwrap();
        let moved = root.path().join("moved");
        // Isolate directory identity from Windows' prohibition on renaming a
        // directory with an open child. Child identity has a separate test.
        fs::rename(&stage, &moved).unwrap();
        fs::write(moved.join("tool"), b"original synthetic").unwrap();
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("tool"), b"foreign synthetic").unwrap();
        assert!(matches!(owned.check(), Err(ToolError::UnsafePath)));
        drop(owned);
        assert_eq!(fs::read(stage.join("tool")).unwrap(), b"foreign synthetic");
        assert_eq!(fs::read(moved.join("tool")).unwrap(), b"original synthetic");
    }
    #[test]
    fn original_file_pin_preserves_same_name_replacement() {
        let root = tempfile::tempdir().unwrap();
        let stage = root.path().join(".staging-file-pin");
        let mut owned = OwnedDirectory::create(stage.clone()).unwrap();
        let mut file = owned.file("tool").unwrap();
        file.write_all(b"original synthetic").unwrap();
        drop(file);
        fs::rename(stage.join("tool"), stage.join("moved")).unwrap();
        fs::write(stage.join("tool"), b"foreign synthetic").unwrap();
        drop(owned);
        assert_eq!(fs::read(stage.join("tool")).unwrap(), b"foreign synthetic");
        assert_eq!(
            fs::read(stage.join("moved")).unwrap(),
            b"original synthetic"
        );
    }
    #[test]
    fn public_supply_urls_reject_auth_queries_fragments_and_unbounded_input() {
        for value in [
            "http://127.0.0.1/tool",
            "https://user:password@host/tool",
            "https://host/tool?token=secret",
            "https://host/tool#fragment",
            "https://host/tool\n",
            "file:///tool",
            "https://",
        ] {
            assert!(source_policy(value).is_err(), "{value}");
        }
        assert!(source_policy(&format!("https://host/{}", "a".repeat(8192))).is_err());
        source_policy("https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_linux")
            .unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn completed_transfer_unlocks_even_when_an_inherited_descriptor_remains() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("transfer-lock");
        let original = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        fs2::FileExt::try_lock_exclusive(&original).unwrap();
        let inherited = original.try_clone().unwrap();
        let contender = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        assert_eq!(
            fs2::FileExt::try_lock_exclusive(&contender)
                .unwrap_err()
                .kind(),
            io::ErrorKind::WouldBlock
        );
        drop(TransferLock(original));
        fs2::FileExt::try_lock_exclusive(&contender).unwrap();
        fs2::FileExt::unlock(&contender).unwrap();
        assert!(inherited.metadata().unwrap().is_file());
    }
    #[test]
    fn transport_rejects_http_before_any_network_transfer() {
        assert!(matches!(
            HttpsTransport.open("http://127.0.0.1:9/tool", Duration::from_millis(100)),
            Err(ToolError::Http)
        ));
    }
    #[test]
    fn size_hash_ownership_and_pre_cancel_fail_without_activation() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let bytes = b"synthetic bytes";
        let cancel = Arc::new(AtomicBool::new(false));
        for (length, expected) in [
            (0, "limit"),
            (MAX_DOWNLOAD_BYTES + 1, "limit"),
            ((bytes.len() - 1) as u64, "limit"),
            ((bytes.len() + 1) as u64, "size"),
        ] {
            let result = manager.install_download(
                manifest(bytes, "1"),
                length,
                &cancel,
                &mut MemoryTransport(bytes.to_vec()),
                Duration::from_secs(1),
            );
            assert!(
                matches!(
                    (&result, expected),
                    (Err(ToolError::DownloadLimit), "limit")
                        | (Err(ToolError::UnexpectedSize), "size")
                ),
                "length={length}, expected={expected}, result={result:?}"
            );
        }
        let mut bad = manifest(bytes, "1");
        bad.sha256 = "0".repeat(64);
        let bad_result = manager.install_download(
            bad,
            bytes.len() as u64,
            &cancel,
            &mut MemoryTransport(bytes.to_vec()),
            Duration::from_secs(1),
        );
        assert!(
            matches!(bad_result, Err(ToolError::HashMismatch)),
            "{bad_result:?}"
        );
        let mut external = manifest(bytes, "1");
        external.ownership = Ownership::External;
        assert!(matches!(
            manager.download_and_install(external, bytes.len() as u64, &cancel),
            Err(ToolError::ExternalOwnership)
        ));
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            manager.download_and_install(manifest(bytes, "1"), bytes.len() as u64, &cancel),
            Err(ToolError::Cancelled)
        ));
        assert!(manager.active("yt-dlp").unwrap().is_none());
        #[cfg(unix)]
        assert!(!fs::read_dir(&manager.root).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".staging-")
        }));
    }
    struct CancelTransport(Arc<AtomicBool>);
    impl DownloadTransport for CancelTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            struct CancelRead(Arc<AtomicBool>);
            impl Read for CancelRead {
                fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                    self.0.store(true, Ordering::Release);
                    out[0] = b'x';
                    Ok(1)
                }
            }
            Ok(Box::new(CancelRead(self.0.clone())))
        }
    }
    #[test]
    fn cancellation_during_read_and_expired_budget_never_activate() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(matches!(
            manager.install_download(
                manifest(b"x", "1"),
                1,
                &cancel,
                &mut CancelTransport(cancel.clone()),
                Duration::from_secs(1)
            ),
            Err(ToolError::Cancelled)
        ));
        cancel.store(false, Ordering::Release);
        assert!(matches!(
            manager.install_download(
                manifest(b"x", "1"),
                1,
                &cancel,
                &mut MemoryTransport(vec![b'x']),
                Duration::ZERO
            ),
            Err(ToolError::TimedOut)
        ));
        assert!(manager.active("yt-dlp").unwrap().is_none());
    }

    #[cfg(unix)]
    fn executable(version: &str) -> Vec<u8> {
        // Synthetic argv-only Python fixture, never a shell command or actual downloader.
        format!("#!/usr/bin/python3\nimport sys\nassert sys.argv[1:] == ['--ignore-config', '--no-plugin-dirs', '--version']\nprint('{version}')\n").into_bytes()
    }
    #[cfg(unix)]
    fn install(
        manager: &mut ToolManager,
        bytes: Vec<u8>,
        version: &str,
    ) -> Result<InstalledTool, ToolError> {
        manager.install_download(
            manifest(&bytes, version),
            bytes.len() as u64,
            &Arc::new(AtomicBool::new(false)),
            &mut MemoryTransport(bytes),
            Duration::from_secs(5),
        )
    }
    #[cfg(unix)]
    #[test]
    fn verified_install_version_failure_and_database_failure_preserve_previous_active() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let first = install(&mut manager, executable("1"), "1").unwrap();
        assert_eq!(fs::read(&first.executable).unwrap(), executable("1"));
        assert!(matches!(
            install(&mut manager, executable("different"), "2"),
            Err(ToolError::VersionMismatch)
        ));
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("1".into(), "synthetic".into()))
        );
        assert!(!manager.root.join("yt-dlp/2/synthetic").exists());
        manager.db.execute_batch("CREATE TRIGGER deny_activation BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT, 'synthetic database failure'); END;").unwrap();
        assert!(matches!(
            install(&mut manager, executable("2"), "2"),
            Err(ToolError::Database(_))
        ));
        assert!(!manager.root.join("yt-dlp/2/synthetic").exists());
        assert_eq!(fs::read(&first.executable).unwrap(), executable("1"));
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("1".into(), "synthetic".into()))
        );
        manager
            .db
            .execute_batch("DROP TRIGGER deny_activation;")
            .unwrap();
        let second = install(&mut manager, executable("2"), "2").unwrap();
        assert!(first.executable.exists());
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("2".into(), "synthetic".into()))
        );
        assert!(install(&mut manager, executable("2"), "2").is_err());
        assert_eq!(fs::read(&second.executable).unwrap(), executable("2"));
    }
    #[cfg(unix)]
    #[test]
    fn changed_executable_and_version_timeout_never_publish() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let changed = b"#!/usr/bin/python3\nimport sys\nwith open(sys.argv[0], 'r+b') as f: f.write(b'X')\nprint('1')\n".to_vec();
        assert!(matches!(
            install(&mut manager, changed, "1"),
            Err(ToolError::HashMismatch)
        ));
        let slow = b"#!/usr/bin/python3\nimport time\ntime.sleep(30)\nprint('1')\n".to_vec();
        let timeout_result = manager.install_download(
            manifest(&slow, "1"),
            slow.len() as u64,
            &Arc::new(AtomicBool::new(false)),
            &mut MemoryTransport(slow),
            Duration::from_millis(200),
        );
        assert!(
            matches!(timeout_result, Err(ToolError::TimedOut)),
            "{timeout_result:?}"
        );
        assert!(manager.active("yt-dlp").unwrap().is_none());
        assert!(!manager.root.join("yt-dlp/1/synthetic").exists());
    }
    #[test]
    fn transport_error_and_body_timeout_preserve_activation_state() {
        struct FailureTransport(bool);
        impl DownloadTransport for FailureTransport {
            fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
                if !self.0 {
                    return Err(ToolError::Http);
                }
                struct TimeoutRead;
                impl Read for TimeoutRead {
                    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                        Err(io::Error::from(io::ErrorKind::TimedOut))
                    }
                }
                Ok(Box::new(TimeoutRead))
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        for timeout in [false, true] {
            let result = manager.install_download(
                manifest(b"x", "1"),
                1,
                &Arc::new(AtomicBool::new(false)),
                &mut FailureTransport(timeout),
                Duration::from_secs(1),
            );
            assert!(if timeout {
                matches!(result, Err(ToolError::Io(ref e)) if e.kind() == io::ErrorKind::TimedOut)
            } else {
                matches!(result, Err(ToolError::Http))
            });
            assert!(manager.active("yt-dlp").unwrap().is_none());
        }
    }
    #[cfg(unix)]
    #[test]
    fn symlink_destination_and_replaced_cleanup_paths_preserve_foreign_files() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let foreign = temp.path().join("foreign");
        fs::create_dir(&foreign).unwrap();
        fs::write(foreign.join("user-data"), b"preserve").unwrap();
        std::os::unix::fs::symlink(&foreign, manager.root.join("yt-dlp")).unwrap();
        assert!(install(&mut manager, executable("1"), "1").is_err());
        assert_eq!(fs::read(foreign.join("user-data")).unwrap(), b"preserve");
        let path = temp.path().join("owned");
        let mut owned = OwnedDirectory::create(path.clone()).unwrap();
        owned.file("partial").unwrap().write_all(b"owned").unwrap();
        fs::rename(path.join("partial"), temp.path().join("original")).unwrap();
        fs::write(path.join("partial"), b"replacement user data").unwrap();
        drop(owned);
        assert_eq!(
            fs::read(path.join("partial")).unwrap(),
            b"replacement user data"
        );
    }
    #[cfg(unix)]
    #[test]
    fn loopback_http_fixture_is_test_transport_only_and_install_stays_https_policy() {
        use std::net::TcpListener;
        let bytes = executable("3");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let served = bytes.clone();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = [0; 8192];
            let _ = stream.read(&mut request).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                served.len()
            )
            .unwrap();
            stream.write_all(&served).unwrap();
        });
        struct LoopbackTransport(String);
        impl DownloadTransport for LoopbackTransport {
            fn open(&mut self, _: &str, timeout: Duration) -> Result<Box<dyn Read>, ToolError> {
                let config = ureq::Agent::config_builder()
                    .proxy(None)
                    .timeout_global(Some(timeout))
                    .build();
                let response = ureq::Agent::new_with_config(config)
                    .get(&self.0)
                    .call()
                    .map_err(|_| ToolError::Http)?;
                Ok(Box::new(response.into_body().into_reader()))
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let installed = manager
            .install_download(
                manifest(&bytes, "3"),
                bytes.len() as u64,
                &Arc::new(AtomicBool::new(false)),
                &mut LoopbackTransport(format!("http://{address}/fixture")),
                Duration::from_secs(5),
            )
            .unwrap();
        server.join().unwrap();
        assert_eq!(fs::read(installed.executable).unwrap(), bytes);
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("3".into(), "synthetic".into()))
        );
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    #[ignore = "downloads pinned official public yt-dlp executable into a temporary managed root"]
    fn official_https_pinned_executable_installs_in_temporary_owned_root() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: "2026.08.19".into(),
            target: "x86_64-unknown-linux-gnu".into(),
            source_url:
                "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_linux".into(),
            sha256: "58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a".into(),
            license: "GPL-3.0-or-later; THIRD_PARTY_LICENSES at tag 2026.08.19".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp".into(),
        };
        let installed = manager
            .download_and_install(manifest, 40_446_224, &Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert_eq!(installed.bytes, 40_446_224);
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("2026.08.19".into(), "x86_64-unknown-linux-gnu".into()))
        );
        assert!(installed.executable.is_file());
    }
}

/// Native synthetic fixtures, never an official download or Default-desktop fallback.
#[cfg(all(test, windows, target_arch = "x86_64"))]
mod windows_inactive_download_tests {
    use super::*;
    use std::{io::Cursor, sync::Mutex};
    static SERIAL: Mutex<()> = Mutex::new(());
    struct Fixture(Option<tempfile::TempDir>);
    impl Fixture {
        fn new() -> Self {
            Self(Some(tempfile::tempdir().unwrap()))
        }
        fn root(&self) -> &Path {
            self.0.as_ref().unwrap().path()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if std::thread::panicking()
                && let Some(temp) = self.0.take()
            {
                eprintln!(
                    "preserving owned failed inactive download fixture {}",
                    temp.keep().display()
                );
            }
        }
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetThreadDesktop(thread: u32) -> isize;
        fn GetUserObjectInformationW(
            handle: isize,
            index: i32,
            info: *mut std::ffi::c_void,
            len: u32,
            needed: *mut u32,
        ) -> i32;
    }
    fn require_private_cli_desktop() {
        let thread = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
        let desktop = unsafe { GetThreadDesktop(thread) };
        assert_ne!(desktop, 0, "current private desktop query must succeed");
        let mut name = [0u16; 128];
        let mut needed = 0;
        let ok = unsafe {
            GetUserObjectInformationW(desktop, 2, name.as_mut_ptr().cast(), 256, &mut needed)
        };
        assert_ne!(ok, 0, "bounded private desktop name query");
        assert!((2..=256).contains(&needed) && needed.is_multiple_of(2));
        let end = name
            .iter()
            .position(|c| *c == 0)
            .expect("bounded terminated desktop name");
        let name = String::from_utf16(&name[..end]).unwrap();
        let nonce = name
            .strip_prefix("dlpad-cli-")
            .expect("only original private CLI context; no Default fallback");
        assert_eq!(nonce.len(), 32);
        assert!(nonce.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    // Legacy CSC rejects verbatim \\?\ drive spelling even for short owned files.
    // Convert only argv spelling; all filesystem/native identity checks keep original paths.
    fn legacy_csc_owned_path(path: &Path, owned_root: &Path) -> PathBuf {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        assert!(path.is_absolute() && owned_root.is_absolute() && path.starts_with(owned_root));
        assert!(
            !path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        );
        let original: Vec<u16> = path.as_os_str().encode_wide().collect();
        let root: Vec<u16> = owned_root.as_os_str().encode_wide().collect();
        let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
        let normal = original
            .strip_prefix(prefix.as_slice())
            .unwrap_or(&original);
        let normal_root = root.strip_prefix(prefix.as_slice()).unwrap_or(&root);
        let drive = |p: &[u16]| {
            p.len() >= 3
                && ((b'A' as u16..=b'Z' as u16).contains(&p[0])
                    || (b'a' as u16..=b'z' as u16).contains(&p[0]))
                && p[1] == b':' as u16
                && p[2] == b'\\' as u16
        };
        assert!(
            drive(normal) && drive(normal_root),
            "CSC accepts only owned drive absolute paths, no UNC/device prefixes"
        );
        assert_eq!(normal[0], normal_root[0], "same original owned drive");
        assert!(
            normal.len() <= 259 && normal_root.len() <= 259,
            "legacy CSC short UTF16 argv bound"
        );
        assert!(normal.iter().all(|c| *c != 0) && normal[3..].iter().all(|c| *c != b':' as u16));
        assert!(std::char::decode_utf16(normal.iter().copied()).all(|c| c.is_ok()));
        let converted = PathBuf::from(std::ffi::OsString::from_wide(normal));
        let converted_root = PathBuf::from(std::ffi::OsString::from_wide(normal_root));
        assert!(
            converted.starts_with(&converted_root),
            "normal argv remains exact owned namespace"
        );
        converted
    }
    fn compile_owned_fixture(fixture: &Fixture, version: &str) -> Vec<u8> {
        assert!(version.len() <= 32 && version.bytes().all(|b| b.is_ascii_digit() || b == b'.'));
        let root = fs::canonicalize(fixture.root()).unwrap();
        let source = root.join(format!("inactive-{version}.cs"));
        let executable = root.join(format!("inactive-{version}.exe"));
        let program = format!(
            "using System; class SyntheticYtdlp {{ static int Main(string[] a) {{ if(a.Length!=3 || a[0]!=\"--ignore-config\" || a[1]!=\"--no-plugin-dirs\" || a[2]!=\"--version\") return 7; Console.WriteLine(\"{version}\"); return 0; }} }}"
        );
        fs::write(&source, program.as_bytes()).unwrap();
        let source_arg = legacy_csc_owned_path(&source, &root);
        let executable_arg = legacy_csc_owned_path(&executable, &root);
        let result = process::run_metadata(
            &ProcessSpec {
                runtime_environment: None,
                executable: PathBuf::from(
                    r"C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
                ),
                args: vec![
                    "/nologo".into(),
                    "/noconfig".into(),
                    "/target:exe".into(),
                    "/platform:x64".into(),
                    "/optimize+".into(),
                    format!("/out:{}", executable_arg.display()).into(),
                    source_arg.as_os_str().to_owned(),
                ],
                timeout: Duration::from_secs(30),
                max_log_bytes: 65536,
            },
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(result.execution.outcome, ProcessOutcome::Exited);
        assert_eq!(
            result.execution.exit_code,
            Some(0),
            "owned CSC SETUP failed (not inactive product result): source={source:?}; output={executable:?}; stdout={:?}; bounded_logs={:?}",
            String::from_utf8_lossy(&result.stdout),
            result.execution.logs
        );
        assert!(!result.execution.truncated, "owned compiler output bounded");
        let bytes = fs::read(executable).unwrap();
        assert!((65..1048576).contains(&bytes.len()));
        assert!(bytes.starts_with(b"MZ"), "actual owned compiler PE output");
        bytes
    }
    fn manifest(bytes: &[u8], version: &str) -> InstallManifest {
        InstallManifest {
            tool: "yt-dlp".into(),
            version: version.into(),
            target: "x86_64-pc-windows-msvc".into(),
            source_url: "https://supply.invalid/releases/fixture".into(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            license: "synthetic owned test fixture only".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp.exe".into(),
        }
    }
    struct MemoryTransport(Vec<u8>);
    impl DownloadTransport for MemoryTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            Ok(Box::new(Cursor::new(self.0.clone())))
        }
    }
    struct FailTransport;
    impl DownloadTransport for FailTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            Err(io::Error::other("injected owned transport failure").into())
        }
    }
    struct CancelTransport {
        bytes: Vec<u8>,
        cancel: Arc<AtomicBool>,
    }
    impl DownloadTransport for CancelTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            struct Input {
                body: Cursor<Vec<u8>>,
                cancel: Arc<AtomicBool>,
            }
            impl Read for Input {
                fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                    let n = self.body.read(out)?;
                    self.cancel.store(true, Ordering::Release);
                    Ok(n)
                }
            }
            Ok(Box::new(Input {
                body: Cursor::new(self.bytes.clone()),
                cancel: self.cancel.clone(),
            }))
        }
    }
    fn install(
        manager: &mut ToolManager,
        m: InstallManifest,
        bytes: Vec<u8>,
        cancel: &Arc<AtomicBool>,
        activate: bool,
    ) -> Result<InstalledTool, ToolError> {
        manager.install_download_until_with_activation(
            m,
            bytes.len() as u64,
            cancel,
            &mut MemoryTransport(bytes),
            Instant::now() + Duration::from_secs(30),
            activate,
        )
    }
    struct Original {
        active: Option<(String, String)>,
        executable: PathBuf,
        bytes: Vec<u8>,
        manifest_path: PathBuf,
        manifest_bytes: Vec<u8>,
        sentinel: PathBuf,
    }
    impl Original {
        fn capture(manager: &ToolManager, installed: &InstalledTool) -> Self {
            // Synthetic sentinel is inside this fixture's root, outside every installed package.
            let sentinel = manager.root.parent().unwrap().join("owned-sentinel.txt");
            fs::write(&sentinel, b"owned fixture sentinel; preserve").unwrap();
            let manifest_path = installed.executable.parent().unwrap().join("manifest.json");
            Self {
                active: manager.active("yt-dlp").unwrap(),
                executable: installed.executable.clone(),
                bytes: fs::read(&installed.executable).unwrap(),
                manifest_bytes: fs::read(&manifest_path).unwrap(),
                manifest_path,
                sentinel,
            }
        }
        fn assert_preserved(&self, manager: &ToolManager) {
            assert_eq!(
                manager.active("yt-dlp").unwrap(),
                self.active,
                "exact original active row"
            );
            assert_eq!(
                fs::read(&self.executable).unwrap(),
                self.bytes,
                "original installed executable bytes"
            );
            assert_eq!(
                fs::read(&self.manifest_path).unwrap(),
                self.manifest_bytes,
                "original manifest bytes"
            );
            assert_eq!(
                fs::read(&self.sentinel).unwrap(),
                b"owned fixture sentinel; preserve"
            );
        }
    }
    #[test]
    #[ignore = "requires root-owned windows-cli-host private desktop and bounded owned CSC fixture"]
    fn actual_windows_inactive_download_denies_active_sql_and_preserves_original_on_failures() {
        let _serial = SERIAL.lock().unwrap();
        require_private_cli_desktop();
        let fixture = Fixture::new();
        let mut manager = ToolManager::new(&fixture.root().join("managed")).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let old_bytes = compile_owned_fixture(&fixture, "1.0.0");
        let old = install(
            &mut manager,
            manifest(&old_bytes, "1.0.0"),
            old_bytes,
            &cancel,
            true,
        )
        .expect("owned original active setup");
        let original = Original::capture(&manager, &old);
        manager.db.execute_batch("CREATE TRIGGER deny_inactive_insert BEFORE INSERT ON active BEGIN SELECT RAISE(ABORT,'inactive must not INSERT active');END;CREATE TRIGGER deny_inactive_update BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT,'inactive must not UPDATE active');END;CREATE TRIGGER deny_inactive_delete BEFORE DELETE ON active BEGIN SELECT RAISE(ABORT,'inactive must not DELETE active');END;").unwrap();
        let next_bytes = compile_owned_fixture(&fixture, "2.0.0");
        let next_manifest = manifest(&next_bytes, "2.0.0");
        let next = install(
            &mut manager,
            next_manifest.clone(),
            next_bytes.clone(),
            &cancel,
            false,
        )
        .expect("inactive success under denial of all active mutations");
        assert_eq!(fs::read(&next.executable).unwrap(), next_bytes);
        original.assert_preserved(&manager);
        assert!(manager.installed("yt-dlp").unwrap().iter().any(|t| {
            serde_json::to_vec(&t.manifest).unwrap() == serde_json::to_vec(&next_manifest).unwrap()
        }));
        let mut hash_bad = manifest(&next_bytes, "10.0.0");
        hash_bad.sha256 = "0".repeat(64);
        let destination = manager.install_dir(&hash_bad).unwrap();
        assert!(matches!(
            install(&mut manager, hash_bad, next_bytes.clone(), &cancel, false),
            Err(ToolError::HashMismatch)
        ));
        assert!(!destination.exists());
        original.assert_preserved(&manager);
        let wrong_version = manifest(&next_bytes, "11.0.0");
        let destination = manager.install_dir(&wrong_version).unwrap();
        assert!(matches!(
            install(
                &mut manager,
                wrong_version,
                next_bytes.clone(),
                &cancel,
                false
            ),
            Err(ToolError::VersionMismatch)
        ));
        assert!(!destination.exists());
        original.assert_preserved(&manager);
        let unavailable = manifest(&next_bytes, "12.0.0");
        let destination = manager.install_dir(&unavailable).unwrap();
        let result = manager.install_download_until_with_activation(
            unavailable,
            next_bytes.len() as u64,
            &cancel,
            &mut FailTransport,
            Instant::now() + Duration::from_secs(30),
            false,
        );
        assert!(matches!(result, Err(ToolError::Io(_))));
        assert!(!destination.exists());
        original.assert_preserved(&manager);
        let cancelled = manifest(&next_bytes, "13.0.0");
        let destination = manager.install_dir(&cancelled).unwrap();
        let result = manager.install_download_until_with_activation(
            cancelled,
            next_bytes.len() as u64,
            &cancel,
            &mut CancelTransport {
                bytes: next_bytes.clone(),
                cancel: cancel.clone(),
            },
            Instant::now() + Duration::from_secs(30),
            false,
        );
        assert!(matches!(result, Err(ToolError::Cancelled)));
        assert!(!destination.exists());
        original.assert_preserved(&manager);
        cancel.store(false, Ordering::Release);
        let denied_bytes = compile_owned_fixture(&fixture, "3.0.0");
        let denied = manifest(&denied_bytes, "3.0.0");
        let destination = manager.install_dir(&denied).unwrap();
        assert!(
            matches!(
                install(&mut manager, denied, denied_bytes, &cancel, true),
                Err(ToolError::Database(_))
            ),
            "true activation must actually hit same SQL denial guard"
        );
        assert!(!destination.exists());
        original.assert_preserved(&manager);
        assert_eq!(
            fs::read(&next.executable).unwrap(),
            next_bytes,
            "previous successful inactive bytes also preserved"
        );
    }
    struct HookState {
        cancel: Arc<AtomicBool>,
        active_updates: AtomicU64,
    }
    struct ActiveUpdateHook {
        db: *mut rusqlite::ffi::sqlite3,
        state: Box<HookState>,
    }
    unsafe extern "C" fn cancel_after_active_update(
        opaque: *mut std::ffi::c_void,
        operation: i32,
        _database: *const std::ffi::c_char,
        table: *const std::ffi::c_char,
        _rowid: i64,
    ) {
        // SQLite calls synchronously on this owned connection. State is pinned in Box
        // until RAII unregister; table is SQLite's valid NUL-terminated callback string.
        if operation == rusqlite::ffi::SQLITE_UPDATE
            && !table.is_null()
            && unsafe { std::ffi::CStr::from_ptr(table) }.to_bytes() == b"active"
        {
            let state = unsafe { &*opaque.cast::<HookState>() };
            state.active_updates.fetch_add(1, Ordering::Release);
            state.cancel.store(true, Ordering::Release);
        }
    }
    impl ActiveUpdateHook {
        fn install(connection: &Connection, cancel: Arc<AtomicBool>) -> Self {
            let mut state = Box::new(HookState {
                cancel,
                active_updates: AtomicU64::new(0),
            });
            // Owned live connection has no hook; no connection/state drop or concurrent use
            // until this guard unregisters. No production hook or extra callback seam.
            let db = unsafe { connection.handle() };
            let old = unsafe {
                rusqlite::ffi::sqlite3_update_hook(
                    db,
                    Some(cancel_after_active_update),
                    (&mut *state as *mut HookState).cast(),
                )
            };
            let hook = Self { db, state };
            // Construct RAII before the assertion so failure also unregisters before state Drop.
            assert!(
                old.is_null(),
                "owned test connection has no previous update hook"
            );
            hook
        }
    }
    impl Drop for ActiveUpdateHook {
        fn drop(&mut self) {
            unsafe { rusqlite::ffi::sqlite3_update_hook(self.db, None, std::ptr::null_mut()) };
        }
    }
    #[test]
    #[ignore = "requires root-owned windows-cli-host private desktop and bounded owned CSC fixture"]
    fn actual_windows_direct_download_precommit_cancel_after_active_update_rolls_back() {
        let _serial = SERIAL.lock().unwrap();
        require_private_cli_desktop();
        let fixture = Fixture::new();
        let mut manager = ToolManager::new(&fixture.root().join("managed")).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let old_bytes = compile_owned_fixture(&fixture, "1.0.0");
        let old = install(
            &mut manager,
            manifest(&old_bytes, "1.0.0"),
            old_bytes,
            &cancel,
            true,
        )
        .expect("owned original active setup");
        let original = Original::capture(&manager, &old);
        let next_bytes = compile_owned_fixture(&fixture, "2.0.0");
        let next = manifest(&next_bytes, "2.0.0");
        let destination = manager.install_dir(&next).unwrap();
        let hook = ActiveUpdateHook::install(&manager.db, cancel.clone());
        let result = install(&mut manager, next, next_bytes, &cancel, true);
        assert_eq!(
            hook.state.active_updates.load(Ordering::Acquire),
            1,
            "actual owned active UPDATE observed before cancellation"
        );
        assert!(cancel.load(Ordering::Acquire));
        assert!(
            matches!(result, Err(ToolError::Cancelled)),
            "existing post-UPDATE checkpoint must refuse commit"
        );
        drop(hook);
        original.assert_preserved(&manager);
        assert!(
            !destination.exists(),
            "uncommitted new publication removed only by original ownership checks"
        );
    }
}
