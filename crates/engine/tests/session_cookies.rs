use dlpad_engine::cookies::{CookieImportError, import_session};

#[cfg(not(any(unix, windows)))]
#[test]
fn unverified_acl_platform_is_unavailable_without_reading_source() {
    assert_eq!(
        import_session(
            std::path::Path::new("synthetic-source"),
            std::path::Path::new("synthetic-cache")
        )
        .unwrap_err(),
        CookieImportError::Unavailable
    );
}

#[cfg(windows)]
#[test]
fn windows_relative_source_is_rejected_before_cache_creation() {
    assert_eq!(
        import_session(
            std::path::Path::new("synthetic-source"),
            std::path::Path::new("synthetic-cache"),
        )
        .unwrap_err(),
        CookieImportError::SourceUnavailable
    );
}

#[cfg(unix)]
mod unix {
    use super::*;
    use dlpad_engine::cookies::{MAX_COOKIE_BYTES, MAX_COOKIE_ENTRIES, MAX_COOKIE_LINE_BYTES};
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
        path::{Path, PathBuf},
    };
    use tempfile::TempDir;

    const HEADER: &str = "# Netscape HTTP Cookie File\n";
    const COOKIE: &str = ".fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-cookie-secret\n";
    fn fixture(bytes: &[u8]) -> (TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("synthetic-sensitive-source.txt");
        fs::write(&source, bytes).unwrap();
        let cache = root.join("app-cookie-cache");
        fs::create_dir(&cache).unwrap();
        fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
        (temp, source, cache)
    }
    fn valid() -> Vec<u8> {
        format!("{HEADER}{COOKIE}").into_bytes()
    }
    fn no_copies(cache: &Path) {
        assert_eq!(fs::read_dir(cache).unwrap().count(), 0);
    }
    fn padded(total: usize) -> Vec<u8> {
        let mut data = valid();
        while data.len() < total {
            let remaining = total - data.len();
            if remaining == 1 {
                data.push(b'\n');
                break;
            }
            let line = (remaining - 1).min(MAX_COOKIE_LINE_BYTES);
            data.push(b'#');
            data.resize(data.len() + line - 1, b'x');
            data.push(b'\n');
        }
        data
    }
    #[test]
    fn exact_literal_private_copy_and_explicit_cleanup_preserve_external_source() {
        let data = b"# HTTP Cookie File\r\n# original literal comment\r\n#HttpOnly_.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-cookie-secret\r\n.fixture.invalid\tTRUE\t/path\tFALSE\t\tother\tvalue\r\n";
        let (_temp, source, cache) = fixture(data);
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let owner = copy.parent().unwrap().to_owned();
        assert_ne!(source, copy);
        assert_eq!(lease.entries(), 2);
        assert_eq!(lease.bytes(), data.len());
        assert_eq!(fs::read(&copy).unwrap(), data);
        assert_eq!(
            fs::metadata(&copy).unwrap().permissions().mode() & 0o7777,
            0o600
        );
        assert_eq!(
            fs::metadata(&owner).unwrap().permissions().mode() & 0o7777,
            0o700
        );
        assert_eq!(format!("{lease:?}"), "SessionCookieLease(<redacted>)");
        lease.cleanup().unwrap();
        lease.cleanup().unwrap();
        assert!(!copy.exists());
        assert!(!owner.exists());
        assert_eq!(fs::read(&source).unwrap(), data);
        no_copies(&cache);
    }
    #[test]
    fn lease_drop_and_independent_session_copies_preserve_source() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let first = import_session(&source, &cache).unwrap();
        let second = import_session(&source, &cache).unwrap();
        let first_path = first.path_for_tool().to_owned();
        let second_path = second.path_for_tool().to_owned();
        assert_ne!(first_path, second_path);
        drop(first);
        assert!(!first_path.exists());
        assert_eq!(fs::read(&second_path).unwrap(), data);
        drop(second);
        assert_eq!(fs::read(&source).unwrap(), data);
        no_copies(&cache);
    }
    #[test]
    fn invalid_rows_never_create_copy_or_disclose_secret() {
        let rows = [
            "wrong header\n",
            "# Netscape HTTP Cookie File\nmissing-columns\n",
            "# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tname\tvalue\textra\n",
            "# Netscape HTTP Cookie File\n.fixture.invalid\tFALSE\t/\tTRUE\t0\tname\tvalue\n",
            "# Netscape HTTP Cookie File\nfixture.invalid\tFALSE\tbadpath\tTRUE\t0\tname\tvalue\n",
            "# Netscape HTTP Cookie File\nfixture.invalid\tFALSE\t/\tinvalid\t0\tname\tvalue\n",
            "# Netscape HTTP Cookie File\nfixture.invalid\tFALSE\t/\tTRUE\tnot-number\tname\tvalue\n",
            "# Netscape HTTP Cookie File\n#HttpOnly_missing\n",
            "# Netscape HTTP Cookie File\n# synthetic-cookie-secret\0\n",
        ];
        for data in rows {
            let (_temp, source, cache) = fixture(data.as_bytes());
            let error = import_session(&source, &cache).unwrap_err();
            assert_eq!(error, CookieImportError::InvalidFormat);
            let message = format!("{error:?}: {error}");
            assert!(!message.contains("synthetic-cookie-secret"));
            assert!(!message.contains("synthetic-sensitive-source"));
            assert_eq!(fs::read(&source).unwrap(), data.as_bytes());
            no_copies(&cache);
        }
        let (_temp, source, cache) = fixture(b"# Netscape HTTP Cookie File\n\xff\n");
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::InvalidFormat
        );
        no_copies(&cache);
    }
    #[test]
    fn byte_limit_is_exact_and_overflow_preserves_source() {
        let allowed = padded(MAX_COOKIE_BYTES);
        let (_temp, source, cache) = fixture(&allowed);
        let lease = import_session(&source, &cache).unwrap();
        assert_eq!(lease.bytes(), MAX_COOKIE_BYTES);
        assert_eq!(fs::read(lease.path_for_tool()).unwrap(), allowed);
        drop(lease);
        let rejected = padded(MAX_COOKIE_BYTES + 1);
        fs::write(&source, &rejected).unwrap();
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::TooLarge
        );
        assert_eq!(fs::read(&source).unwrap(), rejected);
        no_copies(&cache);
    }
    #[test]
    fn physical_line_limit_is_exact() {
        let mut allowed = valid();
        allowed.push(b'#');
        allowed.resize(allowed.len() + MAX_COOKIE_LINE_BYTES - 1, b'x');
        allowed.push(b'\n');
        let (_temp, source, cache) = fixture(&allowed);
        drop(import_session(&source, &cache).unwrap());
        allowed.insert(allowed.len() - 1, b'x');
        fs::write(&source, &allowed).unwrap();
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::LineTooLong
        );
        no_copies(&cache);
    }
    #[test]
    fn entry_limit_is_exact() {
        let allowed = format!("{HEADER}{}", COOKIE.repeat(MAX_COOKIE_ENTRIES));
        let (_temp, source, cache) = fixture(allowed.as_bytes());
        let lease = import_session(&source, &cache).unwrap();
        assert_eq!(lease.entries(), MAX_COOKIE_ENTRIES);
        drop(lease);
        let rejected = format!("{allowed}{COOKIE}");
        fs::write(&source, rejected.as_bytes()).unwrap();
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::TooManyEntries
        );
        no_copies(&cache);
    }
    #[test]
    fn source_symlink_cache_symlink_and_nonprivate_cache_fail_closed() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let alias = source.with_file_name("source-alias");
        symlink(&source, &alias).unwrap();
        assert_eq!(
            import_session(&alias, &cache).unwrap_err(),
            CookieImportError::SourceUnavailable
        );
        let cache_alias = cache.with_file_name("cache-alias");
        symlink(&cache, &cache_alias).unwrap();
        assert_eq!(
            import_session(&source, &cache_alias).unwrap_err(),
            CookieImportError::UnsafeCache
        );
        fs::set_permissions(&cache, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::UnsafeCache
        );
        assert_eq!(fs::read(&source).unwrap(), data);
        no_copies(&cache);
    }
    #[test]
    fn nonregular_source_is_rejected_without_blocking() {
        let (_temp, source, cache) = fixture(&valid());
        fs::remove_file(&source).unwrap();
        use std::os::unix::ffi::OsStrExt;
        let name = std::ffi::CString::new(source.as_os_str().as_bytes()).unwrap();
        // SAFETY: new owned fixture FIFO pathname, no other process uses it.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::SourceUnavailable
        );
        no_copies(&cache);
    }
    #[test]
    fn changed_cookie_entry_and_symlink_leave_foreign_data_intact() {
        let data = valid();
        for symbolic in [false, true] {
            let (_temp, source, cache) = fixture(&data);
            let mut lease = import_session(&source, &cache).unwrap();
            let copy = lease.path_for_tool().to_owned();
            let foreign = source.with_file_name("owned-foreign-sentinel");
            fs::write(&foreign, b"foreign sentinel").unwrap();
            fs::set_permissions(&foreign, fs::Permissions::from_mode(0o600)).unwrap();
            fs::remove_file(&copy).unwrap();
            if symbolic {
                symlink(&foreign, &copy).unwrap();
            } else {
                fs::rename(&foreign, &copy).unwrap();
            }
            assert_eq!(
                lease.validate_for_tool(),
                Err(CookieImportError::CleanupOwnershipChanged)
            );
            assert_eq!(
                lease.cleanup().unwrap_err(),
                CookieImportError::CleanupOwnershipChanged
            );
            drop(lease);
            assert_eq!(fs::read(&copy).unwrap(), b"foreign sentinel");
            if symbolic {
                assert_eq!(fs::read(&foreign).unwrap(), b"foreign sentinel");
            }
            assert_eq!(fs::read(&source).unwrap(), data);
        }
    }
    #[test]
    fn changed_owner_or_cache_directory_is_never_cleaned() {
        for cache_swap in [false, true] {
            let data = valid();
            let (_temp, source, cache) = fixture(&data);
            let mut lease = import_session(&source, &cache).unwrap();
            let original = if cache_swap {
                cache.clone()
            } else {
                lease.path_for_tool().parent().unwrap().to_owned()
            };
            let saved = source.with_file_name("saved-original-owner");
            fs::rename(&original, &saved).unwrap();
            fs::create_dir(&original).unwrap();
            fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
            let sentinel = original.join("foreign-sentinel");
            fs::write(&sentinel, b"foreign sentinel").unwrap();
            assert_eq!(
                lease.validate_for_tool(),
                Err(CookieImportError::CleanupOwnershipChanged)
            );
            assert_eq!(
                lease.cleanup().unwrap_err(),
                CookieImportError::CleanupOwnershipChanged
            );
            drop(lease);
            assert_eq!(fs::read(&sentinel).unwrap(), b"foreign sentinel");
            assert_eq!(fs::read(&source).unwrap(), data);
            assert!(saved.exists());
        }
    }
    #[test]
    fn no_recursive_cleanup_and_mode_change_refusal() {
        let (_temp, source, cache) = fixture(&valid());
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let sentinel = copy.parent().unwrap().join("foreign-sentinel");
        fs::write(&sentinel, b"foreign sentinel").unwrap();
        assert_eq!(
            lease.cleanup().unwrap_err(),
            CookieImportError::CleanupOwnershipChanged
        );
        #[cfg(target_os = "linux")]
        assert!(copy.exists());
        #[cfg(not(target_os = "linux"))]
        assert!(!copy.exists());
        drop(lease);
        assert_eq!(fs::read(&sentinel).unwrap(), b"foreign sentinel");
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        fs::set_permissions(&copy, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            lease.cleanup().unwrap_err(),
            CookieImportError::CleanupOwnershipChanged
        );
        drop(lease);
        assert_eq!(fs::read(&copy).unwrap(), valid());
    }
    #[test]
    fn tool_validation_accepts_live_lease_and_rejects_cleaned_lease() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let mut lease = import_session(&source, &cache).unwrap();
        lease.validate_for_tool().unwrap();
        lease.cleanup().unwrap();
        lease.cleanup().unwrap();
        assert_eq!(
            lease.validate_for_tool(),
            Err(CookieImportError::CleanupOwnershipChanged)
        );
        assert_eq!(fs::read(&source).unwrap(), data);
        no_copies(&cache);
    }
    #[test]
    fn tool_validation_rejects_changed_private_permissions() {
        for target in 0..3 {
            let data = valid();
            let (_temp, source, cache) = fixture(&data);
            let lease = import_session(&source, &cache).unwrap();
            let changed = match target {
                0 => lease.path_for_tool().to_owned(),
                1 => lease.path_for_tool().parent().unwrap().to_owned(),
                _ => cache.clone(),
            };
            let original_mode = if target == 0 { 0o600 } else { 0o700 };
            fs::set_permissions(&changed, fs::Permissions::from_mode(original_mode | 0o044))
                .unwrap();
            assert_eq!(
                lease.validate_for_tool(),
                Err(CookieImportError::CleanupOwnershipChanged)
            );
            fs::set_permissions(&changed, fs::Permissions::from_mode(original_mode)).unwrap();
            lease.validate_for_tool().unwrap();
            drop(lease);
            assert_eq!(fs::read(&source).unwrap(), data);
            no_copies(&cache);
        }
    }
    #[test]
    fn tool_validation_rejects_missing_copy_and_extra_hardlink() {
        let (_temp, source, cache) = fixture(&valid());
        let lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let link = source.with_file_name("owned-extra-link");
        fs::hard_link(&copy, &link).unwrap();
        assert_eq!(
            lease.validate_for_tool(),
            Err(CookieImportError::CleanupOwnershipChanged)
        );
        fs::remove_file(&link).unwrap();
        lease.validate_for_tool().unwrap();
        fs::remove_file(&copy).unwrap();
        assert_eq!(
            lease.validate_for_tool(),
            Err(CookieImportError::CleanupOwnershipChanged)
        );
        drop(lease);
        #[cfg(not(target_os = "linux"))]
        no_copies(&cache);
        #[cfg(target_os = "linux")]
        assert!(copy.parent().unwrap().join("recovery.json").exists());
        assert_eq!(fs::read(&source).unwrap(), valid());
    }
    #[test]
    fn deferred_cleanup_preserves_private_copy_and_invalidates_lease() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let owner = copy.parent().unwrap().to_owned();
        lease.validate_for_tool().unwrap();
        #[cfg(target_os = "linux")]
        let matching_handles = || {
            fs::read_dir("/proc/self/fd")
                .unwrap()
                .filter_map(Result::ok)
                .filter_map(|entry| fs::read_link(entry.path()).ok())
                .filter(|path| path == &cache || path.starts_with(&owner))
                .count()
        };
        #[cfg(target_os = "linux")]
        assert!(matching_handles() >= 3);
        lease.defer_cleanup();
        #[cfg(target_os = "linux")]
        assert_eq!(matching_handles(), 0);
        assert_eq!(
            lease.validate_for_tool(),
            Err(CookieImportError::CleanupOwnershipChanged)
        );
        lease.defer_cleanup();
        lease.cleanup().unwrap();
        assert_eq!(fs::read(&copy).unwrap(), data);
        assert_eq!(
            fs::metadata(&copy).unwrap().permissions().mode() & 0o7777,
            0o600
        );
        assert_eq!(
            fs::metadata(&owner).unwrap().permissions().mode() & 0o7777,
            0o700
        );
        assert_eq!(format!("{lease:?}"), "SessionCookieLease(<redacted>)");
        drop(lease);
        assert_eq!(fs::read(&copy).unwrap(), data);
        assert_eq!(fs::read(&source).unwrap(), data);
        // Owned TempDir alone removes this synthetic fixture after assertions.
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn current_boot_recovery_defers_durable_private_record_without_secrets() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let owner = copy.parent().unwrap();
        for name in ["recovery.json", "owner.marker"] {
            let path = owner.join(name);
            let bytes = fs::read(&path).unwrap();
            assert!(bytes.len() <= if name == "owner.marker" { 512 } else { 4096 });
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("synthetic-cookie-secret"));
            assert!(!text.contains(source.to_str().unwrap()));
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o7777,
                0o600
            );
        }
        lease.defer_cleanup();
        let result = dlpad_engine::cookies::recover_sessions(&cache).unwrap();
        assert_eq!(
            (
                result.scanned,
                result.deferred,
                result.recovered,
                result.rejected
            ),
            (1, 1, 0, 0)
        );
        assert!(copy.exists());
        assert_eq!(fs::read(source).unwrap(), data);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn capacity_and_scan_count_foreign_entries_without_deleting_them() {
        use dlpad_engine::cookies::{MAX_COOKIE_SESSIONS, recover_sessions};
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        for index in 0..MAX_COOKIE_SESSIONS + 7 {
            fs::write(cache.join(format!("foreign-{index}")), b"foreign").unwrap();
        }
        assert_eq!(
            import_session(&source, &cache).unwrap_err(),
            CookieImportError::CacheFull
        );
        let result = recover_sessions(&cache).unwrap();
        assert_eq!(
            (result.scanned, result.rejected, result.recovered),
            (MAX_COOKIE_SESSIONS, MAX_COOKIE_SESSIONS, 0)
        );
        assert!(result.truncated);
        assert_eq!(
            fs::read_dir(&cache).unwrap().count(),
            MAX_COOKIE_SESSIONS + 7
        );
        assert_eq!(fs::read(source).unwrap(), data);
    }
    #[test]
    fn confirmed_reader_allows_normal_cleanup_and_reuse() {
        let (_temp, source, cache) = fixture(&valid());
        let mut lease = import_session(&source, &cache).unwrap();
        let guard = lease.begin_reader().unwrap();
        assert_eq!(format!("{guard:?}"), "SessionCookieReader(<redacted>)");
        assert_eq!(
            lease.begin_reader().unwrap_err(),
            CookieImportError::ReaderUnconfirmed
        );
        guard.confirm();
        lease.begin_reader().unwrap().confirm();
        lease.cleanup().unwrap();
        no_copies(&cache);
        assert!(source.exists());
    }
    #[test]
    fn unconfirmed_reader_explicit_cleanup_preserves_copy() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let mut lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        drop(lease.begin_reader().unwrap());
        assert_eq!(lease.cleanup(), Err(CookieImportError::ReaderUnconfirmed));
        assert_eq!(
            lease.validate_for_tool(),
            Err(CookieImportError::CleanupOwnershipChanged)
        );
        assert_eq!(lease.cleanup(), Err(CookieImportError::ReaderUnconfirmed));
        drop(lease);
        assert_eq!(fs::read(&copy).unwrap(), data);
        assert_eq!(fs::read(source).unwrap(), data);
        #[cfg(target_os = "linux")]
        assert!(copy.parent().unwrap().join("recovery.json").exists());
    }
    #[test]
    fn reader_unwind_preserves_copy_and_releases_all_owned_handles() {
        let data = valid();
        let (_temp, source, cache) = fixture(&data);
        let lease = import_session(&source, &cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let owner = copy.parent().unwrap().to_owned();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _reader = lease.begin_reader().unwrap();
            panic!("synthetic worker panic");
        }));
        assert!(result.is_err());
        assert_eq!(fs::read(&copy).unwrap(), data);
        assert_eq!(fs::read(source).unwrap(), data);
        #[cfg(target_os = "linux")]
        {
            assert!(owner.join("recovery.json").exists());
            assert!(owner.join("owner.marker").exists());
            let handles = fs::read_dir("/proc/self/fd")
                .unwrap()
                .filter_map(Result::ok)
                .filter_map(|entry| fs::read_link(entry.path()).ok())
                .filter(|path| path == &cache || path.starts_with(&owner))
                .count();
            assert_eq!(handles, 0);
        }
        #[cfg(not(target_os = "linux"))]
        assert!(owner.exists());
    }
}
