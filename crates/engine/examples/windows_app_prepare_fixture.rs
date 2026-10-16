//! Local prepared copy acceptance only; no executable launch or cleanup.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::app_install::{
        PreparedRecord,
        manifest::{BundleEntry, BundleManifest},
        prepare_new,
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
        io::Error::new(io::ErrorKind::InvalidData, "app prepare fixture rejected")
    }
    fn open(p: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(p)
    }
    // Observation must share with retained original writable preparation caps.
    // The original source pin independently denies writers and replacement.
    fn open_observer(p: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(p)
    }
    fn hash(p: &Path, size: u64, end: Instant) -> io::Result<String> {
        let mut f = open_observer(p)?;
        let id = dlpad_engine::file_identity::from_file(&f)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() != size || size > 128 * 1024 * 1024 {
            return Err(bad());
        }
        let mut buf = [0u8; 65536];
        let mut digest = Sha256::new();
        let mut n = 0u64;
        loop {
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let count = f.read(&mut buf)?;
            if count == 0 {
                break;
            }
            n = n.checked_add(count as u64).ok_or_else(bad)?;
            if n > size {
                return Err(bad());
            }
            digest.update(&buf[..count]);
        }
        if n != size
            || f.metadata()?.len() != size
            || dlpad_engine::file_identity::from_file(&f)? != id
        {
            return Err(bad());
        }
        Ok(format!("{:x}", digest.finalize()))
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 || args[1] != "--fixed-app-prepare" {
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
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(bad());
    }
    let end = Instant::now() + Duration::from_secs(60);
    let mut input = open(&source)?;
    let size = input.metadata()?.len();
    let original_id = dlpad_engine::file_identity::from_file(&input)?;
    if hash(&source, size, end)? != sha {
        return Err(bad());
    }
    input.seek(SeekFrom::Start(17))?;
    let root = tempfile::Builder::new()
        .prefix("dlpad-app-prepare-owned-")
        .tempdir()?
        .keep();
    println!(
        "{}",
        serde_json::json!({"schema":1,"phase":"owned-fixture-created","root":root,"synthetic_only":true})
    );
    let notice_bytes = b"synthetic preparation notice; no legal coverage claim\n";
    let sentinel = b"synthetic preservation sentinel";
    let np = root.join("notice.txt");
    let sp = root.join("foreign-sentinel");
    for (p, b) in [(&np, notice_bytes.as_slice()), (&sp, sentinel.as_slice())] {
        let mut f = File::create_new(p)?;
        f.write_all(b)?;
        f.sync_all()?;
    }
    let notice = open(&np)?;
    let files = [("dlpad.exe", &input), ("notice.txt", &notice)];
    let manifest = BundleManifest {
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
    let mut wrong = manifest.clone();
    wrong.entries[0].sha256 = if sha == "a".repeat(64) {
        "b".repeat(64)
    } else {
        "a".repeat(64)
    };
    let mut negatives = Vec::new();
    for (name, m, cancel, expired, kind) in [
        (
            "wrong-sha",
            &wrong,
            false,
            false,
            io::ErrorKind::InvalidData,
        ),
        (
            "cancel-before-create",
            &manifest,
            true,
            false,
            io::ErrorKind::Interrupted,
        ),
        (
            "deadline-before-create",
            &manifest,
            false,
            true,
            io::ErrorKind::TimedOut,
        ),
    ] {
        let target = root.join(name);
        let fail = prepare_new(
            m,
            &files,
            &target,
            &AtomicBool::new(cancel),
            if expired { Instant::now() } else { end },
        )
        .err()
        .ok_or_else(bad)?;
        if fail.kind() != kind || fail.mutation_may_have_occurred() || target.try_exists()? {
            return Err(bad());
        }
        negatives.push(serde_json::json!({"case":name,"kind":format!("{:?}",fail.kind()),"mutation_may_have_occurred":false,"target_absent":true}));
    }
    let existing = root.join("existing-target");
    std::fs::create_dir(&existing)?;
    let foreign = existing.join("foreign-sentinel");
    std::fs::write(&foreign, b"preserve-existing")?;
    let fail = prepare_new(&manifest, &files, &existing, &AtomicBool::new(false), end)
        .err()
        .ok_or_else(bad)?;
    if std::fs::read(&foreign)? != b"preserve-existing"
        || std::fs::read_dir(&existing)?.take(2).count() != 1
    {
        return Err(bad());
    }
    negatives.push(serde_json::json!({"case":"existing-target","kind":format!("{:?}",fail.kind()),"foreign_preserved":true,"mutation_may_have_occurred":fail.mutation_may_have_occurred()}));
    let target = root.join("prepared-app");
    let prepared = prepare_new(&manifest, &files, &target, &AtomicBool::new(false), end)
        .map_err(|f| io::Error::new(f.kind(), "app prepare fixture failed"))?;
    prepared.verify_current()?;
    let record = prepared.record();
    let copy = prepared.path().join("dlpad.exe");
    let copy_sha = hash(&copy, size, end)?;
    let cf = open_observer(&copy)?;
    let copy_id = dlpad_engine::file_identity::from_file(&cf)?;
    let marker_path = prepared.path().join("install-ready.json");
    let mut marker = Vec::new();
    open_observer(&marker_path)?
        .take(65537)
        .read_to_end(&mut marker)?;
    if marker.len() > 65536 {
        return Err(bad());
    }
    let decoded: PreparedRecord = serde_json::from_slice(&marker).map_err(|_| bad())?;
    let same_record = serde_json::to_value(&decoded)? == serde_json::to_value(record)?;
    let rf = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(prepared.path())?;
    let root_id = dlpad_engine::file_identity::from_file(&rf)?;
    let source_after = hash(&source, size, end)?;
    if copy_sha != sha
        || source_after != sha
        || copy_id == original_id
        || root_id != record.root_identity
        || !same_record
        || input.stream_position()? != 17
        || dlpad_engine::file_identity::from_file(&input)? != original_id
        || record.trusted_provenance
        || record.entries.len() != 2
        || !record.entries.iter().any(|e| {
            e.name == "dlpad.exe" && e.identity == copy_id && e.bytes == size && e.sha256 == sha
        })
        || std::fs::read(&sp)? != sentinel
        || std::fs::read(&np)? != notice_bytes
        || {
            let mut b = Vec::new();
            open_observer(&prepared.path().join("notice.txt"))?
                .take(65537)
                .read_to_end(&mut b)?;
            b
        } != notice_bytes
    {
        return Err(bad());
    }
    prepared.verify_current()?;
    let result = serde_json::json!({"schema":1,"status":"PASS_PREPARE_ONLY","owned_fixture_root":root,"source_path":source,"source_bytes":size,"source_sha256_before":sha,"source_sha256_after":source_after,"source_identity":original_id,"copy_path":copy,"copy_bytes":size,"copy_sha256":copy_sha,"copy_identity":copy_id,"cursor_before":17,"cursor_after":17,"prepared_path":prepared.path(),"root_identity":root_id,"ready_record":record,"marker_bytes":marker.len(),"marker_equals_record":same_record,"negative_cases":negatives,"sentinel_preserved":true,"source_notice_preserved":true,"copied_notice_preserved":true,"trusted_provenance":false,"actual_launch":false,"installer_lifecycle_verified":false});
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
        eprintln!("app prepare fixture failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows app prepare fixture unavailable");
        std::process::exit(2);
    }
}
