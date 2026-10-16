//! Dev-only synthetic yt-dlp stand-in. Never accepts arbitrary credentials.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
const COOKIES: &[u8] = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-download-fixture-only\n";
const SECRET: &str = "windows-download-fixture-only";
const CAP: &[u8] = b"dlpad-windows-session-cookie-synthetic-fixture-v1";
fn bounded(path: &Path) -> Result<Vec<u8>, &'static str> {
    if fs::metadata(path)
        .map_err(|_| "fixture file unavailable")?
        .len()
        > 4096
    {
        return Err("fixture file bound");
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "fixture read failed")?
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| "fixture read failed")?;
    if bytes.len() > 4096 {
        return Err("fixture file bound");
    }
    Ok(bytes)
}
fn plain(path: &Path) -> Result<(), &'static str> {
    for part in path.ancestors() {
        let meta = fs::symlink_metadata(part).map_err(|_| "fixture path unavailable")?;
        if meta.file_type().is_symlink() {
            return Err("fixture symlink rejected");
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("fixture reparse rejected");
            }
        }
    }
    Ok(())
}
fn value(args: &[String], name: &str) -> Result<String, &'static str> {
    let mut positions = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_str() == name);
    let index = positions.next().ok_or("fixture argv missing")?.0;
    if positions.next().is_some() {
        return Err("fixture argv duplicate");
    }
    args.get(index + 1).cloned().ok_or("fixture argv missing")
}
fn run() -> Result<i32, &'static str> {
    let exe = std::env::current_exe().map_err(|_| "fixture executable unavailable")?;
    let root = exe
        .parent()
        .ok_or("fixture root missing")?
        .canonicalize()
        .map_err(|_| "fixture root unavailable")?;
    let temp = std::env::temp_dir()
        .canonicalize()
        .map_err(|_| "fixture temp unavailable")?;
    if root.parent() != Some(temp.as_path())
        || !root
            .file_name()
            .ok_or("fixture name missing")?
            .to_string_lossy()
            .starts_with("dlpad-windows-cookie-download-owned-")
    {
        return Err("explicit owned temporary fixture required");
    }
    plain(&root)?;
    plain(&root.join("fixture.cap"))?;
    plain(&root.join("source.cookies"))?;
    plain(&root.join("mode"))?;
    if bounded(&root.join("fixture.cap"))? != CAP
        || bounded(&root.join("source.cookies"))? != COOKIES
    {
        return Err("fixed synthetic fixture required");
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 128
        || args.iter().any(|arg| arg.len() > 32768)
        || args.iter().map(String::len).sum::<usize>() > 65536
    {
        return Err("fixture argv bound");
    }
    let stage = if args.iter().any(|arg| arg == "--version") {
        "version"
    } else if args.iter().any(|arg| arg == "--dump-single-json") {
        "metadata"
    } else {
        "download"
    };
    let copy = if stage == "version" {
        if args.iter().any(|arg| arg == "--cookies") {
            return Err("version received cookie capability");
        }
        None
    } else {
        let path = PathBuf::from(value(&args, "--cookies")?);
        plain(&path)?;
        let owner = path.parent().ok_or("fixture owner missing")?;
        let cache = owner.parent().ok_or("fixture cache missing")?;
        let owned_name = owner.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
            n.strip_prefix("cookie-session-").is_some_and(|v| {
                v.len() == 64
                    && v.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        });
        if !path.is_absolute()
            || !(cache == root.join("private-cache")
                || cache == root.join("data").join("session-cookies"))
            || path.file_name().is_none_or(|n| n != "cookies.txt")
            || !owned_name
            || bounded(&path)? != COOKIES
        {
            return Err("fixed synthetic private copy required");
        }
        Some(path)
    };
    let calls = root.join("calls.jsonl");
    if calls.exists() {
        plain(&calls)?;
    }
    if calls.exists()
        && fs::metadata(&calls)
            .map_err(|_| "fixture calls unavailable")?
            .len()
            > 16384
    {
        return Err("fixture calls bound");
    }
    let record = serde_json::json!({"stage":stage,"cookie_path":copy.as_ref().map(|path|path.to_string_lossy().into_owned()),"literal_sha256":copy.as_ref().map(|_|format!("{:x}",Sha256::digest(COOKIES)))});
    let mut output = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(calls)
        .map_err(|_| "fixture calls failed")?;
    writeln!(output, "{record}").map_err(|_| "fixture calls failed")?;
    drop(output);
    if stage == "version" {
        println!("2026.10.04");
        return Ok(0);
    }
    let private = copy
        .as_ref()
        .ok_or("fixture copy missing")?
        .to_string_lossy();
    eprintln!("synthetic stderr {SECRET} {private}");
    let mode =
        String::from_utf8(bounded(&root.join("mode"))?).map_err(|_| "fixture mode invalid")?;
    if !["success", "metadata-failure", "download-failure", "cancel"].contains(&mode.as_str()) {
        return Err("fixture mode invalid");
    }
    if stage == "metadata" {
        if mode == "metadata-failure" {
            return Ok(7);
        }
        println!(
            "{}",
            serde_json::json!({"id":"synthetic-windows-auth","duration":10,"title":SECRET,"cookie_echo":private,"formats":[{"url":private}]})
        );
        return Ok(0);
    }
    println!("synthetic stdout {SECRET} {private}");
    std::io::stdout()
        .flush()
        .map_err(|_| "fixture stdout failed")?;
    if mode == "download-failure" {
        return Ok(9);
    }
    let paths = PathBuf::from(value(&args, "--paths")?);
    plain(&paths)?;
    if !paths.is_absolute() || !paths.starts_with(&root) {
        return Err("fixture output outside owned root");
    }
    let name = value(&args, "--output")?.replace("%(ext)s", "mp4");
    if name.len() > 256 || name.contains(['/', '\\', ':']) || name == "." || name == ".." {
        return Err("fixture artifact name invalid");
    }
    let artifact = paths.join(name);
    if mode == "cancel" {
        fs::write(artifact.with_extension("part"), b"synthetic owned partial")
            .map_err(|_| "fixture partial failed")?;
        fs::write(root.join("worker-ready"), b"synthetic owned download ready")
            .map_err(|_| "fixture readiness failed")?;
        std::thread::sleep(Duration::from_secs(30));
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&artifact)
        .map_err(|_| "fixture artifact failed")?
        .write_all(b"synthetic authenticated Windows artifact")
        .map_err(|_| "fixture artifact failed")?;
    println!(
        "DLPAD_ARTIFACT:{}",
        serde_json::to_string(&artifact).map_err(|_| "fixture artifact invalid")?
    );
    Ok(0)
}
fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(64);
        }
    }
}
