//! Fresh-only Windows root capability. No Drop cleanup or existing ACL mutation.
//! Retained SHARE_DELETE handles prove original identity, not a namespace lock.
//! Leaf creation is relative to original parent HANDLE and returns its original
//! create handle atomically. Ancestor path verification still observes namespace
//! before/after; failed verification preserves the newly created root.
use crate::{
    file_identity::{self, NativeFileIdentity},
    windows_private_security,
};
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Path, PathBuf, Prefix},
};
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            FILE_CREATE, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN,
            FILE_OPEN_REPARSE_POINT, FILE_RENAME_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT,
            FileRenameInformation, NtCreateFile, NtSetInformationFile,
        },
    },
    Win32::{
        Foundation::{OBJ_CASE_INSENSITIVE, RtlNtStatusToDosError, UNICODE_STRING},
        System::IO::IO_STATUS_BLOCK,
    },
};

fn rejected() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "fresh Windows install root validation failed",
    )
}
fn open_directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    validate_directory(&file)?;
    Ok(file)
}
fn validate_directory(file: &File) -> io::Result<()> {
    let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: retained live original disk HANDLE and correctly sized aligned output.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(rejected());
    }
    let mut standard = FILE_STANDARD_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileStandardInfo,
            (&mut standard as *mut FILE_STANDARD_INFO).cast(),
            std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if standard.DeletePending || !standard.Directory {
        return Err(rejected());
    }
    Ok(())
}
fn leaf_units(leaf: &std::ffi::OsStr) -> io::Result<Vec<u16>> {
    let text = leaf.to_str().ok_or_else(rejected)?;
    let wide: Vec<u16> = leaf.encode_wide().collect();
    let stem = text
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    if wide.is_empty()
        || wide.len() > 255
        || text == "."
        || text == ".."
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
        return Err(rejected());
    }
    Ok(wide)
}
fn create_leaf(
    parent: &File,
    leaf: &std::ffi::OsStr,
    descriptor: &windows_private_security::PrivateDescriptor,
) -> io::Result<File> {
    let mut exclusive_collision = false;
    create_leaf_classified(parent, leaf, descriptor, &mut exclusive_collision)
}
fn create_leaf_classified(
    parent: &File,
    leaf: &std::ffi::OsStr,
    descriptor: &windows_private_security::PrivateDescriptor,
    exclusive_collision: &mut bool,
) -> io::Result<File> {
    let mut units = leaf_units(leaf)?;
    let length =
        u16::try_from(units.len().checked_mul(2).ok_or_else(rejected)?).map_err(|_| rejected())?;
    let mut name = UNICODE_STRING {
        Length: length,
        MaximumLength: length,
        Buffer: units.as_mut_ptr(),
    };
    let attrs = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.as_raw_handle(),
        ObjectName: &mut name,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: descriptor.attributes().lpSecurityDescriptor.cast(),
        ..Default::default()
    };
    let mut status = IO_STATUS_BLOCK::default();
    let mut raw = std::ptr::null_mut();
    // SAFETY: borrowed original parent, bounded single-component UTF16, live
    // descriptor and correctly sized binding types. FILE_CREATE never opens existing.
    let result = unsafe {
        NtCreateFile(
            &mut raw,
            FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE | DELETE,
            &attrs,
            &mut status,
            std::ptr::null(),
            FILE_ATTRIBUTE_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_CREATE,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            std::ptr::null(),
            0,
        )
    };
    if result < 0 {
        // Only the original FILE_CREATE operation's exact collision status proves refusal.
        // Never infer this from a later io::ErrorKind::AlreadyExists.
        *exclusive_collision = result == 0xC0000035_u32 as i32;
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(rejected());
    }
    let file = unsafe { File::from_raw_handle(raw) };
    validate_directory(&file)?;
    Ok(file)
}
fn validate_regular(file: &File, single_link: bool) -> io::Result<()> {
    let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut standard = FILE_STANDARD_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileStandardInfo,
            (&mut standard as *mut FILE_STANDARD_INFO).cast(),
            std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
        || standard.Directory
        || standard.DeletePending
        || (single_link && standard.NumberOfLinks != 1)
    {
        return Err(rejected());
    }
    Ok(())
}
fn mark_disposition(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // Original DELETE capability only; no path reopen, POSIX flags, or share bypass.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
pub(super) struct CreatedFile {
    file: File,
    name: String,
    identity: NativeFileIdentity,
    path: PathBuf,
}
/// Failure phase is ownership information, not permission to roll back names.
#[derive(Debug)]
pub(super) enum FreshNamePublishError {
    BeforePublish(io::Error),
    RenameUnconfirmed(io::Error),
    VerificationUnconfirmed(io::Error),
}

impl CreatedFile {
    pub(super) fn file(&self) -> &File {
        &self.file
    }
    pub(super) fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
    pub(super) fn name(&self) -> &str {
        &self.name
    }
    pub(super) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
    /// Publish through original handles without replacing an existing destination.
    /// An error after NT success preserves the renamed file; callers must retain
    /// this capability and treat namespace completion as unconfirmed, not retry
    /// through an absolute path or delete either name.
    pub(super) fn publish_fresh_name(
        &mut self,
        root: &FreshRoot,
        new_leaf: &str,
    ) -> Result<(), FreshNamePublishError> {
        let units = leaf_units(std::ffi::OsStr::new(new_leaf))
            .map_err(FreshNamePublishError::BeforePublish)?;
        if new_leaf.eq_ignore_ascii_case(&self.name) {
            return Err(FreshNamePublishError::BeforePublish(rejected()));
        }
        self.verify(root)
            .map_err(FreshNamePublishError::BeforePublish)?;
        self.file
            .sync_all()
            .map_err(FreshNamePublishError::BeforePublish)?;
        self.verify(root)
            .map_err(FreshNamePublishError::BeforePublish)?;
        let name_bytes = units
            .len()
            .checked_mul(2)
            .ok_or_else(rejected)
            .map_err(FreshNamePublishError::BeforePublish)?;
        let offset = std::mem::offset_of!(FILE_RENAME_INFORMATION, FileName);
        let used = offset
            .checked_add(name_bytes)
            .ok_or_else(rejected)
            .map_err(FreshNamePublishError::BeforePublish)?;
        let used = used.max(std::mem::size_of::<FILE_RENAME_INFORMATION>());
        let length =
            u32::try_from(used).map_err(|_| FreshNamePublishError::BeforePublish(rejected()))?;
        // Integer storage initializes padding as well as the flexible UTF-16 tail.
        if std::mem::align_of::<FILE_RENAME_INFORMATION>() > std::mem::align_of::<u64>() {
            return Err(FreshNamePublishError::BeforePublish(rejected()));
        }
        let count = used.div_ceil(std::mem::size_of::<u64>());
        let mut storage = vec![0u64; count];
        let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        let mut status = IO_STATUS_BLOCK::default();
        // Same-parent rename uses the original file's parent, not a separately
        // opened target directory. Supplying RootDirectory makes older Windows
        // reopen the target and can conflict with our retained DELETE capability.
        // FILE_RENAME_INFORMATION documents NULL + a simple leaf for this case.
        // SAFETY: live original synchronous file handle, aligned initialized
        // storage, bounded validated single leaf, and non-replacing class 10.
        let result = unsafe {
            (*info).Anonymous.ReplaceIfExists = false;
            (*info).RootDirectory = std::ptr::null_mut();
            (*info).FileNameLength = u32::try_from(name_bytes)
                .map_err(|_| FreshNamePublishError::BeforePublish(rejected()))?;
            std::ptr::copy_nonoverlapping(
                units.as_ptr(),
                info.cast::<u8>().add(offset).cast::<u16>(),
                units.len(),
            );
            NtSetInformationFile(
                self.file.as_raw_handle(),
                &mut status,
                info.cast(),
                length,
                FileRenameInformation,
            )
        };
        if result != 0 {
            return Err(FreshNamePublishError::RenameUnconfirmed(io::Error::other(
                format!("original-handle publication unconfirmed: NTSTATUS {result:#010x}"),
            )));
        }
        // Commit capability metadata before verification: even if verification
        // fails, it must not keep a stale pre-publication name.
        self.name = new_leaf.to_owned();
        self.path = root.path.join(new_leaf);
        self.verify(root)
            .map_err(FreshNamePublishError::VerificationUnconfirmed)
    }
    pub(super) fn mark_delete_original(&self, root: &FreshRoot) -> io::Result<()> {
        self.verify(root)?;
        mark_disposition(&self.file)
    }
    pub(super) fn verify(&self, root: &FreshRoot) -> io::Result<()> {
        root.verify()?;
        validate_regular(&self.file, true)?;
        windows_private_security::verify_acl(&self.file, &root.sid)?;
        if self.path != root.path.join(&self.name)
            || file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != root.final_path.join(&self.name)
        {
            return Err(rejected());
        }
        let current = file_identity::open_regular_no_reparse(&self.path)?;
        validate_regular(&current, true)?;
        if file_identity::from_file(&current)? != self.identity {
            return Err(rejected());
        }
        root.verify()
    }
}
pub(super) fn pin_source(original: &File) -> io::Result<File> {
    validate_regular(original, false)?;
    let identity = file_identity::from_file(original)?;
    // SAFETY: original borrowed live disk HANDLE; ReOpenFile creates a new
    // read-only capability/cursor without reopening any caller-supplied path.
    let raw = unsafe {
        ReOpenFile(
            original.as_raw_handle(),
            GENERIC_READ,
            FILE_SHARE_READ,
            FILE_FLAG_OPEN_REPARSE_POINT,
        )
    };
    if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_handle(raw) };
    validate_regular(&file, false)?;
    if file_identity::from_file(&file)? != identity {
        return Err(rejected());
    }
    Ok(file)
}
struct Ancestor {
    path: PathBuf,
    file: File,
    identity: NativeFileIdentity,
    final_path: PathBuf,
}
impl Ancestor {
    fn verify(&self) -> io::Result<()> {
        validate_directory(&self.file)?;
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != self.final_path
        {
            return Err(rejected());
        }
        let current = open_directory(&self.path)?;
        if file_identity::from_file(&current)? != self.identity
            || file_identity::final_path(&current)? != self.final_path
        {
            return Err(rejected());
        }
        Ok(())
    }
}
pub struct FreshRoot {
    path: PathBuf,
    file: File,
    identity: NativeFileIdentity,
    final_path: PathBuf,
    sid: String,
    ancestors: Vec<Ancestor>,
}
pub(super) struct FreshRootCreateFailure {
    pub(super) error: io::Error,
    pub(super) exclusive_collision: bool,
}
impl FreshRoot {
    pub fn create_new(path: &Path) -> io::Result<Self> {
        Self::create_new_classified(path).map_err(|failure| failure.error)
    }
    pub(super) fn create_new_classified(path: &Path) -> Result<Self, FreshRootCreateFailure> {
        let mut exclusive_collision = false;
        let result = (|| -> io::Result<Self> {
            if !path.is_absolute()
                || path
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            {
                return Err(rejected());
            }
            if !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
            {
                return Err(rejected());
            }
            let leaf = path.file_name().ok_or_else(rejected)?;
            leaf_units(leaf)?;
            let units: Vec<u16> = path.as_os_str().encode_wide().collect();
            if units.is_empty() || units.len() > 32766 || units.contains(&0) {
                return Err(rejected());
            }
            let parent = path.parent().ok_or_else(rejected)?;
            let paths: Vec<_> = parent.ancestors().collect();
            if paths.len() > 256 {
                return Err(rejected());
            }
            let mut ancestors = Vec::with_capacity(paths.len());
            for p in paths.into_iter().rev() {
                let file = open_directory(p)?;
                let identity = file_identity::from_file(&file)?;
                let final_path = file_identity::final_path(&file)?;
                ancestors.push(Ancestor {
                    path: p.to_path_buf(),
                    file,
                    identity,
                    final_path,
                });
            }
            for ancestor in &ancestors {
                ancestor.verify()?;
            }
            let sid = windows_private_security::current_sid()?;
            let descriptor = windows_private_security::PrivateDescriptor::new(&sid)?;
            let original_parent = ancestors.last().ok_or_else(rejected)?;
            let file = create_leaf_classified(
                &original_parent.file,
                leaf,
                &descriptor,
                &mut exclusive_collision,
            )?;
            windows_private_security::verify_acl(&file, &sid)?;
            let identity = file_identity::from_file(&file)?;
            let final_path = file_identity::final_path(&file)?;
            let original_parent = ancestors.last().ok_or_else(rejected)?;
            if final_path != original_parent.final_path.join(leaf) {
                return Err(rejected());
            }
            let root = Self {
                path: path.to_path_buf(),
                file,
                identity,
                final_path,
                sid,
                ancestors,
            };
            root.verify()?;
            Ok(root)
        })();
        result.map_err(|error| FreshRootCreateFailure {
            error,
            exclusive_collision,
        })
    }
    /// Creates a new direct child from the retained original directory HANDLE.
    /// Failure never deletes a created child or adopts an existing namespace.
    pub(super) fn create_child(&self, name: &str) -> io::Result<Self> {
        let leaf = std::ffi::OsStr::new(name);
        leaf_units(leaf)?;
        self.verify()?;
        if self.ancestors.len() >= 256 {
            return Err(rejected());
        }
        let path = self.path.join(leaf);
        let units: Vec<u16> = path.as_os_str().encode_wide().collect();
        if units.len() > 32766 || units.contains(&0) {
            return Err(rejected());
        }
        let mut ancestors = Vec::with_capacity(self.ancestors.len() + 1);
        for ancestor in &self.ancestors {
            ancestors.push(Ancestor {
                path: ancestor.path.clone(),
                file: ancestor.file.try_clone()?,
                identity: ancestor.identity.clone(),
                final_path: ancestor.final_path.clone(),
            });
        }
        ancestors.push(Ancestor {
            path: self.path.clone(),
            file: self.file.try_clone()?,
            identity: self.identity.clone(),
            final_path: self.final_path.clone(),
        });
        // All clones are retained capabilities; creation uses original self.file, not a path reopen.
        for ancestor in &ancestors {
            ancestor.verify()?;
        }
        let descriptor = windows_private_security::PrivateDescriptor::new(&self.sid)?;
        let file = create_leaf(&self.file, leaf, &descriptor)?;
        let child = Self {
            path,
            identity: file_identity::from_file(&file)?,
            final_path: file_identity::final_path(&file)?,
            file,
            sid: self.sid.clone(),
            ancestors,
        };
        if child.final_path != self.final_path.join(leaf) {
            return Err(rejected());
        }
        self.verify()?;
        child.verify()?;
        self.verify()?;
        Ok(child)
    }
    pub(super) fn create_file(&self, name: &str) -> io::Result<CreatedFile> {
        let mut units = leaf_units(std::ffi::OsStr::new(name))?;
        self.verify()?;
        let descriptor = windows_private_security::PrivateDescriptor::new(&self.sid)?;
        let length = u16::try_from(units.len() * 2).map_err(|_| rejected())?;
        let mut object_name = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: units.as_mut_ptr(),
        };
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.file.as_raw_handle(),
            ObjectName: &mut object_name,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: descriptor.attributes().lpSecurityDescriptor.cast(),
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        // SAFETY: original root HANDLE, bounded single leaf and original private descriptor.
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL | SYNCHRONIZE | DELETE,
                &attrs,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ,
                FILE_CREATE,
                FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
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
        let identity = file_identity::from_file(&file)?;
        let created = CreatedFile {
            file,
            name: name.to_owned(),
            identity,
            path: self.path.join(name),
        };
        created.verify(self)?;
        Ok(created)
    }
    pub fn verify(&self) -> io::Result<()> {
        if windows_private_security::current_sid()? != self.sid {
            return Err(rejected());
        }
        for ancestor in &self.ancestors {
            ancestor.verify()?;
        }
        validate_directory(&self.file)?;
        windows_private_security::verify_acl(&self.file, &self.sid)?;
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != self.final_path
        {
            return Err(rejected());
        }
        let current = open_directory(&self.path)?;
        windows_private_security::verify_acl(&current, &self.sid)?;
        if file_identity::from_file(&current)? != self.identity
            || file_identity::final_path(&current)? != self.final_path
        {
            return Err(rejected());
        }
        for ancestor in &self.ancestors {
            ancestor.verify()?;
        }
        Ok(())
    }
    pub(super) fn mark_delete_original(&self) -> io::Result<()> {
        self.verify()?;
        // Any entry makes removal fail closed. Never recurse or adopt extra files.
        if let Some(entry) = std::fs::read_dir(&self.path)?.next() {
            entry?;
            return Err(io::Error::new(
                io::ErrorKind::DirectoryNotEmpty,
                "fresh root is not empty",
            ));
        }
        self.verify()?;
        mark_disposition(&self.file)
    }
    pub fn file(&self) -> &File {
        &self.file
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
}

/// Read-only existing root pin. Ancestors permit share-delete, so identity/path
/// verification observes before/after rather than promising a namespace lock.
/// Expected entry allowlists belong to the reopened bundle coordinator.
// Specific rights only: never duplicate write/delete authority into a readonly observer.
fn duplicate_observer(file: &File, access: u32) -> io::Result<File> {
    use windows_sys::Win32::{Foundation::DuplicateHandle, System::Threading::GetCurrentProcess};
    let mut raw = std::ptr::null_mut();
    // SAFETY: borrowed original HANDLE, original current process, one newly owned noninheritable result.
    if unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            file.as_raw_handle(),
            GetCurrentProcess(),
            &mut raw,
            access,
            0,
            0,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() || raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(rejected());
    }
    // SAFETY: successful DuplicateHandle creates this one owned HANDLE, transferred exactly once.
    Ok(unsafe { File::from_raw_handle(raw) })
}
pub(super) struct ReadRoot {
    root: FreshRoot,
}
/// Read-only preflight for a fresh destination. No create/adopt/delete authority.
pub(super) struct FreshDestination {
    ancestors: Vec<Ancestor>,
    parent_pin: File,
    parent_identity: NativeFileIdentity,
    parent_final: PathBuf,
    final_path: PathBuf,
}
impl FreshDestination {
    pub(super) fn observe(target: &Path) -> io::Result<Self> {
        let mut count = 0;
        let bounded = target.as_os_str().encode_wide().take(32768).all(|unit| {
            count += 1;
            unit != 0 && count <= 32766
        });
        if !bounded
            || !target.is_absolute()
            || target
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || !matches!(target.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(rejected());
        }
        let leaf = target.file_name().ok_or_else(rejected)?;
        leaf_units(leaf)?;
        let parent = target.parent().ok_or_else(rejected)?;
        let paths: Vec<_> = parent.ancestors().collect();
        if paths.len() > 256 {
            return Err(rejected());
        }
        let mut ancestors = Vec::with_capacity(paths.len());
        for path in paths.into_iter().rev() {
            let file = open_directory(path)?;
            ancestors.push(Ancestor {
                path: path.to_owned(),
                identity: file_identity::from_file(&file)?,
                final_path: file_identity::final_path(&file)?,
                file,
            });
        }
        // Pin the immediate parent without DELETE sharing. Ancestor observations
        // remain sequential; they do not promise an atomic namespace snapshot.
        let parent_pin = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | READ_CONTROL)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(parent)?;
        validate_directory(&parent_pin)?;
        let parent_identity = file_identity::from_file(&parent_pin)?;
        let parent_final = file_identity::final_path(&parent_pin)?;
        let result = Self {
            ancestors,
            parent_pin,
            parent_identity,
            final_path: parent_final.join(leaf),
            parent_final,
        };
        result.verify()?;
        match std::fs::symlink_metadata(target) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "active destination exists",
                ));
            }
            Err(error) => return Err(error),
        }
        result.verify()?;
        Ok(result)
    }
    pub(super) fn verify(&self) -> io::Result<()> {
        for ancestor in &self.ancestors {
            ancestor.verify()?;
        }
        validate_directory(&self.parent_pin)?;
        if file_identity::from_file(&self.parent_pin)? != self.parent_identity
            || file_identity::final_path(&self.parent_pin)? != self.parent_final
            || self.ancestors.last().ok_or_else(rejected)?.identity != self.parent_identity
            || self.ancestors.last().ok_or_else(rejected)?.final_path != self.parent_final
        {
            return Err(rejected());
        }
        Ok(())
    }
    pub(super) fn final_path(&self) -> &Path {
        &self.final_path
    }
}
pub(super) struct ReadFile {
    file: File,
    name: String,
    identity: NativeFileIdentity,
}
impl ReadRoot {
    /// Observes original fresh capabilities; does not establish immutable writer trust.
    pub(super) fn observe_fresh(fresh: &FreshRoot) -> io::Result<Self> {
        fresh.verify()?;
        let file = duplicate_observer(
            &fresh.file,
            FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE,
        )?;
        let mut ancestors = Vec::with_capacity(fresh.ancestors.len());
        for original in &fresh.ancestors {
            original.verify()?;
            ancestors.push(Ancestor {
                path: original.path.clone(),
                file: duplicate_observer(
                    &original.file,
                    FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE,
                )?,
                identity: original.identity.clone(),
                final_path: original.final_path.clone(),
            });
        }
        let result = Self {
            root: FreshRoot {
                path: fresh.path.clone(),
                file,
                identity: fresh.identity.clone(),
                final_path: fresh.final_path.clone(),
                sid: fresh.sid.clone(),
                ancestors,
            },
        };
        result.verify()?;
        if result.identity() != fresh.identity()
            || result.original_final_path()? != fresh.final_path
        {
            return Err(rejected());
        }
        fresh.verify()?;
        Ok(result)
    }

    pub(super) fn original_final_path(&self) -> io::Result<PathBuf> {
        self.verify()?;
        let observed = file_identity::final_path(&self.root.file)?;
        self.verify()?;
        Ok(observed)
    }
    pub(super) fn open_existing(path: &Path) -> io::Result<Self> {
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(rejected());
        }
        let leaf = path.file_name().ok_or_else(rejected)?;
        leaf_units(leaf)?;
        let units: Vec<u16> = path.as_os_str().encode_wide().collect();
        if units.len() > 32766 || units.contains(&0) {
            return Err(rejected());
        }
        let paths: Vec<_> = path.parent().ok_or_else(rejected)?.ancestors().collect();
        if paths.len() > 256 {
            return Err(rejected());
        }
        let mut ancestors = Vec::with_capacity(paths.len());
        for path in paths.into_iter().rev() {
            let file = open_directory(path)?;
            ancestors.push(Ancestor {
                path: path.to_owned(),
                identity: file_identity::from_file(&file)?,
                final_path: file_identity::final_path(&file)?,
                file,
            });
        }
        for ancestor in &ancestors {
            ancestor.verify()?;
        }
        let file = open_directory(path)?;
        let sid = windows_private_security::current_sid()?;
        windows_private_security::verify_acl(&file, &sid)?;
        let final_path = file_identity::final_path(&file)?;
        if final_path != ancestors.last().ok_or_else(rejected)?.final_path.join(leaf) {
            return Err(rejected());
        }
        let root = FreshRoot {
            path: path.to_owned(),
            identity: file_identity::from_file(&file)?,
            file,
            final_path,
            sid,
            ancestors,
        };
        let result = Self { root };
        result.verify()?;
        Ok(result)
    }
    pub(super) fn verify(&self) -> io::Result<()> {
        self.root.verify()
    }
    pub(super) fn path(&self) -> &Path {
        &self.root.path
    }
    pub(super) fn identity(&self) -> &NativeFileIdentity {
        &self.root.identity
    }
    pub(super) fn open_file(&self, name: &str) -> io::Result<ReadFile> {
        self.open_file_with_share(name, FILE_SHARE_READ, None)
    }
    pub(super) fn open_file_for_fresh_observation(
        &self,
        name: &str,
        expected: &NativeFileIdentity,
    ) -> io::Result<ReadFile> {
        self.open_file_with_share(
            name,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            Some(expected),
        )
    }
    fn open_file_with_share(
        &self,
        name: &str,
        share: u32,
        expected: Option<&NativeFileIdentity>,
    ) -> io::Result<ReadFile> {
        let mut units = leaf_units(std::ffi::OsStr::new(name))?;
        self.verify()?;
        let length = u16::try_from(units.len() * 2).map_err(|_| rejected())?;
        let mut object_name = UNICODE_STRING {
            Length: length,
            MaximumLength: length,
            Buffer: units.as_mut_ptr(),
        };
        let attrs = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.root.file.as_raw_handle(),
            ObjectName: &mut object_name,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        let mut status = IO_STATUS_BLOCK::default();
        let mut raw = std::ptr::null_mut();
        // Original directory-relative open: never creates or adopts a path-open substitute.
        let result = unsafe {
            NtCreateFile(
                &mut raw,
                GENERIC_READ | READ_CONTROL | SYNCHRONIZE,
                &attrs,
                &mut status,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                share,
                FILE_OPEN,
                FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT,
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
        let opened = ReadFile {
            identity: file_identity::from_file(&file)?,
            file,
            name: name.to_owned(),
        };
        if expected.is_some_and(|identity| *identity != opened.identity) {
            return Err(rejected());
        }
        opened.verify(self)?;
        Ok(opened)
    }
}
impl ReadFile {
    pub(super) fn file(&self) -> &File {
        &self.file
    }
    pub(super) fn identity(&self) -> &NativeFileIdentity {
        &self.identity
    }
    pub(super) fn verify(&self, root: &ReadRoot) -> io::Result<()> {
        root.verify()?;
        validate_regular(&self.file, true)?;
        windows_private_security::verify_acl(&self.file, &root.root.sid)?;
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != root.root.final_path.join(&self.name)
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
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disposition_needs_original_delete_access_and_drops_do_not_delete() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        let fresh = FreshRoot::create_new(&path).unwrap();
        let child = fresh.create_file("payload.txt").unwrap();
        let child_path = path.join("payload.txt");
        drop(child);
        assert!(child_path.exists());
        let no_delete = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&child_path)
            .unwrap();
        assert_eq!(
            mark_disposition(&no_delete).unwrap_err().raw_os_error(),
            Some(5)
        );
        drop(no_delete);
        drop(fresh);
        assert!(path.exists() && child_path.exists());
    }
    #[test]
    fn original_child_and_empty_root_disposition_preserve_unknown_entries() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        let fresh = FreshRoot::create_new(&path).unwrap();
        let child = fresh.create_file("payload.txt").unwrap();
        child.mark_delete_original(&fresh).unwrap();
        let mut standard = FILE_STANDARD_INFO::default();
        assert_ne!(
            unsafe {
                GetFileInformationByHandleEx(
                    child.file().as_raw_handle(),
                    FileStandardInfo,
                    (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                    std::mem::size_of::<FILE_STANDARD_INFO>() as u32,
                )
            },
            0
        );
        assert!(standard.DeletePending);
        drop(child);
        assert!(!path.join("payload.txt").exists());
        std::fs::write(path.join("foreign"), b"preserve").unwrap();
        assert_eq!(
            fresh.mark_delete_original().unwrap_err().kind(),
            io::ErrorKind::DirectoryNotEmpty
        );
        assert_eq!(std::fs::read(path.join("foreign")).unwrap(), b"preserve");
        // Explicit fixture cleanup of its owned sentinel, never production recursion.
        std::fs::remove_file(path.join("foreign")).unwrap();
        fresh.mark_delete_original().unwrap();
        drop(fresh);
        assert!(!path.exists());
    }
    #[test]
    fn reopened_private_file_reads_and_busy_writer_is_rejected() {
        use std::io::{Read, Write};
        let temp = tempfile::tempdir().unwrap();
        let fresh = FreshRoot::create_new(&temp.path().join("prepared")).unwrap();
        let mut child = fresh.create_file("payload.txt").unwrap();
        child.file_mut().write_all(b"preserve").unwrap();
        child.file().sync_all().unwrap();
        let root = ReadRoot::open_existing(fresh.path()).unwrap();
        assert_eq!(root.identity(), fresh.identity());
        assert_eq!(
            root.open_file("payload.txt").err().unwrap().raw_os_error(),
            Some(32)
        );
        drop(child);
        let opened = root.open_file("payload.txt").unwrap();
        let mut bytes = Vec::new();
        opened.file().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"preserve");
        assert_eq!(
            opened.identity(),
            &file_identity::from_file(opened.file()).unwrap()
        );
        opened.verify(&root).unwrap();
        assert!(
            OpenOptions::new()
                .write(true)
                .open(root.path().join("payload.txt"))
                .is_err()
        );
        // A read root has no expected namespace allowlist and does not delete extras.
        std::fs::write(root.path().join("foreign"), b"foreign").unwrap();
        assert_eq!(
            std::fs::read(root.path().join("foreign")).unwrap(),
            b"foreign"
        );
    }
    #[test]
    fn reopened_root_rejects_replaced_path() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("prepared");
        let fresh = FreshRoot::create_new(&path).unwrap();
        let root = ReadRoot::open_existing(&path).unwrap();
        std::fs::rename(&path, temp.path().join("original")).unwrap();
        let substitute = FreshRoot::create_new(&path).unwrap();
        assert!(root.verify().is_err());
        assert_ne!(root.identity(), substitute.identity());
        fresh.verify().unwrap_err();
    }
    #[test]
    fn child_original_write_sync_existing_and_replace_guards() {
        use std::io::{Read, Seek, SeekFrom, Write};
        let temp = tempfile::tempdir().unwrap();
        let root = FreshRoot::create_new(&temp.path().join("install")).unwrap();
        let mut child = root.create_file("payload.txt").unwrap();
        child.file_mut().write_all(b"original").unwrap();
        child.file().sync_all().unwrap();
        child.file_mut().seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        child.file_mut().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"original");
        assert_eq!(child.name(), "payload.txt");
        assert_eq!(
            child.identity(),
            &file_identity::from_file(child.file()).unwrap()
        );
        child.verify(&root).unwrap();
        assert!(root.create_file("payload.txt").is_err());
        let error = std::fs::rename(
            root.path().join("payload.txt"),
            root.path().join("moved.txt"),
        )
        .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(32));
        child.verify(&root).unwrap();
    }
    #[test]
    fn original_source_reopen_has_independent_cursor_and_blocks_write() {
        use std::io::{Read, Seek, SeekFrom};
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source");
        std::fs::write(&path, b"abcdef").unwrap();
        let mut original = File::open(&path).unwrap();
        original.seek(SeekFrom::Start(3)).unwrap();
        let mut pinned = pin_source(&original).unwrap();
        let mut one = [0];
        pinned.read_exact(&mut one).unwrap();
        assert_eq!(one, [b'a']);
        assert_eq!(original.stream_position().unwrap(), 3);
        assert!(OpenOptions::new().write(true).open(&path).is_err());
    }
    #[test]
    fn original_create_handle_never_adopts_substitute() {
        let temp = tempfile::tempdir().unwrap();
        let parent = open_directory(temp.path()).unwrap();
        let sid = windows_private_security::current_sid().unwrap();
        let descriptor = windows_private_security::PrivateDescriptor::new(&sid).unwrap();
        let original = create_leaf(&parent, std::ffi::OsStr::new("fresh"), &descriptor).unwrap();
        let id = file_identity::from_file(&original).unwrap();
        let path = temp.path().join("fresh");
        std::fs::rename(&path, temp.path().join("moved")).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert_ne!(
            file_identity::from_file(&open_directory(&path).unwrap()).unwrap(),
            id
        );
        assert_eq!(file_identity::from_file(&original).unwrap(), id);
    }
    #[test]
    fn original_handle_publish_preserves_identity_and_denies_foreign_mutation() {
        use std::io::Write;
        let temp = tempfile::tempdir().unwrap();
        let root = FreshRoot::create_new(&temp.path().join("publish")).unwrap();
        let mut stage = root.create_file("stage").unwrap();
        stage.file_mut().write_all(b"original bytes").unwrap();
        let identity = stage.identity().clone();
        stage.publish_fresh_name(&root, "一").unwrap();
        stage.verify(&root).unwrap();
        assert_eq!(stage.identity(), &identity);
        stage.publish_fresh_name(&root, "한글-final").unwrap();
        assert!(!root.path().join("一").exists());
        assert!(!root.path().join("stage").exists());
        assert_eq!(stage.name(), "한글-final");
        assert_eq!(stage.identity(), &identity);
        assert_eq!(
            std::fs::read(root.path().join(stage.name())).unwrap(),
            b"original bytes"
        );
        stage.verify(&root).unwrap();
        assert!(
            OpenOptions::new()
                .write(true)
                .open(root.path().join(stage.name()))
                .is_err()
        );
        assert!(
            std::fs::rename(
                root.path().join(stage.name()),
                root.path().join("foreign-move")
            )
            .is_err()
        );
        stage.verify(&root).unwrap();
    }
    #[test]
    fn publication_collision_and_invalid_names_preserve_original_caps() {
        use std::io::Write;
        let temp = tempfile::tempdir().unwrap();
        let root = FreshRoot::create_new(&temp.path().join("collision")).unwrap();
        let mut stage = root.create_file("stage").unwrap();
        stage.file_mut().write_all(b"stage bytes").unwrap();
        let mut existing = root.create_file("final").unwrap();
        existing.file_mut().write_all(b"foreign bytes").unwrap();
        existing.file().sync_all().unwrap();
        let stage_id = stage.identity().clone();
        let existing_id = existing.identity().clone();
        assert!(matches!(
            stage.publish_fresh_name(&root, "final"),
            Err(FreshNamePublishError::RenameUnconfirmed(_))
        ));
        for leaf in ["..", "CON.txt", "a:b", "a/b", "a\\b", "trail.", "stage"] {
            assert!(matches!(
                stage.publish_fresh_name(&root, leaf),
                Err(FreshNamePublishError::BeforePublish(_))
            ));
            assert_eq!(stage.name(), "stage");
            stage.verify(&root).unwrap();
            existing.verify(&root).unwrap();
        }
        assert_eq!(stage.identity(), &stage_id);
        assert_eq!(existing.identity(), &existing_id);
        assert_eq!(
            std::fs::read(root.path().join("stage")).unwrap(),
            b"stage bytes"
        );
        assert_eq!(
            std::fs::read(root.path().join("final")).unwrap(),
            b"foreign bytes"
        );
    }
    #[test]
    fn invalid_leaf_names_rejected() {
        for leaf in [
            ".", "..", "CON.txt", "LPT¹", "trail.", "trail ", "a:b", "a/b", "a\\b",
        ] {
            assert!(leaf_units(std::ffi::OsStr::new(leaf)).is_err());
        }
        assert!(leaf_units(std::ffi::OsStr::new("한글 설치")).is_ok());
    }
    #[test]
    fn fresh_root_retains_identity_and_private_acl() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("한글-install");
        let root = FreshRoot::create_new(&path).unwrap();
        root.verify().unwrap();
        assert_eq!(root.path(), path);
        assert_eq!(
            &file_identity::from_file(root.file()).unwrap(),
            root.identity()
        );
    }
    #[test]
    fn existing_root_is_never_adopted_or_changed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("existing");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("sentinel"), b"preserve").unwrap();
        assert!(FreshRoot::create_new(&path).is_err());
        assert_eq!(std::fs::read(path.join("sentinel")).unwrap(), b"preserve");
    }
    #[test]
    fn replaced_path_rejects_original_root() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        let moved = temp.path().join("moved");
        let root = FreshRoot::create_new(&path).unwrap();
        std::fs::rename(&path, &moved).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(root.verify().is_err());
        assert!(moved.is_dir() && path.is_dir());
    }
    #[test]
    fn create_child_uses_original_parent_and_never_adopts_existing() {
        let temp = tempfile::tempdir().unwrap();
        let outer = FreshRoot::create_new(&temp.path().join("outer")).unwrap();
        let child = outer.create_child("registered").unwrap();
        child.verify().unwrap();
        outer.verify().unwrap();
        assert_ne!(child.identity(), outer.identity());
        let sentinel = child.create_file("sentinel.txt").unwrap();
        assert!(outer.create_child("registered").is_err());
        assert!(outer.create_child("../outside").is_err());
        sentinel.verify(&child).unwrap();
        let child_path = child.path().to_owned();
        drop(sentinel);
        drop(child);
        assert!(child_path.join("sentinel.txt").exists());
    }

    #[test]
    fn readonly_fresh_observer_matches_original_and_rejects_wrong_identity() {
        use std::io::{Read, Write};
        let temp = tempfile::tempdir().unwrap();
        let root = FreshRoot::create_new(&temp.path().join("fresh")).unwrap();
        let mut original = root.create_file("payload.txt").unwrap();
        original.file_mut().write_all(b"original").unwrap();
        original.file().sync_all().unwrap();
        let other = root.create_file("other.txt").unwrap();
        let observer = ReadRoot::observe_fresh(&root).unwrap();
        assert_eq!(observer.identity(), root.identity());
        let read = observer
            .open_file_for_fresh_observation("payload.txt", original.identity())
            .unwrap();
        let mut text = String::new();
        (&read.file).read_to_string(&mut text).unwrap();
        assert_eq!(text, "original");
        original.verify(&root).unwrap();
        root.verify().unwrap();
        assert!(
            observer
                .open_file_for_fresh_observation("payload.txt", other.identity())
                .is_err()
        );
        drop(read);
        drop(observer);
        assert!(root.path().join("payload.txt").exists());
    }
    #[test]
    fn fresh_observer_refuses_delete_pending_original_without_deleting_unknown() {
        let temp = tempfile::tempdir().unwrap();
        let root = FreshRoot::create_new(&temp.path().join("fresh")).unwrap();
        let original = root.create_file("payload.txt").unwrap();
        let observer = ReadRoot::observe_fresh(&root).unwrap();
        let read = observer
            .open_file_for_fresh_observation("payload.txt", original.identity())
            .unwrap();
        original.mark_delete_original(&root).unwrap();
        assert!(read.verify(&observer).is_err());
        assert!(
            observer
                .open_file_for_fresh_observation("payload.txt", original.identity())
                .is_err()
        );
        root.verify().unwrap();
    }

    #[test]
    fn fresh_observer_rejects_replaced_empty_root_preserving_original_capabilities() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("fresh");
        let original = FreshRoot::create_new(&path).unwrap();
        // An open payload denies share-delete, which prevents this rename on Windows.
        // Exercise the replaceable empty directory while retaining its original root HANDLE.
        let id = original.identity().clone();
        std::fs::rename(&path, temp.path().join("retained-original")).unwrap();
        let replacement = FreshRoot::create_new(&path).unwrap();
        let foreign = replacement.create_file("foreign.txt").unwrap();
        assert!(ReadRoot::observe_fresh(&original).is_err());
        assert_eq!(file_identity::from_file(&original.file).unwrap(), id);
        foreign.verify(&replacement).unwrap();
        assert!(replacement.path().join("foreign.txt").exists());
        assert!(temp.path().join("retained-original").is_dir());
        assert_eq!(
            std::fs::read_dir(temp.path().join("retained-original"))
                .unwrap()
                .count(),
            0
        );
    }
    // Append inside windows_caps.rs existing cfg(test) module; native Windows tests, unexecuted.
    #[test]
    fn classified_exclusive_collision_preserves_original_directory_identity() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("original");
        let original = FreshRoot::create_new(&target).unwrap();
        let before = original.identity().clone();
        let failure = match FreshRoot::create_new_classified(&target) {
            Ok(_) => panic!("exclusive creation adopted existing root"),
            Err(failure) => failure,
        };
        assert_eq!(failure.error.kind(), io::ErrorKind::AlreadyExists);
        assert!(failure.exclusive_collision);
        original.verify().unwrap();
        assert_eq!(original.identity(), &before);
    }
    #[test]
    fn classified_missing_parent_is_not_exclusive_collision() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("absent-parent").join("leaf");
        let failure = match FreshRoot::create_new_classified(&target) {
            Ok(_) => panic!("missing parent unexpectedly created"),
            Err(failure) => failure,
        };
        assert!(!failure.exclusive_collision);
        assert!(!temp.path().join("absent-parent").exists());
    }
}

mod cold;
pub(super) use cold::{DeletionFile, DeletionRoot};
