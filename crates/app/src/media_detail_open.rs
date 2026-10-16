//! Explicit local-media handoff, off the UI thread. No tool, URL or shell command execution.
use crate::{
    catalog as c,
    library_open::{self, Error, Kind, Report},
};
use dlpad_engine::media_preview::Prepared;
use eframe::egui;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
type Outcome = Result<Report, Error>;
struct Running {
    stop: Arc<AtomicBool>,
    acknowledged: Arc<AtomicBool>,
    rx: Receiver<Outcome>,
    handle: JoinHandle<()>,
    source_changed: bool,
}
#[derive(Default)]
pub(crate) struct DetailOpen {
    running: Option<Running>,
    outcome: Option<Outcome>,
    actions_revealed: bool,
    actions_reveal_frame: Option<u64>,
    outcome_revealed: bool,
    outcome_reveal_frame: Option<u64>,
}
impl DetailOpen {
    pub(crate) fn busy(&self) -> bool {
        self.running.is_some()
    }
    pub(crate) fn stopped(&self) -> bool {
        self.running
            .as_ref()
            .is_none_or(|r| r.acknowledged.load(Ordering::Acquire))
    }
    pub(crate) fn request_stop(&mut self) {
        if let Some(r) = &self.running {
            r.stop.store(true, Ordering::Release);
        }
    }
    pub(crate) fn source_changed(&mut self) {
        self.outcome = None;
        self.actions_revealed = false;
        self.actions_reveal_frame = None;
        self.outcome_revealed = false;
        self.outcome_reveal_frame = None;
        if let Some(r) = &mut self.running {
            r.source_changed = true;
            r.stop.store(true, Ordering::Release);
        }
    }
    pub(crate) fn poll(&mut self) {
        let Some(r) = &self.running else {
            return;
        };
        // Never join an unfinished native picker. A timeout is cooperative, not a hard wall.
        if !r.handle.is_finished() {
            return;
        }
        let r = self.running.take().unwrap();
        let outcome = r.rx.try_recv().unwrap_or(Err(Error::Dispatch));
        let _ = r.handle.join();
        self.publish_outcome(if r.source_changed && outcome.is_ok() {
            Ok(Report::SourceChanged)
        } else {
            outcome
        });
    }
    fn publish_outcome(&mut self, outcome: Outcome) {
        self.outcome = Some(outcome);
        self.outcome_revealed = false;
        self.outcome_reveal_frame = None;
    }
    fn start(&mut self, prepared: Arc<Prepared>, kind: Kind) {
        if self.busy() || !prepared.is_local() || !library_open::available() {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let acknowledged = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker_ack = acknowledged.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        match thread::Builder::new()
            .name("media-detail-open".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    library_open::dispatch_prepared(&prepared, kind, &worker_stop)
                }))
                .unwrap_or(Err(Error::Dispatch));
                let _ = tx.send(result);
                worker_ack.store(true, Ordering::Release);
            }) {
            Ok(handle) => {
                self.outcome = None;
                self.outcome_revealed = false;
                self.outcome_reveal_frame = None;
                self.running = Some(Running {
                    stop,
                    acknowledged,
                    rx,
                    handle,
                    source_changed: false,
                });
            }
            Err(_) => self.publish_outcome(Err(Error::Dispatch)),
        }
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, prepared: Arc<Prepared>, enabled: bool) {
        let local = prepared.is_local();
        let enabled = enabled && !self.busy() && library_open::available();
        if let Some(kind) = self.show_state(ui, local, enabled) {
            self.start(prepared, kind);
        }
    }
    // This draws the actual production state; Context tests use synthetic outcomes
    // here without constructing a local file capability or starting a native viewer.
    fn show_state(&mut self, ui: &mut egui::Ui, local: bool, enabled: bool) -> Option<Kind> {
        let frame = ui.ctx().cumulative_frame_nr();
        let (action, action_row) = actions_with_rect(ui, local, enabled, self.busy());
        if local && !self.actions_revealed {
            self.actions_revealed = true;
            self.actions_reveal_frame = Some(frame);
        }
        if local && self.actions_reveal_frame == Some(frame) {
            ui.scroll_to_rect(action_row, None);
        }
        if !library_open::available() {
            ui.weak(c::LIBRARY_OPEN_UNSUPPORTED);
        }
        if self.busy() {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
        if let Some(outcome) = &self.outcome {
            let message = ui.add(
                egui::Label::new(match outcome {
                    Ok(Report::HandedOff) => c::LIBRARY_OPEN_HANDED_OFF,
                    Ok(Report::AfterCancellation | Report::AfterDeadline) => {
                        c::LIBRARY_OPEN_HANDED_OFF_LATE
                    }
                    Ok(Report::SourceChanged) => c::LIBRARY_OPEN_HANDED_OFF_CHANGED,
                    Err(Error::NoAssociation) => c::MEDIA_OPEN_NO_ASSOCIATION,
                    Err(Error::Unsupported) => c::LIBRARY_OPEN_UNSUPPORTED,
                    Err(Error::Cancelled) => c::LIBRARY_OPEN_CANCELLED,
                    Err(Error::TimedOut) => c::LIBRARY_OPEN_TIMEOUT,
                    Err(Error::AutomationBlocked) => c::LIBRARY_OPEN_AUTOMATION_BLOCKED,
                    Err(Error::Stale | Error::Unsafe) => c::LIBRARY_OPEN_CHANGED,
                    Err(Error::Dispatch) => c::LIBRARY_OPEN_FAILED,
                })
                .wrap(),
            );
            if !self.outcome_revealed {
                self.outcome_revealed = true;
                self.outcome_reveal_frame = Some(frame);
            }
            if self.outcome_reveal_frame == Some(frame) {
                // Exact complete Label response, never the whole preview or a
                // focus change. A later user scroll is not pulled back here.
                message.scroll_to_me(None);
            }
        }
        action
    }
}
impl Drop for DetailOpen {
    fn drop(&mut self) {
        self.request_stop();
    }
}

#[cfg(test)]
fn actions(ui: &mut egui::Ui, local: bool, enabled: bool, busy: bool) -> Option<Kind> {
    actions_with_rect(ui, local, enabled, busy).0
}
fn actions_with_rect(
    ui: &mut egui::Ui,
    local: bool,
    enabled: bool,
    busy: bool,
) -> (Option<Kind>, egui::Rect) {
    if !local {
        return (None, ui.weak(c::MEDIA_OPEN_REMOTE).rect);
    }
    let mut action = None;
    let row = ui.horizontal_wrapped(|ui| {
        if crate::ui_components::secondary(ui, c::LIBRARY_OPEN_DEFAULT, enabled)
            .on_hover_text(c::MEDIA_OPEN_DEFAULT_HELP)
            .on_disabled_hover_text(if busy {
                c::MEDIA_OPEN_BUSY
            } else {
                c::MEDIA_OPEN_BLOCKED
            })
            .clicked()
        {
            action = Some(Kind::File);
        }
        if crate::ui_components::secondary(ui, c::MEDIA_OPEN_CHOOSE, enabled)
            .on_hover_text(c::MEDIA_OPEN_CHOOSE_HELP)
            .on_disabled_hover_text(if busy {
                c::MEDIA_OPEN_BUSY
            } else {
                c::MEDIA_OPEN_BLOCKED
            })
            .clicked()
        {
            action = Some(Kind::ChooseApplication);
        }
        if crate::ui_components::secondary(ui, c::MEDIA_REVEAL_FILE, enabled)
            .on_hover_text(c::MEDIA_REVEAL_HELP)
            .on_disabled_hover_text(if busy {
                c::MEDIA_OPEN_BUSY
            } else {
                c::MEDIA_OPEN_BLOCKED
            })
            .clicked()
        {
            action = Some(Kind::RevealFile);
        }
        if busy {
            ui.spinner();
            ui.weak(c::MEDIA_OPEN_BUSY);
        }
    });
    (action, row.response.rect)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_detail_open_actions_route_once_and_disabled_remote_never_dispatch() {
        for scale in [1.0, 2.0] {
            for dark in [false, true] {
                let ctx = egui::Context::default();
                crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
                ctx.set_pixels_per_point(scale);
                ctx.set_visuals(if dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                ctx.enable_accesskit();
                let frame = |local, enabled, events| {
                    let mut action = None;
                    let out = ctx.run(
                        egui::RawInput {
                            focused: true,
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(640.0, 480.0),
                            )),
                            events,
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default()
                                .show(ctx, |ui| action = actions(ui, local, enabled, false));
                        },
                    );
                    (out, action)
                };
                for (label, expected) in [
                    (c::LIBRARY_OPEN_DEFAULT, Kind::File),
                    (c::MEDIA_OPEN_CHOOSE, Kind::ChooseApplication),
                    (c::MEDIA_REVEAL_FILE, Kind::RevealFile),
                ] {
                    frame(true, true, vec![]);
                    let (out, _) = frame(true, true, vec![]);
                    let (id, node) = out
                        .platform_output
                        .accesskit_update
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, n)| {
                            n.role() == egui::accesskit::Role::Button && n.label() == Some(label)
                        })
                        .unwrap();
                    assert!(!node.is_disabled());
                    let click = || {
                        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                            target: id,
                            action: egui::accesskit::Action::Click,
                            data: None,
                        })
                    };
                    assert_eq!(frame(true, true, vec![click()]).1, Some(expected));
                    assert_eq!(frame(true, true, vec![]).1, None);
                    assert_eq!(frame(true, false, vec![click()]).1, None);
                    assert_eq!(frame(false, true, vec![click()]).1, None);
                }
            }
        }
    }
}

#[cfg(test)]
mod reveal_tests {
    use super::*;

    struct View {
        ctx: egui::Context,
        open: DetailOpen,
        screen: egui::Rect,
        body: egui::Rect,
        offset: f32,
        toolbar: egui::Id,
        clock: f64,
    }
    impl View {
        fn new(scale: f32, dark: bool) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            // set_pixels_per_point applies zoom next pass using the previous
            // input content rect. Seed the actual window before changing zoom;
            // otherwise egui rescales its default 10000-point viewport and
            // consumes initial reveal against an artificial 5000-point body.
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    time: Some(0.0),
                    focused: true,
                    ..Default::default()
                },
                |_| {},
            );
            ctx.set_pixels_per_point(scale);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            ctx.enable_accesskit();
            Self {
                ctx,
                open: DetailOpen::default(),
                screen: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0 / scale, 600.0 / scale),
                ),
                body: egui::Rect::NOTHING,
                offset: 0.0,
                toolbar: egui::Id::NULL,
                clock: 0.0,
            }
        }
        fn frame(&mut self, offset: Option<f32>) -> egui::FullOutput {
            self.clock += 0.5;
            self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(self.screen),
                    time: Some(self.clock),
                    focused: true,
                    ..Default::default()
                },
                |ctx| {
                    egui::TopBottomPanel::top("detail-toolbar")
                        .frame(egui::Frame::NONE)
                        .exact_height(40.0)
                        .show(ctx, |ui| {
                            self.toolbar = crate::ui_components::secondary(ui, "Toolbar", true).id;
                        });
                    egui::TopBottomPanel::bottom("detail-footer")
                        .frame(egui::Frame::NONE)
                        .exact_height(90.0)
                        .show(ctx, |ui| {
                            crate::ui_components::secondary(ui, "Footer", true);
                        });
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::central_panel(&ctx.style())
                                .inner_margin(crate::ui_tokens::PAGE_MARGIN),
                        )
                        .show(ctx, |ui| {
                            let mut area = egui::ScrollArea::vertical()
                                .id_salt("detail-body")
                                .auto_shrink([false, false])
                                .min_scrolled_height(0.0);
                            if let Some(offset) = offset {
                                area = area.vertical_scroll_offset(offset);
                            }
                            let output = area.show(ui, |ui| {
                                ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 32.0),
                                    egui::Sense::hover(),
                                );
                                let height = (ui.clip_rect().height() - 8.0).clamp(90.0, 200.0);
                                ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), height),
                                    egui::Sense::hover(),
                                );
                                ui.label("Synthetic local media");
                                assert_eq!(
                                    self.open.show_state(ui, true, true),
                                    None,
                                    "drawing a result never starts a viewer"
                                );
                                ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 80.0),
                                    egui::Sense::hover(),
                                );
                            });
                            self.body = output.inner_rect;
                            self.offset = output.state.offset.y;
                        });
                },
            )
        }
        fn settle(&mut self) -> egui::FullOutput {
            self.frame(None);
            self.frame(None);
            self.frame(None)
        }
        fn focus_toolbar(&self) {
            self.ctx
                .memory_mut(|memory| memory.request_focus(self.toolbar));
        }
    }

    // Paint acceptance checks the complete production Galley rows inside the
    // actual body and shape clip. AccessKit's clipped node bounds are insufficient.
    fn all_rows_painted(output: &egui::FullOutput, text: &str, body: egui::Rect) -> bool {
        fn shape_has(
            shape: &egui::epaint::Shape,
            clip: egui::Rect,
            text: &str,
            body: egui::Rect,
        ) -> bool {
            match shape {
                egui::epaint::Shape::Text(shape) if shape.galley.text() == text => {
                    !shape.galley.elided
                        && !shape.galley.rows.is_empty()
                        && shape.galley.rows.iter().all(|row| {
                            let rect = row.rect().translate(shape.pos.to_vec2());
                            clip.contains_rect(rect) && body.contains_rect(rect)
                        })
                }
                egui::epaint::Shape::Vec(shapes) => shapes
                    .iter()
                    .any(|shape| shape_has(shape, clip, text, body)),
                _ => false,
            }
        }
        output
            .shapes
            .iter()
            .any(|shape| shape_has(&shape.shape, shape.clip_rect, text, body))
    }

    #[test]
    fn media_detail_reveal_actual_compact_actions_and_new_outcome_paint_without_focus_theft() {
        for scale in [1.0, 2.0] {
            for dark in [false, true] {
                let mut view = View::new(scale, dark);
                let initial = view.settle();
                assert!(all_rows_painted(
                    &initial,
                    c::LIBRARY_OPEN_DEFAULT,
                    view.body
                ));
                assert!(all_rows_painted(&initial, c::MEDIA_OPEN_CHOOSE, view.body));
                assert!(all_rows_painted(&initial, c::MEDIA_REVEAL_FILE, view.body));
                view.focus_toolbar();
                view.frame(None);
                view.open.publish_outcome(Err(Error::AutomationBlocked));
                let feedback = view.settle();
                assert!(
                    all_rows_painted(&feedback, c::LIBRARY_OPEN_AUTOMATION_BLOCKED, view.body),
                    "all error glyph rows clear the actual body/footer clip at scale {scale}"
                );
                assert_eq!(
                    view.ctx.memory(|memory| memory.focused()),
                    Some(view.toolbar)
                );
                assert!(
                    view.open.running.is_none(),
                    "synthetic state draws without a process/thread"
                );
            }
        }
    }

    #[test]
    fn media_detail_reveal_once_respects_manual_scroll_and_repeated_equal_failure_episode() {
        for scale in [1.0, 2.0] {
            let mut view = View::new(scale, false);
            view.settle();
            view.focus_toolbar();
            view.open.publish_outcome(Err(Error::NoAssociation));
            let first = view.settle();
            assert!(all_rows_painted(
                &first,
                c::MEDIA_OPEN_NO_ASSOCIATION,
                view.body
            ));
            if scale == 2.0 {
                view.frame(Some(0.0));
                let manual = view.settle();
                assert_eq!(
                    view.offset, 0.0,
                    "stable outcome does not pull manual scroll back"
                );
                assert!(!all_rows_painted(
                    &manual,
                    c::MEDIA_OPEN_NO_ASSOCIATION,
                    view.body
                ));
                // A second completed request with the exact same error is a new
                // episode. The production publish path resets reveal state.
                view.open.publish_outcome(Err(Error::NoAssociation));
                let second = view.settle();
                assert!(all_rows_painted(
                    &second,
                    c::MEDIA_OPEN_NO_ASSOCIATION,
                    view.body
                ));
                assert!(view.offset > 0.0);
            }
            assert_eq!(
                view.ctx.memory(|memory| memory.focused()),
                Some(view.toolbar)
            );
        }
    }

    #[test]
    fn media_detail_reveal_source_reset_clears_outcome_and_reveals_actions_again() {
        let mut view = View::new(2.0, true);
        view.settle();
        view.open.publish_outcome(Err(Error::AutomationBlocked));
        view.settle();
        view.frame(Some(0.0));
        view.open.source_changed();
        let output = view.settle();
        assert!(view.open.outcome.is_none());
        assert!(all_rows_painted(
            &output,
            c::LIBRARY_OPEN_DEFAULT,
            view.body
        ));
        assert!(all_rows_painted(&output, c::MEDIA_OPEN_CHOOSE, view.body));
        assert!(all_rows_painted(&output, c::MEDIA_REVEAL_FILE, view.body));
        assert!(view.open.running.is_none());
    }
}
