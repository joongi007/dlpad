//! Explicit inactive package removal; all preflight errors preserve the full set.
use super::*;
use crate::file_identity;
fn entries(path: &Path, names: &[String]) -> Result<(), ToolError> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        count += 1;
        if count > 10 {
            return Err(ToolError::InventoryLimit);
        }
        let entry = entry?;
        if !entry.file_type()?.is_file()
            || !names
                .iter()
                .any(|n| entry.file_name() == std::ffi::OsStr::new(n))
        {
            return Err(ToolError::UnexpectedFiles);
        }
    }
    if count != names.len() {
        return Err(ToolError::UnexpectedFiles);
    }
    Ok(())
}
#[cfg(windows)]
fn delete_cap(path: &Path, directory: bool) -> Result<File, ToolError> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::*;
    safe_components(path)?;
    let file = OpenOptions::new()
        .access_mode(DELETE | FILE_GENERIC_READ)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        )
        .open(path)?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: original owned handle and correctly aligned output.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || (!directory
            && (info.nNumberOfLinks != 1 || info.dwFileAttributes & FILE_ATTRIBUTE_READONLY != 0))
    {
        return Err(ToolError::UnsafePath);
    }
    Ok(file)
}
#[cfg(windows)]
fn dispose(file: &File) -> Result<(), ToolError> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::*;
    let mut info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: verified original DELETE capability; no path lookup or recursive fallback.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&mut info as *mut FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}
impl ToolManager {
    pub fn remove_package(&mut self, package: &ArchivePackage) -> Result<(), ToolError> {
        package.validate()?;
        let _install_lock = self.acquire_install_lock()?;
        if self.active(&package.tool)?
            == Some((package.tool_version.clone(), package.target.clone()))
        {
            return Err(ToolError::ActiveVersion);
        }
        let path = self
            .root
            .join(&package.tool)
            .join(&package.tool_version)
            .join(&package.target);
        self.verify_package(&path, Some(package))?;
        let directory = super::native_files::open_directory(&path)?;
        let mut names: Vec<String> = package.components.iter().map(|c| c.name.clone()).collect();
        names.extend(["manifest.json".into(), "archive-package.json".into()]);
        entries(&path, &names)?;
        let mut pins = Vec::with_capacity(names.len());
        for name in &names {
            let p = path.join(name);
            #[cfg(windows)]
            let file = delete_cap(&p, false)?;
            #[cfg(not(windows))]
            let file = super::native_files::open_regular(&p)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if file.metadata()?.nlink() != 1 {
                    return Err(ToolError::UnsafePath);
                }
            }
            #[cfg(windows)]
            file_identity::ensure_regular_under(&directory, &file)?;
            let id = file_identity::from_file(&file)?;
            if let Some(component) = package.components.iter().find(|c| c.name == *name) {
                let mut reader = &file;
                let mut hash = Sha256::new();
                let mut bytes = 0u64;
                let mut buffer = [0u8; 65536];
                loop {
                    let n = reader.read(&mut buffer)?;
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
            pins.push((name.clone(), file, id));
        }
        // Recheck content after exclusive Windows DELETE caps (or retained Unix pins).
        self.verify_package(&path, Some(package))?;
        #[cfg(windows)]
        let directory_delete = delete_cap(&path, true)?;
        for (name, file, id) in &pins {
            if file_identity::from_file(file)? != *id
                || !super::native_files::same_file(&path.join(name), file)
            {
                return Err(ToolError::UnsafePath);
            }
        }
        if !super::native_files::same_directory(&path, &directory) {
            return Err(ToolError::UnsafePath);
        }
        entries(&path, &names)?;
        let mut remaining = names.clone();
        for (name, file, id) in pins {
            entries(&path, &remaining)?;
            if !super::native_files::same_directory(&path, &directory)
                || file_identity::from_file(&file)? != id
                || !super::native_files::same_file(&path.join(&name), &file)
            {
                return Err(ToolError::UnsafePath);
            }
            #[cfg(windows)]
            dispose(&file)?;
            #[cfg(unix)]
            {
                use std::os::fd::AsRawFd;
                let name =
                    std::ffi::CString::new(name.as_str()).map_err(|_| ToolError::UnsafePath)?;
                // SAFETY: retained original directory and validated single component.
                if unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), 0) } != 0 {
                    return Err(io::Error::last_os_error().into());
                }
            }
            #[cfg(not(any(unix, windows)))]
            return Err(ToolError::UnsafePath);
            drop(file);
            remaining.retain(|entry| entry != &name);
        }
        if !super::native_files::same_directory(&path, &directory)
            || fs::read_dir(&path)?.next().is_some()
        {
            return Err(ToolError::UnexpectedFiles);
        }
        #[cfg(windows)]
        {
            dispose(&directory_delete)?;
            drop(directory);
            drop(directory_delete);
        }
        #[cfg(unix)]
        {
            fs::remove_dir(&path)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path) -> (ToolManager, ArchivePackage, PathBuf) {
        let manager = ToolManager::new(&root.join("managed")).unwrap();
        let component = ArchiveComponent {
            name: "tool".into(),
            bytes: 4,
            sha256: format!("{:x}", Sha256::digest(b"data")),
            role: ArchiveComponentRole::Executable,
        };
        let p = ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "fixture".into(),
            tool_version: "1".into(),
            target: "test".into(),
            source_url: "https://example.invalid/a.zip".into(),
            license: "MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: 100,
            archive_sha256: "a".repeat(64),
            components: vec![component],
        };
        let path = manager.root.join("fixture/1/test");
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("tool"), b"data").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path.join("tool"), fs::Permissions::from_mode(0o700)).unwrap();
        }
        let m = InstallManifest {
            tool: p.tool.clone(),
            version: p.tool_version.clone(),
            target: p.target.clone(),
            source_url: p.source_url.clone(),
            license: p.license.clone(),
            ownership: Ownership::Managed,
            executable: "tool".into(),
            sha256: p.components[0].sha256.clone(),
        };
        fs::write(path.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        fs::write(
            path.join("archive-package.json"),
            serde_json::to_vec(&p).unwrap(),
        )
        .unwrap();
        (manager, p, path)
    }
    #[test]
    fn inactive_removes_exact_set_active_and_foreign_preserve() {
        let temp = tempfile::tempdir().unwrap();
        let (mut manager, p, path) = fixture(temp.path());
        manager.select_package(&p).unwrap();
        assert!(matches!(
            manager.remove_package(&p),
            Err(ToolError::ActiveVersion)
        ));
        assert!(path.join("tool").exists());
        manager.db.execute("DELETE FROM active", []).unwrap();
        let foreign = path.join("foreign");
        fs::write(&foreign, b"preserve").unwrap();
        assert!(manager.remove_package(&p).is_err());
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve");
        assert!(path.join("manifest.json").exists());
        fs::remove_file(foreign).unwrap();
        let outside = temp.path().join("source");
        fs::write(&outside, b"preserve").unwrap();
        manager.remove_package(&p).unwrap();
        assert!(!path.exists());
        assert_eq!(fs::read(outside).unwrap(), b"preserve");
    }
    #[test]
    fn hash_failure_preserves_all_files() {
        let temp = tempfile::tempdir().unwrap();
        let (mut manager, p, path) = fixture(temp.path());
        fs::write(path.join("tool"), b"evil").unwrap();
        assert!(matches!(
            manager.remove_package(&p),
            Err(ToolError::HashMismatch)
        ));
        assert_eq!(fs::read(path.join("tool")).unwrap(), b"evil");
        assert!(path.join("manifest.json").exists() && path.join("archive-package.json").exists());
    }
    #[cfg(unix)]
    #[test]
    fn replaced_symlink_preserves_external() {
        let temp = tempfile::tempdir().unwrap();
        let (mut manager, p, path) = fixture(temp.path());
        let outside = temp.path().join("outside");
        fs::write(&outside, b"data").unwrap();
        fs::remove_file(path.join("tool")).unwrap();
        std::os::unix::fs::symlink(&outside, path.join("tool")).unwrap();
        assert!(manager.remove_package(&p).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"data");
        assert!(path.join("manifest.json").exists());
    }
    #[cfg(windows)]
    #[test]
    fn preflight_delete_denied_preserves_full_set() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
        let temp = tempfile::tempdir().unwrap();
        let (mut manager, p, path) = fixture(temp.path());
        let held = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(path.join("manifest.json"))
            .unwrap();
        assert!(manager.remove_package(&p).is_err());
        assert!(
            path.join("tool").exists()
                && path.join("manifest.json").exists()
                && path.join("archive-package.json").exists()
        );
        drop(held);
    }
}
