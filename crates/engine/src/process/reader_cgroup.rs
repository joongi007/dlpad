//! Pinned cgroup2 identity and bounded empty-state evidence; never mutates a cgroup.
#![cfg(target_os = "linux")]
use std::{
    collections::HashSet,
    ffi::CString,
    fs::File,
    io::{self, Read},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    path::Path,
    time::{Duration, Instant},
};

const EVENTS_LIMIT: usize = 4096;
const PROCS_LIMIT: usize = 65536;
const READ_LIMIT: Duration = Duration::from_secs(1);

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CgroupIdentity {
    pub(crate) mount_id: u64,
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) relative_path: String,
}
impl std::fmt::Debug for CgroupIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CgroupIdentity([redacted])")
    }
}
pub(crate) struct PinnedCgroup {
    directory: File,
    identity: CgroupIdentity,
}
impl std::fmt::Debug for PinnedCgroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PinnedCgroup([redacted])")
    }
}
fn rejected() -> io::Error {
    io::Error::other("cgroup ownership evidence unavailable")
}
fn open_at(parent: RawFd, name: &str, directory: bool) -> io::Result<File> {
    let name = CString::new(name).map_err(|_| rejected())?;
    let flags = libc::O_RDONLY
        | libc::O_NOFOLLOW
        | libc::O_CLOEXEC
        | libc::O_NONBLOCK
        | if directory { libc::O_DIRECTORY } else { 0 };
    // SAFETY: name is a live NUL-terminated string; returned descriptor is uniquely owned.
    let fd = unsafe { libc::openat(parent, name.as_ptr(), flags) };
    if fd < 0 {
        return Err(rejected());
    }
    // SAFETY: successful openat returned a fresh descriptor.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn native(file: &File) -> io::Result<(u64, u64, u64)> {
    // SAFETY: initialized output structs and valid live descriptor are passed to libc.
    let (mut fs, mut stat, mut sx): (libc::statfs, libc::stat, libc::statx) =
        unsafe { std::mem::zeroed() };
    let empty = c"";
    // SAFETY: all output pointers are writable; AT_EMPTY_PATH addresses the held descriptor.
    let valid = unsafe {
        libc::fstatfs(file.as_raw_fd(), &mut fs) == 0
            && libc::fstat(file.as_raw_fd(), &mut stat) == 0
            && libc::statx(
                file.as_raw_fd(),
                empty.as_ptr(),
                libc::AT_EMPTY_PATH | libc::AT_SYMLINK_NOFOLLOW,
                libc::STATX_MNT_ID | libc::STATX_INO | libc::STATX_TYPE,
                &mut sx,
            ) == 0
    };
    if !valid
        || fs.f_type as u64 != libc::CGROUP2_SUPER_MAGIC as u64
        || stat.st_mode & libc::S_IFMT != libc::S_IFDIR
        || sx.stx_mask & libc::STATX_MNT_ID == 0
        || sx.stx_mnt_id == 0
        || sx.stx_ino != stat.st_ino
    {
        return Err(rejected());
    }
    Ok((sx.stx_mnt_id, stat.st_dev, stat.st_ino))
}
impl PinnedCgroup {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        let text = path.to_str().ok_or_else(rejected)?;
        let group = text.strip_prefix("/sys/fs/cgroup/").ok_or_else(rejected)?;
        if group.is_empty()
            || group.len() + 1 > 512
            || group.split('/').any(|part| {
                part.is_empty() || part == "." || part == ".." || part.as_bytes().contains(&0)
            })
        {
            return Err(rejected());
        }
        let mut directory = open_at(libc::AT_FDCWD, "/", true)?;
        for part in text[1..].split('/') {
            directory = open_at(directory.as_raw_fd(), part, true)?;
        }
        let (mount_id, device, inode) = native(&directory)?;
        Ok(Self {
            directory,
            identity: CgroupIdentity {
                mount_id,
                device,
                inode,
                relative_path: format!("/{group}"),
            },
        })
    }
    pub(crate) fn identity(&self) -> &CgroupIdentity {
        &self.identity
    }
    pub(crate) fn confirm_empty(&self) -> io::Result<()> {
        let expected = (
            self.identity.mount_id,
            self.identity.device,
            self.identity.inode,
        );
        if native(&self.directory)? != expected {
            return Err(rejected());
        }
        let events = bounded(&self.directory, "cgroup.events", EVENTS_LIMIT)?;
        let procs = bounded(&self.directory, "cgroup.procs", PROCS_LIMIT)?;
        parse_empty(&events, &procs)?;
        if native(&self.directory)? != expected {
            return Err(rejected());
        }
        Ok(())
    }
}
fn bounded(directory: &File, name: &str, limit: usize) -> io::Result<Vec<u8>> {
    let mut file = open_at(directory.as_raw_fd(), name, false)?;
    if !file.metadata().map_err(|_| rejected())?.is_file() {
        return Err(rejected());
    }
    let expected_mount = native(directory)?.0;
    // SAFETY: writable initialized structs and a live anchored file descriptor.
    let (mut fs, mut sx): (libc::statfs, libc::statx) = unsafe { std::mem::zeroed() };
    // SAFETY: AT_EMPTY_PATH inspects only this held file.
    let valid = unsafe {
        libc::fstatfs(file.as_raw_fd(), &mut fs) == 0
            && libc::statx(
                file.as_raw_fd(),
                c"".as_ptr(),
                libc::AT_EMPTY_PATH | libc::AT_SYMLINK_NOFOLLOW,
                libc::STATX_MNT_ID,
                &mut sx,
            ) == 0
    };
    if !valid
        || fs.f_type as u64 != libc::CGROUP2_SUPER_MAGIC as u64
        || sx.stx_mask & libc::STATX_MNT_ID == 0
        || sx.stx_mnt_id != expected_mount
    {
        return Err(rejected());
    }
    let deadline = Instant::now() + READ_LIMIT;
    let mut bytes = Vec::with_capacity(limit.min(4096));
    loop {
        if Instant::now() >= deadline {
            return Err(rejected());
        }
        let mut poll = libc::pollfd {
            fd: file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: poll points to one live descriptor entry and uses a bounded timeout.
        let ready = unsafe { libc::poll(&mut poll, 1, 20) };
        if ready < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(rejected());
        }
        if ready == 0 {
            continue;
        }
        // kernfs may report POLLERR together with readable notification data.
        if poll.revents & libc::POLLNVAL != 0 {
            return Err(rejected());
        }
        let mut block = [0_u8; 4096];
        let available = (limit + 1 - bytes.len()).min(block.len());
        match file.read(&mut block[..available]) {
            Ok(0) => return Ok(bytes),
            Ok(count) => {
                bytes.extend_from_slice(&block[..count]);
                if bytes.len() > limit {
                    return Err(rejected());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return Err(rejected()),
        }
    }
}
fn parse_empty(events: &[u8], procs: &[u8]) -> io::Result<()> {
    if events.is_empty()
        || events.len() > EVENTS_LIMIT
        || procs.len() > PROCS_LIMIT
        || !procs.is_empty()
        || events.last() != Some(&b'\n')
    {
        return Err(rejected());
    }
    if events.contains(&b'\r') || events.contains(&0) {
        return Err(rejected());
    }
    let text = std::str::from_utf8(events).map_err(|_| rejected())?;
    let mut keys = HashSet::new();
    let mut populated = None;
    for line in text.lines() {
        let mut fields = line.split(' ');
        let key = fields.next().ok_or_else(rejected)?;
        let value = fields.next().ok_or_else(rejected)?;
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            || value.is_empty()
            || !value.bytes().all(|b| b.is_ascii_digit())
            || fields.next().is_some()
            || !keys.insert(key)
        {
            return Err(rejected());
        }
        let value: u64 = value.parse().map_err(|_| rejected())?;
        if key == "populated" {
            populated = Some(value);
        }
    }
    if populated != Some(0) {
        return Err(rejected());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_empty_and_populated_rejection() {
        assert!(parse_empty(b"populated 0\nfrozen 0\n", b"").is_ok());
        assert!(parse_empty(b"populated 1\n", b"").is_err());
        assert!(parse_empty(b"populated 0\n", b"123\n").is_err());
    }
    #[test]
    fn malformed_partial_and_duplicate_rejected() {
        for bytes in [
            b"frozen 0\n".as_slice(),
            b"populated 0\npopulated 0\n",
            b"populated -1\n",
            b"populated 2\n",
            b"populated 0",
            b"populated  0\n",
            b"populated 0\nfrozen 0\nfrozen 0\n",
            b"populated 0\r\n",
        ] {
            assert!(parse_empty(bytes, b"").is_err());
        }
    }
    #[test]
    fn byte_bounds_rejected() {
        assert!(parse_empty(&vec![b'x'; EVENTS_LIMIT + 1], b"").is_err());
        assert!(parse_empty(b"populated 0\n", &vec![b'1'; PROCS_LIMIT + 1]).is_err());
    }
    #[test]
    fn foreign_filesystem_and_paths_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let file = File::open(dir.path()).unwrap();
        assert!(native(&file).is_err());
        std::os::unix::fs::symlink(dir.path(), dir.path().join("linked")).unwrap();
        assert!(open_at(file.as_raw_fd(), "linked", true).is_err());
        for path in [
            "/sys/fs/cgroup",
            "/sys/fs/cgroup/",
            "/sys/fs/cgroup/./x",
            "/sys/fs/cgroup/../x",
            "/sys/fs/cgroup/x//y",
            "/tmp/x",
        ] {
            assert!(PinnedCgroup::open(Path::new(path)).is_err());
        }
        let long = format!("/sys/fs/cgroup/{}", "x".repeat(512));
        assert!(PinnedCgroup::open(Path::new(&long)).is_err());
    }
}
