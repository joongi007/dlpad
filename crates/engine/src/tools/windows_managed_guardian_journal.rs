//! Original Windows managed receipt capability; no deletion or PID authority.
use super::windows_managed_records::{ManagedReaderRecord, SessionAnchors, WindowsAnchor};
use crate::{
    file_identity,
    windows_private_security::{current_sid, verify_acl},
};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};
const META: u32 = FILE_READ_ATTRIBUTES | READ_CONTROL;
const SHARE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
const JOURNAL: &str = ".execution-readers.json";
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
fn rejected() -> io::Error {
    io::Error::other("Windows managed guardian capability rejected")
}
fn kind(f: &File, dir: bool) -> io::Result<()> {
    if unsafe { GetFileType(f.as_raw_handle()) } != FILE_TYPE_DISK {
        return Err(rejected());
    }
    let mut tags = FILE_ATTRIBUTE_TAG_INFO::default();
    let mut standard = FILE_STANDARD_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            f.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tags as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&tags) as u32,
        )
    } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                f.as_raw_handle(),
                FileStandardInfo,
                (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&standard) as u32,
            )
        } == 0
        || tags.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (tags.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != dir
        || standard.Directory != dir
        || standard.DeletePending
        || (!dir && standard.NumberOfLinks != 1)
    {
        return Err(rejected());
    }
    Ok(())
}
fn check(f: &File, dir: bool, sid: &str, a: WindowsAnchor, private: bool) -> io::Result<()> {
    kind(f, dir)?;
    if private {
        verify_acl(f, sid).map_err(|_| rejected())?;
    }
    match file_identity::from_file(f).map_err(|_| rejected())? {
        file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } if a.volume_serial == volume_serial && a.file_id == file_id => Ok(()),
        _ => Err(rejected()),
    }
}
fn open(path: &Path, dir: bool, access: u32) -> io::Result<File> {
    super::safe_components(path).map_err(|_| rejected())?;
    if !path.is_absolute()
        || path.as_os_str().encode_wide().any(|c| c == 0)
        || path.as_os_str().encode_wide().count() > 32766
    {
        return Err(rejected());
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let raw = unsafe {
        CreateFileW(
            wide.as_ptr(),
            access,
            SHARE,
            ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT
                | if dir {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    FILE_ATTRIBUTE_NORMAL
                },
            ptr::null_mut(),
        )
    };
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(rejected());
    }
    let file = unsafe { File::from_raw_handle(raw) };
    kind(&file, dir)?;
    Ok(file)
}
#[repr(C)]
#[derive(Default)]
struct ObjectBasic {
    attributes: u32,
    granted: u32,
    handles: u32,
    pointers: u32,
    reserved: [u32; 10],
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut std::ffi::c_void;
    fn GetProcAddress(module: *mut std::ffi::c_void, name: *const u8) -> *mut std::ffi::c_void;
}
fn writer_access(file: &File) -> io::Result<()> {
    type Query = unsafe extern "system" fn(
        *mut std::ffi::c_void,
        u32,
        *mut std::ffi::c_void,
        u32,
        *mut u32,
    ) -> i32;
    let dll: Vec<u16> = "ntdll.dll".encode_utf16().chain([0]).collect();
    let module = unsafe { GetModuleHandleW(dll.as_ptr()) };
    if module.is_null() {
        return Err(rejected());
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryObject".as_ptr().cast()) };
    if address.is_null() {
        return Err(rejected());
    }
    let query: Query = unsafe { std::mem::transmute(address) };
    let mut basic = ObjectBasic::default();
    let mut used = 0;
    let size = std::mem::size_of::<ObjectBasic>() as u32;
    let status = unsafe {
        query(
            file.as_raw_handle(),
            0,
            (&mut basic as *mut ObjectBasic).cast(),
            size,
            &mut used,
        )
    };
    // Metadata readability is permitted; payload readability/deletion/ACL mutation is not.
    if status < 0
        || used != size
        || basic.granted & FILE_WRITE_DATA == 0
        || basic.granted & !(FILE_GENERIC_WRITE | META) != 0
    {
        return Err(rejected());
    }
    if unsafe { SetHandleInformation(file.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(rejected());
    }
    Ok(())
}
fn record(file: &File) -> io::Result<ManagedReaderRecord> {
    let len = file.metadata()?.len();
    if len == 0 || len > 16384 {
        return Err(rejected());
    }
    let mut reader = file;
    reader.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(len as usize);
    reader.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() != len as usize {
        return Err(rejected());
    }
    ManagedReaderRecord::parse(&bytes).map_err(|_| rejected())
}
/// The caller MUST confirm the original native Job has zero processes before complete().
/// Root's launcher validates the original v3 marker binding separately.
pub(crate) struct ManagedGuardianJournal {
    path: PathBuf,
    sid: String,
    anchors: SessionAnchors,
    expected: ManagedReaderRecord,
    index: u8,
    root: File,
    marker: File,
    lock: File,
    journal: Option<File>,
    receipt_pins: [File; 8],
    writer: File,
    attempted: bool,
}
impl std::fmt::Debug for ManagedGuardianJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedGuardianJournalRedacted")
    }
}
impl ManagedGuardianJournal {
    pub(crate) fn acquire(
        root_path: &Path,
        intent: &ManagedReaderRecord,
        index: u8,
        nonce: [u8; 16],
        creation: (u64, u64),
        receipt: File,
    ) -> io::Result<Self> {
        let mut expected = intent.clone();
        let bound = expected
            .bind(index, creation.0, creation.1)
            .map_err(|_| rejected())?;
        if bound.nonce != nonce {
            return Err(rejected());
        }
        let sid = current_sid().map_err(|_| rejected())?;
        let anchors = expected.anchors();
        let root = open(root_path, true, META)?;
        check(&root, true, &sid, anchors.root, false)?;
        let marker = open(&root_path.join(".execution-pending"), false, META)?;
        let lock = open(&root_path.join(".install-transfer-lock"), false, META)?;
        let journal = open(&root_path.join(JOURNAL), false, FILE_GENERIC_READ)?;
        check(&marker, false, &sid, anchors.marker, true)?;
        check(&lock, false, &sid, anchors.lock, false)?;
        check(&journal, false, &sid, anchors.journal, true)?;
        journal.try_lock().map_err(|_| rejected())?;
        let mut pins = Vec::with_capacity(8);
        for (i, name) in RECEIPTS.iter().enumerate() {
            let pin = open(&root_path.join(name), false, META)?;
            check(&pin, false, &sid, anchors.receipts[i], true)?;
            pins.push(pin);
        }
        let receipt_pins: [File; 8] = pins.try_into().map_err(|_| rejected())?;
        let selected = *anchors.receipts.get(index as usize).ok_or_else(rejected)?;
        check(&receipt, false, &sid, selected, true)?;
        writer_access(&receipt)?;
        if receipt.metadata()?.len() != 0 {
            return Err(rejected());
        }
        let result = Self {
            path: root_path.to_owned(),
            sid,
            anchors,
            expected,
            index,
            root,
            marker,
            lock,
            journal: Some(journal),
            receipt_pins,
            writer: receipt,
            attempted: false,
        };
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> io::Result<()> {
        check(&self.root, true, &self.sid, self.anchors.root, false)?;
        let root = open(&self.path, true, META)?;
        check(&root, true, &self.sid, self.anchors.root, false)?;
        let journal = self.journal.as_ref().ok_or_else(rejected)?;
        for (name, file, a, private) in [
            (
                ".execution-pending",
                &self.marker,
                self.anchors.marker,
                true,
            ),
            (
                ".install-transfer-lock",
                &self.lock,
                self.anchors.lock,
                false,
            ),
            (JOURNAL, journal, self.anchors.journal, true),
        ] {
            check(file, false, &self.sid, a, private)?;
            file_identity::ensure_regular_under(&self.root, file).map_err(|_| rejected())?;
            let current = open(&self.path.join(name), false, META)?;
            check(&current, false, &self.sid, a, private)?;
        }
        for (i, pin) in self.receipt_pins.iter().enumerate() {
            check(pin, false, &self.sid, self.anchors.receipts[i], true)?;
            file_identity::ensure_regular_under(&self.root, pin).map_err(|_| rejected())?;
            let current = open(&self.path.join(RECEIPTS[i]), false, META)?;
            check(&current, false, &self.sid, self.anchors.receipts[i], true)?;
        }
        check(
            &self.writer,
            false,
            &self.sid,
            self.anchors.receipts[self.index as usize],
            true,
        )?;
        file_identity::ensure_regular_under(&self.root, &self.writer).map_err(|_| rejected())?;
        writer_access(&self.writer)?;
        if record(journal)?.encode().map_err(|_| rejected())?
            != self.expected.encode().map_err(|_| rejected())?
            || self.expected.pending_bound_identity(self.index).is_none()
        {
            return Err(rejected());
        }
        Ok(())
    }
    pub(crate) fn complete(&mut self) -> io::Result<()> {
        if self.attempted {
            return Err(rejected());
        }
        self.validate()?;
        if self.writer.metadata()?.len() != 0 {
            return Err(rejected());
        }
        let bytes = self
            .expected
            .receipt_for(self.index)
            .map_err(|_| rejected())?
            .encode()
            .map_err(|_| rejected())?;
        self.attempted = true;
        let result = (|| {
            self.writer.seek(SeekFrom::Start(0))?;
            self.writer.write_all(&bytes)?;
            self.writer.sync_all()
        })();
        // Original independent lock is released after the durable attempt, including failure.
        drop(self.journal.take());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::windows_managed_journal::ManagedJournal;
    use crate::tools::windows_managed_records::ReaderStage;
    use crate::windows_private_security::PrivateDescriptor;
    struct Fixture {
        _temp: tempfile::TempDir,
        path: PathBuf,
        root: File,
        marker: File,
        lock: File,
    }
    fn create_private(path: &Path) -> File {
        let sid = current_sid().unwrap();
        let descriptor = PrivateDescriptor::new(&sid).unwrap();
        let attrs = descriptor.attributes();
        let wide: Vec<_> = path.as_os_str().encode_wide().chain([0]).collect();
        let raw = unsafe {
            CreateFileW(
                wide.as_ptr(),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                SHARE,
                &attrs,
                CREATE_NEW,
                FILE_FLAG_OPEN_REPARSE_POINT | FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };
        assert!(!raw.is_null() && raw != INVALID_HANDLE_VALUE);
        unsafe { File::from_raw_handle(raw) }
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("managed");
            std::fs::create_dir(&path).unwrap();
            let root = open(&path, true, META).unwrap();
            let mut marker = create_private(&path.join(".execution-pending"));
            marker.write_all(b"original synthetic marker").unwrap();
            marker.sync_all().unwrap();
            std::fs::write(path.join(".install-transfer-lock"), b"original lock").unwrap();
            let lock = open(
                &path.join(".install-transfer-lock"),
                false,
                FILE_GENERIC_READ,
            )
            .unwrap();
            std::fs::write(temp.path().join("foreign"), b"foreign preserve").unwrap();
            Self {
                _temp: temp,
                path,
                root,
                marker,
                lock,
            }
        }
        fn app(&self) -> ManagedJournal {
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
                b"foreign preserve"
            );
            assert_eq!(
                std::fs::read(self.path.join(".execution-pending")).unwrap(),
                b"original synthetic marker"
            );
        }
    }
    #[test]
    fn readonly_independent_lock_actual_receipt_and_immediate_release() {
        let f = Fixture::new();
        let mut app = f.app();
        let writer = app.begin(0, ReaderStage::VersionProbe, [3; 16]).unwrap();
        let intent = app.record().unwrap().clone();
        let bound = app.bind(0, 10, 20).unwrap();
        assert!(
            ManagedGuardianJournal::acquire(
                &f.path,
                &intent,
                0,
                [3; 16],
                (10, 20),
                writer.file.try_clone().unwrap()
            )
            .is_err()
        );
        app.release_for_guardian(0, bound).unwrap();
        let mut helper =
            ManagedGuardianJournal::acquire(&f.path, &intent, 0, [3; 16], (10, 20), writer.file)
                .unwrap();
        assert!(app.reacquire().is_err());
        // This exercises native file capability and lock only; original Job proof is caller-owned.
        helper.complete().unwrap();
        assert!(helper.complete().is_err());
        assert_eq!(
            std::fs::metadata(f.path.join(RECEIPTS[0])).unwrap().len(),
            256
        );
        // complete releases the independent lock even while facade and its pins stay alive.
        app.reacquire().unwrap();
        app.resolve(0).unwrap();
        assert!(app.all_receipts_match().unwrap());
        f.preserved();
    }
    #[test]
    fn wrong_creation_nonce_and_payload_read_access_preserve() {
        let f = Fixture::new();
        let mut app = f.app();
        let writer = app.begin(0, ReaderStage::Download, [3; 16]).unwrap();
        let intent = app.record().unwrap().clone();
        let bound = app.bind(0, 10, 20).unwrap();
        app.release_for_guardian(0, bound).unwrap();
        for (nonce, creation) in [([4; 16], (10, 20)), ([3; 16], (10, 21))] {
            assert!(
                ManagedGuardianJournal::acquire(
                    &f.path,
                    &intent,
                    0,
                    nonce,
                    creation,
                    writer.file.try_clone().unwrap()
                )
                .is_err()
            );
        }
        for access in [FILE_GENERIC_READ, FILE_GENERIC_READ | FILE_GENERIC_WRITE] {
            let file = open(&f.path.join(RECEIPTS[0]), false, access).unwrap();
            assert!(
                ManagedGuardianJournal::acquire(&f.path, &intent, 0, [3; 16], (10, 20), file)
                    .is_err()
            );
        }
        assert_eq!(writer.file.metadata().unwrap().len(), 0);
        app.reacquire().unwrap();
        assert!(app.all_receipts_match().is_err());
        f.preserved();
    }
    #[test]
    fn partial_receipt_and_foreign_identity_fail_without_overwrite() {
        let f = Fixture::new();
        let mut app = f.app();
        let mut writer = app.begin(0, ReaderStage::Metadata, [3; 16]).unwrap();
        let intent = app.record().unwrap().clone();
        let bound = app.bind(0, 10, 20).unwrap();
        app.release_for_guardian(0, bound).unwrap();
        let foreign = create_private(&f._temp.path().join("foreign-private"));
        assert!(
            ManagedGuardianJournal::acquire(&f.path, &intent, 0, [3; 16], (10, 20), foreign)
                .is_err()
        );
        let mut helper = ManagedGuardianJournal::acquire(
            &f.path,
            &intent,
            0,
            [3; 16],
            (10, 20),
            writer.file.try_clone().unwrap(),
        )
        .unwrap();
        writer.file.write_all(b"partial").unwrap();
        writer.file.sync_all().unwrap();
        assert!(helper.complete().is_err());
        assert_eq!(std::fs::read(f.path.join(RECEIPTS[0])).unwrap(), b"partial");
        drop(helper);
        app.reacquire().unwrap();
        assert!(app.resolve(0).is_err());
        f.preserved();
    }
}
