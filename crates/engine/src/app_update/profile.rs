//! Read-only compatibility preflight for a quiescent profile before candidate UI commit.
//! The original bootstrap keeps its existing crash recovery. No migration or backup restore here.
use rusqlite::{Connection, OpenFlags};
use std::{
    fs, io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub const PROFILE_EPOCH: u32 = 1;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    TimedOut,
    Incompatible,
    PendingRecovery,
    Io,
    Database,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app profile preflight failed ({self:?})")
    }
}
impl std::error::Error for Error {}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<(), Error> {
    if cancel.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Error::TimedOut)
    } else {
        Ok(())
    }
}
fn uri(path: &Path) -> Result<String, Error> {
    let native = path.to_str().ok_or(Error::Incompatible)?.replace('\\', "/");
    // Canonical Windows local paths carry the verbatim prefix. It is not a URI authority.
    let text = native.strip_prefix("//?/").unwrap_or(&native);
    if text.starts_with("//") || text.starts_with("UNC/") {
        return Err(Error::Incompatible);
    }
    let mut encoded = String::from("file:///");
    for b in text.trim_start_matches('/').bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~:".contains(&b) {
            encoded.push(char::from(b));
        } else {
            use std::fmt::Write;
            write!(&mut encoded, "%{b:02X}").map_err(|_| Error::Io)?;
        }
    }
    encoded.push_str("?mode=ro&immutable=1");
    Ok(encoded)
}
fn regular(path: &Path) -> Result<Option<fs::Metadata>, Error> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() && !m.file_type().is_symlink() => {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if m.file_attributes() & 0x400 != 0 {
                    return Err(Error::Incompatible);
                }
            }
            Ok(Some(m))
        }
        Ok(_) => Err(Error::Incompatible),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(Error::Io),
    }
}
fn inspect_database(
    path: &Path,
    migration: &str,
    maximum: i64,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), Error> {
    checkpoint(cancel, deadline)?;
    if regular(path)?.is_none() {
        return Ok(());
    }
    for suffix in ["-wal", "-journal"] {
        let mut sidecar = path.as_os_str().to_os_string();
        sidecar.push(suffix);
        if regular(Path::new(&sidecar))?.is_some_and(|m| m.len() != 0) {
            return Err(Error::PendingRecovery);
        }
    }
    // immutable mode cannot create a -shm or run hot-journal recovery. Caller holds InstanceGuard.
    let db = Connection::open_with_flags(
        uri(path)?,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|_| Error::Database)?;
    db.busy_timeout(Duration::ZERO)
        .map_err(|_| Error::Database)?;
    let mut statement = db
        .prepare(&format!(
            "SELECT version FROM {migration} ORDER BY version LIMIT 33"
        ))
        .map_err(|_| Error::Incompatible)?;
    let rows = statement
        .query_map([], |r| r.get::<_, i64>(0))
        .map_err(|_| Error::Incompatible)?;
    let mut count = 0;
    let mut observed_version = 0;
    for row in rows {
        checkpoint(cancel, deadline)?;
        let value = row.map_err(|_| Error::Incompatible)?;
        observed_version = observed_version.max(value);
        count += 1;
        if count > 32 || value < 1 || value > maximum {
            return Err(Error::Incompatible);
        }
    }
    if count == 0 {
        return Err(Error::Incompatible);
    }
    // Migration labels alone do not prove that the old reader can open this schema.
    // Prepare constant zero-row queries, without reading user records or migrating.
    let required: &[&str] = match migration {
        "schema_migrations" => &[
            "SELECT key,value FROM settings LIMIT 0",
            "SELECT root,path,size,modified_ms,present FROM library_entries LIMIT 0",
        ],
        "download_schema_migrations" => &[
            "SELECT id,revision,job_json,snapshot,tool_path,tool_version FROM download_jobs LIMIT 0",
            "SELECT job_id,attempt_id,number,tool_path,tool_version FROM download_attempts LIMIT 0",
        ],
        _ => return Err(Error::Incompatible),
    };
    for query in required {
        checkpoint(cancel, deadline)?;
        db.prepare(query).map_err(|_| Error::Incompatible)?;
    }
    if migration == "schema_migrations" && observed_version >= 2 {
        db.prepare("SELECT file_identity FROM library_entries LIMIT 0")
            .map_err(|_| Error::Incompatible)?;
    }
    if migration == "download_schema_migrations" {
        db.prepare("SELECT options_json,planned_root,planned_path,planned_source,planned_bytes FROM download_jobs LIMIT 0")
            .map_err(|_| Error::Incompatible)?;
        if observed_version >= 3 {
            db.prepare("SELECT job_id,attempt_id,component,tool_path,tool_version FROM download_attempt_components LIMIT 0")
                .map_err(|_| Error::Incompatible)?;
        }
        if observed_version >= 5 {
            db.prepare("SELECT planned_artifacts_json FROM download_jobs LIMIT 0")
                .map_err(|_| Error::Incompatible)?;
        }
    }
    checkpoint(cancel, deadline)
}
/// Only an epoch-compatible candidate may pass. Original app recovery must run separately.
pub fn inspect(
    root: &Path,
    epoch: u32,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), Error> {
    checkpoint(cancel, deadline)?;
    if epoch != PROFILE_EPOCH {
        return Err(Error::Incompatible);
    }
    let metadata = fs::symlink_metadata(root).map_err(|_| Error::Io)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Error::Incompatible);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(Error::Incompatible);
        }
    }
    for name in ["theme.sqlite", "library.sqlite"] {
        inspect_database(&root.join(name), "schema_migrations", 2, cancel, deadline)?;
    }
    inspect_database(
        &root.join("jobs.sqlite"),
        "download_schema_migrations",
        5,
        cancel,
        deadline,
    )
}
#[cfg(test)]
mod tests;
