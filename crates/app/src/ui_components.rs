//! Product controls: hover, press, selection and focus remain independent.
use eframe::egui::{self, Color32, Response, Stroke};

#[derive(Clone, Copy)]
enum CommandKind {
    Primary,
    Secondary,
    Ghost,
    Help,
    Danger,
}

fn rgb(v: [u8; 3]) -> Color32 {
    Color32::from_rgb(v[0], v[1], v[2])
}

fn finish_control(ui: &egui::Ui, response: Response) -> Response {
    if response.gained_focus() && !ui.clip_rect().contains_rect(response.rect) {
        response.scroll_to_me(None);
    }
    if response.enabled() {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

// Native Button supplies input and accessibility; the role palette supplies its visuals.
// Pointer-down, keyboard focus and persistent selection never share the same fill state.
fn command(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    enabled: bool,
    kind: CommandKind,
    minimum: egui::Vec2,
) -> Response {
    use crate::ui_tokens as t;
    let background = ui.painter().add(egui::Shape::Noop);
    let outline = ui.painter().add(egui::Shape::Noop);
    let strong = if ui.visuals().dark_mode {
        Color32::from_white_alpha(46)
    } else {
        Color32::from_black_alpha(46)
    };
    let neutral = if ui.visuals().dark_mode {
        rgb(t::DARK_SECONDARY)
    } else {
        rgb(t::LIGHT_SECONDARY)
    };
    let active = enabled && ui.is_enabled();
    let foreground = if active && matches!(kind, CommandKind::Primary | CommandKind::Danger) {
        Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(label.into().color(foreground))
            .frame(true)
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(
                minimum.x.max(if matches!(kind, CommandKind::Help) {
                    t::CONTROL_HEIGHT
                } else {
                    t::CONTROL_MIN_WIDTH
                }),
                minimum.y.max(t::CONTROL_HEIGHT),
            )),
    );
    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let (fill, border) = if !response.enabled() {
        (neutral, Stroke::new(1.0, strong))
    } else {
        match kind {
            CommandKind::Primary => {
                let fill = if pressed {
                    crate::ui_theme::ACCENT_PRESSED
                } else if hovered {
                    crate::ui_theme::ACCENT_HOVER
                } else {
                    crate::ui_theme::ACCENT_SOLID
                };
                (fill, Stroke::new(1.0, fill))
            }
            CommandKind::Danger => {
                let fill = if pressed {
                    Color32::from_rgb(136, 27, 32)
                } else if hovered {
                    Color32::from_rgb(156, 33, 38)
                } else {
                    rgb(t::DANGER)
                };
                (fill, Stroke::new(1.0, fill))
            }
            CommandKind::Secondary | CommandKind::Ghost | CommandKind::Help => {
                let fill = if pressed {
                    Color32::from_gray(if ui.visuals().dark_mode { 72 } else { 218 })
                } else if hovered {
                    Color32::from_gray(if ui.visuals().dark_mode { 60 } else { 232 })
                } else if matches!(kind, CommandKind::Ghost | CommandKind::Help) {
                    Color32::TRANSPARENT
                } else {
                    neutral
                };
                let border = if matches!(kind, CommandKind::Ghost | CommandKind::Help) {
                    Stroke::NONE
                } else {
                    Stroke::new(1.0, strong)
                };
                (fill, border)
            }
        }
    };
    ui.painter().set(
        background,
        egui::Shape::Rect(egui::epaint::RectShape::new(
            response.rect,
            t::RADIUS,
            fill,
            border,
            egui::StrokeKind::Inside,
        )),
    );
    if response.has_focus() && response.enabled() {
        ui.painter().set(
            outline,
            egui::Shape::rect_stroke(
                response.rect.shrink(1.0),
                t::RADIUS,
                Stroke::new(
                    1.5,
                    if matches!(kind, CommandKind::Primary | CommandKind::Danger) {
                        Color32::WHITE
                    } else if ui.visuals().dark_mode {
                        crate::ui_theme::ACCENT
                    } else {
                        crate::ui_theme::ACCENT_SOLID
                    },
                ),
                egui::StrokeKind::Inside,
            ),
        );
    }
    finish_control(ui, response)
}

/// Retain the ComboBox's provider, label, ID and value while sharing its focus/cursor rules.
pub(crate) fn combo_response(ui: &egui::Ui, response: Response) -> Response {
    finish_control(ui, response)
}

pub(crate) fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    command(ui, label, enabled, CommandKind::Primary, egui::Vec2::ZERO)
}

pub(crate) fn secondary(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    enabled: bool,
) -> Response {
    command(ui, label, enabled, CommandKind::Secondary, egui::Vec2::ZERO)
}

pub(crate) fn secondary_sized(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    enabled: bool,
    minimum: egui::Vec2,
) -> Response {
    command(ui, label, enabled, CommandKind::Secondary, minimum)
}

pub(crate) fn ghost(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    enabled: bool,
) -> Response {
    command(ui, label, enabled, CommandKind::Ghost, egui::Vec2::ZERO)
}

pub(crate) fn danger(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    enabled: bool,
) -> Response {
    command(ui, label, enabled, CommandKind::Danger, egui::Vec2::ZERO)
}

pub(crate) fn choice(ui: &mut egui::Ui, selected: bool, label: &str) -> Response {
    choice_with_padding(
        ui,
        selected,
        label,
        crate::ui_tokens::CONTROL_MIN_WIDTH,
        crate::ui_tokens::CONTROL_PADDING_X,
    )
}

pub(crate) fn navigation_choice(ui: &mut egui::Ui, selected: bool, label: &str) -> Response {
    // Compact horizontal tabs retain body13/height32 and the existing provider/ID scope.
    choice_with_padding(
        ui,
        selected,
        label,
        crate::ui_tokens::COMPACT_MIN_WIDTH,
        crate::ui_tokens::COMPACT_PADDING_X,
    )
}

fn choice_with_padding(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    minimum_width: f32,
    padding_x: f32,
) -> Response {
    // Background slots precede the standard button text, retaining its native input/AccessKit.
    let background = ui.painter().add(egui::Shape::Noop);
    let outline = ui.painter().add(egui::Shape::Noop);
    let color = if !ui.is_enabled() {
        // The disabled Ui already fades its painter; avoid fading muted text twice.
        ui.visuals().text_color()
    } else if selected {
        if ui.visuals().dark_mode {
            Color32::from_rgb(139, 205, 255)
        } else {
            crate::ui_theme::ACCENT_PRESSED
        }
    } else {
        ui.visuals().text_color()
    };
    let response = ui
        .scope(|ui| {
            let text = egui::RichText::new(label).color(color);
            ui.spacing_mut().button_padding.x = padding_x;
            ui.add(
                egui::Button::new(text)
                    .frame(true)
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .min_size(egui::vec2(minimum_width, crate::ui_tokens::CONTROL_HEIGHT)),
            )
        })
        .inner;
    let neutral = if ui.visuals().dark_mode {
        Color32::from_gray(60)
    } else {
        Color32::from_gray(232)
    };
    let fill = if !response.enabled() {
        // Selection still answers "where am I?" while a worker temporarily locks controls.
        if selected {
            if ui.visuals().dark_mode {
                Color32::from_rgb(27, 53, 72)
            } else {
                Color32::from_rgb(224, 240, 252)
            }
        } else {
            Color32::TRANSPARENT
        }
    } else if response.is_pointer_button_down_on() {
        if selected {
            if ui.visuals().dark_mode {
                Color32::from_rgb(19, 43, 61)
            } else {
                Color32::from_rgb(181, 219, 247)
            }
        } else if ui.visuals().dark_mode {
            Color32::from_gray(80)
        } else {
            Color32::from_gray(212)
        }
    } else if selected && response.hovered() {
        if ui.visuals().dark_mode {
            Color32::from_rgb(28, 70, 100)
        } else {
            Color32::from_rgb(202, 230, 250)
        }
    } else if selected {
        if ui.visuals().dark_mode {
            Color32::from_rgb(27, 53, 72)
        } else {
            Color32::from_rgb(224, 240, 252)
        }
    } else if response.hovered() {
        neutral
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .set(background, egui::Shape::rect_filled(response.rect, 6, fill));
    if response.has_focus() && response.enabled() {
        ui.painter().set(
            outline,
            egui::Shape::rect_stroke(
                response.rect.shrink(1.0),
                6,
                Stroke::new(
                    1.5,
                    if ui.visuals().dark_mode {
                        crate::ui_theme::ACCENT
                    } else {
                        crate::ui_theme::ACCENT_SOLID
                    },
                ),
                egui::StrokeKind::Inside,
            ),
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    finish_control(ui, response)
}

pub(crate) fn select<T: PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    option: T,
    label: &str,
) -> Response {
    let mut response = choice(ui, *value == option, label);
    if response.clicked() && *value != option {
        *value = option;
        response.mark_changed();
    }
    response
}

/// Inner text width plus this margin is the complete field width.
pub(crate) const TEXT_INPUT_HORIZONTAL_MARGIN: f32 = 24.0;

/// Keep the checkbox indicator square enough to distinguish it from a radio button.
/// The native Checkbox retains input, checked/disabled semantics and AccessKit.
pub(crate) fn checkbox(ui: &mut egui::Ui, checked: &mut bool, label: &str) -> Response {
    let response = ui
        .scope(|ui| {
            let widgets = &mut ui.visuals_mut().widgets;
            for widget in [
                &mut widgets.noninteractive,
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
                &mut widgets.open,
            ] {
                widget.corner_radius = egui::CornerRadius::same(3);
            }
            ui.checkbox(checked, label)
        })
        .inner;
    finish_control(ui, response)
}

#[cfg(test)]
fn text_input(ui: &mut egui::Ui, value: &mut String, limit: usize, width: f32) -> Response {
    reveal_input(text_input_output(ui, value, limit, width))
}

/// Shared geometry with stable logical identity for editable product fields.
pub(crate) fn text_input_with_id(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
    id: egui::Id,
) -> Response {
    reveal_input(text_input_output_identified(
        ui,
        value,
        limit,
        width,
        Some(id),
    ))
}

/// Product input geometry and identity, retaining a contextual placeholder.
pub(crate) fn text_input_with_id_and_hint(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
    id: egui::Id,
    hint: &str,
) -> Response {
    reveal_input(text_input_output_with_hint(
        ui,
        value,
        limit,
        width,
        Some(id),
        hint,
    ))
}

fn reveal_input(output: egui::text_edit::TextEditOutput) -> Response {
    let response = output.response;
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    response
}

#[cfg(test)]
fn text_input_output(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
) -> egui::text_edit::TextEditOutput {
    text_input_output_identified(ui, value, limit, width, None)
}

fn text_input_output_identified(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
    id: Option<egui::Id>,
) -> egui::text_edit::TextEditOutput {
    text_input_output_with_hint(ui, value, limit, width, id, "")
}

fn text_input_output_with_hint(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
    id: Option<egui::Id>,
    hint: &str,
) -> egui::text_edit::TextEditOutput {
    ui.scope(|ui| {
        let border = if ui.visuals().dark_mode {
            Color32::from_gray(128)
        } else {
            Color32::from_gray(148)
        };
        let hover = ui.visuals().weak_text_color();
        let accent = if ui.visuals().dark_mode {
            crate::ui_theme::ACCENT
        } else {
            crate::ui_theme::ACCENT_SOLID
        };
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, hover);
        // TextEdit paints keyboard focus using selection.stroke, not widgets.active.
        ui.visuals_mut().selection.stroke = Stroke::new(1.5, accent);
        let input = egui::TextEdit::singleline(value)
            .hint_text(hint)
            .char_limit(limit)
            .margin(egui::Margin::symmetric(12, 2))
            .vertical_align(egui::Align::Center)
            .min_size(egui::vec2(0.0, 32.0))
            .desired_width(width);
        let input = if let Some(id) = id {
            input.id(id)
        } else {
            input
        };
        input.show(ui)
    })
    .inner
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use egui::{
        Event, Key, Modifiers,
        text::{CCursor, CCursorRange},
    };
    #[test]
    fn input_focus_preserves_visible_scroll_and_reveals_clipped_field() {
        fn frame(
            ctx: &egui::Context,
            value: &mut String,
            gap: f32,
            time: f64,
            focus: Option<egui::Id>,
        ) -> (egui::Id, f32, egui::Rect, egui::Rect) {
            let mut observed = None;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let scroll = egui::ScrollArea::vertical()
                            .id_salt("actual-input-focus")
                            .max_height(128.0)
                            .show(ui, |ui| {
                                ui.add_space(gap);
                                if let Some(id) = focus {
                                    ctx.memory_mut(|m| m.request_focus(id));
                                }
                                let response = text_input(ui, value, 8192, 260.0);
                                let result = (response.id, response.rect, ui.clip_rect());
                                ui.add_space(512.0);
                                result
                            });
                        observed = Some((
                            scroll.inner.0,
                            scroll.state.offset.y,
                            scroll.inner.1,
                            scroll.inner.2,
                        ));
                    });
                },
            );
            observed.unwrap()
        }
        for gap in [32.0, 256.0] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            let mut value = "정밀 시간".to_owned();
            let first = frame(&ctx, &mut value, gap, 0.0, None);
            assert_eq!(first.1, 0.0);
            let _ = frame(&ctx, &mut value, gap, 1.0, Some(first.0));
            // ScrollArea updates the animated offset after constructing this frame's child UI.
            // Inspect geometry on the following settled frame, not the first target update.
            let _ = frame(&ctx, &mut value, gap, 2.0, None);
            let after = frame(&ctx, &mut value, gap, 3.0, None);
            if gap < 128.0 {
                assert_eq!(after.1, 0.0, "visible field focus must not jump");
            } else {
                assert!(after.1 > 0.0);
                assert!(
                    after.2.top() >= after.3.top() && after.2.bottom() <= after.3.bottom(),
                    "offset={} rect={:?} clip={:?}",
                    after.1,
                    after.2,
                    after.3
                );
            }
            assert_eq!(value, "정밀 시간");
        }
    }
    #[test]
    fn input_identity_survives_inline_validation_banner_while_typing() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        let mut value = String::new();
        let mut time = 0.0;
        let mut render = |value: &mut String, focus: Option<egui::Id>, events: Vec<egui::Event>| {
            time += 1.0;
            let mut id = None;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if !value.is_empty() {
                            ui.weak(crate::catalog::MEDIA_RANGE_INVALID);
                        }
                        ui.columns(2, |columns| {
                            let ui = &mut columns[0];
                            let label = ui.label(crate::catalog::RANGE_START);
                            if let Some(focus) = focus {
                                ctx.memory_mut(|m| m.request_focus(focus));
                            }
                            id = Some(
                                text_input_with_id(
                                    ui,
                                    value,
                                    32,
                                    260.0,
                                    egui::Id::new((
                                        "download-option-input",
                                        crate::catalog::RANGE_START,
                                    )),
                                )
                                .labelled_by(label.id)
                                .id,
                            );
                        });
                    });
                },
            );
            id.unwrap()
        };
        let initial = render(&mut value, None, vec![]);
        let _ = render(&mut value, Some(initial), vec![]);
        let mut expected = String::new();
        for character in "00:00:01.000".chars() {
            expected.push(character);
            let current = render(
                &mut value,
                None,
                vec![egui::Event::Text(character.to_string())],
            );
            assert_eq!(
                current, initial,
                "validation feedback must not replace the editor"
            );
            assert_eq!(value, expected);
            assert!(ctx.memory(|m| m.has_focus(initial)));
        }
    }
    struct Editor {
        ctx: egui::Context,
        value: String,
        other: String,
        width: f32,
        enabled: bool,
        time: f64,
    }
    impl Editor {
        fn new(value: &str, width: f32, scale: f32, dark: bool) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            // Reconfigure after the explicit theme so focus/caret tokens remain the product's.
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_pixels_per_point(scale);
            Self {
                ctx,
                value: value.into(),
                other: String::new(),
                width,
                enabled: true,
                time: 0.0,
            }
        }
        fn frame(&mut self, events: Vec<Event>) -> (egui::text_edit::TextEditOutput, egui::Id) {
            self.time += 1.0;
            let mut output = None;
            let mut other = None;
            let _ = self.ctx.run(
                egui::RawInput {
                    time: Some(self.time),
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        output = Some(
                            ui.add_enabled_ui(self.enabled, |ui| {
                                text_input_output(ui, &mut self.value, 8192, self.width)
                            })
                            .inner,
                        );
                        other = Some(text_input(ui, &mut self.other, 8192, self.width).id);
                    });
                },
            );
            (output.unwrap(), other.unwrap())
        }
        fn focus(&mut self) -> egui::text_edit::TextEditOutput {
            let o = self.frame(vec![]).0;
            self.ctx.memory_mut(|m| m.request_focus(o.response.id));
            self.frame(vec![]).0
        }
        fn click(&mut self, point: egui::Pos2) -> egui::text_edit::TextEditOutput {
            self.frame(vec![Event::PointerMoved(point)]);
            self.frame(vec![Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }]);
            self.frame(vec![Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }])
            .0
        }
    }
    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }
    fn caret(output: &egui::text_edit::TextEditOutput) -> egui::Rect {
        output
            .galley
            .pos_from_cursor(output.cursor_range.unwrap().primary)
            .translate(output.galley_pos.to_vec2())
    }
    #[test]
    fn product_input_centers_line_and_caret_with_padding_in_both_themes_and_scales() {
        for dark in [false, true] {
            for scale in [1.0, 1.5, 2.0] {
                for text in ["", "URL", "A한글Z", "C:\\fixture\\video.mkv"] {
                    let mut e = Editor::new(text, 240.0, scale, dark);
                    let o = e.focus();
                    assert!((o.response.rect.height() - 32.0).abs() < 0.5);
                    assert!((o.text_clip_rect.left() - o.response.rect.left() - 12.0).abs() < 0.5);
                    assert!(
                        (o.response.rect.right() - o.text_clip_rect.right() - 12.0).abs() < 0.5
                    );
                    assert!(
                        (o.galley_pos.y + o.galley.size().y * 0.5 - o.response.rect.center().y)
                            .abs()
                            < 0.5
                    );
                    assert!((caret(&o).center().y - o.response.rect.center().y).abs() < 1.0);
                }
            }
        }
    }
    #[test]
    fn product_input_pointer_unicode_insertion_and_tab_target_match_actual_glyphs() {
        let mut e = Editor::new("A한글Z", 240.0, 1.0, false);
        let o = e.frame(vec![]).0;
        let pos = o
            .galley
            .pos_from_cursor(CCursor::new(2))
            .translate(o.galley_pos.to_vec2())
            .center();
        let o = e.click(pos);
        assert_eq!(o.cursor_range.unwrap().primary.index, 2);
        e.frame(vec![Event::Text("X".into())]);
        assert_eq!(e.value, "A한X글Z");
        let (_, other) = e.frame(vec![key(Key::Tab, Modifiers::NONE)]);
        e.frame(vec![]);
        assert!(e.ctx.memory(|m| m.has_focus(other)));
        e.frame(vec![Event::Text("next".into())]);
        assert_eq!(e.value, "A한X글Z");
        assert_eq!(e.other, "next");
    }
    #[test]
    fn product_input_home_end_selection_and_long_scroll_keep_caret_inside_clip() {
        for width in [124.0, 480.0] {
            let original = format!("C:\\{}\\한글.mkv", "long-directory".repeat(36));
            let mut e = Editor::new(&original, width, 1.0, true);
            e.focus();
            let end = e.frame(vec![key(Key::End, Modifiers::NONE)]).0;
            assert_eq!(
                end.cursor_range.unwrap().primary.index,
                original.chars().count()
            );
            assert!(end.galley_pos.x < end.text_clip_rect.left());
            assert!((caret(&end).left() - end.text_clip_rect.right()).abs() < 1.0);
            let selected = e
                .frame(vec![key(Key::Home, Modifiers::SHIFT)])
                .0
                .cursor_range
                .unwrap();
            assert_eq!(selected.as_sorted_char_range(), 0..original.chars().count());
            assert_eq!(e.value, original);
            let home = e.frame(vec![key(Key::Home, Modifiers::NONE)]).0;
            assert_eq!(home.cursor_range.unwrap().primary.index, 0);
            assert!((caret(&home).left() - home.text_clip_rect.left()).abs() < 1.0);
        }
    }
    #[test]
    fn product_input_disabled_click_and_typing_preserve_text_without_focus() {
        let mut e = Editor::new("A한글Z", 240.0, 1.0, false);
        e.enabled = false;
        let o = e.frame(vec![]).0;
        let o = e.click(o.response.rect.center());
        e.frame(vec![
            key(Key::End, Modifiers::NONE),
            Event::Text("unexpected".into()),
        ]);
        assert_eq!(e.value, "A한글Z");
        assert!(!e.ctx.memory(|m| m.has_focus(o.response.id)));
    }
    #[test]
    fn product_input_ime_cancel_restores_directed_selection_and_caret_geometry() {
        let mut e = Editor::new("A한글Z", 240.0, 1.0, false);
        let o = e.focus();
        let mut state = egui::TextEdit::load_state(&e.ctx, o.response.id).unwrap();
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(3), CCursor::new(1))));
        state.store(&e.ctx, o.response.id);
        let before = e.frame(vec![]).0;
        e.frame(vec![
            Event::Ime(egui::ImeEvent::Enabled),
            Event::Ime(egui::ImeEvent::Preedit("조합".into())),
        ]);
        let after = e
            .frame(vec![
                Event::Ime(egui::ImeEvent::Disabled),
                Event::Ime(egui::ImeEvent::Commit(String::new())),
            ])
            .0;
        assert_eq!(e.value, "A한글Z");
        assert_eq!(after.cursor_range, before.cursor_range);
        assert_eq!(caret(&after), caret(&before));
    }
}

#[cfg(test)]
mod choice_pointer_tests {
    use super::*;

    struct Fixture {
        ctx: egui::Context,
        selected: bool,
        enabled: bool,
        time: f64,
    }
    impl Fixture {
        fn new(dark: bool, selected: bool, enabled: bool) -> Self {
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            Self {
                ctx,
                selected,
                enabled,
                time: 0.0,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (Response, egui::FullOutput) {
            self.time += 0.05;
            let mut response = None;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 160.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        response = Some(
                            ui.add_enabled_ui(self.enabled, |ui| {
                                choice(ui, self.selected, "Settings")
                            })
                            .inner,
                        );
                    });
                },
            );
            (response.unwrap(), output)
        }
    }
    fn fill(output: &egui::FullOutput, response: &Response) -> Color32 {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Rect(rect) if rect.rect == response.rect => Some(rect.fill),
                _ => None,
            })
            .expect("actual choice background painter shape")
    }
    fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn selected_label_paint_contrast(
        f: &Fixture,
        response: &Response,
        output: &egui::FullOutput,
        state: &str,
    ) {
        let selected_node = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("actual selected label AccessKit update")
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label() == Some("Settings") && node.role() == egui::accesskit::Role::Button
            })
            .expect("actual selected choice Button");
        assert_eq!(
            selected_node.1.toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        assert!(!selected_node.1.is_disabled());
        let background = fill(output, response);
        assert_eq!(background.a(), 255, "selected fill is opaque: {state}");
        let text_shapes: Vec<_> = output
            .shapes
            .iter()
            .filter(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == "Settings")
            })
            .cloned()
            .collect();
        assert_eq!(text_shapes.len(), 1, "one actual choice label: {state}");
        if let egui::Shape::Text(text) = &text_shapes[0].shape {
            assert!(response.rect.contains_rect(text.visual_bounding_rect()));
        }
        // Tessellate the actual label shapes: this resolves galley explicit/fallback/
        // override colors and opacity through epaint rather than guessing a token.
        let primitives = f.ctx.tessellate(text_shapes, output.pixels_per_point);
        let luminance = |color: Color32| {
            let channel = |v: u8| {
                let s = f64::from(v) / 255.0;
                if s <= 0.04045 {
                    s / 12.92
                } else {
                    ((s + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
        };
        let mut glyph_vertices = 0;
        for primitive in primitives {
            let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
                panic!("unexpected label callback: {state}");
            };
            for vertex in mesh.vertices {
                glyph_vertices += 1;
                let foreground = vertex.color;
                assert_eq!(foreground.a(), 255, "enabled label opacity: {state}");
                let text = luminance(foreground);
                let surface = luminance(background);
                let ratio = (text.max(surface) + 0.05) / (text.min(surface) + 0.05);
                assert!(
                    ratio >= 4.5,
                    "actual selected {state} label/fill contrast {ratio}: {foreground:?}/{background:?}"
                );
            }
        }
        assert!(
            glyph_vertices > 0,
            "actual tessellated label glyphs: {state}"
        );
    }
    #[test]
    fn actual_selected_choice_label_contrast_survives_hover_and_press_in_both_themes() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark, true, true);
            f.ctx.enable_accesskit();
            let (idle, output) = f.frame(vec![]);
            assert!(idle.enabled() && !idle.hovered() && !idle.is_pointer_button_down_on());
            selected_label_paint_contrast(&f, &idle, &output, "idle");
            let idle_fill = fill(&output, &idle);
            let pos = idle.rect.center();
            let (hover, output) = f.frame(vec![egui::Event::PointerMoved(pos)]);
            assert!(hover.hovered() && !hover.is_pointer_button_down_on());
            assert_eq!(hover.id, idle.id);
            assert_eq!(hover.rect, idle.rect);
            selected_label_paint_contrast(&f, &hover, &output, "hover");
            let hover_fill = fill(&output, &hover);
            assert_ne!(idle_fill, hover_fill);
            let (press, output) = f.frame(vec![button(pos, true)]);
            assert!(press.is_pointer_button_down_on() && !press.clicked());
            assert_eq!(press.id, idle.id);
            assert_eq!(press.rect, idle.rect);
            assert_ne!(fill(&output, &press), hover_fill);
            selected_label_paint_contrast(&f, &press, &output, "pressed");
            let (release, output) = f.frame(vec![button(pos, false)]);
            assert!(release.clicked() && !release.is_pointer_button_down_on());
            assert_eq!(release.id, idle.id);
            assert_eq!(fill(&output, &release), hover_fill);
            selected_label_paint_contrast(&f, &release, &output, "release");
        }
    }
    #[test]
    fn actual_choice_hover_press_release_are_distinct_in_both_themes() {
        for dark in [false, true] {
            let mut idle_fills = Vec::new();
            for selected in [false, true] {
                let mut f = Fixture::new(dark, selected, true);
                let (idle, idle_output) = f.frame(vec![]);
                let pos = idle.rect.center();
                idle_fills.push(fill(&idle_output, &idle));
                let (hover, hover_output) = f.frame(vec![egui::Event::PointerMoved(pos)]);
                assert!(hover.hovered());
                assert_eq!(
                    hover_output.platform_output.cursor_icon,
                    egui::CursorIcon::PointingHand
                );
                let hover_fill = fill(&hover_output, &hover);
                let (press, press_output) = f.frame(vec![button(pos, true)]);
                assert!(press.is_pointer_button_down_on());
                assert!(!press.clicked());
                assert_ne!(
                    fill(&press_output, &press),
                    hover_fill,
                    "actual painted pointer-down differs from hover: dark={dark} selected={selected}"
                );
                let (release, release_output) = f.frame(vec![button(pos, false)]);
                assert!(
                    release.clicked(),
                    "actual pointer release still activates choice"
                );
                assert_eq!(release.id, idle.id);
                assert!(!release.is_pointer_button_down_on());
                assert_eq!(
                    fill(&release_output, &release),
                    hover_fill,
                    "release returns to hover fill without losing selected policy"
                );
                // Pointer activation and keyboard focus are independent.
                // Exercise the actual keyboard route instead of assuming that
                // an egui button takes focus after a mouse release.
                let (focused, focus_output) = f.frame(vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                assert!(focused.has_focus(), "actual Tab focuses the choice");
                assert!(focus_output.shapes.iter().any(|shape| matches!(&shape.shape,egui::epaint::Shape::Rect(rect) if rect.rect==focused.rect.shrink(1.0) &&rect.stroke.width>0.0)),"actual keyboard focus outline retains separate painter stroke");
            }
            assert_ne!(
                idle_fills[0], idle_fills[1],
                "selected idle remains distinct from neutral idle"
            );
        }
    }
    #[test]
    fn actual_disabled_choice_retains_selection_without_hover_or_activation() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark, true, false);
            let (idle, idle_output) = f.frame(vec![]);
            let selected_fill = fill(&idle_output, &idle);
            assert_ne!(selected_fill, Color32::TRANSPARENT);
            let pos = idle.rect.center();
            f.frame(vec![egui::Event::PointerMoved(pos)]);
            let (press, output) = f.frame(vec![button(pos, true)]);
            assert!(!press.enabled());
            assert_eq!(
                fill(&output, &press),
                selected_fill,
                "disabled selected surface stays visible and does not react to hover/down"
            );
            let (release, _) = f.frame(vec![button(pos, false)]);
            assert!(!release.clicked() && !release.has_focus());
        }
    }
}

#[cfg(test)]
mod checkbox_shape_tests {
    use super::*;

    struct Fixture {
        ctx: egui::Context,
        checked: bool,
        enabled: bool,
        time: f64,
    }
    impl Fixture {
        fn new(dark: bool) -> Self {
            let ctx = egui::Context::default();
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            Self {
                ctx,
                checked: false,
                enabled: true,
                time: 0.0,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (Response, Response, egui::FullOutput) {
            self.time += 0.05;
            let mut responses = None;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 200.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let check = ui
                            .push_id("media-preview-bypass", |ui| {
                                ui.add_enabled_ui(self.enabled, |ui| {
                                    checkbox(ui, &mut self.checked, "Preview bypass")
                                })
                                .inner
                            })
                            .inner;
                        let sibling = ui.button("Sibling");
                        responses = Some((check, sibling));
                    });
                },
            );
            let (check, sibling) = responses.unwrap();
            (check, sibling, output)
        }
    }
    fn paint(output: &egui::FullOutput, check: &Response, sibling: &Response) {
        let icon = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Rect(r)
                    if check.rect.contains_rect(r.rect)
                        && (r.rect.width() - 14.0).abs() < 0.01
                        && (r.rect.height() - 14.0).abs() < 0.01 =>
                {
                    Some(r)
                }
                _ => None,
            })
            .expect("actual 14px checkbox indicator painter");
        assert_eq!(
            icon.corner_radius,
            egui::CornerRadius::same(3),
            "square-checkbox affordance; original common radius6 must fail"
        );
        let button = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Rect(r) if r.rect == sibling.rect => Some(r),
                _ => None,
            })
            .expect("actual sibling button painter");
        assert_eq!(
            button.corner_radius,
            egui::CornerRadius::same(6),
            "scoped checkbox styling cannot leak"
        );
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn semantics(output: &egui::FullOutput, checked: bool) -> egui::accesskit::NodeId {
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("actual AccessKit update");
        let (id, node) = tree
            .nodes
            .iter()
            .find(|(_, n)| n.role() == egui::accesskit::Role::CheckBox)
            .expect("native Checkbox role");
        assert_eq!(
            node.toggled(),
            Some(if checked {
                egui::accesskit::Toggled::True
            } else {
                egui::accesskit::Toggled::False
            })
        );
        *id
    }
    #[test]
    fn actual_checkbox_square_paint_input_focus_semantics_and_sibling_in_both_themes() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark);
            let (idle, sibling, output) = f.frame(vec![]);
            paint(&output, &idle, &sibling);
            let node_id = semantics(&output, false);
            let (settled, _, output) = f.frame(vec![]);
            assert_eq!(idle.id, settled.id);
            assert_eq!(node_id, semantics(&output, false));
            let pos = idle.rect.center();
            f.frame(vec![egui::Event::PointerMoved(pos)]);
            f.frame(vec![pointer(pos, true)]);
            let (release, sibling, output) = f.frame(vec![pointer(pos, false)]);
            assert!(release.clicked());
            assert!(f.checked);
            assert_eq!(release.id, idle.id);
            semantics(&output, true);
            paint(&output, &release, &sibling);
            let mut keyboard = Fixture::new(dark);
            let (keyboard_idle, _, _) = keyboard.frame(vec![]);
            let (focused, _, output) = keyboard.frame(vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]);
            assert!(
                focused.has_focus(),
                "actual Tab focuses first native checkbox"
            );
            assert_eq!(focused.id, keyboard_idle.id);
            let keyboard_node = semantics(&output, false);
            assert_eq!(
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .focus,
                keyboard_node
            );
            f.enabled = false;
            f.frame(vec![egui::Event::PointerMoved(pos)]);
            f.frame(vec![pointer(pos, true)]);
            let (disabled, sibling, output) = f.frame(vec![pointer(pos, false)]);
            assert!(!disabled.enabled());
            assert!(!disabled.clicked());
            assert!(f.checked);
            paint(&output, &disabled, &sibling);
            semantics(&output, true);
        }
    }
}

#[cfg(test)]
mod command_state_tests {
    use super::*;
    struct Fixture {
        ctx: egui::Context,
        time: f64,
        enabled: bool,
        label: &'static str,
        kind: CommandKind,
    }
    impl Fixture {
        fn new(dark: bool) -> Self {
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            Self {
                ctx,
                time: 0.0,
                enabled: true,
                label: "Action",
                kind: CommandKind::Secondary,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (Response, egui::FullOutput) {
            self.time += 0.05;
            let mut response = None;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 200.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        response = Some(match self.kind {
                            CommandKind::Primary => primary(ui, self.label, self.enabled),
                            CommandKind::Secondary => secondary(ui, self.label, self.enabled),
                            CommandKind::Ghost => ghost(ui, self.label, self.enabled),
                            CommandKind::Help => command(
                                ui,
                                self.label,
                                self.enabled,
                                CommandKind::Help,
                                egui::Vec2::ZERO,
                            ),
                            CommandKind::Danger => danger(ui, self.label, self.enabled),
                        });
                    });
                },
            );
            (response.unwrap(), output)
        }
    }
    fn fill(out: &egui::FullOutput, response: &Response) -> Color32 {
        out.shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.rect == response.rect => Some(rect.fill),
                _ => None,
            })
            .expect("actual command background")
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn actual_long_command_label_has_padding_and_unclipped_glyphs() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark);
            f.label = "Installation status";
            let (response, output) = f.frame(vec![]);
            let glyph = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.text() == f.label => {
                        Some(t.visual_bounding_rect())
                    }
                    _ => None,
                })
                .unwrap();
            assert!(response.rect.contains_rect(glyph));
            assert!(
                glyph.left() >= response.rect.left() + crate::ui_tokens::CONTROL_PADDING_X - 0.5
            );
            assert!(
                glyph.right() <= response.rect.right() - crate::ui_tokens::CONTROL_PADDING_X + 0.5
            );
        }
    }
    #[test]
    fn actual_command_hover_press_keyboard_focus_and_disabled_are_distinct() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark);
            let (idle, output) = f.frame(vec![]);
            let idle_fill = fill(&output, &idle);
            let (focused, output) = f.frame(vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]);
            assert!(focused.has_focus());
            assert_eq!(
                fill(&output, &focused),
                idle_fill,
                "keyboard focus must not masquerade as pointer-down"
            );
            assert_eq!(focused.rect, idle.rect);
            let pos = idle.rect.center();
            let (hover, output) = f.frame(vec![egui::Event::PointerMoved(pos)]);
            assert!(hover.hovered());
            let hover_fill = fill(&output, &hover);
            assert_ne!(hover_fill, idle_fill);
            let (pressed, output) = f.frame(vec![pointer(pos, true)]);
            assert!(pressed.is_pointer_button_down_on());
            assert_ne!(fill(&output, &pressed), hover_fill);
            let (released, _) = f.frame(vec![pointer(pos, false)]);
            assert!(released.clicked());
            f.enabled = false;
            let (disabled, output) = f.frame(vec![egui::Event::PointerMoved(pos)]);
            assert!(!disabled.enabled());
            assert!(!disabled.hovered());
            assert_eq!(disabled.id, idle.id);
            let node = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button && n.label() == Some("Action")
                })
                .unwrap();
            assert!(node.1.is_disabled());
            f.frame(vec![pointer(pos, true)]);
            let (release, _) = f.frame(vec![pointer(pos, false)]);
            assert!(!release.clicked());
        }
    }
    #[test]
    fn actual_filled_command_focus_stroke_contrasts_with_painted_fill_in_both_themes() {
        fn outline<'a>(
            out: &'a egui::FullOutput,
            response: &Response,
        ) -> Option<&'a egui::epaint::RectShape> {
            out.shapes.iter().find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect)
                    if rect.rect == response.rect.shrink(1.0) && rect.stroke.width > 0.0 =>
                {
                    Some(rect)
                }
                _ => None,
            })
        }
        fn contrast(a: Color32, b: Color32) -> f64 {
            let luminance = |c: Color32| {
                let channel = |v: u8| {
                    let s = f64::from(v) / 255.0;
                    if s <= 0.04045 {
                        s / 12.92
                    } else {
                        ((s + 0.055) / 1.055).powf(2.4)
                    }
                };
                0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
            };
            let (x, y) = (luminance(a), luminance(b));
            (x.max(y) + 0.05) / (x.min(y) + 0.05)
        }
        fn semantics(
            out: &egui::FullOutput,
            response: &Response,
            disabled: bool,
        ) -> egui::accesskit::NodeId {
            let (id, node) = out
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button && n.label() == Some("Action")
                })
                .expect("actual original command Button provider");
            assert_eq!(node.is_disabled(), disabled);
            assert_eq!(response.enabled(), !disabled);
            *id
        }
        for dark in [false, true] {
            for (kind, name) in [
                (CommandKind::Primary, "primary"),
                (CommandKind::Danger, "danger"),
            ] {
                let mut f = Fixture::new(dark);
                f.kind = kind;
                let (idle, out) = f.frame(vec![]);
                let idle_fill = fill(&out, &idle);
                let node_id = semantics(&out, &idle, false);
                assert!(!idle.has_focus());
                assert!(
                    outline(&out, &idle).is_none(),
                    "unfocused {name} has no focus stroke"
                );
                let (focused, out) = f.frame(vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                assert!(
                    focused.has_focus(),
                    "actual Tab focus ACK: {name} dark={dark}"
                );
                assert_eq!(focused.id, idle.id);
                assert_eq!(
                    fill(&out, &focused),
                    idle_fill,
                    "focus does not change idle fill"
                );
                let pos = focused.rect.center();
                let states = [
                    ("idle", vec![]),
                    ("hover", vec![egui::Event::PointerMoved(pos)]),
                    ("pressed", vec![pointer(pos, true)]),
                ];
                let mut prior_fill = None;
                for (state, events) in states {
                    let (response, out) = f.frame(events);
                    assert_eq!(response.id, idle.id);
                    assert_eq!(response.rect, idle.rect);
                    assert_eq!(semantics(&out, &response, false), node_id);
                    assert!(
                        response.has_focus(),
                        "focus ACK: {name} {state} dark={dark}"
                    );
                    assert_eq!(
                        response.hovered(),
                        state != "idle",
                        "actual hover state ACK"
                    );
                    assert_eq!(
                        response.is_pointer_button_down_on(),
                        state == "pressed",
                        "actual pointer-down state ACK"
                    );
                    let background = fill(&out, &response);
                    assert_eq!(
                        background.a(),
                        255,
                        "actual filled command background opaque"
                    );
                    if let Some(previous) = prior_fill {
                        assert_ne!(background, previous, "actual state fill transitions");
                    }
                    prior_fill = Some(background);
                    let stroke =
                        outline(&out, &response).expect("actual focused command outline painted");
                    assert!(response.rect.contains_rect(stroke.rect));
                    assert_eq!(stroke.stroke_kind, egui::StrokeKind::Inside);
                    assert!(stroke.stroke.width >= 1.5);
                    assert_eq!(stroke.stroke.color.a(), 255, "actual focus stroke opaque");
                    let ratio = contrast(stroke.stroke.color, background);
                    assert!(
                        ratio >= 3.0,
                        "actual {name} {state} focus/fill contrast {ratio}: {:?}/{background:?}, dark={dark}",
                        stroke.stroke.color
                    );
                }
                let (released, _) = f.frame(vec![pointer(pos, false)]);
                assert!(
                    released.clicked(),
                    "actual enabled command activation preserved"
                );
                f.ctx.memory_mut(|memory| memory.surrender_focus(idle.id));
                let (unfocused, out) = f.frame(vec![egui::Event::PointerGone]);
                assert!(!unfocused.has_focus());
                assert_eq!(unfocused.id, idle.id);
                assert_eq!(semantics(&out, &unfocused, false), node_id);
                assert!(
                    outline(&out, &unfocused).is_none(),
                    "focus-off removes separate stroke"
                );
                f.enabled = false;
                f.ctx.memory_mut(|memory| memory.request_focus(idle.id));
                let (disabled, out) = f.frame(vec![egui::Event::PointerMoved(pos)]);
                assert_eq!(disabled.id, idle.id);
                assert_eq!(semantics(&out, &disabled, true), node_id);
                assert!(!disabled.hovered());
                assert!(
                    outline(&out, &disabled).is_none(),
                    "disabled gate suppresses focus stroke"
                );
                f.frame(vec![pointer(pos, true)]);
                let (disabled_release, out) = f.frame(vec![pointer(pos, false)]);
                assert!(!disabled_release.clicked());
                assert!(outline(&out, &disabled_release).is_none());
            }
        }
    }
}

#[cfg(test)]
mod locked_selection_tests {
    use super::*;
    #[test]
    fn actual_selected_choice_keeps_identity_geometry_and_toggle_while_locked() {
        for dark in [false, true] {
            let ctx = egui::Context::default();
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            let mut original = None;
            for (step, enabled) in [true, false, true].into_iter().enumerate() {
                let mut response = None;
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 200.0),
                        )),
                        focused: true,
                        time: Some(step as f64 * 0.05),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            response = Some(
                                ui.add_enabled_ui(enabled, |ui| choice(ui, true, "Selected"))
                                    .inner,
                            );
                        });
                    },
                );
                let response = response.unwrap();
                if let Some((id, rect)) = original {
                    assert_eq!(response.id, id);
                    assert_eq!(response.rect, rect);
                } else {
                    original = Some((response.id, response.rect));
                }
                assert_eq!(response.enabled(), enabled);
                let background = output
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::Shape::Rect(r) if r.rect == response.rect => Some(r.fill),
                        _ => None,
                    })
                    .unwrap();
                assert_ne!(
                    background,
                    Color32::TRANSPARENT,
                    "locked current selection must remain visible"
                );
                let node = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, n)| {
                        n.label() == Some("Selected") && n.role() == egui::accesskit::Role::Button
                    })
                    .unwrap();
                assert_eq!(node.1.toggled(), Some(egui::accesskit::Toggled::True));
                assert_eq!(node.1.is_disabled(), !enabled);
            }
        }
    }
}

/// Optional context only: retain native hover/disabled behavior and the original response.
/// Root may adopt with callers; never replace required inline errors with this helper.
/// Optional contextual explanation. Callers retain mandatory warnings and refusal reasons inline.
/// The explicit logical key keeps both the Button and its pinned popup stable across layout changes.
// Sixteen logical pixels of vector paint, independent of installed glyph fonts.
// Native Button retains the32px hit area, original ID/provider and all popup input.
fn paint_help_icon(ui: &egui::Ui, response: &Response, pinned: bool) {
    if !ui.is_rect_visible(response.rect) {
        return;
    }
    let muted = rgb(if ui.visuals().dark_mode {
        crate::ui_tokens::DARK_MUTED
    } else {
        crate::ui_tokens::LIGHT_MUTED
    });
    let accent = if ui.visuals().dark_mode {
        crate::ui_theme::ACCENT
    } else {
        crate::ui_theme::ACCENT_SOLID
    };
    let color = if !response.enabled() {
        muted.gamma_multiply(0.5)
    } else if response.is_pointer_button_down_on() {
        if ui.visuals().dark_mode {
            crate::ui_theme::ACCENT_HOVER
        } else {
            crate::ui_theme::ACCENT_PRESSED
        }
    } else if pinned {
        accent
    } else if response.hovered() {
        ui.visuals().text_color()
    } else {
        muted
    };
    let center = response.rect.center();
    let fill = if pinned && response.enabled() {
        accent.gamma_multiply(0.12)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .circle(center, 7.25, fill, Stroke::new(1.5, color));
    ui.painter()
        .circle_filled(center + egui::vec2(0.0, -3.3), 0.9, color);
    ui.painter().rect_filled(
        egui::Rect::from_center_size(center + egui::vec2(0.0, 1.6), egui::vec2(1.5, 4.5)),
        egui::CornerRadius::same(1),
        color,
    );
}

// One fixed-size temporary record per dismissed logical help owner, no event history.
// Existing callers use bounded stable keys (fixed fields/tool slots), not user path strings.
#[derive(Clone, Copy)]
struct HelpDismiss {
    focused: bool,
    pointer_inside: bool,
    pointer_reentry: bool,
    dismissed_frame: u64,
}

fn help_after_dismiss(ui: &egui::Ui, response: &Response, dismissed: bool) -> bool {
    let key = response.id.with("help-dismiss");
    let frame = ui.ctx().cumulative_frame_nr();
    // Window activation does not change which logical control owns keyboard focus.
    let focused = ui.ctx().memory(|memory| memory.has_focus(response.id));
    let hovered = response.hovered();
    let clicked = response.clicked();
    // Read Context outside the data write lock. A layer disappearing under a
    // stationary pointer is not a new hover entry. Remember genuine movement
    // until the actual hovered response catches up on a subsequent pass.
    let (pointer_inside, pointer_event) = ui.input(|input| {
        (
            input
                .pointer
                .hover_pos()
                .is_some_and(|position| response.rect.contains(position)),
            input.focused
                && input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::PointerMoved(_) | egui::Event::PointerGone
                    )
                }),
        )
    });
    ui.ctx().data_mut(|data| {
        if dismissed {
            data.insert_temp(
                key,
                HelpDismiss {
                    focused,
                    pointer_inside,
                    pointer_reentry: false,
                    dismissed_frame: frame,
                },
            );
            return false;
        }
        let Some(mut state) = data.get_temp::<HelpDismiss>(key) else {
            return true;
        };
        if pointer_event {
            state.pointer_reentry =
                pointer_inside && (state.pointer_reentry || !state.pointer_inside);
            state.pointer_inside = pointer_inside;
        }
        // Escape's own focus return and repeated sizing passes never re-arm it.
        if frame != state.dismissed_frame
            && (clicked || (!state.focused && focused) || (state.pointer_reentry && hovered))
        {
            data.remove::<HelpDismiss>(key);
            return true;
        }
        state.focused = focused;
        *data.get_temp_mut_or(key, state) = state;
        false
    })
}

pub(crate) fn help_button(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    title: &str,
    text: &str,
) -> Response {
    let response = ui
        .scope_builder(
            egui::UiBuilder::new().id(egui::Id::new(("context-help-button", id))),
            |ui| command(ui, "", true, CommandKind::Help, egui::Vec2::ZERO),
        )
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), title)
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_description(text);
    });
    response
}

pub(crate) fn help(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    title: &str,
    text: &str,
) -> Response {
    let response = help_button(ui, id, title, text);
    show_help(ui, response, title, text, None)
}

/// Finish an allocated help owner after its related controls have been laid out.
pub(crate) fn show_help(
    ui: &mut egui::Ui,
    response: Response,
    title: &str,
    text: &str,
    protected: Option<egui::Rect>,
) -> Response {
    let popup_id = response.id.with("pinned-context-help");
    if !response.enabled() {
        // A disabled parent must not retain a previously open actionable overlay.
        egui::Popup::close_id(ui.ctx(), popup_id);
        ui.ctx()
            .data_mut(|data| data.remove::<HelpDismiss>(response.id.with("help-dismiss")));
        paint_help_icon(ui, &response, false);
        return response;
    }
    let width = ui
        .spacing()
        .tooltip_width
        .min(ui.ctx().content_rect().width() - 2.0 * crate::ui_tokens::PAGE_MARGIN)
        .max(crate::ui_tokens::CONTROL_HEIGHT);
    let layout = protected.map(|rect| protected_help_layout(ui.ctx(), &response, rect, true));
    let was_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    place_help_popup(
        egui::Popup::from_toggle_button_response(&response)
            .id(popup_id)
            .width(width)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
        &response,
        protected,
        layout,
    )
    .show(|ui| {
        ui.set_max_width(width);
        help_content(
            ui,
            Some(title),
            text.into(),
            layout.map(|layout| layout.height),
            Some(response.id),
        );
    });
    let dismissed = was_open
        && !egui::Popup::is_id_open(ui.ctx(), popup_id)
        && ui.input(|input| input.key_pressed(egui::Key::Escape));
    if dismissed {
        // Return to the help owner only for Escape; outside clicks keep their new target.
        response.request_focus();
    }
    let automatic_help_allowed = help_after_dismiss(ui, &response, dismissed);
    let pinned = egui::Popup::is_id_open(ui.ctx(), popup_id);
    paint_help_icon(ui, &response, pinned);
    if pinned || !automatic_help_allowed {
        response
    } else {
        context_help_placement(response, text, protected)
    }
}

// A protected explanation chooses its side and scroll budget together. Otherwise
// Area's viewport constraint can shift an oversized popup over the protected row.
#[derive(Clone, Copy)]
struct ProtectedHelpLayout {
    align: Option<egui::RectAlign>,
    gap: f32,
    height: f32,
}

fn protected_help_layout(
    ctx: &egui::Context,
    owner: &Response,
    protected: egui::Rect,
    titled: bool,
) -> ProtectedHelpLayout {
    let viewport = ctx.content_rect();
    let anchor = owner.rect.union(protected);
    let style = ctx.style();
    let frame = egui::Frame::popup(&style);
    let chrome = frame.total_margin().sum().y;
    let shadow = frame.shadow.margin();
    let font = egui::TextStyle::Body.resolve(&style);
    let row = ctx.fonts_mut(|fonts| fonts.row_height(&font));
    // Keep at least two body lines, and a title row when pinned. Smaller slots
    // cannot provide a usable explanation, even if a one-pixel scroll area fits.
    let minimum = 2.0 * row
        + if titled {
            row + style.spacing.item_spacing.y
        } else {
            0.0
        };
    let above_gap = 4.0 + shadow.bottom.max(0.0);
    let below_gap = 4.0 + shadow.top.max(0.0);
    let above = anchor.top() - viewport.top() - above_gap - chrome - shadow.top.max(0.0);
    let below = viewport.bottom() - anchor.bottom() - below_gap - chrome - shadow.bottom.max(0.0);
    let (height, align, gap) = if above >= below {
        (above, egui::RectAlign::TOP_START, above_gap)
    } else {
        (below, egui::RectAlign::BOTTOM_START, below_gap)
    };
    if height >= minimum {
        ProtectedHelpLayout {
            align: Some(align),
            gap,
            height,
        }
    } else {
        // No readable exterior slot: keep the existing readable overflow fallback.
        // This deliberately does not claim that the protected row stays uncovered.
        ProtectedHelpLayout {
            align: None,
            gap: 4.0,
            height: (viewport.height() - 2.0 * crate::ui_tokens::PAGE_MARGIN).max(1.0),
        }
    }
}

fn place_help_popup<'a>(
    popup: egui::Popup<'a>,
    owner: &Response,
    protected: Option<egui::Rect>,
    layout: Option<ProtectedHelpLayout>,
) -> egui::Popup<'a> {
    if let Some(protected) = protected {
        let popup = popup.anchor(owner.rect.union(protected));
        if let Some(layout) = layout.filter(|layout| layout.align.is_some()) {
            popup
                .align(layout.align.expect("exterior alignment"))
                .align_alternatives(&[])
                .gap(layout.gap)
        } else {
            popup
                .align(egui::RectAlign::TOP_START)
                .align_alternatives(&egui::RectAlign::MENU_ALIGNS)
                .gap(4.0)
        }
    } else {
        popup
    }
}

fn help_content(
    ui: &mut egui::Ui,
    title: Option<&str>,
    text: egui::WidgetText,
    height: Option<f32>,
    keyboard_owner: Option<egui::Id>,
) {
    let content = |ui: &mut egui::Ui| {
        if let Some(title) = title {
            ui.label(egui::RichText::new(title).strong());
        }
        ui.add(egui::Label::new(text).wrap());
    };
    if let Some(height) = height {
        egui::ScrollArea::vertical()
            .id_salt("help-overflow")
            .min_scrolled_height(0.0)
            .max_height(height)
            .show_viewport(ui, |ui, viewport| {
                content(ui);
                let content_height = ui.min_rect().height();
                let owner_focused = keyboard_owner.is_some_and(|id| {
                    ui.memory(|memory| memory.has_focus(id))
                        && egui::Popup::is_id_open(ui.ctx(), id.with("pinned-context-help"))
                });
                if owner_focused && ui.is_enabled() && content_height > viewport.height() {
                    let page =
                        (viewport.height() - ui.text_style_height(&egui::TextStyle::Body)).max(1.0);
                    let delta = ui.input_mut(|input| {
                        if !input.modifiers.is_none() {
                            return 0.0;
                        }
                        let mut delta = 0.0;
                        for (key, amount) in [
                            (egui::Key::PageUp, page),
                            (egui::Key::PageDown, -page),
                            (egui::Key::Home, content_height),
                            (egui::Key::End, -content_height),
                        ] {
                            if input.consume_key(egui::Modifiers::NONE, key) {
                                delta = amount;
                            }
                        }
                        delta
                    });
                    if delta != 0.0 {
                        // The focused info button remains the keyboard owner. This
                        // public scroll request is consumed by this ScrollArea only;
                        // egui clamps Home/End to its natural content bounds.
                        ui.scroll_with_delta_animation(
                            egui::vec2(0.0, delta),
                            egui::style::ScrollAnimation::none(),
                        );
                    }
                }
            });
    } else {
        content(ui);
    }
}

pub(crate) fn context_help(response: Response, text: impl Into<egui::WidgetText>) -> Response {
    context_help_placement(response, text, None)
}

fn context_help_placement(
    response: Response,
    text: impl Into<egui::WidgetText>,
    protected: Option<egui::Rect>,
) -> Response {
    if response.enabled() && response.has_focus() && !response.hovered() {
        // Snapshot under one Context lock; resolve responses after releasing it.
        // Work is limited to the focused/nonhover case, with no heap allocation.
        let (hovered_ids, truncated) = response.ctx.interaction_snapshot(|state| {
            let mut ids = [None; 16];
            for (slot, id) in ids.iter_mut().zip(state.hovered.iter()) {
                *slot = Some(*id);
            }
            (ids, state.hovered.len() > ids.len())
        });
        let other_interactive_hover = truncated
            || hovered_ids
                .into_iter()
                .flatten()
                .filter(|id| *id != response.id)
                .any(|id| {
                    response.ctx.read_response(id).is_some_and(|other| {
                        other.enabled() && other.sense.interactive() && other.hovered()
                    })
                });
        if !other_interactive_hover {
            let width = response.ctx.style().spacing.tooltip_width.min(
                (response.ctx.content_rect().width() - 2.0 * crate::ui_tokens::PAGE_MARGIN)
                    .max(crate::ui_tokens::CONTROL_HEIGHT),
            );
            let layout =
                protected.map(|rect| protected_help_layout(&response.ctx, &response, rect, false));
            let mut tooltip = egui::Tooltip::for_widget(&response);
            tooltip.popup = place_help_popup(tooltip.popup, &response, protected, layout);
            if protected.is_some() {
                tooltip.popup = tooltip.popup.width(width);
            }
            tooltip.show(|ui| {
                if protected.is_some() {
                    ui.set_max_width(width);
                    help_content(
                        ui,
                        None,
                        text.into(),
                        layout.map(|layout| layout.height),
                        None,
                    );
                } else {
                    ui.set_max_width(ui.spacing().tooltip_width);
                    ui.add(egui::Label::new(text));
                }
            });
            return response;
        }
    }
    if protected.is_some() {
        let width = response.ctx.style().spacing.tooltip_width.min(
            (response.ctx.content_rect().width() - 2.0 * crate::ui_tokens::PAGE_MARGIN)
                .max(crate::ui_tokens::CONTROL_HEIGHT),
        );
        let layout =
            protected.map(|rect| protected_help_layout(&response.ctx, &response, rect, false));
        let mut tooltip = egui::Tooltip::for_enabled(&response);
        tooltip.popup = place_help_popup(tooltip.popup.width(width), &response, protected, layout);
        tooltip.show(|ui| {
            ui.set_max_width(width);
            help_content(
                ui,
                None,
                text.into(),
                layout.map(|layout| layout.height),
                None,
            );
        });
        response
    } else {
        response.on_hover_text(text)
    }
}

#[cfg(test)]
mod keyboard_context_help_tests {
    use super::*;
    struct Fixture {
        ctx: egui::Context,
        path: String,
        enabled: bool,
        time: f64,
    }
    impl Fixture {
        fn new(dark: bool) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            Self {
                ctx,
                path: "/synthetic/owned-fixture/long-parent/full-tool-path.exe".into(),
                enabled: true,
                time: 0.0,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (Response, Response, egui::FullOutput) {
            self.time += 1.0;
            let mut edit = None;
            let mut browse = None;
            let out = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label(crate::catalog::TOOL_PATH);
                        ui.add_enabled_ui(self.enabled, |ui| {
                            ui.horizontal(|ui| {
                                let input = text_input_with_id(
                                    ui,
                                    &mut self.path,
                                    32768,
                                    260.0,
                                    egui::Id::new("context-help-path"),
                                );
                                let input = context_help(input, self.path.as_str());
                                edit = Some(input);
                                browse = Some(context_help(
                                    secondary_sized(
                                        ui,
                                        crate::catalog::TOOL_BROWSE_YTDLP,
                                        true,
                                        egui::vec2(112.0, 32.0),
                                    ),
                                    crate::catalog::TOOL_BROWSE_HELP,
                                ));
                            });
                        });
                    });
                },
            );
            (edit.unwrap(), browse.unwrap(), out)
        }
    }
    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn painted_tooltip(out: &egui::FullOutput, target: &Response, text: &str) -> usize {
        out.shapes
            .iter()
            .filter(|shape| match &shape.shape {
                egui::Shape::Text(t) if t.galley.text() == text => {
                    let glyph = t.visual_bounding_rect();
                    // Exclude the actual TextEdit's own text; observe the tooltip label
                    // outside its owner, with all actual glyphs inside the painter clip.
                    !target.rect.intersects(glyph) && shape.clip_rect.contains_rect(glyph)
                }
                _ => false,
            })
            .count()
    }
    #[test]
    fn actual_keyboard_context_help_paints_and_preserves_path_caret_hover_and_disabled() {
        for dark in [false, true] {
            for field in [true, false] {
                let mut f = Fixture::new(dark);
                let original = f.path.clone();
                let (idle_input, idle_browse, out) = f.frame(vec![]);
                let target = if field { &idle_input } else { &idle_browse };
                let help = if field {
                    original.as_str()
                } else {
                    crate::catalog::TOOL_BROWSE_HELP
                };
                assert!(!target.has_focus() && !target.hovered());
                assert_eq!(painted_tooltip(&out, target, help), 0);
                f.frame(vec![key(egui::Key::Tab)]);
                if !field {
                    f.frame(vec![key(egui::Key::Tab)]);
                }
                let mut frame = f.frame(vec![]);
                for _ in 0..2 {
                    frame = f.frame(vec![]);
                }
                let (input, browse, out) = frame;
                let focused = if field { &input } else { &browse };
                let original_id = if field { idle_input.id } else { idle_browse.id };
                assert_eq!(focused.id, original_id);
                assert!(
                    focused.has_focus(),
                    "actual Tab focus ACK: field={field} dark={dark}"
                );
                assert!(!focused.hovered());
                assert_eq!(f.path, original);
                assert_eq!(
                    painted_tooltip(&out, focused, help),
                    1,
                    "exact actual nonhover keyboard tooltip paint: field={field} dark={dark}"
                );
                if field {
                    let (moved, _, out) =
                        f.frame(vec![key(egui::Key::End), key(egui::Key::ArrowLeft)]);
                    let cursor = egui::text_edit::TextEditState::load(&f.ctx, original_id)
                        .unwrap()
                        .cursor
                        .char_range()
                        .expect("actual focused path caret");
                    assert!(cursor.is_empty());
                    assert_eq!(
                        cursor.primary.index,
                        original.chars().count() - 1,
                        "tooltip does not intercept editing keys"
                    );
                    let (settled, _, settled_out) = f.frame(vec![]);
                    assert!(settled.has_focus());
                    assert_eq!(painted_tooltip(&settled_out, &settled, help), 1);
                    assert_eq!(
                        egui::text_edit::TextEditState::load(&f.ctx, original_id)
                            .unwrap()
                            .cursor
                            .char_range(),
                        Some(cursor),
                        "actual saved caret survives tooltip paint"
                    );
                    assert!(moved.has_focus());
                    assert_eq!(painted_tooltip(&out, &moved, help), 1);
                } else {
                    let (_, activated, _) = f.frame(vec![key(egui::Key::Enter)]);
                    assert!(activated.clicked(), "help preserves command key activation");
                    assert_eq!(activated.id, original_id);
                }
                f.ctx.memory_mut(|m| m.surrender_focus(original_id));
                let (input, browse, out) = f.frame(vec![egui::Event::PointerGone]);
                let unfocused = if field { &input } else { &browse };
                assert!(!unfocused.has_focus());
                assert_eq!(
                    painted_tooltip(&out, unfocused, help),
                    0,
                    "focus off closes optional tooltip"
                );
                let pos = unfocused.rect.center();
                f.frame(vec![egui::Event::PointerMoved(pos)]);
                let mut hovered = f.frame(vec![]);
                for _ in 0..2 {
                    hovered = f.frame(vec![]);
                }
                let (input, browse, out) = hovered;
                let target = if field { &input } else { &browse };
                assert!(target.hovered() && !target.has_focus());
                assert_eq!(target.id, original_id);
                assert_eq!(
                    painted_tooltip(&out, target, help),
                    1,
                    "original native hover tooltip still paints once"
                );
                f.enabled = false;
                let (input, browse, out) = f.frame(vec![egui::Event::Text("X".into())]);
                let disabled = if field { &input } else { &browse };
                assert!(!disabled.enabled());
                assert_eq!(disabled.id, original_id);
                assert!(!disabled.clicked() && !disabled.changed());
                assert_eq!(f.path, original, "disabled field keeps original path");
                assert_eq!(
                    painted_tooltip(&out, disabled, help),
                    0,
                    "existing enabled-only tooltip gate retained"
                );
            }
        }
    }
    #[test]
    fn actual_context_help_other_widget_hover_wins_without_changing_keyboard_focus() {
        for dark in [false, true] {
            for field in [true, false] {
                let mut f = Fixture::new(dark);
                let original = f.path.clone();
                let (initial_input, initial_browse, _) = f.frame(vec![]);
                f.frame(vec![key(egui::Key::Tab)]);
                if !field {
                    f.frame(vec![key(egui::Key::Tab)]);
                }
                let mut focused_frame = f.frame(vec![]);
                for _ in 0..2 {
                    focused_frame = f.frame(vec![]);
                }
                let (input, browse, out) = focused_frame;
                let focused = if field { &input } else { &browse };
                let focus_help = if field {
                    original.as_str()
                } else {
                    crate::catalog::TOOL_BROWSE_HELP
                };
                assert!(
                    focused.has_focus() && !focused.hovered(),
                    "actual initial keyboardfocus ACK"
                );
                assert_eq!(
                    painted_tooltip(&out, focused, focus_help),
                    1,
                    "setup: original focus tooltip genuinely paints"
                );
                let focus_id = focused.id;
                let saved_caret = egui::text_edit::TextEditState::load(&f.ctx, initial_input.id)
                    .unwrap()
                    .cursor
                    .char_range();
                let other = if field { &browse } else { &input };
                let hover_help = if field {
                    crate::catalog::TOOL_BROWSE_HELP
                } else {
                    original.as_str()
                };
                let pos = other.rect.center();
                f.frame(vec![egui::Event::PointerMoved(pos)]);
                let mut hovered_frame = f.frame(vec![]);
                for _ in 0..3 {
                    hovered_frame = f.frame(vec![]);
                }
                let (input, browse, out) = hovered_frame;
                let focused = if field { &input } else { &browse };
                let other = if field { &browse } else { &input };
                assert_eq!(input.id, initial_input.id);
                assert_eq!(browse.id, initial_browse.id);
                assert_eq!(focused.id, focus_id);
                assert!(
                    focused.has_focus() && !focused.hovered(),
                    "hover never transfers existing keyboardfocus"
                );
                assert!(
                    other.hovered() && !other.has_focus(),
                    "actual other interactive widget hover ACK: pointer={pos:?} other={:?} layer_at_pointer={:?}",
                    other.rect,
                    f.ctx.layer_id_at(pos)
                );
                assert_eq!(f.path, original);
                assert_eq!(
                    egui::text_edit::TextEditState::load(&f.ctx, initial_input.id)
                        .unwrap()
                        .cursor
                        .char_range(),
                    saved_caret,
                    "pointer hover preserves saved input caret"
                );
                assert_eq!(
                    painted_tooltip(&out, other, hover_help),
                    1,
                    "other widget native hover help paints exactlyonce after bounded original delay: fieldfocus={field} dark={dark}"
                );
                assert_eq!(
                    painted_tooltip(&out, focused, focus_help),
                    0,
                    "focused optional help must not intercept other hover help"
                );
                // Once the pointer leaves, the original focused owner may show context again.
                let mut restored = f.frame(vec![egui::Event::PointerGone]);
                for _ in 0..2 {
                    restored = f.frame(vec![]);
                }
                let (input, browse, out) = restored;
                let focused = if field { &input } else { &browse };
                assert!(focused.has_focus());
                assert_eq!(focused.id, focus_id);
                assert_eq!(painted_tooltip(&out, focused, focus_help), 1);
                assert_eq!(f.path, original);
            }
        }
    }
}

#[cfg(test)]
mod optional_help_button_tests {
    use super::*;
    use std::collections::HashMap;

    struct Fixture {
        ctx: egui::Context,
        nodes: HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        enabled: bool,
        reversed: bool,
        time: f64,
        step: f64,
        trace: bool,
        inline_title: bool,
        other: Option<Response>,
        outside_input: bool,
        other_text: String,
        viewport: egui::Rect,
    }
    impl Fixture {
        fn new(dark: bool, zoom: f32) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            Self {
                ctx,
                nodes: HashMap::new(),
                enabled: true,
                reversed: false,
                time: 0.0,
                step: 1.0,
                trace: false,
                inline_title: false,
                other: None,
                outside_input: false,
                other_text: String::new(),
                viewport: egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0 / zoom, 600.0 / zoom),
                ),
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (Response, egui::FullOutput) {
            self.time += self.step;
            let mut passes = 0;
            let mut target = None;
            let mut other = None;
            let out = self.ctx.run(egui::RawInput {
                screen_rect: Some(self.viewport), focused: true, time: Some(self.time), events,
                ..Default::default()
            }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(self.enabled, |ui| {
                        ui.horizontal(|ui| {
                            if self.inline_title {
                                ui.label("설치 안 됨");
                            }
                            if self.reversed {
                                other = Some(if self.outside_input {
                                    ui.add(egui::TextEdit::singleline(&mut self.other_text).id(egui::Id::new("outside-help-input")))
                                } else {
                                    help(ui, "other", "다른 설명", "다른 선택의 설명입니다.")
                                });
                            }
                            target = Some(help(ui, "quality", "품질 선택 도움말", "원하는 품질을 고릅니다. 긴 설명은 창 안에서 줄바꿈되어야 합니다."));
                            passes += 1;
                            if self.trace {
                                assert!(passes <= 4, "bounded actual help render passes");
                                let actual = target.as_ref().unwrap();
                                eprintln!("help-actual-pass time={} pass={} clicked={} down={} pinned={}", self.time, passes, actual.clicked(), actual.is_pointer_button_down_on(), egui::Popup::is_id_open(ctx, actual.id.with("pinned-context-help")));
                            }
                            if !self.reversed {
                                other = Some(if self.outside_input {
                                    ui.add(egui::TextEdit::singleline(&mut self.other_text).id(egui::Id::new("outside-help-input")))
                                } else {
                                    help(ui, "other", "다른 설명", "다른 선택의 설명입니다.")
                                });
                            }
                        });
                    });
                });
            });
            if let Some(update) = &out.platform_output.accesskit_update {
                for (id, node) in &update.nodes {
                    self.nodes.insert(*id, node.clone());
                }
            }
            self.other = other;
            (target.unwrap(), out)
        }
        fn node(&self) -> (egui::accesskit::NodeId, &egui::accesskit::Node) {
            self.nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == egui::accesskit::Role::Button
                        && n.label() == Some("품질 선택 도움말")
                })
                .map(|(id, node)| (*id, node))
                .expect("actual named help Button from incremental AccessKit updates")
        }
    }
    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn title_paint(out: &egui::FullOutput, viewport: egui::Rect) -> usize {
        out.shapes
            .iter()
            .filter(|shape| match &shape.shape {
                egui::Shape::Text(t) if t.galley.text() == "품질 선택 도움말" => {
                    let glyph = t.visual_bounding_rect();
                    assert!(
                        shape.clip_rect.contains_rect(glyph) && viewport.contains_rect(glyph),
                        "actual pinned Korean heading fully clipped to viewport"
                    );
                    true
                }
                egui::Shape::Text(t)
                    if t.galley.text()
                        == "원하는 품질을 고릅니다. 긴 설명은 창 안에서 줄바꿈되어야 합니다." =>
                {
                    let glyph = t.visual_bounding_rect();
                    assert!(
                        shape.clip_rect.contains_rect(glyph) && viewport.contains_rect(glyph),
                        "actual optional explanation glyphs remain inside popup/tooltip viewport"
                    );
                    false
                }
                _ => false,
            })
            .count()
    }
    #[test]
    fn actual_info_icon_native_hitarea_inline_alignment_and_states() {
        fn ring(out: &egui::FullOutput, response: &Response) -> (Color32, Color32) {
            let circles: Vec<_> = out
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::Shape::Circle(circle) = &shape.shape
                        && circle.center.distance(response.rect.center()) < 0.01
                        && (circle.radius - 7.25).abs() < 0.01
                    {
                        let bounds =
                            egui::Rect::from_center_size(circle.center, egui::vec2(16.0, 16.0));
                        assert!(
                            response.rect.contains_rect(bounds)
                                && shape.clip_rect.contains_rect(bounds),
                            "actual vector icon fully inside hit area and clip"
                        );
                        Some((circle.stroke.color, circle.fill))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                circles.len(),
                1,
                "one actual16px vector ring, not a font glyph"
            );
            assert!(!out.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == "?")), "help icon does not depend on question-mark font paint");
            circles[0]
        }
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                let mut f = Fixture::new(dark, zoom);
                f.inline_title = true;
                let (idle, out) = f.frame(vec![]);
                assert_eq!(idle.rect.size(), egui::vec2(32.0, 32.0));
                let idle_ring = ring(&out, &idle);
                let title = out
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Text(text) if text.galley.text() == "설치 안 됨" => {
                            Some(text.visual_bounding_rect())
                        }
                        _ => None,
                    })
                    .expect("actual adjacent Korean title paint");
                assert!(
                    title.right() <= idle.rect.left()
                        && (title.center().y - idle.rect.center().y).abs() <= 3.0,
                    "title and information icon align on the same row"
                );
                f.step = 0.05; // Original native click threshold; avoid a synthetic long press.
                let point = idle.rect.center();
                let (hover, out) = f.frame(vec![egui::Event::PointerMoved(point)]);
                assert!(hover.hovered());
                let hover_ring = ring(&out, &hover);
                assert_ne!(idle_ring.0, hover_ring.0);
                let (pressed, out) = f.frame(vec![pointer(point, true)]);
                assert!(pressed.is_pointer_button_down_on());
                assert_ne!(ring(&out, &pressed).0, hover_ring.0);
                let (pinned, out) = f.frame(vec![pointer(point, false)]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    pinned.id.with("pinned-context-help")
                ));
                assert_ne!(ring(&out, &pinned).1, Color32::TRANSPARENT);
                assert_eq!(pinned.id, idle.id);
                let (focus, _) = f.frame(vec![egui::Event::PointerGone, key(egui::Key::Escape)]);
                assert!(focus.has_focus());
                // Escape requests owner focus after command paint in this pass.
                // Require actual focus immediately, then inspect its next settled paint.
                let (settled_focus, out) = f.frame(vec![]);
                assert!(settled_focus.has_focus());
                assert_eq!(settled_focus.id, focus.id);
                assert!(out.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.rect == settled_focus.rect.shrink(1.0) && rect.stroke.width == 1.5)), "actual settled focus outline retained independently of icon state");
                f.enabled = false;
                let (disabled, out) = f.frame(vec![]);
                assert!(!disabled.enabled());
                assert_ne!(ring(&out, &disabled).0, idle_ring.0);
                assert!(!egui::Popup::is_id_open(
                    &f.ctx,
                    disabled.id.with("pinned-context-help")
                ));
                assert_eq!(disabled.id, idle.id);
            }
        }
    }

    #[test]
    fn actual_escape_help_dismisses_body_until_new_hover_focus_or_activation() {
        fn body_count(out: &egui::FullOutput) -> usize {
            out.shapes.iter().filter(|shape| matches!(&shape.shape, egui::Shape::Text(t) if t.galley.text() == "원하는 품질을 고릅니다. 긴 설명은 창 안에서 줄바꿈되어야 합니다.")).count()
        }
        for dark in [false, true] {
            for hovered in [false, true] {
                let mut f = Fixture::new(dark, 1.0);
                // A pointer click focuses TextEdit naturally; egui Buttons retain no
                // keyboard focus on pointer activation. Test an actual focus target.
                f.outside_input = true;
                let (initial, _) = f.frame(vec![]);
                let (focused, _) = f.frame(vec![key(egui::Key::Tab)]);
                assert!(focused.has_focus());
                if hovered {
                    let (hover, _) =
                        f.frame(vec![egui::Event::PointerMoved(initial.rect.center())]);
                    assert!(hover.hovered());
                }
                f.frame(vec![key(egui::Key::Enter)]);
                let (_, pinned) = f.frame(vec![]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                assert_eq!(title_paint(&pinned, f.viewport), 1);
                assert_eq!(body_count(&pinned), 1);
                f.frame(vec![key(egui::Key::Escape)]);
                for _ in 0..3 {
                    let (held, closed) = f.frame(vec![]);
                    assert!(
                        held.has_focus(),
                        "dismissal preserves the actual owner keyboard focus"
                    );
                    assert_eq!(held.hovered(), hovered);
                    assert!(!egui::Popup::is_id_open(
                        &f.ctx,
                        initial.id.with("pinned-context-help")
                    ));
                    assert_eq!(title_paint(&closed, f.viewport), 0);
                    assert_eq!(
                        body_count(&closed),
                        0,
                        "Escape must also dismiss the automatic focused/hover explanation"
                    );
                }
                if hovered {
                    let (leaving, out) = f.frame(vec![egui::Event::PointerGone]);
                    assert!(leaving.has_focus(), "pointer exit preserves owner focus");
                    assert_eq!(body_count(&out), 0);
                    let (exit, out) = f.frame(vec![]);
                    assert!(
                        !exit.hovered() && exit.has_focus(),
                        "settled pointer exit: hovered={} focused={} pointer={:?}",
                        exit.hovered(),
                        exit.has_focus(),
                        f.ctx.input(|i| i.pointer.hover_pos())
                    );
                    assert_eq!(body_count(&out), 0);
                    let (entered, _) =
                        f.frame(vec![egui::Event::PointerMoved(initial.rect.center())]);
                    assert!(entered.hovered());
                    f.frame(vec![]);
                    let (_, shown) = f.frame(vec![]);
                    assert_eq!(
                        body_count(&shown),
                        1,
                        "new pointer hover entry restores optional help"
                    );
                } else {
                    let (away, _) = f.frame(vec![key(egui::Key::Tab)]);
                    assert!(!away.has_focus(), "actual Tab leaves help owner");
                    let mut back = key(egui::Key::Tab);
                    if let egui::Event::Key { modifiers, .. } = &mut back {
                        modifiers.shift = true;
                    }
                    f.frame(vec![back]);
                    // egui Focus::interested_in_focus defers Previous through id_next_frame
                    // so the earlier widget observes gained_focus in the next frame.
                    let (returned, shown) = f.frame(vec![]);
                    assert!(
                        returned.has_focus(),
                        "actual Shift+Tab re-enters owner on next frame"
                    );
                    assert_eq!(
                        body_count(&shown),
                        1,
                        "new keyboard focus entry restores optional help"
                    );
                }
                // Explicit keyboard activation stays available, including after another dismiss.
                f.frame(vec![key(egui::Key::Enter)]);
                f.frame(vec![]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                f.frame(vec![key(egui::Key::Escape)]);
                let (_, closed) = f.frame(vec![]);
                assert_eq!(body_count(&closed), 0);
                f.frame(vec![key(egui::Key::Enter)]);
                let (_, reopened) = f.frame(vec![]);
                assert_eq!(title_paint(&reopened, f.viewport), 1);
                assert_eq!(body_count(&reopened), 1);
                f.frame(vec![key(egui::Key::Escape)]);
                let (_, closed) = f.frame(vec![]);
                assert_eq!(body_count(&closed), 0);
                f.step = 0.05;
                f.frame(vec![egui::Event::PointerMoved(initial.rect.center())]);
                f.frame(vec![pointer(initial.rect.center(), true)]);
                let (clicked, _) = f.frame(vec![pointer(initial.rect.center(), false)]);
                assert!(
                    clicked.clicked(),
                    "actual fresh pointer click reactivates the same help owner"
                );
                f.frame(vec![]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                let other = f.other.clone().expect("actual adjacent help response");
                f.frame(vec![
                    egui::Event::PointerMoved(other.rect.center()),
                    pointer(other.rect.center(), true),
                ]);
                f.frame(vec![pointer(other.rect.center(), false)]);
                let (owner, outside) = f.frame(vec![]);
                assert!(!owner.has_focus());
                assert!(
                    f.ctx.memory(|memory| memory.has_focus(other.id)),
                    "outside click retains its actual new focus target"
                );
                assert!(!egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                assert_eq!(body_count(&outside), 0);
            }
        }
    }

    #[test]
    fn actual_optional_help_keyboard_pinned_close_and_stable_identity() {
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                let mut f = Fixture::new(dark, zoom);
                let (initial, _) = f.frame(vec![]);
                let (node_id, node) = f.node();
                assert_eq!(
                    node.description(),
                    Some("원하는 품질을 고릅니다. 긴 설명은 창 안에서 줄바꿈되어야 합니다.")
                );
                assert!(!node.is_disabled());
                assert!(initial.rect.width() >= 28.0 && initial.rect.height() >= 28.0);
                let (focused, _) = f.frame(vec![key(egui::Key::Tab)]);
                assert!(
                    focused.has_focus(),
                    "actual Tab reaches independent help control"
                );
                f.frame(vec![key(egui::Key::Enter)]);
                let (_, opened) = f.frame(vec![]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                assert_eq!(title_paint(&opened, f.viewport), 1);
                f.reversed = true;
                let (reordered, _) = f.frame(vec![]);
                assert_eq!(reordered.id, initial.id);
                assert_eq!(f.node().0, node_id);
                assert!(
                    reordered.has_focus(),
                    "layout reorder preserves logical button focus"
                );
                f.frame(vec![key(egui::Key::Escape)]);
                let (_, closed) = f.frame(vec![]);
                assert!(!egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                assert_eq!(title_paint(&closed, f.viewport), 0);
                assert!(
                    f.ctx.memory(|memory| memory.has_focus(initial.id)),
                    "Escape returns focus to the help owner"
                );
                f.frame(vec![key(egui::Key::Space)]);
                f.frame(vec![]);
                assert!(egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                f.enabled = false;
                let (disabled, _) = f.frame(vec![]);
                assert!(!disabled.enabled() && f.node().1.is_disabled());
                assert!(!egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
                f.frame(vec![key(egui::Key::Enter)]);
                assert!(!egui::Popup::is_id_open(
                    &f.ctx,
                    initial.id.with("pinned-context-help")
                ));
            }
        }
    }
    #[test]
    fn actual_optional_help_pointer_hover_open_and_outside_close() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark, 2.0);
            let (initial, _) = f.frame(vec![]);
            let pos = initial.rect.center();
            f.frame(vec![egui::Event::PointerMoved(pos)]);
            let (hovered, out) = f.frame(vec![]);
            assert!(hovered.hovered());
            assert_eq!(
                out.platform_output.cursor_icon,
                egui::CursorIcon::PointingHand
            );
            assert_eq!(
                title_paint(&out, f.viewport),
                0,
                "hover does not pin the popup"
            );
            let mut revealed = false;
            for settle in 0..4 {
                let (target, settled) = f.frame(vec![]);
                let painted = settled.shapes.iter().any(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if text.galley.text()
                            == "원하는 품질을 고릅니다. 긴 설명은 창 안에서 줄바꿈되어야 합니다." =>
                    {
                        let glyph = text.visual_bounding_rect();
                        !target.rect.intersects(glyph)
                            && shape.clip_rect.contains_rect(glyph)
                            && f.viewport.contains_rect(glyph)
                    }
                    _ => false,
                });
                eprintln!(
                    "help-hover-settle dark={dark} frame={settle} target={:?} hovered={} actual_optional_body_painted={painted}",
                    target.id,
                    target.hovered()
                );
                if painted && target.hovered() && target.contains_pointer() {
                    revealed = true;
                    break;
                }
            }
            assert!(
                revealed,
                "hover tooltip must actually finish its bounded sizing/paint before pointer activation setup"
            );
            // Hover publication still uses original waits; an actual click is shorter than 0.8s.
            f.step = 0.05;
            f.trace = true;
            let previous = f.ctx.read_response(initial.id).map(|response| {
                (
                    response.id,
                    response.rect,
                    response.interact_rect,
                    response.enabled(),
                    response.sense,
                )
            });
            let (pressed, _) = f.frame(vec![pointer(pos, true)]);
            let hit_ids = f.ctx.interaction_snapshot(|state| {
                state
                    .contains_pointer
                    .iter()
                    .take(16)
                    .copied()
                    .collect::<Vec<_>>()
            });
            let hit_responses = hit_ids
                .into_iter()
                .map(|id| {
                    f.ctx.read_response(id).map(|response| {
                        (
                            response.id,
                            response.rect,
                            response.interact_rect,
                            response.enabled(),
                            response.sense,
                        )
                    })
                })
                .collect::<Vec<_>>();
            eprintln!(
                "help-press-registration previous={previous:?} actual_interact={:?} enabled={} contains_responses={hit_responses:?}",
                pressed.interact_rect,
                pressed.enabled()
            );
            assert_eq!(
                pressed.id, initial.id,
                "actual down event retains target identity"
            );
            eprintln!(
                "help-pointer dark={dark} response={:?} layer={:?} focus={} contains={} hovered={} input={:?} top_layer={:?} interaction={:?}",
                pressed.id,
                pressed.layer_id,
                pressed.has_focus(),
                pressed.contains_pointer(),
                pressed.hovered(),
                f.ctx
                    .input(|input| (input.pointer.interact_pos(), input.pointer.primary_down())),
                f.ctx.layer_id_at(pos),
                f.ctx.interaction_snapshot(|state| (
                    state.clicked,
                    state.dragged,
                    state.hovered.iter().copied().collect::<Vec<_>>()
                ))
            );
            assert!(
                pressed.is_pointer_button_down_on(),
                "actual owned Context pointer press hits help Button: initial={:?} pressed={:?}",
                initial.rect,
                pressed.rect
            );
            let (released, _) = f.frame(vec![pointer(pos, false)]);
            assert!(
                released.clicked(),
                "actual help release must dispatch click before popup assertion: target={:?}",
                released.rect
            );
            let (_, opened) = f.frame(vec![]);
            assert!(egui::Popup::is_id_open(
                &f.ctx,
                initial.id.with("pinned-context-help")
            ));
            assert_eq!(title_paint(&opened, f.viewport), 1);
            let outside = f.viewport.right_bottom() - egui::vec2(12.0, 12.0);
            f.frame(vec![
                egui::Event::PointerMoved(outside),
                pointer(outside, true),
            ]);
            f.frame(vec![pointer(outside, false)]);
            let (_, closed) = f.frame(vec![]);
            assert!(!egui::Popup::is_id_open(
                &f.ctx,
                initial.id.with("pinned-context-help")
            ));
            assert_eq!(title_paint(&closed, f.viewport), 0);
        }
    }
}

#[cfg(test)]
mod protected_help_placement_tests {
    use super::*;
    const TITLE: &str = "환경 설정 도움말";
    const BODY: &str =
        "절대 경로를 선택하거나 기본 도구를 사용하세요. 입력한 경로는 실행 전에 확인합니다.";

    struct Harness {
        ctx: egui::Context,
        viewport: egui::Rect,
        top: f32,
        narrow_width: Option<f32>,
        time: f64,
        text: String,
        path: String,
        owner: Option<Response>,
        other: Option<Response>,
        show_other: bool,
        protected: egui::Rect,
        content: egui::Rect,
    }
    impl Harness {
        fn new(dark: bool, zoom: f32, size: egui::Vec2, top: f32, text: &str) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.enable_accesskit();
            let mut harness = Self {
                ctx,
                viewport: egui::Rect::from_min_size(egui::Pos2::ZERO, size),
                top,
                narrow_width: None,
                time: 0.0,
                text: text.into(),
                path: "C:\\synthetic\\한글-tool.exe".into(),
                owner: None,
                other: None,
                show_other: false,
                protected: egui::Rect::NOTHING,
                content: egui::Rect::NOTHING,
            };
            harness.frame(vec![]);
            harness
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += 0.5;
            self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(self.viewport),
                    time: Some(self.time),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if let Some(width) = self.narrow_width { ui.set_max_width(width); }
                        self.content = ui.max_rect().intersect(ui.clip_rect());
                        ui.add_space(self.top);
                        let help = ui
                            .horizontal(|ui| {
                                ui.label("실행 파일");
                                help_button(ui, "placement-fixture", TITLE, &self.text)
                            })
                            .inner;
                        let controls = ui
                            .horizontal(|ui| {
                                let input = text_input_with_id(
                                    ui,
                                    &mut self.path,
                                    32768,
                                    (ui.available_width()
                                        - 112.0
                                        - ui.spacing().item_spacing.x
                                        - TEXT_INPUT_HORIZONTAL_MARGIN)
                                        .max(0.0),
                                    egui::Id::new("protected-tool-input"),
                                );
                                let browse =
                                    secondary_sized(ui, "찾아보기", true, egui::vec2(112.0, 32.0));
                                input.rect.union(browse.rect)
                            })
                            .inner;
                        self.protected = controls;
                        assert!(self.content.contains_rect(controls),
                            "actual input+browse must fit the original content rect: {controls:?} {:?}", self.content);
                        assert!(self.viewport.contains_rect(controls),
                            "actual input+browse must fit viewport in fitting and fallback scenes: {controls:?} {:?}", self.viewport);
                        self.owner = Some(show_help(ui, help, TITLE, &self.text, Some(controls)));
                        if self.show_other { self.other = Some(super::help(ui,"keyboard-other-help","Other help","Short unrelated explanation")); }
                    });
                },
            )
        }
        fn settle(&mut self) -> egui::FullOutput {
            let mut output = self.frame(vec![]);
            for _ in 0..8 {
                output = self.frame(vec![]);
            }
            output
        }
        fn assert_clear(&self, popup_id: egui::Id) {
            let popup = egui::AreaState::load(&self.ctx, popup_id)
                .expect("actual shown Area")
                .rect();
            let owner = self.owner.as_ref().unwrap();
            assert!(
                self.viewport.contains_rect(popup),
                "actual explanation inside viewport: {popup:?}"
            );
            assert!(
                !popup.intersects(self.protected),
                "explanation must avoid actual input+browse: {popup:?} {:?}",
                self.protected
            );
            assert!(
                !popup.intersects(owner.rect),
                "explanation must avoid its actual owner"
            );
        }
    }
    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn body_painted(output: &egui::FullOutput, body: &str) -> bool {
        output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text()==body && shape.clip_rect.intersects(text.visual_bounding_rect())))
    }
    #[test]
    fn actual_protected_help_focus_hover_pinned_avoid_controls_and_escape_preserves_owner() {
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                let size = egui::vec2(960.0 / zoom, 640.0 / zoom);
                for position in [0.12, 0.62] {
                    let mut h = Harness::new(dark, zoom, size, size.y * position, BODY);
                    let owner_id = h.owner.as_ref().unwrap().id;
                    h.frame(vec![key(egui::Key::Tab)]);
                    let focused = h.settle();
                    assert!(h.owner.as_ref().unwrap().has_focus());
                    assert!(body_painted(&focused, BODY));
                    h.assert_clear(egui::Tooltip::tooltip_id(owner_id, 0));
                    let point = h.owner.as_ref().unwrap().rect.center();
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    let hovered = h.settle();
                    assert!(h.owner.as_ref().unwrap().hovered());
                    assert!(body_painted(&hovered, BODY));
                    h.assert_clear(egui::Tooltip::tooltip_id(owner_id, 0));
                    h.frame(vec![key(egui::Key::Enter)]);
                    let pinned = h.settle();
                    let popup_id = owner_id.with("pinned-context-help");
                    assert!(egui::Popup::is_id_open(&h.ctx, popup_id));
                    assert!(body_painted(&pinned, BODY));
                    h.assert_clear(popup_id);
                    h.frame(vec![key(egui::Key::Escape)]);
                    for _ in 0..3 {
                        let closed = h.frame(vec![]);
                        assert_eq!(h.owner.as_ref().unwrap().id, owner_id);
                        assert!(h.owner.as_ref().unwrap().has_focus());
                        assert!(!egui::Popup::is_id_open(&h.ctx, popup_id));
                        assert!(!body_painted(&closed, BODY));
                    }
                    assert_eq!(h.path, "C:\\synthetic\\한글-tool.exe");
                }
            }
        }
    }
    #[test]
    fn actual_protected_help_no_space_uses_readable_scroll_and_escape() {
        let long = format!(
            "{}\nEND-OF-HELP",
            "긴 설명은 좁은 화면에서도 스크롤해서 읽을 수 있어야 합니다.\n".repeat(20)
        );
        for zoom in [1.0, 2.0] {
            let mut h = Harness::new(false, zoom, egui::vec2(240.0, 120.0), 0.0, &long);
            let owner_id = h.owner.as_ref().unwrap().id;
            h.frame(vec![key(egui::Key::Tab)]);
            h.settle();
            h.frame(vec![key(egui::Key::Enter)]);
            h.settle();
            let popup_id = owner_id.with("pinned-context-help");
            let popup = egui::AreaState::load(&h.ctx, popup_id).unwrap().rect();
            assert!(
                h.viewport.contains_rect(popup),
                "bounded fallback stays on screen: {popup:?}"
            );
            assert!(
                popup.intersects(h.protected),
                "fixture must prove there is no room for the long explanation outside the protected group"
            );
            let point = popup.center();
            h.frame(vec![egui::Event::PointerMoved(point)]);
            h.frame(vec![egui::Event::PointerMoved(point)]);
            let mut end_visible = false;
            for _ in 0..16 {
                let output = h.frame(vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -400.0),
                    modifiers: egui::Modifiers::NONE,
                }]);
                end_visible = output.shapes.iter().any(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape {
                        text.galley.rows.iter().any(|row| {
                            row.text().contains("END-OF-HELP")
                                && shape
                                    .clip_rect
                                    .contains_rect(row.rect().translate(text.pos.to_vec2()))
                        })
                    } else {
                        false
                    }
                });
                if end_visible {
                    break;
                }
            }
            assert!(
                end_visible,
                "real wheel must expose the final help line; no manual offset mutation"
            );
            h.frame(vec![key(egui::Key::Escape)]);
            let closed = h.settle();
            assert!(!egui::Popup::is_id_open(&h.ctx, popup_id));
            assert!(h.owner.as_ref().unwrap().has_focus());
            assert!(!body_painted(&closed, &long));
            assert_eq!(h.owner.as_ref().unwrap().id, owner_id);
        }
    }

    #[test]
    fn actual_small_protected_help_caps_frame_and_shadow_without_losing_last_sentence() {
        let body = format!(
            "{}\n\n{}\n{}",
            crate::catalog::DENO_HELP,
            crate::catalog::DENO_DEFAULT_EXAMPLE,
            crate::catalog::DENO_CONFLICT_VERSION
        );
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                for top in [83.0, 183.0] {
                    // The 400x300 logical scene is the 800x600 physical, 200%
                    // native failure geometry. The 100% case tests the same layout.
                    let mut h = Harness::new(dark, zoom, egui::vec2(400.0, 300.0), top, &body);
                    h.narrow_width = Some(324.0);
                    h.frame(vec![]);
                    let owner_id = h.owner.as_ref().unwrap().id;
                    h.frame(vec![key(egui::Key::Tab)]);
                    h.settle();
                    h.assert_clear(egui::Tooltip::tooltip_id(owner_id, 0));
                    h.frame(vec![key(egui::Key::Enter)]);
                    let popup_id = owner_id.with("pinned-context-help");
                    // Check the first visible result, not only a final settled frame.
                    for _ in 0..3 {
                        h.frame(vec![]);
                        assert!(egui::Popup::is_id_open(&h.ctx, popup_id));
                        h.assert_clear(popup_id);
                        let popup = egui::AreaState::load(&h.ctx, popup_id).unwrap().rect();
                        let shadow = egui::Frame::popup(&h.ctx.style()).shadow;
                        let shape = shadow.as_shape(popup, 0);
                        let painted_shadow = shape.rect.expand(0.5 * f32::from(shadow.blur));
                        assert!(
                            !painted_shadow.intersects(h.protected),
                            "actual shadow penumbra must avoid input+browse: {painted_shadow:?} {:?}",
                            h.protected
                        );
                        assert!(
                            !painted_shadow.intersects(h.owner.as_ref().unwrap().rect),
                            "actual shadow penumbra must avoid original info owner"
                        );
                    }
                    let popup = egui::AreaState::load(&h.ctx, popup_id).unwrap().rect();
                    let point = popup.center();
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    h.frame(vec![egui::Event::PointerMoved(point)]);
                    let mut last_visible = false;
                    for _ in 0..16 {
                        let output = h.frame(vec![egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, -120.0),
                            modifiers: egui::Modifiers::NONE,
                        }]);
                        h.assert_clear(popup_id);
                        last_visible = output.shapes.iter().any(|shape| {
                            if let egui::Shape::Text(text) = &shape.shape {
                                text.galley.text() == body
                                    && text.galley.rows.last().is_some_and(|row| {
                                        shape
                                            .clip_rect
                                            .contains_rect(row.rect().translate(text.pos.to_vec2()))
                                    })
                            } else {
                                false
                            }
                        });
                        if last_visible {
                            break;
                        }
                    }
                    assert!(
                        last_visible,
                        "real wheel must expose original final sentence inside its paint clip"
                    );
                    h.frame(vec![key(egui::Key::Escape)]);
                    for _ in 0..6 {
                        let closed = h.frame(vec![]);
                        assert!(!egui::Popup::is_id_open(&h.ctx, popup_id));
                        assert!(!body_painted(&closed, &body));
                        assert_eq!(h.owner.as_ref().unwrap().id, owner_id);
                        assert!(h.owner.as_ref().unwrap().has_focus());
                    }
                    // Explicit reopening is allowed, with the same owner/popup IDs.
                    h.frame(vec![key(egui::Key::Enter)]);
                    h.settle();
                    assert!(egui::Popup::is_id_open(&h.ctx, popup_id));
                    assert_eq!(h.owner.as_ref().unwrap().id, owner_id);
                    h.assert_clear(popup_id);
                    assert_eq!(h.path, "C:\\synthetic\\한글-tool.exe");
                }
            }
        }
    }

    fn keyboard_body<'a>(
        output: &'a egui::FullOutput,
        body: &str,
    ) -> (&'a egui::epaint::TextShape, egui::Rect) {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == body => {
                    Some((text, shape.clip_rect))
                }
                _ => None,
            })
            .expect("original unshortened help Text shape")
    }
    fn keyboard_row_visible(output: &egui::FullOutput, body: &str, last: bool) -> bool {
        let (text, clip) = keyboard_body(output, body);
        let row = if last {
            text.galley.rows.last()
        } else {
            text.galley.rows.first()
        }
        .unwrap();
        clip.contains_rect(row.rect().translate(text.pos.to_vec2()))
    }
    fn keyboard_action(h: &mut Harness, id: egui::Id, action: egui::accesskit::Action) {
        h.frame(vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                target: egui::accesskit::NodeId(id.value()),
                action,
                data: None,
            },
        )]);
    }
    #[test]
    fn actual_pinned_help_keyboard_pages_home_end_and_boundaries_keep_original_owner() {
        let body = format!(
            "{}\n\n{}\n{}",
            crate::catalog::DENO_HELP,
            crate::catalog::DENO_DEFAULT_EXAMPLE,
            crate::catalog::DENO_CONFLICT_VERSION
        );
        for dark in [false, true] {
            for zoom in [1.0, 2.0] {
                for top in [83.0, 183.0] {
                    let mut h = Harness::new(dark, zoom, egui::vec2(400.0, 300.0), top, &body);
                    h.narrow_width = Some(324.0);
                    h.frame(vec![key(egui::Key::Tab)]);
                    let owner = h.owner.as_ref().unwrap().id;
                    let automatic = h.settle();
                    let auto_top = keyboard_body(&automatic, &body).0.pos.y;
                    h.frame(vec![key(egui::Key::PageDown)]);
                    assert!(
                        h.ctx.input(|i| i.key_pressed(egui::Key::PageDown)),
                        "automatic tooltip must not consume page keys"
                    );
                    let automatic = h.settle();
                    assert!((keyboard_body(&automatic, &body).0.pos.y - auto_top).abs() < 0.01);
                    h.frame(vec![key(egui::Key::Enter)]);
                    let first = h.settle();
                    let popup = owner.with("pinned-context-help");
                    assert!(egui::Popup::is_id_open(&h.ctx, popup));
                    assert!(keyboard_row_visible(&first, &body, false));
                    let first_top = keyboard_body(&first, &body).0.pos.y;
                    h.frame(vec![key(egui::Key::PageDown)]);
                    assert!(
                        !h.ctx.input(|i| i.key_pressed(egui::Key::PageDown)),
                        "only pinned owner consumes page keys"
                    );
                    let down = h.settle();
                    assert!(
                        keyboard_body(&down, &body).0.pos.y < first_top - 1.0,
                        "real PageDown moves original body upward"
                    );
                    h.assert_clear(popup);
                    h.frame(vec![key(egui::Key::PageUp)]);
                    assert!(!h.ctx.input(|i| i.key_pressed(egui::Key::PageUp)));
                    let up = h.settle();
                    assert!(
                        (keyboard_body(&up, &body).0.pos.y - first_top).abs() < 0.01,
                        "PageUp returns toward the original start"
                    );
                    h.frame(vec![key(egui::Key::End)]);
                    assert!(!h.ctx.input(|i| i.key_pressed(egui::Key::End)));
                    let end = h.settle();
                    assert!(
                        keyboard_row_visible(&end, &body, true),
                        "original last glyph row must be entirely in paint clip after actual End"
                    );
                    let end_top = keyboard_body(&end, &body).0.pos.y;
                    for _ in 0..3 {
                        h.frame(vec![key(egui::Key::End)]);
                        let stopped = h.settle();
                        assert!(
                            (keyboard_body(&stopped, &body).0.pos.y - end_top).abs() < 0.01,
                            "End clamps to natural content end"
                        );
                        h.assert_clear(popup);
                        assert_eq!(h.owner.as_ref().unwrap().id, owner);
                        assert!(h.ctx.memory(|m| m.has_focus(owner)));
                    }
                    h.frame(vec![key(egui::Key::Home)]);
                    let home = h.settle();
                    assert!(keyboard_row_visible(&home, &body, false));
                    assert!((keyboard_body(&home, &body).0.pos.y - first_top).abs() < 0.01);
                    for _ in 0..2 {
                        h.frame(vec![key(egui::Key::Home)]);
                        let stopped = h.settle();
                        assert!(
                            (keyboard_body(&stopped, &body).0.pos.y - first_top).abs() < 0.01,
                            "Home clamps to natural content start"
                        );
                    }
                    // Actual focus to the original editable input must disable help key handling.
                    let field = egui::Id::new("protected-tool-input");
                    keyboard_action(&mut h, field, egui::accesskit::Action::Focus);
                    assert!(h.ctx.memory(|m| m.has_focus(field)));
                    let input_focus = h.settle();
                    let before = keyboard_body(&input_focus, &body).0.pos.y;
                    h.frame(vec![key(egui::Key::PageDown)]);
                    assert!(
                        h.ctx.input(|i| i.key_pressed(egui::Key::PageDown)),
                        "ordinary input keeps its unhandled PageDown event"
                    );
                    let input_focus = h.settle();
                    assert!(
                        (keyboard_body(&input_focus, &body).0.pos.y - before).abs() < 0.01,
                        "input focus cannot scroll pinned help"
                    );
                    assert!(h.ctx.memory(|m| m.has_focus(field)));
                    assert_eq!(h.path, "C:\\synthetic\\한글-tool.exe");
                    keyboard_action(&mut h, owner, egui::accesskit::Action::Focus);
                    assert!(h.ctx.memory(|m| m.has_focus(owner)));
                    h.frame(vec![key(egui::Key::Escape)]);
                    for _ in 0..6 {
                        let closed = h.frame(vec![]);
                        assert!(!egui::Popup::is_id_open(&h.ctx, popup));
                        assert!(!body_painted(&closed, &body));
                        assert!(h.ctx.memory(|m| m.has_focus(owner)));
                    }
                    h.frame(vec![key(egui::Key::End)]);
                    assert!(
                        h.ctx.input(|i| i.key_pressed(egui::Key::End)),
                        "closed help must leave End unconsumed"
                    );
                    let closed = h.settle();
                    assert!(!body_painted(&closed, &body));
                    assert_eq!(h.owner.as_ref().unwrap().id, owner);
                }
            }
        }
    }
    #[test]
    fn actual_pinned_help_other_owner_and_other_popup_do_not_consume_page_keys() {
        let long = format!("{}\nEND", BODY.repeat(12));
        let mut h = Harness::new(false, 2.0, egui::vec2(400.0, 300.0), 83.0, &long);
        h.narrow_width = Some(324.0);
        h.show_other = true;
        h.frame(vec![key(egui::Key::Tab)]);
        let owner = h.owner.as_ref().unwrap().id;
        h.frame(vec![key(egui::Key::Enter)]);
        h.settle();
        let popup = owner.with("pinned-context-help");
        assert!(egui::Popup::is_id_open(&h.ctx, popup));
        let other = h.other.as_ref().unwrap().id;
        keyboard_action(&mut h, other, egui::accesskit::Action::Focus);
        assert!(h.ctx.memory(|m| m.has_focus(other)));
        let before = h.settle();
        let top = keyboard_body(&before, &long).0.pos.y;
        h.frame(vec![key(egui::Key::PageDown)]);
        assert!(
            h.ctx.input(|i| i.key_pressed(egui::Key::PageDown)),
            "other info owner cannot consume original help page key"
        );
        let after = h.settle();
        assert!((keyboard_body(&after, &long).0.pos.y - top).abs() < 0.01);
        keyboard_action(&mut h, other, egui::accesskit::Action::Click);
        h.settle();
        assert!(!egui::Popup::is_id_open(&h.ctx, popup));
        assert!(egui::Popup::is_id_open(
            &h.ctx,
            other.with("pinned-context-help")
        ));
        h.frame(vec![key(egui::Key::PageDown)]);
        assert!(
            h.ctx.input(|i| i.key_pressed(egui::Key::PageDown)),
            "unrelated non-scroll popup keeps page key unconsumed"
        );
        assert_eq!(h.owner.as_ref().unwrap().id, owner);
        assert_eq!(h.other.as_ref().unwrap().id, other);
    }
}

#[cfg(test)]
mod help_dismiss_reentry_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, NodeId};
    const TITLE: &str = "도움말 복귀 진단";
    const BODY: &str = "설명은 Escape로 닫은 뒤 실제로 다시 진입할 때까지 닫혀 있어야 합니다.";
    struct Harness {
        ctx: egui::Context,
        viewport: egui::Rect,
        focused: bool,
        time: f64,
        body: String,
        value: String,
        owner: Option<Response>,
        input: Option<Response>,
    }
    impl Harness {
        fn new(small: bool, zoom: f32) -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.options_mut(|options| options.zoom_factor = zoom);
            ctx.enable_accesskit();
            let size = if small {
                egui::vec2(240.0, 120.0)
            } else {
                egui::vec2(640.0, 400.0)
            };
            let mut h = Self {
                ctx,
                viewport: egui::Rect::from_min_size(egui::Pos2::ZERO, size),
                focused: true,
                time: 0.0,
                body: if small {
                    format!("{}\n끝", BODY.repeat(10))
                } else {
                    BODY.into()
                },
                value: "C:\\synthetic\\owned.exe".into(),
                owner: None,
                input: None,
            };
            h.frame(vec![]);
            h
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += 0.1;
            self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(self.viewport),
                    focused: self.focused,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let owner = ui
                            .horizontal(|ui| {
                                ui.label("실행 파일");
                                help_button(ui, "dismiss-reentry-fixture", TITLE, &self.body)
                            })
                            .inner;
                        let (input, controls) = ui
                            .horizontal(|ui| {
                                let width = (ui.available_width()
                                    - 112.0
                                    - ui.spacing().item_spacing.x
                                    - TEXT_INPUT_HORIZONTAL_MARGIN)
                                    .max(0.0);
                                let input = text_input_with_id(
                                    ui,
                                    &mut self.value,
                                    32768,
                                    width,
                                    egui::Id::new("dismiss-reentry-input"),
                                );
                                let browse =
                                    secondary_sized(ui, "찾아보기", true, egui::vec2(112.0, 32.0));
                                let controls = input.rect.union(browse.rect);
                                (input, controls)
                            })
                            .inner;
                        self.input = Some(input);
                        self.owner = Some(show_help(ui, owner, TITLE, &self.body, Some(controls)));
                    });
                },
            )
        }
        fn owner_id(&self) -> egui::Id {
            self.owner.as_ref().unwrap().id
        }
        fn input_id(&self) -> egui::Id {
            self.input.as_ref().unwrap().id
        }
        fn action(&mut self, action: Action, id: egui::Id) -> egui::FullOutput {
            self.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target: NodeId(id.value()),
                data: None,
            })])
        }
        fn body_painted(&self, out: &egui::FullOutput) -> bool {
            out.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text)if text.galley.text()==self.body&&shape.clip_rect.intersects(text.visual_bounding_rect())))
        }
        fn trace(&self, label: &str, out: &egui::FullOutput) {
            let owner = self.owner.as_ref().unwrap();
            eprintln!(
                "help-reentry {label} window={} logical_owner={} response_focus={} gained={} hovered={} pointer={:?} delta={:?} pinned={} body={}",
                self.focused,
                self.ctx.memory(|memory| memory.has_focus(owner.id)),
                owner.has_focus(),
                owner.gained_focus(),
                owner.hovered(),
                self.ctx.input(|input| input.pointer.hover_pos()),
                self.ctx.input(|input| input.pointer.delta()),
                egui::Popup::is_id_open(&self.ctx, owner.id.with("pinned-context-help")),
                self.body_painted(out)
            );
        }
        fn pin(&mut self) {
            self.action(Action::Click, self.owner_id());
            for _ in 0..6 {
                self.frame(vec![]);
            }
            assert!(
                egui::Popup::is_id_open(&self.ctx, self.owner_id().with("pinned-context-help")),
                "setup: actual Click action opened pinned popup"
            );
        }
        fn escape(&mut self) -> egui::FullOutput {
            self.frame(vec![key(egui::Key::Escape)])
        }
        fn assert_closed(&self, out: &egui::FullOutput, label: &str) {
            self.trace(label, out);
            assert!(
                !egui::Popup::is_id_open(&self.ctx, self.owner_id().with("pinned-context-help")),
                "actual pinned popup closed"
            );
            assert!(
                self.ctx.memory(|memory| memory.has_focus(self.owner_id())),
                "actual logical owner focus retained"
            );
            assert!(
                !self.body_painted(out),
                "dismissed body reopened without an intentional new logical focus, hover entry or activation: {label}"
            );
        }
    }
    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    fn actual_dismiss_same_logical_owner_survives_window_focus_roundtrip() {
        for zoom in [1.0, 2.0] {
            let mut h = Harness::new(false, zoom);
            let id = h.owner_id();
            h.action(Action::Focus, id);
            h.pin();
            h.escape();
            let settled = h.frame(vec![]);
            h.assert_closed(&settled, "before-window-blur");
            h.focused = false;
            let blurred = h.frame(vec![]);
            h.trace("window-false", &blurred);
            assert!(
                h.ctx.memory(|memory| memory.has_focus(id)),
                "setup: window blur must preserve logical owner"
            );
            assert!(
                !h.owner.as_ref().unwrap().has_focus(),
                "setup: response focus includes window focus"
            );
            h.focused = true;
            for _ in 0..6 {
                let out = h.frame(vec![]);
                h.assert_closed(&out, "same-logical-owner-window-return");
            }
            assert_eq!(h.owner_id(), id);
        }
    }

    #[test]
    fn actual_dismiss_escape_return_from_other_input_does_not_rearm_on_gained_focus() {
        for zoom in [1.0, 2.0] {
            let mut h = Harness::new(false, zoom);
            let id = h.owner_id();
            let field = h.input_id();
            h.action(Action::Focus, field);
            assert!(
                h.ctx.memory(|memory| memory.has_focus(field)),
                "setup: real editable input owns focus"
            );
            h.pin();
            assert!(
                h.ctx.memory(|memory| memory.has_focus(field)),
                "setup: actual Click opens help without changing logical owner"
            );
            let escaped = h.escape();
            h.trace("escape-returns-owner", &escaped);
            assert!(
                h.ctx.memory(|memory| memory.has_focus(id)),
                "setup: production Escape returns actual owner focus"
            );
            for _ in 0..6 {
                let out = h.frame(vec![]);
                h.assert_closed(&out, "escape-return-is-not-new-focus-entry");
            }
            assert_eq!(h.value, "C:\\synthetic\\owned.exe");
        }
    }

    #[test]
    fn actual_dismiss_popup_removal_does_not_rearm_stationary_pointer_hover() {
        for zoom in [1.0, 2.0] {
            let mut h = Harness::new(true, zoom);
            let id = h.owner_id();
            let point = h.owner.as_ref().unwrap().rect.center();
            h.frame(vec![egui::Event::PointerMoved(point)]);
            h.frame(vec![egui::Event::PointerMoved(point)]);
            h.action(Action::Focus, id);
            h.pin();
            let pinned = h.frame(vec![]);
            h.trace("pinned-covering-owner", &pinned);
            let rect = egui::AreaState::load(&h.ctx, id.with("pinned-context-help"))
                .unwrap()
                .rect();
            assert!(
                rect.contains(point),
                "setup: actual constrained popup must cover stationary owner pointer: {rect:?} {point:?}"
            );
            assert!(
                !h.owner.as_ref().unwrap().hovered(),
                "setup: actual popup layer temporarily masks owner hover"
            );
            let escaped = h.escape();
            h.trace("stationary-pointer-escape", &escaped);
            let mut hover_returned = false;
            for _ in 0..12 {
                let out = h.frame(vec![]);
                assert_eq!(
                    h.ctx.input(|input| input.pointer.hover_pos()),
                    Some(point),
                    "setup: no pointer movement during popup removal"
                );
                assert_eq!(
                    h.ctx.input(|input| input.pointer.delta()),
                    egui::Vec2::ZERO,
                    "setup: zero motion after dismissal"
                );
                hover_returned |= h.owner.as_ref().unwrap().hovered();
                h.assert_closed(&out, "stationary-pointer-layer-removal");
            }
            assert!(
                hover_returned,
                "setup: same owner becomes hovered after covering popup disappears"
            );
        }
    }
}
