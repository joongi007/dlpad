//! Blocking explicit removal service; UI receives facts after local capabilities drop.
#![cfg(windows)]
use super::{BundleRemovalFailure, authorize_explicit_user_bundle_removal};
use std::{
    io,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
pub struct LocalRemovalRequest {
    pub bundle_root: PathBuf,
    pub journal_root: PathBuf,
}
pub struct LocalRemovalReport {
    pub kind: Option<io::ErrorKind>,
    pub mutation_may_have_occurred: bool,
    pub accepted_file_dispositions: usize,
    pub accepted_root_dispositions: usize,
    pub outer_namespace_absence_confirmed: bool,
}
impl LocalRemovalReport {
    fn failed(error: &BundleRemovalFailure) -> Self {
        Self {
            kind: Some(error.kind()),
            mutation_may_have_occurred: error.mutation_may_have_occurred(),
            accepted_file_dispositions: error.accepted_file_dispositions(),
            accepted_root_dispositions: error.accepted_root_dispositions(),
            outer_namespace_absence_confirmed: false,
        }
    }
}
pub fn remove_local_journal_bundle(
    request: &LocalRemovalRequest,
    cancel: &AtomicBool,
    deadline: Instant,
) -> LocalRemovalReport {
    let validation = (|| -> io::Result<()> {
        if cancel.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        super::local_prepare::validate(&request.bundle_root)?;
        super::local_prepare::validate(&request.journal_root)
    })();
    if let Err(error) = validation {
        return LocalRemovalReport {
            kind: Some(error.kind()),
            mutation_may_have_occurred: false,
            accepted_file_dispositions: 0,
            accepted_root_dispositions: 0,
            outer_namespace_absence_confirmed: false,
        };
    }
    let removal = match authorize_explicit_user_bundle_removal(
        &request.bundle_root,
        &request.journal_root,
        cancel,
        deadline,
    ) {
        Ok(value) => value,
        Err(error) => return LocalRemovalReport::failed(&error),
    };
    match removal.remove(cancel, deadline) {
        Ok(outcome) => LocalRemovalReport {
            kind: None,
            mutation_may_have_occurred: true,
            accepted_file_dispositions: outcome.accepted_file_dispositions,
            accepted_root_dispositions: outcome.accepted_root_dispositions,
            outer_namespace_absence_confirmed: outcome.outer_namespace_absence_confirmed,
        },
        Err(error) => LocalRemovalReport::failed(&error),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn precontrol_invalid_and_missing_requests_never_create_or_dispose_files() {
        let temp = tempfile::tempdir().unwrap();
        let request = LocalRemovalRequest {
            bundle_root: temp.path().join("missing-bundle"),
            journal_root: temp.path().join("missing-journal"),
        };
        for (cancel, deadline, kind) in [
            (
                true,
                Instant::now() + Duration::from_secs(30),
                io::ErrorKind::Interrupted,
            ),
            (false, Instant::now(), io::ErrorKind::TimedOut),
            (
                false,
                Instant::now() + Duration::from_secs(30),
                io::ErrorKind::NotFound,
            ),
        ] {
            let report = remove_local_journal_bundle(&request, &AtomicBool::new(cancel), deadline);
            assert_eq!(report.kind, Some(kind));
            assert!(!report.mutation_may_have_occurred);
            assert_eq!(report.accepted_file_dispositions, 0);
            assert_eq!(report.accepted_root_dispositions, 0);
            assert!(!report.outer_namespace_absence_confirmed);
            assert!(!request.bundle_root.exists() && !request.journal_root.exists());
        }
        let invalid = LocalRemovalRequest {
            bundle_root: "relative".into(),
            journal_root: request.journal_root,
        };
        let report = remove_local_journal_bundle(
            &invalid,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(30),
        );
        assert_eq!(report.kind, Some(io::ErrorKind::InvalidInput));
        assert!(!report.mutation_may_have_occurred);
    }
}
