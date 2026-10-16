//! Fresh development creation with actual verified stage evidence; no crash ownership authority.
use super::{
    FreshInstallJournal, FreshUserBundle, InstallJournalPhase as Phase, JournalFailure,
    UserBundleFailure, manifest::BundleManifest,
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
/// Invoked by the existing source-pinned transaction/coordinator, never by a post-hoc loop.
pub(super) trait BundleStageObserver {
    /// After all input/source verification, immediately before first outer creation attempt.
    fn before_outer(&mut self, c: &AtomicBool, d: Instant) -> io::Result<()>;
    /// After the corresponding original stage verify, before advancing the next stage.
    fn after_verified(
        &mut self,
        phase: Phase,
        canonical: &[u8],
        c: &AtomicBool,
        d: Instant,
    ) -> io::Result<()>;
}
struct Recorder<'a> {
    manifest: &'a BundleManifest,
    bundle: &'a Path,
    journal: &'a Path,
    writer: Option<FreshInstallJournal>,
    failure: Option<JournalFailure>,
    #[cfg(test)]
    fail_after: Option<usize>,
    #[cfg(test)]
    cancel_after: Option<usize>,
}
impl BundleStageObserver for Recorder<'_> {
    fn before_outer(&mut self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        if self.writer.is_some() || self.failure.is_some() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        match FreshInstallJournal::create(self.manifest, self.bundle, self.journal, c, d) {
            Ok(v) => {
                self.writer = Some(v);
                Ok(())
            }
            Err(e) => {
                let kind = e.kind();
                self.failure = Some(e);
                Err(kind.into())
            }
        }
    }
    fn after_verified(
        &mut self,
        phase: Phase,
        canonical: &[u8],
        c: &AtomicBool,
        d: Instant,
    ) -> io::Result<()> {
        if canonical.len() > 65536 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let writer = self.writer.take().ok_or(io::ErrorKind::InvalidData)?;
        let sha = format!("{:x}", Sha256::digest(canonical));
        let phase_index = match &phase {
            Phase::Prepared => 1,
            Phase::Registered => 2,
            Phase::Receipt => 3,
            Phase::Active => 4,
        };
        #[cfg(not(test))]
        let _ = phase_index;
        match writer.record_phase(phase, &sha, c, d) {
            Ok(v) => {
                self.writer = Some(v);
                #[cfg(test)]
                if self.cancel_after == Some(phase_index) {
                    c.store(true, Ordering::Release);
                }
                #[cfg(test)]
                if self.fail_after == Some(phase_index) {
                    return Err(io::Error::other("synthetic after verified journal phase"));
                }
                Ok(())
            }
            Err(e) => {
                let kind = e.kind();
                self.failure = Some(e);
                Err(kind.into())
            }
        }
    }
}
pub struct FreshJournalAwareBundle {
    bundle: FreshUserBundle,
    _journal: FreshInstallJournal,
}
impl FreshJournalAwareBundle {
    pub fn bundle(&self) -> &FreshUserBundle {
        &self.bundle
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        self.bundle.verify_current(c, d)?;
        self._journal.verify_current(c, d)?;
        self.bundle.verify_current(c, d)
    }
}
pub struct JournalBundleFailure {
    state: Box<FailureState>,
}
struct FailureState {
    error: io::Error,
    bundle_failure: Option<UserBundleFailure>,
    completed_bundle: Option<FreshUserBundle>,
    journal: Option<FreshInstallJournal>,
    journal_failure: Option<JournalFailure>,
}
impl std::fmt::Debug for JournalBundleFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JournalBundleFailure(<redacted>)")
    }
}
impl JournalBundleFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.state.error.kind()
    }
    pub fn bundle_capabilities_present(&self) -> bool {
        self.state.completed_bundle.is_some()
            || self
                .state
                .bundle_failure
                .as_ref()
                .is_some_and(UserBundleFailure::retained_capabilities_present)
    }
    pub fn journal_capabilities_present(&self) -> bool {
        self.state.journal.is_some()
            || self
                .state
                .journal_failure
                .as_ref()
                .is_some_and(|e| e.root_capability_acquired() || e.retained_file_cap_count() != 0)
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.state.completed_bundle.is_some()
            || self.state.journal.is_some()
            || self
                .state
                .bundle_failure
                .as_ref()
                .is_some_and(UserBundleFailure::mutation_may_have_occurred)
            || self
                .state
                .journal_failure
                .as_ref()
                .is_some_and(JournalFailure::mutation_may_have_occurred)
    }
}
/// Records each actual verified stage; all original capabilities remain retained.
pub fn create_journal_aware_bundle(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    bundle_target: &Path,
    journal_target: &Path,
    c: &AtomicBool,
    d: Instant,
) -> Result<FreshJournalAwareBundle, JournalBundleFailure> {
    create_with_fault(manifest, files, bundle_target, journal_target, c, d, None)
}
fn create_with_fault(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    bundle_target: &Path,
    journal_target: &Path,
    c: &AtomicBool,
    d: Instant,
    fail_after: Option<usize>,
) -> Result<FreshJournalAwareBundle, JournalBundleFailure> {
    #[cfg(test)]
    {
        create_with_control(
            manifest,
            files,
            bundle_target,
            journal_target,
            c,
            d,
            fail_after,
            None,
        )
    }
    #[cfg(not(test))]
    {
        create_with_control(
            manifest,
            files,
            bundle_target,
            journal_target,
            c,
            d,
            fail_after,
        )
    }
}
#[cfg_attr(test, allow(clippy::too_many_arguments))]
fn create_with_control(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    bundle_target: &Path,
    journal_target: &Path,
    c: &AtomicBool,
    d: Instant,
    fail_after: Option<usize>,
    #[cfg(test)] cancel_after: Option<usize>,
) -> Result<FreshJournalAwareBundle, JournalBundleFailure> {
    #[cfg(not(test))]
    let _ = fail_after;
    let destinations = (|| {
        if c.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= d {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let bundle = super::windows_caps::FreshDestination::observe(bundle_target)?;
        let journal = super::windows_caps::FreshDestination::observe(journal_target)?;
        bundle.verify()?;
        journal.verify()?;
        Ok::<_, io::Error>((bundle, journal))
    })();
    let (_bundle_destination, _journal_destination) = match destinations {
        Ok(v) => v,
        Err(error) => {
            return Err(JournalBundleFailure {
                state: Box::new(FailureState {
                    error,
                    bundle_failure: None,
                    completed_bundle: None,
                    journal: None,
                    journal_failure: None,
                }),
            });
        }
    };
    let mut recorder = Recorder {
        manifest,
        bundle: bundle_target,
        journal: journal_target,
        writer: None,
        failure: None,
        #[cfg(test)]
        fail_after,
        #[cfg(test)]
        cancel_after,
    };
    let result = super::user_bundle::create_with_stage_observer(
        manifest,
        files,
        bundle_target,
        c,
        d,
        &mut recorder,
    );
    match result {
        Ok(bundle) => match recorder.writer.take() {
            Some(journal) => {
                let value = FreshJournalAwareBundle {
                    bundle,
                    _journal: journal,
                };
                match value.verify_current(c, d) {
                    Ok(()) => Ok(value),
                    Err(error) => Err(JournalBundleFailure {
                        state: Box::new(FailureState {
                            error,
                            bundle_failure: None,
                            completed_bundle: Some(value.bundle),
                            journal: Some(value._journal),
                            journal_failure: recorder.failure,
                        }),
                    }),
                }
            }
            None => Err(JournalBundleFailure {
                state: Box::new(FailureState {
                    error: io::ErrorKind::InvalidData.into(),
                    bundle_failure: None,
                    completed_bundle: Some(bundle),
                    journal: None,
                    journal_failure: recorder.failure,
                }),
            }),
        },
        Err(error) => {
            let kind = error.kind();
            Err(JournalBundleFailure {
                state: Box::new(FailureState {
                    error: kind.into(),
                    bundle_failure: Some(error),
                    completed_bundle: None,
                    journal: recorder.writer,
                    journal_failure: recorder.failure,
                }),
            })
        }
    }
}

// Paste inside journal_bundle module after root adds actual controlled stage seams.
// No execution performed; these call production coordinator, not copied state logic.
#[cfg(test)]
mod journal_bundle_tests {
    use super::*;
    use std::{fs, sync::atomic::AtomicBool, time::Duration};
    fn fixture() -> (tempfile::TempDir, File, BundleManifest) {
        let temp = tempfile::tempdir().unwrap();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let path = temp.path().join("source");
        fs::write(&path, &bytes).unwrap();
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
        (temp, file, manifest)
    }
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    #[test]
    fn preflight_cancel_deadline_wrong_sha_create_neither_namespace() {
        for case in ["cancel", "deadline", "wrong-sha"] {
            let (temp, file, mut manifest) = fixture();
            let source_before = fs::read(temp.path().join("source")).unwrap();
            let bundle = temp.path().join("bundle");
            let journal = temp.path().join("journal");
            if case == "wrong-sha" {
                manifest.entries[0].sha256 = "0".repeat(64);
            }
            let c = AtomicBool::new(case == "cancel");
            let d = if case == "deadline" {
                Instant::now()
            } else {
                later()
            };
            let failure = create_journal_aware_bundle(
                &manifest,
                &[("dlpad.exe", &file)],
                &bundle,
                &journal,
                &c,
                d,
            )
            .err()
            .unwrap();
            assert!(!failure.mutation_may_have_occurred());
            assert!(!failure.bundle_capabilities_present());
            assert!(!failure.journal_capabilities_present());
            assert!(!bundle.exists());
            assert!(!journal.exists());
            assert_eq!(file.metadata().unwrap().len(), 70000);
            assert_eq!(fs::read(temp.path().join("source")).unwrap(), source_before);
        }
    }
    #[test]
    fn existing_either_root_preserves_foreign_and_other_target_absent() {
        for existing in ["bundle", "journal"] {
            let (temp, file, manifest) = fixture();
            let source_before = fs::read(temp.path().join("source")).unwrap();
            let bundle = temp.path().join("bundle");
            let journal = temp.path().join("journal");
            let chosen = if existing == "bundle" {
                &bundle
            } else {
                &journal
            };
            fs::create_dir(chosen).unwrap();
            fs::write(chosen.join("foreign"), b"preserve").unwrap();
            let failure = create_journal_aware_bundle(
                &manifest,
                &[("dlpad.exe", &file)],
                &bundle,
                &journal,
                &AtomicBool::new(false),
                later(),
            )
            .err()
            .unwrap();
            assert_eq!(failure.kind(), io::ErrorKind::AlreadyExists);
            assert!(!failure.mutation_may_have_occurred());
            assert_eq!(fs::read(chosen.join("foreign")).unwrap(), b"preserve");
            assert_eq!(fs::read_dir(chosen).unwrap().count(), 1);
            assert_eq!(fs::read(temp.path().join("source")).unwrap(), source_before);
            assert!(
                !(if existing == "bundle" {
                    journal
                } else {
                    bundle
                })
                .exists()
            );
        }
    }
    #[test]
    fn fault_after_each_actual_verified_phase_retains_both_and_no_later_evidence() {
        for phase in 1..=4 {
            let (temp, file, manifest) = fixture();
            let source_before = fs::read(temp.path().join("source")).unwrap();
            let bundle = temp.path().join("bundle");
            let journal = temp.path().join("journal");
            let failure = create_with_fault(
                &manifest,
                &[("dlpad.exe", &file)],
                &bundle,
                &journal,
                &AtomicBool::new(false),
                later(),
                Some(phase),
            )
            .err()
            .unwrap();
            assert_eq!(failure.kind(), io::ErrorKind::Other);
            assert!(failure.mutation_may_have_occurred());
            assert!(failure.bundle_capabilities_present());
            assert!(failure.journal_capabilities_present());
            // Drop only releases originals; no cleanup. Cold evidence then proves persisted exact prefix.
            drop(failure);
            let inspection =
                super::super::inspect_install_journal(&journal, &AtomicBool::new(false), later())
                    .unwrap();
            assert_eq!(inspection.evidence().len(), phase);
            assert_eq!(fs::read_dir(&journal).unwrap().count(), phase + 1);
            assert_eq!(file.metadata().unwrap().len(), 70000);
            assert_eq!(fs::read(temp.path().join("source")).unwrap(), source_before);
            assert_eq!(bundle.join("receipt").exists(), phase >= 3);
            assert_eq!(bundle.join("active").exists(), phase >= 4);
            assert_eq!(
                bundle.join("registered/install-registration.json").exists(),
                phase >= 2
            );
        }
    }
    #[test]
    fn actual_success_has_all_phases_and_matches_explicit_cold_bundle() {
        let (temp, file, manifest) = fixture();
        let bundle = temp.path().join("bundle");
        let journal = temp.path().join("journal");
        let c = AtomicBool::new(false);
        let deadline = later();
        let live = create_journal_aware_bundle(
            &manifest,
            &[("dlpad.exe", &file)],
            &bundle,
            &journal,
            &c,
            deadline,
        )
        .ok()
        .unwrap();
        live.verify_current(&c, deadline).unwrap();
        assert_eq!(live.bundle().path(), bundle);
        drop(live);
        let records = super::super::inspect_install_journal(&journal, &c, deadline).unwrap();
        assert_eq!(records.evidence().len(), 4);
        let current = super::super::inspect_user_bundle(&bundle, &c, deadline).unwrap();
        records
            .verify_active_bundle(&current, &c, deadline)
            .unwrap();
    }
    #[test]
    fn cancellation_after_actual_durable_phase_uses_next_coordinator_checkpoint() {
        for phase in 1..=4 {
            let (temp, file, manifest) = fixture();
            let original = fs::read(temp.path().join("source")).unwrap();
            let bundle = temp.path().join("bundle");
            let journal = temp.path().join("journal");
            let cancel = AtomicBool::new(false);
            let error = create_with_control(
                &manifest,
                &[("dlpad.exe", &file)],
                &bundle,
                &journal,
                &cancel,
                later(),
                None,
                Some(phase),
            )
            .err()
            .unwrap();
            assert!(cancel.load(Ordering::Acquire));
            assert_eq!(error.kind(), io::ErrorKind::Interrupted);
            assert!(error.bundle_capabilities_present());
            assert!(error.journal_capabilities_present());
            assert!(error.mutation_may_have_occurred());
            drop(error);
            // Fresh readonly cancellation state, never clear the cancelled operation's flag.
            let readonly = AtomicBool::new(false);
            let inspection =
                super::super::inspect_install_journal(&journal, &readonly, later()).unwrap();
            assert_eq!(inspection.evidence().len(), phase);
            inspection.verify_current(&readonly, later()).unwrap();
            assert_eq!(fs::read_dir(&journal).unwrap().count(), phase + 1);
            assert_eq!(
                bundle.join("registered/install-registration.json").exists(),
                phase >= 2
            );
            assert_eq!(bundle.join("receipt").exists(), phase >= 3);
            assert_eq!(bundle.join("active").exists(), phase >= 4);
            assert_eq!(fs::read(temp.path().join("source")).unwrap(), original);
        }
    }
}
