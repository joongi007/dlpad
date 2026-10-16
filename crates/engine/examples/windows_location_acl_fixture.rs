//! Only newly owned temporary directory ACL is modified; no real Documents access.
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows native capability required");
    std::process::exit(2);
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}
#[cfg(windows)]
mod native {
    use dlpad_engine::{
        library::LibraryError,
        location::{self, LocationRequest},
    };
    use std::{
        ffi::c_void,
        fs::{File, OpenOptions},
        io,
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::*,
        Security::{Authorization::*, *},
        Storage::FileSystem::*,
    };
    struct Local(*mut c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    struct OriginalAcl {
        file: File,
        descriptor: Local,
        dacl: *mut ACL,
        protected: bool,
        changed: bool,
    }
    impl OriginalAcl {
        fn restore(&mut self) -> io::Result<()> {
            if !self.changed {
                return Ok(());
            }
            // Original directory HANDLE and live OS-allocated original descriptor remain owned.
            let flags = DACL_SECURITY_INFORMATION
                | if self.protected {
                    PROTECTED_DACL_SECURITY_INFORMATION
                } else {
                    UNPROTECTED_DACL_SECURITY_INFORMATION
                };
            let code = unsafe {
                SetSecurityInfo(
                    self.file.as_raw_handle(),
                    SE_FILE_OBJECT,
                    flags,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    self.dacl,
                    ptr::null_mut(),
                )
            };
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            let mut check = ptr::null_mut();
            let mut dacl = ptr::null_mut();
            let code = unsafe {
                GetSecurityInfo(
                    self.file.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut dacl,
                    ptr::null_mut(),
                    &mut check,
                )
            };
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            let _check = Local(check);
            let mut control = 0;
            let mut revision = 0;
            if unsafe { GetSecurityDescriptorControl(check, &mut control, &mut revision) } == 0 {
                return Err(io::Error::last_os_error());
            }
            if (control & SE_DACL_PROTECTED != 0) != self.protected {
                return Err(io::Error::other("original ACL protection restore mismatch"));
            }
            if dacl.is_null() != self.dacl.is_null() {
                return Err(io::Error::other("original ACL restore mismatch"));
            }
            if !dacl.is_null() {
                // Valid OS ACL allocations, both live; compare exact ACL byte payload.
                let original = unsafe {
                    std::slice::from_raw_parts(
                        self.dacl.cast::<u8>(),
                        (*self.dacl).AclSize as usize,
                    )
                };
                let actual = unsafe {
                    std::slice::from_raw_parts(dacl.cast::<u8>(), (*dacl).AclSize as usize)
                };
                if original != actual {
                    return Err(io::Error::other("original ACL bytes restore mismatch"));
                }
            }
            self.changed = false;
            Ok(())
        }
    }
    impl Drop for OriginalAcl {
        fn drop(&mut self) {
            let _ = &self.descriptor;
            let _ = self.restore();
        }
    }
    fn fail_bool(value: i32) -> io::Result<()> {
        if value == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::Builder::new()
            .prefix("dlpad-location-acl-owned-")
            .tempdir()?;
        let data = root.path().join("data");
        let saved = location::prepare_with_default(
            &data,
            LocationRequest::Selected(root.path().join("원 저장")),
            || unreachable!(),
        )?;
        let sentinel = saved.join("원본.txt");
        std::fs::write(&sentinel, b"owned location sentinel")?;
        let parent = root.path().join("생성 거절 부모");
        std::fs::create_dir(&parent)?;
        let file = OpenOptions::new()
            .access_mode(READ_CONTROL | WRITE_DAC | FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&parent)?;
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        fail_bool(unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) })?;
        if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        {
            return Err(io::Error::other("owned ordinary directory required").into());
        }
        let mut dacl = ptr::null_mut();
        let mut descriptor = ptr::null_mut();
        let code = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut descriptor,
            )
        };
        if code != 0 {
            return Err(io::Error::from_raw_os_error(code as i32).into());
        }
        let original = Local(descriptor);
        let mut control = 0;
        let mut revision = 0;
        fail_bool(unsafe {
            GetSecurityDescriptorControl(descriptor, &mut control, &mut revision)
        })?;
        let mut acl = OriginalAcl {
            file,
            descriptor: original,
            dacl,
            protected: control & SE_DACL_PROTECTED != 0,
            changed: false,
        };
        // Existing public currentSID helper is crate-private, so fixture queries own process token.
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::Threading::*;
        let mut token = ptr::null_mut();
        fail_bool(unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) })?;
        let _token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut bytes = 0;
        unsafe {
            GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut bytes);
        }
        if bytes == 0 || bytes > 65536 {
            return Err(io::Error::other("bounded current token required").into());
        }
        let mut buf = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
        fail_bool(unsafe {
            GetTokenInformation(token, TokenUser, buf.as_mut_ptr().cast(), bytes, &mut bytes)
        })?;
        let sid = unsafe { (*buf.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let mut sidtext = ptr::null_mut();
        fail_bool(unsafe { ConvertSidToStringSidW(sid, &mut sidtext) })?;
        let _sidtext = Local(sidtext.cast());
        let mut used = 0;
        while used < 256 && unsafe { *sidtext.add(used) } != 0 {
            used += 1;
        }
        if used == 256 {
            return Err(io::Error::other("bounded SID required").into());
        }
        let text = String::from_utf16(unsafe { std::slice::from_raw_parts(sidtext, used) })?;
        let sddl: Vec<u16> = format!("D:P(D;;0x00000004;;;{text})(A;;GA;;;{text})")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut denied = ptr::null_mut();
        fail_bool(unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut denied,
                ptr::null_mut(),
            )
        })?;
        let _denied = Local(denied);
        let mut present = 0;
        let mut defaulted = 0;
        let mut deniedacl = ptr::null_mut();
        fail_bool(unsafe {
            GetSecurityDescriptorDacl(denied, &mut present, &mut deniedacl, &mut defaulted)
        })?;
        if present == 0 || deniedacl.is_null() {
            return Err(io::Error::other("explicit deny ACL required").into());
        }
        let code = unsafe {
            SetSecurityInfo(
                acl.file.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                deniedacl,
                ptr::null_mut(),
            )
        };
        if code != 0 {
            return Err(io::Error::from_raw_os_error(code as i32).into());
        }
        acl.changed = true;
        let target = parent.join("dlpad");
        let direct = std::fs::create_dir(&target);
        let location = location::prepare_with_default(
            &data,
            LocationRequest::Selected(target.clone()),
            || unreachable!(),
        );
        // Restore and verify before assertions; preserve root on restore failure.
        if let Err(error) = acl.restore() {
            let _preserved = root.keep();
            return Err(error.into());
        }
        assert!(matches!(direct,Err(ref e) if e.raw_os_error()==Some(5)));
        assert!(matches!(location,Err(LibraryError::Io(ref e)) if e.raw_os_error()==Some(5)));
        assert!(!target.exists());
        assert_eq!(
            location::prepare_with_default(&data, LocationRequest::Restore, || unreachable!())?,
            saved
        );
        assert_eq!(std::fs::read(&sentinel)?, b"owned location sentinel");
        // After exact original ACL restoration, creation succeeds through public API.
        let retry = location::prepare_with_default(
            &data,
            LocationRequest::Selected(target),
            || unreachable!(),
        )?;
        assert!(retry.is_dir());
        drop(acl);
        root.close()?;
        println!(
            "{}",
            serde_json::json!({"schema":1,"passed":true,"native_create_error":5,"location_create_error":5,"saved_root_preserved":true,"sentinel_preserved":true,"original_acl_restored":true,"retry_after_restore":true,"owned_temp_removed":true,"actual_documents_touched":false})
        );
        Ok(())
    }
}
