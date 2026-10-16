//! Explicit synthetic active selection; no execution, discovery or publisher trust.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            create_registration_receipt, inspect_active_selection, inspect_registered_with_receipt,
            manifest::{BundleEntry, BundleManifest},
            prepare_new, register_fresh, select_registered_pair,
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
    if args.len() == 6 && args[1] == "--inspect-existing-active" {
        let active = PathBuf::from(&args[2]);
        let registered = PathBuf::from(&args[3]);
        let receipt = PathBuf::from(&args[4]);
        let expected = args[5].to_str().ok_or_else(bad)?;
        if [active.as_path(), registered.as_path(), receipt.as_path()]
            .iter()
            .any(|p| !p.is_absolute() || p.components().any(|c| matches!(c, Component::ParentDir)))
            || expected.len() != 64
            || !expected
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(bad());
        }
        let end = Instant::now() + Duration::from_secs(90);
        let observed =
            inspect_active_selection(&active, &registered, &receipt, &AtomicBool::new(false), end)?;
        observed.verify_current(&AtomicBool::new(false), end)?;
        let canonical = observed.record().canonical_bytes()?;
        let actual = format!("{:x}", Sha256::digest(&canonical));
        if actual != expected {
            return Err(bad());
        }
        let record = observed.record();
        let output = serde_json::json!({"schema":1,"phase":"existing-active-readonly-verified","canonical_sha256":actual,"canonical_bytes":canonical.len(),"active_root_identity":record.active_root_identity,"marker_identity":record.marker_identity,"selected_executable":record.selected_executable,"registration_root_identity":record.registration_root_identity,"receipt_root_identity":record.receipt_root_identity,"receipt_identity":record.receipt_identity,"actual_launch":false,"mutation":false,"discovery":false,"deletion_authority":false,"publisher_trust":false});
        let raw = serde_json::to_string(&output).map_err(|_| bad())?;
        if raw.len() > 4096 {
            return Err(bad());
        }
        println!("{raw}");
        return Ok(());
    }
    if args.len() != 4 || args[1] != "--fixed-app-active-selection" {
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
        .prefix("dlpad-app-active-selection-owned-")
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

    (&input).seek(SeekFrom::Start(17))?;
    let mut original_pins = Vec::new();
    for name in ["normal", "other"] {
        let reg = root.join(format!("registered-{name}"));
        let receipt = root.join(format!("receipt-{name}"));
        let prepared = prepare_new(&m, &files, &reg, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "active prepare failed"))?;
        let registered = register_fresh(prepared, &AtomicBool::new(false), end)
            .map_err(|e| io::Error::new(e.kind(), "active register failed"))?;
        let proof =
            create_registration_receipt(&registered, &receipt, &AtomicBool::new(false), end)
                .map_err(|e| io::Error::new(e.kind(), "active receipt failed"))?;
        drop(proof);
        drop(registered);
        for leaf in [
            "dlpad.exe",
            "notice.txt",
            "install-ready.json",
            "install-registration.json",
            "install-registration-seal.json",
        ] {
            let path = reg.join(leaf);
            original_pins.push((path.clone(), hash(&path, end)?));
        }
        let path = receipt.join("registration-receipt.json");
        original_pins.push((path.clone(), hash(&path, end)?));
    }
    let reg = root.join("registered-normal");
    let receipt = root.join("receipt-normal");
    let active_path = root.join("active-normal");
    let mut cases = Vec::new();
    for (name, cancelled, deadline, expected) in [
        ("cancel", true, end, io::ErrorKind::Interrupted),
        ("deadline", false, Instant::now(), io::ErrorKind::TimedOut),
    ] {
        let target = root.join(format!("active-{name}"));
        let pair = inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), end)?;
        let error =
            match select_registered_pair(pair, &target, &AtomicBool::new(cancelled), deadline) {
                Err(error) => error,
                Ok(_) => return Err(bad()),
            };
        if error.kind() != expected
            || error.mutation_may_have_occurred()
            || error.root_capability_acquired()
            || error.retained_file_cap_count() != 0
            || target.exists()
        {
            return Err(bad());
        }
        cases.push(serde_json::json!({"case":name,"kind":format!("{:?}",error.kind()),"target_absent":true,"source_preserved":true}));
    }
    let pair = inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), end)?;
    let active = select_registered_pair(pair, &active_path, &AtomicBool::new(false), end)
        .map_err(|e| io::Error::new(e.kind(), "active selection failed"))?;
    active.verify_current(&AtomicBool::new(false), end)?;
    let record = serde_json::to_value(active.record()).map_err(|_| bad())?;
    let record_bytes = serde_json::to_vec(active.record()).map_err(|_| bad())?;
    let entries: Vec<_> = std::fs::read_dir(&active_path)?.collect::<io::Result<_>>()?;
    if entries.len() != 1 {
        return Err(bad());
    }
    let active_file = entries[0].path();
    let original_active = observed_bytes(&active_file, 65536)?;
    if original_active != record_bytes {
        return Err(bad());
    }
    drop(active);
    let reopened =
        inspect_active_selection(&active_path, &reg, &receipt, &AtomicBool::new(false), end)?;
    reopened.verify_current(&AtomicBool::new(false), end)?;
    if serde_json::to_value(reopened.record()).map_err(|_| bad())? != record {
        return Err(bad());
    }
    cases.push(serde_json::json!({"case":"normal","canonical_record":record,"active_record_path":active_file,"active_sha256":format!("{:x}",Sha256::digest(&original_active)),"restart_readonly_verified":true,"registered_path":reg,"receipt_path":receipt}));
    drop(reopened);
    let pair = inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), end)?;
    let error = match select_registered_pair(pair, &active_path, &AtomicBool::new(false), end) {
        Err(error) => error,
        Ok(_) => return Err(bad()),
    };
    if error.kind() != io::ErrorKind::AlreadyExists
        || observed_bytes(&active_file, 65536)? != original_active
    {
        return Err(bad());
    }
    cases.push(serde_json::json!({"case":"existing","kind":"AlreadyExists","original_active_preserved":true}));
    let other_reg = root.join("registered-other");
    let other_receipt = root.join("receipt-other");
    inspect_registered_with_receipt(&other_reg, &other_receipt, &AtomicBool::new(false), end)?;
    let mismatch = match inspect_active_selection(
        &active_path,
        &other_reg,
        &other_receipt,
        &AtomicBool::new(false),
        end,
    ) {
        Err(error) => error,
        Ok(_) => return Err(bad()),
    };
    if mismatch.kind() != io::ErrorKind::InvalidData {
        return Err(bad());
    }
    cases.push(serde_json::json!({"case":"valid-other-pair","kind":"InvalidData","both_pairs_preserved":true}));
    let foreign_active_path = root.join("active-foreign");
    let pair = inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), end)?;
    let selected = select_registered_pair(pair, &foreign_active_path, &AtomicBool::new(false), end)
        .map_err(|e| io::Error::new(e.kind(), "foreign active selection failed"))?;
    let foreign_record = selected.record().canonical_bytes()?;
    drop(selected);
    let observed = inspect_active_selection(
        &foreign_active_path,
        &reg,
        &receipt,
        &AtomicBool::new(false),
        end,
    )?;
    let foreign_path = foreign_active_path.join("fixture-foreign");
    let mut foreign = File::create_new(&foreign_path)?;
    foreign.write_all(sentinel)?;
    foreign.sync_all()?;
    drop(foreign);
    let e = observed
        .verify_current(&AtomicBool::new(false), end)
        .err()
        .ok_or_else(bad)?;
    if e.kind() != io::ErrorKind::InvalidData
        || observed_bytes(&foreign_path, 64)? != sentinel
        || observed_bytes(&foreign_active_path.join("active-selection.json"), 65536)?
            != foreign_record
        || observed_bytes(&active_file, 65536)? != original_active
    {
        return Err(bad());
    }
    cases.push(serde_json::json!({"case":"foreign","kind":"InvalidData","foreign_active_path":foreign_active_path,"foreign_preserved":true,"original_active_preserved":true}));
    drop(observed);
    let pristine =
        inspect_active_selection(&active_path, &reg, &receipt, &AtomicBool::new(false), end)?;
    pristine.verify_current(&AtomicBool::new(false), end)?;
    drop(pristine);

    for (path, pin) in &original_pins {
        if hash(path, end)? != *pin {
            return Err(bad());
        }
    }
    if hash(&source, end)? != sha
        || file_identity::from_file(&input)? != source_id
        || input.metadata()?.len() != size
        || (&input).stream_position()? != 17
        || observed_bytes(&notice_path, 128)? != notice_bytes
        || observed_bytes(&sentinel_path, 64)? != sentinel
    {
        return Err(bad());
    }
    println!(
        "{}",
        serde_json::json!({"schema":1,"status":"PASS_EXPLICIT_ACTIVE_SELECTION_ONLY","owned_fixture_root":root,"cases":cases,"source_path":source,"source_sha256_before":sha,"source_sha256_after":sha,"source_bytes":size,"source_identity":source_id,"source_cursor_before":17,"source_cursor_after":17,"registered_receipt_original_pins":original_pins,"source_notice_preserved":true,"outside_sentinel_preserved":true,"normal_active_pristine":true,"actual_launch":false,"automatic_discovery":false,"upgrade_verified":false,"deletion_authority":false,"historical_ownership_verified":false,"publisher_trust":false,"installer_complete":false})
    );
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(error) = run() {
        eprintln!("active fixture rejected: {:?}", error.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(1);
    }
}
