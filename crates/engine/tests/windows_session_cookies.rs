//! Windows-only synthetic ACL/lease fixtures; never reads actual account cookies.
#![cfg(windows)]
use dlpad_engine::cookies::{CookieImportError, import_session, prepare_session_cache};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, *},
    Storage::FileSystem::*,
    System::Threading::*,
};

const COOKIES: &[u8] = b"# Netscape HTTP Cookie File\n# literal synthetic comment\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\twindows-fixture-only-never-real\n";
struct Local(*mut std::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: this guard owns a Windows LocalAlloc return value.
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}
fn token_user() -> Vec<usize> {
    let mut raw = ptr::null_mut();
    // SAFETY: output is a new query-only token handle for the current process.
    assert_ne!(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) },
        0
    );
    let _token = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut bytes = 0;
    // SAFETY: null-buffer size query, no dereference.
    unsafe {
        GetTokenInformation(raw, TokenUser, ptr::null_mut(), 0, &mut bytes);
    }
    assert!(bytes > 0 && bytes <= 65536);
    let mut data = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
    // SAFETY: pointer-aligned buffer with at least the queried byte size.
    assert_ne!(
        unsafe { GetTokenInformation(raw, TokenUser, data.as_mut_ptr().cast(), bytes, &mut bytes) },
        0
    );
    data
}
fn assert_private_acl(path: &Path) {
    let user = token_user();
    // SAFETY: successful TOKEN_USER buffer is aligned and lives for this function.
    let sid = unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut owner = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut descriptor = ptr::null_mut();
    // SAFETY: existing test-owned path and valid output pointers. Windows allocates SD.
    assert_eq!(
        unsafe {
            GetNamedSecurityInfoW(
                wide(path).as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &mut owner,
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut descriptor,
            )
        },
        0
    );
    let _descriptor = Local(descriptor);
    assert!(!descriptor.is_null() && !owner.is_null() && !dacl.is_null());
    // SAFETY: OS-returned security descriptor and token SID remain live.
    assert_ne!(unsafe { EqualSid(owner, sid) }, 0);
    assert!(unsafe { GetSecurityDescriptorLength(descriptor) } <= 65536);
    let mut control = 0;
    let mut revision = 0;
    // SAFETY: initialized live SD and output pointers.
    assert_ne!(
        unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) },
        0
    );
    assert_ne!(control & SE_DACL_PROTECTED, 0);
    let mut info = ACL_SIZE_INFORMATION::default();
    // SAFETY: valid OS ACL, correctly sized output structure.
    assert_ne!(
        unsafe {
            GetAclInformation(
                dacl,
                (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                AclSizeInformation,
            )
        },
        0
    );
    assert_eq!(info.AceCount, 1);
    assert!(info.AclBytesInUse <= 65536);
    let mut ace = ptr::null_mut();
    // SAFETY: exactly one ACE established above.
    assert_ne!(unsafe { GetAce(dacl, 0, &mut ace) }, 0);
    assert!(!ace.is_null());
    // SAFETY: OS-validated ACL contains this ACE within its allocation.
    let allowed = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
    assert_eq!(allowed.Header.AceType, 0);
    assert_eq!(allowed.Header.AceFlags, 0);
    assert!(allowed.Mask == FILE_ALL_ACCESS || allowed.Mask == GENERIC_ALL);
    let principal = (&allowed.SidStart as *const u32).cast_mut().cast();
    // SAFETY: validated allow ACE SID and live token SID.
    assert_ne!(unsafe { EqualSid(principal, sid) }, 0);
}
fn broaden_owned_acl(path: &Path) {
    let user = token_user();
    let sid = unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut sid_text = ptr::null_mut();
    // SAFETY: live TOKEN_USER SID, Windows allocates string output.
    assert_ne!(unsafe { ConvertSidToStringSidW(sid, &mut sid_text) }, 0);
    let _sid_text = Local(sid_text.cast());
    let mut used = 0;
    while used < 256 && unsafe { *sid_text.add(used) } != 0 {
        used += 1;
    }
    assert!(used < 256);
    let sid_string =
        String::from_utf16(unsafe { std::slice::from_raw_parts(sid_text, used) }).unwrap();
    let text: Vec<u16> = format!("D:P(A;;GA;;;{sid_string})(A;;GR;;;WD)")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut descriptor = ptr::null_mut();
    // SAFETY: bounded valid SDDL for this fixture only, allocated SD returned.
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                ptr::null_mut(),
            )
        },
        0
    );
    let _descriptor = Local(descriptor);
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl = ptr::null_mut();
    // SAFETY: newly created live SD with two explicit allow ACEs.
    assert_ne!(
        unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted) },
        0
    );
    assert_ne!(present, 0);
    // SAFETY: only a newly owned temporary file/cache is mutated; no global settings.
    assert_eq!(
        unsafe {
            SetNamedSecurityInfoW(
                wide(path).as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                dacl,
                ptr::null_mut(),
            )
        },
        0
    );
}

fn descriptor_text(path: &Path) -> String {
    let mut descriptor = ptr::null_mut();
    // SAFETY: OS descriptor is queried only for an owned temporary path.
    assert_eq!(
        unsafe {
            GetNamedSecurityInfoW(
                wide(path).as_ptr(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut descriptor,
            )
        },
        0
    );
    let _descriptor = Local(descriptor);
    let mut text = ptr::null_mut();
    let mut units = 0;
    // SAFETY: OS-owned live SD; converted output is separately LocalAlloc-owned.
    assert_ne!(
        unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                descriptor,
                SDDL_REVISION_1,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &mut text,
                &mut units,
            )
        },
        0
    );
    let _text = Local(text.cast());
    assert!(units > 0 && units <= 65536);
    // SAFETY: OS supplied bounded UTF-16 length including terminal NUL.
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, units as usize - 1) }).unwrap()
}

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    source: PathBuf,
    cache: PathBuf,
    sentinel: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("external-source.cookies");
        let cache = root.join("private-cache");
        let sentinel = root.join("unrelated-sentinel");
        fs::write(&source, COOKIES).unwrap();
        fs::write(&sentinel, b"preserve unrelated synthetic file").unwrap();
        prepare_session_cache(&cache).unwrap();
        Self {
            _temp: temp,
            root,
            source,
            cache,
            sentinel,
        }
    }
    fn preserve(&self) {
        assert_eq!(fs::read(&self.source).unwrap(), COOKIES);
        assert_eq!(
            fs::read(&self.sentinel).unwrap(),
            b"preserve unrelated synthetic file"
        );
    }
}

#[test]
fn literal_private_copy_supports_ordinary_tool_read_write_and_confirmed_cleanup() {
    let f = Fixture::new();
    let mut lease = import_session(&f.source, &f.cache).unwrap();
    let copy = lease.path_for_tool().to_owned();
    assert!(copy.starts_with(&f.cache) && copy != f.source);
    assert_eq!(lease.entries(), 1);
    assert_eq!(lease.bytes(), COOKIES.len());
    for path in [&f.cache, copy.parent().unwrap(), &copy] {
        assert_private_acl(path);
    }
    let reader = lease.begin_reader().unwrap();
    let mut tool = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&copy)
        .unwrap();
    let mut bytes = Vec::new();
    tool.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, COOKIES);
    tool.seek(SeekFrom::Start(0)).unwrap();
    tool.write_all(COOKIES).unwrap();
    tool.flush().unwrap();
    drop(tool);
    reader.confirm();
    lease.validate_for_tool().unwrap();
    lease.cleanup().unwrap();
    assert!(!copy.exists() && !copy.parent().unwrap().exists());
    f.preserve();
}

#[test]
fn pending_reader_and_explicit_defer_drop_preserve_private_copy() {
    for explicit in [false, true] {
        let f = Fixture::new();
        let mut lease = import_session(&f.source, &f.cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        if explicit {
            lease.defer_cleanup();
        } else {
            drop(lease.begin_reader().unwrap());
            assert_eq!(lease.cleanup(), Err(CookieImportError::ReaderUnconfirmed));
        }
        assert!(lease.validate_for_tool().is_err());
        drop(lease);
        assert_eq!(fs::read(&copy).unwrap(), COOKIES);
        assert_private_acl(&copy);
        // Normal tool access remains possible after the preserving Drop.
        drop(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&copy)
                .unwrap(),
        );
        f.preserve();
    }
}

#[test]
fn extra_file_hardlink_or_broadened_copy_acl_is_preserved_and_rejected() {
    for mode in ["extra", "hardlink", "acl"] {
        let f = Fixture::new();
        let mut lease = import_session(&f.source, &f.cache).unwrap();
        let copy = lease.path_for_tool().to_owned();
        let foreign = if mode == "extra" {
            copy.parent().unwrap().join("unrelated-extra")
        } else {
            f.root.join("unrelated-hardlink")
        };
        match mode {
            "extra" => fs::write(&foreign, b"preserve extra fixture").unwrap(),
            "hardlink" => match fs::hard_link(&copy, &foreign) {
                Ok(()) => {
                    eprintln!("synthetic hardlink outcome: created; testing identity rejection");
                }
                Err(error) if error.raw_os_error() == Some(32) => {
                    // Windows may prohibit a hardlink while the pinned private
                    // copy is open. This is a sharing refusal, not a successful
                    // hardlink-detection test, and only exact OS code 32 qualifies.
                    eprintln!(
                        "synthetic hardlink outcome: OS sharing refusal 32; testing unchanged valid lease"
                    );
                    assert!(!foreign.exists());
                    assert_eq!(fs::read(&copy).unwrap(), COOKIES);
                    lease.validate_for_tool().unwrap();
                    lease.begin_reader().unwrap().confirm();
                    f.preserve();
                    lease.cleanup().unwrap();
                    assert!(!copy.exists() && !copy.parent().unwrap().exists());
                    assert!(!foreign.exists());
                    f.preserve();
                    continue;
                }
                Err(error) => panic!("unexpected owned hardlink creation error: {error}"),
            },
            "acl" => {
                broaden_owned_acl(&copy);
                eprintln!(
                    "synthetic copy ACL outcome: broadened; testing validation and cleanup rejection"
                );
            }
            _ => unreachable!(),
        }
        assert!(lease.validate_for_tool().is_err());
        assert!(lease.cleanup().is_err());
        drop(lease);
        assert_eq!(fs::read(&copy).unwrap(), COOKIES);
        if mode == "extra" {
            assert_eq!(fs::read(&foreign).unwrap(), b"preserve extra fixture");
        }
        if mode == "hardlink" {
            assert_eq!(fs::read(&foreign).unwrap(), COOKIES);
        }
        f.preserve();
    }
}

#[test]
fn existing_broadened_cache_acl_is_not_rewritten() {
    let f = Fixture::new();
    broaden_owned_acl(&f.cache);
    let before = descriptor_text(&f.cache);
    assert!(prepare_session_cache(&f.cache).is_err());
    assert!(import_session(&f.source, &f.cache).is_err());
    assert!(
        descriptor_text(&f.cache) == before,
        "existing fixture ACL was rewritten"
    );
    assert_eq!(fs::read_dir(&f.cache).unwrap().count(), 0);
    f.preserve();
}
