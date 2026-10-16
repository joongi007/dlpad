//! Fixed Linux FFmpeg supply experiment. No execution, installation or activation.
use dlpad_engine::tools::{
    ArchiveComponent, ArchiveComponentRole, TarComponentMapping, TarExtractionRequest, TarLimits,
    TarXzSourceSpec, extract_verified_tar_xz,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
const ROOT: &str = "ffmpeg-n9.0.2-17-g2a571b6068-linux64-lgpl-9.0";
const SOURCE_BYTES: u64 = 137863452;
const SOURCE_SHA: &str = "2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a";
fn plain(path: &Path) -> Result<(), ()> {
    if !path.is_absolute() || path.components().any(|p| matches!(p, Component::ParentDir)) {
        return Err(());
    }
    for p in path.ancestors() {
        let m = fs::symlink_metadata(p).map_err(|_| ())?;
        if m.file_type().is_symlink() {
            return Err(());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if m.file_attributes() & 0x400 != 0 {
                return Err(());
            }
        }
    }
    Ok(())
}
fn hash(file: &mut File, bound: u64, deadline: Instant) -> Result<(u64, String), ()> {
    file.seek(SeekFrom::Start(0)).map_err(|_| ())?;
    let mut h = Sha256::new();
    let mut n = 0u64;
    let mut b = [0u8; 65536];
    loop {
        if Instant::now() >= deadline {
            return Err(());
        }
        let read = file.read(&mut b).map_err(|_| ())?;
        if read == 0 {
            break;
        }
        n = n.checked_add(read as u64).ok_or(())?;
        if n > bound {
            return Err(());
        }
        h.update(&b[..read]);
    }
    Ok((n, format!("{:x}", h.finalize())))
}
fn run() -> Result<(), ()> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or(())?);
    if args.next().is_some() {
        return Err(());
    }
    plain(&path)?;
    let mut source = File::open(&path).map_err(|_| ())?;
    if !source.metadata().map_err(|_| ())?.is_file() {
        return Err(());
    }
    let start = Instant::now();
    let deadline = start + Duration::from_secs(180);
    let before = hash(&mut source, SOURCE_BYTES, deadline)?;
    if before != (SOURCE_BYTES, SOURCE_SHA.into()) {
        return Err(());
    }
    let maps: Vec<_> = [
        (
            "bin/ffmpeg",
            "ffmpeg",
            142534568,
            "953632dece404bb1461a4a2285f0191a6105d72c81f9da577eb54326a7fad49a",
            ArchiveComponentRole::Executable,
        ),
        (
            "bin/ffprobe",
            "ffprobe",
            142305128,
            "c0d4ee234730e7b083cc729108ac34f3b718cc4858c3cf8ed6e0dcec545f13c1",
            ArchiveComponentRole::Auxiliary,
        ),
        (
            "LICENSE.txt",
            "LICENSE.txt",
            7651,
            "da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768",
            ArchiveComponentRole::Notice,
        ),
    ]
    .into_iter()
    .map(|(member, name, bytes, sha, role)| TarComponentMapping {
        member: format!("{ROOT}/{member}"),
        component: ArchiveComponent {
            name: name.into(),
            bytes,
            sha256: sha.into(),
            role,
        },
    })
    .collect();
    let limits = TarLimits {
        max_entries: 4096,
        max_total_bytes: 768 * 1024 * 1024,
        max_selected_bytes: 384 * 1024 * 1024,
        max_stream_bytes: 800 * 1024 * 1024,
    };
    let cancel = AtomicBool::new(false);
    let mut rows = Vec::new();
    for map in &maps {
        let mut output = tempfile::tempfile().map_err(|_| ())?;
        let step = Instant::now();
        let bytes = extract_verified_tar_xz(
            &source,
            TarXzSourceSpec {
                bytes: SOURCE_BYTES,
                sha256: SOURCE_SHA,
            },
            TarExtractionRequest {
                root: ROOT,
                mappings: &maps,
                selected: &map.component.name,
                limits: &limits,
            },
            &mut output,
            &cancel,
            deadline,
        )
        .map_err(|_| ())?;
        output.sync_all().map_err(|_| ())?;
        let actual = hash(&mut output, map.component.bytes, deadline)?;
        if bytes != map.component.bytes || actual != (bytes, map.component.sha256.clone()) {
            return Err(());
        }
        rows.push(serde_json::json!({"component":map.component.name,"bytes":bytes,"sha256":actual.1,"elapsed_ms":step.elapsed().as_millis()}));
        drop(output);
    }
    plain(&path)?;
    let after = hash(&mut source, SOURCE_BYTES, deadline)?;
    if after != before {
        return Err(());
    }
    let row = serde_json::json!({"status":"PASS","source_sha256":before.1,"source_after_sha256":after.1,"components":rows,"elapsed_ms":start.elapsed().as_millis(),"executed":false,"installed":false,"supply_adopted":false});
    let text = serde_json::to_string(&row).map_err(|_| ())?;
    if text.len() > 4096 {
        return Err(());
    }
    println!("{text}");
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("{{\"status\":\"FAIL\",\"error\":\"fixed archive verification failed\"}}");
        std::process::exit(2);
    }
}
