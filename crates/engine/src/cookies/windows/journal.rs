//! Retained-handle Windows reader journal. No deletion or boot authority.
use super::{Descriptor, META_ACCESS, SHARE_ALL, acl, current_sid, file_identity, kind, open};
use crate::cookies::windows_records::{
    BoundIdentity, CompletionReceipt, SessionAnchors, WindowsAnchor, WindowsReaderRecord,
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE};
const RECORD: &str = "reader-state.json";
const RECEIPTS: [&str; 2] = ["reader0.receipt", "reader1.receipt"];
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum JournalError {
    Changed,
    Busy,
    Bounds,
    State,
    Io,
}
impl std::fmt::Debug for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsJournalError")
    }
}
pub(super) struct ReaderWriter {
    pub file: File,
    pub anchor: WindowsAnchor,
    pub index: u8,
}
pub(super) struct Journal {
    owner: File,
    owner_path: PathBuf,
    sid: String,
    file: File,
    anchors: SessionAnchors,
    record: WindowsReaderRecord,
    receipts: [Option<ReceiptPin>; 2],
    poisoned: bool,
    initialized: bool,
    locked: bool,
}
struct ReceiptPin {
    file: File,
    anchor: WindowsAnchor,
}
impl std::fmt::Debug for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsJournalRedacted")
    }
}
impl std::fmt::Debug for ReaderWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsReaderWriterRedacted")
    }
}
fn identity(file: &File) -> Result<WindowsAnchor, JournalError> {
    match file_identity::from_file(file).map_err(|_| JournalError::Changed)? {
        file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } if file_id.iter().any(|b| *b != 0) => Ok(WindowsAnchor {
            volume_serial,
            file_id,
        }),
        _ => Err(JournalError::Changed),
    }
}
fn checked(
    file: &File,
    directory: bool,
    sid: &str,
    expected: WindowsAnchor,
) -> Result<(), JournalError> {
    kind(file, directory).map_err(|_| JournalError::Changed)?;
    acl(file, sid).map_err(|_| JournalError::Changed)?;
    if identity(file)? != expected {
        return Err(JournalError::Changed);
    }
    Ok(())
}
impl Journal {
    pub(super) fn create(
        owner: &File,
        owner_path: &Path,
        sid: &str,
        session_nonce: [u8; 16],
        record_digest: [u8; 32],
        cache: WindowsAnchor,
        copy: WindowsAnchor,
    ) -> Result<Self, JournalError> {
        if current_sid().map_err(|_| JournalError::Changed)? != sid {
            return Err(JournalError::Changed);
        }
        let owner_id = identity(owner)?;
        checked(owner, true, sid, owner_id)?;
        let original_path = open(owner_path, true, META_ACCESS, SHARE_ALL, None)
            .map_err(|_| JournalError::Changed)?;
        checked(&original_path, true, sid, owner_id)?;
        // Pin namespace against rename while all original files are created.
        let _namespace_pin = open(
            owner_path,
            true,
            META_ACCESS,
            windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ
                | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE,
            None,
        )
        .map_err(|_| JournalError::Changed)?;
        checked(&_namespace_pin, true, sid, owner_id)?;
        let descriptor = Descriptor::private(sid).map_err(|_| JournalError::Changed)?;
        let file = open(
            &owner_path.join(RECORD),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .map_err(|_| JournalError::Io)?;
        file.try_lock().map_err(|_| JournalError::Busy)?;
        let mut receipts = [None, None];
        let mut receipt_anchors = [identity(&file)?; 2];
        for (index, name) in RECEIPTS.iter().enumerate() {
            let path = owner_path.join(name);
            let writer = open(
                &path,
                false,
                META_ACCESS | FILE_GENERIC_WRITE,
                SHARE_ALL,
                Some(&descriptor),
            )
            .map_err(|_| JournalError::Io)?;
            let anchor = identity(&writer)?;
            checked(&writer, false, sid, anchor)?;
            writer.sync_all().map_err(|_| JournalError::Io)?;
            let reader = open(&path, false, FILE_GENERIC_READ, SHARE_ALL, None)
                .map_err(|_| JournalError::Changed)?;
            checked(&reader, false, sid, anchor)?;
            receipt_anchors[index] = anchor;
            receipts[index] = Some(ReceiptPin {
                file: reader,
                anchor,
            });
        }
        let anchors = SessionAnchors {
            cache,
            owner: owner_id,
            copy,
            journal: identity(&file)?,
            receipts: receipt_anchors,
        };
        let record = WindowsReaderRecord::new(session_nonce, record_digest, anchors)
            .map_err(|_| JournalError::State)?;
        let mut result = Self {
            owner: owner.try_clone().map_err(|_| JournalError::Io)?,
            owner_path: owner_path.to_owned(),
            sid: sid.to_owned(),
            file,
            anchors,
            record,
            receipts,
            poisoned: false,
            initialized: false,
            locked: true,
        };
        result.commit(result.record.clone())?;
        Ok(result)
    }
    /// Caller supplies ORIGINAL durable anchors, not a new observation of current files.
    pub(super) fn reopen(
        owner: &File,
        owner_path: &Path,
        sid: &str,
        expected: SessionAnchors,
        session_nonce: [u8; 16],
        record_digest: [u8; 32],
    ) -> Result<Self, JournalError> {
        if current_sid().map_err(|_| JournalError::Changed)? != sid {
            return Err(JournalError::Changed);
        }
        checked(owner, true, sid, expected.owner)?;
        let file = open(
            &owner_path.join(RECORD),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            None,
        )
        .map_err(|_| JournalError::Changed)?;
        checked(&file, false, sid, expected.journal)?;
        file.try_lock().map_err(|_| JournalError::Busy)?;
        let bytes = read_bound(&file, 4096)?;
        let record = WindowsReaderRecord::parse(&bytes).map_err(|_| JournalError::State)?;
        if record.anchors() != expected
            || record.session_nonce() != session_nonce
            || record.record_digest() != record_digest
        {
            return Err(JournalError::Changed);
        }
        let mut receipts = [None, None];
        for (index, anchor) in expected.receipts.into_iter().enumerate() {
            let file = open(
                &owner_path.join(RECEIPTS[index]),
                false,
                FILE_GENERIC_READ,
                SHARE_ALL,
                None,
            )
            .map_err(|_| JournalError::Changed)?;
            checked(&file, false, sid, anchor)?;
            receipts[index] = Some(ReceiptPin { file, anchor });
        }
        let result = Self {
            owner: owner.try_clone().map_err(|_| JournalError::Io)?,
            owner_path: owner_path.to_owned(),
            sid: sid.to_owned(),
            file,
            anchors: expected,
            record,
            receipts,
            poisoned: false,
            initialized: true,
            locked: true,
        };
        result.validate()?;
        Ok(result)
    }
    pub(super) fn anchors(&self) -> SessionAnchors {
        self.anchors
    }
    pub(super) fn record(&self) -> Result<&WindowsReaderRecord, JournalError> {
        self.validate()?;
        Ok(&self.record)
    }
    pub(super) fn receipt_anchors(&self) -> [Option<WindowsAnchor>; 2] {
        std::array::from_fn(|index| self.receipts[index].as_ref().map(|r| r.anchor))
    }

    /// Caller commits pending Bound before this explicit process-lock handoff.
    pub(super) fn release_for_guardian(
        &mut self,
        index: u8,
        expected: BoundIdentity,
    ) -> Result<(), JournalError> {
        self.validate()?;
        if self.record.pending_bound_identity(index) != Some(expected) {
            return Err(JournalError::State);
        }
        if self.file.sync_all().is_err() {
            self.poisoned = true;
            return Err(JournalError::Io);
        }
        let unlocked = self.file.unlock();
        self.locked = false;
        if unlocked.is_err() {
            self.poisoned = true;
            return Err(JournalError::Io);
        }
        Ok(())
    }
    /// Never treats a copied handle as an inherited lock authority.
    pub(super) fn reacquire(&mut self) -> Result<(), JournalError> {
        if self.poisoned {
            return Err(JournalError::State);
        }
        if self.locked {
            return self.validate();
        }
        match self.file.try_lock() {
            Ok(()) => self.locked = true,
            Err(std::fs::TryLockError::WouldBlock) => return Err(JournalError::Busy),
            Err(std::fs::TryLockError::Error(_)) => {
                self.poisoned = true;
                return Err(JournalError::Io);
            }
        }
        if let Err(error) = self.validate() {
            self.poisoned = true;
            self.locked = false;
            if self.file.unlock().is_err() {
                return Err(JournalError::Io);
            }
            return Err(error);
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), JournalError> {
        if self.poisoned || !self.locked {
            return Err(JournalError::State);
        }
        if current_sid().map_err(|_| JournalError::Changed)? != self.sid {
            return Err(JournalError::Changed);
        }
        checked(&self.owner, true, &self.sid, self.anchors.owner)?;
        let owner = open(&self.owner_path, true, META_ACCESS, SHARE_ALL, None)
            .map_err(|_| JournalError::Changed)?;
        checked(&owner, true, &self.sid, self.anchors.owner)?;
        checked(&self.file, false, &self.sid, self.anchors.journal)?;
        let current = open(
            &self.owner_path.join(RECORD),
            false,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .map_err(|_| JournalError::Changed)?;
        checked(&current, false, &self.sid, self.anchors.journal)?;
        for (index, pin) in self.receipts.iter().enumerate() {
            if let Some(pin) = pin {
                checked(&pin.file, false, &self.sid, pin.anchor)?;
                let current = open(
                    &self.owner_path.join(RECEIPTS[index]),
                    false,
                    META_ACCESS,
                    SHARE_ALL,
                    None,
                )
                .map_err(|_| JournalError::Changed)?;
                checked(&current, false, &self.sid, pin.anchor)?;
            }
        }
        if self.initialized {
            let bytes = read_bound(&self.file, 4096)?;
            let current = WindowsReaderRecord::parse(&bytes).map_err(|_| JournalError::State)?;
            if current.encode().map_err(|_| JournalError::State)?
                != self.record.encode().map_err(|_| JournalError::State)?
            {
                return Err(JournalError::Changed);
            }
        }
        Ok(())
    }
    fn commit(&mut self, next: WindowsReaderRecord) -> Result<(), JournalError> {
        self.validate()?;
        let bytes = next.encode().map_err(|_| JournalError::State)?;
        let result = (|| {
            self.file.seek(SeekFrom::Start(0))?;
            self.file.write_all(&bytes)?;
            self.file.set_len(bytes.len() as u64)?;
            self.file.sync_all()
        })();
        if result.is_err() {
            self.poisoned = true;
            return Err(JournalError::Io);
        }
        self.record = next;
        self.initialized = true;
        if let Err(error) = self.validate() {
            self.poisoned = true;
            return Err(error);
        }
        Ok(())
    }
    pub(super) fn begin(
        &mut self,
        index: u8,
        nonce: [u8; 16],
    ) -> Result<ReaderWriter, JournalError> {
        self.validate()?;
        let mut next = self.record.clone();
        next.begin(index, nonce).map_err(|_| JournalError::State)?;
        let pin = self.receipts[index as usize]
            .as_ref()
            .ok_or(JournalError::State)?;
        if pin.file.metadata().map_err(|_| JournalError::Io)?.len() != 0 {
            return Err(JournalError::State);
        }
        let path = self.owner_path.join(RECEIPTS[index as usize]);
        let writer = open(
            &path,
            false,
            META_ACCESS | FILE_GENERIC_WRITE,
            SHARE_ALL,
            None,
        )
        .map_err(|_| JournalError::Io)?;
        let anchor = pin.anchor;
        checked(&writer, false, &self.sid, anchor)?;
        if writer.metadata().map_err(|_| JournalError::Io)?.len() != 0 {
            return Err(JournalError::State);
        }
        self.commit(next)?;
        Ok(ReaderWriter {
            file: writer,
            anchor,
            index,
        })
    }
    pub(super) fn bind(
        &mut self,
        index: u8,
        guardian: u64,
        tool: u64,
    ) -> Result<BoundIdentity, JournalError> {
        self.validate()?;
        let mut next = self.record.clone();
        let identity = next
            .bind(index, guardian, tool)
            .map_err(|_| JournalError::State)?;
        self.commit(next)?;
        Ok(identity)
    }
    fn receipt(&self, index: usize) -> Result<CompletionReceipt, JournalError> {
        self.validate()?;
        let pin = self
            .receipts
            .get(index)
            .and_then(Option::as_ref)
            .ok_or(JournalError::State)?;
        CompletionReceipt::decode(&read_bound(&pin.file, 256)?).map_err(|_| JournalError::State)
    }
    pub(super) fn resolve(&mut self, index: u8) -> Result<(), JournalError> {
        let receipt = self.receipt(index as usize)?;
        let mut next = self.record.clone();
        next.resolve(&receipt).map_err(|_| JournalError::State)?;
        self.commit(next)
    }
    pub(super) fn all_receipts_match(&self) -> Result<bool, JournalError> {
        self.validate()?;
        let mut receipts = Vec::with_capacity(2);
        for index in 0..2 {
            if self.record.bound_identity(index as u8).is_some() {
                receipts.push(self.receipt(index)?);
            } else {
                let pin = self.receipts[index].as_ref().ok_or(JournalError::State)?;
                if !read_bound(&pin.file, 256)?.is_empty() {
                    return Err(JournalError::State);
                }
            }
        }
        Ok(self.record.all_receipts_match(&receipts))
    }
}
fn read_bound(file: &File, max: usize) -> Result<Vec<u8>, JournalError> {
    let mut file = file;
    let len = file.metadata().map_err(|_| JournalError::Io)?.len();
    if len > max as u64 {
        return Err(JournalError::Bounds);
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| JournalError::Io)?;
    let mut bytes = Vec::with_capacity(len as usize);
    file.take((max + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| JournalError::Io)?;
    if bytes.len() > max {
        return Err(JournalError::Bounds);
    }
    if bytes.len() != len as usize {
        return Err(JournalError::Changed);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        tempfile::TempDir,
        File,
        PathBuf,
        String,
        WindowsAnchor,
        WindowsAnchor,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("private-cache");
        super::super::prepare_cache(&cache).unwrap();
        let sid = current_sid().unwrap();
        let descriptor = Descriptor::private(&sid).unwrap();
        let owner_path = cache.join("synthetic-owner");
        assert!(super::super::create_directory(&owner_path, &descriptor).unwrap());
        let owner = open(&owner_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let cache_file = open(&cache, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let copy = open(
            &owner_path.join("cookies.txt"),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .unwrap();
        let cache_id = identity(&cache_file).unwrap();
        let copy_id = identity(&copy).unwrap();
        (temp, owner, owner_path, sid, cache_id, copy_id)
    }
    struct HandoffChild(std::process::Child);
    impl std::ops::Deref for HandoffChild {
        type Target = std::process::Child;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for HandoffChild {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl Drop for HandoffChild {
        fn drop(&mut self) {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            let _ = self.0.kill();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            while std::time::Instant::now() < deadline {
                if matches!(self.0.try_wait(), Ok(Some(_))) {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
    #[test]
    fn original_receipts_transitions_reopen_and_busy() {
        let (_temp, owner, path, sid, cache, copy) = fixture();
        let mut j = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let anchors = j.anchors();
        assert!(matches!(
            Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]),
            Err(JournalError::Busy)
        ));
        assert!(!j.all_receipts_match().unwrap());
        for index in 0..2 {
            let mut writer = j.begin(index, [index + 3; 16]).unwrap();
            assert_eq!(writer.anchor, anchors.receipts[index as usize]);
            assert_eq!(writer.index, index);
            j.bind(index, 10, 20).unwrap();
            writer
                .file
                .write_all(
                    &j.record()
                        .unwrap()
                        .receipt_for(index)
                        .unwrap()
                        .encode()
                        .unwrap(),
                )
                .unwrap();
            writer.file.sync_all().unwrap();
            j.resolve(index).unwrap();
        }
        assert!(j.all_receipts_match().unwrap());
        drop(j);
        let reopened = Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]).unwrap();
        assert!(reopened.all_receipts_match().unwrap());
    }
    #[test]
    fn partial_record_and_receipt_mismatch_preserve_bytes() {
        let (_temp, owner, path, sid, cache, copy) = fixture();
        let mut j = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let mut writer = j.begin(0, [3; 16]).unwrap();
        j.bind(0, 10, 20).unwrap();
        writer.file.write_all(&[9; 256]).unwrap();
        writer.file.sync_all().unwrap();
        let before = read_bound(&j.file, 4096).unwrap();
        assert!(j.resolve(0).is_err());
        assert_eq!(read_bound(&j.file, 4096).unwrap(), before);
        assert_eq!(std::fs::read(path.join(RECEIPTS[0])).unwrap(), vec![9; 256]);
        let anchors = j.anchors();
        drop(writer);
        drop(j);
        std::fs::write(path.join(RECORD), b"{partial").unwrap();
        assert!(Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]).is_err());
        assert_eq!(std::fs::read(path.join(RECORD)).unwrap(), b"{partial");
    }
    #[test]
    fn original_identity_swap_and_acl_tamper_rejected() {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Security::{
            Authorization::{SE_FILE_OBJECT, SetSecurityInfo},
            DACL_SECURITY_INFORMATION, UNPROTECTED_DACL_SECURITY_INFORMATION,
        };
        let (_temp, owner, path, sid, cache, copy) = fixture();
        let j = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let anchors = j.anchors();
        let bytes = j.record().unwrap().encode().unwrap();
        drop(j);
        std::fs::rename(path.join(RECORD), path.join("original-sentinel")).unwrap();
        let descriptor = Descriptor::private(&sid).unwrap();
        let mut replacement = open(
            &path.join(RECORD),
            false,
            FILE_GENERIC_READ
                | FILE_GENERIC_WRITE
                | windows_sys::Win32::Storage::FileSystem::WRITE_DAC,
            SHARE_ALL,
            Some(&descriptor),
        )
        .unwrap();
        replacement.write_all(&bytes).unwrap();
        replacement.sync_all().unwrap();
        assert!(Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]).is_err());
        // Deliberately remove protection only from this synthetic owned file.
        let code = unsafe {
            SetSecurityInfo(
                replacement.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(code, 0);
        assert!(acl(&replacement, &sid).is_err());
        assert_eq!(std::fs::read(path.join(RECORD)).unwrap(), bytes);
        assert_eq!(
            std::fs::read(path.join("original-sentinel")).unwrap(),
            bytes
        );
    }
    #[test]
    fn unused_receipt_payload_blocks_cleanup_and_preserves_sentinel() {
        let (temp, owner, path, sid, cache, copy) = fixture();
        let sentinel = temp.path().join("foreign-sentinel");
        std::fs::write(&sentinel, b"foreign synthetic sentinel").unwrap();
        let mut j = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let mut reader = j.begin(0, [3; 16]).unwrap();
        j.bind(0, 10, 20).unwrap();
        reader
            .file
            .write_all(
                &j.record()
                    .unwrap()
                    .receipt_for(0)
                    .unwrap()
                    .encode()
                    .unwrap(),
            )
            .unwrap();
        reader.file.sync_all().unwrap();
        j.resolve(0).unwrap();
        assert!(j.all_receipts_match().unwrap());
        let record_before = read_bound(&j.file, 4096).unwrap();
        let mut unused = open(
            &path.join(RECEIPTS[1]),
            false,
            META_ACCESS | FILE_GENERIC_WRITE,
            SHARE_ALL,
            None,
        )
        .unwrap();
        unused.write_all(b"foreign unused receipt payload").unwrap();
        unused.sync_all().unwrap();
        assert!(matches!(j.all_receipts_match(), Err(JournalError::State)));
        assert_eq!(read_bound(&j.file, 4096).unwrap(), record_before);
        assert_eq!(
            std::fs::read(path.join(RECEIPTS[1])).unwrap(),
            b"foreign unused receipt payload"
        );
        assert_eq!(
            std::fs::read(&sentinel).unwrap(),
            b"foreign synthetic sentinel"
        );
        let anchors = j.anchors();
        drop(unused);
        drop(reader);
        drop(j);
        let reopened = Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]).unwrap();
        assert!(reopened.all_receipts_match().is_err());
        assert_eq!(
            std::fs::read(&sentinel).unwrap(),
            b"foreign synthetic sentinel"
        );
    }
    #[test]
    fn explicit_handoff_blocks_mutation_scanner_preserves_and_reacquires() {
        const CHILD_ENV: &str = "DLPAD_TEST_WINDOWS_JOURNAL_LOCK_CHILD";
        if let Some(directory) = std::env::var_os(CHILD_ENV) {
            let directory = PathBuf::from(directory);
            let anchors: SessionAnchors = serde_json::from_slice(
                &std::fs::read(directory.join("expected-anchors.json")).unwrap(),
            )
            .unwrap();
            let owner_path = directory.join("private-cache").join("synthetic-owner");
            let sid = current_sid().unwrap();
            let owner = open(&owner_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
            let guardian =
                Journal::reopen(&owner, &owner_path, &sid, anchors, [1; 16], [2; 32]).unwrap();
            let bound = guardian
                .record()
                .unwrap()
                .pending_bound_identity(0)
                .unwrap();
            assert_eq!(
                (bound.nonce, bound.guardian_created, bound.tool_created),
                ([3; 16], 10, 20)
            );
            std::fs::write(directory.join("child-locked"), b"R").unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while !directory.join("child-release").exists() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "child fixture deadline"
                );
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert_eq!(
                std::fs::read(directory.join("child-release")).unwrap(),
                b"B"
            );
            let mut receipt = open(
                &owner_path.join(RECEIPTS[0]),
                false,
                META_ACCESS | FILE_GENERIC_WRITE,
                SHARE_ALL,
                None,
            )
            .unwrap();
            checked(&receipt, false, &sid, anchors.receipts[0]).unwrap();
            receipt
                .write_all(
                    &guardian
                        .record()
                        .unwrap()
                        .receipt_for(0)
                        .unwrap()
                        .encode()
                        .unwrap(),
                )
                .unwrap();
            receipt.sync_all().unwrap();
            return;
        }
        let (temp, owner, path, sid, cache, copy) = fixture();
        let sentinel = temp.path().join("handoff-foreign-sentinel");
        std::fs::write(&sentinel, b"foreign unchanged").unwrap();
        let mut app = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let _writer = app.begin(0, [3; 16]).unwrap();
        let bound = app.bind(0, 10, 20).unwrap();
        let anchors = app.anchors();
        let expected_receipt = app.record().unwrap().receipt_for(0).unwrap();
        let mut wrong = bound;
        wrong.tool_created += 1;
        assert!(app.release_for_guardian(0, wrong).is_err());
        assert!(app.record().is_ok());
        app.release_for_guardian(0, bound).unwrap();
        assert!(app.record().is_err());
        assert!(app.begin(1, [4; 16]).is_err());
        assert!(app.bind(0, 11, 21).is_err());
        assert!(app.resolve(0).is_err());
        assert!(app.all_receipts_match().is_err());
        {
            let scanner = Journal::reopen(&owner, &path, &sid, anchors, [1; 16], [2; 32]).unwrap();
            assert_eq!(
                scanner.record().unwrap().pending_bound_identity(0),
                Some(bound)
            );
            assert!(scanner.all_receipts_match().is_err());
            assert!(matches!(app.reacquire(), Err(JournalError::Busy)));
            assert_eq!(std::fs::read(&sentinel).unwrap(), b"foreign unchanged");
        }
        std::fs::write(
            temp.path().join("expected-anchors.json"),
            serde_json::to_vec(&anchors).unwrap(),
        )
        .unwrap();
        let test_name = std::thread::current().name().unwrap().to_owned();
        let mut child = HandoffChild(
            std::process::Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg(&test_name)
                .arg("--test-threads=1")
                .arg("--nocapture")
                .env(CHILD_ENV, temp.path())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut failure = None;
        while !temp.path().join("child-locked").exists() {
            if child.try_wait().unwrap().is_some() {
                failure = Some("child exited before lock ACK");
                break;
            }
            if std::time::Instant::now() >= deadline {
                failure = Some("child lock ACK timeout");
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if failure.is_none() {
            assert_eq!(
                std::fs::read(temp.path().join("child-locked")).unwrap(),
                b"R"
            );
            assert!(child.try_wait().unwrap().is_none());
            assert!(matches!(app.reacquire(), Err(JournalError::Busy)));
            std::fs::write(temp.path().join("child-release"), b"B").unwrap();
        } else {
            let _ = child.kill();
        }
        let reap_deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if std::time::Instant::now() >= reap_deadline {
                let _ = child.kill();
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        if status.is_none() {
            let final_deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            while std::time::Instant::now() < final_deadline {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        assert!(failure.is_none(), "owned child lock fixture failed");
        assert!(
            status.is_some_and(|status| status.success()),
            "owned child terminal failure"
        );
        assert_eq!(expected_receipt, app.record.receipt_for(0).unwrap());
        app.reacquire().unwrap();
        assert_eq!(app.record().unwrap().pending_bound_identity(0), Some(bound));
        app.resolve(0).unwrap();
        assert!(app.all_receipts_match().unwrap());
        assert!(app.release_for_guardian(0, bound).is_err());
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"foreign unchanged");
    }
    #[test]
    fn reacquire_tampered_record_unlocks_and_poison_preserves() {
        let (_temp, owner, path, sid, cache, copy) = fixture();
        let mut app = Journal::create(&owner, &path, &sid, [1; 16], [2; 32], cache, copy).unwrap();
        let _writer = app.begin(0, [3; 16]).unwrap();
        let bound = app.bind(0, 10, 20).unwrap();
        app.release_for_guardian(0, bound).unwrap();
        let tampered = b"{partial tampered synthetic state";
        std::fs::write(path.join(RECORD), tampered).unwrap();
        assert!(app.reacquire().is_err());
        assert!(!app.locked);
        assert!(app.poisoned);
        assert!(app.reacquire().is_err());
        assert!(app.record().is_err());
        let probe = open(
            &path.join(RECORD),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            None,
        )
        .unwrap();
        probe.try_lock().unwrap();
        assert_eq!(read_bound(&probe, 4096).unwrap(), tampered);
    }
}
