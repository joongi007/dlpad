//! Developer experiment: owned temporary install; optional verified activation.
use dlpad_engine::tools::{ArchivePackage, MAX_ARCHIVE_BYTES, ToolManager};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buf = [0u8; 65536];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        bytes = bytes.checked_add(n as u64).ok_or("source size overflow")?;
        if bytes > MAX_ARCHIVE_BYTES {
            return Err("source bound exceeded".into());
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 && args.len() != 3 {
        return Err("manifest local source | manifest download".into());
    }
    let manifest = Path::new(&args[0]);
    if !manifest.is_absolute() {
        return Err("absolute manifest required".into());
    }
    let mut data = Vec::new();
    File::open(manifest)?.take(65537).read_to_end(&mut data)?;
    if data.len() > 65536 {
        return Err("manifest bound exceeded".into());
    }
    let package: ArchivePackage = serde_json::from_slice(&data)?;
    package.validate()?;
    let mode = args[1].to_str().ok_or("invalid mode")?;
    let source = if matches!(mode, "local" | "local-verified") && args.len() == 3 {
        Some(Path::new(&args[2]))
    } else if mode == "download" && args.len() == 2 {
        None
    } else {
        return Err("invalid mode arguments".into());
    };
    if source.is_some_and(|p| !p.is_absolute()) {
        return Err("absolute source required".into());
    }
    let source_before = source.map(hash).transpose()?;
    let temporary = tempfile::tempdir()?;
    let mut manager = ToolManager::new(&temporary.path().join("managed"))?;
    let cancel = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let installed = match source {
        Some(path) => manager.install_verified_archive(path, package.clone(), &cancel)?,
        None => manager.download_archive_and_install(package.clone(), &cancel)?,
    };
    if manager.active(&package.tool)?.is_some() {
        return Err("unexpected activation".into());
    }
    let verified = manager
        .package_installed(&installed.main.manifest)?
        .ok_or("missing package")?;
    if verified.directory != installed.directory {
        return Err("directory mismatch".into());
    }
    let install_ms = start.elapsed().as_millis();
    let remove_start = Instant::now();
    let activated = mode == "local-verified";
    if activated {
        match package.tool.as_str() {
            "deno" => {
                manager.select_verified_deno(&package, &cancel)?;
            }
            "ffmpeg" => {
                manager.select_verified_ffmpeg(&package, &cancel)?;
            }
            _ => return Err("unsupported verified tool".into()),
        }
        if manager.active(&package.tool)?
            != Some((package.tool_version.clone(), package.target.clone()))
        {
            return Err("verified activation missing".into());
        }
        if !matches!(
            manager.remove_package(&package),
            Err(dlpad_engine::tools::ToolError::ActiveVersion)
        ) {
            return Err("active removal was not rejected".into());
        }
    } else {
        manager.remove_package(&package)?;
        if installed.directory.try_exists()? {
            return Err("installed directory remained".into());
        }
    }
    let source_after = source.map(hash).transpose()?;
    if source_before != source_after {
        return Err("source changed".into());
    }
    let row = serde_json::json!({"status":"PASS","mode":mode,"tool":package.tool,"version":package.tool_version,"target":package.target,"archive_bytes":package.archive_bytes,"archive_sha256":package.archive_sha256,"component_bytes":package.components.iter().map(|c|c.bytes).sum::<u64>(),"install_ms":install_ms,"verification_and_removal_ms":remove_start.elapsed().as_millis(),"source_before_sha256":source_before,"source_after_sha256":source_after,"inactive":!activated,"removed":!activated,"executed":activated,"active_removal_rejected":activated});
    let json = serde_json::to_string(&row)?;
    if json.len() > 4096 {
        return Err("output bound exceeded".into());
    }
    println!("{json}");
    Ok(())
}
fn main() {
    match dlpad_engine::process::guardian_entry() {
        Ok(true) => return,
        Ok(false) => (),
        Err(_) => {
            eprintln!("owned guardian entry failed");
            std::process::exit(1);
        }
    }
    if dlpad_engine::process::register_self_guardian().is_err() {
        eprintln!("owned guardian registration failed");
        std::process::exit(1);
    }
    if let Err(error) = run() {
        let message: String = error
            .to_string()
            .chars()
            .filter(|c| !c.is_control())
            .take(160)
            .collect();
        eprintln!("owned archive experiment failed: {message}");
        std::process::exit(1);
    }
}
