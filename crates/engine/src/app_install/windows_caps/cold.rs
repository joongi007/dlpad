//! Native deletion capabilities obtained only by the explicit removal factory.
//! Expected IDs are supplied by its previously verified binding, never freshly adopted.
//! Drop closes handles without issuing any disposition. No recursive or path fallback.
use super::*;
pub(in crate::app_install) struct DeletionRoot {
    root: ReadRoot,
    delete_file: File,
}
impl DeletionRoot {
    pub(in crate::app_install) fn from_read_root(
        root: ReadRoot,
        expected: &NativeFileIdentity,
    ) -> io::Result<Self> {
        Self::from_read_root_retaining(root, expected).map_err(|failure| failure.1)
    }
    /// Retains the original readonly root on every acquisition/validation failure.
    pub(in crate::app_install) fn from_read_root_retaining(
        root: ReadRoot,
        expected: &NativeFileIdentity,
    ) -> Result<Self, Box<(ReadRoot, io::Error)>> {
        let acquired = (|| {
            root.verify()?;
            if root.identity() != expected {
                return Err(rejected());
            }
            Self::try_acquire_delete_file(&root)
        })();
        match acquired {
            Ok(delete_file) => Ok(Self { root, delete_file }),
            Err(error) => Err(Box::new((root, error))),
        }
    }
    fn try_acquire_delete_file(root: &ReadRoot) -> io::Result<File> {
        // Open the original directory itself, relative to its retained HANDLE.
        // Empty native name + FILE_OPEN never resolves a replacement pathname,
        // creates a directory, changes an ACL or asks for privilege elevation.
        let mut object_name = UNICODE_STRING::default();
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: root.root.file.as_raw_handle(),
            ObjectName: &mut object_name,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        // SAFETY: live retained original directory and aligned synchronous outputs.
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                DELETE | READ_CONTROL | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                &attrs,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_OPEN,
                FILE_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if result < 0 {
            return Err(io::Error::from_raw_os_error(
                unsafe { RtlNtStatusToDosError(result) } as i32,
            ));
        }
        if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(rejected());
        }
        let delete_file = unsafe { File::from_raw_handle(raw) };
        validate_directory(&delete_file)?;
        windows_private_security::verify_acl(&delete_file, &root.root.sid)?;
        if file_identity::from_file(&delete_file)? != *root.identity()
            || file_identity::final_path(&delete_file)? != root.root.final_path
        {
            return Err(rejected());
        }
        root.verify()?;
        Ok(delete_file)
    }
    pub(in crate::app_install) fn verify(&self) -> io::Result<()> {
        self.root.verify()?;
        validate_directory(&self.delete_file)?;
        windows_private_security::verify_acl(&self.delete_file, &self.root.root.sid)?;
        if file_identity::from_file(&self.delete_file)? != *self.root.identity()
            || file_identity::final_path(&self.delete_file)? != self.root.root.final_path
        {
            return Err(rejected());
        }
        self.root.verify()
    }
    pub(in crate::app_install) fn path(&self) -> &Path {
        self.root.path()
    }
    pub(in crate::app_install) fn identity(&self) -> &NativeFileIdentity {
        self.root.identity()
    }
    pub(in crate::app_install) fn open_file(
        &self,
        name: &str,
        expected: &NativeFileIdentity,
    ) -> io::Result<DeletionFile> {
        self.verify()?;
        let mut units = leaf_units(std::ffi::OsStr::new(name))?;
        let length = u16::try_from(units.len() * 2).map_err(|_| rejected())?;
        let mut object_name = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: units.as_mut_ptr(),
        };
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.root.root.file.as_raw_handle(),
            ObjectName: &mut object_name,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        // FILE_OPEN only. No inherited ACL mutation, new file, path adoption or sharing bypass.
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                GENERIC_READ | DELETE | READ_CONTROL | SYNCHRONIZE,
                &attrs,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ,
                FILE_OPEN,
                FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if result < 0 {
            return Err(io::Error::from_raw_os_error(
                unsafe { RtlNtStatusToDosError(result) } as i32,
            ));
        }
        if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(rejected());
        }
        let file = unsafe { File::from_raw_handle(raw) };
        let opened = DeletionFile {
            file,
            name: name.to_owned(),
            identity: expected.clone(),
        };
        opened.verify(self)?;
        Ok(opened)
    }
    pub(in crate::app_install) fn mark_delete_original(&self) -> io::Result<()> {
        self.verify()?;
        if let Some(entry) = std::fs::read_dir(self.path())?.next() {
            entry?;
            return Err(io::Error::new(
                io::ErrorKind::DirectoryNotEmpty,
                "registered root not empty",
            ));
        }
        self.verify()?;
        mark_disposition(&self.delete_file)
    }
}
pub(in crate::app_install) struct DeletionFile {
    file: File,
    name: String,
    identity: NativeFileIdentity,
}
impl DeletionFile {
    pub(in crate::app_install) fn file(&self) -> &File {
        &self.file
    }
    pub(in crate::app_install) fn name(&self) -> &str {
        &self.name
    }
    pub(in crate::app_install) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
    pub(in crate::app_install) fn verify(&self, root: &DeletionRoot) -> io::Result<()> {
        root.verify()?;
        validate_regular(&self.file, true)?;
        windows_private_security::verify_acl(&self.file, &root.root.root.sid)?;
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != root.root.root.final_path.join(&self.name)
        {
            return Err(rejected());
        }
        let current = file_identity::open_regular_no_reparse(&root.path().join(&self.name))?;
        validate_regular(&current, true)?;
        if file_identity::from_file(&current)? != self.identity {
            return Err(rejected());
        }
        root.verify()
    }
    pub(in crate::app_install) fn mark_delete_original(
        &self,
        root: &DeletionRoot,
    ) -> io::Result<()> {
        self.verify(root)?;
        mark_disposition(&self.file)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        tempfile::TempDir,
        PathBuf,
        NativeFileIdentity,
        NativeFileIdentity,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("fresh");
        let root = FreshRoot::create_new(&path).unwrap();
        let mut file = root.create_file("payload.txt").unwrap();
        use std::io::Write;
        file.file_mut().write_all(b"owned payload").unwrap();
        file.file().sync_all().unwrap();
        let rootid = root.identity().clone();
        let fileid = file.identity().clone();
        drop(file);
        drop(root);
        (temp, path, rootid, fileid)
    }
    #[test]
    fn failed_authorization_retains_original_read_root_without_mutation() {
        let (_temp, path, root_id, file_id) = fixture();
        let read = ReadRoot::open_existing(&path).unwrap();
        let failure = DeletionRoot::from_read_root_retaining(read, &file_id)
            .err()
            .expect("wrong expected root identity must fail");
        let (read, error) = *failure;
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        read.verify().unwrap();
        assert_eq!(read.identity(), &root_id);
        assert_eq!(
            std::fs::read(path.join("payload.txt")).unwrap(),
            b"owned payload"
        );
        let root = DeletionRoot::from_read_root_retaining(read, &root_id)
            .unwrap_or_else(|_| panic!("same original capability must remain usable"));
        root.verify().unwrap();
        drop(root);
        assert!(path.join("payload.txt").exists());
    }
    #[test]
    fn capabilities_drop_never_delete_and_wrong_expected_id_refuses() {
        let (_t, path, rootid, fileid) = fixture();
        let root =
            DeletionRoot::from_read_root(ReadRoot::open_existing(&path).unwrap(), &rootid).unwrap();
        let file = root.open_file("payload.txt", &fileid).unwrap();
        assert_eq!(file.name(), "payload.txt");
        assert_eq!(file.identity(), &fileid);
        drop(file);
        assert!(root.open_file("payload.txt", &rootid).is_err());
        drop(root);
        assert_eq!(
            std::fs::read(path.join("payload.txt")).unwrap(),
            b"owned payload"
        );
        assert!(path.exists());
    }
    #[test]
    fn readonly_child_share_conflict_release_then_original_disposition() {
        let (_t, path, rootid, fileid) = fixture();
        let read = ReadRoot::open_existing(&path).unwrap();
        let reader = read.open_file("payload.txt").unwrap();
        let root = DeletionRoot::from_read_root(read, &rootid).unwrap();
        let error = root
            .open_file("payload.txt", &fileid)
            .err()
            .expect("readonly share must deny DELETE");
        assert_eq!(error.raw_os_error(), Some(32));
        assert!(path.join("payload.txt").exists());
        drop(reader);
        let file = root.open_file("payload.txt", &fileid).unwrap();
        file.mark_delete_original(&root).unwrap();
        drop(file);
        assert!(!path.join("payload.txt").exists());
        root.mark_delete_original().unwrap();
        drop(root);
        assert!(!path.exists());
    }
    #[test]
    fn nonempty_root_is_not_recursively_removed() {
        let (_t, path, rootid, _) = fixture();
        let root =
            DeletionRoot::from_read_root(ReadRoot::open_existing(&path).unwrap(), &rootid).unwrap();
        assert_eq!(
            root.mark_delete_original().unwrap_err().kind(),
            io::ErrorKind::DirectoryNotEmpty
        );
        drop(root);
        assert!(path.exists());
        assert_eq!(
            std::fs::read(path.join("payload.txt")).unwrap(),
            b"owned payload"
        );
    }
}
