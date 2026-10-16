//! Narrow same-STA IFileOpenDialog binding. No payload parsing or installation I/O.
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
const CLASS: GUID = GUID {
    data1: 0xdc1c5a9c,
    data2: 0xe88a,
    data3: 0x4dde,
    data4: [0xa5, 0xa1, 0x60, 0xf8, 0x2a, 0x20, 0xae, 0xf7],
};
const IID: GUID = GUID {
    data1: 0xd57c7288,
    data2: 0xd4ad,
    data3: 0x4768,
    data4: [0xbe, 0x02, 0x9d, 0x96, 0x95, 0x32, 0xd9, 0x60],
};
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
    slots21_22: [*const c_void; 2],
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
fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, PickerError> {
    let v: Vec<u16> = value.encode_wide().take(MAX + 1).collect();
    if v.is_empty() || v.len() > MAX || v.contains(&0) {
        return Err(PickerError::InvalidInput);
    }
    let mut out = v;
    out.push(0);
    Ok(out)
}
fn local_path(units: &[u16]) -> Result<PathBuf, PickerError> {
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
pub struct ManifestPicker {
    inner: Rc<Inner>,
}
#[derive(Clone)]
pub struct PickerCloser {
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
impl ManifestPicker {
    pub fn new(
        initial: Option<&Path>,
        title: &str,
        filter_name: &str,
    ) -> Result<Self, PickerError> {
        let title = wide(std::ffi::OsStr::new(title))?;
        let label = wide(std::ffi::OsStr::new(filter_name))?;
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
        let folder_units: Vec<u16> = folder.as_os_str().encode_wide().take(MAX + 1).collect();
        let folder = local_path(&folder_units)?;
        let folder = wide(folder.as_os_str())?;
        let hint = initial_path
            .as_deref()
            .and_then(Path::file_name)
            .map(wide)
            .transpose()?;
        // SAFETY: null reserved, STA|DISABLE_OLE1DDE; successful S_FALSE also requires one balanced uninitialize.
        hr(unsafe {
            CoInitializeEx(
                ptr::null(),
                (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
            )
        })?;
        let apartment = Apartment;
        let mut object = ptr::null_mut();
        // SAFETY: exact CLSID_FileOpenDialog/IID_IFileOpenDialog and valid writable output; no aggregation.
        hr(unsafe {
            CoCreateInstance(
                &CLASS,
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
            hr((v.get_options)(d, &mut options))?;
            // Preserve unrelated defaults; forbid multiselect/pickfolders/virtual results.
            options = (options & !(0x200 | 0x20 | 0x80))
                | 0x8
                | 0x40
                | 0x800
                | 0x1000
                | 0x2000000
                | 0x100000;
            hr((v.set_options)(d, options))?;
            hr((v.title)(d, title.as_ptr()))?;
            // SAFETY: bounded explicit local folder name, no bind context; original returned shell reference released after SetFolder.
            let mut folder_item = ptr::null_mut();
            hr(SHCreateItemFromParsingName(
                folder.as_ptr(),
                ptr::null_mut(),
                &ITEM_IID,
                &mut folder_item,
            ))?;
            if folder_item.is_null() {
                return Err(PickerError::InitialFolder);
            }
            let folder_item = folder_item.cast::<Item>();
            let folder_result = (v.folder)(d, folder_item);
            ((*(*folder_item).v).release)(folder_item);
            hr(folder_result)?;
            let pattern = wide(std::ffi::OsStr::new("*.json"))?;
            let filter = Filter {
                name: label.as_ptr(),
                pattern: pattern.as_ptr(),
            };
            hr((v.types)(d, 1, &filter))?;
            if let Some(hint) = hint {
                // Slot15 is SetFileName; explicitly typed here, unlike any message/argv execution.
                hr((v.filename)(d, hint.as_ptr()))?;
            }
        }
        Ok(picker)
    }
    pub fn closer(&self) -> PickerCloser {
        // SAFETY: same-STA owned live COM object; each closer owns exactly one added reference.
        unsafe { ((*(*self.inner.dialog).v).add)(self.inner.dialog) };
        PickerCloser {
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
impl PickerCloser {
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
    fn p(s: &str) -> Result<PathBuf, PickerError> {
        local_path(&s.encode_utf16().collect::<Vec<_>>())
    }
    #[test]
    fn local_standard_extended_unicode() {
        assert!(p("C:\\owned\\한글.json").is_ok());
        assert!(p("C:\\").is_ok());
        assert!(p("\\\\?\\C:\\owned\\한글.json").is_ok());
    }
    #[test]
    fn rejects_namespace_escape() {
        for s in [
            "\\\\server\\x",
            "\\\\.\\pipe\\x",
            "C:x",
            "C:\\a\\..\\x",
            "C:\\a:x",
            "C:\\a/xx",
            "C:\\a\\",
            "C:\\a\0x",
        ] {
            assert!(p(s).is_err());
        }
    }
    #[test]
    fn bound_before_path_allocation() {
        assert!(local_path(&vec![65; MAX + 1]).is_err());
        assert!(wide(std::ffi::OsStr::new("a\0b")).is_err());
    }
    #[test]
    fn cancellation_is_only_specific_hresult() {
        assert!(hr(CANCELLED).is_err());
        assert!(hr(0).is_ok());
        assert!(hr(0x80004005u32 as i32).is_err());
    }
}
