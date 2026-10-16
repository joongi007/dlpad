//! Fresh registration observation; not active install, publisher trust or cold removal.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
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
    if args.len() != 4 || args[1] != "--fixed-app-registration" {
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
        .prefix("dlpad-app-registration-owned-")
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
    for name in ["success", "foreign", "cancel", "deadline"] {
        let target = root.join(name);
        let bundle = prepare_new(&m, &files, &target, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "registration fixture prepare failed"))?;
        bundle.verify_current()?;
        let prepared = bundle.record().clone();
        let ready = target.join("install-ready.json");
        let ready_before = observed_bytes(&ready, 65536)?;
        if name == "foreign" {
            let mut f = File::create_new(target.join("own-foreign.txt"))?;
            f.write_all(b"fixture foreign preserve")?;
            f.sync_all()?;
        }
        let result = register_fresh(
            bundle,
            &AtomicBool::new(name == "cancel"),
            if name == "deadline" {
                Instant::now()
            } else {
                end
            },
        );
        if name == "success" {
            let registered = result.map_err(|e| io::Error::new(e.kind(), "registration failed"))?;
            registered.verify_current(&AtomicBool::new(false), end)?;
            let record_bytes = observed_bytes(&target.join("install-registration.json"), 65536)?;
            let seal_bytes = observed_bytes(&target.join("install-registration-seal.json"), 4096)?;
            let record: serde_json::Value = serde_json::from_slice(&record_bytes)?;
            if record != serde_json::to_value(registered.record())? {
                return Err(bad());
            }
            let seal: serde_json::Value = serde_json::from_slice(&seal_bytes)?;
            if seal != serde_json::to_value(registered.seal())? || registered.path() != target {
                return Err(bad());
            }
            let mut names = std::fs::read_dir(&target)?
                .map(|e| e.map(|e| e.file_name()))
                .collect::<io::Result<Vec<_>>>()?;
            names.sort();
            let mut expected = vec![
                "dlpad.exe",
                "notice.txt",
                "install-ready.json",
                "install-registration.json",
                "install-registration-seal.json",
            ]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>();
            expected.sort();
            if names != expected {
                return Err(bad());
            }
            registered.record().validate()?;
            registered.seal().validate()?;
            if record_bytes != serde_json::to_vec(registered.record())?
                || seal_bytes != serde_json::to_vec(registered.seal())?
            {
                return Err(bad());
            }
            let r = registered.record();
            let seal_record = registered.seal();
            let open_id = |path: &Path| -> io::Result<file_identity::NativeFileIdentity> {
                let f = OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
                    .open(path)?;
                file_identity::from_file(&f)
            };
            let root_id = open_id(&target)?;
            if r.root_identity != root_id
                || r.record_identity != open_id(&target.join("install-registration.json"))?
                || r.seal_identity != open_id(&target.join("install-registration-seal.json"))?
                || r.ready_identity != open_id(&ready)?
                || r.prepared.root_identity != root_id
                || serde_json::to_value(&r.prepared)? != serde_json::to_value(&prepared)?
            {
                return Err(bad());
            }
            for entry in &prepared.entries {
                if entry.identity != open_id(&target.join(&entry.name))?
                    || hash(&target.join(&entry.name), end)? != entry.sha256
                {
                    return Err(bad());
                }
            }
            let prepared_hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&prepared)?));
            let mut digest = Sha256::new();
            digest.update(b"dlpad-app-registration-record-v1\0");
            digest.update(&record_bytes);
            let record_hash = format!("{:x}", digest.finalize());
            if r.prepared_sha256 != prepared_hash
                || seal_record.record_sha256 != record_hash
                || seal_record.root_identity != r.root_identity
                || seal_record.record_identity != r.record_identity
                || seal_record.seal_identity != r.seal_identity
                || seal_record.nonce != r.nonce
            {
                return Err(bad());
            }

            cases.push(serde_json::json!({"case":name,"record":record,"seal":seal,"record_bytes":record_bytes.len(),"seal_bytes":seal_bytes.len(),"exact_namespace_count":5,"verified_original_caps":true}));
            drop(registered);
            if !target.try_exists()?
                || observed_bytes(&target.join("install-registration.json"), 65536)? != record_bytes
                || observed_bytes(&target.join("install-registration-seal.json"), 4096)?
                    != seal_bytes
            {
                return Err(bad());
            }
        } else {
            let failure = result.err().ok_or_else(bad)?;
            if failure.mutation_may_have_occurred()
                || failure.retained_file_cap_count() != 0
                || failure.path_hint() != target
            {
                return Err(bad());
            }
            if name == "cancel" && failure.kind() != io::ErrorKind::Interrupted
                || name == "deadline" && failure.kind() != io::ErrorKind::TimedOut
            {
                return Err(bad());
            }
            if observed_bytes(&ready, 65536)? != ready_before
                || target.join("install-registration.json").try_exists()?
                || target.join("install-registration-seal.json").try_exists()?
            {
                return Err(bad());
            }
            for entry in &prepared.entries {
                if hash(&target.join(&entry.name), end)? != entry.sha256 {
                    return Err(bad());
                }
            }
            if name == "foreign"
                && observed_bytes(&target.join("own-foreign.txt"), 64)?
                    != b"fixture foreign preserve"
            {
                return Err(bad());
            }
            cases.push(serde_json::json!({"case":name,"failure_kind":format!("{:?}",failure.kind()),"created_registration_files":0,"prepared_bytes_preserved":true}));
            drop(failure);
            if observed_bytes(&ready, 65536)? != ready_before {
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
    let result = serde_json::json!({"schema":1,"status":"PASS_FRESH_REGISTRATION_ONLY","owned_fixture_root":root,"source_path":source,"source_sha256_before":sha,"source_sha256_after":sha,"source_bytes":size,"source_identity":source_id,"notice_bytes":notice_bytes.len(),"source_notice_preserved":true,"outside_sentinel_preserved":true,"cases":cases,"actual_launch":false,"cold_uninstall_verified":false,"active_installation":false,"source_cursor_before":17,"source_cursor_after":17,"trusted_provenance":false});
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
