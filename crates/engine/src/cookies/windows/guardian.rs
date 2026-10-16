//! Guardian-only retained original-file capability; no PID or deletion authority.
use super::journal::Journal;
use super::{META_ACCESS, SHARE_ALL, acl, current_sid, file_identity, kind, open};
use crate::cookies::windows_records::{WindowsAnchor, WindowsReaderRecord};
use std::{
    fs::File,
    io::{self, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

fn rejected() -> io::Error {
    io::Error::other("Windows guardian journal capability rejected")
}
fn check(file: &File, directory: bool, sid: &str, expected: WindowsAnchor) -> io::Result<()> {
    kind(file, directory).map_err(|_| rejected())?;
    acl(file, sid).map_err(|_| rejected())?;
    match file_identity::from_file(file).map_err(|_| rejected())? {
        file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } if volume_serial == expected.volume_serial && file_id == expected.file_id => Ok(()),
        _ => Err(rejected()),
    }
}
/// Possesses an independently locked journal and original receipt writer.
/// The runtime must confirm its original Job empty before calling complete.
pub(crate) struct GuardianJournal {
    journal: Journal,
    cache: File,
    copy: File,
    receipt: File,
    owner_path: PathBuf,
    sid: String,
    index: u8,
    completed: bool,
}
impl GuardianJournal {
    pub(crate) fn acquire(
        owner_path: &Path,
        intent: &WindowsReaderRecord,
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
        let owner = open(owner_path, true, META_ACCESS, SHARE_ALL, None).map_err(|_| rejected())?;
        check(&owner, true, &sid, anchors.owner)?;
        let cache_path = owner_path.parent().ok_or_else(rejected)?;
        let cache = open(cache_path, true, META_ACCESS, SHARE_ALL, None).map_err(|_| rejected())?;
        check(&cache, true, &sid, anchors.cache)?;
        let copy = open(
            &owner_path.join("cookies.txt"),
            false,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .map_err(|_| rejected())?;
        check(&copy, false, &sid, anchors.copy)?;
        let receipt_anchor = *anchors.receipts.get(index as usize).ok_or_else(rejected)?;
        check(&receipt, false, &sid, receipt_anchor)?;
        if receipt.metadata()?.len() != 0 {
            return Err(rejected());
        }
        let journal = Journal::reopen(
            &owner,
            owner_path,
            &sid,
            anchors,
            expected.session_nonce(),
            expected.record_digest(),
        )
        .map_err(|_| rejected())?;
        if journal
            .record()
            .map_err(|_| rejected())?
            .encode()
            .map_err(|_| rejected())?
            != expected.encode().map_err(|_| rejected())?
        {
            return Err(rejected());
        }
        Ok(Self {
            journal,
            cache,
            copy,
            receipt,
            owner_path: owner_path.to_owned(),
            sid,
            index,
            completed: false,
        })
    }
    /// Invoke only after the caller confirms its original Job has no processes.
    pub(crate) fn complete(&mut self) -> io::Result<()> {
        if self.completed {
            return Err(rejected());
        }
        let record = self.journal.record().map_err(|_| rejected())?;
        let anchors = record.anchors();
        check(&self.cache, true, &self.sid, anchors.cache)?;
        check(&self.copy, false, &self.sid, anchors.copy)?;
        let current_copy = open(
            &self.owner_path.join("cookies.txt"),
            false,
            META_ACCESS,
            SHARE_ALL,
            None,
        )
        .map_err(|_| rejected())?;
        check(&current_copy, false, &self.sid, anchors.copy)?;
        check(
            &self.receipt,
            false,
            &self.sid,
            anchors.receipts[self.index as usize],
        )?;
        if record.pending_bound_identity(self.index).is_none()
            || self.receipt.metadata()?.len() != 0
        {
            return Err(rejected());
        }
        let bytes = record
            .receipt_for(self.index)
            .map_err(|_| rejected())?
            .encode()
            .map_err(|_| rejected())?;
        // A failed/partial write cannot be retried into a fabricated clean receipt.
        self.completed = true;
        self.receipt.seek(SeekFrom::Start(0))?;
        self.receipt.write_all(&bytes)?;
        self.receipt.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Descriptor, create_directory, prepare_cache};
    use super::*;
    use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE};
    fn anchor(file: &File) -> WindowsAnchor {
        match file_identity::from_file(file).unwrap() {
            file_identity::NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } => WindowsAnchor {
                volume_serial,
                file_id,
            },
            _ => panic!("unexpected platform"),
        }
    }
    #[test]
    fn original_bound_lock_and_receipt_capability_reject_foreign_creation() {
        let temp = tempfile::tempdir().unwrap();
        let cache_path = temp.path().join("cache");
        prepare_cache(&cache_path).unwrap();
        let sid = current_sid().unwrap();
        let descriptor = Descriptor::private(&sid).unwrap();
        let path = cache_path.join("owner");
        assert!(create_directory(&path, &descriptor).unwrap());
        let owner = open(&path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let cache = open(&cache_path, true, META_ACCESS, SHARE_ALL, None).unwrap();
        let copy = open(
            &path.join("cookies.txt"),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .unwrap();
        let mut app = Journal::create(
            &owner,
            &path,
            &sid,
            [1; 16],
            [2; 32],
            anchor(&cache),
            anchor(&copy),
        )
        .unwrap();
        let writer = app.begin(0, [3; 16]).unwrap();
        let intent = app.record().unwrap().clone();
        let bound = app.bind(0, 10, 20).unwrap();
        assert!(
            GuardianJournal::acquire(
                &path,
                &intent,
                0,
                [3; 16],
                (10, 20),
                writer.file.try_clone().unwrap()
            )
            .is_err()
        );
        app.release_for_guardian(0, bound).unwrap();
        assert!(
            GuardianJournal::acquire(
                &path,
                &intent,
                0,
                [3; 16],
                (10, 21),
                writer.file.try_clone().unwrap()
            )
            .is_err()
        );
        assert_eq!(writer.file.metadata().unwrap().len(), 0);
        let mut helper =
            GuardianJournal::acquire(&path, &intent, 0, [3; 16], (10, 20), writer.file).unwrap();
        assert!(app.reacquire().is_err());
        // This tests the file capability only; no process Job runs in this fixture.
        helper.complete().unwrap();
        assert!(helper.complete().is_err());
        drop(helper);
        app.reacquire().unwrap();
        app.resolve(0).unwrap();
        assert!(app.all_receipts_match().unwrap());
        assert_eq!(copy.metadata().unwrap().len(), 0);
    }
}
