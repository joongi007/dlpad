//! Fresh prepared copies only; no activation, deletion or durable crash recovery.
use super::{
    FreshRoot, PrepareFailure, PreparedBundle, PreparedEntry, PreparedRecord,
    manifest::BundleManifest,
    payload::inspect_payload,
    windows_caps::{CreatedFile, pin_source},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "prepared app verification failed",
    )
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "app preparation cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "app preparation deadline",
        ))
    } else {
        Ok(())
    }
}
fn namespace(root: &FreshRoot, files: &[CreatedFile], marker: bool) -> io::Result<()> {
    root.verify()?;
    let mut expected: BTreeSet<String> = files.iter().map(|f| f.name().to_owned()).collect();
    if marker {
        expected.insert("install-ready.json".into());
    }
    let mut actual = BTreeSet::new();
    for e in std::fs::read_dir(root.path())? {
        let e = e?;
        if actual.len() >= 65 || !e.file_type()?.is_file() {
            return Err(bad());
        }
        let n = e.file_name().into_string().map_err(|_| bad())?;
        if !actual.insert(n) {
            return Err(bad());
        }
    }
    if actual != expected {
        return Err(bad());
    }
    root.verify()
}
pub fn prepare_new(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<PreparedBundle, PrepareFailure> {
    prepare_with_hook(
        manifest,
        files,
        target,
        cancel,
        deadline,
        &mut |_, _| Ok(()),
    )
}
fn prepare_with_hook(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(&FreshRoot, u64) -> io::Result<()>,
) -> Result<PreparedBundle, PrepareFailure> {
    prepare_with_factory_and_hook(
        manifest,
        files,
        target,
        cancel,
        deadline,
        &mut || FreshRoot::create_new(target),
        hook,
    )
}
pub(super) fn prepare_new_with_root_factory(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target_hint: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    mut factory: impl FnMut() -> io::Result<FreshRoot>,
) -> Result<PreparedBundle, PrepareFailure> {
    prepare_with_factory_and_hook(
        manifest,
        files,
        target_hint,
        cancel,
        deadline,
        &mut factory,
        &mut |_, _| Ok(()),
    )
}
fn prepare_with_factory_and_hook(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
    factory: &mut dyn FnMut() -> io::Result<FreshRoot>,
    hook: &mut dyn FnMut(&FreshRoot, u64) -> io::Result<()>,
) -> Result<PreparedBundle, PrepareFailure> {
    let mut failure = PrepareFailure {
        error: bad(),
        target_hint: None,
        root: None,
        files: Vec::new(),
    };
    let result = (|| -> io::Result<(CreatedFile, PreparedRecord)> {
        manifest.validate()?;
        check(cancel, deadline)?;
        if files.len() != manifest.entries.len() {
            return Err(bad());
        }
        let mut names = BTreeSet::new();
        let mut pins = Vec::new();
        for (n, f) in files {
            check(cancel, deadline)?;
            if !names.insert(*n) || !manifest.entries.iter().any(|e| e.name == *n) {
                return Err(bad());
            }
            pins.push((*n, pin_source(f)?));
        }
        let refs: Vec<_> = pins.iter().map(|(n, f)| (*n, f)).collect();
        inspect_payload(manifest, &refs, cancel, deadline)?;
        check(cancel, deadline)?;
        failure.target_hint = Some(target.to_owned());
        failure.root = Some(Box::new(factory()?));
        let created = failure.root.as_ref().ok_or_else(bad)?;
        created.verify()?;
        if created.path() != target {
            return Err(bad());
        }
        check(cancel, deadline)?;
        let root = failure.root.as_ref().ok_or_else(bad)?;
        for entry in &manifest.entries {
            check(cancel, deadline)?;
            let mut source = pins
                .iter()
                .find(|(n, _)| *n == entry.name)
                .ok_or_else(bad)?
                .1
                .try_clone()?;
            source.seek(SeekFrom::Start(0))?;
            failure.files.push(root.create_file(&entry.name)?);
            let output = failure.files.last_mut().ok_or_else(bad)?;
            let mut remaining = entry.bytes;
            let mut total = 0u64;
            let mut hash = Sha256::new();
            let mut buf = [0u8; 65536];
            while remaining > 0 {
                check(cancel, deadline)?;
                let take = remaining.min(buf.len() as u64) as usize;
                let n = source.read(&mut buf[..take])?;
                if n == 0 {
                    return Err(bad());
                }
                output.file_mut().write_all(&buf[..n])?;
                hash.update(&buf[..n]);
                remaining -= n as u64;
                total += n as u64;
                hook(root, total)?;
            }
            check(cancel, deadline)?;
            if source.read(&mut [0u8; 1])? != 0 || format!("{:x}", hash.finalize()) != entry.sha256
            {
                return Err(bad());
            }
            output.file().sync_all()?;
            check(cancel, deadline)?;
            output.verify(root)?;
        }
        namespace(root, &failure.files, false)?;
        let copied: Vec<_> = failure.files.iter().map(|f| (f.name(), f.file())).collect();
        let observations = inspect_payload(manifest, &copied, cancel, deadline)?;
        inspect_payload(manifest, &refs, cancel, deadline)?;
        let record = PreparedRecord {
            schema: 1,
            domain: "dlpad-prepared-app-v1".into(),
            root_identity: root.identity().clone(),
            version: manifest.version.clone(),
            target: manifest.target.clone(),
            development_only: manifest.development_only,
            trusted_provenance: false,
            entries: observations
                .into_iter()
                .map(|o| PreparedEntry {
                    name: o.name,
                    bytes: o.bytes,
                    sha256: o.sha256,
                    identity: o.identity,
                })
                .collect(),
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| bad())?;
        if bytes.len() > 65536 {
            return Err(bad());
        }
        check(cancel, deadline)?;
        failure.files.push(root.create_file("install-ready.json")?);
        let marker = failure.files.last_mut().ok_or_else(bad)?;
        marker.file_mut().write_all(&bytes)?;
        marker.file().sync_all()?;
        marker.verify(root)?;
        check(cancel, deadline)?;
        namespace(root, &failure.files, true)?;
        inspect_payload(manifest, &refs, cancel, deadline)?;
        namespace(root, &failure.files, true)?;
        check(cancel, deadline)?;
        let marker = failure.files.pop().ok_or_else(bad)?;
        Ok((marker, record))
    })();
    match result {
        Ok((marker, record)) => {
            let bundle = PreparedBundle {
                root: *failure.root.take().expect("successful root"),
                files: std::mem::take(&mut failure.files),
                marker,
                record,
            };
            Ok(bundle)
        }
        Err(error) => {
            failure.error = error;
            Err(failure)
        }
    }
}
pub(super) fn verify_current(bundle: &PreparedBundle) -> io::Result<()> {
    verify_current_controlled(
        bundle,
        &AtomicBool::new(false),
        Instant::now() + std::time::Duration::from_secs(30),
    )
}
pub(super) fn verify_current_controlled(
    bundle: &PreparedBundle,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<()> {
    check(cancel, deadline)?;
    bundle.root.verify()?;
    if bundle.root.identity() != &bundle.record.root_identity
        || bundle.record.schema != 1
        || bundle.record.domain != "dlpad-prepared-app-v1"
        || bundle.record.trusted_provenance
    {
        return Err(bad());
    }
    for f in &bundle.files {
        check(cancel, deadline)?;
        f.verify(&bundle.root)?;
    }
    bundle.marker.verify(&bundle.root)?;
    namespace(&bundle.root, &bundle.files, true)?;
    let manifest = BundleManifest {
        schema: 1,
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
    let refs: Vec<_> = bundle.files.iter().map(|f| (f.name(), f.file())).collect();
    let observations = inspect_payload(&manifest, &refs, cancel, deadline)?;
    for ((o, e), original) in observations
        .iter()
        .zip(&bundle.record.entries)
        .zip(&bundle.files)
    {
        if o.identity != e.identity || &o.identity != original.identity() {
            return Err(bad());
        }
    }
    let expected = serde_json::to_vec(&bundle.record).map_err(|_| bad())?;
    if expected.len() > 65536 {
        return Err(bad());
    }
    use std::os::windows::fs::FileExt;
    if bundle.marker.file().metadata()?.len() != expected.len() as u64 {
        return Err(bad());
    }
    let mut bytes = vec![0; expected.len()];
    let mut offset = 0;
    while offset < bytes.len() {
        check(cancel, deadline)?;
        let n = bundle
            .marker
            .file()
            .seek_read(&mut bytes[offset..], offset as u64)?;
        if n == 0 {
            return Err(bad());
        }
        offset += n;
    }
    if bytes != expected {
        return Err(bad());
    }
    namespace(&bundle.root, &bundle.files, true)?;
    check(cancel, deadline)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn original_prepared_verifier_uses_caller_cancellation_and_deadline() {
        let (temp, file, manifest) = setup();
        let target = temp.path().join("prepared");
        let c = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(30);
        let original = prepare_new(&manifest, &[("dlpad.exe", &file)], &target, &c, deadline)
            .ok()
            .unwrap();
        let before = std::fs::read(target.join("install-ready.json")).unwrap();
        assert_eq!(
            verify_current_controlled(&original, &AtomicBool::new(true), deadline)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(
            verify_current_controlled(&original, &c, Instant::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        verify_current_controlled(&original, &c, deadline).unwrap();
        assert_eq!(
            std::fs::read(target.join("install-ready.json")).unwrap(),
            before
        );
    }
    fn setup() -> (tempfile::TempDir, File, BundleManifest) {
        let d = tempfile::tempdir().unwrap();
        let mut b = vec![0; 70000];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let path = d.path().join("source");
        std::fs::write(&path, &b).unwrap();
        let file = File::open(path).unwrap();
        let m = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: b.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&b)),
            }],
        };
        (d, file, m)
    }
    #[test]
    fn copy_verify_and_cursor_preserve() {
        let (d, mut f, m) = setup();
        f.seek(SeekFrom::Start(7)).unwrap();
        let b = prepare_new(
            &m,
            &[("dlpad.exe", &f)],
            &d.path().join("새 설치"),
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(f.stream_position().unwrap(), 7);
        b.verify_current().unwrap();
        assert_ne!(
            b.record.entries[0].identity,
            crate::file_identity::from_file(&f).unwrap()
        );
        std::fs::write(b.path().join("foreign"), b"sentinel").unwrap();
        assert!(b.verify_current().is_err());
        assert_eq!(
            std::fs::read(b.path().join("foreign")).unwrap(),
            b"sentinel"
        );
    }
    #[test]
    fn failures_before_mutation_and_existing_target() {
        let (d, f, mut m) = setup();
        let p = d.path().join("target");
        m.entries[0].sha256 = "0".repeat(64);
        let e = prepare_new(
            &m,
            &[("dlpad.exe", &f)],
            &p,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap_err();
        assert!(!e.mutation_may_have_occurred() && !p.exists());
        let (d, f, m) = setup();
        let p = d.path().join("target");
        for (cancel, deadline, kind) in [
            (
                true,
                Instant::now() + Duration::from_secs(10),
                io::ErrorKind::Interrupted,
            ),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let e = prepare_new(
                &m,
                &[("dlpad.exe", &f)],
                &p,
                &AtomicBool::new(cancel),
                deadline,
            )
            .unwrap_err();
            assert_eq!(e.kind(), kind);
            assert!(!e.mutation_may_have_occurred());
        }
        std::fs::create_dir(&p).unwrap();
        std::fs::write(p.join("foreign"), b"preserve").unwrap();
        assert!(
            prepare_new(
                &m,
                &[("dlpad.exe", &f)],
                &p,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(10)
            )
            .is_err()
        );
        assert_eq!(std::fs::read(p.join("foreign")).unwrap(), b"preserve");
    }
    #[test]
    fn in_copy_cancel_and_foreign_marker_preserved() {
        let (d, f, m) = setup();
        let cancel = AtomicBool::new(false);
        let p = d.path().join("cancel");
        let e = prepare_with_hook(
            &m,
            &[("dlpad.exe", &f)],
            &p,
            &cancel,
            Instant::now() + Duration::from_secs(10),
            &mut |_, _| {
                cancel.store(true, Ordering::Release);
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::Interrupted);
        assert!(e.root_capability_acquired() && e.retained_file_cap_count() == 1);
        let (d, f, m) = setup();
        let p = d.path().join("foreign");
        let e = prepare_with_hook(
            &m,
            &[("dlpad.exe", &f)],
            &p,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
            &mut |root, _| {
                let path = root.path().join("install-ready.json");
                if !path.exists() {
                    std::fs::write(path, b"foreign").unwrap();
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert!(e.root_capability_acquired());
        assert_eq!(
            std::fs::read(p.join("install-ready.json")).unwrap(),
            b"foreign"
        );
    }
    #[test]
    fn after_chunk_io_failure_preserves_partial_bytes_and_source() {
        let (d, f, m) = setup();
        let source_before = std::fs::read(d.path().join("source")).unwrap();
        let target = d.path().join("injected-error");
        let error = prepare_with_hook(
            &m,
            &[("dlpad.exe", &f)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
            &mut |_, copied| {
                assert_eq!(copied, 65536);
                Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "injected after completed write",
                ))
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WriteZero);
        assert!(error.root_capability_acquired());
        assert_eq!(error.retained_file_cap_count(), 1);
        assert_eq!(
            std::fs::metadata(target.join("dlpad.exe")).unwrap().len(),
            65536
        );
        assert!(!target.join("install-ready.json").exists());
        assert_eq!(
            std::fs::read(d.path().join("source")).unwrap(),
            source_before
        );
    }
    #[test]
    fn changed_original_output_is_rejected_without_removal() {
        let (d, f, m) = setup();
        let target = d.path().join("changed-output");
        let mut bundle = prepare_new(
            &m,
            &[("dlpad.exe", &f)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        bundle.files[0]
            .file_mut()
            .seek(SeekFrom::Start(1000))
            .unwrap();
        bundle.files[0].file_mut().write_all(b"changed").unwrap();
        bundle.files[0].file().sync_all().unwrap();
        assert!(bundle.verify_current().is_err());
        assert!(target.join("dlpad.exe").exists());
        assert!(target.join("install-ready.json").exists());
        assert_eq!(
            std::fs::read(d.path().join("source")).unwrap()[1000..1007],
            [0; 7]
        );
    }

    #[test]
    fn root_factory_is_not_called_before_source_verification() {
        let (d, f, mut manifest) = setup();
        manifest.entries[0].sha256 = "0".repeat(64);
        let calls = std::cell::Cell::new(0);
        let target = d.path().join("never-created");
        let error = prepare_new_with_root_factory(
            &manifest,
            &[("dlpad.exe", &f)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
            || {
                calls.set(calls.get() + 1);
                FreshRoot::create_new(&target)
            },
        )
        .unwrap_err();
        assert_eq!(calls.get(), 0);
        assert!(!error.mutation_may_have_occurred());
        assert!(!target.exists());
    }
    #[test]
    fn original_child_factory_preserves_caller_cursor_and_parent() {
        let (d, mut f, manifest) = setup();
        f.seek(SeekFrom::Start(7)).unwrap();
        let outer = FreshRoot::create_new(&d.path().join("outer")).unwrap();
        let target = outer.path().join("registered");
        let bundle = prepare_new_with_root_factory(
            &manifest,
            &[("dlpad.exe", &f)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
            || outer.create_child("registered"),
        )
        .unwrap();
        assert_eq!(f.stream_position().unwrap(), 7);
        bundle.verify_current().unwrap();
        outer.verify().unwrap();
        assert_ne!(bundle.record.root_identity, *outer.identity());
    }
}
