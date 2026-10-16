//! Asynchronous native folder dialog using a worker-owned proxy window.
//! Integrator must retain Arc<Window> and keep its HWND alive until completion ACK.
use super::native_dialog::{FolderPicker, FolderPickerCloser, PickerError};
use eframe::egui;
use std::{
    cell::RefCell,
    io,
    path::PathBuf,
    ptr,
    sync::{
        Arc, Mutex, TryLockError,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{
        LibraryLoader::GetModuleHandleW,
        Threading::{GetCurrentProcessId, GetCurrentThreadId},
    },
    UI::WindowsAndMessaging::*,
};
// Runs only on the transient STA worker: an editable draft is a hint, never authority.
fn initial_file_hint(initial: Option<&std::path::Path>) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStrExt;
    let path = initial?;
    let text = path.to_str()?;
    if text.is_empty()
        || text.len() > 32768
        || text.chars().any(char::is_control)
        || path.as_os_str().encode_wide().count() > 32767
    {
        return None;
    }
    let units: Vec<u16> = path.as_os_str().encode_wide().collect();
    super::native_dialog::local_path(&units).ok()?;
    path.file_name()?;
    let parent = path.parent()?;
    if !std::fs::metadata(parent).ok()?.is_dir() {
        return None;
    }
    Some(path.to_owned())
}
const CANCEL: u32 = WM_APP + 0x327;
thread_local! { static CLOSER: RefCell<Option<FolderPickerCloser>> = const { RefCell::new(None) }; }
static NEXT_NONCE: AtomicUsize = AtomicUsize::new(1);
struct Life {
    hwnd: usize,
    thread: u32,
    nonce: usize,
}
struct Signals {
    cancelled: AtomicBool,
    life: Mutex<Life>,
    cancel_error: AtomicBool,
    destroy_error: AtomicBool,
}
thread_local! { static NONCE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
thread_local! { static ERRORS: RefCell<Option<Arc<Signals>>> = const { RefCell::new(None) }; }
pub struct AsyncFolderPicker {
    signals: Arc<Signals>,
    completion: Receiver<Result<Option<PathBuf>, String>>,
}
impl AsyncFolderPicker {
    #[cfg(test)]
    pub fn completed_for_test(path: PathBuf) -> Self {
        let (tx, completion) = mpsc::sync_channel(1);
        tx.try_send(Ok(Some(path))).unwrap();
        Self {
            signals: Arc::new(Signals {
                cancelled: AtomicBool::new(false),
                life: Mutex::new(Life {
                    hwnd: 0,
                    thread: 0,
                    nonce: 1,
                }),
                cancel_error: AtomicBool::new(false),
                destroy_error: AtomicBool::new(false),
            }),
            completion,
        }
    }
    // Called on original owner UI thread. No Rust application borrow crosses worker callbacks.
    pub fn start(
        window: Arc<winit::window::Window>,
        initial: Option<PathBuf>,
        title: String,
        ctx: egui::Context,
    ) -> io::Result<Self> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let RawWindowHandle::Win32(raw) = window
            .window_handle()
            .map_err(|_| io::Error::other("picker HWND"))?
            .as_raw()
        else {
            return Err(io::Error::other("picker HWND"));
        };
        let owner = raw.hwnd.get() as HWND;
        let mut pid = 0;
        // SAFETY: explicit caller HWND queried only; original UI retains its lifetime until ACK.
        let owner_thread = unsafe { GetWindowThreadProcessId(owner, &mut pid) };
        if owner.is_null()
            || owner_thread != unsafe { GetCurrentThreadId() }
            || pid != unsafe { GetCurrentProcessId() }
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "picker owner"));
        }
        if title.encode_utf16().take(32768).count() > 32767 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "picker title"));
        }
        let nonce = NEXT_NONCE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_add(1))
            .map_err(|_| io::Error::other("picker nonce exhausted"))?;
        let signals = Arc::new(Signals {
            cancelled: AtomicBool::new(false),
            life: Mutex::new(Life {
                hwnd: 0,
                thread: 0,
                nonce,
            }),
            cancel_error: AtomicBool::new(false),
            destroy_error: AtomicBool::new(false),
        });
        let (tx, completion) = mpsc::sync_channel(1);
        let worker = signals.clone();
        let owner_value = owner as usize;
        thread::Builder::new()
            .name("dlpad-tool-executable-picker".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(
                        owner_value as HWND,
                        owner_thread,
                        pid,
                        initial,
                        title,
                        &worker,
                    )
                }))
                .unwrap_or(Err(PickerError::InvalidResult))
                .map_err(|e| e.to_string());
                let result = if worker.cancel_error.load(Ordering::Acquire)
                    || worker.destroy_error.load(Ordering::Acquire)
                {
                    Err("picker lifecycle failure".into())
                } else {
                    result
                };
                // run RAII has released TLS closer/COM/proxy before this point. Keep owner
                // alive through all native cleanup, then release on worker before ACK.
                drop(window);
                let _ = tx.try_send(result);
                ctx.request_repaint();
            })?;
        Ok(Self {
            signals,
            completion,
        })
    }
    pub fn cancel(&self) -> io::Result<()> {
        self.signals.cancelled.store(true, Ordering::Release);
        match self.signals.life.try_lock() {
            Ok(life) => {
                if life.hwnd != 0 {
                    let mut pid = 0;
                    // SAFETY: life gate prevents proxy destruction/reuse while queried/posted.
                    let thread = unsafe { GetWindowThreadProcessId(life.hwnd as HWND, &mut pid) };
                    if thread != life.thread
                        || pid != unsafe { GetCurrentProcessId() }
                        || unsafe { PostMessageW(life.hwnd as HWND, CANCEL, life.nonce, 0) } == 0
                    {
                        self.signals.cancel_error.store(true, Ordering::Release);
                        return Err(io::Error::other("picker cancel posting"));
                    }
                }
            }
            Err(TryLockError::WouldBlock) => {} // Publication checks flag; cleanup already ends dialog.
            Err(TryLockError::Poisoned(_)) => {
                self.signals.cancel_error.store(true, Ordering::Release);
                return Err(io::Error::other("picker life poisoned"));
            }
        }
        Ok(())
    }
    pub fn try_complete(&self) -> Result<Option<Result<Option<PathBuf>, String>>, TryRecvError> {
        match self.completion.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
impl Drop for AsyncFolderPicker {
    fn drop(&mut self) {
        let _ = self.cancel();
    } // Not an ACK or proof of completion.
}
unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // Never unwind across the Win32 callback ABI.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dispatch(hwnd, message, w, l)
    }))
    .unwrap_or_else(|_| {
        ERRORS.with(|e| {
            if let Some(s) = e.borrow().as_ref() {
                s.cancel_error.store(true, Ordering::Release);
            }
        });
        0
    })
}
fn dispatch(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if message == CANCEL && NONCE.with(|n| n.get() == w && w != 0) {
        // Rc clone performs no COM; borrow ends BEFORE callback-capable COM cancellation.
        let closer = CLOSER.with(|c| c.borrow().clone());
        if let Some(closer) = closer
            && closer.cancel().is_err()
        {
            ERRORS.with(|e| {
                if let Some(s) = e.borrow().as_ref() {
                    s.cancel_error.store(true, Ordering::Release);
                }
            });
        }
        return 0;
    }
    // SAFETY: OS-delivered original proxy message forwarded to native default procedure.
    unsafe { DefWindowProcW(hwnd, message, w, l) }
}
struct Proxy<'a> {
    hwnd: HWND,
    signals: &'a Signals,
}
impl Drop for Proxy<'_> {
    fn drop(&mut self) {
        // No COM under this gate. Recover poison for original native cleanup only,
        // preserving sticky error; no success inferred from poison recovery.
        let mut life = self.signals.life.lock().unwrap_or_else(|e| {
            self.signals.destroy_error.store(true, Ordering::Release);
            e.into_inner()
        });
        life.hwnd = 0;
        // SAFETY: original worker HWND, serialized against all cancel posts.
        if unsafe { DestroyWindow(self.hwnd) } == 0 {
            self.signals.destroy_error.store(true, Ordering::Release);
        }
    }
}
fn register_proxy_class() -> Result<(windows_sys::Win32::Foundation::HMODULE, Vec<u16>), PickerError>
{
    let name: Vec<u16> = "DlpadToolExecutablePickerProxy\0".encode_utf16().collect();
    // SAFETY: current module query only; fixed class registration with static procedure.
    let module = unsafe { GetModuleHandleW(ptr::null()) };
    if module.is_null() {
        return Err(PickerError::InvalidResult);
    }
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: module,
        lpszClassName: name.as_ptr(),
        ..unsafe { std::mem::zeroed() }
    };
    // Multiple sequential adapters can reuse only our module-local registered class.
    if unsafe { RegisterClassW(&class) } == 0 {
        let mut existing: WNDCLASSW = unsafe { std::mem::zeroed() };
        if unsafe { GetClassInfoW(module, name.as_ptr(), &mut existing) } == 0
            || existing.lpfnWndProc.map(|f| f as usize) != Some(window_proc as *const () as usize)
            || existing.hInstance != module
        {
            return Err(PickerError::InvalidResult);
        }
    }
    Ok((module, name))
}
#[cfg(test)]
pub(super) fn registered_proxy_identity_for_test() -> Result<(usize, usize), String> {
    let (module, name) = register_proxy_class().map_err(|error| error.to_string())?;
    let mut native: WNDCLASSW = unsafe { std::mem::zeroed() };
    if unsafe { GetClassInfoW(module, name.as_ptr(), &mut native) } == 0
        || native.hInstance != module
        || native.lpfnWndProc.map(|f| f as usize) != Some(window_proc as *const () as usize)
    {
        return Err("native class no longer has original module/procedure".into());
    }
    Ok((
        native.hInstance as usize,
        native.lpfnWndProc.unwrap() as usize,
    ))
}
fn run(
    owner: HWND,
    owner_thread: u32,
    expected_pid: u32,
    initial: Option<PathBuf>,
    title: String,
    signals: &Arc<Signals>,
) -> Result<Option<PathBuf>, PickerError> {
    let mut pid = 0;
    // SAFETY: original owner is pinned by UI keepalive contract and checked before proxy construction.
    if unsafe { GetWindowThreadProcessId(owner, &mut pid) } != owner_thread || pid != expected_pid {
        return Err(PickerError::WrongOwner);
    }
    let (module, name) = register_proxy_class()?;
    // Hidden owned top-level proxy; NO ShowWindow/foreground/global input. COM may disable proxy.
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            name.as_ptr(),
            ptr::null(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            owner,
            ptr::null_mut(),
            module,
            ptr::null(),
        )
    };
    if hwnd.is_null() {
        return Err(PickerError::InvalidResult);
    }
    let proxy = Proxy { hwnd, signals };
    if unsafe { GetWindow(hwnd, GW_OWNER) } != owner {
        return Err(PickerError::WrongOwner);
    }
    let initial = initial_file_hint(initial.as_deref());
    let picker = FolderPicker::new(initial.as_deref(), &title)?;
    let closer = picker.closer();
    CLOSER.with(|c| *c.borrow_mut() = Some(closer));
    struct TlsGuard;
    impl Drop for TlsGuard {
        fn drop(&mut self) {
            let closer = CLOSER.with(|c| c.borrow_mut().take());
            drop(closer); // Callback-capable Release outside TLS borrow.
            NONCE.with(|n| n.set(0));
            ERRORS.with(|e| {
                let prior = e.borrow_mut().take();
                drop(prior);
            });
        }
    }
    let tls = TlsGuard;
    ERRORS.with(|e| *e.borrow_mut() = Some(signals.clone()));
    {
        let mut life = signals
            .life
            .lock()
            .map_err(|_| PickerError::InvalidResult)?;
        life.hwnd = hwnd as usize;
        life.thread = unsafe { GetCurrentThreadId() };
        NONCE.with(|n| n.set(life.nonce));
    }
    let result = if signals.cancelled.load(Ordering::Acquire) {
        Ok(None)
    } else {
        // A cancel racing this check remains queued to the proxy; Show pumps native messages.
        picker.show(hwnd)
    };
    drop(tls);
    drop(picker); // Releases COM and balances STA before proxy destruction.
    drop(proxy);
    result
}

#[cfg(test)]
mod file_hint_tests {
    use super::*;
    #[test]
    fn relative_missing_parent_and_control_hints_fall_back_without_draft_adoption() {
        assert!(initial_file_hint(Some(std::path::Path::new("relative.exe"))).is_none());
        let missing = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("missing-hint-parent-{}", std::process::id()))
            .join("tool.exe");
        assert!(!missing.parent().unwrap().exists());
        assert!(initial_file_hint(Some(&missing)).is_none());
        assert!(initial_file_hint(Some(std::path::Path::new("C:\\bad\n.exe"))).is_none());
        assert!(initial_file_hint(Some(std::path::Path::new(&"x".repeat(32769)))).is_none());
    }
    #[test]
    fn existing_parent_preserves_exact_unicode_filename_hint_without_tool_validation() {
        let path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("선택🙂-does-not-need-to-exist.exe");
        assert_eq!(initial_file_hint(Some(&path)), Some(path));
        assert!(initial_file_hint(None).is_none());
    }
}

#[cfg(test)]
mod proxy_registration_tests {
    use super::*;
    #[test]
    fn folder_and_executable_proxy_classes_keep_distinct_original_procedures() {
        let folder = crate::path_picker::registered_folder_proxy_identity_for_test().unwrap();
        let file = registered_proxy_identity_for_test().unwrap();
        assert_eq!(folder.0, file.0, "same original application module");
        assert_ne!(folder.1, file.1, "different real window procedures");
        assert_eq!(
            crate::path_picker::registered_folder_proxy_identity_for_test().unwrap(),
            folder
        );
        assert_eq!(registered_proxy_identity_for_test().unwrap(), file);
    }
}
