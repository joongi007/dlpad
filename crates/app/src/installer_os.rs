//! Worker-only Windows integration bridge. Filesystem verification is in engine.
use dlpad_engine::app_install::{
    LocalInspectionRequest, os_registration::register_local_windows_shell,
};
use std::{io, path::Path, sync::atomic::AtomicBool, time::Instant};
pub fn register(
    bundle: &Path,
    journal: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), io::ErrorKind> {
    register_local_windows_shell(
        &LocalInspectionRequest {
            bundle_root: bundle.to_path_buf(),
            journal_root: journal.to_path_buf(),
        },
        cancel,
        deadline,
    )
    .map(|_| ())
    .map_err(|error| error.kind())
}

/// Legacy unregistered bundles retain their existing explicit removal path.
/// Registered bundles use Windows Installed apps and its retry-capable uninstaller.
pub fn remove(
    bundle: &Path,
    journal: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> dlpad_engine::app_install::LocalRemovalReport {
    use dlpad_engine::app_install::{
        LocalRemovalReport, LocalRemovalRequest, os_registration::inspect_local_windows_shell,
        remove_local_journal_bundle,
    };
    let request = LocalInspectionRequest {
        bundle_root: bundle.to_path_buf(),
        journal_root: journal.to_path_buf(),
    };
    match inspect_local_windows_shell(&request, cancel, deadline) {
        Ok(_) => {
            // Registered installations use the dedicated retry-capable uninstaller.
            // Never remove the bundle then discard a shell cleanup capability.
            return LocalRemovalReport {
                kind: Some(io::ErrorKind::Unsupported),
                mutation_may_have_occurred: false,
                accepted_file_dispositions: 0,
                accepted_root_dispositions: 0,
                outer_namespace_absence_confirmed: false,
            };
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return LocalRemovalReport {
                kind: Some(error.kind()),
                mutation_may_have_occurred: false,
                accepted_file_dispositions: 0,
                accepted_root_dispositions: 0,
                outer_namespace_absence_confirmed: false,
            };
        }
    }

    remove_local_journal_bundle(
        &LocalRemovalRequest {
            bundle_root: request.bundle_root,
            journal_root: request.journal_root,
        },
        cancel,
        deadline,
    )
}

/// Read-only observation; failures never grant legacy removal authority.
pub fn is_registered(
    request: &LocalInspectionRequest,
    cancel: &AtomicBool,
    deadline: Instant,
) -> bool {
    dlpad_engine::app_install::inspect_local_windows_shell(request, cancel, deadline).is_ok()
}
