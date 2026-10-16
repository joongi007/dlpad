//! Explicit caller-authorized removal of a currently verified registered pair.
//! No automatic discovery; external receipt is retained, never deleted.
use super::{
    manifest::{BundleEntry, BundleManifest},
    payload::{inspect_payload, reading_pin},
    receipt_reopen::ReceiptInspection,
    registration_pair::RegisteredPairInspection,
    registration_schema::{RegistrationRecord, RegistrationSeal, serialize_record, serialize_seal},
    windows_caps::{DeletionFile, DeletionRoot},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "registered removal rejected")
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "registered removal cancelled",
        ))
    } else if Instant::now() >= d {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "registered removal deadline",
        ))
    } else {
        Ok(())
    }
}
pub struct RegisteredRemoval {
    root: DeletionRoot,
    files: Vec<DeletionFile>,
    record: RegistrationRecord,
    seal: RegistrationSeal,
    receipt: ReceiptInspection,
    accepted: usize,
}
pub struct RemovalAuthorizationFailure {
    error: io::Error,
    root: Option<Box<DeletionRoot>>,
    files: Vec<DeletionFile>,
    receipt: Option<Box<ReceiptInspection>>,
}
impl RemovalAuthorizationFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn acquired_file_cap_count(&self) -> usize {
        self.files.len()
    }
    pub fn root_capability_acquired(&self) -> bool {
        self.root.is_some()
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        false
    }
}
impl std::fmt::Debug for RemovalAuthorizationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemovalAuthorizationFailure")
            .field("kind", &self.kind())
            .field("file_caps", &self.files.len())
            .field("receipt_retained", &self.receipt.is_some())
            .finish()
    }
}
impl std::fmt::Display for RemovalAuthorizationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "registered removal authorization failed ({:?})",
            self.kind()
        )
    }
}
impl std::error::Error for RemovalAuthorizationFailure {}
#[derive(Debug)]
pub struct RegisteredRemovalOutcome {
    pub accepted_file_dispositions: usize,
    pub root_disposition_accepted: bool,
}
pub struct RegisteredRemovalFailure {
    error: io::Error,
    remaining: Box<RegisteredRemoval>,
}
impl RegisteredRemovalFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn accepted_file_dispositions(&self) -> usize {
        self.remaining.accepted
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.remaining.files.len()
    }
    pub fn path_hint(&self) -> &Path {
        self.remaining.root.path()
    }
    pub fn retry(self, c: &AtomicBool, d: Instant) -> Result<RegisteredRemovalOutcome, Self> {
        self.remaining.remove(c, d)
    }
}
impl std::fmt::Debug for RegisteredRemovalFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredRemovalFailure")
            .field("kind", &self.kind())
            .field("accepted", &self.remaining.accepted)
            .finish()
    }
}
impl std::fmt::Display for RegisteredRemovalFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "registered removal incomplete ({:?})", self.kind())
    }
}
impl std::error::Error for RegisteredRemovalFailure {}
fn namespace(r: &RegisteredRemoval, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    r.root.verify()?;
    let expected: BTreeSet<_> = r.files.iter().map(|f| f.name()).collect();
    if expected.len() != r.files.len() {
        return Err(bad());
    }
    let mut actual = BTreeSet::new();
    for e in std::fs::read_dir(r.root.path())? {
        check(c, d)?;
        let e = e?;
        if actual.len() >= 67
            || !e.file_type()?.is_file()
            || !actual.insert(e.file_name().into_string().map_err(|_| bad())?)
        {
            return Err(bad());
        }
    }
    if actual.len() != expected.len() || actual.iter().any(|n| !expected.contains(n.as_str())) {
        return Err(bad());
    }
    r.root.verify()?;
    check(c, d)
}
fn immutable_manifest(r: &RegisteredRemoval) -> BundleManifest {
    let p = &r.record.prepared;
    BundleManifest {
        schema: 1,
        version: p.version.clone(),
        target: p.target.clone(),
        development_only: p.development_only,
        entries: p
            .entries
            .iter()
            .map(|e| BundleEntry {
                name: e.name.clone(),
                bytes: e.bytes,
                sha256: e.sha256.clone(),
            })
            .collect(),
    }
}
fn expected_bytes(r: &RegisteredRemoval, name: &str) -> io::Result<Option<Vec<u8>>> {
    match name {
        "install-ready.json" => Ok(Some(
            serde_json::to_vec(&r.record.prepared).map_err(|_| bad())?,
        )),
        "install-registration.json" => Ok(Some(serialize_record(&r.record)?)),
        "install-registration-seal.json" => Ok(Some(serialize_seal(&r.seal)?)),
        _ => Ok(None),
    }
}
fn preflight(r: &RegisteredRemoval, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    r.record.validate()?;
    r.seal.validate()?;
    super::registration_schema::validate_binding(&r.record, &r.seal)?;
    immutable_manifest(r).validate()?;
    r.receipt.verify_current(c, d)?;
    if serde_json::to_vec(&r.receipt.record().registration).map_err(|_| bad())?
        != serialize_record(&r.record)?
        || serde_json::to_vec(&r.receipt.record().seal).map_err(|_| bad())?
            != serialize_seal(&r.seal)?
        || r.root.identity() != &r.record.root_identity
    {
        return Err(bad());
    }
    namespace(r, c, d)?;
    if r.accepted == 0 {
        let payloads: Vec<_> = r
            .record
            .prepared
            .entries
            .iter()
            .map(|e| {
                let f = r
                    .files
                    .iter()
                    .find(|f| f.name() == e.name)
                    .ok_or_else(bad)?;
                Ok((e.name.as_str(), f.file()))
            })
            .collect::<io::Result<_>>()?;
        let observations = inspect_payload(&immutable_manifest(r), &payloads, c, d)?;
        for o in observations {
            let e = r
                .record
                .prepared
                .entries
                .iter()
                .find(|e| e.name == o.name)
                .ok_or_else(bad)?;
            if o.identity != e.identity {
                return Err(bad());
            }
        }
    }
    for f in &r.files {
        check(c, d)?;
        f.verify(&r.root)?;
        let (mut input, expected_len, expected_sha) =
            if let Some(bytes) = expected_bytes(r, f.name())? {
                if bytes.len() > 65536 {
                    return Err(bad());
                }
                (
                    reading_pin(f.file())?,
                    bytes.len() as u64,
                    format!("{:x}", Sha256::digest(bytes)),
                )
            } else {
                let e = r
                    .record
                    .prepared
                    .entries
                    .iter()
                    .find(|e| e.name == f.name())
                    .ok_or_else(bad)?;
                if f.identity() != &e.identity {
                    return Err(bad());
                }
                (reading_pin(f.file())?, e.bytes, e.sha256.clone())
            };
        let expected_id = match f.name() {
            "install-ready.json" => &r.record.ready_identity,
            "install-registration.json" => &r.record.record_identity,
            "install-registration-seal.json" => &r.record.seal_identity,
            _ => r
                .record
                .prepared
                .entries
                .iter()
                .find(|e| e.name == f.name())
                .map(|e| &e.identity)
                .ok_or_else(bad)?,
        };
        if f.identity() != expected_id
            || expected_len > 128 * 1024 * 1024
            || f.file().metadata()?.len() != expected_len
        {
            return Err(bad());
        }
        input.seek(SeekFrom::Start(0))?;
        let mut left = expected_len;
        let mut hash = Sha256::new();
        let mut buf = [0; 65536];
        while left > 0 {
            check(c, d)?;
            let n = input.read(&mut buf[..left.min(65536) as usize])?;
            if n == 0 {
                return Err(bad());
            }
            hash.update(&buf[..n]);
            left -= n as u64;
        }
        check(c, d)?;
        if input.read(&mut [0])? != 0 || format!("{:x}", hash.finalize()) != expected_sha {
            return Err(bad());
        }
        f.verify(&r.root)?;
    }
    namespace(r, c, d)?;
    r.receipt.verify_current(c, d)?;
    check(c, d)
}
pub fn authorize_explicit_registered_removal(
    pair: RegisteredPairInspection,
    c: &AtomicBool,
    d: Instant,
) -> Result<RegisteredRemoval, RemovalAuthorizationFailure> {
    let mut failure = RemovalAuthorizationFailure {
        error: bad(),
        root: None,
        files: Vec::new(),
        receipt: None,
    };
    let prepared = (|| -> io::Result<(RegistrationRecord, RegistrationSeal)> {
        check(c, d)?;
        pair.verify_current(c, d)?;
        check(c, d)?;
        let (registered, receipt) = pair.into_parts();
        let (root, record, seal) = registered.into_removal_inputs();
        failure.receipt = Some(Box::new(receipt));
        check(c, d)?;
        failure.root = Some(Box::new(DeletionRoot::from_read_root(
            root,
            &record.root_identity,
        )?));
        let root = failure.root.as_ref().ok_or_else(bad)?;
        for e in &record.prepared.entries {
            check(c, d)?;
            failure.files.push(root.open_file(&e.name, &e.identity)?);
        }
        for (name, id) in [
            ("install-ready.json", &record.ready_identity),
            ("install-registration.json", &record.record_identity),
            ("install-registration-seal.json", &record.seal_identity),
        ] {
            check(c, d)?;
            failure.files.push(root.open_file(name, id)?);
        }
        Ok((record, seal))
    })();
    match prepared {
        Err(error) => {
            failure.error = error;
            Err(failure)
        }
        Ok((record, seal)) => {
            let r = RegisteredRemoval {
                root: *failure.root.take().expect("root acquired"),
                files: std::mem::take(&mut failure.files),
                record,
                seal,
                receipt: *failure.receipt.take().expect("receipt acquired"),
                accepted: 0,
            };
            if let Err(error) = preflight(&r, c, d) {
                return Err(RemovalAuthorizationFailure {
                    error,
                    root: Some(Box::new(r.root)),
                    files: r.files,
                    receipt: Some(Box::new(r.receipt)),
                });
            }
            Ok(r)
        }
    }
}
impl RegisteredRemoval {
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn remove(
        self,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<RegisteredRemovalOutcome, RegisteredRemovalFailure> {
        self.remove_with_hook(c, d, &mut |_| Ok(()))
    }
    fn remove_with_hook(
        mut self,
        c: &AtomicBool,
        d: Instant,
        hook: &mut dyn FnMut(usize) -> io::Result<()>,
    ) -> Result<RegisteredRemovalOutcome, RegisteredRemovalFailure> {
        let result = (|| {
            preflight(&self, c, d)?;
            while !self.files.is_empty() {
                check(c, d)?;
                preflight(&self, c, d)?;
                self.files[0].mark_delete_original(&self.root)?;
                drop(self.files.remove(0));
                self.accepted += 1;
                hook(self.accepted)?;
            }
            preflight(&self, c, d)?;
            check(c, d)?;
            self.root.mark_delete_original()?;
            Ok(())
        })();
        match result {
            Ok(()) => Ok(RegisteredRemovalOutcome {
                accepted_file_dispositions: self.accepted,
                root_disposition_accepted: true,
            }),
            Err(error) => Err(RegisteredRemovalFailure {
                error,
                remaining: Box::new(self),
            }),
        }
    }
}

impl std::fmt::Debug for RegisteredRemoval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredRemoval")
            .field("remaining_caps", &self.files.len())
            .field("accepted", &self.accepted)
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use super::super::registration_pair::inspect_registered_with_receipt;
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{fs, fs::File, time::Duration};
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn produced() -> (
        tempfile::TempDir,
        std::path::PathBuf,
        std::path::PathBuf,
        Vec<u8>,
    ) {
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
        let cancel = AtomicBool::new(false);
        let reg = d.path().join("registered");
        let p =
            super::super::prepare_new(&m, &[("dlpad.exe", &f)], &reg, &cancel, later()).unwrap();
        let r = super::super::register_fresh(p, &cancel, later()).unwrap();
        let receipt = d.path().join("receipt");
        let proof =
            super::super::create_registration_receipt(&r, &receipt, &cancel, later()).unwrap();
        let raw = serde_json::to_vec(proof.record()).unwrap();
        drop(proof);
        drop(r);
        (d, reg, receipt, raw)
    }

    fn authority(reg: &Path, receipt: &Path) -> RegisteredRemoval {
        let c = AtomicBool::new(false);
        authorize_explicit_registered_removal(
            inspect_registered_with_receipt(reg, receipt, &c, later()).unwrap(),
            &c,
            later(),
        )
        .unwrap()
    }
    #[test]
    fn original_dispositions_and_drop_have_separate_authority() {
        let (d, reg, receipt, raw) = produced();
        drop(authority(&reg, &receipt));
        assert!(reg.join("dlpad.exe").exists());
        let out = authority(&reg, &receipt)
            .remove(&AtomicBool::new(false), later())
            .unwrap();
        assert_eq!(out.accepted_file_dispositions, 4);
        assert!(out.root_disposition_accepted);
        assert!(!reg.exists());
        assert_eq!(
            fs::read(receipt.join("registration-receipt.json")).unwrap(),
            raw
        );
        assert_eq!(fs::read(d.path().join("sentinel")).unwrap(), b"outside");
    }
    #[test]
    fn authorization_cancel_deadline_and_foreign_never_mutate() {
        let (_d, reg, receipt, raw) = produced();
        for (c, d, k) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let pair =
                inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), later())
                    .unwrap();
            let e =
                authorize_explicit_registered_removal(pair, &AtomicBool::new(c), d).unwrap_err();
            assert_eq!(e.kind(), k);
            assert!(!e.mutation_may_have_occurred());
        }
        let a = authority(&reg, &receipt);
        fs::write(reg.join("foreign"), b"foreign").unwrap();
        let e = a.remove(&AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(e.accepted_file_dispositions(), 0);
        assert_eq!(fs::read(reg.join("foreign")).unwrap(), b"foreign");
        assert_eq!(
            fs::read(receipt.join("registration-receipt.json")).unwrap(),
            raw
        );
    }
    #[test]
    fn original_caps_retry_after_post_disposition_fault() {
        let (_d, reg, receipt, _raw) = produced();
        let e = authority(&reg, &receipt)
            .remove_with_hook(&AtomicBool::new(false), later(), &mut |_| {
                Err(io::Error::other("fixture fault"))
            })
            .unwrap_err();
        assert_eq!(e.accepted_file_dispositions(), 1);
        assert_eq!(e.retained_file_cap_count(), 3);
        let out = e.retry(&AtomicBool::new(false), later()).unwrap();
        assert_eq!(out.accepted_file_dispositions, 4);
        assert!(!reg.exists());
    }
    #[test]
    fn external_share_all_reader_keeps_retired_namespace_until_close() {
        use std::os::windows::fs::OpenOptionsExt;
        let (_d, reg, receipt, _raw) = produced();
        let a = authority(&reg, &receipt);
        let reader = fs::OpenOptions::new()
            .read(true)
            .share_mode(7)
            .open(reg.join("dlpad.exe"))
            .unwrap();
        let e = a.remove(&AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(e.accepted_file_dispositions(), 1);
        assert_eq!(e.retained_file_cap_count(), 3);
        drop(reader);
        let out = e.retry(&AtomicBool::new(false), later()).unwrap();
        assert_eq!(out.accepted_file_dispositions, 4);
        assert!(!reg.exists());
    }
}
