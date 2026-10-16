//! Worker-only, create-only receipt repository. Serialized receipts confer no install authority.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
const OWNER: &[u8] = b"dlpad-tool-update-receipts-v1\n";
pub(crate) const MAX_RECEIPTS: usize = 32;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_LEAF_BYTES: u64 = 2 * 1024 * 1024 + 128 * 1024;
const MAX_REPOSITORY_BYTES: u64 = 72 * 1024 * 1024;
const FILES: [&str; 4] = [
    "LICENSE",
    "THIRD_PARTY_LICENSES.txt",
    "supply-receipt.json",
    "installation-result.json",
];
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Unsafe,
    Busy,
    Full,
    Io,
}
fn io(_: std::io::Error) -> Error {
    Error::Io
}
fn safe(path: &Path, directory: bool) -> Result<(), Error> {
    let m = fs::symlink_metadata(path).map_err(io)?;
    if m.file_type().is_symlink() || (directory && !m.is_dir()) || (!directory && !m.is_file()) {
        return Err(Error::Unsafe);
    }
    #[cfg(all(windows, feature = "cpu"))]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(Error::Unsafe);
        }
    }
    Ok(())
}
fn open(path: &Path, directory: bool, write: bool) -> Result<File, Error> {
    safe(path, directory)?;
    let mut o = OpenOptions::new();
    o.read(true).write(write);
    #[cfg(all(windows, feature = "cpu"))]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        o.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(
                FILE_FLAG_OPEN_REPARSE_POINT
                    | if directory {
                        FILE_FLAG_BACKUP_SEMANTICS
                    } else {
                        0
                    },
            );
        if directory {
            o.access_mode(FILE_READ_ATTRIBUTES);
        }
    }
    let f = o.open(path).map_err(io)?;
    safe(path, directory)?;
    Ok(f)
}
#[cfg(all(windows, feature = "cpu"))]
fn identity(f: &File) -> Result<(u64, u64), Error> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::*;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: borrowed live file handle and initialized correctly sized output.
    if unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut info) } == 0 {
        return Err(Error::Io);
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(Error::Unsafe);
    }
    Ok((
        u64::from(info.dwVolumeSerialNumber),
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    ))
}
#[cfg(unix)]
fn identity(f: &File) -> Result<(u64, u64), Error> {
    use std::os::unix::fs::MetadataExt;
    let m = f.metadata().map_err(io)?;
    Ok((m.dev(), m.ino()))
}
#[cfg(not(any(unix, all(windows, feature = "cpu"))))]
fn identity(_: &File) -> Result<(u64, u64), Error> {
    Err(Error::Unsafe)
}
fn same(path: &Path, pin: &File, directory: bool) -> Result<(), Error> {
    if identity(&open(path, directory, false)?)? != identity(pin)? {
        return Err(Error::Unsafe);
    }
    Ok(())
}
fn bounded_namespace(root: &Path) -> Result<(), Error> {
    if !root.is_absolute()
        || root
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(Error::Unsafe);
    }
    let mut at = PathBuf::new();
    for c in root.components() {
        at.push(c);
        if matches!(c, Component::Normal(_)) {
            safe(&at, true)?;
        }
    }
    Ok(())
}
pub(crate) struct Reservation {
    root: PathBuf,
    parent: PathBuf,
    parent_pin: File,
    root_pin: File,
    lock: File,
    leaf: PathBuf,
}
impl Reservation {
    pub(crate) fn leaf(&self) -> &Path {
        &self.leaf
    }
    pub(crate) fn check(&self) -> Result<(), Error> {
        same(&self.parent, &self.parent_pin, true)?;
        same(&self.root, &self.root_pin, true)?;
        same(&self.root.join(".lock"), &self.lock, false)
    }
    pub(crate) fn retained(&self, path: &Path) -> bool {
        path == self.leaf && self.check().is_ok() && safe(path, true).is_ok()
    }
}
/// Root supplies its existing owned absolute application data directory; never a user-selected path.
/// Hold through source preparation/install/result publication. No cleanup or receipt reopening.
pub(crate) fn reserve(data: &Path, generation: u64) -> Result<Reservation, Error> {
    bounded_namespace(data)?;
    let parent = fs::canonicalize(data).map_err(io)?;
    let parent_pin = open(&parent, true, false)?;
    let root = parent.join("tool-update-receipts");
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&root).map_err(io)?;
            let mut owner = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(".dlpad-owner"))
                .map_err(io)?;
            owner.write_all(OWNER).map_err(io)?;
            owner.sync_all().map_err(io)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(root.join(".lock"))
                .map_err(io)?
                .sync_all()
                .map_err(io)?;
        }
        Ok(_) => {}
        Err(_) => return Err(Error::Io),
    }
    let root_pin = open(&root, true, false)?;
    same(&parent, &parent_pin, true)?;
    let owner = open(&root.join(".dlpad-owner"), false, false)?;
    if owner.metadata().map_err(io)?.len() != OWNER.len() as u64 {
        return Err(Error::Unsafe);
    }
    let mut raw = Vec::with_capacity(OWNER.len());
    owner
        .take(OWNER.len() as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(io)?;
    if raw != OWNER {
        return Err(Error::Unsafe);
    }
    let lock = open(&root.join(".lock"), false, true)?;
    if lock.metadata().map_err(io)?.len() != 0 {
        return Err(Error::Unsafe);
    }
    lock.try_lock().map_err(|_| Error::Busy)?;
    let mut total = 0u64;
    let mut count = 0usize;
    for entry in fs::read_dir(&root).map_err(io)? {
        let entry = entry.map_err(io)?;
        let name = entry.file_name();
        let name = name.to_str().ok_or(Error::Unsafe)?;
        if name == ".dlpad-owner" || name == ".lock" {
            continue;
        }
        if name.len() != 24
            || !name.starts_with("attempt-")
            || !name[8..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::Unsafe);
        }
        count += 1;
        if count > MAX_RECEIPTS {
            return Err(Error::Full);
        }
        let path = entry.path();
        let pin = open(&path, true, false)?;
        let mut bytes = 0u64;
        let mut file_count = 0usize;
        for file in fs::read_dir(&path).map_err(io)? {
            let file = file.map_err(io)?;
            file_count += 1;
            if file_count > FILES.len() {
                return Err(Error::Unsafe);
            }
            let name = file.file_name();
            let name = name.to_str().ok_or(Error::Unsafe)?;
            if !FILES.contains(&name) {
                return Err(Error::Unsafe);
            }
            let f = open(&file.path(), false, false)?;
            let size = f.metadata().map_err(io)?.len();
            let max = if matches!(name, "supply-receipt.json" | "installation-result.json") {
                64 * 1024
            } else {
                MAX_FILE_BYTES
            };
            if size > max {
                return Err(Error::Full);
            }
            bytes = bytes.checked_add(size).ok_or(Error::Full)?;
        }
        same(&path, &pin, true)?;
        if bytes > MAX_LEAF_BYTES {
            return Err(Error::Full);
        }
        total = total.checked_add(bytes).ok_or(Error::Full)?;
    }
    if count >= MAX_RECEIPTS
        || total.checked_add(MAX_LEAF_BYTES).ok_or(Error::Full)? > MAX_REPOSITORY_BYTES
    {
        return Err(Error::Full);
    }
    // Generation is NOT authority; absent slots resolve restart/collision without timestamps or randomness.
    let mut leaf = None;
    for offset in 0..MAX_RECEIPTS {
        let slot = generation.checked_add(offset as u64).ok_or(Error::Full)?;
        let p = root.join(format!("attempt-{slot:016x}"));
        match fs::symlink_metadata(&p) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                leaf = Some(p);
                break;
            }
            Ok(_) => {}
            Err(_) => return Err(Error::Io),
        }
    }
    let reservation = Reservation {
        root,
        parent,
        parent_pin,
        root_pin,
        lock,
        leaf: leaf.ok_or(Error::Full)?,
    };
    reservation.check()?;
    Ok(reservation)
}
#[cfg(all(test, any(unix, all(windows, feature = "cpu"))))]
mod tests {
    use super::*;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "dlpad-receipt-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn fresh_absent_leaf_exclusive_lock_and_retained_source_bytes_survive_reopen() {
        let f = Fixture::new();
        let r = reserve(&f.0, 1).unwrap();
        assert!(!r.leaf().exists());
        assert!(matches!(reserve(&f.0, 2), Err(Error::Busy)));
        fs::create_dir(r.leaf()).unwrap();
        fs::write(r.leaf().join("LICENSE"), b"source evidence").unwrap();
        let old = r.leaf().to_path_buf();
        assert!(r.retained(&old));
        drop(r);
        let r = reserve(&f.0, 1).unwrap();
        assert_ne!(r.leaf(), old);
        assert_eq!(fs::read(old.join("LICENSE")).unwrap(), b"source evidence");
        assert!(!r.leaf().exists());
    }
    #[test]
    fn full_or_foreign_namespace_refuses_without_cleanup() {
        let f = Fixture::new();
        let r = reserve(&f.0, 1).unwrap();
        let root = r.root.clone();
        drop(r);
        fs::write(root.join("foreign.txt"), b"user preserved").unwrap();
        assert!(matches!(reserve(&f.0, 1), Err(Error::Unsafe)));
        assert_eq!(
            fs::read(root.join("foreign.txt")).unwrap(),
            b"user preserved"
        );
        fs::remove_file(root.join("foreign.txt")).unwrap();
        for i in 0..MAX_RECEIPTS {
            let p = root.join(format!("attempt-{i:016x}"));
            fs::create_dir(&p).unwrap();
            fs::write(p.join("LICENSE"), b"keep").unwrap();
        }
        assert!(matches!(reserve(&f.0, 1), Err(Error::Full)));
        assert_eq!(
            fs::read(root.join("attempt-0000000000000000/LICENSE")).unwrap(),
            b"keep"
        );
    }
    #[test]
    fn unmarked_directory_and_oversized_evidence_refuse_and_preserve_existing_bytes() {
        let f = Fixture::new();
        let root = f.0.join("tool-update-receipts");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("personal.txt"), b"keep user bytes").unwrap();
        assert!(matches!(reserve(&f.0, 1), Err(Error::Io)));
        assert_eq!(
            fs::read(root.join("personal.txt")).unwrap(),
            b"keep user bytes"
        );
        assert!(!root.join(".dlpad-owner").exists());
        // Fixture teardown owns these paths; production has no cleanup path.
        fs::remove_file(root.join("personal.txt")).unwrap();
        fs::remove_dir(root).unwrap();
        let r = reserve(&f.0, 1).unwrap();
        let path = r.leaf().to_owned();
        fs::create_dir(&path).unwrap();
        let file = File::create(path.join("LICENSE")).unwrap();
        file.set_len(MAX_FILE_BYTES + 1).unwrap();
        drop(file);
        drop(r);
        assert!(matches!(reserve(&f.0, 2), Err(Error::Full)));
        assert_eq!(
            fs::metadata(path.join("LICENSE")).unwrap().len(),
            MAX_FILE_BYTES + 1
        );
    }
}
