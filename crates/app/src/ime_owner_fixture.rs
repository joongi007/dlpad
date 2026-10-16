//! Explicit default-OFF owner UI-thread IME observation fixture.
//! No composition text, external files, environment switches, or IME mutations.
use std::{
    ffi::OsString,
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering, fence},
    },
};
use windows_sys::Win32::{
    Foundation::{
        GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
        SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::{
        Memory::{
            FILE_MAP_READ, FILE_MAP_WRITE, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
            UnmapViewOfFile,
        },
        Threading::{
            GetCurrentProcessId, GetCurrentThreadId, ResetEvent, SetEvent, WaitForSingleObject,
        },
    },
    UI::{
        Input::{
            Ime::{
                GCS_COMPSTR, GCS_RESULTSTR, ImmGetCompositionStringW, ImmGetContext,
                ImmGetOpenStatus, ImmIsIME, ImmReleaseContext,
            },
            KeyboardAndMouse::GetKeyboardLayout,
        },
        WindowsAndMessaging::GetWindowThreadProcessId,
    },
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
const PAGE_BYTES: usize = 4096;
const PAYLOAD_BYTES: usize = PAGE_BYTES - 8;
fn rejected() -> io::Error {
    io::Error::other("owner IME fixture rejected")
}
pub(crate) struct OwnerImeFixture {
    request: File,
    ack: File,
    _mapping: File,
    view: usize,
    entered: AtomicBool,
}
impl Drop for OwnerImeFixture {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.view as *mut _,
            });
        }
    }
}
static CONFIGURED: OnceLock<Arc<OwnerImeFixture>> = OnceLock::new();
pub(crate) fn configured() -> Option<Arc<OwnerImeFixture>> {
    CONFIGURED.get().cloned()
}
pub(crate) fn initialize(arguments: Vec<OsString>) -> io::Result<()> {
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter(|(_, v)| v.as_os_str() == "--fixture-ime-owner")
        .map(|(i, _)| i)
        .collect();
    if positions.is_empty() {
        return Ok(());
    }
    if positions.len() != 1
        || !arguments.iter().any(|v| v == "--fixture")
        || arguments.len() != positions[0] + 4
    {
        return Err(rejected());
    }
    let value = OwnerImeFixture::from_inherited(&arguments[positions[0] + 1..])?;
    CONFIGURED.set(Arc::new(value)).map_err(|_| rejected())
}
impl OwnerImeFixture {
    pub(crate) fn from_inherited(arguments: &[OsString]) -> io::Result<Self> {
        if arguments.len() != 3 {
            return Err(rejected());
        }
        let mut raw = Vec::<HANDLE>::with_capacity(3);
        let mut files = Vec::<File>::with_capacity(3);
        for (index, value) in arguments.iter().enumerate() {
            let text = value.to_str().ok_or_else(rejected)?;
            if text.is_empty() || text.len() > 20 || !text.bytes().all(|v| v.is_ascii_digit()) {
                return Err(rejected());
            }
            let h = text.parse::<usize>().map_err(|_| rejected())? as HANDLE;
            if h.is_null() || h == INVALID_HANDLE_VALUE || raw.contains(&h) {
                return Err(rejected());
            }
            let mut flags = 0;
            if unsafe { GetHandleInformation(h, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT == 0
            {
                return Err(rejected());
            }
            raw.push(h);
            files.push(unsafe { File::from_raw_handle(h) });
            if unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0) } == 0 {
                return Err(rejected());
            }
            if index < 2 && unsafe { ResetEvent(h) } == 0 {
                return Err(rejected());
            }
        }
        let mapping = files.pop().ok_or_else(rejected)?;
        let view = unsafe {
            MapViewOfFile(
                mapping.as_raw_handle(),
                FILE_MAP_READ | FILE_MAP_WRITE,
                0,
                0,
                PAGE_BYTES,
            )
        };
        if view.Value.is_null() {
            return Err(rejected());
        }
        // Own the mapped view before checking contents so partial validation unmaps it.
        let value = Self {
            request: files.remove(0),
            ack: files.remove(0),
            _mapping: mapping,
            view: view.Value as usize,
            entered: AtomicBool::new(false),
        };
        if unsafe { std::slice::from_raw_parts(view.Value.cast::<u8>(), PAGE_BYTES) }
            .iter()
            .any(|v| *v != 0)
        {
            return Err(rejected());
        }
        Ok(value)
    }
    fn requested(&self) -> io::Result<bool> {
        if self.entered.load(Ordering::Acquire) {
            return Ok(false);
        }
        match unsafe { WaitForSingleObject(self.request.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(false),
            WAIT_OBJECT_0 => Ok(!self.entered.swap(true, Ordering::AcqRel)),
            _ => Err(rejected()),
        }
    }
    fn publish(&self, payload: &[u8]) -> io::Result<()> {
        if payload.len() > PAYLOAD_BYTES {
            return Err(rejected());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                payload.as_ptr(),
                (self.view as *mut u8).add(8),
                payload.len(),
            );
        }
        fence(Ordering::Release);
        unsafe {
            std::ptr::write_volatile(self.view as *mut u32, payload.len() as u32);
        }
        fence(Ordering::Release);
        if unsafe { SetEvent(self.ack.as_raw_handle()) } == 0 {
            return Err(rejected());
        }
        Ok(())
    }
    fn capture(&self, window: &winit::window::Window) -> io::Result<()> {
        if !self.requested()? {
            return Ok(());
        }
        let mut value = serde_json::json!({"schema":1,"phase":"ime-owner-readonly","status":"QUERY_FAILED","process_id":unsafe{GetCurrentProcessId()},"thread_id":unsafe{GetCurrentThreadId()},"error":"owner-query-unavailable"});
        let measured = (|| -> io::Result<serde_json::Value> {
            let RawWindowHandle::Win32(native) =
                window.window_handle().map_err(|_| rejected())?.as_raw()
            else {
                return Err(rejected());
            };
            let hwnd = native.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            let mut pid = 0;
            let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
            if pid != unsafe { GetCurrentProcessId() } || tid != unsafe { GetCurrentThreadId() } {
                return Err(rejected());
            }
            let layout = unsafe { GetKeyboardLayout(0) };
            let context = unsafe { ImmGetContext(hwnd) };
            let mut result = serde_json::json!({"schema":1,"phase":"ime-owner-readonly","status":"QUERY_FAILED","process_id":pid,"thread_id":tid,"hwnd":hwnd as usize as u64,"keyboard_layout":layout as usize as u64,"imm_is_ime":unsafe{ImmIsIME(layout)}!=0,"himc_present":!context.is_null(),"open_status":false,"composition_bytes":null,"result_bytes":null,"context_released":false,"error":"context-unavailable"});
            if context.is_null() {
                return Ok(result);
            }
            let open = unsafe { ImmGetOpenStatus(context) } != 0;
            let comp =
                unsafe { ImmGetCompositionStringW(context, GCS_COMPSTR, std::ptr::null_mut(), 0) };
            let committed = unsafe {
                ImmGetCompositionStringW(context, GCS_RESULTSTR, std::ptr::null_mut(), 0)
            };
            let released = unsafe { ImmReleaseContext(hwnd, context) } != 0;
            result["open_status"] = open.into();
            result["composition_query_code"] = comp.into();
            result["result_query_code"] = committed.into();
            result["context_released"] = released.into();
            if (0..=8192).contains(&comp) {
                result["composition_bytes"] = comp.into();
            }
            if (0..=8192).contains(&committed) {
                result["result_bytes"] = committed.into();
            }
            if released && (0..=8192).contains(&comp) && (0..=8192).contains(&committed) {
                result["status"] = "PASS".into();
                result.as_object_mut().unwrap().remove("error");
            } else {
                result["error"] = "context-query-or-release-unavailable".into();
            }
            Ok(result)
        })();
        if let Ok(result) = measured {
            value = result;
        }
        let bytes = serde_json::to_vec(&value).map_err(|_| rejected())?;
        self.publish(&bytes)
    }
}
/// Root calls after handle_platform_output on the real owning native UI thread.
pub(crate) fn on_frame(window: &winit::window::Window) -> io::Result<()> {
    if let Some(fixture) = configured() {
        fixture.capture(window)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::IntoRawHandle;
    use windows_sys::Win32::{
        Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle},
        System::{
            Memory::{CreateFileMappingW, PAGE_READWRITE},
            Threading::{CreateEventW, GetCurrentProcess},
        },
    };
    fn event() -> File {
        let h = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        assert!(!h.is_null());
        unsafe { File::from_raw_handle(h) }
    }
    fn mapping() -> File {
        let h = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                std::ptr::null(),
                PAGE_READWRITE,
                0,
                PAGE_BYTES as u32,
                std::ptr::null(),
            )
        };
        assert!(!h.is_null());
        unsafe { File::from_raw_handle(h) }
    }
    fn duplicate(original: &File) -> File {
        let mut copy = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    original.as_raw_handle(),
                    GetCurrentProcess(),
                    &mut copy,
                    0,
                    1,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        unsafe { File::from_raw_handle(copy) }
    }
    fn argument(copy: File) -> (OsString, HANDLE) {
        let raw = copy.into_raw_handle();
        (OsString::from((raw as usize).to_string()), raw)
    }
    fn open(raw: HANDLE) -> bool {
        let mut flags = 0;
        (unsafe { GetHandleInformation(raw, &mut flags) }) != 0
    }
    #[test]
    fn actual_three_caps_clear_inheritance_publish_bounded_original_mapping() {
        let originals = [event(), event(), mapping()];
        let args: Vec<_> = originals.iter().map(|f| argument(duplicate(f)).0).collect();
        let fixture = OwnerImeFixture::from_inherited(&args).unwrap();
        for f in [&fixture.request, &fixture.ack, &fixture._mapping] {
            let mut flags = 0;
            assert_ne!(
                unsafe { GetHandleInformation(f.as_raw_handle(), &mut flags) },
                0
            );
            assert_eq!(flags & HANDLE_FLAG_INHERIT, 0);
        }
        assert!(!fixture.requested().unwrap());
        unsafe {
            SetEvent(originals[0].as_raw_handle());
        }
        assert!(fixture.requested().unwrap());
        assert!(!fixture.requested().unwrap());
        assert!(fixture.publish(&vec![0; PAYLOAD_BYTES + 1]).is_err());
        assert_eq!(
            unsafe { WaitForSingleObject(originals[1].as_raw_handle(), 0) },
            WAIT_TIMEOUT
        );
        fixture.publish(b"{\"schema\":1}").unwrap();
        assert_eq!(
            unsafe { WaitForSingleObject(originals[1].as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
        let view = unsafe {
            MapViewOfFile(
                originals[2].as_raw_handle(),
                FILE_MAP_READ,
                0,
                0,
                PAGE_BYTES,
            )
        };
        assert!(!view.Value.is_null());
        assert_eq!(
            unsafe { std::ptr::read_volatile(view.Value.cast::<u32>()) },
            12
        );
        assert_eq!(
            unsafe { std::slice::from_raw_parts(view.Value.cast::<u8>().add(8), 12) },
            b"{\"schema\":1}"
        );
        unsafe {
            UnmapViewOfFile(view);
        }
        drop(fixture);
        for original in originals {
            assert!(open(original.as_raw_handle()));
        }
    }
    #[test]
    fn partial_duplicate_invalid_text_and_wrong_mapping_preserve_observer_caps() {
        for case in 0..3 {
            let request = event();
            let ack = event();
            let (first, one) = argument(duplicate(&request));
            if case == 0 {
                assert!(
                    OwnerImeFixture::from_inherited(&[first.clone(), first, OsString::from("0")])
                        .is_err()
                );
            } else if case == 1 {
                assert!(
                    OwnerImeFixture::from_inherited(&[
                        first,
                        OsString::from("invalid"),
                        OsString::from("0")
                    ])
                    .is_err()
                );
            } else {
                let (second, two) = argument(duplicate(&ack));
                let (third, three) = argument(duplicate(&request));
                assert!(OwnerImeFixture::from_inherited(&[first, second, third]).is_err());
                assert!(!open(two));
                assert!(!open(three));
            }
            assert!(!open(one));
            assert!(open(request.as_raw_handle()));
            assert!(open(ack.as_raw_handle()));
        }
    }
    #[test]
    fn nonzero_mapping_rejected_without_clearing_original_payload() {
        let originals = [event(), event(), mapping()];
        let view = unsafe {
            MapViewOfFile(
                originals[2].as_raw_handle(),
                FILE_MAP_READ | FILE_MAP_WRITE,
                0,
                0,
                PAGE_BYTES,
            )
        };
        assert!(!view.Value.is_null());
        unsafe {
            std::ptr::write_volatile(view.Value.cast::<u8>().add(99), 7);
        }
        let args: Vec<_> = originals.iter().map(|f| argument(duplicate(f)).0).collect();
        assert!(OwnerImeFixture::from_inherited(&args).is_err());
        assert_eq!(
            unsafe { std::ptr::read_volatile(view.Value.cast::<u8>().add(99)) },
            7
        );
        unsafe {
            UnmapViewOfFile(view);
        }
        for original in originals {
            assert!(open(original.as_raw_handle()));
        }
    }
}
