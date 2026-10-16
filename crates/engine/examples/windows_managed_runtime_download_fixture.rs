//! Windows synthetic managed yt-dlp public repository vertical fixture.
//! Never contacts the fixture URL or invokes a real media tool.
#[cfg(windows)]
mod native {
    use dlpad_core::{
        jobs::JobState,
        options::{DownloadOptions, Quality, SourceUrl},
    };
    use dlpad_engine::{
        download::{DownloadError, DownloadRepository, DownloadSession},
        tools::{InstallManifest, Ownership, ToolManager},
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        io::{Read, Write},
        path::{Path, PathBuf},
        sync::{Arc, atomic::AtomicBool},
    };
    const CAP: &[u8] = b"dlpad-managed-runtime-download-synthetic-v1";
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
        let role = exe
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("tool role invalid")?;
        let levels = if role == "yt-dlp.exe" { 6 } else { 2 };
        let root = exe
            .ancestors()
            .nth(levels)
            .ok_or("fixture root missing")?
            .to_owned();
        let temp = std::env::temp_dir().canonicalize()?;
        if root.parent() != Some(temp.as_path())
            || !root
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("dlpad-managed-runtime-download-owned-"))
            || (role == "yt-dlp.exe"
                && exe
                    != root
                        .join("data/managed-tools/yt-dlp/2026.09.30")
                        .join(TARGET)
                        .join(role))
            || (role != "yt-dlp.exe"
                && (!["deno.exe", "ffmpeg.exe"].contains(&role)
                    || exe != root.join("runtimes").join(role)))
            || fs::read(root.join("fixture.cap"))? != CAP
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("synthetic fixture capability rejected".into());
        }
        Ok(root)
    }
    pub(super) fn tool(args: &[std::ffi::OsString]) -> Result<bool, Box<dyn std::error::Error>> {
        if !args.iter().any(|v| {
            v == "--version" || v == "-version" || v == "--dump-single-json" || v == "--output"
        }) {
            return Ok(false);
        }
        let root = root_for_tool()?;
        let role = std::env::current_exe()?
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("role invalid")?
            .to_owned();
        if role != "yt-dlp.exe" {
            let expected = if role == "deno.exe" {
                "--version"
            } else {
                "-version"
            };
            if args != [expected] {
                return Err("runtime argv mismatch".into());
            }
            let marker = if role == "deno.exe" {
                "runtime-deno"
            } else {
                "runtime-ffmpeg"
            };
            let mut seen = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(marker))?;
            seen.write_all(expected.as_bytes())?;
            seen.sync_all()?;
            if role == "deno.exe" {
                if fs::read(root.join("scenario"))? == b"version-mismatch" {
                    println!(
                        "deno 2.invalid.7 (stable, release, x86_64-pc-windows-msvc)\nv8 15.0.245.2-rusty\ntypescript 6.0.3"
                    );
                } else {
                    println!(
                        "deno 2.9.7 (stable, release, x86_64-pc-windows-msvc)\nv8 15.0.245.2-rusty\ntypescript 6.0.3"
                    );
                }
            } else {
                println!("ffmpeg version 7.1-synthetic fixture");
            }
            return Ok(true);
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
            println!("2026.09.30");
            return Ok(true);
        }
        let ffmpeg = root.join("runtimes/ffmpeg.exe").canonicalize()?;
        let deno = root.join("runtimes/deno.exe").canonicalize()?;
        let ffmpeg_i = args
            .iter()
            .position(|v| v == "--ffmpeg-location")
            .ok_or("FFmpeg arg absent")?;
        if args.get(ffmpeg_i + 1) != Some(&ffmpeg.as_os_str().to_owned()) {
            return Err("FFmpeg literal argv mismatch".into());
        }
        let deno_i = args
            .iter()
            .position(|v| v == "--js-runtimes")
            .ok_or("Deno arg absent")?;
        let mut runtime = std::ffi::OsString::from("deno:");
        runtime.push(&deno);
        if args.get(deno_i + 1) != Some(&runtime) || !args.iter().any(|v| v == "--no-js-runtimes") {
            return Err("Deno literal argv mismatch".into());
        }
        fs::write(
            root.join(format!("runtime-argv-{stage}.json")),
            serde_json::to_vec(
                &serde_json::json!({"ffmpeg":ffmpeg,"deno":deno,"deno_argument":runtime}),
            )?,
        )?;
        if stage == "metadata" {
            println!("{{\"id\":\"managed-fixture\",\"duration\":10}}");
            return Ok(true);
        }
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
    pub(super) fn run(output: &Path, scenario: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !["success", "version-mismatch"].contains(&scenario) {
            return Err("scenario invalid".into());
        }
        let mut evidence = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?;
        let temp = tempfile::Builder::new()
            .prefix("dlpad-managed-runtime-download-owned-")
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
        fs::write(root.join("scenario"), scenario)?;
        fs::write(root.join("foreign-sentinel"), SENTINEL)?;
        let managed_root = data.join("managed-tools");
        let mut manager = ToolManager::new(&managed_root)?;
        let directory = managed_root.join("yt-dlp/2026.09.30").join(TARGET);
        fs::create_dir_all(&directory)?;
        let original = std::env::current_exe()?.canonicalize()?;
        let tool = directory.join("yt-dlp.exe");
        fs::copy(&original, &tool)?;
        let runtimes = root.join("runtimes");
        fs::create_dir(&runtimes)?;
        let deno = runtimes.join("deno.exe");
        let ffmpeg = runtimes.join("ffmpeg.exe");
        fs::copy(&original, &deno)?;
        fs::copy(&original, &ffmpeg)?;
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
        let repository = DownloadRepository::open(&data.join("jobs.sqlite"))?;
        let options = DownloadOptions {
            url: SourceUrl::parse("https://fixture.invalid/synthetic-managed-download")
                .map_err(|_| "fixture URL policy")?,
            output_dir: library,
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: Some(ffmpeg.clone()),
            deno: Some(deno.clone()),
            final_name_style: None,
            subtitles: None,
        };
        repository.create_job("managed-native-job", &options)?;
        let cancel = Arc::new(AtomicBool::new(false));
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
                cookies: None,
                managed: Some(&execution),
            },
        );
        let result = match (scenario, result) {
            ("success", Ok(value)) => Some(value),
            ("version-mismatch", Err(DownloadError::DenoDiscovery(_))) => None,
            (_, _) => return Err("runtime actual result mismatch".into()),
        };
        if let Some(result) = &result {
            if result.job.state != JobState::Downloaded || result.job.artifacts.len() != 1 {
                return Err("managed job not downloaded".into());
            }
            let artifact = &result.job.artifacts[0];
            if artifact.bytes != ARTIFACT.len() as u64 || fs::read(&artifact.path)? != ARTIFACT {
                return Err("actual artifact bytes mismatch".into());
            }
            if result.ffmpeg_version.as_deref() != Some("7.1-synthetic")
                || result.deno_version.as_deref() != Some("2.9.7")
            {
                return Err("returned runtime version mismatch".into());
            }
        }
        let stored = repository.job("managed-native-job")?;
        if stored.state
            != if scenario == "success" {
                JobState::Downloaded
            } else {
                JobState::Failed
            }
            || (scenario != "success" && !stored.artifacts.is_empty())
        {
            return Err("persisted job state invalid".into());
        }
        let stages: &[&str] = if scenario == "success" {
            &["version", "metadata", "download"]
        } else {
            &["version"]
        };
        for stage in stages {
            if fs::read(root.join(format!("stage-{stage}")))? != stage.as_bytes() {
                return Err("missing actual fixture stage".into());
            }
        }
        if fs::read(root.join("runtime-ffmpeg"))? != b"-version"
            || fs::read(root.join("runtime-deno"))? != b"--version"
        {
            return Err("runtime discovery not observed".into());
        }
        let receipt_count = if scenario == "success" { 5 } else { 3 };
        let receipt_dir = root.join("receipt-evidence");
        fs::create_dir(&receipt_dir)?;
        let mut receipt_hashes = Vec::new();
        for index in 0..receipt_count {
            let b = fs::read(managed_root.join(format!(".execution-reader-{index}.receipt")))?;
            let expected_stage = if index < 3 {
                1
            } else if index == 3 {
                2
            } else {
                3
            };
            if b.len() != 256
                || &b[..8] != b"DLPMRC01"
                || b[9] != index as u8
                || b[10] != expected_stage
            {
                return Err("actual guarded receipt missing".into());
            }
            let name = format!("reader-{index}.receipt");
            fs::write(receipt_dir.join(&name), &b)?;
            receipt_hashes.push(serde_json::json!({"file":name,"bytes":256,"sha256":format!("{:x}",Sha256::digest(&b))}));
        }
        let db = rusqlite::Connection::open(data.join("jobs.sqlite"))?;
        let actual: Vec<(String, String)> = {
            let mut stmt=db.prepare("SELECT component,tool_version FROM download_attempt_components WHERE job_id='managed-native-job' ORDER BY component")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        let expected: Vec<(String, String)> = if scenario == "success" {
            vec![
                ("deno".into(), "2.9.7".into()),
                ("ffmpeg".into(), "7.1-synthetic".into()),
            ]
        } else {
            vec![("ffmpeg".into(), "7.1-synthetic".into())]
        };
        if actual != expected {
            return Err("persisted runtime component mismatch".into());
        }
        drop(db);
        if scenario == "success" {
            if fs::read(root.join("runtime-argv-metadata.json"))?
                != fs::read(root.join("runtime-argv-download.json"))?
            {
                return Err("runtime paths changed".into());
            }
        } else if root.join("stage-metadata").exists() || root.join("stage-download").exists() {
            return Err("version mismatch continued execution".into());
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
            || hash(&deno)? != original_sha
            || hash(&ffmpeg)? != original_sha
            || fs::read(root.join("foreign-sentinel"))? != SENTINEL
        {
            return Err("original/sentinel changed".into());
        }
        let json = serde_json::json!({"version":1,"passed":true,"owned_root":root,"source_sha256":original_sha,"tool_sha256":tool_sha,"stages":if scenario=="success" {vec!["VersionProbe","VersionProbe","VersionProbe","Metadata","Download"]}else{vec!["VersionProbe","VersionProbe","VersionProbe"]},"receipt_proof":"public managed execution and finish succeeded; no extra journal handle opened","scenario":scenario,"job_state":format!("{:?}",stored.state),"artifact_count":stored.artifacts.len(),"actual_managed_receipts":receipt_count,"receipt_evidence":receipt_hashes,"attempt_components":actual,"runtime_argv_verified":scenario=="success","finish_and_reuse":true,"sentinel_preserved":true,"real_media":false,"network":false,"ffmpeg_deno_verified":false,"synthetic_runtime_discovery_verified":true,"runtime_binaries":"copied self fixture; not actual Deno/FFmpeg"});
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
        return Err(
            "usage: --fixture-run fresh-result.json [--scenario success|version-mismatch]".into(),
        );
    }
    native::run(
        std::path::Path::new(&args[1]),
        args.get(3)
            .map(|v| v.to_str().ok_or("invalid scenario"))
            .transpose()?
            .unwrap_or("success"),
    )
}
#[cfg(not(windows))]
fn main() {
    println!("Windows native synthetic fixture only; not executed");
}
