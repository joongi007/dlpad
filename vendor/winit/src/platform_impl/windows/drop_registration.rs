//! Window-owner-thread OLE registration and bounded WM_DROPFILES fallback.
use super::drop_handler::FileDropHandler;
use crate::event::Event;
use crate::platform::windows::{FileDropFailure, FileDropRegistration};
use std::{marker::PhantomData, path::PathBuf, rc::Rc};
use windows_sys::Win32::{
    Foundation::{HWND, S_OK},
    System::Ole::{OleInitialize, OleUninitialize, RegisterDragDrop, RevokeDragDrop},
    UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP},
};

pub(crate) trait Api {
    type Target;
    fn initialize(&mut self) -> i32;
    fn create_target(&mut self) -> Self::Target;
    fn register(&mut self, target: &Self::Target) -> i32;
    fn revoke(&mut self) -> i32;
    fn uninitialize(&mut self);
    fn accept_files(&mut self, accept: bool);
}
pub(crate) struct Registration<A: Api> {
    api: A,
    target: Option<A::Target>,
    initialized: bool,
    registered: bool,
    fallback: bool,
    status: FileDropRegistration,
    _owner_thread: PhantomData<Rc<()>>,
}
impl<A: Api> Registration<A> {
    fn install(mut api: A) -> Self {
        let init = api.initialize();
        let mut initialized = init >= 0;
        let mut target = None;
        let mut registered = false;
        let failure = if init < 0 {
            Some(FileDropFailure::OleInitialization(init))
        } else {
            target = Some(api.create_target());
            let result = api.register(target.as_ref().unwrap());
            if result == S_OK {
                registered = true;
                None
            } else {
                Some(FileDropFailure::Registration(result))
            }
        };
        let fallback = failure.is_some();
        if fallback {
            // Release only our initial target reference. Failure does not authorize
            // RevokeDragDrop, especially ALREADYREGISTERED on someone else's target.
            drop(target.take());
            if initialized {
                api.uninitialize();
                initialized = false;
            }
        }
        api.accept_files(fallback);
        let status = failure.map_or(
            FileDropRegistration::Ole,
            FileDropRegistration::NativeFallback,
        );
        Self {
            api,
            target,
            initialized,
            registered,
            fallback,
            status,
            _owner_thread: PhantomData,
        }
    }
    pub(crate) fn status(&self) -> FileDropRegistration {
        self.status
    }
    pub(crate) fn uses_fallback(&self) -> bool {
        self.fallback
    }
    pub(crate) fn deactivate(&mut self) {
        if self.fallback {
            self.fallback = false;
            self.api.accept_files(false);
        }
        if self.registered {
            self.registered = false;
            let result = self.api.revoke();
            if result != S_OK {
                tracing::warn!(hresult = result, "OLE drop revocation was not confirmed");
            }
        }
        drop(self.target.take());
        if self.initialized {
            self.initialized = false;
            self.api.uninitialize();
        }
    }
}
impl<A: Api> Drop for Registration<A> {
    fn drop(&mut self) {
        self.deactivate();
    }
}
pub(crate) struct NativeApi {
    window: HWND,
    callback: Option<Box<dyn Fn(Event<()>)>>,
}
impl Api for NativeApi {
    type Target = FileDropHandler;
    fn initialize(&mut self) -> i32 {
        unsafe { OleInitialize(std::ptr::null_mut()) }
    }
    fn create_target(&mut self) -> FileDropHandler {
        FileDropHandler::new(
            self.window,
            self.callback.take().expect("single target factory"),
        )
    }
    fn register(&mut self, target: &FileDropHandler) -> i32 {
        let ptr =
            unsafe { std::ptr::addr_of_mut!((*target.data).interface) }.cast::<std::ffi::c_void>();
        unsafe { RegisterDragDrop(self.window, ptr) }
    }
    fn revoke(&mut self) -> i32 {
        unsafe { RevokeDragDrop(self.window) }
    }
    fn uninitialize(&mut self) {
        unsafe { OleUninitialize() }
    }
    fn accept_files(&mut self, accept: bool) {
        unsafe { DragAcceptFiles(self.window, i32::from(accept)) }
    }
}
pub(crate) type NativeRegistration = Registration<NativeApi>;
pub(crate) fn install(window: HWND, callback: Box<dyn Fn(Event<()>)>) -> NativeRegistration {
    Registration::install(NativeApi {
        window,
        callback: Some(callback),
    })
}
const MAX_FILES: u32 = 128;
const MAX_UNITS: usize = 32767;
const MAX_BATCH_BYTES: usize = 4 * 1024 * 1024;
trait Query {
    fn count(&mut self) -> u32;
    fn length(&mut self, index: u32) -> u32;
    fn fill(&mut self, index: u32, buffer: &mut [u16]) -> u32;
}
fn read_batch(query: &mut impl Query) -> Option<Vec<PathBuf>> {
    let count = query.count();
    if count == 0 || count > MAX_FILES {
        return None;
    }
    let mut paths = Vec::with_capacity(count as usize);
    let mut buffer = Vec::new();
    let mut total = 0usize;
    for index in 0..count {
        let length = query.length(index) as usize;
        if length == 0 || length > MAX_UNITS {
            return None;
        }
        total = total.checked_add(length.checked_mul(2)?)?;
        if total > MAX_BATCH_BYTES {
            return None;
        }
        buffer.resize(length + 1, 0);
        buffer.fill(0);
        if query.fill(index, &mut buffer) as usize != length
            || buffer[length] != 0
            || buffer[..length].contains(&0)
        {
            return None;
        }
        let text = String::from_utf16(&buffer[..length]).ok()?;
        // Bound the larger UTF16/UTF8 payload, not only WCHAR storage.
        total = total.checked_add(text.len().saturating_sub(length * 2))?;
        if total > MAX_BATCH_BYTES {
            return None;
        }
        if text.chars().any(char::is_control) {
            return None;
        }
        let path = PathBuf::from(text);
        if !matches!(path.components().next(),Some(std::path::Component::Prefix(prefix)) if matches!(prefix.kind(),std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_) | std::path::Prefix::UNC(_,_) | std::path::Prefix::VerbatimUNC(_,_)))
        {
            return None;
        }
        if !path.is_absolute()
            || path.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return None;
        }
        paths.push(path);
    }
    Some(paths)
}
struct NativeQuery(HDROP);
impl Query for NativeQuery {
    fn count(&mut self) -> u32 {
        unsafe { DragQueryFileW(self.0, u32::MAX, std::ptr::null_mut(), 0) }
    }
    fn length(&mut self, index: u32) -> u32 {
        unsafe { DragQueryFileW(self.0, index, std::ptr::null_mut(), 0) }
    }
    fn fill(&mut self, index: u32, buffer: &mut [u16]) -> u32 {
        unsafe { DragQueryFileW(self.0, index, buffer.as_mut_ptr(), buffer.len() as u32) }
    }
}
/// Borrowed OLE medium: caller releases STGMEDIUM; this function never DragFinish-es it.
pub(crate) fn borrowed_paths(hdrop: HDROP) -> Option<Vec<PathBuf>> {
    read_batch(&mut NativeQuery(hdrop))
}
/// WM_DROPFILES transfers HDROP cleanup to this callback, exactly once even on refusal.
fn finish_message_with<F: FnOnce()>(
    query: &mut impl Query,
    accept: bool,
    finish: F,
) -> Option<Vec<PathBuf>> {
    struct Cleanup<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Cleanup<F> {
        fn drop(&mut self) {
            if let Some(finish) = self.0.take() {
                finish();
            }
        }
    }
    let _owned = Cleanup(Some(finish));
    if accept {
        read_batch(query)
    } else {
        None
    }
}
pub(crate) fn finish_message(hdrop: HDROP, accept: bool) -> Option<Vec<PathBuf>> {
    if hdrop == 0 {
        return None;
    }
    finish_message_with(&mut NativeQuery(hdrop), accept, || unsafe {
        DragFinish(hdrop)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[derive(Clone)]
    struct Trace(Rc<RefCell<Vec<&'static str>>>);
    struct Target(Trace);
    impl Drop for Target {
        fn drop(&mut self) {
            self.0 .0.borrow_mut().push("release-initial");
        }
    }
    struct Fake {
        trace: Trace,
        init: i32,
        register: i32,
        revoke: i32,
    }
    impl Api for Fake {
        type Target = Target;
        fn initialize(&mut self) -> i32 {
            self.trace.0.borrow_mut().push("init");
            self.init
        }
        fn create_target(&mut self) -> Target {
            self.trace.0.borrow_mut().push("create");
            Target(self.trace.clone())
        }
        fn register(&mut self, _: &Target) -> i32 {
            self.trace.0.borrow_mut().push("register");
            self.register
        }
        fn revoke(&mut self) -> i32 {
            self.trace.0.borrow_mut().push("revoke-owned");
            self.revoke
        }
        fn uninitialize(&mut self) {
            self.trace.0.borrow_mut().push("uninit");
        }
        fn accept_files(&mut self, b: bool) {
            self.trace
                .0
                .borrow_mut()
                .push(if b { "wm-enable" } else { "wm-disable" });
        }
    }
    fn fake(init: i32, register: i32) -> (Fake, Trace) {
        let t = Trace(Rc::new(RefCell::new(vec![])));
        (
            Fake {
                trace: t.clone(),
                init,
                register,
                revoke: 0,
            },
            t,
        )
    }
    #[test]
    fn sdk_registration_failures_preserve_window_and_balance_without_foreign_revoke() {
        use windows_sys::Win32::Foundation::{DRAGDROP_E_ALREADYREGISTERED, RPC_E_UNEXPECTED};
        for hr in [RPC_E_UNEXPECTED, DRAGDROP_E_ALREADYREGISTERED, -1] {
            let (api, t) = fake(0, hr);
            let mut registration = Registration::install(api);
            assert_eq!(
                registration.status(),
                FileDropRegistration::NativeFallback(FileDropFailure::Registration(hr))
            );
            assert!(registration.uses_fallback());
            assert_eq!(
                &*t.0.borrow(),
                &[
                    "init",
                    "create",
                    "register",
                    "release-initial",
                    "uninit",
                    "wm-enable"
                ]
            );
            registration.deactivate();
            drop(registration);
            assert_eq!(
                &*t.0.borrow(),
                &[
                    "init",
                    "create",
                    "register",
                    "release-initial",
                    "uninit",
                    "wm-enable",
                    "wm-disable"
                ]
            );
        }
    }
    #[test]
    fn ole_success_and_s_false_revoke_once_release_then_balance_on_owner_scope() {
        for init in [0, 1] {
            let (api, t) = fake(init, 0);
            let mut registration = Registration::install(api);
            assert_eq!(registration.status(), FileDropRegistration::Ole);
            assert!(!registration.uses_fallback());
            registration.deactivate();
            registration.deactivate();
            drop(registration);
            assert_eq!(
                &*t.0.borrow(),
                &[
                    "init",
                    "create",
                    "register",
                    "wm-disable",
                    "revoke-owned",
                    "release-initial",
                    "uninit"
                ]
            );
        }
    }
    #[test]
    fn init_failure_does_not_create_target_register_revoke_or_uninitialize() {
        let (api, t) = fake(-1, 0);
        let registration = Registration::install(api);
        assert_eq!(
            registration.status(),
            FileDropRegistration::NativeFallback(FileDropFailure::OleInitialization(-1))
        );
        drop(registration);
        assert_eq!(&*t.0.borrow(), &["init", "wm-enable", "wm-disable"]);
    }
    struct Data {
        paths: Vec<Vec<u16>>,
        count_override: Option<u32>,
        length_override: Option<u32>,
        fills: usize,
        corrupt: bool,
    }
    impl Query for Data {
        fn count(&mut self) -> u32 {
            self.count_override.unwrap_or(self.paths.len() as u32)
        }
        fn length(&mut self, i: u32) -> u32 {
            self.length_override
                .unwrap_or(self.paths[i as usize].len() as u32)
        }
        fn fill(&mut self, i: u32, b: &mut [u16]) -> u32 {
            self.fills += 1;
            let p = &self.paths[i as usize];
            if p.len() + 1 > b.len() {
                return 0;
            }
            b[..p.len()].copy_from_slice(p);
            if self.corrupt {
                b[p.len()] = 1;
            }
            p.len() as u32
        }
    }
    fn data(paths: &[&str]) -> Data {
        Data {
            paths: paths.iter().map(|x| x.encode_utf16().collect()).collect(),
            count_override: None,
            length_override: None,
            fills: 0,
            corrupt: false,
        }
    }
    #[test]
    fn bounded_unicode_long_batch_and_reuse_preserve_literal_paths() {
        let long = format!("C:\\{}\\한글 영상.mp4", "a".repeat(598));
        let mut input = data(&[&long, "C:\\media\\b.mp4"]);
        let got = read_batch(&mut input).unwrap();
        assert_eq!(
            got,
            vec![PathBuf::from(long), PathBuf::from("C:\\media\\b.mp4")]
        );
        assert_eq!(input.fills, 2);
    }
    #[test]
    fn overflow_nonfile_invalid_and_torn_queries_decline_whole_batch() {
        let mut input = data(&[]);
        assert!(read_batch(&mut input).is_none());
        input.count_override = Some(129);
        assert!(read_batch(&mut input).is_none());
        assert_eq!(input.fills, 0);
        let mut input = data(&["C:\\a.mp4"]);
        input.length_override = Some(32768);
        assert!(read_batch(&mut input).is_none());
        assert_eq!(input.fills, 0);
        for bad in ["relative.mp4", "C:\\..\\bad.mp4", "C:\\bad\n.mp4"] {
            assert!(read_batch(&mut data(&["C:\\good.mp4", bad])).is_none());
        }
        let mut input = data(&["C:\\bad.mp4"]);
        input.paths[0].push(0xd800);
        assert!(read_batch(&mut input).is_none());
        input = data(&["C:\\bad.mp4"]);
        input.corrupt = true;
        assert!(read_batch(&mut input).is_none());
        let long = format!("C:\\{}", "a".repeat(32764));
        let paths = vec![long.as_str(); 65];
        assert!(read_batch(&mut data(&paths)).is_none());
    }
    #[test]
    fn transferred_message_cleanup_is_exact_once_on_success_refusal_overflow_and_unwind() {
        for (accept, count) in [(true, 1), (false, 1), (true, 129), (true, 0)] {
            let mut input = data(&["C:\\a.mp4"]);
            input.count_override = Some(count);
            let finished = Rc::new(RefCell::new(0));
            let observed = finished.clone();
            let result = finish_message_with(&mut input, accept, move || {
                *observed.borrow_mut() += 1;
            });
            assert_eq!(*finished.borrow(), 1);
            assert_eq!(result.is_some(), accept && count == 1);
            if !accept || count > MAX_FILES {
                assert_eq!(input.fills, 0);
            }
        }
        struct Panics;
        impl Query for Panics {
            fn count(&mut self) -> u32 {
                panic!("injected query panic");
            }
            fn length(&mut self, _: u32) -> u32 {
                unreachable!()
            }
            fn fill(&mut self, _: u32, _: &mut [u16]) -> u32 {
                unreachable!()
            }
        }
        let finished = Rc::new(RefCell::new(0));
        let observed = finished.clone();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            finish_message_with(&mut Panics, true, move || {
                *observed.borrow_mut() += 1;
            })
        }))
        .is_err());
        assert_eq!(*finished.borrow(), 1);
    }
}
