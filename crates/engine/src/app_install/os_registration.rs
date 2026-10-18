//! Current-user Windows shell registration. Blocking service; no shell/process launch.
//! Existing manifest/bundle namespaces are untouched. Drop never removes anything.
#![cfg(windows)]
use super::{
    FreshRoot, InstallJournalInspection, LocalInspectionRequest, UserBundleInspection,
    inspect_install_journal, inspect_user_bundle,
    windows_caps::{CreatedFile, DeletionRoot, ReadFile, ReadRoot},
};
use crate::{
    file_identity::{self, NativeFileIdentity},
    windows_private_security as security,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::{FileExt, OpenOptionsExt},
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Path, PathBuf},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use windows_sys::{
    Win32::{
        Foundation::*,
        Security::{Authorization::*, Cryptography::*, *},
        Storage::FileSystem::*,
        System::{Com::CoTaskMemFree, Registry::*, Threading::*},
        UI::Shell::{FOLDERID_LocalAppData, FOLDERID_Programs, SHGetKnownFolderPath},
    },
    core::GUID,
};
const MAX_IMAGE: u64 = 64 * 1024 * 1024;
const MAX_RECORD: usize = 65536;
const HELPER: &str = "dlpad-uninstall.exe";
const OWNER: &str = "shell-owner.dpapi";
const REG_PARENT: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Windows shell registration did not match its original binding",
    )
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        Err(io::ErrorKind::Interrupted.into())
    } else if Instant::now() >= d {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn wide(v: &std::ffi::OsStr) -> io::Result<Vec<u16>> {
    let mut v: Vec<_> = v.encode_wide().collect();
    if v.is_empty() || v.len() > 32760 || v.contains(&0) {
        return Err(bad());
    }
    v.push(0);
    Ok(v)
}
fn text(v: &str) -> io::Result<Vec<u16>> {
    wide(std::ffi::OsStr::new(v))
}
fn digest(file: &File, c: &AtomicBool, d: Instant) -> io::Result<(String, u64)> {
    let bytes = file.metadata()?.len();
    if bytes == 0 || bytes > MAX_IMAGE {
        return Err(bad());
    }
    let id = file_identity::from_file(file)?;
    let mut h = Sha256::new();
    let mut at = 0;
    let mut buf = [0; 65536];
    while at < bytes {
        check(c, d)?;
        let cap = (bytes - at).min(buf.len() as u64) as usize;
        let n = file.seek_read(&mut buf[..cap], at)?;
        if n == 0 {
            return Err(bad());
        }
        h.update(&buf[..n]);
        at += n as u64;
    }
    check(c, d)?;
    if file.metadata()?.len() != bytes || file_identity::from_file(file)? != id {
        return Err(bad());
    }
    Ok((format!("{:x}", h.finalize()), bytes))
}
fn pe(file: &File) -> io::Result<()> {
    let mut mz = [0; 64];
    if file.seek_read(&mut mz, 0)? != 64 || &mz[..2] != b"MZ" {
        return Err(bad());
    }
    let offset = u32::from_le_bytes(mz[60..64].try_into().map_err(|_| bad())?) as u64;
    if offset > 1024 * 1024 {
        return Err(bad());
    }
    let mut head = [0; 96];
    if file.seek_read(&mut head, offset)? != head.len()
        || &head[..4] != b"PE\0\0"
        || u16::from_le_bytes([head[4], head[5]]) != 0x8664
        || u16::from_le_bytes([head[24], head[25]]) != 0x20b
    {
        return Err(bad());
    }
    Ok(())
}
fn reopen(file: &File, access: u32, share: u32) -> io::Result<File> {
    let raw = unsafe {
        ReOpenFile(
            file.as_raw_handle(),
            access,
            share,
            FILE_FLAG_OPEN_REPARSE_POINT,
        )
    };
    if raw == INVALID_HANDLE_VALUE || raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let out = unsafe { File::from_raw_handle(raw) };
    if file_identity::from_file(&out)? != file_identity::from_file(file)? {
        return Err(bad());
    }
    Ok(out)
}
fn seal(created: CreatedFile) -> io::Result<File> {
    let weak = reopen(
        created.file(),
        GENERIC_READ | READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
    )?;
    drop(created);
    reopen(
        &weak,
        GENERIC_READ | READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_DELETE,
    )
}
fn copy(input: &File, output: &mut File, c: &AtomicBool, d: Instant) -> io::Result<()> {
    let (before, bytes) = digest(input, c, d)?;
    let mut at = 0;
    let mut b = [0; 65536];
    while at < bytes {
        check(c, d)?;
        let cap = (bytes - at).min(b.len() as u64) as usize;
        let n = input.seek_read(&mut b[..cap], at)?;
        if n == 0 {
            return Err(bad());
        }
        output.write_all(&b[..n])?;
        at += n as u64;
    }
    output.sync_all()?;
    output.rewind()?;
    if digest(input, c, d)? != (before.clone(), bytes) || digest(output, c, d)? != (before, bytes) {
        return Err(bad());
    }
    Ok(())
}
fn nonce() -> io::Result<String> {
    let mut v = [0; 16];
    if unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            v.as_mut_ptr(),
            16,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    } < 0
    {
        return Err(bad());
    }
    Ok(v.iter().map(|x| format!("{x:02x}")).collect())
}
fn current_image(c: &AtomicBool, d: Instant) -> io::Result<File> {
    check(c, d)?;
    let path = std::env::current_exe()?;
    let observation = file_identity::open_regular_no_reparse(&path)?;
    // The observation capability grants attributes only. Read PE bytes through
    // the same original file, without adopting a second pathname observation.
    let f = reopen(
        &observation,
        GENERIC_READ | READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
    )?;
    pe(&f)?;
    digest(&f, c, d)?;
    Ok(f)
}
/// Original private temporary copy; no cleanup is performed by Drop.
pub struct OwnedUninstallCopy {
    root: FreshRoot,
    source: File,
    copy: File,
    sha: String,
    sid: String,
    path: PathBuf,
}
impl OwnedUninstallCopy {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn sha256(&self) -> &str {
        &self.sha
    }
    pub fn source(&self) -> &File {
        &self.source
    }
    pub fn copy(&self) -> &File {
        &self.copy
    }
    pub fn current_sid(&self) -> &str {
        &self.sid
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        self.root.verify()?;
        if security::current_sid()? != self.sid
            || file_identity::final_path(&self.copy)? != self.path
            || digest(&self.source, c, d)?.0 != self.sha
            || digest(&self.copy, c, d)?.0 != self.sha
        {
            return Err(bad());
        }
        security::verify_acl(&self.copy, &self.sid)?;
        pe(&self.source)?;
        pe(&self.copy)
    }
}
pub fn prepare_current_uninstall_copy(
    c: &AtomicBool,
    d: Instant,
) -> io::Result<OwnedUninstallCopy> {
    let source = current_image(c, d)?;
    let sid = security::current_sid()?;
    security::verify_acl(&source, &sid)?;
    let root =
        FreshRoot::create_new(&std::env::temp_dir().join(format!("dlpad-uninstall-{}", nonce()?)))?;
    let mut created = root.create_file(HELPER)?;
    copy(&source, created.file_mut(), c, d)?;
    let sha = digest(&source, c, d)?.0;
    let copy = seal(created)?;
    let path = file_identity::final_path(&copy)?;
    let result = OwnedUninstallCopy {
        root,
        source,
        copy,
        sha,
        sid,
        path,
    };
    result.verify_current(c, d)?;
    Ok(result)
}
pub fn verify_uninstall_handoff_files(
    source: &File,
    copy: &File,
    sha: &str,
    sid: &str,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<()> {
    check(c, d)?;
    if security::current_sid()? != sid
        || sha.len() != 64
        || !sha
            .bytes()
            .all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x))
    {
        return Err(bad());
    }
    let this = current_image(c, d)?;
    if file_identity::from_file(&this)? != file_identity::from_file(copy)?
        || file_identity::from_file(source)? == file_identity::from_file(copy)?
        || digest(source, c, d)?.0 != sha
        || digest(copy, c, d)?.0 != sha
    {
        return Err(bad());
    }
    security::verify_acl(source, sid)?;
    security::verify_acl(copy, sid)?;
    pe(source)?;
    pe(copy)
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema: u32,
    domain: String,
    sid: String,
    id: String,
    bundle: PathBuf,
    journal: PathBuf,
    bundle_id: NativeFileIdentity,
    journal_id: NativeFileIdentity,
    active_sha: String,
    version: String,
    exe: PathBuf,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: NativeFileIdentity,
    sha: String,
    bytes: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerRecord {
    binding: Binding,
    support_id: NativeFileIdentity,
    programs_id: NativeFileIdentity,
    helper: Entry,
    shortcut: Entry,
    owner_id: NativeFileIdentity,
}
struct Locations {
    support: PathBuf,
    programs: PathBuf,
    registry: String,
}
fn known(id: &GUID) -> io::Result<PathBuf> {
    let mut p = ptr::null_mut();
    let hr = unsafe { SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut p) };
    if hr < 0 || p.is_null() {
        return Err(io::Error::from_raw_os_error(hr));
    }
    let result = (|| {
        let mut n = 0;
        while n < 32768 && unsafe { *p.add(n) } != 0 {
            n += 1
        }
        if n == 32768 {
            return Err(bad());
        }
        Ok(PathBuf::from(std::ffi::OsString::from_wide(unsafe {
            std::slice::from_raw_parts(p, n)
        })))
    })();
    unsafe { CoTaskMemFree(p.cast()) };
    result
}
fn locations() -> io::Result<Locations> {
    Ok(Locations {
        support: known(&FOLDERID_LocalAppData)?.join("dlpad-shell"),
        programs: known(&FOLDERID_Programs)?.join("dlpad"),
        registry: REG_PARENT.into(),
    })
}
fn binding(
    bundle: &UserBundleInspection,
    journal: &InstallJournalInspection,
    journal_path: &Path,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<Binding> {
    journal.verify_active_bundle(bundle, c, d)?;
    let sid = security::current_sid()?;
    let root = ReadRoot::open_existing(bundle.path())?;
    let jr = ReadRoot::open_existing(journal_path)?;
    let canonical = std::fs::canonicalize(journal_path)?;
    let root_path = root.path().to_path_buf();
    let raw = serde_json::to_vec(&(root.identity(), &root_path, &sid)).map_err(|_| bad())?;
    let id = format!("{:x}", Sha256::digest(raw));
    let prepared = bundle.selection().original_prepared_record();
    let result = Binding {
        schema: 1,
        domain: "dlpad.windows-shell.v1".into(),
        sid,
        id,
        bundle: root_path,
        journal: canonical,
        bundle_id: root.identity().clone(),
        journal_id: jr.identity().clone(),
        active_sha: format!(
            "{:x}",
            Sha256::digest(bundle.selection().record().canonical_bytes()?)
        ),
        version: prepared.version.clone(),
        exe: Path::new(&bundle.selection().record().registered_path)
            .join(&bundle.selection().record().selected_executable.name),
    };
    journal.verify_active_bundle(bundle, c, d)?;
    Ok(result)
}
fn entry(f: &File, c: &AtomicBool, d: Instant) -> io::Result<Entry> {
    let (sha, bytes) = digest(f, c, d)?;
    Ok(Entry {
        id: file_identity::from_file(f)?,
        sha,
        bytes,
    })
}
fn file_matches(file: &File, expected: &Entry, c: &AtomicBool, d: Instant) -> io::Result<()> {
    if entry(file, c, d)? != *expected {
        return Err(bad());
    }
    Ok(())
}
fn protect(bytes: &[u8], decrypt: bool) -> io::Result<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > MAX_RECORD {
        return Err(bad());
    }
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &input,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &input,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = if output.cbData == 0 || output.cbData as usize > MAX_RECORD {
        Err(bad())
    } else {
        Ok(unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec())
    };
    unsafe { LocalFree(output.pbData.cast()) };
    result
}
fn read_small(f: &File) -> io::Result<Vec<u8>> {
    let n = f.metadata()?.len();
    if n == 0 || n > MAX_RECORD as u64 {
        return Err(bad());
    }
    let mut raw = vec![0; n as usize];
    let mut at = 0;
    while at < raw.len() {
        let got = f.seek_read(&mut raw[at..], at as u64)?;
        if got == 0 {
            return Err(bad());
        }
        at += got;
    }
    Ok(raw)
}
fn container(path: &Path) -> io::Result<ReadRoot> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let fresh = FreshRoot::create_new(path)?;
            fresh.verify()?;
            drop(fresh)
        }
        Err(e) => return Err(e),
    }
    let root = ReadRoot::open_existing(path)?;
    let f = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    security::verify_acl(&f, &security::current_sid()?)?;
    Ok(root)
}
#[link(name = "ole32")]
unsafe extern "system" {
    fn CoInitializeEx(reserved: *const c_void, flags: u32) -> i32;
    fn CoUninitialize();
    fn CoCreateInstance(
        class: *const GUID,
        outer: *mut c_void,
        context: u32,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> i32;
    fn CreateStreamOnHGlobal(global: *mut c_void, delete: i32, out: *mut *mut c_void) -> i32;
}
#[link(name = "shell32")]
unsafe extern "system" {
    fn SHParseDisplayName(
        name: *const u16,
        context: *mut c_void,
        idlist: *mut *mut c_void,
        attributes_in: u32,
        attributes_out: *mut u32,
    ) -> i32;
    fn SHGetPathFromIDListEx(idlist: *const c_void, path: *mut u16, length: u32, flags: u32)
    -> i32;
}
struct ParsedIdList(*mut c_void);
impl Drop for ParsedIdList {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(self.0) };
    }
}
struct Apartment(bool);
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() }
        }
    }
}
struct Com(*mut c_void);
impl Com {
    unsafe fn method<T: Copy>(&self, index: usize) -> T {
        let table = unsafe { *self.0.cast::<*const *const c_void>() };
        let p = unsafe { *table.add(index) };
        unsafe { std::mem::transmute_copy(&p) }
    }
}
impl Drop for Com {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let release: unsafe extern "system" fn(*mut c_void) -> u32 = unsafe { self.method(2) };
            unsafe { release(self.0) };
        }
    }
}
fn hr(code: i32) -> io::Result<()> {
    if code < 0 {
        Err(io::Error::from_raw_os_error(code))
    } else {
        Ok(())
    }
}

/// Shell presentation only: retain native observations and prove the DOS/UNC
/// spelling names the same original file and parent. No execution/delete authority.
fn shell_presentation_target(path: &Path) -> io::Result<(PathBuf, File, File)> {
    let original = file_identity::open_regular_no_reparse(path)?;
    let text = path.to_str().ok_or_else(bad)?;
    let target = if let Some(tail) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{tail}"))
    } else if let Some(tail) = text.strip_prefix(r"\\?\") {
        PathBuf::from(tail)
    } else {
        path.to_owned()
    };
    use std::path::{Component, Prefix};
    if !matches!(target.components().next(), Some(Component::Prefix(prefix))
        if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::UNC(_, _)))
        || !target.is_absolute()
    {
        return Err(bad());
    }
    for part in target.components() {
        match part {
            Component::ParentDir | Component::CurDir => return Err(bad()),
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(bad)?;
                let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
                if name.ends_with(['.', ' '])
                    || name.contains(['<', '>', '"', ':', '|', '?', '*'])
                    || name.chars().any(|v| v < ' ')
                    || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (stem.len() == 4
                        && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && matches!(stem.as_bytes()[3], b'1'..=b'9'))
                {
                    return Err(bad());
                }
            }
            _ => {}
        }
    }
    let observed = file_identity::open_regular_no_reparse(&target)?;
    if file_identity::from_file(&observed)? != file_identity::from_file(&original)?
        || file_identity::final_path(&observed)? != file_identity::final_path(&original)?
    {
        return Err(bad());
    }
    let directory = |name: &Path| -> io::Result<File> {
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(name)?;
        use std::os::windows::fs::MetadataExt;
        let metadata = file.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(bad());
        }
        Ok(file)
    };
    let parent = directory(path.parent().ok_or_else(bad)?)?;
    let named_parent = directory(target.parent().ok_or_else(bad)?)?;
    if file_identity::from_file(&parent)? != file_identity::from_file(&named_parent)?
        || file_identity::final_path(&parent)? != file_identity::final_path(&named_parent)?
    {
        return Err(bad());
    }
    Ok((target, original, parent))
}
fn shortcut_bytes(binding: &Binding) -> io::Result<Vec<u8>> {
    let (target, original_target, _parent) = shell_presentation_target(&binding.exe)?;
    let init = unsafe { CoInitializeEx(ptr::null(), 2) };
    if init < 0 && init != 0x80010106u32 as i32 {
        return Err(io::Error::from_raw_os_error(init));
    }
    let _apartment = Apartment(init >= 0);
    let mut raw = ptr::null_mut();
    hr(unsafe {
        CoCreateInstance(
            &GUID::from_u128(0x00021401_0000_0000_c000_000000000046),
            ptr::null_mut(),
            1,
            &GUID::from_u128(0x000214f9_0000_0000_c000_000000000046),
            &mut raw,
        )
    })?;
    if raw.is_null() {
        return Err(bad());
    }
    let link = Com(raw);
    let long_target = target.as_os_str().encode_wide().count() >= 260;
    if long_target {
        let name = wide(target.as_os_str())?;
        let mut idlist = ptr::null_mut();
        hr(unsafe {
            SHParseDisplayName(
                name.as_ptr(),
                ptr::null_mut(),
                &mut idlist,
                0,
                ptr::null_mut(),
            )
        })?;
        if idlist.is_null() {
            return Err(bad());
        }
        let idlist = ParsedIdList(idlist);
        let mut returned = vec![0_u16; 32768];
        let success = unsafe {
            SHGetPathFromIDListEx(idlist.0, returned.as_mut_ptr(), returned.len() as u32, 0)
        };
        hr(if success != 0 {
            0
        } else {
            0x80004005_u32 as i32
        })?;
        let length = returned
            .iter()
            .position(|unit| *unit == 0)
            .ok_or_else(bad)?;
        let observed_path = PathBuf::from(std::ffi::OsString::from_wide(&returned[..length]));
        let observed = file_identity::open_regular_no_reparse(&observed_path)?;
        if file_identity::from_file(&observed)? != file_identity::from_file(&original_target)?
            || file_identity::final_path(&observed)? != file_identity::final_path(&original_target)?
        {
            return Err(bad());
        }
        let set: unsafe extern "system" fn(*mut c_void, *const c_void) -> i32 =
            unsafe { link.method(5) };
        hr(unsafe { set(link.0, idlist.0) })?;
    }
    for (index, value) in [
        (20, target.as_os_str().to_owned()),
        (9, target.parent().ok_or_else(bad)?.as_os_str().to_owned()),
        (
            7,
            std::ffi::OsString::from(format!("dlpad — {}", binding.bundle.display())),
        ),
    ] {
        if long_target && index == 20 {
            continue;
        }
        let value = wide(&value)?;
        let set: unsafe extern "system" fn(*mut c_void, *const u16) -> i32 =
            unsafe { link.method(index) };
        hr(unsafe { set(link.0, value.as_ptr()) })?;
    }
    let icon = wide(target.as_os_str())?;
    let set_icon: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32 =
        unsafe { link.method(17) };
    hr(unsafe { set_icon(link.0, icon.as_ptr(), 0) })?;
    let mut persist = ptr::null_mut();
    let query: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32 =
        unsafe { link.method(0) };
    hr(unsafe {
        query(
            link.0,
            &GUID::from_u128(0x00000109_0000_0000_c000_000000000046),
            &mut persist,
        )
    })?;
    if persist.is_null() {
        return Err(bad());
    }
    let persist = Com(persist);
    let mut stream = ptr::null_mut();
    hr(unsafe { CreateStreamOnHGlobal(ptr::null_mut(), 1, &mut stream) })?;
    if stream.is_null() {
        return Err(bad());
    }
    let stream = Com(stream);
    let save: unsafe extern "system" fn(*mut c_void, *mut c_void, i32) -> i32 =
        unsafe { persist.method(6) };
    hr(unsafe { save(persist.0, stream.0, 1) })?;
    let seek: unsafe extern "system" fn(*mut c_void, i64, u32, *mut u64) -> i32 =
        unsafe { stream.method(5) };
    let mut length = 0;
    hr(unsafe { seek(stream.0, 0, 1, &mut length) })?;
    if length == 0 || length > MAX_RECORD as u64 {
        return Err(bad());
    }
    hr(unsafe { seek(stream.0, 0, 0, ptr::null_mut()) })?;
    let read: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, *mut u32) -> i32 =
        unsafe { stream.method(3) };
    let mut bytes = vec![0; length as usize];
    let mut at = 0;
    while at < bytes.len() {
        let mut got = 0;
        hr(unsafe {
            read(
                stream.0,
                bytes[at..].as_mut_ptr().cast(),
                (bytes.len() - at) as u32,
                &mut got,
            )
        })?;
        if got == 0 || got as usize > bytes.len() - at {
            return Err(bad());
        }
        at += got as usize;
    }
    Ok(bytes)
}
struct Key(HKEY);
unsafe impl Send for Key {}
impl Drop for Key {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}
fn win(code: u32) -> io::Result<()> {
    if code == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}
fn key_path(loc: &Locations, b: &Binding) -> String {
    format!("{}\\dlpad.{}", loc.registry, b.id)
}
fn open_key(path: &str, write: bool) -> io::Result<Key> {
    let mut key = ptr::null_mut();
    let path = text(path)?;
    win(unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            REG_OPTION_OPEN_LINK,
            KEY_READ | KEY_WOW64_64KEY | if write { KEY_SET_VALUE | DELETE } else { 0 },
            &mut key,
        )
    })?;
    if key.is_null() {
        return Err(bad());
    }
    Ok(Key(key))
}
fn new_key(path: &str, sid: &str) -> io::Result<Key> {
    let mut key = ptr::null_mut();
    let mut disposition = 0;
    let path = text(path)?;
    let descriptor = security::PrivateDescriptor::new(sid)?;
    win(unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS | KEY_WOW64_64KEY,
            &descriptor.attributes(),
            &mut key,
            &mut disposition,
        )
    })?;
    let key = Key(key);
    if disposition != REG_CREATED_NEW_KEY {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    Ok(key)
}
fn key_private(key: &Key, sid: &str) -> io::Result<()> {
    let info = OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION;
    let mut size = 0;
    let code = unsafe { RegGetKeySecurity(key.0, info, ptr::null_mut(), &mut size) };
    if code != ERROR_INSUFFICIENT_BUFFER || size == 0 || size > MAX_RECORD as u32 {
        return Err(bad());
    }
    let mut raw = vec![0; size as usize];
    win(unsafe { RegGetKeySecurity(key.0, info, raw.as_mut_ptr().cast(), &mut size) })?;
    private_descriptor(raw.as_mut_ptr().cast(), sid, KEY_ALL_ACCESS)
}
fn private_descriptor(sd: *mut c_void, sid: &str, access: u32) -> io::Result<()> {
    let mut owner = ptr::null_mut();
    let mut defaulted = 0;
    let mut dacl = ptr::null_mut();
    let mut present = 0;
    let mut control = 0;
    let mut revision = 0;
    let mut expected = ptr::null_mut();
    let sid = text(sid)?;
    if unsafe { ConvertStringSidToSidW(sid.as_ptr(), &mut expected) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = (|| {
        if unsafe { GetSecurityDescriptorOwner(sd, &mut owner, &mut defaulted) } == 0
            || unsafe { GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted) }
                == 0
            || unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) } == 0
            || present == 0
            || dacl.is_null()
            || control & SE_DACL_PROTECTED == 0
            || unsafe { EqualSid(owner, expected) } == 0
        {
            return Err(bad());
        }
        if unsafe { (*dacl).AceCount } != 1 {
            return Err(bad());
        }
        let mut ace = ptr::null_mut();
        if unsafe { GetAce(dacl, 0, &mut ace) } == 0 || ace.is_null() {
            return Err(bad());
        }
        let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
        if allowed.Header.AceType != 0 /* ACCESS_ALLOWED_ACE_TYPE */
            || allowed.Header.AceFlags & INHERITED_ACE as u8 != 0
            || allowed.Mask & access != access
            || unsafe {
                EqualSid(
                    (&allowed.SidStart as *const u32).cast_mut().cast(),
                    expected,
                )
            } == 0
        {
            return Err(bad());
        }
        Ok(())
    })();
    unsafe { LocalFree(expected) };
    result
}
#[derive(Clone, PartialEq, Eq)]
enum Value {
    String(String),
    Dword(u32),
}
fn quote(path: &Path) -> io::Result<String> {
    let value = path.to_str().ok_or_else(bad)?;
    if value.is_empty() || value.contains(['"', '\0', '\r', '\n']) || value.ends_with('\\') {
        return Err(bad());
    }
    Ok(format!("\"{value}\""))
}
fn expected_values(
    b: &Binding,
    support: &Path,
    owner_sha: &str,
) -> io::Result<BTreeMap<&'static str, Value>> {
    let command = format!(
        "{} --uninstall --bundle-root {} --journal-root {}",
        quote(&support.join(HELPER))?,
        quote(&b.bundle)?,
        quote(&b.journal)?
    );
    Ok(BTreeMap::from([
        (
            "DisplayName",
            Value::String(format!("dlpad ({})", &b.id[..8])),
        ),
        ("DisplayVersion", Value::String(b.version.clone())),
        ("Publisher", Value::String("dlpad contributors".into())),
        (
            "InstallLocation",
            Value::String(b.bundle.to_str().ok_or_else(bad)?.into()),
        ),
        (
            "DisplayIcon",
            Value::String(format!("{},0", quote(&b.exe)?)),
        ),
        ("UninstallString", Value::String(command)),
        ("NoModify", Value::Dword(1)),
        ("NoRepair", Value::Dword(1)),
        ("DlpadInstallId", Value::String(b.id.clone())),
        ("DlpadOwnerSHA256", Value::String(owner_sha.into())),
    ]))
}
fn encoded_value(value: &Value) -> io::Result<(u32, Vec<u8>)> {
    match value {
        Value::String(v) => Ok((
            REG_SZ,
            text(v)?.iter().flat_map(|v| v.to_le_bytes()).collect(),
        )),
        Value::Dword(v) => Ok((REG_DWORD, v.to_le_bytes().to_vec())),
    }
}
fn set_values(key: &Key, values: &BTreeMap<&str, Value>) -> io::Result<()> {
    for (name, value) in values {
        let name = text(name)?;
        let (kind, bytes) = encoded_value(value)?;
        win(unsafe {
            RegSetValueExW(
                key.0,
                name.as_ptr(),
                0,
                kind,
                bytes.as_ptr(),
                bytes.len() as u32,
            )
        })?;
    }
    win(unsafe { RegFlushKey(key.0) })
}
fn verify_values(key: &Key, values: &BTreeMap<&str, Value>) -> io::Result<()> {
    let mut children = 0;
    let mut count = 0;
    win(unsafe {
        RegQueryInfoKeyW(
            key.0,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            &mut children,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut count,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    })?;
    if children != 0 || count != values.len() as u32 {
        return Err(bad());
    }
    for (name, value) in values {
        let name = text(name)?;
        let mut kind = 0;
        let mut bytes = vec![0; MAX_RECORD];
        let mut size = bytes.len() as u32;
        win(unsafe {
            RegQueryValueExW(
                key.0,
                name.as_ptr(),
                ptr::null(),
                &mut kind,
                bytes.as_mut_ptr(),
                &mut size,
            )
        })?;
        bytes.truncate(size as usize);
        if encoded_value(value)? != (kind, bytes) {
            return Err(bad());
        }
    }
    Ok(())
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtDeleteKey(key: HKEY) -> i32;
}
fn delete_key(key: &Key) -> io::Result<()> {
    let code = unsafe { NtDeleteKey(key.0) };
    if code < 0 { Err(bad()) } else { Ok(()) }
}
fn namespace(root: &ReadRoot, expected: &[&str], c: &AtomicBool, d: Instant) -> io::Result<()> {
    root.verify()?;
    let mut count = 0;
    for file in std::fs::read_dir(root.path())? {
        check(c, d)?;
        let file = file?;
        let kind = file.file_type()?;
        if !kind.is_file()
            || kind.is_symlink()
            || !file
                .file_name()
                .to_str()
                .is_some_and(|v| expected.contains(&v))
        {
            return Err(bad());
        }
        count += 1;
    }
    if count != expected.len() {
        return Err(bad());
    }
    root.verify()
}
struct InstallMutex {
    handle: HANDLE,
    owner: bool,
    _thread_bound: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl InstallMutex {
    fn acquire(id: &str, sid: &str) -> io::Result<Self> {
        let descriptor = security::PrivateDescriptor::new(sid)?;
        let sid_digest = format!("{:x}", Sha256::digest(sid.as_bytes()));
        let name = text(&format!("Global\\dlpad-shell-v1-{sid_digest}-{id}"))?;
        let handle = unsafe { CreateMutexW(&descriptor.attributes(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut lease = Self {
            handle,
            owner: false,
            _thread_bound: std::marker::PhantomData,
        };
        let mut raw = ptr::null_mut();
        win(unsafe {
            GetSecurityInfo(
                handle,
                SE_KERNEL_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut raw,
            )
        })?;
        if raw.is_null() {
            return Err(bad());
        }
        let result = private_descriptor(raw, sid, MUTEX_ALL_ACCESS);
        unsafe { LocalFree(raw) };
        result?;
        match unsafe { WaitForSingleObject(handle, 0) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => {
                lease.owner = true;
                Ok(lease)
            }
            WAIT_TIMEOUT => Err(io::ErrorKind::WouldBlock.into()),
            _ => Err(io::Error::last_os_error()),
        }
    }
}
impl Drop for InstallMutex {
    fn drop(&mut self) {
        if self.owner {
            unsafe { ReleaseMutex(self.handle) };
        }
        unsafe { CloseHandle(self.handle) };
    }
}
/// All-success summary. Paths are presentation, never deletion/execution capabilities.
#[derive(Debug)]
pub struct ShellRegistration {
    pub registration_id: String,
    pub support_path: PathBuf,
    pub shortcut_path: PathBuf,
    pub display_version: String,
    pub already_registered: bool,
}
pub struct ShellRegistrationInspection {
    binding: Binding,
    support: ReadRoot,
    programs: ReadRoot,
    helper: ReadFile,
    owner: ReadFile,
    shortcut: ReadFile,
    record: OwnerRecord,
    owner_entry: Entry,
    key: Key,
    values: BTreeMap<&'static str, Value>,
    shortcut_name: String,
    // Drop after every native file/root/key pin.
    lease: InstallMutex,
}
impl std::fmt::Debug for ShellRegistrationInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShellRegistrationInspection(<retained original shell objects>)")
    }
}
impl ShellRegistrationInspection {
    pub fn support_path(&self) -> &Path {
        self.support.path()
    }
    pub fn shortcut_path(&self) -> PathBuf {
        self.programs.path().join(&self.shortcut_name)
    }
    pub fn registration_id(&self) -> &str {
        &self.binding.id
    }
    pub fn display_version(&self) -> &str {
        &self.binding.version
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        check(c, d)?;
        if security::current_sid()? != self.binding.sid
            || *self.support.identity() != self.record.support_id
            || *self.programs.identity() != self.record.programs_id
            || *self.owner.identity() != self.record.owner_id
        {
            return Err(bad());
        }
        namespace(&self.support, &[HELPER, OWNER], c, d)?;
        self.helper.verify(&self.support)?;
        self.owner.verify(&self.support)?;
        self.shortcut.verify(&self.programs)?;
        file_matches(self.helper.file(), &self.record.helper, c, d)?;
        file_matches(self.owner.file(), &self.owner_entry, c, d)?;
        file_matches(self.shortcut.file(), &self.record.shortcut, c, d)?;
        key_private(&self.key, &self.binding.sid)?;
        verify_values(&self.key, &self.values)?;
        check(c, d)
    }
    fn summary(&self, already_registered: bool) -> ShellRegistration {
        ShellRegistration {
            registration_id: self.binding.id.clone(),
            support_path: self.support.path().into(),
            shortcut_path: self.shortcut_path(),
            display_version: self.binding.version.clone(),
            already_registered,
        }
    }
}
fn inspect_bound(
    b: Binding,
    loc: &Locations,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ShellRegistrationInspection> {
    check(c, d)?;
    let lease = InstallMutex::acquire(&b.id, &b.sid)?;
    let support_path = loc.support.join(&b.id);
    let shortcut_name = format!("dlpad ({}).lnk", &b.id[..8]);
    let shortcut_path = loc.programs.join(&shortcut_name);
    let registry = key_path(loc, &b);
    let key = open_key(&registry, true);
    let support_exists = std::fs::symlink_metadata(&support_path).is_ok();
    let link_exists = std::fs::symlink_metadata(&shortcut_path).is_ok();
    if matches!(&key,Err(e)if e.kind()==io::ErrorKind::NotFound) && !support_exists && !link_exists
    {
        return Err(io::ErrorKind::NotFound.into());
    }
    let key = key.map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            bad()
        } else {
            e
        }
    })?;
    if !support_exists || !link_exists {
        return Err(bad());
    }
    key_private(&key, &b.sid)?;
    let support = ReadRoot::open_existing(&support_path)?;
    let programs = ReadRoot::open_existing(&loc.programs)?;
    namespace(&support, &[HELPER, OWNER], c, d)?;
    let helper = support.open_file(HELPER)?;
    let owner = support.open_file(OWNER)?;
    let shortcut = programs.open_file(&shortcut_name)?;
    for f in [helper.file(), owner.file(), shortcut.file()] {
        security::verify_acl(f, &b.sid)?;
    }
    let record: OwnerRecord =
        serde_json::from_slice(&protect(&read_small(owner.file())?, true)?).map_err(|_| bad())?;
    if record.binding != b
        || record.support_id != *support.identity()
        || record.programs_id != *programs.identity()
        || record.owner_id != *owner.identity()
    {
        return Err(bad());
    }
    pe(helper.file())?;
    let owner_entry = entry(owner.file(), c, d)?;
    let values = expected_values(&b, &support_path, &owner_entry.sha)?;
    let result = ShellRegistrationInspection {
        lease,
        binding: b,
        support,
        programs,
        helper,
        owner,
        shortcut,
        record,
        owner_entry,
        key,
        values,
        shortcut_name,
    };
    result.verify_current(c, d)?;
    Ok(result)
}
/// Repairs only a missing registry leaf after all authenticated shell files committed.
/// Existing keys (including incomplete keys) and unauthenticated partial files are preserved.
fn recover_missing_registry(
    b: Binding,
    loc: &Locations,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ShellRegistration> {
    check(c, d)?;
    let registry = key_path(loc, &b);
    match open_key(&registry, true) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
        Ok(_) => return Err(bad()),
    }
    let support = ReadRoot::open_existing(&loc.support.join(&b.id))?;
    let programs = ReadRoot::open_existing(&loc.programs)?;
    namespace(&support, &[HELPER, OWNER], c, d)?;
    let helper = support.open_file(HELPER)?;
    let owner = support.open_file(OWNER)?;
    let shortcut = programs.open_file(&format!("dlpad ({}).lnk", &b.id[..8]))?;
    for file in [helper.file(), owner.file(), shortcut.file()] {
        security::verify_acl(file, &b.sid)?;
    }
    let record: OwnerRecord =
        serde_json::from_slice(&protect(&read_small(owner.file())?, true)?).map_err(|_| bad())?;
    if record.binding != b
        || record.support_id != *support.identity()
        || record.programs_id != *programs.identity()
        || record.owner_id != *owner.identity()
    {
        return Err(bad());
    }
    pe(helper.file())?;
    file_matches(helper.file(), &record.helper, c, d)?;
    file_matches(shortcut.file(), &record.shortcut, c, d)?;
    let owner_entry = entry(owner.file(), c, d)?;
    let values = expected_values(&b, support.path(), &owner_entry.sha)?;
    helper.verify(&support)?;
    owner.verify(&support)?;
    shortcut.verify(&programs)?;
    namespace(&support, &[HELPER, OWNER], c, d)?;
    check(c, d)?;
    let key = new_key(&registry, &b.sid)?;
    set_values(&key, &values)?;
    key_private(&key, &b.sid)?;
    verify_values(&key, &values)?;
    drop(key);
    Ok(inspect_bound(b, loc, c, d)?.summary(true))
}
fn checked_inputs(
    r: &LocalInspectionRequest,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<(UserBundleInspection, InstallJournalInspection)> {
    check(c, d)?;
    if !r.bundle_root.is_absolute() || !r.journal_root.is_absolute() {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let journal = inspect_install_journal(&r.journal_root, c, d)?;
    let bundle = inspect_user_bundle(&r.bundle_root, c, d)?;
    journal.verify_active_bundle(&bundle, c, d)?;
    Ok((bundle, journal))
}
pub fn inspect_local_windows_shell(
    r: &LocalInspectionRequest,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ShellRegistrationInspection> {
    let (bundle, journal) = checked_inputs(r, c, d)?;
    let b = binding(&bundle, &journal, &r.journal_root, c, d)?;
    let result = inspect_bound(b, &locations()?, c, d)?;
    journal.verify_active_bundle(&bundle, c, d)?;
    Ok(result)
}
pub fn register_local_windows_shell(
    r: &LocalInspectionRequest,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ShellRegistration> {
    let (bundle, journal) = checked_inputs(r, c, d)?;
    let b = binding(&bundle, &journal, &r.journal_root, c, d)?;
    let result = register_bound(b, &locations()?, c, d)?;
    journal.verify_active_bundle(&bundle, c, d)?;
    Ok(result)
}
fn register_bound(
    b: Binding,
    loc: &Locations,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<ShellRegistration> {
    let _lease = InstallMutex::acquire(&b.id, &b.sid)?;
    match inspect_bound(b.clone(), loc, c, d) {
        Ok(registered) => return Ok(registered.summary(true)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == io::ErrorKind::InvalidData => {
            return recover_missing_registry(b, loc, c, d);
        }
        Err(e) => return Err(e),
    }
    check(c, d)?;
    let source = current_image(c, d)?;
    let source_entry = entry(&source, c, d)?;
    // Shell parsing/serialization can reject a valid native path. Preflight it
    // before creating any support folder, owner record, or shortcut.
    let link_bytes = shortcut_bytes(&b)?;
    let _support_parent = container(&loc.support)?;
    let _programs = container(&loc.programs)?;
    check(c, d)?;
    let root = FreshRoot::create_new(&loc.support.join(&b.id))?;
    let mut helper = root.create_file(HELPER)?;
    copy(&source, helper.file_mut(), c, d)?;
    let mut owner = root.create_file(OWNER)?;
    check(c, d)?;
    // Existing per-user Programs folder is current-user private. A new relative
    // child cannot replace another installation's or an unrelated shortcut.
    let programs_path = &loc.programs;
    let programs_fresh = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(programs_path)?;
    security::verify_acl(&programs_fresh, &b.sid)?;
    let shortcut_name = format!("dlpad ({}).lnk", &b.id[..8]);
    let mut shortcut =
        crate::created_relative_file::CreatedRelativeFile::create(&programs_fresh, &shortcut_name)?;
    shortcut.file_mut().write_all(&link_bytes)?;
    shortcut.file_mut().sync_all()?;
    shortcut.verify_current(&file_identity::final_path(&programs_fresh)?)?;
    let record = OwnerRecord {
        binding: b.clone(),
        support_id: root.identity().clone(),
        programs_id: file_identity::from_file(&programs_fresh)?,
        helper: entry(helper.file(), c, d)?,
        shortcut: entry(shortcut.file_mut(), c, d)?,
        owner_id: owner.identity().clone(),
    };
    if record.helper.sha != source_entry.sha || record.helper.bytes != source_entry.bytes {
        return Err(bad());
    }
    let plain = serde_json::to_vec(&record).map_err(|_| bad())?;
    let encrypted = protect(&plain, false)?;
    owner.file_mut().write_all(&encrypted)?;
    owner.file_mut().sync_all()?;
    check(c, d)?;
    let owner_sha = entry(owner.file(), c, d)?.sha;
    let values = expected_values(&b, root.path(), &owner_sha)?;
    let key = new_key(&key_path(loc, &b), &b.sid)?;
    set_values(&key, &values)?;
    key_private(&key, &b.sid)?;
    verify_values(&key, &values)?;
    if entry(&source, c, d)? != source_entry {
        return Err(bad());
    }
    root.verify()?;
    drop((key, shortcut, helper, owner, root));
    let result = inspect_bound(b, loc, c, d)?;
    Ok(result.summary(false))
}
struct CleanupFile {
    name: String,
    root: usize,
    file: Option<super::windows_caps::DeletionFile>,
    weak: Option<File>,
    expected: Entry,
}
struct CleanupState {
    inspection: Option<ShellRegistrationInspection>,
    reads: Vec<Option<ReadRoot>>,
    roots: Vec<Option<DeletionRoot>>,
    files: Vec<CleanupFile>,
    key: Option<Key>,
    values: BTreeMap<&'static str, Value>,
    sid: String,
    accepted: usize,
    attempted: bool,
    // Drop after both retained inspection and partially acquired capabilities.
    _lease: Option<InstallMutex>,
}
pub struct ShellCleanupFailure {
    kind: io::ErrorKind,
    state: Box<CleanupState>,
}
impl std::fmt::Debug for ShellCleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellCleanupFailure")
            .field("kind", &self.kind)
            .field("accepted_file_dispositions", &self.state.accepted)
            .finish()
    }
}
impl ShellCleanupFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.kind
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.state.attempted
    }
    pub fn accepted_file_dispositions(&self) -> usize {
        self.state.accepted
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.state.files.iter().filter(|v| v.file.is_some()).count()
    }
    /// Retries only retained original capabilities. No paths are reopened.
    /// Pending original weak handles survive busy/share failures during acquisition.
    pub fn retry(self, c: &AtomicBool, d: Instant) -> Result<ShellCleanupOutcome, Self> {
        let mut state = self.state;
        if let Some(inspection) = state.inspection.take() {
            return inspection.cleanup(c, d);
        }
        if state.files.len() != 3 {
            return Err(Self {
                kind: self.kind,
                state,
            });
        }
        let result = finish_cleanup(&mut state, c, d);
        cleanup_result(state, result)
    }
}
#[derive(Debug)]
pub struct ShellCleanupOutcome {
    pub removed_files: usize,
    pub registry_removed: bool,
    pub support_removed: bool,
}
impl ShellRegistrationInspection {
    /// Caller must first finish verified bundle removal. Never removes app data/media.
    /// Errors retain pending original capabilities and every unknown foreign item.
    pub fn cleanup(
        self,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<ShellCleanupOutcome, ShellCleanupFailure> {
        let mut state = Box::new(CleanupState {
            _lease: None,
            inspection: Some(self),
            reads: vec![],
            roots: vec![None, None],
            files: vec![],
            key: None,
            values: BTreeMap::new(),
            sid: String::new(),
            accepted: 0,
            attempted: false,
        });
        let operation = (|| -> io::Result<()> {
            check(c, d)?;
            let inspect = state.inspection.as_ref().ok_or_else(bad)?;
            inspect.verify_current(c, d)?;
            let weak_helper = reopen(
                inspect.helper.file(),
                GENERIC_READ | READ_CONTROL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            )?;
            let weak_link = reopen(
                inspect.shortcut.file(),
                GENERIC_READ | READ_CONTROL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            )?;
            let weak_owner = reopen(
                inspect.owner.file(),
                GENERIC_READ | READ_CONTROL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            )?;
            let original = state.inspection.take().ok_or_else(bad)?;
            let ShellRegistrationInspection {
                lease,
                binding,
                support,
                programs,
                helper,
                owner,
                shortcut,
                record,
                owner_entry,
                key,
                values,
                shortcut_name,
            } = original;
            state._lease = Some(lease);
            state.sid = binding.sid;
            state.values = values;
            state.key = Some(key);
            state.reads = vec![Some(support), Some(programs)];
            drop((helper, owner, shortcut));
            let expected = [
                (0, HELPER.to_owned(), record.helper, weak_helper),
                (1, shortcut_name, record.shortcut, weak_link),
                (0, OWNER.to_owned(), owner_entry, weak_owner),
            ];
            for (root, name, expected, weak) in expected {
                state.files.push(CleanupFile {
                    name,
                    root,
                    file: None,
                    weak: Some(weak),
                    expected,
                });
            }
            finish_cleanup(&mut state, c, d)
        })();
        cleanup_result(state, operation)
    }
}

fn finish_cleanup(state: &mut CleanupState, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    // Every original weak handle and leaf is retained before any acquisition.
    // A sharing/busy error can resume here without observing a new path authority.
    for index in 0..state.roots.len() {
        check(c, d)?;
        if let Some(root) = state.reads[index].take() {
            let identity = root.identity().clone();
            match DeletionRoot::from_read_root_retaining(root, &identity) {
                Ok(root) => state.roots[index] = Some(root),
                Err(failure) => {
                    let (root, error) = *failure;
                    state.reads[index] = Some(root);
                    return Err(error);
                }
            }
        }
    }
    for item in &mut state.files {
        check(c, d)?;
        if item.file.is_none() && item.weak.is_some() {
            let root = state.roots[item.root].as_ref().ok_or_else(bad)?;
            let file = root.open_file(&item.name, &item.expected.id)?;
            file_matches(file.file(), &item.expected, c, d)?;
            item.file = Some(file);
        }
    }
    key_private(state.key.as_ref().ok_or_else(bad)?, &state.sid)?;
    verify_values(state.key.as_ref().ok_or_else(bad)?, &state.values)?;
    for item in &mut state.files {
        check(c, d)?;
        let Some(file) = item.file.as_ref() else {
            continue;
        };
        let root = state.roots[item.root].as_ref().ok_or_else(bad)?;
        file_matches(file.file(), &item.expected, c, d)?;
        state.attempted = true;
        file.mark_delete_original(root)?;
        state.accepted += 1;
        drop(item.file.take());
        drop(item.weak.take());
    }
    check(c, d)?;
    if let Some(support) = state.roots[0].as_ref() {
        support.mark_delete_original()?;
        drop(state.roots[0].take());
    }
    check(c, d)?;
    verify_values(state.key.as_ref().ok_or_else(bad)?, &state.values)?;
    state.attempted = true;
    delete_key(state.key.as_ref().ok_or_else(bad)?)?;
    Ok(())
}
fn cleanup_result(
    state: Box<CleanupState>,
    result: io::Result<()>,
) -> Result<ShellCleanupOutcome, ShellCleanupFailure> {
    match result {
        Ok(()) => Ok(ShellCleanupOutcome {
            removed_files: state.accepted,
            registry_removed: true,
            support_removed: true,
        }),
        Err(error) => Err(ShellCleanupFailure {
            kind: error.kind(),
            state,
        }),
    }
}
#[cfg(test)]
#[path = "os_registration/tests.rs"]
mod tests;
