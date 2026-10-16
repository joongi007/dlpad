//! Fixed, synthetic public-API cost experiment; never opens user media.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlpad_engine::store::Store;
    use rusqlite::{Connection, OpenFlags};
    use std::{fs, time::Instant};
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentProcess, GetProcessTimes},
    };

    fn cpu() -> Result<(u64, u64), Box<dyn std::error::Error>> {
        fn ticks(time: FILETIME) -> u64 {
            (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
        }
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        // SAFETY: current-process pseudo handle and correctly typed live outputs.
        if unsafe {
            GetProcessTimes(
                GetCurrentProcess(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok((
            ticks(created),
            ticks(kernel)
                .checked_add(ticks(user))
                .ok_or("CPU tick overflow")?,
        ))
    }

    let mut args = std::env::args_os();
    args.next();
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--fixed-cost-experiment"))
        || args.next().is_some()
    {
        return Err("fixed synthetic experiment flag required".into());
    }
    let root = tempfile::Builder::new()
        .prefix("dlpad-reconcile-cost-owned-")
        .tempdir()?
        .keep();
    println!(
        "{}",
        serde_json::json!({"owned_fixture_root": root, "synthetic_only": true})
    );
    let media = root.join("media");
    fs::create_dir(&media)?;
    fs::write(root.join("sentinel"), b"dlpad-reconcile-cost-v1")?;
    for index in 0..10_000 {
        fs::write(media.join(format!("fixture-{index:05}.mp4")), b"abc")?;
    }
    let database = root.join("library.sqlite");
    let mut store = Store::open(&database)?;
    let mut rounds = Vec::new();
    let mut original_page = None;
    for round in 1..=3 {
        let (creation, before) = cpu()?;
        let clock = Instant::now();
        let count = store.reconcile_library(&media)?;
        let elapsed = clock.elapsed().as_secs_f64();
        let (after_creation, after) = cpu()?;
        if count != 10_000 || creation != after_creation || after < before {
            return Err("original process identity/count/ticks mismatch".into());
        }
        let page = store.library_page(&media, None, 128)?;
        if page.len() != 128
            || page.iter().any(|entry| {
                entry.size != 3
                    || !entry
                        .file_identity
                        .as_deref()
                        .is_some_and(|id| id.starts_with("windows:v1:"))
            })
        {
            return Err("native identity page mismatch".into());
        }
        if let Some(original) = &original_page {
            if &page != original {
                return Err("unchanged fixture page drift".into());
            }
        } else {
            original_page = Some(page);
        }
        rounds.push(serde_json::json!({
            "round": round, "files": count, "wall_seconds": elapsed,
            "original_process_creation": creation,
            "process_cpu_begin_100ns": before, "process_cpu_end_100ns": after,
            "process_cpu_seconds": (after - before) as f64 / 10_000_000.0,
            "page_native_identity_preserved": true
        }));
    }
    drop(store);
    let reopened = Store::open(&database)?;
    if reopened.library_page(&media, None, 128)?.as_slice()
        != original_page
            .as_ref()
            .ok_or("missing original page")?
            .as_slice()
    {
        return Err("reopened page mismatch".into());
    }
    drop(reopened);
    let readonly = Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let present: i64 = readonly.query_row(
        "SELECT count(*) FROM library_entries WHERE present=1",
        [],
        |row| row.get(0),
    )?;
    let integrity: String = readonly.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if present != 10_000 || integrity != "ok" {
        return Err("reopened database integrity/count mismatch".into());
    }
    println!(
        "{}",
        serde_json::json!({
            "status": "PASS", "owned_fixture_root": root, "rounds": rounds,
            "database_reopen_verified": true, "present_rows": present,
            "scope": "public reconcile API; synthetic 10k; not GUI idle acceptance",
            "quiet_host_verified": false
        })
    );
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows-only fixed cost experiment");
    std::process::exit(2);
}
