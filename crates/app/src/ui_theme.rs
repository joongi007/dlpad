//! DESIGN.md gtmux snapshot translated to egui0.33.3. Call once during Context setup.
//! Both style variants are configured without overriding System/theme preference.
use crate::ui_tokens as t;
use eframe::egui;
use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2};

pub const ACCENT: Color32 = Color32::from_rgb(t::ACCENT[0], t::ACCENT[1], t::ACCENT[2]);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(
    t::PRIMARY_HOVER[0],
    t::PRIMARY_HOVER[1],
    t::PRIMARY_HOVER[2],
);
pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(
    t::PRIMARY_PRESSED[0],
    t::PRIMARY_PRESSED[1],
    t::PRIMARY_PRESSED[2],
);
pub const ACCENT_SOLID: Color32 = Color32::from_rgb(t::PRIMARY[0], t::PRIMARY[1], t::PRIMARY[2]);
#[allow(dead_code)]
pub const SUCCESS_MARKER: Color32 = Color32::from_rgb(34, 197, 94);
#[allow(dead_code)]
pub const WARNING_MARKER: Color32 = Color32::from_rgb(250, 204, 21);
#[allow(dead_code)]
pub const DANGER_MARKER: Color32 = Color32::from_rgb(229, 72, 77);

pub fn configure_dlpad_theme(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(t::BODY_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(t::BODY_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(t::SMALL_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(t::SMALL_FONT_SIZE, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(t::HEADING_FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Name("screen-title".into()),
            FontId::new(t::TITLE_FONT_SIZE, FontFamily::Proportional),
        );
        style.spacing.item_spacing = Vec2::new(t::CONTROL_GAP, t::CONTROL_GAP);
        style.spacing.button_padding = Vec2::new(t::CONTROL_PADDING_X, 6.0);
        style.spacing.window_margin = Margin::same(16);
        style.spacing.menu_margin = Margin::same(8);
        style.spacing.indent = 16.0;
        style.spacing.interact_size = Vec2::new(t::CONTROL_HEIGHT, t::CONTROL_HEIGHT);
        style.animation_time = 0.0;
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        style.spacing.scroll.bar_width = 8.0;
        style.spacing.scroll.dormant_handle_opacity = 0.5;
        style.spacing.scroll.dormant_background_opacity = 0.05;

        let v = &mut style.visuals;
        let (background, surface, secondary, foreground, muted, border, strong) = if v.dark_mode {
            (
                Color32::from_gray(30),
                Color32::from_gray(44),
                Color32::from_gray(56),
                Color32::from_gray(245),
                Color32::from_gray(154),
                Color32::from_white_alpha(26),
                Color32::from_white_alpha(46),
            )
        } else {
            (
                Color32::WHITE,
                Color32::WHITE,
                Color32::from_gray(245),
                Color32::BLACK,
                Color32::from_gray(107),
                Color32::from_black_alpha(26),
                Color32::from_black_alpha(46),
            )
        };
        // Do not force override_text_color: it would erase per-component disabled/state colors.
        v.override_text_color = None;
        v.weak_text_color = Some(muted);
        v.panel_fill = background;
        v.window_fill = surface;
        v.faint_bg_color = secondary;
        v.extreme_bg_color = secondary;
        v.text_edit_bg_color = Some(surface);
        v.code_bg_color = secondary;
        v.window_stroke = Stroke::new(1.0, border);
        v.window_corner_radius = CornerRadius::same(t::RADIUS);
        v.menu_corner_radius = CornerRadius::same(t::RADIUS);
        v.window_shadow = egui::epaint::Shadow::NONE;
        // Existing popup shadow retained; decoration only belongs to menus/popovers.
        v.hyperlink_color = if v.dark_mode { ACCENT } else { ACCENT_SOLID };
        // Selection is a tinted persistent surface, unlike hover's neutral surface.
        v.selection.bg_fill = if v.dark_mode {
            Color32::from_rgba_unmultiplied(13, 153, 255, 52)
        } else {
            Color32::from_rgba_unmultiplied(13, 153, 255, 36)
        };
        v.selection.stroke = Stroke::new(1.0, foreground);
        v.text_cursor.stroke = Stroke::new(1.5, if v.dark_mode { ACCENT } else { ACCENT_SOLID });
        // Color is a marker, not the only status or small-text signal. Callers pair it with a label.
        v.warn_fg_color = foreground;
        v.error_fg_color = if v.dark_mode {
            Color32::from_rgb(255, 128, 130)
        } else {
            Color32::from_rgb(t::DANGER[0], t::DANGER[1], t::DANGER[2])
        };
        let radius = CornerRadius::same(t::RADIUS);
        for widget in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            widget.corner_radius = radius;
            widget.expansion = 0.0;
            widget.fg_stroke = Stroke::new(1.0, foreground);
        }
        v.widgets.noninteractive.bg_fill = surface;
        v.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        v.widgets.inactive.bg_fill = surface;
        v.widgets.inactive.weak_bg_fill = surface;
        // General controls use GTMUX's quiet boundary. Text inputs retain a distinct
        // accessible boundary in ui_components::text_input instead of boxing every action.
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, strong);
        v.widgets.hovered.bg_fill = Color32::from_gray(if v.dark_mode { 60 } else { 232 });
        v.widgets.hovered.weak_bg_fill = v.widgets.hovered.bg_fill;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, strong);
        // egui Widgets::style maps keyboard focus and pointer-down to active. Accent outline
        // makes focus visible; fill stays neutral and cannot masquerade as persistent selection.
        v.widgets.active.bg_fill = Color32::from_gray(if v.dark_mode { 72 } else { 218 });
        v.widgets.active.weak_bg_fill = v.widgets.active.bg_fill;
        v.widgets.active.bg_stroke =
            Stroke::new(1.5, if v.dark_mode { ACCENT } else { ACCENT_SOLID });
        v.widgets.open.bg_fill = secondary;
        v.widgets.open.weak_bg_fill = secondary;
        v.widgets.open.bg_stroke = Stroke::new(1.0, strong);
    });
}

#[cfg(test)]
mod contrast_tests {
    use super::*;
    fn luminance(c: Color32) -> f64 {
        let channel = |v: u8| {
            let s = v as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }
    #[test]
    fn small_white_primary_labels_meet_design_contrast_in_every_interactive_state() {
        for fill in [ACCENT_SOLID, ACCENT_HOVER, ACCENT_PRESSED] {
            assert!(1.05 / (luminance(fill) + 0.05) >= 4.5);
        }
    }
}
