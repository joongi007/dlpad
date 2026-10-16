//! Retained-handle Windows reader journal. No deletion or boot authority.
use super::native_files;
use super::windows_managed_records::{
    BoundIdentity, CompletionReceipt, ManagedReaderRecord, ReaderStage, SessionAnchors,
    WindowsAnchor,
};
use crate::{
    file_identity,
    windows_private_security::{PrivateDescriptor, current_sid, verify_acl},
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};
const META_ACCESS: u32 = FILE_READ_ATTRIBUTES | READ_CONTROL;
const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
const RECORD: &str = ".execution-readers.json";
const RECEIPTS: [&str; 8] = [
    ".execution-reader-0.receipt",
    ".execution-reader-1.receipt",
    ".execution-reader-2.receipt",
    ".execution-reader-3.receipt",
    ".execution-reader-4.receipt",
    ".execution-reader-5.receipt",
    ".execution-reader-6.receipt",
    ".execution-reader-7.receipt",
];
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum JournalError {
    Changed,
    Busy,
    Bounds,
    State,
    Io,
}
impl std::fmt::Debug for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedJournalError")
    }
}
pub(crate) struct ReaderWriter {
    pub file: File,
    pub anchor: WindowsAnchor,
    pub index: u8,
}
pub(crate) struct ManagedJournal {
    owner: File,
    owner_path: PathBuf,
    marker: File,
    execution_lock: File,
    sid: String,
    file: File,
    anchors: SessionAnchors,
    record: ManagedReaderRecord,
    receipts: [Option<ReceiptPin>; 8],
    poisoned: bool,
    initialized: bool,
    created_here: bool,
    locked: bool,
}
struct ReceiptPin {
    file: File,
    delete: File,
    anchor: WindowsAnchor,
}
impl std::fmt::Debug for ManagedJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedJournalRedacted")
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
    verify_acl(file, sid).map_err(|_| JournalError::Changed)?;
    if identity(file)? != expected {
        return Err(JournalError::Changed);
    }
    Ok(())
}
impl ManagedJournal {
    pub(crate) fn create(
        owner: &File,
        owner_path: &Path,
        marker: &File,
        execution_lock: &File,
        session_nonce: [u8; 16],
        record_digest: [u8; 32],
    ) -> Result<Self, JournalError> {
        let sid = current_sid().map_err(|_| JournalError::Changed)?;
        let sid = sid.as_str();
        kind(marker, false)?;
        verify_acl(marker, sid).map_err(|_| JournalError::Changed)?;
        kind(execution_lock, false)?;
        file_identity::ensure_regular_under(owner, marker).map_err(|_| JournalError::Changed)?;
        file_identity::ensure_regular_under(owner, execution_lock)
            .map_err(|_| JournalError::Changed)?;
        let owner_id = identity(owner)?;
        kind(owner, true)?;
        let original_path = open(owner_path, true, META_ACCESS, SHARE_ALL, None)
            .map_err(|_| JournalError::Changed)?;
        if identity(&original_path)? != owner_id {
            return Err(JournalError::Changed);
        }
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
        if identity(&_namespace_pin)? != owner_id {
            return Err(JournalError::Changed);
        }
        let descriptor = PrivateDescriptor::new(sid).map_err(|_| JournalError::Changed)?;
        let file = open(
            &owner_path.join(RECORD),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .map_err(|_| JournalError::Io)?;
        file.try_lock().map_err(|_| JournalError::Busy)?;
        let mut receipts = std::array::from_fn(|_| None);
        let mut receipt_anchors = [identity(&file)?; 8];
        for (index, name) in RECEIPTS.iter().enumerate() {
            let path = owner_path.join(name);
            let writer = open(
                &path,
                false,
                META_ACCESS | FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE,
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
                delete: writer,
                anchor,
            });
        }
        let anchors = SessionAnchors {
            root: owner_id,
            marker: identity(marker)?,
            lock: identity(execution_lock)?,
            journal: identity(&file)?,
            receipts: receipt_anchors,
        };
        let record = ManagedReaderRecord::new(session_nonce, record_digest, anchors)
            .map_err(|_| JournalError::State)?;
        let mut result = Self {
            owner: owner.try_clone().map_err(|_| JournalError::Io)?,
            owner_path: owner_path.to_owned(),
            marker: marker.try_clone().map_err(|_| JournalError::Io)?,
            execution_lock: execution_lock.try_clone().map_err(|_| JournalError::Io)?,
            sid: sid.to_owned(),
            file,
            anchors,
            record,
            receipts,
            poisoned: false,
            initialized: false,
            created_here: true,
            locked: true,
        };
        result.commit(result.record.clone())?;
        Ok(result)
    }
    /// Caller supplies ORIGINAL durable anchors, not a new observation of current files.
    pub(crate) fn reopen(
        owner: &File,
        owner_path: &Path,
        marker: &File,
        execution_lock: &File,
        expected: SessionAnchors,
        session_nonce: [u8; 16],
        record_digest: [u8; 32],
    ) -> Result<Self, JournalError> {
        let sid = current_sid().map_err(|_| JournalError::Changed)?;
        let sid = sid.as_str();
        kind(owner, true)?;
        if identity(owner)? != expected.root || identity(execution_lock)? != expected.lock {
            return Err(JournalError::Changed);
        }
        checked(marker, false, sid, expected.marker)?;
        kind(execution_lock, false)?;
        let file = open(
            &owner_path.join(RECORD),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE,
            SHARE_ALL,
            None,
        )
        .map_err(|_| JournalError::Changed)?;
        checked(&file, false, sid, expected.journal)?;
        file.try_lock().map_err(|_| JournalError::Busy)?;
        let bytes = read_bound(&file, 16384)?;
        let record = ManagedReaderRecord::parse(&bytes).map_err(|_| JournalError::State)?;
        if record.anchors() != expected
            || record.session_nonce() != session_nonce
            || record.record_digest() != record_digest
        {
            return Err(JournalError::Changed);
        }
        let mut receipts = std::array::from_fn(|_| None);
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
            let delete = open(
                &owner_path.join(RECEIPTS[index]),
                false,
                FILE_GENERIC_READ | DELETE,
                SHARE_ALL,
                None,
            )
            .map_err(|_| JournalError::Changed)?;
            checked(&delete, false, sid, anchor)?;
            receipts[index] = Some(ReceiptPin {
                file,
                delete,
                anchor,
            });
        }
        let result = Self {
            owner: owner.try_clone().map_err(|_| JournalError::Io)?,
            owner_path: owner_path.to_owned(),
            marker: marker.try_clone().map_err(|_| JournalError::Io)?,
            execution_lock: execution_lock.try_clone().map_err(|_| JournalError::Io)?,
            sid: sid.to_owned(),
            file,
            anchors: expected,
            record,
            receipts,
            poisoned: false,
            initialized: true,
            created_here: false,
            locked: true,
        };
        result.validate()?;
        Ok(result)
    }
    pub(crate) fn anchors(&self) -> SessionAnchors {
        self.anchors
    }
    pub(crate) fn record(&self) -> Result<&ManagedReaderRecord, JournalError> {
        self.validate()?;
        Ok(&self.record)
    }
    pub(crate) fn receipt_anchors(&self) -> [Option<WindowsAnchor>; 8] {
        std::array::from_fn(|index| self.receipts[index].as_ref().map(|r| r.anchor))
    }

    /// Caller commits pending Bound before this explicit process-lock handoff.
    pub(crate) fn release_for_guardian(
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
    pub(crate) fn reacquire(&mut self) -> Result<(), JournalError> {
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
        kind(&self.owner, true)?;
        if identity(&self.owner)? != self.anchors.root {
            return Err(JournalError::Changed);
        }
        checked(&self.marker, false, &self.sid, self.anchors.marker)?;
        kind(&self.execution_lock, false)?;
        if identity(&self.execution_lock)? != self.anchors.lock {
            return Err(JournalError::Changed);
        }
        if !native_files::same_file(&self.owner_path.join(".execution-pending"), &self.marker)
            || !native_files::same_file(
                &self.owner_path.join(".install-transfer-lock"),
                &self.execution_lock,
            )
        {
            return Err(JournalError::Changed);
        }
        let owner = open(&self.owner_path, true, META_ACCESS, SHARE_ALL, None)
            .map_err(|_| JournalError::Changed)?;
        if identity(&owner)? != self.anchors.root {
            return Err(JournalError::Changed);
        }
        checked(&self.file, false, &self.sid, self.anchors.journal)?;
        file_identity::ensure_regular_under(&self.owner, &self.file)
            .map_err(|_| JournalError::Changed)?;
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
                file_identity::ensure_regular_under(&self.owner, &pin.file)
                    .map_err(|_| JournalError::Changed)?;
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
            let bytes = read_bound(&self.file, 16384)?;
            let current = ManagedReaderRecord::parse(&bytes).map_err(|_| JournalError::State)?;
            if current.encode().map_err(|_| JournalError::State)?
                != self.record.encode().map_err(|_| JournalError::State)?
            {
                return Err(JournalError::Changed);
            }
        }
        Ok(())
    }
    fn commit(&mut self, next: ManagedReaderRecord) -> Result<(), JournalError> {
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
    pub(crate) fn begin(
        &mut self,
        index: u8,
        stage: ReaderStage,
        nonce: [u8; 16],
    ) -> Result<ReaderWriter, JournalError> {
        self.validate()?;
        let mut next = self.record.clone();
        next.begin(index, stage, nonce)
            .map_err(|_| JournalError::State)?;
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
    pub(crate) fn bind(
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
    pub(crate) fn resolve(&mut self, index: u8) -> Result<(), JournalError> {
        let receipt = self.receipt(index as usize)?;
        let mut next = self.record.clone();
        next.resolve(&receipt).map_err(|_| JournalError::State)?;
        self.commit(next)
    }
    /// Consumes every original handle before success. Root clears marker last.
    pub(crate) fn cleanup_completed(self) -> Result<(), JournalError> {
        if !self.all_receipts_match()? {
            return Err(JournalError::State);
        }
        self.cleanup_original_files()
    }
    /// Only the original live creator may prove no reader was ever started.
    pub(crate) fn cleanup_pristine(self) -> Result<(), JournalError> {
        self.validate()?;
        if !self.created_here || !self.record.is_pristine() {
            return Err(JournalError::State);
        }
        for pin in &self.receipts {
            if !read_bound(&pin.as_ref().ok_or(JournalError::State)?.file, 256)?.is_empty() {
                return Err(JournalError::State);
            }
        }
        self.cleanup_original_files()
    }
    fn cleanup_original_files(self) -> Result<(), JournalError> {
        self.validate()?;
        // All original DELETE capabilities are checked before the first mutation.
        checked(&self.file, false, &self.sid, self.anchors.journal)?;
        for pin in &self.receipts {
            let pin = pin.as_ref().ok_or(JournalError::State)?;
            checked(&pin.delete, false, &self.sid, pin.anchor)?;
            file_identity::ensure_regular_under(&self.owner, &pin.delete)
                .map_err(|_| JournalError::Changed)?;
        }
        let path = self.owner_path.clone();
        for pin in &self.receipts {
            dispose(&pin.as_ref().ok_or(JournalError::State)?.delete)?;
        }
        dispose(&self.file)?;
        // Closes receipt readers/writers, journal lock, and marker/root aliases.
        drop(self);
        for name in RECEIPTS.iter().copied().chain([RECORD]) {
            match std::fs::symlink_metadata(path.join(name)) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                _ => return Err(JournalError::Changed),
            }
        }
        Ok(())
    }
    pub(crate) fn all_receipts_match(&self) -> Result<bool, JournalError> {
        self.validate()?;
        let mut receipts = Vec::with_capacity(8);
        for index in 0..8 {
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
fn dispose(file: &File) -> Result<(), JournalError> {
    let mut info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: retained original DELETE handle, validated before any mutation.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&mut info as *mut FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(JournalError::Io);
    }
    Ok(())
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

fn open(
    path: &Path,
    directory: bool,
    access: u32,
    share: u32,
    create: Option<&PrivateDescriptor>,
) -> Result<File, JournalError> {
    if !path.is_absolute()
        || path.as_os_str().encode_wide().any(|c| c == 0)
        || path.as_os_str().encode_wide().count() > 32766
    {
        return Err(JournalError::Changed);
    }
    super::safe_components(path).map_err(|_| JournalError::Changed)?;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let attrs = create.map(PrivateDescriptor::attributes);
    let raw = unsafe {
        CreateFileW(
            wide.as_ptr(),
            access,
            share,
            attrs.as_ref().map_or(ptr::null(), |v| v),
            if create.is_some() {
                CREATE_NEW
            } else {
                OPEN_EXISTING
            },
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    FILE_ATTRIBUTE_NORMAL
                },
            ptr::null_mut(),
        )
    };
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(JournalError::Io);
    }
    let file = unsafe { File::from_raw_handle(raw) };
    kind(&file, directory)?;
    Ok(file)
}
fn kind(file: &File, directory: bool) -> Result<(), JournalError> {
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK {
        return Err(JournalError::Changed);
    }
    let mut tags = FILE_ATTRIBUTE_TAG_INFO::default();
    let mut standard = FILE_STANDARD_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tags as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&tags) as u32,
        )
    } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&standard) as u32,
            )
        } == 0
        || tags.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (tags.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || standard.Directory != directory
        || (!directory && standard.NumberOfLinks != 1)
        || standard.DeletePending
    {
        return Err(JournalError::Changed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        path: PathBuf,
        root: File,
        marker: File,
        lock: File,
        _temp: tempfile::TempDir,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("managed");
            std::fs::create_dir(&path).unwrap();
            let root = open(&path, true, META_ACCESS, SHARE_ALL, None).unwrap();
            let sid = current_sid().unwrap();
            let descriptor = PrivateDescriptor::new(&sid).unwrap();
            let mut marker = open(
                &path.join(".execution-pending"),
                false,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                SHARE_ALL,
                Some(&descriptor),
            )
            .unwrap();
            marker.write_all(b"synthetic original marker").unwrap();
            marker.sync_all().unwrap();
            std::fs::write(path.join(".install-transfer-lock"), b"original lock").unwrap();
            let lock = open(
                &path.join(".install-transfer-lock"),
                false,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                SHARE_ALL,
                None,
            )
            .unwrap();
            std::fs::write(temp.path().join("foreign"), b"preserve").unwrap();
            Self {
                path,
                root,
                marker,
                lock,
                _temp: temp,
            }
        }
        fn create(&self) -> ManagedJournal {
            ManagedJournal::create(
                &self.root,
                &self.path,
                &self.marker,
                &self.lock,
                [1; 16],
                [2; 32],
            )
            .unwrap()
        }
        fn preserved(&self) {
            assert_eq!(
                std::fs::read(self._temp.path().join("foreign")).unwrap(),
                b"preserve"
            );
        }
    }
    #[test]
    fn original_handles_receipt_handoff_and_reopen() {
        let f = Fixture::new();
        let mut journal = f.create();
        assert!(!journal.all_receipts_match().unwrap());
        let mut writer = journal
            .begin(0, ReaderStage::VersionProbe, [3; 16])
            .unwrap();
        assert_eq!(writer.index, 0);
        assert_eq!(writer.anchor, journal.anchors.receipts[0]);
        let bound = journal.bind(0, 10, 20).unwrap();
        let expected = journal
            .record()
            .unwrap()
            .receipt_for(0)
            .unwrap()
            .encode()
            .unwrap();
        journal.release_for_guardian(0, bound).unwrap();
        assert!(journal.all_receipts_match().is_err());
        assert!(journal.begin(1, ReaderStage::Metadata, [4; 16]).is_err());
        // Simulates native original receipt writing only; NOT actual Job authority.
        writer.file.seek(SeekFrom::Start(0)).unwrap();
        writer.file.write_all(&expected).unwrap();
        writer.file.sync_all().unwrap();
        drop(writer);
        journal.reacquire().unwrap();
        journal.resolve(0).unwrap();
        assert!(journal.all_receipts_match().unwrap());
        let anchors = journal.anchors();
        drop(journal);
        let reopened = ManagedJournal::reopen(
            &f.root, &f.path, &f.marker, &f.lock, anchors, [1; 16], [2; 32],
        )
        .unwrap();
        assert!(reopened.all_receipts_match().unwrap());
        f.preserved();
    }
    #[test]
    fn busy_and_original_record_corruption_preserve() {
        let f = Fixture::new();
        let mut journal = f.create();
        let anchors = journal.anchors();
        assert!(matches!(
            ManagedJournal::reopen(
                &f.root, &f.path, &f.marker, &f.lock, anchors, [1; 16], [2; 32]
            ),
            Err(JournalError::Busy)
        ));
        let writer = journal.begin(0, ReaderStage::Download, [3; 16]).unwrap();
        let bound = journal.bind(0, 10, 20).unwrap();
        journal.release_for_guardian(0, bound).unwrap();
        drop(writer);
        std::fs::write(f.path.join(RECORD), vec![b'x'; 16385]).unwrap();
        assert!(journal.reacquire().is_err());
        assert!(f.path.join(RECORD).exists());
        f.preserved();
    }
    #[test]
    fn replaced_receipt_and_unknown_record_preserve() {
        let f = Fixture::new();
        let journal = f.create();
        let anchors = journal.anchors();
        drop(journal);
        let path = f.path.join(RECEIPTS[0]);
        std::fs::rename(&path, f.path.join("original-receipt")).unwrap();
        std::fs::write(&path, b"foreign replacement").unwrap();
        assert!(
            ManagedJournal::reopen(
                &f.root, &f.path, &f.marker, &f.lock, anchors, [1; 16], [2; 32]
            )
            .is_err()
        );
        assert_eq!(std::fs::read(path).unwrap(), b"foreign replacement");
        f.preserved();
    }
    #[test]
    fn widened_owned_record_acl_is_rejected_without_deletion() {
        use windows_sys::Win32::Security::{
            Authorization::{SE_FILE_OBJECT, SetSecurityInfo},
            DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
        };
        let f = Fixture::new();
        let journal = f.create();
        let capability = open(
            &f.path.join(RECORD),
            false,
            WRITE_DAC | READ_CONTROL,
            SHARE_ALL,
            None,
        )
        .unwrap();
        // Only this test's newly owned journal. A null DACL grants access to all.
        let result = unsafe {
            SetSecurityInfo(
                capability.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_eq!(result, 0);
        assert!(journal.record().is_err());
        assert!(f.path.join(RECORD).exists());
        f.preserved();
    }
    #[test]
    fn live_pristine_cleanup_closes_handles_and_reopen_cannot_claim_pristine() {
        let f = Fixture::new();
        let journal = f.create();
        journal.cleanup_pristine().unwrap();
        for name in RECEIPTS.iter().copied().chain([RECORD]) {
            assert!(!f.path.join(name).exists());
        }
        assert!(f.path.join(".execution-pending").exists());
        f.preserved();
        let f = Fixture::new();
        let journal = f.create();
        let anchors = journal.anchors();
        drop(journal);
        let reopened = ManagedJournal::reopen(
            &f.root, &f.path, &f.marker, &f.lock, anchors, [1; 16], [2; 32],
        )
        .unwrap();
        assert!(reopened.cleanup_pristine().is_err());
        assert!(f.path.join(RECORD).exists());
        f.preserved();
    }
    #[test]
    fn completed_cleanup_requires_actual_receipt_and_preserves_marker() {
        let f = Fixture::new();
        let mut journal = f.create();
        let mut writer = journal
            .begin(0, ReaderStage::VersionProbe, [3; 16])
            .unwrap();
        journal.bind(0, 10, 20).unwrap();
        let bytes = journal
            .record()
            .unwrap()
            .receipt_for(0)
            .unwrap()
            .encode()
            .unwrap();
        writer.file.write_all(&bytes).unwrap();
        writer.file.sync_all().unwrap();
        drop(writer);
        journal.resolve(0).unwrap();
        journal.cleanup_completed().unwrap();
        for name in RECEIPTS.iter().copied().chain([RECORD]) {
            assert!(!f.path.join(name).exists());
        }
        assert_eq!(
            std::fs::read(f.path.join(".execution-pending")).unwrap(),
            b"synthetic original marker"
        );
        f.preserved();
    }
    #[test]
    fn unresolved_unknown_and_replaced_receipts_refuse_cleanup() {
        for mode in 0..4 {
            let f = Fixture::new();
            let mut journal = f.create();
            if mode < 2 {
                let writer = journal.begin(0, ReaderStage::Download, [3; 16]).unwrap();
                drop(writer);
                if mode == 1 {
                    journal.bind(0, 10, 20).unwrap();
                }
                assert!(journal.cleanup_completed().is_err());
            } else if mode == 2 {
                std::fs::write(f.path.join(RECEIPTS[7]), vec![b'x'; 257]).unwrap();
                assert!(journal.cleanup_pristine().is_err());
            } else {
                let name = f.path.join(RECEIPTS[7]);
                std::fs::rename(&name, f.path.join("original-receipt")).unwrap();
                std::fs::write(&name, b"foreign").unwrap();
                assert!(journal.cleanup_pristine().is_err());
                assert_eq!(std::fs::read(name).unwrap(), b"foreign");
            }
            assert!(f.path.join(RECORD).exists());
            assert!(f.path.join(".execution-pending").exists());
            f.preserved();
        }
    }
}
