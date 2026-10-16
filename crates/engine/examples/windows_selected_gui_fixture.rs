//! Explicit selected GUI fixture producer; only root private native host may run.
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
        os::windows::{ffi::OsStrExt, fs::OpenOptionsExt},
        path::{Component, Path, PathBuf, Prefix},
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

    fn absolute(value: &std::ffi::OsStr) -> io::Result<PathBuf> {
        let p = PathBuf::from(value);
        let units: Vec<_> = value.encode_wide().take(32768).collect();
        if !p.is_absolute()
            || units.len() > 32766
            || units.contains(&0)
            || p.components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || !matches!(p.components().next(),Some(Component::Prefix(v))if matches!(v.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(bad());
        }
        Ok(p)
    }
    fn directory(path: &Path) -> io::Result<File> {
        let f = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: retained native directory capability and matching bounded output.
        if unsafe {
            GetFileInformationByHandleEx(
                std::os::windows::io::AsRawHandle::as_raw_handle(&f),
                FileAttributeTagInfo,
                (&mut attrs as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of_val(&attrs) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if attrs.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || attrs.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(bad());
        }
        Ok(f)
    }
    fn emit(value: &serde_json::Value) -> io::Result<()> {
        let raw = serde_json::to_string(value).map_err(|_| bad())?;
        if raw.len() > 16384 {
            return Err(bad());
        }
        let mut out = io::stdout().lock();
        out.write_all(raw.as_bytes())?;
        out.write_all(b"\n")?;
        out.flush()
    }
    let args: Vec<_> = std::env::args_os().take(7).collect();
    let fresh = args.len() == 4 && args[1] == "--selected-gui-new";
    let restart = args.len() == 6 && args[1] == "--selected-gui-restart";
    if !fresh && !restart {
        return Err(bad());
    }
    let temp_parent = std::env::temp_dir().canonicalize()?;
    let end = Instant::now() + Duration::from_secs(180);
    let root = if fresh {
        tempfile::Builder::new()
            .prefix("dlpad-selected-gui-owned-")
            .tempdir()?
            .keep()
    } else {
        absolute(&args[5])?
    };
    if !root
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with("dlpad-selected-gui-owned-"))
        || root.parent().ok_or_else(bad)?.canonicalize()? != temp_parent
    {
        return Err(bad());
    }
    let root_pin = directory(&root)?;
    let root_id = file_identity::from_file(&root_pin)?;
    let data = root.join("data");
    let media = root.join("media");
    let tmp = root.join("tmp");
    if fresh {
        for p in [&data, &media, &tmp] {
            std::fs::create_dir(p)?;
        }
        let mut theme = File::create_new(data.join("theme.json"))?;
        theme.write_all(br#""System""#)?;
        theme.sync_all()?;
    }
    let mut env_pins = Vec::new();
    for p in [&data, &media, &tmp] {
        env_pins.push(directory(p)?);
    }
    // SAFETY: single-threaded example; set process-only synthetic roots once before
    // any launch, no user settings or machine/global environment modifications.
    unsafe {
        std::env::set_var("DLPAD_TEST_DATA", &data);
        std::env::set_var("DLPAD_LIBRARY_ROOT", &media);
        std::env::set_var("TMP", &tmp);
        std::env::set_var("TEMP", &tmp);
    }
    let registered = root.join("registered");
    let receipt = root.join("receipt");
    let active = root.join("active");
    let notice = root.join("notice.txt");
    let sentinel = root.join("foreign-sentinel");
    let mut source_input = None;
    let mut source_path = None;
    let mut source_sha = None;
    let mut source_id = None;
    let mut source_bytes = None;
    let c = AtomicBool::new(false);
    if fresh {
        let source = absolute(&args[2])?;
        let expected = args[3].to_str().ok_or_else(bad)?;
        if expected.len() != 64
            || !expected
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(bad());
        }
        let m = std::fs::symlink_metadata(&source)?;
        if !m.is_file() || m.file_type().is_symlink() {
            return Err(bad());
        }
        let input = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&source)?;
        let bytes = input.metadata()?.len();
        if bytes > 128 * 1024 * 1024 || hash(&source, end)? != expected {
            return Err(bad());
        }
        let notice_bytes = b"synthetic local CLI notice; no legal coverage claim\n";
        for (p, b) in [
            (&notice, notice_bytes.as_slice()),
            (
                &sentinel,
                b"owned synthetic selected GUI preserve".as_slice(),
            ),
        ] {
            let mut f = File::create_new(p)?;
            f.write_all(b)?;
            f.sync_all()?;
        }
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
                    bytes,
                    sha256: expected.into(),
                },
                BundleEntry {
                    name: "notice.txt".into(),
                    bytes: notice_bytes.len() as u64,
                    sha256: hash(&notice, end)?,
                },
            ],
        };
        (&input).seek(SeekFrom::Start(17))?;
        let prepared = prepare_new(
            &manifest,
            &[("dlpad.exe", &input), ("notice.txt", &notice_input)],
            &registered,
            &c,
            end,
        )
        .map_err(|e| io::Error::new(e.kind(), "GUI fixture prepare failed"))?;
        let registration = register_fresh(prepared, &c, end)
            .map_err(|e| io::Error::new(e.kind(), "GUI fixture register failed"))?;
        let external = create_registration_receipt(&registration, &receipt, &c, end)
            .map_err(|e| io::Error::new(e.kind(), "GUI fixture receipt failed"))?;
        drop(external);
        drop(registration);
        let pair = inspect_registered_with_receipt(&registered, &receipt, &c, end)?;
        let selected = select_registered_pair(pair, &active, &c, end)
            .map_err(|e| io::Error::new(e.kind(), "GUI fixture selection failed"))?;
        drop(selected);
        source_id = Some(file_identity::from_file(&input)?);
        source_path = Some(source);
        source_sha = Some(expected.to_owned());
        source_bytes = Some(bytes);
        source_input = Some(input);
    } else {
        for (arg, known) in [
            (&args[2], &active),
            (&args[3], &registered),
            (&args[4], &receipt),
        ] {
            let explicit = absolute(arg)?;
            let _pin = directory(&explicit)?;
            if explicit.canonicalize()? != known.canonicalize()? {
                return Err(bad());
            }
        }
    }
    let notice_sha = hash(&notice, end)?;
    let sentinel_sha = hash(&sentinel, end)?;
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
    let selected = inspection.record().selected_executable.clone();
    let active_sha = hash(&active.join("active-selection.json"), end)?;
    let mut process = launch_explicit_selection(inspection, &c, end)
        .map_err(|e| io::Error::new(e.kind(), "GUI fixture launch failed"))?;
    let pid = process.process_id()?;
    let creation = process.creation_filetime();
    let ready = serde_json::json!({"schema":1,"phase":"selected-gui-ready","mode":if fresh{"new"}else{"restart"},"child_pid":pid,"child_creation_filetime":creation,"owned_root":root,"owned_root_identity":root_id,"data_root":data,"registration_path":registered,"receipt_path":receipt,"active_path":active,"selected_executable":registered.join("dlpad.exe"),"selected_executable_descriptor":selected,"active_sha256":active_sha,"original_preservation_pins":original,"source_path":source_path,"source_sha256":source_sha,"source_bytes":source_bytes,"source_identity":source_id,"source_cursor_before":if fresh{Some(17)}else{None},"sentinel_sha256":sentinel_sha,"notice_sha256":notice_sha,"actual_child_image_verified_by_runtime":true,"resume_attempted":process.resume_attempted(),"may_have_executed":process.may_have_executed(),"argv_override_count":0,"inherited_handles":0,"publisher_trust":false,"installer_complete":false});
    emit(&ready)?;
    let terminal = process.wait(&c, end);
    let outcome = match terminal {
        Ok(v) => v,
        Err(e) => {
            emit(
                &serde_json::json!({"schema":1,"phase":"selected-gui-terminal","status":"FAIL","kind":format!("{:?}",e.kind()),"terminal_unconfirmed":e.terminal_unconfirmed(),"child_pid":pid,"child_creation_filetime":creation}),
            )?;
            return Err(io::Error::new(e.kind(), "GUI fixture terminal unconfirmed"));
        }
    };
    if outcome.reason != SelectedExitReason::Natural
        || outcome.code != 0
        || !outcome.job_terminal_confirmed
    {
        emit(
            &serde_json::json!({"schema":1,"phase":"selected-gui-terminal","status":"FAIL","reason":format!("{:?}",outcome.reason),"exit_code":outcome.code,"job_terminal_confirmed":outcome.job_terminal_confirmed}),
        )?;
        return Err(bad());
    }
    drop(process);
    let verify = (|| -> io::Result<()> {
        for (p, sha) in &original {
            if hash(p, end)? != *sha {
                return Err(bad());
            }
        }
        if hash(&notice, end)? != notice_sha
            || hash(&sentinel, end)? != sentinel_sha
            || file_identity::from_file(&root_pin)? != root_id
            || file_identity::from_file(&directory(&root)?)? != root_id
        {
            return Err(bad());
        }
        let mut root_names = Vec::new();
        for entry in std::fs::read_dir(&root)?.take(9) {
            root_names.push(entry?.file_name());
        }
        root_names.sort();
        let mut expected_names = [
            "active",
            "data",
            "foreign-sentinel",
            "media",
            "notice.txt",
            "receipt",
            "registered",
            "tmp",
        ]
        .map(std::ffi::OsString::from);
        expected_names.sort();
        if root_names.as_slice() != expected_names {
            return Err(bad());
        }
        if std::fs::read_dir(&registered)?.take(6).count() != 5
            || std::fs::read_dir(&receipt)?.take(2).count() != 1
            || std::fs::read_dir(&active)?.take(2).count() != 1
        {
            return Err(bad());
        }
        inspect_active_selection(&active, &registered, &receipt, &c, end)?
            .verify_current(&c, end)?;
        if let Some(input) = &source_input
            && ((&*input).stream_position()? != 17
                || Some(file_identity::from_file(input)?) != source_id
                || hash(source_path.as_ref().ok_or_else(bad)?, end)?
                    != *source_sha.as_ref().ok_or_else(bad)?)
        {
            return Err(bad());
        }
        Ok(())
    })();
    let failed = verify.as_ref().err().map(|e| format!("{:?}", e.kind()));
    emit(
        &serde_json::json!({"schema":1,"phase":"selected-gui-terminal","status":if failed.is_none(){"PASS_SELECTED_GUI_PROCESS_ONLY"}else{"FAIL"},"kind":failed,"mode":if fresh{"new"}else{"restart"},"child_pid":pid,"child_creation_filetime":creation,"reason":format!("{:?}",outcome.reason),"exit_code":outcome.code,"job_terminal_confirmed":outcome.job_terminal_confirmed,"original_files_preserved":failed.is_none(),"original_preservation_pins":original,"active_sha256":active_sha,"owned_root_identity":root_id,"source_cursor_after":if fresh{Some(17)}else{None},"restart_source_cursor_verified":false,"sentinel_sha256":sentinel_sha,"notice_sha256":notice_sha,"actual_gui_verified":false,"theme_verified":false,"publisher_trust":false,"installer_complete":false,"historical_ownership_verified":false,"deletion_authority":false}),
    )?;
    verify
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = run() {
        eprintln!("selected-gui-fixture failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows-only fixture");
        std::process::exit(2);
    }
}
