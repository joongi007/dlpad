//! Windows owned runtime cache foundation. No deletion and no hard quota.
#![cfg(windows)]
use crate::windows_deno_cache_binding::CacheBinding;
use crate::{
    file_identity::{self, NativeFileIdentity},
    windows_private_security::{PrivateDescriptor, current_sid, verify_acl},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};
const META: u32 = FILE_READ_ATTRIBUTES | READ_CONTROL;
const SHARES: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;
const RECORD_LIMIT: u64 = 4096;
const ENTRIES: usize = 4096;
const LOGICAL_LIMIT: u64 = 32 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheError {
    Changed,
    Unknown,
    Busy,
    Bounds,
    Io,
}
impl std::fmt::Debug for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OwnedDenoCacheError")
    }
}
impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("owned runtime cache validation failed")
    }
}
impl std::error::Error for CacheError {}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerRecord {
    version: u8,
    root: NativeFileIdentity,
    runtime: NativeFileIdentity,
    cache: NativeFileIdentity,
    lock: NativeFileIdentity,
}
pub(crate) struct OwnedDenoCache {
    root_path: PathBuf,
    path: PathBuf,
    sid: String,
    root: File,
    runtime: File,
    cache: File,
    owner: File,
    lock: File,
    record: OwnerRecord,
    owner_id: NativeFileIdentity,
    owner_digest: [u8; 32],
    locked: bool,
}
impl std::fmt::Debug for OwnedDenoCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OwnedDenoCacheRedacted")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CacheInventory {
    pub entries: usize,
    pub logical_bytes: u64,
    pub budget_exceeded: bool,
}
fn units(path: &Path) -> Result<Vec<u16>, CacheError> {
    let p: Vec<u16> = path.as_os_str().encode_wide().collect();
    if !path.is_absolute()
        || p.len() > 32767
        || p.contains(&0)
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(CacheError::Changed);
    }
    Ok(p.into_iter().chain([0]).collect())
}
fn kind(file: &File, directory: bool) -> Result<(), CacheError> {
    let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
    let mut info = FILE_STANDARD_INFO::default();
    if unsafe { GetFileType(file.as_raw_handle()) } != FILE_TYPE_DISK
        || unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of_val(&tag) as u32,
            )
        } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut info as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&info) as u32,
            )
        } == 0
        || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || info.Directory != directory
        || (!directory && info.NumberOfLinks != 1)
        || info.DeletePending
    {
        return Err(CacheError::Changed);
    }
    Ok(())
}
fn open(
    path: &Path,
    directory: bool,
    access: u32,
    create: Option<&PrivateDescriptor>,
) -> Result<File, CacheError> {
    let text = units(path)?;
    let attrs = create.map(PrivateDescriptor::attributes);
    let raw = unsafe {
        CreateFileW(
            text.as_ptr(),
            access,
            SHARES,
            attrs.as_ref().map_or(ptr::null(), |a| a),
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
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(CacheError::Io);
    }
    let file = unsafe { File::from_raw_handle(raw) };
    kind(&file, directory)?;
    Ok(file)
}
fn id(file: &File) -> Result<NativeFileIdentity, CacheError> {
    let i = file_identity::from_file(file).map_err(|_| CacheError::Changed)?;
    match &i {
        NativeFileIdentity::Windows { file_id, .. } if file_id.iter().any(|x| *x != 0) => Ok(i),
        _ => Err(CacheError::Changed),
    }
}
fn directory(path: &Path, descriptor: &PrivateDescriptor) -> Result<(), CacheError> {
    let p = units(path)?;
    let a = descriptor.attributes();
    if unsafe { CreateDirectoryW(p.as_ptr(), &a) } == 0 {
        return Err(CacheError::Unknown);
    }
    Ok(())
}
fn read_record(file: &File) -> Result<OwnerRecord, CacheError> {
    if file.metadata().map_err(|_| CacheError::Io)?.len() > RECORD_LIMIT {
        return Err(CacheError::Bounds);
    }
    let mut f = file;
    f.seek(SeekFrom::Start(0)).map_err(|_| CacheError::Io)?;
    let mut bytes = Vec::new();
    f.take(RECORD_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CacheError::Io)?;
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err(CacheError::Bounds);
    }
    let r: OwnerRecord = serde_json::from_slice(&bytes).map_err(|_| CacheError::Changed)?;
    if r.version != 1 {
        return Err(CacheError::Changed);
    }
    Ok(r)
}
fn same(path: &Path, pin: &File, directory: bool) -> Result<(), CacheError> {
    let current = open(path, directory, META, None)?;
    if id(&current)? != id(pin)? {
        return Err(CacheError::Changed);
    }
    Ok(())
}
fn beneath(parent: &File, child: &File) -> Result<(), CacheError> {
    let p = file_identity::final_path(parent).map_err(|_| CacheError::Changed)?;
    let c = file_identity::final_path(child).map_err(|_| CacheError::Changed)?;
    if !c.starts_with(&p) || c == p {
        return Err(CacheError::Changed);
    }
    Ok(())
}
fn owner_hash(file: &File) -> Result<[u8; 32], CacheError> {
    if file.metadata().map_err(|_| CacheError::Io)?.len() > RECORD_LIMIT {
        return Err(CacheError::Bounds);
    }
    let mut input = file;
    input.seek(SeekFrom::Start(0)).map_err(|_| CacheError::Io)?;
    let mut bytes = Vec::new();
    input
        .take(RECORD_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CacheError::Io)?;
    if bytes.is_empty() || bytes.len() as u64 > RECORD_LIMIT {
        return Err(CacheError::Bounds);
    }
    Ok(Sha256::digest(bytes).into())
}
#[repr(C)]
#[derive(Default)]
struct ObjectBasic {
    attributes: u32,
    granted: u32,
    handles: u32,
    pointers: u32,
    reserved: [u32; 10],
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut std::ffi::c_void;
    fn GetProcAddress(module: *mut std::ffi::c_void, name: *const u8) -> *mut std::ffi::c_void;
}
fn lock_access(file: &File) -> Result<(), CacheError> {
    type Query = unsafe extern "system" fn(
        *mut std::ffi::c_void,
        u32,
        *mut std::ffi::c_void,
        u32,
        *mut u32,
    ) -> i32;
    let dll: Vec<u16> = "ntdll.dll".encode_utf16().chain([0]).collect();
    let module = unsafe { GetModuleHandleW(dll.as_ptr()) };
    if module.is_null() {
        return Err(CacheError::Changed);
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryObject".as_ptr().cast()) };
    if address.is_null() {
        return Err(CacheError::Changed);
    }
    let query: Query = unsafe { std::mem::transmute(address) };
    let mut basic = ObjectBasic::default();
    let mut used = 0;
    let size = std::mem::size_of::<ObjectBasic>() as u32;
    let status = unsafe {
        query(
            file.as_raw_handle(),
            0,
            (&mut basic as *mut ObjectBasic).cast(),
            size,
            &mut used,
        )
    };
    // The original cache lock is read/write, with metadata. Delete/ACL mutation rights are forbidden.
    if status < 0
        || used != size
        || basic.granted & FILE_WRITE_DATA == 0
        || basic.granted & !(FILE_GENERIC_WRITE | FILE_GENERIC_READ | META) != 0
    {
        return Err(CacheError::Changed);
    }
    if unsafe { SetHandleInformation(file.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
        return Err(CacheError::Changed);
    }
    Ok(())
}
impl OwnedDenoCache {
    /// Existing directories require an exact versioned owner record; no ACL repair/adoption.
    pub(crate) fn create_or_reopen(
        root_path: &Path,
        original_root: &File,
    ) -> Result<Self, CacheError> {
        let root = open(root_path, true, META, None)?;
        if id(&root)? != id(original_root)? {
            return Err(CacheError::Changed);
        }
        let sid = current_sid().map_err(|_| CacheError::Changed)?;
        let desc = PrivateDescriptor::new(&sid).map_err(|_| CacheError::Changed)?;
        let runtime_path = root_path.join(".runtime-cache");
        let path = runtime_path.join("deno");
        let created = match fs::symlink_metadata(&runtime_path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                same(root_path, &root, true)?;
                directory(&runtime_path, &desc)?;
                true
            }
            Ok(_) => false,
            Err(_) => return Err(CacheError::Io),
        };
        let runtime = open(&runtime_path, true, META, None)?;
        verify_acl(&runtime, &sid).map_err(|_| CacheError::Changed)?;
        beneath(&root, &runtime)?;
        if created {
            same(root_path, &root, true)?;
            same(&runtime_path, &runtime, true)?;
            directory(&path, &desc)?;
        }
        let cache = open(&path, true, META, None)?;
        verify_acl(&cache, &sid).map_err(|_| CacheError::Changed)?;
        beneath(&runtime, &cache)?;
        let lock = open(
            &path.join("cache.lock"),
            false,
            META | FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            created.then_some(&desc),
        )?;
        verify_acl(&lock, &sid).map_err(|_| CacheError::Changed)?;
        beneath(&cache, &lock)?;
        lock.try_lock().map_err(|_| CacheError::Busy)?;
        let record = OwnerRecord {
            version: 1,
            root: id(&root)?,
            runtime: id(&runtime)?,
            cache: id(&cache)?,
            lock: id(&lock)?,
        };
        let owner_path = path.join("owner.json");
        let owner = open(
            &owner_path,
            false,
            META | FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            created.then_some(&desc),
        )?;
        verify_acl(&owner, &sid).map_err(|_| CacheError::Changed)?;
        beneath(&cache, &owner)?;
        if created {
            let bytes = serde_json::to_vec(&record).map_err(|_| CacheError::Io)?;
            if bytes.len() as u64 > RECORD_LIMIT {
                return Err(CacheError::Bounds);
            }
            let mut f = &owner;
            f.write_all(&bytes).map_err(|_| CacheError::Io)?;
            f.sync_all().map_err(|_| CacheError::Io)?;
        }
        let owner_id = id(&owner)?;
        let owner_digest = owner_hash(&owner)?;
        let owned = Self {
            root_path: root_path.to_owned(),
            path,
            sid,
            root,
            runtime,
            cache,
            owner,
            lock,
            record,
            owner_id,
            owner_digest,
            locked: true,
        };
        owned.verify()?;
        Ok(owned)
    }
    pub(crate) fn binding(&self) -> Result<CacheBinding, CacheError> {
        self.verify()?;
        let binding = CacheBinding {
            version: 1,
            root_path: self.root_path.as_os_str().encode_wide().collect(),
            root: self.record.root.clone(),
            runtime: self.record.runtime.clone(),
            cache: self.record.cache.clone(),
            owner: self.owner_id.clone(),
            lock: self.record.lock.clone(),
            owner_sha256: self.owner_digest,
        };
        binding.validate().map_err(|_| CacheError::Changed)?;
        Ok(binding)
    }
    pub(crate) fn lock_file(&self) -> Result<&File, CacheError> {
        self.verify()?;
        Ok(&self.lock)
    }
    pub(crate) fn release_for_guardian(&mut self) -> Result<(), CacheError> {
        self.verify()?;
        self.lock.unlock().map_err(|_| CacheError::Io)?;
        self.locked = false;
        Ok(())
    }
    pub(crate) fn reacquire(&mut self) -> Result<(), CacheError> {
        if self.locked {
            return Err(CacheError::Busy);
        }
        self.lock.try_lock().map_err(|_| CacheError::Busy)?;
        if let Err(error) = self.verify_original() {
            let _ = self.lock.unlock();
            return Err(error);
        }
        self.locked = true;
        Ok(())
    }
    /// Consumes exactly the inherited original capability. No creation, repair or lock adoption.
    pub(crate) fn from_inherited(binding: &CacheBinding, lock: File) -> Result<Self, CacheError> {
        binding.validate().map_err(|_| CacheError::Changed)?;
        lock_access(&lock)?;
        let root_path = binding.root_path().map_err(|_| CacheError::Changed)?;
        let path = root_path.join(".runtime-cache").join("deno");
        let sid = current_sid().map_err(|_| CacheError::Changed)?;
        // Compare each reopen to the app's original attachment, never to an observed replacement.
        let root = open(&root_path, true, META, None)?;
        let runtime = open(&root_path.join(".runtime-cache"), true, META, None)?;
        let cache = open(&path, true, META, None)?;
        let owner = open(
            &path.join("owner.json"),
            false,
            META | FILE_GENERIC_READ,
            None,
        )?;
        let record = OwnerRecord {
            version: 1,
            root: binding.root.clone(),
            runtime: binding.runtime.clone(),
            cache: binding.cache.clone(),
            lock: binding.lock.clone(),
        };
        let owned = Self {
            root_path,
            path,
            sid,
            root,
            runtime,
            cache,
            owner,
            lock,
            record,
            owner_id: binding.owner.clone(),
            owner_digest: binding.owner_sha256,
            locked: false,
        };
        owned.verify_original()?;
        Ok(owned)
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn verify(&self) -> Result<(), CacheError> {
        if !self.locked {
            return Err(CacheError::Busy);
        }
        self.verify_original()
    }
    fn verify_original(&self) -> Result<(), CacheError> {
        if current_sid().map_err(|_| CacheError::Changed)? != self.sid {
            return Err(CacheError::Changed);
        }
        let runtime_path = self.root_path.join(".runtime-cache");
        same(&self.root_path, &self.root, true)?;
        same(&runtime_path, &self.runtime, true)?;
        same(&self.path, &self.cache, true)?;
        same(&self.path.join("owner.json"), &self.owner, false)?;
        same(&self.path.join("cache.lock"), &self.lock, false)?;
        for (pin, directory) in [
            (&self.runtime, true),
            (&self.cache, true),
            (&self.owner, false),
            (&self.lock, false),
        ] {
            kind(pin, directory)?;
            verify_acl(pin, &self.sid).map_err(|_| CacheError::Changed)?;
        }
        beneath(&self.root, &self.runtime)?;
        beneath(&self.runtime, &self.cache)?;
        beneath(&self.cache, &self.owner)?;
        beneath(&self.cache, &self.lock)?;
        if id(&self.root)? != self.record.root
            || id(&self.runtime)? != self.record.runtime
            || id(&self.cache)? != self.record.cache
            || id(&self.lock)? != self.record.lock
            || id(&self.owner)? != self.owner_id
            || owner_hash(&self.owner)? != self.owner_digest
            || read_record(&self.owner)? != self.record
        {
            return Err(CacheError::Changed);
        }
        Ok(())
    }
    /// Postflight logical accounting only: execution can exceed the budget before this call.
    pub(crate) fn inventory(&self) -> Result<CacheInventory, CacheError> {
        self.verify()?;
        let mut pending = vec![(
            self.path.clone(),
            self.cache.try_clone().map_err(|_| CacheError::Io)?,
        )];
        let mut result = CacheInventory {
            entries: 0,
            logical_bytes: 0,
            budget_exceeded: false,
        };
        while let Some((path, parent)) = pending.pop() {
            same(&path, &parent, true)?;
            for entry in fs::read_dir(&path).map_err(|_| CacheError::Io)? {
                let entry = entry.map_err(|_| CacheError::Io)?;
                result.entries += 1;
                if result.entries > ENTRIES {
                    return Err(CacheError::Bounds);
                }
                let p = entry.path();
                let m = fs::symlink_metadata(&p).map_err(|_| CacheError::Changed)?;
                if m.file_type().is_symlink() {
                    return Err(CacheError::Changed);
                }
                let dir = m.is_dir();
                let file = open(&p, dir, META, None)?;
                beneath(&parent, &file)?;
                same(&p, &file, dir)?;
                if dir {
                    pending.push((p, file));
                } else {
                    result.logical_bytes = result
                        .logical_bytes
                        .checked_add(file.metadata().map_err(|_| CacheError::Io)?.len())
                        .ok_or(CacheError::Bounds)?;
                    result.budget_exceeded = result.logical_bytes > LOGICAL_LIMIT;
                }
            }
            same(&path, &parent, true)?;
        }
        self.verify()?;
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> (tempfile::TempDir, File) {
        let d = tempfile::tempdir().unwrap();
        let pin = open(d.path(), true, META, None).unwrap();
        (d, pin)
    }
    #[test]
    fn create_reopen_busy_and_inventory() {
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        assert_eq!(c.inventory().unwrap().entries, 2);
        assert!(matches!(
            OwnedDenoCache::create_or_reopen(d.path(), &p),
            Err(CacheError::Busy)
        ));
        drop(c);
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        assert!(c.verify().is_ok());
    }
    #[test]
    fn unknown_partial_and_hardlink_preserve() {
        let (d, p) = root();
        fs::create_dir(d.path().join(".runtime-cache")).unwrap();
        assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
        assert!(d.path().join(".runtime-cache").is_dir());
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let record = c.path().join("owner.json");
        drop(c);
        fs::write(&record, b"{").unwrap();
        assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
        assert_eq!(fs::read(record).unwrap(), b"{");
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let a = c.path().join("synthetic");
        let b = c.path().join("alias");
        fs::write(&a, b"fixture").unwrap();
        fs::hard_link(&a, &b).unwrap();
        assert!(matches!(c.inventory(), Err(CacheError::Changed)));
        assert_eq!(fs::read(a).unwrap(), b"fixture");
    }
    #[test]
    fn logical_bound_and_record_mismatch_preserve() {
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let f = File::create(c.path().join("oversized")).unwrap();
        f.set_len(LOGICAL_LIMIT + 1).unwrap();
        let report = c.inventory().unwrap();
        assert!(report.budget_exceeded);
        assert!(report.logical_bytes > LOGICAL_LIMIT);
        assert_eq!(f.metadata().unwrap().len(), LOGICAL_LIMIT + 1);
        let record = c.path().join("owner.json");
        drop(c);
        let mut r: OwnerRecord = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
        r.version = 2;
        let bytes = serde_json::to_vec(&r).unwrap();
        fs::write(&record, &bytes).unwrap();
        assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
        assert_eq!(fs::read(record).unwrap(), bytes);
    }
    #[test]
    fn original_root_mismatch_and_owner_hardlink_rejected() {
        let (d, p) = root();
        let (_foreign_directory, foreign) = root();
        assert!(matches!(
            OwnedDenoCache::create_or_reopen(d.path(), &foreign),
            Err(CacheError::Changed)
        ));
        assert!(!d.path().join(".runtime-cache").exists());
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let record = c.path().join("owner.json");
        let original = fs::read(&record).unwrap();
        let foreign_link = d.path().join("external-sentinel");
        fs::hard_link(&record, &foreign_link).unwrap();
        assert!(matches!(c.verify(), Err(CacheError::Changed)));
        drop(c);
        assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
        assert_eq!(fs::read(&foreign_link).unwrap(), original);
    }
    #[test]
    fn explicit_release_helper_acquire_and_original_attachment() {
        let (directory, root) = root();
        let mut app = OwnedDenoCache::create_or_reopen(directory.path(), &root).unwrap();
        let binding = app.binding().unwrap();
        let inherited = app.lock_file().unwrap().try_clone().unwrap();
        let mut helper = OwnedDenoCache::from_inherited(&binding, inherited).unwrap();
        assert!(matches!(helper.verify(), Err(CacheError::Busy)));
        assert!(matches!(helper.reacquire(), Err(CacheError::Busy)));
        app.release_for_guardian().unwrap();
        assert!(app.binding().is_err());
        assert!(app.inventory().is_err());
        assert!(app.lock_file().is_err());
        helper.reacquire().unwrap();
        assert_eq!(helper.binding().unwrap(), binding);
        assert!(matches!(app.reacquire(), Err(CacheError::Busy)));
        helper.release_for_guardian().unwrap();
        // Close the helper duplicate before parent acquisition; this is same-process unit scope.
        drop(helper);
        app.reacquire().unwrap();
        app.verify().unwrap();
    }
    #[test]
    fn inherited_binding_mismatch_and_wrong_capability_preserve() {
        let (directory, root) = root();
        let app = OwnedDenoCache::create_or_reopen(directory.path(), &root).unwrap();
        let binding = app.binding().unwrap();
        let original = fs::read(app.path().join("owner.json")).unwrap();
        for mode in 0..6 {
            let mut changed = binding.clone();
            match mode {
                0 => changed.owner_sha256[0] ^= 1,
                1 => changed.owner = changed.lock.clone(),
                2 => changed.cache = changed.root.clone(),
                3 => {
                    changed.root_path = directory
                        .path()
                        .join("foreign")
                        .as_os_str()
                        .encode_wide()
                        .collect()
                }
                4 => {
                    changed.runtime = NativeFileIdentity::Windows {
                        volume_serial: 1,
                        file_id: [7; 16],
                    }
                }
                _ => changed.version = 2,
            }
            assert!(
                OwnedDenoCache::from_inherited(
                    &changed,
                    app.lock_file().unwrap().try_clone().unwrap()
                )
                .is_err()
            );
            assert_eq!(fs::read(app.path().join("owner.json")).unwrap(), original);
        }
        let wrong = open(
            &app.path().join("owner.json"),
            false,
            META | FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            None,
        )
        .unwrap();
        assert!(OwnedDenoCache::from_inherited(&binding, wrong).is_err());
        let readonly = open(&app.path().join("cache.lock"), false, META, None).unwrap();
        assert!(OwnedDenoCache::from_inherited(&binding, readonly).is_err());
    }
    #[test]
    fn reacquire_raw_owner_change_releases_lock_and_preserves_bytes() {
        let (directory, root) = root();
        let mut app = OwnedDenoCache::create_or_reopen(directory.path(), &root).unwrap();
        let binding = app.binding().unwrap();
        let inherited = app.lock_file().unwrap().try_clone().unwrap();
        let mut helper = OwnedDenoCache::from_inherited(&binding, inherited).unwrap();
        app.release_for_guardian().unwrap();
        let path = app.path().join("owner.json");
        let mut original = fs::read(&path).unwrap();
        original.push(b'\n');
        fs::write(&path, &original).unwrap();
        assert!(matches!(helper.reacquire(), Err(CacheError::Changed)));
        assert!(helper.verify().is_err());
        let check = open(
            &app.path().join("cache.lock"),
            false,
            META | FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            None,
        )
        .unwrap();
        check.try_lock().unwrap();
        check.unlock().unwrap();
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(matches!(app.reacquire(), Err(CacheError::Changed)));
    }
    // Paste inside windows_owned_deno_cache::tests; existing root()/super helpers reused.
    fn failure_widen_owned_acl(path: &Path, directory: bool, sid: &str) {
        use windows_sys::Win32::Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SE_FILE_OBJECT,
                SetSecurityInfo,
            },
            DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl,
            PROTECTED_DACL_SECURITY_INFORMATION,
        };
        let text: Vec<u16> = format!("D:P(A;;FA;;;{sid})(A;;GR;;;WD)")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut descriptor = ptr::null_mut();
        // SAFETY: test-only SDDL uses actual bounded SID; fresh allocated descriptor is freed below.
        assert_ne!(
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    text.as_ptr(),
                    1,
                    &mut descriptor,
                    ptr::null_mut(),
                )
            },
            0
        );
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = ptr::null_mut();
        assert_ne!(
            unsafe {
                GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted)
            },
            0
        );
        assert_ne!(present, 0);
        let writable = open(path, directory, META | WRITE_DAC, None).unwrap();
        // Only the original synthetic path inside this test's TempDir is altered.
        let status = unsafe {
            SetSecurityInfo(
                writable.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                acl,
                ptr::null_mut(),
            )
        };
        unsafe {
            LocalFree(descriptor);
        }
        assert_eq!(status, 0);
    }

    #[test]
    fn widened_owned_cache_and_owner_acl_preserve_and_refuse_reopen() {
        for directory in [true, false] {
            let (d, p) = root();
            let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
            let owner = c.path().join("owner.json");
            let before = fs::read(&owner).unwrap();
            let sentinel = d.path().join("sentinel");
            fs::write(&sentinel, b"foreign synthetic sentinel").unwrap();
            let path = if directory {
                c.path().to_owned()
            } else {
                owner.clone()
            };
            failure_widen_owned_acl(&path, directory, &current_sid().unwrap());
            assert!(matches!(c.verify(), Err(CacheError::Changed)));
            assert_eq!(fs::read(&owner).unwrap(), before);
            drop(c);
            assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
            assert_eq!(fs::read(&owner).unwrap(), before);
            assert_eq!(fs::read(&sentinel).unwrap(), b"foreign synthetic sentinel");
            assert!(path.exists());
        }
    }

    #[test]
    fn retained_no_delete_share_blocks_original_file_and_directory_replacement() {
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let owner = c.path().join("owner.json");
        let original = fs::read(&owner).unwrap();
        let owner_id = id(&c.owner).unwrap();
        let cache_id = id(&c.cache).unwrap();
        let runtime_id = id(&c.runtime).unwrap();
        for (path, backup) in [
            (owner.clone(), d.path().join("owner-backup")),
            (c.path().to_owned(), d.path().join("cache-backup")),
            (
                d.path().join(".runtime-cache"),
                d.path().join("runtime-backup"),
            ),
        ] {
            assert!(fs::rename(&path, &backup).is_err());
            assert!(path.exists());
            assert!(!backup.exists());
            assert!(c.verify().is_ok());
        }
        assert_eq!(id(&c.owner).unwrap(), owner_id);
        assert_eq!(id(&c.cache).unwrap(), cache_id);
        assert_eq!(id(&c.runtime).unwrap(), runtime_id);
        assert_eq!(fs::read(owner).unwrap(), original);
    }

    #[test]
    fn oversized_and_unknown_owner_json_are_preserved() {
        for oversized in [true, false] {
            let (d, p) = root();
            let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
            let path = c.path().join("owner.json");
            let mut record: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            drop(c);
            let bytes = if oversized {
                vec![b' '; RECORD_LIMIT as usize + 1]
            } else {
                record["unknown"] = true.into();
                serde_json::to_vec(&record).unwrap()
            };
            fs::write(&path, &bytes).unwrap();
            assert!(OwnedDenoCache::create_or_reopen(d.path(), &p).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert!(path.parent().unwrap().join("cache.lock").exists());
        }
    }

    #[test]
    fn inventory_4097_entries_stops_bounded_and_preserves_owned_files() {
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        // owner.json + cache.lock + 4095 synthetic files = exactly 4097 entries.
        for index in 0..ENTRIES - 1 {
            let path = c.path().join(format!("fixture-{index:04}"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap();
            file.write_all(b"owned synthetic").unwrap();
        }
        assert_eq!(fs::read_dir(c.path()).unwrap().count(), ENTRIES + 1);
        assert!(matches!(c.inventory(), Err(CacheError::Bounds)));
        assert_eq!(fs::read_dir(c.path()).unwrap().count(), ENTRIES + 1);
        assert_eq!(
            fs::read(c.path().join("fixture-0000")).unwrap(),
            b"owned synthetic"
        );
        assert_eq!(
            fs::read(c.path().join("fixture-4094")).unwrap(),
            b"owned synthetic"
        );
        assert!(c.verify().is_ok());
    }

    // This is deliberately ignored in the default suite. Run it explicitly only after
    // CreateSymbolicLinkW capability is confirmed on the private native host. A failed
    // creation is an explicit test failure, never a successful skip or a product PASS.
    #[test]
    #[ignore = "requires actual Windows symlink capability; explicit native acceptance only"]
    fn inventory_actual_reparse_link_is_rejected_and_foreign_target_preserved() {
        let (d, p) = root();
        let c = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let target = d.path().join("outside-cache-sentinel");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("sentinel"), b"foreign preserved").unwrap();
        let link = c.path().join("reparse-fixture");
        let a = units(&link).unwrap();
        let b = units(&target).unwrap();
        // Flags: directory + allow unprivileged create; no system configuration changes.
        let status = unsafe { CreateSymbolicLinkW(a.as_ptr(), b.as_ptr(), 1 | 2) };
        assert!(
            status,
            "actual reparse capability unavailable; no product acceptance"
        );
        assert!(c.inventory().is_err());
        assert_eq!(
            fs::read(target.join("sentinel")).unwrap(),
            b"foreign preserved"
        );
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    }
    // Paste inside windows_owned_deno_cache::tests. No dependency/feature additions.
    // This test MUST fail when native mount-point creation is unavailable; no success skip.
    #[test]
    #[ignore = "requires actual local junction capability; explicit native acceptance"]
    fn inventory_native_owned_junction_is_rejected_and_target_preserved() {
        use windows_sys::Win32::System::IO::DeviceIoControl;
        const SET: u32 = 0x0009_00a4; // FSCTL_SET_REPARSE_POINT, winioctl.h
        const GET: u32 = 0x0009_00a8; // FSCTL_GET_REPARSE_POINT, winioctl.h
        const MOUNT: u32 = 0xa000_0003; // IO_REPARSE_TAG_MOUNT_POINT
        let (d, p) = root();
        let cache = OwnedDenoCache::create_or_reopen(d.path(), &p).unwrap();
        let target = d.path().join("junction-outside-cache");
        fs::create_dir(&target).unwrap();
        let target_pin = open(&target, true, META, None).unwrap();
        let target_id = id(&target_pin).unwrap();
        let sentinel = target.join("sentinel");
        fs::write(&sentinel, b"synthetic target preserved").unwrap();
        let original_owner = fs::read(cache.path().join("owner.json")).unwrap();
        let junction = cache.path().join("owned-junction");
        // Native create fails if any name exists. Parent cache remains pinned without DELETE share.
        let name = units(&junction).unwrap();
        assert_ne!(unsafe { CreateDirectoryW(name.as_ptr(), ptr::null()) }, 0);
        let pin = open(&junction, true, META | FILE_GENERIC_WRITE, None).unwrap();
        let original_id = id(&pin).unwrap();
        beneath(&cache.cache, &pin).unwrap();
        assert_eq!(fs::read_dir(&junction).unwrap().count(), 0);
        // Use the original target handle's actual final local path, not a user-supplied NT path.
        let final_target = file_identity::final_path(&target_pin).unwrap();
        let print: Vec<u16> = final_target.as_os_str().encode_wide().collect();
        assert!(print.starts_with(&[92, 92, 63, 92])); // \\?\ prefix
        assert!(!print[4..].starts_with(&[85, 78, 67, 92])); // no UNC/network target
        let mut substitute: Vec<u16> = vec![92, 63, 63, 92]; // \??\ NT local path
        substitute.extend_from_slice(&print[4..]);
        let sub_bytes = u16::try_from(substitute.len() * 2).unwrap();
        let print_bytes = u16::try_from(print.len() * 2).unwrap();
        let print_offset = sub_bytes.checked_add(2).unwrap();
        let data_len = 8usize + usize::from(sub_bytes) + 2 + usize::from(print_bytes) + 2;
        assert!(data_len + 8 <= 16 * 1024);
        // Exact little-endian Microsoft MountPointReparseBuffer; offsets relative to PathBuffer.
        let mut buffer = Vec::with_capacity(data_len + 8);
        buffer.extend_from_slice(&MOUNT.to_le_bytes());
        buffer.extend_from_slice(&u16::try_from(data_len).unwrap().to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        for value in [0u16, sub_bytes, print_offset, print_bytes] {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        for value in substitute
            .iter()
            .chain([&0u16])
            .chain(print.iter())
            .chain([&0u16])
        {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        assert_eq!(buffer.len(), data_len + 8);
        // Identity still belongs to the original freshly-created empty directory.
        same(&junction, &pin, true).unwrap();
        assert_eq!(id(&pin).unwrap(), original_id);
        let mut returned = 0u32;
        let made = unsafe {
            DeviceIoControl(
                pin.as_raw_handle(),
                SET,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                ptr::null_mut(),
                0,
                &mut returned,
                ptr::null_mut(),
            )
        };
        let create_error = if made == 0 {
            Some(std::io::Error::last_os_error())
        } else {
            None
        };
        let mut observed = [0u8; 16 * 1024];
        let got = if made != 0 {
            unsafe {
                DeviceIoControl(
                    pin.as_raw_handle(),
                    GET,
                    ptr::null(),
                    0,
                    observed.as_mut_ptr().cast(),
                    observed.len() as u32,
                    &mut returned,
                    ptr::null_mut(),
                )
            }
        } else {
            0
        };
        let inventory = if made != 0 {
            Some(cache.inventory())
        } else {
            None
        };
        let target_after = fs::read(&sentinel).unwrap();
        let owner_after = fs::read(cache.path().join("owner.json")).unwrap();
        assert_eq!(id(&pin).unwrap(), original_id);
        assert_eq!(id(&target_pin).unwrap(), target_id);
        // Delete ONLY the original junction/empty-directory capability, never follow its target.
        // Cleanup precedes acceptance assertions so an unsupported native operation leaves no link.
        drop(pin);
        // Reopen without following reparse; compare to the retained original ID before any deletion.
        let raw = unsafe {
            CreateFileW(
                name.as_ptr(),
                META | DELETE,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS,
                ptr::null_mut(),
            )
        };
        assert_ne!(raw, INVALID_HANDLE_VALUE, "original cleanup reopen failed");
        let cleanup = unsafe { File::from_raw_handle(raw) };
        assert_eq!(
            id(&cleanup).unwrap(),
            original_id,
            "replacement preserved; refuse deletion"
        );
        let mut disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
        let deleted = unsafe {
            SetFileInformationByHandle(
                cleanup.as_raw_handle(),
                FileDispositionInfo,
                (&mut disposition as *mut FILE_DISPOSITION_INFO).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        };
        assert_ne!(deleted, 0, "original junction cleanup failed");
        drop(cleanup);
        assert_eq!(
            fs::symlink_metadata(&junction).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(target_after, b"synthetic target preserved");
        assert_eq!(fs::read(&sentinel).unwrap(), b"synthetic target preserved");
        assert_eq!(owner_after, original_owner);
        assert!(cache.verify().is_ok());
        assert!(
            create_error.is_none(),
            "native junction unavailable: {create_error:?}"
        );
        assert_ne!(got, 0, "native reparse readback failed");
        assert!(returned >= 16);
        assert_eq!(u32::from_le_bytes(observed[..4].try_into().unwrap()), MOUNT);
        assert_eq!(observed[..buffer.len()], buffer);
        assert!(matches!(inventory, Some(Err(CacheError::Changed))));
    }
}
