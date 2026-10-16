//! Windows-only public Store acceptance draft; root builds/runs after scanner integration.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlpad_engine::{
        file_identity::{self, NativeFileIdentity},
        store::Store,
    };
    use std::{
        fs,
        io::{self, Write},
        sync::atomic::AtomicBool,
        time::{Instant, UNIX_EPOCH},
    };
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--fixed-all-identities-experiment"))
        || args.next().is_some()
    {
        return Err("fixed owned experiment flag required".into());
    }
    let budget = Instant::now();
    let check = || -> io::Result<()> {
        if budget.elapsed().as_secs() >= 120 {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "identity acceptance deadline",
            ))
        } else {
            Ok(())
        }
    };
    let root = tempfile::Builder::new()
        .prefix("dlpad-library-all-identities-owned-")
        .tempdir()?
        .keep();
    println!(
        "{}",
        serde_json::json!({"schema":1,"phase":"created","owned_root":root,"synthetic_only":true})
    );
    io::stdout().flush()?;
    let media = root.join("media");
    fs::create_dir(&media)?;
    fs::write(root.join("foreign-sentinel"), b"preserve")?;
    for index in 0..10000 {
        check()?;
        fs::write(media.join(format!("한글-{index:05}.mp4")), b"abc")?;
    }
    fs::write(media.join("ignored.part"), b"pending")?;
    let db = root.join("library.sqlite");
    let mut store = Store::open(&db)?;
    assert_eq!(store.reconcile_library(&media)?, 10000);
    drop(store);
    let mut store = Store::open(&db)?;
    let mut cursor = None;
    let mut count = 0;
    let mut seen = [0u8; 1250];
    loop {
        check()?;
        let page = store.library_page(&media, cursor.as_deref(), 128)?;
        if page.is_empty() {
            break;
        }
        assert!(page.len() <= 128);
        for row in &page {
            check()?;
            let name = row
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("unsupported fixture name")?;
            let index: usize = name
                .strip_prefix("한글-")
                .and_then(|s| s.strip_suffix(".mp4"))
                .ok_or("unexpected row")?
                .parse()?;
            assert!(index < 10000);
            let bit = 1u8 << (index % 8);
            assert_eq!(seen[index / 8] & bit, 0);
            seen[index / 8] |= bit;
            let metadata = fs::symlink_metadata(&row.path)?;
            assert!(metadata.is_file() && !metadata.file_type().is_symlink());
            let original = file_identity::open_regular_no_reparse(&row.path)?;
            assert_eq!(
                file_identity::final_path(&original)?,
                fs::canonicalize(&row.path)?
            );
            let NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } = file_identity::from_file(&original)?
            else {
                return Err("native identity unavailable".into());
            };
            let identity = format!(
                "windows:v1:{volume_serial}:{}",
                file_id
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .and_then(|d| i64::try_from(d.as_millis()).ok());
            assert_eq!(row.file_identity.as_deref(), Some(identity.as_str()));
            assert_eq!(row.size, metadata.len());
            assert_eq!(row.size, 3);
            assert_eq!(row.modified_ms, modified);
            assert_eq!(fs::read(&row.path)?, b"abc");
            count += 1;
        }
        cursor = page.last().map(|r| r.path.clone());
    }
    assert_eq!(count, 10000);
    assert!(seen.iter().all(|b| *b == 255));
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        store.reconcile_library_with_cancel(&media, &cancel),
        Err(dlpad_engine::library::LibraryError::Cancelled)
    ));
    drop(store);
    let readonly =
        rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let rows: i64 = readonly.query_row(
        "SELECT count(*) FROM library_entries WHERE present=1",
        [],
        |r| r.get(0),
    )?;
    assert_eq!(rows, 10000);
    let integrity: String = readonly.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    assert_eq!(integrity, "ok");
    assert_eq!(fs::read(root.join("foreign-sentinel"))?, b"preserve");
    println!(
        "{}",
        serde_json::json!({"schema":1,"phase":"all-identities-verified","rows":count,"public_pages_max":128,"all_native_128bit_ids_compared":true,"size_mtime_compared":true,"partial_skipped":true,"pre_cancel_preserved":true,"readonly_rows":rows,"sentinel_preserved":true,"owned_root":root,"production_adoption":false})
    );
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows identity acceptance unavailable");
    std::process::exit(2);
}
