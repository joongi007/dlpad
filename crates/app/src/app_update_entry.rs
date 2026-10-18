//! CPU app bootstrap. Candidate preflight completes before any UI/profile migration.
#![cfg(windows)]
use dlpad_engine::{
    app_update::{bootstrap, profile},
    file_identity,
};
use std::{
    fs::OpenOptions,
    io,
    os::windows::fs::OpenOptionsExt,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};
static REQUIRE_RECOVERY: AtomicBool = AtomicBool::new(false);

pub fn accept_child(
    mut child: bootstrap::UiChild,
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cancel = AtomicBool::new(false);
    let deadline = Instant::now() + Duration::from_secs(120);
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(root)?;
    let identity = file_identity::from_file(&file)?;
    let preflight = child.requires_profile_preflight();
    if preflight {
        profile::inspect(root, profile::PROFILE_EPOCH, &cancel, deadline)?;
    }
    // The parent's authenticated original directory ID is checked by await_commit.
    if file_identity::from_file(&file)? != identity || file_identity::final_path(&file)? != root {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "profile root changed").into());
    }
    child.await_commit(&identity, profile::PROFILE_EPOCH, &cancel, deadline)?;
    REQUIRE_RECOVERY.store(preflight, Ordering::Release);
    crate::app_update_client::install(child)?;
    Ok(())
}

/// Called only after a successful actual frame presentation. No UI-thread IPC.
pub fn frame_presented(recovery_ready: bool) {
    if recovery_ready || !REQUIRE_RECOVERY.load(Ordering::Acquire) {
        crate::app_update_client::notify_runtime_ready();
    }
}
