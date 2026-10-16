//! Explicit development selection snapshot; no launch, discovery, delete or trust authority.
use super::{
    RegisteredPairInspection,
    active_schema::ActiveSelectionRecord,
    inspect_registered_with_receipt,
    registration_schema::record_digest,
    windows_caps::{
        CreatedFile, FreshDestination, FreshNamePublishError, FreshRoot, ReadFile, ReadRoot,
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Write},
    os::windows::{ffi::OsStrExt, fs::FileExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "active selection rejected")
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "active selection cancelled",
        ))
    } else if Instant::now() >= d {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "active selection deadline",
        ))
    } else {
        Ok(())
    }
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(a: *const u16, al: i32, b: *const u16, bl: i32, ignore: i32) -> i32;
}
fn contains(a: &Path, b: &Path) -> bool {
    let a: Vec<_> = a.as_os_str().encode_wide().collect();
    let b: Vec<_> = b.as_os_str().encode_wide().collect();
    a.len() <= b.len()
        && unsafe {
            CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1)
        } == 2
        && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92))
}
fn paths(active: &Path, pair: &RegisteredPairInspection) -> io::Result<(PathBuf, PathBuf)> {
    let reg = pair.registered().original_final_path()?;
    let receipt = pair.receipt().original_final_path()?;
    for other in [&reg, &receipt] {
        if contains(active, other) || contains(other, active) {
            return Err(bad());
        }
    }
    Ok((reg, receipt))
}
fn text(p: &Path) -> io::Result<String> {
    p.to_str().map(str::to_owned).ok_or_else(bad)
}
fn bytes(f: &File, c: &AtomicBool, d: Instant) -> io::Result<Vec<u8>> {
    check(c, d)?;
    let size = f.metadata()?.len();
    if size == 0 || size > 65536 {
        return Err(bad());
    }
    let mut raw = vec![0; size as usize];
    let mut at = 0;
    while at < raw.len() {
        check(c, d)?;
        let n = f.seek_read(&mut raw[at..], at as u64)?;
        if n == 0 {
            return Err(bad());
        }
        at += n;
    }
    if f.seek_read(&mut [0], size)? != 0 || f.metadata()?.len() != size {
        return Err(bad());
    }
    check(c, d)?;
    Ok(raw)
}
fn namespace(p: &Path, c: &AtomicBool, d: Instant) -> io::Result<()> {
    namespace_named(p, "active-selection.json", c, d)
}
fn namespace_named(p: &Path, leaf: &str, c: &AtomicBool, d: Instant) -> io::Result<()> {
    let mut count = 0;
    for e in std::fs::read_dir(p)? {
        check(c, d)?;
        let e = e?;
        count += 1;
        if count > 1 || e.file_name() != leaf || !e.file_type()?.is_file() {
            return Err(bad());
        }
    }
    if count != 1 {
        return Err(bad());
    }
    check(c, d)
}
fn matching(
    record: &ActiveSelectionRecord,
    active: &Path,
    pair: &RegisteredPairInspection,
) -> io::Result<()> {
    let (reg, receipt) = paths(active, pair)?;
    if record.active_path != text(active)?
        || record.registered_path != text(&reg)?
        || record.receipt_path != text(&receipt)?
    {
        return Err(bad());
    }
    record.verify_pair(pair.registered().record(), pair.receipt().record())
}
pub struct FreshActiveSelection {
    root: FreshRoot,
    file: CreatedFile,
    record: ActiveSelectionRecord,
    pair: RegisteredPairInspection,
}
impl std::fmt::Debug for FreshActiveSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FreshActiveSelection(<redacted>)")
    }
}
impl FreshActiveSelection {
    pub fn record(&self) -> &ActiveSelectionRecord {
        &self.record
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        check(c, d)?;
        self.pair.verify_current(c, d)?;
        self.root.verify()?;
        self.file.verify(&self.root)?;
        let final_path = crate::file_identity::final_path(self.root.file())?;
        matching(&self.record, &final_path, &self.pair)?;
        if self.record.active_root_identity != *self.root.identity()
            || self.record.marker_identity != *self.file.identity()
            || bytes(self.file.file(), c, d)? != self.record.canonical_bytes()?
        {
            return Err(bad());
        }
        namespace(self.root.path(), c, d)?;
        self.file.verify(&self.root)?;
        self.root.verify()?;
        self.pair.verify_current(c, d)?;
        matching(
            &self.record,
            &crate::file_identity::final_path(self.root.file())?,
            &self.pair,
        )?;
        check(c, d)
    }
}
pub struct ActiveSelectionFailure {
    error: io::Error,
    root: Option<Box<FreshRoot>>,
    file: Option<Box<CreatedFile>>,
    pair: Box<RegisteredPairInspection>,
    mutation: bool,
}
impl ActiveSelectionFailure {
    pub fn retained_pair(&self) -> &RegisteredPairInspection {
        &self.pair
    }
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
}
impl std::fmt::Debug for ActiveSelectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveSelectionFailure")
            .field("kind", &self.kind())
            .field("mutation", &self.mutation)
            .field("root_capability_acquired", &self.root_capability_acquired())
            .field("retained_file_caps", &self.retained_file_cap_count())
            .finish()
    }
}
impl std::fmt::Display for ActiveSelectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("active selection failed; retained objects preserved")
    }
}
impl std::error::Error for ActiveSelectionFailure {}
pub fn select_registered_pair(
    pair: RegisteredPairInspection,
    target: &Path,
    c: &AtomicBool,
    d: Instant,
) -> Result<FreshActiveSelection, ActiveSelectionFailure> {
    select_with_hook(pair, target, c, d, &mut || Ok(()))
}
pub(super) fn select_registered_pair_with_root_factory(
    pair: RegisteredPairInspection,
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    factory: &mut dyn FnMut() -> io::Result<FreshRoot>,
) -> Result<FreshActiveSelection, ActiveSelectionFailure> {
    select_with_hook_and_factory(pair, target, c, d, &mut || Ok(()), factory)
}
fn select_with_hook(
    pair: RegisteredPairInspection,
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    hook: &mut dyn FnMut() -> io::Result<()>,
) -> Result<FreshActiveSelection, ActiveSelectionFailure> {
    select_with_hook_and_factory(pair, target, c, d, hook, &mut || {
        FreshRoot::create_new(target)
    })
}
fn select_with_hook_and_factory(
    pair: RegisteredPairInspection,
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    hook: &mut dyn FnMut() -> io::Result<()>,
    factory: &mut dyn FnMut() -> io::Result<FreshRoot>,
) -> Result<FreshActiveSelection, ActiveSelectionFailure> {
    let mut root = None;
    let mut file = None;
    let mut mutation = false;
    let operation = (|| -> io::Result<ActiveSelectionRecord> {
        check(c, d)?;
        pair.verify_current(c, d)?;
        let dest = FreshDestination::observe(target)?;
        dest.verify()?;
        let (reg, receipt) = paths(dest.final_path(), &pair)?;
        let registration = pair.registered().record();
        let proof = pair.receipt().record();
        let selected = registration
            .prepared
            .entries
            .iter()
            .find(|e| e.name == "dlpad.exe")
            .ok_or_else(bad)?
            .clone();
        let reg_hash = record_digest(registration)?;
        let receipt_hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(proof).map_err(|_| bad())?)
        );
        // Conservative pre-mutation path/record budget; actual canonical record bound remains authoritative.
        if serde_json::to_vec(&(
            text(dest.final_path())?,
            text(&reg)?,
            text(&receipt)?,
            &selected,
        ))
        .map_err(|_| bad())?
        .len()
            > 65536 - 8192
        {
            return Err(bad());
        }
        check(c, d)?;
        pair.verify_current(c, d)?;
        dest.verify()?;
        mutation = true;
        root = Some(Box::new(factory()?));
        let owned = root.as_ref().ok_or_else(bad)?;
        check(c, d)?;
        owned.verify()?;
        dest.verify()?;
        let actual = crate::file_identity::final_path(owned.file())?;
        if actual != dest.final_path() {
            return Err(bad());
        }
        paths(&actual, &pair)?;
        file = Some(Box::new(owned.create_file("active-selection.pending")?));
        let marker = file.as_mut().ok_or_else(bad)?;
        let record = ActiveSelectionRecord {
            schema: 1,
            domain: "dlpad-explicit-active-selection-v1".into(),
            development_only: true,
            active_root_identity: owned.identity().clone(),
            marker_identity: marker.identity().clone(),
            active_path: text(&actual)?,
            registered_path: text(&reg)?,
            receipt_path: text(&receipt)?,
            registration_root_identity: registration.root_identity.clone(),
            registration_record_identity: registration.record_identity.clone(),
            registration_seal_identity: registration.seal_identity.clone(),
            registration_ready_identity: registration.ready_identity.clone(),
            receipt_root_identity: proof.receipt_root_identity.clone(),
            receipt_identity: proof.receipt_identity.clone(),
            registration_sha256: reg_hash,
            receipt_sha256: receipt_hash,
            selected_executable: selected,
        };
        record.verify_pair(registration, proof)?;
        let raw = record.canonical_bytes()?;
        check(c, d)?;
        marker.file_mut().write_all(&raw)?;
        check(c, d)?;
        hook()?;
        marker.file_mut().sync_all()?;
        check(c, d)?;
        owned.verify()?;
        marker.verify(owned)?;
        if bytes(marker.file(), c, d)? != raw {
            return Err(bad());
        }
        pair.verify_current(c, d)?;
        dest.verify()?;
        check(c, d)?;
        namespace_named(owned.path(), "active-selection.pending", c, d)?;
        marker
            .publish_fresh_name(owned, "active-selection.json")
            .map_err(|error| {
                // Every failure after root acquisition retains original objects below.
                // Publication uncertainty is never permission for namespace cleanup.
                match error {
                    FreshNamePublishError::BeforePublish(error) => error,
                    FreshNamePublishError::RenameUnconfirmed(error)
                    | FreshNamePublishError::VerificationUnconfirmed(error) => {
                        io::Error::other(error)
                    }
                }
            })?;
        namespace(owned.path(), c, d)?;
        pair.verify_current(c, d)?;
        dest.verify()?;
        matching(
            &record,
            &crate::file_identity::final_path(owned.file())?,
            &pair,
        )?;
        owned.verify()?;
        marker.verify(owned)?;
        namespace(owned.path(), c, d)?;
        check(c, d)?;
        Ok(record)
    })();
    match operation {
        Ok(record) => Ok(FreshActiveSelection {
            root: *root.expect("successful original root"),
            file: *file.expect("successful original marker"),
            record,
            pair,
        }),
        Err(error) => Err(ActiveSelectionFailure {
            error,
            root,
            file,
            pair: Box::new(pair),
            mutation,
        }),
    }
}
pub struct ActiveSelectionInspection {
    root: ReadRoot,
    file: ReadFile,
    record: ActiveSelectionRecord,
    pair: RegisteredPairInspection,
}
impl std::fmt::Debug for ActiveSelectionInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActiveSelectionInspection(<redacted>)")
    }
}
impl ActiveSelectionInspection {
    pub(super) fn removal_pair(&self) -> &RegisteredPairInspection {
        &self.pair
    }
    pub(super) fn into_bundle_removal_inputs(
        self,
    ) -> (ReadRoot, ActiveSelectionRecord, RegisteredPairInspection) {
        let Self {
            root,
            file,
            record,
            pair,
        } = self;
        drop(file);
        (root, record, pair)
    }
    /// Borrow original selected file; derive its leaf from the original registered root.
    pub(super) fn launch_inputs(&self) -> io::Result<(&std::fs::File, std::path::PathBuf)> {
        let registered = self.pair.registered();
        let root = registered.original_final_path()?;
        Ok((registered.executable(), root.join("dlpad.exe")))
    }
    pub(super) fn original_prepared_record(&self) -> &super::PreparedRecord {
        &self.pair.registered().record().prepared
    }

    pub fn record(&self) -> &ActiveSelectionRecord {
        &self.record
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        check(c, d)?;
        self.pair.verify_current(c, d)?;
        self.root.verify()?;
        self.file.verify(&self.root)?;
        let final_path = self.root.original_final_path()?;
        matching(&self.record, &final_path, &self.pair)?;
        if self.record.active_root_identity != *self.root.identity()
            || self.record.marker_identity != *self.file.identity()
            || bytes(self.file.file(), c, d)? != self.record.canonical_bytes()?
        {
            return Err(bad());
        }
        namespace(self.root.path(), c, d)?;
        self.root.verify()?;
        self.file.verify(&self.root)?;
        self.pair.verify_current(c, d)?;
        matching(&self.record, &self.root.original_final_path()?, &self.pair)?;
        namespace(self.root.path(), c, d)?;
        check(c, d)
    }
}
pub fn inspect_active_selection(
    active: &Path,
    registered: &Path,
    receipt: &Path,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ActiveSelectionInspection> {
    check(c, d)?;
    let pair = inspect_registered_with_receipt(registered, receipt, c, d)?;
    let root = ReadRoot::open_existing(active)?;
    paths(&root.original_final_path()?, &pair)?;
    namespace(root.path(), c, d)?;
    let file = root.open_file("active-selection.json")?;
    let record = ActiveSelectionRecord::read(bytes(file.file(), c, d)?.as_slice())?;
    let value = ActiveSelectionInspection {
        root,
        file,
        record,
        pair,
    };
    value.verify_current(c, d)?;
    Ok(value)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::Duration};
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn produced() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let mut data = vec![0; 256];
        data[..2].copy_from_slice(b"MZ");
        data[60..64].copy_from_slice(&64u32.to_le_bytes());
        data[64..68].copy_from_slice(b"PE\0\0");
        data[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        data[84..86].copy_from_slice(&112u16.to_le_bytes());
        data[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(dir.path().join("source"), &data).unwrap();
        fs::write(dir.path().join("sentinel"), b"outside").unwrap();
        let source = File::open(dir.path().join("source")).unwrap();
        let m = super::super::manifest::BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: format!("{:x}", Sha256::digest(&data)),
            }],
        };
        let c = AtomicBool::new(false);
        let reg = dir.path().join("registered");
        let ready =
            super::super::prepare_new(&m, &[("dlpad.exe", &source)], &reg, &c, later()).unwrap();
        let registered = super::super::register_fresh(ready, &c, later()).unwrap();
        let receipt = dir.path().join("receipt");
        let external =
            super::super::create_registration_receipt(&registered, &receipt, &c, later()).unwrap();
        drop(external);
        drop(registered);
        (dir, reg, receipt)
    }
    fn pair(reg: &Path, receipt: &Path) -> RegisteredPairInspection {
        inspect_registered_with_receipt(reg, receipt, &AtomicBool::new(false), later()).unwrap()
    }
    #[test]
    fn actual_select_drop_reopen_preserves_sources() {
        let (dir, reg, receipt) = produced();
        let target = dir.path().join("active");
        let value = select_registered_pair(
            pair(&reg, &receipt),
            &target,
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        value
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
        let raw = value.record().canonical_bytes().unwrap();
        drop(value);
        let reopened =
            inspect_active_selection(&target, &reg, &receipt, &AtomicBool::new(false), later())
                .unwrap();
        assert_eq!(reopened.record().canonical_bytes().unwrap(), raw);
        drop(reopened);
        assert_eq!(fs::read(target.join("active-selection.json")).unwrap(), raw);
        assert_eq!(fs::read(dir.path().join("sentinel")).unwrap(), b"outside");
    }
    #[test]
    fn preflight_cancel_deadline_existing_overlap_do_not_mutate() {
        let (dir, reg, receipt) = produced();
        for (c, d, kind) in [
            (AtomicBool::new(true), later(), io::ErrorKind::Interrupted),
            (
                AtomicBool::new(false),
                Instant::now(),
                io::ErrorKind::TimedOut,
            ),
        ] {
            let target = dir.path().join("absent");
            let fail = select_registered_pair(pair(&reg, &receipt), &target, &c, d).unwrap_err();
            assert_eq!(fail.kind(), kind);
            assert!(!fail.mutation_may_have_occurred());
            assert!(!target.exists());
        }
        let existing = dir.path().join("existing");
        fs::create_dir(&existing).unwrap();
        fs::write(existing.join("foreign"), b"foreign").unwrap();
        assert!(
            select_registered_pair(
                pair(&reg, &receipt),
                &existing,
                &AtomicBool::new(false),
                later()
            )
            .is_err()
        );
        assert_eq!(fs::read(existing.join("foreign")).unwrap(), b"foreign");
        let overlap = reg.join("active");
        let fail = select_registered_pair(
            pair(&reg, &receipt),
            &overlap,
            &AtomicBool::new(false),
            later(),
        )
        .unwrap_err();
        assert!(!fail.mutation_may_have_occurred());
        assert!(!overlap.exists());
    }
    #[test]
    fn postwrite_hook_failure_preserves_original_caps_and_foreign_reopen_refuses() {
        let (dir, reg, receipt) = produced();
        let target = dir.path().join("partial");
        let fail = select_with_hook(
            pair(&reg, &receipt),
            &target,
            &AtomicBool::new(false),
            later(),
            &mut || {
                Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "injected owned fixture failure",
                ))
            },
        )
        .unwrap_err();
        assert_eq!(fail.kind(), io::ErrorKind::WriteZero);
        assert!(fail.mutation_may_have_occurred());
        assert!(fail.root_capability_acquired());
        assert_eq!(fail.retained_file_cap_count(), 1);
        let before = bytes(
            fail.file.as_ref().unwrap().file(),
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        assert!(!target.join("active-selection.json").exists());
        drop(fail);
        assert_eq!(
            fs::read(target.join("active-selection.pending")).unwrap(),
            before
        );
        fs::write(target.join("foreign"), b"keep").unwrap();
        assert!(
            inspect_active_selection(&target, &reg, &receipt, &AtomicBool::new(false), later())
                .is_err()
        );
        assert_eq!(fs::read(target.join("foreign")).unwrap(), b"keep");
        assert_eq!(
            fs::read(target.join("active-selection.pending")).unwrap(),
            before
        );
    }
    #[test]
    fn valid_other_pair_cannot_match_selection() {
        let (dir, reg, receipt) = produced();
        let (_other, reg2, receipt2) = produced();
        let target = dir.path().join("active");
        let active = select_registered_pair(
            pair(&reg, &receipt),
            &target,
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        let raw = active.record().canonical_bytes().unwrap();
        drop(active);
        assert!(
            inspect_active_selection(&target, &reg2, &receipt2, &AtomicBool::new(false), later())
                .is_err()
        );
        assert_eq!(fs::read(target.join("active-selection.json")).unwrap(), raw);
    }
    // Insert inside active_selection::tests; reuse actual produced(), pair(), later().
    #[test]
    fn concurrent_fresh_creators_exactly_one_wins_loser_never_adopts() {
        let (dir, reg, receipt) = produced();
        let target = dir.path().join("concurrent-active");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let mut threads = Vec::new();
        for _ in 0..2 {
            let (reg, receipt, target, barrier) = (
                reg.clone(),
                receipt.clone(),
                target.clone(),
                barrier.clone(),
            );
            threads.push(std::thread::spawn(move || {
                let pair = pair(&reg, &receipt);
                barrier.wait();
                select_registered_pair(pair, &target, &AtomicBool::new(false), later())
            }));
        }
        let mut winner = None;
        let mut failures = 0;
        for thread in threads {
            match thread.join().unwrap() {
                Ok(value) => {
                    assert!(winner.is_none(), "two callers acquired the same fresh root");
                    winner = Some(value);
                }
                Err(failure) => {
                    failures += 1;
                    assert!(
                        !failure.root_capability_acquired(),
                        "loser adopted another caller's root"
                    );
                    assert_eq!(failure.retained_file_cap_count(), 0);
                    // Observation may lose before create or create itself may lose;
                    // conservative mutation hint may therefore be false OR true.
                    failure
                        .retained_pair()
                        .verify_current(&AtomicBool::new(false), later())
                        .unwrap();
                }
            }
        }
        assert_eq!(failures, 1);
        let winner = winner.expect("one original create-only root owner");
        winner
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
        let raw = winner.record().canonical_bytes().unwrap();
        drop(winner);
        assert_eq!(
            std::fs::read(target.join("active-selection.json")).unwrap(),
            raw
        );
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
        assert_eq!(
            std::fs::read(dir.path().join("sentinel")).unwrap(),
            b"outside"
        );
    }
    #[test]
    fn identical_bytes_marker_replacement_is_not_original_identity() {
        let (dir, reg, receipt) = produced();
        let target = dir.path().join("replace-marker");
        let active = select_registered_pair(
            pair(&reg, &receipt),
            &target,
            &AtomicBool::new(false),
            later(),
        )
        .unwrap();
        let expected = active.record().marker_identity.clone();
        let raw = active.record().canonical_bytes().unwrap();
        drop(active);
        let marker = target.join("active-selection.json");
        let retired = dir.path().join("original-marker-preserved");
        std::fs::rename(&marker, &retired).unwrap();
        let replacement_root =
            FreshRoot::create_new(&dir.path().join("private-replacement")).unwrap();
        let mut replacement = replacement_root.create_file("replacement.json").unwrap();
        replacement.file_mut().write_all(&raw).unwrap();
        replacement.file_mut().sync_all().unwrap();
        replacement.verify(&replacement_root).unwrap();
        let replacement_id = replacement.identity().clone();
        assert_ne!(replacement_id, expected);
        let sid = crate::windows_private_security::current_sid().unwrap();
        crate::windows_private_security::verify_acl(replacement.file(), &sid).unwrap();
        let replacement_path = replacement_root.path().join("replacement.json");
        drop(replacement);
        drop(replacement_root);
        std::fs::rename(&replacement_path, &marker).unwrap();
        // Prove the replacement's ACL and nativeID survive the rename, before
        // schema inspection. Do not let an inherited default ACL explain refusal.
        let readonly_root = ReadRoot::open_existing(&target).unwrap();
        let readonly_replacement = readonly_root.open_file("active-selection.json").unwrap();
        readonly_replacement.verify(&readonly_root).unwrap();
        assert_eq!(*readonly_replacement.identity(), replacement_id);
        crate::windows_private_security::verify_acl(readonly_replacement.file(), &sid).unwrap();
        drop(readonly_replacement);
        drop(readonly_root);
        assert_eq!(
            inspect_active_selection(&target, &reg, &receipt, &AtomicBool::new(false), later())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(std::fs::read(&marker).unwrap(), raw);
        assert_eq!(std::fs::read(&retired).unwrap(), raw);
        assert_eq!(
            std::fs::read(dir.path().join("sentinel")).unwrap(),
            b"outside"
        );
    }
    #[test]
    fn postwrite_cancellation_preserves_confirmed_original_caps() {
        let (dir, reg, receipt) = produced();
        let target = dir.path().join("cancel-after-write");
        let cancel = AtomicBool::new(false);
        let failure =
            select_with_hook(pair(&reg, &receipt), &target, &cancel, later(), &mut || {
                cancel.store(true, Ordering::Release);
                Ok(())
            })
            .unwrap_err();
        assert_eq!(failure.kind(), io::ErrorKind::Interrupted);
        assert!(failure.mutation_may_have_occurred());
        assert!(failure.root_capability_acquired());
        assert_eq!(failure.retained_file_cap_count(), 1);
        let file = failure.file.as_ref().unwrap();
        let identity = file.identity().clone();
        let raw = bytes(file.file(), &AtomicBool::new(false), later()).unwrap();
        assert_eq!(
            crate::file_identity::from_file(file.file()).unwrap(),
            identity
        );
        failure
            .retained_pair()
            .verify_current(&AtomicBool::new(false), later())
            .unwrap();
        assert!(!target.join("active-selection.json").exists());
        drop(failure);
        let current = File::open(target.join("active-selection.pending")).unwrap();
        assert_eq!(crate::file_identity::from_file(&current).unwrap(), identity);
        assert_eq!(
            std::fs::read(target.join("active-selection.pending")).unwrap(),
            raw
        );
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
        assert_eq!(
            std::fs::read(dir.path().join("sentinel")).unwrap(),
            b"outside"
        );
    }
}
