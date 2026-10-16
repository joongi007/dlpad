//! Explicit synthetic registered removal; no automatic cleanup or publisher trust.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            authorize_explicit_registered_removal, create_registration_receipt,
            inspect_registered_with_receipt,
            manifest::{BundleEntry, BundleManifest},
            prepare_new, register_fresh,
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
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
        let initial_len = f.metadata()?.len();
        let id = file_identity::from_file(&f)?;
        let mut total = 0u64;
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
            total = total.checked_add(n as u64).ok_or_else(bad)?;
            if total > initial_len || total > 128 * 1024 * 1024 {
                return Err(bad());
            }
            h.update(&b[..n]);
        }
        if total != initial_len
            || f.metadata()?.len() != initial_len
            || file_identity::from_file(&f)? != id
        {
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
    if args.len() != 4 || args[1] != "--fixed-app-registered-removal" {
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
    let end = Instant::now() + Duration::from_secs(140);
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
        .prefix("dlpad-app-cold-removal-owned-")
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
    for name in ["normal", "cancel", "deadline", "foreign"] {
        let registered_path = root.join(format!("registered-{name}"));
        let receipt_path = root.join(format!("receipt-{name}"));
        let prepared = prepare_new(&m, &files, &registered_path, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "removal prepare failed"))?;
        let registered = register_fresh(prepared, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "removal register failed"))?;
        let receipt =
            create_registration_receipt(&registered, &receipt_path, &AtomicBool::new(false), end)
                .map_err(|e| io::Error::new(e.kind(), "removal receipt failed"))?;
        let receipt_file = receipt_path.join("registration-receipt.json");
        let receipt_before = observed_bytes(&receipt_file, 65536)?;
        let record = registered.record().clone();
        drop(receipt);
        drop(registered);
        let mut original_bytes = Vec::new();
        for file_name in [
            "dlpad.exe",
            "notice.txt",
            "install-ready.json",
            "install-registration.json",
            "install-registration-seal.json",
        ] {
            original_bytes.push((file_name, hash(&registered_path.join(file_name), end)?));
        }
        let pair = inspect_registered_with_receipt(
            &registered_path,
            &receipt_path,
            &AtomicBool::new(false),
            end,
        )?;
        let authority = authorize_explicit_registered_removal(pair, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "removal authorization failed"))?;
        let mut foreign = None;
        if name == "foreign" {
            let p = registered_path.join("fixture-foreign");
            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .access_mode(FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .create_new(true)
                .open(&p)?;
            f.write_all(b"original fixture foreign")?;
            f.sync_all()?;
            let id = file_identity::from_file(&f)?;
            foreign = Some((p, f, id));
        }
        let removed = authority.remove(
            &AtomicBool::new(name == "cancel"),
            if name == "deadline" {
                Instant::now()
            } else {
                end
            },
        );
        let outcome = if name == "normal" {
            removed.map_err(|e| io::Error::new(e.kind(), "normal removal failed"))?
        } else {
            let failure = removed.err().ok_or_else(bad)?;
            if failure.accepted_file_dispositions() != 0 || failure.retained_file_cap_count() != 5 {
                return Err(bad());
            }
            if name == "cancel" && failure.kind() != io::ErrorKind::Interrupted
                || name == "deadline" && failure.kind() != io::ErrorKind::TimedOut
            {
                return Err(bad());
            }
            for (file_name, expected) in &original_bytes {
                if hash(&registered_path.join(file_name), end)? != *expected {
                    return Err(bad());
                }
            }
            if let Some((p, f, id)) = foreign.take() {
                let current = OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(&p)?;
                if file_identity::from_file(&current)? != id
                    || file_identity::from_file(&f)? != id
                    || observed_bytes(&p, 64)? != b"original fixture foreign"
                {
                    return Err(bad());
                }
                drop(current);
                let mut disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
                // SAFETY: retained fixture-created original DELETE HANDLE only;
                // no inferred path ownership or deletion of external objects.
                if unsafe {
                    SetFileInformationByHandle(
                        f.as_raw_handle(),
                        FileDispositionInfo,
                        (&mut disposition as *mut FILE_DISPOSITION_INFO).cast(),
                        std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                drop(f);
                if p.try_exists()? {
                    return Err(bad());
                }
            }
            cases.push(serde_json::json!({"case":name,"failure_kind":format!("{:?}",failure.kind()),"accepted_before_retry":0,"retained_original_file_caps":5}));
            failure
                .retry(&AtomicBool::new(false), end)
                .map_err(|e| io::Error::new(e.kind(), "same-cap removal retry failed"))?
        };
        if outcome.accepted_file_dispositions != 5
            || !outcome.root_disposition_accepted
            || registered_path.try_exists()?
            || observed_bytes(&receipt_file, 65536)? != receipt_before
        {
            return Err(bad());
        }
        cases.push(serde_json::json!({"case":name,"accepted_file_dispositions":5,"root_disposition_accepted":true,"target_absent":true,"receipt_sha256":format!("{:x}",Sha256::digest(&receipt_before)),"receipt_preserved":true,"retry_same_caps":name!="normal","original_registration":record}));
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
    let result = serde_json::json!({"schema":1,"status":"PASS_EXPLICIT_REGISTERED_REMOVAL_ONLY","owned_fixture_root":root,"source_path":source,"source_sha256_before":sha,"source_sha256_after":sha,"source_bytes":size,"source_identity":source_id,"notice_bytes":notice_bytes.len(),"source_notice_preserved":true,"outside_sentinel_preserved":true,"cases":cases,"actual_launch":false,"explicit_synthetic_registered_removal_verified":true,"automatic_cold_cleanup":false,"secure_erasure_verified":false,"historical_ownership_verified":false,"active_installation":false,"source_cursor_before":17,"source_cursor_after":17,"trusted_provenance":false});
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
