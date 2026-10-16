//! Pure exact Deno release output verification. No process or filesystem authority.
use super::*;
use crate::process::{self, ProcessOutcome, ProcessSpec};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};
pub(super) fn verified_deno_version(
    stdout: &[u8],
    expected_version: &str,
    expected_target: &str,
) -> Result<(), ToolError> {
    if stdout.len() > 4096 {
        return Err(ToolError::VersionMismatch);
    }
    let text = std::str::from_utf8(stdout).map_err(|_| ToolError::VersionMismatch)?;
    let lines: Vec<_> = text.lines().collect();
    if lines.len() != 3
        || lines.iter().any(|line| {
            line.is_empty()
                || line.len() > 256
                || !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
        })
    {
        return Err(ToolError::VersionMismatch);
    }
    let first = format!("deno {expected_version} (stable, release, {expected_target})");
    if lines[0] != first {
        return Err(ToolError::VersionMismatch);
    }
    for (line, prefix) in [(lines[1], "v8 "), (lines[2], "typescript ")] {
        let token = line
            .strip_prefix(prefix)
            .ok_or(ToolError::VersionMismatch)?;
        if token.is_empty()
            || token.len() > 128
            || token.bytes().any(|byte| byte.is_ascii_whitespace())
        {
            return Err(ToolError::VersionMismatch);
        }
    }
    Ok(())
}
/// Actual version proof and activation share one root lock and original pins.
impl ToolManager {
    pub fn select_verified_deno(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_deno_operation(package, cancel, true)
    }
    /// Rechecks owned bytes and runtime version without changing any active row.
    pub fn inspect_verified_deno(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledPackage, ToolError> {
        self.verify_deno_operation(package, cancel, false)
    }
    fn verify_deno_operation(
        &mut self,
        package: &ArchivePackage,
        cancel: &Arc<AtomicBool>,
        activate: bool,
    ) -> Result<InstalledPackage, ToolError> {
        package.validate()?;
        if package.tool != "deno" || !compatible_target(&package.target) {
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
        let main_pin = native_files::open_regular(&installed.main.executable)?;
        let result = process::run_metadata(
            &ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: installed.main.executable.clone(),
                args: vec!["--version".into()],
                timeout: direct_download::checkpoint(cancel, deadline)?,
                max_log_bytes: 4096,
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
        if result.execution.exit_code != Some(0) || result.execution.truncated {
            return Err(ToolError::VersionMismatch);
        }
        verified_deno_version(&result.stdout, &package.tool_version, &package.target)?;
        if !native_files::same_directory(&path, &directory_pin)
            || !native_files::same_file(&installed.main.executable, &main_pin)
        {
            return Err(ToolError::UnsafePath);
        }
        let verified = self.verify_package_with_checkpoint(&path, Some(package), &mut || {
            direct_download::checkpoint(cancel, deadline).map(|_| ())
        })?;
        direct_download::checkpoint(cancel, deadline)?;
        self.verify_root()?;
        direct_download::checkpoint(cancel, deadline)?;
        if activate {
            let transaction = self.db.transaction()?;
            transaction.execute("INSERT INTO active(tool,version,target) VALUES(?1,?2,?3) ON CONFLICT(tool) DO UPDATE SET version=excluded.version,target=excluded.target",params![package.tool,package.tool_version,package.target])?;
            direct_download::checkpoint(cancel, deadline)?;
            transaction.commit()?;
        }
        Ok(verified)
    }
}
fn compatible_target(target: &str) -> bool {
    (cfg!(all(target_os = "linux", target_arch = "x86_64")) && target == "x86_64-unknown-linux-gnu")
        || (cfg!(all(target_os = "windows", target_arch = "x86_64"))
            && target == "x86_64-pc-windows-msvc")
        || (cfg!(all(target_os = "macos", target_arch = "x86_64"))
            && target == "x86_64-apple-darwin")
        || (cfg!(all(target_os = "macos", target_arch = "aarch64"))
            && target == "aarch64-apple-darwin")
}
#[cfg(test)]
mod tests {
    use super::*;
    const OUTPUT: &[u8] = b"deno 2.9.7 (stable, release, x86_64-unknown-linux-gnu)\nv8 15.0.245.2-rusty\ntypescript 6.0.3\n";
    #[test]
    fn exact_release_shape_and_crlf_are_accepted() {
        assert!(verified_deno_version(OUTPUT, "2.9.7", "x86_64-unknown-linux-gnu").is_ok());
        let windows = String::from_utf8(OUTPUT.to_vec())
            .unwrap()
            .replace("x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc")
            .replace('\n', "\r\n");
        assert!(
            verified_deno_version(windows.as_bytes(), "2.9.7", "x86_64-pc-windows-msvc").is_ok()
        );
    }
    #[test]
    fn wrong_version_target_prefix_and_extra_lines_are_rejected() {
        assert!(verified_deno_version(OUTPUT, "2.9.8", "x86_64-unknown-linux-gnu").is_err());
        assert!(verified_deno_version(OUTPUT, "2.9.7", "aarch64-apple-darwin").is_err());
        for text in [
            String::from_utf8(OUTPUT.to_vec())
                .unwrap()
                .replace("v8 ", "engine "),
            String::from_utf8(OUTPUT.to_vec()).unwrap() + "banner\n",
            String::from_utf8(OUTPUT.to_vec())
                .unwrap()
                .replace("typescript 6.0.3", "typescript "),
            String::from_utf8(OUTPUT.to_vec())
                .unwrap()
                .replace("v8 15.0.245.2-rusty", "v8 two tokens"),
        ] {
            assert!(
                verified_deno_version(text.as_bytes(), "2.9.7", "x86_64-unknown-linux-gnu")
                    .is_err()
            );
        }
    }
    #[test]
    fn invalid_bytes_bounds_empty_and_truncation_are_rejected() {
        for bytes in [
            vec![0xff],
            vec![b'x'; 4097],
            Vec::new(),
            OUTPUT[..OUTPUT.len() - 10].to_vec(),
            b"\nden o\n\n".to_vec(),
        ] {
            assert!(verified_deno_version(&bytes, "2.9.7", "x86_64-unknown-linux-gnu").is_err());
        }
        let long = format!(
            "deno 2.9.7 (stable, release, x86_64-unknown-linux-gnu)\nv8 {}\ntypescript 6.0.3\n",
            "x".repeat(257)
        );
        assert!(
            verified_deno_version(long.as_bytes(), "2.9.7", "x86_64-unknown-linux-gnu").is_err()
        );
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn readonly_deno_inspection_never_activates_absent_valid_or_invalid_rows() {
        for active in [None, Some("2.9.6"), Some("../invalid")] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = process_fixture(&manager, "2.9.6", "good");
            if let Some(version) = active {
                manager
                    .db
                    .execute(
                        "INSERT INTO active(tool,version,target)VALUES('deno',?1,?2)",
                        params![version, prior.target],
                    )
                    .unwrap();
            }
            let before = manager.active("deno").unwrap();
            let next = process_fixture(&manager, "2.9.7", "good");
            manager.db.execute_batch("CREATE TRIGGER deny_inspection_insert BEFORE INSERT ON active BEGIN SELECT RAISE(ABORT,'no inspection writes');END;CREATE TRIGGER deny_inspection_update BEFORE UPDATE ON active BEGIN SELECT RAISE(ABORT,'no inspection writes');END;").unwrap();
            let verified = manager
                .inspect_verified_deno(&next, &Arc::new(AtomicBool::new(false)))
                .unwrap();
            assert_eq!(verified.package.tool_version, "2.9.7");
            assert_eq!(manager.active("deno").unwrap(), before);
        }
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn readonly_deno_failure_and_cancel_preserve_active_rows() {
        for mode in ["wrong", "nonzero", "mutate", "cancel", "foreign"] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = process_fixture(&manager, "2.9.6", "good");
            manager.select_package(&prior).unwrap();
            let before = manager.active("deno").unwrap();
            let next = process_fixture(&manager, "2.9.7", mode);
            let foreign = manager
                .root
                .join("deno/2.9.7/x86_64-unknown-linux-gnu/foreign");
            if mode == "foreign" {
                fs::write(&foreign, b"preserve").unwrap();
            }
            let result =
                manager.inspect_verified_deno(&next, &Arc::new(AtomicBool::new(mode == "cancel")));
            match mode {
                "cancel" => assert!(matches!(result, Err(ToolError::Cancelled))),
                "mutate" => assert!(matches!(
                    result,
                    Err(ToolError::HashMismatch) | Err(ToolError::UnexpectedSize)
                )),
                "foreign" => assert!(matches!(result, Err(ToolError::UnexpectedFiles))),
                _ => assert!(matches!(result, Err(ToolError::VersionMismatch))),
            }
            assert_eq!(manager.active("deno").unwrap(), before);
            if mode == "foreign" {
                assert_eq!(fs::read(foreign).unwrap(), b"preserve");
            }
        }
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    fn process_fixture(manager: &ToolManager, version: &str, mode: &str) -> ArchivePackage {
        use std::os::unix::fs::PermissionsExt;
        let path = manager
            .root
            .join("deno")
            .join(version)
            .join("x86_64-unknown-linux-gnu");
        fs::create_dir_all(&path).unwrap();
        let reported = if mode == "wrong" { "0.0.0" } else { version };
        let script = format!(
            "#!/usr/bin/python3\nimport pathlib,sys\nassert sys.argv[1:]==['--version']\np=pathlib.Path(__file__)\n{}\nprint('deno {} (stable, release, x86_64-unknown-linux-gnu)')\nprint('v8 15.0.245.2-rusty')\nprint('typescript 6.0.3')\nsys.exit({})\n",
            if mode == "mutate" {
                "p.write_bytes(p.read_bytes()+b'\\n#changed')"
            } else {
                "pass"
            },
            reported,
            if mode == "nonzero" { 7 } else { 0 }
        );
        fs::write(path.join("deno"), script.as_bytes()).unwrap();
        fs::set_permissions(path.join("deno"), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(path.join("LICENSE"), b"notice").unwrap();
        let package = ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "deno".into(),
            tool_version: version.into(),
            target: "x86_64-unknown-linux-gnu".into(),
            source_url: "https://example.invalid/deno.zip".into(),
            license: "synthetic-MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: 123,
            archive_sha256: "a".repeat(64),
            components: vec![
                ArchiveComponent {
                    name: "deno".into(),
                    bytes: script.len() as u64,
                    sha256: format!("{:x}", Sha256::digest(script.as_bytes())),
                    role: ArchiveComponentRole::Executable,
                },
                ArchiveComponent {
                    name: "LICENSE".into(),
                    bytes: 6,
                    sha256: format!("{:x}", Sha256::digest(b"notice")),
                    role: ArchiveComponentRole::Notice,
                },
            ],
        };
        let manifest = InstallManifest {
            tool: package.tool.clone(),
            version: version.into(),
            target: package.target.clone(),
            source_url: package.source_url.clone(),
            sha256: package.components[0].sha256.clone(),
            license: package.license.clone(),
            ownership: Ownership::Managed,
            executable: "deno".into(),
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
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn actual_synthetic_deno_version_activates_exact_release() {
        let temp = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let package = process_fixture(&manager, "2.9.7", "good");
        let selected = manager
            .select_verified_deno(&package, &Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert_eq!(selected.package.tool_version, "2.9.7");
        assert_eq!(
            manager.active("deno").unwrap(),
            Some(("2.9.7".into(), package.target.clone()))
        );
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn actual_wrong_nonzero_and_mutating_deno_preserve_prior_active() {
        for mode in ["wrong", "nonzero", "mutate"] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = process_fixture(&manager, "2.9.6", "good");
            manager.select_package(&prior).unwrap();
            let active = manager.active("deno").unwrap();
            let package = process_fixture(&manager, "2.9.7", mode);
            let result = manager.select_verified_deno(&package, &Arc::new(AtomicBool::new(false)));
            if mode == "mutate" {
                assert!(matches!(
                    result,
                    Err(ToolError::UnexpectedSize) | Err(ToolError::HashMismatch)
                ));
            } else {
                assert!(matches!(result, Err(ToolError::VersionMismatch)));
            }
            assert_eq!(manager.active("deno").unwrap(), active);
            assert!(
                manager
                    .root
                    .join("deno/2.9.6/x86_64-unknown-linux-gnu/deno")
                    .exists()
            );
        }
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn foreign_cancel_and_unsupported_target_preserve_prior_active() {
        for mode in ["foreign", "cancel", "target"] {
            let temp = tempfile::tempdir().unwrap();
            let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let prior = process_fixture(&manager, "2.9.6", "good");
            manager.select_package(&prior).unwrap();
            let active = manager.active("deno").unwrap();
            let mut package = process_fixture(&manager, "2.9.7", "good");
            let foreign = manager
                .root
                .join("deno/2.9.7/x86_64-unknown-linux-gnu/foreign");
            if mode == "foreign" {
                fs::write(&foreign, b"preserve").unwrap();
            }
            if mode == "target" {
                package.target = "unsupported-target".into();
            }
            let result = manager
                .select_verified_deno(&package, &Arc::new(AtomicBool::new(mode == "cancel")));
            assert!(matches!(
                (mode, result),
                ("foreign", Err(ToolError::UnexpectedFiles))
                    | ("cancel", Err(ToolError::Cancelled))
                    | ("target", Err(ToolError::InvalidManifest))
            ));
            assert_eq!(manager.active("deno").unwrap(), active);
            if mode == "foreign" {
                assert_eq!(fs::read(foreign).unwrap(), b"preserve");
            }
        }
    }
}
