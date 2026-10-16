//! Verified local imports and bounded HTTPS direct-executable installations.
mod archive_download;
mod archive_install;
mod archive_layout;
mod archive_manifest;
mod bounded_tar;
mod bounded_xz;
mod bounded_zip;
mod deno_version;
mod direct_download;
mod execution_lease;
mod execution_marker;
#[cfg(windows)]
mod windows_managed_recovery;
// The bounded format is shared with the upcoming native managed-reader gate.
mod ffmpeg_version;
// Native persistence is being connected to the managed guardian protocol.
#[cfg(windows)]
#[allow(dead_code)]
mod windows_managed_guardian_journal;
#[cfg(windows)]
#[allow(dead_code)]
pub(crate) mod windows_managed_journal;
#[cfg(windows)]
pub(crate) use windows_managed_guardian_journal::ManagedGuardianJournal;
#[cfg(windows)]
pub(crate) use windows_managed_journal::ManagedJournal;
#[cfg(windows)]
pub use windows_managed_records::ReaderStage as ManagedReaderStage;
#[cfg(any(windows, test))]
#[allow(dead_code)]
pub(crate) mod windows_managed_records;
pub use execution_lease::{ExecutionLease, ManagedExecution};
/// Recovery state after validating original Windows records and actual reader receipts.
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagedExecutionRecovery {
    pub recovered: bool,
    pub pending: bool,
}
// Owner classification is wired before production environment activation.
mod install_capacity;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod managed_deno_classification;
#[cfg(windows)]
pub(crate) use managed_deno_classification::VerifiedManagedDeno;
pub(crate) mod native_files;
mod package_inventory;
mod package_removal;
pub use archive_layout::{ArchiveLayout, ArchiveMemberMapping};
pub use archive_manifest::{
    ArchiveComponent, ArchiveComponentRole, ArchivePackage, MAX_ARCHIVE_BYTES, MAX_EXPANDED_BYTES,
    PinnedArchiveNotice,
};
pub use bounded_tar::{
    TarComponentMapping, TarExtractionRequest, TarLimits, extract_tar_component,
};
pub use bounded_xz::{MAX_XZ_MEMORY_KIB, TarXzSourceSpec, extract_verified_tar_xz};
pub use bounded_zip::extract_verified_zip;
mod selection;
mod staging_recovery;
pub use direct_download::MAX_DOWNLOAD_BYTES;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use staging_recovery::StagingRecovery;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};
const OWNER: &[u8] = b"dlpad-managed-tools-v1\n";
const MANIFEST_LIMIT: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ownership {
    Managed,
    External,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub tool: String,
    pub version: String,
    pub target: String,
    pub source_url: String,
    pub sha256: String,
    pub license: String,
    pub ownership: Ownership,
    pub executable: String,
}
#[derive(Debug)]
pub enum ToolError {
    Io(io::Error),
    Database(rusqlite::Error),
    Json(serde_json::Error),
    InvalidManifest,
    UnsafePath,
    ExternalOwnership,
    HashMismatch,
    ActiveVersion,
    ExecutionPending,
    UnexpectedFiles,
    Cancelled,
    DownloadLimit,
    UnexpectedSize,
    Http,
    TimedOut,
    VersionMismatch,
    InventoryLimit,
}
impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ToolError {}
impl From<io::Error> for ToolError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for ToolError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
impl From<serde_json::Error> for ToolError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
#[derive(Debug, Clone)]
pub struct InstalledTool {
    pub manifest: InstallManifest,
    pub executable: PathBuf,
    pub bytes: u64,
}
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub package: ArchivePackage,
    pub directory: PathBuf,
    pub main: InstalledTool,
}
pub struct ToolManager {
    root: PathBuf,
    root_pin: File,
    db: Connection,
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && value != "."
        && value != ".."
}
// OS aliases such as macOS /tmp are accepted before establishing ownership.
// The explicitly selected final component itself must never be a symlink.
fn selected_path(path: &Path) -> Result<(), ToolError> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(ToolError::UnsafePath);
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(ToolError::UnsafePath),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn safe_components(path: &Path) -> Result<(), ToolError> {
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir) {
            return Err(ToolError::UnsafePath);
        }
        current.push(part.as_os_str());
        if matches!(part, Component::Prefix(_)) {
            continue; // A Windows verbatim prefix alone is not a queryable path.
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(ToolError::UnsafePath),
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn validate(m: &InstallManifest) -> Result<(), ToolError> {
    if m.ownership != Ownership::Managed {
        return Err(ToolError::ExternalOwnership);
    }
    if ![&m.tool, &m.version, &m.target, &m.executable]
        .into_iter()
        .all(|s| identifier(s))
        || m.executable == "manifest.json"
        || m.sha256.len() != 64
        || !m.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || m.source_url.is_empty()
        || m.license.is_empty()
    {
        return Err(ToolError::InvalidManifest);
    }
    if serde_json::to_vec(m)?.len() as u64 > MANIFEST_LIMIT {
        return Err(ToolError::InvalidManifest);
    }
    Ok(())
}
impl ToolManager {
    /// Recover only an original completed Windows reader namespace. Unknown,
    /// live and incomplete evidence is preserved; false recovered never means ready.
    #[cfg(windows)]
    pub fn recover_execution(&self) -> Result<ManagedExecutionRecovery, ToolError> {
        let recovered = windows_managed_recovery::recover(self)?;
        let pending = execution_marker::pending(&self.root)?;
        Ok(ManagedExecutionRecovery { recovered, pending })
    }
    /// Accepts a dedicated empty directory, or a directory already carrying our marker.
    pub fn new(root: &Path) -> Result<Self, ToolError> {
        selected_path(root)?;
        fs::create_dir_all(root)?;
        selected_path(root)?;
        let root = root.canonicalize()?;
        let root_pin = native_files::open_directory(&root)?;
        let marker = root.join(".dlpad-owner");
        if marker.exists() {
            safe_components(&marker)?;
            if fs::metadata(&marker)?.len() != OWNER.len() as u64 || fs::read(&marker)? != OWNER {
                return Err(ToolError::UnsafePath);
            }
        } else {
            if fs::read_dir(&root)?.next().is_some() {
                return Err(ToolError::UnsafePath);
            }
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&marker)?
                .write_all(OWNER)?;
        }
        let db_path = root.join("activation.sqlite");
        safe_components(&db_path)?;
        let db = Connection::open(db_path)?;
        db.execute_batch("CREATE TABLE IF NOT EXISTS active(tool TEXT PRIMARY KEY, version TEXT NOT NULL, target TEXT NOT NULL);")?;
        db.execute_batch(include_str!("tools/staging_schema.sql"))?;
        if !native_files::same_directory(&root, &root_pin) {
            return Err(ToolError::UnsafePath);
        }
        Ok(Self { root, root_pin, db })
    }
    fn verify_root(&self) -> Result<(), ToolError> {
        verify_root_pin(&self.root, &self.root_pin)
    }
    fn install_dir(&self, m: &InstallManifest) -> Result<PathBuf, ToolError> {
        validate(m)?;
        self.verify_root()?;
        let path = self.root.join(&m.tool).join(&m.version).join(&m.target);
        safe_components(&path)?;
        Ok(path)
    }
    /// Imports a regular executable using a constant 64 KiB copy/hash buffer.
    /// Existing versions and the selected active version survive any verification failure.
    pub fn import(
        &mut self,
        source: &Path,
        manifest: InstallManifest,
    ) -> Result<InstalledTool, ToolError> {
        let destination = self.install_dir(&manifest)?;
        selected_path(source)?;
        let source = source.canonicalize()?;
        safe_components(&source)?;
        let _install_lock = self.acquire_install_lock()?;
        self.verify_install_inventory_capacity()?;
        let source_meta = fs::metadata(&source)?;
        if !source_meta.is_file() {
            return Err(ToolError::UnsafePath);
        }
        if source_meta.len() == 0 || source_meta.len() > MAX_DOWNLOAD_BYTES {
            return Err(ToolError::DownloadLimit);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if source_meta.permissions().mode() & 0o111 == 0 {
                return Err(ToolError::UnsafePath);
            }
        }
        let parent = destination.parent().ok_or(ToolError::UnsafePath)?;
        fs::create_dir_all(parent)?;
        safe_components(parent)?;
        fs::create_dir(&destination)?; // create-new, never overwrite an existing version
        let executable = destination.join(&manifest.executable);
        let result = (|| {
            let mut input = File::open(&source)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&executable)?;
            let mut hash = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            let mut bytes = 0u64;
            loop {
                let n = input.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
                output.write_all(&buffer[..n])?;
                bytes += n as u64;
                if bytes > MAX_DOWNLOAD_BYTES {
                    return Err(ToolError::DownloadLimit);
                }
            }
            if format!("{:x}", hash.finalize()) != manifest.sha256.to_ascii_lowercase() {
                return Err(ToolError::HashMismatch);
            }
            output.sync_all()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))?;
            }
            let data = serde_json::to_vec(&manifest)?;
            let mut record = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join("manifest.json"))?;
            record.write_all(&data)?;
            record.sync_all()?;
            self.db.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![manifest.tool,manifest.version,manifest.target])?;
            Ok(InstalledTool {
                manifest: manifest.clone(),
                executable: executable.clone(),
                bytes,
            })
        })();
        if result.is_err() {
            // Only files this invocation created, never recursive deletion.
            let _ = fs::remove_file(&executable);
            let _ = fs::remove_file(destination.join("manifest.json"));
            let _ = fs::remove_dir(&destination);
        }
        result
    }
    pub fn active(&self, tool: &str) -> Result<Option<(String, String)>, ToolError> {
        Ok(self
            .db
            .query_row(
                "SELECT version,target FROM active WHERE tool=?1",
                [tool],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }
    /// Removes only the recorded binary and manifest of an inactive managed install.
    /// Foreign files make removal fail rather than being swept up recursively.
    pub fn remove(&mut self, manifest: &InstallManifest) -> Result<(), ToolError> {
        let directory = self.install_dir(manifest)?;
        if self.active(&manifest.tool)? == Some((manifest.version.clone(), manifest.target.clone()))
        {
            return Err(ToolError::ActiveVersion);
        }
        if let Some(package) = self.package_installed(manifest)? {
            return self.remove_package(&package.package);
        }
        let _install_lock = self.acquire_install_lock()?;
        if self.active(&manifest.tool)? == Some((manifest.version.clone(), manifest.target.clone()))
        {
            return Err(ToolError::ActiveVersion);
        }
        let record = directory.join("manifest.json");
        safe_components(&record)?;
        if fs::metadata(&record)?.len() > MANIFEST_LIMIT {
            return Err(ToolError::InvalidManifest);
        }
        let stored: InstallManifest = serde_json::from_reader(File::open(&record)?)?;
        validate(&stored)?;
        if serde_json::to_vec(&stored)? != serde_json::to_vec(manifest)? {
            return Err(ToolError::InvalidManifest);
        }
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            if entry.file_name() != std::ffi::OsStr::new(&manifest.executable)
                && entry.file_name() != std::ffi::OsStr::new("manifest.json")
            {
                return Err(ToolError::UnexpectedFiles);
            }
            safe_components(&entry.path())?;
            if !entry.file_type()?.is_file() {
                return Err(ToolError::UnsafePath);
            }
        }
        fs::remove_file(directory.join(&manifest.executable))?;
        fs::remove_file(record)?;
        fs::remove_dir(directory)?;
        Ok(())
    }
}

/// Shared bounded stable-release parser for external runtime discovery.
pub(crate) fn verify_deno_output(stdout: &[u8], version: &str, target: &str) -> bool {
    deno_version::verified_deno_version(stdout, version, target).is_ok()
}

fn verify_root_pin(root: &Path, root_pin: &File) -> Result<(), ToolError> {
    safe_components(root)?;
    let marker = root.join(".dlpad-owner");
    safe_components(&marker)?;
    if !native_files::same_directory(root, root_pin)
        || root.canonicalize()? != root
        || fs::metadata(&marker)?.len() != OWNER.len() as u64
        || fs::read(marker)? != OWNER
    {
        return Err(ToolError::UnsafePath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(dir: &Path) -> (PathBuf, InstallManifest) {
        let source = dir.join("fixture-tool");
        fs::write(&source, b"synthetic executable").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let manifest = InstallManifest {
            tool: "fixture".into(),
            version: "1".into(),
            target: "test".into(),
            source_url: "fixture://local".into(),
            sha256: format!("{:x}", Sha256::digest(b"synthetic executable")),
            license: "synthetic fixture only".into(),
            ownership: Ownership::Managed,
            executable: "fixture-tool".into(),
        };
        (source, manifest)
    }
    #[test]
    fn verification_rollback_and_active_removal_guard() {
        let dir = tempfile::tempdir().unwrap();
        let (source, m) = fixture(dir.path());
        let mut manager = ToolManager::new(&dir.path().join("tools")).unwrap();
        let installed = manager.import(&source, m.clone()).unwrap();
        assert_eq!(installed.bytes, 20);
        assert!(manager.import(&source, m.clone()).is_err());
        assert!(matches!(manager.remove(&m), Err(ToolError::ActiveVersion)));
        let mut bad = m.clone();
        bad.version = "2".into();
        bad.sha256 = "0".repeat(64);
        assert!(matches!(
            manager.import(&source, bad),
            Err(ToolError::HashMismatch)
        ));
        assert_eq!(
            manager.active("fixture").unwrap(),
            Some(("1".into(), "test".into()))
        );
        let mut next = m.clone();
        next.version = "2".into();
        manager.import(&source, next).unwrap();
        manager.remove(&m).unwrap();
        assert!(source.exists());
    }
    #[test]
    fn external_escapes_and_user_media_are_never_removed() {
        let dir = tempfile::tempdir().unwrap();
        let (source, mut m) = fixture(dir.path());
        let mut manager = ToolManager::new(&dir.path().join("tools")).unwrap();
        m.ownership = Ownership::External;
        assert!(matches!(
            manager.remove(&m),
            Err(ToolError::ExternalOwnership)
        ));
        m.ownership = Ownership::Managed;
        m.version = "../media".into();
        assert!(matches!(
            manager.import(&source, m),
            Err(ToolError::InvalidManifest)
        ));
        let media = dir.path().join("media");
        fs::create_dir(&media).unwrap();
        fs::write(media.join("video.mp4"), b"fixture").unwrap();
        assert!(ToolManager::new(&media).is_err());
        assert!(media.join("video.mp4").exists());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_roots_and_foreign_install_files_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (source, m) = fixture(dir.path());
        std::os::unix::fs::symlink(dir.path(), dir.path().join("linked")).unwrap();
        assert!(ToolManager::new(&dir.path().join("linked")).is_err());
        let mut manager = ToolManager::new(&dir.path().join("tools")).unwrap();
        let installed = manager.import(&source, m.clone()).unwrap();
        let mut next = m.clone();
        next.version = "2".into();
        manager.import(&source, next).unwrap();
        let media = installed.executable.parent().unwrap().join("video.mp4");
        fs::write(&media, b"keep").unwrap();
        assert!(matches!(
            manager.remove(&m),
            Err(ToolError::UnexpectedFiles)
        ));
        assert!(media.exists());
        assert!(installed.executable.exists());
    }
    #[cfg(unix)]
    #[test]
    fn alias_parent_is_accepted_but_selected_root_symlink_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir(&real).unwrap();
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        let (source, manifest) = fixture(&real);
        let mut manager = ToolManager::new(&alias.join("tools")).unwrap();
        manager
            .import(&alias.join(source.file_name().unwrap()), manifest)
            .unwrap();
        let linked_root = dir.path().join("linked-root");
        std::os::unix::fs::symlink(real.join("tools"), &linked_root).unwrap();
        assert!(matches!(
            ToolManager::new(&linked_root),
            Err(ToolError::UnsafePath)
        ));
    }
}

pub mod tool_update_flow;
