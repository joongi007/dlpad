//! Worker-only explicit development preparation. No launch, cleanup or historical authority.
#![cfg(windows)]
use super::{
    create_journal_aware_bundle, inspect_install_journal, inspect_user_bundle,
    manifest::BundleManifest,
    payload::{inspect_payload, reading_pin},
};
use crate::file_identity::{self, NativeFileIdentity};
use std::os::windows::{
    ffi::OsStrExt,
    fs::OpenOptionsExt,
    io::{AsRawHandle, FromRawHandle},
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT,
            FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
        },
    },
    Win32::{
        Foundation::{GENERIC_READ, OBJ_CASE_INSENSITIVE, RtlNtStatusToDosError, UNICODE_STRING},
        Storage::FileSystem::*,
        System::IO::IO_STATUS_BLOCK,
    },
};
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(a: *const u16, al: i32, b: *const u16, bl: i32, ignore: i32) -> i32;
}
pub struct LocalJournalRequest {
    pub manifest: PathBuf,
    pub source_dir: PathBuf,
    pub bundle_root: PathBuf,
    pub journal_root: PathBuf,
}
/// Two caller-explicit roots for readonly verification; no source payload inputs are required.
pub struct LocalInspectionRequest {
    pub bundle_root: PathBuf,
    pub journal_root: PathBuf,
}
/// Presence means a fully acquired returned service/coordinator capability, not a count or authority after Drop.
pub struct LocalJournalReport {
    pub kind: Option<io::ErrorKind>,
    pub mutation_may_have_occurred: bool,
    pub bundle_capabilities_present: bool,
    pub journal_capabilities_present: bool,
    pub current_consistency_verified: bool,
    pub entries: Option<usize>,
}
impl LocalJournalReport {
    fn empty() -> Self {
        Self {
            kind: None,
            mutation_may_have_occurred: false,
            bundle_capabilities_present: false,
            journal_capabilities_present: false,
            current_consistency_verified: false,
            entries: None,
        }
    }
}
fn rejected() -> io::Error {
    io::ErrorKind::InvalidInput.into()
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= d {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
pub(super) fn validate(p: &Path) -> io::Result<()> {
    use std::path::{Component, Prefix};
    let units: Vec<_> = p.as_os_str().encode_wide().collect();
    String::from_utf16(&units).map_err(|_| rejected())?;
    if units.is_empty()
        || units.len() > 32767
        || units.iter().any(|v| *v < 32 || *v == 127)
        || !p.is_absolute()
        || !matches!(p.components().next(),Some(Component::Prefix(v)) if matches!(v.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        || p.components()
            .any(|v| matches!(v, Component::ParentDir | Component::CurDir))
    {
        return Err(rejected());
    }
    Ok(())
}
fn contains(a: &Path, b: &Path) -> io::Result<bool> {
    let a: Vec<_> = a.as_os_str().encode_wide().collect();
    let b: Vec<_> = b.as_os_str().encode_wide().collect();
    if a.len() > b.len() {
        return Ok(false);
    } // SAFETY: both bounded UTF16 buffers remain live for the call.
    let result =
        unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result == 2 && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92)))
}
fn bounded_manifest(file: &File, c: &AtomicBool, d: Instant) -> io::Result<BundleManifest> {
    let mut reader = reading_pin(file)?;
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        check(c, d)?;
        let len = reader.read(&mut buffer)?;
        check(c, d)?;
        if len == 0 {
            break;
        }
        if bytes.len() + len > 65536 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        bytes.extend_from_slice(&buffer[..len]);
    }
    BundleManifest::read(bytes.as_slice())
}
struct Pin {
    path: PathBuf,
    file: File,
    id: NativeFileIdentity,
    final_path: PathBuf,
}
impl Pin {
    fn verify(&self) -> io::Result<()> {
        if file_identity::from_file(&self.file)? != self.id
            || file_identity::final_path(&self.file)? != self.final_path
        {
            return Err(rejected());
        }
        let current = directory(&self.path)?;
        if file_identity::from_file(&current)? != self.id
            || file_identity::final_path(&current)? != self.final_path
        {
            return Err(rejected());
        }
        Ok(())
    }
}
fn directory(p: &Path) -> io::Result<File> {
    let f = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: live file HANDLE and correctly sized output buffer.
    if unsafe {
        GetFileInformationByHandleEx(
            f.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
    {
        return Err(rejected());
    }
    Ok(f)
}
fn ancestors(path: &Path, cancel: &AtomicBool, deadline: Instant) -> io::Result<Vec<Pin>> {
    let paths: Vec<_> = path.ancestors().collect();
    if paths.len() > 256 {
        return Err(rejected());
    }
    let mut pins = Vec::new();
    for p in paths.into_iter().rev() {
        check(cancel, deadline)?;
        let f = directory(p)?;
        let id = file_identity::from_file(&f)?;
        let final_path = file_identity::final_path(&f)?;
        pins.push(Pin {
            path: p.to_owned(),
            file: f,
            id,
            final_path,
        });
    }
    Ok(pins)
}
fn source(path: &Path, pins: &[Pin], cancel: &AtomicBool, deadline: Instant) -> io::Result<File> {
    check(cancel, deadline)?;
    for pin in pins {
        check(cancel, deadline)?;
        pin.verify()?;
    }
    let parent = pins.last().ok_or_else(rejected)?;
    let leaf = path.file_name().ok_or_else(rejected)?;
    let mut units: Vec<u16> = leaf.encode_wide().collect();
    if units.is_empty()
        || units.len() > 32766
        || units
            .iter()
            .any(|v| *v < 32 || *v == 127 || matches!(*v, 58 | 47 | 92))
        || matches!(units.last(), Some(32 | 46))
    {
        return Err(rejected());
    }
    let length = u16::try_from(units.len() * 2).map_err(|_| rejected())?;
    let mut name = UNICODE_STRING {
        Length: length,
        MaximumLength: length,
        Buffer: units.as_mut_ptr(),
    };
    let attrs = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.file.as_raw_handle(),
        ObjectName: &mut name,
        Attributes: OBJ_CASE_INSENSITIVE,
        ..Default::default()
    };
    let mut status = IO_STATUS_BLOCK::default();
    let mut raw = std::ptr::null_mut();
    // SAFETY: borrowed parent HANDLE and bounded live UTF16 buffers; output transferred once.
    let result = unsafe {
        NtCreateFile(
            &mut raw,
            GENERIC_READ | SYNCHRONIZE,
            &attrs,
            &mut status,
            std::ptr::null(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ,
            FILE_OPEN,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
            std::ptr::null(),
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            // SAFETY: converts the returned status value without pointer access.
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(rejected());
    }
    // SAFETY: successful NtCreateFile returned a unique valid owned handle.
    let file = unsafe { File::from_raw_handle(raw) };
    let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: live file HANDLE and correctly sized output buffer.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
        || !file.metadata()?.is_file()
    {
        return Err(rejected());
    }
    let parent = pins.last().ok_or_else(rejected)?;
    if file_identity::final_path(&file)?
        != parent
            .final_path
            .join(path.file_name().ok_or_else(rejected)?)
    {
        return Err(rejected());
    }
    for pin in pins {
        check(cancel, deadline)?;
        pin.verify()?;
    }
    Ok(file)
}

pub fn prepare_local_journal_bundle(
    r: &LocalJournalRequest,
    c: &AtomicBool,
    d: Instant,
) -> LocalJournalReport {
    let mut report = LocalJournalReport::empty();
    let mut work = || -> io::Result<()> {
        check(c, d)?;
        for p in [&r.manifest, &r.source_dir, &r.bundle_root, &r.journal_root] {
            validate(p)?;
        }
        let mp = ancestors(r.manifest.parent().ok_or_else(rejected)?, c, d)?;
        let mf = source(&r.manifest, &mp, c, d)?;
        let m = bounded_manifest(&mf, c, d)?;
        if !m.development_only {
            return Err(rejected());
        }
        let pp = ancestors(&r.source_dir, c, d)?;
        let bp = ancestors(r.bundle_root.parent().ok_or_else(rejected)?, c, d)?;
        let jp = ancestors(r.journal_root.parent().ok_or_else(rejected)?, c, d)?;
        let src = &pp.last().ok_or_else(rejected)?.final_path;
        let manifest_path = file_identity::final_path(&mf)?;
        let manifest_id = file_identity::from_file(&mf)?;
        let dest = bp
            .last()
            .ok_or_else(rejected)?
            .final_path
            .join(r.bundle_root.file_name().ok_or_else(rejected)?);
        let journal = jp
            .last()
            .ok_or_else(rejected)?
            .final_path
            .join(r.journal_root.file_name().ok_or_else(rejected)?);
        for target in [&dest, &journal] {
            if contains(src, target)? || contains(target, src)? || contains(target, &manifest_path)?
            {
                return Err(rejected());
            }
        }
        if contains(&dest, &journal)? || contains(&journal, &dest)? {
            return Err(rejected());
        }
        let mut files = Vec::with_capacity(m.entries.len());
        for e in &m.entries {
            check(c, d)?;
            files.push((
                e.name.as_str(),
                source(&r.source_dir.join(&e.name), &pp, c, d)?,
            ));
        }
        let refs: Vec<_> = files.iter().map(|(name, file)| (*name, file)).collect();
        let original = inspect_payload(&m, &refs, c, d)?;
        let verify = || -> io::Result<()> {
            for pin in mp.iter().chain(pp.iter()).chain(bp.iter()).chain(jp.iter()) {
                check(c, d)?;
                pin.verify()?;
            }
            if file_identity::from_file(&mf)? != manifest_id
                || file_identity::final_path(&mf)? != manifest_path
            {
                return Err(rejected());
            }
            let now = inspect_payload(&m, &refs, c, d)?;
            if now.len() != original.len() {
                return Err(rejected());
            }
            for old in &original {
                let current = now
                    .iter()
                    .find(|v| v.name == old.name)
                    .ok_or_else(rejected)?;
                if current.identity != old.identity
                    || current.bytes != old.bytes
                    || current.sha256 != old.sha256
                {
                    return Err(rejected());
                }
                let f = refs
                    .iter()
                    .find(|(name, _)| *name == old.name)
                    .ok_or_else(rejected)?
                    .1;
                if file_identity::final_path(f)? != src.join(&old.name) {
                    return Err(rejected());
                }
            }
            check(c, d)
        };
        verify()?;
        let created =
            match create_journal_aware_bundle(&m, &refs, &r.bundle_root, &r.journal_root, c, d) {
                Ok(v) => v,
                Err(e) => {
                    report.mutation_may_have_occurred = e.mutation_may_have_occurred();
                    report.bundle_capabilities_present = e.bundle_capabilities_present();
                    report.journal_capabilities_present = e.journal_capabilities_present();
                    return Err(e.kind().into());
                }
            };
        report.mutation_may_have_occurred = true;
        report.bundle_capabilities_present = true;
        report.journal_capabilities_present = true;
        created.verify_current(c, d)?;
        verify()?;
        report.entries = Some(
            created
                .bundle()
                .registration()
                .record()
                .prepared
                .entries
                .len(),
        );
        report.current_consistency_verified = true;
        Ok(())
    };
    if let Err(e) = work() {
        report.kind = Some(e.kind());
    }
    report
}
pub fn inspect_local_journal_bundle(
    r: &LocalJournalRequest,
    c: &AtomicBool,
    d: Instant,
) -> LocalJournalReport {
    inspect_roots(&r.bundle_root, &r.journal_root, c, d)
}
/// Verify only current explicit bundle/journal consistency, without mutation or restored authority.
pub fn inspect_local_existing_bundle(
    r: &LocalInspectionRequest,
    c: &AtomicBool,
    d: Instant,
) -> LocalJournalReport {
    inspect_roots(&r.bundle_root, &r.journal_root, c, d)
}
fn inspect_roots(
    bundle_root: &Path,
    journal_root: &Path,
    c: &AtomicBool,
    d: Instant,
) -> LocalJournalReport {
    let mut report = LocalJournalReport::empty();
    let mut work = || -> io::Result<()> {
        check(c, d)?;
        validate(bundle_root)?;
        validate(journal_root)?;
        let journal = inspect_install_journal(journal_root, c, d)?;
        report.journal_capabilities_present = true;
        let bundle = inspect_user_bundle(bundle_root, c, d)?;
        report.bundle_capabilities_present = true;
        journal.verify_active_bundle(&bundle, c, d)?;
        report.current_consistency_verified = true;
        Ok(())
    };
    if let Err(e) = work() {
        report.kind = Some(e.kind());
    }
    report
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::time::Duration;
    fn fixture() -> (tempfile::TempDir, LocalJournalRequest) {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        std::fs::create_dir(&src).unwrap();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        std::fs::write(src.join("dlpad.exe"), &bytes).unwrap();
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            }],
        };
        let mp = dir.path().join("manifest.json");
        std::fs::write(&mp, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let request = LocalJournalRequest {
            manifest: mp,
            source_dir: src,
            bundle_root: dir.path().join("bundle"),
            journal_root: dir.path().join("journal"),
        };
        (dir, request)
    }
    #[test]
    fn cancelled_and_expired_service_creates_neither_root() {
        for expired in [false, true] {
            let (_dir, r) = fixture();
            let c = AtomicBool::new(!expired);
            let d = if expired {
                Instant::now()
            } else {
                Instant::now() + Duration::from_secs(30)
            };
            let report = prepare_local_journal_bundle(&r, &c, d);
            assert_eq!(
                report.kind,
                Some(if expired {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::Interrupted
                })
            );
            assert!(
                !report.mutation_may_have_occurred
                    && !report.bundle_capabilities_present
                    && !report.journal_capabilities_present
            );
            assert!(!r.bundle_root.exists() && !r.journal_root.exists());
        }
    }
    #[test]
    fn wrong_sha_preserves_sources_without_creation() {
        let (_dir, r) = fixture();
        let old = std::fs::read(r.source_dir.join("dlpad.exe")).unwrap();
        let mut m = BundleManifest::read(File::open(&r.manifest).unwrap()).unwrap();
        m.entries[0].sha256 = "0".repeat(64);
        std::fs::write(&r.manifest, serde_json::to_vec(&m).unwrap()).unwrap();
        let report = prepare_local_journal_bundle(
            &r,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(30),
        );
        assert_eq!(report.kind, Some(io::ErrorKind::InvalidData));
        assert!(!report.mutation_may_have_occurred);
        assert!(!r.bundle_root.exists() && !r.journal_root.exists());
        assert_eq!(std::fs::read(r.source_dir.join("dlpad.exe")).unwrap(), old);
    }
    #[test]
    fn either_existing_root_preserves_foreign_and_opposite_absence() {
        for existing_bundle in [false, true] {
            let (_dir, r) = fixture();
            let existing = if existing_bundle {
                &r.bundle_root
            } else {
                &r.journal_root
            };
            std::fs::create_dir(existing).unwrap();
            std::fs::write(existing.join("foreign"), b"preserve").unwrap();
            let report = prepare_local_journal_bundle(
                &r,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(30),
            );
            assert_eq!(report.kind, Some(io::ErrorKind::AlreadyExists));
            assert!(
                !report.mutation_may_have_occurred
                    && !report.bundle_capabilities_present
                    && !report.journal_capabilities_present
            );
            assert_eq!(
                std::fs::read(existing.join("foreign")).unwrap(),
                b"preserve"
            );
            assert!(
                !(if existing_bundle {
                    &r.journal_root
                } else {
                    &r.bundle_root
                })
                .exists()
            );
        }
    }
    #[test]
    fn service_success_then_cold_inspection_preserves_record_bytes() {
        let (_dir, r) = fixture();
        let c = AtomicBool::new(false);
        let d = Instant::now() + Duration::from_secs(30);
        let report = prepare_local_journal_bundle(&r, &c, d);
        assert_eq!(report.kind, None);
        assert!(
            report.current_consistency_verified
                && report.mutation_may_have_occurred
                && report.bundle_capabilities_present
                && report.journal_capabilities_present
        );
        assert_eq!(report.entries, Some(1));
        let before = std::fs::read(r.journal_root.join("active.json")).unwrap();
        let inspected = inspect_local_journal_bundle(&r, &c, d);
        assert_eq!(inspected.kind, None);
        assert!(
            inspected.current_consistency_verified
                && inspected.bundle_capabilities_present
                && inspected.journal_capabilities_present
        );
        assert!(!inspected.mutation_may_have_occurred);
        assert_eq!(
            std::fs::read(r.journal_root.join("active.json")).unwrap(),
            before
        );
    }
    #[test]
    fn existing_two_root_inspection_ignores_missing_original_sources() {
        let (_dir, r) = fixture();
        let c = AtomicBool::new(false);
        let d = Instant::now() + Duration::from_secs(30);
        assert_eq!(prepare_local_journal_bundle(&r, &c, d).kind, None);
        let before = std::fs::read(r.journal_root.join("active.json")).unwrap();
        // Only this test-owned original source is removed; installed roots remain untouched.
        std::fs::remove_file(&r.manifest).unwrap();
        std::fs::remove_file(r.source_dir.join("dlpad.exe")).unwrap();
        std::fs::remove_dir(&r.source_dir).unwrap();
        let two = LocalInspectionRequest {
            bundle_root: r.bundle_root.clone(),
            journal_root: r.journal_root.clone(),
        };
        for report in [
            inspect_local_existing_bundle(&two, &c, d),
            inspect_local_journal_bundle(&r, &c, d),
        ] {
            assert_eq!(report.kind, None);
            assert!(
                report.current_consistency_verified
                    && report.bundle_capabilities_present
                    && report.journal_capabilities_present
            );
            assert!(!report.mutation_may_have_occurred);
            assert_eq!(report.entries, None);
        }
        assert_eq!(
            std::fs::read(r.journal_root.join("active.json")).unwrap(),
            before
        );
        assert!(!r.source_dir.exists() && !r.manifest.exists());
    }
    #[test]
    fn existing_two_root_cancel_and_deadline_do_not_create_paths() {
        let dir = tempfile::tempdir().unwrap();
        let r = LocalInspectionRequest {
            bundle_root: dir.path().join("missing-bundle"),
            journal_root: dir.path().join("missing-journal"),
        };
        for expired in [false, true] {
            let c = AtomicBool::new(!expired);
            let d = if expired {
                Instant::now()
            } else {
                Instant::now() + Duration::from_secs(30)
            };
            let report = inspect_local_existing_bundle(&r, &c, d);
            assert_eq!(
                report.kind,
                Some(if expired {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::Interrupted
                })
            );
            assert!(
                !report.mutation_may_have_occurred
                    && !report.current_consistency_verified
                    && !report.bundle_capabilities_present
                    && !report.journal_capabilities_present
            );
            assert!(!r.bundle_root.exists() && !r.journal_root.exists());
        }
    }
}
