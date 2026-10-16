//! Windows current-user private descriptor and strict retained-handle ACL checks.
//! No filesystem opening, deletion, permission mutation or raw SID diagnostics.
use std::{
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, *},
    Storage::FileSystem::FILE_ALL_ACCESS,
    System::Threading::*,
};
const MAX_SECURITY_BYTES: usize = 65536;
fn changed() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "private Windows security validation failed",
    )
}
fn owned(raw: HANDLE) -> Result<OwnedHandle, io::Error> {
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(changed());
    }
    // SAFETY: fresh valid Win32 handle transferred once to RAII ownership.
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}
struct Local(*mut std::ffi::c_void);
impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: returned LocalAlloc allocation retained until this drop.
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn sid_string(sid: PSID) -> Result<String, io::Error> {
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(changed());
    }
    if unsafe { GetLengthSid(sid) } > 68 {
        return Err(changed());
    }
    let mut text = ptr::null_mut();
    // SAFETY: validated OS SID; Windows allocates output owned by Local.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(changed());
    }
    let _guard = Local(text.cast());
    let mut used = 0;
    while used < 256 && unsafe { *text.add(used) } != 0 {
        used += 1;
    }
    if used == 256 {
        return Err(changed());
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, used) }).map_err(|_| changed())
}
pub(crate) fn current_sid() -> Result<String, io::Error> {
    let mut impersonation = ptr::null_mut();
    // Reject thread impersonation instead of silently selecting the process SID.
    if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut impersonation) } != 0 {
        drop(owned(impersonation)?);
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "private Windows security context unavailable",
        ));
    }
    if unsafe { GetLastError() } != ERROR_NO_TOKEN {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "private Windows security context unavailable",
        ));
    }
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(changed());
    }
    let token = owned(raw)?;
    let mut length = 0;
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            ptr::null_mut(),
            0,
            &mut length,
        );
    }
    if length == 0 || length as usize > MAX_SECURITY_BYTES {
        return Err(changed());
    }
    let mut buffer = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            length,
            &mut length,
        )
    } == 0
    {
        return Err(changed());
    }
    // SAFETY: pointer-aligned successful TOKEN_USER buffer lives during SID conversion.
    sid_string(unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid })
}
pub(crate) struct PrivateDescriptor(Local);
impl PrivateDescriptor {
    pub(crate) fn new(sid: &str) -> Result<Self, io::Error> {
        // Public(crate) callers must not inject additional SDDL or allocate
        // unbounded descriptor strings. Current OS SID strings satisfy this bound.
        if sid.len() > 255
            || !sid.starts_with("S-")
            || !sid
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'-' || b == b'S')
        {
            return Err(changed());
        }
        let text: Vec<u16> = format!("O:{sid}D:P(A;;GA;;;{sid})")
            .encode_utf16()
            .chain([0])
            .collect();
        let mut raw = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                SDDL_REVISION_1,
                &mut raw,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(changed());
        }
        Ok(Self(Local(raw)))
    }
    pub(crate) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.0,
            bInheritHandle: 0,
        }
    }
}
pub(crate) fn verify_acl(file: &File, sid: &str) -> Result<(), io::Error> {
    let mut owner = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut descriptor = ptr::null_mut();
    let result = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if result != 0 {
        return Err(changed());
    }
    let descriptor = Local(descriptor);
    if descriptor.0.is_null()
        || unsafe { GetSecurityDescriptorLength(descriptor.0) } as usize > MAX_SECURITY_BYTES
        || sid_string(owner)? != sid
        || dacl.is_null()
    {
        return Err(changed());
    }
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision) } == 0
        || control & SE_DACL_PROTECTED == 0
    {
        return Err(changed());
    }
    let mut present = 0;
    let mut defaulted = 0;
    let mut verified = ptr::null_mut();
    if unsafe {
        GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut verified, &mut defaulted)
    } == 0
        || present == 0
        || verified.is_null()
        || verified != dacl
    {
        return Err(changed());
    }
    let mut info = ACL_SIZE_INFORMATION::default();
    if unsafe {
        GetAclInformation(
            dacl,
            (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
            std::mem::size_of_val(&info) as u32,
            AclSizeInformation,
        )
    } == 0
        || info.AceCount != 1
        || info.AclBytesInUse as usize > MAX_SECURITY_BYTES
    {
        return Err(changed());
    }
    let mut raw = ptr::null_mut();
    if unsafe { GetAce(dacl, 0, &mut raw) } == 0 {
        return Err(changed());
    }
    // SAFETY: GetAce returns an OS-validated ACE within this live ACL allocation.
    let header = unsafe { &*raw.cast::<ACE_HEADER>() };
    if header.AceType != 0 || header.AceFlags != 0 || usize::from(header.AceSize) < 16 {
        return Err(changed());
    }
    let start = dacl as usize;
    let end = start
        .checked_add(info.AclBytesInUse as usize)
        .ok_or_else(changed)?;
    if (raw as usize) < start
        || (raw as usize)
            .checked_add(usize::from(header.AceSize))
            .ok_or_else(changed)?
            > end
    {
        return Err(changed());
    }
    let ace = unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() };
    let principal = (&ace.SidStart as *const u32).cast_mut().cast();
    if !matches!(ace.Mask, FILE_ALL_ACCESS | GENERIC_ALL) || sid_string(principal)? != sid {
        return Err(changed());
    }
    let sid_size = unsafe { GetLengthSid(principal) } as usize;
    if 8 + sid_size != usize::from(header.AceSize) {
        return Err(changed());
    }
    Ok(())
}
