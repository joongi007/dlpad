//! Synthetic save-location regression. Does not create real Documents directories.
use dlpad_engine::{
    library::{self, LibraryError},
    location::{self, LocationRequest},
    metrics,
};
use std::{
    io,
    path::{Path, PathBuf},
};

fn restore(data: &Path, expected: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        location::prepare_with_default(data, LocationRequest::Restore, || panic!(
            "saved root must win"
        ))?,
        expected
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("dlpad-location-owned-")
        .tempdir()?;
    let data = root.path().join("앱 데이터");
    let synthetic = root
        .path()
        .join("사용자 김한글")
        .join("OneDrive - 합성 조직")
        .join("문서 📁 漢字")
        .join("dlpad");
    let selected =
        location::prepare_with_default(&data, LocationRequest::Default, || Ok(synthetic.clone()))?;
    let sentinel = selected.join("원본 보존.txt");
    std::fs::write(&sentinel, b"owned synthetic sentinel")?;
    assert_eq!(
        location::prepare_with_default(&data, LocationRequest::Default, || Ok(synthetic.clone()))?,
        selected
    );
    assert_eq!(std::fs::read(&sentinel)?, b"owned synthetic sentinel");
    let explicit = location::prepare_with_default(
        &data,
        LocationRequest::Selected(root.path().join("별도 저장 🎥")),
        || panic!("selected must bypass provider"),
    )?;
    restore(&data, &explicit)?;
    let lookup = location::prepare_with_default(&data, LocationRequest::Default, || {
        Err(LibraryError::DocumentsUnavailable)
    });
    assert!(matches!(lookup, Err(LibraryError::DocumentsUnavailable)));
    restore(&data, &explicit)?;
    let denied = location::prepare_with_default(&data, LocationRequest::Default, || {
        Err(LibraryError::Io(io::Error::from(
            io::ErrorKind::PermissionDenied,
        )))
    });
    assert!(
        matches!(denied, Err(LibraryError::Io(ref e)) if e.kind() == io::ErrorKind::PermissionDenied)
    );
    restore(&data, &explicit)?;
    let file = root.path().join("디렉터리 아님");
    std::fs::write(&file, b"owned file preserved")?;
    assert!(
        location::prepare_with_default(
            &data,
            LocationRequest::Selected(file.clone()),
            || unreachable!()
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&file)?, b"owned file preserved");
    let child = file.join("must-not-create");
    assert!(
        location::prepare_with_default(
            &data,
            LocationRequest::Selected(child.clone()),
            || unreachable!()
        )
        .is_err()
    );
    assert!(!child.exists());
    restore(&data, &explicit)?;
    assert!(matches!(
        location::prepare_with_default(
            &data,
            LocationRequest::Selected(PathBuf::from("relative")),
            || unreachable!()
        ),
        Err(LibraryError::UnsupportedPath)
    ));
    let overlap = data.join("must-not-create");
    assert!(
        location::prepare_with_default(
            &data,
            LocationRequest::Selected(overlap.clone()),
            || unreachable!()
        )
        .is_err()
    );
    assert!(!overlap.exists());
    restore(&data, &explicit)?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let malformed = root.path().join(std::ffi::OsString::from_wide(&[0xd800]));
        assert!(matches!(
            location::prepare_with_default(
                &data,
                LocationRequest::Selected(malformed),
                || unreachable!()
            ),
            Err(LibraryError::UnsupportedPath)
        ));
        restore(&data, &explicit)?;
    }
    let volume_root: PathBuf = explicit.components().take(2).collect();
    #[cfg(windows)]
    assert!(metrics::volume_free_bytes(&volume_root)?.is_some());
    #[cfg(not(windows))]
    let _ = volume_root;
    let available = metrics::volume_free_bytes(&explicit)?;
    assert!(
        available.is_some(),
        "Windows target volume must be measured"
    );
    assert!(metrics::volume_free_bytes(&root.path().join("missing-volume-path")).is_err());
    let os_documents = library::default_library_root();
    let documents_lookup_available = match os_documents {
        Ok(path) => {
            assert!(path.is_absolute());
            assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("dlpad"));
            true
        }
        Err(LibraryError::DocumentsUnavailable) => false,
        Err(e) => return Err(e.into()),
    };
    assert_eq!(std::fs::read(&sentinel)?, b"owned synthetic sentinel");
    println!(
        "{}",
        serde_json::json!({
            "schema": 1, "passed": true,
            "synthetic_unicode_redirect_create_reuse": true,
            "saved_selection_preserved_after_failures": true,
            "original_sentinels_preserved": true,
            "injected_lookup_permission_denied_verified": true,
            "native_create_access_denied_verified": false,
            "actual_documents_lookup_available": documents_lookup_available,
            "actual_documents_creation_attempted": false,
            "actual_onedrive_profile_verified": false,
            "volume_available_bytes": available,
            "missing_volume_error_verified": true,
            "volume_unknown_verified": false,
            "windows_root_volume_lookup_verified": cfg!(windows),
            "windows_root_save_creation_verified": false,
            "malformed_utf16_rejected": cfg!(windows)
        })
    );
    Ok(())
}
