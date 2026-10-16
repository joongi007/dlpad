//! Same-process original-capability discard; not cold uninstall or secure erase.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            discard_fresh,
            manifest::{BundleEntry, BundleManifest},
            prepare_new,
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Write},
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        path::{Component, Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    fn bad() -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "fresh discard fixture rejected")
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
    if args.len() != 4 || args[1] != "--fixed-app-discard" {
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
        .prefix("dlpad-app-discard-owned-")
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
    for name in ["success", "foreign", "cancel", "deadline"] {
        let target = root.join(name);
        let bundle = prepare_new(&m, &files, &target, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "discard fixture prepare failed"))?;
        bundle.verify_current()?;
        let record = bundle.record().clone();
        let marker = target.join("install-ready.json");
        let marker_before = observed_bytes(&marker, 65536)?;
        if marker_before.len() > 65536 {
            return Err(bad());
        }
        let mut foreign = None;
        if name == "foreign" {
            let path = target.join("own-foreign.txt");
            let mut f = OpenOptions::new()
                .read(true)
                .write(true)
                .access_mode(FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .create_new(true)
                .open(&path)?;
            f.write_all(b"known fixture foreign")?;
            f.sync_all()?;
            let id = file_identity::from_file(&f)?;
            foreign = Some((path, f, id));
        }
        let result = discard_fresh(
            bundle,
            &AtomicBool::new(name == "cancel"),
            if name == "deadline" {
                Instant::now()
            } else {
                end
            },
        );
        let outcome = if name == "success" {
            result.map_err(|e| io::Error::new(e.kind(), "discard success failed"))?
        } else {
            let failure = result.err().ok_or_else(bad)?;
            if failure.accepted_file_dispositions() != 0
                || failure.retained_file_cap_count() != 3
                || failure.path_hint() != target
                || serde_json::to_value(failure.original_record())?
                    != serde_json::to_value(&record)?
                || observed_bytes(&marker, 65536)? != marker_before
            {
                return Err(bad());
            }
            if name == "cancel" && failure.kind() != io::ErrorKind::Interrupted
                || name == "deadline" && failure.kind() != io::ErrorKind::TimedOut
            {
                return Err(bad());
            }
            for entry in &record.entries {
                if hash(&target.join(&entry.name), end)? != entry.sha256 {
                    return Err(bad());
                }
            }
            let kind = format!("{:?}", failure.kind());
            if let Some((path, f, id)) = foreign.take() {
                let current = OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(&path)?;
                if file_identity::from_file(&current)? != id
                    || file_identity::from_file(&f)? != id
                    || observed_bytes(&path, 64)? != b"known fixture foreign"
                {
                    return Err(bad());
                }
                drop(current);
                let mut info = FILE_DISPOSITION_INFO { DeleteFile: true };
                // SAFETY: fixture-created original DELETE HANDLE, retained identity
                // checked; this never reopens a path for delete or adopts foreign files.
                if unsafe {
                    SetFileInformationByHandle(
                        f.as_raw_handle(),
                        FileDispositionInfo,
                        (&mut info as *mut FILE_DISPOSITION_INFO).cast(),
                        std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                drop(f);
                if path.try_exists()? {
                    return Err(bad());
                }
            }
            cases.push(serde_json::json!({"case":name,"initial_failure_kind":kind,"initial_accepted":0,"initial_retained_caps":3,"original_record_unchanged":true,"original_bytes_unchanged":true}));
            failure
                .retry(&AtomicBool::new(false), end)
                .map_err(|e| io::Error::new(e.kind(), "original remainder retry failed"))?
        };
        if outcome.accepted_file_dispositions != 3
            || !outcome.root_disposition_accepted
            || target.try_exists()?
        {
            return Err(bad());
        }
        cases.push(serde_json::json!({"case":name,"final_outcome":outcome,"target_absent":true,"retry_used_original_caps":name!="success"}));
    }
    if hash(&source, end)? != sha
        || file_identity::from_file(&input)? != source_id
        || observed_bytes(&notice_path, 64)? != notice_bytes
        || observed_bytes(&sentinel_path, 64)? != sentinel
    {
        return Err(bad());
    }
    let result = serde_json::json!({"schema":1,"status":"PASS_FRESH_DISCARD_ONLY","owned_fixture_root":root,"source_path":source,"source_sha256_before":sha,"source_sha256_after":sha,"source_bytes":size,"source_identity":source_id,"notice_bytes":notice_bytes.len(),"source_notice_preserved":true,"outside_sentinel_preserved":true,"cases":cases,"actual_launch":false,"cold_uninstall_verified":false,"secure_erasure_verified":false,"trusted_provenance":false});
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
        eprintln!("fresh discard fixture failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows fresh discard fixture unavailable");
        std::process::exit(2);
    }
}
