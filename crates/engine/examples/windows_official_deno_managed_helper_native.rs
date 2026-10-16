//! Official pinned Windows Deno through the public owned factory and independent guardian.
//! Offline preprocessed EJS fixture only; no site/player/credentials/media execution.
#[cfg(windows)]
mod native {
    use dlpad_engine::{
        file_identity,
        process::{ProcessOutcome, ProcessSpec},
        tools::{
            ArchiveComponent, ArchiveComponentRole, ArchivePackage, InstallManifest,
            ManagedReaderStage, Ownership, ToolManager,
        },
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{self, File},
        io::{Read, Seek, SeekFrom, Write},
        os::windows::{fs::MetadataExt, io::AsRawHandle},
        path::{Path, PathBuf},
        sync::{Arc, atomic::AtomicBool},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    const DENO_SHA: &str = "e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33";
    const LICENSE_SHA: &str = "f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588";
    const SOLVER_SHA: &str = "5d7d7a43f656ab8c555ee1ec5d6566d094577cf0f8f81ca33190f2115fba1887";
    const TARGET: &str = "x86_64-pc-windows-msvc";
    type Error = Box<dyn std::error::Error>;
    fn fixed() -> Error {
        "official managed Deno fixture rejected".into()
    }
    fn hash(file: &File, max: u64) -> Result<String, Error> {
        let mut f = file;
        f.seek(SeekFrom::Start(0))?;
        let mut sha = Sha256::new();
        let mut total = 0u64;
        let mut block = [0; 65536];
        loop {
            let n = f.read(&mut block)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > max {
                return Err(fixed());
            }
            sha.update(&block[..n]);
        }
        Ok(format!("{:x}", sha.finalize()))
    }
    fn checked_source(path: &Path, bytes: u64, sha: &str) -> Result<File, Error> {
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(fixed());
        }
        let mut ancestor = PathBuf::new();
        for c in path.components() {
            ancestor.push(c.as_os_str());
            if matches!(c, std::path::Component::Prefix(_)) {
                continue;
            }
            if fs::symlink_metadata(&ancestor)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT
                != 0
            {
                return Err(fixed());
            }
        }
        let original = file_identity::open_regular_no_reparse(path)?;
        let file = File::open(path)?;
        if file_identity::from_file(&original)? != file_identity::from_file(&file)?
            || file.metadata()?.len() != bytes
            || hash(&file, bytes)? != sha
        {
            return Err(fixed());
        }
        Ok(file)
    }
    fn copy(source: &File, path: &Path, bytes: u64, sha: &str) -> Result<(), Error> {
        let mut input = source;
        input.seek(SeekFrom::Start(0))?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        let mut total = 0;
        let mut block = [0; 65536];
        loop {
            let n = input.read(&mut block)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > bytes {
                return Err(fixed());
            }
            output.write_all(&block[..n])?;
        }
        output.sync_all()?;
        if total != bytes || hash(&File::open(path)?, bytes)? != sha {
            return Err(fixed());
        }
        Ok(())
    }
    fn native(file: &File) -> Result<(file_identity::NativeFileIdentity, u32, u64), Error> {
        let mut info = FILE_STANDARD_INFO::default();
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut info as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&info) as u32,
            )
        } == 0
            || info.Directory
            || info.DeletePending
            || info.NumberOfLinks != 1
        {
            return Err(fixed());
        }
        Ok((
            file_identity::from_file(file)?,
            info.NumberOfLinks,
            file.metadata()?.len(),
        ))
    }
    fn cache_inventory(cache: &Path) -> Result<(usize, u64), Error> {
        let mut stack = vec![cache.to_owned()];
        let mut count = 0;
        let mut bytes = 0u64;
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                count += 1;
                if count > 4096 {
                    return Err(fixed());
                }
                let meta = fs::symlink_metadata(entry.path())?;
                if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return Err(fixed());
                }
                if meta.is_dir() {
                    stack.push(entry.path());
                } else {
                    let file = file_identity::open_regular_no_reparse(&entry.path())?;
                    native(&file)?;
                    bytes = bytes.checked_add(meta.len()).ok_or_else(fixed)?;
                    if bytes > 32 * 1024 * 1024 {
                        return Err(fixed());
                    }
                }
            }
        }
        Ok((count, bytes))
    }
    pub(super) fn run(args: &[std::ffi::OsString]) -> Result<(), Error> {
        if args.len() != 5 || args[0] != "--fixture-run" {
            return Err("usage: --fixture-run pinned-deno.exe pinned-LICENSE pinned-solver.mjs fresh-result.json".into());
        }
        let deno_source = PathBuf::from(&args[1]);
        let license_source = PathBuf::from(&args[2]);
        let solver_source = PathBuf::from(&args[3]);
        let output_path = PathBuf::from(&args[4]);
        let deno = checked_source(&deno_source, 97462048, DENO_SHA)?;
        let license = checked_source(&license_source, 1074, LICENSE_SHA)?;
        let solver = checked_source(&solver_source, 159275, SOLVER_SHA)?;
        let original_ids = [
            file_identity::from_file(&deno)?,
            file_identity::from_file(&license)?,
            file_identity::from_file(&solver)?,
        ];
        let mut evidence = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_path)?;
        let temp = tempfile::Builder::new()
            .prefix("dlpad-official-deno-managed-owned-")
            .tempdir()?;
        let root = temp.keep().canonicalize()?;
        eprintln!("DLPAD_FIXTURE_ROOT:{}", serde_json::to_string(&root)?);
        fs::write(
            root.join("fixture.cap"),
            b"official-deno-managed-helper-synthetic-v1",
        )?;
        let sentinel = root.join("foreign-sentinel");
        fs::write(&sentinel, b"preserve owned offline sentinel")?;
        let managed = root.join("managed-tools");
        let manager = ToolManager::new(&managed)?;
        let directory = managed.join("deno/2.9.7").join(TARGET);
        fs::create_dir_all(&directory)?;
        let executable = directory.join("deno.exe");
        copy(&deno, &executable, 97462048, DENO_SHA)?;
        copy(&license, &directory.join("LICENSE"), 1074, LICENSE_SHA)?;
        let package=ArchivePackage{notices:vec![],version:1,layout:None,tool:"deno".into(),tool_version:"2.9.7".into(),target:TARGET.into(),source_url:"https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-pc-windows-msvc.zip".into(),archive_bytes:42630221,archive_sha256:"a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238".into(),license:"MIT; fixture assembly, transitive notices not adopted".into(),ownership:Ownership::Managed,components:vec![ArchiveComponent{name:"deno.exe".into(),role:ArchiveComponentRole::Executable,bytes:97462048,sha256:DENO_SHA.into()},ArchiveComponent{name:"LICENSE".into(),role:ArchiveComponentRole::Notice,bytes:1074,sha256:LICENSE_SHA.into()}]};
        let manifest = InstallManifest {
            tool: package.tool.clone(),
            version: package.tool_version.clone(),
            target: package.target.clone(),
            source_url: package.source_url.clone(),
            sha256: DENO_SHA.into(),
            license: package.license.clone(),
            ownership: Ownership::Managed,
            executable: "deno.exe".into(),
        };
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        fs::write(
            directory.join("archive-package.json"),
            serde_json::to_vec(&package)?,
        )?;
        manager.package_installed(&manifest)?.ok_or_else(fixed)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let policy = manager
            .prepare_managed_deno_environment(&executable, &cancel)?
            .ok_or_else(fixed)?;
        let cache = managed.join(".runtime-cache/deno");
        let owner = File::open(cache.join("owner.json"))?;
        let owner_sha = hash(&owner, 4096)?;
        let pin = file_identity::open_regular_no_reparse(&executable)?;
        let before = native(&pin)?;
        let solver_path = root.join("solver.mjs");
        copy(&solver, &solver_path, 159275, SOLVER_SHA)?;
        let pure = root.join("pure.mjs");
        let script = format!(
            "let sum=0;for(let i=1;i<=100000;i++)sum+=i;const states=await Promise.all(['read','net','env'].map(async name=>(await Deno.permissions.query({{name}})).state));let denied=false;try{{await Deno.readTextFile({});}}catch(e){{denied=e.name==='NotCapable'||e.name==='PermissionDenied';}}const result={{sum,states,denied}};console.log('DLPAD_PURE:'+JSON.stringify(result));if(sum!==5000050000||states.some(v=>v!=='denied')||!denied)Deno.exit(7);",
            serde_json::to_string(&sentinel)?
        );
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pure)?;
        f.write_all(script.as_bytes())?;
        f.sync_all()?;
        let execution = manager.begin_execution()?;
        let started = Instant::now();
        let mut outputs = Vec::new();
        for (index, path) in [None, Some(&pure), Some(&solver_path)]
            .into_iter()
            .enumerate()
        {
            let args = if let Some(path) = path {
                vec![
                    "run".into(),
                    "--no-config".into(),
                    "--no-lock".into(),
                    "--no-prompt".into(),
                    "--deny-read".into(),
                    "--deny-net".into(),
                    "--deny-env".into(),
                    path.as_os_str().to_owned(),
                ]
            } else {
                vec!["--version".into()]
            };
            let spec = ProcessSpec {
                runtime_environment: Some(policy.clone()),
                executable: executable.clone(),
                args,
                timeout: Duration::from_secs(20),
                max_log_bytes: 16384,
            };
            let result =
                execution.run_tool(&spec, &cancel, true, None, ManagedReaderStage::VersionProbe)?;
            if result.execution.exit_code != Some(0)
                || result.execution.outcome != ProcessOutcome::Exited
                || result.execution.truncated
                || result.stdout.len() > 16384
            {
                return Err(fixed());
            }
            let text = std::str::from_utf8(&result.stdout)?;
            if index == 0 {
                let lines: Vec<_> = text.lines().collect();
                if lines.len() != 3
                    || lines[0] != "deno 2.9.7 (stable, release, x86_64-pc-windows-msvc)"
                    || !lines[1].starts_with("v8 ")
                    || !lines[2].starts_with("typescript ")
                {
                    return Err(fixed());
                }
            } else {
                let prefix = if index == 1 {
                    "DLPAD_PURE:"
                } else {
                    "DLPAD_EJS:"
                };
                let rows: Vec<_> = text
                    .lines()
                    .filter_map(|line| line.strip_prefix(prefix))
                    .collect();
                if rows.len() != 1 {
                    return Err(fixed());
                }
                let value: serde_json::Value = serde_json::from_str(rows[0])?;
                if index == 1 {
                    if value["sum"] != 5000050000u64
                        || value["denied"] != true
                        || value["states"] != serde_json::json!(["denied", "denied", "denied"])
                    {
                        return Err(fixed());
                    }
                } else if value["actual"]["responses"][0]["data"]["dlpad-synthetic"]
                    != "citehtnys-dapld"
                    || value["actual"]["responses"][1]["data"]["fixture-signature"]
                        != "ixture-signature"
                    || value["actual"]["responses"][2]["type"] != "error"
                    || value["missing"]["responses"][0]["type"] != "error"
                {
                    return Err(fixed());
                }
                outputs.push(value);
            }
            if native(&pin)? != before {
                return Err(fixed());
            }
        }
        if cache.join("node_compat_bin/node.exe").exists() {
            return Err(fixed());
        }
        let inventory = cache_inventory(&cache)?;
        if hash(&owner, 4096)? != owner_sha
            || hash(&File::open(&executable)?, 97462048)? != DENO_SHA
        {
            return Err(fixed());
        }
        let receipts = root.join("receipt-evidence");
        fs::create_dir(&receipts)?;
        let mut receipt_evidence = Vec::new();
        for index in 0..3 {
            let bytes = fs::read(managed.join(format!(".execution-reader-{index}.receipt")))?;
            if bytes.len() != 256
                || &bytes[..8] != b"DLPMRC01"
                || bytes[9] != index
                || bytes[10] != 1
            {
                return Err(fixed());
            }
            let digest = format!("{:x}", Sha256::digest(&bytes));
            fs::write(receipts.join(format!("reader-{index}.receipt")), bytes)?;
            receipt_evidence.push(digest);
        }
        execution.finish()?;
        manager.begin_execution()?.finish()?;
        for index in 0..8 {
            if managed
                .join(format!(".execution-reader-{index}.receipt"))
                .exists()
            {
                return Err(fixed());
            }
        }
        if managed.join(".execution-pending").exists()
            || managed.join(".execution-readers.json").exists()
        {
            return Err(fixed());
        }
        for (file, path, bytes, sha, identity) in [
            (&deno, &deno_source, 97462048, DENO_SHA, &original_ids[0]),
            (
                &license,
                &license_source,
                1074,
                LICENSE_SHA,
                &original_ids[1],
            ),
            (
                &solver,
                &solver_source,
                159275,
                SOLVER_SHA,
                &original_ids[2],
            ),
        ] {
            if hash(file, bytes)? != sha
                || hash(&File::open(path)?, bytes)? != sha
                || file_identity::from_file(file)? != *identity
            {
                return Err(fixed());
            }
        }
        if fs::read(sentinel)? != b"preserve owned offline sentinel"
            || hash(&File::open(directory.join("LICENSE"))?, 1074)? != LICENSE_SHA
        {
            return Err(fixed());
        }
        let result = serde_json::json!({"schema":1,"passed":true,"owned_root":root,"official_deno_sha256":DENO_SHA,"official_license_sha256":LICENSE_SHA,"solver_sha256":SOLVER_SHA,"full_package_verified":true,"public_owned_factory":true,"independent_guardian_readers":3,"actual_receipt_sha256":receipt_evidence,"finish_and_reuse":true,"native_id_and_link_count_preserved":true,"native_before":{"identity":before.0,"links":before.1,"bytes":before.2},"native_after":{"identity":file_identity::from_file(&pin)?,"links":native(&pin)?.1,"bytes":native(&pin)?.2},"cache_owner_preserved":true,"node_shim_absent":true,"cache_entries":inventory.0,"cache_logical_bytes":inventory.1,"runtime_hard_quota":false,"offline_api_results":outputs,"elapsed_ms":started.elapsed().as_millis(),"actual_upstream_player":false,"site_media_network":false,"supply_adopted":false});
        let bytes = serde_json::to_vec_pretty(&result)?;
        if bytes.len() > 16384 {
            return Err(fixed());
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
    native::run(&std::env::args_os().skip(1).collect::<Vec<_>>())
}
#[cfg(not(windows))]
fn main() {
    println!("Windows-only official managed Deno fixture; not executed");
}
