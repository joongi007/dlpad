//! Private Windows session copies. No startup recovery or source mutation.
// Native journal is verified independently before product recovery activation.
#[allow(dead_code)]
pub(crate) mod guardian;
#[allow(dead_code)]
mod journal;
mod reader_gate;
pub(super) mod recovery;
#[allow(dead_code)]
mod recovery_marker;
pub(crate) use reader_gate::WindowsGatedCookieReader;
#[cfg(test)]
pub(crate) mod test_fixture;
use super::format::{SensitiveBlock, SensitiveBytes, validate};
use super::{CookieImportError, MAX_COOKIE_BYTES, MAX_COOKIE_SESSIONS, SensitivePath};
use crate::file_identity::{self, NativeFileIdentity};
use std::{
    ffi::OsString,
    fmt,
    fs::{self, File},
    io::{Read, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Component, Path, PathBuf, Prefix},
    ptr,
    sync::Mutex,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Cryptography::*, *},
    Storage::FileSystem::*,
};
const MAX_UNITS: usize = 32_767;
const MAX_COMPONENTS: usize = 64;
// Win32 DRIVE_FIXED=3 and FILE_PERSISTENT_ACLS=0x8; avoid unused feature modules.
const FIXED_DRIVE: u32 = 3;
const PERSISTENT_ACLS: u32 = 8;
const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
const META_ACCESS: u32 = FILE_READ_ATTRIBUTES | READ_CONTROL;

fn changed() -> CookieImportError {
    CookieImportError::CleanupOwnershipChanged
}
fn owned(raw: HANDLE) -> Result<OwnedHandle, CookieImportError> {
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(changed());
    }
    // SAFETY: fresh valid Win32 handle transferred once to RAII ownership.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}
fn current_sid() -> Result<String, CookieImportError> {
    crate::windows_private_security::current_sid().map_err(|error| {
        if error.kind() == std::io::ErrorKind::Unsupported {
            CookieImportError::Unavailable
        } else {
            changed()
        }
    })
}
struct Descriptor(crate::windows_private_security::PrivateDescriptor);
impl Descriptor {
    fn private(sid: &str) -> Result<Self, CookieImportError> {
        crate::windows_private_security::PrivateDescriptor::new(sid)
            .map(Self)
            .map_err(|_| changed())
    }
    fn attributes(&self) -> SECURITY_ATTRIBUTES {
        self.0.attributes()
    }
}
fn acl(file: &File, sid: &str) -> Result<(), CookieImportError> {
    crate::windows_private_security::verify_acl(file, sid).map_err(|_| changed())
}
fn wide(path: &Path) -> Result<Vec<u16>, CookieImportError> {
    let mut units: Vec<u16> = path.as_os_str().encode_wide().take(MAX_UNITS + 1).collect();
    if units.len() > MAX_UNITS || units.contains(&0) {
        return Err(changed());
    }
    units.push(0);
    Ok(units)
}
fn reserved(value: &str) -> bool {
    let base = value.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || (base.len() == 4
        && (base.starts_with("COM") || base.starts_with("LPT"))
        && matches!(base.as_bytes()[3], b'1'..=b'9'))
}
fn path_parts(path: &Path) -> Result<(PathBuf, Vec<OsString>), CookieImportError> {
    wide(path)?;
    let mut components = path.components();
    let drive = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(d) | Prefix::VerbatimDisk(d) => d,
            _ => return Err(changed()),
        },
        _ => return Err(changed()),
    };
    if !drive.is_ascii_alphabetic() || components.next() != Some(Component::RootDir) {
        return Err(changed());
    }
    let dos: Vec<u16> = format!("{}:\\", char::from(drive))
        .encode_utf16()
        .chain([0])
        .collect();
    if unsafe { GetDriveTypeW(dos.as_ptr()) } != FIXED_DRIVE {
        return Err(CookieImportError::Unavailable);
    }
    let root = PathBuf::from(format!("\\\\?\\{}:\\", char::from(drive)));
    let mut names = Vec::new();
    for component in components {
        let Component::Normal(name) = component else {
            return Err(changed());
        };
        let text = name.to_str().ok_or_else(changed)?;
        if text.is_empty()
            || text.ends_with(['.', ' '])
            || text.contains([':', '\0', '/', '\\'])
            || reserved(text)
            || names.len() >= MAX_COMPONENTS
        {
            return Err(changed());
        }
        names.push(name.to_owned());
    }
    if names.is_empty() {
        return Err(changed());
    }
    Ok((root, names))
}
fn open(
    path: &Path,
    directory: bool,
    access: u32,
    share: u32,
    create: Option<&Descriptor>,
) -> Result<File, CookieImportError> {
    let path = wide(path)?;
    let attributes = create.map(Descriptor::attributes);
    let raw = unsafe {
        CreateFileW(
            path.as_ptr(),
            access,
            share,
            attributes.as_ref().map_or(ptr::null(), |a| a),
            if create.is_some() {
                CREATE_NEW
            } else {
                OPEN_EXISTING
            },
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    FILE_ATTRIBUTE_NORMAL
                },
            ptr::null_mut(),
        )
    };
    let handle = owned(raw)?;
    let file = File::from(handle);
    kind(&file, directory)?;
    Ok(file)
}
fn kind(file: &File, directory: bool) -> Result<(), CookieImportError> {
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK {
        return Err(changed());
    }
    let mut tags = FILE_ATTRIBUTE_TAG_INFO::default();
    let mut standard = FILE_STANDARD_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut tags as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&tags) as u32,
        )
    } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&standard) as u32,
            )
        } == 0
        || tags.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (tags.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || standard.Directory != directory
        || (!directory && standard.NumberOfLinks != 1)
        || standard.DeletePending
    {
        return Err(changed());
    }
    Ok(())
}
fn ntfs(root: &File) -> Result<(), CookieImportError> {
    let mut fs = [0u16; 32];
    let mut flags = 0;
    if unsafe {
        GetVolumeInformationByHandleW(
            root.as_raw_handle(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut flags,
            fs.as_mut_ptr(),
            fs.len() as u32,
        )
    } == 0
        || flags & PERSISTENT_ACLS == 0
    {
        return Err(CookieImportError::Unavailable);
    }
    let used = fs
        .iter()
        .position(|unit| *unit == 0)
        .ok_or(CookieImportError::Unavailable)?;
    if String::from_utf16(&fs[..used]).map_err(|_| CookieImportError::Unavailable)? != "NTFS" {
        return Err(CookieImportError::Unavailable);
    }
    Ok(())
}
struct Pin {
    path: SensitivePath,
    file: File,
    identity: NativeFileIdentity,
}
fn pin(path: PathBuf) -> Result<Pin, CookieImportError> {
    let file = open(
        &path,
        true,
        META_ACCESS | FILE_LIST_DIRECTORY,
        FILE_SHARE_READ,
        None,
    )?;
    let identity = file_identity::from_file(&file).map_err(|_| changed())?;
    Ok(Pin {
        path: SensitivePath(path),
        file,
        identity,
    })
}
fn chain(root: PathBuf, names: &[OsString]) -> Result<Vec<Pin>, CookieImportError> {
    let mut pins = Vec::with_capacity(names.len() + 1);
    let mut current = root;
    let first = pin(current.clone())?;
    ntfs(&first.file)?;
    pins.push(first);
    for name in names {
        current.push(name);
        pins.push(pin(current.clone())?);
    }
    Ok(pins)
}
fn check_pin(pin: &Pin) -> Result<(), CookieImportError> {
    kind(&pin.file, true)?;
    if file_identity::from_file(&pin.file).map_err(|_| changed())? != pin.identity {
        return Err(changed());
    }
    let current = open(&pin.path.0, true, META_ACCESS, SHARE_ALL, None)?;
    if file_identity::from_file(&current).map_err(|_| changed())? != pin.identity {
        return Err(changed());
    }
    Ok(())
}
fn count(path: &Path, limit: usize) -> Result<usize, CookieImportError> {
    fs::read_dir(path)
        .map_err(|_| changed())?
        .take(limit + 1)
        .try_fold(0, |n, e| e.map(|_| n + 1).map_err(|_| changed()))
}
fn create_directory(path: &Path, descriptor: &Descriptor) -> Result<bool, CookieImportError> {
    let units = wide(path)?;
    let attributes = descriptor.attributes();
    if unsafe { CreateDirectoryW(units.as_ptr(), &attributes) } != 0 {
        return Ok(true);
    }
    let code = unsafe { GetLastError() };
    if code == ERROR_ALREADY_EXISTS {
        Ok(false)
    } else {
        Err(changed())
    }
}
/// Only a newly created cache receives a private descriptor. Existing ACLs are untouched.
pub(super) fn prepare_cache(path: &Path) -> Result<(), CookieImportError> {
    let sid = current_sid()?;
    let descriptor = Descriptor::private(&sid)?;
    let (root, mut names) = path_parts(path)?;
    let leaf = names.pop().ok_or_else(changed)?;
    let pins = chain(root, &names)?;
    let parent = pins.last().ok_or_else(changed)?;
    let selected = parent.path.0.join(leaf);
    let units = wide(&selected)?;
    let attributes = descriptor.attributes();
    if unsafe { CreateDirectoryW(units.as_ptr(), &attributes) } == 0
        && unsafe { GetLastError() } != ERROR_ALREADY_EXISTS
    {
        return Err(CookieImportError::UnsafeCache);
    }
    let cache = pin(selected).map_err(|_| CookieImportError::UnsafeCache)?;
    for parent in &pins {
        check_pin(parent).map_err(|_| CookieImportError::UnsafeCache)?;
    }
    acl(&cache.file, &sid).map_err(|_| CookieImportError::UnsafeCache)
}
fn random_name() -> Result<String, CookieImportError> {
    let mut bytes = [0u8; 32];
    if unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            bytes.as_mut_ptr(),
            bytes.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } != 0
    {
        return Err(CookieImportError::StorageUnavailable);
    }
    Ok(format!(
        "cookie-session-{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ))
}
fn source_bytes(source: &Path) -> Result<SensitiveBytes, CookieImportError> {
    let (root, mut names) = path_parts(source).map_err(|_| CookieImportError::SourceUnavailable)?;
    let leaf = names.pop().ok_or(CookieImportError::SourceUnavailable)?;
    let pins = chain(root, &names).map_err(|_| CookieImportError::SourceUnavailable)?;
    let parent = pins.last().ok_or(CookieImportError::SourceUnavailable)?;
    let mut file = open(
        &parent.path.0.join(leaf),
        false,
        FILE_GENERIC_READ,
        FILE_SHARE_READ,
        None,
    )
    .map_err(|_| CookieImportError::SourceUnavailable)?;
    let before = file
        .metadata()
        .map_err(|_| CookieImportError::SourceUnavailable)?;
    let id = file_identity::from_file(&file).map_err(|_| CookieImportError::SourceUnavailable)?;
    if before.len() > MAX_COOKIE_BYTES as u64 {
        return Err(CookieImportError::TooLarge);
    }
    let mut bytes = SensitiveBytes(Vec::with_capacity(MAX_COOKIE_BYTES + 1));
    let mut block = SensitiveBlock([0; 4096]);
    loop {
        let remaining = (MAX_COOKIE_BYTES + 1 - bytes.0.len()).min(block.0.len());
        let n = file
            .read(&mut block.0[..remaining])
            .map_err(|_| CookieImportError::SourceUnavailable)?;
        if n == 0 {
            break;
        }
        if bytes.0.len() + n > MAX_COOKIE_BYTES {
            return Err(CookieImportError::TooLarge);
        }
        bytes.0.extend_from_slice(&block.0[..n]);
    }
    let after = file
        .metadata()
        .map_err(|_| CookieImportError::SourceUnavailable)?;
    if before.len() != bytes.0.len() as u64
        || before.len() != after.len()
        || before
            .modified()
            .map_err(|_| CookieImportError::SourceUnavailable)?
            != after
                .modified()
                .map_err(|_| CookieImportError::SourceUnavailable)?
        || before
            .created()
            .map_err(|_| CookieImportError::SourceUnavailable)?
            != after
                .created()
                .map_err(|_| CookieImportError::SourceUnavailable)?
        || file_identity::from_file(&file).map_err(|_| CookieImportError::SourceUnavailable)? != id
    {
        return Err(CookieImportError::SourceUnavailable);
    }
    for pin in &pins {
        check_pin(pin).map_err(|_| CookieImportError::SourceUnavailable)?;
    }
    kind(&file, false).map_err(|_| CookieImportError::SourceUnavailable)?;
    Ok(bytes)
}

pub(super) struct OwnedCopy {
    pins: Vec<Pin>,
    owner: Option<File>,
    owner_path: SensitivePath,
    owner_id: NativeFileIdentity,
    copy: Option<File>,
    copy_path: SensitivePath,
    copy_id: NativeFileIdentity,
    sid: String,
    removed: bool,
    initialization_started: bool,
    journal: Option<Mutex<journal::Journal>>,
    marker: Option<(File, recovery_marker::Marker, NativeFileIdentity)>,
}
impl fmt::Debug for OwnedCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OwnedCopy(<redacted>)")
    }
}
impl OwnedCopy {
    pub(super) fn defer_cleanup(&mut self) {
        self.removed = true;
        self.journal = None;
        self.marker = None;
        self.copy = None;
        self.owner = None;
        self.pins.clear();
    }
    fn owner_valid(&self) -> Result<(), CookieImportError> {
        if self.removed {
            return Err(changed());
        }
        for pin in &self.pins {
            check_pin(pin)?;
        }
        let cache = self.pins.last().ok_or_else(changed)?;
        acl(&cache.file, &self.sid)?;
        let owner = self.owner.as_ref().ok_or_else(changed)?;
        kind(owner, true)?;
        acl(owner, &self.sid)?;
        if file_identity::from_file(owner).map_err(|_| changed())? != self.owner_id {
            return Err(changed());
        }
        let current = open(&self.owner_path.0, true, META_ACCESS, SHARE_ALL, None)?;
        if file_identity::from_file(&current).map_err(|_| changed())? != self.owner_id {
            return Err(changed());
        }
        Ok(())
    }
    pub(super) fn validate_for_tool(&self) -> Result<(), CookieImportError> {
        let guard = self
            .journal
            .as_ref()
            .map(|journal| journal.try_lock().map_err(|_| changed()))
            .transpose()?;
        self.validate_with_journal(guard.as_deref())
    }
    fn validate_with_journal(
        &self,
        locked: Option<&journal::Journal>,
    ) -> Result<(), CookieImportError> {
        self.owner_valid()?;
        let file = self.copy.as_ref().ok_or_else(changed)?;
        kind(file, false)?;
        acl(file, &self.sid)?;
        if file_identity::from_file(file).map_err(|_| changed())? != self.copy_id {
            return Err(changed());
        }
        let current = open(&self.copy_path.0, false, META_ACCESS, SHARE_ALL, None)?;
        kind(&current, false)?;
        acl(&current, &self.sid)?;
        if file_identity::from_file(&current).map_err(|_| changed())? != self.copy_id {
            return Err(changed());
        }
        let allowed: &[&str] = if self.journal.is_some() && self.marker.is_some() {
            &recovery_marker::OWNER_ENTRIES
        } else if !self.initialization_started {
            &["cookies.txt"]
        } else {
            return Err(changed());
        };
        let mut count = 0;
        for entry in fs::read_dir(&self.owner_path.0)
            .map_err(|_| changed())?
            .take(allowed.len() + 1)
        {
            let name = entry.map_err(|_| changed())?.file_name();
            if !allowed.iter().any(|known| name == *known) {
                return Err(changed());
            }
            count += 1;
        }
        if count != allowed.len() {
            return Err(changed());
        }
        if let (Some(journal), Some((marker_file, marker, marker_id))) = (locked, &self.marker) {
            let cache = self.pins.last().ok_or_else(changed)?;
            marker
                .check_actual(
                    &self.sid,
                    [
                        &cache.file,
                        self.owner.as_ref().ok_or_else(changed)?,
                        file,
                        marker_file,
                    ],
                )
                .map_err(|_| changed())?;
            if recovery_marker::read_marker(marker_file).map_err(|_| changed())? != *marker {
                return Err(changed());
            }
            let current = open(
                &self.owner_path.0.join(recovery_marker::MARKER_NAME),
                false,
                META_ACCESS,
                SHARE_ALL,
                None,
            )?;
            kind(&current, false)?;
            acl(&current, &self.sid)?;
            if file_identity::from_file(&current).map_err(|_| changed())? != *marker_id {
                return Err(changed());
            }
            let (anchors, nonce, digest) = marker.journal_seed();
            let record = journal.record().map_err(|_| changed())?;
            if record.anchors() != anchors
                || record.session_nonce() != nonce
                || record.record_digest() != digest
            {
                return Err(changed());
            }
        }
        Ok(())
    }
    fn initialize_binding(&mut self) -> Result<(), CookieImportError> {
        let mut nonce = [0u8; 16];
        if unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                nonce.as_mut_ptr(),
                nonce.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } != 0
            || nonce == [0; 16]
        {
            return Err(CookieImportError::StorageUnavailable);
        }
        let anchor = |file: &File| match file_identity::from_file(file).map_err(|_| changed())? {
            NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } => Ok(crate::cookies::windows_records::WindowsAnchor {
                volume_serial,
                file_id,
            }),
            _ => Err(changed()),
        };
        let cache = anchor(&self.pins.last().ok_or_else(changed)?.file)?;
        let copy = anchor(self.copy.as_ref().ok_or_else(changed)?)?;
        let owner = self.owner.as_ref().ok_or_else(changed)?;
        let base = recovery_marker::base_digest(nonce, cache, anchor(owner)?, copy)
            .map_err(|_| changed())?;
        let journal = journal::Journal::create(
            owner,
            &self.owner_path.0,
            &self.sid,
            nonce,
            base,
            cache,
            copy,
        )
        .map_err(|_| CookieImportError::StorageUnavailable)?;
        self.journal = Some(Mutex::new(journal));
        let descriptor = Descriptor::private(&self.sid)?;
        let mut file = open(
            &self.owner_path.0.join(recovery_marker::MARKER_NAME),
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )?;
        let id = file_identity::from_file(&file).map_err(|_| changed())?;
        let marker = recovery_marker::Marker::from_created(
            nonce,
            self.journal
                .as_ref()
                .ok_or_else(changed)?
                .try_lock()
                .map_err(|_| changed())?
                .anchors(),
            anchor(&file)?,
        )
        .map_err(|_| changed())?;
        file.write_all(&marker.encode().map_err(|_| changed())?)
            .and_then(|_| file.sync_all())
            .map_err(|_| CookieImportError::StorageUnavailable)?;
        self.marker = Some((file, marker, id));
        self.validate_for_tool()
    }
    pub(super) fn cleanup(&mut self) -> Result<(), CookieImportError> {
        if self.removed {
            return Ok(());
        }
        let guard = self
            .journal
            .as_ref()
            .map(|journal| journal.try_lock().map_err(|_| changed()))
            .transpose()?;
        self.validate_with_journal(guard.as_deref())?;
        let expected: Vec<(&str, NativeFileIdentity)> = if let Some(journal) = guard.as_deref() {
            let record = journal.record().map_err(|_| changed())?;
            let is_empty = record.is_pristine();
            let receipts = journal.all_receipts_match().map_err(|_| changed())?;
            if !is_empty && !receipts {
                return Err(CookieImportError::ReaderUnconfirmed);
            }
            let a = journal.anchors();
            let id = |anchor: crate::cookies::windows_records::WindowsAnchor| {
                NativeFileIdentity::Windows {
                    volume_serial: anchor.volume_serial,
                    file_id: anchor.file_id,
                }
            };
            vec![
                ("cookies.txt", self.copy_id.clone()),
                ("reader-state.json", id(a.journal)),
                ("reader0.receipt", id(a.receipts[0])),
                ("reader1.receipt", id(a.receipts[1])),
                (
                    recovery_marker::MARKER_NAME,
                    self.marker.as_ref().ok_or_else(changed)?.2.clone(),
                ),
            ]
        } else {
            if self.initialization_started {
                return Err(changed());
            }
            vec![("cookies.txt", self.copy_id.clone())]
        };
        let mut deletes = Vec::with_capacity(expected.len());
        for (name, identity) in expected {
            let delete = open(
                &self.owner_path.0.join(name),
                false,
                DELETE | META_ACCESS,
                SHARE_ALL,
                None,
            )?;
            kind(&delete, false)?;
            acl(&delete, &self.sid)?;
            if file_identity::from_file(&delete).map_err(|_| changed())? != identity {
                return Err(changed());
            }
            deletes.push(delete);
        }
        self.validate_with_journal(guard.as_deref())?;
        for delete in &deletes {
            disposition(delete)?;
        }
        drop(deletes);
        drop(guard);
        self.journal = None;
        self.marker = None;
        self.copy = None;
        self.owner_valid()?;
        if count(&self.owner_path.0, 0)? != 0 {
            return Err(changed());
        }
        disposition(self.owner.as_ref().ok_or_else(changed)?)?;
        self.owner = None;
        self.removed = true;
        self.pins.clear();
        Ok(())
    }
}
impl Drop for OwnedCopy {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
fn disposition(file: &File) -> Result<(), CookieImportError> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: retained verified DELETE handle; no path-based deletion or POSIX override.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            std::mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(changed());
    }
    Ok(())
}

/// Hold exact RAII ownership throughout every write/flush failure and unwind.
/// Only this private helper accepts a writer; public imports always use real I/O.
fn write_owned_copy(
    mut owned: OwnedCopy,
    writer: impl FnOnce(&mut File) -> std::io::Result<()>,
) -> Result<OwnedCopy, CookieImportError> {
    let output = owned
        .copy
        .as_mut()
        .ok_or(CookieImportError::StorageUnavailable)?;
    writer(output).map_err(|_| CookieImportError::StorageUnavailable)?;
    Ok(owned)
}

pub(super) fn import(
    source: &Path,
    cache: &Path,
) -> Result<(OwnedCopy, PathBuf, usize, usize), CookieImportError> {
    let data = source_bytes(source)?;
    let entries = validate(&data.0)?;
    let sid = current_sid()?;
    let descriptor = Descriptor::private(&sid)?;
    let (root, names) = path_parts(cache).map_err(|_| CookieImportError::UnsafeCache)?;
    let pins = chain(root, &names).map_err(|_| CookieImportError::UnsafeCache)?;
    let selected = &pins.last().ok_or_else(changed)?.path.0;
    acl(&pins.last().ok_or_else(changed)?.file, &sid)
        .map_err(|_| CookieImportError::UnsafeCache)?;
    if count(selected, MAX_COOKIE_SESSIONS)? >= MAX_COOKIE_SESSIONS {
        return Err(CookieImportError::CacheFull);
    }
    let mut owner_path = None;
    for _ in 0..16 {
        let candidate = selected.join(random_name()?);
        if create_directory(&candidate, &descriptor)? {
            owner_path = Some(candidate);
            break;
        }
    }
    let owner_path = owner_path.ok_or(CookieImportError::StorageUnavailable)?;
    let owner = open(
        &owner_path,
        true,
        META_ACCESS | FILE_LIST_DIRECTORY | DELETE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
    )?;
    acl(&owner, &sid)?;
    let owner_id = file_identity::from_file(&owner).map_err(|_| changed())?;
    let copy_path = owner_path.join("cookies.txt");
    let copy = open(
        &copy_path,
        false,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | READ_CONTROL,
        SHARE_ALL,
        Some(&descriptor),
    )?;
    acl(&copy, &sid)?;
    let copy_id = file_identity::from_file(&copy).map_err(|_| changed())?;
    let owned = OwnedCopy {
        pins,
        owner: Some(owner),
        owner_path: SensitivePath(owner_path),
        owner_id,
        copy: Some(copy),
        copy_path: SensitivePath(copy_path.clone()),
        copy_id,
        sid,
        removed: false,
        initialization_started: false,
        journal: None,
        marker: None,
    };
    let mut owned = write_owned_copy(owned, |output| {
        output.write_all(&data.0).and_then(|_| output.sync_all())
    })?;
    owned.validate_for_tool()?;
    // From this point, any partial journal/marker creation is conservatively retained.
    owned.initialization_started = true;
    if let Err(error) = owned.initialize_binding() {
        owned.defer_cleanup();
        return Err(error);
    }
    if let Err(error) = owned.validate_for_tool() {
        owned.defer_cleanup();
        return Err(error);
    }
    Ok((owned, copy_path, entries, data.0.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unsafe_namespace_traversal_ads_and_device_names() {
        for text in [
            "relative",
            "C:relative",
            "\\\\server\\share\\cookies",
            "\\\\.\\NUL",
            "C:\\tmp\\..\\cookie",
            "C:\\tmp\\cookie:secret",
            "C:\\tmp\\CON.txt",
            "C:\\tmp\\trailing.",
            "C:\\tmp\\trailing ",
        ] {
            assert!(path_parts(Path::new(text)).is_err(), "unsafe path accepted");
        }
    }
    #[test]
    fn reserved_names_are_bounded_and_unicode_is_allowed() {
        for name in ["con", "NUL.txt", "COM1", "Lpt9.cfg", "CONIN$"] {
            assert!(reserved(name));
        }
        for name in ["한글 쿠키.txt", "COM10", "normal.txt"] {
            assert!(!reserved(name));
        }
    }
    fn owned_fixture() -> (
        tempfile::TempDir,
        PathBuf,
        PathBuf,
        PathBuf,
        Vec<u8>,
        OwnedCopy,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("합성 원본 cookie.txt");
        let bytes=b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-cookie-secret\n".to_vec();
        fs::write(&source, &bytes).unwrap();
        let cache = temp.path().join("private-cookie-cache");
        prepare_cache(&cache).unwrap();
        let sentinel = cache.join("foreign-sentinel");
        fs::write(&sentinel, b"foreign sentinel").unwrap();
        let (owned, copy, _, _) = import(&source, &cache).unwrap();
        (temp, source, cache, copy, bytes, owned)
    }
    fn assert_failure_preserved_source(
        source: &Path,
        cache: &Path,
        copy: &Path,
        bytes: &[u8],
        error: CookieImportError,
    ) {
        assert_eq!(error, CookieImportError::StorageUnavailable);
        assert!(!format!("{error:?} {error}").contains("synthetic-cookie-secret"));
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
        assert!(cache.exists());
        assert!(!copy.exists());
        assert!(!copy.parent().unwrap().exists());
        assert_eq!(fs::read_dir(cache).unwrap().count(), 1);
    }
    #[test]
    fn import_has_original_binding_and_normal_cleanup_preserves_foreign_cache() {
        let (_temp, source, cache, copy, bytes, mut owned) = owned_fixture();
        assert!(
            owned
                .journal
                .as_ref()
                .unwrap()
                .try_lock()
                .unwrap()
                .record()
                .unwrap()
                .is_pristine()
        );
        assert_eq!(fs::read_dir(copy.parent().unwrap()).unwrap().count(), 5);
        owned.validate_for_tool().unwrap();
        owned.cleanup().unwrap();
        assert!(!copy.parent().unwrap().exists());
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
    }
    #[test]
    fn foreign_owner_entry_and_unconfirmed_intent_preserve_bound_copy() {
        let (_temp, source, cache, copy, bytes, mut owned) = owned_fixture();
        let foreign = copy.parent().unwrap().join("foreign-entry");
        fs::write(&foreign, b"foreign").unwrap();
        assert!(owned.cleanup().is_err());
        assert_eq!(fs::read(&copy).unwrap(), bytes);
        fs::remove_file(&foreign).unwrap(); // Owned synthetic fixture only.
        let writer = owned
            .journal
            .as_ref()
            .unwrap()
            .try_lock()
            .unwrap()
            .begin(0, [9; 16])
            .unwrap();
        assert_eq!(
            owned.cleanup().unwrap_err(),
            CookieImportError::ReaderUnconfirmed
        );
        drop(writer);
        owned.defer_cleanup();
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(fs::read(copy).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
    }
    #[test]
    fn gated_intent_drop_and_invalid_order_preserve_pending() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let (_temp, source, cache, copy, bytes, mut owned) = owned_fixture();
        let pending = AtomicBool::new(false);
        {
            assert!(owned.begin_windows_reader(&pending, 2).is_err());
            assert!(!pending.load(Ordering::Acquire));
            let mut gate = owned.begin_windows_reader(&pending, 0).unwrap();
            assert!(gate.release_for_guardian().is_err());
            assert!(gate.confirm_terminal().is_err());
            let launch = gate.take_launch().unwrap();
            assert!(gate.take_launch().is_err());
            assert_eq!(launch.receipt_anchor, launch.intent.anchors().receipts[0]);
            drop(launch);
            assert!(owned.begin_windows_reader(&pending, 0).is_err());
        }
        assert!(pending.load(Ordering::Acquire));
        assert_eq!(
            owned.cleanup().unwrap_err(),
            CookieImportError::ReaderUnconfirmed
        );
        owned.defer_cleanup();
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(fs::read(copy).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
    }
    #[test]
    fn marker_replacement_preserves_original_and_foreign_bytes() {
        use windows_sys::Wdk::Storage::FileSystem::{
            FILE_RENAME_INFORMATION, FileRenameInformation, NtSetInformationFile,
        };
        use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;
        let (_temp, source, cache, copy, bytes, mut owned) = owned_fixture();
        let marker = copy.parent().unwrap().join(recovery_marker::MARKER_NAME);
        let original = copy.parent().unwrap().join("original-marker-sentinel");
        let marker_bytes = fs::read(&marker).unwrap();
        // Inject replacement through a retained DELETE capability. MoveFileEx's
        // path-based target-parent reopen conflicts with the owner's DELETE
        // handle on Server 2022, before this test reaches identity validation.
        // This fixture-only capability does not alter production sharing rights.
        let raw = unsafe {
            ReOpenFile(
                owned.marker.as_ref().unwrap().0.as_raw_handle(),
                DELETE,
                SHARE_ALL,
                FILE_FLAG_OPEN_REPARSE_POINT,
            )
        };
        assert!(!raw.is_null() && raw != INVALID_HANDLE_VALUE);
        let rename_cap = unsafe { File::from_raw_handle(raw) };
        let units: Vec<u16> = original.file_name().unwrap().encode_wide().collect();
        let offset = std::mem::offset_of!(FILE_RENAME_INFORMATION, FileName);
        let used = (offset + units.len() * 2).max(std::mem::size_of::<FILE_RENAME_INFORMATION>());
        let mut storage = vec![0u64; used.div_ceil(8)];
        let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        let mut io_status = IO_STATUS_BLOCK::default();
        // SAFETY: aligned initialized storage, original live DELETE handle and
        // fixed simple fixture leaf; NULL root means original same-parent rename.
        let status = unsafe {
            (*info).FileNameLength = (units.len() * 2) as u32;
            std::ptr::copy_nonoverlapping(
                units.as_ptr(),
                info.cast::<u8>().add(offset).cast::<u16>(),
                units.len(),
            );
            NtSetInformationFile(
                rename_cap.as_raw_handle(),
                &mut io_status,
                info.cast(),
                used as u32,
                FileRenameInformation,
            )
        };
        assert_eq!(status, 0, "fixture rename NTSTATUS {status:#010x}");
        drop(rename_cap);
        fs::write(&marker, &marker_bytes).unwrap();
        assert_eq!(
            owned.cleanup().unwrap_err(),
            CookieImportError::CleanupOwnershipChanged
        );
        owned.defer_cleanup();
        assert_eq!(fs::read(original).unwrap(), marker_bytes);
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(fs::read(copy).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
    }
    #[test]
    fn partial_write_zero_failure_cleans_only_exact_owned_leaf() {
        use std::io::{Seek, SeekFrom};
        let (_temp, source, cache, copy, bytes, owned) = owned_fixture();
        let error = write_owned_copy(owned, |file| {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(b"partial synthetic cookie")?;
            assert_eq!(fs::read(&copy).unwrap(), b"partial synthetic cookie");
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "synthetic write failure",
            ))
        })
        .unwrap_err();
        assert_failure_preserved_source(&source, &cache, &copy, &bytes, error);
    }
    #[test]
    fn full_write_sync_failure_cleans_only_exact_owned_leaf() {
        use std::io::{Seek, SeekFrom};
        let (_temp, source, cache, copy, bytes, owned) = owned_fixture();
        let error = write_owned_copy(owned, |file| {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&bytes)?;
            assert_eq!(fs::read(&copy).unwrap(), bytes);
            // Inject the sync-stage error after an actual complete OS file write.
            Err(std::io::Error::other("synthetic sync failure"))
        })
        .unwrap_err();
        assert_failure_preserved_source(&source, &cache, &copy, &bytes, error);
    }
    #[test]
    fn writer_refuses_deferred_copy_without_cleanup_or_reopening_handles() {
        let (_temp, source, cache, copy, bytes, mut owned) = owned_fixture();
        owned.defer_cleanup();
        let error =
            write_owned_copy(owned, |_| panic!("deferred writer must not run")).unwrap_err();
        assert_eq!(error, CookieImportError::StorageUnavailable);
        assert_eq!(fs::read(&copy).unwrap(), bytes);
        assert_eq!(fs::read(source).unwrap(), bytes);
        assert_eq!(
            fs::read(cache.join("foreign-sentinel")).unwrap(),
            b"foreign sentinel"
        );
    }
}
