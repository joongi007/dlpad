//! OS file identity observed through an owned handle, never a content hash.
use serde::{Deserialize, Serialize};
use std::{fs::File, io};
#[cfg(windows)]
mod windows;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NativeFileIdentity {
    Unix {
        device: u64,
        inode: u64,
    },
    Windows {
        volume_serial: u64,
        file_id: [u8; 16],
    },
}
/// The caller still owns path containment, symlink/reparse and file-type policy.
pub fn from_file(file: &File) -> io::Result<NativeFileIdentity> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata()?;
        Ok(NativeFileIdentity::Unix {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(windows)]
    {
        windows::from_file(file)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native file identity unavailable",
        ))
    }
}
/// Open an existing regular Windows file without following its final reparse point.
/// Ancestor-path races require separate caller ownership and containment policy.
#[cfg(windows)]
pub fn open_regular_no_reparse(path: &std::path::Path) -> io::Result<File> {
    windows::open_regular_no_reparse(path)
}
/// Observe the current final path through an owned Windows handle.
#[cfg(windows)]
pub fn final_path(file: &File) -> io::Result<std::path::PathBuf> {
    windows::final_path(file)
}
/// Validate regular-file policy and strict descendant final handle path.
#[cfg(windows)]
pub fn ensure_regular_under(root: &File, file: &File) -> io::Result<()> {
    windows::ensure_regular_under(root, file)
}
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn hardlink_and_rename_keep_identity_while_replacement_is_distinct() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("synthetic");
        let link = directory.path().join("link");
        let renamed = directory.path().join("renamed");
        std::fs::write(&source, b"fixture").unwrap();
        let original = File::open(&source).unwrap();
        let identity = from_file(&original).unwrap();
        std::fs::hard_link(&source, &link).unwrap();
        std::fs::rename(&source, &renamed).unwrap();
        assert_eq!(from_file(&File::open(link).unwrap()).unwrap(), identity);
        assert_eq!(from_file(&File::open(renamed).unwrap()).unwrap(), identity);
        std::fs::write(&source, b"fixture").unwrap();
        assert_ne!(from_file(&File::open(source).unwrap()).unwrap(), identity);
        assert_eq!(
            serde_json::from_str::<NativeFileIdentity>(&serde_json::to_string(&identity).unwrap())
                .unwrap(),
            identity
        );
    }
}
