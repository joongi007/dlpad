//! Blocking save-location use case; tests inject a temporary Documents provider.
use crate::{
    library::{self, LibraryError},
    store::Store,
};
use std::path::{Path, PathBuf};

pub enum LocationRequest {
    Restore,
    Default,
    Selected(PathBuf),
}

pub fn default_data_dir() -> Result<PathBuf, LibraryError> {
    directories::ProjectDirs::from("", "dlpad", "dlpad")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .ok_or(LibraryError::DocumentsUnavailable)
}

/// Read-only default per-user Windows install location, including an existing folder.
/// This is not an installation capability or permission to create/replace anything.
/// All paths must be revalidated by the existing install preparation use case.
pub fn default_install_target_hint() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let base = directories::BaseDirs::new()?;
        let data = default_data_dir().ok()?;
        install_hint_with(base.data_local_dir(), &data, |path| {
            std::fs::symlink_metadata(path)
        })
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(any(windows, test))]
fn install_hint_with(
    local: &Path,
    data: &Path,
    mut metadata: impl FnMut(&Path) -> std::io::Result<std::fs::Metadata>,
) -> Option<PathBuf> {
    if !install_hint_path_valid(local) || !install_hint_path_valid(data) {
        return None;
    }
    // Bound ancestor inspection; follow neither symlinks nor Windows reparse points.
    for (path, missing_allowed) in [(local, false), (data, true)] {
        let ancestors: Vec<_> = path.ancestors().take(257).collect();
        if ancestors.len() > 256 {
            return None;
        }
        for ancestor in ancestors {
            match metadata(ancestor) {
                Ok(value) if install_hint_plain_dir(&value) => (),
                Err(error) if missing_allowed && error.kind() == std::io::ErrorKind::NotFound => (),
                _ => return None,
            }
        }
    }
    let programs = local.join("Programs");
    if !install_hint_path_valid(&programs) {
        return None;
    }
    let (parent, stem) = match metadata(&programs) {
        Ok(value) if install_hint_plain_dir(&value) => (programs.as_path(), "dlpad"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (local, "dlpad-app"),
        _ => return None,
    };
    let target = parent.join(stem);
    if !install_hint_path_valid(&target) || install_hint_overlaps(&target, data) {
        return None;
    }
    match metadata(&target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(target),
        // Show the existing default so the installer can offer verification;
        // this is never proof of ownership or permission to replace its files.
        Ok(value) if install_hint_plain_dir(&value) => Some(target),
        _ => None,
    }
}

#[cfg(any(windows, test))]
fn install_hint_plain_dir(metadata: &std::fs::Metadata) -> bool {
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT includes junctions as well as symlinks.
        metadata.file_attributes() & 0x400 == 0
    }
    #[cfg(not(windows))]
    {
        true
    }
}

#[cfg(any(windows, test))]
fn install_hint_path_valid(path: &Path) -> bool {
    use std::path::Component;
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return false;
    }
    #[cfg(windows)]
    {
        use std::{os::windows::ffi::OsStrExt, path::Prefix};
        let units: Vec<_> = path.as_os_str().encode_wide().take(32768).collect();
        !units.is_empty()
            && units.len() <= 32767
            && !units.iter().any(|unit| *unit < 32 || *unit == 127)
            && String::from_utf16(&units).is_ok()
            && matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
    }
    #[cfg(not(windows))]
    {
        path.to_str().is_some_and(|text| {
            text.encode_utf16().take(32768).count() <= 32767 && !text.chars().any(char::is_control)
        })
    }
}

#[cfg(any(windows, test))]
fn install_hint_overlaps(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        // Conservative component comparison for provider-derived local paths;
        // both providers and every existing ancestor have already been checked.
        let parts = |path: &Path| {
            path.components()
                .map(|part| {
                    use std::path::{Component, Prefix};
                    match part {
                        Component::Prefix(prefix) => match prefix.kind() {
                            Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                                format!("{}:", char::from(drive).to_ascii_lowercase())
                            }
                            _ => unreachable!("validated local disk path"),
                        },
                        _ => part.as_os_str().to_string_lossy().to_lowercase(),
                    }
                })
                .collect::<Vec<_>>()
        };
        let left = parts(left);
        let right = parts(right);
        left.starts_with(&right) || right.starts_with(&left)
    }
    #[cfg(not(windows))]
    {
        left.starts_with(right) || right.starts_with(left)
    }
}

pub fn prepare(data: &Path, request: LocationRequest) -> Result<PathBuf, LibraryError> {
    prepare_with_default(data, request, library::default_library_root)
}

pub fn prepare_with_default(
    data: &Path,
    request: LocationRequest,
    default: impl FnOnce() -> Result<PathBuf, LibraryError>,
) -> Result<PathBuf, LibraryError> {
    std::fs::create_dir_all(data)?;
    let store = Store::open(&data.join("theme.sqlite"))?;
    let root = match request {
        LocationRequest::Restore => match store.setting("library_root")? {
            Some(root) => PathBuf::from(root),
            None => default()?,
        },
        LocationRequest::Default => default()?,
        LocationRequest::Selected(root) => root,
    };
    if !root.is_absolute() || root.to_str().is_none_or(|text| text.len() > 32 * 1024) {
        return Err(LibraryError::UnsupportedPath);
    }
    if std::fs::symlink_metadata(&root).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "save root symlink rejected",
        )
        .into());
    }
    // Check the nearest existing ancestor before creating any media directories.
    let actual_data = data.canonicalize()?;
    let mut ancestor = root.as_path();
    while !ancestor.try_exists()? {
        ancestor = ancestor.parent().ok_or(LibraryError::UnsupportedPath)?;
    }
    let actual_ancestor = ancestor.canonicalize()?;
    if actual_ancestor.starts_with(&actual_data) || actual_data.starts_with(&root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "save root overlaps app data",
        )
        .into());
    }
    library::create_library_root(&root)?;
    let actual_root = root.canonicalize()?;
    if actual_root.starts_with(&actual_data) || actual_data.starts_with(&actual_root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "save root overlaps app data",
        )
        .into());
    }
    let text = actual_root.to_str().ok_or(LibraryError::UnsupportedPath)?;
    if store.setting("library_root")?.as_deref() != Some(text) {
        store.set_setting("library_root", text)?;
    }
    Ok(actual_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_created_reused_and_explicit_choice_restored() {
        let directory = tempfile::tempdir().unwrap();
        let data = directory.path().join("data");
        let documents = directory.path().join("한글 문서").join("dlpad");
        let resolved =
            prepare_with_default(&data, LocationRequest::Restore, || Ok(documents.clone()))
                .unwrap();
        std::fs::write(resolved.join("existing.mp4"), b"synthetic").unwrap();
        assert_eq!(
            prepare_with_default(&data, LocationRequest::Default, || Ok(documents)).unwrap(),
            resolved
        );
        assert_eq!(
            std::fs::read(resolved.join("existing.mp4")).unwrap(),
            b"synthetic"
        );
        let selected = directory.path().join("별도 폴더");
        let chosen = prepare_with_default(&data, LocationRequest::Selected(selected), || {
            panic!("unexpected Documents lookup")
        })
        .unwrap();
        assert_eq!(
            prepare_with_default(&data, LocationRequest::Restore, || panic!(
                "saved root must win"
            ))
            .unwrap(),
            chosen
        );
    }
    #[test]
    fn failed_lookup_file_and_overlap_preserve_previous_selection() {
        let directory = tempfile::tempdir().unwrap();
        let data = directory.path().join("data");
        let good = directory.path().join("media");
        let chosen =
            prepare_with_default(&data, LocationRequest::Selected(good), || unreachable!())
                .unwrap();
        assert!(
            prepare_with_default(&data, LocationRequest::Default, || Err(
                LibraryError::DocumentsUnavailable
            ))
            .is_err()
        );
        let file = directory.path().join("file");
        std::fs::write(&file, b"user fixture").unwrap();
        assert!(
            prepare_with_default(
                &data,
                LocationRequest::Selected(file.clone()),
                || unreachable!()
            )
            .is_err()
        );
        assert_eq!(std::fs::read(file).unwrap(), b"user fixture");
        let nested = data.join("must-not-create");
        assert!(
            prepare_with_default(
                &data,
                LocationRequest::Selected(nested.clone()),
                || unreachable!()
            )
            .is_err()
        );
        assert!(!nested.exists());
        assert_eq!(
            prepare_with_default(&data, LocationRequest::Restore, || unreachable!()).unwrap(),
            chosen
        );
    }
    #[cfg(unix)]
    #[test]
    fn denied_default_creation_keeps_saved_root() {
        // Mode permission denial is not meaningful when the test runs as root.
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let data = directory.path().join("data");
        let chosen = prepare_with_default(
            &data,
            LocationRequest::Selected(directory.path().join("media")),
            || unreachable!(),
        )
        .unwrap();
        let documents = directory.path().join("readonly-documents");
        std::fs::create_dir(&documents).unwrap();
        std::fs::set_permissions(&documents, std::fs::Permissions::from_mode(0o500)).unwrap();
        let result = prepare_with_default(&data, LocationRequest::Default, || {
            Ok(documents.join("dlpad"))
        });
        std::fs::set_permissions(&documents, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(result.is_err());
        assert_eq!(
            prepare_with_default(&data, LocationRequest::Restore, || unreachable!()).unwrap(),
            chosen
        );
    }
    #[cfg(unix)]
    #[test]
    fn symlink_selection_is_rejected_without_changing_its_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("media");
        std::fs::create_dir(&target).unwrap();
        let alias = directory.path().join("alias");
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        assert!(
            prepare_with_default(
                &directory.path().join("data"),
                LocationRequest::Selected(alias),
                || unreachable!()
            )
            .is_err()
        );
        assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
    }
}
#[cfg(test)]
mod install_hint_tests {
    use super::*;
    use std::{fs, io};
    fn hint(local: &Path) -> Option<PathBuf> {
        install_hint_with(local, &local.join("dlpad-data"), |path| {
            fs::symlink_metadata(path)
        })
    }
    #[test]
    fn programs_leaf_is_absent_and_no_directory_is_created() {
        let temp = tempfile::tempdir().unwrap();
        let programs = temp.path().join("Programs");
        fs::create_dir(&programs).unwrap();
        let target = hint(temp.path()).unwrap();
        assert_eq!(target, programs.join("dlpad"));
        assert!(!target.exists());
        assert_eq!(fs::read_dir(programs).unwrap().count(), 0);
    }
    #[test]
    fn missing_programs_uses_disjoint_fallback_without_any_creation() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(hint(temp.path()), Some(temp.path().join("dlpad-app")));
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }
    #[test]
    fn existing_default_is_shown_and_preserves_sentinel_without_duplicate_install() {
        let temp = tempfile::tempdir().unwrap();
        let existing = temp.path().join("dlpad-app");
        fs::create_dir(&existing).unwrap();
        fs::write(existing.join("user.txt"), b"keep").unwrap();
        assert_eq!(hint(temp.path()), Some(existing.clone()));
        assert_eq!(fs::read(existing.join("user.txt")).unwrap(), b"keep");
        assert!(!temp.path().join("dlpad-app-1").exists());
    }
    #[test]
    fn suffixed_folders_do_not_hide_the_existing_default() {
        let temp = tempfile::tempdir().unwrap();
        const LEGACY_SUFFIX_COUNT: usize = 16;
        for index in 0..LEGACY_SUFFIX_COUNT {
            let leaf = if index == 0 {
                "dlpad-app".to_owned()
            } else {
                format!("dlpad-app-{index}")
            };
            fs::create_dir(temp.path().join(leaf)).unwrap();
        }
        assert_eq!(hint(temp.path()), Some(temp.path().join("dlpad-app")));
        assert_eq!(
            fs::read_dir(temp.path()).unwrap().count(),
            LEGACY_SUFFIX_COUNT
        );
    }
    #[test]
    fn programs_or_candidate_file_is_preserved_and_never_accepted() {
        let temp = tempfile::tempdir().unwrap();
        let programs = temp.path().join("Programs");
        fs::write(&programs, b"user-file").unwrap();
        assert_eq!(hint(temp.path()), None);
        assert_eq!(fs::read(&programs).unwrap(), b"user-file");
        fs::remove_file(programs).unwrap();
        let leaf = temp.path().join("dlpad-app");
        fs::write(&leaf, b"payload").unwrap();
        assert_eq!(hint(temp.path()), None);
        assert_eq!(fs::read(leaf).unwrap(), b"payload");
    }
    #[test]
    fn permission_and_other_query_errors_are_not_treated_as_absent() {
        let temp = tempfile::tempdir().unwrap();
        for failed in [
            temp.path().to_path_buf(),
            temp.path().join("Programs"),
            temp.path().join("dlpad-app"),
        ] {
            for kind in [io::ErrorKind::PermissionDenied, io::ErrorKind::Other] {
                assert_eq!(
                    install_hint_with(temp.path(), &temp.path().join("dlpad-data"), |path| {
                        if path == failed {
                            Err(io::Error::new(kind, "injected metadata error"))
                        } else {
                            fs::symlink_metadata(path)
                        }
                    }),
                    None
                );
            }
        }
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }
    #[test]
    fn unicode_is_retained_but_overlap_relative_and_excessive_paths_fail() {
        let temp = tempfile::tempdir().unwrap();
        let local = temp.path().join("한글 사용자");
        fs::create_dir(&local).unwrap();
        assert_eq!(hint(&local), Some(local.join("dlpad-app")));
        for data in [
            local.clone(),
            local.join("dlpad-app"),
            local.join("dlpad-app").join("data"),
        ] {
            assert_eq!(
                install_hint_with(&local, &data, |path| fs::symlink_metadata(path)),
                None
            );
        }
        for invalid in [
            PathBuf::from("relative"),
            local.join("a".repeat(32768)),
            PathBuf::from(r"\\server\share"),
        ] {
            assert_eq!(
                install_hint_with(&invalid, &local.join("dlpad-data"), |_| panic!(
                    "invalid path must not query"
                )),
                None
            );
        }
    }
    #[cfg(unix)]
    #[test]
    fn symlink_base_programs_candidate_or_data_is_rejected_without_target_mutation() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let local = temp.path().join("local");
        let target = temp.path().join("target");
        fs::create_dir(&local).unwrap();
        fs::create_dir(&target).unwrap();
        let base = temp.path().join("base-link");
        symlink(&local, &base).unwrap();
        assert_eq!(hint(&base), None);
        for leaf in ["Programs", "dlpad-app", "dlpad-data"] {
            let link = local.join(leaf);
            symlink(&target, &link).unwrap();
            assert_eq!(hint(&local), None);
            fs::remove_file(link).unwrap();
        }
        assert_eq!(fs::read_dir(target).unwrap().count(), 0);
    }
    #[cfg(windows)]
    #[test]
    fn invalid_utf16_unc_and_device_paths_do_not_query_metadata() {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};
        for invalid in [
            PathBuf::from(OsString::from_wide(&[67, 58, 92, 0xd800])),
            PathBuf::from(r"\\server\share\local"),
            PathBuf::from(r"\\.\C:\local"),
        ] {
            assert_eq!(
                install_hint_with(&invalid, Path::new(r"C:\data"), |_| panic!(
                    "invalid path must not query"
                )),
                None
            );
        }
    }
}
