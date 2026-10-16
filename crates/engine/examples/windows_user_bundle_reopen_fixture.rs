//! Windows-only: original passed bundle is readonly; separate owned negative fixtures mutate.
//! No process launch. Fixture-owned namespace changes are explicit and preserved for review.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            create_fresh_user_bundle, inspect_user_bundle,
            manifest::{BundleEntry, BundleManifest},
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read},
        os::windows::fs::OpenOptionsExt,
        path::Path,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE,
    };
    fn bad() -> io::Error {
        io::ErrorKind::InvalidData.into()
    }
    fn open(p: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(p)
    }
    fn snapshot(root: &Path, end: Instant) -> io::Result<serde_json::Value> {
        let outer = open(root)?;
        let outer_id = file_identity::from_file(&outer)?;
        let mut rows = Vec::new();
        for child in ["registered", "receipt", "active"] {
            let dir = root.join(child);
            let pin = open(&dir)?;
            let id = file_identity::from_file(&pin)?;
            let mut names = Vec::new();
            for e in std::fs::read_dir(&dir)? {
                let e = e?;
                if names.len() >= 64 || !e.file_type()?.is_file() || e.file_type()?.is_symlink() {
                    return Err(bad());
                }
                names.push(e.path());
            }
            names.sort();
            for p in names {
                let mut f = open(&p)?;
                let size = f.metadata()?.len();
                if size > 128 * 1024 * 1024 {
                    return Err(bad());
                }
                let before = file_identity::from_file(&f)?;
                let mut h = Sha256::new();
                let mut total = 0u64;
                let mut buffer = [0; 65536];
                loop {
                    if Instant::now() >= end {
                        return Err(io::ErrorKind::TimedOut.into());
                    }
                    let n = f.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    total = total.checked_add(n as u64).ok_or_else(bad)?;
                    if total > size {
                        return Err(bad());
                    }
                    h.update(&buffer[..n]);
                }
                if total != size
                    || f.metadata()?.len() != size
                    || file_identity::from_file(&f)? != before
                {
                    return Err(bad());
                }
                rows.push(serde_json::json!({"child":child,"name":p.file_name().ok_or_else(bad)?,"root_identity":id,"file_identity":before,"bytes":size,"sha256":format!("{:x}",h.finalize())}));
            }
        }
        if file_identity::from_file(&outer)? != outer_id {
            return Err(bad());
        }
        Ok(serde_json::json!({"outer_identity":outer_id,"files":rows}))
    }
    fn negative_cases(end: Instant) -> io::Result<serde_json::Value> {
        // Exact public factory fixture used by user_bundle.rs; synthetic PE is never executed.
        let temp = tempfile::Builder::new()
            .prefix("dlpad-bundle-reopen-negative-")
            .tempdir()?;
        let parent = temp.keep();
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let source = parent.join("source.bin");
        std::fs::write(&source, &bytes)?;
        let source_file = File::open(&source)?;
        let manifest = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            }],
        };
        let c = AtomicBool::new(false);
        let mut rows = Vec::new();
        for case in ["foreign", "missing"] {
            let target = parent.join(case);
            let value = create_fresh_user_bundle(
                &manifest,
                &[("dlpad.exe", &source_file)],
                &target,
                &c,
                end,
            )
            .map_err(|e| io::Error::new(e.kind(), "synthetic factory failed"))?;
            value.verify_current(&c, end)?;
            drop(value);
            let original = snapshot(&target, end)?;
            let foreign = target.join("foreign.bin");
            let moved = parent.join("missing-receipt-retained");
            if case == "foreign" {
                let mut f = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&foreign)?;
                use std::io::Write;
                f.write_all(b"foreign-preserve")?;
                f.sync_all()?;
            } else {
                std::fs::rename(target.join("receipt"), &moved)?;
            }
            let failure = inspect_user_bundle(&target, &c, end).unwrap_err();
            if failure.kind() != io::ErrorKind::InvalidData {
                return Err(bad());
            }
            if case == "foreign" {
                if std::fs::read(&foreign)? != b"foreign-preserve" {
                    return Err(bad());
                }
                // Remove only this fixture's own new known file to compare the original tree.
                std::fs::remove_file(&foreign)?;
            } else {
                if target.join("receipt").exists() || !moved.is_dir() {
                    return Err(bad());
                }
                // Restore this fixture's own moved child; API must not have repaired it.
                std::fs::rename(&moved, target.join("receipt"))?;
            }
            if snapshot(&target, end)? != original {
                return Err(bad());
            }
            rows.push(serde_json::json!({"case":case,"kind":"InvalidData","original_files_preserved":true,"fixture_only_namespace_restoration":true}));
        }
        if std::fs::read(&source)? != bytes {
            return Err(bad());
        }
        Ok(
            serde_json::json!({"retained_owned_fixture":parent,"cases":rows,"actual_launch":false,"valid_pair_mix_tested":false}),
        )
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 || args[1] != "--fixed-user-bundle-reopen" {
        return Err(bad());
    }
    let root = std::path::PathBuf::from(&args[2]);
    if !root.is_absolute() {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(90);
    let before = snapshot(&root, end)?;
    let c = AtomicBool::new(false);
    let value = inspect_user_bundle(&root, &c, end)?;
    value.verify_current(&c, end)?;
    for (cancel, deadline, kind) in [
        (true, end, io::ErrorKind::Interrupted),
        (false, Instant::now(), io::ErrorKind::TimedOut),
    ] {
        let flag = AtomicBool::new(cancel);
        if inspect_user_bundle(&root, &flag, deadline)
            .unwrap_err()
            .kind()
            != kind
            || value.verify_current(&flag, deadline).unwrap_err().kind() != kind
        {
            return Err(bad());
        }
    }
    drop(value);
    let after = snapshot(&root, end)?;
    if before != after {
        return Err(bad());
    }
    let negatives = negative_cases(end)?;
    let report = serde_json::json!({"schema":1,"status":"PASS","phase":"user-bundle-reopened","owned_root":root,"readonly":true,"four_typed_failures_preserved":true,"files":after,"fresh_negative_cases":negatives,"actual_launch":false,"historical_ownership":false,"deletion_authority":false,"installer_complete":false});
    let bytes = serde_json::to_vec(&report)?;
    if bytes.len() > 16384 {
        return Err(bad());
    }
    use std::io::Write;
    let mut output = std::io::stdout().lock();
    output.write_all(&bytes)?;
    output.write_all(b"\n")?;
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("user bundle fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(1);
    }
}
