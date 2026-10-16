//! Fresh Windows-only scan CloseWait seed; leaves owned namespace for observer.
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
    use dlpad_engine::store::Store;
    use sha2::{Digest, Sha256};
    use std::{
        ffi::c_void,
        fs::{self, OpenOptions},
        io::{self, Write},
        os::windows::{
            ffi::OsStrExt,
            io::{FromRawHandle, OwnedHandle},
        },
        path::Path,
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::*,
        Security::{Authorization::*, Cryptography::*, *},
        System::Threading::*,
    };
    struct Local(*mut c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    fn check(value: i32) -> io::Result<()> {
        if value == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    fn current_sid() -> Result<String, Box<dyn std::error::Error>> {
        let mut token = ptr::null_mut();
        check(unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) })?;
        let _token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut bytes = 0;
        unsafe {
            GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut bytes);
        }
        if bytes == 0 || bytes > 65536 {
            return Err("bounded token required".into());
        }
        let mut data = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
        check(unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                data.as_mut_ptr().cast(),
                bytes,
                &mut bytes,
            )
        })?;
        let sid = unsafe { (*data.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let mut text = ptr::null_mut();
        check(unsafe { ConvertSidToStringSidW(sid, &mut text) })?;
        let _text = Local(text.cast());
        let mut len = 0;
        while len < 256 && unsafe { *text.add(len) } != 0 {
            len += 1;
        }
        if len == 256 {
            return Err("bounded SID required".into());
        }
        Ok(String::from_utf16(unsafe {
            std::slice::from_raw_parts(text, len)
        })?)
    }
    fn private_directory(path: &Path, sid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = fs::symlink_metadata(path)?;
        use std::os::windows::fs::MetadataExt;
        if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
            return Err("fresh ordinary directory required".into());
        }
        // Use the concrete file/directory full-access mask. Windows may split
        // generic GA into a self FA ACE plus an inherit-only generic ACE.
        let text: Vec<u16> = format!("D:P(A;OICI;FA;;;{sid})")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut descriptor = ptr::null_mut();
        check(unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                ptr::null_mut(),
            )
        })?;
        let _descriptor = Local(descriptor);
        let mut present = 0;
        let mut defaulted = 0;
        let mut dacl = ptr::null_mut();
        check(unsafe {
            GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted)
        })?;
        if present == 0 || dacl.is_null() {
            return Err("private DACL required".into());
        }
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        let code = unsafe {
            SetNamedSecurityInfoW(
                wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                dacl,
                ptr::null_mut(),
            )
        };
        if code != 0 {
            return Err(io::Error::from_raw_os_error(code as i32).into());
        }
        Ok(())
    }
    fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(bytes)?;
        file.sync_all()
    }
    fn sha(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
        if std::env::args_os().len() != 1 {
            return Err("seed accepts no arguments".into());
        }
        let temp = tempfile::Builder::new()
            .prefix("dlpad-library-scan-closewait-owned-")
            .tempdir()?;
        let root = temp.path().canonicalize()?;
        if root.parent() != Some(std::env::temp_dir().canonicalize()?.as_path()) {
            return Err("original TEMP direct child required".into());
        }
        let sid = current_sid()?;
        private_directory(&root, &sid)?;
        let data = root.join("data");
        let media = root.join("media");
        for path in [&data, &media] {
            fs::create_dir(path)?;
            private_directory(path, &sid)?;
        }
        let mut nonce = [0u8; 32];
        if unsafe {
            BCryptGenRandom(
                ptr::null_mut(),
                nonce.as_mut_ptr(),
                nonce.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } < 0
            || nonce.iter().all(|n| *n == 0)
        {
            return Err("native nonce unavailable".into());
        }
        let cap = [b"dlpad-library-scan-closewait-v1\n".as_slice(), &nonce].concat();
        write_new(&root.join("fixture.cap"), &cap)?;
        let sentinel = b"dlpad-library-scan-closewait-foreign-v1";
        let old = b"dlpad-library-scan-old-v1";
        let new = b"dlpad-library-scan-new-v1";
        write_new(&root.join("foreign-sentinel"), sentinel)?;
        write_new(&media.join("old.mp4"), old)?;
        let database = data.join("library.sqlite");
        let mut store = Store::open(&database)?;
        if store.reconcile_library(&media)? != 1 {
            return Err("prior scan count mismatch".into());
        }
        let prior = store.library_page(&media, None, 128)?;
        if prior.len() != 1
            || prior[0].path != media.join("old.mp4")
            || prior[0].size != old.len() as u64
            || prior[0].file_identity.is_none()
        {
            return Err("prior native identity missing".into());
        }
        let rows:Vec<_>=prior.iter().map(|r|serde_json::json!({"path":r.path,"size":r.size,"file_identity":r.file_identity})).collect();
        drop(store);
        write_new(&media.join("new.mp4"), new)?;
        let readonly = rusqlite::Connection::open_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        readonly.execute_batch("PRAGMA query_only=ON;")?;
        let observed: (String, i64, Option<String>) = readonly.query_row(
            "SELECT path,size,file_identity FROM library_entries WHERE present=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let count: i64 = readonly.query_row(
            "SELECT COUNT(*) FROM library_entries WHERE present=1",
            [],
            |r| r.get(0),
        )?;
        if count != 1
            || observed.0 != prior[0].path.to_str().ok_or("prior path unsupported")?
            || u64::try_from(observed.1)? != prior[0].size
            || observed.2 != prior[0].file_identity
        {
            return Err("new file unexpectedly indexed".into());
        }
        drop(readonly);
        let summary = serde_json::json!({"schema":1,"phase":"scan-closewait-seeded","passed":true,"owned_root":root,"cap_sha256":sha(&cap),"cap_bytes":cap.len(),"prior_rows":rows,"old_sha256":sha(old),"new_sha256":sha(new),"sentinel_sha256":sha(sentinel),"actual_documents_touched":false,"root_acl":"protected-current-SID-OICI-FA","cleanup_deferred_to_original_observer":true});
        write_new(
            &root.join("seed-summary.json"),
            serde_json::to_vec(&summary)?.as_slice(),
        )?;
        let kept = temp.keep();
        if kept.canonicalize()? != root {
            return Err("original namespace keep mismatch".into());
        }
        println!("{summary}");
        Ok(())
    }
}
