//! Process-lifetime, user-data-directory single-instance lock. Never kills a PID.
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};
const OWNER: &[u8] = b"dlpad-user-data-v1\n";

#[derive(Debug)]
pub enum InstanceError {
    AlreadyRunning,
    UnsafeDirectory,
    Io(io::Error),
}
impl std::fmt::Display for InstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => f.write_str("dlpad is already running for this data directory"),
            Self::UnsafeDirectory => {
                f.write_str("a dedicated dlpad data directory without symlinks is required")
            }
            Self::Io(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for InstanceError {}
impl From<io::Error> for InstanceError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub struct InstanceGuard {
    file: File,
    root: PathBuf,
}
impl InstanceGuard {
    /// Keep the guard alive until all workers have stopped. The lockfile is persistent;
    /// unlinking it would permit another process to lock a different inode.
    pub fn acquire(root: &Path) -> Result<Self, InstanceError> {
        check_selected_root(root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(root)?;
        }
        #[cfg(not(unix))]
        fs::create_dir_all(root)?;
        check_selected_root(root)?;
        let root = root.canonicalize()?;
        let marker = root.join(".dlpad-data-owner");
        check_components(&marker)?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&marker)
        {
            Ok(mut marker) => {
                marker.write_all(OWNER)?;
                marker.sync_all()?;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
        let mut marker_file = File::open(&marker)?;
        if marker_file.metadata()?.len() != OWNER.len() as u64 {
            return Err(InstanceError::UnsafeDirectory);
        }
        let mut bytes = [0u8; OWNER.len()];
        marker_file.read_exact(&mut bytes)?;
        if bytes.as_slice() != OWNER {
            return Err(InstanceError::UnsafeDirectory);
        }
        let path = root.join("instance.lock");
        check_components(&path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => Ok(Self { file, root }),
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || fs2::lock_contended_error()
                        .raw_os_error()
                        .is_some_and(|code| e.raw_os_error() == Some(code)) =>
            {
                Err(InstanceError::AlreadyRunning)
            }
            Err(e) => Err(e.into()),
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}
impl Drop for InstanceGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}
fn check_selected_root(path: &Path) -> Result<(), InstanceError> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(InstanceError::UnsafeDirectory);
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(InstanceError::UnsafeDirectory),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn check_components(path: &Path) -> Result<(), InstanceError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(InstanceError::UnsafeDirectory);
        }
        current.push(component.as_os_str());
        // A Windows drive/verbatim prefix alone is not a filesystem object.
        // Keep it for the following RootDir, then inspect the rooted path.
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(InstanceError::UnsafeDirectory);
            }
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_is_rejected_drop_releases_and_lockfile_survives() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("data");
        let first = InstanceGuard::acquire(&root).unwrap();
        assert_eq!(first.root(), root.canonicalize().unwrap());
        assert!(matches!(
            InstanceGuard::acquire(&root),
            Err(InstanceError::AlreadyRunning)
        ));
        drop(first);
        assert!(root.join("instance.lock").exists());
        assert!(InstanceGuard::acquire(&root).is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn new_directory_is_private_existing_permissions_are_preserved() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let new = dir.path().join("new");
        let guard = InstanceGuard::acquire(&new).unwrap();
        assert_eq!(
            fs::metadata(&new).unwrap().permissions().mode() & 0o777,
            0o700
        );
        drop(guard);
        fs::set_permissions(&new, fs::Permissions::from_mode(0o755)).unwrap();
        let _guard = InstanceGuard::acquire(&new).unwrap();
        assert_eq!(
            fs::metadata(&new).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
    #[test]
    fn wrong_marker_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".dlpad-data-owner"), b"foreign").unwrap();
        assert!(matches!(
            InstanceGuard::acquire(dir.path()),
            Err(InstanceError::UnsafeDirectory)
        ));
        assert_eq!(
            fs::read(dir.path().join(".dlpad-data-owner")).unwrap(),
            b"foreign"
        );
    }
    #[cfg(unix)]
    #[test]
    fn symlinked_lock_does_not_touch_target() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("data");
        fs::create_dir(&root).unwrap();
        let media = dir.path().join("video.mp4");
        fs::write(&media, b"fixture").unwrap();
        std::os::unix::fs::symlink(&media, root.join("instance.lock")).unwrap();
        assert!(matches!(
            InstanceGuard::acquire(&root),
            Err(InstanceError::UnsafeDirectory)
        ));
        assert_eq!(fs::read(&media).unwrap(), b"fixture");
    }
    #[cfg(unix)]
    #[test]
    fn alias_parent_locks_same_canonical_root_and_root_symlink_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir(&real).unwrap();
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        let guard = InstanceGuard::acquire(&alias.join("data")).unwrap();
        assert!(matches!(
            InstanceGuard::acquire(&real.join("data")),
            Err(InstanceError::AlreadyRunning)
        ));
        drop(guard);
        let linked_root = dir.path().join("linked-root");
        std::os::unix::fs::symlink(real.join("data"), &linked_root).unwrap();
        assert!(matches!(
            InstanceGuard::acquire(&linked_root),
            Err(InstanceError::UnsafeDirectory)
        ));
    }
}
