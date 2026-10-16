//! Default-OFF first successful present receipt; no claim of compositor scanout.
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
    UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindowVisible},
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
const PAGE_BYTES: usize = 4096;
const PAYLOAD_BYTES: usize = PAGE_BYTES - 8;
fn rejected() -> io::Error {
    io::Error::other("first paint fixture rejected")
}
pub(crate) struct FirstPaintFixture {
    request: File,
    ack: File,
    _mapping: File,
    view: usize,
    entered: AtomicBool,
    first: std::sync::Mutex<Option<Vec<u8>>>,
}
impl Drop for FirstPaintFixture {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.view as *mut _,
            });
        }
    }
}
static CONFIGURED: OnceLock<Arc<FirstPaintFixture>> = OnceLock::new();
pub(crate) fn configured() -> Option<Arc<FirstPaintFixture>> {
    CONFIGURED.get().cloned()
}
pub(crate) fn initialize(arguments: Vec<OsString>) -> io::Result<()> {
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter(|(_, v)| v.as_os_str() == "--fixture-first-paint")
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
    let value = FirstPaintFixture::from_inherited(&arguments[positions[0] + 1..])?;
    CONFIGURED.set(Arc::new(value)).map_err(|_| rejected())
}
impl FirstPaintFixture {
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
            first: std::sync::Mutex::new(None),
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
    fn latch(
        &self,
        window: &winit::window::Window,
        observation: FirstPresentObservation,
    ) -> io::Result<()> {
        let first = self.first.try_lock().map_err(|_| rejected())?;
        if first.is_some() {
            return Ok(());
        }
        drop(first);
        let RawWindowHandle::Win32(handle) =
            window.window_handle().map_err(|_| rejected())?.as_raw()
        else {
            return Err(rejected());
        };
        let mut pid = 0;
        let thread = unsafe { GetWindowThreadProcessId(handle.hwnd.get() as _, &mut pid) };
        if pid != unsafe { GetCurrentProcessId() } || thread != unsafe { GetCurrentThreadId() } {
            return Err(rejected());
        }
        // Caller invokes ONLY after successful original buffer.present and the
        // subsequent original initial-show operation. Visibility is native readback.
        let payload = serde_json::to_vec(&serde_json::json!({
            "version":1,"sequence":1,"pid":pid,"thread":thread,"hwnd":handle.hwnd.get(),
            "before_present_visible":observation.before_present_visible,
            "after_present_visible":observation.after_present_visible,
            "initial_show_performed":observation.initial_show_performed,
            "after_show_native_visible":unsafe {IsWindowVisible(handle.hwnd.get() as _)} != 0,
            "dark_mode":observation.dark_mode,"system_theme":observation.system_theme,
            "background_rgb":observation.background_rgb,"actual_pixel_rgb":observation.actual_pixel_rgb,
            "width":observation.width,"height":observation.height,
            "successful_original_present":true,"compositor_scanout_verified":false
        })).map_err(|_| rejected())?;
        if payload.len() > PAYLOAD_BYTES {
            return Err(rejected());
        }
        self.store_first(payload)
    }
    fn store_first(&self, payload: Vec<u8>) -> io::Result<()> {
        if payload.len() > PAYLOAD_BYTES {
            return Err(rejected());
        }
        let mut first = self.first.try_lock().map_err(|_| rejected())?;
        if first.is_none() {
            *first = Some(payload);
        }
        Ok(())
    }
    fn respond(&self) -> io::Result<()> {
        // No request consumption until a successful first frame is actually latched.
        let first = self.first.try_lock().map_err(|_| rejected())?;
        if let Some(payload) = first.as_ref()
            && self.requested()?
        {
            self.publish(payload)?;
        }
        Ok(())
    }
}
/// Fixed-size data derived from the SAME rendered buffer/context. No text/path input.
pub(crate) struct FirstPresentObservation {
    pub(crate) before_present_visible: bool,
    pub(crate) after_present_visible: bool,
    pub(crate) initial_show_performed: bool,
    pub(crate) dark_mode: bool,
    /// "Light", "Dark", or null from original native window.theme().
    pub(crate) system_theme: Option<&'static str>,
    pub(crate) background_rgb: [u8; 3],
    pub(crate) actual_pixel_rgb: [u8; 3],
    pub(crate) width: u32,
    pub(crate) height: u32,
}
pub(crate) fn after_first_present(
    window: &winit::window::Window,
    observation: FirstPresentObservation,
) -> io::Result<()> {
    if let Some(fixture) = configured() {
        fixture.latch(window, observation)?;
        fixture.respond()?;
    }
    Ok(())
}
pub(crate) fn on_frame() -> io::Result<()> {
    if let Some(fixture) = configured() {
        fixture.respond()?;
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
        let fixture = FirstPaintFixture::from_inherited(&args).unwrap();
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
        let owned_handles = [
            fixture.request.as_raw_handle(),
            fixture.ack.as_raw_handle(),
            fixture._mapping.as_raw_handle(),
        ];
        drop(fixture);
        for raw in owned_handles {
            assert!(!open(raw));
        }
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
                    FirstPaintFixture::from_inherited(&[first.clone(), first, OsString::from("0")])
                        .is_err()
                );
            } else if case == 1 {
                assert!(
                    FirstPaintFixture::from_inherited(&[
                        first,
                        OsString::from("invalid"),
                        OsString::from("0")
                    ])
                    .is_err()
                );
            } else {
                let (second, two) = argument(duplicate(&ack));
                let (third, three) = argument(duplicate(&request));
                assert!(FirstPaintFixture::from_inherited(&[first, second, third]).is_err());
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
        assert!(FirstPaintFixture::from_inherited(&args).is_err());
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
    #[test]
    fn request_not_consumed_before_original_first_payload_and_latch_is_one_shot() {
        let originals = [event(), event(), mapping()];
        let args: Vec<_> = originals.iter().map(|f| argument(duplicate(f)).0).collect();
        let fixture = FirstPaintFixture::from_inherited(&args).unwrap();
        unsafe {
            SetEvent(originals[0].as_raw_handle());
        }
        fixture.respond().unwrap();
        assert!(!fixture.entered.load(Ordering::Acquire));
        assert_eq!(
            unsafe { WaitForSingleObject(originals[1].as_raw_handle(), 0) },
            WAIT_TIMEOUT
        );
        fixture.store_first(b"{\"sequence\":1}".to_vec()).unwrap();
        fixture.store_first(b"{\"sequence\":2}".to_vec()).unwrap();
        fixture.respond().unwrap();
        assert!(fixture.entered.load(Ordering::Acquire));
        assert_eq!(
            fixture.first.lock().unwrap().as_ref().unwrap(),
            b"{\"sequence\":1}"
        );
        assert_eq!(
            unsafe { WaitForSingleObject(originals[1].as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
    }
    #[test]
    fn zero_and_noninheritable_rejected_without_adopting_original_observer_handle() {
        assert!(FirstPaintFixture::from_inherited(&["0".into(), "0".into(), "0".into()]).is_err());
        let original = event();
        let argument = OsString::from((original.as_raw_handle() as usize).to_string());
        assert!(FirstPaintFixture::from_inherited(&[argument, "0".into(), "0".into()]).is_err());
        assert!(open(original.as_raw_handle()));
    }
}
