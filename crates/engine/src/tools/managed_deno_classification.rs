//! Read-only selected Deno classification. No activation or cache ownership authority.
use super::*;
use crate::file_identity;
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

pub(crate) struct VerifiedManagedDeno {
    root: PathBuf,
    root_pin: File,
    owner_pin: File,
    directory_pin: File,
    installed: InstalledPackage,
    files: Vec<(PathBuf, File)>,
}
impl std::fmt::Debug for VerifiedManagedDeno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VerifiedManagedDenoRedacted")
    }
}
fn compatible(target: &str) -> bool {
    (cfg!(all(target_os = "windows", target_arch = "x86_64")) && target == "x86_64-pc-windows-msvc")
        || (cfg!(all(target_os = "linux", target_arch = "x86_64"))
            && target == "x86_64-unknown-linux-gnu")
        || (cfg!(all(target_os = "macos", target_arch = "x86_64"))
            && target == "x86_64-apple-darwin")
        || (cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && target == "aarch64-apple-darwin")
}
fn checked_components(path: &Path) -> Result<(), ToolError> {
    if path.components().count() > 512 {
        return Err(ToolError::UnsafePath);
    }
    safe_components(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let mut current = PathBuf::new();
        for part in path.components() {
            current.push(part.as_os_str());
            if matches!(part, Component::Prefix(_)) {
                continue;
            }
            match fs::symlink_metadata(&current) {
                Ok(meta) if meta.file_attributes() & 0x400 != 0 => {
                    return Err(ToolError::UnsafePath);
                }
                Ok(_) => (),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}
fn single_link(file: &File) -> Result<(), ToolError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if file.metadata()?.nlink() != 1 {
            return Err(ToolError::UnsafePath);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::*;
        let mut info = FILE_STANDARD_INFO::default();
        // SAFETY: borrowed live disk handle and correctly sized writable output.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut info as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error().into());
        }
        if info.NumberOfLinks != 1 || info.DeletePending || info.Directory {
            return Err(ToolError::UnsafePath);
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = file;
        return Err(ToolError::UnsafePath);
    }
    Ok(())
}
fn missing_is_external(
    selected: &Path,
    root: &Path,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<bool, ToolError> {
    let mut ancestor = selected.parent();
    for _ in 0..512 {
        direct_download::checkpoint(cancel, deadline)?;
        let Some(path) = ancestor else {
            return Err(ToolError::UnsafePath);
        };
        checked_components(path)?;
        match path.canonicalize() {
            Ok(actual) => return Ok(!actual.starts_with(root)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => ancestor = path.parent(),
            Err(e) => return Err(e.into()),
        }
    }
    Err(ToolError::UnsafePath)
}
impl VerifiedManagedDeno {
    #[cfg(windows)]
    pub(crate) fn root_identity(&self) -> io::Result<file_identity::NativeFileIdentity> {
        file_identity::from_file(&self.root_pin)
    }
    pub(crate) fn executable(&self) -> &Path {
        &self.installed.main.executable
    }
    pub(crate) fn root_path(&self) -> &Path {
        &self.root
    }
    pub(crate) fn verify(
        &self,
        manager: &ToolManager,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), ToolError> {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .ok_or(ToolError::TimedOut)?;
        self.verify_until(manager, cancel, deadline)
    }
    fn verify_until(
        &self,
        manager: &ToolManager,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<(), ToolError> {
        manager.verify_root()?;
        if manager.root != self.root
            || file_identity::from_file(&manager.root_pin)?
                != file_identity::from_file(&self.root_pin)?
        {
            return Err(ToolError::UnsafePath);
        }
        self.verify_retained_until(cancel, deadline)
    }
    pub(crate) fn verify_retained_until(
        &self,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<(), ToolError> {
        direct_download::checkpoint(cancel, deadline)?;
        super::verify_root_pin(&self.root, &self.root_pin)?;
        checked_components(&self.root)?;
        checked_components(&self.root.join(".dlpad-owner"))?;
        checked_components(&self.installed.directory)?;
        single_link(&self.owner_pin)?;
        if !native_files::same_directory(&self.root, &self.root_pin)
            || !native_files::same_file(&self.root.join(".dlpad-owner"), &self.owner_pin)
            || !native_files::same_directory(&self.installed.directory, &self.directory_pin)
        {
            return Err(ToolError::UnsafePath);
        }
        for (path, original) in &self.files {
            direct_download::checkpoint(cancel, deadline)?;
            checked_components(path)?;
            single_link(original)?;
            if !native_files::same_file(path, original) {
                return Err(ToolError::UnsafePath);
            }
        }
        let actual = super::package_inventory::verify_package_from_root(
            &self.root,
            &self.root_pin,
            &self.installed.directory,
            Some(&self.installed.package),
            true,
            &mut || direct_download::checkpoint(cancel, deadline).map(|_| ()),
        )?;
        if serde_json::to_vec(&actual.main.manifest)?
            != serde_json::to_vec(&self.installed.main.manifest)?
            || actual.main.executable != self.installed.main.executable
            || actual.main.bytes != self.installed.main.bytes
        {
            return Err(ToolError::InvalidManifest);
        }
        for (path, original) in &self.files {
            direct_download::checkpoint(cancel, deadline)?;
            checked_components(path)?;
            single_link(original)?;
            if !native_files::same_file(path, original) {
                return Err(ToolError::UnsafePath);
            }
        }
        if !native_files::same_directory(&self.installed.directory, &self.directory_pin)
            || !native_files::same_file(&self.root.join(".dlpad-owner"), &self.owner_pin)
        {
            return Err(ToolError::UnsafePath);
        }
        checked_components(&self.root)?;
        checked_components(&self.root.join(".dlpad-owner"))?;
        single_link(&self.owner_pin)?;
        super::verify_root_pin(&self.root, &self.root_pin)?;
        direct_download::checkpoint(cancel, deadline).map(|_| ())
    }
}
impl ToolManager {
    /// Owns exact managed package/cache proof. External selections create no cache.
    #[cfg(windows)]
    pub fn prepare_managed_deno_environment(
        &self,
        selected: &Path,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<crate::runtime_environment::WindowsManagedDenoEnvironment>, ToolError> {
        let Some(verified) = self.classify_managed_deno(selected, cancel)? else {
            return Ok(None);
        };
        let cache = crate::windows_owned_deno_cache::OwnedDenoCache::create_or_reopen(
            &self.root,
            &self.root_pin,
        )
        .map_err(|_| ToolError::UnsafePath)?;
        verified.verify(self, cancel)?;
        crate::runtime_environment::WindowsManagedDenoEnvironment::from_owned(cache, verified)
            .map(Some)
            .map_err(|_| ToolError::UnsafePath)
    }
    pub(crate) fn classify_managed_deno(
        &self,
        selected: &Path,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<VerifiedManagedDeno>, ToolError> {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .ok_or(ToolError::TimedOut)?;
        direct_download::checkpoint(cancel, deadline)?;
        self.verify_root()?;
        checked_components(&self.root)?;
        if !selected.is_absolute()
            || selected
                .components()
                .any(|c| matches!(c, Component::ParentDir))
        {
            return Err(ToolError::UnsafePath);
        }
        // Canonical lookup also prevents aliases/case differences from hiding a root member.
        let lexical_inside = selected.starts_with(&self.root);
        let actual = match selected.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if !lexical_inside && missing_is_external(selected, &self.root, cancel, deadline)? {
                    return Ok(None);
                }
                return Err(ToolError::UnsafePath);
            }
            Err(e) => return Err(e.into()),
        };
        if !lexical_inside && !actual.starts_with(&self.root) {
            return Ok(None);
        }
        checked_components(selected)?;
        checked_components(&actual)?;
        let relative = actual
            .strip_prefix(&self.root)
            .map_err(|_| ToolError::UnsafePath)?;
        if relative.components().count() != 4 {
            return Err(ToolError::InvalidManifest);
        }
        let directory = actual.parent().ok_or(ToolError::UnsafePath)?;
        let installed = self.verify_package_with_checkpoint(directory, None, &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        if installed.package.tool != "deno"
            || installed.main.manifest.tool != "deno"
            || !compatible(&installed.package.target)
            || installed.main.executable != actual
        {
            return Err(ToolError::InvalidManifest);
        }
        let mut names = vec![
            "manifest.json".to_string(),
            "archive-package.json".to_string(),
        ];
        names.extend(installed.package.components.iter().map(|c| c.name.clone()));
        let files = names
            .into_iter()
            .map(|name| {
                let path = directory.join(name);
                native_files::open_regular(&path).map(|pin| (path, pin))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let verified = VerifiedManagedDeno {
            root: self.root.clone(),
            root_pin: self.root_pin.try_clone()?,
            owner_pin: native_files::open_regular(&self.root.join(".dlpad-owner"))?,
            directory_pin: native_files::open_directory(directory)?,
            installed,
            files,
        };
        verified.verify_until(self, cancel, deadline)?;
        Ok(Some(verified))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn target() -> &'static str {
        if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64-apple-darwin"
            }
        } else {
            "x86_64-unknown-linux-gnu"
        }
    }
    fn fixture(root: &Path) -> (ToolManager, PathBuf) {
        fixture_target(root, target())
    }
    fn fixture_target(root: &Path, release_target: &str) -> (ToolManager, PathBuf) {
        let manager = ToolManager::new(&root.join("managed")).unwrap();
        let executable = if cfg!(windows) { "deno.exe" } else { "deno" };
        let package = ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "deno".into(),
            tool_version: "2.9.7".into(),
            target: release_target.into(),
            source_url: "https://fixture.invalid/deno.zip".into(),
            license: "MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: 123,
            archive_sha256: "a".repeat(64),
            components: [
                (executable, ArchiveComponentRole::Executable),
                ("LICENSE", ArchiveComponentRole::Notice),
            ]
            .into_iter()
            .map(|(name, role)| ArchiveComponent {
                name: name.into(),
                role,
                bytes: 4,
                sha256: format!("{:x}", Sha256::digest(b"data")),
            })
            .collect(),
        };
        let manifest = InstallManifest {
            tool: package.tool.clone(),
            version: package.tool_version.clone(),
            target: package.target.clone(),
            source_url: package.source_url.clone(),
            license: package.license.clone(),
            ownership: Ownership::Managed,
            executable: executable.into(),
            sha256: package.components[0].sha256.clone(),
        };
        let directory = manager.install_dir(&manifest).unwrap();
        fs::create_dir_all(&directory).unwrap();
        for c in &package.components {
            fs::write(directory.join(&c.name), b"data").unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                directory.join(executable),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("archive-package.json"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        (manager, directory.join(executable))
    }
    #[test]
    fn explicit_nonactive_full_package_and_external_none_are_read_only() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(manager.active("deno").unwrap().is_none());
        let verified = manager
            .classify_managed_deno(&path, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(verified.executable(), path);
        assert_eq!(verified.root_path(), manager.root);
        verified.verify(&manager, &cancel).unwrap();
        assert!(manager.active("deno").unwrap().is_none());
        let external = temp.path().join("deno-external");
        fs::write(&external, b"foreign").unwrap();
        assert!(
            manager
                .classify_managed_deno(&external, &cancel)
                .unwrap()
                .is_none()
        );
        assert_eq!(fs::read(external).unwrap(), b"foreign");
        assert!(
            manager
                .classify_managed_deno(&path.with_file_name("LICENSE"), &cancel)
                .is_err()
        );
        assert!(
            manager
                .classify_managed_deno(&manager.root.join("unknown"), &cancel)
                .is_err()
        );
    }
    #[test]
    fn content_inventory_manifest_target_and_cancel_fail_closed() {
        for mode in ["main", "notice", "extra", "manifest", "target", "malformed"] {
            let temp = tempfile::tempdir().unwrap();
            let (manager, path) = fixture(temp.path());
            let cancel = Arc::new(AtomicBool::new(false));
            let directory = path.parent().unwrap();
            let verified = manager
                .classify_managed_deno(&path, &cancel)
                .unwrap()
                .unwrap();
            match mode {
                "main" => fs::write(&path, b"evil").unwrap(),
                "notice" => fs::write(directory.join("LICENSE"), b"evil").unwrap(),
                "extra" => fs::write(directory.join("foreign"), b"foreign").unwrap(),
                "malformed" => fs::write(directory.join("archive-package.json"), b"{").unwrap(),
                "manifest" => {
                    let mut m: InstallManifest =
                        serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap())
                            .unwrap();
                    m.tool = "other".into();
                    fs::write(
                        directory.join("manifest.json"),
                        serde_json::to_vec(&m).unwrap(),
                    )
                    .unwrap();
                }
                "target" => {
                    let mut p: ArchivePackage = serde_json::from_slice(
                        &fs::read(directory.join("archive-package.json")).unwrap(),
                    )
                    .unwrap();
                    p.target = "wrong-os".into();
                    fs::write(
                        directory.join("archive-package.json"),
                        serde_json::to_vec(&p).unwrap(),
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
            assert!(verified.verify(&manager, &cancel).is_err(), "{mode}");
            assert!(
                manager.classify_managed_deno(&path, &cancel).is_err(),
                "{mode}"
            );
        }
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        assert!(matches!(
            manager.classify_managed_deno(&path, &Arc::new(AtomicBool::new(true))),
            Err(ToolError::Cancelled)
        ));
        assert_eq!(fs::read(path).unwrap(), b"data");
    }
    #[test]
    fn fully_consistent_wrong_platform_package_is_not_managed_deno() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture_target(temp.path(), "wrong-platform");
        assert!(manager.verify_package(path.parent().unwrap(), None).is_ok());
        assert!(matches!(
            manager.classify_managed_deno(&path, &Arc::new(AtomicBool::new(false))),
            Err(ToolError::InvalidManifest)
        ));
    }
    #[cfg(windows)]
    #[test]
    fn windows_missing_plain_and_canonical_inside_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let missing = path.parent().unwrap().join("missing").join("deno.exe");
        assert!(manager.classify_managed_deno(&missing, &cancel).is_err());
        let text = missing.to_str().unwrap();
        let plain = PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(text));
        assert!(manager.classify_managed_deno(&plain, &cancel).is_err());
        let lower = PathBuf::from(plain.to_str().unwrap().to_lowercase());
        assert!(manager.classify_managed_deno(&lower, &cancel).is_err());
        assert!(
            manager
                .classify_managed_deno(
                    &temp.path().join("external-missing").join("deno.exe"),
                    &cancel
                )
                .unwrap()
                .is_none()
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_hardlink_main_and_owner_rejected_without_mutation() {
        for owner in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let (manager, path) = fixture(temp.path());
            let cancel = Arc::new(AtomicBool::new(false));
            let verified = manager
                .classify_managed_deno(&path, &cancel)
                .unwrap()
                .unwrap();
            let source = if owner {
                manager.root.join(".dlpad-owner")
            } else {
                path.clone()
            };
            let bytes = fs::read(&source).unwrap();
            let alias = temp.path().join("foreign-hardlink");
            fs::hard_link(&source, &alias).unwrap();
            assert!(verified.verify(&manager, &cancel).is_err());
            assert!(manager.classify_managed_deno(&path, &cancel).is_err());
            assert_eq!(fs::read(&alias).unwrap(), bytes);
            assert_eq!(fs::read(&source).unwrap(), bytes);
        }
    }
    #[cfg(unix)]
    #[test]
    fn original_id_replacement_hardlink_and_external_symlink_alias_rejected() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let verified = manager
            .classify_managed_deno(&path, &cancel)
            .unwrap()
            .unwrap();
        let original = path.with_file_name("old");
        fs::rename(&path, &original).unwrap();
        fs::write(&path, b"data").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_file(&original).unwrap();
        assert!(verified.verify(&manager, &cancel).is_err());
        let outside = temp.path().join("alias");
        symlink(&path, &outside).unwrap();
        assert!(manager.classify_managed_deno(&outside, &cancel).is_err());
        fs::remove_file(&outside).unwrap();
        fs::hard_link(&path, &outside).unwrap();
        assert!(manager.classify_managed_deno(&path, &cancel).is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"data");
    }
    #[cfg(windows)]
    #[test]
    fn owned_factory_external_none_and_arc_authority_lifetime() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let external = temp.path().join("external-deno.exe");
        fs::write(&external, b"external synthetic").unwrap();
        assert!(
            manager
                .prepare_managed_deno_environment(&external, &cancel)
                .unwrap()
                .is_none()
        );
        assert!(!manager.root.join(".runtime-cache").exists());
        let policy = manager
            .prepare_managed_deno_environment(&path, &cancel)
            .unwrap()
            .unwrap();
        let binding = policy.cache_binding().unwrap();
        assert_eq!(binding.root_path().unwrap(), manager.root);
        assert_eq!(policy.selected_executable(), Some(path.as_path()));
        assert_eq!(
            policy.cache_path(),
            manager.root.join(".runtime-cache").join("deno")
        );
        assert!(
            policy
                .authority()
                .unwrap()
                .cache
                .try_lock()
                .unwrap()
                .verify()
                .is_ok()
        );
        let cloned = policy.clone();
        assert_eq!(policy, cloned);
        drop(policy);
        assert!(
            crate::windows_owned_deno_cache::OwnedDenoCache::create_or_reopen(
                &manager.root,
                &manager.root_pin
            )
            .is_err()
        );
        assert!(
            cloned
                .authority()
                .unwrap()
                .cache
                .try_lock()
                .unwrap()
                .verify()
                .is_ok()
        );
        drop(cloned);
        assert!(
            crate::windows_owned_deno_cache::OwnedDenoCache::create_or_reopen(
                &manager.root,
                &manager.root_pin
            )
            .is_ok()
        );
        assert_eq!(fs::read(external).unwrap(), b"external synthetic");
        assert_eq!(fs::read(path).unwrap(), b"data");
        assert!(manager.active("deno").unwrap().is_none());
    }
    #[cfg(windows)]
    #[test]
    fn owned_factory_partial_cache_and_cancel_preserve_originals() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cache = crate::windows_owned_deno_cache::OwnedDenoCache::create_or_reopen(
            &manager.root,
            &manager.root_pin,
        )
        .unwrap();
        let owner = cache.path().join("owner.json");
        drop(cache);
        fs::write(&owner, b"{").unwrap();
        assert!(
            manager
                .prepare_managed_deno_environment(&path, &Arc::new(AtomicBool::new(false)))
                .is_err()
        );
        assert_eq!(fs::read(&owner).unwrap(), b"{");
        assert_eq!(fs::read(&path).unwrap(), b"data");
        assert!(manager.active("deno").unwrap().is_none());
        let fresh = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(fresh.path());
        assert!(matches!(
            manager.prepare_managed_deno_environment(&path, &Arc::new(AtomicBool::new(true))),
            Err(ToolError::Cancelled)
        ));
        assert!(!manager.root.join(".runtime-cache").exists());
    }
    #[cfg(windows)]
    #[test]
    fn owned_factory_rejects_cross_root_package_cache_authority() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(first.path());
        let (foreign, foreign_path) = fixture(second.path());
        let verified = manager
            .classify_managed_deno(&path, &Arc::new(AtomicBool::new(false)))
            .unwrap()
            .unwrap();
        let cache = crate::windows_owned_deno_cache::OwnedDenoCache::create_or_reopen(
            &foreign.root,
            &foreign.root_pin,
        )
        .unwrap();
        let owner = cache.path().join("owner.json");
        let original = fs::read(&owner).unwrap();
        assert!(
            crate::runtime_environment::WindowsManagedDenoEnvironment::from_owned(cache, verified)
                .is_err()
        );
        assert_eq!(fs::read(owner).unwrap(), original);
        assert_eq!(fs::read(path).unwrap(), b"data");
        assert_eq!(fs::read(foreign_path).unwrap(), b"data");
    }
    // Draft: insert into tools::managed_deno_classification::tests, reusing fixture(root).
    // Explicit Windows native selection only; current built adjacent helper is a prerequisite.
    #[cfg(windows)]
    fn require_original_adjacent_guardian() {
        let current = std::env::current_exe().unwrap();
        let directory = current.parent().unwrap();
        assert!(
            directory.join("dlpad-process-guard.exe").is_file()
                || directory
                    .parent()
                    .unwrap()
                    .join("dlpad-process-guard.exe")
                    .is_file(),
            "build current original guardian before native regression"
        );
    }
    #[cfg(windows)]
    fn environment_missing_tool_spec(
        root: &Path,
        policy: &crate::runtime_environment::WindowsManagedDenoEnvironment,
    ) -> crate::process::ProcessSpec {
        crate::process::ProcessSpec {
            executable: root.join("owned-fixture-no-executable.exe"),
            args: vec![],
            timeout: Duration::from_secs(3),
            max_log_bytes: 4096,
            runtime_environment: Some(policy.clone()),
        }
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "actual Windows helper required; select explicitly after matching helper build"]
    fn public_environment_foreign_execution_root_rejected_before_intent() {
        require_original_adjacent_guardian();
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        let (manager, path) = fixture(&source);
        let cancel = Arc::new(AtomicBool::new(false));
        let policy = manager
            .prepare_managed_deno_environment(&path, &cancel)
            .unwrap()
            .unwrap();
        let unrelated = ToolManager::new(&temp.path().join("other-managed")).unwrap();
        let execution = unrelated.begin_execution().unwrap();
        let spec = environment_missing_tool_spec(temp.path(), &policy);
        assert!(
            execution
                .run_tool(
                    &spec,
                    &cancel,
                    true,
                    None,
                    crate::tools::ManagedReaderStage::Metadata
                )
                .is_err()
        );
        // Error alone is insufficient: no helper/Intent may have been created on the wrong root.
        execution
            .finish()
            .expect("wrong-root policy must leave original journal pristine");
        unrelated.begin_execution().unwrap().finish().unwrap();
        assert_eq!(fs::read(path).unwrap(), b"data");
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "actual Windows helper required; select explicitly after matching helper build"]
    fn public_environment_package_notice_changed_after_factory_rejected_before_intent() {
        require_original_adjacent_guardian();
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let policy = manager
            .prepare_managed_deno_environment(&path, &cancel)
            .unwrap()
            .unwrap();
        fs::write(path.parent().unwrap().join("LICENSE"), b"evil").unwrap();
        let execution = manager.begin_execution().unwrap();
        let spec = environment_missing_tool_spec(temp.path(), &policy);
        assert!(
            execution
                .run_tool(
                    &spec,
                    &cancel,
                    true,
                    None,
                    crate::tools::ManagedReaderStage::Metadata
                )
                .is_err()
        );
        execution
            .finish()
            .expect("changed original runtime package must leave journal pristine");
        assert_eq!(
            fs::read(path.parent().unwrap().join("LICENSE")).unwrap(),
            b"evil"
        );
        assert_eq!(fs::read(path).unwrap(), b"data");
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "actual Windows original manager namespace; select explicitly"]
    fn public_environment_cancel_before_journal_is_pristine() {
        let temp = tempfile::tempdir().unwrap();
        let (manager, path) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let policy = manager
            .prepare_managed_deno_environment(&path, &cancel)
            .unwrap()
            .unwrap();
        let execution = manager.begin_execution().unwrap();
        cancel.store(true, std::sync::atomic::Ordering::Release);
        let result = execution
            .run_tool(
                &environment_missing_tool_spec(temp.path(), &policy),
                &cancel,
                true,
                None,
                crate::tools::ManagedReaderStage::Metadata,
            )
            .unwrap();
        assert_eq!(
            result.execution.outcome,
            crate::process::ProcessOutcome::Cancelled
        );
        execution
            .finish()
            .expect("pre-journal cancellation must be pristine");
    }
    // Insert into tools::managed_deno_classification::tests; reuses original fixture(root).
    // Actual Windows public repository/managed cache namespace. No tool/helper is needed:
    // the exact Policy outcome must occur before opening the deliberately missing yt-dlp.
    #[cfg(windows)]
    fn public_download_preflight_case(mode: &str) {
        use crate::download::{DownloadError, DownloadRepository, DownloadSession};
        use dlpad_core::{
            jobs::JobState,
            options::{DownloadOptions, Quality, SourceUrl},
        };
        let temp = tempfile::tempdir().unwrap();
        let (manager, deno) = fixture(temp.path());
        let cancel = Arc::new(AtomicBool::new(false));
        let policy = manager
            .prepare_managed_deno_environment(&deno, &cancel)
            .unwrap()
            .unwrap();
        let other = ToolManager::new(&temp.path().join("other-managed")).unwrap();
        let foreign = temp.path().join("foreign-deno.exe");
        fs::write(&foreign, b"synthetic foreign runtime preserved").unwrap();
        let selected = if mode == "selected-mismatch" {
            foreign.clone()
        } else {
            deno.clone()
        };
        if mode == "notice-changed" {
            fs::write(deno.parent().unwrap().join("LICENSE"), b"evil").unwrap();
        }
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/watch").unwrap(),
            output_dir: temp.path().join("synthetic-output"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: Some(selected),
            final_name_style: None,
            subtitles: None,
        };
        let repository = DownloadRepository::open(&temp.path().join("jobs.sqlite")).unwrap();
        repository
            .create_job("environment-preflight", &options)
            .unwrap();
        let execution = if mode == "root-mismatch" {
            other.begin_execution().unwrap()
        } else {
            manager.begin_execution().unwrap()
        };
        let missing = temp.path().join("owned-never-created-tool.exe");
        assert!(!missing.exists());
        let error = match repository.execute_with_session(
            "environment-preflight",
            &options,
            &missing,
            DownloadSession {
                runtime_environment: Some(&policy),
                cancel: &cancel,
                pause: None,
                progress: None,
                cookies: None,
                managed: Some(&execution),
            },
        ) {
            Err(error) => error,
            Ok(_) => panic!("invalid original runtime proof unexpectedly downloaded"),
        };
        assert!(
            matches!(
                error,
                DownloadError::Policy("managed runtime original proof changed")
            ),
            "{mode}: expected original-proof Policy, got {error:?}"
        );
        let job = repository.job("environment-preflight").unwrap();
        assert_eq!(job.state, JobState::Failed);
        assert!(job.artifacts.is_empty());
        execution
            .finish()
            .expect("preflight must leave original managed journal pristine");
        if mode == "root-mismatch" {
            other.begin_execution().unwrap().finish().unwrap();
        } else {
            manager.begin_execution().unwrap().finish().unwrap();
        }
        assert_eq!(
            fs::read(&foreign).unwrap(),
            b"synthetic foreign runtime preserved"
        );
        assert_eq!(fs::read(&deno).unwrap(), b"data");
        assert!(!missing.exists());
        if mode == "notice-changed" {
            assert_eq!(
                fs::read(deno.parent().unwrap().join("LICENSE")).unwrap(),
                b"evil"
            );
        }
    }
    #[cfg(windows)]
    #[test]
    fn public_download_foreign_managed_root_rejected_before_first_tool() {
        public_download_preflight_case("root-mismatch");
    }
    #[cfg(windows)]
    #[test]
    fn public_download_different_deno_rejected_before_first_tool() {
        public_download_preflight_case("selected-mismatch");
    }
    #[cfg(windows)]
    #[test]
    fn public_download_changed_notice_rejected_before_first_tool() {
        public_download_preflight_case("notice-changed");
    }
}
