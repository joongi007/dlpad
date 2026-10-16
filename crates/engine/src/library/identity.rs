//! Filesystem identity evidence, never inferred from names, timestamps, or contents.
use std::{fs::Metadata, path::Path};

/// Identity is scoped by the selected library root by its caller. It is evidence
/// for an observation, not a permanent ID: inode reuse and hardlinks exist.
pub(super) fn observe(path: &Path, metadata: &Metadata) -> Option<String> {
    #[cfg(unix)]
    {
        let _ = path;
        use std::os::unix::fs::MetadataExt;
        Some(format!("unix:{}:{}", metadata.dev(), metadata.ino()))
    }
    #[cfg(windows)]
    {
        let file = crate::file_identity::open_regular_no_reparse(path).ok()?;
        if crate::file_identity::final_path(&file).ok()? != std::fs::canonicalize(path).ok()? {
            return None;
        }
        let current = file.metadata().ok()?;
        if current.len() != metadata.len() || current.modified().ok() != metadata.modified().ok() {
            return None;
        }
        let crate::file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } = crate::file_identity::from_file(&file).ok()?
        else {
            return None;
        };
        let id = file_id
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Some(format!("windows:v1:{volume_serial}:{id}"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (path, metadata);
        None
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rename_and_hardlink_share_identity_but_independent_files_do_not() {
        let fixture = tempfile::tempdir().unwrap();
        let original = fixture.path().join("원본 영상.mp4");
        let renamed = fixture.path().join("새 이름.mp4");
        fs::write(&original, b"synthetic").unwrap();
        let identity = observe(&original, &fs::metadata(&original).unwrap()).unwrap();
        fs::rename(&original, &renamed).unwrap();
        assert_eq!(
            observe(&renamed, &fs::metadata(&renamed).unwrap()),
            Some(identity.clone())
        );
        let linked = fixture.path().join("linked.mp4");
        fs::hard_link(&renamed, &linked).unwrap();
        assert_eq!(
            observe(&linked, &fs::metadata(&linked).unwrap()),
            Some(identity.clone())
        );
        let independent = fixture.path().join("independent.mp4");
        fs::write(&independent, b"synthetic").unwrap();
        assert_ne!(
            observe(&independent, &fs::metadata(&independent).unwrap()),
            Some(identity)
        );
    }
}
