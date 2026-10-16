//! Unsigned development-only local commands; removal requires an explicit paired factory.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug)]
enum Command {
    RemoveUserBundle {
        bundle_root: PathBuf,
        journal_root: PathBuf,
    },
    JournalUserBundle {
        manifest: PathBuf,
        source_dir: PathBuf,
        bundle_root: PathBuf,
        journal_root: PathBuf,
    },
    InspectInstallJournal {
        journal_root: PathBuf,
        bundle_root: PathBuf,
    },
    InspectUserBundle {
        bundle_root: PathBuf,
    },
    UserBundle {
        manifest: PathBuf,
        source_dir: PathBuf,
        bundle_root: PathBuf,
    },
    Prepare {
        register: bool,
        receipt_target: Option<PathBuf>,
        manifest: PathBuf,
        payload: PathBuf,
        target: PathBuf,
    },
    Pair {
        remove: bool,
        target: PathBuf,
        receipt_target: PathBuf,
    },
    Active {
        inspect: bool,
        target: PathBuf,
        receipt_target: PathBuf,
        active_target: PathBuf,
    },
    LaunchSelected {
        run_until_cancelled: bool,
        target: PathBuf,
        receipt_target: PathBuf,
        active_target: PathBuf,
    },
    Inspect {
        registered: bool,
        target: PathBuf,
    },
}
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug)]
struct Args {
    command: Command,
    timeout: Duration,
}
fn rejected() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid local preparation arguments",
    )
}
fn absolute(s: &OsString) -> io::Result<PathBuf> {
    if s.as_encoded_bytes().len() > 131064 || s.to_string_lossy().chars().any(|c| c.is_control()) {
        return Err(rejected());
    }
    let p = PathBuf::from(s);
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::path::{Component, Prefix};
        let units: Vec<u16> = s.encode_wide().collect();
        if units.len() > 32767 || units.contains(&0) {
            return Err(rejected());
        }
        if !matches!(p.components().next(), Some(Component::Prefix(v)) if matches!(v.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err(rejected());
        }
    }
    if !p.is_absolute()
        || p.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(rejected());
    }
    Ok(p)
}
fn parse(args: &[OsString]) -> io::Result<Args> {
    if args.is_empty()
        || args.len() > 11
        || args.len().is_multiple_of(2)
        || args.iter().any(|a| {
            a.as_encoded_bytes().len() > 131064
                || a.to_string_lossy().chars().any(|c| c.is_control())
        })
    {
        return Err(rejected());
    }
    let command = args[0].to_str().ok_or_else(rejected)?;
    if !matches!(
        command,
        "prepare"
            | "inspect"
            | "prepare-register"
            | "inspect-registered"
            | "prepare-register-receipt"
            | "inspect-pair"
            | "remove-registered"
            | "select-active"
            | "inspect-active"
            | "launch-selected"
            | "run-selected"
            | "prepare-user-bundle"
            | "inspect-user-bundle"
            | "inspect-install-journal"
            | "prepare-journal-user-bundle"
            | "remove-user-bundle"
    ) {
        return Err(rejected());
    }
    let mut flags = BTreeMap::new();
    for pair in args[1..].chunks_exact(2) {
        let name = pair[0].to_str().ok_or_else(rejected)?;
        if !matches!(
            name,
            "--manifest"
                | "--payload-dir"
                | "--target"
                | "--receipt-target"
                | "--active-target"
                | "--timeout-seconds"
                | "--startup-timeout-seconds"
                | "--source-dir"
                | "--bundle-root"
                | "--journal-root"
        ) || flags.insert(name, &pair[1]).is_some()
        {
            return Err(rejected());
        }
    }
    let timeout_flag = if command == "run-selected" {
        "--startup-timeout-seconds"
    } else {
        "--timeout-seconds"
    };
    let timeout = match flags.remove(timeout_flag) {
        None => 300,
        Some(s) => {
            let s = s.to_str().ok_or_else(rejected)?;
            if s.is_empty() || s.len() > 20 || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(rejected());
            }
            let n = s.parse::<u64>().map_err(|_| rejected())?;
            if !(1..=600).contains(&n) {
                return Err(rejected());
            }
            n
        }
    };
    let target_key = if matches!(
        command,
        "prepare-user-bundle"
            | "inspect-user-bundle"
            | "inspect-install-journal"
            | "prepare-journal-user-bundle"
            | "remove-user-bundle"
    ) {
        "--bundle-root"
    } else {
        "--target"
    };
    let target = absolute(flags.remove(target_key).ok_or_else(rejected)?)?;
    let command = if command == "remove-user-bundle" {
        Command::RemoveUserBundle {
            bundle_root: target,
            journal_root: absolute(flags.remove("--journal-root").ok_or_else(rejected)?)?,
        }
    } else if command == "prepare-journal-user-bundle" {
        Command::JournalUserBundle {
            manifest: absolute(flags.remove("--manifest").ok_or_else(rejected)?)?,
            source_dir: absolute(flags.remove("--source-dir").ok_or_else(rejected)?)?,
            bundle_root: target,
            journal_root: absolute(flags.remove("--journal-root").ok_or_else(rejected)?)?,
        }
    } else if command == "inspect-install-journal" {
        Command::InspectInstallJournal {
            journal_root: absolute(flags.remove("--journal-root").ok_or_else(rejected)?)?,
            bundle_root: target,
        }
    } else if command == "inspect-user-bundle" {
        Command::InspectUserBundle {
            bundle_root: target,
        }
    } else if command == "prepare-user-bundle" {
        Command::UserBundle {
            manifest: absolute(flags.remove("--manifest").ok_or_else(rejected)?)?,
            source_dir: absolute(flags.remove("--source-dir").ok_or_else(rejected)?)?,
            bundle_root: target,
        }
    } else if matches!(
        command,
        "prepare" | "prepare-register" | "prepare-register-receipt"
    ) {
        Command::Prepare {
            register: command != "prepare",
            receipt_target: if command == "prepare-register-receipt" {
                Some(absolute(
                    flags.remove("--receipt-target").ok_or_else(rejected)?,
                )?)
            } else {
                None
            },
            manifest: absolute(flags.remove("--manifest").ok_or_else(rejected)?)?,
            payload: absolute(flags.remove("--payload-dir").ok_or_else(rejected)?)?,
            target,
        }
    } else if matches!(command, "launch-selected" | "run-selected") {
        Command::LaunchSelected {
            run_until_cancelled: command == "run-selected",
            target,
            receipt_target: absolute(flags.remove("--receipt-target").ok_or_else(rejected)?)?,
            active_target: absolute(flags.remove("--active-target").ok_or_else(rejected)?)?,
        }
    } else if matches!(command, "select-active" | "inspect-active") {
        Command::Active {
            inspect: command == "inspect-active",
            target,
            receipt_target: absolute(flags.remove("--receipt-target").ok_or_else(rejected)?)?,
            active_target: absolute(flags.remove("--active-target").ok_or_else(rejected)?)?,
        }
    } else if matches!(command, "inspect-pair" | "remove-registered") {
        Command::Pair {
            remove: command == "remove-registered",
            target,
            receipt_target: absolute(flags.remove("--receipt-target").ok_or_else(rejected)?)?,
        }
    } else {
        Command::Inspect {
            registered: command == "inspect-registered",
            target,
        }
    };
    if !flags.is_empty() {
        return Err(rejected());
    }
    Ok(Args {
        command,
        timeout: Duration::from_secs(timeout),
    })
}
#[cfg(windows)]
mod native {
    use super::*;
    use dlpad_engine::{
        app_install::{
            SelectedExitReason, authorize_explicit_registered_removal,
            authorize_explicit_user_bundle_removal, create_fresh_user_bundle,
            create_journal_aware_bundle, create_registration_receipt, inspect_active_selection,
            inspect_prepared, inspect_registered, inspect_registered_with_receipt,
            launch_explicit_selection, manifest::BundleManifest, payload::inspect_payload,
            prepare_new, register_fresh, select_registered_pair,
        },
        file_identity::{self, NativeFileIdentity},
    };
    use std::{
        fs::{File, OpenOptions},
        os::windows::{
            ffi::OsStrExt,
            fs::OpenOptionsExt,
            io::{AsRawHandle, FromRawHandle},
        },
        path::Path,
        sync::atomic::{AtomicBool, Ordering},
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, OBJ_CASE_INSENSITIVE, RtlNtStatusToDosError, UNICODE_STRING},
        Storage::FileSystem::*,
        System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler},
    };
    use windows_sys::{
        Wdk::{
            Foundation::OBJECT_ATTRIBUTES,
            Storage::FileSystem::{
                FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT,
                FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
            },
        },
        Win32::System::IO::IO_STATUS_BLOCK,
        core::BOOL,
    };
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CompareStringOrdinal(
            a: *const u16,
            al: i32,
            b: *const u16,
            bl: i32,
            ignore: BOOL,
        ) -> i32;
    }
    // Compare original native final paths, with an explicit component boundary.
    fn contains(parent: &Path, child: &Path) -> bool {
        let a: Vec<u16> = parent.as_os_str().encode_wide().collect();
        let b: Vec<u16> = child.as_os_str().encode_wide().collect();
        a.len() <= b.len()
            && unsafe {
                CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1)
            } == 2
            && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92))
    }
    static CANCEL: AtomicBool = AtomicBool::new(false);
    unsafe extern "system" fn handler(event: u32) -> BOOL {
        if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
            CANCEL.store(true, Ordering::Release);
            1
        } else {
            0
        }
    }
    struct Handler;
    impl Drop for Handler {
        fn drop(&mut self) {
            unsafe {
                SetConsoleCtrlHandler(Some(handler), 0);
            }
        }
    }
    fn check(deadline: Instant) -> io::Result<()> {
        if CANCEL.load(Ordering::Acquire) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "cancelled"));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "deadline"));
        }
        Ok(())
    }
    struct Pin {
        path: PathBuf,
        file: File,
        id: NativeFileIdentity,
        final_path: PathBuf,
    }
    impl Pin {
        fn verify(&self) -> io::Result<()> {
            if file_identity::from_file(&self.file)? != self.id
                || file_identity::final_path(&self.file)? != self.final_path
            {
                return Err(rejected());
            }
            let current = directory(&self.path)?;
            if file_identity::from_file(&current)? != self.id
                || file_identity::final_path(&current)? != self.final_path
            {
                return Err(rejected());
            }
            Ok(())
        }
    }
    fn directory(p: &Path) -> io::Result<File> {
        let f = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(p)?;
        let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
        if unsafe {
            GetFileInformationByHandleEx(
                f.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        {
            return Err(rejected());
        }
        Ok(f)
    }
    fn ancestors(path: &Path, deadline: Instant) -> io::Result<Vec<Pin>> {
        let paths: Vec<_> = path.ancestors().collect();
        if paths.len() > 256 {
            return Err(rejected());
        }
        let mut pins = Vec::new();
        for p in paths.into_iter().rev() {
            check(deadline)?;
            let f = directory(p)?;
            let id = file_identity::from_file(&f)?;
            let final_path = file_identity::final_path(&f)?;
            pins.push(Pin {
                path: p.to_owned(),
                file: f,
                id,
                final_path,
            });
        }
        Ok(pins)
    }
    fn source(path: &Path, pins: &[Pin], deadline: Instant) -> io::Result<File> {
        check(deadline)?;
        for pin in pins {
            pin.verify()?;
        }
        let parent = pins.last().ok_or_else(rejected)?;
        let leaf = path.file_name().ok_or_else(rejected)?;
        let mut units: Vec<u16> = leaf.encode_wide().collect();
        if units.is_empty()
            || units.len() > 32766
            || units
                .iter()
                .any(|v| *v < 32 || *v == 127 || matches!(*v, 58 | 47 | 92))
            || matches!(units.last(), Some(32 | 46))
        {
            return Err(rejected());
        }
        let length = u16::try_from(units.len() * 2).map_err(|_| rejected())?;
        let mut name = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: units.as_mut_ptr(),
        };
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent.file.as_raw_handle(),
            ObjectName: &mut name,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                GENERIC_READ | SYNCHRONIZE,
                &attrs,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ,
                FILE_OPEN,
                FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if result < 0 {
            return Err(io::Error::from_raw_os_error(
                unsafe { RtlNtStatusToDosError(result) } as i32,
            ));
        }
        if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(rejected());
        }
        let file = unsafe { File::from_raw_handle(raw) };
        let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if tag.FileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
            || !file.metadata()?.is_file()
        {
            return Err(rejected());
        }
        let parent = pins.last().ok_or_else(rejected)?;
        if file_identity::final_path(&file)?
            != parent
                .final_path
                .join(path.file_name().ok_or_else(rejected)?)
        {
            return Err(rejected());
        }
        for pin in pins {
            pin.verify()?;
        }
        Ok(file)
    }
    // A default-OFF test variant acknowledges the original inherited event only
    // after parser/deadline/console handler setup and before pair inspection.
    #[cfg(feature = "active-selection-control-fixture")]
    fn active_control_gate(deadline: Instant) -> io::Result<()> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::{
            Foundation::{
                GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, WAIT_OBJECT_0, WAIT_TIMEOUT,
            },
            System::Threading::{SetEvent, WaitForSingleObject},
        };
        fn inherited(name: &str) -> io::Result<HANDLE> {
            let text = std::env::var(name).map_err(|_| rejected())?;
            if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err(rejected());
            }
            let raw = text.parse::<usize>().map_err(|_| rejected())?;
            if raw == 0 || raw == usize::MAX {
                return Err(rejected());
            }
            let handle = raw as HANDLE;
            let mut flags = 0;
            // SAFETY: only queries the child-local handle table, no ownership is inferred on failure.
            if unsafe { GetHandleInformation(handle, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT == 0
            {
                return Err(rejected());
            }
            Ok(handle)
        }
        let ready = inherited("DLPAD_ACTIVE_CONTROL_READY_HANDLE")?;
        let release = inherited("DLPAD_ACTIVE_CONTROL_RELEASE_HANDLE")?;
        if ready == release {
            return Err(rejected());
        }
        // SAFETY: this test-only process takes its two distinct inherited handles.
        // Parent originals live in the parent table and are never closed here.
        let ready = unsafe { OwnedHandle::from_raw_handle(ready) };
        let release = unsafe { OwnedHandle::from_raw_handle(release) };
        check(deadline)?;
        if unsafe { SetEvent(ready.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        loop {
            check(deadline)?;
            match unsafe { WaitForSingleObject(release.as_raw_handle(), 20) } {
                WAIT_OBJECT_0 => return check(deadline),
                WAIT_TIMEOUT => (),
                _ => return Err(io::Error::last_os_error()),
            }
        }
    }
    pub fn run(args: Args, began: Instant) -> io::Result<serde_json::Value> {
        let deadline = began.checked_add(args.timeout).ok_or_else(rejected)?;
        check(deadline)?;
        if unsafe { SetConsoleCtrlHandler(Some(handler), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let _handler = Handler;
        match args.command {
            Command::RemoveUserBundle {
                bundle_root,
                journal_root,
            } => {
                let failure = |error: &dlpad_engine::app_install::BundleRemovalFailure,
                               phase: &str| {
                    serde_json::json!({"status":"FAIL","phase":phase,"kind":format!("{:?}",error.kind()),"mutation_may_have_occurred":error.mutation_may_have_occurred(),"accepted_file_dispositions":error.accepted_file_dispositions(),"accepted_root_dispositions":error.accepted_root_dispositions(),"retained_delete_file_caps_before_drop":error.retained_file_cap_count(),"retained_delete_root_caps_before_drop":error.retained_root_cap_count(),"retained_unconverted_bundle_read_root_caps_before_drop":error.retained_unconverted_bundle_read_root_cap_count(),"original_read_bundle_retained_before_drop":error.original_read_bundle_retained(),"bundle_root_disposition_accepted":error.accepted_root_dispositions()==4,"outer_namespace_absence_confirmed":false})
                };
                let authorized = match authorize_explicit_user_bundle_removal(
                    &bundle_root,
                    &journal_root,
                    &CANCEL,
                    deadline,
                ) {
                    Ok(value) => value,
                    Err(error) => return Ok(failure(&error, "bundle-remove-authorize")),
                };
                match authorized.remove(&CANCEL, deadline) {
                    Err(error) => Ok(failure(&error, "bundle-remove")),
                    Ok(outcome) => {
                        let success = outcome.bundle_root_disposition_accepted
                            && outcome.outer_namespace_absence_confirmed;
                        Ok(
                            serde_json::json!({"status":if success {"REMOVED_USER_BUNDLE_DEVELOPMENT_ONLY"} else {"FAIL"},"phase":"bundle-remove-complete","kind":if success {None} else {Some("Other")},"mutation_may_have_occurred":true,"accepted_file_dispositions":outcome.accepted_file_dispositions,"accepted_root_dispositions":outcome.accepted_root_dispositions,"bundle_root_disposition_accepted":outcome.bundle_root_disposition_accepted,"outer_namespace_absence_confirmed":outcome.outer_namespace_absence_confirmed,"retained_delete_file_caps_before_drop":0,"retained_delete_root_caps_before_drop":0,"retained_unconverted_bundle_read_root_caps_before_drop":null,"original_read_bundle_retained_before_drop":false}),
                        )
                    }
                }
            }
            Command::InspectInstallJournal {
                journal_root,
                bundle_root,
            } => {
                let checked = (|| {
                    let journal = dlpad_engine::app_install::inspect_install_journal(
                        &journal_root,
                        &CANCEL,
                        deadline,
                    )?;
                    let bundle = dlpad_engine::app_install::inspect_user_bundle(
                        &bundle_root,
                        &CANCEL,
                        deadline,
                    )?;
                    journal.verify_active_bundle(&bundle, &CANCEL, deadline)
                })();
                Ok(match checked {
                    Ok(()) => {
                        serde_json::json!({"status":"INSPECTED_INSTALL_JOURNAL_DEVELOPMENT_ONLY","phase":"inspect-install-journal","current_consistency_verified":true})
                    }
                    Err(error) => {
                        serde_json::json!({"status":"FAIL","phase":"inspect-install-journal","kind":format!("{:?}",error.kind()),"current_consistency_verified":false})
                    }
                })
            }
            Command::InspectUserBundle { bundle_root } => {
                let inspected = dlpad_engine::app_install::inspect_user_bundle(
                    &bundle_root,
                    &CANCEL,
                    deadline,
                )?;
                inspected.verify_current(&CANCEL, deadline)?;
                Ok(
                    serde_json::json!({"status":"INSPECTED_USER_BUNDLE_DEVELOPMENT_ONLY","phase":"complete","mutation_may_have_occurred":false,"bundle_namespace_verified":true,"active_selection_verified":true,"pair_verified":true,"registration_verified":true,"receipt_verified":true,"trusted_provenance":false,"installer_complete":false,"upgrade_verified":false,"deletion_authority":false}),
                )
            }
            Command::JournalUserBundle {
                manifest,
                source_dir,
                bundle_root,
                journal_root,
            } => {
                let mp = ancestors(manifest.parent().ok_or_else(rejected)?, deadline)?;
                let mf = source(&manifest, &mp, deadline)?;
                let m = BundleManifest::read(&mf)?;
                if !m.development_only {
                    return Err(rejected());
                }
                let pp = ancestors(&source_dir, deadline)?;
                let tp = ancestors(bundle_root.parent().ok_or_else(rejected)?, deadline)?;
                let jp = ancestors(journal_root.parent().ok_or_else(rejected)?, deadline)?;
                let journal_dest = jp
                    .last()
                    .ok_or_else(rejected)?
                    .final_path
                    .join(journal_root.file_name().ok_or_else(rejected)?);
                let dest = tp
                    .last()
                    .ok_or_else(rejected)?
                    .final_path
                    .join(bundle_root.file_name().ok_or_else(rejected)?);
                let src = &pp.last().ok_or_else(rejected)?.final_path;
                let manifest_final = file_identity::final_path(&mf)?;
                if contains(src, &dest) || contains(&dest, src) || contains(&dest, &manifest_final)
                {
                    return Err(rejected());
                }
                if contains(src, &journal_dest)
                    || contains(&journal_dest, src)
                    || contains(&journal_dest, &manifest_final)
                    || contains(&journal_dest, &dest)
                    || contains(&dest, &journal_dest)
                {
                    return Err(rejected());
                }
                let manifest_id = file_identity::from_file(&mf)?;
                let mut files = Vec::with_capacity(m.entries.len());
                for entry in &m.entries {
                    check(deadline)?;
                    files.push((
                        entry.name.as_str(),
                        source(&source_dir.join(&entry.name), &pp, deadline)?,
                    ));
                }
                let refs: Vec<_> = files.iter().map(|(n, f)| (*n, f)).collect();
                let originals = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                let verify_sources = || -> io::Result<()> {
                    for pin in mp.iter().chain(pp.iter()).chain(tp.iter()).chain(jp.iter()) {
                        check(deadline)?;
                        pin.verify()?;
                    }
                    if file_identity::from_file(&mf)? != manifest_id
                        || file_identity::final_path(&mf)? != manifest_final
                    {
                        return Err(rejected());
                    }
                    let current = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                    if originals.len() != current.len() {
                        return Err(rejected());
                    }
                    for original in &originals {
                        let now = current
                            .iter()
                            .find(|x| x.name == original.name)
                            .ok_or_else(rejected)?;
                        if now.identity != original.identity
                            || now.bytes != original.bytes
                            || now.sha256 != original.sha256
                        {
                            return Err(rejected());
                        }
                        let (_, file) = refs
                            .iter()
                            .find(|(n, _)| *n == original.name)
                            .ok_or_else(rejected)?;
                        if file_identity::final_path(file)? != src.join(&original.name) {
                            return Err(rejected());
                        }
                    }
                    check(deadline)
                };
                verify_sources()?;
                let bundle = match create_journal_aware_bundle(
                    &m,
                    &refs,
                    &bundle_root,
                    &journal_root,
                    &CANCEL,
                    deadline,
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"journal-user-bundle","journal_bundle_stage":null,"journal_bundle_stage_reported":false,"kind":format!("{:?}",error.kind()),"mutation_may_have_occurred":error.mutation_may_have_occurred(),"bundle_capabilities_present_before_drop":error.bundle_capabilities_present(),"journal_capabilities_present_before_drop":error.journal_capabilities_present(),"retained_file_caps":null,"retained_file_cap_count_reported":false,"permanent_detach":false,"journal_recovery_verified":false}),
                        );
                    }
                };
                if let Err(error) = bundle
                    .verify_current(&CANCEL, deadline)
                    .and_then(|()| verify_sources())
                {
                    return Ok(
                        serde_json::json!({"status":"FAIL","phase":"journal-user-bundle-postverify","kind":format!("{:?}",error.kind()),"mutation_may_have_occurred":true,"bundle_capabilities_present_before_drop":true,"journal_capabilities_present_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false,"permanent_detach":false,"journal_recovery_verified":false}),
                    );
                }
                Ok(
                    serde_json::json!({"status":"PREPARED_JOURNAL_USER_BUNDLE_DEVELOPMENT_ONLY","phase":"complete","mutation_may_have_occurred":true,"bundle_capabilities_present_before_drop":true,"journal_capabilities_present_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false,"registration_verified":true,"receipt_verified":true,"pair_verified":true,"active_selection_verified":true,"entries":bundle.bundle().registration().record().prepared.entries.len(),"permanent_detach":false,"journal_recovery_verified":false,"actual_launch":false,"current_consistency_verified":true,"four_verified_stage_records_observed":true,"manual_backfill_used":false}),
                )
            }
            Command::UserBundle {
                manifest,
                source_dir,
                bundle_root,
            } => {
                let mp = ancestors(manifest.parent().ok_or_else(rejected)?, deadline)?;
                let mf = source(&manifest, &mp, deadline)?;
                let m = BundleManifest::read(&mf)?;
                if !m.development_only {
                    return Err(rejected());
                }
                let pp = ancestors(&source_dir, deadline)?;
                let tp = ancestors(bundle_root.parent().ok_or_else(rejected)?, deadline)?;
                let dest = tp
                    .last()
                    .ok_or_else(rejected)?
                    .final_path
                    .join(bundle_root.file_name().ok_or_else(rejected)?);
                let src = &pp.last().ok_or_else(rejected)?.final_path;
                let manifest_final = file_identity::final_path(&mf)?;
                if contains(src, &dest) || contains(&dest, src) || contains(&dest, &manifest_final)
                {
                    return Err(rejected());
                }
                let manifest_id = file_identity::from_file(&mf)?;
                let mut files = Vec::with_capacity(m.entries.len());
                for entry in &m.entries {
                    check(deadline)?;
                    files.push((
                        entry.name.as_str(),
                        source(&source_dir.join(&entry.name), &pp, deadline)?,
                    ));
                }
                let refs: Vec<_> = files.iter().map(|(n, f)| (*n, f)).collect();
                let originals = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                let verify_sources = || -> io::Result<()> {
                    for pin in mp.iter().chain(pp.iter()).chain(tp.iter()) {
                        check(deadline)?;
                        pin.verify()?;
                    }
                    if file_identity::from_file(&mf)? != manifest_id
                        || file_identity::final_path(&mf)? != manifest_final
                    {
                        return Err(rejected());
                    }
                    let current = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                    if originals.len() != current.len() {
                        return Err(rejected());
                    }
                    for original in &originals {
                        let now = current
                            .iter()
                            .find(|x| x.name == original.name)
                            .ok_or_else(rejected)?;
                        if now.identity != original.identity
                            || now.bytes != original.bytes
                            || now.sha256 != original.sha256
                        {
                            return Err(rejected());
                        }
                        let (_, file) = refs
                            .iter()
                            .find(|(n, _)| *n == original.name)
                            .ok_or_else(rejected)?;
                        if file_identity::final_path(file)? != src.join(&original.name) {
                            return Err(rejected());
                        }
                    }
                    check(deadline)
                };
                verify_sources()?;
                let bundle = match create_fresh_user_bundle(
                    &m,
                    &refs,
                    &bundle_root,
                    &CANCEL,
                    deadline,
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"user-bundle","bundle_stage":error.stage(),"kind":format!("{:?}",error.kind()),"mutation_may_have_occurred":error.mutation_may_have_occurred(),"outer_capability_acquired":error.outer_capability_acquired(),"retained_capabilities_present_before_drop":error.retained_capabilities_present(),"retained_file_caps":null,"retained_file_cap_count_reported":false,"permanent_detach":false,"journal_recovery_verified":false}),
                        );
                    }
                };
                if let Err(error) = bundle
                    .verify_current(&CANCEL, deadline)
                    .and_then(|()| verify_sources())
                {
                    return Ok(
                        serde_json::json!({"status":"FAIL","phase":"user-bundle-postverify","kind":format!("{:?}",error.kind()),"mutation_may_have_occurred":true,"outer_capability_acquired":true,"retained_capabilities_present_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false,"permanent_detach":false,"journal_recovery_verified":false}),
                    );
                }
                Ok(
                    serde_json::json!({"status":"PREPARED_USER_BUNDLE_DEVELOPMENT_ONLY","phase":"complete","mutation_may_have_occurred":true,"outer_capability_acquired":true,"retained_capabilities_present_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false,"registration_verified":true,"receipt_verified":true,"pair_verified":true,"active_selection_verified":true,"entries":bundle.registration().record().prepared.entries.len(),"permanent_detach":false,"journal_recovery_verified":false,"actual_launch":false}),
                )
            }
            Command::Prepare {
                register,
                receipt_target,
                manifest,
                payload,
                target,
            } => {
                let mp = ancestors(manifest.parent().ok_or_else(rejected)?, deadline)?;
                let mf = source(&manifest, &mp, deadline)?;
                let m = BundleManifest::read(&mf)?;
                if !m.development_only {
                    return Err(rejected());
                }
                let pp = ancestors(&payload, deadline)?;
                let tp = ancestors(target.parent().ok_or_else(rejected)?, deadline)?;
                let dest = tp
                    .last()
                    .ok_or_else(rejected)?
                    .final_path
                    .join(target.file_name().ok_or_else(rejected)?);
                let src = &pp.last().ok_or_else(rejected)?.final_path;
                // A manifest in a shared fixture parent is allowed; it must not be inside the new target.
                if contains(src, &dest)
                    || contains(&dest, src)
                    || contains(&dest, &mp.last().ok_or_else(rejected)?.final_path)
                {
                    return Err(rejected());
                }
                // Native ancestor pins and actual manifest leaf, not its whole parent.
                let rp = receipt_target
                    .as_ref()
                    .map(|p| ancestors(p.parent().ok_or_else(rejected)?, deadline))
                    .transpose()?;
                if let (Some(path), Some(pins)) = (&receipt_target, &rp) {
                    let rd = pins
                        .last()
                        .ok_or_else(rejected)?
                        .final_path
                        .join(path.file_name().ok_or_else(rejected)?);
                    let manifest_final = file_identity::final_path(&mf)?;
                    for other in [src.as_path(), dest.as_path(), manifest_final.as_path()] {
                        if contains(other, &rd) || contains(&rd, other) {
                            return Err(rejected());
                        }
                    }
                    match std::fs::symlink_metadata(path) {
                        Ok(_) => {
                            return Err(io::Error::new(
                                io::ErrorKind::AlreadyExists,
                                "receipt target exists",
                            ));
                        }
                        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                        Err(e) => return Err(e),
                    }
                    for pin in pins {
                        check(deadline)?;
                        pin.verify()?;
                    }
                }
                for pin in tp.iter() {
                    check(deadline)?;
                    pin.verify()?;
                }
                let mut files = Vec::new();
                for e in &m.entries {
                    files.push((
                        e.name.as_str(),
                        source(&payload.join(&e.name), &pp, deadline)?,
                    ));
                }
                let refs: Vec<_> = files.iter().map(|(n, f)| (*n, f)).collect();
                let source_observations = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                let manifest_id = file_identity::from_file(&mf)?;
                let prepared = match prepare_new(&m, &refs, &target, &CANCEL, deadline) {
                    Ok(p) => p,
                    Err(e) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"prepare","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":e.mutation_may_have_occurred(),"retained_file_caps":e.retained_file_cap_count()}),
                        );
                    }
                };
                let verify_sources = || -> io::Result<()> {
                    for pin in mp
                        .iter()
                        .chain(pp.iter())
                        .chain(tp.iter())
                        .chain(rp.iter().flatten())
                    {
                        check(deadline)?;
                        pin.verify()?;
                    }
                    if file_identity::from_file(&mf)? != manifest_id
                        || file_identity::final_path(&mf)?
                            != mp
                                .last()
                                .ok_or_else(rejected)?
                                .final_path
                                .join(manifest.file_name().ok_or_else(rejected)?)
                    {
                        return Err(rejected());
                    }
                    let observations = inspect_payload(&m, &refs, &CANCEL, deadline)?;
                    for original in &source_observations {
                        let now = observations
                            .iter()
                            .find(|n| n.name == original.name)
                            .ok_or_else(rejected)?;
                        if now.identity != original.identity
                            || now.bytes != original.bytes
                            || now.sha256 != original.sha256
                        {
                            return Err(rejected());
                        }
                        let (_, file) = refs
                            .iter()
                            .find(|(name, _)| *name == original.name)
                            .ok_or_else(rejected)?;
                        if file_identity::final_path(file)?
                            != pp
                                .last()
                                .ok_or_else(rejected)?
                                .final_path
                                .join(&original.name)
                        {
                            return Err(rejected());
                        }
                    }
                    check(deadline)
                };
                let post = verify_sources();
                if let Err(e) = post {
                    return Ok(
                        serde_json::json!({"status":"FAIL","phase":"prepare-postverify","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"retained_file_caps":prepared.record().entries.len()+1}),
                    );
                }
                if register {
                    let prepared_caps = prepared.record().entries.len() + 1;
                    let registered = match register_fresh(prepared, &CANCEL, deadline) {
                        Ok(value) => value,
                        Err(e) => {
                            return Ok(
                                serde_json::json!({"status":"FAIL","phase":"register","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"retained_file_caps":prepared_caps+e.retained_file_cap_count(),"registration_mutation_may_have_occurred":e.mutation_may_have_occurred(),"new_registration_file_caps":e.retained_file_cap_count()}),
                            );
                        }
                    };
                    if let Err(e) = registered
                        .verify_current(&CANCEL, deadline)
                        .and_then(|()| verify_sources())
                    {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"register-postverify","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"retained_file_caps":prepared_caps+2,"registration_mutation_may_have_occurred":true,"new_registration_file_caps":2}),
                        );
                    }
                    if let Some(receipt_target) = &receipt_target {
                        let receipt = match create_registration_receipt(
                            &registered,
                            receipt_target,
                            &CANCEL,
                            deadline,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Ok(
                                    serde_json::json!({"status":"FAIL","phase":"receipt-create","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"retained_file_caps":prepared_caps+2+e.retained_file_cap_count(),"registration_verified":true,"registration_mutation_may_have_occurred":true,"new_registration_file_caps":2,"receipt_mutation_may_have_occurred":e.mutation_may_have_occurred(),"receipt_root_capability_acquired":e.root_capability_acquired(),"new_receipt_file_caps":e.retained_file_cap_count()}),
                                );
                            }
                        };
                        if let Err(e) = receipt
                            .verify_current(&CANCEL, deadline)
                            .and_then(|()| registered.verify_current(&CANCEL, deadline))
                            .and_then(|()| verify_sources())
                        {
                            return Ok(
                                serde_json::json!({"status":"FAIL","phase":"receipt-postverify","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"retained_file_caps":prepared_caps+3,"registration_mutation_may_have_occurred":true,"new_registration_file_caps":2,"receipt_mutation_may_have_occurred":true,"new_receipt_file_caps":1,"receipt_root_capability_acquired":true}),
                            );
                        }
                        return Ok(
                            serde_json::json!({"status":"REGISTERED_WITH_RECEIPT_DEVELOPMENT_ONLY","phase":"complete","mutation_may_have_occurred":true,"registration_mutation_may_have_occurred":true,"new_registration_file_caps":2,"receipt_mutation_may_have_occurred":true,"new_receipt_file_caps":1,"receipt_root_capability_acquired":true,"entries":registered.record().prepared.entries.len(),"registration_verified":true,"receipt_verified":true}),
                        );
                    }
                    return Ok(
                        serde_json::json!({"status":"REGISTERED_DEVELOPMENT_ONLY","mutation_may_have_occurred":true,"registration_mutation_may_have_occurred":true,"new_registration_file_caps":2,"entries":registered.record().prepared.entries.len(),"registration_verified":true}),
                    );
                }
                Ok(
                    serde_json::json!({"status":"PREPARED_DEVELOPMENT_ONLY","mutation_may_have_occurred":true,"entries":prepared.record().entries.len(),"trusted_provenance":false,"installer_complete":false,"upgrade_verified":false,"deletion_authority":false}),
                )
            }
            Command::Pair {
                remove,
                target,
                receipt_target,
            } => {
                let pair =
                    inspect_registered_with_receipt(&target, &receipt_target, &CANCEL, deadline)?;
                pair.verify_current(&CANCEL, deadline)?;
                if !remove {
                    return Ok(
                        serde_json::json!({"status":"INSPECTED_PAIR_DEVELOPMENT_ONLY","phase":"complete","entries":pair.registered().record().prepared.entries.len(),"registration_verified":true,"receipt_verified":true,"pair_verified":true}),
                    );
                }
                let authorized = match authorize_explicit_registered_removal(
                    pair, &CANCEL, deadline,
                ) {
                    Ok(value) => value,
                    Err(e) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"remove-authorize","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":e.mutation_may_have_occurred(),"retained_file_caps":e.acquired_file_cap_count(),"root_capability_acquired":e.root_capability_acquired(),"accepted_file_dispositions":0}),
                        );
                    }
                };
                match authorized.remove(&CANCEL, deadline) {
                    Ok(out) => Ok(
                        serde_json::json!({"status":"REMOVED_REGISTERED_DEVELOPMENT_ONLY","phase":"complete","mutation_may_have_occurred":out.accepted_file_dispositions>0 || out.root_disposition_accepted,"accepted_file_dispositions":out.accepted_file_dispositions,"root_disposition_accepted":out.root_disposition_accepted,"pair_verified":true}),
                    ),
                    Err(e) => Ok(
                        serde_json::json!({"status":"FAIL","phase":"remove","kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":e.accepted_file_dispositions()>0,"accepted_file_dispositions":e.accepted_file_dispositions(),"root_disposition_accepted":false,"retained_file_caps":e.retained_file_cap_count()}),
                    ),
                }
            }
            Command::Active {
                inspect,
                target,
                receipt_target,
                active_target,
            } => {
                let phase = if inspect {
                    "inspect-active"
                } else {
                    "active-selection"
                };
                let failure = |e: io::Error| serde_json::json!({"status":"FAIL","phase":phase,"kind":format!("{:?}",e.kind())});
                #[cfg(feature = "active-selection-control-fixture")]
                if let Err(e) = active_control_gate(deadline) {
                    return Ok(failure(e));
                }
                if inspect {
                    let selected = match inspect_active_selection(
                        &active_target,
                        &target,
                        &receipt_target,
                        &CANCEL,
                        deadline,
                    ) {
                        Ok(v) => v,
                        Err(e) => return Ok(failure(e)),
                    };
                    if let Err(e) = selected.verify_current(&CANCEL, deadline) {
                        return Ok(failure(e));
                    }
                    return Ok(
                        serde_json::json!({"status":"INSPECTED_ACTIVE_DEVELOPMENT_ONLY","phase":phase,"active_selection_verified":true,"registration_verified":true,"receipt_verified":true,"pair_verified":true}),
                    );
                }
                let pair = match inspect_registered_with_receipt(
                    &target,
                    &receipt_target,
                    &CANCEL,
                    deadline,
                ) {
                    Ok(v) => v,
                    Err(e) => return Ok(failure(e)),
                };
                let selected = match select_registered_pair(pair, &active_target, &CANCEL, deadline)
                {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":phase,"kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":e.mutation_may_have_occurred(),"active_root_capability_acquired":e.root_capability_acquired(),"new_active_file_caps":e.retained_file_cap_count(),"retained_file_caps":e.retained_file_cap_count()}),
                        );
                    }
                };
                if let Err(e) = selected.verify_current(&CANCEL, deadline) {
                    return Ok(
                        serde_json::json!({"status":"FAIL","phase":phase,"kind":format!("{:?}",e.kind()),"mutation_may_have_occurred":true,"active_root_capability_acquired":true,"new_active_file_caps":1,"retained_file_caps":1}),
                    );
                }
                Ok(
                    serde_json::json!({"status":"SELECTED_ACTIVE_DEVELOPMENT_ONLY","phase":phase,"mutation_may_have_occurred":true,"active_root_capability_acquired":true,"new_active_file_caps":1,"active_selection_verified":true,"registration_verified":true,"receipt_verified":true,"pair_verified":true}),
                )
            }
            Command::LaunchSelected {
                run_until_cancelled,
                target,
                receipt_target,
                active_target,
            } => {
                let selected = match inspect_active_selection(
                    &active_target,
                    &target,
                    &receipt_target,
                    &CANCEL,
                    deadline,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"launch-inspect","kind":format!("{:?}",e.kind()),"child_created":false,"selection_capabilities_held_before_drop":false,"original_owner_retained_before_drop":false,"retained_file_caps":null,"retained_file_cap_count_reported":false}),
                        );
                    }
                };
                let mut process = match launch_explicit_selection(selected, &CANCEL, deadline) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(
                            serde_json::json!({"status":"FAIL","phase":"launch-selected","kind":format!("{:?}",e.kind()),"child_created":e.child_created(),"child_resumed":e.child_resumed(),"resume_attempted":e.resume_attempted(),"may_have_executed":e.may_have_executed(),"terminal_confirmed":e.terminal_confirmed(),"cleanup_unconfirmed":e.cleanup_unconfirmed(),"retained_orphan_handle_count":e.retained_orphan_handle_count(),"selection_retained":e.selection_retained(),"selection_capabilities_held_before_drop":e.selection_retained(),"retained_file_caps":null,"retained_file_cap_count_reported":false}),
                        );
                    }
                };
                let creation = process.creation_filetime();
                let attempted = process.resume_attempted();
                let may_run = process.may_have_executed();
                let waited = if run_until_cancelled {
                    process.wait_cancelled(&CANCEL)
                } else {
                    process.wait(&CANCEL, deadline)
                };
                let receipt: io::Result<serde_json::Value> = match waited {
                    Ok(out) => {
                        let kind = match out.reason {
                            SelectedExitReason::Natural
                                if out.code == 0 && out.job_terminal_confirmed =>
                            {
                                None
                            }
                            SelectedExitReason::Natural => Some(io::ErrorKind::Other),
                            SelectedExitReason::Cancelled => Some(io::ErrorKind::Interrupted),
                            SelectedExitReason::TimedOut => Some(io::ErrorKind::TimedOut),
                        };
                        Ok(
                            serde_json::json!({"status":if kind.is_none(){"LAUNCHED_SELECTED_DEVELOPMENT_ONLY"}else{"FAIL"},"phase":"launch-wait","kind":kind.map(|v|format!("{v:?}")),"reason":format!("{:?}",out.reason),"exit_code":out.code,"child_created":true,"resume_attempted":attempted,"may_have_executed":may_run,"child_creation_filetime":creation,"job_terminal_confirmed":out.job_terminal_confirmed,"terminal_unconfirmed":!out.job_terminal_confirmed,"supervised_timeout_seconds":args.timeout.as_secs(),"permanent_detach":false,"active_selection_verified":true,"pair_verified":true,"original_owner_retained_before_drop":true,"selection_capabilities_held_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false}),
                        )
                    }
                    Err(e) => Ok(
                        serde_json::json!({"status":"FAIL","phase":"launch-wait","kind":format!("{:?}",e.kind()),"child_created":true,"resume_attempted":attempted,"may_have_executed":may_run,"child_creation_filetime":creation,"terminal_unconfirmed":e.terminal_unconfirmed(),"original_owner_retained_before_drop":true,"selection_capabilities_held_before_drop":true,"retained_file_caps":null,"retained_file_cap_count_reported":false,"job_terminal_confirmed":false,"permanent_detach":false}),
                    ),
                };
                let mut receipt = receipt?;
                if run_until_cancelled && let Some(obj) = receipt.as_object_mut() {
                    obj.insert("phase".into(), "run-wait".into());
                    obj.remove("supervised_timeout_seconds");
                    obj.insert(
                        "startup_timeout_seconds".into(),
                        args.timeout.as_secs().into(),
                    );
                    obj.insert("runtime_deadline_seconds".into(), serde_json::Value::Null);
                    obj.insert("runtime_until_cancel_or_exit".into(), true.into());
                    if obj.get("status").and_then(|v| v.as_str())
                        == Some("LAUNCHED_SELECTED_DEVELOPMENT_ONLY")
                    {
                        obj.insert("status".into(), "RAN_SELECTED_DEVELOPMENT_ONLY".into());
                    }
                }
                Ok(receipt)
            }
            Command::Inspect { registered, target } => {
                if registered {
                    let inspected = inspect_registered(&target, &CANCEL, deadline)?;
                    if !inspected.record().prepared.development_only {
                        return Err(rejected());
                    }
                    inspected.verify_current(&CANCEL, deadline)?;
                    check(deadline)?;
                    return Ok(
                        serde_json::json!({"status":"INSPECTED_REGISTERED_DEVELOPMENT_ONLY","entries":inspected.record().prepared.entries.len(),"registration_verified":true}),
                    );
                }
                let inspected = inspect_prepared(&target, &CANCEL, deadline)?;
                if !inspected.record().development_only {
                    return Err(rejected());
                }
                check(deadline)?;
                Ok(
                    serde_json::json!({"status":"INSPECTED_DEVELOPMENT_ONLY","entries":inspected.record().entries.len(),"trusted_provenance":false,"installer_complete":false,"upgrade_verified":false,"deletion_authority":false}),
                )
            }
        }
    }
}
fn main() {
    let began = Instant::now();
    let args: Vec<_> = std::env::args_os().skip(1).take(12).collect();
    if args == [OsString::from("--help")] {
        println!(
            "remove-user-bundle --bundle-root ABS --journal-root ABS [--timeout-seconds 1..600]: explicit current verified development bundle removal; preserves external journal/source/data/media, no discovery, crash ownership or automatic resume.\nprepare-journal-user-bundle --manifest ABS --source-dir ABS --bundle-root NEW_ABS --journal-root NEW_ABS [--timeout-seconds 1..600]: records actual verified stages; no crash authority, repair, resume or launch.\nlaunch-selected --target REG --receipt-target RECEIPT --active-target ACTIVE [--timeout-seconds 1..600]: development-only supervised execution (default 300s). Supervisor exit or deadline may terminate the child. No permanent detach, automatic discovery, or publisher trust.\nrun-selected --target REG --receipt-target RECEIPT --active-target ACTIVE [--startup-timeout-seconds 1..600]: startup deadline only (default 300s); retains supervisor until natural exit or cancellation. No runtime deadline or detach.\nprepare-user-bundle --manifest ABS --source-dir ABS --bundle-root NEW_ABS [--timeout-seconds 1..600]: fresh development bundle only; fixed registered/receipt/active children, no replacement, journal repair, upgrade or permanent detach.\ninspect-user-bundle --bundle-root ABS [--timeout-seconds 1..600]: explicit readonly development consistency inspection; no discovery, repair, launch or deletion.\ninspect-install-journal --journal-root ABS --bundle-root ABS [--timeout-seconds 1..600]: explicit readonly current journal/bundle consistency (default 300s); no crash-commit proof, historical authority, repair, resume, launch or deletion."
        );
        return;
    }
    let parsed = parse(&args);
    #[cfg(windows)]
    let result = parsed.and_then(|args| native::run(args, began));
    #[cfg(not(windows))]
    let result: io::Result<serde_json::Value> = {
        let _ = (parsed, began);
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
    };
    let (mut value, code) = match result {
        Ok(v) => {
            let code = if v["status"] == "FAIL" { 1 } else { 0 };
            (v, code)
        }
        Err(e) => (
            serde_json::json!({"status":"FAIL","kind":format!("{:?}",e.kind()),"trusted_provenance":false,"installer_complete":false}),
            1,
        ),
    };
    let operation = args
        .first()
        .and_then(|v| v.to_str())
        .filter(|v| {
            matches!(
                *v,
                "prepare"
                    | "inspect"
                    | "prepare-register"
                    | "inspect-registered"
                    | "prepare-register-receipt"
                    | "inspect-pair"
                    | "remove-registered"
                    | "select-active"
                    | "inspect-active"
                    | "launch-selected"
                    | "run-selected"
                    | "prepare-user-bundle"
                    | "inspect-user-bundle"
                    | "inspect-install-journal"
                    | "prepare-journal-user-bundle"
                    | "remove-user-bundle"
            )
        })
        .unwrap_or("invalid");
    if let Some(obj) = value.as_object_mut() {
        obj.insert("schema".into(), 1.into());
        obj.insert("operation".into(), operation.into());
        for name in [
            "trusted_provenance",
            "publisher_trust",
            "active_installation",
            "installer_complete",
            "upgrade_verified",
            "deletion_authority",
        ] {
            obj.insert(name.into(), false.into());
        }
        obj.insert("development_only".into(), true.into());
        obj.insert("capabilities_retained_after_exit".into(), false.into());
        obj.insert("cap_counts_are_pre_drop_observations".into(), true.into());
        obj.insert(
            "explicit_removal_requested".into(),
            (operation == "remove-registered" || operation == "remove-user-bundle").into(),
        );
        obj.entry("phase").or_insert("preflight".into());
        for name in [
            "receipt_verified",
            "pair_verified",
            "receipt_mutation_may_have_occurred",
            "receipt_root_capability_acquired",
            "root_disposition_accepted",
            "active_root_capability_acquired",
            "active_selection_verified",
        ] {
            obj.entry(name).or_insert(false.into());
        }
        for name in [
            "new_receipt_file_caps",
            "accepted_file_dispositions",
            "new_active_file_caps",
        ] {
            obj.entry(name).or_insert(0.into());
        }
        obj.entry("mutation_may_have_occurred")
            .or_insert(false.into());
        obj.entry("retained_file_caps").or_insert(0.into());
        obj.entry("registration_verified").or_insert(false.into());
        obj.entry("registration_mutation_may_have_occurred")
            .or_insert(false.into());
        obj.entry("new_registration_file_caps").or_insert(0.into());
        if operation == "remove-user-bundle" {
            // Generic retained_file_caps includes unknown readonly journal handles;
            // report DELETE cap counts separately from the real opaque failure.
            for key in [
                "retained_file_caps",
                "new_registration_file_caps",
                "new_receipt_file_caps",
                "new_active_file_caps",
                "registration_mutation_may_have_occurred",
                "receipt_mutation_may_have_occurred",
                "receipt_root_capability_acquired",
                "active_root_capability_acquired",
                "registration_verified",
                "receipt_verified",
                "pair_verified",
                "active_selection_verified",
            ] {
                obj.insert(key.into(), serde_json::Value::Null);
            }
            obj.insert("retained_file_cap_count_reported".into(), false.into());
            for key in [
                "crash_commit_proven",
                "historical_ownership",
                "actual_launch",
                "permanently_detached",
                "journal_deleted",
                "source_deleted",
                "media_deleted",
            ] {
                obj.insert(key.into(), false.into());
            }
        }
        if operation == "prepare-journal-user-bundle" {
            for key in [
                "crash_commit_proven",
                "historical_ownership",
                "deletion_authority",
                "actual_launch",
                "prior_phase_authority_verified_by_journal",
                "permanently_detached",
                "manual_backfill_used",
            ] {
                obj.insert(key.into(), false.into());
            }
            // This API reports aggregate presence, not individual stage cap counts.
            for key in [
                "retained_file_caps",
                "new_registration_file_caps",
                "new_receipt_file_caps",
                "new_active_file_caps",
                "registration_mutation_may_have_occurred",
                "receipt_mutation_may_have_occurred",
                "receipt_root_capability_acquired",
                "active_root_capability_acquired",
            ] {
                obj.insert(key.into(), serde_json::Value::Null);
            }
            for key in [
                "registration_verified",
                "receipt_verified",
                "pair_verified",
                "active_selection_verified",
            ] {
                if obj.get(key).and_then(serde_json::Value::as_bool) != Some(true) {
                    obj.insert(key.into(), serde_json::Value::Null);
                }
            }
            for key in [
                "retained_file_cap_count_reported",
                "registration_mutation_reported",
                "receipt_mutation_reported",
                "stage_capability_counts_reported",
                "cap_counts_are_pre_drop_observations",
            ] {
                obj.insert(key.into(), false.into());
            }
        }
        if operation == "inspect-install-journal" {
            for key in [
                "crash_commit_proven",
                "historical_ownership",
                "deletion_authority",
                "mutation_may_have_occurred",
                "actual_launch",
                "prior_phase_authority_verified_by_journal",
                "permanently_detached",
            ] {
                obj.insert(key.into(), false.into());
            }
            obj.entry("current_consistency_verified")
                .or_insert(false.into());
            obj.insert("cap_counts_are_pre_drop_observations".into(), false.into());
            obj.insert("cap_count_reported".into(), false.into());
            obj.insert("retained_file_caps".into(), serde_json::Value::Null);
        }
        if operation == "inspect-user-bundle" {
            obj.insert("cap_counts_are_pre_drop_observations".into(), false.into());
            obj.insert("cap_count_reported".into(), false.into());
            obj.insert("retained_file_caps".into(), serde_json::Value::Null);
        }
    }
    let text = serde_json::to_string(&value).unwrap_or_else(|_| "{\"status\":\"FAIL\"}".into());
    if text.len() > 4096 {
        println!("{{\"status\":\"FAIL\"}}");
        std::process::exit(1);
    }
    println!("{text}");
    std::process::exit(code)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn args(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }
    fn root() -> String {
        std::env::temp_dir().to_string_lossy().into_owned()
    }
    #[test]
    fn active_commands_require_exact_three_paths_and_strict_flags() {
        let p = root();
        for command in ["select-active", "inspect-active"] {
            let full = args(&[
                command,
                "--target",
                &p,
                "--receipt-target",
                &p,
                "--active-target",
                &p,
                "--timeout-seconds",
                "600",
            ]);
            assert!(matches!(
                parse(&full).unwrap().command,
                Command::Active { .. }
            ));
            assert!(parse(&full[..5]).is_err());
            for extra in [
                "--manifest",
                "--payload-dir",
                "--unknown",
                "--active-target",
            ] {
                let mut v = full.clone();
                v.extend(args(&[extra, &p]));
                assert!(parse(&v).is_err());
            }
            for value in ["relative", "../escape", "bad\npath"] {
                let mut v = full.clone();
                v[6] = value.into();
                assert!(parse(&v).is_err());
            }
            for value in ["0", "601", "-1", "1.0"] {
                let mut v = full.clone();
                v[8] = value.into();
                assert!(parse(&v).is_err());
            }
        }
        assert!(parse(&args(&["inspect", "--target", &p, "--active-target", &p])).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn active_paths_accept_disk_unicode_and_reject_unc_device_utf16_overflow() {
        let good = r"D:\한글 공간\active";
        for value in [good, r"\\?\D:\한글 공간\active"] {
            assert!(
                parse(&args(&[
                    "inspect-active",
                    "--target",
                    good,
                    "--receipt-target",
                    good,
                    "--active-target",
                    value
                ]))
                .is_ok()
            );
        }
        for value in [r"\\server\share\active", r"\\.\C:\active"] {
            assert!(
                parse(&args(&[
                    "inspect-active",
                    "--target",
                    good,
                    "--receipt-target",
                    good,
                    "--active-target",
                    value
                ]))
                .is_err()
            );
        }
        let oversized = format!("D:\\{}", "한".repeat(32768));
        assert!(
            parse(&args(&[
                "inspect-active",
                "--target",
                good,
                "--receipt-target",
                good,
                "--active-target",
                &oversized
            ]))
            .is_err()
        );
    }
    #[test]
    fn receipt_and_pair_command_boundaries() {
        let p = root();
        let full = args(&[
            "prepare-register-receipt",
            "--manifest",
            &p,
            "--payload-dir",
            &p,
            "--target",
            &p,
            "--receipt-target",
            &p,
            "--timeout-seconds",
            "600",
        ]);
        assert_eq!(full.len(), 11);
        assert!(matches!(
            parse(&full).unwrap().command,
            Command::Prepare {
                register: true,
                receipt_target: Some(_),
                ..
            }
        ));
        assert!(parse(&full[..7]).is_err());
        let mut over = full.clone();
        over.extend(args(&["--target", &p]));
        assert!(parse(&over).is_err());
        for c in ["inspect-pair", "remove-registered"] {
            assert!(matches!(
                parse(&args(&[c, "--target", &p, "--receipt-target", &p]))
                    .unwrap()
                    .command,
                Command::Pair { .. }
            ));
            assert!(parse(&args(&[c, "--target", &p])).is_err());
            assert!(
                parse(&args(&[
                    c,
                    "--target",
                    &p,
                    "--receipt-target",
                    &p,
                    "--manifest",
                    &p
                ]))
                .is_err()
            );
            assert!(parse(&args(&[c, "--target", &p, "--receipt-target", "relative"])).is_err());
            assert!(
                parse(&args(&[
                    c,
                    "--target",
                    &p,
                    "--receipt-target",
                    &p,
                    "--receipt-target",
                    &p
                ]))
                .is_err()
            );
        }
        for c in [
            "prepare",
            "prepare-register",
            "inspect",
            "inspect-registered",
        ] {
            assert!(parse(&args(&[c, "--target", &p, "--receipt-target", &p])).is_err());
        }
    }
    #[test]
    fn registered_commands_use_exact_command_specific_flags() {
        let root = root();
        for command in ["prepare", "prepare-register"] {
            assert!(
                parse(&args(&[
                    command,
                    "--manifest",
                    &root,
                    "--payload-dir",
                    &root,
                    "--target",
                    &root
                ]))
                .is_ok()
            );
            assert!(parse(&args(&[command, "--target", &root])).is_err());
        }
        for command in ["inspect", "inspect-registered"] {
            assert!(parse(&args(&[command, "--target", &root])).is_ok());
            assert!(parse(&args(&[command, "--target", &root, "--manifest", &root])).is_err());
            assert!(parse(&args(&[command, "--target", &root, "--payload-dir", &root])).is_err());
        }
        for command in ["prepare-registered", "register", "inspect-register"] {
            assert!(parse(&args(&[command, "--target", &root])).is_err());
        }
    }
    #[test]
    fn parser_bounds_and_complete_commands() {
        let root = root();
        let parsed = parse(&args(&[
            "prepare",
            "--manifest",
            &root,
            "--payload-dir",
            &root,
            "--target",
            &root,
        ]))
        .unwrap();
        assert_eq!(parsed.timeout, Duration::from_secs(300));
        assert_eq!(
            parse(&args(&[
                "inspect",
                "--target",
                &root,
                "--timeout-seconds",
                "600"
            ]))
            .unwrap()
            .timeout,
            Duration::from_secs(600)
        );
        for path in [
            format!("{root}/../escape"),
            format!("{root}/bad\nname"),
            "a".repeat(131065),
        ] {
            assert!(parse(&args(&["inspect", "--target", &path])).is_err());
        }
        assert!(
            parse(&args(&[
                "inspect",
                "--target",
                &root,
                "--timeout-seconds",
                "000000000000000000001"
            ]))
            .is_err()
        );
    }
    #[test]
    fn strict_parser() {
        let root = root();
        assert!(parse(&args(&["inspect", "--target", &root])).is_ok());
        for a in [
            vec!["inspect", "--target", &root, "--target", &root],
            vec!["inspect", "--target", &root, "--manifest", &root],
            vec!["prepare", "--target", &root],
            vec!["inspect", "--target", "relative"],
            vec!["inspect", "--target", &root, "--unknown", "a"],
        ] {
            assert!(parse(&args(&a)).is_err());
        }
        for n in ["0", "601", "-1", "+1", "18446744073709551616"] {
            assert!(
                parse(&args(&[
                    "inspect",
                    "--target",
                    &root,
                    "--timeout-seconds",
                    n
                ]))
                .is_err()
            );
        }
    }
}

#[cfg(test)]
mod launch_parser_tests {
    use super::*;
    fn paths() -> Vec<OsString> {
        let root = std::env::temp_dir();
        vec![
            "launch-selected".into(),
            "--target".into(),
            root.join("reg").into_os_string(),
            "--receipt-target".into(),
            root.join("receipt").into_os_string(),
            "--active-target".into(),
            root.join("active").into_os_string(),
        ]
    }
    #[test]
    fn launch_exact_paths_and_supervised_deadline() {
        let mut args = paths();
        assert!(matches!(
            parse(&args).unwrap().command,
            Command::LaunchSelected { .. }
        ));
        args.extend(["--timeout-seconds".into(), "600".into()]);
        assert_eq!(parse(&args).unwrap().timeout, Duration::from_secs(600));
        *args.last_mut().unwrap() = "601".into();
        assert!(parse(&args).is_err());
        *args.last_mut().unwrap() = "0".into();
        assert!(parse(&args).is_err());
    }
    #[test]
    fn launch_missing_duplicate_unknown_and_legacy_extra_rejected() {
        let args = paths();
        let mut missing = args.clone();
        missing.truncate(5);
        assert!(parse(&missing).is_err());
        for key in ["--target", "--unknown", "--payload-dir"] {
            let mut bad = args.clone();
            bad.extend([key.into(), std::env::temp_dir().into_os_string()]);
            assert!(parse(&bad).is_err());
        }
        let mut legacy = args;
        legacy[0] = "inspect".into();
        assert!(parse(&legacy).is_err());
    }
    #[test]
    fn launch_bad_path_and_windows_unc_rejected() {
        let mut bad = paths();
        bad[2] = "relative".into();
        assert!(parse(&bad).is_err());
        bad[2] = std::env::temp_dir().join("../bad").into_os_string();
        assert!(parse(&bad).is_err());
        #[cfg(windows)]
        {
            bad[2] = r"\\server\share\reg".into();
            assert!(parse(&bad).is_err());
        }
    }
}

#[cfg(test)]
mod run_selected_parser_tests {
    use super::*;
    fn paths() -> Vec<OsString> {
        let root = std::env::temp_dir();
        vec![
            "run-selected".into(),
            "--target".into(),
            root.join("reg").into_os_string(),
            "--receipt-target".into(),
            root.join("receipt").into_os_string(),
            "--active-target".into(),
            root.join("active").into_os_string(),
        ]
    }
    #[test]
    fn runtime_has_only_a_startup_budget() {
        let mut args = paths();
        let parsed = parse(&args).unwrap();
        assert!(matches!(
            parsed.command,
            Command::LaunchSelected {
                run_until_cancelled: true,
                ..
            }
        ));
        assert_eq!(parsed.timeout, Duration::from_secs(300));
        args.extend(["--startup-timeout-seconds".into(), "600".into()]);
        assert_eq!(parse(&args).unwrap().timeout, Duration::from_secs(600));
        for bad in ["0", "601", "-1", "18446744073709551616"] {
            *args.last_mut().unwrap() = bad.into();
            assert!(parse(&args).is_err());
        }
    }
    #[test]
    fn deadline_flags_cannot_cross_commands_or_duplicate() {
        let mut args = paths();
        args.extend(["--timeout-seconds".into(), "1".into()]);
        assert!(parse(&args).is_err());
        args[7] = "--startup-timeout-seconds".into();
        args.extend(["--startup-timeout-seconds".into(), "2".into()]);
        assert!(parse(&args).is_err());
        args.truncate(9);
        for command in ["launch-selected", "inspect-active", "select-active"] {
            args[0] = command.into();
            assert!(parse(&args).is_err());
        }
        let mut args = paths();
        args.truncate(5);
        assert!(parse(&args).is_err());
    }
    #[test]
    fn user_bundle_requires_exact_options_and_finite_prepare_timeout() {
        let path = if cfg!(windows) { r"D:\owned" } else { "/owned" };
        let good: Vec<OsString> = [
            "prepare-user-bundle",
            "--manifest",
            path,
            "--source-dir",
            path,
            "--bundle-root",
            path,
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        assert!(matches!(
            parse(&good).unwrap().command,
            Command::UserBundle { .. }
        ));
        for missing in [1usize, 3, 5] {
            let mut args = good.clone();
            args.drain(missing..missing + 2);
            assert!(parse(&args).is_err());
        }
        for flag in [
            "--target",
            "--payload-dir",
            "--receipt-target",
            "--active-target",
            "--startup-timeout-seconds",
        ] {
            let mut args = good.clone();
            args.extend([flag.into(), path.into()]);
            assert!(parse(&args).is_err());
        }
        for value in ["0", "601", "x"] {
            let mut args = good.clone();
            args.extend(["--timeout-seconds".into(), value.into()]);
            assert!(parse(&args).is_err());
        }
        let mut duplicate = good.clone();
        duplicate.extend(["--bundle-root".into(), path.into()]);
        assert!(parse(&duplicate).is_err());
    }
    #[test]
    fn user_bundle_flags_cannot_leak_into_legacy_commands() {
        let path = if cfg!(windows) { r"D:\owned" } else { "/owned" };
        for command in ["inspect", "inspect-registered"] {
            for flag in ["--bundle-root", "--source-dir"] {
                let args = [command, "--target", path, flag, path].map(OsString::from);
                assert!(parse(&args).is_err());
            }
        }
    }
    #[test]
    fn inspect_user_bundle_requires_only_explicit_root_and_finite_timeout() {
        let path = if cfg!(windows) { r"D:\owned" } else { "/owned" };
        let good = ["inspect-user-bundle", "--bundle-root", path].map(OsString::from);
        assert!(matches!(
            parse(&good).unwrap().command,
            Command::InspectUserBundle { .. }
        ));
        assert!(parse(&[OsString::from("inspect-user-bundle")]).is_err());
        for flag in [
            "--target",
            "--manifest",
            "--source-dir",
            "--receipt-target",
            "--active-target",
            "--startup-timeout-seconds",
            "--bundle-root",
        ] {
            let mut args = good.to_vec();
            args.extend([flag.into(), path.into()]);
            assert!(parse(&args).is_err());
        }
        for value in ["0", "601", "x"] {
            let mut args = good.to_vec();
            args.extend(["--timeout-seconds".into(), value.into()]);
            assert!(parse(&args).is_err());
        }
        for value in ["1", "600"] {
            let mut args = good.to_vec();
            args.extend(["--timeout-seconds".into(), value.into()]);
            assert!(parse(&args).is_ok());
        }
        let relative = ["inspect-user-bundle", "--bundle-root", "relative"].map(OsString::from);
        assert!(parse(&relative).is_err());
    }
    #[test]
    fn journal_inspection_requires_two_explicit_roots_and_rejects_foreign_flags() {
        fn args(parts: &[&str]) -> Vec<OsString> {
            parts.iter().map(OsString::from).collect()
        }
        let path = if cfg!(windows) { r"D:\owned" } else { "/owned" };
        let good = args(&[
            "inspect-install-journal",
            "--journal-root",
            path,
            "--bundle-root",
            path,
        ]);
        assert!(matches!(
            parse(&good).unwrap().command,
            Command::InspectInstallJournal { .. }
        ));
        assert_eq!(parse(&good).unwrap().timeout, Duration::from_secs(300));
        for missing in [
            args(&["inspect-install-journal", "--journal-root", path]),
            args(&["inspect-install-journal", "--bundle-root", path]),
        ] {
            assert!(parse(&missing).is_err());
        }
        for flag in [
            "--journal-root",
            "--bundle-root",
            "--unknown",
            "--target",
            "--manifest",
            "--startup-timeout-seconds",
        ] {
            let mut bad = good.clone();
            bad.extend([flag.into(), path.into()]);
            assert!(parse(&bad).is_err());
        }
        for value in ["0", "601", "x"] {
            let mut bad = good.clone();
            bad.extend(["--timeout-seconds".into(), value.into()]);
            assert!(parse(&bad).is_err());
        }
        for value in ["1", "600"] {
            let mut valid = good.clone();
            valid.extend(["--timeout-seconds".into(), value.into()]);
            assert!(parse(&valid).is_ok());
        }
        for key in ["--journal-root", "--bundle-root"] {
            let mut relative = good.clone();
            let index = relative
                .iter()
                .position(|v| v == &OsString::from(key))
                .unwrap()
                + 1;
            relative[index] = "relative".into();
            assert!(parse(&relative).is_err());
            #[cfg(windows)]
            {
                relative[index] = r"\\server\share\owned".into();
                assert!(parse(&relative).is_err());
            }
        }
        for command in [
            "prepare",
            "inspect",
            "prepare-register",
            "inspect-registered",
            "prepare-register-receipt",
            "inspect-pair",
            "remove-registered",
            "select-active",
            "inspect-active",
            "launch-selected",
            "run-selected",
            "prepare-user-bundle",
            "inspect-user-bundle",
        ] {
            let foreign = args(&[command, "--journal-root", path, "--bundle-root", path]);
            assert!(parse(&foreign).is_err());
        }
    }
    #[test]
    fn journal_user_bundle_requires_exact_options() {
        let path = if cfg!(windows) {
            "D:\\owned\\new"
        } else {
            "/owned/new"
        };
        let good = [
            "prepare-journal-user-bundle",
            "--manifest",
            path,
            "--source-dir",
            path,
            "--bundle-root",
            path,
            "--journal-root",
            path,
        ];
        assert!(matches!(
            parse(&good.map(OsString::from)).unwrap().command,
            Command::JournalUserBundle { .. }
        ));
        for missing in [1usize, 3, 5, 7] {
            let mut bad = good.map(OsString::from).to_vec();
            bad.drain(missing..missing + 2);
            assert!(parse(&bad).is_err());
        }
        for seconds in ["1", "600"] {
            let mut valid = good.map(OsString::from).to_vec();
            valid.extend(["--timeout-seconds".into(), seconds.into()]);
            let parsed = parse(&valid).unwrap();
            assert_eq!(
                parsed.timeout,
                Duration::from_secs(seconds.parse().unwrap())
            );
            assert!(matches!(parsed.command, Command::JournalUserBundle { .. }));
        }
        for i in [2usize, 4, 6, 8] {
            let mut unc = good.map(OsString::from).to_vec();
            unc[i] = r"\\server\share\target".into();
            assert!(parse(&unc).is_err());
        }
        for invalid in ["0", "601"] {
            let mut bad = good.map(OsString::from).to_vec();
            bad.extend(["--timeout-seconds".into(), invalid.into()]);
            assert!(parse(&bad).is_err());
        }
        for i in [2usize, 4, 6, 8] {
            let mut bad = good.map(OsString::from).to_vec();
            bad[i] = "relative".into();
            assert!(parse(&bad).is_err());
        }
        for extra in [
            "--target",
            "--startup-timeout-seconds",
            "--unknown",
            "--journal-root",
        ] {
            let mut bad = good.map(OsString::from).to_vec();
            bad.extend([extra.into(), path.into()]);
            assert!(parse(&bad).is_err());
        }
    }
    #[test]
    fn remove_user_bundle_requires_only_explicit_roots_and_finite_timeout() {
        let path = if cfg!(windows) {
            r"D:\owned\root"
        } else {
            "/owned/root"
        };
        let good = [
            "remove-user-bundle",
            "--bundle-root",
            path,
            "--journal-root",
            path,
        ]
        .map(OsString::from);
        assert!(matches!(
            parse(&good).unwrap().command,
            Command::RemoveUserBundle { .. }
        ));
        for seconds in ["1", "600"] {
            let mut valid = good.to_vec();
            valid.extend(["--timeout-seconds".into(), seconds.into()]);
            assert_eq!(
                parse(&valid).unwrap().timeout,
                Duration::from_secs(seconds.parse().unwrap())
            );
        }
        for missing in [1usize, 3] {
            let mut bad = good.to_vec();
            bad.drain(missing..missing + 2);
            assert!(parse(&bad).is_err());
        }
        for key in [
            "--manifest",
            "--source-dir",
            "--startup-timeout-seconds",
            "--target",
            "--bundle-root",
            "--journal-root",
            "--unknown",
        ] {
            let mut bad = good.to_vec();
            bad.extend([key.into(), path.into()]);
            assert!(parse(&bad).is_err());
        }
        for seconds in ["0", "601"] {
            let mut bad = good.to_vec();
            bad.extend(["--timeout-seconds".into(), seconds.into()]);
            assert!(parse(&bad).is_err());
        }
        for index in [2usize, 4] {
            for badpath in ["relative", r"\\server\share\root"] {
                let mut bad = good.to_vec();
                bad[index] = badpath.into();
                assert!(parse(&bad).is_err());
            }
        }
    }
}
