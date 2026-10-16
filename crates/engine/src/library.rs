//! Streaming filesystem reconciliation and bounded keyset pagination.
mod identity;
#[cfg(windows)]
mod windows_scan;

use crate::store::Store;
use rusqlite::params;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::UNIX_EPOCH,
};

pub const MAX_PAGE_SIZE: usize = 256;
pub const MAX_SCAN_DEPTH: usize = 128;
// 32766 Windows UTF-16 units fit in at most 98298 UTF-8 bytes, including
// non-ASCII native paths. The larger bound preserves those paths.
const MAX_LIBRARY_PATH_BYTES: usize = 128 * 1024;
const MAX_LIBRARY_IDENTITY_BYTES: usize = 128;

#[derive(Debug)]
pub enum LibraryError {
    Io(io::Error),
    Database(rusqlite::Error),
    DocumentsUnavailable,
    UnsupportedPath,
    DepthLimit,
    Cancelled,
}
impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for LibraryError {}
impl From<io::Error> for LibraryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for LibraryError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified_ms: Option<i64>,
    pub file_identity: Option<String>,
}

/// Resolves the OS document directory without creating any user directory.
pub fn default_library_root() -> Result<PathBuf, LibraryError> {
    directories::UserDirs::new()
        .and_then(|d| d.document_dir().map(|p| p.join("dlpad")))
        .ok_or(LibraryError::DocumentsUnavailable)
}

/// Explicitly called only after selecting a save root.
pub fn create_library_root(root: &Path) -> Result<(), LibraryError> {
    fs::create_dir_all(root)?;
    Ok(())
}

fn text_path(path: &Path) -> Result<&str, LibraryError> {
    path.to_str().ok_or(LibraryError::UnsupportedPath)
}

fn partial(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name == ".dlpad-partials"
        || name.ends_with(".part")
        || name.ends_with(".ytdl")
        || name.ends_with(".tmp")
        || name.contains(".part-Frag")
}

struct FileObservation {
    path: PathBuf,
    size: u64,
    modified: Option<i64>,
    file_identity: Option<String>,
}
#[cfg(windows)]
fn native_unsupported(error: &LibraryError) -> bool {
    matches!(error, LibraryError::Io(error) if error.kind() == io::ErrorKind::Unsupported)
}
fn conservative_next(
    stack: &mut Vec<fs::ReadDir>,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Option<FileObservation>, LibraryError> {
    while let Some(directory) = stack.last_mut() {
        if cancelled() {
            return Err(LibraryError::Cancelled);
        }
        let Some(entry) = directory.next() else {
            stack.pop();
            continue;
        };
        let path = entry?.path();
        if partial(&path) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if stack.len() >= MAX_SCAN_DEPTH {
                return Err(LibraryError::DepthLimit);
            }
            stack.push(fs::read_dir(path)?);
        } else if metadata.is_file() {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .and_then(|time| i64::try_from(time.as_millis()).ok());
            let file_identity = identity::observe(&path, &metadata);
            return Ok(Some(FileObservation {
                path,
                size: metadata.len(),
                modified,
                file_identity,
            }));
        }
    }
    Ok(None)
}

#[cfg(windows)]
fn observations_match_previous(
    transaction: &rusqlite::Transaction<'_>,
    root: &str,
    observed_count: usize,
) -> Result<bool, LibraryError> {
    let count = i64::try_from(observed_count).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "library count exceeds SQLite integer",
        )
    })?;
    let previous_count: i64 = transaction.query_row(
        "SELECT count(*) FROM library_entries WHERE root=?1",
        [root],
        |row| row.get(0),
    )?;
    if previous_count != count {
        return Ok(false);
    }
    // Equal cardinality and a unique root/path match for every previous row
    // establish the entire previous set, including absent rows and NULL fields.
    // Storage-class checks prevent SQLite numeric equality hiding corrupt rows.
    let changed: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM library_entries previous
         LEFT JOIN library_observations observed ON observed.path=previous.path
         WHERE previous.root=?1 AND (observed.path IS NULL OR typeof(previous.path)!='text'
            OR typeof(previous.size)!='integer' OR previous.size IS NOT observed.size
            OR typeof(previous.modified_ms) NOT IN ('integer','null')
            OR previous.modified_ms IS NOT observed.modified_ms
            OR typeof(previous.file_identity) NOT IN ('text','null')
            OR previous.file_identity IS NOT observed.file_identity
            OR typeof(previous.present)!='integer' OR previous.present IS NOT 1))",
        [root],
        |row| row.get(0),
    )?;
    Ok(!changed)
}

#[cfg(windows)]
fn verify_native_rename_candidates(
    transaction: &rusqlite::Transaction<'_>,
    root: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<(), LibraryError> {
    // Kernel records are snapshots, not live authority to transfer a previous record.
    // Only unique, genuinely moved candidates require the original per-file evidence.
    let missing_previous_path: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM library_entries previous
           WHERE previous.root=?1 AND previous.file_identity IS NOT NULL
             AND NOT EXISTS(SELECT 1 FROM library_observations observed
                            WHERE observed.path=previous.path))",
        [root],
        |row| row.get(0),
    )?;
    if !missing_previous_path {
        return Ok(());
    }

    let mut statement = transaction.prepare(
        "SELECT observed.path,observed.size,observed.modified_ms,observed.file_identity
         FROM library_observations observed
         WHERE observed.file_identity IS NOT NULL
           AND NOT EXISTS(SELECT 1 FROM library_entries occupied
                WHERE occupied.root=?1 AND occupied.path=observed.path)
           AND (SELECT count(*) FROM library_observations same
                WHERE same.file_identity=observed.file_identity)=1
           AND (SELECT count(*) FROM library_entries previous
                WHERE previous.root=?1 AND previous.file_identity=observed.file_identity)=1
           AND EXISTS(SELECT 1 FROM library_entries previous
                WHERE previous.root=?1 AND previous.file_identity=observed.file_identity
                  AND NOT EXISTS(SELECT 1 FROM library_observations old_path
                                 WHERE old_path.path=previous.path))",
    )?;
    let mut rows = statement.query([root])?;
    while let Some(row) = rows.next()? {
        if cancelled() {
            return Err(LibraryError::Cancelled);
        }
        let path = PathBuf::from(row.get::<_, String>(0)?);
        let expected_size: i64 = row.get(1)?;
        let expected_modified: Option<i64> = row.get(2)?;
        let expected_identity: String = row.get(3)?;
        let metadata = fs::symlink_metadata(&path)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .and_then(|time| i64::try_from(time.as_millis()).ok());
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || i64::try_from(metadata.len()).ok() != Some(expected_size)
            || modified != expected_modified
            || identity::observe(&path, &metadata).as_deref() != Some(expected_identity.as_str())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "library rename candidate changed before publication",
            )
            .into());
        }
    }
    Ok(())
}

impl Store {
    /// Commits observations only after the entire root was read successfully.
    /// Inaccessible directories, oversized depth, or unsupported names roll back,
    /// preserving the previous snapshot instead of treating disconnection as deletion.
    pub fn reconcile_library(&mut self, root: &Path) -> Result<usize, LibraryError> {
        self.reconcile_library_checked(root, || false)
    }

    /// Cooperatively stops at scan/publication boundaries, preserving the last
    /// committed snapshot. Individual filesystem and SQLite calls are not interrupted.
    pub fn reconcile_library_with_cancel(
        &mut self,
        root: &Path,
        cancel: &AtomicBool,
    ) -> Result<usize, LibraryError> {
        self.reconcile_library_checked(root, || cancel.load(Ordering::Acquire))
    }

    /// Runs a bounded caller policy at existing cooperative cancellation boundaries.
    /// Returning true rolls back unpublished observations. The callback executes on
    /// the caller's thread; individual filesystem/SQLite calls remain uninterrupted.
    pub fn reconcile_library_with_checkpoint(
        &mut self,
        root: &Path,
        cancel: &AtomicBool,
        mut checkpoint: impl FnMut() -> bool,
    ) -> Result<usize, LibraryError> {
        self.reconcile_library_checked(root, || cancel.load(Ordering::Acquire) || checkpoint())
    }

    /// Explicit fixture seam after the first real file observation has been inserted.
    /// The callback runs on the scan caller's thread inside the unpublished transaction.
    #[cfg(feature = "library-scan-closewait-fixture")]
    pub fn reconcile_library_after_first_observation(
        &mut self,
        root: &Path,
        cancel: &AtomicBool,
        after_first: impl FnOnce() -> Result<(), LibraryError>,
    ) -> Result<usize, LibraryError> {
        self.reconcile_library_inner(root, || cancel.load(Ordering::Acquire), Some(after_first))
    }

    /// Owned developer fixture combining normal checkpoints with a real INSERT barrier.
    #[cfg(feature = "library-scan-closewait-fixture")]
    pub fn reconcile_library_with_checkpoint_after_first(
        &mut self,
        root: &Path,
        cancel: &AtomicBool,
        mut checkpoint: impl FnMut() -> bool,
        after_first: impl FnOnce() -> Result<(), LibraryError>,
    ) -> Result<usize, LibraryError> {
        self.reconcile_library_inner(
            root,
            || cancel.load(Ordering::Acquire) || checkpoint(),
            Some(after_first),
        )
    }

    fn reconcile_library_checked(
        &mut self,
        root: &Path,
        cancelled: impl FnMut() -> bool,
    ) -> Result<usize, LibraryError> {
        self.reconcile_library_inner(root, cancelled, None::<fn() -> Result<(), LibraryError>>)
    }

    fn reconcile_library_inner(
        &mut self,
        root: &Path,
        mut cancelled: impl FnMut() -> bool,
        mut after_first: Option<impl FnOnce() -> Result<(), LibraryError>>,
    ) -> Result<usize, LibraryError> {
        let check = |cancelled: &mut dyn FnMut() -> bool| {
            if cancelled() {
                Err(LibraryError::Cancelled)
            } else {
                Ok(())
            }
        };
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let _scan_profile = scan_profile::Scan::new();
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let preflight_profile = scan_profile::Phase::new(0);
        check(&mut cancelled)?;
        let root_key = text_path(root)?;
        if fs::symlink_metadata(root)?.file_type().is_symlink() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "library root is a symlink").into(),
            );
        }
        #[cfg(windows)]
        let mut native = if root.is_absolute()
            && !root
                .components()
                .any(|part| part == std::path::Component::ParentDir)
        {
            match windows_scan::NativeWalker::new(root) {
                Ok(walker) => walker,
                Err(error) if native_unsupported(&error) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        #[cfg(windows)]
        let native_enabled = if let Some(walker) = native.as_mut() {
            loop {
                match walker.preflight_next(&mut cancelled) {
                    Ok(Some(())) => (),
                    Ok(None) => {
                        walker.verify_root()?;
                        walker.restart()?;
                        break true;
                    }
                    Err(error) if native_unsupported(&error) => {
                        walker.verify_root()?;
                        break false;
                    }
                    Err(error) => return Err(error),
                }
            }
        } else {
            false
        };
        #[cfg(windows)]
        let mut stack = if native_enabled {
            Vec::new()
        } else {
            vec![fs::read_dir(root)?]
        };
        #[cfg(not(windows))]
        let mut stack = vec![fs::read_dir(root)?];
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(preflight_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let setup_profile = scan_profile::Phase::new(1);
        let transaction = self.connection.transaction()?;
        transaction.execute_batch(
            "CREATE TEMP TABLE IF NOT EXISTS library_observations(
                path TEXT PRIMARY KEY, size INTEGER NOT NULL,
                modified_ms INTEGER, file_identity TEXT);
             DROP INDEX IF EXISTS temp.library_observation_identity;
             DELETE FROM library_observations;",
        )?;
        // Reuse one prepared insertion across bounded streaming observations.
        // Statement drops before publication queries/transaction commit.
        let mut observation_insert = transaction.prepare(
            "INSERT INTO library_observations(path,size,modified_ms,file_identity)
             VALUES(?1,?2,?3,?4)",
        )?;
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(setup_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let traversal_insert_profile = scan_profile::Phase::new(2);
        let mut count = 0;
        loop {
            check(&mut cancelled)?;
            #[cfg(windows)]
            let next = if native_enabled {
                native
                    .as_mut()
                    .ok_or_else(|| io::Error::other("native scanner missing"))?
                    .next(&mut cancelled)?
                    .map(|row| FileObservation {
                        path: row.path,
                        size: row.size,
                        modified: row.modified_ms,
                        file_identity: Some(row.file_identity),
                    })
            } else {
                conservative_next(&mut stack, &mut cancelled)?
            };
            #[cfg(not(windows))]
            let next = conservative_next(&mut stack, &mut cancelled)?;
            let Some(row) = next else {
                break;
            };
            let size = i64::try_from(row.size).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "file exceeds SQLite size range")
            })?;
            observation_insert.execute(params![
                text_path(&row.path)?,
                size,
                row.modified,
                row.file_identity
            ])?;
            count += 1;
            if let Some(after_first) = after_first.take() {
                after_first()?;
                check(&mut cancelled)?;
            }
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(traversal_insert_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let candidate_profile = scan_profile::Phase::new(3);
        drop(observation_insert);
        check(&mut cancelled)?;
        #[cfg(windows)]
        let publication_needed =
            !native_enabled || !observations_match_previous(&transaction, root_key, count)?;
        #[cfg(not(windows))]
        let publication_needed = true;
        // Identity lookup is needed only for publication/rename processing.
        // Keep the path primary key during every observation to reject duplicates.
        if publication_needed {
            transaction.execute_batch(
                "CREATE INDEX library_observation_identity
                 ON library_observations(file_identity);",
            )?;
        }
        #[cfg(windows)]
        if native_enabled && publication_needed {
            verify_native_rename_candidates(&transaction, root_key, &mut cancelled)?;
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(candidate_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let update_profile = scan_profile::Phase::new(4);
        // An exact previous set has no rename candidate or changed row.
        if publication_needed {
            transaction.execute(
                "UPDATE library_entries AS old SET path=(
                SELECT observed.path FROM library_observations observed
                WHERE observed.file_identity=old.file_identity)
             WHERE old.root=?1 AND old.file_identity IS NOT NULL
               AND NOT EXISTS(SELECT 1 FROM library_observations WHERE path=old.path)
               AND (SELECT count(*) FROM library_entries previous
                    WHERE previous.root=?1 AND previous.file_identity=old.file_identity)=1
               AND (SELECT count(*) FROM library_observations observed
                    WHERE observed.file_identity=old.file_identity)=1
               AND NOT EXISTS(SELECT 1 FROM library_entries occupied
                    WHERE occupied.root=?1 AND occupied.path=(
                        SELECT observed.path FROM library_observations observed
                        WHERE observed.file_identity=old.file_identity))",
                [root_key],
            )?;
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(update_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let upsert_profile = scan_profile::Phase::new(5);
        if publication_needed {
            transaction.execute(
                "INSERT INTO library_entries(root,path,size,modified_ms,present,file_identity)
             SELECT ?1,path,size,modified_ms,1,file_identity FROM library_observations WHERE 1
             ON CONFLICT(root,path) DO UPDATE SET size=excluded.size,
                modified_ms=excluded.modified_ms,present=1,file_identity=excluded.file_identity",
                [root_key],
            )?;
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(upsert_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let delete_profile = scan_profile::Phase::new(6);
        if publication_needed {
            transaction.execute(
                "DELETE FROM library_entries WHERE root=?1 AND NOT EXISTS(
                SELECT 1 FROM library_observations WHERE path=library_entries.path)",
                [root_key],
            )?;
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(delete_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let clear_profile = scan_profile::Phase::new(7);
        transaction.execute("DELETE FROM library_observations", [])?;
        check(&mut cancelled)?;
        #[cfg(windows)]
        if let Some(walker) = native.as_ref() {
            walker.verify_root()?;
        }
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        drop(clear_profile);
        #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
        let _commit_profile = scan_profile::Phase::new(8);
        transaction.commit()?;
        Ok(count)
    }

    /// Read a bounded page from an existing database without creating or migrating it.
    /// This is blocking I/O, with the same five-second SQLite busy bound as Store.
    pub fn read_library_page_readonly(
        database: &Path,
        root: &Path,
        after: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<LibraryEntry>, LibraryError> {
        let connection = rusqlite::Connection::open_with_flags(
            database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        Self { connection }.library_page(root, after, limit)
    }

    /// Existing-file readonly search with the same busy bound as readonly page reads.
    pub fn read_library_search_page_readonly(
        database: &Path,
        root: &Path,
        after: Option<&Path>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<LibraryEntry>, LibraryError> {
        let connection = rusqlite::Connection::open_with_flags(
            database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        Self { connection }.library_search_page(root, after, query, limit)
    }

    pub fn library_page(
        &self,
        root: &Path,
        after: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<LibraryEntry>, LibraryError> {
        self.library_filtered_page(root, after, "", limit)
    }

    /// Literal substring over the committed database, never a filesystem scan.
    /// Case folding follows SQLite lower(): ASCII case-insensitive, other Unicode literal.
    pub fn library_search_page(
        &self,
        root: &Path,
        after: Option<&Path>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<LibraryEntry>, LibraryError> {
        self.library_filtered_page(root, after, query, limit.min(128))
    }
    fn library_filtered_page(
        &self,
        root: &Path,
        after: Option<&Path>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<LibraryEntry>, LibraryError> {
        let query = query.trim();
        if query.len() > 256 || query.contains('\0') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid bounded search query",
            )
            .into());
        }
        let mut statement = self.connection.prepare(
            "SELECT
             CASE WHEN typeof(path)='text' AND length(CAST(path AS BLOB)) BETWEEN 1 AND ?4
                  THEN path ELSE NULL END,
             size,modified_ms,
             CASE WHEN file_identity IS NULL OR
                  (typeof(file_identity)='text' AND length(CAST(file_identity AS BLOB))<=?5)
                  THEN file_identity ELSE NULL END,
             typeof(path)='text' AND length(CAST(path AS BLOB)) BETWEEN 1 AND ?4
             AND (file_identity IS NULL OR
                  (typeof(file_identity)='text' AND length(CAST(file_identity AS BLOB))<=?5))
             FROM library_entries WHERE root=?1 AND present=1 AND path>?2
             AND (?6='' OR CASE WHEN typeof(path)!='text' OR length(CAST(path AS BLOB)) NOT BETWEEN 1 AND ?4 THEN 1 ELSE instr(lower(path),lower(?6))>0 END)
             ORDER BY path LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![
                text_path(root)?,
                after.map(text_path).transpose()?.unwrap_or(""),
                limit.min(MAX_PAGE_SIZE) as i64,
                MAX_LIBRARY_PATH_BYTES as i64,
                MAX_LIBRARY_IDENTITY_BYTES as i64,
                query
            ],
            |row| {
                // Read the small validity result before any String decoding. CASE
                // removes rejected payloads from result columns; do not silently
                // skip a corrupt row or change keyset ordering.
                if !row.get::<_, bool>(4)? {
                    return Err(rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Integer,
                        Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "library field exceeds bounded text contract",
                        )),
                    ));
                }
                Ok(LibraryEntry {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    size: row.get::<_, i64>(1)? as u64,
                    modified_ms: row.get(2)?,
                    file_identity: row.get(3)?,
                })
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    // Insert in library.rs tests ONLY after root review. Never production scheduling.
    // Insert in library.rs tests ONLY after root review. Never production scheduling.
    #[test]
    fn search_full_committed_database_literal_hangul_keyset_and_clear() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("media");
        let db = dir.path().join("library.sqlite");
        let writer = Store::open(&db).unwrap();
        for index in 0..140 {
            let path = root
                .join("nested 한글")
                .join(format!("영상_%'_{index:03}.mp4"));
            writer
                .connection
                .execute(
                    "INSERT INTO library_entries(root,path,size,present) VALUES(?1,?2,3,1)",
                    params![text_path(&root).unwrap(), text_path(&path).unwrap()],
                )
                .unwrap();
        }
        let other = dir.path().join("other");
        writer
            .connection
            .execute(
                "INSERT INTO library_entries(root,path,size,present) VALUES(?1,?2,3,1)",
                params![
                    text_path(&other).unwrap(),
                    text_path(&other.join("영상_%'foreign.mp4")).unwrap()
                ],
            )
            .unwrap();
        let original = fs::read(&db).unwrap();
        let first =
            Store::read_library_search_page_readonly(&db, &root, None, "  영상_%'  ", 500).unwrap();
        assert_eq!(first.len(), 128);
        let second = Store::read_library_search_page_readonly(
            &db,
            &root,
            Some(&first.last().unwrap().path),
            "영상_%'",
            500,
        )
        .unwrap();
        assert_eq!(second.len(), 12);
        assert!(
            second
                .iter()
                .all(|row| row.path > first.last().unwrap().path)
        );
        assert!(
            Store::read_library_search_page_readonly(&db, &root, None, "literal-not-in-db", 128)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            Store::read_library_search_page_readonly(&db, &root, None, " ", 128).unwrap(),
            writer.library_page(&root, None, 128).unwrap()
        );
        assert_eq!(
            writer.library_page(&root, None, 500).unwrap().len(),
            140,
            "legacy256 cap retained independently of new128 search cap"
        );
        assert_eq!(
            fs::read(&db).unwrap(),
            original,
            "readonly searches never write/migrate DB"
        );
        for query in ["x".repeat(257), "한".repeat(86), "a\0b".into()] {
            assert!(
                Store::read_library_search_page_readonly(&db, &root, None, &query, 128).is_err()
            );
        }
    }
    #[test]
    fn search_readonly_noncreate_busy_corrupt_and_oversize_preserve_originals() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("media");
        let db = dir.path().join("library.sqlite");
        assert!(Store::read_library_search_page_readonly(&db, &root, None, "q", 128).is_err());
        assert!(!db.exists());
        fs::write(&db, b"invalid sqlite preserve").unwrap();
        assert!(Store::read_library_search_page_readonly(&db, &root, None, "q", 128).is_err());
        assert_eq!(fs::read(&db).unwrap(), b"invalid sqlite preserve");
        fs::remove_file(&db).unwrap();
        let writer = Store::open(&db).unwrap();
        writer
            .connection
            .execute(
                "INSERT INTO library_entries(root,path,size,present) VALUES(?1,?2,3,1)",
                params![
                    text_path(&root).unwrap(),
                    "x".repeat(MAX_LIBRARY_PATH_BYTES + 1)
                ],
            )
            .unwrap();
        let original = fs::read(&db).unwrap();
        assert!(
            Store::read_library_search_page_readonly(&db, &root, None, "not matching", 128)
                .is_err(),
            "invalid bounded path cannot be hidden by search"
        );
        assert_eq!(fs::read(&db).unwrap(), original);
        writer.connection.execute_batch("BEGIN EXCLUSIVE").unwrap();
        assert!(
            matches!(Store::read_library_search_page_readonly(&db,&root,None,"q",128),Err(LibraryError::Database(rusqlite::Error::SqliteFailure(error,_)))if error.code==rusqlite::ErrorCode::DatabaseBusy)
        );
        writer.connection.execute_batch("ROLLBACK").unwrap();
        assert_eq!(fs::read(&db).unwrap(), original);
    }
    #[test]
    fn readonly_page_preserves_missing_empty_legacy_and_corrupt_databases() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("media");
        fs::create_dir(&root).unwrap();
        let database = dir.path().join("library.sqlite");
        assert!(Store::read_library_page_readonly(&database, &root, None, 128).is_err());
        assert!(!database.exists());
        fs::write(&database, b"").unwrap();
        assert!(Store::read_library_page_readonly(&database, &root, None, 128).is_err());
        assert_eq!(fs::read(&database).unwrap(), b"");
        let legacy = rusqlite::Connection::open(&database).unwrap();
        legacy.execute_batch("CREATE TABLE library_entries(root TEXT NOT NULL,path TEXT NOT NULL,size INTEGER NOT NULL,modified_ms INTEGER,present INTEGER NOT NULL DEFAULT 1,PRIMARY KEY(root,path));").unwrap();
        drop(legacy);
        let original = fs::read(&database).unwrap();
        assert!(Store::read_library_page_readonly(&database, &root, None, 128).is_err());
        assert_eq!(fs::read(&database).unwrap(), original);
        fs::write(&database, b"synthetic invalid sqlite, preserve").unwrap();
        assert!(Store::read_library_page_readonly(&database, &root, None, 128).is_err());
        assert_eq!(
            fs::read(&database).unwrap(),
            b"synthetic invalid sqlite, preserve"
        );
    }

    #[test]
    fn readonly_page_busy_failure_preserves_committed_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("media");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("a.mp4"), b"preserve").unwrap();
        let database = dir.path().join("library.sqlite");
        let mut writer = Store::open(&database).unwrap();
        writer.reconcile_library(&root).unwrap();
        let original = writer.library_page(&root, None, 128).unwrap();
        writer.connection.execute_batch("BEGIN EXCLUSIVE").unwrap();
        assert!(
            matches!(Store::read_library_page_readonly(&database, &root, None, 128), Err(LibraryError::Database(rusqlite::Error::SqliteFailure(error, _))) if error.code == rusqlite::ErrorCode::DatabaseBusy)
        );
        writer.connection.execute_batch("ROLLBACK").unwrap();
        assert_eq!(
            Store::read_library_page_readonly(&database, &root, None, 128).unwrap(),
            original
        );
        assert_eq!(fs::read(root.join("a.mp4")).unwrap(), b"preserve");
    }

    #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
    mod event_wake_pacing_experiment {
        use super::*;
        use std::{
            sync::{
                Arc, Condvar, Mutex,
                atomic::{AtomicBool, Ordering},
                mpsc,
            },
            time::{Duration, Instant},
        };
        #[repr(C)]
        #[derive(Default)]
        struct Time {
            low: u32,
            high: u32,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetCurrentThread() -> *mut std::ffi::c_void;
            fn GetProcessTimes(
                h: *mut std::ffi::c_void,
                c: *mut Time,
                e: *mut Time,
                k: *mut Time,
                u: *mut Time,
            ) -> i32;
            fn GetThreadTimes(
                h: *mut std::ffi::c_void,
                c: *mut Time,
                e: *mut Time,
                k: *mut Time,
                u: *mut Time,
            ) -> i32;
        }
        fn cpu() -> io::Result<(u64, u64)> {
            let (mut c, mut e, mut k, mut u) = (
                Time::default(),
                Time::default(),
                Time::default(),
                Time::default(),
            );
            let ticks = |t: &Time| (u64::from(t.high) << 32) | u64::from(t.low);
            // SAFETY: original current pseudo HANDLEs and initialized exact FILETIME outputs.
            if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            let process = ticks(&k) + ticks(&u);
            if unsafe { GetThreadTimes(GetCurrentThread(), &mut c, &mut e, &mut k, &mut u) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok((process, ticks(&k) + ticks(&u)))
        }
        #[derive(Clone, Default)]
        struct InterruptNotifier(Arc<(Mutex<bool>, Condvar)>);
        impl InterruptNotifier {
            fn interrupt(&self) {
                let (lock, wake) = &*self.0;
                *lock.lock().unwrap() = true;
                wake.notify_all();
            }
        }
        struct Pacer {
            wait_ready: Option<(Arc<AtomicBool>, mpsc::SyncSender<()>)>,
            notifier: InterruptNotifier,
            start: Instant,
            epoch: Instant,
            next_sample: Duration,
            samples: Vec<(f64, u64, u64)>,
            checkpoints: u64,
            sleeps: u64,
            expired: bool,
        }
        impl Pacer {
            fn new() -> Self {
                let now = Instant::now();
                Self {
                    wait_ready: None,
                    notifier: InterruptNotifier::default(),
                    start: now,
                    epoch: now,
                    next_sample: Duration::ZERO,
                    samples: Vec::with_capacity(20),
                    checkpoints: 0,
                    sleeps: 0,
                    expired: false,
                }
            }
            fn checkpoint(&mut self, mut interrupt: impl FnMut() -> bool) -> bool {
                self.checkpoints += 1;
                loop {
                    if interrupt() {
                        return true;
                    }
                    let elapsed = self.start.elapsed();
                    if elapsed >= Duration::from_secs(180) {
                        self.expired = true;
                        return true;
                    }
                    if elapsed >= self.next_sample {
                        assert!(self.samples.len() < 20, "bounded actual CPU samples");
                        let (p, t) = cpu().expect("original process/thread counters");
                        self.samples.push((elapsed.as_secs_f64(), p, t));
                        self.next_sample += Duration::from_secs(10);
                    }
                    let (lock, wake) = &*self.notifier.0;
                    let guard = lock.lock().unwrap();
                    if *guard {
                        return true;
                    }
                    let used = self.epoch.elapsed();
                    if used < Duration::from_millis(1) {
                        return false;
                    }
                    if used >= Duration::from_millis(250) {
                        self.epoch = Instant::now();
                        return false;
                    }
                    let wait = (Duration::from_millis(250) - used)
                        .min(self.next_sample.saturating_sub(elapsed))
                        .min(Duration::from_secs(180).saturating_sub(elapsed));
                    self.sleeps += 1;
                    if self
                        .wait_ready
                        .as_ref()
                        .is_some_and(|(armed, _)| armed.load(Ordering::Acquire))
                    {
                        let (_, ready) = self.wait_ready.take().unwrap();
                        ready.send(()).unwrap();
                    }
                    // Notification sets protected state BEFORE notify: pre-wait interrupts
                    // cannot be lost; spurious wakeups retain the original timeout bound.
                    let (guard, _) = wake.wait_timeout_while(guard, wait, |flag| !*flag).unwrap();
                    if *guard {
                        return true;
                    }
                }
            }
        }

        fn last_row(store: &Store, root: &Path) -> LibraryEntry {
            let mut after = None;
            for _ in 0..100 {
                let page = store.library_page(root, after.as_deref(), 128).unwrap();
                if let Some(row) = page.iter().find(|r| r.path.ends_with("zz-last.mp4")) {
                    return row.clone();
                }
                if page.len() < 128 {
                    break;
                }
                after = page.last().map(|r| r.path.clone());
            }
            panic!("bounded ordinary row missing")
        }
        #[test]
        #[ignore = "explicit physical Windows pacing diagnostic; convergence <=180sec, not 30sec SLA or GUI budget"]
        fn actual_event_wake_full_fresh_scan_outside_alias_commit() {
            let tmp = tempfile::Builder::new()
                .prefix("dlpad-event-paced-owned-")
                .tempdir()
                .unwrap()
                .keep();
            let root = tmp.join("media");
            fs::create_dir(&root).unwrap();
            fs::write(tmp.join("sentinel"), b"preserve").unwrap();
            eprintln!(
                "{}",
                serde_json::json!({"phase":"event-pacing-fixture-created","owned_fixture_root":tmp,"synthetic_only":true})
            );
            for n in 0..10000 {
                fs::write(root.join(format!("file-{n:05}.mp4")), b"abc").unwrap()
            }
            let final_path = root.join("zz-last.mp4");
            fs::write(&final_path, b"xxxxxxxxxxxx").unwrap();
            let alias = tmp.join("outside-alias.part");
            fs::hard_link(&final_path, &alias).unwrap();
            let mut store = Store::open(&tmp.join("library.sqlite")).unwrap();
            assert_eq!(store.reconcile_library(&root).unwrap(), 10001);
            let before = last_row(&store, &root);
            use std::io::Write;
            let mut writer = fs::OpenOptions::new().append(true).open(&alias).unwrap();
            writer.write_all(b"z").unwrap();
            writer.sync_all().unwrap();
            drop(writer);
            let mut pacer = Pacer::new();
            let baseline_cpu = cpu().unwrap();
            let observed = store
                .reconcile_library_checked(&root, || pacer.checkpoint(|| false))
                .unwrap();
            let scan_elapsed = pacer.start.elapsed();
            let ending = cpu().unwrap();
            assert!(!pacer.expired);
            assert_eq!(observed, 10001);
            assert!(pacer.sleeps > 0);
            assert!(pacer.checkpoints > 10000);
            let after = last_row(&store, &root);
            assert_eq!(after.size, 13);
            assert_eq!(after.file_identity, before.file_identity);
            for n in 0..10000 {
                assert_eq!(
                    fs::read(root.join(format!("file-{n:05}.mp4"))).unwrap(),
                    b"abc"
                )
            }
            assert_eq!(fs::read(&final_path).unwrap(), b"xxxxxxxxxxxxz");
            assert_eq!(fs::read(&alias).unwrap(), b"xxxxxxxxxxxxz");
            assert_eq!(fs::read(tmp.join("sentinel")).unwrap(), b"preserve");
            drop(store);
            let reopened = Store::open(&tmp.join("library.sqlite")).unwrap();
            assert_eq!(last_row(&reopened, &root), after);
            eprintln!(
                "{}",
                serde_json::json!({"phase":"event-pacing-scan-completed","scan_elapsed_seconds":scan_elapsed.as_secs_f64(),"process_cpu_100ns":ending.0-baseline_cpu.0,"thread_cpu_100ns":ending.1-baseline_cpu.1,"checkpoints":pacer.checkpoints,"wait_calls":pacer.sleeps,"samples":pacer.samples,"observed_count":observed,"alias_size":after.size,"original_identity_preserved":after.file_identity==before.file_identity,"db_reopen_equal":true,"source_and_sentinel_preserved":true,"product_policy_adopted":false,"same_30_second_workload_cpu_budget_proven":false})
            );
            eprintln!(
                "event-wake-pacing-diagnostic elapsed={} process_ticks={} thread_ticks={} checkpoints={} sleeps={} samples={:?} convergence_only=true",
                scan_elapsed.as_secs_f64(),
                ending.0 - baseline_cpu.0,
                ending.1 - baseline_cpu.1,
                pacer.checkpoints,
                pacer.sleeps,
                pacer.samples
            );
        }
        #[test]
        fn event_wake_stop_and_supersession_rollback_without_publishing() {
            for stop_kind in [true, false] {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().join("media");
                fs::create_dir(&root).unwrap();
                fs::write(root.join("old.mp4"), b"abc").unwrap();
                let mut store = Store::open(&tmp.path().join("library.sqlite")).unwrap();
                store.reconcile_library(&root).unwrap();
                let before = store.library_page(&root, None, 128).unwrap();
                fs::write(root.join("new.mp4"), b"new").unwrap();
                let stop = Arc::new(AtomicBool::new(false));
                let copied = stop.clone();
                let (tx, rx) = mpsc::sync_channel(1);
                let (ready_tx, ready_rx) = mpsc::sync_channel(1);
                let armed = Arc::new(AtomicBool::new(false));
                let copied_armed = armed.clone();
                let mut pacer = Pacer::new();
                pacer.wait_ready = Some((armed, ready_tx));
                let notifier = pacer.notifier.clone();
                let trigger = std::thread::spawn(move || {
                    ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                    if stop_kind {
                        copied.store(true, Ordering::Release)
                    } else {
                        tx.send(()).unwrap()
                    }
                    notifier.interrupt()
                });
                let result = store.reconcile_library_inner(
                    &root,
                    || pacer.checkpoint(|| stop.load(Ordering::Acquire) || rx.try_recv().is_ok()),
                    Some(|| {
                        copied_armed.store(true, Ordering::Release);
                        std::thread::sleep(Duration::from_millis(2));
                        Ok(())
                    }),
                );
                trigger.join().unwrap();
                assert!(matches!(result, Err(LibraryError::Cancelled)));
                assert_eq!(store.library_page(&root, None, 128).unwrap(), before);
                assert!(pacer.start.elapsed() < Duration::from_secs(2));
            }
        }
    }

    #[cfg(all(windows, feature = "library-scan-profile-fixture"))]
    mod pacing_experiment {
        use super::*;
        use std::{
            sync::{
                Arc,
                atomic::{AtomicBool, Ordering},
                mpsc,
            },
            time::{Duration, Instant},
        };
        #[repr(C)]
        #[derive(Default)]
        struct Time {
            low: u32,
            high: u32,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetCurrentThread() -> *mut std::ffi::c_void;
            fn GetProcessTimes(
                h: *mut std::ffi::c_void,
                c: *mut Time,
                e: *mut Time,
                k: *mut Time,
                u: *mut Time,
            ) -> i32;
            fn GetThreadTimes(
                h: *mut std::ffi::c_void,
                c: *mut Time,
                e: *mut Time,
                k: *mut Time,
                u: *mut Time,
            ) -> i32;
        }
        fn cpu() -> io::Result<(u64, u64)> {
            let (mut c, mut e, mut k, mut u) = (
                Time::default(),
                Time::default(),
                Time::default(),
                Time::default(),
            );
            let ticks = |t: &Time| (u64::from(t.high) << 32) | u64::from(t.low);
            // SAFETY: original current pseudo HANDLEs and initialized exact FILETIME outputs.
            if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            let process = ticks(&k) + ticks(&u);
            if unsafe { GetThreadTimes(GetCurrentThread(), &mut c, &mut e, &mut k, &mut u) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok((process, ticks(&k) + ticks(&u)))
        }
        struct Pacer {
            start: Instant,
            epoch: Instant,
            next_sample: Duration,
            samples: Vec<(f64, u64, u64)>,
            checkpoints: u64,
            sleeps: u64,
            expired: bool,
        }
        impl Pacer {
            fn new() -> Self {
                let now = Instant::now();
                Self {
                    start: now,
                    epoch: now,
                    next_sample: Duration::ZERO,
                    samples: Vec::with_capacity(20),
                    checkpoints: 0,
                    sleeps: 0,
                    expired: false,
                }
            }
            fn checkpoint(&mut self, mut interrupt: impl FnMut() -> bool) -> bool {
                self.checkpoints += 1;
                loop {
                    if interrupt() {
                        return true;
                    }
                    let elapsed = self.start.elapsed();
                    if elapsed >= Duration::from_secs(180) {
                        self.expired = true;
                        return true;
                    }
                    if elapsed >= self.next_sample {
                        let (p, t) = cpu().expect("actual original process/thread CPU sample");
                        self.samples.push((elapsed.as_secs_f64(), p, t));
                        self.next_sample += Duration::from_secs(10);
                    }
                    let used = self.epoch.elapsed();
                    if used < Duration::from_millis(1) {
                        return false;
                    }
                    if used >= Duration::from_millis(250) {
                        self.epoch = Instant::now();
                        return false;
                    }
                    self.sleeps += 1;
                    std::thread::sleep(
                        (Duration::from_millis(250) - used).min(Duration::from_millis(10)),
                    );
                }
            }
        }
        fn last_row(store: &Store, root: &Path) -> LibraryEntry {
            let mut after = None;
            for _ in 0..100 {
                let page = store.library_page(root, after.as_deref(), 128).unwrap();
                if let Some(row) = page.iter().find(|r| r.path.ends_with("zz-last.mp4")) {
                    return row.clone();
                }
                if page.len() < 128 {
                    break;
                }
                after = page.last().map(|r| r.path.clone());
            }
            panic!("bounded ordinary row missing")
        }
        #[test]
        #[ignore = "explicit physical Windows pacing diagnostic; convergence <=180sec, not 30sec SLA or GUI budget"]
        fn actual_full_fresh_scan_paced_with_outside_alias_eventual_commit() {
            let tmp = tempfile::Builder::new()
                .prefix("dlpad-paced-scan-owned-")
                .tempdir()
                .unwrap()
                .keep();
            let root = tmp.join("media");
            fs::create_dir(&root).unwrap();
            eprintln!(
                "{}",
                serde_json::json!({"phase":"pacing-fixture-created","owned_fixture_root":tmp,"synthetic_only":true})
            );
            fs::write(tmp.join("sentinel"), b"preserve").unwrap();
            for n in 0..10000 {
                fs::write(root.join(format!("file-{n:05}.mp4")), b"abc").unwrap()
            }
            let final_path = root.join("zz-last.mp4");
            fs::write(&final_path, b"xxxxxxxxxxxx").unwrap();
            let alias = tmp.join("outside-alias.part");
            fs::hard_link(&final_path, &alias).unwrap();
            let mut store = Store::open(&tmp.join("library.sqlite")).unwrap();
            assert_eq!(store.reconcile_library(&root).unwrap(), 10001);
            let before = last_row(&store, &root);
            use std::io::Write;
            let mut writer = fs::OpenOptions::new().append(true).open(&alias).unwrap();
            writer.write_all(b"z").unwrap();
            writer.sync_all().unwrap();
            drop(writer);
            let mut pacer = Pacer::new();
            let baseline_cpu = cpu().unwrap();
            let observed = store
                .reconcile_library_checked(&root, || pacer.checkpoint(|| false))
                .unwrap();
            let scan_elapsed = pacer.start.elapsed();
            let ending = cpu().unwrap();
            assert!(!pacer.expired);
            assert_eq!(observed, 10001);
            assert!(pacer.sleeps > 0);
            assert!(pacer.checkpoints > 10000);
            let after = last_row(&store, &root);
            assert_eq!(after.size, 13);
            assert_eq!(after.file_identity, before.file_identity);
            for n in 0..10000 {
                assert_eq!(
                    fs::read(root.join(format!("file-{n:05}.mp4"))).unwrap(),
                    b"abc"
                )
            }
            assert_eq!(fs::read(tmp.join("sentinel")).unwrap(), b"preserve");
            assert_eq!(fs::read(&final_path).unwrap(), b"xxxxxxxxxxxxz");
            assert_eq!(fs::read(&alias).unwrap(), b"xxxxxxxxxxxxz");
            drop(store);
            let reopened = Store::open(&tmp.join("library.sqlite")).unwrap();
            assert_eq!(last_row(&reopened, &root), after);
            eprintln!(
                "{}",
                serde_json::json!({"phase":"pacing-scan-completed", "scan_elapsed_seconds":scan_elapsed.as_secs_f64(), "process_cpu_100ns":ending.0-baseline_cpu.0,"thread_cpu_100ns":ending.1-baseline_cpu.1,"checkpoints":pacer.checkpoints,"sleep_slices":pacer.sleeps,"samples":pacer.samples,"observed_count":observed,"outside_alias_size":after.size,"outside_alias_identity_preserved":after.file_identity==before.file_identity,"db_reopen_equal":true,"sentinel_and_source_bytes_preserved":true,"product_policy_adopted":false,"same_30_second_workload_cpu_budget_proven":false})
            );
            eprintln!(
                "pacing-diagnostic elapsed={} process_ticks={} thread_ticks={} checkpoints={} sleeps={} samples={:?} convergence_only=true",
                scan_elapsed.as_secs_f64(),
                ending.0 - baseline_cpu.0,
                ending.1 - baseline_cpu.1,
                pacer.checkpoints,
                pacer.sleeps,
                pacer.samples
            );
        }
        #[test]
        fn pacing_stop_and_supersession_rollback_without_publishing() {
            for stop_kind in [true, false] {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().join("media");
                fs::create_dir(&root).unwrap();
                fs::write(root.join("old.mp4"), b"abc").unwrap();
                let mut store = Store::open(&tmp.path().join("library.sqlite")).unwrap();
                store.reconcile_library(&root).unwrap();
                let before = store.library_page(&root, None, 128).unwrap();
                fs::write(root.join("new.mp4"), b"new").unwrap();
                let stop = Arc::new(AtomicBool::new(false));
                let copied = stop.clone();
                let (tx, rx) = mpsc::sync_channel(1);
                let (ready_tx, ready_rx) = mpsc::sync_channel(1);
                let (done_tx, done_rx) = mpsc::sync_channel(1);
                let trigger = std::thread::spawn(move || {
                    ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                    if stop_kind {
                        copied.store(true, Ordering::Release)
                    } else {
                        tx.send(()).unwrap()
                    }
                    done_tx.send(()).unwrap()
                });
                let mut pacer = Pacer::new();
                let result = store.reconcile_library_inner(
                    &root,
                    || pacer.checkpoint(|| stop.load(Ordering::Acquire) || rx.try_recv().is_ok()),
                    Some(|| {
                        ready_tx.send(()).unwrap();
                        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                        Ok(())
                    }),
                );
                trigger.join().unwrap();
                assert!(matches!(result, Err(LibraryError::Cancelled)));
                assert_eq!(store.library_page(&root, None, 128).unwrap(), before);
                assert!(pacer.start.elapsed() < Duration::from_secs(2));
            }
        }
    }

    // Windows native regression draft for library::tests; eight fresh fixture cases.
    #[cfg(windows)]
    #[test]
    fn alias_changes_refresh_size_and_mtime_without_prescan_metadata_touch() {
        use std::{io::Write, os::windows::io::AsRawHandle, time::Duration};
        use windows_sys::Win32::{Foundation::FILETIME, Storage::FileSystem::SetFileTime};
        for inside in [false, true] {
            for hold_attributes in [false, true] {
                for same_size in [false, true] {
                    let fixture = tempfile::tempdir().unwrap();
                    let root = fixture.path().join("media");
                    fs::create_dir(&root).unwrap();
                    let alias_dir = if inside {
                        root.join(".dlpad-partials/job")
                    } else {
                        fixture.path().join("outside-alias")
                    };
                    fs::create_dir_all(&alias_dir).unwrap();
                    let alias = alias_dir.join("file.part");
                    fs::write(&alias, b"abcdefghijkl").unwrap();
                    let finalpath = root.join("completed.mp4");
                    fs::hard_link(&alias, &finalpath).unwrap();
                    let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
                    assert_eq!(store.reconcile_library(&root).unwrap(), 1);
                    let baseline = store.library_page(&root, None, 128).unwrap();
                    assert_eq!(baseline[0].size, 12);
                    // Capture the known original modification time before the tested alias mutation.
                    let future = fs::metadata(&finalpath).unwrap().modified().unwrap()
                        + Duration::from_secs(4);
                    let future_ms =
                        i64::try_from(future.duration_since(UNIX_EPOCH).unwrap().as_millis())
                            .unwrap();
                    let original = file_identity_for_test(&finalpath);
                    let held = if hold_attributes {
                        Some(crate::file_identity::open_regular_no_reparse(&finalpath).unwrap())
                    } else {
                        None
                    };
                    let mut writer = fs::OpenOptions::new()
                        .write(true)
                        .append(!same_size)
                        .open(&alias)
                        .unwrap();
                    if same_size {
                        writer.write_all(b"ZYXWVUTSRQPO").unwrap();
                        let unix = future.duration_since(UNIX_EPOCH).unwrap();
                        let ticks = unix.as_secs().checked_mul(10_000_000).unwrap()
                            + u64::from(unix.subsec_nanos() / 100)
                            + 116_444_736_000_000_000;
                        let native = FILETIME {
                            dwLowDateTime: ticks as u32,
                            dwHighDateTime: (ticks >> 32) as u32,
                        };
                        // SAFETY: owned original alias File HANDLE; only lastwrite output changed to synthetic future.
                        assert_ne!(
                            unsafe {
                                SetFileTime(
                                    writer.as_raw_handle(),
                                    std::ptr::null(),
                                    std::ptr::null(),
                                    &native,
                                )
                            },
                            0
                        );
                    } else {
                        writer.write_all(b"z").unwrap();
                    }
                    writer.sync_all().unwrap();
                    drop(writer);
                    // Critical: do not call finalpath metadata/canonicalize/open before reconcile.
                    assert_eq!(store.reconcile_library(&root).unwrap(), 1);
                    let fresh = store.library_page(&root, None, 128).unwrap();
                    assert_eq!(fresh.len(), 1);
                    assert_eq!(
                        fresh[0].size,
                        if same_size { 12 } else { 13 },
                        "inside={inside}, hold={hold_attributes}, same_size={same_size}"
                    );
                    assert_eq!(fresh[0].file_identity, baseline[0].file_identity);
                    if same_size {
                        assert_eq!(fresh[0].modified_ms, Some(future_ms));
                        assert_ne!(fresh[0].modified_ms, baseline[0].modified_ms);
                    }
                    // Independent native/payload evidence only after the tested publication.
                    assert_eq!(file_identity_for_test(&finalpath), original);
                    assert_eq!(file_identity_for_test(&alias), original);
                    assert_eq!(
                        fs::read(&finalpath).unwrap(),
                        if same_size {
                            b"ZYXWVUTSRQPO".as_slice()
                        } else {
                            b"abcdefghijklz".as_slice()
                        }
                    );
                    drop(held);
                    drop(store);
                    let reopened = Store::open(&fixture.path().join("library.sqlite")).unwrap();
                    assert_eq!(reopened.library_page(&root, None, 128).unwrap(), fresh);
                }
            }
        }
    }
    #[cfg(windows)]
    fn file_identity_for_test(path: &Path) -> crate::file_identity::NativeFileIdentity {
        let file = crate::file_identity::open_regular_no_reparse(path).unwrap();
        crate::file_identity::from_file(&file).unwrap()
    }
    // Existing cancellation_rolls_back_observations_and_publication and feature
    // first_real_observation_cancel_rolls_back_and_preserves_original_snapshot,
    // callback_timeout_rolls_back_and_does_not_delete_missing_previous_entry remain
    // required alongside this new freshness regression; no weakening or duplication.

    use super::*;
    // Draft include inside library::tests after root no-change implementation.
    // Uses real SQLite failure triggers: no implementation-mirroring equality helper.
    #[cfg(windows)]
    #[test]
    fn unchanged_snapshot_skips_publication_writes_and_clears_observations() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("media");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("a.mp4"), b"abc").unwrap();
        let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        store.reconcile_library(&root).unwrap();
        let prior = store.library_page(&root, None, 128).unwrap();
        store
            .connection
            .execute_batch(
                "CREATE TRIGGER fail_publication_update BEFORE UPDATE ON library_entries
          BEGIN SELECT RAISE(ABORT,'unexpected publication update'); END;
         CREATE TRIGGER fail_publication_delete BEFORE DELETE ON library_entries
          BEGIN SELECT RAISE(ABORT,'unexpected publication delete'); END;
         CREATE TRIGGER fail_publication_insert BEFORE INSERT ON library_entries
          BEGIN SELECT RAISE(ABORT,'unexpected publication insert'); END;",
            )
            .unwrap();
        assert_eq!(store.reconcile_library(&root).unwrap(), 1);
        assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
        // Observations clear/commit still must occur: repeat must not hit duplicate TEMP PK.
        assert_eq!(store.reconcile_library(&root).unwrap(), 1);
        assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
    }

    #[cfg(windows)]
    #[test]
    fn equal_count_changed_paths_or_fields_never_take_noop_branch() {
        for changed in ["path", "size", "identity_null", "mtime_null", "absent"] {
            let fixture = tempfile::tempdir().unwrap();
            let root = fixture.path().join("media");
            fs::create_dir(&root).unwrap();
            let original = root.join("a.mp4");
            fs::write(&original, b"abc").unwrap();
            let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
            store.reconcile_library(&root).unwrap();
            match changed {
                "path" => fs::rename(&original, root.join("b.mp4")).unwrap(),
                "size" => fs::write(&original, b"abcdef").unwrap(),
                "identity_null" => {
                    store
                        .connection
                        .execute("UPDATE library_entries SET file_identity=NULL", [])
                        .unwrap();
                }
                "mtime_null" => {
                    store
                        .connection
                        .execute("UPDATE library_entries SET modified_ms=NULL", [])
                        .unwrap();
                }
                "absent" => {
                    store
                        .connection
                        .execute("UPDATE library_entries SET present=0", [])
                        .unwrap();
                }
                _ => unreachable!(),
            }
            let snapshot = |store: &Store| -> (String, i64, Option<i64>, Option<String>, i64) {
                store
                    .connection
                    .query_row(
                        "SELECT path,size,modified_ms,file_identity,present FROM library_entries",
                        [],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                    )
                    .unwrap()
            };
            let prior = snapshot(&store);
            // Fail every actual publication mutation; equal count cannot bypass changed data.
            store.connection.execute_batch("CREATE TRIGGER fail_update BEFORE UPDATE ON library_entries BEGIN SELECT RAISE(ABORT,'expected mutation'); END; CREATE TRIGGER fail_insert BEFORE INSERT ON library_entries BEGIN SELECT RAISE(ABORT,'expected mutation'); END;").unwrap();
            assert!(
                matches!(
                    store.reconcile_library(&root),
                    Err(LibraryError::Database(_))
                ),
                "{changed}"
            );
            let entries: i64 = store
                .connection
                .query_row("SELECT count(*) FROM library_entries", [], |r| r.get(0))
                .unwrap();
            assert_eq!(entries, 1);
            assert_eq!(snapshot(&store), prior, "{changed}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn unchanged_hardlink_snapshot_is_noop_but_removed_alias_changes_publication() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("media");
        fs::create_dir(&root).unwrap();
        let a = root.join("a.mp4");
        let b = root.join("b.mp4");
        fs::write(&a, b"abc").unwrap();
        fs::hard_link(&a, &b).unwrap();
        let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        assert_eq!(store.reconcile_library(&root).unwrap(), 2);
        let prior = store.library_page(&root, None, 128).unwrap();
        assert_eq!(prior[0].file_identity, prior[1].file_identity);
        store.connection.execute_batch("CREATE TRIGGER fail_update BEFORE UPDATE ON library_entries BEGIN SELECT RAISE(ABORT,'unexpected mutation'); END; CREATE TRIGGER fail_insert BEFORE INSERT ON library_entries BEGIN SELECT RAISE(ABORT,'unexpected mutation'); END;").unwrap();
        assert_eq!(store.reconcile_library(&root).unwrap(), 2);
        assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
        fs::remove_file(&b).unwrap();
        assert!(matches!(
            store.reconcile_library(&root),
            Err(LibraryError::Database(_))
        ));
        assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
    }

    #[test]
    fn oversized_database_path_and_identity_fail_before_result_decode() {
        for (path_bytes, identity_bytes) in [
            (MAX_LIBRARY_PATH_BYTES + 1, 1),
            (1, MAX_LIBRARY_IDENTITY_BYTES + 1),
            (8 * 1024 * 1024, 1),
            (1, 8 * 1024 * 1024),
        ] {
            let store = Store::in_memory().unwrap();
            store
                .connection
                .execute(
                    "INSERT INTO library_entries(root,path,size,present,file_identity)
                 VALUES('/fixture',CAST(zeroblob(?1) AS TEXT),1,1,CAST(zeroblob(?2) AS TEXT))",
                    params![path_bytes as i64, identity_bytes as i64],
                )
                .unwrap();
            let measured: (i64,i64) = store.connection.query_row(
                "SELECT length(CAST(path AS BLOB)),length(CAST(file_identity AS BLOB)) FROM library_entries",
                [], |row| Ok((row.get(0)?,row.get(1)?)),
            ).unwrap();
            assert_eq!(measured, (path_bytes as i64, identity_bytes as i64));
            assert!(matches!(
                store.library_page(Path::new("/fixture"), None, 128),
                Err(LibraryError::Database(
                    rusqlite::Error::FromSqlConversionFailure(..)
                ))
            ));
            let remaining: i64 = store
                .connection
                .query_row("SELECT count(*) FROM library_entries", [], |r| r.get(0))
                .unwrap();
            assert_eq!(remaining, 1);
            let after: (i64, i64) = store.connection.query_row(
                "SELECT length(CAST(path AS BLOB)),length(CAST(file_identity AS BLOB)) FROM library_entries",
                [], |row| Ok((row.get(0)?,row.get(1)?)),
            ).unwrap();
            assert_eq!(after, measured); // No write or payload truncation on error.
        }
    }
    #[test]
    fn unicode_paths_identity_and_keyset_survive_byte_guards() {
        let store = Store::in_memory().unwrap();
        let path = format!("/fixture/{}", "한".repeat(32766));
        let identity = "windows:v1:18446744073709551615:00112233445566778899aabbccddeeff";
        store.connection.execute(
            "INSERT INTO library_entries(root,path,size,present,file_identity) VALUES('/fixture',?1,1,1,?2)",
            params![path,identity],
        ).unwrap();
        let page = store
            .library_page(Path::new("/fixture"), None, 128)
            .unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].path, PathBuf::from(&path));
        assert_eq!(page[0].file_identity.as_deref(), Some(identity));
        assert!(
            store
                .library_page(Path::new("/fixture"), Some(&page[0].path), 128)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn pre_cancelled_scan_never_opens_missing_root() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::in_memory().unwrap();
        let cancel = AtomicBool::new(true);
        assert!(matches!(
            store.reconcile_library_with_cancel(&dir.path().join("absent"), &cancel),
            Err(LibraryError::Cancelled)
        ));
    }

    #[test]
    fn cancellation_rolls_back_observations_and_publication() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let old = root.join("old.mp4");
        fs::write(&old, b"synthetic-old").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(root).unwrap();
        let previous = store.library_page(root, None, 128).unwrap();
        fs::remove_file(&old).unwrap();
        fs::write(root.join("new.mp4"), b"synthetic-new").unwrap();
        // One file produces five checks: start, entry, exhausted iterator,
        // publication and pre-commit. Test both observation and DB rollback.
        for stop_at in [2, 4, 5] {
            let mut checks = 0;
            assert!(matches!(
                store.reconcile_library_checked(root, || {
                    checks += 1;
                    checks == stop_at
                }),
                Err(LibraryError::Cancelled)
            ));
            assert_eq!(store.library_page(root, None, 128).unwrap(), previous);
        }
        let cancel = AtomicBool::new(false);
        assert_eq!(
            store.reconcile_library_with_cancel(root, &cancel).unwrap(),
            1
        );
        let next = store.library_page(root, None, 128).unwrap();
        assert_eq!(next[0].path, root.join("new.mp4"));
    }

    #[cfg(any(unix, windows))]
    fn row_id(store: &Store, root: &Path, path: &Path) -> i64 {
        store
            .connection
            .query_row(
                "SELECT rowid FROM library_entries WHERE root=?1 AND path=?2",
                params![root.to_str().unwrap(), path.to_str().unwrap()],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn native_reused_store_unchanged_rename_and_cancel_preserve_rows() {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let original = root.join("original.mp4");
        let renamed = root.join("renamed.mp4");
        fs::write(&original, b"original").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(&root).unwrap();
        let original_id = row_id(&store, &root, &original);
        let previous = store.library_page(&root, None, 128).unwrap();
        store.reconcile_library(&root).unwrap();
        assert_eq!(store.library_page(&root, None, 128).unwrap(), previous);
        assert_eq!(row_id(&store, &root, &original), original_id);
        fs::rename(&original, &renamed).unwrap();
        store.reconcile_library(&root).unwrap();
        assert_eq!(row_id(&store, &root, &renamed), original_id);
        let committed = store.library_page(&root, None, 128).unwrap();
        fs::write(root.join("added.mp4"), b"new").unwrap();
        let cancel = AtomicBool::new(false);
        let result = store.reconcile_library_inner(
            &root,
            || cancel.load(Ordering::Acquire),
            Some(|| {
                cancel.store(true, Ordering::Release);
                Ok(())
            }),
        );
        assert!(matches!(result, Err(LibraryError::Cancelled)));
        assert_eq!(store.library_page(&root, None, 128).unwrap(), committed);
        assert_eq!(row_id(&store, &root, &renamed), original_id);
        store.reconcile_library(&root).unwrap();
        let changed = store.library_page(&root, None, 128).unwrap();
        assert_eq!(changed.len(), 2);
        assert!(
            changed
                .iter()
                .any(|entry| entry.path == root.join("added.mp4") && entry.size == 3)
        );
        assert_eq!(row_id(&store, &root, &renamed), original_id);
        store.reconcile_library(&root).unwrap();
        assert_eq!(store.library_page(&root, None, 128).unwrap(), changed);
    }

    #[cfg(windows)]
    #[test]
    fn native_windows_external_rename_preserves_record() {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let original = root.join("원본.mp4");
        let renamed = root.join("새 이름.mp4");
        fs::write(&original, b"synthetic").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(&root).unwrap();
        let previous = store.library_page(&root, None, 10).unwrap().remove(0);
        assert!(
            previous
                .file_identity
                .as_ref()
                .is_some_and(|id| id.starts_with("windows:v1:"))
        );
        let record = row_id(&store, &root, &original);
        fs::rename(&original, &renamed).unwrap();
        store.reconcile_library(&root).unwrap();
        let current = store.library_page(&root, None, 10).unwrap().remove(0);
        assert_eq!(current.path, renamed);
        assert_eq!(current.file_identity, previous.file_identity);
        assert_eq!(row_id(&store, &root, &renamed), record);
    }
    #[cfg(unix)]
    #[test]
    fn unique_external_rename_preserves_record_and_identity() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("원본.mp4");
        let renamed = dir.path().join("새 이름.mp4");
        fs::write(&original, b"synthetic").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(dir.path()).unwrap();
        let previous = store.library_page(dir.path(), None, 10).unwrap().remove(0);
        let record = row_id(&store, dir.path(), &original);
        fs::rename(&original, &renamed).unwrap();
        store.reconcile_library(dir.path()).unwrap();
        let current = store.library_page(dir.path(), None, 10).unwrap().remove(0);
        assert_eq!(current.path, renamed);
        assert_eq!(current.file_identity, previous.file_identity);
        assert_eq!(row_id(&store, dir.path(), &renamed), record);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn hardlink_ambiguity_and_replacement_do_not_transfer_records() {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let a = root.join("a.mp4");
        let b = root.join("b.mp4");
        let c = root.join("c.mp4");
        fs::write(&a, b"synthetic").unwrap();
        fs::hard_link(&a, &b).unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(&root).unwrap();
        let a_record = row_id(&store, root.as_path(), &a);
        fs::rename(&a, &c).unwrap();
        store.reconcile_library(&root).unwrap();
        assert_ne!(row_id(&store, root.as_path(), &c), a_record);
        let old_identity = store
            .library_page(root.as_path(), None, 10)
            .unwrap()
            .into_iter()
            .find(|e| e.path == c)
            .unwrap()
            .file_identity;
        fs::remove_file(&c).unwrap();
        fs::write(&c, b"synthetic").unwrap();
        store.reconcile_library(&root).unwrap();
        let new_identity = store
            .library_page(root.as_path(), None, 10)
            .unwrap()
            .into_iter()
            .find(|e| e.path == c)
            .unwrap()
            .file_identity;
        assert_ne!(old_identity, new_identity); // b pins the original inode.
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn multiple_new_hardlinks_do_not_claim_the_missing_record() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("original.mp4");
        let renamed = dir.path().join("renamed.mp4");
        let linked = dir.path().join("linked.mp4");
        fs::write(&original, b"synthetic").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(dir.path()).unwrap();
        let record = row_id(&store, dir.path(), &original);
        fs::rename(&original, &renamed).unwrap();
        fs::hard_link(&renamed, &linked).unwrap();
        store.reconcile_library(dir.path()).unwrap();
        assert_ne!(row_id(&store, dir.path(), &renamed), record);
        assert_ne!(row_id(&store, dir.path(), &linked), record);
        assert_eq!(store.library_page(dir.path(), None, 10).unwrap().len(), 2);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn failed_scan_keeps_old_identity_and_path() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("original.mp4");
        fs::write(&original, b"synthetic").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(dir.path()).unwrap();
        let previous = store.library_page(dir.path(), None, 10).unwrap();
        fs::rename(&original, dir.path().join("renamed.mp4")).unwrap();
        let mut deep = dir.path().to_path_buf();
        for _ in 0..MAX_SCAN_DEPTH {
            deep.push("d");
            fs::create_dir(&deep).unwrap();
        }
        assert!(matches!(
            store.reconcile_library(dir.path()),
            Err(LibraryError::DepthLimit)
        ));
        assert_eq!(store.library_page(dir.path(), None, 10).unwrap(), previous);
    }

    #[test]
    fn unicode_add_remove_and_unavailable_root_preserve_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("한글 문서");
        create_library_root(&root).unwrap();
        fs::write(root.join("영상.mp4"), b"synthetic").unwrap();
        fs::write(root.join("video.part"), b"partial").unwrap();
        fs::create_dir(root.join(".dlpad-partials")).unwrap();
        fs::write(root.join(".dlpad-partials/hidden.mp4"), b"partial").unwrap();
        let mut db = Store::in_memory().unwrap();
        assert_eq!(db.reconcile_library(&root).unwrap(), 1);
        fs::rename(&root, dir.path().join("unmounted")).unwrap();
        assert!(db.reconcile_library(&root).is_err());
        assert_eq!(db.library_page(&root, None, 1000).unwrap().len(), 1);
        fs::rename(dir.path().join("unmounted"), &root).unwrap();
        fs::remove_file(root.join("영상.mp4")).unwrap();
        assert_eq!(db.reconcile_library(&root).unwrap(), 0);
        assert!(db.library_page(&root, None, 10).unwrap().is_empty());
    }
    #[cfg(unix)]
    #[test]
    fn denied_scan_preserves_observations() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("denied");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("video.mp4"), b"fixture").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o0)).unwrap();
        let scan = store.reconcile_library(&root);
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        // Running this regression as root would bypass mode permissions.
        if unsafe { libc::geteuid() } != 0 {
            assert!(scan.is_err());
        }
        assert_eq!(store.library_page(&root, None, 128).unwrap().len(), 1);
    }
    #[test]
    fn keyset_pages_are_bounded() {
        let dir = tempfile::tempdir().unwrap();
        for n in 0..300 {
            fs::write(dir.path().join(format!("{n:04}.mp4")), b"fixture").unwrap();
        }
        let mut db = Store::in_memory().unwrap();
        db.reconcile_library(dir.path()).unwrap();
        let first = db.library_page(dir.path(), None, usize::MAX).unwrap();
        assert_eq!(first.len(), MAX_PAGE_SIZE);
        assert_eq!(
            db.library_page(dir.path(), Some(&first.last().unwrap().path), MAX_PAGE_SIZE)
                .unwrap()
                .len(),
            44
        );
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_followed_and_scan_failure_rolls_back() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("old.mp4"), b"fixture").unwrap();
        std::os::unix::fs::symlink("/", dir.path().join("outside")).unwrap();
        let mut db = Store::in_memory().unwrap();
        assert_eq!(db.reconcile_library(dir.path()).unwrap(), 1);
        let mut deep = dir.path().to_path_buf();
        for _ in 0..MAX_SCAN_DEPTH {
            deep.push("d");
            fs::create_dir(&deep).unwrap();
        }
        fs::remove_file(dir.path().join("old.mp4")).unwrap();
        assert!(matches!(
            db.reconcile_library(dir.path()),
            Err(LibraryError::DepthLimit)
        ));
        assert_eq!(db.library_page(dir.path(), None, 10).unwrap().len(), 1);
    }
}

#[cfg(all(test, feature = "library-scan-closewait-fixture"))]
mod closewait_fixture_tests {
    use super::*;
    use std::cell::Cell;
    // Include in library::closewait_fixture_tests. Windows-only actual candidate revalidation.
    #[cfg(windows)]
    #[test]
    fn rename_candidate_same_id_size_or_mtime_change_rolls_back_previous_snapshot() {
        use std::io::Write;
        for change_size in [true, false] {
            let fixture = tempfile::tempdir().unwrap();
            let root = fixture.path().join("media");
            fs::create_dir(&root).unwrap();
            let original = root.join("original.mp4");
            let moved = root.join("moved.mp4");
            fs::write(&original, b"abc").unwrap();
            let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
            store.reconcile_library(&root).unwrap();
            let prior = store.library_page(&root, None, 128).unwrap();
            fs::rename(&original, &moved).unwrap();
            // The retained ordinary File keeps original identity while in-place writing.
            let mut writer = fs::OpenOptions::new().write(true).open(&moved).unwrap();
            let original_id = crate::file_identity::from_file(&writer).unwrap();
            let baseline_modified = writer.metadata().unwrap().modified().unwrap();
            let mut callback_count = 0;
            let result = store.reconcile_library_after_first_observation(
                &root,
                &AtomicBool::new(false),
                || {
                    callback_count += 1;
                    if change_size {
                        writer.write_all(b"abcdef")?;
                    } else {
                        writer.write_all(b"xyz")?;
                        writer
                            .set_modified(baseline_modified + std::time::Duration::from_secs(4))?;
                    }
                    writer.sync_all()?;
                    assert_eq!(crate::file_identity::from_file(&writer)?, original_id);
                    Ok(())
                },
            );
            assert_eq!(callback_count, 1);
            assert!(
                matches!(result, Err(LibraryError::Io(error)) if error.kind() == io::ErrorKind::InvalidData)
            );
            assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
            assert_eq!(
                crate::file_identity::from_file(&writer).unwrap(),
                original_id
            );
            assert_eq!(
                fs::read(&moved).unwrap(),
                if change_size {
                    b"abcdef".as_slice()
                } else {
                    b"xyz".as_slice()
                }
            );
            drop(writer);
            drop(store);
            let reopened = Store::open(&fixture.path().join("library.sqlite")).unwrap();
            assert_eq!(reopened.library_page(&root, None, 128).unwrap(), prior);
        }
    }

    #[cfg(windows)]
    #[test]
    fn multiple_rename_candidates_mutated_after_first_observation_roll_back_all_previous_rows() {
        use std::io::Write;
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("media");
        fs::create_dir(&root).unwrap();
        let original_a = root.join("original-a.mp4");
        let original_z = root.join("original-z.mp4");
        let moved_a = root.join("moved-a.mp4");
        let moved_z = root.join("moved-z.mp4");
        fs::write(&original_a, b"aaa").unwrap();
        fs::write(&original_z, b"zzz").unwrap();
        let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        store.reconcile_library(&root).unwrap();
        let prior = store.library_page(&root, None, 128).unwrap();
        assert_eq!(prior.len(), 2);
        assert_ne!(prior[0].file_identity, prior[1].file_identity);
        fs::rename(&original_a, &moved_a).unwrap();
        fs::rename(&original_z, &moved_z).unwrap();
        let mut writer_a = fs::OpenOptions::new().write(true).open(&moved_a).unwrap();
        let mut writer_z = fs::OpenOptions::new().write(true).open(&moved_z).unwrap();
        let original_id_a = crate::file_identity::from_file(&writer_a).unwrap();
        let original_id_z = crate::file_identity::from_file(&writer_z).unwrap();
        assert_ne!(original_id_a, original_id_z);
        let mut callbacks = 0;
        let result =
            store.reconcile_library_after_first_observation(&root, &AtomicBool::new(false), || {
                callbacks += 1;
                // Exactly one real row is already INSERTed. Mutate both candidates:
                // whichever native enumeration order, that first row's3B observation
                // now mismatches its same original object before publication. The
                // second candidate may correctly be freshly observed at7B.
                writer_a.write_all(b"changed")?;
                writer_a.sync_all()?;
                writer_z.write_all(b"changed")?;
                writer_z.sync_all()?;
                assert_eq!(crate::file_identity::from_file(&writer_a)?, original_id_a);
                assert_eq!(crate::file_identity::from_file(&writer_z)?, original_id_z);
                Ok(())
            });
        assert_eq!(callbacks, 1);
        assert!(
            matches!(result, Err(LibraryError::Io(error)) if error.kind()==io::ErrorKind::InvalidData)
        );
        assert_eq!(store.library_page(&root, None, 128).unwrap(), prior);
        assert_eq!(
            crate::file_identity::from_file(&writer_a).unwrap(),
            original_id_a
        );
        assert_eq!(
            crate::file_identity::from_file(&writer_z).unwrap(),
            original_id_z
        );
        assert_eq!(fs::read(&moved_a).unwrap(), b"changed");
        assert_eq!(fs::read(&moved_z).unwrap(), b"changed");
        drop(writer_a);
        drop(writer_z);
        drop(store);
        let reopened = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        assert_eq!(reopened.library_page(&root, None, 128).unwrap(), prior);
    }

    #[cfg(windows)]
    #[test]
    fn late_unsupported_callback_never_falls_back_or_publishes() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("media");
        fs::create_dir(&root).unwrap();
        let old = root.join("old.mp4");
        fs::write(&old, b"abc").unwrap();
        let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        store.reconcile_library(&root).unwrap();
        let previous = store.library_page(&root, None, 128).unwrap();
        fs::remove_file(&old).unwrap();
        fs::write(root.join("new.mp4"), b"xyz").unwrap();
        let callbacks = Cell::new(0);
        let result =
            store.reconcile_library_after_first_observation(&root, &AtomicBool::new(false), || {
                callbacks.set(callbacks.get() + 1);
                Err(io::Error::new(io::ErrorKind::Unsupported, "late fixture unsupported").into())
            });
        assert!(
            matches!(result, Err(LibraryError::Io(error)) if error.kind()==io::ErrorKind::Unsupported)
        );
        assert_eq!(callbacks.get(), 1);
        assert_eq!(store.library_page(&root, None, 128).unwrap(), previous);
    }

    #[cfg(windows)]
    #[test]
    fn rename_candidate_replacement_before_publication_preserves_previous_snapshot() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("media");
        fs::create_dir(&root).unwrap();
        let original = root.join("original.mp4");
        let moved = root.join("moved.mp4");
        let replacement = fixture.path().join("different.mp4");
        fs::write(&original, b"abc").unwrap();
        fs::write(&replacement, b"xyz").unwrap();
        let mut store = Store::open(&fixture.path().join("library.sqlite")).unwrap();
        store.reconcile_library(&root).unwrap();
        let previous = store.library_page(&root, None, 128).unwrap();
        fs::rename(&original, &moved).unwrap();
        let cancel = AtomicBool::new(false);
        let result = store.reconcile_library_after_first_observation(&root, &cancel, || {
            fs::remove_file(&moved)?;
            fs::rename(&replacement, &moved)?;
            Ok(())
        });
        assert!(
            result.is_err(),
            "changed rename candidate must not transfer its record"
        );
        assert_eq!(store.library_page(&root, None, 128).unwrap(), previous);
        assert_eq!(fs::read(&moved).unwrap(), b"xyz");
    }

    #[test]
    fn first_real_observation_cancel_rolls_back_and_preserves_original_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let media = temp.path().join("media");
        fs::create_dir(&media).unwrap();
        fs::write(media.join("original.mp4"), b"original synthetic").unwrap();
        let mut store = Store::open(&temp.path().join("library.sqlite")).unwrap();
        store.reconcile_library(&media).unwrap();
        let prior = store.library_page(&media, None, MAX_PAGE_SIZE).unwrap();
        fs::write(media.join("new.mp4"), b"new synthetic").unwrap();
        let cancel = AtomicBool::new(false);
        let calls = Cell::new(0);
        let result = store.reconcile_library_after_first_observation(&media, &cancel, || {
            calls.set(calls.get() + 1);
            cancel.store(true, Ordering::Release);
            Ok(())
        });
        assert!(matches!(result, Err(LibraryError::Cancelled)));
        assert_eq!(calls.get(), 1);
        assert_eq!(
            store.library_page(&media, None, MAX_PAGE_SIZE).unwrap(),
            prior
        );
        assert_eq!(
            fs::read(media.join("original.mp4")).unwrap(),
            b"original synthetic"
        );
        assert_eq!(fs::read(media.join("new.mp4")).unwrap(), b"new synthetic");
        cancel.store(false, Ordering::Release);
        assert_eq!(
            store
                .reconcile_library_with_cancel(&media, &cancel)
                .unwrap(),
            2
        );
    }
    #[test]
    fn callback_timeout_rolls_back_and_does_not_delete_missing_previous_entry() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("previous.mp4"), b"prior").unwrap();
        let mut store = Store::in_memory().unwrap();
        store.reconcile_library(temp.path()).unwrap();
        let prior = store
            .library_page(temp.path(), None, MAX_PAGE_SIZE)
            .unwrap();
        fs::remove_file(temp.path().join("previous.mp4")).unwrap();
        fs::write(temp.path().join("replacement.mp4"), b"replacement").unwrap();
        let cancel = AtomicBool::new(false);
        let result = store.reconcile_library_after_first_observation(temp.path(), &cancel, || {
            Err(io::Error::new(io::ErrorKind::TimedOut, "owned fixture deadline").into())
        });
        assert!(matches!(result,Err(LibraryError::Io(ref e)) if e.kind()==io::ErrorKind::TimedOut));
        assert_eq!(
            store
                .library_page(temp.path(), None, MAX_PAGE_SIZE)
                .unwrap(),
            prior
        );
        assert_eq!(
            fs::read(temp.path().join("replacement.mp4")).unwrap(),
            b"replacement"
        );
    }
    #[test]
    fn empty_or_partial_only_root_never_signals_observation() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::in_memory().unwrap();
        let cancel = AtomicBool::new(false);
        assert_eq!(
            store
                .reconcile_library_after_first_observation(temp.path(), &cancel, || panic!(
                    "empty scan has no observation"
                ))
                .unwrap(),
            0
        );
        fs::write(temp.path().join("synthetic.part"), b"partial").unwrap();
        assert_eq!(
            store
                .reconcile_library_after_first_observation(temp.path(), &cancel, || panic!(
                    "partial is not ordinary observation"
                ))
                .unwrap(),
            0
        );
    }
    #[test]
    fn callback_runs_once_for_multiple_observations_and_pre_cancel_runs_none() {
        let temp = tempfile::tempdir().unwrap();
        for i in 0..3 {
            fs::write(temp.path().join(format!("{i}.mp4")), b"synthetic").unwrap();
        }
        let mut store = Store::in_memory().unwrap();
        let cancel = AtomicBool::new(false);
        let calls = Cell::new(0);
        assert_eq!(
            store
                .reconcile_library_after_first_observation(temp.path(), &cancel, || {
                    calls.set(calls.get() + 1);
                    Ok(())
                })
                .unwrap(),
            3
        );
        assert_eq!(calls.get(), 1);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            store.reconcile_library_after_first_observation(temp.path(), &cancel, || panic!(
                "pre-cancelled scan cannot start"
            )),
            Err(LibraryError::Cancelled)
        ));
    }
}

#[cfg(all(windows, feature = "library-scan-profile-fixture"))]
mod scan_profile {
    use std::{cell::RefCell, time::Instant};
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentThread, GetThreadTimes},
    };
    thread_local! {static TOTALS: RefCell<[(u128,u64,u64);9]> = const { RefCell::new([(0,0,0);9]) };}
    fn cpu() -> Option<u64> {
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        // SAFETY: current-thread pseudo HANDLE and typed live FILETIME outputs.
        if unsafe {
            GetThreadTimes(
                GetCurrentThread(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        } == 0
        {
            return None;
        }
        let ticks = |t: FILETIME| (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime);
        ticks(kernel).checked_add(ticks(user))
    }
    pub(super) struct Phase {
        index: usize,
        start: Instant,
        cpu: Option<u64>,
    }
    impl Phase {
        pub(super) fn new(index: usize) -> Self {
            Self {
                index,
                start: Instant::now(),
                cpu: cpu(),
            }
        }
    }
    impl Drop for Phase {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed().as_nanos();
            let delta = self.cpu.zip(cpu()).and_then(|(a, b)| b.checked_sub(a));
            TOTALS.with(|s| {
                let mut s = s.borrow_mut();
                s[self.index].0 += elapsed;
                s[self.index].1 += delta.unwrap_or(0);
                s[self.index].2 += u64::from(delta.is_none());
            });
        }
    }
    pub(super) struct Scan {
        start: Instant,
        cpu: Option<u64>,
    }
    impl Scan {
        pub(super) fn new() -> Self {
            TOTALS.with(|s| *s.borrow_mut() = [(0, 0, 0); 9]);
            Self {
                start: Instant::now(),
                cpu: cpu(),
            }
        }
    }
    impl Drop for Scan {
        fn drop(&mut self) {
            TOTALS.with(|s|eprintln!("DLPAD_SCAN_PROFILE elapsed_ns={} thread_cpu_100ns={:?} phases_preflight_setup_traversal_insert_candidates_update_upsert_delete_clear_commit={:?}",self.start.elapsed().as_nanos(),self.cpu.zip(cpu()).and_then(|(a,b)|b.checked_sub(a)),*s.borrow()));
        }
    }
}
