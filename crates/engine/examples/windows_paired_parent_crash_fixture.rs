//! Windows synthetic paired authenticated managed yt-dlp public repository vertical fixture.
//! Never contacts the fixture URL or invokes a real media tool.
#[cfg(windows)]
mod native {
    use dlpad_core::{
        jobs::JobState,
        options::{DownloadOptions, Quality, SourceUrl},
    };
    use dlpad_engine::{
        cookies::{import_session, prepare_session_cache},
        download::{DownloadRepository, DownloadSession},
        tools::{InstallManifest, Ownership, ToolManager},
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs as native_fs,
        io::{Read, Write},
        os::windows::fs::MetadataExt,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::{Duration, Instant},
    };
    // Fixture reads are bounded even when a local file changes after metadata inspection.
    mod fs {
        pub use super::native_fs::*;
        pub fn read(path: impl AsRef<std::path::Path>) -> std::io::Result<Vec<u8>> {
            use std::io::Read;
            const MAX: u64 = 8 * 1024 * 1024;
            let file = std::fs::File::open(path)?;
            if file.metadata()?.len() > MAX {
                return Err(std::io::Error::other("fixture read bound"));
            }
            let mut bytes = Vec::new();
            file.take(MAX + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX {
                return Err(std::io::Error::other("fixture read bound"));
            }
            Ok(bytes)
        }
        pub fn read_to_string(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
            let bytes = read(path)?;
            String::from_utf8(bytes).map_err(|_| std::io::Error::other("fixture UTF8 required"))
        }
    }
    const COOKIES: &[u8] = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tpaired-fixture-secret-only\n";
    const SECRET: &str = "paired-fixture-secret-only";
    const CAP: &[u8] = b"dlpad-paired-parent-crash-synthetic-v1";
    const SENTINEL: &[u8] = b"preserve unrelated managed download fixture";
    const ARTIFACT: &[u8] = b"synthetic media fixture bytes; not a real video";
    const TARGET: &str = "x86_64-pc-windows-msvc";
    fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = fs::File::open(path)?;
        if file.metadata()?.len() > 256 * 1024 * 1024 {
            return Err("fixture hash size bound".into());
        }
        let mut sha = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > 256 * 1024 * 1024 {
                return Err("fixture hash size bound".into());
            }
            sha.update(&buffer[..n]);
        }
        Ok(format!("{:x}", sha.finalize()))
    }
    fn root_for_tool() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let exe = std::env::current_exe()?.canonicalize()?;
        let root = exe
            .ancestors()
            .nth(6)
            .ok_or("fixture root missing")?
            .to_owned();
        validate_root(&root)?;
        if root.file_name().and_then(|v| v.to_str()) != Some("owned-fixture")
            || exe
                != root
                    .join("data/managed-tools/yt-dlp/2026.09.30")
                    .join(TARGET)
                    .join("yt-dlp.exe")
            || fs::read(root.join("fixture.cap"))? != CAP
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("synthetic fixture capability rejected".into());
        }
        Ok(root)
    }
    pub(super) fn tool(args: &[std::ffi::OsString]) -> Result<bool, Box<dyn std::error::Error>> {
        if !args
            .iter()
            .any(|v| v == "--version" || v == "--dump-single-json" || v == "--output")
        {
            return Ok(false);
        }
        let root = root_for_tool()?;
        let mode = fs::read_to_string(root.join("mode"))?;
        if !["success", "metadata-failure", "download-failure", "cancel"].contains(&mode.as_str()) {
            return Err("fixture mode invalid".into());
        }
        let stage = if args.iter().any(|v| v == "--version") {
            "version"
        } else if args.iter().any(|v| v == "--dump-single-json") {
            "metadata"
        } else {
            "download"
        };
        let mut marker = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(format!("stage-{stage}")))?;
        marker.write_all(stage.as_bytes())?;
        marker.sync_all()?;
        if stage == "version" {
            if args != ["--ignore-config", "--no-plugin-dirs", "--version"] {
                return Err("fixture version argv mismatch".into());
            }
            if args.iter().any(|v| v == "--cookies") {
                return Err("version received cookies".into());
            }
            println!("2026.09.30");
            return Ok(true);
        }
        let positions: Vec<_> = args
            .iter()
            .enumerate()
            .filter(|(_, v)| *v == "--cookies")
            .map(|(i, _)| i)
            .collect();
        if positions.len() != 1 {
            return Err("exact cookie argument required".into());
        }
        let private = PathBuf::from(
            args.get(positions[0] + 1)
                .ok_or("cookie argument missing")?,
        )
        .canonicalize()?;
        let cache = root.join("data/session-cookies").canonicalize()?;
        if !private.starts_with(&cache) || private == cache || fs::read(&private)? != COOKIES {
            return Err("synthetic private cookie mismatch".into());
        }
        let literal = serde_json::to_vec(&private)?;
        fs::write(root.join(format!("cookie-path-{stage}.json")), &literal)?;
        if stage == "metadata" && mode == "metadata-failure" {
            println!("{SECRET}");
            eprintln!("{SECRET}");
            std::process::exit(41);
        }
        if stage == "download" && mode == "download-failure" {
            println!("{SECRET}");
            eprintln!("{SECRET}");
            std::process::exit(42);
        }
        if stage == "download" && mode == "cancel" {
            println!("{SECRET}");
            eprintln!("{SECRET}");
            std::io::stdout().flush()?;
            std::io::stderr().flush()?;
            let mut ready = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("cancel-ready"))?;
            ready.write_all(CAP)?;
            ready.sync_all()?;
            std::thread::sleep(Duration::from_secs(30));
            return Err("cancelled worker unexpectedly survived".into());
        }
        if stage == "metadata" {
            println!("{{\"id\":\"managed-fixture\",\"duration\":10,\"title\":\"{SECRET}\"}}");
            eprintln!("{SECRET}");
            return Ok(true);
        }
        eprintln!("{SECRET}");
        println!("{SECRET}");
        let value = |flag: &str| -> Result<&std::ffi::OsString, Box<dyn std::error::Error>> {
            let index = args
                .iter()
                .position(|v| v == flag)
                .ok_or("fixture argument missing")?;
            args.get(index + 1)
                .ok_or_else(|| "fixture argument value missing".into())
        };
        let output = PathBuf::from(value("--paths")?).canonicalize()?;
        let library = root.join("media").canonicalize()?;
        if !output.starts_with(&library) || output == library {
            return Err("owned partial output required".into());
        }
        let template = value("--output")?
            .to_str()
            .ok_or("fixture template nonunicode")?;
        if template != "managed-native-job.%(ext)s" {
            return Err("fixture output template mismatch".into());
        }
        let path = output.join(template.replace("%(ext)s", "mp4"));
        let mut artifact = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        artifact.write_all(ARTIFACT)?;
        artifact.sync_all()?;
        println!("DLPAD_ARTIFACT:{}", serde_json::to_string(&path)?);
        Ok(true)
    }
    fn validate_root(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !root.is_absolute()
            || root.file_name().and_then(|v| v.to_str()) != Some("owned-fixture")
            || !root
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("dlpad-paired-parent-crash-"))
        {
            return Err("owned fixture namespace required".into());
        }
        for p in root.ancestors() {
            if p.exists() && fs::symlink_metadata(p)?.file_attributes() & 0x400 != 0 {
                return Err("fixture symlink rejected".into());
            }
        }
        Ok(())
    }
    fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if bytes.len() > 16384 {
            return Err("fixture evidence write bound".into());
        }
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        Ok(())
    }
    fn creation() -> Result<u64, Box<dyn std::error::Error>> {
        use windows_sys::Win32::{
            Foundation::FILETIME,
            System::Threading::{GetCurrentProcess, GetProcessTimes},
        };
        let mut c = FILETIME::default();
        let mut e = FILETIME::default();
        let mut k = FILETIME::default();
        let mut u = FILETIME::default();
        if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0 {
            return Err("parent identity unavailable".into());
        }
        Ok((u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime))
    }
    pub(super) fn parent(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        validate_root(root)?;
        if root.exists() {
            if !root.is_dir() || fs::read_dir(root)?.next().is_some() {
                return Err("fresh empty fixture required".into());
            }
        } else {
            fs::create_dir(root)?;
        }
        let root = root.canonicalize()?;
        validate_root(&root)?;
        write_new(&root.join("fixture.cap"), CAP)?;
        write_new(&root.join("foreign-sentinel"), SENTINEL)?;
        write_new(&root.join("mode"), b"cancel")?;
        let data = root.join("data");
        let library = root.join("media");
        fs::create_dir(&data)?;
        fs::create_dir(&library)?;
        let managed_root = data.join("managed-tools");
        let mut manager = ToolManager::new(&managed_root)?;
        let directory = managed_root.join("yt-dlp/2026.09.30").join(TARGET);
        fs::create_dir_all(&directory)?;
        let original = std::env::current_exe()?.canonicalize()?;
        let tool = directory.join("yt-dlp.exe");
        fs::copy(&original, &tool)?;
        let original_sha = hash(&original)?;
        let tool_sha = hash(&tool)?;
        if original_sha != tool_sha {
            return Err("tool copy mismatch".into());
        }
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: "2026.09.30".into(),
            target: TARGET.into(),
            source_url: "https://example.invalid/synthetic-self.exe".into(),
            sha256: tool_sha.clone(),
            license: "synthetic native fixture only".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp.exe".into(),
        };
        write_new(
            &directory.join("manifest.json"),
            &serde_json::to_vec(&manifest)?,
        )?;
        manager.select_existing(&manifest)?;
        let execution = manager.begin_execution()?;
        let source = root.join("source.cookies");
        write_new(&source, COOKIES)?;
        let cache = data.join("session-cookies");
        prepare_session_cache(&cache)?;
        write_new(&cache.join("foreign-sentinel"), SENTINEL)?;
        let mut cookies = import_session(&source, &cache)?;
        let private = cookies.path_for_tool().to_owned();
        let owner = private.parent().ok_or("cookie owner missing")?.to_owned();
        let parent_creation = creation()?;
        let pins = serde_json::json!({"schema":1,"parent_creation":parent_creation,"original":original,"original_sha":original_sha,"tool":tool,"tool_sha":tool_sha,"source":source,"source_sha":hash(&source)?,"private":private,"private_sha":hash(&private)?,"cookie_owner":owner,"cap_sha":hash(&root.join("fixture.cap"))?,"foreign_sha":hash(&root.join("foreign-sentinel"))?});
        write_new(
            &root.join("original-pins.json"),
            &serde_json::to_vec_pretty(&pins)?,
        )?;
        let repo = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/synthetic-managed-download")
                .map_err(|_| "fixture URL")?,
            output_dir: library,
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        repo.create_job("managed-native-job", &options)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let ready = root.join("cancel-ready");
        let signal = std::thread::spawn(move || -> Result<(), String> {
            let end = Instant::now() + Duration::from_secs(20);
            while Instant::now() < end {
                if fs::read(&ready).is_ok_and(|b| b == CAP) {
                    println!(
                        "{}",
                        serde_json::json!({"schema":1,"phase":"paired-tool-started","parent_creation":parent_creation})
                    );
                    std::io::stdout()
                        .flush()
                        .map_err(|_| "ready flush failed".to_owned())?;
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            flag.store(true, Ordering::Release);
            Err("actual reader readiness missing".into())
        });
        let result = repo.execute_with_session(
            "managed-native-job",
            &options,
            &tool,
            DownloadSession {
                #[cfg(windows)]
                runtime_environment: None,
                cancel: &cancel,
                pause: None,
                progress: None,
                cookies: Some(&cookies),
                managed: Some(&execution),
            },
        );
        let observed = signal.join().map_err(|_| "readiness observer panic")?;
        cookies.defer_cleanup();
        observed.map_err(|_| "actual reader readiness missing")?;
        let _ = result;
        Err("parent was not terminated during live reader".into())
    }
    pub(super) fn prepare(root: &Path, malformed: bool) -> Result<(), Box<dyn std::error::Error>> {
        validate_root(root)?;
        if root.exists() {
            if !root.is_dir() || fs::read_dir(root)?.next().is_some() {
                return Err("fresh empty fixture required".into());
            }
        } else {
            fs::create_dir(root)?;
        }
        let root = root.canonicalize()?;
        validate_root(&root)?;
        write_new(&root.join("fixture.cap"), CAP)?;
        write_new(&root.join("foreign-sentinel"), SENTINEL)?;
        let data = root.join("data");
        fs::create_dir(&data)?;
        let managed = data.join("managed-tools");
        let manager = ToolManager::new(&managed)?;
        let execution = manager.begin_execution()?;
        // Drop deliberately preserves original pristine pending state; no finish/recover.
        drop(execution);
        let marker = managed.join(".execution-pending");
        if malformed {
            if fs::symlink_metadata(&marker)?.file_attributes() & 0x400 != 0
                || fs::metadata(&marker)?.len() > 16384
            {
                return Err("original marker unsafe".into());
            }
            let mut original = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&marker)?;
            original.write_all(b"{\"version\":")?;
            original.sync_all()?;
        }
        if !marker.is_file()
            || !managed.join(".execution-readers.json").is_file()
            || (0..8).any(|i| {
                !managed
                    .join(format!(".execution-reader-{i}.receipt"))
                    .is_file()
            })
        {
            return Err("prepared pending files missing".into());
        }
        if fs::read(root.join("foreign-sentinel"))? != SENTINEL
            || fs::read(root.join("fixture.cap"))? != CAP
        {
            return Err("prepare originals changed".into());
        }
        let result = serde_json::json!({"schema":1,"phase":if malformed {"malformed-prepared"} else {"pending-prepared"},"pending_preserved":true,"pristine_reader":true,"source_foreign_preserved":true,"recovery_called":false,"marker_sha256":hash(&marker)?,"marker_bytes":fs::metadata(&marker)?.len(),"cap_sha256":hash(&root.join("fixture.cap"))?,"foreign_sha256":hash(&root.join("foreign-sentinel"))?});
        write_new(
            &root.join("prepared-evidence.json"),
            &serde_json::to_vec_pretty(&result)?,
        )?;
        println!("{result}");
        std::io::stdout().flush()?;
        Ok(())
    }
    pub(super) fn inspect(root: &Path, preserve: bool) -> Result<(), Box<dyn std::error::Error>> {
        validate_root(root)?;
        let root = root.canonicalize()?;
        validate_root(&root)?;
        if fs::read(root.join("fixture.cap"))? != CAP
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("fixture capability changed".into());
        }
        let pins_path = root.join("original-pins.json");
        if fs::metadata(&pins_path)?.len() > 16384 {
            return Err("pins bound".into());
        }
        let pins: serde_json::Value = serde_json::from_slice(&fs::read(&pins_path)?)?;
        let path = |key: &str| -> Result<PathBuf, Box<dyn std::error::Error>> {
            let p = PathBuf::from(pins[key].as_str().ok_or("missing pin")?);
            if !p.starts_with(&root) || p == root {
                return Err("pin outside fixture".into());
            }
            for a in p.ancestors() {
                if a.exists() && fs::symlink_metadata(a)?.file_attributes() & 0x400 != 0 {
                    return Err("pin symlink".into());
                }
            }
            Ok(p)
        };
        let owner = path("cookie_owner")?;
        let data = root.join("data");
        let managed = data.join("managed-tools");
        let tool = path("tool")?;
        let source = path("source")?;
        let private = path("private")?;
        let end = Instant::now() + Duration::from_secs(15);
        let receipts: Vec<_> = (0..3)
            .map(|i| {
                (
                    format!("managed-{i}.receipt"),
                    managed.join(format!(".execution-reader-{i}.receipt")),
                )
            })
            .chain((0..2).map(|i| {
                (
                    format!("cookie-{i}.receipt"),
                    owner.join(format!("reader{i}.receipt")),
                )
            }))
            .collect();
        loop {
            if receipts
                .iter()
                .all(|(_, p)| fs::metadata(p).is_ok_and(|m| m.len() == 256))
            {
                break;
            }
            if Instant::now() >= end {
                return Err("actual receipt readiness deadline".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let receipt_dir = root.join("crash-receipt-evidence");
        fs::create_dir(&receipt_dir)?;
        let mut hashes = Vec::new();
        for (name, p) in &receipts {
            let b = fs::read(p)?;
            if b.len() != 256 {
                return Err("receipt changed".into());
            }
            write_new(&receipt_dir.join(name), &b)?;
            hashes.push(serde_json::json!({"file":name,"bytes":b.len(),"sha256":format!("{:x}",Sha256::digest(&b))}));
        }
        for (key, p) in [
            ("source_sha", &source),
            ("private_sha", &private),
            ("tool_sha", &tool),
            ("cap_sha", &root.join("fixture.cap")),
            ("foreign_sha", &root.join("foreign-sentinel")),
        ] {
            if hash(p)? != pins[key].as_str().ok_or("missing digest")? {
                return Err("original payload changed".into());
            }
        }
        if fs::read(&source)? != COOKIES
            || fs::read(data.join("session-cookies/foreign-sentinel"))? != SENTINEL
        {
            return Err("source/sentinel changed".into());
        }
        let original = std::env::current_exe()?.canonicalize()?;
        if pins["original"].as_str() != original.to_str()
            || hash(&original)?
                != pins["original_sha"]
                    .as_str()
                    .ok_or("original digest missing")?
        {
            return Err("original fixture binary changed".into());
        }
        if preserve {
            if !owner.is_dir()
                || !private.is_file()
                || !managed.join(".execution-pending").is_file()
                || !managed.join(".execution-readers.json").is_file()
                || (0..8).any(|i| {
                    !managed
                        .join(format!(".execution-reader-{i}.receipt"))
                        .is_file()
                })
            {
                return Err("original pending namespace incomplete".into());
            }
            let result = serde_json::json!({"schema":1,"phase":"paired-preserved","both_receipts_ready":true,"source_foreign_preserved":true,"recovery_called":false,"receipt_evidence":hashes,"pending_preserved":true,"cookie_owner_preserved":true});
            write_new(
                &root.join("preserved-evidence.json"),
                &serde_json::to_vec_pretty(&result)?,
            )?;
            println!("{result}");
            std::io::stdout().flush()?;
            return Ok(());
        }
        let manager = ToolManager::new(&managed)?;
        let recovered = manager.recover_execution()?;
        if !recovered.recovered || recovered.pending {
            return Err("managed public recovery incomplete".into());
        }
        let cookie = dlpad_engine::cookies::recover_sessions(&data.join("session-cookies"))?;
        // The unrelated sentinel is deliberately rejected and preserved by the
        // bounded cache scan; only the one original cookie owner is recovered.
        if cookie.scanned != 2
            || cookie.recovered != 1
            || cookie.deferred != 0
            || cookie.rejected != 1
            || cookie.truncated
        {
            return Err("cookie public recovery incomplete".into());
        }
        if owner.exists()
            || managed.join(".execution-pending").exists()
            || managed.join(".execution-readers.json").exists()
            || (0..8).any(|i| {
                managed
                    .join(format!(".execution-reader-{i}.receipt"))
                    .exists()
            })
        {
            return Err("recovery files remain".into());
        }
        manager.begin_execution()?.finish()?;
        let recovered_jobs = dlpad_engine::download::recover_at_startup(&data)?;
        let repo = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let job = repo.job("managed-native-job")?;
        if job.state != JobState::Interrupted || !job.artifacts.is_empty() || recovered_jobs != 1 {
            return Err("crash database recovery invalid".into());
        }
        let saved = serde_json::to_string(&job)?;
        for forbidden in [
            SECRET.to_owned(),
            source.to_string_lossy().into_owned(),
            private.to_string_lossy().into_owned(),
        ] {
            if saved.contains(&forbidden) {
                return Err("job authentication leak".into());
            }
        }
        for name in ["jobs.sqlite", "jobs.sqlite-wal", "jobs.sqlite-shm"] {
            let p = data.join(name);
            if !p.exists() {
                continue;
            }
            if fs::metadata(&p)?.len() > 8 * 1024 * 1024 {
                return Err("database bound".into());
            }
            let b = fs::read(p)?;
            let text = String::from_utf8_lossy(&b);
            for forbidden in [
                SECRET.to_owned(),
                source.to_string_lossy().into_owned(),
                private.to_string_lossy().into_owned(),
            ] {
                if text.contains(&forbidden) {
                    return Err("database authentication leak".into());
                }
            }
        }
        if hash(&tool)? != pins["tool_sha"].as_str().ok_or("tool digest")?
            || hash(&source)? != pins["source_sha"].as_str().ok_or("source digest")?
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("postrecovery source changed".into());
        }
        let result = serde_json::json!({"schema":1,"phase":"paired-recovery-evidence","both_receipts_valid":true,"source_foreign_preserved":true,"managed_recovered":true,"cookie_recovered":true,"managed_files_removed":true,"cookie_owner_removed":true,"finish_and_reuse":true,"db_authentication_absent":true,"job_state":"Interrupted","receipt_proof":"original receipt copies preserved; actual validity established by both public product recovery APIs","receipt_evidence":hashes});
        write_new(
            &root.join("recovery-evidence.json"),
            &serde_json::to_vec_pretty(&result)?,
        )?;
        println!("{result}");
        std::io::stdout().flush()?;
        Ok(())
    }
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dlpad_engine::process::guardian_entry()? {
        return Ok(());
    }
    dlpad_engine::process::register_self_guardian()?;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if native::tool(&args)? {
        return Ok(());
    }
    if args.len() != 2 {
        return Err("usage: paired-parent|paired-inspect|paired-preserve|prepare-pending|prepare-malformed owned-root".into());
    }
    match args[0].to_str() {
        Some("paired-parent") => native::parent(std::path::Path::new(&args[1])),
        Some("paired-inspect") => native::inspect(std::path::Path::new(&args[1]), false),
        Some("paired-preserve") => native::inspect(std::path::Path::new(&args[1]), true),
        Some("prepare-pending") => native::prepare(std::path::Path::new(&args[1]), false),
        Some("prepare-malformed") => native::prepare(std::path::Path::new(&args[1]), true),
        _ => Err("invalid fixture role".into()),
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows native paired parent crash fixture only");
}
