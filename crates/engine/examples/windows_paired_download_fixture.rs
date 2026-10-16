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
        download::{DownloadError, DownloadRepository, DownloadSession, SESSION_OUTPUT_OMITTED},
        tools::{InstallManifest, Ownership, ToolManager},
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        io::{Read, Write},
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::{Duration, Instant},
    };
    const COOKIES: &[u8] = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tpaired-fixture-secret-only\n";
    const SECRET: &str = "paired-fixture-secret-only";
    const CAP: &[u8] = b"dlpad-paired-download-synthetic-v1";
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
        let temp = std::env::temp_dir().canonicalize()?;
        if root.parent() != Some(temp.as_path())
            || !root
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("dlpad-paired-download-owned-"))
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
    pub(super) fn run(output: &Path, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !["success", "metadata-failure", "download-failure", "cancel"].contains(&mode) {
            return Err("fixture mode invalid".into());
        }
        let mut evidence = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?;
        let temp = tempfile::Builder::new()
            .prefix("dlpad-paired-download-owned-")
            .tempdir()?;
        // Evidence collector receives ownership. Preserve failures too; no blind cleanup
        // while a helper may retain a reader after uncertain terminal proof.
        let root = temp.keep().canonicalize()?;
        eprintln!("DLPAD_FIXTURE_ROOT:{}", serde_json::to_string(&root)?);
        let data = root.join("data");
        let library = root.join("media");
        fs::create_dir(&data)?;
        fs::create_dir(&library)?;
        fs::write(root.join("fixture.cap"), CAP)?;
        fs::write(root.join("mode"), mode)?;
        fs::write(root.join("foreign-sentinel"), SENTINEL)?;
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
            return Err("copied self fixture hash mismatch".into());
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
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        manager.select_existing(&manifest)?;
        let execution = manager.begin_execution()?;
        let source = root.join("source.cookies");
        fs::write(&source, COOKIES)?;
        let source_sha = hash(&source)?;
        let cache = data.join("session-cookies");
        prepare_session_cache(&cache)?;
        let cache_sentinel = cache.join("foreign-sentinel");
        fs::write(&cache_sentinel, SENTINEL)?;
        let mut cookies = import_session(&source, &cache)?;
        let private = cookies.path_for_tool().to_owned();
        let cookie_owner = private.parent().ok_or("private owner missing")?.to_owned();
        let repository = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/synthetic-managed-download")
                .map_err(|_| "fixture URL policy")?,
            output_dir: library,
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        repository.create_job("managed-native-job", &options)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let stopper = if mode == "cancel" {
            let flag = cancel.clone();
            let ready = root.join("cancel-ready");
            Some(std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(8);
                while Instant::now() < deadline {
                    if fs::read(&ready).is_ok_and(|bytes| bytes == CAP) {
                        flag.store(true, Ordering::Release);
                        return true;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                flag.store(true, Ordering::Release);
                false
            }))
        } else {
            None
        };
        let result = repository.execute_with_session(
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
        if let Some(stopper) = stopper
            && !stopper.join().map_err(|_| "cancel observer panic")?
        {
            cookies.defer_cleanup();
            return Err("actual cookie-reader readiness not observed".into());
        }
        let execution_summary;
        let result = match (mode, result) {
            ("success", Ok(value)) => {
                execution_summary = serde_json::json!({"exit_code":value.execution.exit_code,"outcome":format!("{:?}",value.execution.outcome)});
                Some(value)
            }
            (
                "metadata-failure",
                Err(DownloadError::Execution {
                    stage: "metadata",
                    result,
                }),
            ) if result.exit_code == Some(41)
                && result.outcome == dlpad_engine::process::ProcessOutcome::Exited
                && result.logs == SESSION_OUTPUT_OMITTED =>
            {
                execution_summary = serde_json::json!({"exit_code":result.exit_code,"outcome":format!("{:?}",result.outcome)});
                None
            }
            (
                "download-failure",
                Err(DownloadError::Execution {
                    stage: "download",
                    result,
                }),
            ) if result.exit_code == Some(42)
                && result.outcome == dlpad_engine::process::ProcessOutcome::Exited
                && result.logs == SESSION_OUTPUT_OMITTED =>
            {
                execution_summary = serde_json::json!({"exit_code":result.exit_code,"outcome":format!("{:?}",result.outcome)});
                None
            }
            (
                "cancel",
                Err(DownloadError::Execution {
                    stage: "download",
                    result,
                }),
            ) if result.outcome == dlpad_engine::process::ProcessOutcome::Cancelled
                && result.logs == SESSION_OUTPUT_OMITTED =>
            {
                execution_summary = serde_json::json!({"exit_code":result.exit_code,"outcome":format!("{:?}",result.outcome)});
                None
            }
            (_, _) => {
                cookies.defer_cleanup();
                return Err("actual paired outcome mismatch or unconfirmed cleanup".into());
            }
        };
        let stored = repository.job("managed-native-job")?;
        let expected_state = match mode {
            "success" => JobState::Downloaded,
            "cancel" => JobState::Cancelled,
            _ => JobState::Failed,
        };
        if stored.state != expected_state {
            cookies.defer_cleanup();
            return Err("persisted job state invalid".into());
        }
        if let Some(result) = &result {
            if result.execution.logs != SESSION_OUTPUT_OMITTED
                || result.job.state != JobState::Downloaded
                || result.job.artifacts.len() != 1
            {
                return Err("successful authenticated job invalid".into());
            }
            let artifact = &result.job.artifacts[0];
            if artifact.bytes != ARTIFACT.len() as u64 || fs::read(&artifact.path)? != ARTIFACT {
                return Err("actual artifact bytes mismatch".into());
            }
            if serde_json::to_string(&result.metadata)?.contains(SECRET)
                || result.metadata["duration"].as_f64() != Some(10.0)
            {
                return Err("successful metadata privacy mismatch".into());
            }
        } else if !stored.artifacts.is_empty() {
            return Err("failed job has artifact".into());
        }
        let managed_count = if mode == "metadata-failure" { 2 } else { 3 };
        let cookie_count = managed_count - 1;
        let stages: &[&str] = if mode == "metadata-failure" {
            &["version", "metadata"]
        } else {
            &["version", "metadata", "download"]
        };
        for stage in stages {
            if fs::read(root.join(format!("stage-{stage}")))? != stage.as_bytes() {
                return Err("missing actual fixture stage".into());
            }
        }
        let meta_path = fs::read(root.join("cookie-path-metadata.json"))?;
        if meta_path != serde_json::to_vec(&private)? {
            return Err("metadata cookie path mismatch".into());
        }
        if mode == "metadata-failure" {
            if root.join("stage-download").exists()
                || root.join("cookie-path-download.json").exists()
            {
                return Err("metadata failure ran download".into());
            }
        } else if fs::read(root.join("cookie-path-download.json"))? != meta_path {
            return Err("cookie path changed between readers".into());
        }
        // The original parent journal holds its native byte-range lock here.
        // A second reader handle cannot safely inspect it on Windows. Finish
        // validates the original Resolved records and exact bound receipts.
        let receipt_evidence = root.join("receipt-evidence");
        fs::create_dir(&receipt_evidence)?;
        let mut receipt_hashes = Vec::new();
        for i in 0..managed_count {
            let receipt = fs::read(managed_root.join(format!(".execution-reader-{i}.receipt")))?;
            if receipt.len() != 256 || &receipt[..8] != b"DLPMRC01" {
                return Err("managed actual receipt missing".into());
            }
            let name = format!("managed-{i}.receipt");
            fs::write(receipt_evidence.join(&name), &receipt)?;
            receipt_hashes.push(serde_json::json!({"file":name,"bytes":receipt.len(),"sha256":format!("{:x}",Sha256::digest(&receipt))}));
        }
        for i in 0..cookie_count {
            let receipt = fs::read(cookie_owner.join(format!("reader{i}.receipt")))?;
            if receipt.len() != 256 {
                return Err("cookie actual receipt missing".into());
            }
            let name = format!("cookie-{i}.receipt");
            fs::write(receipt_evidence.join(&name), &receipt)?;
            receipt_hashes.push(serde_json::json!({"file":name,"bytes":receipt.len(),"sha256":format!("{:x}",Sha256::digest(&receipt))}));
        }
        let saved = serde_json::to_string(&stored)?;
        for forbidden in [
            SECRET.to_owned(),
            source.to_string_lossy().into_owned(),
            private.to_string_lossy().into_owned(),
        ] {
            if saved.contains(&forbidden) {
                return Err("job evidence leaked authentication".into());
            }
        }
        for name in ["jobs.sqlite", "jobs.sqlite-wal", "jobs.sqlite-shm"] {
            let path = data.join(name);
            if !path.exists() {
                continue;
            }
            if fs::metadata(&path)?.len() > 8 * 1024 * 1024 {
                return Err("database fixture bound".into());
            }
            let db = fs::read(&path)?;
            let db_text = String::from_utf8_lossy(&db);
            for forbidden in [
                SECRET.to_owned(),
                source.to_string_lossy().into_owned(),
                private.to_string_lossy().into_owned(),
            ] {
                if db_text.contains(&forbidden) {
                    return Err("database leaked authentication".into());
                }
            }
        }
        cookies.validate_for_tool()?;
        cookies.cleanup()?;
        if cookie_owner.exists()
            || fs::read(&source)? != COOKIES
            || hash(&source)? != source_sha
            || fs::read(&cache_sentinel)? != SENTINEL
        {
            return Err("cookie cleanup/source preserve failed".into());
        }
        execution.finish()?;
        if managed_root.join(".execution-pending").exists()
            || managed_root.join(".execution-readers.json").exists()
        {
            return Err("completion gate remained".into());
        }
        for index in 0..8 {
            if managed_root
                .join(format!(".execution-reader-{index}.receipt"))
                .exists()
            {
                return Err("receipt cleanup incomplete".into());
            }
        }
        manager.begin_execution()?.finish()?;
        if hash(&tool)? != tool_sha
            || hash(&original)? != original_sha
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("original/sentinel changed".into());
        }
        let json = serde_json::json!({"version":1,"passed":true,"owned_root":root,"source_sha256":original_sha,"tool_sha256":tool_sha,"stages":stages,"receipt_proof":"actual stage markers and managed receipt magic/length plus cookie receipt lengths inspected; finish validated original records","job_state":format!("{:?}", stored.state),"artifact_count":stored.artifacts.len(),"mode":mode,"scenario":mode,"actual_execution":execution_summary,"receipt_evidence":receipt_hashes,"finish_and_reuse":true,"sentinel_preserved":true,"real_media":false,"network":false,"ffmpeg_deno_verified":false,"authenticated_output_omitted":true,"actual_managed_receipts":managed_count,"actual_cookie_receipts":cookie_count,"cookie_owner_removed":true,"source_sha256_cookie":source_sha,"source_cookie_preserved":true,"cache_sentinel_preserved":true,"db_authentication_absent":true,"metadata_download_private_path_equal":mode != "metadata-failure","metadata_private_path_verified":true});
        let bytes = serde_json::to_vec_pretty(&json)?;
        if bytes.len() > 16384 {
            return Err("evidence bound".into());
        }
        evidence.write_all(&bytes)?;
        evidence.sync_all()?;
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
    if (args.len() != 2 && args.len() != 4)
        || args[0] != "--fixture-run"
        || (args.len() == 4 && args[2] != "--scenario")
    {
        return Err("usage: --fixture-run fresh-result.json [--scenario success|metadata-failure|download-failure|cancel]".into());
    }
    native::run(
        std::path::Path::new(&args[1]),
        args.get(3)
            .map(|v| v.to_str().ok_or("nonunicode mode"))
            .transpose()?
            .unwrap_or("success"),
    )
}
#[cfg(not(windows))]
fn main() {
    println!("Windows native synthetic fixture only; not executed");
}
