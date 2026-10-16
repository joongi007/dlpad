//! Explicit actual payload/root foundation acceptance, not an installer.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::app_install::{
        FreshRoot,
        manifest::{BundleEntry, BundleManifest},
        payload::inspect_payload,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::PathBuf,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ};
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 || args[1] != "--fixed-app-install-foundation" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "explicit flag CPU path SHA required",
        ));
    }
    let source = PathBuf::from(&args[2]);
    let expected = args[3]
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "SHA"))?;
    if !source.is_absolute()
        || source
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        || expected.len() != 64
        || !expected
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source arguments rejected",
        ));
    }
    let mut cpu = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&source)?;
    let cpu_len = cpu.metadata()?.len();
    let original_id = dlpad_engine::file_identity::from_file(&cpu)?;
    let parent = tempfile::Builder::new()
        .prefix("dlpad-app-foundation-owned-")
        .tempdir()?
        .keep();
    println!(
        "{}",
        serde_json::json!({"phase":"owned-fixture-created","root":parent,"synthetic_only":true})
    );
    let notice_path = parent.join("notice.txt");
    let sentinel_path = parent.join("foreign-sentinel");
    let notice = b"synthetic foundation notice; no legal coverage claim\n";
    let sentinel = b"synthetic foreign preservation sentinel";
    for (p, b) in [
        (&notice_path, notice.as_slice()),
        (&sentinel_path, sentinel.as_slice()),
    ] {
        let mut f = File::create_new(p)?;
        f.write_all(b)?;
        f.sync_all()?;
    }
    let notice_file = File::open(&notice_path)?;
    let manifest = BundleManifest {
        schema: 1,
        version: "0.1.0".into(),
        target: "x86_64-pc-windows-gnu".into(),
        development_only: true,
        entries: vec![
            BundleEntry {
                name: "dlpad.exe".into(),
                bytes: cpu_len,
                sha256: expected.into(),
            },
            BundleEntry {
                name: "notice.txt".into(),
                bytes: notice.len() as u64,
                sha256: format!("{:x}", Sha256::digest(notice)),
            },
        ],
    };
    cpu.seek(SeekFrom::Start(17))?;
    let before = cpu.stream_position()?;
    let observations = inspect_payload(
        &manifest,
        &[("dlpad.exe", &cpu), ("notice.txt", &notice_file)],
        &AtomicBool::new(false),
        Instant::now() + Duration::from_secs(30),
    )?;
    let after = cpu.stream_position()?;
    if before != after || observations[0].identity != original_id {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "original capability changed",
        ));
    }
    let path = parent.join("한글 private 설치 root");
    let cap = FreshRoot::create_new(&path)?;
    cap.verify()?;
    if std::fs::read(&sentinel_path)? != sentinel || std::fs::read(&notice_path)? != notice {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fixture preservation failed",
        ));
    }
    let obs:Vec<_>=observations.iter().map(|o|serde_json::json!({"name":o.name,"bytes":o.bytes,"sha256":o.sha256,"identity":o.identity})).collect();
    let result = serde_json::json!({"status":"PASS_FOUNDATION_ONLY","source_sha256":expected,"source_bytes":cpu_len,"original_native_id":original_id,"cursor_before":before,"cursor_after":after,"observations":obs,"owned_fixture_root":parent,"cap_root":cap.path(),"cap_root_id":cap.identity(),"private_root_verified":true,"sentinel_preserved":true,"notice_preserved":true,"installer_verified":false,"source_copy_performed":false,"trusted_provenance":false});
    let text = serde_json::to_string(&result)?;
    if text.len() > 16384 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "receipt bound"));
    }
    println!("{text}");
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("foundation failed kind={:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows native foundation unavailable");
        std::process::exit(2);
    }
}
