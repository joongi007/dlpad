//! Synthetic selected-process public API acceptance; root runs only in owned private desktop.
#[cfg(windows)]
fn run() -> std::io::Result<()> {
    use dlpad_engine::{
        app_install::{
            SelectedExitReason, create_registration_receipt, inspect_active_selection,
            inspect_registered_with_receipt, launch_explicit_selection,
            manifest::{BundleEntry, BundleManifest},
            prepare_new, register_fresh, select_registered_pair,
        },
        file_identity,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Component, Path, PathBuf},
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::Storage::FileSystem::*;
    fn bad() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "fresh registration fixture rejected",
        )
    }
    fn hash(path: &Path, end: Instant) -> io::Result<String> {
        let mut f = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        if !f.metadata()?.is_file() || f.metadata()?.len() > 128 * 1024 * 1024 {
            return Err(bad());
        }
        let initial_len = f.metadata()?.len();
        let id = file_identity::from_file(&f)?;
        let mut total = 0u64;
        let mut h = Sha256::new();
        let mut b = [0; 65536];
        loop {
            if Instant::now() >= end {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let n = f.read(&mut b)?;
            if n == 0 {
                break;
            }
            total = total.checked_add(n as u64).ok_or_else(bad)?;
            if total > initial_len || total > 128 * 1024 * 1024 {
                return Err(bad());
            }
            h.update(&b[..n]);
        }
        if total != initial_len
            || f.metadata()?.len() != initial_len
            || file_identity::from_file(&f)? != id
        {
            return Err(bad());
        }
        Ok(format!("{:x}", h.finalize()))
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetThreadDesktop(thread: u32) -> *mut std::ffi::c_void;
        fn OpenInputDesktop(flags: u32, inherit: i32, access: u32) -> *mut std::ffi::c_void;
        fn CloseDesktop(desktop: *mut std::ffi::c_void) -> i32;
        fn GetUserObjectInformationW(
            object: *mut std::ffi::c_void,
            index: i32,
            buffer: *mut std::ffi::c_void,
            length: u32,
            needed: *mut u32,
        ) -> i32;
    }
    fn desktop_name(handle: *mut std::ffi::c_void) -> io::Result<Vec<u16>> {
        if handle.is_null() {
            return Err(bad());
        }
        let mut name = [0u16; 512];
        let mut needed = 0;
        // SAFETY: borrowed desktop, UOI_NAME query and bounded UTF16 output.
        if unsafe {
            GetUserObjectInformationW(handle, 2, name.as_mut_ptr().cast(), 1024, &mut needed)
        } == 0
            || !(2..=1024).contains(&needed)
            || needed % 2 != 0
        {
            return Err(bad());
        }
        let units = needed as usize / 2;
        if name[units - 1] != 0 {
            return Err(bad());
        }
        Ok(name[..units - 1].to_vec())
    }
    // Refuse default/input desktop before any executable launch. This fixture never switches it.
    // SAFETY: readonly desktop handles; input handle is closed regardless of name query result.
    let current =
        unsafe { GetThreadDesktop(windows_sys::Win32::System::Threading::GetCurrentThreadId()) };
    let input = unsafe { OpenInputDesktop(0, 0, 1) };
    let private_name = desktop_name(current);
    let input_name = desktop_name(input);
    if !input.is_null() {
        unsafe {
            CloseDesktop(input);
        }
    }
    let private_name = private_name?;
    if private_name == input_name?
        || !String::from_utf16_lossy(&private_name).starts_with("dlpad-cli-")
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "owned private desktop required",
        ));
    }
    let args: Vec<_> = std::env::args_os().take(7).collect();
    if !(args.len() == 4 || args.len() == 6) || args[1] != "--selected-process-owned-fixture" {
        return Err(bad());
    }
    fn source_arg(path: &std::ffi::OsStr, sha: &std::ffi::OsStr) -> io::Result<(PathBuf, String)> {
        let path = PathBuf::from(path);
        let sha = sha.to_str().ok_or_else(bad)?.to_owned();
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || sha.len() != 64
            || !sha
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(bad());
        }
        let m = std::fs::symlink_metadata(&path)?;
        if !m.is_file() || m.file_type().is_symlink() {
            return Err(bad());
        }
        Ok((path, sha))
    }
    let cpu = source_arg(&args[2], &args[3])?;
    let helper = if args.len() == 6 {
        Some(source_arg(&args[4], &args[5])?)
    } else {
        None
    };
    let end = Instant::now() + Duration::from_secs(180);
    if hash(&cpu.0, end)? != cpu.1 {
        return Err(bad());
    }
    let root = tempfile::Builder::new()
        .prefix("dlpad-selected-process-owned-")
        .tempdir()?
        .keep();
    let data = root.join("data");
    let media = root.join("media");
    let tmp = root.join("tmp");
    for p in [&data, &media, &tmp] {
        std::fs::create_dir(p)?;
    }
    // SAFETY: this single-threaded fixture creates no parent threads. Set once,
    // before app_install work/any launch; no global machine/account environment edit.
    unsafe {
        std::env::set_var("DLPAD_TEST_DATA", &data);
        std::env::set_var("DLPAD_LIBRARY_ROOT", &media);
        std::env::set_var("TMP", &tmp);
        std::env::set_var("TEMP", &tmp);
    }
    let sentinel = root.join("foreign-sentinel");
    let notice = root.join("notice.txt");
    let notice_bytes = b"synthetic local CLI notice; no legal coverage claim\n";
    for (p, bytes) in [
        (&sentinel, b"owned synthetic preserve".as_slice()),
        (&notice, notice_bytes.as_slice()),
    ] {
        let mut file = File::create_new(p)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    let sentinel_sha = hash(&sentinel, end)?;
    let notice_sha = hash(&notice, end)?;
    let mut cases = Vec::new();
    let mut failure_phase = "preflight";
    let mut failure_kind = None;
    let mut actual_launch = false;
    let mut launch_failure = None;
    let mut wait_failure = None;
    let mut boundary_observation = serde_json::json!({});
    let operation = (|| -> io::Result<()> {
        let h = helper.as_ref().ok_or_else(bad)?;
        let specs = [("natural", h), ("cancel", h), ("deadline", h)];
        for (name, (source, expected_sha)) in specs {
            failure_phase = name;
            let input = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(source)?;
            let source_id = file_identity::from_file(&input)?;
            let source_bytes = input.metadata()?.len();
            if source_bytes > 128 * 1024 * 1024 || hash(source, end)? != *expected_sha {
                return Err(bad());
            }
            (&input).seek(SeekFrom::Start(17))?;
            let notice_input = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(&notice)?;
            let manifest = BundleManifest {
                schema: 1,
                version: "0.1.0".into(),
                target: if cfg!(target_env = "msvc") {
                    "x86_64-pc-windows-msvc"
                } else {
                    "x86_64-pc-windows-gnu"
                }
                .into(),
                development_only: true,
                entries: vec![
                    BundleEntry {
                        name: "dlpad.exe".into(),
                        bytes: source_bytes,
                        sha256: expected_sha.clone(),
                    },
                    BundleEntry {
                        name: "notice.txt".into(),
                        bytes: notice_bytes.len() as u64,
                        sha256: notice_sha.clone(),
                    },
                ],
            };
            let c = AtomicBool::new(false);
            let registered = root.join(format!("registered-{name}"));
            let receipt = root.join(format!("receipt-{name}"));
            let active = root.join(format!("active-{name}"));
            let prepared = prepare_new(
                &manifest,
                &[("dlpad.exe", &input), ("notice.txt", &notice_input)],
                &registered,
                &c,
                end,
            )
            .map_err(|e| io::Error::new(e.kind(), "fixture prepare failed"))?;
            let registration = register_fresh(prepared, &c, end)
                .map_err(|e| io::Error::new(e.kind(), "fixture registration failed"))?;
            let external = create_registration_receipt(&registration, &receipt, &c, end)
                .map_err(|e| io::Error::new(e.kind(), "fixture receipt failed"))?;
            drop(external);
            drop(registration);
            let pair = inspect_registered_with_receipt(&registered, &receipt, &c, end)?;
            let selected = select_registered_pair(pair, &active, &c, end)
                .map_err(|e| io::Error::new(e.kind(), "fixture selection failed"))?;
            drop(selected);
            let mut original = Vec::new();
            for leaf in [
                "dlpad.exe",
                "notice.txt",
                "install-ready.json",
                "install-registration.json",
                "install-registration-seal.json",
            ] {
                let p = registered.join(leaf);
                original.push((p.clone(), hash(&p, end)?));
            }
            for p in [
                receipt.join("registration-receipt.json"),
                active.join("active-selection.json"),
            ] {
                original.push((p.clone(), hash(&p, end)?));
            }
            let inspection = inspect_active_selection(&active, &registered, &receipt, &c, end)?;
            let selected_entry = inspection.record().selected_executable.clone();
            let canonical_sha = hash(&active.join("active-selection.json"), end)?;
            let launch_deadline = (Instant::now() + Duration::from_secs(90)).min(end);
            let mut process = launch_explicit_selection(inspection, &c, launch_deadline)
                .map_err(|e| {
                    actual_launch |= e.may_have_executed();
                    launch_failure = Some(serde_json::json!({"kind":format!("{:?}",e.kind()),"child_created":e.child_created(),"child_resumed":e.child_resumed(),"resume_attempted":e.resume_attempted(),"may_have_executed":e.may_have_executed(),"terminal_confirmed":e.terminal_confirmed(),"cleanup_unconfirmed":e.cleanup_unconfirmed()}));
                    io::Error::new(e.kind(), "fixture selected launch failed")
                })?;
            actual_launch = true;
            let creation = process.creation_filetime();
            let resume_attempted = process.resume_attempted();
            let may_run = process.may_have_executed();
            // Read-only observation of the original leader; this HANDLE grants no Job authority.
            use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
            use windows_sys::Win32::System::Threading::{
                GetProcessTimes, OpenProcess, WaitForSingleObject,
            };
            failure_phase = "leader-open-readonly";
            let handle = unsafe { OpenProcess(0x00101000, 0, process.process_id()?) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let leader = unsafe { OwnedHandle::from_raw_handle(handle) };
            let mut start = windows_sys::Win32::Foundation::FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            };
            let mut exit = start;
            let mut kernel = start;
            let mut user = start;
            failure_phase = "leader-times";
            let times_ok = unsafe {
                GetProcessTimes(
                    leader.as_raw_handle(),
                    &mut start,
                    &mut exit,
                    &mut kernel,
                    &mut user,
                )
            };
            let times_error = if times_ok == 0 {
                Some(io::Error::last_os_error().raw_os_error())
            } else {
                None
            };
            let observed_creation =
                u64::from(start.dwHighDateTime) << 32 | u64::from(start.dwLowDateTime);
            boundary_observation = serde_json::json!({"case":name,"times_api_succeeded":times_ok!=0,"times_os_error":times_error,"observed_creation":observed_creation,"expected_creation":creation});
            if times_ok == 0 {
                return Err(bad());
            }
            failure_phase = "leader-creation";
            if observed_creation != creation {
                return Err(bad());
            }
            failure_phase = "leader-terminal-wait";
            let leader_wait_began = Instant::now();
            let leader_wait = unsafe { WaitForSingleObject(leader.as_raw_handle(), 2000) };
            let wait_os_error = if leader_wait == u32::MAX {
                io::Error::last_os_error().raw_os_error()
            } else {
                None
            };
            boundary_observation["leader_wait_state"] = leader_wait.into();
            boundary_observation["leader_wait_elapsed_ms"] =
                serde_json::json!(leader_wait_began.elapsed().as_millis());
            boundary_observation["leader_wait_os_error"] = serde_json::json!(wait_os_error);
            if leader_wait != 0 {
                return Err(bad());
            }
            failure_phase = "job-active-query";
            let actual_descendant_count = process.active_process_count()?;
            boundary_observation["actual_job_active_after_leader_terminal"] =
                actual_descendant_count.into();
            failure_phase = "job-active-nonzero";
            if actual_descendant_count < 1 {
                return Err(bad());
            }
            let wait_began = Instant::now();
            let wait_cancel = AtomicBool::new(name == "cancel");
            let wait_deadline = if name == "deadline" {
                Instant::now()
            } else {
                launch_deadline
            };
            failure_phase = "selected-wait";
            let waited = if name == "deadline" {
                process.wait(&wait_cancel, wait_deadline)
            } else {
                process.wait_cancelled(&wait_cancel)
            };
            let outcome = waited.map_err(|e| {
                    wait_failure = Some(serde_json::json!({"kind":format!("{:?}",e.kind()),"terminal_unconfirmed":e.terminal_unconfirmed(),"original_creation_filetime":creation,"resume_attempted":resume_attempted,"may_have_executed":may_run}));
                    io::Error::new(e.kind(), "fixture terminal unconfirmed")
                })?;
            let expected_reason = match name {
                "natural" => SelectedExitReason::Natural,
                "cancel" => SelectedExitReason::Cancelled,
                _ => SelectedExitReason::TimedOut,
            };
            boundary_observation["outcome"] = serde_json::json!({"reason":format!("{:?}",outcome.reason),"code":outcome.code,"job_terminal_confirmed":outcome.job_terminal_confirmed});
            failure_phase = "outcome-exact";
            if outcome.reason != expected_reason
                || !outcome.job_terminal_confirmed
                || creation == 0
                || !resume_attempted
                || !may_run
                || (name == "natural" && outcome.code != 17)
            {
                return Err(bad());
            }
            let descendant_wait_ms = wait_began.elapsed().as_millis();
            boundary_observation["descendant_wait_elapsed_ms"] =
                serde_json::json!(descendant_wait_ms);
            failure_phase = "natural-descendant-duration";
            if name == "natural" && descendant_wait_ms < 1000 {
                return Err(bad());
            }
            drop(leader);
            drop(process);
            failure_phase = "source-preservation";
            if (&input).stream_position()? != 17
                || file_identity::from_file(&input)? != source_id
                || hash(source, end)? != *expected_sha
                || hash(&notice, end)? != notice_sha
                || hash(&sentinel, end)? != sentinel_sha
            {
                return Err(bad());
            }
            for (p, sha) in &original {
                if hash(p, end)? != *sha {
                    return Err(bad());
                }
            }
            if std::fs::read_dir(&registered)?.take(6).count() != 5
                || std::fs::read_dir(&receipt)?.take(2).count() != 1
                || std::fs::read_dir(&active)?.take(2).count() != 1
            {
                return Err(bad());
            }
            inspect_active_selection(&active, &registered, &receipt, &c, end)?
                .verify_current(&c, end)?;
            cases.push(serde_json::json!({"case":name,"leader_terminal_observed_before_wait":true,"actual_job_active_after_leader_terminal":actual_descendant_count,"descendant_wait_elapsed_ms":descendant_wait_ms,"helper_child_hold_ms":10000,"original_creation_filetime":creation,"resume_attempted":resume_attempted,"may_have_executed":may_run,"reason":format!("{:?}",outcome.reason),"exit_code":outcome.code,"job_terminal_confirmed":outcome.job_terminal_confirmed,"actual_child_image_verified_by_runtime":true,"selected_executable":selected_entry,"source_sha256":expected_sha,"source_bytes":source_bytes,"source_identity":source_id,"source_cursor_before":17,"source_cursor_after":17,"active_canonical_sha256":canonical_sha,"original_preservation_pins":original,"original_files_preserved":true,"actual_payload_work_verified":false,"actual_gui_verified":false}));
        }
        Ok(())
    })();
    if let Err(e) = operation {
        failure_kind = Some(format!("{:?}", e.kind()));
    }
    let status = if failure_kind.is_none() {
        "PASS_SELECTED_PROCESS_SCOPED"
    } else {
        "FAIL"
    };
    let output = serde_json::json!({"schema":1,"status":status,"owned_fixture_root":root,"cases":cases,"failure_phase":failure_kind.as_ref().map(|_|failure_phase),"failure_kind":failure_kind,"natural_helper_supplied":helper.is_some(),"all_three_cases_verified":helper.is_some()&&status=="PASS_SELECTED_PROCESS_SCOPED","private_desktop_required":true,"sentinel_sha256":sentinel_sha,"notice_sha256":notice_sha,"actual_launch":actual_launch,"launch_failure":launch_failure,"wait_failure":wait_failure,"boundary_observation":boundary_observation,"actual_gui_verified":false,"actual_payload_work_verified":false,"startup_shared_deadline_stall_verified":false,"publisher_trust":false,"installer_complete":false,"historical_ownership_verified":false,"deletion_authority":false});
    let raw = serde_json::to_string(&output).map_err(|_| bad())?;
    if raw.len() > 16384 {
        return Err(bad());
    }
    println!("{raw}");
    if status == "FAIL" {
        return Err(bad());
    }
    Ok(())
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("selected-process-fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(2);
    }
}
