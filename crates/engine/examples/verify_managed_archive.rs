//! Read-only package experiment. Never installs or executes extracted bytes.
use dlpad_engine::tools::{ArchivePackage, MAX_ARCHIVE_BYTES, extract_verified_zip};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
const MANIFEST_LIMIT: u64 = 64 * 1024;
fn hash(file: &mut File, bound: u64) -> Result<(u64, String), ()> {
    file.seek(SeekFrom::Start(0)).map_err(|_| ())?;
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|_| ())?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).ok_or(())?;
        if count > bound {
            return Err(());
        }
        digest.update(&buffer[..n]);
    }
    Ok((count, format!("{:x}", digest.finalize())))
}
#[cfg(target_os = "linux")]
fn peak_rss_kib() -> Option<u64> {
    let file = File::open("/proc/self/status").ok()?;
    let mut text = String::new();
    file.take(65537).read_to_string(&mut text).ok()?;
    if text.len() > 65536 {
        return None;
    }
    let row = text.lines().find(|line| line.starts_with("VmHWM:"))?;
    let mut parts = row.split_whitespace();
    if parts.next()? != "VmHWM:" {
        return None;
    }
    let value = parts.next()?.parse().ok()?;
    if parts.next()? != "kB" || parts.next().is_some() {
        return None;
    }
    Some(value)
}
#[cfg(not(target_os = "linux"))]
fn peak_rss_kib() -> Option<u64> {
    None
}
fn run() -> Result<(), ()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(());
    }
    let archive = PathBuf::from(&args[0]);
    let manifest = PathBuf::from(&args[1]);
    let component = args[2].to_str().ok_or(())?;
    if !archive.is_absolute() || !manifest.is_absolute() {
        return Err(());
    }
    let input = File::open(manifest).map_err(|_| ())?;
    if input.metadata().map_err(|_| ())?.len() > MANIFEST_LIMIT {
        return Err(());
    }
    let mut bytes = Vec::new();
    input
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err(());
    }
    let package: ArchivePackage = serde_json::from_slice(&bytes).map_err(|_| ())?;
    package.validate().map_err(|_| ())?;
    let expected = package
        .components
        .iter()
        .find(|entry| entry.name == component)
        .ok_or(())?;
    let mut source = File::open(archive).map_err(|_| ())?;
    let before = hash(&mut source, MAX_ARCHIVE_BYTES)?;
    if before.0 != package.archive_bytes || !before.1.eq_ignore_ascii_case(&package.archive_sha256)
    {
        return Err(());
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ())?;
    let mut output = tempfile::tempfile().map_err(|_| ())?;
    let start = Instant::now();
    let cancel = AtomicBool::new(false);
    let extracted = extract_verified_zip(
        &source,
        &package,
        component,
        &mut output,
        &cancel,
        start + Duration::from_secs(120),
    )
    .map_err(|_| ())?;
    output.sync_all().map_err(|_| ())?;
    let actual = hash(&mut output, expected.bytes)?;
    let after = hash(&mut source, MAX_ARCHIVE_BYTES)?;
    if extracted != expected.bytes
        || actual.0 != extracted
        || !actual.1.eq_ignore_ascii_case(&expected.sha256)
        || before != after
    {
        return Err(());
    }
    let row = serde_json::json!({"status":"PASS","component":component,"bytes":extracted,"elapsed_ms":start.elapsed().as_millis(),"output_sha256":actual.1,"source_before_sha256":before.1,"source_after_sha256":after.1,"peak_rss_kib":peak_rss_kib(),"installed":false,"executed":false});
    let encoded = serde_json::to_string(&row).map_err(|_| ())?;
    if encoded.len() > 4096 {
        return Err(());
    }
    println!("{encoded}");
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("{{\"status\":\"FAIL\",\"error\":\"archive fixture verification failed\"}}");
        std::process::exit(2);
    }
}
