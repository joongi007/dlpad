//! Windows local export UI state. All filesystem/process work belongs to the worker.
use crate::{
    catalog as c,
    local_export_worker::{ExportCompletion, LocalExportWorker},
};
use dlpad_engine::media_preview::{Tools, local_export::ExportRequest};
use eframe::egui;
use std::{path::PathBuf, sync::Arc};
#[derive(Default)]
pub(crate) struct LocalExportUi {
    worker: Option<LocalExportWorker>,
    completion: Option<ExportCompletion>,
    uncertain: bool,
}
impl LocalExportUi {
    /// Gate all shared mutations until accepted publication has returned and ACK is consumed.
    pub(crate) fn blocks_mutations(&self) -> bool {
        self.worker.is_some() || self.uncertain
    }
    pub(crate) fn close_ready(&self) -> bool {
        self.worker.is_none() && !self.uncertain
    }
    pub(crate) fn uncertainty(&self) -> bool {
        self.uncertain
    }
    pub(crate) fn busy(&self) -> bool {
        self.worker.as_ref().is_some_and(|worker| worker.busy())
    }
    pub(crate) fn start(
        &mut self,
        tools: Tools,
        root: PathBuf,
        request: ExportRequest,
        ctx: &egui::Context,
    ) -> Result<(), ()> {
        if self.blocks_mutations() {
            return Err(());
        }
        let wake = ctx.clone();
        let mut worker =
            LocalExportWorker::new(tools, root, Arc::new(move || wake.request_repaint()))
                .map_err(|_| ())?;
        worker.submit(request).map_err(|_| ())?;
        // Previous facts remain until a new explicit request has been accepted.
        self.completion = None;
        self.worker = Some(worker);
        Ok(())
    }
    fn accept_completion(&mut self, completion: ExportCompletion) {
        self.uncertain |=
            completion.cleanup_uncertain || completion.panicked || completion.result.is_none();
        self.completion = Some(completion);
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context) {
        if let Some(worker) = self.worker.as_mut() {
            let completion = worker.poll();
            let uncertain = worker.ownership_unconfirmed();
            if completion.is_some() {
                worker.request_stop();
            }
            if let Some(completion) = completion {
                self.accept_completion(completion);
            }
            self.uncertain |= uncertain;
        }
        // Stop ACK alone never discards an unconsumed completion or active request.
        if self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.stopped() && !worker.busy())
        {
            crate::local_export_worker::export_trace(5, true, false);
            self.worker = None;
        }
        if self.worker.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    pub(crate) fn request_stop(&mut self) {
        if let Some(worker) = &mut self.worker {
            worker.request_stop();
        }
    }
    pub(crate) fn show_status(&mut self, ui: &mut egui::Ui) {
        if self.busy() {
            ui.label(c::LOCAL_EXPORT_RUNNING);
            if crate::ui_components::ghost(ui, c::LOCAL_EXPORT_CANCEL, true).clicked() {
                self.request_stop();
            }
        } else if self.worker.is_some() {
            ui.label(c::LOCAL_EXPORT_WAITING_ACK);
        }
        if let Some(completion) = &self.completion {
            if self.uncertain {
                ui.label(c::LOCAL_EXPORT_CLEANUP_UNCERTAIN);
            }
            match &completion.result {
                Some(Ok(exported)) => {
                    ui.label(if self.uncertain {
                        c::LOCAL_EXPORT_FILE_VERIFIED_CLEANUP_PENDING
                    } else {
                        c::LOCAL_EXPORT_SAVED
                    });
                    let path = crate::path_picker::display_path(&exported.path);
                    ui.add(egui::Label::new(&path).truncate())
                        .on_hover_text(&path);
                    ui.label(c::LOCAL_EXPORT_RETAINED_STAGE);
                    let stage = crate::path_picker::display_path(&exported.retained_stage_path);
                    ui.add(egui::Label::new(&stage).truncate())
                        .on_hover_text(&stage);
                }
                Some(Err(failure)) => {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        c::LOCAL_EXPORT_FAILED_PRESERVED,
                    );
                    if let Some(directory) = &failure.fresh_directory {
                        let path = crate::path_picker::display_path(directory);
                        ui.add(egui::Label::new(&path).truncate())
                            .on_hover_text(&path);
                    }
                    if failure.stage_preserved {
                        ui.label(c::LOCAL_EXPORT_PARTIAL_STAGE);
                    }
                    if failure.final_created {
                        ui.label(c::LOCAL_EXPORT_FINAL_PRESENT_ON_FAILURE);
                    }
                }
                None => {
                    ui.label(c::LOCAL_EXPORT_MUTATION_UNKNOWN);
                    let path = crate::path_picker::display_path(&completion.requested_directory);
                    ui.add(egui::Label::new(&path).truncate())
                        .on_hover_text(&path);
                }
            }
        } else if self.uncertain {
            ui.label(c::LOCAL_EXPORT_CLEANUP_UNCERTAIN);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dlpad_engine::media_preview::{
        Error,
        local_export::{ExportFailure, Exported},
    };
    #[test]
    fn published_file_and_retained_stage_survive_cleanup_uncertainty() {
        let mut state = LocalExportUi::default();
        state.accept_completion(ExportCompletion {
            result: Some(Ok(Exported {
                path: PathBuf::from("synthetic/clip.mp4"),
                bytes: 123,
                duration_ms: 1000,
                retained_stage_path: PathBuf::from("synthetic/stage.mp4"),
            })),
            requested_directory: PathBuf::from("synthetic"),
            cleanup_uncertain: true,
            panicked: false,
        });
        assert!(state.blocks_mutations());
        assert!(!state.close_ready());
        let exported = state
            .completion
            .as_ref()
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .as_ref()
            .ok()
            .unwrap();
        assert_eq!(exported.bytes, 123);
        assert_eq!(
            exported.retained_stage_path,
            PathBuf::from("synthetic/stage.mp4")
        );
    }
    #[test]
    fn cancelled_partial_facts_remain_visible_without_becoming_uncertain() {
        let mut state = LocalExportUi::default();
        state.accept_completion(ExportCompletion {
            result: Some(Err(ExportFailure {
                error: Error::Cancelled,
                fresh_directory: Some(PathBuf::from("synthetic")),
                stage_preserved: true,
                final_created: false,
            })),
            requested_directory: PathBuf::from("synthetic"),
            cleanup_uncertain: false,
            panicked: false,
        });
        assert!(!state.uncertainty());
        assert!(state.close_ready());
        let failure = state
            .completion
            .as_ref()
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .as_ref()
            .err()
            .unwrap();
        assert!(failure.stage_preserved);
        assert_eq!(
            failure.fresh_directory.as_deref(),
            Some(std::path::Path::new("synthetic"))
        );
    }
    #[test]
    fn panic_blocks_new_work_and_close_without_inventing_no_mutation() {
        let mut state = LocalExportUi::default();
        state.accept_completion(ExportCompletion {
            result: None,
            requested_directory: PathBuf::from("possible-only"),
            cleanup_uncertain: true,
            panicked: true,
        });
        assert!(state.blocks_mutations());
        assert!(!state.close_ready());
        assert!(state.completion.unwrap().result.is_none());
    }
}
