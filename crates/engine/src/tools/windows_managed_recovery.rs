//! Windows startup recovery: a persisted claim alone never authorizes deletion.
//! Requires execution_marker::recovery_seed's single-authority v3 parser.
use super::windows_managed_journal::{JournalError, ManagedJournal};
use super::{ToolError, ToolManager, execution_marker, native_files};
use crate::{file_identity, windows_private_security};
use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};
use windows_sys::Win32::{
    Foundation::{ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION},
    Storage::FileSystem::*,
};
const NAME: &str = ".execution-pending";
const LIMIT: u64 = 4096;
fn changed() -> ToolError {
    ToolError::UnsafePath
}
fn busy(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
        || matches!(error.raw_os_error(), Some(n) if n == ERROR_LOCK_VIOLATION as i32 || n == ERROR_SHARING_VIOLATION as i32)
}
fn open(path: &Path, access: u32, share: u32) -> io::Result<File> {
    super::safe_components(path)
        .map_err(|_| io::Error::other("managed recovery namespace rejected"))?;
    OpenOptions::new()
        .access_mode(access)
        .share_mode(share)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}
fn regular(root: &File, file: &File) -> Result<(), ToolError> {
    let mut info = FILE_STANDARD_INFO::default();
    // SAFETY: retained disk capability and correctly sized initialized output.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileStandardInfo,
            (&mut info as *mut FILE_STANDARD_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    if info.Directory || info.DeletePending || info.NumberOfLinks != 1 {
        return Err(changed());
    }
    file_identity::ensure_regular_under(root, file)?;
    Ok(())
}
fn bytes(file: &File) -> Result<Vec<u8>, ToolError> {
    if file.metadata()?.len() > LIMIT {
        return Err(changed());
    }
    let mut reader = file;
    reader.seek(SeekFrom::Start(0))?;
    let mut value = Vec::new();
    reader.take(LIMIT + 1).read_to_end(&mut value)?;
    if value.is_empty() || value.len() as u64 > LIMIT {
        return Err(changed());
    }
    Ok(value)
}
fn marker_checked(
    manager: &ToolManager,
    file: &File,
    sid: &str,
    seed: &execution_marker::RecoverySeed,
    original: &[u8],
) -> Result<(), ToolError> {
    manager.verify_root()?;
    regular(&manager.root_pin, file)?;
    windows_private_security::verify_acl(file, sid)?;
    if file_identity::from_file(&manager.root_pin)? != seed.root_id
        || file_identity::from_file(file)? != seed.marker_id
        || !native_files::same_file(&manager.root.join(NAME), file)
        || bytes(file)? != original
    {
        return Err(changed());
    }
    Ok(())
}
/// false: absent or still pending. Errors preserve every unknown/changed file.
/// No startup pristine authority and no PID/boot/mtime inference.
pub(super) fn recover(manager: &ToolManager) -> Result<bool, ToolError> {
    manager.verify_root()?;
    let path = manager.root.join(NAME);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
        Ok(_) => {}
    }
    let sid = windows_private_security::current_sid()?;
    // Read-only observation shares the creator's original DELETE/write capabilities.
    let read_marker = match open(
        &path,
        FILE_GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
    ) {
        Ok(file) => file,
        Err(error) if busy(&error) => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    regular(&manager.root_pin, &read_marker)?;
    windows_private_security::verify_acl(&read_marker, &sid)?;
    let original = bytes(&read_marker)?;
    let seed = execution_marker::recovery_seed(&original)?;
    marker_checked(manager, &read_marker, &sid, &seed, &original)?;
    let lock_path = manager.root.join(".install-transfer-lock");
    // Never create a missing lock or adopt the current namespace's identity as original.
    let lock = open(
        &lock_path,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
    )?;
    regular(&manager.root_pin, &lock)?;
    let lock_id = file_identity::from_file(&lock)?;
    let expected = file_identity::NativeFileIdentity::Windows {
        volume_serial: seed.anchors.lock.volume_serial,
        file_id: seed.anchors.lock.file_id,
    };
    if lock_id != expected || !native_files::same_file(&lock_path, &lock) {
        return Err(changed());
    }
    match fs2::FileExt::try_lock_exclusive(&lock) {
        Ok(()) => {}
        Err(error) if busy(&error) => return Ok(false),
        Err(error) => return Err(error.into()),
    }
    // Creator FILE_SHARE_READ prohibits this DELETE reopen while it remains live.
    let marker = match open(&path, FILE_GENERIC_READ | DELETE, FILE_SHARE_READ) {
        Ok(file) => file,
        Err(error) if busy(&error) => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    marker_checked(manager, &marker, &sid, &seed, &original)?;
    let journal = match ManagedJournal::reopen(
        &manager.root_pin,
        &manager.root,
        &marker,
        &lock,
        seed.anchors,
        seed.session_nonce,
        seed.digest,
    ) {
        Ok(journal) => journal,
        Err(JournalError::Busy) => return Ok(false),
        Err(_) => return Err(changed()),
    };
    // Reopen-created_here=false deliberately rules out cleanup_pristine.
    if !journal.all_receipts_match().map_err(|_| changed())? {
        return Ok(false);
    }
    marker_checked(manager, &marker, &sid, &seed, &original)?;
    journal.cleanup_completed().map_err(|_| changed())?;
    // Original lock is still exclusive; no new managed session can begin here.
    marker_checked(manager, &marker, &sid, &seed, &original)?;
    if file_identity::from_file(&lock)? != expected || !native_files::same_file(&lock_path, &lock) {
        return Err(changed());
    }
    let mut disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: DELETE was reopened for the exact pinned marker, never an arbitrary current file.
    if unsafe {
        SetFileInformationByHandle(
            marker.as_raw_handle(),
            FileDispositionInfo,
            (&mut disposition as *mut FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of_val(&disposition) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    drop(marker);
    drop(read_marker);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        _ => return Err(changed()),
    }
    // Close-only original lock lifetime, after marker actual absence.
    drop(lock);
    Ok(true)
}

#[cfg(test)]
#[path = "windows_managed_recovery_tests.rs"]
mod tests;
