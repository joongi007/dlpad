//! Read-only per-user install suggestion; no preparation or mutation authority.
#[cfg(windows)]
pub(crate) fn recommended_target() -> Option<std::path::PathBuf> {
    dlpad_engine::location::default_install_target_hint()
}
