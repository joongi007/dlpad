//! Durable failclosed managed-execution gate. Drop never clears authority.
use super::{ToolError, ToolManager, native_files};
use crate::file_identity;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
const NAME: &str = ".execution-pending";
const LIMIT: u64 = 4096;
/// Immutable observations from retained original handles, not cleanup authority.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkerBinding {
    version: u8,
    session_nonce: [u8; 16],
    root_id: file_identity::NativeFileIdentity,
    marker_id: file_identity::NativeFileIdentity,
    #[cfg(windows)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    managed: Option<ManagedBinding>,
}

#[cfg(windows)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ManagedBinding {
    anchors: super::windows_managed_records::SessionAnchors,
    digest: [u8; 32],
}

#[cfg(windows)]
pub(super) struct RecoverySeed {
    pub(super) root_id: file_identity::NativeFileIdentity,
    pub(super) marker_id: file_identity::NativeFileIdentity,
    pub(super) anchors: super::windows_managed_records::SessionAnchors,
    pub(super) session_nonce: [u8; 16],
    pub(super) digest: [u8; 32],
}
/// Format-only original binding. Caller must separately validate retained native capabilities.
#[cfg(windows)]
pub(super) fn recovery_seed(bytes: &[u8]) -> Result<RecoverySeed, ToolError> {
    use super::windows_managed_records::{ManagedReaderRecord, WindowsAnchor};
    use sha2::{Digest, Sha256};
    if bytes.is_empty() || bytes.len() as u64 > LIMIT {
        return Err(ToolError::UnsafePath);
    }
    let mut binding: MarkerBinding = serde_json::from_slice(bytes)?;
    let managed = binding.managed.take().ok_or(ToolError::UnsafePath)?;
    let to_anchor = |id: &file_identity::NativeFileIdentity| match id {
        file_identity::NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } => Some(WindowsAnchor {
            volume_serial: *volume_serial,
            file_id: *file_id,
        }),
        _ => None,
    };
    if binding.version != 3
        || to_anchor(&binding.root_id) != Some(managed.anchors.root)
        || to_anchor(&binding.marker_id) != Some(managed.anchors.marker)
    {
        return Err(ToolError::UnsafePath);
    }
    // Reuse strict anchor/nonce/domain validation without inventing a reader receipt.
    ManagedReaderRecord::new(binding.session_nonce, managed.digest, managed.anchors)
        .map_err(|_| ToolError::UnsafePath)?;
    binding.version = 2;
    let mut hash = Sha256::new();
    hash.update(b"dlpad-managed-execution-base-v1\0");
    hash.update(serde_json::to_vec(&binding)?);
    let digest: [u8; 32] = hash.finalize().into();
    if digest != managed.digest {
        return Err(ToolError::UnsafePath);
    }
    Ok(RecoverySeed {
        root_id: binding.root_id,
        marker_id: binding.marker_id,
        anchors: managed.anchors,
        session_nonce: binding.session_nonce,
        digest,
    })
}

pub(super) struct ExecutionMarker {
    root: PathBuf,
    root_pin: File,
    marker: File,
    bytes: Vec<u8>,
    #[cfg(windows)]
    sid: String,
}
pub(super) fn pending(root: &Path) -> Result<bool, ToolError> {
    match fs::symlink_metadata(root.join(NAME)) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
fn nonce() -> Result<[u8; 16], ToolError> {
    let mut bytes = [0; 16];
    #[cfg(unix)]
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::Security::Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
        };
        if unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } != 0
        {
            return Err(std::io::Error::other("execution marker random unavailable").into());
        }
    }
    #[cfg(not(any(unix, windows)))]
    return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "execution marker unsupported",
    )
    .into());
    Ok(bytes)
}
impl ExecutionMarker {
    /// Caller holds original managed-root exclusive execution lock throughout creation.
    pub(super) fn create(manager: &ToolManager) -> Result<Self, ToolError> {
        manager.verify_root()?;
        let root = manager.root.clone();
        let root_pin = manager.root_pin.try_clone()?;
        let random = nonce()?;
        #[cfg(windows)]
        let sid = crate::windows_private_security::current_sid()?;
        let mut marker = create_file(&root, &root_pin)?;
        // Any post-create failure intentionally preserves pending evidence.
        let binding = MarkerBinding {
            version: 2,
            session_nonce: random,
            root_id: file_identity::from_file(&root_pin)?,
            marker_id: file_identity::from_file(&marker)?,
            #[cfg(windows)]
            managed: None,
        };
        let bytes = serde_json::to_vec(&binding)?;
        if bytes.len() as u64 > LIMIT {
            return Err(ToolError::UnsafePath);
        }
        marker.write_all(&bytes)?;
        marker.sync_all()?;
        #[cfg(unix)]
        root_pin.sync_all()?;
        let result = Self {
            root,
            root_pin,
            marker,
            bytes,
            #[cfg(windows)]
            sid,
        };
        result.verify()?;
        Ok(result)
    }
    /// Create and seal the managed reader namespace while the caller owns
    /// the exclusive transfer lock. A partial write preserves the pending gate.
    #[cfg(windows)]
    pub(super) fn create_journal(
        &mut self,
        lock: &File,
    ) -> Result<super::windows_managed_journal::ManagedJournal, ToolError> {
        use sha2::{Digest, Sha256};
        self.verify()?;
        let mut binding: MarkerBinding = serde_json::from_slice(&self.bytes)?;
        if binding.version != 2 || binding.managed.is_some() {
            return Err(ToolError::UnsafePath);
        }
        let mut hash = Sha256::new();
        hash.update(b"dlpad-managed-execution-base-v1\0");
        hash.update(&self.bytes);
        let digest: [u8; 32] = hash.finalize().into();
        let journal = super::windows_managed_journal::ManagedJournal::create(
            &self.root_pin,
            &self.root,
            &self.marker,
            lock,
            binding.session_nonce,
            digest,
        )
        .map_err(|_| ToolError::UnsafePath)?;
        binding.version = 3;
        binding.managed = Some(ManagedBinding {
            anchors: journal.anchors(),
            digest,
        });
        let bytes = serde_json::to_vec(&binding)?;
        if bytes.len() as u64 > LIMIT {
            return Err(ToolError::UnsafePath);
        }
        let mut marker = &self.marker;
        marker.seek(SeekFrom::Start(0))?;
        marker.write_all(&bytes)?;
        marker.set_len(bytes.len() as u64)?;
        marker.sync_all()?;
        self.bytes = bytes;
        self.verify()?;
        Ok(journal)
    }
    #[cfg(windows)]
    pub(super) fn verify_journal(
        &self,
        record: &super::windows_managed_records::ManagedReaderRecord,
    ) -> Result<(), ToolError> {
        self.verify()?;
        let seed = recovery_seed(&self.bytes)?;
        if seed.anchors != record.anchors()
            || seed.session_nonce != record.session_nonce()
            || seed.digest != record.record_digest()
        {
            return Err(ToolError::UnsafePath);
        }
        Ok(())
    }
    #[cfg(windows)]
    pub(super) fn root_path(&self) -> &Path {
        &self.root
    }
    fn verify(&self) -> Result<(), ToolError> {
        if !native_files::same_directory(&self.root, &self.root_pin)
            || !native_files::same_file(&self.root.join(NAME), &self.marker)
        {
            return Err(ToolError::UnsafePath);
        }
        if !self.marker.metadata()?.is_file()
            || self.marker.metadata()?.len() != self.bytes.len() as u64
        {
            return Err(ToolError::UnsafePath);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if self.marker.metadata()?.nlink() != 1 {
                return Err(ToolError::UnsafePath);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::*;
            let mut info = BY_HANDLE_FILE_INFORMATION::default();
            if unsafe { GetFileInformationByHandle(self.marker.as_raw_handle(), &mut info) } == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            if info.nNumberOfLinks != 1 {
                return Err(ToolError::UnsafePath);
            }
            file_identity::ensure_regular_under(&self.root_pin, &self.marker)?;
            if crate::windows_private_security::current_sid()? != self.sid {
                return Err(ToolError::UnsafePath);
            }
            crate::windows_private_security::verify_acl(&self.marker, &self.sid)?;
        }
        let mut file = &self.marker;
        file.seek(SeekFrom::Start(0))?;
        let mut actual = Vec::new();
        file.take(LIMIT + 1).read_to_end(&mut actual)?;
        if actual != self.bytes {
            return Err(ToolError::UnsafePath);
        }
        Ok(())
    }
    /// Reserved for completion with confirmed owned-terminal proof; never call on Drop.
    pub(super) fn clear(&self) -> Result<(), ToolError> {
        self.verify()?;
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let name = c".execution-pending";
            let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
            if unsafe {
                libc::fstatat(
                    self.root_pin.as_raw_fd(),
                    name.as_ptr(),
                    stat.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            let stat = unsafe { stat.assume_init() };
            let original = file_identity::from_file(&self.marker)?;
            if original
                != (file_identity::NativeFileIdentity::Unix {
                    device: stat.st_dev,
                    inode: stat.st_ino,
                })
                || stat.st_nlink != 1
            {
                return Err(ToolError::UnsafePath);
            }
            if unsafe { libc::unlinkat(self.root_pin.as_raw_fd(), name.as_ptr(), 0) } != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            self.root_pin.sync_all()?;
            Ok(())
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::*;
            let mut info = FILE_DISPOSITION_INFO { DeleteFile: true };
            if unsafe {
                SetFileInformationByHandle(
                    self.marker.as_raw_handle(),
                    FileDispositionInfo,
                    (&mut info as *mut FILE_DISPOSITION_INFO).cast(),
                    std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "execution marker unsupported",
        )
        .into())
    }
}
fn create_file(root: &Path, root_pin: &File) -> Result<File, ToolError> {
    #[cfg(unix)]
    {
        use std::os::fd::{AsRawFd, FromRawFd};
        let fd = unsafe {
            libc::openat(
                root_pin.as_raw_fd(),
                c".execution-pending".as_ptr(),
                libc::O_CREAT | libc::O_EXCL | libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let _ = root;
        Ok(unsafe { File::from_raw_fd(fd) })
    }
    #[cfg(windows)]
    {
        use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
        use windows_sys::Win32::{Foundation::INVALID_HANDLE_VALUE, Storage::FileSystem::*};
        let sid = crate::windows_private_security::current_sid()?;
        let descriptor = crate::windows_private_security::PrivateDescriptor::new(&sid)?;
        let attributes = descriptor.attributes();
        let mut path: Vec<u16> = root
            .join(NAME)
            .as_os_str()
            .encode_wide()
            .take(32768)
            .collect();
        if path.len() > 32767 || path.contains(&0) {
            return Err(ToolError::UnsafePath);
        }
        path.push(0);
        // SAFETY: bounded NUL-terminated path and live private descriptor; the
        // fresh non-inheritable CREATE_NEW handle is transferred once to File.
        let raw = unsafe {
            CreateFileW(
                path.as_ptr(),
                DELETE | FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_READ,
                &attributes,
                CREATE_NEW,
                FILE_FLAG_OPEN_REPARSE_POINT,
                std::ptr::null_mut(),
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error().into());
        }
        let file = unsafe { File::from_raw_handle(raw) };
        crate::windows_private_security::verify_acl(&file, &sid)?;
        file_identity::ensure_regular_under(root_pin, &file)?;
        Ok(file)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (root, root_pin);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "execution marker unsupported",
        )
        .into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn widened_acl_preserves_pending_and_foreign_file() {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows_sys::Win32::{
            Security::{
                Authorization::{SE_FILE_OBJECT, SetSecurityInfo},
                DACL_SECURITY_INFORMATION, UNPROTECTED_DACL_SECURITY_INFORMATION,
            },
            Storage::FileSystem::{
                FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, READ_CONTROL, WRITE_DAC,
            },
        };
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let foreign = temp.path().join("foreign");
        fs::write(&foreign, b"synthetic preserve").unwrap();
        let marker = ExecutionMarker::create(&manager).unwrap();
        let original = marker.bytes.clone();
        let writable_acl = std::fs::OpenOptions::new()
            .access_mode(WRITE_DAC | READ_CONTROL)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .open(manager.root.join(NAME))
            .unwrap();
        // This is only our temporary synthetic marker. Deliberately remove its
        // protection to prove clear rejects it while retaining the original file.
        let code = unsafe {
            SetSecurityInfo(
                writable_acl.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(code, 0);
        assert!(marker.clear().is_err());
        assert_eq!(fs::read(manager.root.join(NAME)).unwrap(), original);
        assert!(pending(&manager.root).unwrap());
        assert_eq!(fs::read(foreign).unwrap(), b"synthetic preserve");
    }
    #[test]
    fn recorded_binding_matches_original_handles() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let marker = ExecutionMarker::create(&manager).unwrap();
        let binding: MarkerBinding = serde_json::from_slice(&marker.bytes).unwrap();
        assert_eq!(binding.version, 2);
        assert_ne!(binding.session_nonce, [0; 16]);
        assert_eq!(
            binding.root_id,
            file_identity::from_file(&marker.root_pin).unwrap()
        );
        assert_eq!(
            binding.marker_id,
            file_identity::from_file(&marker.marker).unwrap()
        );
        assert!(marker.bytes.len() as u64 <= LIMIT);
        let mut altered: serde_json::Value = serde_json::from_slice(&marker.bytes).unwrap();
        altered["cleanup_confirmed"] = serde_json::json!(true);
        assert!(serde_json::from_value::<MarkerBinding>(altered).is_err());
        marker.clear().unwrap();
    }
    #[test]
    fn drop_preserves_gate_and_clear_original() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        assert!(!pending(&manager.root).unwrap());
        let marker = ExecutionMarker::create(&manager).unwrap();
        assert!(pending(&manager.root).unwrap());
        marker.clear().unwrap();
        drop(marker);
        assert!(!pending(&manager.root).unwrap());
        let marker = ExecutionMarker::create(&manager).unwrap();
        drop(marker);
        assert!(pending(&manager.root).unwrap());
        assert!(ExecutionMarker::create(&manager).is_err());
    }
    #[test]
    fn tampered_marker_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
        let marker = ExecutionMarker::create(&manager).unwrap();
        #[cfg(unix)]
        {
            fs::write(manager.root.join(NAME), b"foreign changed bytes").unwrap();
            assert!(marker.clear().is_err());
            assert_eq!(
                fs::read(manager.root.join(NAME)).unwrap(),
                b"foreign changed bytes"
            );
        }
        drop(marker);
        assert!(pending(&manager.root).unwrap());
    }
    #[cfg(unix)]
    #[test]
    fn replacement_symlink_hardlink_preserved() {
        use std::os::unix::fs::symlink;
        for kind in 0..3 {
            let temp = tempfile::tempdir().unwrap();
            let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
            let marker = ExecutionMarker::create(&manager).unwrap();
            let path = manager.root.join(NAME);
            let foreign = temp.path().join("foreign");
            fs::write(&foreign, b"foreign").unwrap();
            if kind == 0 {
                fs::rename(&path, manager.root.join("original-marker")).unwrap();
                fs::write(&path, b"replacement").unwrap();
            } else if kind == 1 {
                fs::remove_file(&path).unwrap();
                symlink(&foreign, &path).unwrap();
            } else {
                fs::hard_link(&path, temp.path().join("alias")).unwrap();
            }
            assert!(marker.clear().is_err());
            assert!(pending(&manager.root).unwrap());
            assert_eq!(fs::read(&foreign).unwrap(), b"foreign");
        }
    }
}
