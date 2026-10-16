//! Synthetic native downloader only; error112 is SOFTWARE INJECTION, not disk exhaustion.
#[cfg(windows)]
mod native {
    use std::{
        fs::{self, OpenOptions},
        io::{Read, Write},
        os::windows::fs::MetadataExt,
        path::{Path, PathBuf},
    };
    const CAP: &[u8] = b"dlpad-windows-writer-injected112-fixture-v1";
    fn plain(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for node in path.ancestors() {
            if node.exists() && fs::symlink_metadata(node)?.file_attributes() & 0x400 != 0 {
                return Err("fixture reparse path refused".into());
            }
        }
        Ok(())
    }
    fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        plain(path)?;
        let mut file = fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > limit as u64 {
            return Err("fixture read bound".into());
        }
        let mut bytes = Vec::new();
        (&mut file).take(limit as u64 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err("fixture grew".into());
        }
        Ok(bytes)
    }
    pub(super) fn run() -> Result<i32, Box<dyn std::error::Error>> {
        let exe = std::env::current_exe()?.canonicalize()?;
        plain(&exe)?;
        let root = exe.parent().ok_or("fixture root")?;
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("workspace parent")?
            .parent()
            .ok_or("workspace")?;
        let temp = project.join(".local-tools/windows/tmp").canonicalize()?;
        if root.parent() != Some(temp.as_path())
            || !root
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("dlpad-writer-failure-owned-"))
            || read(&root.join("fixture.cap"), 64)? != CAP
        {
            return Err("owned project temp CAP required".into());
        }
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args == ["--ignore-config", "--no-plugin-dirs", "--version"] {
            println!("2026.10.11");
            return Ok(0);
        }
        if args.last().and_then(|s| s.to_str())
            != Some("https://fixture.invalid/writer-injected112")
        {
            return Err("only synthetic source accepted".into());
        }
        if args.iter().any(|s| s == "--dump-single-json") {
            let path = root.join("extractions.txt");
            plain(&path)?;
            if path.exists() && fs::metadata(&path)?.len() > 512 {
                return Err("extraction history bound".into());
            }
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            file.write_all(b"https://fixture.invalid/writer-injected112\n")?;
            file.sync_all()?;
            println!("{{\"id\":\"writer-fixture\",\"duration\":1}}");
            return Ok(0);
        }
        let value = |flag: &str| -> Result<&std::ffi::OsString, Box<dyn std::error::Error>> {
            let index = args
                .iter()
                .position(|s| s == flag)
                .ok_or("fixture flag missing")?;
            args.get(index + 1)
                .ok_or_else(|| "fixture value missing".into())
        };
        let output = PathBuf::from(value("--paths")?).canonicalize()?;
        plain(&output)?;
        let expected = root
            .join("media 한글/.dlpad-partials/writer-native-job")
            .canonicalize()?;
        if output != expected || value("--output")? != "writer-native-job.%(ext)s" {
            return Err("exact owned output required".into());
        }
        let partial = output.join("writer-native-job.mp4.part");
        match read(&root.join("mode"), 32)?.as_slice() {
            b"injected112" => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&partial)?;
                file.write_all(&[b'p'; 65536])?;
                file.sync_all()?;
                eprintln!(
                    "DLPAD_WRITER_ERROR:{{\"win32_error\":112,\"injected\":true,\"partial_bytes\":65536}}"
                );
                Ok(7)
            }
            b"success" => {
                if read(&partial, 65536)? != vec![b'p'; 65536] {
                    return Err("original partial bytes changed".into());
                }
                let path = output.join("writer-native-job.mp4");
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                file.write_all(b"synthetic recovered writer bytes; not a real video")?;
                file.sync_all()?;
                println!("DLPAD_ARTIFACT:{}", serde_json::to_string(&path)?);
                Ok(0)
            }
            _ => Err("unknown fixture mode".into()),
        }
    }
}
#[cfg(windows)]
fn main() {
    match native::run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("synthetic writer fixture refused: {error}");
            std::process::exit(9);
        }
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows-only synthetic writer fixture; no execution");
    std::process::exit(2);
}
