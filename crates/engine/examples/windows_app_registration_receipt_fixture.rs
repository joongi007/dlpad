//! Fresh registration observation; not active install, publisher trust or cold removal.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            create_registration_receipt,
            manifest::{BundleEntry, BundleManifest},
            prepare_new, register_fresh,
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Component, Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    fn bad() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "fresh registration fixture rejected",
        )
    }
    fn hash(path: &Path, end: Instant) -> io::Result<String> {
        let mut f = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() > 128 * 1024 * 1024 {
            return Err(bad());
        }
        let id = file_identity::from_file(&f)?;
        let mut h = Sha256::new();
        let mut b = [0; 65536];
        loop {
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let n = f.read(&mut b)?;
            if n == 0 {
                break;
            }
            h.update(&b[..n]);
        }
        if file_identity::from_file(&f)? != id {
            return Err(bad());
        }
        Ok(format!("{:x}", h.finalize()))
    }
    fn observed_bytes(path: &Path, max: usize) -> io::Result<Vec<u8>> {
        let f = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() > max as u64 {
            return Err(bad());
        }
        let mut b = Vec::new();
        f.take(max as u64 + 1).read_to_end(&mut b)?;
        if b.len() > max {
            return Err(bad());
        }
        Ok(b)
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 || args[1] != "--fixed-app-registration-receipt" {
        return Err(bad());
    }
    let source = PathBuf::from(&args[2]);
    let sha = args[3].to_str().ok_or_else(bad)?;
    if !source.is_absolute()
        || source
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        || sha.len() != 64
        || !sha
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(bad());
    }
    let source_meta = std::fs::symlink_metadata(&source)?;
    if !source_meta.is_file() || source_meta.file_type().is_symlink() {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(120);
    let input = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&source)?;
    let size = input.metadata()?.len();
    let source_id = file_identity::from_file(&input)?;
    if hash(&source, end)? != sha {
        return Err(bad());
    }
    let root = tempfile::Builder::new()
        .prefix("dlpad-app-receipt-owned-")
        .tempdir()?
        .keep();
    println!(
        "{}",
        serde_json::json!({"schema":1,"phase":"owned-fixture-created","root":root,"synthetic_only":true})
    );
    let notice_bytes = b"synthetic local CLI notice; no legal coverage claim\n";
    let sentinel = b"outside foreign synthetic preserve";
    let notice_path = root.join("notice.txt");
    let sentinel_path = root.join("foreign-sentinel");
    for (p, b) in [
        (&notice_path, notice_bytes.as_slice()),
        (&sentinel_path, sentinel.as_slice()),
    ] {
        let mut f = File::create_new(p)?;
        f.write_all(b)?;
        f.sync_all()?;
    }
    let notice = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&notice_path)?;
    let files = [("dlpad.exe", &input), ("notice.txt", &notice)];
    let m = BundleManifest {
        schema: 1,
        version: "0.1.0".into(),
        target: "x86_64-pc-windows-gnu".into(),
        development_only: true,
        entries: vec![
            BundleEntry {
                name: "dlpad.exe".into(),
                bytes: size,
                sha256: sha.into(),
            },
            BundleEntry {
                name: "notice.txt".into(),
                bytes: notice_bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(notice_bytes)),
            },
        ],
    };
    let mut cases = Vec::new();
    (&input).seek(SeekFrom::Start(17))?;
    let registered_path = root.join("registered-app");
    let prepared = prepare_new(&m, &files, &registered_path, &AtomicBool::new(false), end)
        .map_err(|e| io::Error::new(e.kind(), "receipt fixture prepare failed"))?;
    let registered = register_fresh(prepared, &AtomicBool::new(false), end)
        .map_err(|e| io::Error::new(e.kind(), "receipt fixture registration failed"))?;
    registered.verify_current(&AtomicBool::new(false), end)?;
    let registered_names = [
        "dlpad.exe",
        "notice.txt",
        "install-ready.json",
        "install-registration.json",
        "install-registration-seal.json",
    ];
    let mut original_hashes = Vec::new();
    for name in registered_names {
        original_hashes.push((name, hash(&registered_path.join(name), end)?));
    }
    for name in ["success", "cancel", "deadline", "overlap", "existing"] {
        let target = if name == "overlap" {
            registered_path.join("nested-receipt")
        } else {
            root.join(format!("receipt-{name}"))
        };
        if name == "existing" {
            std::fs::create_dir(&target)?;
            let mut f = File::create_new(target.join("foreign"))?;
            f.write_all(b"preserved fixture foreign")?;
            f.sync_all()?;
        }
        let outcome = create_registration_receipt(
            &registered,
            &target,
            &AtomicBool::new(name == "cancel"),
            if name == "deadline" {
                Instant::now()
            } else {
                end
            },
        );
        if name == "success" {
            let receipt =
                outcome.map_err(|e| io::Error::new(e.kind(), "receipt creation failed"))?;
            receipt.verify_current(&AtomicBool::new(false), end)?;
            let path = target.join("registration-receipt.json");
            let raw = observed_bytes(&path, 65536)?;
            let r = receipt.record();
            r.validate()?;
            if raw != serde_json::to_vec(r)?
                || receipt.path() != target
                || serde_json::to_value(&r.registration)?
                    != serde_json::to_value(registered.record())?
                || serde_json::to_value(&r.seal)? != serde_json::to_value(registered.seal())?
            {
                return Err(bad());
            }
            let open_id = |p: &Path| -> io::Result<file_identity::NativeFileIdentity> {
                let f = OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
                    .open(p)?;
                file_identity::from_file(&f)
            };
            if r.receipt_root_identity != open_id(&target)? || r.receipt_identity != open_id(&path)?
            {
                return Err(bad());
            }
            let names = std::fs::read_dir(&target)?
                .take(3)
                .map(|e| e.map(|e| e.file_name()))
                .collect::<io::Result<Vec<_>>>()?;
            if names != [std::ffi::OsString::from("registration-receipt.json")] {
                return Err(bad());
            }
            cases.push(serde_json::json!({"case":name,"record":r,"receipt_bytes":raw.len(),"receipt_sha256":format!("{:x}",Sha256::digest(&raw)),"exact_namespace_count":1}));
            drop(receipt);
            if observed_bytes(&path, 65536)? != raw {
                return Err(bad());
            }
        } else {
            let failure = outcome.err().ok_or_else(bad)?;
            if failure.retained_file_cap_count() != 0 || failure.root_capability_acquired() {
                return Err(bad());
            }
            if name == "cancel" && failure.kind() != io::ErrorKind::Interrupted
                || name == "deadline" && failure.kind() != io::ErrorKind::TimedOut
            {
                return Err(bad());
            }
            if matches!(name, "cancel" | "deadline" | "overlap")
                && (failure.mutation_may_have_occurred() || target.try_exists()?)
            {
                return Err(bad());
            }
            if name == "existing"
                && (observed_bytes(&target.join("foreign"), 64)? != b"preserved fixture foreign"
                    || std::fs::read_dir(&target)?.count() != 1)
            {
                return Err(bad());
            }
            cases.push(serde_json::json!({"case":name,"kind":format!("{:?}",failure.kind()),"mutation_may_have_occurred":failure.mutation_may_have_occurred(),"root_capability_acquired":false,"retained_file_caps":0,"original_preserved":true}));
            drop(failure);
        }
        registered.verify_current(&AtomicBool::new(false), end)?;
        for (original, sha) in &original_hashes {
            if hash(&registered_path.join(original), end)? != *sha {
                return Err(bad());
            }
        }
    }
    if (&input).stream_position()? != 17 {
        return Err(bad());
    }
    if hash(&source, end)? != sha
        || file_identity::from_file(&input)? != source_id
        || observed_bytes(&notice_path, 64)? != notice_bytes
        || observed_bytes(&sentinel_path, 64)? != sentinel
    {
        return Err(bad());
    }
    let result = serde_json::json!({"schema":1,"status":"PASS_REGISTRATION_RECEIPT_ONLY","owned_fixture_root":root,"source_path":source,"source_sha256_before":sha,"source_sha256_after":sha,"source_bytes":size,"source_identity":source_id,"notice_bytes":notice_bytes.len(),"source_notice_preserved":true,"outside_sentinel_preserved":true,"cases":cases,"actual_launch":false,"cold_uninstall_verified":false,"active_installation":false,"source_cursor_before":17,"source_cursor_after":17,"trusted_provenance":false});
    let text = serde_json::to_string(&result)?;
    if text.len() > 16384 {
        return Err(bad());
    }
    println!("{text}");
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("fresh registration fixture failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows fresh registration fixture unavailable");
        std::process::exit(2);
    }
}
