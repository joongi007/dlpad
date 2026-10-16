//! Evidence-backed installer leftovers. Missing or mismatched evidence is never deletion authority.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
const MAX_STAGES: usize = 8;
const MAX_STAGE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_STAGE_ENTRIES: usize = 2048;
const MAX_EVIDENCE_BYTES: u64 = 16 * 1024;

#[derive(Debug, Default, Clone, Copy)]
pub struct StagingRecovery {
    pub removed: usize,
    pub preserved: usize,
    pub bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    dev: u64,
    ino: u64,
}
fn identity(metadata: &fs::Metadata) -> Option<Identity> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(Identity {
            dev: metadata.dev(),
            ino: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerIdentity {
    pid: u32,
    start_ticks: u64,
    boot_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileEvidence {
    name: String,
    identity: Option<Identity>,
    bytes: u64,
    sha256: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    schema_version: u32,
    root: Option<Identity>,
    directory: Option<Identity>,
    owner: Option<OwnerIdentity>,
    executable: String,
    files: Vec<FileEvidence>,
}
#[cfg(target_os = "linux")]
fn bounded_read(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Identity evidence exceeded its limit",
        ));
    }
    Ok(bytes)
}
#[cfg(target_os = "linux")]
fn boot_id() -> io::Result<String> {
    let bytes = bounded_read(Path::new("/proc/sys/kernel/random/boot_id"), 64)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(io::Error::other)?
        .trim();
    if text.len() != 36 || !text.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        return Err(io::Error::other("Invalid boot identity"));
    }
    Ok(text.to_owned())
}
#[cfg(target_os = "linux")]
fn process_identity(pid: u32) -> io::Result<(u64, bool)> {
    let bytes = bounded_read(&PathBuf::from(format!("/proc/{pid}/stat")), 8192)?;
    let text = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    let (_, fields) = text
        .rsplit_once(") ")
        .ok_or_else(|| io::Error::other("Invalid process identity"))?;
    let mut fields = fields.split_whitespace();
    let state = fields
        .next()
        .ok_or_else(|| io::Error::other("Missing process state"))?;
    let start_ticks = fields
        .nth(18)
        .ok_or_else(|| io::Error::other("Missing process start"))?
        .parse()
        .map_err(io::Error::other)?;
    Ok((start_ticks, !matches!(state, "Z" | "X")))
}
fn current_owner() -> Option<OwnerIdentity> {
    #[cfg(target_os = "linux")]
    {
        let pid = std::process::id();
        let (start_ticks, live) = process_identity(pid).ok()?;
        {
            let boot_id = boot_id().ok()?;
            live.then_some(OwnerIdentity {
                pid,
                start_ticks,
                boot_id,
            })
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
fn owner_live(owner: &OwnerIdentity) -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        if boot_id().ok()? != owner.boot_id {
            return Some(false);
        }
        match process_identity(owner.pid) {
            Ok((start_ticks, live)) => Some(start_ticks == owner.start_ticks && live),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Some(false),
            Err(_) => None,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = owner;
        None
    }
}
fn stage_name(name: &str) -> bool {
    identifier(name) && name.starts_with(".staging-")
}
pub(super) fn file_evidence(
    name: &str,
    file: &File,
    bytes: u64,
    sha256: String,
) -> Result<FileEvidence, ToolError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != bytes {
        return Err(ToolError::UnsafePath);
    }
    Ok(FileEvidence {
        name: name.to_owned(),
        identity: identity(&metadata),
        bytes,
        sha256,
    })
}
fn evidence_valid(evidence: &Evidence) -> bool {
    matches!(evidence.schema_version, 1 | 2)
        && identifier(&evidence.executable)
        && evidence.executable != "manifest.json"
        && !evidence.files.is_empty()
        && evidence.files.len() <= if evidence.schema_version == 1 { 2 } else { 10 }
        && evidence.files.iter().all(|f| {
            (evidence.schema_version == 2
                || f.name == evidence.executable
                || f.name == "manifest.json")
                && identifier(&f.name)
                && f.bytes <= MAX_DOWNLOAD_BYTES
                && f.sha256.len() == 64
                && f.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        })
        && evidence
            .files
            .iter()
            .map(|f| &f.name)
            .collect::<BTreeSet<_>>()
            .len()
            == evidence.files.len()
}
impl ToolManager {
    pub(super) fn record_staging(
        &self,
        path: &Path,
        directory: &fs::Metadata,
        executable: &str,
        files: Vec<FileEvidence>,
    ) -> Result<(), ToolError> {
        self.record_staging_version(path, directory, executable, files, 1)
    }
    pub(super) fn record_package_staging(
        &self,
        path: &Path,
        directory: &fs::Metadata,
        executable: &str,
        files: Vec<FileEvidence>,
    ) -> Result<(), ToolError> {
        self.record_staging_version(path, directory, executable, files, 2)
    }
    fn record_staging_version(
        &self,
        path: &Path,
        directory: &fs::Metadata,
        executable: &str,
        files: Vec<FileEvidence>,
        schema_version: u32,
    ) -> Result<(), ToolError> {
        self.verify_root()?;
        safe_components(path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| stage_name(n))
            .ok_or(ToolError::UnsafePath)?;
        if path.parent() != Some(self.root.as_path()) {
            return Err(ToolError::UnsafePath);
        }
        let evidence = Evidence {
            schema_version,
            root: identity(&fs::metadata(&self.root)?),
            directory: identity(directory),
            owner: current_owner(),
            executable: executable.to_owned(),
            files,
        };
        if !evidence_valid(&evidence) {
            return Err(ToolError::UnsafePath);
        }
        let json = serde_json::to_string(&evidence)?;
        if json.len() as u64 > MAX_EVIDENCE_BYTES {
            return Err(ToolError::InventoryLimit);
        }
        #[cfg(unix)]
        File::open(path)?.sync_all()?;
        self.db.execute("INSERT INTO install_staging(name,evidence_json) VALUES(?1,?2) ON CONFLICT(name) DO UPDATE SET evidence_json=excluded.evidence_json", params![name,json])?;
        Ok(())
    }
    pub(super) fn forget_staging(&self, path: &Path) -> Result<(), ToolError> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| stage_name(n))
            .ok_or(ToolError::UnsafePath)?;
        self.db
            .execute("DELETE FROM install_staging WHERE name=?1", [name])?;
        Ok(())
    }
    fn staging_inventory(&self) -> Result<(usize, u64), ToolError> {
        self.verify_root()?;
        let mut count = 0usize;
        let mut bytes = 0u64;
        let mut scanned = 0usize;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            scanned += 1;
            if scanned > MAX_STAGE_ENTRIES {
                return Err(ToolError::InventoryLimit);
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return Err(ToolError::UnsafePath);
            };
            if !name.starts_with(".staging-") {
                continue;
            }
            count += 1;
            if count > MAX_STAGES {
                return Err(ToolError::InventoryLimit);
            }
            if !stage_name(name) || !entry.file_type()?.is_dir() {
                return Err(ToolError::UnsafePath);
            }
            for file in fs::read_dir(entry.path())? {
                let file = file?;
                scanned += 1;
                if scanned > MAX_STAGE_ENTRIES {
                    return Err(ToolError::InventoryLimit);
                }
                let metadata = fs::symlink_metadata(file.path())?;
                if !metadata.is_file() {
                    return Err(ToolError::UnsafePath);
                }
                bytes = bytes
                    .checked_add(metadata.len())
                    .ok_or(ToolError::DownloadLimit)?;
                if bytes > MAX_STAGE_BYTES {
                    return Err(ToolError::DownloadLimit);
                }
            }
        }
        Ok((count, bytes))
    }
    pub(super) fn reserve_staging(&mut self, expected_bytes: u64) -> Result<(), ToolError> {
        self.recover_staging()?;
        let (count, bytes) = self.staging_inventory()?;
        if count >= MAX_STAGES {
            return Err(ToolError::InventoryLimit);
        }
        if bytes
            .checked_add(expected_bytes)
            .is_none_or(|total| total > MAX_STAGE_BYTES)
        {
            return Err(ToolError::DownloadLimit);
        }
        Ok(())
    }
    /// Linux owner/start/boot and Unix file identity are required for deletion.
    /// Unknown/live/tampered staging remains; inventory bounds can reject further installs.
    pub fn recover_staging(&mut self) -> Result<StagingRecovery, ToolError> {
        let (count, _) = self.staging_inventory()?;
        let mut result = StagingRecovery {
            preserved: count,
            ..StagingRecovery::default()
        };
        let records = {
            let mut query = self.db.prepare("SELECT name,evidence_json FROM install_staging WHERE length(CAST(name AS BLOB)) <=128 AND length(CAST(evidence_json AS BLOB)) <=16384 LIMIT 9")?;
            let rows = query.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut records = Vec::new();
            for row in rows {
                records.push(row?);
            }
            if records.len() > MAX_STAGES {
                return Err(ToolError::InventoryLimit);
            }
            records
        };
        for (name, json) in records {
            if !stage_name(&name) {
                continue;
            }
            let path = self.root.join(&name);
            if matches!(fs::symlink_metadata(&path), Err(ref e) if e.kind() == io::ErrorKind::NotFound)
            {
                self.forget_staging(&path)?;
                continue;
            }
            let Ok(evidence) = serde_json::from_str::<Evidence>(&json) else {
                continue;
            };
            if !evidence_valid(&evidence)
                || evidence.root.is_none()
                || evidence.directory.is_none()
                || evidence.root != identity(&fs::metadata(&self.root)?)
                || evidence.directory != identity(&fs::symlink_metadata(&path)?)
                || evidence.owner.as_ref().and_then(owner_live) != Some(false)
            {
                continue;
            }
            if self.verify_stage_files(&path, &evidence)? {
                self.cleanup_verified_stage(&path, &evidence)?;
                self.forget_staging(&path)?;
                result.removed += 1;
                result.preserved = result.preserved.saturating_sub(1);
            }
        }
        result.bytes = self.staging_inventory()?.1;
        Ok(result)
    }
    fn verify_stage_files(&self, path: &Path, evidence: &Evidence) -> Result<bool, ToolError> {
        self.verify_root()?;
        safe_components(path)?;
        let expected: BTreeMap<_, _> = evidence
            .files
            .iter()
            .map(|f| (f.name.as_str(), f))
            .collect();
        let mut seen = BTreeSet::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return Ok(false);
            };
            let Some(record) = expected.get(name) else {
                return Ok(false);
            };
            let metadata = fs::symlink_metadata(entry.path())?;
            if !metadata.is_file()
                || record.identity.is_none()
                || record.identity != identity(&metadata)
                || record.bytes != metadata.len()
            {
                return Ok(false);
            }
            let mut file = File::open(entry.path())?;
            if record.identity != identity(&file.metadata()?) {
                return Ok(false);
            }
            let mut hash = Sha256::new();
            let mut buffer = [0; 64 * 1024];
            let mut bytes = 0u64;
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > record.bytes {
                    return Ok(false);
                }
                hash.update(&buffer[..n]);
            }
            if bytes != record.bytes
                || format!("{:x}", hash.finalize()) != record.sha256.to_ascii_lowercase()
            {
                return Ok(false);
            }
            seen.insert(name.to_owned());
        }
        Ok(seen.len() == expected.len())
    }
    fn cleanup_verified_stage(&self, path: &Path, evidence: &Evidence) -> Result<(), ToolError> {
        self.verify_root()?;
        safe_components(path)?;
        if evidence.directory != identity(&fs::symlink_metadata(path)?)
            || !self.verify_stage_files(path, evidence)?
        {
            return Err(ToolError::UnsafePath);
        }
        for record in &evidence.files {
            let file_path = path.join(&record.name);
            let file = File::open(&file_path)?;
            let current = fs::symlink_metadata(&file_path)?;
            if !current.is_file()
                || record.identity != identity(&current)
                || record.identity != identity(&file.metadata()?)
            {
                return Err(ToolError::UnsafePath);
            }
            // Raw-path unlink retains a same-user malicious concurrent replacement gate.
            fs::remove_file(file_path)?;
        }
        fs::remove_dir(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    fn stage(manager: &ToolManager, name: &str, bytes: &[u8]) -> PathBuf {
        let path = manager.root.join(name);
        fs::create_dir(&path).unwrap();
        let file_path = path.join("yt-dlp");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .unwrap();
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
        manager
            .record_staging(
                &path,
                &fs::metadata(&path).unwrap(),
                "yt-dlp",
                vec![
                    file_evidence(
                        "yt-dlp",
                        &file,
                        bytes.len() as u64,
                        format!("{:x}", Sha256::digest(bytes)),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
        path
    }
    #[cfg(target_os = "linux")]
    fn dead_owner(manager: &ToolManager, path: &Path) -> Evidence {
        let name = path.file_name().unwrap().to_str().unwrap();
        let json: String = manager
            .db
            .query_row(
                "SELECT evidence_json FROM install_staging WHERE name=?1",
                [name],
                |r| r.get(0),
            )
            .unwrap();
        let mut evidence: Evidence = serde_json::from_str(&json).unwrap();
        evidence.owner = Some(OwnerIdentity {
            pid: u32::MAX,
            start_ticks: 1,
            boot_id: boot_id().unwrap(),
        });
        manager
            .db
            .execute(
                "UPDATE install_staging SET evidence_json=?2 WHERE name=?1",
                params![name, serde_json::to_string(&evidence).unwrap()],
            )
            .unwrap();
        evidence
    }
    #[test]
    fn unrecorded_staging_capacity_and_nested_unknown_sources_fail_safely() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        for number in 0..MAX_STAGES {
            fs::create_dir(manager.root.join(format!(".staging-unrecorded-{number}"))).unwrap();
        }
        assert!(matches!(
            manager.reserve_staging(1),
            Err(ToolError::InventoryLimit)
        ));
        assert_eq!(manager.recover_staging().unwrap().preserved, MAX_STAGES);
        let path = manager.root.join(".staging-unrecorded-0");
        fs::create_dir(path.join("foreign-directory")).unwrap();
        fs::write(path.join("foreign-directory/user-data"), b"preserve").unwrap();
        assert!(matches!(
            manager.reserve_staging(1),
            Err(ToolError::UnsafePath)
        ));
        assert_eq!(
            fs::read(path.join("foreign-directory/user-data")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn aggregate_bytes_plus_new_reservation_never_exceed_budget() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let path = manager.root.join(".staging-unrecorded-budget");
        fs::create_dir(&path).unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join("unknown-original"))
            .unwrap();
        #[cfg(windows)]
        {
            use std::os::windows::{fs::MetadataExt, io::AsRawHandle};
            use windows_sys::Win32::{
                Storage::FileSystem::FILE_ATTRIBUTE_SPARSE_FILE, System::IO::DeviceIoControl,
            };
            // winioctl.h FSCTL_SET_SPARSE, also windows-sys 0.61.2's value.
            // Keep this test-only constant to avoid adding Win32_System_Ioctl.
            const FSCTL_SET_SPARSE: u32 = 0x0009_00c4;
            let mut returned = 0;
            // SAFETY: freshly-created test file owns this live writable handle.
            // Null input sets sparse=true; this synchronous call has no output buffer.
            let marked = unsafe {
                DeviceIoControl(
                    file.as_raw_handle(),
                    FSCTL_SET_SPARSE,
                    std::ptr::null(),
                    0,
                    std::ptr::null_mut(),
                    0,
                    &mut returned,
                    std::ptr::null_mut(),
                )
            };
            assert_ne!(
                marked,
                0,
                "staging budget test requires a sparse fixture before extending logical length: {}",
                std::io::Error::last_os_error()
            );
            assert_ne!(
                file.metadata().unwrap().file_attributes() & FILE_ATTRIBUTE_SPARSE_FILE,
                0,
                "FSCTL_SET_SPARSE must mark the original test file; no skip or dense fallback"
            );
        }
        file.set_len(MAX_STAGE_BYTES - 10).unwrap();
        assert!(matches!(
            manager.reserve_staging(11),
            Err(ToolError::DownloadLimit)
        ));
        manager.reserve_staging(10).unwrap();
        assert_eq!(
            fs::metadata(path.join("unknown-original")).unwrap().len(),
            MAX_STAGE_BYTES - 10
        );
        file.set_len(MAX_STAGE_BYTES + 1).unwrap();
        assert!(matches!(
            manager.recover_staging(),
            Err(ToolError::DownloadLimit)
        ));
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn live_owner_unknown_owner_changed_bytes_and_foreign_files_are_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let path = stage(&manager, ".staging-live-owner", b"owned");
        assert_eq!(manager.recover_staging().unwrap().preserved, 1);
        assert!(path.join("yt-dlp").exists());
        let mut evidence = dead_owner(&manager, &path);
        fs::write(path.join("yt-dlp"), b"other").unwrap(); // same size/inode, different digest
        assert_eq!(manager.recover_staging().unwrap().removed, 0);
        fs::write(path.join("yt-dlp"), b"owned").unwrap();
        fs::write(path.join("user-file"), b"preserve").unwrap();
        assert_eq!(manager.recover_staging().unwrap().removed, 0);
        assert_eq!(fs::read(path.join("user-file")).unwrap(), b"preserve");
        fs::remove_file(path.join("user-file")).unwrap(); // this test's synthetic fixture
        evidence.owner = None;
        manager
            .db
            .execute(
                "UPDATE install_staging SET evidence_json=?1",
                [serde_json::to_string(&evidence).unwrap()],
            )
            .unwrap();
        assert_eq!(manager.recover_staging().unwrap().removed, 0);
        dead_owner(&manager, &path);
        assert_eq!(manager.recover_staging().unwrap().removed, 1);
        assert!(!path.exists());
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn pid_reuse_or_previous_boot_is_dead_but_file_replacement_has_no_authority() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let path = stage(&manager, ".staging-identity-change", b"owned");
        let evidence = dead_owner(&manager, &path);
        let original = temp.path().join("original-owner-file");
        fs::rename(path.join("yt-dlp"), &original).unwrap();
        fs::write(path.join("yt-dlp"), b"owned").unwrap();
        assert_eq!(manager.recover_staging().unwrap().removed, 0);
        assert!(matches!(
            manager.cleanup_verified_stage(&path, &evidence),
            Err(ToolError::UnsafePath)
        ));
        assert_eq!(fs::read(path.join("yt-dlp")).unwrap(), b"owned");
        assert_eq!(fs::read(&original).unwrap(), b"owned");
        let mut reused = current_owner().unwrap();
        reused.start_ticks = reused.start_ticks.saturating_add(1);
        assert_eq!(owner_live(&reused), Some(false));
        let mut previous_boot = current_owner().unwrap();
        previous_boot.boot_id = "previous-boot".into();
        assert_eq!(owner_live(&previous_boot), Some(false));
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn symlink_and_unrecorded_directory_identity_keep_external_data() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let path = stage(&manager, ".staging-symlink", b"owned");
        dead_owner(&manager, &path);
        let original = temp.path().join("original");
        fs::rename(path.join("yt-dlp"), &original).unwrap();
        std::os::unix::fs::symlink(&original, path.join("yt-dlp")).unwrap();
        assert!(matches!(
            manager.recover_staging(),
            Err(ToolError::UnsafePath)
        ));
        assert_eq!(fs::read(&original).unwrap(), b"owned");
        fs::remove_file(path.join("yt-dlp")).unwrap();
        fs::rename(&original, path.join("yt-dlp")).unwrap();
        let old = temp.path().join("old-stage");
        fs::rename(&path, &old).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join("yt-dlp"), b"foreign").unwrap();
        assert_eq!(manager.recover_staging().unwrap().removed, 0);
        assert_eq!(fs::read(path.join("yt-dlp")).unwrap(), b"foreign");
        assert_eq!(fs::read(old.join("yt-dlp")).unwrap(), b"owned");
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn cleanup_validation_failure_leaves_active_installation_unchanged() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let source = temp.path().join("external-fixture");
        let original_bytes = b"synthetic previous active installation";
        fs::write(&source, original_bytes).unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let installed = manager
            .import(
                &source,
                InstallManifest {
                    tool: "fixture".into(),
                    version: "1".into(),
                    target: "test".into(),
                    source_url: "fixture://synthetic".into(),
                    sha256: format!("{:x}", Sha256::digest(original_bytes)),
                    license: "synthetic".into(),
                    ownership: Ownership::Managed,
                    executable: "fixture-tool".into(),
                },
            )
            .unwrap();
        let path = stage(&manager, ".staging-failure", b"owned");
        let evidence = dead_owner(&manager, &path);
        fs::write(path.join("yt-dlp"), b"changed after preflight").unwrap();
        assert!(matches!(
            manager.cleanup_verified_stage(&path, &evidence),
            Err(ToolError::UnsafePath)
        ));
        assert_eq!(
            manager.active("fixture").unwrap(),
            Some(("1".into(), "test".into()))
        );
        assert_eq!(
            fs::read(path.join("yt-dlp")).unwrap(),
            b"changed after preflight"
        );
        fs::write(path.join("yt-dlp"), b"owned").unwrap();
        manager.db.execute_batch("CREATE TRIGGER deny_stage_delete BEFORE DELETE ON install_staging BEGIN SELECT RAISE(ABORT, 'synthetic cleanup journal failure'); END;").unwrap();
        assert!(matches!(
            manager.recover_staging(),
            Err(ToolError::Database(_))
        ));
        assert_eq!(
            manager.active("fixture").unwrap(),
            Some(("1".into(), "test".into()))
        );
        assert!(!path.exists());
        assert_eq!(fs::read(installed.executable).unwrap(), original_bytes);
        assert_eq!(fs::read(source).unwrap(), original_bytes);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn filesystem_cleanup_permission_failure_preserves_owned_bytes_and_active_state() {
        use std::os::unix::fs::PermissionsExt;
        // Root ignores directory permissions; do not claim that failure path on root.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        manager
            .db
            .execute(
                "INSERT INTO active(tool,version,target) VALUES('fixture','1','test')",
                [],
            )
            .unwrap();
        let path = stage(&manager, ".staging-unwritable", b"owned");
        dead_owner(&manager, &path);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o500)).unwrap();
        let result = manager.recover_staging();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            matches!(result, Err(ToolError::Io(ref error)) if error.kind() == io::ErrorKind::PermissionDenied)
        );
        assert_eq!(fs::read(path.join("yt-dlp")).unwrap(), b"owned");
        assert_eq!(
            manager.active("fixture").unwrap(),
            Some(("1".into(), "test".into()))
        );
        assert_eq!(manager.recover_staging().unwrap().removed, 1);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn crash_fixture_child() {
        let Some(root) = std::env::var_os("DLPAD_STAGE_CRASH_FIXTURE") else {
            return;
        };
        let root = PathBuf::from(root);
        let manager = ToolManager::new(&root).unwrap();
        stage(
            &manager,
            ".staging-owned-crash-fixture",
            b"synthetic completed checkpoint",
        );
        fs::write(root.parent().unwrap().join("ready"), b"fixture ready").unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn owned_process_sigkill_staging_is_recovered_after_live_owner_preservation() {
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tools::staging_recovery::tests::crash_fixture_child",
                "--nocapture",
            ])
            .env("DLPAD_STAGE_CRASH_FIXTURE", &root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        struct OwnedChild(std::process::Child);
        impl Drop for OwnedChild {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while !temp.path().join("ready").exists() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("owned fixture did not become ready");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut child = OwnedChild(child);
        let mut manager = ToolManager::new(&root).unwrap();
        assert_eq!(manager.recover_staging().unwrap().preserved, 1);
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        let result = manager.recover_staging().unwrap();
        assert_eq!((result.removed, result.preserved, result.bytes), (1, 0, 0));
        assert!(!root.join(".staging-owned-crash-fixture").exists());
    }
}
