//! Owned journal-aware public creation. No manual phase backfill or app launch.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            authorize_explicit_user_bundle_removal, create_journal_aware_bundle,
            manifest::{BundleEntry, BundleManifest},
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::Path,
        sync::atomic::{AtomicBool, Ordering},
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
    fn check(cancel: &AtomicBool, end: Instant) -> io::Result<()> {
        if cancel.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    fn hash(
        file: &mut File,
        maxsize: u64,
        cancel: &AtomicBool,
        end: Instant,
    ) -> io::Result<(String, u64)> {
        check(cancel, end)?;
        let size = file.metadata()?.len();
        if !file.metadata()?.is_file() || size > maxsize {
            return Err(bad());
        }
        let id = file_identity::from_file(file)?;
        file.seek(SeekFrom::Start(0))?;
        let mut digest = Sha256::new();
        let mut buffer = [0; 65536];
        let mut total = 0u64;
        loop {
            check(cancel, end)?;
            let remaining = (size - total + 1).min(buffer.len() as u64) as usize;
            let n = file.read(&mut buffer[..remaining])?;
            check(cancel, end)?;
            if n == 0 {
                break;
            }
            total = total.checked_add(n as u64).ok_or_else(bad)?;
            if total > size {
                return Err(bad());
            }
            digest.update(&buffer[..n]);
        }
        if total != size || file.metadata()?.len() != size || file_identity::from_file(file)? != id
        {
            return Err(bad());
        }
        Ok((format!("{:x}", digest.finalize()), size))
    }
    fn snapshot(
        path: &Path,
        maxsize: u64,
        cancel: &AtomicBool,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        let root = open(path)?;
        let rootid = file_identity::from_file(&root)?;
        let mut names = Vec::new();
        for e in std::fs::read_dir(path)? {
            let e = e?;
            if names.len() >= 6 || !e.file_type()?.is_file() || e.file_type()?.is_symlink() {
                return Err(bad());
            }
            names.push(e.path());
        }
        names.sort();
        let mut rows = Vec::new();
        for p in names {
            let mut f = open(&p)?;
            let id = file_identity::from_file(&f)?;
            let (digest, bytes) = hash(&mut f, maxsize, cancel, end)?;
            if bytes > maxsize {
                return Err(bad());
            }
            rows.push(serde_json::json!({"name":p.file_name().ok_or_else(bad)?,"identity":id,"sha256":digest,"bytes":bytes}));
        }
        Ok(serde_json::json!({"root_identity":rootid,"entries":rows}))
    }
    let args: Vec<_> = std::env::args_os().take(3).collect();
    if args.len() != 2 || args[1] != "--fixed-explicit-bundle-removal" {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(120);
    let c = AtomicBool::new(false);
    let temp = tempfile::Builder::new()
        .prefix("dlpad-bundle-removal-owned-")
        .tempdir()?;
    let owned = temp.keep();
    let mut bytes = vec![0u8; 70000];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
    bytes[64..68].copy_from_slice(b"PE\0\0");
    bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
    bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
    bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
    let sourcepath = owned.join("synthetic-source.exe");
    let mut created = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sourcepath)?;
    created.write_all(&bytes)?;
    created.sync_all()?;
    drop(created);
    let mut source = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&sourcepath)?;
    let sourceid = file_identity::from_file(&source)?;
    let (sourcesha, sourcebytes) = hash(&mut source, 70000, &c, end)?;
    source.seek(SeekFrom::Start(17))?;
    let sentinel = owned.join("foreign-sentinel");
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sentinel)?;
    f.write_all(b"outside-preserve")?;
    f.sync_all()?;
    drop(f);
    let manifest = BundleManifest {
        schema: 1,
        version: "0.1.0".into(),
        target: if cfg!(target_env = "msvc") {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-pc-windows-gnu"
        }
        .into(),
        development_only: true,
        entries: vec![BundleEntry {
            name: "dlpad.exe".into(),
            bytes: sourcebytes,
            sha256: sourcesha.clone(),
        }],
    };
    let mut rows = Vec::new();
    for case in ["normal", "cancel-retry", "foreign", "deadline"] {
        check(&c, end)?;
        let bundle = owned.join(format!("{case}-bundle"));
        let journal = owned.join(format!("{case}-journal"));
        let prepared = create_journal_aware_bundle(
            &manifest,
            &[("dlpad.exe", &source)],
            &bundle,
            &journal,
            &c,
            end,
        )
        .map_err(|e| io::Error::from(e.kind()))?;
        prepared.verify_current(&c, end)?;
        drop(prepared);
        // These roots are freshly observed after producer Drop, not historical capabilities.
        let rootid = file_identity::from_file(&open(&bundle)?)?;
        let before = snapshot(&journal, 65536, &c, end)?;
        if case == "deadline" {
            let error =
                match authorize_explicit_user_bundle_removal(&bundle, &journal, &c, Instant::now())
                {
                    Err(e) => e,
                    Ok(_) => return Err(bad()),
                };
            if error.kind() != io::ErrorKind::TimedOut
                || error.accepted_file_dispositions() != 0
                || error.accepted_root_dispositions() != 0
                || error.mutation_may_have_occurred()
            {
                return Err(bad());
            }
            rows.push(serde_json::json!({"case":case,"kind":"TimedOut","accepted_files":0,"accepted_roots":0,"bundle_preserved":bundle.exists(),"root_identity_current_before":rootid}));
        } else {
            let removal = authorize_explicit_user_bundle_removal(&bundle, &journal, &c, end)
                .map_err(|e| io::Error::from(e.kind()))?;
            if case == "foreign" {
                let path = bundle.join("foreign.txt");
                let mut f = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                f.write_all(b"foreign-preserve")?;
                f.sync_all()?;
                drop(f);
                let error = match removal.remove(&c, end) {
                    Err(e) => e,
                    Ok(_) => return Err(bad()),
                };
                if error.kind() != io::ErrorKind::InvalidData
                    || error.accepted_file_dispositions() != 0
                    || error.accepted_root_dispositions() != 0
                    || error.mutation_may_have_occurred()
                    || std::fs::read(&path)? != b"foreign-preserve"
                {
                    return Err(bad());
                }
                rows.push(serde_json::json!({"case":case,"kind":"InvalidData","accepted_files":0,"accepted_roots":0,"foreign_preserved":true,"root_identity_current_before":rootid}));
            } else {
                let outcome = if case == "cancel-retry" {
                    let stop = AtomicBool::new(true);
                    let error = match removal.remove(&stop, end) {
                        Err(e) => e,
                        Ok(_) => return Err(bad()),
                    };
                    if error.kind() != io::ErrorKind::Interrupted
                        || error.accepted_file_dispositions() != 0
                        || error.accepted_root_dispositions() != 0
                        || error.mutation_may_have_occurred()
                        || error.retained_file_cap_count() != 6
                        || error.retained_root_cap_count() != 4
                    {
                        return Err(bad());
                    }
                    error
                        .retry(&c, end)
                        .map_err(|e| io::Error::from(e.kind()))?
                } else {
                    removal
                        .remove(&c, end)
                        .map_err(|e| io::Error::from(e.kind()))?
                };
                if outcome.accepted_file_dispositions != 6
                    || outcome.accepted_root_dispositions != 4
                    || !outcome.bundle_root_disposition_accepted
                    || !outcome.outer_namespace_absence_confirmed
                    || bundle.exists()
                {
                    return Err(bad());
                }
                rows.push(serde_json::json!({"case":case,"accepted_files":6,"accepted_roots":4,"bundle_absent":true,"original_failure_retry_used":case=="cancel-retry","root_identity_current_before":rootid}));
            }
        }
        if snapshot(&journal, 65536, &c, end)? != before
            || source.stream_position()? != 17
            || file_identity::from_file(&source)? != sourceid
            || std::fs::read(&sentinel)? != b"outside-preserve"
        {
            return Err(bad());
        }
        rows.last_mut()
            .ok_or_else(bad)?
            .as_object_mut()
            .ok_or_else(bad)?
            .insert("journal_preserved".into(), true.into());
    }
    source.seek(SeekFrom::Start(0))?;
    let (now, len) = hash(&mut source, 70000, &c, end)?;
    if now != sourcesha || len != sourcebytes || file_identity::from_file(&source)? != sourceid {
        return Err(bad());
    }
    source.seek(SeekFrom::Start(17))?;
    let result = serde_json::json!({"schema":1,"status":"PASS","phase":"explicit-user-bundle-removal","owned_root":owned,"cases":rows,"source_identity":sourceid,"source_sha256":sourcesha,"source_bytes":sourcebytes,"source_cursor17_preserved":true,"synthetic_PE_not_currentCPU":true,"historical_ownership":false,"journal_deleted":false,"actual_launch":false,"installer_complete":false,"partial_postdisposition_cancel_verified":false});
    let encoded = serde_json::to_vec(&result)?;
    if encoded.len() + 1 > 16384 {
        return Err(bad());
    }
    let mut out = std::io::stdout().lock();
    out.write_all(&encoded)?;
    out.write_all(b"\n")?;
    out.flush()
}
fn main() {
    #[cfg(windows)]
    if let Err(error) = run() {
        eprintln!("bundle removal fixture failed: {:?}", error.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    std::process::exit(1);
}
