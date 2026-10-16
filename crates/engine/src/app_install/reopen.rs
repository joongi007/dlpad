//! Explicit readonly inspection. Record claims grant no cleanup/adoption authority.
use super::{
    PreparedInspection, PreparedRecord,
    manifest::{BundleEntry, BundleManifest},
    payload::inspect_payload,
    windows_caps::ReadRoot,
};
use std::{
    collections::BTreeSet,
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "prepared inspection rejected")
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "prepared inspection cancelled",
        ))
    } else if Instant::now() >= d {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "prepared inspection deadline",
        ))
    } else {
        Ok(())
    }
}
fn marker_bytes(f: &std::fs::File, c: &AtomicBool, d: Instant) -> io::Result<Vec<u8>> {
    use std::os::windows::fs::FileExt;
    check(c, d)?;
    let len = f.metadata()?.len();
    if len > 65536 {
        return Err(bad());
    }
    let mut bytes = vec![0; len as usize];
    let mut n = 0;
    while n < bytes.len() {
        check(c, d)?;
        let count = f.seek_read(&mut bytes[n..], n as u64)?;
        if count == 0 {
            return Err(bad());
        }
        n += count;
    }
    let mut extra = [0];
    if f.seek_read(&mut extra, len)? != 0 || f.metadata()?.len() != len {
        return Err(bad());
    }
    check(c, d)?;
    Ok(bytes)
}
fn descriptor(record: &PreparedRecord) -> io::Result<BundleManifest> {
    if record.schema != 1 || record.domain != "dlpad-prepared-app-v1" || record.trusted_provenance {
        return Err(bad());
    }
    let manifest = BundleManifest {
        schema: 1,
        version: record.version.clone(),
        target: record.target.clone(),
        development_only: record.development_only,
        entries: record
            .entries
            .iter()
            .map(|e| BundleEntry {
                name: e.name.clone(),
                bytes: e.bytes,
                sha256: e.sha256.clone(),
            })
            .collect(),
    };
    manifest.validate()?;
    Ok(manifest)
}
fn namespace(i: &PreparedInspection, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    i.root.verify()?;
    let mut expected: BTreeSet<_> = i.record.entries.iter().map(|e| e.name.as_str()).collect();
    expected.insert("install-ready.json");
    let mut seen = BTreeSet::new();
    for e in std::fs::read_dir(i.root.path())? {
        check(c, d)?;
        let e = e?;
        if seen.len() >= 65 || !e.file_type()?.is_file() {
            return Err(bad());
        }
        let name = e.file_name().into_string().map_err(|_| bad())?;
        if !seen.insert(name) {
            return Err(bad());
        }
    }
    if seen.len() != expected.len() || seen.iter().any(|n| !expected.contains(n.as_str())) {
        return Err(bad());
    }
    i.root.verify()?;
    check(c, d)
}
pub fn inspect_prepared(
    path: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<PreparedInspection> {
    check(cancel, deadline)?;
    let root = ReadRoot::open_existing(path)?;
    check(cancel, deadline)?;
    let marker = root.open_file("install-ready.json")?;
    let bytes = marker_bytes(marker.file(), cancel, deadline)?;
    let record: PreparedRecord = serde_json::from_slice(&bytes).map_err(|_| bad())?;
    let manifest = descriptor(&record)?;
    if root.identity() != &record.root_identity {
        return Err(bad());
    }
    let mut files = Vec::with_capacity(manifest.entries.len());
    for e in &record.entries {
        check(cancel, deadline)?;
        let file = root.open_file(&e.name)?;
        if file.identity() != &e.identity {
            return Err(bad());
        }
        files.push(file);
    }
    let result = PreparedInspection {
        root,
        files,
        marker,
        record,
    };
    verify_current(&result, cancel, deadline)?;
    Ok(result)
}
pub(super) fn verify_current(
    i: &PreparedInspection,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    check(cancel, deadline)?;
    let manifest = descriptor(&i.record)?;
    i.root.verify()?;
    if i.root.identity() != &i.record.root_identity || i.files.len() != i.record.entries.len() {
        return Err(bad());
    }
    i.marker.verify(&i.root)?;
    let bytes = marker_bytes(i.marker.file(), cancel, deadline)?;
    let current: PreparedRecord = serde_json::from_slice(&bytes).map_err(|_| bad())?;
    if serde_json::to_vec(&current).map_err(|_| bad())?
        != serde_json::to_vec(&i.record).map_err(|_| bad())?
    {
        return Err(bad());
    }
    namespace(i, cancel, deadline)?;
    for (f, e) in i.files.iter().zip(&i.record.entries) {
        check(cancel, deadline)?;
        f.verify(&i.root)?;
        if f.identity() != &e.identity {
            return Err(bad());
        }
    }
    let refs: Vec<_> = i
        .files
        .iter()
        .zip(&i.record.entries)
        .map(|(f, e)| (e.name.as_str(), f.file()))
        .collect();
    let observations = inspect_payload(&manifest, &refs, cancel, deadline)?;
    for (o, e) in observations.iter().zip(&i.record.entries) {
        if o.identity != e.identity {
            return Err(bad());
        }
    }
    i.marker.verify(&i.root)?;
    if marker_bytes(i.marker.file(), cancel, deadline)? != bytes {
        return Err(bad());
    }
    namespace(i, cancel, deadline)
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{fs::File, time::Duration};
    fn prepared() -> (tempfile::TempDir, std::path::PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let mut b = vec![0; 256];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let src = d.path().join("source");
        std::fs::write(&src, &b).unwrap();
        let file = File::open(src).unwrap();
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: format!("{:x}", Sha256::digest(&b)),
            }],
        };
        let target = d.path().join("prepared");
        let value = super::super::prepare_new(
            &manifest,
            &[("dlpad.exe", &file)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        drop(value);
        (d, target)
    }
    fn inspect(p: &Path) -> io::Result<PreparedInspection> {
        inspect_prepared(
            p,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
    }
    #[test]
    fn natural_drop_reopens_and_extra_preserved() {
        let (_d, p) = prepared();
        let value = inspect(&p).unwrap();
        verify_current(
            &value,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        std::fs::write(p.join("foreign"), b"sentinel").unwrap();
        assert!(inspect(&p).is_err());
        assert_eq!(std::fs::read(p.join("foreign")).unwrap(), b"sentinel");
    }
    #[test]
    fn marker_unknown_oversize_and_wrong_sha_preserved() {
        for case in 0..3 {
            let (_d, p) = prepared();
            let path = p.join("install-ready.json");
            let mut j: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let bytes = match case {
                0 => {
                    j["unknown"] = true.into();
                    serde_json::to_vec(&j).unwrap()
                }
                1 => vec![b' '; 65537],
                _ => {
                    j["entries"][0]["sha256"] = "0".repeat(64).into();
                    serde_json::to_vec(&j).unwrap()
                }
            };
            std::fs::write(&path, &bytes).unwrap();
            assert!(inspect(&p).is_err());
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
    #[test]
    fn missing_payload_cancel_deadline() {
        let (_d, p) = prepared();
        let moved = p.join("moved.txt");
        std::fs::rename(p.join("dlpad.exe"), &moved).unwrap();
        assert!(inspect(&p).is_err());
        assert!(moved.exists());
        assert_eq!(
            inspect_prepared(
                &p,
                &AtomicBool::new(true),
                Instant::now() + Duration::from_secs(10)
            )
            .err()
            .unwrap()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(
            inspect_prepared(&p, &AtomicBool::new(false), Instant::now())
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::TimedOut
        );
    }
    #[test]
    fn replaced_same_bytes_identity_and_live_writer_rejected() {
        let (_d, p) = prepared();
        let payload = p.join("dlpad.exe");
        let original = std::fs::read(&payload).unwrap();
        std::fs::rename(&payload, _d.path().join("original-preserved")).unwrap();
        std::fs::write(&payload, &original).unwrap();
        assert!(inspect(&p).is_err());
        assert_eq!(
            std::fs::read(_d.path().join("original-preserved")).unwrap(),
            original
        );
        let (_d, p) = prepared();
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        };
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .open(p.join("dlpad.exe"))
            .unwrap();
        assert!(inspect(&p).is_err());
        drop(writer);
        assert!(inspect(&p).is_ok());
    }
    #[test]
    fn marker_schema_domain_trust_and_original_ids_rejected_without_mutation() {
        for field in [
            "schema",
            "domain",
            "trusted",
            "root_identity",
            "entry_identity",
        ] {
            let (_owned, path) = prepared();
            let payload_before = std::fs::read(path.join("dlpad.exe")).unwrap();
            let marker = path.join("install-ready.json");
            let mut value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&marker).unwrap()).unwrap();
            match field {
                "schema" => value["schema"] = 2.into(),
                "domain" => value["domain"] = "foreign-domain".into(),
                "trusted" => value["trusted_provenance"] = true.into(),
                // Valid wire format but definitely wrong OS-domain identity, not
                // merely invalid JSON. Root native observation still decides.
                "root_identity" => {
                    value["root_identity"] = serde_json::json!({"Unix":{"device":0,"inode":0}})
                }
                "entry_identity" => {
                    value["entries"][0]["identity"] =
                        serde_json::json!({"Unix":{"device":0,"inode":0}})
                }
                _ => unreachable!(),
            }
            let bytes = serde_json::to_vec(&value).unwrap();
            std::fs::write(&marker, &bytes).unwrap();
            assert!(inspect(&path).is_err(), "{field}");
            assert_eq!(
                std::fs::read(&marker).unwrap(),
                bytes,
                "{field} marker mutation"
            );
            assert_eq!(
                std::fs::read(path.join("dlpad.exe")).unwrap(),
                payload_before
            );
        }
    }
    #[test]
    fn unknown_namespace_refusal_preserves_all_original_and_foreign_bytes() {
        let (_owned, path) = prepared();
        let payload = std::fs::read(path.join("dlpad.exe")).unwrap();
        let marker = std::fs::read(path.join("install-ready.json")).unwrap();
        let foreign = path.join("user-media.mp4");
        std::fs::write(&foreign, b"synthetic foreign media").unwrap();
        assert!(inspect(&path).is_err());
        assert_eq!(std::fs::read(&foreign).unwrap(), b"synthetic foreign media");
        assert_eq!(std::fs::read(path.join("dlpad.exe")).unwrap(), payload);
        assert_eq!(
            std::fs::read(path.join("install-ready.json")).unwrap(),
            marker
        );
    }
    #[test]
    fn verify_preserves_callers_pinned_executable_cursor() {
        use std::io::{Seek, SeekFrom};
        let (_owned, path) = prepared();
        let inspection = inspect(&path).unwrap();
        let mut borrowed = inspection.executable();
        borrowed.seek(SeekFrom::Start(37)).unwrap();
        let identity = crate::file_identity::from_file(borrowed).unwrap();
        verify_current(
            &inspection,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(borrowed.stream_position().unwrap(), 37);
        assert_eq!(crate::file_identity::from_file(borrowed).unwrap(), identity);
    }
}
