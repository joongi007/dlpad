//! Full-set package verification; filesystem I/O must run on a worker.
use super::*;
use crate::file_identity::{self, NativeFileIdentity};
use serde::de::DeserializeOwned;
fn open_checked(path: &Path) -> Result<File, ToolError> {
    safe_components(path)?;
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() {
        return Err(ToolError::UnsafePath);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        if meta.nlink() != 1 {
            return Err(ToolError::UnsafePath);
        }
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let actual = file.metadata()?;
        if !actual.is_file()
            || actual.nlink() != 1
            || actual.dev() != meta.dev()
            || actual.ino() != meta.ino()
        {
            return Err(ToolError::UnsafePath);
        }
        Ok(file)
    }
    #[cfg(windows)]
    {
        super::native_files::open_regular(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = meta;
        Err(ToolError::UnsafePath)
    }
}
fn same(path: &Path, file: &File, id: &NativeFileIdentity) -> Result<(), ToolError> {
    if &file_identity::from_file(file)? != id
        || file_identity::from_file(&open_checked(path)?)? != *id
    {
        return Err(ToolError::UnsafePath);
    }
    Ok(())
}
fn record<T: DeserializeOwned>(path: &Path) -> Result<T, ToolError> {
    let mut file = open_checked(path)?;
    let id = file_identity::from_file(&file)?;
    if file.metadata()?.len() > MANIFEST_LIMIT {
        return Err(ToolError::InvalidManifest);
    }
    let mut data = Vec::new();
    (&mut file)
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut data)?;
    if data.len() as u64 > MANIFEST_LIMIT {
        return Err(ToolError::InvalidManifest);
    }
    let result = serde_json::from_slice(&data)?;
    same(path, &file, &id)?;
    Ok(result)
}
impl ToolManager {
    pub fn package_installed(
        &self,
        manifest: &InstallManifest,
    ) -> Result<Option<InstalledPackage>, ToolError> {
        let path = self.install_dir(manifest)?;
        self.verify_root()?;
        match fs::symlink_metadata(path.join("archive-package.json")) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
            Ok(_) => {}
        }
        let result = self.verify_package(&path, None)?;
        if serde_json::to_vec(&result.main.manifest)? != serde_json::to_vec(manifest)? {
            return Err(ToolError::InvalidManifest);
        }
        Ok(Some(result))
    }
    /// Bounded metadata lookup only; activation must verify bytes and runtime version.
    pub fn recorded_package_for_manifest(
        &self,
        manifest: &InstallManifest,
    ) -> Result<Option<InstalledPackage>, ToolError> {
        let path = self.install_dir(manifest)?;
        self.verify_root()?;
        match fs::symlink_metadata(path.join("archive-package.json")) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
            Ok(_) => (),
        }
        let package = self.recorded_package(&path)?;
        if serde_json::to_vec(&package.main.manifest)? != serde_json::to_vec(manifest)? {
            return Err(ToolError::InvalidManifest);
        }
        Ok(Some(package))
    }
    pub fn select_package(
        &mut self,
        package: &ArchivePackage,
    ) -> Result<InstalledPackage, ToolError> {
        package.validate()?;
        let _install_lock = self.acquire_install_lock()?;
        let path = self
            .root
            .join(&package.tool)
            .join(&package.tool_version)
            .join(&package.target);
        let installed = self.verify_package(&path, Some(package))?;
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![package.tool,package.tool_version,package.target])?;
        tx.commit()?;
        Ok(installed)
    }
    pub(super) fn verify_package(
        &self,
        path: &Path,
        expected: Option<&ArchivePackage>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_package_with_checkpoint(path, expected, &mut || Ok(()))
    }
    pub(super) fn verify_package_with_checkpoint(
        &self,
        path: &Path,
        expected: Option<&ArchivePackage>,
        checkpoint: &mut dyn FnMut() -> Result<(), ToolError>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_package_inner(path, expected, true, checkpoint)
    }
    /// Metadata inventory only; component bytes are rehashed at active use or selection.
    pub(super) fn recorded_package(&self, path: &Path) -> Result<InstalledPackage, ToolError> {
        self.verify_package_inner(path, None, false, &mut || Ok(()))
    }
    fn verify_package_inner(
        &self,
        path: &Path,
        expected: Option<&ArchivePackage>,
        verify_contents: bool,
        checkpoint: &mut dyn FnMut() -> Result<(), ToolError>,
    ) -> Result<InstalledPackage, ToolError> {
        verify_package_from_root(
            &self.root,
            &self.root_pin,
            path,
            expected,
            verify_contents,
            checkpoint,
        )
    }
}
pub(super) fn verify_package_from_root(
    root: &Path,
    root_pin: &File,
    path: &Path,
    expected: Option<&ArchivePackage>,
    verify_contents: bool,
    checkpoint: &mut dyn FnMut() -> Result<(), ToolError>,
) -> Result<InstalledPackage, ToolError> {
    checkpoint()?;
    super::verify_root_pin(root, root_pin)?;
    safe_components(path)?;
    if !fs::symlink_metadata(path)?.is_dir() {
        return Err(ToolError::UnsafePath);
    }
    let directory_pin = super::native_files::open_directory(path)?;
    let package: ArchivePackage = record(&path.join("archive-package.json"))?;
    package.validate()?;
    if expected.is_some_and(|p| serde_json::to_vec(p).ok() != serde_json::to_vec(&package).ok()) {
        return Err(ToolError::InvalidManifest);
    }
    if path
        != root
            .join(&package.tool)
            .join(&package.tool_version)
            .join(&package.target)
    {
        return Err(ToolError::InvalidManifest);
    }
    let manifest: InstallManifest = record(&path.join("manifest.json"))?;
    validate(&manifest)?;
    let main = package
        .components
        .iter()
        .find(|c| c.role == ArchiveComponentRole::Executable)
        .ok_or(ToolError::InvalidManifest)?;
    if manifest.tool != package.tool
        || manifest.version != package.tool_version
        || manifest.target != package.target
        || manifest.source_url != package.source_url
        || manifest.license != package.license
        || manifest.ownership != package.ownership
        || manifest.executable != main.name
        || !manifest.sha256.eq_ignore_ascii_case(&main.sha256)
    {
        return Err(ToolError::InvalidManifest);
    }
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        checkpoint()?;
        count += 1;
        if count > 10 {
            return Err(ToolError::InventoryLimit);
        }
        let entry = entry?;
        let name = entry.file_name();
        if name != "manifest.json"
            && name != "archive-package.json"
            && !package
                .components
                .iter()
                .any(|c| name == std::ffi::OsStr::new(&c.name))
        {
            return Err(ToolError::UnexpectedFiles);
        }
        if !entry.file_type()?.is_file() {
            return Err(ToolError::UnsafePath);
        }
        #[cfg(windows)]
        {
            let pinned = open_checked(&entry.path())?;
            file_identity::ensure_regular_under(&directory_pin, &pinned)?;
        }
    }
    if count != package.components.len() + 2 {
        return Err(ToolError::UnexpectedFiles);
    }
    for component in &package.components {
        checkpoint()?;
        let p = path.join(&component.name);
        let mut file = open_checked(&p)?;
        let id = file_identity::from_file(&file)?;
        #[cfg(windows)]
        file_identity::ensure_regular_under(&directory_pin, &file)?;
        if file.metadata()?.len() != component.bytes {
            return Err(ToolError::UnexpectedSize);
        }
        #[cfg(unix)]
        if component.role == ArchiveComponentRole::Executable {
            use std::os::unix::fs::PermissionsExt;
            if file.metadata()?.permissions().mode() & 0o111 == 0 {
                return Err(ToolError::UnsafePath);
            }
        }
        if verify_contents {
            let mut hash = Sha256::new();
            let mut bytes = 0u64;
            let mut buffer = [0u8; 65536];
            loop {
                checkpoint()?;
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                bytes = bytes
                    .checked_add(n as u64)
                    .ok_or(ToolError::DownloadLimit)?;
                if bytes > component.bytes {
                    return Err(ToolError::UnexpectedSize);
                }
                hash.update(&buffer[..n]);
            }
            if bytes != component.bytes
                || format!("{:x}", hash.finalize()) != component.sha256.to_ascii_lowercase()
            {
                return Err(ToolError::HashMismatch);
            }
        }
        same(&p, &file, &id)?;
    }
    super::verify_root_pin(root, root_pin)?;
    safe_components(path)?;
    let bytes = main.bytes;
    let executable = path.join(&main.name);
    if !super::native_files::same_directory(path, &directory_pin) {
        return Err(ToolError::UnsafePath);
    }
    checkpoint()?;
    Ok(InstalledPackage {
        package,
        directory: path.to_owned(),
        main: InstalledTool {
            manifest,
            executable,
            bytes,
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path) -> (ToolManager, ArchivePackage, InstallManifest, PathBuf) {
        let manager = ToolManager::new(&root.join("managed")).unwrap();
        let components: Vec<_> = [
            ("tool", ArchiveComponentRole::Executable),
            ("probe", ArchiveComponentRole::Auxiliary),
            ("LICENSE", ArchiveComponentRole::Notice),
        ]
        .into_iter()
        .map(|(name, role)| ArchiveComponent {
            name: name.into(),
            bytes: 4,
            sha256: format!("{:x}", Sha256::digest(b"data")),
            role,
        })
        .collect();
        let package = ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "fixture".into(),
            tool_version: "1".into(),
            target: "test".into(),
            source_url: "https://example.invalid/fixture.zip".into(),
            license: "MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: 123,
            archive_sha256: "a".repeat(64),
            components,
        };
        let manifest = InstallManifest {
            tool: package.tool.clone(),
            version: package.tool_version.clone(),
            target: package.target.clone(),
            source_url: package.source_url.clone(),
            license: package.license.clone(),
            ownership: Ownership::Managed,
            sha256: package.components[0].sha256.clone(),
            executable: "tool".into(),
        };
        let path = manager.install_dir(&manifest).unwrap();
        fs::create_dir_all(&path).unwrap();
        for c in &package.components {
            fs::write(path.join(&c.name), b"data").unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path.join("tool"), fs::Permissions::from_mode(0o700)).unwrap();
        }
        fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            path.join("archive-package.json"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        (manager, package, manifest, path)
    }
    #[test]
    fn full_set_reselection_and_failure_preserves_active() {
        let temp = tempfile::tempdir().unwrap();
        let (mut manager, p, m, path) = fixture(temp.path());
        assert!(manager.package_installed(&m).unwrap().is_some());
        manager.select_package(&p).unwrap();
        let active = manager.active("fixture").unwrap();
        fs::write(path.join("probe"), b"evil").unwrap();
        assert!(manager.recorded_package(&path).is_ok());
        assert!(manager.package_installed(&m).is_err());
        assert!(matches!(
            manager.select_package(&p),
            Err(ToolError::HashMismatch)
        ));
        assert_eq!(manager.active("fixture").unwrap(), active);
        fs::write(path.join("probe"), b"data").unwrap();
        fs::write(path.join("foreign"), b"preserve").unwrap();
        assert!(matches!(
            manager.select_package(&p),
            Err(ToolError::UnexpectedFiles)
        ));
        assert_eq!(fs::read(path.join("foreign")).unwrap(), b"preserve");
        assert!(manager.recorded_package(&path).is_err());
    }
    #[test]
    fn metadata_mismatch_malformed_and_only_absence_is_none() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, _, m, path) = fixture(temp.path());
        let mut changed = m.clone();
        changed.license = "other".into();
        fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        assert!(manager.package_installed(&m).is_err());
        fs::write(path.join("archive-package.json"), b"{").unwrap();
        assert!(manager.package_installed(&m).is_err());
        fs::remove_file(path.join("archive-package.json")).unwrap();
        assert!(manager.package_installed(&m).unwrap().is_none());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_and_hardlink_component_are_rejected_without_deletion() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, p, _, path) = fixture(temp.path());
        let aux = path.join("probe");
        fs::remove_file(&aux).unwrap();
        let external = temp.path().join("external");
        fs::write(&external, b"data").unwrap();
        std::os::unix::fs::symlink(&external, &aux).unwrap();
        assert!(manager.verify_package(&path, Some(&p)).is_err());
        fs::remove_file(&aux).unwrap();
        fs::hard_link(&external, &aux).unwrap();
        assert!(manager.verify_package(&path, Some(&p)).is_err());
        assert_eq!(fs::read(external).unwrap(), b"data");
    }
    // Draft: insert inside package_inventory::tests; reuses fixture(root).
    // Requires root verify_package_with_checkpoint(path, expected, callback) API.
    fn checkpoint_large_fixture(root: &Path) -> (ToolManager, ArchivePackage, PathBuf, Vec<u8>) {
        let (manager, mut package, mut manifest, path) = fixture(root);
        let bytes = vec![b'x'; 1024 * 1024 + 13];
        fs::write(path.join("tool"), &bytes).unwrap();
        package.components[0].bytes = bytes.len() as u64;
        package.components[0].sha256 = format!("{:x}", Sha256::digest(&bytes));
        manifest.sha256 = package.components[0].sha256.clone();
        fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            path.join("archive-package.json"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        fs::write(root.join("foreign-sentinel"), b"external unchanged").unwrap();
        (manager, package, path, bytes)
    }

    #[test]
    fn checkpoint_cancellation_and_budget_failure_preserve_bytes() {
        for timed_out in [false, true] {
            for fail_at in [1, 3, 9, 14] {
                let temp = tempfile::tempdir().unwrap();
                let (manager, package, path, bytes) = checkpoint_large_fixture(temp.path());
                let metadata = fs::read(path.join("archive-package.json")).unwrap();
                let mut calls = 0;
                let result =
                    manager.verify_package_with_checkpoint(&path, Some(&package), &mut || {
                        calls += 1;
                        if calls == fail_at {
                            return Err(if timed_out {
                                ToolError::TimedOut
                            } else {
                                ToolError::Cancelled
                            });
                        }
                        Ok(())
                    });
                assert_eq!(
                    calls, fail_at,
                    "verification continued after checkpoint failure"
                );
                if timed_out {
                    assert!(matches!(result, Err(ToolError::TimedOut)));
                } else {
                    assert!(matches!(result, Err(ToolError::Cancelled)));
                }
                assert_eq!(fs::read(path.join("tool")).unwrap(), bytes);
                assert_eq!(
                    fs::read(path.join("archive-package.json")).unwrap(),
                    metadata
                );
                assert_eq!(
                    fs::read(temp.path().join("foreign-sentinel")).unwrap(),
                    b"external unchanged"
                );
                assert!(manager.active("fixture").unwrap().is_none());
            }
        }
    }

    #[test]
    fn checkpoint_visits_each_read_boundary_and_success_remains_read_only() {
        let small = tempfile::tempdir().unwrap();
        let (small_manager, small_package, _, small_path) = fixture(small.path());
        let mut small_calls = 0;
        small_manager
            .verify_package_with_checkpoint(&small_path, Some(&small_package), &mut || {
                small_calls += 1;
                Ok(())
            })
            .unwrap();
        let temp = tempfile::tempdir().unwrap();
        let (manager, package, path, bytes) = checkpoint_large_fixture(temp.path());
        let mut calls = 0;
        let result = manager
            .verify_package_with_checkpoint(&path, Some(&package), &mut || {
                calls += 1;
                Ok(())
            })
            .unwrap();
        // The large component alone needs 17 data reads. Other checkpoints can add
        // calls, so avoid coupling this test to the exact metadata-loop order.
        assert!(
            calls >= small_calls + bytes.len().div_ceil(65536) - 1,
            "missing per-buffer checkpoint"
        );
        assert_eq!(result.directory, path);
        assert_eq!(fs::read(result.main.executable).unwrap(), bytes);
        assert_eq!(
            fs::read(temp.path().join("foreign-sentinel")).unwrap(),
            b"external unchanged"
        );
    }
}
