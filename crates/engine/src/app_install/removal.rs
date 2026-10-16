//! Explicit disposal of this process's original fresh capabilities only.
//! Accepted dispositions are not atomic deletion, secure erasure, or cold adoption.
use super::{
    DiscardFailure, DiscardOutcome, DiscardRemainder, PreparedBundle, payload::reading_pin,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "fresh discard verification failed",
    )
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "fresh discard cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "fresh discard deadline",
        ))
    } else {
        Ok(())
    }
}
fn namespace(r: &DiscardRemainder, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    check(cancel, deadline)?;
    r.root.verify()?;
    let mut expected: BTreeSet<&str> = r.files.iter().map(|f| f.name()).collect();
    if expected.len() != r.files.len() {
        return Err(bad());
    }
    if r.marker.is_some() && !expected.insert("install-ready.json") {
        return Err(bad());
    }
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(r.root.path())? {
        check(cancel, deadline)?;
        let entry = entry?;
        if actual.len() >= 65 || !entry.file_type()?.is_file() {
            return Err(bad());
        }
        let name = entry.file_name().into_string().map_err(|_| bad())?;
        if !actual.insert(name) {
            return Err(bad());
        }
    }
    if actual.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err(bad());
    }
    r.root.verify()?;
    check(cancel, deadline)
}
fn preflight(r: &DiscardRemainder, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    namespace(r, cancel, deadline)?;
    if r.root.identity() != &r.record.root_identity
        || r.record.schema != 1
        || r.record.domain != "dlpad-prepared-app-v1"
        || r.record.trusted_provenance
    {
        return Err(bad());
    }
    // Validate the complete immutable original record, never the shrinking remainder.
    super::manifest::BundleManifest {
        schema: r.record.schema,
        version: r.record.version.clone(),
        target: r.record.target.clone(),
        development_only: r.record.development_only,
        entries: r
            .record
            .entries
            .iter()
            .map(|entry| super::manifest::BundleEntry {
                name: entry.name.clone(),
                bytes: entry.bytes,
                sha256: entry.sha256.clone(),
            })
            .collect(),
    }
    .validate()?;
    check(cancel, deadline)?;
    for file in &r.files {
        check(cancel, deadline)?;
        file.verify(&r.root)?;
        let entry = r
            .record
            .entries
            .iter()
            .find(|e| e.name == file.name())
            .ok_or_else(bad)?;
        if file.identity() != &entry.identity
            || file.file().metadata()?.len() != entry.bytes
            || entry.bytes > 128 * 1024 * 1024
        {
            return Err(bad());
        }
        let mut input = reading_pin(file.file())?;
        input.seek(SeekFrom::Start(0))?;
        let mut left = entry.bytes;
        let mut hash = Sha256::new();
        let mut buf = [0u8; 65536];
        while left > 0 {
            check(cancel, deadline)?;
            let take = left.min(buf.len() as u64) as usize;
            let n = input.read(&mut buf[..take])?;
            if n == 0 {
                return Err(bad());
            }
            hash.update(&buf[..n]);
            left -= n as u64;
        }
        check(cancel, deadline)?;
        if input.read(&mut [0u8; 1])? != 0 || format!("{:x}", hash.finalize()) != entry.sha256 {
            return Err(bad());
        }
        file.verify(&r.root)?;
    }
    if let Some(marker) = &r.marker {
        check(cancel, deadline)?;
        marker.verify(&r.root)?;
        if marker.name() != "install-ready.json" {
            return Err(bad());
        }
        let expected = serde_json::to_vec(&r.record).map_err(|_| bad())?;
        if expected.len() > 65536 || marker.file().metadata()?.len() != expected.len() as u64 {
            return Err(bad());
        }
        let mut input = reading_pin(marker.file())?;
        input.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::with_capacity(expected.len());
        input.take(65537).read_to_end(&mut bytes)?;
        check(cancel, deadline)?;
        if bytes != expected {
            return Err(bad());
        }
        marker.verify(&r.root)?;
    }
    namespace(r, cancel, deadline)
}
pub fn discard_fresh(
    bundle: PreparedBundle,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<DiscardOutcome, DiscardFailure> {
    let r = DiscardRemainder {
        root: bundle.root,
        files: bundle.files,
        marker: Some(bundle.marker),
        record: bundle.record,
        accepted_file_dispositions: 0,
    };
    // Retry preflight independently verifies every remaining original capability.
    discard_remaining(r, cancel, deadline)
}
pub(super) fn discard_remaining(
    r: DiscardRemainder,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<DiscardOutcome, DiscardFailure> {
    discard_with_hook(r, cancel, deadline, &mut |_, _| Ok(()))
}
fn discard_with_hook(
    mut r: DiscardRemainder,
    cancel: &AtomicBool,
    deadline: Instant,
    hook: &mut dyn FnMut(&super::FreshRoot, usize) -> io::Result<()>,
) -> Result<DiscardOutcome, DiscardFailure> {
    let result = (|| -> io::Result<()> {
        preflight(&r, cancel, deadline)?;
        while !r.files.is_empty() {
            check(cancel, deadline)?;
            preflight(&r, cancel, deadline)?;
            r.files[0].mark_delete_original(&r.root)?;
            drop(r.files.remove(0));
            r.accepted_file_dispositions += 1;
            hook(&r.root, r.accepted_file_dispositions)?;
        }
        preflight(&r, cancel, deadline)?;
        check(cancel, deadline)?;
        if let Some(marker) = r.marker.as_ref() {
            marker.mark_delete_original(&r.root)?;
            drop(r.marker.take());
            r.accepted_file_dispositions += 1;
            hook(&r.root, r.accepted_file_dispositions)?;
        }
        preflight(&r, cancel, deadline)?;
        check(cancel, deadline)?;
        r.root.mark_delete_original()?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(DiscardOutcome {
            accepted_file_dispositions: r.accepted_file_dispositions,
            root_disposition_accepted: true,
        }),
        Err(error) => Err(DiscardFailure {
            error,
            remainder: Box::new(r),
        }),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self, File, OpenOptions},
        io::Write,
        os::windows::fs::OpenOptionsExt,
        time::Duration,
    };
    fn ready() -> (tempfile::TempDir, PreparedBundle, Vec<u8>) {
        let d = tempfile::tempdir().unwrap();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(d.path().join("source"), &bytes).unwrap();
        fs::write(d.path().join("sentinel"), b"outside").unwrap();
        let f = File::open(d.path().join("source")).unwrap();
        let m = super::super::manifest::BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            }],
        };
        let bundle = super::super::prepare_new(
            &m,
            &[("dlpad.exe", &f)],
            &d.path().join("fresh"),
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        (d, bundle, bytes)
    }
    fn observe(path: impl AsRef<std::path::Path>) -> Vec<u8> {
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(7)
            .open(path)
            .unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes
    }
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn remainder(b: PreparedBundle) -> DiscardRemainder {
        DiscardRemainder {
            root: b.root,
            files: b.files,
            marker: Some(b.marker),
            record: b.record,
            accepted_file_dispositions: 0,
        }
    }
    #[test]
    fn exact_fresh_discard_leaves_outside_sources_untouched() {
        let (d, b, bytes) = ready();
        let path = b.path().to_owned();
        let result = discard_fresh(b, &AtomicBool::new(false), later()).unwrap();
        assert_eq!(result.accepted_file_dispositions, 2);
        assert!(result.root_disposition_accepted);
        assert!(!path.exists());
        assert_eq!(observe(d.path().join("source")), bytes);
        assert_eq!(observe(d.path().join("sentinel")), b"outside");
    }
    #[test]
    fn foreign_or_changed_original_preflight_accepts_nothing() {
        let (_d, b, _) = ready();
        fs::write(b.path().join("foreign"), b"foreign").unwrap();
        let err = discard_fresh(b, &AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(err.accepted_file_dispositions(), 0);
        assert_eq!(observe(err.path_hint().join("foreign")), b"foreign");
        assert!(err.path_hint().join("dlpad.exe").exists());
        let (_d, mut b, _) = ready();
        b.files[0].file_mut().write_all(b"XX").unwrap();
        let err = discard_fresh(b, &AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(err.accepted_file_dispositions(), 0);
        assert!(err.path_hint().join("install-ready.json").exists());
    }
    #[test]
    fn interrupted_timed_out_and_io_failure_keep_retriable_original_remainder() {
        for kind in [
            io::ErrorKind::Interrupted,
            io::ErrorKind::TimedOut,
            io::ErrorKind::WriteZero,
        ] {
            let (_d, b, _) = ready();
            let path = b.path().to_owned();
            let err = discard_with_hook(
                remainder(b),
                &AtomicBool::new(false),
                later(),
                &mut |_, n| {
                    assert_eq!(n, 1);
                    Err(io::Error::new(kind, "injected boundary"))
                },
            )
            .unwrap_err();
            assert_eq!(err.kind(), kind);
            assert_eq!(err.accepted_file_dispositions(), 1);
            assert_eq!(err.retained_file_cap_count(), 1);
            assert!(!path.join("dlpad.exe").exists());
            assert!(path.join("install-ready.json").exists());
            let done = err.retry(&AtomicBool::new(false), later()).unwrap();
            assert_eq!(done.accepted_file_dispositions, 2);
            assert!(!path.exists());
        }
    }
    #[test]
    fn unknown_after_first_disposition_is_not_deleted() {
        let (_d, b, _) = ready();
        let err = discard_with_hook(
            remainder(b),
            &AtomicBool::new(false),
            later(),
            &mut |root, n| {
                if n == 1 {
                    fs::write(root.path().join("foreign"), b"new foreign")?;
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(err.accepted_file_dispositions(), 1);
        assert_eq!(observe(err.path_hint().join("foreign")), b"new foreign");
        assert!(err.path_hint().join("install-ready.json").exists());
        assert!(err.retry(&AtomicBool::new(false), later()).is_err());
    }
    #[test]
    fn actual_pending_reader_requires_original_reader_close_before_retry() {
        let (_d, b, _) = ready();
        let path = b.path().to_owned();
        let reader = OpenOptions::new()
            .read(true)
            .share_mode(7)
            .open(path.join("dlpad.exe"))
            .unwrap();
        let err = discard_fresh(b, &AtomicBool::new(false), later()).unwrap_err();
        assert_eq!(err.accepted_file_dispositions(), 1);
        assert!(path.join("install-ready.json").exists());
        drop(reader);
        let done = err.retry(&AtomicBool::new(false), later()).unwrap();
        assert_eq!(done.accepted_file_dispositions, 2);
        assert!(!path.exists());
    }
    #[test]
    fn original_root_rename_replacement_never_authorizes_foreign_deletion() {
        let (d, b, bytes) = ready();
        let path = b.path().to_owned();
        let moved = d.path().join("moved-original");
        let original = b.record.root_identity.clone();
        match fs::rename(&path, &moved) {
            Ok(()) => {
                fs::create_dir(&path).unwrap();
                fs::write(path.join("foreign"), b"replacement sentinel").unwrap();
                let err = discard_fresh(b, &AtomicBool::new(false), later()).unwrap_err();
                assert_eq!(err.accepted_file_dispositions(), 0);
                assert_eq!(err.retained_file_cap_count(), 2);
                assert_eq!(observe(path.join("foreign")), b"replacement sentinel");
                assert_eq!(observe(moved.join("dlpad.exe")), bytes);
                assert_eq!(
                    crate::file_identity::from_file(err.remainder.root.file()).unwrap(),
                    original
                );
                assert!(err.retry(&AtomicBool::new(false), later()).is_err());
            }
            Err(error) => {
                // A deny-delete-sharing kernel refusal is protection, not a performed replacement.
                assert!(
                    matches!(error.raw_os_error(), Some(5 | 32 | 33)),
                    "unexpected rename error kind {:?}",
                    error.kind()
                );
                assert_eq!(
                    crate::file_identity::from_file(b.root.file()).unwrap(),
                    original
                );
                b.verify_current().unwrap();
                fs::write(path.join("foreign"), b"unreplaced sentinel").unwrap();
                let err = discard_fresh(b, &AtomicBool::new(false), later()).unwrap_err();
                assert_eq!(err.accepted_file_dispositions(), 0);
                assert_eq!(observe(path.join("foreign")), b"unreplaced sentinel");
            }
        }
        assert_eq!(observe(d.path().join("source")), bytes);
    }
    #[test]
    fn preexisting_cancel_and_expired_deadline_preserve_all_caps() {
        for (cancel, deadline, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            let (_d, b, _) = ready();
            let err = discard_fresh(b, &AtomicBool::new(cancel), deadline).unwrap_err();
            assert_eq!(err.kind(), kind);
            assert_eq!(err.accepted_file_dispositions(), 0);
            assert_eq!(err.retained_file_cap_count(), 2);
            assert!(err.path_hint().join("dlpad.exe").exists());
        }
    }
}
