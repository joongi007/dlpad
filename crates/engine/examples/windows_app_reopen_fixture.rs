//! Read-only prepared-folder observation; no ownership, execution, or deletion.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::app_install::inspect_prepared;
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read},
        os::windows::fs::{FileExt, OpenOptionsExt},
        path::{Component, Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    fn bad() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "prepared reopen fixture rejected",
        )
    }
    fn open(p: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(p)
    }
    fn bytes(p: &Path, max: usize) -> io::Result<Vec<u8>> {
        let f = open(p)?;
        if !f.metadata()?.is_file() {
            return Err(bad());
        }
        let mut b = Vec::new();
        f.take(max as u64 + 1).read_to_end(&mut b)?;
        if b.len() > max {
            return Err(bad());
        }
        Ok(b)
    }
    fn hash(f: &File, size: u64, deadline: Instant) -> io::Result<String> {
        if !f.metadata()?.is_file() || f.metadata()?.len() != size || size > 128 * 1024 * 1024 {
            return Err(bad());
        }
        let id = dlpad_engine::file_identity::from_file(f)?;
        let mut offset = 0u64;
        let mut digest = Sha256::new();
        let mut b = [0u8; 65536];
        loop {
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let want = (size.saturating_sub(offset)).min(b.len() as u64) as usize;
            if want == 0 {
                if f.seek_read(&mut [0], offset)? != 0 {
                    return Err(bad());
                }
                break;
            }
            let n = f.seek_read(&mut b[..want], offset)?;
            if n == 0 {
                return Err(bad());
            }
            offset += n as u64;
            digest.update(&b[..n]);
        }
        if f.metadata()?.len() != size || dlpad_engine::file_identity::from_file(f)? != id {
            return Err(bad());
        }
        Ok(format!("{:x}", digest.finalize()))
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 || args[1] != "--fixed-app-reopen" {
        return Err(bad());
    }
    let path = PathBuf::from(&args[2]);
    let expected = args[3].to_str().ok_or_else(bad)?;
    if !path.is_absolute()
        || path.components().any(|c| matches!(c, Component::ParentDir))
        || expected.len() != 64
        || !expected
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(60);
    let marker = path.join("install-ready.json");
    let before = bytes(&marker, 65536)?;
    let inspection = inspect_prepared(&path, &AtomicBool::new(false), end)?;
    inspection.verify_current(&AtomicBool::new(false), end)?;
    let record = inspection.record();
    let executable = inspection.executable();
    let size = executable.metadata()?.len();
    let identity = dlpad_engine::file_identity::from_file(executable)?;
    let digest = hash(executable, size, end)?;
    if digest != expected
        || inspection.path() != path
        || !record.entries.iter().any(|e| {
            e.name == "dlpad.exe"
                && e.bytes == size
                && e.sha256 == expected
                && e.identity == identity
        })
        || record.trusted_provenance
    {
        return Err(bad());
    }
    let value: serde_json::Value = serde_json::from_slice(&before).map_err(|_| bad())?;
    if value != serde_json::to_value(record)? {
        return Err(bad());
    }
    let mut snapshots = Vec::new();
    for entry in &record.entries {
        let f = open(&path.join(&entry.name))?;
        let sha = hash(&f, entry.bytes, end)?;
        if sha != entry.sha256 {
            return Err(bad());
        }
        snapshots.push((entry.name.clone(), entry.bytes, sha));
    }
    let mut failures = Vec::new();
    for (cancel, expired, kind) in [
        (true, false, io::ErrorKind::Interrupted),
        (false, true, io::ErrorKind::TimedOut),
    ] {
        let deadline = if expired { Instant::now() } else { end };
        let error = inspect_prepared(&path, &AtomicBool::new(cancel), deadline)
            .err()
            .ok_or_else(bad)?;
        if error.kind() != kind {
            return Err(bad());
        }
        let current_error = inspection
            .verify_current(&AtomicBool::new(cancel), deadline)
            .err()
            .ok_or_else(bad)?;
        if current_error.kind() != kind {
            return Err(bad());
        }
        failures.push(serde_json::json!({"cancel":cancel,"expired":expired,"inspect_kind":format!("{:?}",error.kind()),"verify_kind":format!("{:?}",current_error.kind())}));
    }
    for (name, size, sha) in &snapshots {
        if hash(&open(&path.join(name))?, *size, end)? != *sha {
            return Err(bad());
        }
    }
    let after = bytes(&marker, 65536)?;
    if before != after {
        return Err(bad());
    }
    inspection.verify_current(&AtomicBool::new(false), end)?;
    let result = serde_json::json!({"schema":1,"status":"PASS_REOPEN_ONLY","prepared_path":inspection.path(),"record":record,"executable_bytes":size,"executable_sha256":digest,"executable_identity":identity,"marker_bytes":before.len(),"marker_sha256_before":format!("{:x}",Sha256::digest(&before)),"marker_sha256_after":format!("{:x}",Sha256::digest(&after)),"marker_unchanged":true,"entry_bytes_preserved":true,"negative_cases":failures,"actual_launch":false,"ownership_granted":false,"deletion_authority":false,"trusted_provenance":false,"installer_lifecycle_verified":false});
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
        eprintln!("prepared reopen fixture failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows prepared reopen fixture unavailable");
        std::process::exit(2);
    }
}
