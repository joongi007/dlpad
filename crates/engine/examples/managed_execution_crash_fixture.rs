//! Synthetic Linux managed pending crash acceptance. No tool reader is spawned.
use dlpad_engine::tools::{InstallManifest, Ownership, ToolError, ToolManager};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::PathBuf,
};
fn manifest() -> InstallManifest {
    InstallManifest {
        tool: "fixture".into(),
        version: "1".into(),
        target: "synthetic".into(),
        source_url: "https://example.invalid/fixture".into(),
        sha256: format!("{:x}", Sha256::digest(b"synthetic executable bytes")),
        license: "fixture".into(),
        ownership: Ownership::Managed,
        executable: "tool".into(),
    }
}
fn run() -> Result<(), ()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(());
    }
    let mode = args[0].to_str().ok_or(())?;
    let root = PathBuf::from(&args[1]);
    if !root.is_absolute() || !root.is_dir() {
        return Err(());
    }
    let mut manager = ToolManager::new(&root.join("managed")).map_err(|_| ())?;
    match mode {
        "setup" => {
            let source = root.join("source");
            fs::write(&source, b"synthetic executable bytes").map_err(|_| ())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).map_err(|_| ())?;
            }
            manager.import(&source, manifest()).map_err(|_| ())?;
            println!("{{\"status\":\"setup\"}}");
        }
        "hold" => {
            let execution = manager.begin_execution().map_err(|_| ())?;
            println!("{{\"status\":\"ready\"}}");
            io::stdout().flush().map_err(|_| ())?;
            let mut byte = [0];
            io::stdin().read_exact(&mut byte).map_err(|_| ())?;
            if byte != [b'F'] {
                return Err(());
            }
            execution.finish().map_err(|_| ())?;
            println!("{{\"status\":\"finished\"}}");
        }
        "probe-pending" => {
            if !matches!(
                manager.select_existing(&manifest()),
                Err(ToolError::ExecutionPending)
            ) {
                return Err(());
            }
            println!("{{\"status\":\"pending\"}}");
        }
        "probe-clear" => {
            manager.select_existing(&manifest()).map_err(|_| ())?;
            println!("{{\"status\":\"clear\"}}");
        }
        _ => return Err(()),
    }
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("managed fixture failed");
        std::process::exit(2);
    }
}
