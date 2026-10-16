//! Bounded installation inventory and explicit existing-version selection.
use super::*;
use crate::process::{self, ProcessOutcome, ProcessSpec};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

/// Same exact bounded yt-dlp version token rule as direct installation.
pub(super) fn verified_ytdlp_version(stdout: &[u8], expected: &str) -> Result<(), ToolError> {
    if stdout.len() > 4096 || std::str::from_utf8(stdout).ok().map(str::trim) != Some(expected) {
        return Err(ToolError::VersionMismatch);
    }
    Ok(())
}
const MAX_INSTALLATIONS: usize = 128;
const MAX_INVENTORY_ENTRIES: usize = 2048;

fn read_manifest(path: &Path) -> Result<InstallManifest, ToolError> {
    safe_components(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > MANIFEST_LIMIT {
        return Err(ToolError::InvalidManifest);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)?
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err(ToolError::InvalidManifest);
    }
    let manifest: InstallManifest = serde_json::from_slice(&bytes)?;
    validate(&manifest)?;
    Ok(manifest)
}

impl ToolManager {
    fn recorded_install(
        &self,
        path: &Path,
        tool: &str,
        version: &str,
        target: &str,
    ) -> Result<InstalledTool, ToolError> {
        self.verify_root()?;
        safe_components(path)?;
        let manifest = read_manifest(&path.join("manifest.json"))?;
        if manifest.tool != tool || manifest.version != version || manifest.target != target {
            return Err(ToolError::InvalidManifest);
        }
        match fs::symlink_metadata(path.join("archive-package.json")) {
            Ok(_) => {
                let package = self.recorded_package(path)?;
                if serde_json::to_vec(&package.main.manifest)? != serde_json::to_vec(&manifest)? {
                    return Err(ToolError::InvalidManifest);
                }
                return Ok(package.main);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let executable = path.join(&manifest.executable);
        for entry in fs::read_dir(path)? {
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
        let metadata = fs::symlink_metadata(&executable)?;
        if !metadata.is_file() {
            return Err(ToolError::UnsafePath);
        }
        if metadata.len() == 0 || metadata.len() > MAX_DOWNLOAD_BYTES {
            return Err(ToolError::DownloadLimit);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(ToolError::UnsafePath);
            }
        }
        Ok(InstalledTool {
            manifest,
            executable,
            bytes: metadata.len(),
        })
    }
    /// Lists recorded managed installations, not new binary/version verification evidence.
    /// Fails explicitly above 128 installs or 2048 filesystem entries; no silent truncation.
    pub fn installed(&self, tool: &str) -> Result<Vec<InstalledTool>, ToolError> {
        if !identifier(tool) {
            return Err(ToolError::InvalidManifest);
        }
        self.verify_root()?;
        let path = self.root.join(tool);
        safe_components(&path)?;
        let versions = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut result = Vec::new();
        let mut scanned = 0;
        for version in versions {
            let version = version?;
            scanned += 1;
            if scanned > MAX_INVENTORY_ENTRIES {
                return Err(ToolError::InventoryLimit);
            }
            if !version.file_type()?.is_dir() {
                return Err(ToolError::UnsafePath);
            }
            let name = version.file_name();
            let version_name = name
                .to_str()
                .filter(|s| identifier(s))
                .ok_or(ToolError::InvalidManifest)?;
            for target in fs::read_dir(version.path())? {
                let target = target?;
                scanned += 1;
                if scanned > MAX_INVENTORY_ENTRIES {
                    return Err(ToolError::InventoryLimit);
                }
                if !target.file_type()?.is_dir() {
                    return Err(ToolError::UnsafePath);
                }
                if result.len() >= MAX_INSTALLATIONS {
                    return Err(ToolError::InventoryLimit);
                }
                let name = target.file_name();
                let target_name = name
                    .to_str()
                    .filter(|s| identifier(s))
                    .ok_or(ToolError::InvalidManifest)?;
                // Reserve the largest package set (10 files) and one overflow detection.
                scanned += 11;
                if scanned > MAX_INVENTORY_ENTRIES {
                    return Err(ToolError::InventoryLimit);
                }
                result.push(self.recorded_install(
                    &target.path(),
                    tool,
                    version_name,
                    target_name,
                )?);
            }
        }
        result.sort_by(|a, b| {
            (&a.manifest.version, &a.manifest.target)
                .cmp(&(&b.manifest.version, &b.manifest.target))
        });
        Ok(result)
    }
    fn verify_recorded(&self, manifest: &InstallManifest) -> Result<InstalledTool, ToolError> {
        self.verify_recorded_with_checkpoint(manifest, &mut || Ok(()))
    }
    fn verify_recorded_with_checkpoint(
        &self,
        manifest: &InstallManifest,
        checkpoint: &mut dyn FnMut() -> Result<(), ToolError>,
    ) -> Result<InstalledTool, ToolError> {
        checkpoint()?;
        let path = self.install_dir(manifest)?;
        let installed =
            self.recorded_install(&path, &manifest.tool, &manifest.version, &manifest.target)?;
        if serde_json::to_vec(&installed.manifest)? != serde_json::to_vec(manifest)? {
            return Err(ToolError::InvalidManifest);
        }
        match fs::symlink_metadata(path.join("archive-package.json")) {
            Ok(_) => {
                let package = self.verify_package_with_checkpoint(&path, None, checkpoint)?;
                if serde_json::to_vec(&package.main.manifest)? != serde_json::to_vec(manifest)? {
                    return Err(ToolError::InvalidManifest);
                }
                return Ok(package.main);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        let mut file = File::open(&installed.executable)?;
        let opened = file.metadata()?;
        let mut hash = Sha256::new();
        let mut bytes = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            checkpoint()?;
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            bytes = bytes
                .checked_add(n as u64)
                .ok_or(ToolError::DownloadLimit)?;
            if bytes > MAX_DOWNLOAD_BYTES {
                return Err(ToolError::DownloadLimit);
            }
            hash.update(&buffer[..n]);
        }
        if bytes != installed.bytes
            || format!("{:x}", hash.finalize()) != manifest.sha256.to_ascii_lowercase()
        {
            return Err(ToolError::HashMismatch);
        }
        safe_components(&installed.executable)?;
        let current = fs::symlink_metadata(&installed.executable)?;
        if !current.is_file() {
            return Err(ToolError::UnsafePath);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if opened.dev() != current.dev() || opened.ino() != current.ino() {
                return Err(ToolError::UnsafePath);
            }
        }
        #[cfg(not(unix))]
        let _ = (opened, current);
        self.verify_root()?;
        checkpoint()?;
        Ok(installed)
    }
    /// Worker-only owned direct yt-dlp inspection. Never activates, downloads or removes.
    pub fn inspect_existing(
        &mut self,
        manifest: &InstallManifest,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledTool, ToolError> {
        self.verify_existing_operation(manifest, cancel, false)
    }
    /// Explicit verified activation; cancellation is checked before transaction commit.
    pub fn select_verified_existing(
        &mut self,
        manifest: &InstallManifest,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledTool, ToolError> {
        self.verify_existing_operation(manifest, cancel, true)
    }
    fn verify_existing_operation(
        &mut self,
        manifest: &InstallManifest,
        cancel: &Arc<AtomicBool>,
        activate: bool,
    ) -> Result<InstalledTool, ToolError> {
        validate(manifest)?;
        if manifest.tool != "yt-dlp" || !inspection_target(&manifest.target) {
            return Err(ToolError::InvalidManifest);
        }
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .ok_or(ToolError::TimedOut)?;
        direct_download::checkpoint(cancel, deadline)?;
        let _lock = self.acquire_install_lock()?;
        let path = self.install_dir(manifest)?;
        let directory_pin = native_files::open_directory(&path)?;
        let executable = path.join(&manifest.executable);
        let executable_pin = native_files::open_regular(&executable)?;
        let installed = self.verify_recorded_with_checkpoint(manifest, &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        #[cfg(windows)]
        {
            // Reject an obvious non-executable before Windows launch/cleanup.
            // This DOS prefix check does not establish full PE or publisher trust.
            if installed.bytes < 64 {
                return Err(ToolError::VersionMismatch);
            }
            let mut prefix = [0u8; 2];
            File::open(&installed.executable)?.read_exact(&mut prefix)?;
            direct_download::checkpoint(cancel, deadline)?;
            if prefix != *b"MZ" {
                return Err(ToolError::VersionMismatch);
            }
        }
        let result = process::run_metadata(
            &ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: installed.executable.clone(),
                args: vec![
                    "--ignore-config".into(),
                    "--no-plugin-dirs".into(),
                    "--version".into(),
                ],
                timeout: direct_download::checkpoint(cancel, deadline)?,
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
        direct_download::checkpoint(cancel, deadline)?;
        match result.execution.outcome {
            ProcessOutcome::Cancelled => return Err(ToolError::Cancelled),
            ProcessOutcome::TimedOut => return Err(ToolError::TimedOut),
            ProcessOutcome::OutputLimit => return Err(ToolError::VersionMismatch),
            ProcessOutcome::Exited => (),
        }
        if result.execution.exit_code != Some(0) || result.execution.truncated {
            return Err(ToolError::VersionMismatch);
        }
        verified_ytdlp_version(&result.stdout, &manifest.version)?;
        if !native_files::same_directory(&path, &directory_pin)
            || !native_files::same_file(&executable, &executable_pin)
        {
            return Err(ToolError::UnsafePath);
        }
        let verified = self.verify_recorded_with_checkpoint(manifest, &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        direct_download::checkpoint(cancel, deadline)?;
        self.verify_root()?;
        direct_download::checkpoint(cancel, deadline)?;
        if !native_files::same_directory(&path, &directory_pin)
            || !native_files::same_file(&executable, &executable_pin)
        {
            return Err(ToolError::UnsafePath);
        }
        if activate {
            let transaction = self.db.transaction()?;
            transaction.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![manifest.tool,manifest.version,manifest.target])?;
            direct_download::checkpoint(cancel, deadline)?;
            transaction.commit()?;
        }
        Ok(verified)
    }
    /// Explicitly reselects pinned bytes. Does not rerun --version or modify existing files.
    /// A legacy local import has hash evidence, not the new download's version-check evidence.
    pub fn select_existing(
        &mut self,
        manifest: &InstallManifest,
    ) -> Result<InstalledTool, ToolError> {
        let _install_lock = self.acquire_install_lock()?;
        let installed = self.verify_recorded(manifest)?;
        self.db.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![manifest.tool,manifest.version,manifest.target])?;
        Ok(installed)
    }
    /// Returns an active path only after rechecking the recorded ownership/path and bytes.
    pub fn active_installed(&self, tool: &str) -> Result<Option<InstalledTool>, ToolError> {
        if !identifier(tool) {
            return Err(ToolError::InvalidManifest);
        }
        let Some((version, target)) = self.active(tool)? else {
            return Ok(None);
        };
        if !identifier(&version) || !identifier(&target) {
            return Err(ToolError::InvalidManifest);
        }
        let installed = self.recorded_install(
            &self.root.join(tool).join(&version).join(&target),
            tool,
            &version,
            &target,
        )?;
        self.verify_recorded(&installed.manifest).map(Some)
    }
}

fn inspection_target(target: &str) -> bool {
    (cfg!(all(target_os = "linux", target_arch = "x86_64")) && target == "x86_64-unknown-linux-gnu")
        || (cfg!(all(target_os = "windows", target_arch = "x86_64"))
            && matches!(target, "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu"))
        || (cfg!(all(target_os = "macos", target_arch = "x86_64"))
            && target == "x86_64-apple-darwin")
        || (cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && target == "aarch64-apple-darwin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_ytdlp_version_rule_rejects_wrong_bounds_and_encoding() {
        assert!(verified_ytdlp_version(b"2026.08.19\r\n", "2026.08.19").is_ok());
        for bytes in [
            b"2026.08.190".to_vec(),
            b"banner\n2026.08.19".to_vec(),
            vec![0xff],
            vec![b'x'; 4097],
        ] {
            assert!(matches!(
                verified_ytdlp_version(&bytes, "2026.08.19"),
                Err(ToolError::VersionMismatch)
            ));
        }
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn inspection_fixture(manager: &ToolManager, version: &str, mode: &str) -> InstallManifest {
        use std::os::unix::fs::PermissionsExt;
        let path = manager
            .root
            .join("yt-dlp")
            .join(version)
            .join("x86_64-unknown-linux-gnu");
        fs::create_dir_all(&path).unwrap();
        let code = if mode == "nonzero" { 7 } else { 0 };
        let reported = if mode == "wrong" { "wrong" } else { version };
        let mutation = if mode == "mutate" {
            "p.write_bytes(p.read_bytes()+b'\\n#changed')"
        } else {
            "pass"
        };
        let output = if mode == "limit" {
            "print('x'*4097)".to_owned()
        } else {
            format!("print('{reported}')")
        };
        let script = format!(
            "#!/usr/bin/python3\nimport pathlib,sys\nassert sys.argv[1:]==['--ignore-config','--no-plugin-dirs','--version']\np=pathlib.Path(__file__)\n{mutation}\n{output}\nsys.exit({code})\n"
        );
        let executable = path.join("yt-dlp");
        fs::write(&executable, script.as_bytes()).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: version.into(),
            target: "x86_64-unknown-linux-gnu".into(),
            source_url: "https://example.invalid/fixture".into(),
            sha256: format!("{:x}", Sha256::digest(script.as_bytes())),
            license: "synthetic".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp".into(),
        };
        fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        manifest
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn readonly_ytdlp_inspection_preserves_absent_valid_and_invalid_active_rows() {
        for version in [None, Some("1.0.0"), Some("../invalid")] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = inspection_fixture(&manager, "1.0.0", "good");
            if let Some(version) = version {
                manager.db.execute("INSERT INTO active(tool,version,target)VALUES('yt-dlp',?1,'x86_64-unknown-linux-gnu')",[version]).unwrap();
            }
            let before = manager.active("yt-dlp").unwrap();
            let next = inspection_fixture(&manager, "2.0.0", "good");
            manager.db.execute_batch("CREATE TRIGGER deny_inspect_insert BEFORE INSERT ON active BEGIN SELECT RAISE(ABORT,'inspection must not activate');END;CREATE TRIGGER deny_inspect_update BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT,'inspection must not activate');END;").unwrap();
            let inspected = manager
                .inspect_existing(&next, &Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert_eq!(inspected.manifest.version, "2.0.0");
            assert_eq!(manager.active("yt-dlp").unwrap(), before);
            assert!(manager.install_dir(&prior).unwrap().join("yt-dlp").exists());
        }
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn readonly_ytdlp_failure_cancel_and_post_version_mutation_preserve_selection() {
        for mode in [
            "wrong", "nonzero", "mutate", "limit", "cancel", "tamper", "foreign",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = inspection_fixture(&manager, "1.0.0", "good");
            manager.select_existing(&prior).unwrap();
            let before = manager.active("yt-dlp").unwrap();
            let next = inspection_fixture(&manager, "2.0.0", mode);
            let directory = manager.install_dir(&next).unwrap();
            if mode == "tamper" {
                fs::write(directory.join("yt-dlp"), b"tampered").unwrap();
            }
            if mode == "foreign" {
                fs::write(directory.join("video.mp4"), b"keep").unwrap();
            }
            let result =
                manager.inspect_existing(&next, &Arc::new(AtomicBool::new(mode == "cancel")));
            match mode {
                "cancel" => assert!(matches!(result, Err(ToolError::Cancelled))),
                "mutate" | "tamper" => assert!(matches!(result, Err(ToolError::HashMismatch))),
                "foreign" => assert!(matches!(result, Err(ToolError::UnexpectedFiles))),
                _ => assert!(matches!(result, Err(ToolError::VersionMismatch))),
            }
            assert_eq!(manager.active("yt-dlp").unwrap(), before);
            assert!(manager.install_dir(&prior).unwrap().join("yt-dlp").exists());
            if mode == "foreign" {
                assert_eq!(fs::read(directory.join("video.mp4")).unwrap(), b"keep");
            }
        }
    }
    #[test]
    fn recorded_hash_checkpoint_can_cancel_between_chunks_without_activation() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let (source, mut manifest) = fixture(temp.path(), "1");
        let bytes = vec![b'x'; 128 * 1024 + 1];
        fs::write(&source, &bytes).unwrap();
        manifest.sha256 = format!("{:x}", Sha256::digest(&bytes));
        manager.import(&source, manifest.clone()).unwrap();
        let active = manager.active("fixture").unwrap();
        let mut calls = 0;
        let result = manager.verify_recorded_with_checkpoint(&manifest, &mut || {
            calls += 1;
            if calls >= 3 {
                Err(ToolError::Cancelled)
            } else {
                Ok(())
            }
        });
        assert!(matches!(result, Err(ToolError::Cancelled)));
        assert_eq!(calls, 3);
        assert_eq!(manager.active("fixture").unwrap(), active);
        assert_eq!(fs::read(source).unwrap(), bytes);
    }

    fn fixture(temp: &Path, version: &str) -> (PathBuf, InstallManifest) {
        let bytes = format!("synthetic executable {version}");
        let source = temp.join(format!("source-{version}"));
        fs::write(&source, bytes.as_bytes()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
        }
        (
            source,
            InstallManifest {
                tool: "fixture".into(),
                version: version.into(),
                target: "test".into(),
                source_url: "fixture://synthetic".into(),
                sha256: format!("{:x}", Sha256::digest(bytes.as_bytes())),
                license: "synthetic fixture only".into(),
                ownership: Ownership::Managed,
                executable: "fixture-tool".into(),
            },
        )
    }
    #[test]
    fn bounded_inventory_explicit_rollback_and_active_path_are_persisted() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let mut manager = ToolManager::new(&root).unwrap();
        let (source, first) = fixture(temp.path(), "1");
        let original = manager.import(&source, first.clone()).unwrap();
        let (source2, second) = fixture(temp.path(), "2");
        manager.import(&source2, second.clone()).unwrap();
        let versions = manager.installed("fixture").unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].manifest.version, "1");
        assert_eq!(
            manager.select_existing(&first).unwrap().executable,
            original.executable
        );
        assert_eq!(
            manager
                .active_installed("fixture")
                .unwrap()
                .unwrap()
                .manifest
                .version,
            "1"
        );
        drop(manager);
        let manager = ToolManager::new(&root).unwrap();
        assert_eq!(
            manager
                .active_installed("fixture")
                .unwrap()
                .unwrap()
                .manifest
                .version,
            "1"
        );
        assert!(source.exists() && source2.exists());
        assert!(manager.installed("absent").unwrap().is_empty());
        assert!(manager.active_installed("absent").unwrap().is_none());
    }
    #[test]
    fn tamper_corrupt_manifest_foreign_files_and_database_failure_preserve_selection() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let (source, first) = fixture(temp.path(), "1");
        let one = manager.import(&source, first.clone()).unwrap();
        let (source, second) = fixture(temp.path(), "2");
        let two = manager.import(&source, second.clone()).unwrap();
        let original = fs::read(&one.executable).unwrap();
        let record = one.executable.parent().unwrap().join("manifest.json");
        let original_record = fs::read(&record).unwrap();
        fs::write(&one.executable, b"tampered executable").unwrap();
        assert!(matches!(
            manager.select_existing(&first),
            Err(ToolError::HashMismatch)
        ));
        assert_eq!(
            manager
                .active_installed("fixture")
                .unwrap()
                .unwrap()
                .executable,
            two.executable
        );
        fs::write(&one.executable, &original).unwrap();
        fs::write(&record, b"corrupt json").unwrap();
        assert!(matches!(
            manager.select_existing(&first),
            Err(ToolError::Json(_))
        ));
        fs::write(&record, &original_record).unwrap();
        let foreign = one.executable.parent().unwrap().join("user-video.mp4");
        fs::write(&foreign, b"preserve").unwrap();
        assert!(matches!(
            manager.select_existing(&first),
            Err(ToolError::UnexpectedFiles)
        ));
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve");
        fs::remove_file(&foreign).unwrap(); // only this test's synthetic owned fixture
        let mut external = first.clone();
        external.ownership = Ownership::External;
        assert!(matches!(
            manager.select_existing(&external),
            Err(ToolError::ExternalOwnership)
        ));
        let mut changed_record = first.clone();
        changed_record.license = "not the stored record".into();
        assert!(matches!(
            manager.select_existing(&changed_record),
            Err(ToolError::InvalidManifest)
        ));
        manager.db.execute_batch("CREATE TRIGGER deny_selection BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT, 'synthetic db failure'); END;").unwrap();
        assert!(matches!(
            manager.select_existing(&first),
            Err(ToolError::Database(_))
        ));
        assert_eq!(
            manager.active("fixture").unwrap(),
            Some(("2".into(), "test".into()))
        );
        assert_eq!(fs::read(&one.executable).unwrap(), original);
        assert_eq!(fs::read(record).unwrap(), original_record);
    }
    #[test]
    fn active_tamper_invalid_database_identifiers_and_record_bounds_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let (source, manifest) = fixture(temp.path(), "1");
        let installed = manager.import(&source, manifest).unwrap();
        fs::write(&installed.executable, b"tampered").unwrap();
        assert!(matches!(
            manager.active_installed("fixture"),
            Err(ToolError::HashMismatch)
        ));
        let record = installed.executable.parent().unwrap().join("manifest.json");
        fs::write(&record, vec![b'x'; MANIFEST_LIMIT as usize + 1]).unwrap();
        assert!(matches!(
            manager.installed("fixture"),
            Err(ToolError::InvalidManifest)
        ));
        manager
            .db
            .execute(
                "UPDATE active SET version='../external' WHERE tool='fixture'",
                [],
            )
            .unwrap();
        assert!(matches!(
            manager.active_installed("fixture"),
            Err(ToolError::InvalidManifest)
        ));
        assert!(manager.installed("../external").is_err());
    }
    #[test]
    fn inventory_installation_and_filesystem_scan_caps_are_explicit() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        for number in 0..=MAX_INSTALLATIONS {
            let version = number.to_string();
            let bytes = b"synthetic fixture";
            let manifest = InstallManifest {
                tool: "many".into(),
                version: version.clone(),
                target: "test".into(),
                source_url: "fixture://synthetic".into(),
                sha256: format!("{:x}", Sha256::digest(bytes)),
                license: "fixture".into(),
                ownership: Ownership::Managed,
                executable: "tool".into(),
            };
            let path = manager.root.join("many").join(version).join("test");
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join("manifest.json"),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            fs::write(path.join("tool"), bytes).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path.join("tool"), fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
        assert!(matches!(
            manager.installed("many"),
            Err(ToolError::InventoryLimit)
        ));
        for number in 0..=MAX_INVENTORY_ENTRIES {
            fs::create_dir_all(manager.root.join("empty-versions").join(number.to_string()))
                .unwrap();
        }
        assert!(matches!(
            manager.installed("empty-versions"),
            Err(ToolError::InventoryLimit)
        ));
    }
    #[cfg(unix)]
    #[test]
    fn symlink_record_and_oversized_binary_preserve_external_data() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let (source, manifest) = fixture(temp.path(), "1");
        let installed = manager.import(&source, manifest.clone()).unwrap();
        let record = installed.executable.parent().unwrap().join("manifest.json");
        let external = temp.path().join("external-record");
        fs::rename(&record, &external).unwrap();
        std::os::unix::fs::symlink(&external, &record).unwrap();
        assert!(matches!(
            manager.select_existing(&manifest),
            Err(ToolError::UnsafePath)
        ));
        assert!(external.exists());
        fs::remove_file(&record).unwrap();
        fs::rename(&external, &record).unwrap();
        OpenOptions::new()
            .write(true)
            .open(&installed.executable)
            .unwrap()
            .set_len(MAX_DOWNLOAD_BYTES + 1)
            .unwrap();
        assert!(matches!(
            manager.select_existing(&manifest),
            Err(ToolError::DownloadLimit)
        ));
        assert!(installed.executable.exists());
    }
}

/// Actual Windows fixtures run only on Root's owned private native CLI host.
#[cfg(all(windows, target_arch = "x86_64"))]
#[cfg(test)]
mod windows_inspection_tests {
    use super::*;
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    static SERIAL: Mutex<()> = Mutex::new(());
    static BUSY: AtomicUsize = AtomicUsize::new(0);
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
    fn seed(manager: &ToolManager, version: &str, reported: &str) -> InstallManifest {
        let path = manager
            .root
            .join("yt-dlp")
            .join(version)
            .join("x86_64-pc-windows-msvc");
        fs::create_dir_all(&path).unwrap();
        // Source and compiler output are owned temp siblings, never package foreign entries.
        let fixture_root = manager.root.parent().unwrap();
        let source = fixture_root.join(format!("fixture-{version}.cs"));
        let program = format!(
            "using System; class SyntheticYtdlp {{ static int Main(string[] a) {{ if(a.Length!=3 || a[0]!=\"--ignore-config\" || a[1]!=\"--no-plugin-dirs\" || a[2]!=\"--version\") return 7; Console.WriteLine(\"{reported}\"); return 0; }} }}"
        );
        fs::write(&source, program.as_bytes()).unwrap();
        let executable = path.join("yt-dlp.exe");
        let csc_source = legacy_csc_owned_path(&source, fixture_root);
        let csc_output = legacy_csc_owned_path(&executable, fixture_root);
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
                    format!("/out:{}", csc_output.display()).into(),
                    csc_source.as_os_str().to_owned(),
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
            "owned CSC fixture setup failed: source={source:?}; output={executable:?}; source_text={program:?}; stdout={:?}; bounded_logs={:?}",
            String::from_utf8_lossy(&result.stdout),
            result.execution.logs,
        );
        assert!(!result.execution.truncated, "owned compiler output bounded");
        let data = fs::read(&executable).unwrap();
        assert!(data.len() > 64 && data.len() < 1048576);
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: version.into(),
            target: "x86_64-pc-windows-msvc".into(),
            source_url: "https://example.invalid/owned-synthetic-only".into(),
            sha256: format!("{:x}", Sha256::digest(&data)),
            license: "synthetic fixture only".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp.exe".into(),
        };
        fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        manifest
    }
    #[test]
    #[ignore = "requires root-owned windows-cli-host private desktop and owned csc fixture"]
    fn actual_windows_readonly_inspection_and_verified_activation_preserve_failures() {
        let _serial = SERIAL.lock().unwrap();
        require_private_cli_desktop();
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let prior = seed(&manager, "1.0.0", "1.0.0");
        let next = seed(&manager, "2.0.0", "2.0.0");
        let wrong = seed(&manager, "3.0.0", "0.0.0");
        manager
            .select_verified_existing(&prior, &Arc::new(AtomicBool::new(false)))
            .unwrap();
        let active = manager.active("yt-dlp").unwrap();
        manager.db.execute_batch("CREATE TRIGGER deny_inspect_insert BEFORE INSERT ON active BEGIN SELECT RAISE(ABORT,'readonly must not insert');END;CREATE TRIGGER deny_inspect_update BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT,'readonly must not update');END;").unwrap();
        let original = fs::read(manager.install_dir(&next).unwrap().join("yt-dlp.exe")).unwrap();
        assert_eq!(
            manager
                .inspect_existing(&next, &Arc::new(AtomicBool::new(false)))
                .unwrap()
                .manifest
                .version,
            "2.0.0"
        );
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert!(matches!(
            manager.inspect_existing(&wrong, &Arc::new(AtomicBool::new(false))),
            Err(ToolError::VersionMismatch)
        ));
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert!(matches!(
            manager.select_verified_existing(&wrong, &Arc::new(AtomicBool::new(false))),
            Err(ToolError::VersionMismatch)
        ));
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert!(matches!(
            manager.select_verified_existing(&next, &Arc::new(AtomicBool::new(true))),
            Err(ToolError::Cancelled)
        ));
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        let mut incompatible = next.clone();
        incompatible.target = "unsupported-target".into();
        assert!(matches!(
            manager.inspect_existing(&incompatible, &Arc::new(AtomicBool::new(false))),
            Err(ToolError::InvalidManifest)
        ));
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        // Intentional owned tamper remains in fixture; no restore pretending a passing hash.
        let file = manager.install_dir(&next).unwrap().join("yt-dlp.exe");
        let mut changed = original.clone();
        changed[0] ^= 1;
        fs::write(&file, &changed).unwrap();
        assert!(matches!(
            manager.inspect_existing(&next, &Arc::new(AtomicBool::new(false))),
            Err(ToolError::HashMismatch)
        ));
        assert_eq!(fs::read(file).unwrap(), changed);
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert!(
            manager
                .install_dir(&prior)
                .unwrap()
                .join("yt-dlp.exe")
                .exists()
        );
    }
    fn busy(_: i32) -> bool {
        BUSY.fetch_add(1, Ordering::Release);
        std::thread::yield_now();
        true
    }
    #[test]
    #[ignore = "requires root-owned windows-cli-host private desktop and owned csc fixture"]
    fn actual_windows_verified_selection_cancel_after_sqlite_wait_rolls_back() {
        let _serial = SERIAL.lock().unwrap();
        require_private_cli_desktop();
        BUSY.store(0, Ordering::Release);
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let mut manager = ToolManager::new(&root).unwrap();
        let prior = seed(&manager, "1.0.0", "1.0.0");
        let next = seed(&manager, "2.0.0", "2.0.0");
        manager
            .select_verified_existing(&prior, &Arc::new(AtomicBool::new(false)))
            .unwrap();
        let active = manager.active("yt-dlp").unwrap();
        manager.db.busy_handler(Some(busy)).unwrap();
        let blocker = Connection::open(root.join("activation.sqlite")).unwrap();
        blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker = std::thread::spawn(move || {
            let result = manager.select_verified_existing(&next, &worker_cancel);
            (manager, result)
        });
        let until = Instant::now() + Duration::from_secs(5);
        while BUSY.load(Ordering::Acquire) == 0 {
            if Instant::now() >= until {
                cancel.store(true, Ordering::Release);
                blocker.execute_batch("ROLLBACK").unwrap();
                let _ = worker.join();
                panic!("actual selector never reached SQLite write lock");
            }
            std::thread::yield_now();
        }
        cancel.store(true, Ordering::Release);
        blocker.execute_batch("ROLLBACK").unwrap();
        let (manager, result) = worker.join().unwrap();
        assert!(matches!(result, Err(ToolError::Cancelled)));
        assert_eq!(manager.active("yt-dlp").unwrap(), active);
        assert!(
            manager
                .install_dir(&prior)
                .unwrap()
                .join("yt-dlp.exe")
                .exists()
        );
    }
}
