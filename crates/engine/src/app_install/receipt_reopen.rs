//! Readonly external receipt consistency, not verification of a live registered target.
//! No DELETE access, adoption, launch or historical ownership authority.
use super::{
    receipt_schema::{ReceiptRecord, serialize_receipt},
    windows_caps::{ReadFile, ReadRoot},
};
use std::os::windows::fs::FileExt;
use std::{
    collections::BTreeSet,
    fs::File,
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "receipt inspection rejected")
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "receipt inspection cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "receipt inspection deadline",
        ))
    } else {
        Ok(())
    }
}
fn bytes(file: &File, cancel: &AtomicBool, deadline: Instant) -> io::Result<Vec<u8>> {
    check(cancel, deadline)?;
    let len = file.metadata()?.len();
    if len == 0 || len > 65536 {
        return Err(bad());
    }
    let mut bytes = vec![0; len as usize];
    let mut offset = 0;
    while offset < bytes.len() {
        check(cancel, deadline)?;
        let n = file.seek_read(&mut bytes[offset..], offset as u64)?;
        if n == 0 {
            return Err(bad());
        }
        offset += n;
    }
    check(cancel, deadline)?;
    if file.seek_read(&mut [0], len)? != 0 || file.metadata()?.len() != len {
        return Err(bad());
    }
    check(cancel, deadline)?;
    Ok(bytes)
}
fn namespace(root: &ReadRoot, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    check(cancel, deadline)?;
    root.verify()?;
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(root.path())? {
        check(cancel, deadline)?;
        let entry = entry?;
        if !names.is_empty() || !entry.file_type()?.is_file() || !names.insert(entry.file_name()) {
            return Err(bad());
        }
    }
    if names != BTreeSet::from([std::ffi::OsString::from("registration-receipt.json")]) {
        return Err(bad());
    }
    root.verify()?;
    check(cancel, deadline)
}
pub struct ReceiptInspection {
    root: ReadRoot,
    file: ReadFile,
    record: ReceiptRecord,
}
impl ReceiptInspection {
    pub(super) fn into_bundle_removal_inputs(self) -> (ReadRoot, ReceiptRecord) {
        let Self { root, file, record } = self;
        drop(file);
        (root, record)
    }
    pub(super) fn original_final_path(&self) -> io::Result<std::path::PathBuf> {
        self.root.original_final_path()
    }
    pub fn record(&self) -> &ReceiptRecord {
        &self.record
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn verify_current(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        check(cancel, deadline)?;
        self.record.validate()?;
        self.root.verify()?;
        self.file.verify(&self.root)?;
        if self.root.identity() != &self.record.receipt_root_identity
            || self.file.identity() != &self.record.receipt_identity
        {
            return Err(bad());
        }
        if bytes(self.file.file(), cancel, deadline)? != serialize_receipt(&self.record)? {
            return Err(bad());
        }
        self.file.verify(&self.root)?;
        namespace(&self.root, cancel, deadline)
    }
}
impl std::fmt::Debug for ReceiptInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiptInspection").finish_non_exhaustive()
    }
}
pub fn inspect_registration_receipt(
    path: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<ReceiptInspection> {
    check(cancel, deadline)?;
    let root = ReadRoot::open_existing(path)?;
    namespace(&root, cancel, deadline)?;
    let file = root.open_file("registration-receipt.json")?;
    check(cancel, deadline)?;
    let raw = bytes(file.file(), cancel, deadline)?;
    let record = ReceiptRecord::read(raw.as_slice())?;
    if raw != serialize_receipt(&record)?
        || root.identity() != &record.receipt_root_identity
        || file.identity() != &record.receipt_identity
    {
        return Err(bad());
    }
    let value = ReceiptInspection { root, file, record };
    value.verify_current(cancel, deadline)?;
    Ok(value)
}

/// Private fresh observer, not an alternate public cold path constructor.
pub(super) fn inspect_fresh_receipt(
    fresh: &super::FreshRegistrationReceipt,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<ReceiptInspection> {
    check(cancel, deadline)?;
    let original = fresh.fresh_observation_root(cancel, deadline)?;
    let root = ReadRoot::observe_fresh(original)?;
    let record = fresh.record().clone();
    let file = root
        .open_file_for_fresh_observation("registration-receipt.json", &record.receipt_identity)?;
    let inspection = ReceiptInspection { root, file, record };
    inspection.verify_current(cancel, deadline)?;
    fresh.verify_current(cancel, deadline)?;
    inspection.verify_current(cancel, deadline)?;
    Ok(inspection)
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::time::Duration;
    use std::{fs, io::Write, os::windows::fs::OpenOptionsExt};
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn written() -> (tempfile::TempDir, std::path::PathBuf, Vec<u8>) {
        let d = tempfile::tempdir().unwrap();
        let mut b = vec![0; 256];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(d.path().join("source"), &b).unwrap();
        fs::write(d.path().join("sentinel"), b"outside").unwrap();
        let source = File::open(d.path().join("source")).unwrap();
        let m = super::super::manifest::BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: format!("{:x}", Sha256::digest(&b)),
            }],
        };
        let cancel = AtomicBool::new(false);
        let p = super::super::prepare_new(
            &m,
            &[("dlpad.exe", &source)],
            &d.path().join("registration"),
            &cancel,
            later(),
        )
        .unwrap();
        let r = super::super::register_fresh(p, &cancel, later()).unwrap();
        let path = d.path().join("receipt");
        let receipt =
            super::super::create_registration_receipt(&r, &path, &cancel, later()).unwrap();
        let raw = serialize_receipt(receipt.record()).unwrap();
        drop(receipt);
        drop(r);
        (d, path, raw)
    }
    #[test]
    fn actual_writer_drop_then_readonly_inspection_preserves_bytes() {
        let (d, path, raw) = written();
        let view = inspect_registration_receipt(&path, &AtomicBool::new(false), later()).unwrap();
        view.verify_current(&AtomicBool::new(false), later())
            .unwrap();
        assert_eq!(
            view.record().receipt_root_identity,
            view.root.identity().clone()
        );
        drop(view);
        assert_eq!(
            fs::read(path.join("registration-receipt.json")).unwrap(),
            raw
        );
        assert_eq!(fs::read(d.path().join("sentinel")).unwrap(), b"outside");
    }
    #[test]
    fn actual_same_bytes_different_file_identity_refused() {
        let (_d, path, raw) = written();
        let original = path.join("registration-receipt.json");
        let retired = path.with_file_name("retired-receipt");
        fs::rename(&original, &retired).unwrap();
        let replacement =
            super::super::FreshRoot::create_new(&path.with_file_name("replacement-cap-root"))
                .unwrap();
        let mut created = replacement
            .create_file("registration-receipt.json")
            .unwrap();
        created.file_mut().write_all(&raw).unwrap();
        created.file().sync_all().unwrap();
        drop(created);
        drop(replacement);
        fs::rename(
            path.with_file_name("replacement-cap-root")
                .join("registration-receipt.json"),
            &original,
        )
        .unwrap();
        let root = ReadRoot::open_existing(&path).unwrap();
        let current = root.open_file("registration-receipt.json").unwrap();
        let record = ReceiptRecord::read(raw.as_slice()).unwrap();
        assert_ne!(current.identity(), &record.receipt_identity);
        assert_eq!(fs::read(&retired).unwrap(), raw);
        drop(current);
        drop(root);
        assert_eq!(
            inspect_registration_receipt(&path, &AtomicBool::new(false), later())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read(&original).unwrap(), raw);
    }
    #[test]
    fn cancel_deadline_and_unknown_namespace_preserved() {
        let (_d, path, raw) = written();
        for (c, end, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            assert_eq!(
                inspect_registration_receipt(&path, &AtomicBool::new(c), end)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        fs::write(path.join("foreign"), b"foreign").unwrap();
        assert!(inspect_registration_receipt(&path, &AtomicBool::new(false), later()).is_err());
        assert_eq!(fs::read(path.join("foreign")).unwrap(), b"foreign");
        assert_eq!(
            fs::read(path.join("registration-receipt.json")).unwrap(),
            raw
        );
    }
    #[test]
    fn mutated_unknown_or_oversize_receipt_is_never_repaired() {
        for oversize in [false, true] {
            let (_d, path, raw) = written();
            let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
            value["unknown"] = true.into();
            let changed = if oversize {
                vec![b' '; 65537]
            } else {
                serde_json::to_vec(&value).unwrap()
            };
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .share_mode(7)
                .open(path.join("registration-receipt.json"))
                .unwrap();
            f.write_all(&changed).unwrap();
            drop(f);
            assert!(inspect_registration_receipt(&path, &AtomicBool::new(false), later()).is_err());
            assert_eq!(
                fs::read(path.join("registration-receipt.json")).unwrap(),
                changed
            );
        }
    }
}
