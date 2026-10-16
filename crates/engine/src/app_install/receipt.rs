//! Fresh original registration observation; no deletion/adoption/active authority.
use super::{
    payload::reading_pin,
    receipt_schema::{ReceiptRecord, serialize_receipt},
    registration::FreshRegistration,
    windows_caps::{CreatedFile, FreshRoot},
};
use crate::file_identity;
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
    FILE_SHARE_READ, FILE_SHARE_WRITE,
};
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(a: *const u16, al: i32, b: *const u16, bl: i32, ignore: i32) -> i32;
}
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "registration receipt verification failed",
    )
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "receipt cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(io::ErrorKind::TimedOut, "receipt deadline"))
    } else {
        Ok(())
    }
}
fn contains(parent: &Path, child: &Path) -> bool {
    let a: Vec<u16> = parent.as_os_str().encode_wide().collect();
    let b: Vec<u16> = child.as_os_str().encode_wide().collect();
    a.len() <= b.len()
        && unsafe {
            CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1)
        } == 2
        && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92))
}
fn disjoint(a: &Path, b: &Path) -> io::Result<()> {
    if contains(a, b) || contains(b, a) {
        Err(bad())
    } else {
        Ok(())
    }
}
fn namespace(root: &FreshRoot, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    root.verify()?;
    let mut names = BTreeSet::new();
    for e in std::fs::read_dir(root.path())? {
        check(cancel, deadline)?;
        let e = e?;
        if !e.file_type()?.is_file() || !names.insert(e.file_name()) || names.len() > 1 {
            return Err(bad());
        }
    }
    if names != BTreeSet::from([std::ffi::OsString::from("registration-receipt.json")]) {
        return Err(bad());
    }
    root.verify()?;
    check(cancel, deadline)
}
pub struct FreshRegistrationReceipt {
    root: FreshRoot,
    file: CreatedFile,
    record: ReceiptRecord,
}
impl FreshRegistrationReceipt {
    /// Narrow original-root input; never converts a path/DTO into ownership.
    pub(super) fn fresh_observation_root(
        &self,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<&FreshRoot> {
        self.verify_current(cancel, deadline)?;
        Ok(&self.root)
    }
    pub fn record(&self) -> &ReceiptRecord {
        &self.record
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn file(&self) -> &File {
        self.file.file()
    }
    pub fn verify_current(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        check(cancel, deadline)?;
        self.record.validate()?;
        self.root.verify()?;
        self.file.verify(&self.root)?;
        if self.file.name() != "registration-receipt.json"
            || self.root.identity() != &self.record.receipt_root_identity
            || self.file.identity() != &self.record.receipt_identity
        {
            return Err(bad());
        }
        let expected = serialize_receipt(&self.record)?;
        if expected.len() > 65536 || self.file.file().metadata()?.len() != expected.len() as u64 {
            return Err(bad());
        }
        let mut input = reading_pin(self.file.file())?;
        input.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        let mut buf = [0u8; 65536];
        loop {
            check(cancel, deadline)?;
            let n = input.read(&mut buf)?;
            if n == 0 {
                break;
            }
            if bytes.len() + n > 65536 {
                return Err(bad());
            }
            bytes.extend_from_slice(&buf[..n]);
        }
        if bytes != expected {
            return Err(bad());
        }
        self.file.verify(&self.root)?;
        namespace(&self.root, cancel, deadline)
    }
}
impl std::fmt::Debug for FreshRegistrationReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FreshRegistrationReceipt")
            .finish_non_exhaustive()
    }
}
pub struct ReceiptFailure {
    error: io::Error,
    mutation: bool,
    target_hint: PathBuf,
    root: Option<Box<FreshRoot>>,
    file: Option<Box<CreatedFile>>,
}
impl ReceiptFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.mutation
    }
    pub fn root_capability_acquired(&self) -> bool {
        self.root.is_some()
    }
    pub fn retained_file_cap_count(&self) -> usize {
        usize::from(self.file.is_some())
    }
    pub fn target_hint(&self) -> &Path {
        &self.target_hint
    }
}
impl std::fmt::Debug for ReceiptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiptFailure")
            .field("kind", &self.kind())
            .field("mutation", &self.mutation)
            .field("root_capability", &self.root_capability_acquired())
            .field("file_caps", &self.retained_file_cap_count())
            .finish()
    }
}
impl std::fmt::Display for ReceiptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "registration receipt failed ({:?}); partial objects preserved",
            self.kind()
        )
    }
}
impl std::error::Error for ReceiptFailure {}
pub fn create_registration_receipt(
    registration: &FreshRegistration,
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<FreshRegistrationReceipt, ReceiptFailure> {
    create_with_hook(registration, target, cancel, deadline, &mut |_| Ok(()))
}
pub(super) fn create_registration_receipt_with_root_factory(
    registration: &FreshRegistration,
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    factory: &mut dyn FnMut() -> io::Result<FreshRoot>,
) -> Result<FreshRegistrationReceipt, ReceiptFailure> {
    create_with_hook_and_factory(
        registration,
        target,
        cancel,
        deadline,
        &mut |_| Ok(()),
        factory,
    )
}
fn create_with_hook(
    registration: &FreshRegistration,
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(&str) -> io::Result<()>,
) -> Result<FreshRegistrationReceipt, ReceiptFailure> {
    create_with_hook_and_factory(registration, target, cancel, deadline, hook, &mut || {
        FreshRoot::create_new(target)
    })
}
fn create_with_hook_and_factory(
    registration: &FreshRegistration,
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(&str) -> io::Result<()>,
    factory: &mut dyn FnMut() -> io::Result<FreshRoot>,
) -> Result<FreshRegistrationReceipt, ReceiptFailure> {
    // Limit UTF-16 scanning and path duplication before native calls.
    let mut length = 0usize;
    let valid_target = target.as_os_str().encode_wide().take(32768).all(|unit| {
        length += 1;
        unit != 0 && length <= 32767
    });
    let mut failed = ReceiptFailure {
        error: bad(),
        mutation: false,
        target_hint: if valid_target {
            target.to_owned()
        } else {
            PathBuf::new()
        },
        root: None,
        file: None,
    };
    let result = (|| -> io::Result<ReceiptRecord> {
        check(cancel, deadline)?;
        if !valid_target {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "receipt path bound",
            ));
        }
        registration.verify_current(cancel, deadline)?;
        // Conservative admission: reserve bounded metadata overhead before any creation.
        let nested =
            serde_json::to_vec(&(registration.record(), registration.seal())).map_err(|_| bad())?;
        if nested.len() > 65536 - 2048 {
            return Err(bad());
        }
        check(cancel, deadline)?;

        if !target.is_absolute()
            || target.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return Err(bad());
        }
        let parent = target.parent().ok_or_else(bad)?;
        // Retain the original immediate parent; never create against an unpinned resolved alias.
        let parent_file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(parent)?;
        let parent_id = file_identity::from_file(&parent_file)?;
        let parent_final = file_identity::final_path(&parent_file)?;
        if !parent_file.metadata()?.is_dir() || parent_file.metadata()?.file_type().is_symlink() {
            return Err(bad());
        }
        let registered_final = std::fs::canonicalize(registration.path())?;
        disjoint(
            &registered_final,
            &parent_final.join(target.file_name().ok_or_else(bad)?),
        )?;
        check(cancel, deadline)?;
        failed.mutation = true;
        failed.root = Some(Box::new(factory()?));
        hook("root-created")?;
        check(cancel, deadline)?;
        let root = failed.root.as_ref().ok_or_else(bad)?;
        root.verify()?;
        let actual = file_identity::final_path(root.file())?;
        if actual != parent_final.join(target.file_name().ok_or_else(bad)?) {
            return Err(bad());
        }
        disjoint(&registered_final, &actual)?;
        if file_identity::from_file(&parent_file)? != parent_id
            || file_identity::final_path(&parent_file)? != parent_final
        {
            return Err(bad());
        }
        failed.file = Some(Box::new(root.create_file("registration-receipt.json")?));
        hook("file-created")?;
        check(cancel, deadline)?;
        let file = failed.file.as_mut().ok_or_else(bad)?;
        let record = ReceiptRecord {
            schema: 1,
            domain: "dlpad-app-registration-receipt-v1".into(),
            receipt_root_identity: root.identity().clone(),
            receipt_identity: file.identity().clone(),
            registration: registration.record().clone(),
            seal: registration.seal().clone(),
        };
        record.validate()?;
        let bytes = serialize_receipt(&record)?;
        if bytes.len() > 65536 {
            return Err(bad());
        }
        check(cancel, deadline)?;
        file.file_mut().write_all(&bytes)?;
        hook("file-written")?;
        check(cancel, deadline)?;
        file.file().sync_all()?;
        hook("file-synced")?;
        check(cancel, deadline)?;
        registration.verify_current(cancel, deadline)?;
        root.verify()?;
        file.verify(root)?;
        namespace(root, cancel, deadline)?;
        if file_identity::from_file(&parent_file)? != parent_id
            || file_identity::final_path(&parent_file)? != parent_final
            || std::fs::canonicalize(parent)? != parent_final
        {
            return Err(bad());
        }
        check(cancel, deadline)?;
        Ok(record)
    })();
    match result {
        Ok(record) => {
            let receipt = FreshRegistrationReceipt {
                root: *failed
                    .root
                    .take()
                    .expect("root created before successful record"),
                file: *failed
                    .file
                    .take()
                    .expect("file created before successful record"),
                record,
            };
            if let Err(error) = receipt.verify_current(cancel, deadline) {
                return Err(ReceiptFailure {
                    error,
                    mutation: true,
                    target_hint: target.to_owned(),
                    root: Some(Box::new(receipt.root)),
                    file: Some(Box::new(receipt.file)),
                });
            }
            Ok(receipt)
        }
        Err(error) => {
            failed.error = error;
            Err(failed)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::time::Duration;
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn registered() -> (tempfile::TempDir, FreshRegistration) {
        let d = tempfile::tempdir().unwrap();
        let mut b = vec![0; 256];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        std::fs::write(d.path().join("source"), &b).unwrap();
        std::fs::write(d.path().join("sentinel"), b"outside").unwrap();
        let f = File::open(d.path().join("source")).unwrap();
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
        let p = super::super::prepare_new(
            &m,
            &[("dlpad.exe", &f)],
            &d.path().join("registration"),
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        let r = super::super::register_fresh(p, &AtomicBool::new(false), later()).unwrap();
        (d, r)
    }
    #[test]
    fn success_original_bound_receipt_drop_preserves_observation() {
        let (d, r) = registered();
        let target = d.path().join("receipt");
        let receipt =
            create_registration_receipt(&r, &target, &AtomicBool::new(false), later()).unwrap();
        receipt
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
        assert_eq!(
            receipt.record().receipt_root_identity,
            file_identity::from_file(receipt.root.file()).unwrap()
        );
        assert_eq!(
            receipt.record().receipt_identity,
            file_identity::from_file(receipt.file()).unwrap()
        );
        let expected = serialize_receipt(receipt.record()).unwrap();
        drop(receipt);
        assert_eq!(
            std::fs::read(target.join("registration-receipt.json")).unwrap(),
            expected
        );
        r.verify_current(&AtomicBool::new(false), later()).unwrap();
        assert_eq!(
            std::fs::read(d.path().join("sentinel")).unwrap(),
            b"outside"
        );
    }
    #[test]
    fn cancellation_expiry_and_overlap_are_before_creation() {
        let (d, r) = registered();
        for (cancel, end, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let target = d.path().join(format!("reject-{cancel}"));
            let err = create_registration_receipt(&r, &target, &AtomicBool::new(cancel), end)
                .unwrap_err();
            assert_eq!(err.kind(), kind);
            assert!(!err.mutation_may_have_occurred());
            assert!(!err.root_capability_acquired());
            assert_eq!(err.retained_file_cap_count(), 0);
            assert!(!target.exists());
        }
        let target = r.path().join("inside");
        let err =
            create_registration_receipt(&r, &target, &AtomicBool::new(false), later()).unwrap_err();
        assert!(!err.mutation_may_have_occurred());
        assert!(!target.exists());
        r.verify_current(&AtomicBool::new(false), later()).unwrap();
    }
    #[test]
    fn oversized_public_target_is_rejected_before_duplication_or_mutation() {
        let (_d, r) = registered();
        let path = PathBuf::from(format!("C:\\{}", "a".repeat(32768)));
        let err =
            create_registration_receipt(&r, &path, &AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(!err.mutation_may_have_occurred());
        assert_eq!(err.retained_file_cap_count(), 0);
        assert!(err.target_hint().as_os_str().is_empty());
    }
    #[test]
    fn existing_and_foreign_names_never_deleted() {
        let (d, r) = registered();
        let target = d.path().join("existing");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("foreign"), b"foreign").unwrap();
        let err =
            create_registration_receipt(&r, &target, &AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(err.retained_file_cap_count(), 0);
        assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"foreign");
        let receipt = create_registration_receipt(
            &r,
            &d.path().join("receipt"),
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        std::fs::write(receipt.path().join("foreign"), b"added").unwrap();
        assert!(
            receipt
                .verify_current(&AtomicBool::new(false), later())
                .is_err()
        );
        assert_eq!(
            std::fs::read(receipt.path().join("foreign")).unwrap(),
            b"added"
        );
        r.verify_current(&AtomicBool::new(false), later()).unwrap();
    }
    #[test]
    fn write_and_sync_boundary_fault_retains_original_created_caps() {
        for step in ["file-created", "file-written", "file-synced"] {
            let (d, r) = registered();
            let target = d.path().join("fault");
            let err = create_with_hook(
                &r,
                &target,
                &AtomicBool::new(false),
                later(),
                &mut |phase| {
                    if phase == step {
                        Err(io::Error::new(io::ErrorKind::WriteZero, "boundary fault"))
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::WriteZero);
            assert!(err.mutation_may_have_occurred());
            assert!(err.root_capability_acquired());
            assert_eq!(err.retained_file_cap_count(), 1);
            assert!(target.join("registration-receipt.json").exists());
            r.verify_current(&AtomicBool::new(false), later()).unwrap();
            drop(err);
            assert!(target.join("registration-receipt.json").exists());
        }
    }
}
