//! Windows startup recovery requires original native bindings and actual receipts.
use super::*;
use crate::cookies::CookieRecoverySummary;

pub(crate) fn recover(cache_path: &Path) -> Result<CookieRecoverySummary, CookieImportError> {
    let sid = current_sid()?;
    let (root, names) = path_parts(cache_path).map_err(|_| CookieImportError::UnsafeCache)?;
    let cache_pins = chain(root, &names).map_err(|_| CookieImportError::UnsafeCache)?;
    let cache = cache_pins.last().ok_or_else(changed)?;
    acl(&cache.file, &sid).map_err(|_| CookieImportError::UnsafeCache)?;
    let mut summary = CookieRecoverySummary::default();
    for item in fs::read_dir(cache_path)
        .map_err(|_| changed())?
        .take(MAX_COOKIE_SESSIONS + 1)
    {
        if summary.scanned == MAX_COOKIE_SESSIONS {
            summary.truncated = true;
            break;
        }
        summary.scanned += 1;
        let entry = match item {
            Ok(e) => e,
            Err(_) => {
                summary.rejected += 1;
                continue;
            }
        };
        let name = entry.file_name();
        let valid = name.to_str().is_some_and(|s| {
            s.strip_prefix("cookie-session-").is_some_and(|suffix| {
                suffix.len() == 64
                    && suffix
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        });
        if !valid {
            summary.rejected += 1;
            continue;
        }
        match recover_one(cache_path, &name, &sid) {
            Ok(true) => summary.recovered += 1,
            Ok(false) => summary.deferred += 1,
            Err(_) => summary.rejected += 1,
        }
    }
    Ok(summary)
}
fn recover_one(cache_path: &Path, name: &OsString, sid: &str) -> Result<bool, CookieImportError> {
    let (root, names) = path_parts(cache_path)?;
    let pins = chain(root, &names)?;
    let owner_path = cache_path.join(name);
    let owner = open(
        &owner_path,
        true,
        META_ACCESS | FILE_LIST_DIRECTORY | DELETE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
    )?;
    acl(&owner, sid)?;
    let marker_file = open(
        &owner_path.join(recovery_marker::MARKER_NAME),
        false,
        FILE_GENERIC_READ | READ_CONTROL,
        SHARE_ALL,
        None,
    )?;
    acl(&marker_file, sid)?;
    let marker = recovery_marker::read_marker(&marker_file).map_err(|_| changed())?;
    let copy_path = owner_path.join("cookies.txt");
    let copy = open(&copy_path, false, META_ACCESS, SHARE_ALL, None)?;
    marker
        .check_actual(
            sid,
            [
                &pins.last().ok_or_else(changed)?.file,
                &owner,
                &copy,
                &marker_file,
            ],
        )
        .map_err(|_| changed())?;
    let (anchors, nonce, digest) = marker.journal_seed();
    let journal = match journal::Journal::reopen(&owner, &owner_path, sid, anchors, nonce, digest) {
        Ok(journal) => journal,
        Err(journal::JournalError::Busy) => return Ok(false),
        Err(_) => return Err(changed()),
    };
    // Never use pristine, boot guesses, persisted Resolved, PID, or mtime as recovery authority.
    if !journal.all_receipts_match().map_err(|_| changed())? {
        return Ok(false);
    }
    let marker_id = file_identity::from_file(&marker_file).map_err(|_| changed())?;
    let mut owned = OwnedCopy {
        owner_id: file_identity::from_file(&owner).map_err(|_| changed())?,
        copy_id: file_identity::from_file(&copy).map_err(|_| changed())?,
        pins,
        owner: Some(owner),
        owner_path: SensitivePath(owner_path),
        copy: Some(copy),
        copy_path: SensitivePath(copy_path),
        sid: sid.to_owned(),
        removed: false,
        initialization_started: true,
        journal: Some(std::sync::Mutex::new(journal)),
        marker: Some((marker_file, marker, marker_id)),
    };
    let result = owned.cleanup();
    if result.is_err() {
        owned.defer_cleanup();
    }
    result.map(|_| true)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn imported() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.txt");
        fs::write(
            &source,
            b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-only\n",
        )
        .unwrap();
        let cache = temp.path().join("cache");
        prepare_cache(&cache).unwrap();
        fs::write(cache.join("foreign-sentinel"), b"preserve").unwrap();
        let (mut owned, copy, _, _) = import(&source, &cache).unwrap();
        owned.defer_cleanup();
        drop(owned);
        (temp, source, cache, copy)
    }
    #[test]
    fn public_recovery_preserves_pristine_without_receipts_and_foreign_source() {
        let (_temp, source, cache, copy) = imported();
        let before = fs::read(&copy).unwrap();
        let summary = crate::cookies::recover_sessions(&cache).unwrap();
        assert_eq!(summary.scanned, 2);
        assert_eq!(summary.deferred, 1);
        assert_eq!(summary.recovered, 0);
        assert_eq!(summary.rejected, 1);
        assert!(!summary.truncated);
        assert_eq!(fs::read(copy).unwrap(), before);
        assert_eq!(fs::read(source).unwrap(), before);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn malformed_marker_preserves_original_copy_and_sentinel() {
        let (_temp, source, cache, copy) = imported();
        let before = fs::read(&copy).unwrap();
        fs::write(
            copy.parent().unwrap().join(recovery_marker::MARKER_NAME),
            b"{}",
        )
        .unwrap();
        let summary = crate::cookies::recover_sessions(&cache).unwrap();
        assert_eq!(summary.recovered, 0);
        assert_eq!(summary.rejected, 2);
        assert_eq!(fs::read(copy).unwrap(), before);
        assert_eq!(fs::read(source).unwrap(), before);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn recovery_bounds_foreign_scan_and_does_not_delete_entries() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        prepare_cache(&cache).unwrap();
        for i in 0..130 {
            fs::write(cache.join(format!("foreign-{i}")), b"preserve").unwrap();
        }
        let summary = crate::cookies::recover_sessions(&cache).unwrap();
        assert_eq!(summary.scanned, MAX_COOKIE_SESSIONS);
        assert!(summary.truncated);
        assert_eq!(summary.recovered, 0);
        assert_eq!(fs::read_dir(cache).unwrap().count(), 130);
    }
}
