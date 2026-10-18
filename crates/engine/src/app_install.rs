//! Local app-installation foundations, separate from managed external tools.
//!
//! A valid manifest describes bytes; it does not establish publisher trust,
//! authorize replacement of an existing installation, or grant deletion rights.
//! Blocking verification and filesystem operations belong off the UI thread.
#[cfg(windows)]
mod active_schema;
pub mod manifest;
pub mod payload;
#[cfg(windows)]
pub use active_schema::ActiveSelectionRecord;
#[cfg(windows)]
mod active_launch;
#[cfg(windows)]
pub use active_launch::{
    OwnedSelectedProcess, SelectedExitReason, SelectedLaunchFailure, SelectedProcessExit,
    SelectedWaitFailure, launch_explicit_selection,
};
#[cfg(windows)]
mod active_selection;
#[cfg(windows)]
pub use active_selection::{
    ActiveSelectionFailure, ActiveSelectionInspection, FreshActiveSelection,
    inspect_active_selection, select_registered_pair,
};
#[cfg(windows)]
mod bundle_removal;
#[cfg(windows)]
mod windows_caps;
#[cfg(windows)]
pub use bundle_removal::{
    BundleRemovalFailure, BundleRemovalOutcome, ExplicitBundleRemoval,
    authorize_explicit_user_bundle_removal,
};

#[cfg(windows)]
pub use windows_caps::FreshRoot;

#[cfg(windows)]
mod transaction;
#[cfg(windows)]
pub use transaction::prepare_new;

/// A prepared local copy is neither an active installation nor publisher trust.
#[cfg(windows)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedRecord {
    pub schema: u32,
    pub domain: String,
    pub root_identity: crate::file_identity::NativeFileIdentity,
    pub version: String,
    pub target: String,
    pub development_only: bool,
    pub trusted_provenance: bool,
    pub entries: Vec<PreparedEntry>,
}
#[cfg(windows)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedEntry {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
    pub identity: crate::file_identity::NativeFileIdentity,
}

/// Original capabilities remain private; no cleanup runs when this value drops.
#[cfg(windows)]
pub struct PreparedBundle {
    root: FreshRoot,
    files: Vec<windows_caps::CreatedFile>,
    marker: windows_caps::CreatedFile,
    record: PreparedRecord,
}
#[cfg(windows)]
impl PreparedBundle {
    pub fn record(&self) -> &PreparedRecord {
        &self.record
    }
    pub fn path(&self) -> &std::path::Path {
        self.root.path()
    }
    pub fn verify_current(&self) -> std::io::Result<()> {
        transaction::verify_current(self)
    }
}
#[cfg(windows)]
impl std::fmt::Debug for PreparedBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedBundle")
            .field("entries", &self.record.entries.len())
            .finish_non_exhaustive()
    }
}

/// Failed preparation preserves partial objects. A path hint grants no authority.
#[cfg(windows)]
pub struct PrepareFailure {
    error: std::io::Error,
    target_hint: Option<std::path::PathBuf>,
    root: Option<Box<FreshRoot>>,
    files: Vec<windows_caps::CreatedFile>,
}
#[cfg(windows)]
impl PrepareFailure {
    pub fn kind(&self) -> std::io::ErrorKind {
        self.error.kind()
    }
    pub fn target_hint(&self) -> Option<&std::path::Path> {
        self.target_hint.as_deref()
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.target_hint.is_some()
    }
    pub fn root_capability_acquired(&self) -> bool {
        self.root.is_some()
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.files.len()
    }
}
#[cfg(windows)]
impl std::fmt::Debug for PrepareFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrepareFailure")
            .field("kind", &self.kind())
            .field("root_capability_acquired", &self.root_capability_acquired())
            .field("retained_file_caps", &self.retained_file_cap_count())
            .finish()
    }
}
#[cfg(windows)]
impl std::fmt::Display for PrepareFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "app preparation failed ({:?}); partial objects are preserved",
            self.kind()
        )
    }
}
#[cfg(windows)]
impl std::error::Error for PrepareFailure {}

/// Explicit local-byte inspection; it grants no installation or deletion rights.
#[cfg(windows)]
mod reopen;
#[cfg(windows)]
pub use reopen::inspect_prepared;

#[cfg(windows)]
pub struct PreparedInspection {
    root: windows_caps::ReadRoot,
    files: Vec<windows_caps::ReadFile>,
    marker: windows_caps::ReadFile,
    record: PreparedRecord,
}
#[cfg(windows)]
impl PreparedInspection {
    pub fn record(&self) -> &PreparedRecord {
        &self.record
    }
    pub fn path(&self) -> &std::path::Path {
        self.root.path()
    }
    pub fn verify_current(
        &self,
        cancel: &std::sync::atomic::AtomicBool,
        deadline: std::time::Instant,
    ) -> std::io::Result<()> {
        reopen::verify_current(self, cancel, deadline)
    }
    /// Borrow the original read pin; the validated record has exactly one dlpad.exe.
    pub fn executable(&self) -> &std::fs::File {
        let index = self
            .record
            .entries
            .iter()
            .position(|e| e.name == "dlpad.exe")
            .expect("private validated inspection always has dlpad.exe");
        self.files[index].file()
    }
}
#[cfg(windows)]
impl std::fmt::Debug for PreparedInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedInspection")
            .field("entries", &self.record.entries.len())
            .finish_non_exhaustive()
    }
}

#[cfg(windows)]
mod removal;
#[cfg(windows)]
pub use removal::discard_fresh;

/// Accepted original-object dispositions, not secure erasure or cold uninstall.
#[cfg(windows)]
#[derive(Debug, serde::Serialize)]
pub struct DiscardOutcome {
    pub accepted_file_dispositions: usize,
    pub root_disposition_accepted: bool,
}
#[cfg(windows)]
struct DiscardRemainder {
    root: FreshRoot,
    files: Vec<windows_caps::CreatedFile>,
    marker: Option<windows_caps::CreatedFile>,
    record: PreparedRecord,
    accepted_file_dispositions: usize,
}
#[cfg(windows)]
pub struct DiscardFailure {
    error: std::io::Error,
    remainder: Box<DiscardRemainder>,
}
#[cfg(windows)]
impl DiscardFailure {
    pub fn kind(&self) -> std::io::ErrorKind {
        self.error.kind()
    }
    pub fn accepted_file_dispositions(&self) -> usize {
        self.remainder.accepted_file_dispositions
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.remainder.files.len() + usize::from(self.remainder.marker.is_some())
    }
    pub fn original_record(&self) -> &PreparedRecord {
        &self.remainder.record
    }
    pub fn path_hint(&self) -> &std::path::Path {
        self.remainder.root.path()
    }
    pub fn retry(
        self,
        cancel: &std::sync::atomic::AtomicBool,
        deadline: std::time::Instant,
    ) -> Result<DiscardOutcome, DiscardFailure> {
        removal::discard_remaining(*self.remainder, cancel, deadline)
    }
}
#[cfg(windows)]
impl std::fmt::Debug for DiscardFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscardFailure")
            .field("kind", &self.kind())
            .field(
                "accepted_file_dispositions",
                &self.accepted_file_dispositions(),
            )
            .field("retained_file_caps", &self.retained_file_cap_count())
            .finish_non_exhaustive()
    }
}
#[cfg(windows)]
impl std::fmt::Display for DiscardFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fresh app discard failed ({:?}); original remainder preserved",
            self.kind()
        )
    }
}
#[cfg(windows)]
impl std::error::Error for DiscardFailure {}

#[cfg(windows)]
mod registration;
#[cfg(windows)]
mod registration_schema;
#[cfg(windows)]
pub use registration::{FreshRegistration, RegistrationFailure, register_fresh};
#[cfg(windows)]
pub use registration_schema::{RegistrationRecord, RegistrationSeal};

#[cfg(windows)]
mod registration_reopen;
#[cfg(windows)]
pub use registration_reopen::{RegisteredInspection, inspect_registered};

#[cfg(windows)]
mod receipt_schema;
#[cfg(windows)]
pub use receipt_schema::ReceiptRecord;

#[cfg(windows)]
mod receipt;
#[cfg(windows)]
pub use receipt::{FreshRegistrationReceipt, ReceiptFailure, create_registration_receipt};

#[cfg(windows)]
mod receipt_reopen;
#[cfg(windows)]
pub use receipt_reopen::{ReceiptInspection, inspect_registration_receipt};

#[cfg(windows)]
mod registration_pair;
#[cfg(windows)]
pub use registration_pair::{RegisteredPairInspection, inspect_registered_with_receipt};

#[cfg(windows)]
mod registered_removal;
#[cfg(windows)]
pub use registered_removal::{
    RegisteredRemoval, RegisteredRemovalFailure, RegisteredRemovalOutcome,
    RemovalAuthorizationFailure, authorize_explicit_registered_removal,
};

#[cfg(windows)]
mod user_bundle;
#[cfg(windows)]
pub use user_bundle::{FreshUserBundle, UserBundleFailure, create_fresh_user_bundle};

#[cfg(windows)]
mod user_bundle_reopen;
#[cfg(windows)]
pub use user_bundle_reopen::{UserBundleInspection, inspect_user_bundle};

#[cfg(windows)]
mod install_journal;
#[cfg(windows)]
pub use install_journal::{
    FreshInstallJournal, InstallIntent, InstallJournalInspection, JournalFailure,
    Phase as InstallJournalPhase, PhaseEvidence as InstallPhaseEvidence, inspect_install_journal,
};

#[cfg(windows)]
mod journal_bundle;
#[cfg(windows)]
pub use journal_bundle::{
    FreshJournalAwareBundle, JournalBundleFailure, create_journal_aware_bundle,
};

#[cfg(windows)]
mod local_prepare;
#[cfg(windows)]
pub use local_prepare::{
    LocalInspectionRequest, LocalJournalReport, LocalJournalRequest, inspect_local_existing_bundle,
    inspect_local_journal_bundle, prepare_local_journal_bundle,
};

#[cfg(windows)]
mod local_remove;
#[cfg(windows)]
pub use local_remove::{LocalRemovalReport, LocalRemovalRequest, remove_local_journal_bundle};

#[cfg(windows)]
mod update_prepare;
#[cfg(windows)]
pub use update_prepare::{
    PreparedUpdateBundle, UpdatePreparationError, UpdatePreparationFailure,
    UpdatePreparationRequest, prepare_staged_update,
};

/// Windows installed-app/Start Menu integration; distinct from bundle registration.
#[cfg(windows)]
pub mod os_registration;

#[cfg(windows)]
pub use os_registration::{
    OwnedUninstallCopy, ShellCleanupFailure, ShellCleanupOutcome, ShellRegistration,
    ShellRegistrationInspection, inspect_local_windows_shell, prepare_current_uninstall_copy,
    register_local_windows_shell, verify_uninstall_handoff_files,
};
