//! Windows-only synthetic startup receipt seed.
//! Requires fresh session_cookie_fixture_tool with fixed cache alternative below.
#[cfg(windows)]
mod native {
    use dlpad_core::{
        jobs::JobState,
        options::{DownloadOptions, Quality, SourceUrl},
    };
    use dlpad_engine::{
        cookies::{import_session, prepare_session_cache},
        download::{DownloadRepository, DownloadSession},
    };
    use sha2::{Digest, Sha256};
    use std::{fs, path::PathBuf, sync::atomic::AtomicBool};
    const COOKIES:&[u8]=b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-download-fixture-only\n";
    const CAP: &[u8] = b"dlpad-windows-session-cookie-synthetic-fixture-v1";
    const SENTINEL: &[u8] = b"preserve unrelated startup fixture";
    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() != 3 {
            return Err(
                "usage: seed recovered|pending|error fresh_fixture_exe manifest_path".into(),
            );
        }
        let expected = args[0].to_str().ok_or("case invalid")?;
        if !["recovered", "pending", "error"].contains(&expected) {
            return Err("case invalid".into());
        }
        let tool_source = PathBuf::from(&args[1]).canonicalize()?;
        if !tool_source.is_file() {
            return Err("fresh fixture missing".into());
        }
        let manifest_path = PathBuf::from(&args[2]);
        // create_new output prevents overwriting existing caller evidence.
        let mut manifest_output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&manifest_path)?;
        let temp = tempfile::Builder::new()
            .prefix("dlpad-windows-cookie-download-owned-")
            .tempdir()?;
        let root = temp.path().canonicalize()?;
        let actual_temp = std::env::temp_dir().canonicalize()?;
        if root.parent() != Some(actual_temp.as_path()) {
            return Err("original Windows TEMP direct child required".into());
        }
        let appdata = root.join("data");
        let library = root.join("media");
        fs::create_dir(&appdata)?;
        fs::create_dir(&library)?;
        let source = root.join("source.cookies");
        fs::write(&source, COOKIES)?;
        let sentinel = root.join("foreign-sentinel");
        fs::write(&sentinel, SENTINEL)?;
        fs::write(root.join("fixture.cap"), CAP)?;
        fs::write(root.join("mode"), b"success")?;
        let tool = root.join("fixture-tool.exe");
        fs::copy(tool_source, &tool)?;
        let cache = appdata.join("session-cookies");
        prepare_session_cache(&cache)?;
        let mut lease = import_session(&source, &cache)?;
        let mut cookie_owner = lease
            .path_for_tool()
            .parent()
            .ok_or("copy parent missing")?
            .to_owned();
        if expected == "recovered" {
            let repo = DownloadRepository::open(&appdata.join("jobs.sqlite"))?;
            let options = DownloadOptions {
                url: SourceUrl::parse("https://fixture.invalid/native-cookie")
                    .map_err(|_| "synthetic URL invalid")?,
                output_dir: library.clone(),
                quality: Quality::Best,
                time_range: None,
                advanced: vec![],
                ffmpeg: None,
                deno: None,
                final_name_style: None,
                subtitles: None,
            };
            repo.create_job("startup-cookie-seed", &options)?;
            let cancel = std::sync::Arc::new(AtomicBool::new(false));
            let result = repo.execute_with_session(
                "startup-cookie-seed",
                &options,
                &tool,
                DownloadSession {
                    #[cfg(windows)]
                    runtime_environment: None,
                    cancel: &cancel,
                    pause: None,
                    progress: None,
                    cookies: Some(&lease),
                    managed: None,
                },
            )?;
            if result.job.state != JobState::Downloaded {
                return Err("fixture terminal state invalid".into());
            }
            // Actual public engine gate must complete both native helpers before defer.
            // Do not call recover_sessions here: the GUI is the recovery subject.
            lease.defer_cleanup();
            let record: serde_json::Value =
                serde_json::from_slice(&fs::read(cookie_owner.join("reader-state.json"))?)?;
            for index in 0..2 {
                if record["slots"][index]["state"] != "Resolved"
                    || fs::metadata(cookie_owner.join(format!("reader{index}.receipt")))?.len()
                        != 256
                {
                    return Err("actual completed receipt missing".into());
                }
            }
        } else {
            // Actual import creates immutable marker; pristine journal is not recovery authority.
            lease.defer_cleanup();
            if expected == "error" {
                // Owned fixture namespace only; no ACL/security setting changes.
                // Preserve the original complete session, replace expected cache with
                // a plain synthetic file so startup fails its directory/ownership gate.
                let original = appdata.join("preserved-session-cookies");
                fs::rename(&cache, &original)?;
                cookie_owner = original.join(cookie_owner.file_name().ok_or("owner name missing")?);
                fs::write(&cache, b"owned synthetic unsafe cache regular file")?;
            }
        }
        if fs::read(&source)? != COOKIES || fs::read(&sentinel)? != SENTINEL {
            return Err("preservation failed".into());
        }
        let manifest = serde_json::json!({"version":1,"owned_root":root,"appdata":appdata,"library":library,"source":source,"sentinel":sentinel,"cookie_owner":cookie_owner,"source_sha256":format!("{:x}",Sha256::digest(COOKIES)),"sentinel_sha256":format!("{:x}",Sha256::digest(SENTINEL)),"expected":expected});
        let bytes = serde_json::to_vec_pretty(&manifest)?;
        if bytes.len() > 16 * 1024 {
            return Err("manifest bound".into());
        }
        use std::io::Write;
        manifest_output.write_all(&bytes)?;
        manifest_output.sync_all()?;
        // Collector owns this explicit temporary root afterwards. Never glob clean.
        let _preserved_root = temp.keep();
        Ok(())
    }
    // REQUIRED NARROW FIXTURE CONTRACT PATCH (root owns actual example):
    // let legacy=root.join("private-cache");
    // let gui=root.join("data").join("session-cookies");
    // let owner=path.parent().ok_or("owner missing")?;
    // let cache=owner.parent().ok_or("cache missing")?;
    // Require cache == legacy OR cache == gui; exact filename cookies.txt;
    // owner basename cookie-session- + 64 lowercase hex; no additional ancestors;
    // retain existing direct ORIGINAL TEMP root/cap/source fixed-content checks,
    // plain path/reparse checks and bounded(path)==COOKIES. Do not broaden starts_with
    // to arbitrary root/appdata paths or accept CLI-provided arbitrary credentials.
    // Guardian helper executable must be fresh dlpad-process-guard.exe beside seed
    // (engine registration fallback resolves sibling); do not register seed as self.
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows native fixture required");
    std::process::exit(2);
}
