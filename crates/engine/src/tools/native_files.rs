//! Retained native file identity for newly owned installer paths.
use super::{ToolError, safe_components};
use crate::file_identity;
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
pub(crate) fn open_directory(path: &Path) -> Result<File, ToolError> {
    safe_components(path)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows_sys::Win32::Storage::FileSystem::*;
        options
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
        let file = options.open(path)?;
        let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: live borrowed disk handle and correctly sized/aligned output.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        if info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(ToolError::UnsafePath);
        }
        safe_components(path)?;
        Ok(file)
    }
    #[cfg(not(windows))]
    {
        let file = options.open(path)?;
        if !file.metadata()?.is_dir() {
            return Err(ToolError::UnsafePath);
        }
        safe_components(path)?;
        Ok(file)
    }
}
pub(crate) fn open_regular(path: &Path) -> Result<File, ToolError> {
    safe_components(path)?;
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        // The existing identity opener has FILE_READ_ATTRIBUTES only. Keep it
        // as the original no-reparse pin, then obtain actual read-data access.
        let original = file_identity::open_regular_no_reparse(path)?;
        let readable = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if file_identity::from_file(&original)? != file_identity::from_file(&readable)? {
            return Err(ToolError::UnsafePath);
        }
        readable
    };
    #[cfg(not(windows))]
    let file = {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC);
        }
        options.open(path)?
    };
    if !file.metadata()?.is_file() {
        return Err(ToolError::UnsafePath);
    }
    safe_components(path)?;
    Ok(file)
}
pub(crate) fn same_file(path: &Path, original: &File) -> bool {
    open_regular(path)
        .and_then(|current| {
            let before = file_identity::from_file(original)?;
            let now = file_identity::from_file(&current)?;
            Ok(before == now)
        })
        .unwrap_or(false)
}
pub(crate) fn same_directory(path: &Path, original: &File) -> bool {
    open_directory(path)
        .and_then(|current| {
            Ok(file_identity::from_file(original)? == file_identity::from_file(&current)?)
        })
        .unwrap_or(false)
}
