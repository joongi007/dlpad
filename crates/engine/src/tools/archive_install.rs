//! Local archive installs remain inactive. Publication is create-only, not atomic.
use super::{
    archive_download::VerifiedNoticeFile,
    direct_download::OwnedDirectory,
    native_files,
    staging_recovery::{FileEvidence, file_evidence},
    *,
};
use std::{
    io::{Seek, SeekFrom},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<(), ToolError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ToolError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(ToolError::TimedOut);
    }
    Ok(())
}
fn verify_bytes(
    file: &File,
    bytes: u64,
    digest: &str,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), ToolError> {
    if !file.metadata()?.is_file() || file.metadata()?.len() != bytes {
        return Err(ToolError::UnexpectedSize);
    }
    let mut source = file;
    source.seek(SeekFrom::Start(0))?;
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buf = [0u8; 64 * 1024];
    loop {
        checkpoint(cancel, deadline)?;
        let n = source.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if total > bytes {
            return Err(ToolError::UnexpectedSize);
        }
        hash.update(&buf[..n]);
    }
    if total != bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", hash.finalize()) != digest.to_ascii_lowercase() {
        return Err(ToolError::HashMismatch);
    }
    Ok(())
}
fn source_valid(path: &Path, file: &File) -> Result<(), ToolError> {
    selected_path(path)?;
    if !native_files::same_file(path, file) {
        return Err(ToolError::UnsafePath);
    }
    Ok(())
}
fn record_file(
    manager: &ToolManager,
    stage: &OwnedDirectory,
    main: &str,
    evidence: &mut Vec<FileEvidence>,
    name: &str,
    bytes: u64,
    digest: String,
) -> Result<(), ToolError> {
    stage.check()?;
    let file = native_files::open_regular(&stage.path.join(name))?;
    evidence.push(file_evidence(name, &file, bytes, digest)?);
    manager.record_package_staging(&stage.path, &stage.identity, main, evidence.clone())
}
fn validate_notice_capabilities(
    package: &ArchivePackage,
    notices: &[VerifiedNoticeFile],
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), ToolError> {
    if notices.len() != package.notices.len() || notices.len() > 2 {
        return Err(ToolError::InvalidManifest);
    }
    for (index, notice) in notices.iter().enumerate() {
        checkpoint(cancel, deadline)?;
        if notices[..index]
            .iter()
            .any(|n| n.component == notice.component)
            || !package
                .notices
                .iter()
                .any(|n| n.component == notice.component)
        {
            return Err(ToolError::InvalidManifest);
        }
        let component = package
            .components
            .iter()
            .find(|c| c.name == notice.component && c.role == ArchiveComponentRole::Notice)
            .ok_or(ToolError::InvalidManifest)?;
        if component.bytes == 0 || component.bytes > 1024 * 1024 {
            return Err(ToolError::InvalidManifest);
        }
        source_valid(&notice.path, &notice.file)?;
        verify_bytes(
            &notice.file,
            component.bytes,
            &component.sha256,
            cancel,
            deadline,
        )?;
        source_valid(&notice.path, &notice.file)?;
    }
    Ok(())
}
fn copy_notice(
    notice: &VerifiedNoticeFile,
    component: &ArchiveComponent,
    output: &mut File,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<u64, ToolError> {
    source_valid(&notice.path, &notice.file)?;
    verify_bytes(
        &notice.file,
        component.bytes,
        &component.sha256,
        cancel,
        deadline,
    )?;
    let mut input = &notice.file;
    input.seek(SeekFrom::Start(0))?;
    let mut copied_hash = Sha256::new();
    let mut total = 0u64;
    let mut buf = [0u8; 64 * 1024];
    loop {
        checkpoint(cancel, deadline)?;
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if total > component.bytes {
            return Err(ToolError::UnexpectedSize);
        }
        copied_hash.update(&buf[..n]);
        output.write_all(&buf[..n])?;
    }
    if total != component.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    output.sync_all()?;
    // OwnedDirectory.file is write-only; hash the exact bytes actually written.
    if format!("{:x}", copied_hash.finalize()) != component.sha256.to_ascii_lowercase() {
        return Err(ToolError::HashMismatch);
    }
    source_valid(&notice.path, &notice.file)?;
    verify_bytes(
        &notice.file,
        component.bytes,
        &component.sha256,
        cancel,
        deadline,
    )?;
    checkpoint(cancel, deadline)?;
    Ok(total)
}
impl ToolManager {
    /// Caller-selected local archive, never source deletion or implicit activation.
    /// Runtime execution/version proof and final-publication crash recovery are separate gates.
    pub fn install_verified_archive(
        &mut self,
        source: &Path,
        package: ArchivePackage,
        cancel: &AtomicBool,
    ) -> Result<InstalledPackage, ToolError> {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(120))
            .ok_or(ToolError::TimedOut)?;
        checkpoint(cancel, deadline)?;
        package.validate()?;
        if !package.notices.is_empty() {
            return Err(ToolError::InvalidManifest);
        }
        self.verify_root()?;
        selected_path(source)?;
        let source_path = source.canonicalize()?;
        let input = native_files::open_regular(&source_path)?;
        let _install_lock = self.acquire_install_lock()?;
        self.install_verified_archive_from_file(&source_path, &input, package, cancel, deadline)
    }
    /// Tools-private capability adapter. Caller retains the same root install lock.
    /// Uses the caller's absolute deadline; never reacquires the lock or reopens source.
    pub(super) fn install_verified_archive_from_file(
        &mut self,
        source_path: &Path,
        input: &File,
        package: ArchivePackage,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<InstalledPackage, ToolError> {
        self.install_verified_archive_with_notices_from_file(
            source_path,
            input,
            package,
            cancel,
            deadline,
            &[],
        )
    }
    // Exact tools-private capability signature coordinated with root.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn install_verified_archive_with_notices_from_file(
        &mut self,
        source_path: &Path,
        input: &File,
        package: ArchivePackage,
        cancel: &AtomicBool,
        deadline: Instant,
        notices: &[VerifiedNoticeFile],
    ) -> Result<InstalledPackage, ToolError> {
        checkpoint(cancel, deadline)?;
        package.validate()?;
        validate_notice_capabilities(&package, notices, cancel, deadline)?;
        self.verify_root()?;
        if self.installed(&package.tool)?.len() >= 128 {
            return Err(ToolError::InventoryLimit);
        }
        selected_path(source_path)?;
        source_valid(source_path, input)?;
        verify_bytes(
            input,
            package.archive_bytes,
            &package.archive_sha256,
            cancel,
            deadline,
        )?;
        let main = package
            .components
            .iter()
            .find(|c| c.role == ArchiveComponentRole::Executable)
            .ok_or(ToolError::InvalidManifest)?;
        let manifest = InstallManifest {
            tool: package.tool.clone(),
            version: package.tool_version.clone(),
            target: package.target.clone(),
            source_url: package.source_url.clone(),
            sha256: main.sha256.clone(),
            license: package.license.clone(),
            ownership: package.ownership,
            executable: main.name.clone(),
        };
        let destination = self.install_dir(&manifest)?;
        match fs::symlink_metadata(&destination) {
            Ok(_) => return Err(ToolError::UnexpectedFiles),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let package_json = serde_json::to_vec(&package)?;
        let manifest_json = serde_json::to_vec(&manifest)?;
        if package_json.len() as u64 > MANIFEST_LIMIT || manifest_json.len() as u64 > MANIFEST_LIMIT
        {
            return Err(ToolError::InvalidManifest);
        }
        let reserve = package
            .components
            .iter()
            .try_fold((package_json.len() + manifest_json.len()) as u64, |n, c| {
                n.checked_add(c.bytes).ok_or(ToolError::DownloadLimit)
            })?;
        self.verify_install_inventory_capacity()?;
        self.reserve_staging(reserve)?;
        let stage_path = self.root.join(format!(
            ".staging-archive-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut stage = OwnedDirectory::create(stage_path.clone())?;
        let mut evidence = Vec::with_capacity(10);
        let mut records: Vec<(String, u64, String)> = Vec::with_capacity(10);
        for component in &package.components {
            checkpoint(cancel, deadline)?;
            source_valid(source_path, input)?;
            let mut output = stage.file(&component.name)?;
            let bytes = if let Some(notice) = notices.iter().find(|n| n.component == component.name)
            {
                copy_notice(notice, component, &mut output, cancel, deadline)?
            } else {
                match &package.layout {
                    None | Some(ArchiveLayout::Zip { .. }) => extract_verified_zip(
                        input,
                        &package,
                        &component.name,
                        &mut output,
                        cancel,
                        deadline,
                    )?,
                    Some(ArchiveLayout::TarXz { root, mappings }) => {
                        let maps: Vec<_> = mappings
                            .iter()
                            .map(|mapping| {
                                let mut declared = package
                                    .components
                                    .iter()
                                    .find(|c| c.name == mapping.component)
                                    .ok_or(ToolError::InvalidManifest)?
                                    .clone();
                                declared.sha256 = declared.sha256.to_ascii_lowercase();
                                Ok(TarComponentMapping {
                                    member: mapping.member.clone(),
                                    component: declared,
                                })
                            })
                            .collect::<Result<Vec<_>, ToolError>>()?;
                        let limits = TarLimits {
                            max_entries: 4096,
                            max_total_bytes: 768 * 1024 * 1024,
                            max_selected_bytes: 384 * 1024 * 1024,
                            max_stream_bytes: 800 * 1024 * 1024,
                        };
                        extract_verified_tar_xz(
                            input,
                            TarXzSourceSpec {
                                bytes: package.archive_bytes,
                                sha256: &package.archive_sha256.to_ascii_lowercase(),
                            },
                            TarExtractionRequest {
                                root,
                                mappings: &maps,
                                selected: &component.name,
                                limits: &limits,
                            },
                            &mut output,
                            cancel,
                            deadline,
                        )?
                    }
                }
            };
            output.sync_all()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                output.set_permissions(fs::Permissions::from_mode(
                    if component.role == ArchiveComponentRole::Notice {
                        0o600
                    } else {
                        0o700
                    },
                ))?;
                output.sync_all()?;
            }
            drop(output);
            stage.seal_executable(&component.name)?;
            record_file(
                self,
                &stage,
                &main.name,
                &mut evidence,
                &component.name,
                bytes,
                component.sha256.clone(),
            )?;
            records.push((component.name.clone(), bytes, component.sha256.clone()));
        }
        for (name, bytes) in [
            ("archive-package.json", package_json),
            ("manifest.json", manifest_json),
        ] {
            checkpoint(cancel, deadline)?;
            let mut output = stage.file(name)?;
            output.write_all(&bytes)?;
            output.sync_all()?;
            drop(output);
            stage.seal_executable(name)?;
            let digest = format!("{:x}", Sha256::digest(&bytes));
            record_file(
                self,
                &stage,
                &main.name,
                &mut evidence,
                name,
                bytes.len() as u64,
                digest.clone(),
            )?;
            records.push((name.into(), bytes.len() as u64, digest));
        }
        source_valid(source_path, input)?;
        verify_bytes(
            input,
            package.archive_bytes,
            &package.archive_sha256,
            cancel,
            deadline,
        )?;
        let parent = destination.parent().ok_or(ToolError::UnsafePath)?;
        safe_components(parent)?;
        fs::create_dir_all(parent)?;
        let parent_pin = native_files::open_directory(parent)?;
        self.verify_root()?;
        stage.check()?;
        validate_notice_capabilities(&package, notices, cancel, deadline)?;
        let mut published = OwnedDirectory::create(destination.clone())?;
        for (name, bytes, digest) in &records {
            checkpoint(cancel, deadline)?;
            stage.check()?;
            published.check()?;
            if !native_files::same_directory(parent, &parent_pin) {
                return Err(ToolError::UnsafePath);
            }
            let path = stage.path.join(name);
            let file = native_files::open_regular(&path)?;
            verify_bytes(&file, *bytes, digest, cancel, deadline)?;
            published.link(&path, name)?;
            let final_file = native_files::open_regular(&destination.join(name))?;
            verify_bytes(&final_file, *bytes, digest, cancel, deadline)?;
        }
        #[cfg(unix)]
        File::open(&destination)?.sync_all()?;
        stage.check()?;
        published.check()?;
        source_valid(source_path, input)?;
        // Remove only original staging links before strict nlink1 package verification.
        drop(stage);
        checkpoint(cancel, deadline)?;
        validate_notice_capabilities(&package, notices, cancel, deadline)?;
        let installed = self.verify_package(&destination, Some(&package))?;
        validate_notice_capabilities(&package, notices, cancel, deadline)?;
        published.check()?;
        published.keep = true;
        // A DB error here preserves a complete inactive package, never prior active mutation.
        self.forget_staging(&stage_path)?;
        Ok(installed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(multi: bool) -> (tempfile::TempDir, PathBuf, ArchivePackage) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("user-source.zip");
        let members: Vec<(&str, &[u8], ArchiveComponentRole)> = if multi {
            vec![
                (
                    "deno",
                    b"synthetic executable",
                    ArchiveComponentRole::Executable,
                ),
                (
                    "helper",
                    b"synthetic helper",
                    ArchiveComponentRole::Auxiliary,
                ),
                ("LICENSE", b"synthetic notice", ArchiveComponentRole::Notice),
            ]
        } else {
            vec![(
                "deno",
                b"synthetic executable",
                ArchiveComponentRole::Executable,
            )]
        };
        let mut z = Vec::new();
        let mut writer = rawzip::ZipArchiveWriter::new(&mut z);
        for (name, data, _) in &members {
            let (mut entry, config) = writer.new_file(*name).start().unwrap();
            let mut output = config.wrap(&mut entry);
            output.write_all(data).unwrap();
            let (_, desc) = output.finish().unwrap();
            entry.finish(desc).unwrap();
        }
        writer.finish().unwrap();
        fs::write(&source, &z).unwrap();
        fs::write(temp.path().join("foreign-sentinel"), b"preserve").unwrap();
        let package = ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "deno".into(),
            tool_version: "fixture-1".into(),
            target: "native-fixture".into(),
            source_url: "https://example.invalid/runtime.zip".into(),
            license: "synthetic-MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: z.len() as u64,
            archive_sha256: format!("{:x}", Sha256::digest(&z)),
            components: members
                .into_iter()
                .map(|(name, data, role)| ArchiveComponent {
                    name: name.into(),
                    bytes: data.len() as u64,
                    sha256: format!("{:x}", Sha256::digest(data)),
                    role,
                })
                .collect(),
        };
        (temp, source, package)
    }
    #[test]
    fn stored_and_multicomponent_install_are_inactive_until_explicit_selection() {
        for multi in [false, true] {
            let (temp, source, package) = fixture(multi);
            let source_bytes = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let installed = manager
                .install_verified_archive(&source, package.clone(), &AtomicBool::new(false))
                .unwrap();
            assert!(manager.active("deno").unwrap().is_none());
            assert_eq!(
                fs::read_dir(&installed.directory).unwrap().count(),
                package.components.len() + 2
            );
            let selected = manager.select_package(&installed.package).unwrap();
            assert_eq!(selected.directory, installed.directory);
            assert!(manager.active("deno").unwrap().is_some());
            assert_eq!(fs::read(&source).unwrap(), source_bytes);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
        }
    }
    #[test]
    fn cancellation_hash_and_existing_destination_preserve_active_source() {
        let (temp, source, package) = fixture(false);
        let source_bytes = fs::read(&source).unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        assert!(matches!(
            manager.install_verified_archive(&source, package.clone(), &AtomicBool::new(true)),
            Err(ToolError::Cancelled)
        ));
        let mut bad = package.clone();
        bad.archive_sha256 = "a".repeat(64);
        assert!(matches!(
            manager.install_verified_archive(&source, bad, &AtomicBool::new(false)),
            Err(ToolError::HashMismatch)
        ));
        let installed = manager
            .install_verified_archive(&source, package.clone(), &AtomicBool::new(false))
            .unwrap();
        manager.select_package(&installed.package).unwrap();
        let active = manager.active("deno").unwrap();
        assert!(matches!(
            manager.install_verified_archive(&source, package, &AtomicBool::new(false)),
            Err(ToolError::UnexpectedFiles)
        ));
        assert_eq!(manager.active("deno").unwrap(), active);
        assert_eq!(fs::read(&source).unwrap(), source_bytes);
    }
    #[test]
    fn component_failure_and_checkpoint_db_failure_preserve_foreign() {
        let (temp, source, package) = fixture(true);
        let source_bytes = fs::read(&source).unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let mut bad = package.clone();
        bad.components[1].sha256 = "b".repeat(64);
        assert!(matches!(
            manager.install_verified_archive(&source, bad, &AtomicBool::new(false)),
            Err(ToolError::HashMismatch)
        ));
        assert!(manager.active("deno").unwrap().is_none());
        manager.db.execute_batch("CREATE TRIGGER reject_package_checkpoint BEFORE INSERT ON install_staging BEGIN SELECT RAISE(ABORT,'synthetic checkpoint failure'); END;").unwrap();
        assert!(matches!(
            manager.install_verified_archive(&source, package, &AtomicBool::new(false)),
            Err(ToolError::Database(_))
        ));
        assert_eq!(fs::read(&source).unwrap(), source_bytes);
        assert_eq!(
            fs::read(temp.path().join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
        assert!(manager.active("deno").unwrap().is_none());
    }
    #[cfg(unix)]
    #[test]
    fn source_symlink_is_rejected_without_mutation() {
        let (temp, source, package) = fixture(false);
        let alias = temp.path().join("alias.zip");
        std::os::unix::fs::symlink(&source, &alias).unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        assert!(matches!(
            manager.install_verified_archive(&alias, package, &AtomicBool::new(false)),
            Err(ToolError::UnsafePath)
        ));
        assert!(source.exists());
        assert!(
            fs::symlink_metadata(alias)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    struct ArchiveMemoryTransport {
        bytes: Vec<u8>,
        fail: bool,
        calls: usize,
    }
    impl super::super::direct_download::DownloadTransport for ArchiveMemoryTransport {
        fn open(&mut self, _: &str, _: Duration) -> Result<Box<dyn Read>, ToolError> {
            self.calls += 1;
            if self.fail {
                return Err(ToolError::Http);
            }
            Ok(Box::new(std::io::Cursor::new(self.bytes.clone())))
        }
    }
    fn transport(bytes: Vec<u8>) -> ArchiveMemoryTransport {
        ArchiveMemoryTransport {
            bytes,
            fail: false,
            calls: 0,
        }
    }
    fn no_stages(manager: &ToolManager) {
        assert!(!fs::read_dir(&manager.root).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".staging-")
        }));
    }
    #[test]
    fn https_memory_package_exact_preserves_prior_active_and_source() {
        for multi in [false, true] {
            let (temp, source, mut package) = fixture(multi);
            let original = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = manager
                .install_verified_archive(&source, package.clone(), &AtomicBool::new(false))
                .unwrap();
            manager.select_package(&prior.package).unwrap();
            let active = manager.active("deno").unwrap();
            package.tool_version = "fixture-2".into();
            let mut stream = transport(original.clone());
            let result = manager
                .install_archive_download(
                    package.clone(),
                    &std::sync::Arc::new(AtomicBool::new(false)),
                    &mut stream,
                    Duration::from_secs(10),
                )
                .unwrap();
            assert_eq!(stream.calls, 1);
            assert_eq!(manager.active("deno").unwrap(), active);
            assert_eq!(
                fs::read_dir(&result.directory).unwrap().count(),
                package.components.len() + 2
            );
            assert_eq!(fs::read(&source).unwrap(), original);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
            no_stages(&manager);
        }
    }
    #[test]
    fn https_memory_failures_preserve_prior_active_and_external_bytes() {
        for case in [
            "short",
            "extra",
            "error",
            "hash",
            "cancel",
            "deadline",
            "checkpoint",
        ] {
            let (temp, source, mut package) = fixture(true);
            let original = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = manager
                .install_verified_archive(&source, package.clone(), &AtomicBool::new(false))
                .unwrap();
            manager.select_package(&prior.package).unwrap();
            let active = manager.active("deno").unwrap();
            package.tool_version = "fixture-2".into();
            let mut stream = transport(original.clone());
            match case {
                "short" => {
                    stream.bytes.pop();
                }
                "extra" => stream.bytes.push(0),
                "error" => stream.fail = true,
                "hash" => package.archive_sha256 = "a".repeat(64),
                "checkpoint" => {
                    manager.db.execute_batch("CREATE TRIGGER deny_https_checkpoint BEFORE INSERT ON install_staging BEGIN SELECT RAISE(ABORT,'synthetic checkpoint failure'); END;").unwrap();
                }
                _ => {}
            }
            let cancel = std::sync::Arc::new(AtomicBool::new(case == "cancel"));
            let result = manager.install_archive_download(
                package,
                &cancel,
                &mut stream,
                if case == "deadline" {
                    Duration::ZERO
                } else {
                    Duration::from_secs(10)
                },
            );
            assert!(
                matches!(
                    (case, result),
                    ("short", Err(ToolError::UnexpectedSize))
                        | ("extra", Err(ToolError::DownloadLimit))
                        | ("error", Err(ToolError::Http))
                        | ("hash", Err(ToolError::HashMismatch))
                        | ("cancel", Err(ToolError::Cancelled))
                        | ("deadline", Err(ToolError::TimedOut))
                        | ("checkpoint", Err(ToolError::Database(_)))
                ),
                "wrong failure outcome for {case}"
            );
            assert_eq!(manager.active("deno").unwrap(), active);
            assert_eq!(fs::read(&source).unwrap(), original);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
            assert!(!manager.root.join("deno/fixture-2/native-fixture").exists());
            no_stages(&manager);
        }
    }
    #[test]
    fn https_memory_source_policy_rejects_before_transport() {
        for url in [
            "http://example.invalid/runtime.zip",
            "https://user:secret@example.invalid/runtime.zip",
        ] {
            let (temp, source, mut package) = fixture(false);
            let original = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            package.source_url = url.into();
            let mut stream = transport(original.clone());
            assert!(matches!(
                manager.install_archive_download(
                    package,
                    &std::sync::Arc::new(AtomicBool::new(false)),
                    &mut stream,
                    Duration::from_secs(10)
                ),
                Err(ToolError::InvalidManifest)
            ));
            assert_eq!(stream.calls, 0);
            assert_eq!(fs::read(source).unwrap(), original);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
            no_stages(&manager);
        }
    }
    struct InterruptedArchiveReader {
        deadline: Instant,
        data: std::io::Cursor<Vec<u8>>,
        mode: &'static str,
        cancel: std::sync::Arc<AtomicBool>,
        reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Read for InterruptedArchiveReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let call = self.reads.fetch_add(1, Ordering::SeqCst);
            if self.mode == "error" && call == 1 {
                return Err(io::Error::other("synthetic midstream failure"));
            }
            let count = buffer.len().min(8);
            let n = self.data.read(&mut buffer[..count])?;
            if self.mode == "cancel" {
                self.cancel.store(true, Ordering::Release);
            } else if self.mode == "slow" {
                std::thread::sleep(
                    self.deadline.saturating_duration_since(Instant::now())
                        + Duration::from_millis(20),
                );
            }
            Ok(n)
        }
    }
    struct InterruptedArchiveTransport {
        data: Vec<u8>,
        mode: &'static str,
        cancel: std::sync::Arc<AtomicBool>,
        reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    impl super::super::direct_download::DownloadTransport for InterruptedArchiveTransport {
        fn open(&mut self, _: &str, timeout: Duration) -> Result<Box<dyn Read>, ToolError> {
            Ok(Box::new(InterruptedArchiveReader {
                deadline: Instant::now()
                    .checked_add(timeout)
                    .ok_or(ToolError::TimedOut)?,
                data: std::io::Cursor::new(self.data.clone()),
                mode: self.mode,
                cancel: self.cancel.clone(),
                reads: self.reads.clone(),
            }))
        }
    }
    #[test]
    fn https_midstream_error_cancel_and_deadline_preserve_originals() {
        for mode in ["error", "cancel", "slow"] {
            let (temp, source, mut package) = fixture(true);
            let original = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = manager
                .install_verified_archive(&source, package.clone(), &AtomicBool::new(false))
                .unwrap();
            manager.select_package(&prior.package).unwrap();
            let active = manager.active("deno").unwrap();
            package.tool_version = "fixture-2".into();
            let cancel = std::sync::Arc::new(AtomicBool::new(false));
            let reads = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut transport = InterruptedArchiveTransport {
                data: original.clone(),
                mode,
                cancel: cancel.clone(),
                reads: reads.clone(),
            };
            let result = manager.install_archive_download(
                package,
                &cancel,
                &mut transport,
                if mode == "slow" {
                    Duration::from_secs(2)
                } else {
                    Duration::from_secs(10)
                },
            );
            assert!(
                matches!((&result,mode),(Err(ToolError::Io(error)),"error") if error.kind()==io::ErrorKind::Other)
                    || matches!(
                        (&result, mode),
                        (Err(ToolError::Cancelled), "cancel") | (Err(ToolError::TimedOut), "slow")
                    ),
                "wrong midstream result: {result:?}"
            );
            assert_eq!(
                reads.load(Ordering::Acquire),
                if mode == "error" { 2 } else { 1 },
                "failure must occur during actual read"
            );
            assert_eq!(manager.active("deno").unwrap(), active);
            assert_eq!(fs::read(&source).unwrap(), original);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
            assert!(!manager.root.join("deno/fixture-2/native-fixture").exists());
            no_stages(&manager);
        }
    }
    // Add INSIDE existing archive_install::tests. Reuses actual existing fixture(false).
    fn notice_fixture() -> (
        tempfile::TempDir,
        PathBuf,
        ArchivePackage,
        Vec<VerifiedNoticeFile>,
    ) {
        let (temp, source, mut package) = fixture(false);
        let bytes = b"synthetic pinned notice\n";
        let path = temp.path().join("original-notice.txt");
        fs::write(&path, bytes).unwrap();
        package.version = 3;
        package.components.push(ArchiveComponent {
            name: "LICENSE".into(),
            bytes: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(bytes)),
            role: ArchiveComponentRole::Notice,
        });
        package.notices.push(super::super::PinnedArchiveNotice {
            component: "LICENSE".into(),
            source_url: "https://example.invalid/revision/LICENSE".into(),
            revision: "d".repeat(40),
        });
        let file = native_files::open_regular(&path).unwrap();
        (
            temp,
            source,
            package,
            vec![VerifiedNoticeFile {
                component: "LICENSE".into(),
                path,
                file,
            }],
        )
    }
    #[test]
    fn external_notices_local_public_install_rejected_before_any_publication() {
        let (temp, source, package, _caps) = notice_fixture();
        let before = fs::read(&source).unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        assert!(matches!(
            manager.install_verified_archive(&source, package, &AtomicBool::new(false)),
            Err(ToolError::InvalidManifest)
        ));
        assert!(manager.active("deno").unwrap().is_none());
        assert_eq!(fs::read(&source).unwrap(), before);
    }
    #[test]
    fn original_notice_copy_is_inactive_exact_and_all_originals_are_preserved() {
        let (temp, source, package, caps) = notice_fixture();
        let before = fs::read(&source).unwrap();
        let notice_before = fs::read(&caps[0].path).unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let input = native_files::open_regular(&source).unwrap();
        let _lock = manager.acquire_install_lock().unwrap();
        let installed = manager
            .install_verified_archive_with_notices_from_file(
                &source,
                &input,
                package,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(10),
                &caps,
            )
            .unwrap();
        assert_eq!(
            fs::read(installed.directory.join("LICENSE")).unwrap(),
            notice_before
        );
        assert!(manager.active("deno").unwrap().is_none());
        assert_eq!(fs::read(source).unwrap(), before);
        assert_eq!(fs::read(&caps[0].path).unwrap(), notice_before);
        assert_eq!(
            fs::read(temp.path().join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn notice_missing_wrong_name_duplicate_hash_cancel_and_deadline_fail_before_publish() {
        for case in 0..8 {
            let (temp, source, package, mut caps) = notice_fixture();
            let before = fs::read(&source).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            if case == 0 {
                caps.clear();
            }
            if case == 1 {
                caps[0].component = "OTHER".into();
            }
            if case == 2 {
                caps.push(VerifiedNoticeFile {
                    component: caps[0].component.clone(),
                    path: caps[0].path.clone(),
                    file: caps[0].file.try_clone().unwrap(),
                });
            }
            if case == 3 {
                fs::write(&caps[0].path, b"tampered notice").unwrap();
            }
            if case == 6 {
                let moved = temp.path().join("original-notice-moved");
                let retained = fs::read(&caps[0].path).unwrap();
                fs::rename(&caps[0].path, &moved).unwrap();
                fs::write(&caps[0].path, retained).unwrap();
            }
            if case == 7 {
                let foreign = temp.path().join("foreign-notice");
                fs::write(&foreign, fs::read(&caps[0].path).unwrap()).unwrap();
                caps[0].path = foreign;
            }
            let input = native_files::open_regular(&source).unwrap();
            let _lock = manager.acquire_install_lock().unwrap();
            let cancel = AtomicBool::new(case == 4);
            let deadline = if case == 5 {
                Instant::now()
            } else {
                Instant::now() + Duration::from_secs(10)
            };
            assert!(
                manager
                    .install_verified_archive_with_notices_from_file(
                        &source, &input, package, &cancel, deadline, &caps
                    )
                    .is_err()
            );
            assert!(manager.active("deno").unwrap().is_none());
            assert_eq!(fs::read(source).unwrap(), before);
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
        }
    }
    // Fragment INSIDE existing archive_install::tests; source/Cargo changes root only.
    struct NoticeUrlTransport {
        bodies: std::collections::BTreeMap<String, Vec<u8>>,
        notice_url: String,
        mode: &'static str,
        cancel: std::sync::Arc<AtomicBool>,
        opened: Vec<String>,
        notice_reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    struct NoticeRead {
        body: std::io::Cursor<Vec<u8>>,
        mode: &'static str,
        cancel: std::sync::Arc<AtomicBool>,
        first: bool,
        is_notice: bool,
        notice_reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Read for NoticeRead {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let n = self.body.read(output)?;
            if self.is_notice {
                self.notice_reads.fetch_add(1, Ordering::AcqRel);
            }
            if n > 0 && self.mode == "read-mutation" {
                output[0] ^= 1;
            }
            if self.first {
                self.first = false;
                if self.mode == "read-cancel" {
                    self.cancel.store(true, Ordering::Release);
                }
                if self.mode == "read-timeout" {
                    std::thread::sleep(Duration::from_millis(1100));
                }
            }
            Ok(n)
        }
    }
    impl super::super::direct_download::DownloadTransport for NoticeUrlTransport {
        fn open(&mut self, url: &str, _remaining: Duration) -> Result<Box<dyn Read>, ToolError> {
            self.opened.push(url.into());
            let notice = url == self.notice_url;
            if notice && self.mode == "http" {
                return Err(ToolError::Http);
            }
            let mut data = self.bodies.get(url).ok_or(ToolError::Http)?.clone();
            if notice {
                match self.mode {
                    "short" => {
                        data.pop();
                    }
                    "extra" => data.push(0),
                    "hash" => data[0] ^= 1,
                    _ => {}
                }
            }
            Ok(Box::new(NoticeRead {
                body: std::io::Cursor::new(data),
                mode: if notice { self.mode } else { "normal" },
                cancel: self.cancel.clone(),
                first: true,
                is_notice: notice,
                notice_reads: self.notice_reads.clone(),
            }))
        }
    }
    fn notice_transport(
        source: &Path,
        package: &ArchivePackage,
        caps: &[VerifiedNoticeFile],
        cancel: &std::sync::Arc<AtomicBool>,
        mode: &'static str,
    ) -> NoticeUrlTransport {
        let mut bodies = std::collections::BTreeMap::new();
        bodies.insert(package.source_url.clone(), fs::read(source).unwrap());
        for notice in &package.notices {
            let cap = caps
                .iter()
                .find(|c| c.component == notice.component)
                .unwrap();
            bodies.insert(notice.source_url.clone(), fs::read(&cap.path).unwrap());
        }
        NoticeUrlTransport {
            bodies,
            notice_url: package.notices[0].source_url.clone(),
            mode,
            cancel: cancel.clone(),
            opened: vec![],
            notice_reads: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
    #[test]
    fn url_pinned_archive_and_one_or_two_notices_install_inactive_reopen_verify_remove() {
        for two in [false, true] {
            let (temp, source, mut package, mut caps) = notice_fixture();
            if two {
                let bytes = b"synthetic second exact revision notice";
                let path = temp.path().join("notice-two");
                fs::write(&path, bytes).unwrap();
                package.components.push(ArchiveComponent {
                    name: "THIRD_PARTY".into(),
                    bytes: bytes.len() as u64,
                    sha256: format!("{:x}", Sha256::digest(bytes)),
                    role: ArchiveComponentRole::Notice,
                });
                package.notices.push(super::super::PinnedArchiveNotice {
                    component: "THIRD_PARTY".into(),
                    source_url: "https://example.invalid/revision/third-party".into(),
                    revision: "e".repeat(40),
                });
                caps.push(VerifiedNoticeFile {
                    component: "THIRD_PARTY".into(),
                    file: native_files::open_regular(&path).unwrap(),
                    path,
                });
            }
            let original = fs::read(&source).unwrap();
            let notice_originals: Vec<_> =
                caps.iter().map(|c| fs::read(&c.path).unwrap()).collect();
            let cancel = std::sync::Arc::new(AtomicBool::new(false));
            let mut transport = notice_transport(&source, &package, &caps, &cancel, "normal");
            let root = temp.path().join("managed");
            let mut manager = ToolManager::new(&root).unwrap();
            let installed = manager
                .install_archive_download(
                    package.clone(),
                    &cancel,
                    &mut transport,
                    Duration::from_secs(10),
                )
                .unwrap();
            assert_eq!(transport.opened.len(), 1 + package.notices.len());
            assert!(manager.active("deno").unwrap().is_none());
            drop(manager);
            let mut reopened = ToolManager::new(&root).unwrap();
            assert_eq!(
                serde_json::to_vec(
                    &reopened
                        .verify_package(&installed.directory, Some(&package))
                        .unwrap()
                        .package
                )
                .unwrap(),
                serde_json::to_vec(&package).unwrap()
            );
            assert!(
                reopened
                    .package_installed(&installed.main.manifest)
                    .unwrap()
                    .is_some()
            );
            reopened.remove_package(&package).unwrap();
            assert!(!installed.directory.exists());
            assert!(reopened.active("deno").unwrap().is_none());
            assert_eq!(fs::read(source).unwrap(), original);
            for (cap, bytes) in caps.iter().zip(notice_originals) {
                assert_eq!(fs::read(&cap.path).unwrap(), bytes);
            }
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
        }
    }
    #[test]
    fn notice_transfer_failures_preserve_original_inputs_prior_active_and_foreign_destination() {
        for mode in [
            "short",
            "extra",
            "hash",
            "http",
            "read-cancel",
            "read-timeout",
            "read-mutation",
            "pre-cancel",
            "pre-deadline",
            "foreign",
        ] {
            let (temp, source, mut package, caps) = notice_fixture();
            package.tool_version = "fixture-2".into();
            let original = fs::read(&source).unwrap();
            let notice_original = fs::read(&caps[0].path).unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            // Keep the prior fixture guard alive until all original input checks finish.
            let (prior_temp, prior_source, prior_package) = fixture(false);
            let prior = manager
                .install_verified_archive(&prior_source, prior_package, &AtomicBool::new(false))
                .unwrap();
            manager.select_package(&prior.package).unwrap();
            let active = manager.active("deno").unwrap();
            let foreign = manager.root.join("deno/fixture-2/native-fixture");
            if mode == "foreign" {
                fs::create_dir_all(&foreign).unwrap();
                fs::write(foreign.join("foreign"), b"never delete").unwrap();
            }
            let cancel = std::sync::Arc::new(AtomicBool::new(mode == "pre-cancel"));
            let mut transport = notice_transport(&source, &package, &caps, &cancel, mode);
            let timeout = if mode == "pre-deadline" {
                Duration::ZERO
            } else if mode == "read-timeout" {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(10)
            };
            let result =
                manager.install_archive_download(package, &cancel, &mut transport, timeout);
            assert!(
                result.is_err(),
                "unexpected notice transfer success: {mode}"
            );
            if mode == "http" {
                assert!(matches!(result, Err(ToolError::Http)));
            }
            if mode == "read-cancel" || mode == "pre-cancel" {
                assert!(matches!(result, Err(ToolError::Cancelled)));
            }
            if mode == "read-timeout" || mode == "pre-deadline" {
                assert!(matches!(result, Err(ToolError::TimedOut)));
            }
            if matches!(mode, "read-cancel" | "read-timeout" | "read-mutation") {
                assert!(
                    transport.notice_reads.load(Ordering::Acquire) > 0,
                    "must fail during actual notice read: {mode}"
                );
            }
            assert_eq!(manager.active("deno").unwrap(), active);
            manager
                .verify_package(&prior.directory, Some(&prior.package))
                .unwrap();
            assert_eq!(fs::read(source).unwrap(), original);
            assert_eq!(fs::read(&caps[0].path).unwrap(), notice_original);
            if mode == "foreign" {
                assert_eq!(fs::read(foreign.join("foreign")).unwrap(), b"never delete");
            }
            assert_eq!(
                fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                b"preserve"
            );
            drop(prior_temp);
        }
    }
}
