// The Windows capability surface is deliberately unconstructible on other platforms.
// Methods/fields stay available for exact caller typing while factory returns Unsupported.
#![cfg_attr(not(windows), allow(dead_code))]
//! Create-only relative file capability for an EXISTING user-selected parent.
//! No FreshRoot cast, Deserialize/from-existing-file constructor, or Drop cleanup.
//! NT publication ambiguity returns facts only; no file/delete capability survives.
use crate::file_identity::NativeFileIdentity;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CleanupOutcome {
    RemovedOriginal,
    RetainedOriginal,
    Unconfirmed,
}
#[derive(Debug)]
pub(crate) enum PublishError {
    BeforePublish {
        created: CreatedRelativeFile,
        error: io::Error,
    },
    Unconfirmed {
        source_path: PathBuf,
        target_path: PathBuf,
        identity: NativeFileIdentity,
        error: io::Error,
    },
}
#[derive(Debug)]
struct ParentPin {
    file: File,
    path: PathBuf,
    identity: NativeFileIdentity,
}
#[derive(Debug)]
pub(crate) struct CreatedRelativeFile {
    file: File,
    parent: Box<ParentPin>,
    leaf: String,
    identity: NativeFileIdentity,
    sid: String,
}
#[derive(Debug)]
pub(crate) struct PublishedRelativeFile {
    reader: File,
    parent: Box<ParentPin>,
    leaf: String,
    path: PathBuf,
    identity: NativeFileIdentity,
    sid: String,
}
impl CreatedRelativeFile {
    pub(crate) fn create(parent: &File, stage_leaf: &str) -> io::Result<Self> {
        #[cfg(windows)]
        {
            windows::create(parent, stage_leaf)
        }
        #[cfg(not(windows))]
        {
            let _ = (parent, stage_leaf);
            Err(unsupported())
        }
    }
    pub(crate) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
    pub(crate) fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
    pub(crate) fn verify_current(&self, parent_path: &Path) -> io::Result<()> {
        #[cfg(windows)]
        {
            windows::verify(
                &self.file,
                &self.parent,
                &self.leaf,
                &self.identity,
                &self.sid,
                parent_path,
            )
        }
        #[cfg(not(windows))]
        {
            let _ = parent_path;
            Err(unsupported())
        }
    }
    pub(crate) fn cancel_cleanup(self) -> CleanupOutcome {
        #[cfg(windows)]
        {
            windows::cleanup(self)
        }
        #[cfg(not(windows))]
        {
            CleanupOutcome::RetainedOriginal
        }
    }
    pub(crate) fn publish_no_replace(
        self,
        final_leaf: &str,
    ) -> Result<PublishedRelativeFile, PublishError> {
        #[cfg(windows)]
        {
            windows::publish(self, final_leaf, || Ok(()), || Ok(()))
        }
        #[cfg(not(windows))]
        {
            let _ = final_leaf;
            Err(PublishError::BeforePublish {
                created: self,
                error: unsupported(),
            })
        }
    }
}
impl PublishedRelativeFile {
    pub(crate) fn reader_mut(&mut self) -> &mut File {
        &mut self.reader
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
    pub(crate) fn verify_current(&self, parent_path: &Path) -> io::Result<()> {
        #[cfg(windows)]
        {
            windows::verify(
                &self.reader,
                &self.parent,
                &self.leaf,
                &self.identity,
                &self.sid,
                parent_path,
            )
        }
        #[cfg(not(windows))]
        {
            let _ = parent_path;
            Err(unsupported())
        }
    }
}
#[cfg(not(windows))]
fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "created relative file publication is Windows-only",
    )
}
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::{file_identity, windows_private_security};
    use std::{
        fs::OpenOptions,
        os::windows::{
            ffi::OsStrExt,
            fs::OpenOptionsExt,
            io::{AsRawHandle, FromRawHandle},
        },
        path::{Component, Prefix},
    };
    use windows_sys::{
        Wdk::{
            Foundation::OBJECT_ATTRIBUTES,
            Storage::FileSystem::{
                FILE_CREATE, FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_REPARSE_POINT,
                FILE_RENAME_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileRenameInformation,
                NtCreateFile, NtSetInformationFile,
            },
        },
        Win32::{
            Foundation::{
                GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, OBJ_CASE_INSENSITIVE,
                RtlNtStatusToDosError, STATUS_OBJECT_NAME_NOT_FOUND, UNICODE_STRING,
            },
            Storage::FileSystem::*,
            System::IO::IO_STATUS_BLOCK,
        },
    };
    const MAX_BODY: u64 = 1024 * 1024 * 1024 * 1024;
    fn invalid() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "created relative file identity or namespace changed",
        )
    }
    fn nt_error(status: i32) -> io::Error {
        io::Error::from_raw_os_error(unsafe { RtlNtStatusToDosError(status) } as i32)
    }
    fn local(path: &Path) -> io::Result<()> {
        if !path.is_absolute()
            || path.components().count() > 256
            || path.as_os_str().encode_wide().count() > 32766
            || path
                .components()
                .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
            || !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(invalid());
        }
        if path
            .components()
            .any(|c| matches!(c,Component::Normal(s) if s.to_string_lossy().contains(':')))
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn leaf(text: &str) -> io::Result<Vec<u16>> {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let stem = text
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if wide.is_empty()
            || wide.len() > 255
            || matches!(text, "." | "..")
            || text.ends_with(['.', ' '])
            || text.chars().any(|c| {
                c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
            })
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
            || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(
                    &stem[3..],
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                ))
        {
            return Err(invalid());
        }
        Ok(wide)
    }
    fn attributes(file: &File) -> io::Result<FILE_ATTRIBUTE_TAG_INFO> {
        if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK {
            return Err(invalid());
        }
        let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: borrowed live disk HANDLE and initialized generated SDK output.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(info)
    }
    fn standard(file: &File) -> io::Result<FILE_STANDARD_INFO> {
        let mut info = FILE_STANDARD_INFO::default();
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut info as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(info)
    }
    fn directory(file: &File) -> io::Result<()> {
        let tag = attributes(file)?;
        let s = standard(file)?;
        if tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || s.DeletePending
            || !s.Directory
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn regular(file: &File) -> io::Result<()> {
        let tag = attributes(file)?;
        let s = standard(file)?;
        if tag.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
            || s.Directory
            || s.DeletePending
            || s.NumberOfLinks != 1
            || s.EndOfFile < 0
            || s.EndOfFile as u64 > MAX_BODY
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn open_parent(path: &Path) -> io::Result<File> {
        let f = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        directory(&f)?;
        Ok(f)
    }
    fn parent_current(parent: &File, path: &Path, id: &NativeFileIdentity) -> io::Result<()> {
        local(path)?;
        directory(parent)?;
        if file_identity::from_file(parent)? != *id || file_identity::final_path(parent)? != path {
            return Err(invalid());
        }
        let observed = open_parent(path)?;
        if file_identity::from_file(&observed)? != *id
            || file_identity::final_path(&observed)? != path
        {
            return Err(invalid());
        }
        Ok(())
    }
    // Exact original-parent FILE_OPEN observation; caller classifies STATUS_NAME_NOT_FOUND only.
    fn relative_open(parent: &File, name: &str) -> Result<File, i32> {
        let mut units =
            leaf(name).map_err(|_| windows_sys::Win32::Foundation::STATUS_INVALID_PARAMETER)?;
        let len = u16::try_from(units.len() * 2)
            .map_err(|_| windows_sys::Win32::Foundation::STATUS_INVALID_PARAMETER)?;
        let mut name = UNICODE_STRING {
            Length: len,
            MaximumLength: len,
            Buffer: units.as_mut_ptr(),
        };
        let object = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent.as_raw_handle(),
            ObjectName: &mut name,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE,
                &object,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_OPEN,
                FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if result != 0 {
            return Err(result);
        }
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(windows_sys::Win32::Foundation::STATUS_INVALID_HANDLE);
        }
        // SAFETY: exact successful FILE_OPEN returned one new owned HANDLE.
        Ok(unsafe { File::from_raw_handle(raw) })
    }
    pub(super) fn verify(
        file: &File,
        parent: &ParentPin,
        name: &str,
        id: &NativeFileIdentity,
        sid: &str,
        expected_parent: &Path,
    ) -> io::Result<()> {
        if expected_parent != parent.path {
            return Err(invalid());
        }
        parent_current(&parent.file, &parent.path, &parent.identity)?;
        regular(file)?;
        windows_private_security::verify_acl(file, sid)?;
        if windows_private_security::current_sid()? != sid
            || file_identity::from_file(file)? != *id
            || file_identity::final_path(file)? != parent.path.join(name)
        {
            return Err(invalid());
        }
        let observed = relative_open(&parent.file, name).map_err(nt_error)?;
        regular(&observed)?;
        if file_identity::from_file(&observed)? != *id {
            return Err(invalid());
        }
        parent_current(&parent.file, &parent.path, &parent.identity)
    }
    pub(super) fn create(original_parent: &File, name: &str) -> io::Result<CreatedRelativeFile> {
        create_with(original_parent, name, |_| Ok(()))
    }
    fn create_with(
        original_parent: &File,
        name: &str,
        after_create: impl FnOnce(&File) -> io::Result<()>,
    ) -> io::Result<CreatedRelativeFile> {
        let mut units = leaf(name)?;
        directory(original_parent)?;
        let parent_path = file_identity::final_path(original_parent)?;
        local(&parent_path)?;
        let parent_identity = file_identity::from_file(original_parent)?;
        local(&parent_path.join(name))?;
        let parent = original_parent.try_clone()?;
        parent_current(&parent, &parent_path, &parent_identity)?;
        let sid = windows_private_security::current_sid()?;
        let descriptor = windows_private_security::PrivateDescriptor::new(&sid)?;
        let length = u16::try_from(units.len() * 2).map_err(|_| invalid())?;
        let mut name_native = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: units.as_mut_ptr(),
        };
        let object = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent.as_raw_handle(),
            ObjectName: &mut name_native,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: descriptor.attributes().lpSecurityDescriptor.cast(),
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        // SAFETY: retained original user directory, generated SDK types, bounded local leaf and live private SID descriptor.
        // FILE_CREATE cannot open/adopt an existing file; no DELETE_ON_CLOSE.
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL | SYNCHRONIZE | DELETE,
                &object,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ,
                FILE_CREATE,
                FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if result != 0 {
            return Err(nt_error(result));
        }
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(invalid());
        }
        // From this success onward any failed validation preserves the original created namespace.
        let file = unsafe { File::from_raw_handle(raw) };
        after_create(&file)?;
        let identity = file_identity::from_file(&file)?;
        let created = CreatedRelativeFile {
            file,
            parent: Box::new(ParentPin {
                file: parent,
                path: parent_path,
                identity: parent_identity,
            }),
            leaf: name.to_owned(),
            identity,
            sid,
        };
        created.verify_current(&created.parent.path)?;
        Ok(created)
    }
    fn ambiguous(created: CreatedRelativeFile, target: &str, error: io::Error) -> PublishError {
        PublishError::Unconfirmed {
            source_path: created.parent.path.join(&created.leaf),
            target_path: if leaf(target).is_ok() {
                created.parent.path.join(target)
            } else {
                PathBuf::new()
            },
            identity: created.identity.clone(),
            error,
        }
    }
    fn before(created: CreatedRelativeFile, target: &str, error: io::Error) -> PublishError {
        // A pre-call error returns cleanup authority ONLY if the original staged name is still confirmed.
        if created.verify_current(&created.parent.path).is_ok() {
            PublishError::BeforePublish { created, error }
        } else {
            ambiguous(created, target, error)
        }
    }
    fn rename(file: &File, units: &[u16]) -> io::Result<()> {
        let name_bytes = units.len().checked_mul(2).ok_or_else(invalid)?;
        let offset = std::mem::offset_of!(FILE_RENAME_INFORMATION, FileName);
        let used = offset
            .checked_add(name_bytes)
            .ok_or_else(invalid)?
            .max(std::mem::size_of::<FILE_RENAME_INFORMATION>());
        if std::mem::align_of::<FILE_RENAME_INFORMATION>() > std::mem::align_of::<u64>() {
            return Err(invalid());
        }
        let mut storage = vec![0u64; used.div_ceil(std::mem::size_of::<u64>())];
        let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        let mut status = IO_STATUS_BLOCK::default();
        // SAFETY: initialized aligned buffer, original DELETE-capable source HANDLE and
        // validated simple leaf only. NULL RootDirectory selects the documented original
        // same-directory form, independent of CWD, rather than target-directory-relative lookup.
        // ReplaceIfExists=false preserves every pre-existing destination.
        let result = unsafe {
            (*info).Anonymous.ReplaceIfExists = false;
            (*info).RootDirectory = std::ptr::null_mut();
            (*info).FileNameLength = u32::try_from(name_bytes).map_err(|_| invalid())?;
            std::ptr::copy_nonoverlapping(
                units.as_ptr(),
                info.cast::<u8>().add(offset).cast::<u16>(),
                units.len(),
            );
            NtSetInformationFile(
                file.as_raw_handle(),
                &mut status,
                info.cast(),
                u32::try_from(used).map_err(|_| invalid())?,
                FileRenameInformation,
            )
        };
        if result != 0 {
            return Err(nt_error(result));
        }
        Ok(())
    }
    fn reader(file: &File) -> io::Result<File> {
        use windows_sys::Win32::{
            Foundation::DuplicateHandle, System::Threading::GetCurrentProcess,
        };
        let mut raw = std::ptr::null_mut();
        // SAFETY: duplicate our original file only; reduced read/metadata rights, no WRITE/DELETE.
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                file.as_raw_handle(),
                GetCurrentProcess(),
                &mut raw,
                FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | READ_CONTROL | SYNCHRONIZE,
                0,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(invalid());
        }
        Ok(unsafe { File::from_raw_handle(raw) })
    }
    pub(super) fn publish(
        created: CreatedRelativeFile,
        final_leaf: &str,
        before_call: impl FnOnce() -> io::Result<()>,
        after_call: impl FnOnce() -> io::Result<()>,
    ) -> Result<PublishedRelativeFile, PublishError> {
        let units = match leaf(final_leaf) {
            Ok(u) => u,
            Err(e) => return Err(before(created, final_leaf, e)),
        };
        if let Err(error) = local(&created.parent.path.join(final_leaf)) {
            return Err(before(created, final_leaf, error));
        }
        if final_leaf.eq_ignore_ascii_case(&created.leaf) {
            return Err(before(created, final_leaf, invalid()));
        }
        if let Err(e) = created.verify_current(&created.parent.path) {
            return Err(ambiguous(created, final_leaf, e));
        }
        match relative_open(&created.parent.file, final_leaf) {
            Err(status) if status == STATUS_OBJECT_NAME_NOT_FOUND => (),
            Ok(_) => {
                return Err(before(
                    created,
                    final_leaf,
                    io::ErrorKind::AlreadyExists.into(),
                ));
            }
            Err(status) => return Err(before(created, final_leaf, nt_error(status))),
        }
        if let Err(e) = created
            .file
            .sync_all()
            .and_then(|()| created.verify_current(&created.parent.path))
            .and_then(|()| before_call())
            .and_then(|()| created.verify_current(&created.parent.path))
        {
            return Err(before(created, final_leaf, e));
        }
        // All pre-call checks are complete. Any return from the rename invocation loses cleanup authority.
        if let Err(e) = rename(&created.file, &units) {
            return Err(ambiguous(created, final_leaf, e));
        }
        let id = created.identity.clone();
        let source_path = created.parent.path.join(&created.leaf);
        let target_path = created.parent.path.join(final_leaf);
        let outcome = (|| {
            after_call()?;
            verify(
                &created.file,
                &created.parent,
                final_leaf,
                &created.identity,
                &created.sid,
                &created.parent.path,
            )?;
            let read = reader(&created.file)?;
            regular(&read)?;
            Ok(read)
        })();
        let reader = match outcome {
            Ok(r) => r,
            Err(error) => {
                return Err(PublishError::Unconfirmed {
                    source_path,
                    target_path,
                    identity: id,
                    error,
                });
            }
        };
        Ok(PublishedRelativeFile {
            reader,
            parent: created.parent,
            leaf: final_leaf.to_owned(),
            path: target_path,
            identity: created.identity,
            sid: created.sid,
        })
    }
    pub(super) fn cleanup(created: CreatedRelativeFile) -> CleanupOutcome {
        if created.verify_current(&created.parent.path).is_err() {
            return CleanupOutcome::Unconfirmed;
        }
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // SAFETY: ONLY original freshly-created DELETE handle after original parent/leaf/ID proof. No path delete/reopen.
        if unsafe {
            SetFileInformationByHandle(
                created.file.as_raw_handle(),
                FileDispositionInfo,
                (&info as *const FILE_DISPOSITION_INFO).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        } == 0
        {
            return if created.verify_current(&created.parent.path).is_ok() {
                CleanupOutcome::RetainedOriginal
            } else {
                CleanupOutcome::Unconfirmed
            };
        }
        let CreatedRelativeFile {
            file, parent, leaf, ..
        } = created;
        drop(file);
        if parent_current(&parent.file, &parent.path, &parent.identity).is_err() {
            return CleanupOutcome::Unconfirmed;
        }
        match relative_open(&parent.file, &leaf) {
            Err(status) if status == STATUS_OBJECT_NAME_NOT_FOUND => {
                CleanupOutcome::RemovedOriginal
            }
            _ => CleanupOutcome::Unconfirmed,
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::{Read, Seek, SeekFrom, Write};
        fn parent(path: &Path) -> File {
            open_parent(path).unwrap()
        }
        #[test]
        fn postcreate_validation_error_keeps_stage_and_attempted_path_never_grants_cleanup() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            std::fs::write(path.join("user.txt"), b"user original").unwrap();
            let result = create_with(&parent, "uncertain.partial", |file| {
                let mut own = file;
                own.write_all(b"created but not confirmed")?;
                Err(io::Error::other(
                    "injected after actual exclusive NT create",
                ))
            });
            assert!(result.is_err());
            assert_eq!(
                std::fs::read(path.join("uncertain.partial")).unwrap(),
                b"created but not confirmed"
            );
            assert_eq!(
                std::fs::read(path.join("user.txt")).unwrap(),
                b"user original"
            );
            assert!(
                CreatedRelativeFile::create(&parent, "uncertain.partial").is_err(),
                "kept stage is never adopted as a new capability"
            );
        }
        #[test]
        fn original_create_publish_same_parent_readonly_reader_and_drop_never_delete_final() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            std::fs::write(path.join("existing-user.txt"), b"preserve user").unwrap();
            let mut c = CreatedRelativeFile::create(&parent, ".owned-stage.partial").unwrap();
            let id = c.identity().clone();
            c.file_mut().write_all(b"copied bytes").unwrap();
            c.verify_current(&path).unwrap();
            let mut published = c.publish_no_replace("한글 final.mp4").unwrap();
            assert_eq!(published.identity(), &id);
            published.verify_current(&path).unwrap();
            assert!(published.reader_mut().write_all(b"forbidden").is_err());
            published.reader_mut().seek(SeekFrom::Start(0)).unwrap();
            let mut bytes = vec![];
            published.reader_mut().read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes, b"copied bytes");
            let final_path = published.path().to_owned();
            drop(published);
            assert_eq!(std::fs::read(final_path).unwrap(), b"copied bytes");
            assert_eq!(
                std::fs::read(path.join("existing-user.txt")).unwrap(),
                b"preserve user"
            );
        }
        #[test]
        fn same_directory_publish_with_retained_fresh_root_preserves_identity_and_foreign_target() {
            let tmp = tempfile::tempdir().unwrap();
            let fresh_path = tmp.path().join("retained-parent");
            let fresh = crate::app_install::FreshRoot::create_new(&fresh_path).unwrap();
            let path = std::fs::canonicalize(&fresh_path).unwrap();
            // Match the host: caller's read-data directory HANDLE and the original
            // FreshRoot list/DELETE HANDLE both remain live throughout publication.
            let parent = OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)
                .unwrap();
            assert_ne!(std::env::current_dir().unwrap(), path);
            std::fs::write(path.join("foreign.txt"), b"foreign bytes").unwrap();
            let mut created = CreatedRelativeFile::create(&parent, "stage.partial").unwrap();
            created
                .file_mut()
                .write_all(b"same directory original")
                .unwrap();
            let original = created.identity().clone();
            let mut published = created.publish_no_replace("final.txt").unwrap();
            fresh.verify().unwrap();
            published.verify_current(&path).unwrap();
            assert_eq!(published.identity(), &original);
            assert_eq!(published.path(), path.join("final.txt"));
            let mut bytes = Vec::new();
            published.reader_mut().seek(SeekFrom::Start(0)).unwrap();
            published.reader_mut().read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes, b"same directory original");
            assert!(!path.join("stage.partial").exists());
            assert_eq!(
                std::fs::read(path.join("foreign.txt")).unwrap(),
                b"foreign bytes"
            );
            let mut collision = CreatedRelativeFile::create(&parent, "collision.partial").unwrap();
            collision
                .file_mut()
                .write_all(b"collision original")
                .unwrap();
            let PublishError::BeforePublish { created, .. } =
                collision.publish_no_replace("foreign.txt").unwrap_err()
            else {
                panic!("existing target must be refused before native rename");
            };
            assert_eq!(created.cancel_cleanup(), CleanupOutcome::RemovedOriginal);
            assert_eq!(
                std::fs::read(path.join("foreign.txt")).unwrap(),
                b"foreign bytes"
            );
            fresh.verify().unwrap();
        }
        #[test]
        fn collision_before_publish_retains_cleanup_capability_only_for_original_stage() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            std::fs::write(path.join("final.mp4"), b"foreign target").unwrap();
            let mut c = CreatedRelativeFile::create(&parent, "stage.partial").unwrap();
            c.file_mut().write_all(b"new stage").unwrap();
            let PublishError::BeforePublish { created, .. } =
                c.publish_no_replace("final.mp4").unwrap_err()
            else {
                panic!("preflight never attempted native publication")
            };
            assert_eq!(created.cancel_cleanup(), CleanupOutcome::RemovedOriginal);
            assert!(!path.join("stage.partial").exists());
            assert_eq!(
                std::fs::read(path.join("final.mp4")).unwrap(),
                b"foreign target"
            );
        }
        #[test]
        fn target_race_after_preflight_is_unconfirmed_and_preserves_both_original_bytes() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            let mut c = CreatedRelativeFile::create(&parent, "stage.partial").unwrap();
            let id = c.identity().clone();
            c.file_mut().write_all(b"new stage").unwrap();
            let error = publish(
                c,
                "final.mp4",
                || std::fs::write(path.join("final.mp4"), b"raced foreign"),
                || Ok(()),
            )
            .unwrap_err();
            let PublishError::Unconfirmed { identity, .. } = error else {
                panic!("attempted NT rename returns NO cleanup capability")
            };
            assert_eq!(identity, id);
            assert_eq!(
                std::fs::read(path.join("stage.partial")).unwrap(),
                b"new stage"
            );
            assert_eq!(
                std::fs::read(path.join("final.mp4")).unwrap(),
                b"raced foreign"
            );
        }
        #[test]
        fn committed_rename_postcheck_failure_retains_final_and_exposes_no_cleanup_capability() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            let mut c = CreatedRelativeFile::create(&parent, "stage.partial").unwrap();
            let id = c.identity().clone();
            c.file_mut().write_all(b"preserve committed").unwrap();
            let error = publish(
                c,
                "final.mp4",
                || Ok(()),
                || Err(io::Error::other("injected after actual rename")),
            )
            .unwrap_err();
            let PublishError::Unconfirmed {
                source_path,
                target_path,
                identity,
                ..
            } = error
            else {
                panic!("no late cleanup")
            };
            assert_eq!(identity, id);
            assert_eq!(source_path, path.join("stage.partial"));
            assert_eq!(target_path, path.join("final.mp4"));
            assert!(!source_path.exists());
            assert_eq!(std::fs::read(target_path).unwrap(), b"preserve committed");
        }
        #[test]
        fn substituted_original_stage_refuses_verification_and_cleanup_preserving_foreign_and_moved_original()
         {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            let mut c = CreatedRelativeFile::create(&parent, "stage.partial").unwrap();
            c.file_mut().write_all(b"moved original").unwrap();
            rename(&c.file, &leaf("moved.partial").unwrap()).unwrap();
            std::fs::write(path.join("stage.partial"), b"substitute user bytes").unwrap();
            assert!(c.verify_current(&path).is_err());
            assert_eq!(c.cancel_cleanup(), CleanupOutcome::Unconfirmed);
            assert_eq!(
                std::fs::read(path.join("stage.partial")).unwrap(),
                b"substitute user bytes"
            );
            assert_eq!(
                std::fs::read(path.join("moved.partial")).unwrap(),
                b"moved original"
            );
        }
        #[test]
        fn existing_stage_invalid_leaf_and_wrong_parent_cannot_adopt_or_cleanup_user_files() {
            let tmp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(tmp.path()).unwrap();
            let parent = parent(&path);
            std::fs::write(path.join("stage.partial"), b"existing user").unwrap();
            assert!(CreatedRelativeFile::create(&parent, "stage.partial").is_err());
            for name in [
                "../escape",
                "bad:name",
                "CON",
                "NUL.txt",
                "folder/file",
                "trail.",
            ] {
                assert!(CreatedRelativeFile::create(&parent, name).is_err());
            }
            let mut c = CreatedRelativeFile::create(&parent, "new.partial").unwrap();
            c.file_mut().write_all(b"original").unwrap();
            assert!(c.verify_current(&path.join("different")).is_err());
            assert_eq!(c.cancel_cleanup(), CleanupOutcome::RemovedOriginal);
            assert_eq!(
                std::fs::read(path.join("stage.partial")).unwrap(),
                b"existing user"
            );
        }
    }
}
#[cfg(all(test, not(windows)))]
mod unsupported_tests {
    use super::*;
    #[test]
    fn non_windows_factory_refuses_without_creating_or_changing_output() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("user.txt");
        std::fs::write(&p, b"unchanged").unwrap();
        let parent = File::open(t.path()).unwrap();
        let e = CreatedRelativeFile::create(&parent, "new.partial").unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::Unsupported);
        assert!(!t.path().join("new.partial").exists());
        assert_eq!(std::fs::read(p).unwrap(), b"unchanged");
    }
}
