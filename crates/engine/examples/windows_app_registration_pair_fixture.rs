//! Receipt-only readonly observation, no current installed-app authority.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{app_install::inspect_registered_with_receipt, file_identity};
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read},
        os::windows::fs::OpenOptionsExt,
        path::{Component, Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    fn bad() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "receipt reopen fixture rejected",
        )
    }
    fn open(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
    }
    fn bytes(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
        let f = open(path)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() > limit as u64 {
            return Err(bad());
        }
        let mut b = Vec::new();
        f.take(limit as u64 + 1).read_to_end(&mut b)?;
        if b.len() > limit {
            return Err(bad());
        }
        Ok(b)
    }
    fn hash(path: &Path, end: Instant) -> io::Result<String> {
        let mut f = open(path)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() > 128 * 1024 * 1024 {
            return Err(bad());
        }
        let id = file_identity::from_file(&f)?;
        let mut h = Sha256::new();
        let mut buf = [0; 65536];
        loop {
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        if file_identity::from_file(&f)? != id {
            return Err(bad());
        }
        Ok(format!("{:x}", h.finalize()))
    }
    let argv: Vec<_> = std::env::args_os().collect();
    if argv.len() != 4 || argv[1] != "--fixed-app-registration-pair" {
        return Err(bad());
    }
    let parent_path = PathBuf::from(&argv[2]);
    let parent = parent_path.as_path();
    let expected = argv[3].to_str().ok_or_else(bad)?;
    if !parent.is_absolute()
        || parent
            .components()
            .any(|c| matches!(c, Component::ParentDir))
        || !parent
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.starts_with("dlpad-app-receipt-owned-"))
        || expected.len() != 64
        || !expected
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(bad());
    }
    for p in parent.ancestors() {
        let m = std::fs::symlink_metadata(p)?;
        if !m.is_dir() || m.file_type().is_symlink() {
            return Err(bad());
        }
    }
    let path = parent.join("receipt-success");
    let registered_path = parent.join("registered-app");
    let end = Instant::now() + Duration::from_secs(60);
    let file = path.join("registration-receipt.json");
    let before = bytes(&file, 65536)?;
    if format!("{:x}", Sha256::digest(&before)) != expected {
        return Err(bad());
    }
    let preserved = [
        parent.join("notice.txt"),
        parent.join("foreign-sentinel"),
        parent.join("registered-app/dlpad.exe"),
        parent.join("registered-app/notice.txt"),
        parent.join("registered-app/install-ready.json"),
        parent.join("registered-app/install-registration.json"),
        parent.join("registered-app/install-registration-seal.json"),
    ];
    let mut original = Vec::new();
    for p in &preserved {
        original.push((p.clone(), hash(p, end)?));
    }
    let inspect =
        inspect_registered_with_receipt(&registered_path, &path, &AtomicBool::new(false), end)?;
    inspect.verify_current(&AtomicBool::new(false), end)?;
    let record = inspect.receipt().record();
    record.validate()?;
    if serde_json::to_vec(record)? != before
        || inspect.receipt().path() != path
        || file_identity::from_file(&open(&path)?)? != record.receipt_root_identity
        || file_identity::from_file(&open(&file)?)? != record.receipt_identity
    {
        return Err(bad());
    }
    let names = || -> io::Result<Vec<std::ffi::OsString>> {
        std::fs::read_dir(&path)?
            .take(3)
            .map(|e| e.map(|e| e.file_name()))
            .collect()
    };
    if names()? != vec![std::ffi::OsString::from("registration-receipt.json")] {
        return Err(bad());
    }
    let registered = inspect.registered();
    if registered.path() != registered_path
        || serde_json::to_value(registered.record())? != serde_json::to_value(&record.registration)?
        || serde_json::to_value(registered.seal())? != serde_json::to_value(&record.seal)?
    {
        return Err(bad());
    }
    let registration_raw = bytes(&registered_path.join("install-registration.json"), 65536)?;
    let seal_raw = bytes(
        &registered_path.join("install-registration-seal.json"),
        4096,
    )?;
    if registration_raw != serde_json::to_vec(registered.record())?
        || seal_raw != serde_json::to_vec(registered.seal())?
    {
        return Err(bad());
    }
    let native = |p: &Path| -> io::Result<file_identity::NativeFileIdentity> {
        file_identity::from_file(&open(p)?)
    };
    let r = registered.record();
    if native(&registered_path)? != r.root_identity
        || native(&registered_path.join("install-registration.json"))? != r.record_identity
        || native(&registered_path.join("install-registration-seal.json"))? != r.seal_identity
        || native(&registered_path.join("install-ready.json"))? != r.ready_identity
    {
        return Err(bad());
    }
    let mut wanted = vec![
        "dlpad.exe",
        "notice.txt",
        "install-ready.json",
        "install-registration.json",
        "install-registration-seal.json",
    ]
    .into_iter()
    .map(std::ffi::OsString::from)
    .collect::<Vec<_>>();
    wanted.sort();
    let mut actual = std::fs::read_dir(&registered_path)?
        .take(7)
        .map(|e| e.map(|e| e.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    actual.sort();
    if actual != wanted {
        return Err(bad());
    }
    for entry in &r.prepared.entries {
        let p = registered_path.join(&entry.name);
        if native(&p)? != entry.identity
            || open(&p)?.metadata()?.len() != entry.bytes
            || hash(&p, end)? != entry.sha256
        {
            return Err(bad());
        }
    }
    let record_value = serde_json::to_value(record)?;
    let mut failures = Vec::new();
    for (cancel, expired, kind) in [
        (true, false, io::ErrorKind::Interrupted),
        (false, true, io::ErrorKind::TimedOut),
    ] {
        let deadline = if expired { Instant::now() } else { end };
        let e = inspect_registered_with_receipt(
            &registered_path,
            &path,
            &AtomicBool::new(cancel),
            deadline,
        )
        .err()
        .ok_or_else(bad)?;
        let v = inspect
            .verify_current(&AtomicBool::new(cancel), deadline)
            .err()
            .ok_or_else(bad)?;
        if e.kind() != kind || v.kind() != kind || bytes(&file, 65536)? != before {
            return Err(bad());
        }
        failures.push(serde_json::json!({"cancel":cancel,"expired":expired,"inspect_kind":format!("{:?}",e.kind()),"verify_kind":format!("{:?}",v.kind())}));
    }
    inspect.verify_current(&AtomicBool::new(false), end)?;
    drop(inspect);
    if bytes(&file, 65536)? != before
        || names()? != vec![std::ffi::OsString::from("registration-receipt.json")]
    {
        return Err(bad());
    }
    for (p, sha) in &original {
        if hash(p, end)? != *sha {
            return Err(bad());
        }
    }
    let mut after_names = std::fs::read_dir(&registered_path)?
        .take(7)
        .map(|e| e.map(|e| e.file_name()))
        .collect::<io::Result<Vec<_>>>()?;
    after_names.sort();
    if after_names != wanted {
        return Err(bad());
    }
    let result = serde_json::json!({"schema":1,"status":"PASS_REGISTRATION_PAIR_ONLY","receipt_path":path,"record":record_value,"receipt_bytes":before.len(),"receipt_sha256_before":expected,"receipt_sha256_after":expected,"original_bytes_preserved":true,"registered_and_external_bytes_preserved":true,"negative_cases":failures,"current_registered_target_verified":true,"mixed_valid_receipt_rejection_verified":false,"actual_launch":false,"deletion_authority":false,"historical_ownership_verified":false,"publisher_trust":false});
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
        eprintln!("receipt reopen fixture failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows receipt reopen fixture unavailable");
        std::process::exit(2);
    }
}
