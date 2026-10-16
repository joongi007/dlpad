//! Bounded journal generation hints. Every operation still revalidates fresh/native roots.
use std::{
    io,
    path::{Path, PathBuf},
};
pub(crate) const LIMIT: usize = 32;
pub(crate) fn choose(
    target: &Path,
    base: &Path,
    mut exists: impl FnMut(&Path) -> io::Result<bool>,
    mut matches: impl FnMut(&Path) -> io::Result<bool>,
) -> io::Result<PathBuf> {
    let installed = exists(target)?;
    let parent = base.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let name = base.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    let mut matched = None;
    for generation in 0..LIMIT {
        let candidate = if generation == 0 {
            base.to_path_buf()
        } else {
            let mut leaf = name.to_os_string();
            leaf.push(format!("-{generation}"));
            parent.join(leaf)
        };
        let present = exists(&candidate)?;
        if installed {
            if present && matches(&candidate)? {
                if matched.is_some() {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                matched = Some(candidate);
            }
        } else if !present {
            return Ok(candidate);
        }
    }
    if let Some(journal) = matched {
        return Ok(journal);
    }
    Err(if installed {
        io::ErrorKind::InvalidData
    } else {
        io::ErrorKind::AlreadyExists
    }
    .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "dlpad-journal-choice-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        root
    }
    fn exists(p: &Path) -> io::Result<bool> {
        match std::fs::symlink_metadata(p) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    #[test]
    fn removed_target_keeps_original_journal_and_chooses_new_generation() {
        let root = fixture();
        let target = root.join("app");
        let base = root.join("journal");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(&base, b"original receipt").unwrap();
        std::fs::remove_dir(&target).unwrap();
        assert_eq!(
            choose(&target, &base, exists, |_| panic!(
                "absent target is not inspected"
            ))
            .unwrap(),
            root.join("journal-1")
        );
        assert_eq!(std::fs::read(&base).unwrap(), b"original receipt");
        std::fs::remove_file(base).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn restart_finds_matching_current_generation_without_adopting_old_record() {
        let root = fixture();
        let target = root.join("app");
        let base = root.join("journal");
        let current = root.join("journal-2");
        std::fs::create_dir(&target).unwrap();
        for p in [&base, &root.join("journal-1"), &current] {
            std::fs::write(p, b"preserve").unwrap();
        }
        assert_eq!(
            choose(&target, &base, exists, |p| Ok(p == current)).unwrap(),
            current
        );
        for p in [&base, &root.join("journal-1"), &current] {
            assert_eq!(std::fs::read(p).unwrap(), b"preserve");
            std::fs::remove_file(p).unwrap();
        }
        std::fs::remove_dir(target).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn multiple_matches_are_ambiguous_and_never_authorize_a_selection() {
        let root = fixture();
        let target = root.join("app");
        let base = root.join("journal");
        assert_eq!(
            choose(
                &target,
                &base,
                |p| Ok(p == target || p == base || p == root.join("journal-1")),
                |_| Ok(true)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidData
        );
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn bounded_exhaustion_and_cancel_never_issue_a_path() {
        let base = Path::new("/journal");
        let target = Path::new("/app");
        let mut calls = 0;
        assert_eq!(
            choose(
                target,
                base,
                |p| {
                    calls += 1;
                    Ok(p != target)
                },
                |_| Ok(false)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(calls, LIMIT + 1);
        assert_eq!(
            choose(
                target,
                base,
                |_| Err(io::ErrorKind::Interrupted.into()),
                |_| Ok(false)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(
            choose(target, base, |_| Ok(true), |_| Ok(false))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
