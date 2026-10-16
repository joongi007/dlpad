//! Registered app reuses its headless entry; independent engine fixtures require a guard.
use std::{
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};
static SELF_GUARDIAN: OnceLock<PathBuf> = OnceLock::new();
pub(super) fn register() -> io::Result<()> {
    let current = std::env::current_exe()?;
    if !current.is_absolute() || !current.is_file() {
        return Err(io::Error::other(
            "Windows guardian registration unavailable",
        ));
    }
    if let Some(registered) = SELF_GUARDIAN.get() {
        if registered != &current {
            return Err(io::Error::other("Windows guardian registration changed"));
        }
        return Ok(());
    }
    if SELF_GUARDIAN.set(current.clone()).is_err() && SELF_GUARDIAN.get() != Some(&current) {
        return Err(io::Error::other("Windows guardian registration changed"));
    }
    Ok(())
}
fn candidates(current: &Path) -> io::Result<Vec<PathBuf>> {
    let directory = current
        .parent()
        .ok_or_else(|| io::Error::other("Windows executable directory unavailable"))?;
    let mut paths = vec![directory.join("dlpad-process-guard.exe")];
    if directory
        .file_name()
        .is_some_and(|name| name == "deps" || name == "examples")
    {
        let build = directory
            .parent()
            .ok_or_else(|| io::Error::other("Windows build directory unavailable"))?;
        paths.push(build.join("dlpad-process-guard.exe"));
    }
    Ok(paths)
}
pub(super) fn executable() -> io::Result<PathBuf> {
    if let Some(registered) = SELF_GUARDIAN.get() {
        if registered.is_file() {
            return Ok(registered.clone());
        }
        return Err(io::Error::other("Registered Windows guardian unavailable"));
    }
    for path in candidates(&std::env::current_exe()?)? {
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Windows guardian missing: build workspace or register app entry",
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_candidates_stay_in_original_build_directory() {
        let base = std::env::temp_dir().join("synthetic-build");
        assert_eq!(
            candidates(&base.join("deps").join("engine-test.exe")).unwrap(),
            vec![
                base.join("deps").join("dlpad-process-guard.exe"),
                base.join("dlpad-process-guard.exe")
            ]
        );
        assert_eq!(
            candidates(&base.join("app.exe")).unwrap(),
            vec![base.join("dlpad-process-guard.exe")]
        );
        assert!(candidates(Path::new("/")).is_err());
    }
}
