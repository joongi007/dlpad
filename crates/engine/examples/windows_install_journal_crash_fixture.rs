//! Fixture-only explicit process exit; readonly evidence, no recovery authority.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            FreshInstallJournal, InstallJournalPhase, inspect_install_journal,
            manifest::{BundleEntry, BundleManifest},
        },
        file_identity,
        process::{self, ProcessOutcome, ProcessSpec},
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::OpenOptions,
        io::{self, Read, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Path, PathBuf},
        sync::{Arc, atomic::AtomicBool},
        time::{Duration, Instant},
    };
    fn bad() -> io::Error {
        io::ErrorKind::InvalidData.into()
    }
    fn checkpoint(c: &AtomicBool, d: Instant) -> io::Result<()> {
        use std::sync::atomic::Ordering;
        if c.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= d {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    fn snapshot(p: &Path, c: &AtomicBool, d: Instant) -> io::Result<serde_json::Value> {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
            FILE_SHARE_READ, FILE_SHARE_WRITE,
        };
        checkpoint(c, d)?;
        let open = |p: &Path| {
            OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
                .open(p)
        };
        let root = open(p)?;
        let rootmeta = root.metadata()?;
        if !rootmeta.is_dir() {
            return Err(bad());
        }
        let id = file_identity::from_file(&root)?;
        let mut rows = Vec::with_capacity(2);
        for e in std::fs::read_dir(p)? {
            checkpoint(c, d)?;
            let e = e?;
            let name = e.file_name().into_string().map_err(|_| bad())?;
            if rows.len() >= 2
                || !matches!(name.as_str(), "intent.json" | "prepared.json")
                || !e.file_type()?.is_file()
                || e.file_type()?.is_symlink()
            {
                return Err(bad());
            }
            let mut f = open(&e.path())?;
            let identity = file_identity::from_file(&f)?;
            let before = f.metadata()?;
            let size = before.len();
            if !before.is_file() || size > 65536 {
                return Err(bad());
            }
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                checkpoint(c, d)?;
                let limit = (65537usize - bytes.len()).min(buffer.len());
                let n = f.read(&mut buffer[..limit])?;
                checkpoint(c, d)?;
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.len() > 65536 {
                    return Err(bad());
                }
            }
            let after = f.metadata()?;
            if bytes.len() as u64 != size
                || after.len() != size
                || before.modified()? != after.modified()?
                || file_identity::from_file(&f)? != identity
                || file_identity::from_file(&open(&e.path())?)? != identity
            {
                return Err(bad());
            }
            rows.push(serde_json::json!({"name":name,"identity":identity,"bytes":size,"sha256":format!("{:x}",Sha256::digest(&bytes))}));
        }
        rows.sort_by_key(|r| r["name"].as_str().unwrap_or("").to_owned());
        checkpoint(c, d)?;
        if rows.len() != 2
            || rows[0]["name"] != "intent.json"
            || rows[1]["name"] != "prepared.json"
            || file_identity::from_file(&root)? != id
            || file_identity::from_file(&open(p)?)? != id
            || root.metadata()?.modified()? != rootmeta.modified()?
        {
            return Err(bad());
        }
        Ok(serde_json::json!({"root_identity":id,"files":rows}))
    }
    fn owned_root(p: &Path) -> io::Result<()> {
        use std::path::{Component, Prefix};
        if !p.is_absolute()
            || !matches!(p.components().next(),Some(Component::Prefix(v)) if matches!(v.kind(),Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
            || p.components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || !p
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("dlpad-journal-explicit-exit-owned-"))
        {
            return Err(bad());
        }
        for ancestor in p.ancestors() {
            let m = std::fs::symlink_metadata(ancestor)?;
            use std::os::windows::fs::MetadataExt;
            if !m.is_dir()
                || m.file_attributes()
                    & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                    != 0
            {
                return Err(bad());
            }
        }
        Ok(())
    }
    let a: Vec<_> = std::env::args_os().collect();
    if a.len() == 3 && a[1] == "--journal-child-after-prepared" {
        let root = PathBuf::from(&a[2]);
        owned_root(&root)?;
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: "a".repeat(64),
            }],
        };
        let c = AtomicBool::new(false);
        let d = Instant::now() + Duration::from_secs(20);
        let journal = FreshInstallJournal::create(
            &manifest,
            &root.join("absent-bundle"),
            &root.join("journal"),
            &c,
            d,
        )
        .map_err(|e| io::Error::from(e.kind()))?;
        let _retained = journal
            .record_phase(InstallJournalPhase::Prepared, &"b".repeat(64), &c, d)
            .map_err(|e| io::Error::from(e.kind()))?;
        let ready = serde_json::to_vec(
            &serde_json::json!({"schema":1,"phase":"prepared-synced-before-explicit-exit","snapshot":snapshot(&root.join("journal"),&c,d)?}),
        )?;
        if ready.len() + 1 > 4096 {
            return Err(bad());
        }
        let mut out = std::io::stdout().lock();
        out.write_all(&ready)?;
        out.write_all(b"\n")?;
        out.flush()?;
        // Intentionally bypass Rust Drop; no other process is signalled.
        std::process::exit(73);
    }
    if a.len() != 2 || a[1] != "--fixed-journal-explicit-exit" {
        return Err(bad());
    }
    let dir = tempfile::Builder::new()
        .prefix("dlpad-journal-explicit-exit-owned-")
        .tempdir()?;
    let root = dir.keep();
    let sentinel = root.join("foreign-sentinel");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sentinel)?
        .write_all(b"preserve")?;
    let cancel = Arc::new(AtomicBool::new(false));
    let result = process::run_metadata(
        &ProcessSpec {
            runtime_environment: None,
            executable: std::env::current_exe()?,
            args: vec![
                "--journal-child-after-prepared".into(),
                root.clone().into_os_string(),
            ],
            timeout: Duration::from_secs(30),
            max_log_bytes: 4096,
        },
        &cancel,
    )?;
    if result.execution.outcome != ProcessOutcome::Exited
        || result.execution.exit_code != Some(73)
        || result.execution.truncated
        || result.stdout.len() > 4096
        || !result.stdout.ends_with(b"\n")
        || result.stdout[..result.stdout.len() - 1].contains(&b'\n')
    {
        return Err(bad());
    }
    let ready: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    if ready["schema"] != 1 || ready["phase"] != "prepared-synced-before-explicit-exit" {
        return Err(bad());
    }
    let inspection_deadline = Instant::now() + Duration::from_secs(10);
    let before = snapshot(&root.join("journal"), &cancel, inspection_deadline)?;
    if before != ready["snapshot"] {
        return Err(bad());
    }
    let inspection = inspect_install_journal(&root.join("journal"), &cancel, inspection_deadline)?;
    if inspection.evidence().len() != 1 {
        return Err(bad());
    }
    inspection.verify_current(&cancel, inspection_deadline)?;
    drop(inspection);
    if snapshot(&root.join("journal"), &cancel, inspection_deadline)? != before
        || std::fs::read(&sentinel)? != b"preserve"
    {
        return Err(bad());
    }
    let report = serde_json::json!({"schema":1,"phase":"journal-explicit-process-exit-evidence","child_exit":73,"prefix_preserved":true,"snapshot":before,"owned_root":root,"crash_commit_proven":false,"resume_authority":false,"deletion_authority":false,"actual_launch":false});
    let b = serde_json::to_vec(&report)?;
    if b.len() + 1 > 4096 {
        return Err(bad());
    }
    let mut out = std::io::stdout().lock();
    out.write_all(&b)?;
    out.write_all(b"\n")?;
    out.flush()
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("journal interruption fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    std::process::exit(1);
}
