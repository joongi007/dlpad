//! One selection band, independent playhead, visible draggable boundaries and keyboard controls.
use crate::catalog as c;
use dlpad_core::media_selection::MediaSelection;
use eframe::egui::{self, Key, Response, Sense, Stroke};
#[derive(Default)]
pub(crate) struct Change {
    pub range: bool,
    pub seek: Option<u64>,
}
pub(crate) fn time(ms: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        (ms / 60_000) % 60,
        (ms / 1000) % 60,
        ms % 1000
    )
}
fn position(track: egui::Rect, x: f32, duration: u64) -> u64 {
    if !x.is_finite() || track.width() <= 0.0 {
        return 0;
    }
    (((x - track.left()) / track.width()).clamp(0.0, 1.0) as f64 * duration as f64).round() as u64
}
fn keyboard(ui: &egui::Ui, r: &Response, value: u64, max: u64) -> Option<u64> {
    if !r.enabled() || !r.has_focus() {
        return None;
    }
    ui.ctx().memory_mut(|m| {
        m.set_focus_lock_filter(
            r.id,
            egui::EventFilter {
                horizontal_arrows: true,
                ..Default::default()
            },
        )
    });
    ui.input(|i| {
        let step = if i.modifiers.shift { 1000 } else { 100 };
        if i.key_pressed(Key::Home) {
            Some(0)
        } else if i.key_pressed(Key::End) {
            Some(max)
        } else if i.key_pressed(Key::ArrowRight) {
            Some(value.saturating_add(step).min(max))
        } else if i.key_pressed(Key::ArrowLeft) {
            Some(value.saturating_sub(step))
        } else {
            None
        }
    })
}
fn thumb(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: egui::Id,
    label: &str,
    value: u64,
    max: u64,
    interaction: (bool, egui::CursorIcon, &str),
) -> (Response, Option<u64>) {
    let (pointer_active, cursor, help) = interaction;
    let mut r = ui.interact(
        rect,
        id,
        if pointer_active {
            Sense::click_and_drag()
        } else {
            Sense::click()
        },
    );
    if r.clicked() || r.drag_started() {
        r.request_focus();
    }
    r.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), value as f64, label));
    if !r.enabled() {
        return (r, None);
    }
    if r.gained_focus() {
        r.scroll_to_me(None);
    }
    let mut update = keyboard(ui, &r, value, max);
    ui.ctx().accesskit_node_builder(id, |node| {
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(max as f64);
        node.set_numeric_value_step(100.0);
        node.add_action(egui::accesskit::Action::SetValue);
        node.add_action(egui::accesskit::Action::Increment);
        node.add_action(egui::accesskit::Action::Decrement);
    });
    ui.input(|i| {
        for event in i.accesskit_action_requests(id, egui::accesskit::Action::SetValue) {
            if let Some(egui::accesskit::ActionData::NumericValue(v)) = &event.data
                && v.is_finite()
                && *v >= 0.0
                && *v <= max as f64
            {
                update = Some(v.round() as u64);
            }
        }
        let inc = i.num_accesskit_action_requests(id, egui::accesskit::Action::Increment);
        let dec = i.num_accesskit_action_requests(id, egui::accesskit::Action::Decrement);
        if inc > 0 {
            update = Some(value.saturating_add(100).min(max));
        } else if dec > 0 {
            update = Some(value.saturating_sub(100));
        }
    });
    if update.is_some() {
        r.mark_changed();
    }
    (
        r.on_hover_cursor(if pointer_active {
            cursor
        } else {
            egui::CursorIcon::Default
        })
        .on_hover_text(help),
        update,
    )
}
#[derive(Clone, Copy)]
struct ViewWindow {
    duration: u64,
    start: u64,
    end: u64,
}
impl ViewWindow {
    fn full(duration: u64) -> Self {
        Self {
            duration,
            start: 0,
            end: duration,
        }
    }
    fn selection(s: MediaSelection) -> Self {
        let span = (s.end_ms() - s.start_ms())
            .saturating_mul(3)
            .max(10_000)
            .min(s.duration_ms());
        let center = s.start_ms() + (s.end_ms() - s.start_ms()) / 2;
        let start = center.saturating_sub(span / 2).min(s.duration_ms() - span);
        Self {
            duration: s.duration_ms(),
            start,
            end: start + span,
        }
    }
    fn contains(self, value: u64) -> bool {
        value >= self.start && value <= self.end
    }
    fn x(self, track: egui::Rect, value: u64) -> f32 {
        track.left()
            + track.width()
                * ((value.clamp(self.start, self.end) - self.start) as f64
                    / (self.end - self.start) as f64) as f32
    }
    fn position(self, track: egui::Rect, x: f32) -> u64 {
        self.start + position(track, x, self.end - self.start)
    }
}
pub(crate) fn show(ui: &mut egui::Ui, s: &mut MediaSelection) -> Change {
    let before = *s;
    let mut change = Change::default();
    ui.horizontal_wrapped(|ui| {
        ui.strong(c::MEDIA_RANGE);
        if s.is_full() {
            ui.weak(c::MEDIA_FULL_RANGE_SELECTED);
        } else {
            ui.weak(format!("{} – {}", time(s.start_ms()), time(s.end_ms())));
        }
    });
    let viewport_id = ui.id().with("media-timeline-window");
    let duration = s.duration_ms();
    let mut window = ui
        .ctx()
        .data_mut(|data| data.get_temp::<ViewWindow>(viewport_id))
        .filter(|v| v.duration == duration)
        .unwrap_or_else(|| ViewWindow::full(duration));
    ui.horizontal_wrapped(|ui| {
        if crate::ui_components::ghost(ui, c::MEDIA_ZOOM_SELECTION, true).clicked() {
            window = ViewWindow::selection(*s);
        }
        if crate::ui_components::ghost(ui, c::MEDIA_SHOW_ALL, true).clicked() {
            window = ViewWindow::full(duration);
        }
    });
    ui.ctx()
        .data_mut(|data| data.insert_temp(viewport_id, window));
    // The full overview only adds context when the main time axis is zoomed.
    // In the normal view it repeated the same entire-video axis twice.
    if window.start != 0 || window.end != duration {
        ui.weak(c::MEDIA_OVERVIEW_CONTEXT);
        let (overview, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), Sense::hover());
        let overview_x = |value: u64| {
            overview.left() + overview.width() * (value as f64 / duration as f64) as f32
        };
        ui.painter()
            .rect_filled(overview, 2, ui.visuals().faint_bg_color);
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(overview_x(s.start_ms()), overview.top()),
                egui::pos2(
                    overview_x(s.end_ms()).max(overview_x(s.start_ms()) + 1.0),
                    overview.bottom(),
                ),
            ),
            2,
            crate::ui_theme::ACCENT.gamma_multiply(0.45),
        );
        ui.painter().rect_stroke(
            egui::Rect::from_min_max(
                egui::pos2(overview_x(window.start), overview.top()),
                egui::pos2(overview_x(window.end), overview.bottom()),
            ),
            2,
            Stroke::new(1.0, ui.visuals().text_color()),
            egui::StrokeKind::Inside,
        );
        ui.label(c::MEDIA_OVERVIEW_HELP);
    }
    ui.weak(c::MEDIA_RANGE_TIMELINE_HELP);
    ui.weak(c::MEDIA_VISIBLE_WINDOW);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 96.0), Sense::hover());
    let track = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 24.0, rect.top() + 32.0),
        egui::pos2(
            (rect.right() - 24.0).max(rect.left() + 25.0),
            rect.top() + 48.0,
        ),
    );
    let x = |value: u64| window.x(track, value);
    let start_x = x(s.start_ms());
    let end_x = x(s.end_ms());
    let (band, khead) = thumb(
        ui,
        track.expand2(egui::vec2(0.0, 8.0)),
        ui.id().with("media-playhead"),
        c::MEDIA_PLAYHEAD,
        s.playhead_ms(),
        s.duration_ms() - 1,
        (
            true,
            egui::CursorIcon::PointingHand,
            c::MEDIA_TIMELINE_SEEK_HELP,
        ),
    );
    let (start, kstart) = thumb(
        ui,
        egui::Rect::from_min_max(
            egui::pos2(start_x - 16.0, track.top() - 32.0),
            egui::pos2(start_x + 16.0, track.top()),
        ),
        ui.id().with("range-start"),
        c::RANGE_START,
        s.start_ms(),
        s.duration_ms(),
        (
            window.contains(s.start_ms()),
            egui::CursorIcon::ResizeHorizontal,
            c::MEDIA_RANGE_START_HANDLE_HELP,
        ),
    );
    let (end, kend) = thumb(
        ui,
        egui::Rect::from_min_max(
            egui::pos2(end_x - 16.0, track.bottom()),
            egui::pos2(end_x + 16.0, track.bottom() + 32.0),
        ),
        ui.id().with("range-end"),
        c::RANGE_END,
        s.end_ms(),
        s.duration_ms(),
        (
            window.contains(s.end_ms()),
            egui::CursorIcon::ResizeHorizontal,
            c::MEDIA_RANGE_END_HANDLE_HELP,
        ),
    );
    if let Some(v) = kstart {
        s.move_start(v);
    }
    if let Some(v) = kend {
        s.move_end(v);
    }
    if window.contains(before.start_ms())
        && start.dragged()
        && let Some(p) = start.interact_pointer_pos()
    {
        s.move_start(window.position(track, p.x));
    }
    if window.contains(before.end_ms())
        && end.dragged()
        && let Some(p) = end.interact_pointer_pos()
    {
        s.move_end(window.position(track, p.x));
    }
    if (band.clicked() || band.dragged())
        && !start.dragged()
        && !end.dragged()
        && let Some(p) = band.interact_pointer_pos()
    {
        s.seek(window.position(track, p.x));
        change.seek = Some(s.playhead_ms());
    }
    if let Some(v) = khead {
        s.seek(v);
        change.seek = Some(s.playhead_ms());
    }
    band.widget_info(|| {
        egui::WidgetInfo::slider(ui.is_enabled(), s.playhead_ms() as f64, c::MEDIA_PLAYHEAD)
    });
    let normal = ui.visuals().faint_bg_color;
    let accent = crate::ui_theme::ACCENT;
    ui.painter().rect_filled(track, 4, normal);
    if band.enabled() && band.hovered() {
        ui.painter().rect_stroke(
            track,
            4,
            Stroke::new(1.0, crate::ui_theme::ACCENT_HOVER),
            egui::StrokeKind::Inside,
        );
    }
    if band.enabled() && band.has_focus() {
        ui.painter().rect_stroke(
            track.expand(3.0),
            4,
            Stroke::new(2.0, crate::ui_theme::ACCENT),
            egui::StrokeKind::Outside,
        );
    }
    let select = egui::Rect::from_min_max(
        egui::pos2(x(s.start_ms()), track.top()),
        egui::pos2(x(s.end_ms()), track.bottom()),
    );
    ui.painter().rect_filled(
        select,
        3,
        if ui.is_enabled() {
            accent.gamma_multiply(0.45)
        } else {
            normal
        },
    );
    for (value, r, upper) in [(s.start_ms(), &start, true), (s.end_ms(), &end, false)] {
        let cx = x(value);
        let handle = egui::Rect::from_center_size(
            egui::pos2(
                cx,
                if upper {
                    track.top() - 14.0
                } else {
                    track.bottom() + 14.0
                },
            ),
            egui::vec2(8.0, 28.0),
        );
        let fill = if !ui.is_enabled() || !window.contains(value) {
            ui.visuals().weak_text_color()
        } else if r.is_pointer_button_down_on() {
            ui.visuals().text_color()
        } else if r.hovered() {
            crate::ui_theme::ACCENT_HOVER
        } else {
            accent
        };
        ui.painter().rect_filled(handle, 3, fill);
        if r.enabled() && r.has_focus() {
            ui.painter().rect_stroke(
                handle.expand(3.0),
                4,
                Stroke::new(2.0, accent),
                egui::StrokeKind::Outside,
            );
        }
    }
    let hx = x(s.playhead_ms());
    if window.contains(s.playhead_ms()) {
        ui.painter().line_segment(
            [
                egui::pos2(hx, track.top() - 10.0),
                egui::pos2(hx, track.bottom() + 2.0),
            ],
            Stroke::new(2.0, ui.visuals().text_color()),
        );
    }
    ui.painter().text(
        egui::pos2(track.left(), rect.bottom()),
        egui::Align2::LEFT_BOTTOM,
        time(window.start),
        egui::FontId::proportional(crate::ui_tokens::SMALL_FONT_SIZE),
        ui.visuals().weak_text_color(),
    );
    ui.painter().text(
        egui::pos2(track.right(), rect.bottom()),
        egui::Align2::RIGHT_BOTTOM,
        time(window.end),
        egui::FontId::proportional(crate::ui_tokens::SMALL_FONT_SIZE),
        ui.visuals().weak_text_color(),
    );
    change.range = before.start_ms() != s.start_ms() || before.end_ms() != s.end_ms();
    if change.range || change.seek.is_some() {
        ui.ctx().request_repaint();
    }
    change
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        selection: &mut MediaSelection,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        frame_enabled(ctx, selection, events, true)
    }
    fn frame_enabled(
        ctx: &egui::Context,
        selection: &mut MediaSelection,
        events: Vec<egui::Event>,
        enabled: bool,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 320.0),
                )),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(enabled, |ui| {
                        show(ui, selection);
                    });
                });
            },
        )
    }
    fn key(key: Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn range_ux_frame(
        ctx: &egui::Context,
        selection: &mut MediaSelection,
        events: Vec<egui::Event>,
        enabled: bool,
    ) -> (egui::FullOutput, Change) {
        let mut change = Change::default();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(480.0, 480.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(enabled, |ui| {
                        change = show(ui, selection);
                    });
                });
            },
        );
        (output, change)
    }
    fn range_ux_context(dark: bool) -> egui::Context {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        crate::ui_theme::configure_dlpad_theme(&ctx);
        ctx.enable_accesskit();
        ctx
    }
    #[test]
    fn actual_seek_cursor_hit_area_and_handle_drag_keep_distinct_meanings() {
        for dark in [false, true] {
            let ctx = range_ux_context(dark);
            let mut s = MediaSelection::new(10_000).unwrap();
            s.set_range(2000, 8000).unwrap();
            s.seek(7000);
            range_ux_frame(&ctx, &mut s, vec![], true);
            let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            let (_, band) = node(out, c::MEDIA_PLAYHEAD);
            let seek = egui::pos2(
                (band.x0 + band.x1) as f32 * 0.5,
                (band.y0 + band.y1) as f32 * 0.5,
            );
            assert_eq!(
                band.y1 - band.y0,
                32.0,
                "real accessible seek hit target is32 logical pixels"
            );
            range_ux_frame(&ctx, &mut s, vec![egui::Event::PointerMoved(seek)], true);
            let (hover, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            assert_eq!(
                hover.platform_output.cursor_icon,
                egui::CursorIcon::PointingHand,
                "seek is a target, not a range resize handle"
            );
            let before = s;
            range_ux_frame(&ctx, &mut s, vec![pointer(seek, true)], true);
            let (_, event) = range_ux_frame(&ctx, &mut s, vec![pointer(seek, false)], true);
            assert!(event.seek.is_some());
            assert_eq!(
                (s.start_ms(), s.end_ms()),
                (before.start_ms(), before.end_ms()),
                "click seek preserves saved range"
            );
            assert!(s.playhead_ms().abs_diff(5000) < 100);
            let playhead = s.playhead_ms();
            let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            let (_, start) = node(out, c::RANGE_START);
            assert_eq!(start.y1 - start.y0, 32.0);
            let at = center(start);
            range_ux_frame(&ctx, &mut s, vec![egui::Event::PointerMoved(at)], true);
            let (hover, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            assert_eq!(
                hover.platform_output.cursor_icon,
                egui::CursorIcon::ResizeHorizontal
            );
            range_ux_frame(&ctx, &mut s, vec![pointer(at, true)], true);
            let moved = at + egui::vec2(30.0, 0.0);
            let (_, event) =
                range_ux_frame(&ctx, &mut s, vec![egui::Event::PointerMoved(moved)], true);
            range_ux_frame(&ctx, &mut s, vec![pointer(moved, false)], true);
            assert!(event.range);
            assert!(s.start_ms() > before.start_ms());
            assert_eq!(s.end_ms(), before.end_ms());
            assert_eq!(
                s.playhead_ms(),
                playhead,
                "range adjustment never becomes seek"
            );
            let before = s;
            let (out, _) =
                range_ux_frame(&ctx, &mut s, vec![egui::Event::PointerMoved(seek)], false);
            let (_, event) = range_ux_frame(
                &ctx,
                &mut s,
                vec![pointer(seek, true), pointer(seek, false)],
                false,
            );
            assert_eq!(s, before);
            assert!(event.seek.is_none() && !event.range);
            assert_eq!(
                out.platform_output.cursor_icon,
                egui::CursorIcon::Default,
                "disabled target does not promise interaction"
            );
        }
    }
    #[test]
    fn actual_full_axis_avoids_duplicate_overview_and_zoom_keeps_context_without_range_mutation() {
        for dark in [false, true] {
            let ctx = range_ux_context(dark);
            let mut s = MediaSelection::new(28_800_000).unwrap();
            s.set_range(14_000_000, 14_001_000).unwrap();
            range_ux_frame(&ctx, &mut s, vec![], true);
            let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            let painted = |out: &egui::FullOutput, caption: &str| {
                out.shapes.iter().any(|shape|matches!(&shape.shape,egui::epaint::Shape::Text(text) if text.galley.text()==caption))
            };
            assert!(
                !painted(&out, c::MEDIA_OVERVIEW_CONTEXT),
                "full time axis should not render a second entire-video overview"
            );
            assert!(
                painted(&out, c::MEDIA_RANGE_TIMELINE_HELP),
                "seek/drag/time-entry explanation is actual painted copy"
            );
            let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            let (_, zoom) = node(out, c::MEDIA_ZOOM_SELECTION);
            let at = center(zoom);
            let before = s;
            range_ux_frame(
                &ctx,
                &mut s,
                vec![egui::Event::PointerMoved(at), pointer(at, true)],
                true,
            );
            range_ux_frame(&ctx, &mut s, vec![pointer(at, false)], true);
            let (out, event) = range_ux_frame(&ctx, &mut s, vec![], true);
            assert!(
                painted(&out, c::MEDIA_OVERVIEW_CONTEXT),
                "zoom preserves labelled entire-video orientation"
            );
            assert_eq!(s, before);
            assert!(
                !event.range && event.seek.is_none(),
                "zoom only changes view, not selection/playhead"
            );
            let (_, all) = node(out, c::MEDIA_SHOW_ALL);
            let at = center(all);
            range_ux_frame(
                &ctx,
                &mut s,
                vec![egui::Event::PointerMoved(at), pointer(at, true)],
                true,
            );
            range_ux_frame(&ctx, &mut s, vec![pointer(at, false)], true);
            let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
            assert!(!painted(&out, c::MEDIA_OVERVIEW_CONTEXT));
            assert_eq!(s, before);
        }
    }

    #[test]
    fn actual_timeline_targets_remain_inside_painter_clip_at_both_zoom_factors() {
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                let ctx = range_ux_context(dark);
                ctx.set_zoom_factor(zoom);
                let mut s = MediaSelection::new(10_000).unwrap();
                s.set_range(2000, 8000).unwrap();
                range_ux_frame(&ctx, &mut s, vec![], true);
                let (out, _) = range_ux_frame(&ctx, &mut s, vec![], true);
                let nodes = out.platform_output.accesskit_update.as_ref().unwrap();
                for caption in [c::MEDIA_PLAYHEAD, c::RANGE_START, c::RANGE_END] {
                    let b = nodes
                        .nodes
                        .iter()
                        .find(|(_, n)| n.label() == Some(caption))
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    let rect = egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    );
                    assert_eq!(
                        rect.height(),
                        32.0,
                        "actual logical target size for {caption}"
                    );
                    assert!(
                        rect.left() >= 0.0
                            && rect.right() <= 480.0
                            && rect.top() >= 0.0
                            && rect.bottom() <= 480.0,
                        "actual target contained in Context logical viewport: {rect:?}"
                    );
                }
                let b = nodes
                    .nodes
                    .iter()
                    .find(|(_, n)| n.label() == Some(c::MEDIA_PLAYHEAD))
                    .unwrap()
                    .1
                    .bounds()
                    .unwrap();
                let track = egui::Rect::from_min_max(
                    egui::pos2(b.x0 as f32, b.y0 as f32 + 8.0),
                    egui::pos2(b.x1 as f32, b.y1 as f32 - 8.0),
                );
                assert!(out.shapes.iter().any(|shape|matches!(&shape.shape,egui::epaint::Shape::Rect(r) if r.rect==track&&shape.clip_rect.contains_rect(track))),"actual center track is painted fully within its real clip at zoom{zoom}");
            }
        }
    }

    #[test]
    fn disabled_focused_timeline_rejects_keyboard_and_accessibility_edits() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut selection = MediaSelection::new(10_000).unwrap();
        selection.seek(7000);
        frame(&ctx, &mut selection, vec![]);
        let output = frame(&ctx, &mut selection, vec![]);
        let tree = output.platform_output.accesskit_update.unwrap();
        let (node_id, _) = tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some(c::RANGE_START))
            .unwrap();
        // Focus via the real hit region, then disable the already focused control.
        let bounds = tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some(c::RANGE_START))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let pos = egui::pos2(
            (bounds.x0 + bounds.x1) as f32 * 0.5,
            (bounds.y0 + bounds.y1) as f32 * 0.5,
        );
        let mouse = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut selection,
            vec![egui::Event::PointerMoved(pos), mouse(true)],
        );
        frame(&ctx, &mut selection, vec![mouse(false)]);
        let before = selection;
        frame_enabled(
            &ctx,
            &mut selection,
            vec![
                key(Key::ArrowRight),
                egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::SetValue,
                    target: *node_id,
                    data: Some(egui::accesskit::ActionData::NumericValue(5000.0)),
                }),
            ],
            false,
        );
        assert_eq!(selection, before);
    }
    #[test]
    fn real_handle_pointer_drag_and_focused_keyboard_preserve_playhead() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut selection = MediaSelection::new(10_000).unwrap();
        selection.seek(7000);
        frame(&ctx, &mut selection, vec![]);
        let output = frame(&ctx, &mut selection, vec![]);
        let tree = output.platform_output.accesskit_update.unwrap();
        let bounds = tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some(c::RANGE_START))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let start = egui::pos2(
            (bounds.x0 + bounds.x1) as f32 * 0.5,
            (bounds.y0 + bounds.y1) as f32 * 0.5,
        );
        let mouse = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut selection,
            vec![egui::Event::PointerMoved(start), mouse(start, true)],
        );
        let moved = start + egui::vec2(100.0, 0.0);
        frame(&ctx, &mut selection, vec![egui::Event::PointerMoved(moved)]);
        frame(&ctx, &mut selection, vec![mouse(moved, false)]);
        assert!(selection.start_ms() > 2000 && selection.start_ms() < 4000);
        assert_eq!(selection.playhead_ms(), 7000);
        let before = selection.start_ms();
        frame(&ctx, &mut selection, vec![key(Key::ArrowRight)]);
        assert_eq!(selection.start_ms(), before + 100);
        assert_eq!(selection.playhead_ms(), 7000);
        frame(&ctx, &mut selection, vec![key(Key::End)]);
        assert_eq!(selection.start_ms(), selection.end_ms() - 1);
    }
    fn node(
        output: egui::FullOutput,
        label: &str,
    ) -> (egui::accesskit::NodeId, egui::accesskit::Rect) {
        let tree = output.platform_output.accesskit_update.unwrap();
        let (id, node) = tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some(label))
            .unwrap();
        (*id, node.bounds().unwrap())
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn center(bounds: egui::accesskit::Rect) -> egui::Pos2 {
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }
    #[test]
    fn long_media_zoom_separates_handles_and_pointer_moves_in_visible_window() {
        for duration in [8 * 60 * 60 * 1000, 7 * 24 * 60 * 60 * 1000] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut selection = MediaSelection::new(duration).unwrap();
            selection
                .set_range(duration / 2, duration / 2 + 100)
                .unwrap();
            selection.seek(1234);
            frame(&ctx, &mut selection, vec![]);
            let (_, zoom) = node(frame(&ctx, &mut selection, vec![]), c::MEDIA_ZOOM_SELECTION);
            let zoom = center(zoom);
            frame(
                &ctx,
                &mut selection,
                vec![egui::Event::PointerMoved(zoom), pointer(zoom, true)],
            );
            frame(&ctx, &mut selection, vec![pointer(zoom, false)]);
            let (_, start) = node(frame(&ctx, &mut selection, vec![]), c::RANGE_START);
            let (_, end) = node(frame(&ctx, &mut selection, vec![]), c::RANGE_END);
            assert!(
                start.y1 <= end.y0,
                "overlapping horizontal handles still have separate vertical hit areas"
            );
            let pos = center(start);
            let before = selection;
            frame(
                &ctx,
                &mut selection,
                vec![egui::Event::PointerMoved(pos), pointer(pos, true)],
            );
            let moved = pos - egui::vec2(10.0, 0.0);
            frame(&ctx, &mut selection, vec![egui::Event::PointerMoved(moved)]);
            frame(&ctx, &mut selection, vec![pointer(moved, false)]);
            assert!(selection.start_ms() < before.start_ms());
            assert!(before.start_ms() - selection.start_ms() < 1000);
            assert_eq!(selection.end_ms(), before.end_ms());
            assert_eq!(selection.playhead_ms(), before.playhead_ms());
            let start = selection.start_ms();
            frame(&ctx, &mut selection, vec![key(Key::ArrowLeft)]);
            assert_eq!(selection.start_ms(), start - 100);
            frame(&ctx, &mut selection, vec![key(Key::Home)]);
            assert_eq!(selection.start_ms(), 0);
            let (id, clipped) = node(frame(&ctx, &mut selection, vec![]), c::RANGE_START);
            let clipped = center(clipped);
            frame(
                &ctx,
                &mut selection,
                vec![egui::Event::PointerMoved(clipped), pointer(clipped, true)],
            );
            frame(
                &ctx,
                &mut selection,
                vec![egui::Event::PointerMoved(clipped + egui::vec2(50.0, 0.0))],
            );
            frame(&ctx, &mut selection, vec![pointer(clipped, false)]);
            assert_eq!(
                selection.start_ms(),
                0,
                "offscreen marker cannot jump into viewport"
            );
            frame(
                &ctx,
                &mut selection,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::SetValue,
                        target: id,
                        data: Some(egui::accesskit::ActionData::NumericValue(
                            (duration / 2 - 1000) as f64,
                        )),
                    },
                )],
            );
            assert_eq!(selection.start_ms(), duration / 2 - 1000);
            assert_eq!(selection.playhead_ms(), 1234);
            frame(&ctx, &mut selection, vec![key(Key::End)]);
            assert_eq!(selection.start_ms(), selection.end_ms() - 1);
        }
    }
}
