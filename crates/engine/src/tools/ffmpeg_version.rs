//! Pure bounded version shape verification; license/process/hash proof is caller-owned.
pub(super) fn verified_ffmpeg_version(
    stdout: &[u8],
    expected_tool: &str,
    expected_version: &str,
) -> bool {
    if !matches!(expected_tool, "ffmpeg" | "ffprobe")
        || stdout.len() > 64 * 1024
        || expected_version.is_empty()
        || expected_version.len() > 128
        || !expected_version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_+~".contains(&b))
    {
        return false;
    }
    let Ok(text) = std::str::from_utf8(stdout) else {
        return false;
    };
    let lines: Vec<_> = text.lines().collect();
    if lines.is_empty()
        || lines.len() > 64
        || lines
            .iter()
            .any(|line| line.len() > 4096 || !line.bytes().all(|b| (0x20..=0x7e).contains(&b)))
    {
        return false;
    }
    let prefix = format!("{expected_tool} version {expected_version} Copyright (c) ");
    let Some(copyright) = lines[0].strip_prefix(&prefix) else {
        return false;
    };
    let Some(years) = copyright.strip_suffix(" the FFmpeg developers") else {
        return false;
    };
    let Some((first, last)) = years.split_once('-') else {
        return false;
    };
    first.len() == 4
        && last.len() == 4
        && first
            .bytes()
            .chain(last.bytes())
            .all(|b| b.is_ascii_digit())
}

use super::*;
use crate::process::{self, ProcessOutcome, ProcessSpec};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};
impl ToolManager {
    /// Verify both original executables before changing the active package.
    pub fn select_verified_ffmpeg(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_ffmpeg_operation(package, cancel, true)
    }
    /// Rechecks owned bytes and runtime version without changing any active row.
    pub fn inspect_verified_ffmpeg(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_ffmpeg_operation(package, cancel, false)
    }
    fn verify_ffmpeg_operation(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
        activate: bool,
    ) -> Result<InstalledPackage, ToolError> {
        package.validate()?;
        let (main_name, probe_name) = if cfg!(target_os = "windows") {
            ("ffmpeg.exe", "ffprobe.exe")
        } else {
            ("ffmpeg", "ffprobe")
        };
        if package.tool != "ffmpeg"
            || !compatible_target(&package.target)
            || !package
                .components
                .iter()
                .any(|c| c.name == main_name && c.role == ArchiveComponentRole::Executable)
            || !package
                .components
                .iter()
                .any(|c| c.name == probe_name && c.role == ArchiveComponentRole::Auxiliary)
        {
            return Err(ToolError::InvalidManifest);
        }
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .ok_or(ToolError::TimedOut)?;
        direct_download::checkpoint(cancel, deadline)?;
        let _lock = self.acquire_install_lock()?;
        let path = self
            .root
            .join(&package.tool)
            .join(&package.tool_version)
            .join(&package.target);
        let installed = self.verify_package_with_checkpoint(&path, Some(package), &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        let directory_pin = native_files::open_directory(&path)?;
        let main_path = installed.main.executable.clone();
        let probe_path = path.join(probe_name);
        let main_pin = native_files::open_regular(&main_path)?;
        let probe_pin = native_files::open_regular(&probe_path)?;
        for (executable, tool) in [(&main_path, "ffmpeg"), (&probe_path, "ffprobe")] {
            let result = process::run_metadata(
                &ProcessSpec {
                    #[cfg(windows)]
                    runtime_environment: None,
                    executable: executable.clone(),
                    args: vec!["-version".into()],
                    timeout: direct_download::checkpoint(cancel, deadline)?,
                    max_log_bytes: 65536,
                },
                cancel,
            )
            .map_err(|error| {
                if error.kind() == io::ErrorKind::TimedOut {
                    ToolError::TimedOut
                } else {
                    ToolError::Io(error)
                }
            })?;
            direct_download::checkpoint(cancel, deadline)?;
            match result.execution.outcome {
                ProcessOutcome::Cancelled => return Err(ToolError::Cancelled),
                ProcessOutcome::TimedOut => return Err(ToolError::TimedOut),
                ProcessOutcome::OutputLimit => return Err(ToolError::VersionMismatch),
                ProcessOutcome::Exited => (),
            }
            if result.execution.exit_code != Some(0)
                || result.execution.truncated
                || !verified_ffmpeg_version(&result.stdout, tool, &package.tool_version)
            {
                return Err(ToolError::VersionMismatch);
            }
            if !native_files::same_directory(&path, &directory_pin)
                || !native_files::same_file(&main_path, &main_pin)
                || !native_files::same_file(&probe_path, &probe_pin)
            {
                return Err(ToolError::UnsafePath);
            }
        }
        let verified = self.verify_package_with_checkpoint(&path, Some(package), &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        direct_download::checkpoint(cancel, deadline)?;
        self.verify_root()?;
        direct_download::checkpoint(cancel, deadline)?;
        if activate {
            let transaction = self.db.transaction()?;
            transaction.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target", params![package.tool,package.tool_version,package.target])?;
            direct_download::checkpoint(cancel, deadline)?;
            transaction.commit()?;
        }
        Ok(verified)
    }
}
fn compatible_target(target: &str) -> bool {
    (cfg!(all(target_os = "linux", target_arch = "x86_64")) && target == "x86_64-unknown-linux-gnu")
        || (cfg!(all(target_os = "windows", target_arch = "x86_64"))
            && matches!(target, "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu"))
        || (cfg!(all(target_os = "macos", target_arch = "x86_64"))
            && target == "x86_64-apple-darwin")
        || (cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && target == "aarch64-apple-darwin")
}

#[cfg(test)]
mod tests {
    use super::*;
    const VERSION: &str = "n9.0.2-17-g2a571b6068-20260930";
    fn output(tool: &str) -> String {
        format!(
            "{tool} version {VERSION} Copyright (c) 2000-2026 the FFmpeg developers\nbuilt with gcc 16.2.0\nconfiguration: --enable-version3 --disable-libx264\nlibavutil      61.  1.102 / 61.  1.102\n"
        )
    }
    #[test]
    fn exact_tool_token_and_crlf_are_accepted() {
        for tool in ["ffmpeg", "ffprobe"] {
            let text = output(tool);
            assert!(verified_ffmpeg_version(text.as_bytes(), tool, VERSION));
            assert!(verified_ffmpeg_version(
                text.replace('\n', "\r\n").as_bytes(),
                tool,
                VERSION
            ));
        }
    }
    #[test]
    fn wrong_names_version_substring_and_banner_are_rejected() {
        let text = output("ffmpeg");
        assert!(!verified_ffmpeg_version(
            text.as_bytes(),
            "ffprobe",
            VERSION
        ));
        assert!(!verified_ffmpeg_version(
            text.as_bytes(),
            "ffmpeg",
            "n9.0.2"
        ));
        assert!(!verified_ffmpeg_version(
            text.as_bytes(),
            "unknown",
            VERSION
        ));
        assert!(!verified_ffmpeg_version(
            format!("banner\n{text}").as_bytes(),
            "ffmpeg",
            VERSION
        ));
        assert!(!verified_ffmpeg_version(
            text.replace(VERSION, &format!("{VERSION}extra")).as_bytes(),
            "ffmpeg",
            VERSION
        ));
    }
    #[test]
    fn bounds_invalid_bytes_and_controls_are_rejected() {
        for bytes in [
            vec![0xff],
            vec![b'x'; 65537],
            Vec::new(),
            output("ffmpeg").replace("gcc", "g\tc").into_bytes(),
            format!("{}{}", output("ffmpeg"), "line\n".repeat(65)).into_bytes(),
            format!("{}{}", output("ffmpeg"), "x".repeat(4097)).into_bytes(),
        ] {
            assert!(!verified_ffmpeg_version(&bytes, "ffmpeg", VERSION));
        }
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    mod db_cancel {
        // Draft for ffmpeg_version::tests (private ToolManager.db access required).
        // Reuse seed below; marker is manager-root sibling, never package foreign entry.
        use super::*;
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
            path::Path,
            sync::{
                Mutex,
                atomic::{AtomicUsize, Ordering},
            },
            thread,
        };
        static DB_TEST_SERIAL: Mutex<()> = Mutex::new(());
        static SQLITE_BUSY_EVENTS: AtomicUsize = AtomicUsize::new(0);
        fn selection_busy_handler(_: i32) -> bool {
            SQLITE_BUSY_EVENTS.fetch_add(1, Ordering::Release);
            thread::yield_now();
            true
        }

        #[test]
        fn readonly_ffmpeg_inspection_never_activates_absent_valid_or_invalid_rows() {
            for active in [None, Some("1.0.0"), Some("../invalid")] {
                let temp = tempfile::tempdir().unwrap();
                let root = temp.path().join("managed");
                let mut manager = ToolManager::new(&root).unwrap();
                let prior = seed(&root, "1.0.0", "success");
                if let Some(version) = active {
                    manager
                        .db
                        .execute(
                            "INSERT INTO active(tool,version,target)VALUES('ffmpeg',?1,?2)",
                            params![version, prior.target],
                        )
                        .unwrap();
                }
                let before = manager.active("ffmpeg").unwrap();
                let next = seed(&root, "2.0.0", "success");
                manager.db.execute_batch("CREATE TRIGGER deny_inspection_insert BEFORE INSERT ON active BEGIN SELECT RAISE(ABORT,'no inspection writes');END;CREATE TRIGGER deny_inspection_update BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT,'no inspection writes');END;").unwrap();
                let verified = manager
                    .inspect_verified_ffmpeg(&next, &Arc::new(AtomicBool::new(false)))
                    .unwrap();
                assert_eq!(verified.package.tool_version, "2.0.0");
                assert_eq!(manager.active("ffmpeg").unwrap(), before);
            }
        }
        #[test]
        fn readonly_ffmpeg_version_failure_cancel_and_mutation_preserve_rows() {
            for mode in [
                "probe-mismatch",
                "probe-nonzero",
                "mutation",
                "cancel",
                "foreign",
            ] {
                let temp = tempfile::tempdir().unwrap();
                let root = temp.path().join("managed");
                let mut manager = ToolManager::new(&root).unwrap();
                let prior = seed(&root, "1.0.0", "success");
                manager.select_package(&prior).unwrap();
                let before = manager.active("ffmpeg").unwrap();
                let next = seed(&root, "2.0.0", mode);
                let foreign = root.join("ffmpeg/2.0.0/x86_64-unknown-linux-gnu/foreign");
                if mode == "foreign" {
                    fs::write(&foreign, b"preserve").unwrap();
                }
                let result = manager
                    .inspect_verified_ffmpeg(&next, &Arc::new(AtomicBool::new(mode == "cancel")));
                match mode {
                    "cancel" => assert!(matches!(result, Err(ToolError::Cancelled))),
                    "mutation" => assert!(matches!(
                        result,
                        Err(ToolError::HashMismatch) | Err(ToolError::UnexpectedSize)
                    )),
                    "foreign" => assert!(matches!(result, Err(ToolError::UnexpectedFiles))),
                    _ => assert!(matches!(result, Err(ToolError::VersionMismatch))),
                }
                assert_eq!(manager.active("ffmpeg").unwrap(), before);
                if mode == "foreign" {
                    assert_eq!(fs::read(foreign).unwrap(), b"preserve");
                }
            }
        }

        fn seed(root: &Path, version: &str, mode: &str) -> ArchivePackage {
            let path = root
                .join("ffmpeg")
                .join(version)
                .join("x86_64-unknown-linux-gnu");
            fs::create_dir_all(&path).unwrap();
            let mut components = Vec::new();
            for (name, role) in [
                ("ffmpeg", ArchiveComponentRole::Executable),
                ("ffprobe", ArchiveComponentRole::Auxiliary),
            ] {
                let reported = if mode == "probe-mismatch" && name == "ffprobe" {
                    "0.0.0"
                } else {
                    version
                };
                let mutation = if mode == "db-sync" && name == "ffprobe" {
                    "p.parents[4].joinpath('probe-finished').write_bytes(b'owned marker')"
                } else if mode == "mutation" && name == "ffprobe" {
                    "p.with_name('ffmpeg').write_bytes(p.with_name('ffmpeg').read_bytes()+b'\\n#mutated')"
                } else {
                    "pass"
                };
                let code = if mode == "probe-nonzero" && name == "ffprobe" {
                    7
                } else {
                    0
                };
                let script = format!(
                    "#!/usr/bin/python3\nimport pathlib,sys\nassert sys.argv[1:]==['-version']\np=pathlib.Path(__file__)\n{mutation}\nprint('{name} version {reported} Copyright (c) 2000-2026 the FFmpeg developers')\nprint('synthetic configuration')\nsys.exit({code})\n"
                );
                fs::write(path.join(name), script.as_bytes()).unwrap();
                fs::set_permissions(path.join(name), fs::Permissions::from_mode(0o700)).unwrap();
                components.push(ArchiveComponent {
                    name: name.into(),
                    bytes: script.len() as u64,
                    sha256: format!("{:x}", Sha256::digest(script.as_bytes())),
                    role,
                });
            }
            fs::write(path.join("LICENSE"), b"synthetic notice").unwrap();
            components.push(ArchiveComponent {
                name: "LICENSE".into(),
                bytes: 16,
                sha256: format!("{:x}", Sha256::digest(b"synthetic notice")),
                role: ArchiveComponentRole::Notice,
            });
            let package = ArchivePackage {
                notices: vec![],
                version: 1,
                layout: None,
                tool: "ffmpeg".into(),
                tool_version: version.into(),
                target: "x86_64-unknown-linux-gnu".into(),
                source_url: "https://example.invalid/synthetic.zip".into(),
                license: "synthetic only".into(),
                ownership: Ownership::Managed,
                archive_bytes: 123,
                archive_sha256: "a".repeat(64),
                components,
            };
            let manifest = InstallManifest {
                tool: package.tool.clone(),
                version: version.into(),
                target: package.target.clone(),
                source_url: package.source_url.clone(),
                sha256: package.components[0].sha256.clone(),
                license: package.license.clone(),
                ownership: Ownership::Managed,
                executable: "ffmpeg".into(),
            };
            fs::write(
                path.join("manifest.json"),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            fs::write(
                path.join("archive-package.json"),
                serde_json::to_vec(&package).unwrap(),
            )
            .unwrap();
            package
        }

        #[test]
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        fn sqlite_wait_cancel_rolls_back_before_commit() {
            let _serial = DB_TEST_SERIAL.lock().unwrap();
            SQLITE_BUSY_EVENTS.store(0, Ordering::Release);
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("managed");
            let mut manager = ToolManager::new(&root).unwrap();
            let prior = seed(&root, "1.0.0", "success");
            let installed = manager
                .select_verified_ffmpeg(&prior, &Arc::new(AtomicBool::new(false)))
                .unwrap();
            let prior_bytes = fs::read(installed.main.executable.clone()).unwrap();
            let next = seed(&root, "2.0.0", "db-sync");
            // Private test-only hook: production never exposes the activation connection.
            manager
                .db
                .busy_handler(Some(selection_busy_handler))
                .unwrap();
            let blocker = rusqlite::Connection::open(root.join("activation.sqlite")).unwrap();
            blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let worker_cancel = cancel.clone();
            let worker = thread::spawn(move || {
                let result = manager.select_verified_ffmpeg(&next, &worker_cancel);
                (manager, result)
            });
            let observation_deadline = Instant::now() + Duration::from_secs(5);
            while SQLITE_BUSY_EVENTS.load(Ordering::Acquire) == 0 {
                if Instant::now() >= observation_deadline {
                    cancel.store(true, Ordering::Release);
                    blocker.execute_batch("ROLLBACK").unwrap();
                    let _ = worker.join();
                    panic!("selector never entered SQLite busy handler");
                }
                thread::yield_now();
            }
            // Busy callback is actual SQLite lock observation, not a guessed sleep.
            let marker = temp.path().join("probe-finished");
            let marker_bytes = fs::read(marker);
            cancel.store(true, Ordering::Release);
            blocker.execute_batch("ROLLBACK").unwrap();
            let (manager, result) = worker.join().unwrap();
            assert_eq!(marker_bytes.unwrap(), b"owned marker");
            assert!(matches!(result, Err(ToolError::Cancelled)));
            assert_eq!(
                manager.active("ffmpeg").unwrap(),
                Some(("1.0.0".into(), prior.target))
            );
            assert_eq!(fs::read(installed.main.executable).unwrap(), prior_bytes);
        }
    }
}
