//! Windows-only developer experiment. Exact pinned HTTPS installation, never execution/adoption.
#[cfg(windows)]
mod experiment {
    use dlpad_engine::tools::{
        ArchiveComponent, ArchiveComponentRole, ArchivePackage, Ownership, PinnedArchiveNotice,
        ToolError, ToolManager,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{self, File},
        io::Read,
        path::Path,
        sync::{Arc, atomic::AtomicBool},
        time::Instant,
    };
    const ARCHIVE_SHA: &str = "a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238";
    const EXE_SHA: &str = "e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33";
    const NOTICE_SHA: &str = "f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588";
    const REVISION: &str = "0c071246a412575e07423263404a5d13e7ed6aa2";
    fn package() -> ArchivePackage {
        ArchivePackage { version: 3, tool: "deno".into(), tool_version: "2.9.7".into(),
            target: "x86_64-pc-windows-msvc".into(),
            source_url: "https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-pc-windows-msvc.zip".into(),
            license: "MIT; exact upstream LICENSE.md only; transitive notice/trust adoption unverified".into(),
            ownership: Ownership::Managed, archive_bytes: 42_630_221, archive_sha256: ARCHIVE_SHA.into(),
            layout: None, components: vec![
                ArchiveComponent { name: "deno.exe".into(), bytes: 97_462_048, sha256: EXE_SHA.into(), role: ArchiveComponentRole::Executable },
                ArchiveComponent { name: "LICENSE.md".into(), bytes: 1074, sha256: NOTICE_SHA.into(), role: ArchiveComponentRole::Notice },
            ], notices: vec![PinnedArchiveNotice { component: "LICENSE.md".into(), revision: REVISION.into(),
                source_url: "https://raw.githubusercontent.com/denoland/deno/0c071246a412575e07423263404a5d13e7ed6aa2/LICENSE.md".into() }],
        }
    }
    fn hash(path: &Path, bound: u64) -> Result<(u64, String), Box<dyn std::error::Error>> {
        let mut source = File::open(path)?;
        let mut total = 0u64;
        let mut h = Sha256::new();
        let mut bytes = [0u8; 65536];
        loop {
            let n = source.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            total = total.checked_add(n as u64).ok_or("size overflow")?;
            if total > bound {
                return Err("file bound exceeded".into());
            }
            h.update(&bytes[..n]);
        }
        Ok((total, format!("{:x}", h.finalize())))
    }
    pub(super) fn run() -> i32 {
        if std::env::args_os().skip(1).collect::<Vec<_>>()
            != [std::ffi::OsString::from("--fixed-notice-experiment")]
        {
            eprintln!("fixed experiment requires --fixed-notice-experiment");
            return 2;
        }
        let start = Instant::now();
        let mut phase = "create-owned-fixture";
        let mut report = serde_json::json!({"schema":1,"status":"FAIL","scope":"Windows official v3 ZIP+exact upstream notice; no execution/activation/supply adoption",
            "native_tool_executed":false,"activated":false,"supply_adopted":false,"archive_bytes":42630221,"archive_sha256":ARCHIVE_SHA,"revision":REVISION});
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let fixture = tempfile::Builder::new()
                .prefix("dlpad-deno-v3-notice-owned-")
                .tempdir()?;
            let root = fixture.keep();
            report["owned_root"] = serde_json::json!(root);
            let sentinel = root.join("foreign-sentinel");
            fs::write(&sentinel, b"owned synthetic foreign sentinel")?;
            let sentinel_before = hash(&sentinel, 1024)?;
            let managed = root.join("managed");
            let package = package();
            package.validate()?;
            phase = "fixed-https-download-install";
            let mut manager = ToolManager::new(&managed)?;
            let installed = manager
                .download_archive_and_install(package.clone(), &Arc::new(AtomicBool::new(false)))?;
            if manager.active("deno")?.is_some() {
                return Err("inactive install invariant".into());
            }
            let manifest = installed.main.manifest.clone();
            let directory = installed.directory.clone();
            let exe = hash(&installed.main.executable, 97_462_048)?;
            if exe != (97_462_048, EXE_SHA.into()) {
                return Err("executable proof mismatch".into());
            }
            let notice = directory.join("LICENSE.md");
            let mut original = Vec::new();
            File::open(&notice)?
                .take(1024 * 1024 + 1)
                .read_to_end(&mut original)?;
            if original.len() > 1024 * 1024
                || hash(&notice, 1024 * 1024)? != (1074, NOTICE_SHA.into())
            {
                return Err("notice proof mismatch".into());
            }
            report["installed_executable"] = serde_json::json!({"bytes":exe.0,"sha256":exe.1});
            report["installed_notice"] = serde_json::json!({"bytes":1074,"sha256":NOTICE_SHA});
            drop(installed);
            drop(manager);
            phase = "fresh-manager-whole-set";
            let mut manager = ToolManager::new(&managed)?;
            let verified = manager
                .package_installed(&manifest)?
                .ok_or("package missing on reopen")?;
            if verified.package.notices != package.notices || manager.active("deno")?.is_some() {
                return Err("reopen proof mismatch".into());
            }
            drop(verified);
            report["fresh_fullset_verified"] = serde_json::json!(true);
            phase = "owned-notice-tamper-rejection";
            let mut damaged = original.clone();
            damaged[0] ^= 1;
            fs::write(&notice, &damaged)?;
            let rejected = matches!(
                manager.package_installed(&manifest),
                Err(ToolError::HashMismatch)
            );
            // Restore only this owned temporary notice, even if the expected rejection failed.
            fs::write(&notice, &original)?;
            fs::OpenOptions::new()
                .write(true)
                .open(&notice)?
                .sync_all()?;
            if !rejected {
                return Err("tampered notice was not rejected".into());
            }
            if manager.active("deno")?.is_some() {
                return Err("tamper changed active".into());
            }
            manager
                .package_installed(&manifest)?
                .ok_or("restored package missing")?;
            report["tampered_notice_rejected"] = serde_json::json!(true);
            phase = "exact-package-remove";
            manager.remove_package(&package)?;
            if directory.try_exists()?
                || manager.package_installed(&manifest)?.is_some()
                || manager.active("deno")?.is_some()
            {
                return Err("remove invariant".into());
            }
            if hash(&sentinel, 1024)? != sentinel_before {
                return Err("sentinel changed".into());
            }
            report["package_removed"] = serde_json::json!(true);
            report["sentinel_preserved"] = serde_json::json!(true);
            report["status"] = serde_json::json!("PASS");
            Ok(())
        })();
        report["phase"] = serde_json::json!(phase);
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        // Fixed redacted failure: no raw transport, source path, or credentials from errors.
        if result.is_err() {
            report["error"] = serde_json::json!(
                "fixed experiment failure; inspect phase and retained owned fixture"
            );
        }
        let json = serde_json::to_string(&report).expect("bounded fixed report");
        if json.len() > 16384 {
            eprintln!("experiment report bound exceeded");
            return 2;
        }
        println!("{json}");
        if result.is_ok() { 0 } else { 2 }
    }
}
#[cfg(windows)]
fn main() {
    std::process::exit(experiment::run());
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows-only developer experiment; not executed");
    std::process::exit(2);
}
