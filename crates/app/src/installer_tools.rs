//! Optional supply runs in the original installer worker, independent of app-bundle success.
use crate::{
    catalog as c,
    managed::{ToolId, candidate, tool_error},
    managed_candidates,
};
use dlpad_engine::tools::{ToolError, ToolManager};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

/// Compare projected canonical roots on the worker before mutating an app bundle.
pub(crate) fn tool_root_is_disjoint(data: &Path, roots: &[&Path]) -> bool {
    fn projected(path: &Path) -> Option<std::path::PathBuf> {
        if !path.is_absolute() {
            return None;
        }
        let mut current = path.to_path_buf();
        let mut missing = Vec::new();
        for _ in 0..256 {
            match std::fs::symlink_metadata(&current) {
                Ok(_) => {
                    let mut actual = current.canonicalize().ok()?;
                    for part in missing.into_iter().rev() {
                        actual.push(part);
                    }
                    #[cfg(windows)]
                    {
                        actual = std::path::PathBuf::from(actual.to_string_lossy().to_lowercase());
                    }
                    return Some(actual);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    missing.push(current.file_name()?.to_os_string());
                    if !current.pop() {
                        return None;
                    }
                }
                Err(_) => return None,
            }
        }
        None
    }
    let Some(tool) = projected(&data.join("managed-tools")) else {
        return false;
    };
    roots.iter().all(|root| {
        projected(root).is_some_and(|root| !root.starts_with(&tool) && !tool.starts_with(root))
    })
}

pub(crate) fn install_selected(
    data: &Path,
    selected: &[bool; 3],
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> String {
    if !selected.iter().any(|v| *v) {
        return String::new();
    }
    let tools = [ToolId::YtDlp, ToolId::Ffmpeg, ToolId::Deno];
    let mut manager = None;
    let mut stopped = false;
    let mut lines = Vec::with_capacity(3);
    for (tool, requested) in tools.into_iter().zip(selected) {
        if !requested {
            continue;
        }
        let result = if stopped {
            lines.push(format!("{}: {}", tool.label(), c::INSTALL_TOOL_NOT_STARTED));
            continue;
        } else if cancel.load(Ordering::Acquire) {
            Err(ToolError::Cancelled)
        } else if Instant::now() >= deadline {
            Err(ToolError::TimedOut)
        } else {
            if manager.is_none() {
                match ToolManager::new(&data.join("managed-tools")) {
                    Ok(m) => manager = Some(m),
                    Err(e) => {
                        stopped = true;
                        lines.push(format!("{}: {}", tool.label(), tool_error(e)));
                        continue;
                    }
                }
            }
            let m = manager.as_mut().expect("initialized manager");
            if tool == ToolId::YtDlp {
                match candidate() {
                    Ok((manifest, bytes)) => m
                        .download_and_install_until(manifest, bytes, cancel, deadline)
                        .map(|_| ()),
                    Err(_) => Err(ToolError::InvalidManifest),
                }
            } else {
                match managed_candidates::archive_candidate(tool) {
                    Some(package) => m
                        .download_archive_and_install_until(package, cancel, deadline)
                        .map(|_| ()),
                    None => Err(ToolError::InvalidManifest),
                }
            }
        };
        let message = match result {
            Ok(()) => c::INSTALL_TOOL_OK.to_owned(),
            Err(e) => {
                stopped = true;
                tool_error(e)
            }
        };
        lines.push(format!("{}: {}", tool.label(), message));
    }
    format!("{}\n{}", c::INSTALL_TOOL_SUMMARY, lines.join("\n"))
}
#[cfg(test)]
mod tests {
    use super::*;
    struct TestDir(std::path::PathBuf);
    impl TestDir {
        fn new() -> Self {
            static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "dlpad-installer-tools-test-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn projected_tool_root_rejects_source_bundle_and_journal_overlap_before_creation() {
        let parent = TestDir::new();
        let data = parent.path().join("data");
        assert!(tool_root_is_disjoint(
            &data,
            &[
                &parent.path().join("bundle"),
                &parent.path().join("journal"),
                &parent.path().join("source")
            ]
        ));
        assert!(!tool_root_is_disjoint(&data, &[&data]));
        assert!(!tool_root_is_disjoint(
            &data,
            &[&data.join("managed-tools").join("bundle")]
        ));
        assert!(!data.exists());
    }

    #[test]
    fn unchecked_cancelled_or_expired_supply_never_creates_user_data() {
        let parent = TestDir::new();
        let data = parent.path().join("data");
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(install_selected(&data, &[false; 3], &cancel, Instant::now()).is_empty());
        let expired = install_selected(&data, &[true; 3], &cancel, Instant::now());
        assert!(expired.contains(c::TOOL_TIMEOUT));
        assert_eq!(expired.matches(c::INSTALL_TOOL_NOT_STARTED).count(), 2);
        cancel.store(true, Ordering::Release);
        let cancelled = install_selected(&data, &[true; 3], &cancel, Instant::now());
        assert!(cancelled.contains(c::TOOL_CANCELLED));
        assert!(!data.exists());
    }
    #[test]
    fn caller_deadline_rejects_direct_and_archive_without_network_or_activation() {
        let parent = TestDir::new();
        let mut m = ToolManager::new(&parent.path().join("tools")).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let (manifest, bytes) = candidate().unwrap();
        assert!(matches!(
            m.download_and_install_until(manifest, bytes, &cancel, Instant::now()),
            Err(ToolError::TimedOut)
        ));
        if let Some(package) = managed_candidates::archive_candidate(ToolId::Deno) {
            assert!(matches!(
                m.download_archive_and_install_until(package, &cancel, Instant::now()),
                Err(ToolError::TimedOut)
            ));
        }
        assert!(m.active("yt-dlp").unwrap().is_none());
        assert!(m.active("deno").unwrap().is_none());
    }
}
