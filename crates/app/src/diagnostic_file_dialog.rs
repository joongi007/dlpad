//! Same-STA diagnostic draft selection. No report export, overwrite or MPD parsing authority.
use std::{
    cell::Cell,
    ffi::c_void,
    fmt,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr,
    rc::Rc,
};
use windows_sys::{
    Win32::{
        Foundation::HWND,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
            },
            Threading::{GetCurrentProcessId, GetCurrentThreadId},
        },
        UI::{
            Shell::{IUnknown_GetWindow, SHCreateItemFromParsingName},
            WindowsAndMessaging::{
                GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindow, PostMessageW, WM_CLOSE,
            },
        },
    },
    core::GUID,
};
type Hr = i32;
const CANCELLED: Hr = 0x800704c7u32 as i32;
const MAX: usize = 32767;
const OPEN_CLASS: GUID = GUID {
    data1: 0xdc1c5a9c,
    data2: 0xe88a,
    data3: 0x4dde,
    data4: [0xa5, 0xa1, 0x60, 0xf8, 0x2a, 0x20, 0xae, 0xf7],
};
const SAVE_CLASS: GUID = GUID::from_u128(0xc0b4e2f3_ba21_4773_8dba_335ec946eb8b);
// Common IFileDialog IID: only inherited slots0..23 are called for either class.
const IID: GUID = GUID::from_u128(0x42f85136_db7e_439c_85f1_e4075d135fc8);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    ReportMarkdown,
    ReportJson,
    Mpd,
}
impl Mode {
    fn setup(
        self,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        Option<&'static str>,
    ) {
        match self {
            Self::ReportMarkdown => (
                crate::catalog::REPORT_MARKDOWN_FILTER,
                "*.md",
                "md",
                Some("diagnostic-report.md"),
            ),
            Self::ReportJson => (
                crate::catalog::REPORT_JSON_FILTER,
                "*.json",
                "json",
                Some("diagnostic-report.json"),
            ),
            Self::Mpd => (crate::catalog::REPORT_MPD_FILTER, "*.mpd", "mpd", None),
        }
    }
}
const ITEM_IID: GUID = GUID {
    data1: 0x43826d1e,
    data2: 0xe718,
    data3: 0x42ee,
    data4: [0xbc, 0x55, 0xa1, 0xe2, 0x61, 0xc3, 0x7b, 0xfe],
};
#[repr(C)]
struct Filter {
    name: *const u16,
    pattern: *const u16,
}
#[repr(C)]
struct Dialog {
    v: *const DialogV,
}
// IUnknown slots0..2, IModalWindow Show3, IFileDialog slots4..26, IFileOpenDialog27..28.
#[repr(C)]
struct DialogV {
    query: *const c_void,
    add: unsafe extern "system" fn(*mut Dialog) -> u32,
    release: unsafe extern "system" fn(*mut Dialog) -> u32,
    show: unsafe extern "system" fn(*mut Dialog, HWND) -> Hr,
    types: unsafe extern "system" fn(*mut Dialog, u32, *const Filter) -> Hr,
    slots5_8: [*const c_void; 4],
    set_options: unsafe extern "system" fn(*mut Dialog, u32) -> Hr,
    get_options: unsafe extern "system" fn(*mut Dialog, *mut u32) -> Hr,
    slot11: *const c_void,
    folder: unsafe extern "system" fn(*mut Dialog, *mut Item) -> Hr,
    slots13_14: [*const c_void; 2],
    filename: unsafe extern "system" fn(*mut Dialog, *const u16) -> Hr,
    slot16: *const c_void,
    title: unsafe extern "system" fn(*mut Dialog, *const u16) -> Hr,
    slots18_19: [*const c_void; 2],
    result: unsafe extern "system" fn(*mut Dialog, *mut *mut Item) -> Hr,
    slot21: *const c_void,
    extension: unsafe extern "system" fn(*mut Dialog, *const u16) -> Hr,
    close: unsafe extern "system" fn(*mut Dialog, Hr) -> Hr,
}
#[repr(C)]
struct Item {
    v: *const ItemV,
}
#[repr(C)]
struct ItemV {
    query: *const c_void,
    add: *const c_void,
    release: unsafe extern "system" fn(*mut Item) -> u32,
    slots3_4: [*const c_void; 2],
    name: unsafe extern "system" fn(*mut Item, u32, *mut *mut u16) -> Hr,
}
#[derive(Debug)]
pub enum PickerError {
    Hresult(i32),
    InvalidInput,
    InvalidResult,
    WrongOwner,
    AlreadyShown,
    InitialFolder,
    WorkingDirectory,
}
impl fmt::Display for PickerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hresult(hr) => write!(f, "native picker HRESULT {hr:#x}"),
            _ => f.write_str("native picker contract failure"),
        }
    }
}
impl std::error::Error for PickerError {}
fn hr(value: Hr) -> Result<(), PickerError> {
    if value < 0 {
        Err(PickerError::Hresult(value))
    } else {
        Ok(())
    }
}
// Preserve the public error and production constructor behavior; only tests log stage/HRESULT.
#[inline]
fn constructor_hr(stage: &str, value: Hr) -> Result<(), PickerError> {
    #[cfg(test)]
    eprintln!("diagnostic-picker-constructor stage={stage} HRESULT={value:#010x}");
    #[cfg(not(test))]
    let _ = stage;
    hr(value)
}
fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, PickerError> {
    let v: Vec<u16> = value.encode_wide().take(MAX + 1).collect();
    if v.is_empty() || v.len() > MAX || v.contains(&0) {
        return Err(PickerError::InvalidInput);
    }
    let mut out = v;
    out.push(0);
    Ok(out)
}
pub(crate) fn local_path(units: &[u16]) -> Result<PathBuf, PickerError> {
    if units.is_empty() || units.len() > MAX || units.contains(&0) || units.iter().any(|&u| u < 32)
    {
        return Err(PickerError::InvalidResult);
    }
    // Explicit disk-prefix paths only: standard C:\... or native extended \\?\C:\... .
    let u = if units.starts_with(&[92, 92, 63, 92]) {
        &units[4..]
    } else {
        units
    };
    if u.len() < 3
        || !((65..=90).contains(&u[0]) || (97..=122).contains(&u[0]))
        || u[1] != 58
        || u[2] != 92
    {
        return Err(PickerError::InvalidResult);
    }
    for component in u[3..].split(|&v| v == 92) {
        if (component.is_empty() && u.len() != 3)
            || component == [46]
            || component == [46, 46]
            || component
                .iter()
                .any(|&v| matches!(v, 47 | 58 | 42 | 63 | 34 | 60 | 62 | 124))
        {
            return Err(PickerError::InvalidResult);
        }
    }
    let path = PathBuf::from(std::ffi::OsString::from_wide(units));
    if !path.is_absolute() {
        return Err(PickerError::InvalidResult);
    }
    Ok(path)
}
// Shell parsing rejects extended disk prefixes. Convert only the validated initial
// folder hint; selected paths and file authority retain their original spelling.
fn shell_folder_hint(path: &Path) -> Result<Vec<u16>, PickerError> {
    let units: Vec<u16> = path.as_os_str().encode_wide().take(MAX + 1).collect();
    local_path(&units)?;
    let shell_units = units.strip_prefix(&[92, 92, 63, 92]).unwrap_or(&units);
    let mut out = shell_units.to_vec();
    out.push(0);
    Ok(out)
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: only constructed after successful same-thread STA initialization; Rc keeps it !Send/!Sync.
        unsafe { CoUninitialize() };
    }
}
struct Inner {
    dialog: *mut Dialog,
    _apartment: Apartment,
    shown: Cell<bool>,
    owner: Cell<HWND>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        // SAFETY: original successful CoCreate reference, released before Apartment field destruction on its owning STA.
        unsafe { ((*(*self.dialog).v).release)(self.dialog) };
    }
}
pub struct FilePicker {
    inner: Rc<Inner>,
}
#[derive(Clone)]
pub struct FilePickerCloser {
    reference: Rc<CloseReference>,
}
struct CloseReference {
    inner: Rc<Inner>,
}
impl Drop for CloseReference {
    fn drop(&mut self) {
        // SAFETY: balances this closer's AddRef; shared Inner keeps original reference and STA alive.
        unsafe { ((*(*self.inner.dialog).v).release)(self.inner.dialog) };
    }
}
impl FilePicker {
    pub fn new(initial: Option<&Path>, title: &str, mode: Mode) -> Result<Self, PickerError> {
        let title = wide(std::ffi::OsStr::new(title))?;
        let (filter_label, filter_pattern, extension, default_name) = mode.setup();
        let label = wide(std::ffi::OsStr::new(filter_label))?;
        // Initial is only a bounded file-name hint, never a file/root capability or selected folder adoption.
        let initial_path = if let Some(initial) = initial {
            let units: Vec<u16> = initial.as_os_str().encode_wide().take(MAX + 1).collect();
            Some(local_path(&units)?)
        } else {
            None
        };
        let default_exe = if initial_path.is_none() {
            Some(std::env::current_exe().map_err(|_| PickerError::InitialFolder)?)
        } else {
            None
        };
        let anchor = initial_path
            .as_deref()
            .or(default_exe.as_deref())
            .ok_or(PickerError::InitialFolder)?;
        let folder = anchor.parent().ok_or(PickerError::InitialFolder)?;
        let folder = shell_folder_hint(folder)?;
        let hint = initial_path
            .as_deref()
            .and_then(Path::file_name)
            .or_else(|| default_name.map(std::ffi::OsStr::new))
            .map(wide)
            .transpose()?;
        // SAFETY: null reserved, STA|DISABLE_OLE1DDE; successful S_FALSE also requires one balanced uninitialize.
        constructor_hr("CoInitializeEx", unsafe {
            CoInitializeEx(
                ptr::null(),
                (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
            )
        })?;
        let apartment = Apartment;
        let mut object = ptr::null_mut();
        // SAFETY: mode-specific dialog class/common IID_IFileDialog and valid writable output; no aggregation.
        constructor_hr("CoCreateInstance", unsafe {
            CoCreateInstance(
                if mode == Mode::Mpd {
                    &OPEN_CLASS
                } else {
                    &SAVE_CLASS
                },
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID,
                &mut object,
            )
        })?;
        if object.is_null() {
            return Err(PickerError::InvalidResult);
        }
        let inner = Rc::new(Inner {
            dialog: object.cast(),
            _apartment: apartment,
            shown: Cell::new(false),
            owner: Cell::new(ptr::null_mut()),
        });
        let picker = Self { inner };
        let d = picker.inner.dialog;
        // SAFETY: successful exact IID establishes DialogV ABI; buffers live for synchronous setup calls.
        unsafe {
            let v = &*(*d).v;
            let mut options = 0;
            constructor_hr("GetOptions", (v.get_options)(d, &mut options))?;
            #[cfg(test)]
            eprintln!("diagnostic-picker-constructor options-before={options:#010x}");
            // Preserve unrelated defaults; forbid multiselect/pickfolders/virtual results.
            // Save is draft selection only: skip test-file creation and keep filesystem/local path.
            // An overwrite prompt never authorizes the export worker to replace a file.
            options = (options & !(0x200 | 0x20 | 0x80 | 0x1000 | 0x2000 | 0x2))
                | 0x8
                | 0x40
                | 0x800
                | 0x2000000
                | 0x100000;
            options |= if mode == Mode::Mpd {
                0x1000
            } else {
                0x10000 | 0x2
            };
            #[cfg(test)]
            eprintln!("diagnostic-picker-constructor options-requested={options:#010x}");
            constructor_hr("SetOptions", (v.set_options)(d, options))?;
            constructor_hr("SetTitle", (v.title)(d, title.as_ptr()))?;
            // SAFETY: bounded explicit local folder name, no bind context; original returned shell reference released after SetFolder.
            let mut folder_item = ptr::null_mut();
            constructor_hr(
                "SHCreateItemFromParsingName",
                SHCreateItemFromParsingName(
                    folder.as_ptr(),
                    ptr::null_mut(),
                    &ITEM_IID,
                    &mut folder_item,
                ),
            )?;
            if folder_item.is_null() {
                return Err(PickerError::InitialFolder);
            }
            let folder_item = folder_item.cast::<Item>();
            let folder_result = (v.folder)(d, folder_item);
            ((*(*folder_item).v).release)(folder_item);
            constructor_hr("SetFolder", folder_result)?;
            let pattern = wide(std::ffi::OsStr::new(filter_pattern))?;
            let extension = wide(std::ffi::OsStr::new(extension))?;
            constructor_hr("SetDefaultExtension", (v.extension)(d, extension.as_ptr()))?;
            let filter = Filter {
                name: label.as_ptr(),
                pattern: pattern.as_ptr(),
            };
            constructor_hr("SetFileTypes", (v.types)(d, 1, &filter))?;
            if let Some(hint) = hint {
                // Slot15 is SetFileName; explicitly typed here, unlike any message/argv execution.
                constructor_hr("SetFileName", (v.filename)(d, hint.as_ptr()))?;
            }
        }
        Ok(picker)
    }
    pub fn closer(&self) -> FilePickerCloser {
        // SAFETY: same-STA owned live COM object; each closer owns exactly one added reference.
        unsafe { ((*(*self.inner.dialog).v).add)(self.inner.dialog) };
        FilePickerCloser {
            reference: Rc::new(CloseReference {
                inner: Rc::clone(&self.inner),
            }),
        }
    }
    pub fn show(&self, owner: HWND) -> Result<Option<PathBuf>, PickerError> {
        let mut pid = 0;
        // SAFETY: query-only API on explicit caller owner; rejects foreign process or GUI thread.
        let thread = unsafe { GetWindowThreadProcessId(owner, &mut pid) };
        // SAFETY: query current identities, no mutation or handles.
        if owner.is_null()
            || thread != unsafe { GetCurrentThreadId() }
            || pid != unsafe { GetCurrentProcessId() }
        {
            return Err(PickerError::WrongOwner);
        }
        if self.inner.shown.replace(true) {
            return Err(PickerError::AlreadyShown);
        }
        self.inner.owner.set(owner);
        let d = self.inner.dialog;
        // SAFETY: same STA valid owned COM dialog; modal callback may use closer without Rust mutable borrow.
        let cwd = std::env::current_dir().map_err(|_| PickerError::WorkingDirectory)?;
        let result = unsafe { ((*(*d).v).show)(d, owner) };
        if std::env::current_dir().map_err(|_| PickerError::WorkingDirectory)? != cwd {
            return Err(PickerError::WorkingDirectory);
        }
        if result == CANCELLED {
            return Ok(None);
        }
        hr(result)?;
        let mut item = ptr::null_mut();
        // SAFETY: exact GetResult ABI with writable original out pointer.
        hr(unsafe { ((*(*d).v).result)(d, &mut item) })?;
        if item.is_null() {
            return Err(PickerError::InvalidResult);
        }
        struct ItemGuard(*mut Item);
        impl Drop for ItemGuard {
            fn drop(&mut self) {
                // SAFETY: returned owned IShellItem ref.
                unsafe { ((*(*self.0).v).release)(self.0) };
            }
        }
        let item = ItemGuard(item);
        let mut text = ptr::null_mut();
        // SAFETY: SIGDN_FILESYSPATH; successful PWSTR is COM task allocation, not an arbitrary caller buffer.
        hr(unsafe { ((*(*item.0).v).name)(item.0, 0x80058000, &mut text) })?;
        if text.is_null() {
            return Err(PickerError::InvalidResult);
        }
        struct TextGuard(*mut u16);
        impl Drop for TextGuard {
            fn drop(&mut self) {
                // SAFETY: GetDisplayName task allocation, exactly once.
                unsafe { CoTaskMemFree(self.0.cast()) };
            }
        }
        let text = TextGuard(text);
        let mut len = 0;
        // SAFETY: COM returns NUL-terminated PWSTR; scan at most32768 units, no unbounded Rust allocation.
        unsafe {
            while len <= MAX && *text.0.add(len) != 0 {
                len += 1
            }
        }
        if len > MAX {
            return Err(PickerError::InvalidResult);
        }
        // SAFETY: bounded initialized prefix of the returned COM string, consumed before TextGuard Drop.
        local_path(unsafe { std::slice::from_raw_parts(text.0, len) }).map(Some)
    }
}
impl FilePickerCloser {
    pub fn cancel(&self) -> Result<(), PickerError> {
        let d = self.reference.inner.dialog;
        let mut window = ptr::null_mut();
        // SAFETY: query only this original COM object, never search arbitrary desktops/windows.
        hr(unsafe { IUnknown_GetWindow(d.cast(), &mut window) })?;
        let mut pid = 0;
        let owner = self.reference.inner.owner.get();
        if window.is_null()
            || owner.is_null()
            || unsafe { GetWindowThreadProcessId(window, &mut pid) }
                != unsafe { GetCurrentThreadId() }
            || pid != unsafe { GetCurrentProcessId() }
            || unsafe { GetWindow(window, GW_OWNER) } != owner
        {
            return Err(PickerError::WrongOwner);
        }
        // SAFETY: added reference on same STA; Close reports ERROR_CANCELLED through Show, no foreign HWND/input.
        let result = unsafe { ((*(*d).v).close)(d, CANCELLED) };
        // Close may acknowledge without ending the modal loop. Dispatch cancellation only
        // to the still-live original same-STA window; no window enumeration or global input.
        if unsafe { IsWindow(window) } != 0 {
            let mut current = ptr::null_mut();
            // Recheck association after COM callbacks; numeric HWNDs alone are reusable.
            hr(unsafe { IUnknown_GetWindow(d.cast(), &mut current) })?;
            if current != window {
                return Err(PickerError::WrongOwner);
            }
            if unsafe { GetWindowThreadProcessId(window, &mut pid) }
                != unsafe { GetCurrentThreadId() }
                || pid != unsafe { GetCurrentProcessId() }
                || unsafe { GetWindow(window, GW_OWNER) } != owner
            {
                return Err(PickerError::WrongOwner);
            }
            if unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } == 0 {
                return Err(PickerError::InvalidResult);
            }
        }
        hr(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_each_mode_constructor_releases_without_show_or_file_creation() {
        std::thread::spawn(|| {
            let executable = std::env::current_exe().unwrap();
            let hint = executable.parent().unwrap().join("진단🙂-unused.json");
            for mode in [Mode::ReportMarkdown, Mode::ReportJson, Mode::Mpd] {
                let picker = FilePicker::new(Some(&hint), "진단 파일 선택", mode).unwrap();
                drop(picker);
                assert!(!hint.exists(), "constructor must not create a draft file");
            }
        })
        .join()
        .unwrap();
    }
    #[test]
    fn result_path_policy_rejects_virtual_unc_relative_and_nul() {
        for value in [
            "relative.md",
            r"\\server\share\report.json",
            "C:\\bad\0.json",
        ] {
            assert!(local_path(&value.encode_utf16().collect::<Vec<_>>()).is_err());
        }
        let value: Vec<u16> = "C:\\진단🙂\\report.md".encode_utf16().collect();
        assert!(local_path(&value).is_ok());
    }
}
