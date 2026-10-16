//! Narrow launcher-owned receipt writer. Native terminal-empty proof must be supplied by the caller.
use super::reader_protocol::{
    Capability, CleanupReceipt, RECEIPT_SIZE, encode_capability, encode_cleanup_receipt,
};
use std::{
    fmt,
    fs::File,
    io::{self, Write},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::fs::MetadataExt,
    },
};
const MAX_WRITE_ATTEMPTS: usize = 32;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReceiptError {
    InvalidHandle,
    OwnershipChanged,
    CapabilityMismatch,
    LockUnavailable,
    WriteFailed,
    AlreadyAttempted,
}
impl fmt::Debug for ReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderReceiptError")
    }
}
impl fmt::Display for ReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cookie reader receipt unavailable")
    }
}
impl std::error::Error for ReceiptError {}
pub(crate) struct ReceiptHandles {
    writer: File,
    lock: File,
    capability: Capability,
    record_dev: u64,
    record_ino: u64,
    attempted: bool,
}
impl fmt::Debug for ReceiptHandles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderReceiptHandles(<redacted>)")
    }
}
fn private(fd: &OwnedFd) -> Result<(), ReceiptError> {
    if fd.as_raw_fd() < 3 {
        return Err(ReceiptError::InvalidHandle);
    }
    // SAFETY: restore flags on the supplied live owned descriptor only.
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) };
    if flags < 0
        || unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
    {
        return Err(ReceiptError::InvalidHandle);
    }
    Ok(())
}
fn check(file: &File, dev: u64, ino: u64, access: i32) -> Result<(), ReceiptError> {
    let meta = file
        .metadata()
        .map_err(|_| ReceiptError::OwnershipChanged)?;
    // SAFETY: bounded metadata/flag queries for owned descriptors.
    let uid = unsafe { libc::geteuid() };
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    let descriptor = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
    if !meta.is_file()
        || meta.uid() != uid
        || meta.mode() & 0o7777 != 0o600
        || meta.nlink() != 1
        || meta.dev() != dev
        || meta.ino() != ino
        || ino == 0
        || flags < 0
        || flags & libc::O_ACCMODE != access
        || descriptor < 0
        || descriptor & libc::FD_CLOEXEC == 0
    {
        return Err(ReceiptError::OwnershipChanged);
    }
    Ok(())
}
impl ReceiptHandles {
    pub fn validate(
        writer: OwnedFd,
        lock: OwnedFd,
        capability: &Capability,
        expected_record_dev: u64,
        expected_record_ino: u64,
    ) -> Result<Self, ReceiptError> {
        private(&writer)?;
        private(&lock)?;
        if writer.as_raw_fd() == lock.as_raw_fd()
            || (capability.receipt_dev == expected_record_dev
                && capability.receipt_ino == expected_record_ino)
        {
            return Err(ReceiptError::InvalidHandle);
        }
        encode_capability(capability).map_err(|_| ReceiptError::CapabilityMismatch)?;
        let writer = File::from(writer);
        let lock = File::from(lock);
        check(
            &writer,
            capability.receipt_dev,
            capability.receipt_ino,
            libc::O_WRONLY,
        )?;
        check(
            &lock,
            expected_record_dev,
            expected_record_ino,
            libc::O_RDWR,
        )?;
        if writer
            .metadata()
            .map_err(|_| ReceiptError::OwnershipChanged)?
            .len()
            != 0
        {
            return Err(ReceiptError::AlreadyAttempted);
        }
        // SAFETY: nonblocking flock on the original record open description; never unlock here.
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(ReceiptError::LockUnavailable);
        }
        Ok(Self {
            writer,
            lock,
            capability: capability.clone(),
            record_dev: expected_record_dev,
            record_ino: expected_record_ino,
            attempted: false,
        })
    }
    fn revalidate(&self) -> Result<(), ReceiptError> {
        check(
            &self.writer,
            self.capability.receipt_dev,
            self.capability.receipt_ino,
            libc::O_WRONLY,
        )?;
        check(&self.lock, self.record_dev, self.record_ino, libc::O_RDWR)?;
        // The same open-description lock remains held across app disappearance.
        if unsafe { libc::flock(self.lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(ReceiptError::LockUnavailable);
        }
        Ok(())
    }
    /// The caller must first prove same owned Invocation and pinned cgroup terminal-empty.
    /// Success acknowledges durable cleanup evidence, not successful tool execution.
    pub fn write_receipt(&mut self, receipt: &CleanupReceipt) -> Result<(), ReceiptError> {
        self.write_with(receipt, |file, bytes| file.write(bytes))
    }
    fn write_with(
        &mut self,
        receipt: &CleanupReceipt,
        mut write: impl FnMut(&mut File, &[u8]) -> io::Result<usize>,
    ) -> Result<(), ReceiptError> {
        if self.attempted {
            return Err(ReceiptError::AlreadyAttempted);
        }
        self.revalidate()?;
        if receipt.capability != self.capability {
            return Err(ReceiptError::CapabilityMismatch);
        }
        let bytes =
            encode_cleanup_receipt(receipt).map_err(|_| ReceiptError::CapabilityMismatch)?;
        if bytes.len() != RECEIPT_SIZE
            || self
                .writer
                .metadata()
                .map_err(|_| ReceiptError::OwnershipChanged)?
                .len()
                != 0
        {
            return Err(ReceiptError::AlreadyAttempted);
        }
        self.attempted = true;
        let mut offset = 0;
        for _ in 0..MAX_WRITE_ATTEMPTS {
            self.revalidate()?;
            match write(&mut self.writer, &bytes[offset..]) {
                Ok(0) => return Err(ReceiptError::WriteFailed),
                Ok(count) if count <= bytes.len() - offset => offset += count,
                Ok(_) => return Err(ReceiptError::WriteFailed),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(ReceiptError::WriteFailed),
            }
            if offset == bytes.len() {
                break;
            }
        }
        if offset != bytes.len() {
            return Err(ReceiptError::WriteFailed);
        }
        self.revalidate()?;
        self.writer
            .sync_all()
            .map_err(|_| ReceiptError::WriteFailed)?;
        self.revalidate()?;
        if self
            .writer
            .metadata()
            .map_err(|_| ReceiptError::OwnershipChanged)?
            .len()
            != RECEIPT_SIZE as u64
        {
            return Err(ReceiptError::WriteFailed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::reader_protocol::{ReaderIdentity, decode_cleanup_receipt};
    use super::*;
    use std::{fs::OpenOptions, os::unix::fs::PermissionsExt};
    fn files() -> (tempfile::TempDir, File, File, Capability) {
        let folder = tempfile::tempdir().unwrap();
        let writer = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(folder.path().join("receipt"))
            .unwrap();
        let lock = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(folder.path().join("record"))
            .unwrap();
        for name in ["receipt", "record"] {
            std::fs::set_permissions(
                folder.path().join(name),
                std::fs::Permissions::from_mode(0o600),
            )
            .unwrap();
        }
        let meta = writer.metadata().unwrap();
        let capability = Capability {
            index: 0,
            nonce: [1; 16],
            digest: [2; 32],
            boot: [3; 16],
            receipt_dev: meta.dev(),
            receipt_ino: meta.ino(),
        };
        (folder, writer, lock, capability)
    }
    fn handles(writer: File, lock: File, cap: &Capability) -> Result<ReceiptHandles, ReceiptError> {
        let meta = lock.metadata().unwrap();
        ReceiptHandles::validate(writer.into(), lock.into(), cap, meta.dev(), meta.ino())
    }
    fn receipt(cap: Capability) -> CleanupReceipt {
        CleanupReceipt::from_identity(&ReaderIdentity {
            capability: cap,
            identity_digest: [4; 32],
            unit: [5; 16],
            invocation: [6; 16],
            cgroup_mount: 7,
            cgroup_dev: 8,
            cgroup_ino: 9,
            path: "/owned.service".into(),
        })
    }
    fn assert_flock_released(file: &File) {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        let errno = if result < 0 {
            io::Error::last_os_error().raw_os_error()
        } else {
            None
        };
        if result != 0 {
            let meta = file.metadata().unwrap();
            let mut matching = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
                for entry in entries.flatten().take(4096) {
                    let Ok(fd) = entry.file_name().to_string_lossy().parse::<i32>() else {
                        continue;
                    };
                    // SAFETY: numeric diagnostic queries only; no FD adoption or mutation.
                    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
                    if unsafe { libc::fstat(fd, &mut stat) } == 0
                        && stat.st_dev == meta.dev()
                        && stat.st_ino == meta.ino()
                    {
                        matching.push((fd, unsafe { libc::fcntl(fd, libc::F_GETFD) }));
                    }
                }
            }
            eprintln!(
                "receipt lock numeric diagnostic: errno={errno:?} fd={} dev={} ino={} mode={} nlink={} flags={} matching={matching:?}",
                file.as_raw_fd(),
                meta.dev(),
                meta.ino(),
                meta.mode(),
                meta.nlink(),
                unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) }
            );
        }
        assert_eq!(
            result, 0,
            "record flock after final handle drop errno={errno:?}"
        );
    }
    #[test]
    fn actual_256_sync_and_record_lock_live_until_handles_drop() {
        if super::super::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, writer, lock, cap) = files();
        let clone = lock.try_clone().unwrap();
        assert_eq!(
            unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        let mut handles = handles(writer, clone, &cap).unwrap();
        drop(lock);
        let other = File::open(folder.path().join("record")).unwrap();
        assert_ne!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        let value = receipt(cap);
        handles.write_receipt(&value).unwrap();
        let bytes = std::fs::read(folder.path().join("receipt")).unwrap();
        assert_eq!(bytes.len(), 256);
        assert_eq!(decode_cleanup_receipt(&bytes).unwrap(), value);
        assert_eq!(
            handles.write_receipt(&value),
            Err(ReceiptError::AlreadyAttempted)
        );
        drop(handles);
        assert_flock_released(&other);
        assert!(folder.path().join("receipt").is_file());
    }
    #[test]
    fn wrong_access_mode_hardlink_and_expected_identity_fail_closed() {
        let (folder, writer, lock, cap) = files();
        drop(writer);
        let writer = File::open(folder.path().join("receipt")).unwrap();
        assert!(handles(writer, lock, &cap).is_err());
        let (folder, writer, lock, cap) = files();
        std::fs::set_permissions(
            folder.path().join("receipt"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        assert!(handles(writer, lock, &cap).is_err());
        let (folder, writer, lock, cap) = files();
        std::fs::hard_link(folder.path().join("receipt"), folder.path().join("extra")).unwrap();
        assert!(handles(writer, lock, &cap).is_err());
        assert!(folder.path().join("extra").exists());
        let (_folder, writer, lock, mut cap) = files();
        cap.receipt_ino += 1;
        assert!(handles(writer, lock, &cap).is_err());
        let (_folder, writer, lock, cap) = files();
        let meta = lock.metadata().unwrap();
        assert!(
            ReceiptHandles::validate(writer.into(), lock.into(), &cap, meta.dev(), meta.ino() + 1)
                .is_err()
        );
        let (folder, writer, lock, mut cap) = files();
        drop(writer);
        let writer = OpenOptions::new()
            .write(true)
            .open(folder.path().join("record"))
            .unwrap();
        let meta = lock.metadata().unwrap();
        cap.receipt_dev = meta.dev();
        cap.receipt_ino = meta.ino();
        assert!(handles(writer, lock, &cap).is_err());
    }
    #[test]
    fn mismatched_capability_and_partial_write_error_preserve_owned_file() {
        let (folder, writer, lock, cap) = files();
        let mut handles = handles(writer, lock, &cap).unwrap();
        let mut bad = receipt(cap.clone());
        bad.capability.nonce = [9; 16];
        assert_eq!(
            handles.write_receipt(&bad),
            Err(ReceiptError::CapabilityMismatch)
        );
        assert_eq!(
            std::fs::metadata(folder.path().join("receipt"))
                .unwrap()
                .len(),
            0
        );
        let value = receipt(cap);
        let mut first = true;
        let result = handles.write_with(&value, |file, bytes| {
            if first {
                first = false;
                file.write(&bytes[..8])
            } else {
                Err(io::Error::other("synthetic secret write failure"))
            }
        });
        assert_eq!(result, Err(ReceiptError::WriteFailed));
        assert_eq!(
            std::fs::metadata(folder.path().join("receipt"))
                .unwrap()
                .len(),
            8
        );
        assert_eq!(
            handles.write_receipt(&value),
            Err(ReceiptError::AlreadyAttempted)
        );
        drop(handles);
        assert!(folder.path().join("receipt").exists());
    }
    #[test]
    fn interrupted_writer_has_a_fixed_attempt_bound_and_restores_cloexec() {
        let (_folder, writer, lock, cap) = files();
        for fd in [writer.as_raw_fd(), lock.as_raw_fd()] {
            assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }, 0);
        }
        let mut handles = handles(writer, lock, &cap).unwrap();
        for fd in [handles.writer.as_raw_fd(), handles.lock.as_raw_fd()] {
            assert_ne!(
                unsafe { libc::fcntl(fd, libc::F_GETFD) } & libc::FD_CLOEXEC,
                0
            );
        }
        let mut attempts = 0;
        let result = handles.write_with(&receipt(cap), |_, _| {
            attempts += 1;
            Err(io::Error::from(io::ErrorKind::Interrupted))
        });
        assert_eq!(result, Err(ReceiptError::WriteFailed));
        assert_eq!(attempts, MAX_WRITE_ATTEMPTS);
    }
}
