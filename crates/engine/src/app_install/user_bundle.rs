//! Fresh development bundle only. No crash recovery, replacement or automatic deletion.
use super::{
    ActiveSelectionFailure, FreshActiveSelection, FreshRegistration, FreshRegistrationReceipt,
    PrepareFailure, PreparedBundle, ReceiptFailure, RegisteredPairInspection, RegistrationFailure,
    manifest::BundleManifest, windows_caps::FreshRoot,
};
use std::{
    collections::BTreeSet,
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
const CHILDREN: [&str; 3] = ["registered", "receipt", "active"];
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "fresh user bundle rejected")
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "bundle cancelled",
        ))
    } else if Instant::now() >= d {
        Err(io::Error::new(io::ErrorKind::TimedOut, "bundle deadline"))
    } else {
        Ok(())
    }
}
fn namespace_for(
    root: &FreshRoot,
    expected: &[&str],
    c: &AtomicBool,
    d: Instant,
) -> io::Result<()> {
    check(c, d)?;
    root.verify()?;
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(root.path())? {
        check(c, d)?;
        let entry = entry?;
        if names.len() >= 3 || !entry.file_type()?.is_dir() {
            return Err(bad());
        }
        let name = entry.file_name().into_string().map_err(|_| bad())?;
        if !names.insert(name) {
            return Err(bad());
        }
    }
    if names != expected.iter().map(|s| s.to_string()).collect() {
        return Err(bad());
    }
    root.verify()?;
    check(c, d)
}
fn namespace(root: &FreshRoot, c: &AtomicBool, d: Instant) -> io::Result<()> {
    namespace_for(root, &CHILDREN, c, d)
}
/// Retains all original create capabilities. Drop never deletes or repairs files.
pub struct FreshUserBundle {
    outer: FreshRoot,
    registration: FreshRegistration,
    receipt: FreshRegistrationReceipt,
    active: FreshActiveSelection,
}
impl FreshUserBundle {
    pub fn path(&self) -> &Path {
        self.outer.path()
    }
    pub fn registration(&self) -> &FreshRegistration {
        &self.registration
    }
    pub fn receipt(&self) -> &FreshRegistrationReceipt {
        &self.receipt
    }
    pub fn active(&self) -> &FreshActiveSelection {
        &self.active
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        namespace(&self.outer, c, d)?;
        self.registration.verify_current(c, d)?;
        self.receipt.verify_current(c, d)?;
        self.active.verify_current(c, d)?;
        namespace(&self.outer, c, d)
    }
}
impl std::fmt::Debug for FreshUserBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FreshUserBundle(<redacted>)")
    }
}
enum StageFailure {
    Prepare(Box<PrepareFailure>),
    Register(Box<RegistrationFailure>),
    Receipt(Box<ReceiptFailure>),
    Active(Box<ActiveSelectionFailure>),
}
/// Partial original capabilities are retained opaquely; no path-based recovery authority.
pub struct UserBundleFailure {
    state: Box<UserBundleFailureState>,
}
struct UserBundleFailureState {
    error: io::Error,
    stage: &'static str,
    mutation: bool,
    target: PathBuf,
    outer: Option<FreshRoot>,
    prepared: Option<PreparedBundle>,
    registration: Option<FreshRegistration>,
    receipt: Option<FreshRegistrationReceipt>,
    active: Option<FreshActiveSelection>,
    failure: Option<StageFailure>,
}
impl UserBundleFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.state.error.kind()
    }
    pub fn stage(&self) -> &'static str {
        self.state.stage
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.state.mutation
    }
    pub fn outer_capability_acquired(&self) -> bool {
        self.state.outer.is_some()
    }
    pub fn target_hint(&self) -> &Path {
        &self.state.target
    }
    /// A total count is deliberately not reported; nested retained capabilities remain opaque.
    pub fn retained_capabilities_present(&self) -> bool {
        self.state.outer.is_some()
            || self.state.prepared.is_some()
            || self.state.registration.is_some()
            || self.state.receipt.is_some()
            || self.state.active.is_some()
            || match self.state.failure.as_ref() {
                Some(StageFailure::Prepare(e)) => {
                    e.root_capability_acquired() || e.retained_file_cap_count() != 0
                }
                // RegistrationFailure always owns the original PreparedBundle;
                // its file count describes only newly created registration files.
                Some(StageFailure::Register(_)) => true,
                Some(StageFailure::Receipt(e)) => {
                    e.root_capability_acquired() || e.retained_file_cap_count() != 0
                }
                // ActiveSelectionFailure always retains the original pair,
                // independently of newly created active root/file capabilities.
                Some(StageFailure::Active(e)) => {
                    let _original_pair = e.retained_pair();
                    true
                }
                None => false,
            }
    }
}
impl std::fmt::Debug for UserBundleFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserBundleFailure")
            .field("kind", &self.kind())
            .field("stage", &self.state.stage)
            .field("mutation_may_have_occurred", &self.state.mutation)
            .field(
                "outer_capability_acquired",
                &self.outer_capability_acquired(),
            )
            .finish_non_exhaustive()
    }
}
impl std::fmt::Display for UserBundleFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fresh bundle {} failed: {}",
            self.state.stage, self.state.error
        )
    }
}
impl std::error::Error for UserBundleFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.state.failure.as_ref() {
            Some(StageFailure::Prepare(e)) => Some(e.as_ref()),
            Some(StageFailure::Register(e)) => Some(e.as_ref()),
            Some(StageFailure::Receipt(e)) => Some(e.as_ref()),
            Some(StageFailure::Active(e)) => Some(e.as_ref()),
            None => Some(&self.state.error),
        }
    }
}
/// Uses the private original-capability observer conversion; never cold path adoption.
/// It must retain caller originals and compare expected identities, not adopt currently observed IDs.
pub fn create_fresh_user_bundle(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    c: &AtomicBool,
    d: Instant,
) -> Result<FreshUserBundle, UserBundleFailure> {
    create_with_pair_factory(
        manifest,
        files,
        target,
        c,
        d,
        &mut |registration, receipt, c, d| make_pair(registration, receipt, c, d),
    )
}
fn create_with_pair_factory(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    pair_factory: &mut dyn FnMut(
        &FreshRegistration,
        &FreshRegistrationReceipt,
        &AtomicBool,
        Instant,
    ) -> io::Result<RegisteredPairInspection>,
) -> Result<FreshUserBundle, UserBundleFailure> {
    create_with_factories(manifest, files, target, c, d, pair_factory, None)
}
pub(super) fn create_with_stage_observer(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    observer: &mut dyn super::journal_bundle::BundleStageObserver,
) -> Result<FreshUserBundle, UserBundleFailure> {
    create_with_factories(
        manifest,
        files,
        target,
        c,
        d,
        &mut make_pair,
        Some(observer),
    )
}
fn create_with_factories(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    target: &Path,
    c: &AtomicBool,
    d: Instant,
    pair_factory: &mut dyn FnMut(
        &FreshRegistration,
        &FreshRegistrationReceipt,
        &AtomicBool,
        Instant,
    ) -> io::Result<RegisteredPairInspection>,
    mut observer: Option<&mut dyn super::journal_bundle::BundleStageObserver>,
) -> Result<FreshUserBundle, UserBundleFailure> {
    let mut f = UserBundleFailureState {
        error: bad(),
        stage: "preflight",
        mutation: false,
        target: target.to_owned(),
        outer: None,
        prepared: None,
        registration: None,
        receipt: None,
        active: None,
        failure: None,
    };
    let result = (|| -> io::Result<()> {
        check(c, d)?;
        manifest.validate()?;
        if !manifest.development_only {
            return Err(bad());
        }
        f.stage = "prepare";
        let registered = target.join("registered");
        // The transaction checks/pins/hashes every source BEFORE calling this factory.
        let mut outer_exclusive_collision = false;
        let prepared = super::transaction::prepare_new_with_root_factory(
            manifest,
            files,
            &registered,
            c,
            d,
            &mut || {
                check(c, d)?;
                if let Some(observer) = observer.as_deref_mut() {
                    observer.before_outer(c, d)?;
                }
                check(c, d)?;
                f.mutation = true;
                f.outer = Some(match FreshRoot::create_new_classified(target) {
                    Ok(root) => root,
                    Err(failure) => {
                        if failure.exclusive_collision {
                            outer_exclusive_collision = true;
                            f.mutation = false;
                        }
                        return Err(failure.error);
                    }
                });
                f.outer.as_ref().ok_or_else(bad)?.create_child("registered")
            },
        );
        match prepared {
            Ok(value) => f.prepared = Some(value),
            Err(error) => {
                let kind = error.kind();
                // The transaction marks a target hint before invoking its factory.
                // Only our exact outer FILE_CREATE collision proves no bundle mutation.
                if !outer_exclusive_collision {
                    f.mutation |= error.mutation_may_have_occurred();
                }
                f.failure = Some(StageFailure::Prepare(Box::new(error)));
                return Err(io::Error::new(kind, "bundle prepare failed"));
            }
        }
        if let Some(observer) = observer.as_deref_mut() {
            f.stage = "journal-prepared";
            let prepared = f.prepared.as_ref().ok_or_else(bad)?;
            super::transaction::verify_current_controlled(prepared, c, d)?;
            observer.after_verified(
                super::InstallJournalPhase::Prepared,
                &serde_json::to_vec(prepared.record())?,
                c,
                d,
            )?;
        }
        f.stage = "register";
        check(c, d)?;
        match super::register_fresh(f.prepared.take().ok_or_else(bad)?, c, d) {
            Ok(value) => f.registration = Some(value),
            Err(error) => {
                let kind = error.kind();
                f.failure = Some(StageFailure::Register(Box::new(error)));
                return Err(io::Error::new(kind, "bundle registration failed"));
            }
        }
        if let Some(observer) = observer.as_deref_mut() {
            f.stage = "journal-registered";
            let registration = f.registration.as_ref().ok_or_else(bad)?;
            registration.verify_current(c, d)?;
            observer.after_verified(
                super::InstallJournalPhase::Registered,
                &serde_json::to_vec(registration.record())?,
                c,
                d,
            )?;
        }
        f.stage = "receipt";
        check(c, d)?;
        namespace_for(f.outer.as_ref().ok_or_else(bad)?, &["registered"], c, d)?;
        let receipt_path = target.join("receipt");
        let outer = f.outer.as_ref().ok_or_else(bad)?;
        match super::receipt::create_registration_receipt_with_root_factory(
            f.registration.as_ref().ok_or_else(bad)?,
            &receipt_path,
            c,
            d,
            &mut || {
                check(c, d)?;
                outer.create_child("receipt")
            },
        ) {
            Ok(value) => f.receipt = Some(value),
            Err(error) => {
                let kind = error.kind();
                f.failure = Some(StageFailure::Receipt(Box::new(error)));
                return Err(io::Error::new(kind, "bundle receipt failed"));
            }
        }
        if let Some(observer) = observer.as_deref_mut() {
            f.stage = "journal-receipt";
            let receipt = f.receipt.as_ref().ok_or_else(bad)?;
            receipt.verify_current(c, d)?;
            observer.after_verified(
                super::InstallJournalPhase::Receipt,
                &serde_json::to_vec(receipt.record())?,
                c,
                d,
            )?;
        }
        f.stage = "pair";
        check(c, d)?;
        let pair = pair_factory(
            f.registration.as_ref().ok_or_else(bad)?,
            f.receipt.as_ref().ok_or_else(bad)?,
            c,
            d,
        )?;
        f.stage = "active";
        check(c, d)?;
        namespace_for(
            f.outer.as_ref().ok_or_else(bad)?,
            &["registered", "receipt"],
            c,
            d,
        )?;
        // Active marker is last. Previous completed stages stay retained on failure.
        let active_path = target.join("active");
        match super::active_selection::select_registered_pair_with_root_factory(
            pair,
            &active_path,
            c,
            d,
            &mut || {
                check(c, d)?;
                namespace_for(outer, &["registered", "receipt"], c, d)?;
                outer.create_child("active")
            },
        ) {
            Ok(value) => f.active = Some(value),
            Err(error) => {
                let kind = error.kind();
                f.failure = Some(StageFailure::Active(Box::new(error)));
                return Err(io::Error::new(kind, "bundle active failed"));
            }
        }
        f.stage = "postverify";
        namespace(outer, c, d)?;
        f.registration
            .as_ref()
            .ok_or_else(bad)?
            .verify_current(c, d)?;
        f.receipt.as_ref().ok_or_else(bad)?.verify_current(c, d)?;
        f.active.as_ref().ok_or_else(bad)?.verify_current(c, d)?;
        namespace(outer, c, d)?;
        if let Some(observer) = observer.as_deref_mut() {
            f.stage = "journal-active";
            observer.after_verified(
                super::InstallJournalPhase::Active,
                &f.active
                    .as_ref()
                    .ok_or_else(bad)?
                    .record()
                    .canonical_bytes()?,
                c,
                d,
            )?;
        }
        check(c, d)
    })();
    if let Err(error) = result {
        f.error = error;
        return Err(UserBundleFailure { state: Box::new(f) });
    }
    // These fields can only be absent after a returned error; check rather than panic.
    let Some(outer) = f.outer.take() else {
        return Err(UserBundleFailure { state: Box::new(f) });
    };
    let Some(registration) = f.registration.take() else {
        f.outer = Some(outer);
        return Err(UserBundleFailure { state: Box::new(f) });
    };
    let Some(receipt) = f.receipt.take() else {
        f.outer = Some(outer);
        f.registration = Some(registration);
        return Err(UserBundleFailure { state: Box::new(f) });
    };
    let Some(active) = f.active.take() else {
        f.outer = Some(outer);
        f.registration = Some(registration);
        f.receipt = Some(receipt);
        return Err(UserBundleFailure { state: Box::new(f) });
    };
    Ok(FreshUserBundle {
        outer,
        registration,
        receipt,
        active,
    })
}

fn make_pair(
    registration: &FreshRegistration,
    receipt: &FreshRegistrationReceipt,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<RegisteredPairInspection> {
    super::registration_pair::inspect_fresh_registered_pair(registration, receipt, c, d)
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::time::Duration;
    fn fixture() -> (tempfile::TempDir, File, BundleManifest) {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let path = dir.path().join("source");
        std::fs::write(&path, &bytes).unwrap();
        let file = File::open(path).unwrap();
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
        (dir, file, manifest)
    }
    #[test]
    fn wrong_hash_does_not_create_outer() {
        let (dir, file, mut manifest) = fixture();
        manifest.entries[0].sha256 = "0".repeat(64);
        let target = dir.path().join("bundle");
        let error = create_fresh_user_bundle(
            &manifest,
            &[("dlpad.exe", &file)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap_err();
        assert!(!error.mutation_may_have_occurred());
        assert!(!error.outer_capability_acquired());
        assert!(!error.retained_capabilities_present());
        assert!(!target.exists());
        assert_eq!(
            std::fs::metadata(dir.path().join("source")).unwrap().len(),
            70000
        );
    }
    #[test]
    fn existing_outer_never_adopted() {
        let (dir, file, manifest) = fixture();
        let target = dir.path().join("bundle");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("foreign"), b"preserve").unwrap();
        let error = create_fresh_user_bundle(
            &manifest,
            &[("dlpad.exe", &file)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(!error.mutation_may_have_occurred());
        assert!(!error.outer_capability_acquired());
        assert!(!error.retained_capabilities_present());
        assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"preserve");
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
    }
    #[test]
    fn injected_pair_failure_retains_completed_stages_and_no_active() {
        let (dir, file, manifest) = fixture();
        let target = dir.path().join("bundle");
        let error = create_with_pair_factory(
            &manifest,
            &[("dlpad.exe", &file)],
            &target,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
            &mut |_, _, _, _| {
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "injected pair failure",
                ))
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert_eq!(error.stage(), "pair");
        assert!(error.outer_capability_acquired() && error.retained_capabilities_present());
        assert!(error.state.registration.is_some() && error.state.receipt.is_some());
        assert!(!target.join("active").exists());
        drop(error);
        assert!(target.join("registered/install-registration.json").exists());
        assert!(target.join("receipt/registration-receipt.json").exists());
    }
    #[test]
    fn original_fresh_pair_converts_and_cold_policy_stays_restrictive() {
        let (dir, file, manifest) = fixture();
        let target = dir.path().join("bundle");
        let c = AtomicBool::new(false);
        let d = Instant::now() + Duration::from_secs(30);
        let bundle =
            create_fresh_user_bundle(&manifest, &[("dlpad.exe", &file)], &target, &c, d).unwrap();
        bundle.verify_current(&c, d).unwrap();
        assert_eq!(
            bundle.active().record().registration_root_identity,
            bundle.registration().record().root_identity
        );
        // Writable creation caps remain retained: cold restrictive inspection must not be weakened.
        assert!(
            super::super::inspect_registered_with_receipt(
                bundle.registration().path(),
                bundle.receipt().path(),
                &c,
                d
            )
            .is_err()
        );
        let reg = bundle.registration().path().to_owned();
        let receipt = bundle.receipt().path().to_owned();
        drop(bundle);
        let pair = super::super::inspect_registered_with_receipt(&reg, &receipt, &c, d).unwrap();
        pair.verify_current(&c, d).unwrap();
    }

    #[test]
    fn foreign_inserted_by_pair_factory_prevents_active_publication() {
        let (dir, file, manifest) = fixture();
        let target = dir.path().join("bundle");
        let c = AtomicBool::new(false);
        let d = Instant::now() + Duration::from_secs(30);
        let error = create_with_pair_factory(
            &manifest,
            &[("dlpad.exe", &file)],
            &target,
            &c,
            d,
            &mut |reg, receipt, c, d| {
                let pair = make_pair(reg, receipt, c, d)?;
                std::fs::write(target.join("foreign"), b"preserve")?;
                Ok(pair)
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.outer_capability_acquired());
        assert!(error.state.registration.is_some() && error.state.receipt.is_some());
        assert!(!target.join("active").exists());
        assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"preserve");
        drop(error);
        assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"preserve");
    }

    #[test]
    fn opaque_failure_representation_is_small() {
        assert_eq!(
            std::mem::size_of::<UserBundleFailure>(),
            std::mem::size_of::<usize>()
        );
        assert!(std::mem::size_of::<StageFailure>() <= 2 * std::mem::size_of::<usize>());
    }
}
