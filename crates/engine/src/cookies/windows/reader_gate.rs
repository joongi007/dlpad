//! Durable Windows reader intent and explicit receipt acknowledgement.
use super::{CookieImportError, OwnedCopy, changed, journal};
use crate::cookies::windows_records::{BoundIdentity, WindowsAnchor, WindowsReaderRecord};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
use windows_sys::Win32::Security::Cryptography::{
    BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
};

pub(crate) struct WindowsCookieLaunch {
    pub(crate) intent: WindowsReaderRecord,
    pub(crate) owner_path: std::path::PathBuf,
    pub(crate) slot: u8,
    pub(crate) nonce: [u8; 16],
    pub(crate) receipt: std::fs::File,
    pub(crate) receipt_anchor: WindowsAnchor,
}
impl std::fmt::Debug for WindowsCookieLaunch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsCookieLaunchRedacted")
    }
}
pub(crate) struct WindowsGatedCookieReader<'a> {
    journal: &'a Mutex<journal::Journal>,
    pending: &'a AtomicBool,
    launch: Option<WindowsCookieLaunch>,
    original_intent: WindowsReaderRecord,
    slot: u8,
    nonce: [u8; 16],
    bound: Option<BoundIdentity>,
    released: bool,
    confirmed: bool,
}
impl std::fmt::Debug for WindowsGatedCookieReader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsGatedCookieReaderRedacted")
    }
}
impl OwnedCopy {
    pub(in crate::cookies) fn begin_windows_reader<'a>(
        &'a self,
        pending: &'a AtomicBool,
        slot: u8,
    ) -> Result<WindowsGatedCookieReader<'a>, CookieImportError> {
        self.validate_for_tool()?;
        if slot > 1 {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let journal = self.journal.as_ref().ok_or_else(changed)?;
        let mut locked = journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        // Validate/CSPRNG first; CAS immediately precedes durable begin.
        let mut nonce = [0u8; 16];
        if unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                nonce.as_mut_ptr(),
                nonce.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } != 0
            || nonce == [0; 16]
        {
            return Err(CookieImportError::StorageUnavailable);
        }
        pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        // Any failure from this point remains pending, even before intent is durable.
        let writer = locked
            .begin(slot, nonce)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        let intent = locked
            .record()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .clone();
        if writer.index != slot || intent.anchors().receipts[slot as usize] != writer.anchor {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let launch = WindowsCookieLaunch {
            intent: intent.clone(),
            owner_path: self.owner_path.0.clone(),
            slot,
            nonce,
            receipt: writer.file,
            receipt_anchor: writer.anchor,
        };
        drop(locked);
        Ok(WindowsGatedCookieReader {
            journal,
            pending,
            launch: Some(launch),
            original_intent: intent,
            slot,
            nonce,
            bound: None,
            released: false,
            confirmed: false,
        })
    }
}
impl WindowsGatedCookieReader<'_> {
    pub(crate) fn take_launch(&mut self) -> Result<WindowsCookieLaunch, CookieImportError> {
        self.launch
            .take()
            .ok_or(CookieImportError::ReaderUnconfirmed)
    }
    /// Called from verified helper Spawned protocol frame, before Go/handoff.
    pub(crate) fn bind_identity(
        &mut self,
        guardian_created: u64,
        tool_created: u64,
    ) -> Result<BoundIdentity, CookieImportError> {
        if self.confirmed || self.released || self.launch.is_some() || self.bound.is_some() {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let mut expected = self.original_intent.clone();
        let identity = expected
            .bind(self.slot, guardian_created, tool_created)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        if identity.nonce != self.nonce {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let mut locked = self
            .journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        // Prevent binding an unexpected record (no current-record adoption).
        if locked
            .record()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .encode()
            .map_err(|_| changed())?
            != self.original_intent.encode().map_err(|_| changed())?
        {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let committed = locked
            .bind(self.slot, guardian_created, tool_created)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        if committed != identity {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        self.bound = Some(identity);
        Ok(identity)
    }
    /// Called only after Bound commit and parent receipt writer has been dropped.
    pub(crate) fn release_for_guardian(&mut self) -> Result<(), CookieImportError> {
        if self.confirmed || self.released || self.launch.is_some() {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let bound = self.bound.ok_or(CookieImportError::ReaderUnconfirmed)?;
        self.journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .release_for_guardian(self.slot, bound)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        self.released = true;
        Ok(())
    }
    /// Caller must first await original helper process termination/owned Job ack.
    /// A protocol message alone is insufficient. Actual receipt is independently read.
    pub(crate) fn confirm_terminal(&mut self) -> Result<(), CookieImportError> {
        if self.confirmed || !self.released || self.launch.is_some() {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let bound = self.bound.ok_or(CookieImportError::ReaderUnconfirmed)?;
        let mut locked = self
            .journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        locked
            .reacquire()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        if locked
            .record()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .pending_bound_identity(self.slot)
            != Some(bound)
        {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        locked
            .resolve(self.slot)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        if !locked
            .all_receipts_match()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
        {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        self.confirmed = true;
        self.pending.store(false, Ordering::Release);
        Ok(())
    }
}
impl Drop for WindowsGatedCookieReader<'_> {
    fn drop(&mut self) {
        // No reacquire, receipt fabrication, resolve, or pending clear on Drop.
    }
}
