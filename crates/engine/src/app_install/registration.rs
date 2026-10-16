//! Fresh-capability development registration, never cold adoption or deletion.
use super::registration_schema::{
    RegistrationRecord, RegistrationSeal, record_digest, serialize_record, serialize_seal,
    validate_binding,
};
use super::{
    PreparedBundle, manifest::BundleManifest, payload::inspect_payload, windows_caps::CreatedFile,
};
use sha2::{Digest, Sha256};
use std::os::windows::fs::FileExt;
use std::{
    collections::BTreeSet,
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "fresh registration verification failed",
    )
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "registration cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "registration deadline",
        ))
    } else {
        Ok(())
    }
}
fn exact_namespace(
    bundle: &PreparedBundle,
    extra: &[CreatedFile],
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    check(cancel, deadline)?;
    bundle.root.verify()?;
    let mut expected: BTreeSet<String> = bundle.files.iter().map(|f| f.name().to_owned()).collect();
    if expected.len() != bundle.files.len() || !expected.insert("install-ready.json".into()) {
        return Err(bad());
    }
    for f in extra {
        if !expected.insert(f.name().to_owned()) {
            return Err(bad());
        }
    }
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(bundle.root.path())? {
        check(cancel, deadline)?;
        let entry = entry?;
        if actual.len() >= 67
            || !entry.file_type()?.is_file()
            || !actual.insert(entry.file_name().into_string().map_err(|_| bad())?)
        {
            return Err(bad());
        }
    }
    if actual != expected {
        return Err(bad());
    }
    bundle.root.verify()?;
    check(cancel, deadline)
}
fn exact_bytes(
    file: &CreatedFile,
    bytes: &[u8],
    bundle: &PreparedBundle,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    check(cancel, deadline)?;
    file.verify(&bundle.root)?;
    if file.file().metadata()?.len() != bytes.len() as u64 {
        return Err(bad());
    }
    let mut actual = vec![0; bytes.len()];
    let mut offset = 0;
    while offset < actual.len() {
        check(cancel, deadline)?;
        let n = file
            .file()
            .seek_read(&mut actual[offset..], offset as u64)?;
        if n == 0 {
            return Err(bad());
        }
        offset += n;
    }
    if actual != bytes {
        return Err(bad());
    }
    file.verify(&bundle.root)?;
    check(cancel, deadline)
}
fn verify_prepared(
    bundle: &PreparedBundle,
    extra: &[CreatedFile],
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    exact_namespace(bundle, extra, cancel, deadline)?;
    if bundle.record.schema != 1
        || bundle.record.domain != "dlpad-prepared-app-v1"
        || bundle.record.trusted_provenance
        || !bundle.record.development_only
        || bundle.root.identity() != &bundle.record.root_identity
    {
        return Err(bad());
    }
    let manifest = BundleManifest {
        schema: bundle.record.schema,
        version: bundle.record.version.clone(),
        target: bundle.record.target.clone(),
        development_only: bundle.record.development_only,
        entries: bundle
            .record
            .entries
            .iter()
            .map(|e| super::manifest::BundleEntry {
                name: e.name.clone(),
                bytes: e.bytes,
                sha256: e.sha256.clone(),
            })
            .collect(),
    };
    for f in &bundle.files {
        check(cancel, deadline)?;
        f.verify(&bundle.root)?;
    }
    let refs: Vec<_> = bundle.files.iter().map(|f| (f.name(), f.file())).collect();
    let observations = inspect_payload(&manifest, &refs, cancel, deadline)?;
    for e in &bundle.record.entries {
        let f = bundle
            .files
            .iter()
            .find(|f| f.name() == e.name)
            .ok_or_else(bad)?;
        let o = observations
            .iter()
            .find(|o| o.name == e.name)
            .ok_or_else(bad)?;
        if f.identity() != &e.identity || o.identity != e.identity {
            return Err(bad());
        }
    }
    let bytes = serde_json::to_vec(&bundle.record).map_err(|_| bad())?;
    if bytes.len() > 65536 {
        return Err(bad());
    }
    exact_bytes(&bundle.marker, &bytes, bundle, cancel, deadline)?;
    for f in extra {
        check(cancel, deadline)?;
        f.verify(&bundle.root)?;
    }
    exact_namespace(bundle, extra, cancel, deadline)
}
pub struct FreshRegistration {
    bundle: PreparedBundle,
    files: Vec<CreatedFile>,
    record: RegistrationRecord,
    seal: RegistrationSeal,
}
impl FreshRegistration {
    /// Narrow fresh observer input: original creation caps remain retained by caller.
    pub(super) fn fresh_observation_root(
        &self,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<&super::windows_caps::FreshRoot> {
        self.verify_current(cancel, deadline)?;
        Ok(&self.bundle.root)
    }
    pub fn record(&self) -> &RegistrationRecord {
        &self.record
    }
    pub fn seal(&self) -> &RegistrationSeal {
        &self.seal
    }
    pub fn path(&self) -> &Path {
        self.bundle.path()
    }
    pub fn verify_current(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        verify_prepared(&self.bundle, &self.files, cancel, deadline)?;
        validate_binding(&self.record, &self.seal)?;
        if self.files.len() != 2
            || self.files[0].name() != "install-registration.json"
            || self.files[1].name() != "install-registration-seal.json"
            || self.files[0].identity() != &self.record.record_identity
            || self.files[1].identity() != &self.record.seal_identity
            || self.bundle.marker.identity() != &self.record.ready_identity
            || self.bundle.root.identity() != &self.record.root_identity
            || serde_json::to_vec(&self.bundle.record).map_err(|_| bad())?
                != serde_json::to_vec(&self.record.prepared).map_err(|_| bad())?
        {
            return Err(bad());
        }
        exact_bytes(
            &self.files[0],
            &serialize_record(&self.record)?,
            &self.bundle,
            cancel,
            deadline,
        )?;
        exact_bytes(
            &self.files[1],
            &serialize_seal(&self.seal)?,
            &self.bundle,
            cancel,
            deadline,
        )?;
        exact_namespace(&self.bundle, &self.files, cancel, deadline)
    }
}
pub struct RegistrationFailure {
    mutation_may_have_occurred: bool,
    error: io::Error,
    bundle: Box<PreparedBundle>,
    files: Vec<CreatedFile>,
}
impl RegistrationFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    /// Conservative flag set before the first native creation attempt.
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.mutation_may_have_occurred
    }
    /// Confirmed new registration caps, not all possibly created native objects.
    pub fn retained_file_cap_count(&self) -> usize {
        self.files.len()
    }
    pub fn path_hint(&self) -> &Path {
        self.bundle.path()
    }
}
impl std::fmt::Debug for RegistrationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationFailure")
            .field("kind", &self.kind())
            .field("retained_file_caps", &self.files.len())
            .finish()
    }
}
impl std::fmt::Display for RegistrationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fresh registration failed ({:?})", self.kind())
    }
}
impl std::error::Error for RegistrationFailure {}
pub fn register_fresh(
    bundle: PreparedBundle,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<FreshRegistration, RegistrationFailure> {
    register_with_hook(bundle, cancel, deadline, &mut |_| Ok(()))
}
fn register_with_hook(
    bundle: PreparedBundle,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(&str) -> io::Result<()>,
) -> Result<FreshRegistration, RegistrationFailure> {
    let mut files = Vec::new();
    let mut mutation_may_have_occurred = false;
    let result = (|| -> io::Result<(RegistrationRecord, RegistrationSeal)> {
        verify_prepared(&bundle, &files, cancel, deadline)?;
        mutation_may_have_occurred = true;
        files.push(bundle.root.create_file("install-registration.json")?);
        hook("record-created")?;
        check(cancel, deadline)?;
        files.push(bundle.root.create_file("install-registration-seal.json")?);
        hook("seal-created")?;
        check(cancel, deadline)?;
        let mut nonce = [0u8; 32];
        if unsafe {
            windows_sys::Win32::Security::Cryptography::BCryptGenRandom(
                std::ptr::null_mut(),
                nonce.as_mut_ptr(),
                32,
                windows_sys::Win32::Security::Cryptography::BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
            || nonce == [0; 32]
        {
            return Err(bad());
        }
        let nonce = nonce.iter().map(|b| format!("{b:02x}")).collect::<String>();
        let prepared_bytes = serde_json::to_vec(&bundle.record).map_err(|_| bad())?;
        if prepared_bytes.len() > 65536 {
            return Err(bad());
        }
        let record = RegistrationRecord {
            schema: 1,
            domain: "dlpad-app-registration-v1".into(),
            nonce: nonce.clone(),
            root_identity: bundle.root.identity().clone(),
            record_identity: files[0].identity().clone(),
            seal_identity: files[1].identity().clone(),
            ready_identity: bundle.marker.identity().clone(),
            prepared_sha256: format!("{:x}", Sha256::digest(&prepared_bytes)),
            prepared: bundle.record.clone(),
        };
        let seal = RegistrationSeal {
            schema: 1,
            domain: "dlpad-app-registration-seal-v1".into(),
            nonce,
            root_identity: record.root_identity.clone(),
            record_identity: record.record_identity.clone(),
            seal_identity: record.seal_identity.clone(),
            record_sha256: record_digest(&record)?,
        };
        validate_binding(&record, &seal)?;
        let rb = serialize_record(&record)?;
        let sb = serialize_seal(&seal)?;
        check(cancel, deadline)?;
        files[0].file_mut().write_all(&rb)?;
        hook("record-written")?;
        check(cancel, deadline)?;
        files[0].file().sync_all()?;
        hook("record-synced")?;
        check(cancel, deadline)?;
        files[1].file_mut().write_all(&sb)?;
        hook("seal-written")?;
        check(cancel, deadline)?;
        files[1].file().sync_all()?;
        hook("seal-synced")?;
        verify_prepared(&bundle, &files, cancel, deadline)?;
        exact_bytes(&files[0], &rb, &bundle, cancel, deadline)?;
        exact_bytes(&files[1], &sb, &bundle, cancel, deadline)?;
        exact_namespace(&bundle, &files, cancel, deadline)?;
        Ok((record, seal))
    })();
    match result {
        Ok((record, seal)) => Ok(FreshRegistration {
            bundle,
            files,
            record,
            seal,
        }),
        Err(error) => Err(RegistrationFailure {
            mutation_may_have_occurred,
            error,
            bundle: Box::new(bundle),
            files,
        }),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, time::Duration};
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn ready() -> (tempfile::TempDir, PreparedBundle) {
        ready_with_development(true)
    }
    fn ready_with_development(development_only: bool) -> (tempfile::TempDir, PreparedBundle) {
        let temp = tempfile::tempdir().unwrap();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        std::fs::write(temp.path().join("source"), &bytes).unwrap();
        std::fs::write(temp.path().join("sentinel"), b"preserve").unwrap();
        let source = File::open(temp.path().join("source")).unwrap();
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            }],
        };
        let bundle = super::super::prepare_new(
            &manifest,
            &[("dlpad.exe", &source)],
            &temp.path().join("prepared"),
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        (temp, bundle)
    }
    #[test]
    fn non_development_bundle_is_rejected_before_registration_creation() {
        let (temp, bundle) = ready_with_development(false);
        let path = bundle.path().to_owned();
        let before: Vec<_> = bundle
            .record
            .entries
            .iter()
            .map(|e| (e.name.clone(), std::fs::read(path.join(&e.name)).unwrap()))
            .collect();
        let marker = std::fs::read(path.join("install-ready.json")).unwrap();
        let failure = register_fresh(bundle, &AtomicBool::new(false), later())
            .err()
            .unwrap();
        assert_eq!(failure.kind(), io::ErrorKind::InvalidData);
        assert!(!failure.mutation_may_have_occurred());
        assert_eq!(failure.retained_file_cap_count(), 0);
        assert!(!path.join("install-registration.json").exists());
        assert!(!path.join("install-registration-seal.json").exists());
        for (name, bytes) in before {
            assert_eq!(std::fs::read(path.join(name)).unwrap(), bytes);
        }
        assert_eq!(
            std::fs::read(path.join("install-ready.json")).unwrap(),
            marker
        );
        assert_eq!(
            std::fs::read(temp.path().join("sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn cancellation_deadline_and_foreign_namespace_preserve_original_bundle() {
        for (cancel, deadline, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let (_t, b) = ready();
            let e = register_fresh(b, &AtomicBool::new(cancel), deadline)
                .err()
                .unwrap();
            assert_eq!(e.kind(), kind);
            assert_eq!(e.retained_file_cap_count(), 0);
            assert!(!e.mutation_may_have_occurred());
            assert!(e.path_hint().join("dlpad.exe").exists());
        }
        let (_t, b) = ready();
        std::fs::write(b.path().join("foreign"), b"preserve").unwrap();
        let e = register_fresh(b, &AtomicBool::new(false), later())
            .err()
            .unwrap();
        assert_eq!(e.retained_file_cap_count(), 0);
        assert_eq!(
            std::fs::read(e.path_hint().join("foreign")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn actual_registration_verifies_and_drop_preserves() {
        let (t, b) = ready();
        let registered = register_fresh(b, &AtomicBool::new(false), later()).unwrap();
        registered
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
        let path = registered.path().to_owned();
        drop(registered);
        assert!(path.join("install-registration.json").exists());
        assert!(path.join("install-registration-seal.json").exists());
        assert_eq!(
            std::fs::read(t.path().join("sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn stage_failures_preserve_created_caps_without_retry_authority() {
        for fail in [
            "record-created",
            "seal-created",
            "record-written",
            "record-synced",
            "seal-written",
            "seal-synced",
        ] {
            let (_t, b) = ready();
            let e = register_with_hook(b, &AtomicBool::new(false), later(), &mut |phase| {
                if phase == fail {
                    Err(io::Error::new(io::ErrorKind::WriteZero, "fixture boundary"))
                } else {
                    Ok(())
                }
            })
            .err()
            .unwrap();
            assert_eq!(e.kind(), io::ErrorKind::WriteZero);
            assert!(e.mutation_may_have_occurred());
            assert_eq!(
                e.retained_file_cap_count(),
                if fail == "record-created" { 1 } else { 2 }
            );
            assert!(e.path_hint().join("dlpad.exe").exists());
            assert!(e.path_hint().join("install-registration.json").exists());
        }
    }
    #[test]
    fn changed_original_is_rejected_before_new_creation() {
        let (_t, mut b) = ready();
        b.files[0].file_mut().write_all(b"XX").unwrap();
        let e = register_fresh(b, &AtomicBool::new(false), later())
            .err()
            .unwrap();
        assert_eq!(e.retained_file_cap_count(), 0);
        assert!(!e.path_hint().join("install-registration.json").exists());
    }
}
