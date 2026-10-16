//! Standalone exploratory query; NEVER a deletion authority.
#[cfg(windows)]
mod native {
    use std::{ffi::c_void, time::Instant};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }
    type Query = unsafe extern "system" fn(u32, *mut c_void, u32, *mut u32) -> i32;
    pub fn run() -> i32 {
        let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        // Only already-loaded system ntdll; no search-path DLL load.
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        let address = if module.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { GetProcAddress(module, c"NtQuerySystemInformation".as_ptr().cast()) }
        };
        if address.is_null() {
            println!("{{\"outcome\":\"unavailable\",\"authority_eligible\":false}}");
            return 2;
        }
        let query: Query = unsafe { std::mem::transmute(address) };
        let started = Instant::now();
        let mut original = None;
        for round in 0..3 {
            // Experimental class/layout, not a supported SDK authority contract.
            // Aligned bounded storage protects against interpreting arbitrary structs.
            let mut storage = [0u64; 4];
            let mut returned = 0u32;
            let status = unsafe { query(90, storage.as_mut_ptr().cast(), 32, &mut returned) };
            let raw = unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(), 32) };
            let valid = status == 0 && returned == 32 && raw[..16].iter().any(|b| *b != 0);
            let candidate = raw[..16]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            println!(
                "{{\"round\":{round},\"status_hex\":\"{:08x}\",\"returned_bytes\":{returned},\"buffer_bytes\":32,\"candidate_guid_raw\":\"{}\",\"valid_candidate\":{valid},\"authority_eligible\":false}}",
                status as u32,
                if valid { &candidate } else { "omitted" }
            );
            if !valid {
                return if status < 0 { 2 } else { 3 };
            }
            if let Some(previous) = &original {
                if previous != &candidate {
                    return 3;
                }
            } else {
                original = Some(candidate);
            }
        }
        println!(
            "{{\"outcome\":\"stable_candidate_only\",\"pid\":{},\"queries\":3,\"query_elapsed_ns\":{},\"authority_eligible\":false}}",
            std::process::id(),
            started.elapsed().as_nanos()
        );
        0
    }
}
fn main() {
    #[cfg(windows)]
    std::process::exit(native::run());
    #[cfg(not(windows))]
    {
        println!("{{\"outcome\":\"unavailable_non_windows\",\"authority_eligible\":false}}");
        std::process::exit(2);
    }
}
