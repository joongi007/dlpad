//! Headless Windows entry; capabilities arrive through the inherited handle allowlist.
use super::windows_paired_guardian::{self, Request};
use super::windows_paired_guardian_config::{MAX_CONFIG_BYTES, PairedGuardianConfig};
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    os::windows::io::{AsRawHandle, FromRawHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        SetHandleInformation,
    },
    Storage::FileSystem::{FILE_TYPE_DISK, FILE_TYPE_PIPE, GetFileType},
    System::Pipes::PeekNamedPipe,
};
pub(super) const MARKER: &str = "--dlpad-internal-windows-paired-guardian";
fn rejected() -> io::Error {
    io::Error::other("Windows guardian entry rejected")
}
fn read_exact_before(file: &mut File, buffer: &mut [u8], deadline: Instant) -> io::Result<()> {
    let mut offset = 0;
    while offset < buffer.len() {
        if Instant::now() >= deadline {
            return Err(rejected());
        }
        let mut available = 0;
        // SAFETY: retained original pipe handle and live byte-count output.
        if unsafe {
            PeekNamedPipe(
                file.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(rejected());
        }
        if available == 0 {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let count = (buffer.len() - offset).min(available as usize);
        let read = file
            .read(&mut buffer[offset..offset + count])
            .map_err(|_| rejected())?;
        if read == 0 {
            return Err(rejected());
        }
        offset += read;
    }
    Ok(())
}
fn capability(value: &OsStr, expected_type: u32) -> io::Result<HANDLE> {
    let text = value.to_str().ok_or_else(rejected)?;
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(rejected());
    }
    let handle = text.parse::<usize>().map_err(|_| rejected())? as HANDLE;
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(rejected());
    }
    let mut flags = 0;
    // SAFETY: querying a process-local borrowed handle; invalid values are rejected.
    if unsafe { GetHandleInformation(handle, &mut flags) } == 0
        || flags & HANDLE_FLAG_INHERIT == 0
        || unsafe { GetFileType(handle) } != expected_type
    {
        return Err(rejected());
    }
    Ok(handle)
}
fn execute(arguments: &[std::ffi::OsString]) -> io::Result<u32> {
    if arguments.len() != 6 && arguments.len() != 7 {
        return Err(rejected());
    }
    let mut types = vec![
        FILE_TYPE_PIPE,
        FILE_TYPE_PIPE,
        FILE_TYPE_DISK,
        FILE_TYPE_DISK,
        FILE_TYPE_PIPE,
        FILE_TYPE_PIPE,
    ];
    if arguments.len() == 7 {
        types.push(FILE_TYPE_DISK);
    }
    let mut handles = Vec::with_capacity(7);
    for (argument, expected_type) in arguments.iter().zip(types) {
        let handle = capability(argument, expected_type)?;
        if handles.contains(&handle) {
            return Err(rejected());
        }
        handles.push(handle);
    }
    // Exclude management capabilities from all later implicit inheritance.
    for &handle in &handles {
        // SAFETY: all handles were validated, remain live and are exclusively transferred here.
        if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(rejected());
        }
    }
    // SAFETY: distinct original inherited handles transfer once into RAII ownership.
    let mut files: Vec<File> = handles
        .into_iter()
        .map(|handle| unsafe { File::from_raw_handle(handle) })
        .collect();
    let cache_lock = if arguments.len() == 7 {
        Some(files.pop().ok_or_else(rejected)?)
    } else {
        None
    };
    let stderr = files.pop().ok_or_else(rejected)?;
    let stdout = files.pop().ok_or_else(rejected)?;
    let cookie_receipt = files.pop().ok_or_else(rejected)?;
    let managed_receipt = files.pop().ok_or_else(rejected)?;
    let events = files.pop().ok_or_else(rejected)?;
    let mut control = files.pop().ok_or_else(rejected)?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut prefix = [0; 4];
    read_exact_before(&mut control, &mut prefix, deadline)?;
    let len = u32::from_le_bytes(prefix) as usize;
    if len == 0 || len > MAX_CONFIG_BYTES {
        return Err(rejected());
    }
    let mut bytes = vec![0; len];
    read_exact_before(&mut control, &mut bytes, deadline)?;
    let config = PairedGuardianConfig::decode(&bytes).map_err(|_| rejected())?;
    let cache = match (
        config
            .runtime_environment
            .as_ref()
            .and_then(|policy| policy.cache_binding()),
        cache_lock,
    ) {
        (None, None) => None,
        (Some(binding), Some(lock)) => Some(
            crate::windows_owned_deno_cache::OwnedDenoCache::from_inherited(binding, lock)
                .map_err(|_| rejected())?,
        ),
        _ => return Err(rejected()),
    };
    let spec = config.process_spec().map_err(|_| rejected())?;
    let root_path = config.managed_root_path().map_err(|_| rejected())?;
    let owner_path = config.cookie_owner_path().map_err(|_| rejected())?;
    windows_paired_guardian::run(Request {
        spec: &spec,
        cache,
        managed_intent: &config.managed_intent,
        index: config.managed_index,
        managed_nonce: config.managed_nonce,
        root_path: &root_path,
        managed_receipt,
        cookie_intent: &config.cookie_intent,
        slot: config.cookie_index,
        cookie_nonce: config.cookie_nonce,
        owner_path: &owner_path,
        cookie_receipt,
        control,
        events,
        stdout,
        stderr,
    })
}
pub(super) fn entry() -> io::Result<bool> {
    let mut arguments = std::env::args_os().skip(1);
    if arguments.next().as_deref() != Some(OsStr::new(MARKER)) {
        return Ok(false);
    }
    let mut capabilities = Vec::with_capacity(7);
    for argument in arguments.take(8) {
        capabilities.push(argument);
    }
    // Internal helpers never enter GUI initialization, including rejected invocation.
    let code = execute(&capabilities).unwrap_or(126);
    std::process::exit(code as i32);
}
