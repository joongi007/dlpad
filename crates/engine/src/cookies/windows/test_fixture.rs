//! Synthetic Windows fixture only. Does not authorize product recovery.
use super::journal::Journal;
use super::{
    Descriptor, META_ACCESS, SHARE_ALL, create_directory, current_sid, file_identity, open,
    prepare_cache,
};
use crate::cookies::windows_records::{BoundIdentity, WindowsAnchor, WindowsReaderRecord};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE};
pub(crate) struct TestFixture {
    temp: Option<tempfile::TempDir>,
    owner_path: PathBuf,
    source: PathBuf,
    copy_path: PathBuf,
    bytes: Vec<u8>,
    journal: Option<Journal>,
    owner: Option<File>,
    cache: Option<File>,
    copy: Option<File>,
}
fn anchor(file: &File) -> WindowsAnchor {
    match file_identity::from_file(file).unwrap() {
        file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } => WindowsAnchor {
            volume_serial,
            file_id,
        },
        _ => panic!("unexpected native fixture identity"),
    }
}
impl TestFixture {
    pub(crate) fn create() -> Self {
        Self::create_seeded(false)
    }
    pub(crate) fn create_with_base_digest() -> Self {
        Self::create_seeded(true)
    }
    fn create_seeded(use_base: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let mut fixture = Self::create_in_seeded(temp.path(), use_base);
        fixture.temp = Some(temp);
        fixture
    }
    pub(crate) fn reopen_seed(
        &mut self,
        expected: crate::cookies::windows_records::SessionAnchors,
        nonce: [u8; 16],
        digest: [u8; 32],
    ) -> bool {
        drop(self.journal.take());
        match Journal::reopen(
            self.owner.as_ref().unwrap(),
            &self.owner_path,
            &current_sid().unwrap(),
            expected,
            nonce,
            digest,
        ) {
            Ok(journal) => {
                self.journal = Some(journal);
                true
            }
            Err(_) => false,
        }
    }
    fn journal(&self) -> &Journal {
        self.journal.as_ref().unwrap()
    }
    fn journal_mut(&mut self) -> &mut Journal {
        self.journal.as_mut().unwrap()
    }
    pub(crate) fn owner_path(&self) -> &Path {
        &self.owner_path
    }
    pub(crate) fn begin(&mut self, index: u8, nonce: [u8; 16]) -> File {
        self.journal_mut().begin(index, nonce).unwrap().file
    }
    pub(crate) fn record(&self) -> WindowsReaderRecord {
        self.journal().record().unwrap().clone()
    }
    pub(crate) fn bind(&mut self, index: u8, guardian: u64, tool: u64) -> BoundIdentity {
        self.journal_mut().bind(index, guardian, tool).unwrap()
    }
    pub(crate) fn release(&mut self, index: u8, bound: BoundIdentity) {
        self.journal_mut()
            .release_for_guardian(index, bound)
            .unwrap()
    }
    pub(crate) fn reacquire(&mut self) {
        self.journal_mut().reacquire().unwrap()
    }
    pub(crate) fn resolve(&mut self, index: u8) {
        self.journal_mut().resolve(index).unwrap()
    }
    pub(crate) fn all_receipts_match(&self) -> bool {
        self.journal().all_receipts_match().unwrap_or(false)
    }
    pub(crate) fn receipt_bytes(&self, index: u8) -> Vec<u8> {
        assert!(index < 2);
        let name = if index == 0 {
            "reader0.receipt"
        } else {
            "reader1.receipt"
        };
        let mut bytes = Vec::new();
        File::open(self.owner_path.join(name))
            .unwrap()
            .take(257)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() <= 256);
        bytes
    }
    pub(crate) fn copybytes(&self) -> Vec<u8> {
        let bytes = fs::read(&self.copy_path).unwrap();
        assert_eq!(bytes, self.bytes);
        assert_eq!(fs::read(&self.source).unwrap(), self.bytes);
        bytes
    }
}
impl Drop for TestFixture {
    fn drop(&mut self) {
        drop(self.journal.take());
        drop(self.copy.take());
        drop(self.owner.take());
        drop(self.cache.take());
        if let Some(temp) = self.temp.take() {
            let _ = temp.close();
        }
    }
}

/// Uses the real product importer and public startup recovery, synthetic files only.
pub(crate) struct ProductFixture {
    _temp: tempfile::TempDir,
    cache: PathBuf,
    source: PathBuf,
    copy_path: PathBuf,
    bytes: Vec<u8>,
    owned: Option<super::OwnedCopy>,
}
impl ProductFixture {
    pub(crate) fn create() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("synthetic-source.txt");
        let bytes =
            b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-only\n"
                .to_vec();
        fs::write(&source, &bytes).unwrap();
        let cache = temp.path().join("private-cache");
        prepare_cache(&cache).unwrap();
        fs::write(cache.join("foreign-sentinel"), b"preserve").unwrap();
        let (owned, copy_path, _, _) = super::import(&source, &cache).unwrap();
        Self {
            _temp: temp,
            cache,
            source,
            copy_path,
            bytes,
            owned: Some(owned),
        }
    }
    fn owned(&self) -> &super::OwnedCopy {
        self.owned.as_ref().unwrap()
    }
    fn journal(&mut self) -> &mut Journal {
        self.owned
            .as_mut()
            .unwrap()
            .journal
            .as_mut()
            .unwrap()
            .get_mut()
            .unwrap()
    }
    pub(crate) fn owner_path(&self) -> &Path {
        self.copy_path.parent().unwrap()
    }
    pub(crate) fn record(&self) -> WindowsReaderRecord {
        self.owned()
            .journal
            .as_ref()
            .unwrap()
            .try_lock()
            .unwrap()
            .record()
            .unwrap()
            .clone()
    }
    pub(crate) fn begin(&mut self, slot: u8, nonce: [u8; 16]) -> File {
        self.journal().begin(slot, nonce).unwrap().file
    }
    pub(crate) fn bind(&mut self, slot: u8, guardian: u64, tool: u64) -> BoundIdentity {
        self.journal().bind(slot, guardian, tool).unwrap()
    }
    pub(crate) fn release(&mut self, slot: u8, bound: BoundIdentity) {
        self.journal().release_for_guardian(slot, bound).unwrap();
    }
    pub(crate) fn receipt_bytes(&self, slot: u8) -> Vec<u8> {
        assert!(slot < 2);
        let mut bytes = Vec::new();
        File::open(self.owner_path().join(format!("reader{slot}.receipt")))
            .unwrap()
            .take(257)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() <= 256);
        bytes
    }
    pub(crate) fn defer_and_recover(&mut self) {
        let owner = self.owner_path().to_owned();
        self.owned.as_mut().unwrap().defer_cleanup();
        drop(self.owned.take());
        let summary = crate::cookies::recover_sessions(&self.cache).unwrap();
        assert_eq!(summary.recovered, 1, "{summary:?}");
        assert_eq!(summary.deferred, 0);
        assert!(!self.copy_path.exists());
        assert!(!owner.exists());
        assert_eq!(fs::read(&self.source).unwrap(), self.bytes);
        assert_eq!(
            fs::read(self.cache.join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
        assert_eq!(
            crate::cookies::recover_sessions(&self.cache)
                .unwrap()
                .recovered,
            0
        );
    }
}

impl TestFixture {
    pub(crate) fn create_in(base: &Path) -> Self {
        Self::create_in_seeded(base, true)
    }
    fn create_in_seeded(base: &Path, use_base: bool) -> Self {
        assert!(base.is_absolute() && base.is_dir());
        let cache_path = base.join("private-cache");
        prepare_cache(&cache_path).unwrap();
        let sid = current_sid().unwrap();
        let descriptor = Descriptor::private(&sid).unwrap();
        let owner_path = cache_path.join("synthetic-owner");
        assert!(create_directory(&owner_path, &descriptor).unwrap());
        let owner = open(&owner_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let cache = open(&cache_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let copy_path = owner_path.join("cookies.txt");
        let mut copy = open(
            &copy_path,
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .unwrap();
        let bytes =
            b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-only\n"
                .to_vec();
        copy.write_all(&bytes).unwrap();
        copy.sync_all().unwrap();
        let source = base.join("synthetic-source.txt");
        fs::write(&source, &bytes).unwrap();
        let journal = Journal::create(
            &owner,
            &owner_path,
            &sid,
            [1; 16],
            if use_base {
                super::recovery_marker::base_digest(
                    [1; 16],
                    anchor(&cache),
                    anchor(&owner),
                    anchor(&copy),
                )
                .unwrap()
            } else {
                [2; 32]
            },
            anchor(&cache),
            anchor(&copy),
        )
        .unwrap();
        Self {
            temp: None,
            owner_path,
            source,
            copy_path,
            bytes,
            journal: Some(journal),
            owner: Some(owner),
            cache: Some(cache),
            copy: Some(copy),
        }
    }
}
// Retain original metadata pins before killing producer; data/journal lock access only afterward.
pub(crate) struct AppKillObserver {
    owner_path: PathBuf,
    base: PathBuf,
    sid: String,
    expected: WindowsReaderRecord,
    owner: File,
    cache: File,
    copy: File,
    record_pin: File,
    receipt_pins: [File; 2],
    _ancestors: Vec<super::Pin>,
}
impl AppKillObserver {
    pub(crate) fn pin_original(base: &Path, expected: WindowsReaderRecord) -> Self {
        let owner_path = base.join("private-cache").join("synthetic-owner");
        let (root, names) = super::path_parts(&owner_path).unwrap();
        let ancestors = super::chain(root, &names).unwrap();
        let sid = current_sid().unwrap();
        let anchors = expected.anchors();
        let owner = open(&owner_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let cache = open(
            owner_path.parent().unwrap(),
            true,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .unwrap();
        let copy = open(
            &owner_path.join("cookies.txt"),
            false,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .unwrap();
        let record_pin = open(
            &owner_path.join("reader-state.json"),
            false,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .unwrap();
        let receipt_pins = std::array::from_fn(|index| {
            open(
                &owner_path.join(if index == 0 {
                    "reader0.receipt"
                } else {
                    "reader1.receipt"
                }),
                false,
                FILE_GENERIC_READ,
                SHARE_ALL,
                None,
            )
            .unwrap()
        });
        for (file, directory, id) in [
            (&owner, true, anchors.owner),
            (&cache, true, anchors.cache),
            (&copy, false, anchors.copy),
            (&record_pin, false, anchors.journal),
            (&receipt_pins[0], false, anchors.receipts[0]),
            (&receipt_pins[1], false, anchors.receipts[1]),
        ] {
            super::kind(file, directory).unwrap();
            super::acl(file, &sid).unwrap();
            assert_eq!(anchor(file), id);
        }
        Self {
            owner_path,
            base: base.to_owned(),
            sid,
            expected,
            owner,
            cache,
            copy,
            record_pin,
            receipt_pins,
            _ancestors: ancestors,
        }
    }
    pub(crate) fn receipt_empty(&self) -> bool {
        self.receipt_pins
            .iter()
            .all(|file| file.metadata().unwrap().len() == 0)
    }
    pub(crate) fn recover_original(&self) {
        let anchors = self.expected.anchors();
        for (file, directory, id) in [
            (&self.owner, true, anchors.owner),
            (&self.cache, true, anchors.cache),
            (&self.copy, false, anchors.copy),
            (&self.record_pin, false, anchors.journal),
            (&self.receipt_pins[0], false, anchors.receipts[0]),
            (&self.receipt_pins[1], false, anchors.receipts[1]),
        ] {
            super::kind(file, directory).unwrap();
            super::acl(file, &self.sid).unwrap();
            assert_eq!(anchor(file), id);
        }
        let mut journal = Journal::reopen(
            &self.owner,
            &self.owner_path,
            &self.sid,
            anchors,
            self.expected.session_nonce(),
            self.expected.record_digest(),
        )
        .unwrap();
        assert_eq!(
            journal.record().unwrap().encode().unwrap(),
            self.expected.encode().unwrap()
        );
        assert!(journal.all_receipts_match().unwrap());
        assert_eq!(self.receipt_pins[0].metadata().unwrap().len(), 256);
        assert_eq!(self.receipt_pins[1].metadata().unwrap().len(), 0);
        journal.resolve(0).unwrap();
        assert!(journal.all_receipts_match().unwrap());
        let expected =
            b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-only\n";
        assert_eq!(
            fs::read(self.owner_path.join("cookies.txt")).unwrap(),
            expected
        );
        assert_eq!(
            fs::read(self.base.join("synthetic-source.txt")).unwrap(),
            expected
        );
    }
}
