use super::*;
fn limit() -> Instant {
    Instant::now() + std::time::Duration::from_secs(30)
}
fn fixture() -> (FreshRoot, Binding, Locations) {
    let root = FreshRoot::create_new(
        &std::env::temp_dir().join(format!("dlpad-shell-test-{}", nonce().unwrap())),
    )
    .unwrap();
    let id = root.identity().clone();
    let sid = security::current_sid().unwrap();
    let b = Binding {
        schema: 1,
        domain: "dlpad.windows-shell.v1".into(),
        sid,
        id: nonce().unwrap() + &nonce().unwrap(),
        bundle: root.path().to_path_buf(),
        journal: root.path().to_path_buf(),
        bundle_id: id.clone(),
        journal_id: id,
        active_sha: "a".repeat(64),
        version: "0.1.2".into(),
        exe: std::fs::canonicalize(std::env::current_exe().unwrap()).unwrap(),
    };
    let loc = Locations {
        support: root.path().join("support"),
        programs: root.path().join("programs"),
        registry: format!("Software\\DlpadShellFixture\\{}", nonce().unwrap()),
    };
    (root, b, loc)
}
#[test]
fn native_register_idempotence_link_and_owned_cleanup() {
    let c = AtomicBool::new(false);
    let (root, b, loc) = fixture();
    let sentinel = root.path().join("foreign-sentinel");
    std::fs::write(&sentinel, b"preserve").unwrap();
    assert_eq!(
        inspect_bound(b.clone(), &loc, &c, limit())
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    let report = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    assert!(!report.already_registered);
    assert_eq!(report.display_version, "0.1.2");
    assert!(report.shortcut_path.is_file());
    let again = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    assert!(again.already_registered);
    let raw = std::fs::read(&report.shortcut_path).unwrap();
    assert_eq!(&raw[..4], &[0x4c, 0, 0, 0]);
    let inspected = inspect_bound(b.clone(), &loc, &c, limit()).unwrap();
    let cancelled = AtomicBool::new(true);
    let failed = inspected.cleanup(&cancelled, limit()).unwrap_err();
    assert_eq!(failed.kind(), io::ErrorKind::Interrupted);
    assert!(!failed.mutation_may_have_occurred());
    let out = failed.retry(&c, limit()).unwrap();
    assert_eq!(out.removed_files, 3);
    assert!(out.registry_removed && out.support_removed);
    assert!(!report.support_path.exists() && !report.shortcut_path.exists());
    assert_eq!(std::fs::read(sentinel).unwrap(), b"preserve");
    assert_eq!(
        inspect_bound(b, &loc, &c, limit()).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn cancellation_never_creates_shell_objects() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(true);
    assert_eq!(
        register_bound(b.clone(), &loc, &c, limit())
            .unwrap_err()
            .kind(),
        io::ErrorKind::Interrupted
    );
    assert!(!loc.support.exists() && !loc.programs.exists());
    assert!(
        matches!(open_key(&key_path(&loc,&b),false),Err(e)if e.kind()==io::ErrorKind::NotFound)
    );
}
#[test]
fn existing_shortcut_conflict_preserved() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    container(&loc.programs).unwrap();
    let file = loc.programs.join(format!("dlpad ({}).lnk", &b.id[..8]));
    std::fs::write(&file, b"foreign shortcut").unwrap();
    assert!(register_bound(b, &loc, &c, limit()).is_err());
    assert_eq!(std::fs::read(file).unwrap(), b"foreign shortcut");
    assert!(!loc.support.exists());
}
#[test]
fn tamper_and_unknown_child_refuse_cleanup() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    let report = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    let inspected = inspect_bound(b.clone(), &loc, &c, limit()).unwrap();
    std::fs::write(report.support_path.join("foreign-file"), b"preserve").unwrap();
    let failure = inspected.cleanup(&c, limit()).unwrap_err();
    assert!(!failure.mutation_may_have_occurred());
    assert_eq!(failure.accepted_file_dispositions(), 0);
    assert_eq!(
        std::fs::read(report.support_path.join("foreign-file")).unwrap(),
        b"preserve"
    );
    let failure = failure.retry(&c, limit()).unwrap_err();
    assert!(!failure.mutation_may_have_occurred());
    drop(failure);
    std::fs::remove_file(report.support_path.join("foreign-file")).unwrap();
    {
        let mut file = OpenOptions::new()
            .write(true)
            .open(report.support_path.join(HELPER))
            .unwrap();
        file.write_all(b"tampered!").unwrap();
    }
    assert!(inspect_bound(b, &loc, &c, limit()).is_err());
    assert!(report.support_path.join(HELPER).exists() && report.shortcut_path.exists());
}
#[test]
fn quote_rejects_ambiguous_registry_command() {
    assert!(quote(Path::new("D:\\has\"quote\\app.exe")).is_err());
    assert!(quote(Path::new("D:\\trailing\\")).is_err());
    assert_eq!(
        quote(Path::new("D:\\한 글\\app.exe")).unwrap(),
        "\"D:\\한 글\\app.exe\""
    );
}
#[test]
fn dpapi_rejects_modified_or_foreign_record() {
    let plain = b"synthetic shell record";
    let mut cipher = protect(plain, false).unwrap();
    assert_eq!(protect(&cipher, true).unwrap(), plain);
    let n = cipher.len() / 2;
    cipher[n] ^= 1;
    assert!(protect(&cipher, true).is_err());
}
#[test]
fn handoff_rejects_wrong_sid_before_file_reads() {
    let c = AtomicBool::new(false);
    let f = File::open(std::env::current_exe().unwrap()).unwrap();
    assert!(
        verify_uninstall_handoff_files(&f, &f, &"a".repeat(64), "S-1-0-0", &c, limit()).is_err()
    );
}

#[test]
fn missing_registry_recovers_only_authenticated_completed_files() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    let original = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    let helper_before = entry(
        &File::open(original.support_path.join(HELPER)).unwrap(),
        &c,
        limit(),
    )
    .unwrap();
    let owner_before = std::fs::read(original.support_path.join(OWNER)).unwrap();
    let link_before = std::fs::read(&original.shortcut_path).unwrap();
    let key = open_key(&key_path(&loc, &b), true).unwrap();
    delete_key(&key).unwrap();
    drop(key);
    assert_eq!(
        inspect_bound(b.clone(), &loc, &c, limit())
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    let restored = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    assert!(restored.already_registered);
    assert!(
        entry(
            &File::open(original.support_path.join(HELPER)).unwrap(),
            &c,
            limit()
        )
        .unwrap()
            == helper_before
    );
    assert_eq!(
        std::fs::read(original.support_path.join(OWNER)).unwrap(),
        owner_before
    );
    assert_eq!(std::fs::read(original.shortcut_path).unwrap(), link_before);
    inspect_bound(b, &loc, &c, limit())
        .unwrap()
        .cleanup(&c, limit())
        .unwrap();
}
#[test]
fn missing_registry_never_repairs_tampered_or_incomplete_objects() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    let original = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    let key = open_key(&key_path(&loc, &b), true).unwrap();
    delete_key(&key).unwrap();
    drop(key);
    let unknown = original.support_path.join("foreign-item");
    std::fs::write(&unknown, b"preserve").unwrap();
    assert!(register_bound(b.clone(), &loc, &c, limit()).is_err());
    assert!(
        matches!(open_key(&key_path(&loc, &b), true), Err(e) if e.kind() == io::ErrorKind::NotFound)
    );
    assert_eq!(std::fs::read(&unknown).unwrap(), b"preserve");
    std::fs::remove_file(&unknown).unwrap();
    let incomplete = new_key(&key_path(&loc, &b), &b.sid).unwrap();
    assert!(register_bound(b.clone(), &loc, &c, limit()).is_err());
    let mut count = 1;
    win(unsafe {
        RegQueryInfoKeyW(
            incomplete.0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut count,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    })
    .unwrap();
    assert_eq!(count, 0);
    delete_key(&incomplete).unwrap();
    drop(incomplete);
    {
        let mut helper = OpenOptions::new()
            .write(true)
            .open(original.support_path.join(HELPER))
            .unwrap();
        helper.write_all(b"tampered!").unwrap();
    }
    assert!(register_bound(b.clone(), &loc, &c, limit()).is_err());
    assert!(
        matches!(open_key(&key_path(&loc, &b), true), Err(e) if e.kind() == io::ErrorKind::NotFound)
    );
}

#[test]
fn shell_presentation_preserves_long_native_identity_and_rejects_ambiguity() {
    let (root, mut binding, _loc) = fixture();
    let mut directory = root.path().to_owned();
    for index in 0..6 {
        directory = directory.join(format!("long-{index}-{}", "x".repeat(42)));
    }
    std::fs::create_dir_all(&directory).unwrap();
    let target = directory.join("한글 target.exe");
    std::fs::write(&target, b"synthetic shortcut target, never executed").unwrap();
    let native = std::fs::canonicalize(&target).unwrap();
    assert!(native.as_os_str().encode_wide().count() > 260);
    let (presentation, original, _parent) = shell_presentation_target(&native).unwrap();
    assert!(
        !presentation
            .as_os_str()
            .to_string_lossy()
            .starts_with(r"\\?\")
    );
    assert_eq!(
        file_identity::from_file(&original).unwrap(),
        file_identity::from_file(&File::open(&presentation).unwrap()).unwrap()
    );
    binding.exe = native;
    let serialized = shortcut_bytes(&binding).unwrap();
    assert_eq!(&serialized[..4], &[0x4c, 0, 0, 0]);
    let saved_link = root.path().join("long-target.lnk");
    std::fs::write(&saved_link, &serialized).unwrap();
    let reloaded_target = persisted_shortcut_target(&File::open(saved_link).unwrap()).unwrap();
    assert_eq!(
        file_identity::from_file(&reloaded_target).unwrap(),
        file_identity::from_file(&original).unwrap()
    );
    assert_eq!(
        file_identity::final_path(&reloaded_target).unwrap(),
        file_identity::final_path(&original).unwrap()
    );
    let ambiguous = root.path().join("ambiguous.");
    std::fs::write(&ambiguous, b"preserve").unwrap();
    assert!(shell_presentation_target(&ambiguous).is_err());
    assert_eq!(std::fs::read(&ambiguous).unwrap(), b"preserve");
}

#[test]
fn busy_original_helper_acquisition_can_retry_without_losing_pending_caps() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    let report = register_bound(b.clone(), &loc, &c, limit()).unwrap();
    let busy = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(report.support_path.join(HELPER))
        .unwrap();
    let inspected = inspect_bound(b.clone(), &loc, &c, limit()).unwrap();
    let failure = inspected.cleanup(&c, limit()).unwrap_err();
    assert!(!failure.mutation_may_have_occurred());
    assert_eq!(failure.accepted_file_dispositions(), 0);
    assert_eq!(failure.state.files.len(), 3);
    assert!(failure.state.files.iter().all(|item| item.weak.is_some()));
    assert!(report.support_path.join(HELPER).exists() && report.shortcut_path.exists());
    drop(busy);
    let complete = failure.retry(&c, limit()).unwrap();
    assert_eq!(complete.removed_files, 3);
    assert!(complete.registry_removed && complete.support_removed);
    assert_eq!(
        inspect_bound(b, &loc, &c, limit()).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn concurrent_confirmation_is_rejected_before_strong_shell_pins() {
    let (_root, b, loc) = fixture();
    let c = AtomicBool::new(false);
    register_bound(b.clone(), &loc, &c, limit()).unwrap();
    let first = inspect_bound(b.clone(), &loc, &c, limit()).unwrap();
    let other_binding = b.clone();
    let other = Locations {
        support: loc.support.clone(),
        programs: loc.programs.clone(),
        registry: loc.registry.clone(),
    };
    let error = std::thread::spawn(move || {
        inspect_bound(other_binding, &other, &AtomicBool::new(false), limit())
            .unwrap_err()
            .kind()
    })
    .join()
    .unwrap();
    assert_eq!(error, io::ErrorKind::WouldBlock);
    first.cleanup(&c, limit()).unwrap();
    assert_eq!(
        inspect_bound(b, &loc, &c, limit()).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

// Reload actual saved .lnk bytes through IPersistStream, observing only its target.
fn persisted_shortcut_target(saved: &File) -> io::Result<File> {
    let bytes = read_small(saved)?;
    let init = unsafe { CoInitializeEx(ptr::null(), 2) };
    if init < 0 && init != 0x80010106_u32 as i32 {
        hr(init)?;
    }
    let _apartment = Apartment(init >= 0);
    let mut raw = ptr::null_mut();
    hr(unsafe {
        CoCreateInstance(
            &GUID::from_u128(0x00021401_0000_0000_c000_000000000046),
            ptr::null_mut(),
            1,
            &GUID::from_u128(0x000214f9_0000_0000_c000_000000000046),
            &mut raw,
        )
    })?;
    if raw.is_null() {
        return Err(bad());
    }
    let link = Com(raw);
    let query: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32 =
        unsafe { link.method(0) };
    let mut raw = ptr::null_mut();
    hr(unsafe {
        query(
            link.0,
            &GUID::from_u128(0x00000109_0000_0000_c000_000000000046),
            &mut raw,
        )
    })?;
    if raw.is_null() {
        return Err(bad());
    }
    let persist = Com(raw);
    let mut raw = ptr::null_mut();
    hr(unsafe { CreateStreamOnHGlobal(ptr::null_mut(), 1, &mut raw) })?;
    if raw.is_null() {
        return Err(bad());
    }
    let stream = Com(raw);
    let write: unsafe extern "system" fn(*mut c_void, *const c_void, u32, *mut u32) -> i32 =
        unsafe { stream.method(4) };
    let mut written = 0;
    hr(unsafe {
        write(
            stream.0,
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            &mut written,
        )
    })?;
    if written as usize != bytes.len() {
        return Err(bad());
    }
    let seek: unsafe extern "system" fn(*mut c_void, i64, u32, *mut u64) -> i32 =
        unsafe { stream.method(5) };
    hr(unsafe { seek(stream.0, 0, 0, ptr::null_mut()) })?;
    let load: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32 =
        unsafe { persist.method(5) };
    hr(unsafe { load(persist.0, stream.0) })?;
    let get: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32 =
        unsafe { link.method(4) };
    let mut raw = ptr::null_mut();
    hr(unsafe { get(link.0, &mut raw) })?;
    if raw.is_null() {
        return Err(bad());
    }
    let idlist = ParsedIdList(raw);
    let mut path = vec![0_u16; 32768];
    if unsafe { SHGetPathFromIDListEx(idlist.0, path.as_mut_ptr(), path.len() as u32, 0) } == 0 {
        return Err(bad());
    }
    let length = path.iter().position(|unit| *unit == 0).ok_or_else(bad)?;
    file_identity::open_regular_no_reparse(&PathBuf::from(std::ffi::OsString::from_wide(
        &path[..length],
    )))
}
