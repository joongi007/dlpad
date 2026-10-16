//! Owned journal-aware public creation. No manual phase backfill or app launch.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            create_journal_aware_bundle, inspect_install_journal, inspect_user_bundle,
            manifest::{BundleEntry, BundleManifest},
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Path, PathBuf},
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
    fn sha(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
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
    let argv: Vec<_> = std::env::args_os().collect();
    if argv.len() != 4 || argv[1] != "--fixed-journal-aware-bundle" {
        return Err(bad());
    }
    let sourcepath = PathBuf::from(&argv[2]);
    let expected = argv[3].to_str().ok_or_else(bad)?;
    if !sourcepath.is_absolute()
        || expected.len() != 64
        || !expected
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(140);
    let c = AtomicBool::new(false);
    // Source denies write/delete for its entire use; hashing happens before cursor17 is set.
    let mut source = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&sourcepath)?;
    let sourceid = file_identity::from_file(&source)?;
    let (sourcesha, sourcebytes) = hash(&mut source, 128 * 1024 * 1024, &c, end)?;
    if sourcesha != expected {
        return Err(bad());
    }
    source.seek(SeekFrom::Start(17))?;
    let temp = tempfile::Builder::new()
        .prefix("dlpad-journal-aware-owned-")
        .tempdir()?;
    let owned = temp.keep();
    let noticepath = owned.join("notice.txt");
    let noticebytes = b"Synthetic journal fixture notice; not license grant.\n";
    std::fs::write(&noticepath, noticebytes)?;
    let notice = File::open(&noticepath)?;
    let sentinel = owned.join("foreign-sentinel");
    std::fs::write(&sentinel, b"preserve outside journal and bundle")?;
    let manifest = BundleManifest {
        schema: 1,
        version: "0.1.0".into(),
        target: "x86_64-pc-windows-msvc".into(),
        development_only: true,
        entries: vec![
            BundleEntry {
                name: "dlpad.exe".into(),
                bytes: sourcebytes,
                sha256: sourcesha.clone(),
            },
            BundleEntry {
                name: "notice.txt".into(),
                bytes: noticebytes.len() as u64,
                sha256: sha(noticebytes),
            },
        ],
    };
    let bundlepath = owned.join("bundle");
    let journalpath = owned.join("journal");
    let created = create_journal_aware_bundle(
        &manifest,
        &[("dlpad.exe", &source), ("notice.txt", &notice)],
        &bundlepath,
        &journalpath,
        &c,
        end,
    )
    .map_err(|e| io::Error::new(e.kind(), "journal-aware creation failed"))?;
    created.verify_current(&c, end)?;
    created.bundle().verify_current(&c, end)?;
    drop(created);
    let journalbefore = snapshot(&journalpath, 65536, &c, end)?;
    let outer = open(&bundlepath)?;
    let outerid = file_identity::from_file(&outer)?;
    let mut bundlebefore = Vec::new();
    for name in ["registered", "receipt", "active"] {
        bundlebefore.push(snapshot(
            &bundlepath.join(name),
            128 * 1024 * 1024,
            &c,
            end,
        )?);
    }
    let current = inspect_user_bundle(&bundlepath, &c, end)?;
    let journal = inspect_install_journal(&journalpath, &c, end)?;
    journal.verify_active_bundle(&current, &c, end)?;
    if journal.evidence().len() != 4 {
        return Err(bad());
    }
    let mut failures = Vec::new();
    for mode in ["cancel", "deadline", "existing-bundle", "existing-journal"] {
        let bp = owned.join(format!("{mode}-bundle"));
        let jp = owned.join(format!("{mode}-journal"));
        let existing = match mode {
            "existing-bundle" => Some(&bp),
            "existing-journal" => Some(&jp),
            _ => None,
        };
        if let Some(path) = existing {
            std::fs::create_dir(path)?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join("foreign"))?;
            f.write_all(b"foreign-preserve")?;
            f.sync_all()?;
        }
        let flag = AtomicBool::new(mode == "cancel");
        let deadline = if mode == "deadline" {
            Instant::now()
        } else {
            end
        };
        let failure = create_journal_aware_bundle(
            &manifest,
            &[("dlpad.exe", &source), ("notice.txt", &notice)],
            &bp,
            &jp,
            &flag,
            deadline,
        )
        .err()
        .ok_or_else(bad)?;
        let expected = match mode {
            "cancel" => io::ErrorKind::Interrupted,
            "deadline" => io::ErrorKind::TimedOut,
            _ => io::ErrorKind::AlreadyExists,
        };
        if failure.kind() != expected
            || failure.bundle_capabilities_present()
            || failure.journal_capabilities_present()
            || failure.mutation_may_have_occurred()
        {
            return Err(bad());
        }
        if let Some(path) = existing {
            if std::fs::read(path.join("foreign"))? != b"foreign-preserve"
                || std::fs::read_dir(path)?.count() != 1
            {
                return Err(bad());
            }
            if (if mode == "existing-bundle" { &jp } else { &bp }).exists() {
                return Err(bad());
            }
        } else if bp.exists() || jp.exists() {
            return Err(bad());
        }
        failures.push(serde_json::json!({"case":mode,"kind":format!("{:?}",failure.kind()),"mutation_may_have_occurred":false,"bundle_capabilities_present":false,"journal_capabilities_present":false,"original_foreign_preserved":true}));
    }
    journal.verify_active_bundle(&current, &c, end)?;
    drop(journal);
    drop(current);
    if snapshot(&journalpath, 65536, &c, end)? != journalbefore
        || file_identity::from_file(&outer)? != outerid
    {
        return Err(bad());
    }
    for (i, name) in ["registered", "receipt", "active"].iter().enumerate() {
        if snapshot(&bundlepath.join(name), 128 * 1024 * 1024, &c, end)? != bundlebefore[i] {
            return Err(bad());
        }
    }
    if source.stream_position()? != 17
        || file_identity::from_file(&source)? != sourceid
        || std::fs::read(&noticepath)? != noticebytes
        || std::fs::read(&sentinel)? != b"preserve outside journal and bundle"
    {
        return Err(bad());
    }
    let (sourceafter, bytesafter) = hash(&mut source, 128 * 1024 * 1024, &c, end)?;
    if sourceafter != sourcesha || bytesafter != sourcebytes {
        return Err(bad());
    }
    let result = serde_json::json!({"schema":1,"status":"PASS","phase":"journal-aware-bundle-created","owned_root":owned,"source_sha256":sourcesha,"source_bytes":sourcebytes,"source_identity":sourceid,"source_cursor17_preserved_by_operations":true,"journal":journalbefore,"bundle_outer_identity":outerid,"bundle_children":bundlebefore,"four_verified_stage_records_observed":true,"manual_backfill_used":false,"current_bundle_binding_verified":true,"negative_cases":failures,"historical_ownership":false,"crash_commit_proven":false,"deletion_authority":false,"actual_launch":false,"installer_complete":false});
    let bytes = serde_json::to_vec(&result)?;
    if bytes.len() > 16384 {
        return Err(bad());
    }
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&bytes)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("journal-aware fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(1);
    }
}
