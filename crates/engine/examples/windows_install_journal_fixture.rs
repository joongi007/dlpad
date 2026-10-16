//! Owned development journal evidence. No app launch, recovery mutation or deletion authority.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            FreshInstallJournal, InstallJournalPhase as Phase, create_fresh_user_bundle,
            inspect_install_journal, inspect_user_bundle,
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
    fn snapshot(path: &Path, cancel: &AtomicBool, end: Instant) -> io::Result<serde_json::Value> {
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
            let (digest, bytes) = hash(&mut f, 65536, cancel, end)?;
            if bytes > 65536 {
                return Err(bad());
            }
            rows.push(serde_json::json!({"name":p.file_name().ok_or_else(bad)?,"identity":id,"sha256":digest,"bytes":bytes}));
        }
        Ok(serde_json::json!({"root_identity":rootid,"entries":rows}))
    }
    let argv: Vec<_> = std::env::args_os().collect();
    if argv.len() != 4 || argv[1] != "--fixed-install-journal" {
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
        .prefix("dlpad-install-journal-owned-")
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
    let prefixpath = owned.join("prefix");
    let prefix = FreshInstallJournal::create(&manifest, &bundlepath, &prefixpath, &c, end)
        .map_err(|e| io::Error::new(e.kind(), "journal create failed"))?;
    let prefix = prefix
        .record_phase(Phase::Prepared, &"b".repeat(64), &c, end)
        .map_err(|e| io::Error::new(e.kind(), "phase record failed"))?;
    drop(prefix);
    let prefixbefore = snapshot(&prefixpath, &c, end)?;
    let prefix = inspect_install_journal(&prefixpath, &c, end)?;
    prefix.verify_current(&c, end)?;
    if prefix.evidence().len() != 1 {
        return Err(bad());
    }
    drop(prefix);
    if snapshot(&prefixpath, &c, end)? != prefixbefore {
        return Err(bad());
    }
    let bundle = create_fresh_user_bundle(
        &manifest,
        &[("dlpad.exe", &source), ("notice.txt", &notice)],
        &bundlepath,
        &c,
        end,
    )
    .map_err(|e| io::Error::new(e.kind(), "bundle failed"))?;
    bundle.verify_current(&c, end)?;
    let completepath = owned.join("complete");
    let mut journal = FreshInstallJournal::create(&manifest, &bundlepath, &completepath, &c, end)
        .map_err(|e| io::Error::new(e.kind(), "journal create failed"))?;
    // These supplied fingerprints are observations of this live fixture's verified caps;
    // journal alone does not validate prior phase authority or prove crash commit.
    for (phase, bytes) in [
        (
            Phase::Prepared,
            serde_json::to_vec(&bundle.registration().record().prepared)?,
        ),
        (
            Phase::Registered,
            serde_json::to_vec(bundle.registration().record())?,
        ),
        (
            Phase::Receipt,
            serde_json::to_vec(bundle.receipt().record())?,
        ),
        (Phase::Active, bundle.active().record().canonical_bytes()?),
    ] {
        bundle.verify_current(&c, end)?;
        journal = journal
            .record_phase(phase, &sha(&bytes), &c, end)
            .map_err(|e| io::Error::new(e.kind(), "phase record failed"))?;
    }
    drop(journal);
    let pristinepath = owned.join("cli-current");
    let mut journal = FreshInstallJournal::create(&manifest, &bundlepath, &pristinepath, &c, end)
        .map_err(|e| io::Error::new(e.kind(), "pristine journal create failed"))?;
    // These supplied fingerprints are observations of this live fixture's verified caps;
    // journal alone does not validate prior phase authority or prove crash commit.
    for (phase, bytes) in [
        (
            Phase::Prepared,
            serde_json::to_vec(&bundle.registration().record().prepared)?,
        ),
        (
            Phase::Registered,
            serde_json::to_vec(bundle.registration().record())?,
        ),
        (
            Phase::Receipt,
            serde_json::to_vec(bundle.receipt().record())?,
        ),
        (Phase::Active, bundle.active().record().canonical_bytes()?),
    ] {
        bundle.verify_current(&c, end)?;
        journal = journal
            .record_phase(phase, &sha(&bytes), &c, end)
            .map_err(|e| io::Error::new(e.kind(), "phase record failed"))?;
    }
    drop(journal);
    let pristine = snapshot(&pristinepath, &c, end)?;
    drop(bundle);
    let original = snapshot(&completepath, &c, end)?;
    let current = inspect_user_bundle(&bundlepath, &c, end)?;
    let pristine_inspection = inspect_install_journal(&pristinepath, &c, end)?;
    pristine_inspection.verify_active_bundle(&current, &c, end)?;
    drop(pristine_inspection);
    let incomplete = inspect_install_journal(&prefixpath, &c, end)?;
    if incomplete
        .verify_active_bundle(&current, &c, end)
        .err()
        .ok_or_else(bad)?
        .kind()
        != io::ErrorKind::InvalidData
    {
        return Err(bad());
    }
    drop(incomplete);
    if snapshot(&prefixpath, &c, end)? != prefixbefore {
        return Err(bad());
    }
    let reader = inspect_install_journal(&completepath, &c, end)?;
    reader.verify_active_bundle(&current, &c, end)?;
    for (cancel, deadline, kind) in [
        (true, end, io::ErrorKind::Interrupted),
        (false, Instant::now(), io::ErrorKind::TimedOut),
    ] {
        let flag = AtomicBool::new(cancel);
        if inspect_install_journal(&completepath, &flag, deadline)
            .err()
            .ok_or_else(bad)?
            .kind()
            != kind
            || reader
                .verify_current(&flag, deadline)
                .err()
                .ok_or_else(bad)?
                .kind()
                != kind
        {
            return Err(bad());
        }
    }
    drop(reader);
    if snapshot(&completepath, &c, end)? != original {
        return Err(bad());
    }
    let foreign = completepath.join("foreign.txt");
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&foreign)?;
    f.write_all(b"unknown-preserve")?;
    f.sync_all()?;
    drop(f);
    let unknownbefore = snapshot(&completepath, &c, end)?;
    if inspect_install_journal(&completepath, &c, end).is_ok()
        || snapshot(&completepath, &c, end)? != unknownbefore
    {
        return Err(bad());
    }
    // Separate fresh journal for torn evidence. Never overwrite prior successful evidence.
    let tornpath = owned.join("torn");
    let torn = FreshInstallJournal::create(&manifest, &bundlepath, &tornpath, &c, end)
        .map_err(|e| io::Error::new(e.kind(), "journal create failed"))?;
    let torn = torn
        .record_phase(Phase::Prepared, &"b".repeat(64), &c, end)
        .map_err(|e| io::Error::new(e.kind(), "torn phase creation failed"))?;
    drop(torn);
    let original_torn = inspect_install_journal(&tornpath, &c, end)?;
    let protected_identity = original_torn.evidence()[0].file_identity.clone();
    drop(original_torn);
    check(&c, end)?;
    let mut f = OpenOptions::new()
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(tornpath.join("prepared.json"))?;
    if file_identity::from_file(&f)? != protected_identity {
        return Err(bad());
    }
    // Only this new fixture's protected phase is truncated; no successful prior evidence touched.
    f.set_len(0)?;
    f.write_all(b"{torn")?;
    if file_identity::from_file(&f)? != protected_identity {
        return Err(bad());
    }
    check(&c, end)?;
    f.sync_all()?;
    drop(f);
    let tornbefore = snapshot(&tornpath, &c, end)?;
    if inspect_install_journal(&tornpath, &c, end).is_ok()
        || snapshot(&tornpath, &c, end)? != tornbefore
    {
        return Err(bad());
    }
    current.verify_current(&c, end)?;
    if source.stream_position()? != 17
        || file_identity::from_file(&source)? != sourceid
        || std::fs::read(&noticepath)? != noticebytes
        || std::fs::read(&sentinel)? != b"preserve outside journal and bundle"
    {
        return Err(bad());
    }
    let oversizedpath = owned.join("oversized-journal-bound-fixture");
    let oversizedwriter = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&oversizedpath)?;
    oversizedwriter.set_len(65537)?;
    drop(oversizedwriter);
    let mut oversized = File::open(&oversizedpath)?;
    oversized.seek(SeekFrom::Start(17))?;
    if hash(&mut oversized, 65536, &c, end)
        .err()
        .ok_or_else(bad)?
        .kind()
        != io::ErrorKind::InvalidData
        || oversized.stream_position()? != 17
    {
        return Err(bad());
    }
    let (sourceafter, bytesafter) = hash(&mut source, 128 * 1024 * 1024, &c, end)?;
    if sourceafter != sourcesha || bytesafter != sourcebytes {
        return Err(bad());
    }
    if snapshot(&pristinepath, &c, end)? != pristine {
        return Err(bad());
    }
    let result = serde_json::json!({"schema":1,"status":"PASS","phase":"install-journal-evidence","owned_root":owned,"source_sha256":sourcesha,"source_bytes":sourcebytes,"source_identity":sourceid,"source_cursor17_preserved_by_operations":true,"pristine_current":pristine,"incomplete_prefix":prefixbefore,"complete_before_unknown":original,"unknown_preserved":unknownbefore,"torn_preserved":tornbefore,"current_bundle_binding_verified":true,"four_typed_failures_preserved":true,"crash_commit_proven":false,"prior_phase_authority_verified_by_journal":false,"deletion_authority":false,"actual_launch":false,"installer_complete":false,"journal_size_rejected_before_read":true,"torn_protected_identity_preserved":true});
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
        eprintln!("install journal fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(1);
    }
}
