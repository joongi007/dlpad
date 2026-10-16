//! SQLite storage; callers run these blocking operations away from the UI thread.
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

pub struct Store {
    pub(crate) connection: Connection,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    /// Reads an existing setting without creating a database or running migrations.
    /// The byte limit is checked by SQLite before a String is materialized.
    pub fn read_setting_readonly(
        path: &Path,
        key: &str,
        max_bytes: usize,
    ) -> rusqlite::Result<Option<String>> {
        let connection = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        let limit = i64::try_from(max_bytes).unwrap_or(i64::MAX);
        connection
            .query_row(
                "SELECT length(CAST(value AS BLOB)),
                 CASE WHEN length(CAST(value AS BLOB)) <= ?2 THEN value ELSE NULL END
                 FROM settings WHERE key=?1",
                rusqlite::params![key, limit],
                |row| {
                    let length: i64 = row.get(0)?;
                    if length > limit {
                        return Err(rusqlite::Error::SqliteFailure(
                            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_TOOBIG),
                            Some("setting exceeds byte limit".into()),
                        ));
                    }
                    row.get(1)
                },
            )
            .optional()
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(mut connection: Connection) -> rusqlite::Result<Self> {
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys=ON;")?;
        let transaction = connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS library_entries(
                root TEXT NOT NULL, path TEXT NOT NULL, size INTEGER NOT NULL,
                modified_ms INTEGER, present INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY(root,path));
             INSERT OR IGNORE INTO schema_migrations(version) VALUES(1);",
        )?;
        let columns: Vec<String> = transaction
            .prepare("PRAGMA table_info(library_entries)")?
            .query_map([], |row| row.get(1))?
            .collect::<Result<_, _>>()?;
        if !columns.iter().any(|column| column == "file_identity") {
            transaction
                .execute_batch("ALTER TABLE library_entries ADD COLUMN file_identity TEXT;")?;
        }
        transaction.execute_batch("CREATE INDEX IF NOT EXISTS library_identity ON library_entries(root,file_identity); INSERT OR IGNORE INTO schema_migrations(version) VALUES(2);")?;
        transaction.commit()?;
        Ok(Self { connection })
    }

    pub fn setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [key,value])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readonly_setting_bounds_unicode_bytes_and_preserves_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.sqlite");
        let store = Store::open(&path).unwrap();
        store.set_setting("theme", "한글").unwrap();
        drop(store);
        let before = std::fs::read(&path).unwrap();
        assert_eq!(
            Store::read_setting_readonly(&path, "theme", 6)
                .unwrap()
                .as_deref(),
            Some("한글")
        );
        assert_eq!(
            Store::read_setting_readonly(&path, "missing", 0).unwrap(),
            None
        );
        assert!(
            matches!(Store::read_setting_readonly(&path, "theme", 5), Err(rusqlite::Error::SqliteFailure(error, _)) if error.code == rusqlite::ErrorCode::TooBig)
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn readonly_setting_never_creates_or_migrates_and_preserves_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.sqlite");
        assert!(Store::read_setting_readonly(&missing, "theme", 4096).is_err());
        assert!(!missing.exists());
        let empty = dir.path().join("empty.sqlite");
        drop(Connection::open(&empty).unwrap());
        let before = std::fs::read(&empty).unwrap();
        assert!(Store::read_setting_readonly(&empty, "theme", 4096).is_err());
        assert_eq!(std::fs::read(&empty).unwrap(), before);
        let corrupt = dir.path().join("corrupt.sqlite");
        std::fs::write(&corrupt, b"synthetic invalid database").unwrap();
        assert!(Store::read_setting_readonly(&corrupt, "theme", 4096).is_err());
        assert_eq!(
            std::fs::read(&corrupt).unwrap(),
            b"synthetic invalid database"
        );
    }

    #[test]
    fn readonly_setting_busy_failure_preserves_selection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked.sqlite");
        let store = Store::open(&path).unwrap();
        store.set_setting("theme", "Dark").unwrap();
        store.connection.execute_batch("BEGIN EXCLUSIVE").unwrap();
        assert!(
            matches!(Store::read_setting_readonly(&path, "theme", 4096), Err(rusqlite::Error::SqliteFailure(error, _)) if error.code == rusqlite::ErrorCode::DatabaseBusy)
        );
        store.connection.execute_batch("ROLLBACK").unwrap();
        assert_eq!(store.setting("theme").unwrap().as_deref(), Some("Dark"));
    }

    #[cfg(unix)]
    #[test]
    fn readonly_setting_refuses_final_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("real.sqlite");
        Store::open(&path)
            .unwrap()
            .set_setting("theme", "Light")
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        let link = dir.path().join("link.sqlite");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(Store::read_setting_readonly(&link, "theme", 4096).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn legacy_library_migration_preserves_rows_and_is_idempotent() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE library_entries(root TEXT NOT NULL,path TEXT NOT NULL,size INTEGER NOT NULL,modified_ms INTEGER,present INTEGER NOT NULL DEFAULT 1,PRIMARY KEY(root,path)); INSERT INTO library_entries VALUES('/fixture','/fixture/media.mp4',17,123,1);").unwrap();
        let store = Store::from_connection(connection).unwrap();
        let row: (String, i64, Option<String>) = store
            .connection
            .query_row(
                "SELECT path,size,file_identity FROM library_entries",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(row, ("/fixture/media.mp4".into(), 17, None));
        let store = Store::from_connection(store.connection).unwrap();
        assert_eq!(
            store
                .connection
                .query_row("SELECT COUNT(*) FROM library_entries", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn failed_setting_write_preserves_previous_selection() {
        let store = Store::in_memory().unwrap();
        store.set_setting("theme", "dark").unwrap();
        store
            .connection
            .execute_batch("PRAGMA query_only=ON;")
            .unwrap();
        assert!(store.set_setting("theme", "light").is_err());
        assert_eq!(store.setting("theme").unwrap().as_deref(), Some("dark"));
        store
            .connection
            .execute_batch("PRAGMA query_only=OFF;")
            .unwrap();
        store.set_setting("theme", "system").unwrap();
        assert_eq!(store.setting("theme").unwrap().as_deref(), Some("system"));
    }
    #[test]
    fn migration_and_settings_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.sqlite");
        Store::open(&path)
            .unwrap()
            .set_setting("theme", "system")
            .unwrap();
        let db = Store::open(&path).unwrap();
        assert_eq!(db.setting("theme").unwrap().as_deref(), Some("system"));
        assert_eq!(db.setting("missing").unwrap(), None);
    }
}
