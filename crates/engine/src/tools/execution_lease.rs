//! Shared mutation exclusion foundation; caller must cover the entire reader lifetime.
use super::*;
/// Closing this capability releases only this descriptor. Never explicitly unlock
/// an open-file description that an independent guardian may still hold.
pub struct ExecutionLease {
    _file: File,
}
/// Durable management exclusion. Drop preserves the pending marker.
/// Finish only after the execution adapter confirms reader cleanup.
pub struct ManagedExecution {
    marker: super::execution_marker::ExecutionMarker,
    _lease: ExecutionLease,
    #[cfg(windows)]
    journal: std::sync::Mutex<super::windows_managed_journal::ManagedJournal>,
}
impl ManagedExecution {
    #[cfg(windows)]
    pub(crate) fn validate_runtime_environment(
        &self,
        policy: &crate::runtime_environment::WindowsManagedDenoEnvironment,
        selected: &Path,
        cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), ToolError> {
        let journal = self.journal.lock().map_err(|_| ToolError::UnsafePath)?;
        self.marker
            .verify_journal(journal.record().map_err(|_| ToolError::UnsafePath)?)?;
        let anchor = journal.anchors().root;
        let binding = policy.cache_binding().ok_or(ToolError::UnsafePath)?;
        if binding.root_path().map_err(|_| ToolError::UnsafePath)? != self.marker.root_path()
            || binding.root
                != (crate::file_identity::NativeFileIdentity::Windows {
                    volume_serial: anchor.volume_serial,
                    file_id: anchor.file_id,
                })
            || policy.selected_executable() != Some(selected.canonicalize()?.as_path())
        {
            return Err(ToolError::UnsafePath);
        }
        let deadline = std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(10))
            .ok_or(ToolError::TimedOut)?;
        policy.verify_original(cancel, deadline)
    }
    /// Runs one explicitly typed reader through an independent Windows guardian.
    /// This execution-only capability never enters persisted download options.
    #[cfg(windows)]
    pub fn run_tool(
        &self,
        spec: &crate::process::ProcessSpec,
        cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        metadata: bool,
        progress: Option<crate::metrics::ProgressFeed>,
        stage: super::ManagedReaderStage,
    ) -> std::io::Result<crate::process::MetadataResult> {
        let rejected =
            || std::io::Error::other(crate::process::OwnershipCleanupError::TerminalUnconfirmed);
        let mut journal = self.journal.lock().map_err(|_| rejected())?;
        let record = journal.record().map_err(|_| rejected())?;
        self.marker.verify_journal(record).map_err(|_| rejected())?;
        let index = (0..8)
            .find(|index| record.bound_identity(*index).is_none())
            .ok_or_else(rejected)?;
        crate::process::run_managed_tool(
            spec,
            cancel,
            metadata,
            progress,
            &mut journal,
            self.marker.root_path(),
            index,
            stage,
        )
    }
    /// Runs one explicitly typed reader through an independent Windows guardian.
    /// This execution-only capability never enters persisted download options.
    #[cfg(windows)]
    pub fn run_paired_tool(
        &self,
        spec: &crate::process::ProcessSpec,
        cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        metadata: bool,
        progress: Option<crate::metrics::ProgressFeed>,
        stage: super::ManagedReaderStage,
        cookies: &crate::cookies::SessionCookieLease,
    ) -> std::io::Result<crate::process::MetadataResult> {
        let rejected =
            || std::io::Error::other(crate::process::OwnershipCleanupError::TerminalUnconfirmed);
        let mut journal = self.journal.lock().map_err(|_| rejected())?;
        let record = journal.record().map_err(|_| rejected())?;
        self.marker.verify_journal(record).map_err(|_| rejected())?;
        let index = (0..8)
            .find(|index| record.bound_identity(*index).is_none())
            .ok_or_else(rejected)?;
        crate::process::run_paired_tool(
            spec,
            cancel,
            metadata,
            progress,
            &mut journal,
            self.marker.root_path(),
            index,
            stage,
            cookies,
        )
    }
    pub fn finish(self) -> Result<(), ToolError> {
        #[cfg(windows)]
        {
            let journal = self
                .journal
                .into_inner()
                .map_err(|_| ToolError::UnsafePath)?;
            let record = journal.record().map_err(|_| ToolError::UnsafePath)?;
            self.marker.verify_journal(record)?;
            if record.is_pristine() {
                journal.cleanup_pristine()
            } else {
                journal.cleanup_completed()
            }
            .map_err(|_| ToolError::UnsafePath)?;
        }
        self.marker.clear()
    }
}
impl ToolManager {
    pub fn begin_execution(&self) -> Result<ManagedExecution, ToolError> {
        let exclusive = self.acquire_install_lock()?;
        #[cfg(not(windows))]
        let marker = super::execution_marker::ExecutionMarker::create(self)?;
        #[cfg(windows)]
        let mut marker = super::execution_marker::ExecutionMarker::create(self)?;
        #[cfg(windows)]
        let journal = marker.create_journal(exclusive.original_file())?;
        // Journal pins clone the original lock handle. Release the creation-only
        // exclusive lock explicitly before acquiring the independent shared lease.
        #[cfg(windows)]
        fs2::FileExt::unlock(exclusive.original_file())?;
        drop(exclusive);
        let lease = self.acquire_execution_lease()?;
        Ok(ManagedExecution {
            marker,
            _lease: lease,
            #[cfg(windows)]
            journal: std::sync::Mutex::new(journal),
        })
    }
    pub fn acquire_execution_lease(&self) -> Result<ExecutionLease, ToolError> {
        self.verify_root()?;
        let path = self.root.join(".install-transfer-lock");
        safe_components(&path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        if !native_files::same_file(&path, &file) {
            return Err(ToolError::UnsafePath);
        }
        fs2::FileExt::try_lock_shared(&file)?;
        self.verify_root()?;
        if !native_files::same_file(&path, &file) {
            return Err(ToolError::UnsafePath);
        }
        Ok(ExecutionLease { _file: file })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_readers_exclude_mutation_until_last_release() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let other = ToolManager::new(&temp.path().join("managed")).unwrap();
        let first = manager.acquire_execution_lease().unwrap();
        let second = other.acquire_execution_lease().unwrap();
        assert!(other.acquire_install_lock().is_err());
        drop(first);
        assert!(manager.acquire_install_lock().is_err());
        drop(second);
        let exclusive = other.acquire_install_lock().unwrap();
        assert!(manager.acquire_execution_lease().is_err());
        drop(exclusive);
        assert!(manager.acquire_execution_lease().is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn unsafe_lock_namespace_preserves_external_bytes() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let external = temp.path().join("external");
        fs::write(&external, b"original").unwrap();
        symlink(&external, manager.root.join(".install-transfer-lock")).unwrap();
        assert!(matches!(
            manager.acquire_execution_lease(),
            Err(ToolError::UnsafePath)
        ));
        assert_eq!(fs::read(external).unwrap(), b"original");
    }

    #[cfg(unix)]
    #[test]
    fn closing_parent_capability_keeps_duplicate_reader_lock() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let lease = manager.acquire_execution_lease().unwrap();
        let reader = lease._file.try_clone().unwrap();
        drop(lease);
        assert!(manager.acquire_install_lock().is_err());
        drop(reader);
        assert!(manager.acquire_install_lock().is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn inherited_reader_holds_lock_after_parent_capability_closes() {
        use std::os::{fd::AsRawFd, unix::process::CommandExt};
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};
        struct OwnedReader(std::process::Child);
        impl Drop for OwnedReader {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let lease = manager.acquire_execution_lease().unwrap();
        let descriptor = lease._file.as_raw_fd();
        let ready = temp.path().join("reader-ready");
        let mut command = Command::new("/usr/bin/python3");
        command.args(["-c", "import os,pathlib,sys; fd=int(sys.argv[1]); os.fstat(fd); pathlib.Path(sys.argv[2]).write_bytes(b'ready'); sys.stdin.buffer.readline(); os.close(fd)"])
            .arg(descriptor.to_string()).arg(&ready)
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        // SAFETY: only the child-side flag of this live, parent-owned descriptor
        // changes before exec; the hook performs no allocation or filesystem I/O.
        unsafe {
            command.pre_exec(move || {
                if libc::fcntl(descriptor, libc::F_SETFD, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut reader = OwnedReader(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.exists() {
            assert!(reader.0.try_wait().unwrap().is_none());
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        drop(lease);
        assert!(manager.acquire_install_lock().is_err());
        reader
            .0
            .stdin
            .take()
            .unwrap()
            .write_all(b"finish\n")
            .unwrap();
        loop {
            if let Some(status) = reader.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(manager.acquire_install_lock().is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn sealed_native_reader_namespace_is_removed_before_reuse() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let manager = ToolManager::new(&root).unwrap();
        let execution = manager.begin_execution().unwrap();
        let binding: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(".execution-pending")).unwrap()).unwrap();
        assert_eq!(binding["version"], 3);
        assert_eq!(
            binding["managed"]["anchors"]["receipts"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        assert_eq!(
            binding["managed"]["anchors"],
            serde_json::to_value(execution.journal.lock().unwrap().anchors()).unwrap()
        );
        assert!(root.join(".execution-readers.json").is_file());
        for index in 0..8 {
            assert!(
                root.join(format!(".execution-reader-{index}.receipt"))
                    .is_file()
            );
        }
        execution.finish().unwrap();
        assert!(!root.join(".execution-pending").exists());
        assert!(!root.join(".execution-readers.json").exists());
        for index in 0..8 {
            assert!(
                !root
                    .join(format!(".execution-reader-{index}.receipt"))
                    .exists()
            );
        }
        manager.begin_execution().unwrap().finish().unwrap();
    }

    #[test]
    fn durable_scope_finish_and_abandon_preserve_mutation_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let manager = ToolManager::new(&root).unwrap();
        let execution = manager.begin_execution().unwrap();
        assert!(manager.acquire_install_lock().is_err());
        execution.finish().unwrap();
        assert!(manager.acquire_install_lock().is_ok());
        let execution = manager.begin_execution().unwrap();
        drop(execution);
        let reopened = ToolManager::new(&root).unwrap();
        assert!(matches!(
            reopened.acquire_install_lock(),
            Err(ToolError::ExecutionPending)
        ));
        assert!(matches!(
            reopened.begin_execution(),
            Err(ToolError::ExecutionPending)
        ));
        assert!(root.join(".execution-pending").is_file());
    }
}
