//! Readonly registered development consistency, never cold deletion authority.
use super::{
    manifest::{BundleEntry, BundleManifest},
    payload::inspect_payload,
    registration_schema::{
        RegistrationRecord, RegistrationSeal, serialize_record, serialize_seal, validate_binding,
    },
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
    io::Error::new(io::ErrorKind::InvalidData, "registered inspection rejected")
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "registered inspection cancelled",
        ))
    } else if Instant::now() >= d {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "registered inspection deadline",
        ))
    } else {
        Ok(())
    }
}
fn bytes(f: &File, limit: usize, c: &AtomicBool, d: Instant) -> io::Result<Vec<u8>> {
    check(c, d)?;
    let len = f.metadata()?.len();
    if len == 0 || len > limit as u64 {
        return Err(bad());
    }
    let mut b = vec![0; len as usize];
    let mut n = 0;
    while n < b.len() {
        check(c, d)?;
        let k = f.seek_read(&mut b[n..], n as u64)?;
        if k == 0 {
            return Err(bad());
        }
        n += k;
    }
    if f.seek_read(&mut [0], len)? != 0 || f.metadata()?.len() != len {
        return Err(bad());
    }
    check(c, d)?;
    Ok(b)
}
pub struct RegisteredInspection {
    root: ReadRoot,
    files: Vec<ReadFile>,
    ready: ReadFile,
    record_file: ReadFile,
    seal_file: ReadFile,
    record: RegistrationRecord,
    seal: RegistrationSeal,
}
impl RegisteredInspection {
    pub(super) fn original_final_path(&self) -> io::Result<std::path::PathBuf> {
        self.root.original_final_path()
    }
    /// Internal explicit-removal handoff: release READ-only child shares while
    /// retaining the original root and immutable identities. This grants no DELETE access.
    pub(super) fn into_removal_inputs(self) -> (ReadRoot, RegistrationRecord, RegistrationSeal) {
        let Self {
            root,
            files,
            ready,
            record_file,
            seal_file,
            record,
            seal,
        } = self;
        drop((files, ready, record_file, seal_file));
        (root, record, seal)
    }
    pub fn record(&self) -> &RegistrationRecord {
        &self.record
    }
    pub fn seal(&self) -> &RegistrationSeal {
        &self.seal
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn executable(&self) -> &File {
        self.files
            .iter()
            .zip(&self.record.prepared.entries)
            .find(|(_, e)| e.name.eq_ignore_ascii_case("dlpad.exe"))
            .expect("validated manifest executable")
            .0
            .file()
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        check(c, d)?;
        validate_binding(&self.record, &self.seal)?;
        self.root.verify()?;
        if self.root.identity() != &self.record.root_identity
            || self.ready.identity() != &self.record.ready_identity
            || self.record_file.identity() != &self.record.record_identity
            || self.seal_file.identity() != &self.record.seal_identity
        {
            return Err(bad());
        }
        for f in [&self.ready, &self.record_file, &self.seal_file] {
            check(c, d)?;
            f.verify(&self.root)?;
        }
        if bytes(self.record_file.file(), 65536, c, d)? != serialize_record(&self.record)?
            || bytes(self.seal_file.file(), 4096, c, d)? != serialize_seal(&self.seal)?
            || bytes(self.ready.file(), 65536, c, d)?
                != serde_json::to_vec(&self.record.prepared).map_err(|_| bad())?
        {
            return Err(bad());
        }
        namespace(self, c, d)?;
        let p = &self.record.prepared;
        let m = BundleManifest {
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
        };
        m.validate()?;
        if self.files.len() != p.entries.len() {
            return Err(bad());
        }
        for (f, e) in self.files.iter().zip(&p.entries) {
            check(c, d)?;
            f.verify(&self.root)?;
            if f.identity() != &e.identity {
                return Err(bad());
            }
        }
        let refs: Vec<_> = self
            .files
            .iter()
            .zip(&p.entries)
            .map(|(f, e)| (e.name.as_str(), f.file()))
            .collect();
        let obs = inspect_payload(&m, &refs, c, d)?;
        for (o, e) in obs.iter().zip(&p.entries) {
            if o.identity != e.identity {
                return Err(bad());
            }
        }
        for f in &self.files {
            check(c, d)?;
            f.verify(&self.root)?;
        }
        for f in [&self.ready, &self.record_file, &self.seal_file] {
            check(c, d)?;
            f.verify(&self.root)?;
        }
        namespace(self, c, d)
    }
}
fn namespace(i: &RegisteredInspection, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    i.root.verify()?;
    let mut expected: BTreeSet<_> = i
        .record
        .prepared
        .entries
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    for n in [
        "install-ready.json",
        "install-registration.json",
        "install-registration-seal.json",
    ] {
        if !expected.insert(n) {
            return Err(bad());
        }
    }
    let mut seen = BTreeSet::new();
    for e in std::fs::read_dir(i.path())? {
        check(c, d)?;
        let e = e?;
        if seen.len() >= 67
            || !e.file_type()?.is_file()
            || !seen.insert(e.file_name().into_string().map_err(|_| bad())?)
        {
            return Err(bad());
        }
    }
    if seen.len() != expected.len() || seen.iter().any(|n| !expected.contains(n.as_str())) {
        return Err(bad());
    }
    i.root.verify()?;
    check(c, d)
}
pub fn inspect_registered(
    target: &Path,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<RegisteredInspection> {
    check(c, d)?;
    let root = ReadRoot::open_existing(target)?;
    check(c, d)?;
    let record_file = root.open_file("install-registration.json")?;
    check(c, d)?;
    let seal_file = root.open_file("install-registration-seal.json")?;
    check(c, d)?;
    let ready = root.open_file("install-ready.json")?;
    let rb = bytes(record_file.file(), 65536, c, d)?;
    let sb = bytes(seal_file.file(), 4096, c, d)?;
    let record = RegistrationRecord::read(rb.as_slice())?;
    let seal = RegistrationSeal::read(sb.as_slice())?;
    validate_binding(&record, &seal)?;
    if rb != serialize_record(&record)? || sb != serialize_seal(&seal)? {
        return Err(bad());
    }
    let mut files = Vec::with_capacity(record.prepared.entries.len());
    for e in &record.prepared.entries {
        check(c, d)?;
        files.push(root.open_file(&e.name)?);
    }
    let result = RegisteredInspection {
        root,
        files,
        ready,
        record_file,
        seal_file,
        record,
        seal,
    };
    result.verify_current(c, d)?;
    Ok(result)
}

/// Private fresh observer; cold public inspection keeps restrictive shares unchanged.
pub(super) fn inspect_fresh_registration(
    fresh: &super::FreshRegistration,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<RegisteredInspection> {
    check(c, d)?;
    let original = fresh.fresh_observation_root(c, d)?;
    let root = ReadRoot::observe_fresh(original)?;
    let record = fresh.record().clone();
    let seal = fresh.seal().clone();
    // Immutable expected identities come from verified original creation caps, not observed opens.
    let record_file =
        root.open_file_for_fresh_observation("install-registration.json", &record.record_identity)?;
    check(c, d)?;
    let seal_file = root
        .open_file_for_fresh_observation("install-registration-seal.json", &record.seal_identity)?;
    check(c, d)?;
    let ready =
        root.open_file_for_fresh_observation("install-ready.json", &record.ready_identity)?;
    let mut files = Vec::with_capacity(record.prepared.entries.len());
    for entry in &record.prepared.entries {
        check(c, d)?;
        files.push(root.open_file_for_fresh_observation(&entry.name, &entry.identity)?);
    }
    let inspection = RegisteredInspection {
        root,
        files,
        ready,
        record_file,
        seal_file,
        record,
        seal,
    };
    // Exactly reuse strict namespace, canonical serializers, payload size/hash/ID policy.
    inspection.verify_current(c, d)?;
    fresh.verify_current(c, d)?;
    inspection.verify_current(c, d)?;
    Ok(inspection)
}
#[cfg(test)]
mod tests {
    use super::super::{PreparedBundle, registration::register_fresh};
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{
        io::{Seek, SeekFrom},
        time::Duration,
    };
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
    fn fresh_drop_reopens_cursor_and_live_caps_are_busy() {
        let (_t, b) = ready();
        let registered = register_fresh(b, &AtomicBool::new(false), later()).unwrap();
        let path = registered.path().to_owned();
        assert_eq!(
            inspect_registered(&path, &AtomicBool::new(false), later())
                .err()
                .unwrap()
                .raw_os_error(),
            Some(32)
        );
        drop(registered);
        let i = inspect_registered(&path, &AtomicBool::new(false), later()).unwrap();
        let mut f = i.executable();
        f.seek(SeekFrom::Start(37)).unwrap();
        i.verify_current(&AtomicBool::new(false), later()).unwrap();
        assert_eq!(f.stream_position().unwrap(), 37);
    }
    #[test]
    fn cancelled_deadline_and_extra_preserve() {
        let (_t, b) = ready();
        let r = register_fresh(b, &AtomicBool::new(false), later()).unwrap();
        let path = r.path().to_owned();
        drop(r);
        for (c, d, k) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            assert_eq!(
                inspect_registered(&path, &AtomicBool::new(c), d)
                    .err()
                    .unwrap()
                    .kind(),
                k
            );
        }
        std::fs::write(path.join("foreign"), b"preserve").unwrap();
        assert!(inspect_registered(&path, &AtomicBool::new(false), later()).is_err());
        assert_eq!(std::fs::read(path.join("foreign")).unwrap(), b"preserve");
    }
    #[test]
    fn changed_record_fields_and_bytes_rejected_without_mutation() {
        for field in ["schema", "trusted", "nonce", "unknown", "payload"] {
            let (_t, b) = ready();
            let r = register_fresh(b, &AtomicBool::new(false), later()).unwrap();
            let path = r.path().to_owned();
            drop(r);
            let file = path.join("install-registration.json");
            if field == "payload" {
                std::fs::write(path.join("dlpad.exe"), b"changed").unwrap();
            } else {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
                match field {
                    "schema" => value["schema"] = 2.into(),
                    "trusted" => value["prepared"]["trusted_provenance"] = true.into(),
                    "nonce" => value["nonce"] = "0".repeat(64).into(),
                    _ => value["unknown"] = true.into(),
                };
                std::fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            let before = std::fs::read(&file).unwrap();
            assert!(inspect_registered(&path, &AtomicBool::new(false), later()).is_err());
            assert_eq!(std::fs::read(&file).unwrap(), before);
        }
    }
    #[test]
    fn exact_same_bytes_replacements_keep_original_objects_and_reject_new_ids() {
        for name in [
            "dlpad.exe",
            "install-ready.json",
            "install-registration.json",
            "install-registration-seal.json",
        ] {
            let (temp, bundle) = ready();
            let registered = register_fresh(bundle, &AtomicBool::new(false), later()).unwrap();
            let path = registered.path().to_owned();
            drop(registered);
            let target = path.join(name);
            let original_file = crate::file_identity::open_regular_no_reparse(&target).unwrap();
            let original_id = crate::file_identity::from_file(&original_file).unwrap();
            let original_bytes = std::fs::read(&target).unwrap();
            let saved = temp.path().join("old-file");
            std::fs::rename(&target, &saved).unwrap();
            use std::io::Write;
            use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
            use windows_sys::Win32::{
                Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
                Storage::FileSystem::{
                    CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OPEN_REPARSE_POINT,
                    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
                },
            };
            let sid = crate::windows_private_security::current_sid().unwrap();
            let descriptor = crate::windows_private_security::PrivateDescriptor::new(&sid).unwrap();
            let mut units: Vec<u16> = target.as_os_str().encode_wide().collect();
            assert!(units.len() < 32767 && !units.contains(&0));
            units.push(0);
            // New owned fixture object only. Original private descriptor and
            // bounded NUL-terminated path remain live through CreateFileW.
            let raw = unsafe {
                CreateFileW(
                    units.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    &descriptor.attributes(),
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
                    std::ptr::null_mut(),
                )
            };
            assert!(
                raw != INVALID_HANDLE_VALUE && !raw.is_null(),
                "fixture private creation failed: {:?}",
                io::Error::last_os_error()
            );
            let mut replacement = unsafe { File::from_raw_handle(raw) };
            replacement.write_all(&original_bytes).unwrap();
            replacement.sync_all().unwrap();
            crate::windows_private_security::verify_acl(&replacement, &sid).unwrap();
            drop(replacement);
            let replacement_file = crate::file_identity::open_regular_no_reparse(&target).unwrap();
            let replacement_id = crate::file_identity::from_file(&replacement_file).unwrap();
            assert_ne!(original_id, replacement_id);
            assert_eq!(
                crate::file_identity::from_file(&original_file).unwrap(),
                original_id
            );
            assert_eq!(std::fs::read(&target).unwrap(), original_bytes);
            assert_eq!(std::fs::read(&saved).unwrap(), original_bytes);
            assert_eq!(std::fs::read_dir(&path).unwrap().count(), 4);
            // ACL was verified on the original read/write creation HANDLE;
            // the identity-only observer lacks READ_CONTROL by design.
            let root = ReadRoot::open_existing(&path).unwrap();
            let read = root.open_file(name).unwrap();
            assert_eq!(read.identity(), &replacement_id);
            assert_eq!(
                inspect_registered(&path, &AtomicBool::new(false), later())
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(std::fs::read(&target).unwrap(), original_bytes);
            assert_eq!(std::fs::read(&saved).unwrap(), original_bytes);
        }
    }
}
