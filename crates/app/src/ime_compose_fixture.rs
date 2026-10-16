//! Explicit default-OFF owner UI-thread synthetic IMM composition fixture.
//! Fixed 한글 only; no external strings/files, environment commands, forged WM_IME,
//! global keyboard/layout changes. Owner context open-state changes are restored.
use std::{
    collections::VecDeque,
    ffi::OsString,
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU32, Ordering, fence},
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
                CPS_CANCEL, CPS_COMPLETE, GCS_COMPSTR, GCS_RESULTSTR, ImmGetCompositionStringW,
                ImmGetContext, ImmGetOpenStatus, ImmIsIME, ImmNotifyIME, ImmReleaseContext,
                ImmSetCompositionStringW, ImmSetOpenStatus, NI_COMPOSITIONSTR, SCS_SETSTR,
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
    io::Error::other("owner IME compose fixture rejected")
}
pub(crate) struct ComposeFixture {
    request: File,
    ack: File,
    _mapping: File,
    view: usize,
    state: Mutex<State>,
    trace: Mutex<Trace>,
    trace_dropped: AtomicU32,
}
impl Drop for ComposeFixture {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.view as *mut _,
            });
        }
    }
}
static CONFIGURED: OnceLock<Arc<ComposeFixture>> = OnceLock::new();
pub(crate) fn configured() -> Option<Arc<ComposeFixture>> {
    CONFIGURED.get().cloned()
}
pub(crate) fn initialize(arguments: Vec<OsString>) -> io::Result<()> {
    let positions: Vec<_> = arguments
        .iter()
        .enumerate()
        .filter(|(_, v)| v.as_os_str() == "--fixture-ime-compose")
        .map(|(i, _)| i)
        .collect();
    if positions.is_empty() {
        return Ok(());
    }
    if positions.len() != 1
        || !arguments.iter().any(|v| v == "--fixture")
        || arguments.len() != positions[0] + 5
    {
        return Err(rejected());
    }
    let start = positions[0] + 1;
    let mode = match arguments[start + 3].to_str() {
        Some("cancel") => Mode::Cancel,
        Some("commit") => Mode::Commit,
        _ => return Err(rejected()),
    };
    let value = ComposeFixture::from_inherited(&arguments[start..start + 3], mode)?;
    CONFIGURED.set(Arc::new(value)).map_err(|_| rejected())
}
impl ComposeFixture {
    fn from_inherited(arguments: &[OsString], mode: Mode) -> io::Result<Self> {
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
            state: Mutex::new(State::new(mode)),
            trace: Mutex::new(Trace::default()),
            trace_dropped: AtomicU32::new(0),
        };
        if unsafe { std::slice::from_raw_parts(view.Value.cast::<u8>(), PAGE_BYTES) }
            .iter()
            .any(|v| *v != 0)
        {
            return Err(rejected());
        }
        Ok(value)
    }
    fn take_request(&self) -> io::Result<bool> {
        match unsafe { WaitForSingleObject(self.request.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(false),
            WAIT_OBJECT_0 => {
                if unsafe { ResetEvent(self.request.as_raw_handle()) } == 0 {
                    return Err(rejected());
                }
                Ok(true)
            }
            _ => Err(rejected()),
        }
    }
    fn publish(&self, payload: &[u8]) -> io::Result<()> {
        if payload.len() > PAYLOAD_BYTES {
            return Err(rejected());
        }
        let mut row: serde_json::Value = serde_json::from_slice(payload).map_err(|_| rejected())?;
        let trace = self.trace.try_lock().map_err(|_| rejected())?;
        row["native_ime_trace"] = serde_json::Value::Array(trace.events.iter().cloned().collect());
        let dropped = self.trace_dropped.load(Ordering::Relaxed);
        row["native_ime_trace_dropped"] = dropped.into();
        row["native_ime_trace_overflow"] = (dropped > 0).into();
        let payload = serde_json::to_vec(&row).map_err(|_| rejected())?;
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
    fn frame(&self, window: &winit::window::Window, closing: bool) -> io::Result<()> {
        let mut state = self.state.try_lock().map_err(|_| rejected())?;
        let hwnd = owner_window(window)?;
        if state.skip_frame(closing) {
            return Ok(());
        }
        if closing {
            let row = restore(&mut state, hwnd, "close-restoration");
            state.phase = Phase::Done;
            return self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?);
        }
        if state.phase != Phase::WaitingStart
            && state.started.elapsed() > std::time::Duration::from_secs(5)
        {
            let row = restore(&mut state, hwnd, "phase-timeout");
            state.phase = Phase::Done;
            return self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?);
        }
        match state.phase {
            Phase::WaitingStart if self.take_request()? => {
                state.started = std::time::Instant::now();
                let result = start(&mut state, hwnd);
                if let Err(code) = result {
                    let row = restore(&mut state, hwnd, code);
                    state.phase = Phase::Done;
                    self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
                } else {
                    state.phase = Phase::ObservePreedit;
                }
            }
            Phase::ObservePreedit => {
                let mut row = base(hwnd, 1, "preedit");
                match context_strings(&state, hwnd) {
                    Ok((comp, result, released))
                        if comp == "한글" && result.is_empty() && released =>
                    {
                        row["status"] = "PASS".into();
                        row["original_open_status"] = state.original_open.into();
                        row["open_status_change_attempted"] = state.opened.into();
                        row["composition"] = comp.into();
                        row["result"] = result.into();
                        row["context_released"] = released.into();
                        state.phase = Phase::WaitingFinish;
                        state.started = std::time::Instant::now();
                        self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
                    }
                    _ => {
                        let row = restore(&mut state, hwnd, "preedit-not-observed");
                        state.phase = Phase::Done;
                        self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
                    }
                }
            }
            Phase::WaitingFinish if self.take_request()? => {
                state.started = std::time::Instant::now();
                let context = unsafe { ImmGetContext(hwnd) };
                if context.is_null() || context as usize != state.context {
                    if !context.is_null() {
                        unsafe {
                            ImmReleaseContext(hwnd, context);
                        }
                    }
                    let row = restore(&mut state, hwnd, "context-changed");
                    state.phase = Phase::Done;
                    self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
                } else {
                    let code = if state.mode == Mode::Cancel {
                        CPS_CANCEL
                    } else {
                        CPS_COMPLETE
                    };
                    let ok = unsafe { ImmNotifyIME(context, NI_COMPOSITIONSTR, code, 0) } != 0;
                    let released = unsafe { ImmReleaseContext(hwnd, context) } != 0;
                    if ok && released {
                        state.phase = Phase::ObserveTerminal;
                    } else {
                        let row = restore(&mut state, hwnd, "finish-api-failed");
                        state.phase = Phase::Done;
                        self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
                    }
                }
            }
            Phase::ObserveTerminal => {
                // Native messages have traversed a subsequent real event-loop frame.
                // Observer checks committed TextEdit/PNG; this probe does not forge events.
                let mut row = restore(&mut state, hwnd, "terminal");
                row["seq"] = 2.into();
                row["finish_api_succeeded"] = true.into();
                row["phase"] = "terminal".into();
                if row["open_status_restored"] == true
                    && row["composition_cancelled"] == true
                    && row["context_released"] == true
                {
                    row["status"] = "PASS".into();
                    row.as_object_mut().unwrap().remove("error");
                }
                state.phase = Phase::Done;
                self.publish(&serde_json::to_vec(&row).map_err(|_| rejected())?)?;
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Default)]
struct Trace {
    events: VecDeque<serde_json::Value>,
    sequence: u32,
}
impl ComposeFixture {
    fn record_native(&self, event: &winit::event::Ime) {
        let Ok(mut trace) = self.trace.try_lock() else {
            let _ = self
                .trace_dropped
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    Some(v.saturating_add(1))
                });
            return;
        };
        trace.sequence = trace.sequence.saturating_add(1);
        let mut row = serde_json::json!({"sequence":trace.sequence});
        match event {
            winit::event::Ime::Enabled => row["kind"] = "Enabled".into(),
            winit::event::Ime::Disabled => row["kind"] = "Disabled".into(),
            winit::event::Ime::Preedit(text, cursor) => {
                row["kind"] = "Preedit".into();
                row["text_bytes"] = text.len().into();
                row["prefix"] = text.chars().take(32).collect::<String>().into();
                row["cursor"] = serde_json::to_value(cursor).unwrap_or(serde_json::Value::Null);
            }
            winit::event::Ime::Commit(text) => {
                row["kind"] = "Commit".into();
                row["text_bytes"] = text.len().into();
                row["prefix"] = text.chars().take(32).collect::<String>().into();
            }
        }
        if trace.events.len() == 16 {
            trace.events.pop_front();
            let _ = self
                .trace_dropped
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    Some(v.saturating_add(1))
                });
        }
        trace.events.push_back(row);
    }
}
/// Root calls for the actual owned-window WindowEvent::Ime BEFORE input bridge.
/// Separate mutex permits capturing synchronous IMM callbacks while phase lock held.
pub(crate) fn on_native_ime(event: &winit::event::Ime) {
    if let Some(fixture) = configured() {
        fixture.record_native(event);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Cancel,
    Commit,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingStart,
    ObservePreedit,
    WaitingFinish,
    ObserveTerminal,
    Done,
}
struct State {
    mode: Mode,
    phase: Phase,
    context: usize,
    original_open: bool,
    opened: bool,
    mutation_attempted: bool,
    restoration_done: bool,
    started: std::time::Instant,
}
impl State {
    fn skip_frame(&self, closing: bool) -> bool {
        (self.phase == Phase::Done && !closing) || (closing && self.restoration_done)
    }
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            phase: Phase::WaitingStart,
            context: 0,
            original_open: false,
            opened: false,
            mutation_attempted: false,
            restoration_done: true,
            started: std::time::Instant::now(),
        }
    }
}
fn owner_window(
    window: &winit::window::Window,
) -> io::Result<windows_sys::Win32::Foundation::HWND> {
    let RawWindowHandle::Win32(native) = window.window_handle().map_err(|_| rejected())?.as_raw()
    else {
        return Err(rejected());
    };
    let hwnd = native.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    let mut pid = 0;
    if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } != unsafe { GetCurrentThreadId() }
        || pid != unsafe { GetCurrentProcessId() }
    {
        return Err(rejected());
    }
    Ok(hwnd)
}
fn base(hwnd: windows_sys::Win32::Foundation::HWND, seq: u32, phase: &str) -> serde_json::Value {
    serde_json::json!({"schema":1,"seq":seq,"phase":phase,"status":"QUERY_FAILED","process_id":unsafe{GetCurrentProcessId()},"thread_id":unsafe{GetCurrentThreadId()},"hwnd":hwnd as usize as u64,"actual_physical_keys":false})
}
fn string(
    context: windows_sys::Win32::UI::Input::Ime::HIMC,
    kind: u32,
) -> Result<String, &'static str> {
    let count = unsafe { ImmGetCompositionStringW(context, kind, std::ptr::null_mut(), 0) };
    if count == -1 || count == 0 {
        return Ok(String::new());
    }
    if !(0..=8192).contains(&count) || count % 2 != 0 {
        return Err("native-string-count");
    }
    let mut buffer = [0u16; 4096];
    let actual = unsafe {
        ImmGetCompositionStringW(context, kind, buffer.as_mut_ptr().cast(), count as u32)
    };
    if actual != count {
        return Err("native-string-changed");
    }
    let text = String::from_utf16(&buffer[..count as usize / 2]).map_err(|_| "invalid-utf16")?;
    if text.chars().count() > 32 {
        return Err("unexpected-synthetic-string");
    }
    Ok(text)
}
fn start(
    state: &mut State,
    hwnd: windows_sys::Win32::Foundation::HWND,
) -> Result<(), &'static str> {
    let layout = unsafe { GetKeyboardLayout(0) };
    if unsafe { ImmIsIME(layout) } == 0 {
        return Err("layout-not-ime");
    }
    let context = unsafe { ImmGetContext(hwnd) };
    if context.is_null() {
        return Err("context-unavailable");
    }
    state.context = context as usize;
    state.original_open = unsafe { ImmGetOpenStatus(context) } != 0;
    let attempt = (|| {
        if !string(context, GCS_COMPSTR)?.is_empty() || !string(context, GCS_RESULTSTR)?.is_empty()
        {
            return Err("existing-composition");
        }
        if !state.original_open {
            state.opened = true;
            state.restoration_done = false;
            if unsafe { ImmSetOpenStatus(context, 1) } == 0
                || unsafe { ImmGetOpenStatus(context) } == 0
            {
                return Err("open-api-failed");
            }
        }
        let fixed = [0xd55cu16, 0xae00];
        state.mutation_attempted = true;
        state.restoration_done = false;
        if unsafe {
            ImmSetCompositionStringW(
                context,
                SCS_SETSTR,
                fixed.as_ptr().cast(),
                4,
                std::ptr::null(),
                0,
            )
        } == 0
        {
            return Err("set-composition-failed");
        }
        Ok(())
    })();
    if unsafe { ImmReleaseContext(hwnd, context) } == 0 {
        return Err("context-release-failed");
    }
    attempt
}
fn context_strings(
    state: &State,
    hwnd: windows_sys::Win32::Foundation::HWND,
) -> Result<(String, String, bool), &'static str> {
    let context = unsafe { ImmGetContext(hwnd) };
    if context.is_null() {
        return Err("context-unavailable");
    }
    let result = if context as usize != state.context {
        Err("context-changed")
    } else {
        string(context, GCS_COMPSTR)
            .and_then(|comp| string(context, GCS_RESULTSTR).map(|result| (comp, result)))
    };
    let released = unsafe { ImmReleaseContext(hwnd, context) } != 0;
    result.map(|(comp, result)| (comp, result, released))
}
fn restore(
    state: &mut State,
    hwnd: windows_sys::Win32::Foundation::HWND,
    reason: &str,
) -> serde_json::Value {
    let seq = if state.phase == Phase::WaitingStart || state.phase == Phase::ObservePreedit {
        1
    } else {
        2
    };
    let mut row = base(hwnd, seq, "failed");
    row["error"] = reason.into();
    row["original_open_status"] = state.original_open.into();
    row["open_status_restored"] = false.into();
    row["composition_cancelled"] = false.into();
    row["context_released"] = false.into();
    let context = unsafe { ImmGetContext(hwnd) };
    if context.is_null() {
        return row;
    }
    if state.context != 0 && context as usize == state.context {
        let before = string(context, GCS_COMPSTR);
        let needed = state.mutation_attempted && before.as_ref().is_ok_and(|v| !v.is_empty());
        row["cancellation_needed"] = needed.into();
        let cancelled = if needed {
            let result = unsafe { ImmNotifyIME(context, NI_COMPOSITIONSTR, CPS_CANCEL, 0) } != 0;
            row["cancellation_api_result"] = result.into();
            result
        } else {
            row["cancellation_api_result"] = serde_json::Value::Null;
            true
        };
        let cleared = if state.mutation_attempted {
            string(context, GCS_COMPSTR).is_ok_and(|v| v.is_empty())
        } else {
            true
        };
        let restored = if state.opened {
            (unsafe { ImmSetOpenStatus(context, state.original_open as i32) }) != 0
        } else {
            true
        };
        let final_open = unsafe { ImmGetOpenStatus(context) } != 0;
        row["composition_cancelled"] = (cancelled && cleared).into();
        row["open_status_restored"] = (restored && final_open == state.original_open).into();
        row["final_open_status"] = final_open.into();
    }
    row["context_released"] = (unsafe { ImmReleaseContext(hwnd, context) } != 0).into();
    state.restoration_done = row["composition_cancelled"] == true
        && row["open_status_restored"] == true
        && row["context_released"] == true;
    row
}
pub(crate) fn on_frame(window: &winit::window::Window) -> io::Result<()> {
    if let Some(fixture) = configured() {
        fixture.frame(window, false)?;
    }
    Ok(())
}
pub(crate) fn restore_on_close(window: &winit::window::Window) -> io::Result<()> {
    if let Some(fixture) = configured() {
        fixture.frame(window, true)?;
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
    fn dup(file: &File) -> OsString {
        let mut h = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    file.as_raw_handle(),
                    GetCurrentProcess(),
                    &mut h,
                    0,
                    1,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        let f = unsafe { File::from_raw_handle(h) };
        OsString::from((f.into_raw_handle() as usize).to_string())
    }
    #[test]
    fn original_event_requests_reset_and_two_phase_mapping_reuses_exact_caps() {
        for mode in [Mode::Cancel, Mode::Commit] {
            let original = [event(), event(), mapping()];
            let args: Vec<_> = original.iter().map(dup).collect();
            let fixture = ComposeFixture::from_inherited(&args, mode).unwrap();
            assert!(!fixture.take_request().unwrap());
            for (seq, phase) in [(1, "preedit"), (2, "terminal")] {
                assert_ne!(unsafe { SetEvent(original[0].as_raw_handle()) }, 0);
                assert!(fixture.take_request().unwrap());
                assert!(!fixture.take_request().unwrap());
                let payload =
                    serde_json::to_vec(&serde_json::json!({"seq":seq,"phase":phase})).unwrap();
                fixture.publish(&payload).unwrap();
                assert_eq!(
                    unsafe { WaitForSingleObject(original[1].as_raw_handle(), 0) },
                    WAIT_OBJECT_0
                );
                let view = unsafe {
                    MapViewOfFile(original[2].as_raw_handle(), FILE_MAP_READ, 0, 0, PAGE_BYTES)
                };
                assert!(!view.Value.is_null());
                let len = unsafe { std::ptr::read_volatile(view.Value.cast::<u32>()) } as usize;
                let published: serde_json::Value = serde_json::from_slice(unsafe {
                    std::slice::from_raw_parts(view.Value.cast::<u8>().add(8), len)
                })
                .unwrap();
                assert_eq!(published["seq"], seq);
                assert_eq!(published["phase"], phase);
                assert_eq!(published["native_ime_trace"], serde_json::json!([]));
                assert_eq!(published["native_ime_trace_dropped"], 0);
                assert_eq!(published["native_ime_trace_overflow"], false);
                unsafe {
                    UnmapViewOfFile(view);
                }
                assert_ne!(unsafe { ResetEvent(original[1].as_raw_handle()) }, 0);
            }
        }
    }
    #[test]
    fn native_payload_limit_fails_without_ack_or_overwriting_original_zero_mapping() {
        let original = [event(), event(), mapping()];
        let args: Vec<_> = original.iter().map(dup).collect();
        let fixture = ComposeFixture::from_inherited(&args, Mode::Cancel).unwrap();
        assert!(fixture.publish(&vec![1; PAYLOAD_BYTES + 1]).is_err());
        assert_eq!(
            unsafe { WaitForSingleObject(original[1].as_raw_handle(), 0) },
            WAIT_TIMEOUT
        );
        let view =
            unsafe { MapViewOfFile(original[2].as_raw_handle(), FILE_MAP_READ, 0, 0, PAGE_BYTES) };
        assert!(!view.Value.is_null());
        assert!(
            unsafe { std::slice::from_raw_parts(view.Value.cast::<u8>(), PAGE_BYTES) }
                .iter()
                .all(|v| *v == 0)
        );
        unsafe {
            UnmapViewOfFile(view);
        }
    }
    fn handle_open(handle: HANDLE) -> bool {
        let mut flags = 0;
        (unsafe { GetHandleInformation(handle, &mut flags) }) != 0
    }
    fn raw_argument(copy: OsString) -> HANDLE {
        copy.to_str().unwrap().parse::<usize>().unwrap() as HANDLE
    }
    #[test]
    fn alias_invalid_text_and_wrong_mapping_close_accepted_copies_preserve_originals() {
        for case in 0..3 {
            let request = event();
            let ack = event();
            let first = dup(&request);
            let one = raw_argument(first.clone());
            if case == 0 {
                assert!(
                    ComposeFixture::from_inherited(
                        &[first.clone(), first, OsString::from("0")],
                        Mode::Cancel
                    )
                    .is_err()
                );
            } else if case == 1 {
                assert!(
                    ComposeFixture::from_inherited(
                        &[first, OsString::from("invalid"), OsString::from("0")],
                        Mode::Cancel
                    )
                    .is_err()
                );
            } else {
                let second = dup(&ack);
                let two = raw_argument(second.clone());
                let third = dup(&request);
                let three = raw_argument(third.clone());
                assert!(
                    ComposeFixture::from_inherited(&[first, second, third], Mode::Commit).is_err()
                );
                assert!(!handle_open(two));
                assert!(!handle_open(three));
            }
            assert!(!handle_open(one));
            assert!(handle_open(request.as_raw_handle()));
            assert!(handle_open(ack.as_raw_handle()));
        }
    }
    #[test]
    fn initialized_mapping_rejected_without_adopting_or_clearing_observer_data() {
        let original = [event(), event(), mapping()];
        let view = unsafe {
            MapViewOfFile(
                original[2].as_raw_handle(),
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
        let args: Vec<_> = original.iter().map(dup).collect();
        let copies: Vec<_> = args.iter().cloned().map(raw_argument).collect();
        assert!(ComposeFixture::from_inherited(&args, Mode::Commit).is_err());
        for copy in copies {
            assert!(!handle_open(copy));
        }
        assert_eq!(
            unsafe { std::ptr::read_volatile(view.Value.cast::<u8>().add(99)) },
            7
        );
        unsafe {
            UnmapViewOfFile(view);
        }
        for file in original {
            assert!(handle_open(file.as_raw_handle()));
        }
    }
    #[test]
    fn failed_terminal_restoration_keeps_close_retry_eligible_without_replaying_mutation() {
        for mode in [Mode::Cancel, Mode::Commit] {
            let mut state = State::new(mode);
            assert!(!state.skip_frame(false));
            assert!(state.skip_frame(true));
            state.phase = Phase::ObservePreedit;
            state.mutation_attempted = true;
            state.restoration_done = false;
            assert!(!state.skip_frame(false));
            assert!(!state.skip_frame(true));
            state.phase = Phase::Done; // Error response delivered but restoration failed.
            assert!(state.skip_frame(false));
            assert!(!state.skip_frame(true));
            state.restoration_done = true; // Only actual successful native restore sets this.
            assert!(state.skip_frame(true));
        }
        // Pure control-state contract; native HIMC/open/cancel restoration is GUI-only.
    }
    #[test]
    fn trace_is_independent_of_phase_lock_bounded_and_reports_overflow() {
        let original = [event(), event(), mapping()];
        let args: Vec<_> = original.iter().map(dup).collect();
        let fixture = ComposeFixture::from_inherited(&args, Mode::Cancel).unwrap();
        let phase = fixture.state.lock().unwrap();
        fixture.record_native(&winit::event::Ime::Preedit("한글".repeat(40), Some((0, 6))));
        {
            let trace = fixture.trace.lock().unwrap();
            assert_eq!(
                trace.events[0]["prefix"].as_str().unwrap().chars().count(),
                32
            );
            assert_eq!(trace.events[0]["text_bytes"], 240);
        }
        for _ in 0..20 {
            fixture.record_native(&winit::event::Ime::Disabled);
        }
        assert_eq!(fixture.trace.lock().unwrap().events.len(), 16);
        assert_eq!(fixture.trace_dropped.load(Ordering::Relaxed), 5);
        drop(phase);
    }
}
