//! Media content is the primary workspace; preview processes remain off the UI thread.
use crate::{catalog as c, media_preview_worker::PreviewWorker, media_timeline};
use dlpad_core::{media_selection::MediaSelection, options::SourceUrl};
use dlpad_engine::media_preview::{Error, FrameBatch, Prepared, Product, Request, Source, Tools};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
#[derive(Default)]
pub(crate) struct Workspace {
    worker: Option<PreviewWorker>,
    detail_open: crate::media_detail_open::DetailOpen,
    tools: Option<(String, String, String)>,
    pending: Option<Source>,
    retry_source: Option<Source>,
    prepared: Option<Arc<Prepared>>,
    selection: Option<MediaSelection>,
    texture: Option<egui::TextureHandle>,
    url_key: String,
    changed_at: Option<Instant>,
    attempted: String,
    error: Option<&'static str>,
    tool_error_rect: Option<egui::Rect>,
    tool_error_rect_reason: Option<&'static str>,
    revealed_error: Option<&'static str>,
    error_reveal_frame: Option<u64>,
    preview_revealed: bool,
    preview_reveal_frame: Option<u64>,
    playing: bool,
    batch: Option<FrameBatch>,
    clock: Option<Instant>,
    frame_index: Option<usize>,
    range_edit: Option<MediaSelection>,
    closing: bool,
    fatal: bool,
}
fn playback_button(ui: &mut egui::Ui, playing: bool, busy: bool) -> egui::Response {
    ui.scope_builder(egui::UiBuilder::new().id("media-playback-action"), |ui| {
        crate::ui_components::secondary(
            ui,
            if playing {
                c::MEDIA_PAUSE
            } else {
                c::MEDIA_PLAY
            },
            !busy || playing,
        )
    })
    .inner
}
impl Workspace {
    pub(crate) fn drain(&mut self) {
        self.detail_open.poll();
        if let Some(Err(error)) = self.worker.as_mut().and_then(PreviewWorker::poll) {
            self.handle_failure(error);
        }
    }
    pub(crate) fn tool_error(&self) -> Option<&'static str> {
        self.error.filter(|error| {
            matches!(
                *error,
                c::MEDIA_TOOLS | c::MEDIA_TOOL_REQUIRED | c::MEDIA_PREVIEW_FFMPEG_REQUIRED
            )
        })
    }
    /// The exact Label response from the last show pass, not a preview viewport.
    /// An offscreen layout rect is a valid scroll target, not paint acceptance.
    pub(crate) fn error_target(&self) -> Option<egui::Rect> {
        if self.tool_error().is_some() && self.tool_error() == self.tool_error_rect_reason {
            self.tool_error_rect
        } else {
            None
        }
    }
    pub(crate) fn fatal(&self) -> bool {
        self.fatal
    }
    pub(crate) fn closing(&self) -> bool {
        self.closing
    }
    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some()
            || self.worker.as_ref().is_some_and(PreviewWorker::busy)
            || self.detail_open.busy()
    }
    pub(crate) fn stopped(&self) -> bool {
        !self.fatal
            && self.worker.as_ref().is_none_or(PreviewWorker::stopped)
            && self.detail_open.stopped()
    }
    pub(crate) fn has_local_intent(&self) -> bool {
        matches!(&self.retry_source, Some(Source::Local(_)))
    }
    pub(crate) fn source_unsupported(&self) -> bool {
        self.error == Some(c::MEDIA_UNSUPPORTED)
    }
    pub(crate) fn tool_execution_failed(&self) -> bool {
        self.error == Some(c::MEDIA_TOOL_FAILED)
    }
    pub(crate) fn can_retry_after_tool_setup(&self) -> bool {
        !self.fatal
            && !self.closing
            && !self.busy()
            && self.selection.is_none()
            && self.retry_source.is_some()
            && (self.tool_error().is_some() || self.tool_execution_failed())
    }
    /// Explicit user retry. The normal worker creates fresh original pins and revalidates tools.
    pub(crate) fn retry_after_tool_setup(&mut self) -> bool {
        if !self.can_retry_after_tool_setup() {
            return false;
        }
        self.pending = self.retry_source.clone();
        self.tools = None;
        self.error = None;
        true
    }
    pub(crate) fn local(&self) -> bool {
        self.prepared.as_ref().is_some_and(|p| p.is_local())
    }
    pub(crate) fn prepared_for_export(&self) -> Option<Arc<Prepared>> {
        self.prepared
            .as_ref()
            .filter(|prepared| prepared.is_local())
            .cloned()
    }
    pub(crate) fn selection(&self) -> Option<MediaSelection> {
        self.selection
    }
    pub(crate) fn synchronize_range(&mut self, start: u64, end: u64) {
        if let Some(s) = &mut self.selection {
            let _ = s.set_range(start, end);
        }
    }
    pub(crate) fn take_range_edit(&mut self) -> Option<MediaSelection> {
        self.range_edit.take()
    }
    pub(crate) fn request_stop(&mut self) {
        self.detail_open.request_stop();
        self.closing = true;
        self.pending = None;
        self.playing = false;
        self.batch = None;
        if let Some(w) = &mut self.worker {
            w.request_stop();
        }
    }
    pub(crate) fn cancel(&mut self) {
        self.pending = None;
        self.playing = false;
        self.batch = None;
        if let Some(w) = &mut self.worker {
            w.cancel();
        }
    }
    fn reset_content(&mut self) {
        self.retry_source = None;
        self.detail_open.source_changed();
        self.prepared = None;
        self.selection = None;
        self.texture = None;
        self.playing = false;
        self.batch = None;
        self.clock = None;
        self.error = None;
        self.revealed_error = None;
        self.error_reveal_frame = None;
        self.preview_revealed = false;
        self.preview_reveal_frame = None;
        self.range_edit = None;
    }
    pub(crate) fn source_changed(&mut self, url: &str) -> bool {
        let key = url.trim();
        if self.url_key != key {
            self.url_key = key.into();
            self.attempted.clear();
            self.changed_at = Some(Instant::now());
            self.cancel();
            self.reset_content();
            return true;
        }
        false
    }
    pub(crate) fn clear_source(&mut self) {
        self.retry_source = None;
        self.pending = None;
        if self.fatal {
            return;
        }
        self.cancel();
        self.reset_content();
        self.pending = None;
        self.changed_at = None;
        self.attempted.clear();
        self.url_key.clear();
    }
    pub(crate) fn prepare_url(&mut self, url: &str) {
        self.retry_source = None;
        self.pending = None;
        let key = url.trim();
        if SourceUrl::parse(key).is_err() {
            self.error = Some(c::MEDIA_URL_INVALID);
            return;
        }
        self.attempted = key.into();
        self.reset_content();
        self.retry_source = Some(Source::Url(key.into()));
        self.pending = self.retry_source.clone();
        if let Some(w) = &mut self.worker {
            w.cancel();
        }
    }
    pub(crate) fn dropped(&mut self, path: PathBuf) {
        self.cancel();
        self.reset_content();
        self.changed_at = None;
        // Intent only, never a validated local pin or a prepared/selected video.
        let bounded = path.is_absolute()
            && path.as_os_str().len() <= 32767
            && path.components().count() <= 256;
        #[cfg(windows)]
        let bounded = {
            use std::os::windows::ffi::OsStrExt;
            bounded && path.as_os_str().encode_wide().take(32767).count() <= 32766
        };
        self.retry_source = bounded.then(|| Source::Local(path.clone()));
        self.pending = Some(Source::Local(path));
    }
    fn handle_failure(&mut self, error: Error) {
        self.playing = false;
        self.batch = None;
        if error == Error::OwnershipUnconfirmed {
            self.fatal = true;
        }
        self.error = Some(match error {
            Error::Cancelled => c::MEDIA_CANCELLED,
            Error::UnsupportedMedia => c::MEDIA_UNSUPPORTED,
            Error::TimedOut => c::MEDIA_TIMEOUT,
            Error::FrameUnavailable => c::MEDIA_FRAME_UNAVAILABLE,
            Error::OwnershipUnconfirmed => c::MEDIA_CLEANUP_FAILED,
            Error::InvalidMetadata => c::MEDIA_INVALID_DATA,
            Error::OutputLimit => c::MEDIA_DATA_LIMIT,
            Error::ToolFailed => c::MEDIA_TOOL_FAILED,
            _ => c::MEDIA_FAILED,
        });
    }
    fn submit(&mut self, request: Request) -> bool {
        let result = self.worker.as_mut().map(|w| w.submit(request));
        match result {
            Some(Ok(())) => true,
            Some(Err(e)) => {
                self.handle_failure(e);
                false
            }
            None => {
                self.handle_failure(Error::ToolFailed);
                false
            }
        }
    }
    fn upload(&mut self, ctx: &egui::Context, bytes: &[u8]) {
        if bytes.len() != 320 * 180 * 3 {
            return;
        }
        let image = egui::ColorImage::from_rgb([320, 180], bytes);
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.texture =
                Some(ctx.load_texture("media-preview", image, egui::TextureOptions::LINEAR));
        }
    }
    fn seek(&mut self, ms: u64) {
        self.playing = false;
        self.batch = None;
        if let Some(s) = &mut self.selection {
            s.seek(ms);
        }
        if let (Some(w), Some(p)) = (&mut self.worker, &self.prepared) {
            let result = w.submit(Request::Seek {
                prepared: p.clone(),
                position_ms: ms,
            });
            if let Err(e) = result {
                self.handle_failure(e);
            }
        }
    }
    fn play(&mut self) {
        if let (Some(w), Some(p), Some(s)) = (&mut self.worker, &self.prepared, self.selection) {
            let pos = if s.playhead_ms() + 1 >= s.duration_ms() {
                0
            } else {
                s.playhead_ms()
            };
            let result = w.submit(Request::PreviewFrames {
                prepared: p.clone(),
                position_ms: pos,
            });
            match result {
                Ok(()) => self.playing = true,
                Err(e) => self.handle_failure(e),
            }
        }
    }
    pub(crate) fn poll(
        &mut self,
        ctx: &egui::Context,
        url: &str,
        ytdlp: &str,
        ffmpeg: &str,
        deno: &str,
        managed_root: &std::path::Path,
    ) {
        self.detail_open.poll();
        if self.detail_open.busy() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if self.closing {
            if let Some(Err(e)) = self.worker.as_mut().and_then(PreviewWorker::poll) {
                self.handle_failure(e);
            }
            return;
        }
        if self.fatal {
            return;
        }
        if !url.trim().is_empty()
            && SourceUrl::parse(url.trim()).is_ok()
            && self.attempted != url.trim()
            && self
                .changed_at
                .is_some_and(|t| t.elapsed() >= Duration::from_millis(700))
            && SourceUrl::parse(url.trim()).is_ok()
        {
            self.prepare_url(url);
        }
        let snapshot = (ytdlp.to_owned(), ffmpeg.to_owned(), deno.to_owned());
        if self.pending.is_some() && self.tools.as_ref() != Some(&snapshot) {
            if let Some(w) = &mut self.worker {
                w.request_stop();
                let fatal = matches!(w.poll(), Some(Err(Error::OwnershipUnconfirmed)));
                if fatal {
                    self.handle_failure(Error::OwnershipUnconfirmed);
                    return;
                }
                if !w.stopped() {
                    ctx.request_repaint_after(Duration::from_millis(50));
                    return;
                }
            }
            self.worker = None;
            self.tools = None;
        }
        if self.pending.is_some() && self.worker.is_none() {
            let ff = PathBuf::from(ffmpeg);
            if !ff.is_absolute()
                || (matches!(self.pending, Some(Source::Url(_)))
                    && !PathBuf::from(ytdlp).is_absolute())
            {
                let missing_ffmpeg = !ff.is_absolute();
                let missing_ytdlp = matches!(self.pending, Some(Source::Url(_)))
                    && !PathBuf::from(ytdlp).is_absolute();
                self.error = Some(match (missing_ytdlp, missing_ffmpeg) {
                    (true, true) => c::MEDIA_TOOLS,
                    (true, false) => c::MEDIA_TOOL_REQUIRED,
                    (false, _) => c::MEDIA_PREVIEW_FFMPEG_REQUIRED,
                });
                self.pending = None;
                return;
            }
            let probe = ff
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(if cfg!(windows) {
                    "ffprobe.exe"
                } else {
                    "ffprobe"
                });
            let wake = ctx.clone();
            match PreviewWorker::new(
                Tools {
                    ytdlp: PathBuf::from(ytdlp),
                    deno: (!deno.is_empty()).then(|| PathBuf::from(deno)),
                    ffmpeg: ff,
                    ffprobe: probe,
                },
                managed_root.to_path_buf(),
                Arc::new(move || wake.request_repaint()),
            ) {
                Ok(worker) => {
                    self.worker = Some(worker);
                    self.tools = Some(snapshot);
                }
                Err(_) => {
                    self.error = Some(c::MEDIA_FAILED);
                    self.pending = None;
                    return;
                }
            }
        }
        if let Some(request) = self.pending.take() {
            self.submit(Request::Prepare(request));
        }
        let result = self.worker.as_mut().and_then(PreviewWorker::poll);
        match result {
            Some(Ok(Product::Prepared(p))) => {
                self.selection = MediaSelection::new(p.duration_ms).ok();
                self.prepared = Some(p.clone());
                self.error = None;
                self.submit(Request::Seek {
                    prepared: p,
                    position_ms: 0,
                });
            }
            Some(Ok(Product::Frame(frame))) => {
                self.upload(ctx, &frame.rgb);
                if let Some(selection) = &mut self.selection {
                    selection.seek(frame.seek_ms);
                }
                self.error = None;
            }
            Some(Ok(Product::Frames(batch))) => {
                self.batch = Some(batch);
                self.clock = Some(Instant::now());
                self.frame_index = None;
                self.error = None;
            }
            Some(Ok(Product::Finished)) => {
                self.playing = false;
                self.batch = None;
                self.clock = None;
            }
            Some(Err(e)) => self.handle_failure(e),
            None => {}
        }
        if self.playing {
            if let (Some(batch), Some(clock)) = (&self.batch, self.clock) {
                let index = (clock.elapsed().as_secs_f64() * batch.fps() as f64).floor() as usize;
                if index < batch.frames() {
                    if self.frame_index != Some(index) {
                        let start = index * 320 * 180 * 3;
                        let bytes = batch.rgb[start..start + 320 * 180 * 3].to_vec();
                        let ms = batch.start_ms + (index as u64 * 1000 / batch.fps() as u64);
                        self.upload(ctx, &bytes);
                        self.frame_index = Some(index);
                        if let Some(s) = &mut self.selection {
                            s.seek(ms);
                        }
                    }
                } else {
                    let next = batch.start_ms + batch.frames() as u64 * 1000 / batch.fps() as u64;
                    self.batch = None;
                    if let (Some(w), Some(p)) = (&mut self.worker, &self.prepared) {
                        if next < p.duration_ms {
                            let result = w.submit(Request::PreviewFrames {
                                prepared: p.clone(),
                                position_ms: next,
                            });
                            if let Err(e) = result {
                                self.handle_failure(e);
                            }
                        } else {
                            self.playing = false;
                        }
                    }
                }
            }
            ctx.request_repaint_after(Duration::from_millis(1000 / 6));
        } else if self.busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if !url.trim().is_empty()
            && SourceUrl::parse(url.trim()).is_ok()
            && self.attempted != url.trim()
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    #[cfg(test)]
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        self.show_with_open(ui, false);
    }
    pub(crate) fn show_with_open(&mut self, ui: &mut egui::Ui, can_open: bool) {
        self.tool_error_rect = None;
        self.tool_error_rect_reason = None;
        let busy = self.busy();
        let width = ui.available_width();
        // Reserve a media-sized canvas for ready or in-flight content. Idle
        // empty/blocked states need only the icon and their visible explanation.
        // The frame is a minimum layout allocation, so a long error can still
        // grow naturally instead of being clipped or silently abbreviated.
        let height = if !busy && self.prepared.is_none() && self.texture.is_none() {
            (width * 9.0 / 16.0).clamp(96.0, 120.0)
        } else {
            (width * 9.0 / 16.0).clamp(140.0, 200.0)
        };
        // Keep a small inset inside the body: scrolling and pixel rounding
        // must not put an otherwise fitted frame on the clip boundary.
        let height = height.min((ui.clip_rect().height() - 8.0).max(90.0));
        let frame = egui::Frame::new()
            .fill(if self.texture.is_some() {
                egui::Color32::from_gray(18)
            } else if ui.visuals().dark_mode {
                egui::Color32::from_gray(22)
            } else {
                egui::Color32::from_gray(245)
            })
            .corner_radius(6)
            .inner_margin(0);
        let preview = frame.show(ui, |ui| {
            let size = egui::vec2(ui.available_width(), height);
            if let Some(texture) = &self.texture {
                // A media canvas is one exact allocation: widget spacing must
                // not inflate it beyond the visible body at high DPI.
                let (canvas, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let scale = (size.x / 320.0).min(size.y / 180.0);
                let image = egui::Rect::from_center_size(
                    canvas.center(),
                    egui::vec2(320.0 * scale, 180.0 * scale),
                );
                egui::Image::new(texture).paint_at(ui, image);
            } else {
                ui.allocate_ui_with_layout(
                    size,
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.set_min_size(size);
                        ui.add_space((height - 72.0) * 0.5);
                        let (icon, _) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                        let ink = ui.visuals().weak_text_color();
                        // This is an empty media placeholder, not a play action.
                        // Reserve the play triangle for the real playback button.
                        let film =
                            egui::Rect::from_center_size(icon.center(), egui::vec2(30.0, 22.0));
                        let stroke = egui::Stroke::new(1.0, ink);
                        ui.painter()
                            .rect_stroke(film, 3, stroke, egui::StrokeKind::Inside);
                        for offset in [-6.0, 0.0, 6.0] {
                            for x in [film.left() + 4.0, film.right() - 4.0] {
                                ui.painter().rect_filled(
                                    egui::Rect::from_center_size(
                                        egui::pos2(x, film.center().y + offset),
                                        egui::vec2(2.0, 2.0),
                                    ),
                                    0,
                                    ink,
                                );
                            }
                        }
                        for x in [film.left() + 7.0, film.right() - 7.0] {
                            ui.painter().line_segment(
                                [egui::pos2(x, film.top()), egui::pos2(x, film.bottom())],
                                stroke,
                            );
                        }
                        let text = if let Some(error) = self.error {
                            error
                        } else if busy {
                            c::MEDIA_WAIT
                        } else {
                            c::MEDIA_EMPTY
                        };
                        let message = ui.add(
                            egui::Label::new(
                                egui::RichText::new(text).color(ui.visuals().weak_text_color()),
                            )
                            .wrap(),
                        );
                        if self.tool_error() == Some(text) {
                            self.tool_error_rect = Some(message.rect);
                            self.tool_error_rect_reason = Some(text);
                        }
                        let frame = ui.ctx().cumulative_frame_nr();
                        if self.error.is_some() && self.error != self.revealed_error {
                            self.revealed_error = self.error;
                            self.error_reveal_frame = Some(frame);
                        }
                        if self.error.is_some() && self.error_reveal_frame == Some(frame) {
                            message.scroll_to_me(None);
                        }
                    },
                );
            }
        });
        let current_frame = ui.ctx().cumulative_frame_nr();
        if self.texture.is_some() && !self.preview_revealed {
            self.preview_revealed = true;
            self.preview_reveal_frame = Some(current_frame);
        }
        // Local file actions reveal their exact row. Do not later move back to the
        // first decoded canvas and hide those actions behind the footer again.
        if self.texture.is_some()
            && !self.local()
            && self.preview_reveal_frame == Some(current_frame)
        {
            preview.response.scroll_to_me(Some(egui::Align::Center));
        }
        if let Some(p) = &self.prepared {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&p.title);
                ui.small(format!(
                    "{} · {} {}",
                    if p.is_local() {
                        c::MEDIA_LOCAL
                    } else {
                        c::MEDIA_REMOTE
                    },
                    c::MEDIA_DURATION,
                    media_timeline::time(p.duration_ms)
                ));
            });
        }
        if let Some(prepared) = self.prepared.clone() {
            self.detail_open.show(
                ui,
                prepared,
                can_open && !busy && !self.closing && !self.fatal,
            );
        }
        if self.prepared.is_some() || busy {
            ui.horizontal_wrapped(|ui| {
                if self.prepared.is_some() {
                    if playback_button(ui, self.playing, busy).clicked() {
                        if self.playing {
                            self.cancel();
                        } else {
                            self.play();
                        }
                    }
                    ui.small(c::MEDIA_PREVIEW_SILENT);
                }
                if busy {
                    ui.weak(if self.prepared.is_some() {
                        c::MEDIA_FRAME_WAIT
                    } else {
                        c::MEDIA_WAIT
                    });
                    if crate::ui_components::ghost(ui, c::CANCEL, true).clicked() {
                        self.cancel();
                        self.error = Some(c::MEDIA_CANCELLED);
                    }
                }
            });
        }
        if self.texture.is_some()
            && let Some(error) = self.error
        {
            let message = ui.label(error);
            if self.tool_error() == Some(error) {
                self.tool_error_rect = Some(message.rect);
                self.tool_error_rect_reason = Some(error);
            }
        }
        if let Some(selection) = &mut self.selection {
            let event = media_timeline::show(ui, selection);
            if event.range {
                self.range_edit = Some(*selection);
            }
            let seek = event.seek;
            ui.horizontal_wrapped(|ui| {
                if ui
                    .scope_builder(
                        egui::UiBuilder::new().id("media-range-reset-action"),
                        |ui| crate::ui_components::ghost(ui, c::MEDIA_ALL, true),
                    )
                    .inner
                    .clicked()
                {
                    selection.select_all();
                    self.range_edit = Some(*selection);
                }
                ui.weak(format!(
                    "{} {}",
                    c::MEDIA_PLAYHEAD,
                    media_timeline::time(selection.playhead_ms())
                ));
            });
            if let Some(ms) = seek {
                self.seek(ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_tool_local_intent_survives_missing_tools_without_becoming_prepared() {
        let mut workspace = Workspace::default();
        let path = std::env::temp_dir().join("dlpad-synthetic-unopened-local.mp4");
        workspace.dropped(path.clone());
        workspace.poll(
            &egui::Context::default(),
            "",
            "",
            "",
            "",
            &std::env::temp_dir(),
        );
        assert!(workspace.has_local_intent());
        assert!(!workspace.local());
        assert!(workspace.prepared_for_export().is_none());
        assert!(workspace.selection().is_none());
        assert_eq!(
            workspace.tool_error(),
            Some(c::MEDIA_PREVIEW_FFMPEG_REQUIRED)
        );
        assert!(!workspace.busy());
        assert!(workspace.can_retry_after_tool_setup());
        assert!(workspace.retry_after_tool_setup());
        assert!(matches!(&workspace.pending,Some(Source::Local(actual))if actual==&path));
        assert!(
            workspace.tools.is_none(),
            "next normal worker must obtain fresh tool/local pins"
        );
        assert!(!workspace.local());
        assert!(workspace.selection().is_none());
        // Do not poll a configured worker: no real video/tool/process is opened here.
    }
    #[test]
    fn feature_tool_new_clear_or_explicit_invalid_url_cannot_retry_an_old_local_file() {
        let mut workspace = Workspace::default();
        let path = std::env::temp_dir().join("dlpad-synthetic-old-intent.mp4");
        workspace.dropped(path.clone());
        workspace.prepare_url("not-a-url");
        assert!(!workspace.has_local_intent());
        assert!(workspace.retry_source.is_none());
        assert!(workspace.pending.is_none());
        workspace.dropped(path.clone());
        workspace.source_changed("https://fixture.invalid/new-video");
        assert!(workspace.retry_source.is_none());
        assert!(workspace.pending.is_none());
        workspace.dropped(path);
        workspace.clear_source();
        assert!(workspace.retry_source.is_none());
        assert!(!workspace.can_retry_after_tool_setup());
        workspace.dropped(std::env::temp_dir().join("dlpad-synthetic-fatal-clear.mp4"));
        workspace.fatal = true;
        workspace.clear_source();
        assert!(workspace.retry_source.is_none() && workspace.pending.is_none());
        workspace.fatal = false;
        workspace.prepare_url("https://fixture.invalid/new-video");
        assert!(
            matches!(&workspace.retry_source,Some(Source::Url(url))if url=="https://fixture.invalid/new-video")
        );
        assert!(!workspace.has_local_intent());
    }
    #[test]
    fn feature_tool_retry_intent_obeys_source_byte_component_bounds_and_failure_kind() {
        let mut workspace = Workspace::default();
        let oversized = std::env::temp_dir().join("x".repeat(32768));
        workspace.dropped(oversized);
        assert!(workspace.retry_source.is_none());
        let mut deep = std::env::temp_dir();
        for _ in 0..257 {
            deep.push("x");
        }
        workspace.dropped(deep);
        assert!(workspace.retry_source.is_none());
        workspace.prepare_url(&format!("https://fixture.invalid/{}", "x".repeat(8192)));
        assert!(workspace.retry_source.is_none());
        workspace.dropped(std::env::temp_dir().join("dlpad-synthetic-unsupported.mp4"));
        workspace.pending = None;
        workspace.handle_failure(Error::UnsupportedMedia);
        assert!(workspace.source_unsupported());
        assert!(!workspace.tool_execution_failed());
        assert!(!workspace.can_retry_after_tool_setup());
        workspace.handle_failure(Error::ToolFailed);
        assert!(workspace.tool_execution_failed());
        assert!(workspace.can_retry_after_tool_setup());
    }
    #[test]
    fn feature_tool_setup_retry_refuses_to_replace_existing_selection() {
        let mut workspace = Workspace {
            retry_source: Some(Source::Local(
                std::env::temp_dir().join("dlpad-synthetic-range.mp4"),
            )),
            ..Default::default()
        };
        let mut selection = MediaSelection::new(10000).unwrap();
        selection.set_range(1500, 6500).unwrap();
        workspace.selection = Some(selection);
        workspace.handle_failure(Error::ToolFailed);
        assert!(!workspace.retry_after_tool_setup());
        assert_eq!(workspace.selection().unwrap().start_ms(), 1500);
        assert_eq!(workspace.selection().unwrap().end_ms(), 6500);
        assert!(workspace.pending.is_none());
    }
    #[test]
    fn actual_full_range_reset_republishes_full_selection_without_media_io() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut workspace = Workspace {
            selection: Some(MediaSelection::new(10_000).unwrap()),
            ..Default::default()
        };
        let draw = |workspace: &mut Workspace, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 700.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| workspace.show(ui));
                },
            )
        };
        draw(&mut workspace, vec![]);
        let out = draw(&mut workspace, vec![]);
        let (id, n) = out
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, n)| {
                n.role() == egui::accesskit::Role::Button && n.label() == Some(c::MEDIA_ALL)
            })
            .expect("actual Workspace reset command");
        assert!(
            !n.is_disabled(),
            "full valid selection may coexist with an invalid form draft; reset must stay usable"
        );
        draw(
            &mut workspace,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Click,
                    target: id,
                    data: None,
                },
            )],
        );
        let selection = workspace
            .take_range_edit()
            .expect("real reset publishes same full selection so parent can repair draft");
        assert!(selection.is_full());
        assert_eq!(selection.duration_ms(), 10_000);
        assert!(workspace.take_range_edit().is_none());
        assert!(
            workspace.worker.is_none(),
            "reset does not execute any media IO"
        );
    }

    #[test]
    fn actual_reset_entity_identity_survives_conditional_zoom_overview_and_routes_prior_action() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut s = MediaSelection::new(28_800_000).unwrap();
        s.set_range(14_000_000, 14_001_000).unwrap();
        s.seek(1000);
        let mut workspace = Workspace {
            selection: Some(s),
            ..Default::default()
        };
        let draw = |workspace: &mut Workspace, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 850.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| workspace.show(ui));
                },
            )
        };
        let action = |target| {
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target,
                data: None,
            })
        };
        draw(&mut workspace, vec![]);
        let out = draw(&mut workspace, vec![]);
        let nodes = out.platform_output.accesskit_update.unwrap().nodes;
        let reset = nodes
            .iter()
            .find(|(_, n)| {
                n.role() == egui::accesskit::Role::Button && n.label() == Some(c::MEDIA_ALL)
            })
            .unwrap()
            .0;
        let zoom = nodes
            .iter()
            .find(|(_, n)| {
                n.role() == egui::accesskit::Role::Button
                    && n.label() == Some(c::MEDIA_ZOOM_SELECTION)
            })
            .unwrap()
            .0;
        draw(&mut workspace, vec![action(zoom)]);
        let out = draw(&mut workspace, vec![]);
        let nodes = out.platform_output.accesskit_update.unwrap().nodes;
        let after = nodes
            .iter()
            .find(|(_, n)| {
                n.role() == egui::accesskit::Role::Button && n.label() == Some(c::MEDIA_ALL)
            })
            .unwrap()
            .0;
        assert_eq!(
            reset, after,
            "conditional overview/help layout must not change reset action identity"
        );
        assert_eq!(workspace.selection, Some(s));
        assert!(
            workspace.take_range_edit().is_none(),
            "view zoom alone cannot change savedrange"
        );
        draw(&mut workspace, vec![action(reset)]);
        let selected = workspace
            .take_range_edit()
            .expect("prior exact reset action still routes after overview reflow");
        assert!(selected.is_full());
        assert_eq!(selected.playhead_ms(), s.playhead_ms());
        assert!(workspace.worker.is_none());
    }

    #[test]
    fn playback_action_keeps_identity_when_parent_status_changes() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut previous = None;
        let mut accessible_id = None;
        for banner in [false, true, false] {
            let mut action_id = None;
            let mut clicked = false;
            let events = accessible_id
                .map(|target| {
                    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Click,
                        target,
                        data: None,
                    })
                })
                .into_iter()
                .collect();
            let output = ctx.run(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if banner {
                            ui.weak(c::MEDIA_WAIT);
                        }
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(400.0, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    let response = playback_button(ui, banner, banner);
                                    action_id = Some(response.id);
                                    clicked = response.clicked();
                                },
                            );
                        });
                    });
                },
            );
            let id = action_id.unwrap();
            if let Some(previous) = previous {
                assert_eq!(id, previous, "media action identity survives parent status");
                assert!(
                    clicked,
                    "previous accessible action still invokes the current control"
                );
            }
            let tree = output.platform_output.accesskit_update.unwrap();
            accessible_id = Some(
                tree.nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::Button
                            && node.label()
                                == Some(if banner {
                                    c::MEDIA_PAUSE
                                } else {
                                    c::MEDIA_PLAY
                                })
                    })
                    .unwrap()
                    .0,
            );
            previous = Some(id);
        }
    }
}
