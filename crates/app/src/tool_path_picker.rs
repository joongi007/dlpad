//! Bounded GUI coordinator. Native dialog/OS I/O live on one transient STA worker.
use eframe::egui;
use std::path::PathBuf;
#[cfg(all(windows, feature = "cpu"))]
#[path = "tool_file_dialog.rs"]
mod native_dialog;
#[cfg(all(windows, feature = "cpu"))]
#[path = "tool_file_worker.rs"]
mod native_worker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    YtDlp,
    Ffmpeg,
    Deno,
}
#[derive(Default)]
pub(crate) struct PathPicker {
    #[cfg(all(windows, feature = "cpu"))]
    owner: Option<std::sync::Arc<winit::window::Window>>,
    #[cfg(all(windows, feature = "cpu"))]
    worker: Option<native_worker::AsyncFolderPicker>,
    target: Option<Target>,
    cancel_requested: std::cell::Cell<bool>,
}
impl PathPicker {
    #[cfg(all(test, windows, feature = "cpu"))]
    pub(crate) fn completed_for_test(target: Target, path: PathBuf) -> Self {
        Self {
            worker: Some(native_worker::AsyncFolderPicker::completed_for_test(path)),
            target: Some(target),
            owner: None,
            cancel_requested: std::cell::Cell::new(false),
        }
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[allow(dead_code, reason = "Used by the native CPU window host")]
    pub(crate) fn set_owner(&mut self, owner: std::sync::Arc<winit::window::Window>) {
        self.owner = Some(owner);
    }
    pub(crate) fn available(&self) -> bool {
        #[cfg(all(windows, feature = "cpu"))]
        {
            self.owner.is_some()
        }
        #[cfg(not(all(windows, feature = "cpu")))]
        {
            false
        }
    }
    pub(crate) fn busy(&self) -> bool {
        self.target.is_some()
    }
    pub(crate) fn start(
        &mut self,
        target: Target,
        initial: PathBuf,
        ctx: &egui::Context,
    ) -> Result<(), ()> {
        if self.busy() || !self.available() {
            return Err(());
        }
        #[cfg(all(windows, feature = "cpu"))]
        {
            let owner = self.owner.clone().ok_or(())?;
            let worker = native_worker::AsyncFolderPicker::start(
                owner,
                if initial.as_os_str().is_empty() {
                    None
                } else {
                    Some(initial)
                },
                crate::catalog::TOOL_CHOOSE_EXECUTABLE.into(),
                ctx.clone(),
            )
            .map_err(|_| ())?;
            self.cancel_requested.set(false);
            self.worker = Some(worker);
            self.target = Some(target);
            Ok(())
        }
        #[cfg(not(all(windows, feature = "cpu")))]
        {
            let _ = (target, initial, ctx);
            Err(())
        }
    }
    pub(crate) fn cancel(&self) {
        self.cancel_requested.set(true);
        #[cfg(all(windows, feature = "cpu"))]
        if let Some(worker) = &self.worker {
            let _ = worker.cancel();
        }
    }
    pub(crate) fn poll(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<(Target, Result<Option<PathBuf>, ()>)> {
        if !self.busy() {
            return None;
        }
        #[cfg(all(windows, feature = "cpu"))]
        {
            let worker = self.worker.as_ref()?;
            match worker.try_complete() {
                Ok(Some(result)) => {
                    let target = self.target.take()?;
                    self.worker = None;
                    let result = result.map_err(|_| ()).and_then(|path| {
                        if path
                            .as_ref()
                            .is_some_and(|p| p.to_str().is_none_or(|s| s.len() > 32768))
                        {
                            Err(())
                        } else {
                            Ok(path)
                        }
                    });
                    return Some((
                        target,
                        cancelled_selection(self.cancel_requested.get(), result),
                    ));
                }
                Err(_) => {
                    let target = self.target.take()?;
                    self.worker = None;
                    return Some((target, Err(())));
                }
                Ok(None) => {}
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        None
    }
}
impl Drop for PathPicker {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(any(test, all(windows, feature = "cpu")))]
fn cancelled_selection(
    cancelled: bool,
    result: Result<Option<PathBuf>, ()>,
) -> Result<Option<PathBuf>, ()> {
    if cancelled {
        result.map(|_| None)
    } else {
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_owner_cannot_begin_or_mutate_selection() {
        let mut picker = PathPicker::default();
        let ctx = egui::Context::default();
        assert!(picker.start(Target::YtDlp, PathBuf::new(), &ctx).is_err());
        assert!(!picker.busy());
        assert!(picker.poll(&ctx).is_none());
    }
    #[cfg(all(windows, feature = "cpu"))]
    #[test]
    fn queued_executable_selection_is_discarded_by_cancel_before_poll() {
        let path = std::env::temp_dir().join("owned-synthetic-tool.exe");
        let mut picker = PathPicker::completed_for_test(Target::Ffmpeg, path);
        picker.cancel();
        let result = picker.poll(&egui::Context::default()).unwrap();
        assert_eq!(result.0, Target::Ffmpeg);
        assert!(matches!(result.1, Ok(None)));
        assert!(!picker.busy());
    }
}
