//! Native handle evidence. Identity alone does not prove path containment.
use super::NativeFileIdentity;
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::OpenOptionsExt,
        io::AsRawHandle,
    },
    path::{Component, Path, PathBuf, Prefix},
};
use windows_sys::Win32::Storage::FileSystem::*;
const MAX_PATH_UNITS: usize = 32_768;

fn disk(file: &File) -> io::Result<()> {
    // SAFETY: borrowed handle remains owned by File throughout the call.
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file identity requires a disk handle",
        ));
    }
    Ok(())
}
pub(super) fn from_file(file: &File) -> io::Result<NativeFileIdentity> {
    disk(file)?;
    let mut info = FILE_ID_INFO::default();
    // SAFETY: correct info class/size, aligned writable output and live handle.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileIdInfo,
            (&mut info as *mut FILE_ID_INFO).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    validated_identity(info)
}
fn validated_identity(info: FILE_ID_INFO) -> io::Result<NativeFileIdentity> {
    if info.FileId.Identifier == [0; 16] {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filesystem returned no usable 128-bit file ID",
        ));
    }
    // Keep all 128 bits opaque. No truncation to a u64 or content/hash inference.
    Ok(NativeFileIdentity::Windows {
        volume_serial: info.VolumeSerialNumber,
        file_id: info.FileId.Identifier,
    })
}
fn attributes(file: &File) -> io::Result<FILE_ATTRIBUTE_TAG_INFO> {
    disk(file)?;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: correct info class/size and writable output through live file handle.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(info)
}
fn regular(file: &File) -> io::Result<()> {
    let info = attributes(file)?;
    if info.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "regular non-reparse file required",
        ));
    }
    Ok(())
}
pub(super) fn open_regular_no_reparse(path: &Path) -> io::Result<File> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute file path without parent traversal required",
        ));
    }
    if !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::UNC(_, _) | Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(_, _)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported Windows path namespace",
        ));
    }
    for (index, unit) in path.as_os_str().encode_wide().enumerate() {
        if unit == 0 || index >= MAX_PATH_UNITS - 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid or oversized Windows path",
            ));
        }
    }
    // OPEN_REPARSE_POINT applies to the final component. Ancestors may still be
    // reparse points; caller must use handle containment and its path/race policy.
    // Rust's Windows OpenOptions performs extended-length path normalization.
    // Read-attributes access opens an existing file, never creates/truncates it.
    let file = OpenOptions::new()
        .read(true)
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    regular(&file)?;
    Ok(file)
}
pub(super) fn final_path(file: &File) -> io::Result<PathBuf> {
    disk(file)?;
    let mut units = vec![0; 512];
    loop {
        // SAFETY: live handle and writable buffer whose length matches the API.
        let length = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                units.as_mut_ptr(),
                units.len() as u32,
                FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
            )
        } as usize;
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        if length >= MAX_PATH_UNITS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "final Windows path exceeds bound",
            ));
        }
        if length >= units.len() {
            units.resize(length + 1, 0);
            continue;
        }
        units.truncate(length);
        if units.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NUL in final Windows path",
            ));
        }
        let path = PathBuf::from(OsString::from_wide(&units));
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "final handle path is not absolute",
            ));
        }
        return Ok(path);
    }
}
pub(super) fn ensure_regular_under(root: &File, file: &File) -> io::Result<()> {
    let root_info = attributes(root)?;
    if root_info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || root_info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "non-reparse directory root required",
        ));
    }
    regular(file)?;
    let NativeFileIdentity::Windows {
        volume_serial: root_volume,
        ..
    } = from_file(root)?
    else {
        return Err(io::Error::other("Windows root identity unavailable"));
    };
    let NativeFileIdentity::Windows {
        volume_serial: file_volume,
        ..
    } = from_file(file)?
    else {
        return Err(io::Error::other("Windows file identity unavailable"));
    };
    if root_volume != file_volume {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file is on a different volume",
        ));
    }
    let root_path = final_path(root)?;
    let file_path = final_path(file)?;
    let relative = file_path.strip_prefix(root_path).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "file handle resolves outside root",
        )
    })?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "strict descendant file required",
        ));
    }
    // Observed handle paths are contained now. This does not lock ancestor names
    // against later changes or prove a separately reopened pathname is this file.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self, OpenOptions},
        os::windows::fs::{OpenOptionsExt, symlink_dir, symlink_file},
        os::windows::io::FromRawHandle,
        ptr,
    };
    fn directory(path: &Path) -> File {
        OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .unwrap()
    }
    #[test]
    fn unsupported_zero_file_id_is_never_identity_evidence() {
        let error = validated_identity(FILE_ID_INFO::default()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        let mut info = FILE_ID_INFO::default();
        info.FileId.Identifier[15] = 1;
        assert_eq!(
            validated_identity(info).unwrap(),
            NativeFileIdentity::Windows {
                volume_serial: 0,
                file_id: info.FileId.Identifier
            }
        );
    }
    #[test]
    fn hardlink_rename_and_pinned_handle_keep_full_identity_while_replacement_changes_it() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("원본 파일.bin");
        let renamed = temp.path().join("이름 변경.bin");
        let linked = temp.path().join("linked.bin");
        fs::write(&source, b"synthetic").unwrap();
        let pinned = File::open(&source).unwrap();
        let original = from_file(&pinned).unwrap();
        fs::hard_link(&source, &linked).unwrap();
        fs::rename(&source, &renamed).unwrap();
        assert_eq!(from_file(&File::open(linked).unwrap()).unwrap(), original);
        assert_eq!(from_file(&File::open(renamed).unwrap()).unwrap(), original);
        fs::write(&source, b"synthetic").unwrap();
        assert_ne!(from_file(&File::open(&source).unwrap()).unwrap(), original);
        assert_eq!(from_file(&pinned).unwrap(), original);
        let encoded = serde_json::to_string(&original).unwrap();
        assert_eq!(
            serde_json::from_str::<NativeFileIdentity>(&encoded).unwrap(),
            original
        );
        let maximal = NativeFileIdentity::Windows {
            volume_serial: u64::MAX,
            file_id: [u8::MAX; 16],
        };
        assert_eq!(
            serde_json::from_str::<NativeFileIdentity>(&serde_json::to_string(&maximal).unwrap())
                .unwrap(),
            maximal
        );
    }
    #[test]
    fn handle_containment_regular_type_and_final_path_buffer_growth() {
        let temp = tempfile::tempdir().unwrap();
        let root_path = temp.path().join("루트");
        let outside_path = temp.path().join("루트 다른");
        fs::create_dir(&root_path).unwrap();
        fs::create_dir(&outside_path).unwrap();
        let root = directory(&root_path);
        let mut parent = root_path.clone();
        for index in 0..8 {
            parent = parent.join(format!("{index}-{}", "segment".repeat(10)));
            fs::create_dir(&parent).unwrap();
        }
        let inside_path = parent.join("video.bin");
        let outside_path = outside_path.join("video.bin");
        fs::write(&inside_path, b"fixture").unwrap();
        fs::write(&outside_path, b"fixture").unwrap();
        let inside = open_regular_no_reparse(&inside_path).unwrap();
        let outside = open_regular_no_reparse(&outside_path).unwrap();
        ensure_regular_under(&root, &inside).unwrap();
        assert!(
            final_path(&inside)
                .unwrap()
                .as_os_str()
                .encode_wide()
                .count()
                > 512
        );
        assert!(ensure_regular_under(&root, &outside).is_err());
        assert!(ensure_regular_under(&inside, &inside).is_err());
        assert!(ensure_regular_under(&root, &root).is_err());
        assert!(open_regular_no_reparse(&root_path).is_err());
        assert!(open_regular_no_reparse(Path::new("relative.bin")).is_err());
        assert!(open_regular_no_reparse(&root_path.join("..\\outside.bin")).is_err());
        assert!(open_regular_no_reparse(Path::new("C:\\bad\0path")).is_err());
    }
    #[test]
    fn pipe_is_not_disk_identity_or_regular_file() {
        use windows_sys::Win32::System::Pipes::CreatePipe;
        let mut read = ptr::null_mut();
        let mut write = ptr::null_mut();
        unsafe {
            assert_ne!(CreatePipe(&mut read, &mut write, ptr::null(), 0), 0);
        }
        let reader = unsafe { File::from_raw_handle(read) };
        let _writer = unsafe { File::from_raw_handle(write) };
        assert!(from_file(&reader).is_err());
        assert!(final_path(&reader).is_err());
    }
    #[test]
    fn symlink_leaf_and_external_directory_reparse_are_rejected_when_permitted() {
        let temp = tempfile::tempdir().unwrap();
        let root_path = temp.path().join("root");
        let external = temp.path().join("external");
        fs::create_dir(&root_path).unwrap();
        fs::create_dir(&external).unwrap();
        let external_file = external.join("video.bin");
        fs::write(&external_file, b"fixture").unwrap();
        let root = directory(&root_path);
        let link = root_path.join("linked.bin");
        if let Err(error) = symlink_file(&external_file, &link) {
            if error.raw_os_error() == Some(1314) || error.kind() == io::ErrorKind::PermissionDenied
            {
                println!("UNVERIFIED: Windows symlink permission unavailable: {error}");
                return;
            }
            panic!("unexpected synthetic symlink failure: {error}");
        }
        assert!(open_regular_no_reparse(&link).is_err());
        assert!(ensure_regular_under(&root, &File::open(&link).unwrap()).is_err());
        let directory_link = root_path.join("linked-dir");
        symlink_dir(&external, &directory_link).unwrap();
        let followed_ancestor = open_regular_no_reparse(&directory_link.join("video.bin")).unwrap();
        assert!(ensure_regular_under(&root, &followed_ancestor).is_err());
        assert!(
            ensure_regular_under(
                &directory(&directory_link),
                &File::open(external_file).unwrap()
            )
            .is_err()
        );
        println!("VERIFIED: Windows synthetic file symlink and directory symlink containment");
    }
}
