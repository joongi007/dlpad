//! Bounded CPU form inputs; core owns validation, snapshots and argv policy.
use crate::catalog as c;
use dlpad_core::options::{
    AdvancedOption, CATALOG, DownloadOptions, OptionHelp, Quality, SUBTITLE_HELP, SourceUrl,
    SubtitleLanguage, SubtitleOptions, TimeRange, ValidationError,
};
use eframe::egui;
use std::path::Path;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum QualityMode {
    #[default]
    Best,
    Height,
    Audio,
}

// UI unit conversion is exact; core policy still validates canonical bytes.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum RateUnit {
    Bytes,
    Kilobytes,
    #[default]
    Megabytes,
    Gigabytes,
}
impl RateUnit {
    const ALL: [Self; 4] = [
        Self::Bytes,
        Self::Kilobytes,
        Self::Megabytes,
        Self::Gigabytes,
    ];
    fn decimals(self) -> u32 {
        match self {
            Self::Bytes => 0,
            Self::Kilobytes => 3,
            Self::Megabytes => 6,
            Self::Gigabytes => 9,
        }
    }
    fn scale(self) -> u64 {
        10u64.pow(self.decimals())
    }
    fn label(self) -> &'static str {
        match self {
            Self::Bytes => c::RATE_BPS,
            Self::Kilobytes => c::RATE_KBPS,
            Self::Megabytes => c::RATE_MBPS,
            Self::Gigabytes => c::RATE_GBPS,
        }
    }
    fn format(self, bytes: u64) -> String {
        let scale = self.scale();
        let whole = bytes / scale;
        let remainder = bytes % scale;
        if remainder == 0 {
            return whole.to_string();
        }
        let digits = format!("{:0width$}", remainder, width = self.decimals() as usize);
        format!("{whole}.{}", digits.trim_end_matches('0'))
    }
    fn parse(self, text: &str) -> Result<Option<u64>, ValidationError> {
        if text.is_empty() {
            return Ok(None);
        }
        let invalid = || ValidationError(c::RATE_NUMBER_ERROR);
        if text.len() > 32 {
            return Err(invalid());
        }
        let (whole, fraction) = match text.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (text, None),
        };
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let whole: u64 = whole.parse().map_err(|_| invalid())?;
        let fractional = if let Some(fraction) = fraction {
            if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            // Additional zeros are exact, not a rounded fractional byte.
            let fraction = fraction.trim_end_matches('0');
            if fraction.len() > self.decimals() as usize {
                return Err(invalid());
            }
            if fraction.is_empty() {
                0
            } else {
                fraction
                    .parse::<u64>()
                    .map_err(|_| invalid())?
                    .checked_mul(10u64.pow(self.decimals() - fraction.len() as u32))
                    .ok_or_else(invalid)?
            }
        } else {
            0
        };
        let bytes = whole
            .checked_mul(self.scale())
            .and_then(|v| v.checked_add(fractional))
            .ok_or_else(invalid)?;
        AdvancedOption {
            key: CATALOG[0].id.into(),
            value: bytes.to_string(),
        }
        .validate()
        .map_err(|_| invalid())?;
        Ok(Some(bytes))
    }
}

#[derive(Default)]
struct RateControl {
    text: String,
    unit: RateUnit,
    // canonical tracks only accepted changes; invalid edits never replace it.
    canonical: String,
}
impl RateControl {
    fn sync_external(&mut self, canonical: &str) {
        if canonical == self.canonical {
            return;
        }
        self.canonical = canonical.into();
        // Never silently normalize an externally invalid value into valid UI.
        if let Ok(bytes) = canonical.parse::<u64>() {
            self.text = self.unit.format(bytes);
        } else {
            self.text = canonical.into();
            self.unit = RateUnit::Bytes;
        }
    }
    fn error(&self) -> Option<ValidationError> {
        self.unit.parse(&self.text).err()
    }
    fn accept(&mut self, canonical: &mut String) {
        if let Ok(value) = self.unit.parse(&self.text) {
            *canonical = value.map_or_else(String::new, |bytes| bytes.to_string());
            self.canonical.clone_from(canonical);
        }
    }
    fn reset(&mut self, canonical: &mut String) {
        self.text.clear();
        canonical.clear();
        self.canonical.clear();
        self.unit = RateUnit::default();
    }
    fn select_unit(&mut self, unit: RateUnit) {
        if let Ok(bytes) = self.unit.parse(&self.text) {
            self.text = bytes.map_or_else(String::new, |bytes| unit.format(bytes));
            self.unit = unit;
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, canonical: &mut String, shared_version_visible: bool) {
        self.sync_external(canonical);
        let label = ui.label(egui::RichText::new(CATALOG[0].label).small().weak());
        ui.horizontal_wrapped(|ui| {
            for (value, label) in [
                (None, c::RATE_UNLIMITED),
                (Some(1_000_000), c::RATE_PRESET_ONE),
                (Some(5_000_000), c::RATE_PRESET_FIVE),
                (Some(10_000_000), c::RATE_PRESET_TEN),
            ] {
                let selected = self
                    .unit
                    .parse(&self.text)
                    .is_ok_and(|bytes| bytes == value);
                let preset = crate::ui_components::choice(ui, selected, label);
                if preset.gained_focus() {
                    preset.scroll_to_me(None);
                }
                if preset.clicked() {
                    self.unit = RateUnit::Megabytes;
                    self.text = value.map_or_else(String::new, |bytes| self.unit.format(bytes));
                    self.accept(canonical);
                }
            }
        });
        let available = ui.available_width();
        let stacked = available < 240.0;
        let input_width = if stacked {
            available
        } else {
            available - 108.0
        };
        let mut input = None;
        let mut controls = |ui: &mut egui::Ui| {
            let response = crate::ui_components::text_input_with_id_and_hint(
                ui,
                &mut self.text,
                32,
                (input_width.min(240.0) - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                    .max(1.0),
                egui::Id::new(("download-option-input", CATALOG[0].label)),
                c::RATE_UNLIMITED,
            )
            .labelled_by(label.id);
            if response.changed() {
                self.accept(canonical);
            }
            input = Some(response);
            let current_unit = self.unit;
            ui.add_enabled_ui(self.error().is_none(), |ui| {
                let unit_combo = egui::ComboBox::from_id_salt("download-rate-unit")
                    .selected_text(current_unit.label())
                    .width(88.0)
                    .show_ui(ui, |ui| {
                        for unit in RateUnit::ALL {
                            if crate::ui_components::choice(ui, self.unit == unit, unit.label())
                                .clicked()
                            {
                                self.select_unit(unit);
                                ui.close();
                            }
                        }
                    });
                let unit = crate::ui_components::combo_response(ui, unit_combo.response);
                ui.ctx().accesskit_node_builder(unit.id, |node| {
                    node.set_label(c::RATE_UNIT_LABEL);
                    node.set_description(c::RATE_UNIT_HELP);
                });
                if unit.gained_focus() {
                    unit.scroll_to_me(None);
                }
            });
        };
        if stacked {
            ui.vertical(&mut controls);
        } else {
            ui.horizontal(&mut controls);
        }
        let response = input.unwrap();
        let error = self.error();
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_label(format!("{} ({})", CATALOG[0].label, self.unit.label()));
            node.clear_labelled_by();
            if let Some(error) = &error {
                node.set_invalid(egui::accesskit::Invalid::True);
                node.set_description(error.0);
            } else {
                node.clear_invalid();
                node.set_description(c::RATE_UNIT_HELP);
            }
        });
        if response.gained_focus() {
            response.scroll_to_me(None);
        }
        if let Some(error) = error {
            if response.enabled() {
                ui.painter().rect_stroke(
                    response.rect,
                    ui.visuals().widgets.inactive.corner_radius,
                    egui::Stroke::new(1.5, crate::ui_theme::DANGER_MARKER),
                    egui::StrokeKind::Inside,
                );
            }
            ui.colored_label(ui.visuals().error_fg_color, error.0);
        }
        ui.label(egui::RichText::new(c::RATE_UNIT_HELP).small());
        advanced_help(ui, &CATALOG[0], shared_version_visible);
    }
}

pub(crate) struct DownloadForm {
    quality: QualityMode,
    media_prepared: bool,
    preserve_title: bool,
    height: String,
    start: String,
    end: String,
    accurate: bool,
    subtitles: Option<SubtitleOptions>,
    advanced_values: [String; 3],
    rate: RateControl,
    advanced_open: bool,
    height_reveal: Option<(u64, egui::Rect)>,
}
impl Default for DownloadForm {
    fn default() -> Self {
        Self {
            quality: QualityMode::Best,
            media_prepared: false,
            preserve_title: true,
            height: "1080".into(),
            start: String::new(),
            end: String::new(),
            accurate: false,
            subtitles: None,
            advanced_values: std::array::from_fn(|_| String::new()),
            rate: RateControl::default(),
            advanced_open: false,
            height_reveal: None,
        }
    }
}
impl DownloadForm {
    pub(crate) fn height_reveal_target(&self, ctx: &egui::Context) -> Option<egui::Rect> {
        self.height_reveal
            .filter(|(frame, _)| *frame == ctx.cumulative_frame_nr())
            .map(|(_, rect)| rect)
    }
    pub(crate) fn set_media_prepared(&mut self, prepared: bool) {
        self.media_prepared = prepared;
    }
    pub(crate) fn apply_media_selection(
        &mut self,
        selection: dlpad_core::media_selection::MediaSelection,
    ) {
        if selection.start_ms() == 0 && selection.end_ms() == selection.duration_ms() {
            self.start.clear();
            self.end.clear();
        } else {
            self.start = crate::media_timeline::time(selection.start_ms());
            self.end = crate::media_timeline::time(selection.end_ms());
        }
    }
    pub(crate) fn media_range(&self, duration: u64) -> Result<(u64, u64), ValidationError> {
        let (start, end) = match self.range()? {
            None => (0, duration),
            Some(range) => (range.start_ms, range.end_ms),
        };
        if start >= end || end > duration {
            return Err(ValidationError(c::MEDIA_RANGE_INVALID));
        }
        Ok((start, end))
    }
    pub(crate) fn show_media_range(&mut self, ui: &mut egui::Ui, duration: u64) {
        let (start, end) = ui.columns(2, |columns| {
            let start = field_response(&mut columns[0], c::RANGE_START, &mut self.start, 32).0;
            let end = field_response(&mut columns[1], c::RANGE_END, &mut self.end, 32).0;
            (start, end)
        });
        self.annotate_range_fields(ui, &start, &end, Some(duration));
        if let Err(error) = self.media_range(duration) {
            ui.colored_label(ui.visuals().error_fg_color, error.0);
        }
    }
    fn quality(&self) -> Result<Quality, ValidationError> {
        let quality = match self.quality {
            QualityMode::Best => Quality::Best,
            QualityMode::Audio => Quality::Audio,
            QualityMode::Height => Quality::Height(
                self.height
                    .parse()
                    .map_err(|_| ValidationError(c::QUALITY_HEIGHT_NUMBER))?,
            ),
        };
        quality.validate()?;
        Ok(quality)
    }
    fn range_field_errors(
        &self,
        duration: Option<u64>,
    ) -> (Option<ValidationError>, Option<ValidationError>) {
        // Both blank remains the whole-video default, even if media duration is
        // unavailable/zero. A media/group error must not mark valid blank fields.
        if self.start.is_empty() && self.end.is_empty() {
            return (None, None);
        }
        let start = TimeRange::parse_time(&self.start);
        let end = TimeRange::parse_time(&self.end);
        let mut start_error = start.as_ref().err().cloned();
        let mut end_error = end.as_ref().err().cloned();
        if let (Ok(start), Ok(end)) = (&start, &end) {
            // The core order error specifically requires end after chosen start.
            end_error = TimeRange::new(*start, *end, self.accurate).err();
        }
        if let Some(duration) = duration {
            if start.as_ref().is_ok_and(|start| *start >= duration) {
                start_error.get_or_insert(ValidationError(c::MEDIA_RANGE_INVALID));
            }
            if end.as_ref().is_ok_and(|end| *end > duration) {
                end_error.get_or_insert(ValidationError(c::MEDIA_RANGE_INVALID));
            }
        }
        (start_error, end_error)
    }
    fn annotate_range_fields(
        &self,
        ui: &egui::Ui,
        start: &egui::Response,
        end: &egui::Response,
        duration: Option<u64>,
    ) {
        let (start_error, end_error) = self.range_field_errors(duration);
        for (response, error) in [(start, start_error), (end, end_error)] {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                if let Some(error) = &error {
                    node.set_invalid(egui::accesskit::Invalid::True);
                    node.set_description(error.0);
                } else {
                    node.clear_invalid();
                    node.clear_description();
                }
            });
            if error.is_some() && response.enabled() {
                ui.painter().rect_stroke(
                    response.rect,
                    ui.visuals().widgets.inactive.corner_radius,
                    egui::Stroke::new(1.5, crate::ui_theme::DANGER_MARKER),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    /// Presentation only: required-tool guidance must not parse or validate a draft.
    pub(crate) fn range_requested(&self) -> bool {
        !self.start.is_empty() || !self.end.is_empty()
    }
    fn range(&self) -> Result<Option<TimeRange>, ValidationError> {
        if self.start.is_empty() && self.end.is_empty() {
            Ok(None)
        } else {
            TimeRange::new(
                TimeRange::parse_time(&self.start)?,
                TimeRange::parse_time(&self.end)?,
                self.accurate,
            )
            .map(Some)
        }
    }
    fn choose_subtitles(&mut self, language: Option<SubtitleLanguage>) {
        self.subtitles = language.map(|language| SubtitleOptions {
            language,
            auto_generated: self.subtitles.is_some_and(|s| s.auto_generated),
        });
    }
    pub(crate) fn build(
        &self,
        url: &str,
        output: &Path,
        ffmpeg: Option<&Path>,
    ) -> Result<DownloadOptions, ValidationError> {
        if let Some(error) = self.rate.error() {
            return Err(error);
        }
        let options = DownloadOptions {
            url: SourceUrl::parse(url)?,
            output_dir: output.into(),
            quality: self.quality()?,
            time_range: self.range()?,
            ffmpeg: ffmpeg.map(Into::into),
            deno: None,
            final_name_style: self
                .preserve_title
                .then_some(dlpad_core::options::FinalNameStyle::TitleWithJobId),
            subtitles: self.subtitles,
            advanced: self
                .advanced_values
                .iter()
                .zip(CATALOG)
                .filter(|(value, _)| !value.is_empty())
                .map(|(value, help)| AdvancedOption {
                    key: help.id.into(),
                    value: value.clone(),
                })
                .collect(),
        };
        options.plan()?;
        Ok(options)
    }
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.add_enabled_ui(enabled, |ui| {
            let quality_label = ui.label(egui::RichText::new(c::QUALITY_MODE).small().weak());
            let quality = egui::ComboBox::from_id_salt("download-quality")
                .selected_text(match self.quality {
                    QualityMode::Best => c::QUALITY_BEST,
                    QualityMode::Height => c::QUALITY_HEIGHT,
                    QualityMode::Audio => c::QUALITY_AUDIO,
                })
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for (quality, label) in [
                        (QualityMode::Best, c::QUALITY_BEST),
                        (QualityMode::Height, c::QUALITY_HEIGHT),
                        (QualityMode::Audio, c::QUALITY_AUDIO),
                    ] {
                        if crate::ui_components::select(ui, &mut self.quality, quality, label)
                            .clicked()
                        {
                            ui.close();
                        }
                    }
                })
                .response
                .labelled_by(quality_label.id);
            let quality = crate::ui_components::combo_response(ui, quality);
            if quality.gained_focus() {
                ui.scroll_to_rect(quality_label.rect.union(quality.rect), None);
            }
            crate::accessible::collapsing(ui, c::QUALITY_HELP_TITLE, |ui| {
                ui.label(c::QUALITY_SUMMARY);
                ui.label(c::QUALITY_HELP);
                ui.label(c::QUALITY_DEFAULT_EXAMPLE);
                ui.label(c::QUALITY_CONFLICT_VERSION);
            });
            let naming = crate::ui_components::checkbox(
                ui,
                &mut self.preserve_title,
                c::PRESERVE_VIDEO_TITLE,
            );
            naming.on_hover_text(c::PRESERVE_VIDEO_TITLE_HELP);
            ui.weak(c::PRESERVE_VIDEO_TITLE_EXAMPLE);
            if self.quality == QualityMode::Height {
                let (height, label_rect) =
                    field_response(ui, c::QUALITY_HEIGHT, &mut self.height, 5);
                let mut control_error = height.rect;
                let error = self.quality().err();
                ui.ctx().accesskit_node_builder(height.id, |node| {
                    if let Some(error) = &error {
                        node.set_invalid(egui::accesskit::Invalid::True);
                        node.set_description(error.0);
                    } else {
                        node.clear_invalid();
                        node.clear_description();
                    }
                });
                if let Some(error) = error {
                    if height.enabled() {
                        ui.painter().rect_stroke(
                            height.rect,
                            ui.visuals().widgets.inactive.corner_radius,
                            egui::Stroke::new(1.5, crate::ui_theme::DANGER_MARKER),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let explanation = ui.colored_label(ui.visuals().error_fg_color, error.0);
                    control_error = control_error.union(explanation.rect);
                }
                if height.enabled()
                    && (height.gained_focus() || (height.has_focus() && height.changed()))
                {
                    self.height_reveal = Some((ui.ctx().cumulative_frame_nr(), control_error));
                }
                if self.height_reveal_target(ui.ctx()).is_some() {
                    let full = label_rect.union(control_error);
                    let target = if full.height() <= ui.clip_rect().height() {
                        full
                    } else {
                        control_error
                    };
                    self.height_reveal = Some((ui.ctx().cumulative_frame_nr(), target));
                }
            }
            crate::accessible::collapsing(ui, c::OPTIONAL_MEDIA, |ui| {
                if !self.media_prepared {
                    let start = field_response(ui, c::RANGE_START, &mut self.start, 32).0;
                    let end = field_response(ui, c::RANGE_END, &mut self.end, 32).0;
                    self.annotate_range_fields(ui, &start, &end, None);
                }
                crate::accessible::collapsing(ui, c::RANGE_HELP_TITLE, |ui| {
                    ui.label(c::RANGE_HELP);
                    ui.label(c::RANGE_DEFAULT_EXAMPLE);
                    ui.label(c::RANGE_CONFLICT_VERSION);
                });
                ui.label(c::CUT_MODE);
                ui.horizontal_wrapped(|ui| {
                    crate::ui_components::select(ui, &mut self.accurate, false, c::FAST_CUT);
                    crate::ui_components::select(ui, &mut self.accurate, true, c::ACCURATE_CUT);
                });
                ui.label(c::CUT_COST);
                crate::accessible::collapsing(ui, c::CUT_HELP_TITLE, |ui| {
                    ui.label(c::CUT_DEFAULT_EXAMPLE);
                    ui.label(c::CUT_CONFLICT_VERSION);
                });
                if let Err(error) = self.range() {
                    ui.colored_label(ui.visuals().error_fg_color, error.0);
                }
                ui.label(c::SUBTITLE_LANGUAGE);
                ui.horizontal_wrapped(|ui| {
                    if crate::ui_components::choice(ui, self.subtitles.is_none(), c::SUBTITLE_NONE)
                        .clicked()
                    {
                        self.choose_subtitles(None);
                    }
                    for language in SubtitleLanguage::ALL {
                        if crate::ui_components::choice(
                            ui,
                            self.subtitles.is_some_and(|s| s.language == language),
                            language.label(),
                        )
                        .clicked()
                        {
                            self.choose_subtitles(Some(language));
                        }
                    }
                });
                let mut automatic = self.subtitles.is_some_and(|s| s.auto_generated);
                if ui
                    .add_enabled_ui(self.subtitles.is_some(), |ui| {
                        crate::ui_components::checkbox(ui, &mut automatic, c::SUBTITLE_AUTO)
                    })
                    .inner
                    .changed()
                    && let Some(subtitles) = &mut self.subtitles
                {
                    subtitles.auto_generated = automatic;
                }
                help(ui, &SUBTITLE_HELP);
            });
            let advanced = crate::ui_components::ghost(
                ui,
                if self.advanced_open {
                    c::ADVANCED_CLOSE
                } else {
                    c::ADVANCED_OPEN
                },
                true,
            );
            if advanced.gained_focus() {
                advanced.scroll_to_me(None);
            }
            if advanced.clicked() {
                self.advanced_open = !self.advanced_open;
            }
            if self.advanced_open {
                ui.weak(c::ADVANCED_HELP);
                let reset = crate::ui_components::ghost(ui, c::ADVANCED_RESET, true);
                if reset.gained_focus() {
                    reset.scroll_to_me(None);
                }
                if reset.clicked() {
                    self.advanced_values.iter_mut().for_each(String::clear);
                    self.rate.reset(&mut self.advanced_values[0]);
                }
                let shared_version = CATALOG
                    .first()
                    .map(|entry| entry.version)
                    .filter(|version| CATALOG.iter().all(|entry| entry.version == *version));
                if let Some(version) = shared_version {
                    ui.label(
                        egui::RichText::new(format!("{}: {}", c::HELP_VERSION, version)).small(),
                    );
                }
                self.rate
                    .show(ui, &mut self.advanced_values[0], shared_version.is_some());
                for (value, description) in self.advanced_values.iter_mut().zip(CATALOG).skip(1) {
                    let (response, _) =
                        field_response_with_width(ui, description.label, value, 20, 360.0);
                    let error = if value.is_empty() {
                        None
                    } else {
                        AdvancedOption {
                            key: description.id.into(),
                            value: value.clone(),
                        }
                        .validate()
                        .err()
                    };
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        if let Some(error) = &error {
                            node.set_invalid(egui::accesskit::Invalid::True);
                            node.set_description(error.0);
                        } else {
                            node.clear_invalid();
                            node.clear_description();
                        }
                    });
                    if error.is_some() && response.enabled() {
                        ui.painter().rect_stroke(
                            response.rect,
                            ui.visuals().widgets.inactive.corner_radius,
                            egui::Stroke::new(1.5, crate::ui_theme::DANGER_MARKER),
                            egui::StrokeKind::Inside,
                        );
                    }
                    // Validation stays adjacent to its field, outside optional explanations.
                    if let Some(error) = error {
                        ui.colored_label(ui.visuals().error_fg_color, error.0);
                    }
                    advanced_help(ui, description, shared_version.is_some());
                }
            }
        });
    }
}
fn field_response(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut String,
    limit: usize,
) -> (egui::Response, egui::Rect) {
    field_response_with_width(ui, name, value, limit, f32::INFINITY)
}

fn field_response_with_width(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut String,
    limit: usize,
    max_width: f32,
) -> (egui::Response, egui::Rect) {
    let visible = match name {
        c::RANGE_START => c::RANGE_START_LABEL,
        c::RANGE_END => c::RANGE_END_LABEL,
        _ => name,
    };
    let label = ui
        .label(egui::RichText::new(visible).small().weak())
        .on_hover_text(name);
    let hint = CATALOG
        .iter()
        .find(|option| option.label == name)
        .map_or("", |option| option.default);
    let response = crate::ui_components::text_input_with_id_and_hint(
        ui,
        value,
        limit,
        (max_width - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN).min(ui.available_width()),
        egui::Id::new(("download-option-input", name)),
        hint,
    )
    .labelled_by(label.id);
    if visible != name {
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.clear_labelled_by();
            node.set_label(name);
        });
    }
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    (response, label.rect)
}
fn advanced_help(ui: &mut egui::Ui, description: &OptionHelp, shared_version_visible: bool) {
    ui.label(egui::RichText::new(description.summary).small());
    ui.label(egui::RichText::new(format!("{}: {}", c::HELP_DEFAULT, description.default)).small());
    ui.push_id(("advanced-option-help", description.id), |ui| {
        crate::accessible::collapsing(ui, c::ADVANCED_OPTION_DETAILS, |ui| {
            ui.label(format!("{}: {}", c::HELP_EXAMPLE, description.example));
            if description.id == CATALOG[0].id {
                ui.label(c::RATE_CLI_EXAMPLE_HELP);
            }
            ui.label(description.tradeoff);
            ui.label(format!("{}: {}", c::HELP_CONFLICT, description.conflicts));
            if !shared_version_visible {
                ui.label(format!("{}: {}", c::HELP_VERSION, description.version));
            }
        });
    });
}

fn help(ui: &mut egui::Ui, description: &OptionHelp) {
    ui.label(description.summary);
    ui.label(format!(
        "{}: {} · {}: {}",
        c::HELP_DEFAULT,
        description.default,
        c::HELP_EXAMPLE,
        description.example
    ));
    ui.label(description.tradeoff);
    ui.label(format!(
        "{}: {} · {}: {}",
        c::HELP_CONFLICT,
        description.conflicts,
        c::HELP_VERSION,
        description.version
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accessible_quality_selection_commits_and_closes_actual_popup() {
        use egui::{
            Event, RawInput,
            accesskit::{Action, ActionRequest, Role},
        };
        for (target_label, expected) in [
            (c::QUALITY_AUDIO, Quality::Audio),
            (c::QUALITY_BEST, Quality::Best),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut form = DownloadForm::default();
            fn frame(
                ctx: &egui::Context,
                form: &mut DownloadForm,
                events: Vec<Event>,
            ) -> egui::FullOutput {
                ctx.run(
                    RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(960.0, 640.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| form.show(ui, true));
                    },
                )
            }
            fn action(target: egui::accesskit::NodeId) -> Event {
                Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Click,
                    target,
                    data: None,
                })
            }
            let first = frame(&ctx, &mut form, vec![]);
            let tree = first
                .platform_output
                .accesskit_update
                .expect("actual form tree");
            // Quality is linked to its visible label; identify the current Best choice by value.
            let combos: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::ComboBox && node.value() == Some(c::QUALITY_BEST)
                })
                .collect();
            assert_eq!(combos.len(), 1, "actual unique Best ComboBox");
            let combo_id = combos[0].0;
            assert!(!egui::Popup::is_any_open(&ctx));
            frame(&ctx, &mut form, vec![action(combo_id)]);
            let opened = frame(&ctx, &mut form, vec![]);
            assert!(
                egui::Popup::is_any_open(&ctx),
                "actual menu opened before selection"
            );
            let tree = opened
                .platform_output
                .accesskit_update
                .expect("actual open menu tree");
            let choices: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.label() == Some(target_label) && node.supports_action(Action::Click)
                })
                .collect();
            assert_eq!(choices.len(), 1, "unique actionable audio choice");
            let choice_id = choices[0].0;
            frame(&ctx, &mut form, vec![action(choice_id)]);
            let closed = frame(&ctx, &mut form, vec![]);
            assert_eq!(form.quality().unwrap(), expected);
            assert!(
                !egui::Popup::is_any_open(&ctx),
                "non-pointer commit closes popup"
            );
            assert_eq!(build(&form).unwrap().quality, expected);
            let tree = closed
                .platform_output
                .accesskit_update
                .expect("actual closed form tree");
            assert!(tree.nodes.iter().any(|(_, node)| {
                node.role() == Role::ComboBox && node.value() == Some(target_label)
            }));
        }
    }

    fn build(form: &DownloadForm) -> Result<DownloadOptions, ValidationError> {
        form.build("https://fixture.invalid/watch", Path::new("fixture"), None)
    }
    fn argv(form: &DownloadForm) -> Vec<String> {
        build(form)
            .unwrap()
            .plan()
            .unwrap()
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }
    #[test]
    fn visual_range_roundtrips_and_invalid_manual_end_preserves_core_selection() {
        let mut form = DownloadForm::default();
        let mut selection = dlpad_core::media_selection::MediaSelection::new(5000).unwrap();
        selection.set_range(750, 4250).unwrap();
        form.apply_media_selection(selection);
        assert_eq!(form.media_range(5000).unwrap(), (750, 4250));
        form.end = "00:00:06.000".into();
        assert!(form.media_range(5000).is_err());
        assert_eq!((selection.start_ms(), selection.end_ms()), (750, 4250));
        selection.select_all();
        form.apply_media_selection(selection);
        assert!(form.range().unwrap().is_none());
        assert_eq!(form.media_range(5000).unwrap(), (0, 5000));
    }
    #[test]
    fn defaults_and_audio_preserve_typed_snapshot_and_finite_plan() {
        let mut form = DownloadForm::default();
        let options = build(&form).unwrap();
        assert_eq!(options.quality, Quality::Best);
        assert!(options.time_range.is_none() && options.subtitles.is_none());
        assert!(options.advanced.is_empty());
        assert!(!argv(&form).contains(&"--download-sections".into()));
        form.quality = QualityMode::Audio;
        let restored =
            DownloadOptions::from_stored_json(&build(&form).unwrap().to_stored_json().unwrap())
                .unwrap();
        assert_eq!(restored.quality, Quality::Audio);
        assert!(argv(&form).contains(&"bestaudio/best".into()));
    }
    #[test]
    fn selected_height_uses_core_bounds_and_never_changes_best() {
        let mut form = DownloadForm {
            height: "invalid".into(),
            ..DownloadForm::default()
        };
        assert!(build(&form).is_ok());
        form.quality = QualityMode::Height;
        for invalid in ["0", "143", "4321", "65536", "-1", " 1080", "1.0"] {
            form.height = invalid.into();
            assert!(build(&form).is_err(), "{invalid}");
        }
        for valid in ["144", "1080", "4320"] {
            form.height = valid.into();
            assert_eq!(
                build(&form).unwrap().quality,
                Quality::Height(valid.parse().unwrap())
            );
        }
    }
    #[test]
    fn eight_hour_range_modes_and_invalid_inputs_cross_form_boundary() {
        let mut form = DownloadForm {
            start: "0.001".into(),
            end: "08:30:00.123".into(),
            ..DownloadForm::default()
        };
        assert_eq!(build(&form).unwrap().time_range.unwrap().end_ms, 30_600_123);
        assert!(!argv(&form).contains(&"--force-keyframes-at-cuts".into()));
        form.accurate = true;
        assert!(argv(&form).contains(&"--force-keyframes-at-cuts".into()));
        for invalid in ["", "0", "00:60:00", "1.0000", "18446744073709551615"] {
            form.end = invalid.into();
            assert!(build(&form).is_err(), "{invalid}");
        }
    }
    #[test]
    fn subtitles_reselection_resets_auto_and_conflicts_fail_before_plan() {
        let mut form = DownloadForm::default();
        for language in SubtitleLanguage::ALL {
            form.choose_subtitles(Some(language));
            form.subtitles.as_mut().unwrap().auto_generated = true;
            assert!(argv(&form).contains(&language.codes().into()));
            assert!(argv(&form).contains(&"--write-auto-subs".into()));
        }
        form.choose_subtitles(None);
        form.choose_subtitles(Some(SubtitleLanguage::Korean));
        assert!(!form.subtitles.unwrap().auto_generated);
        form.start = "0".into();
        form.end = "1".into();
        assert!(build(&form).is_err());
    }
    #[test]
    fn advanced_finite_fields_validate_and_replace_default_retries_once() {
        let form = DownloadForm {
            advanced_values: ["1048576".into(), "0".into(), "16".into()],
            ..DownloadForm::default()
        };
        let args = argv(&form);
        assert_eq!(args.iter().filter(|s| *s == "--retries").count(), 1);
        let index = args.iter().position(|s| s == "--retries").unwrap();
        assert_eq!(args[index + 1], "0");
        for (index, value) in [
            (0, "0"),
            (0, "1000000000001"),
            (1, "21"),
            (2, "17"),
            (2, "-1"),
            (1, " 3"),
            (1, "３"),
        ] {
            let mut invalid = DownloadForm::default();
            invalid.advanced_values[index] = value.into();
            assert!(build(&invalid).is_err(), "{index}:{value}");
        }
    }
    #[test]
    fn external_inputs_keep_core_policy_and_literal_ffmpeg_path() {
        let form = DownloadForm::default();
        assert!(
            form.build(
                "https://user:pass@fixture.invalid/",
                Path::new("fixture"),
                None
            )
            .is_err()
        );
        assert!(
            form.build("https://fixture.invalid/", Path::new(""), None)
                .is_err()
        );
        assert!(
            form.build(
                "https://fixture.invalid/",
                Path::new("fixture"),
                Some(Path::new("relative"))
            )
            .is_err()
        );
        let path = std::env::temp_dir().join("한글 ffmpeg");
        let options = form
            .build(
                "https://fixture.invalid/",
                Path::new("fixture"),
                Some(&path),
            )
            .unwrap();
        let args = options.plan().unwrap().args;
        let index = args.iter().position(|s| s == "--ffmpeg-location").unwrap();
        assert_eq!(args[index + 1], path.as_os_str());
    }

    #[test]
    fn actual_subtitle_choices_checkbox_and_disabled_form_preserve_semantics() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled};
        fn frame(
            ctx: &egui::Context,
            form: &mut DownloadForm,
            tree: &mut std::collections::HashMap<NodeId, Node>,
            events: Vec<egui::Event>,
            enabled: bool,
        ) {
            let time = ctx.input(|i| i.time) + 1.0;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 1200.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| form.show(ui, enabled));
                },
            );
            if let Some(update) = output.platform_output.accesskit_update {
                for (id, node) in update.nodes {
                    tree.insert(id, node);
                }
            }
        }
        fn node(
            tree: &std::collections::HashMap<NodeId, Node>,
            label: &str,
            role: Role,
        ) -> (NodeId, Node) {
            let matches: Vec<_> = tree
                .iter()
                .filter(|(_, n)| n.label() == Some(label) && n.role() == role)
                .collect();
            assert_eq!(matches.len(), 1, "unique actual {label}");
            (*matches[0].0, matches[0].1.clone())
        }
        fn click(target: NodeId) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target,
                data: None,
            })
        }
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_theme(theme);
            ctx.enable_accesskit();
            let mut form = DownloadForm::default();
            let mut tree = std::collections::HashMap::new();
            frame(&ctx, &mut form, &mut tree, vec![], true);
            let (disclosure, _) = node(&tree, c::OPTIONAL_MEDIA, Role::Button);
            frame(&ctx, &mut form, &mut tree, vec![click(disclosure)], true);
            frame(&ctx, &mut form, &mut tree, vec![], true);
            let labels: Vec<_> = std::iter::once(c::SUBTITLE_NONE)
                .chain(SubtitleLanguage::ALL.map(SubtitleLanguage::label))
                .collect();
            let ids: Vec<_> = labels
                .iter()
                .map(|label| node(&tree, label, Role::Button).0)
                .collect();
            let (checkbox, initial) = node(&tree, c::SUBTITLE_AUTO, Role::CheckBox);
            assert!(initial.is_disabled());
            assert_eq!(initial.toggled(), Some(Toggled::False));
            frame(&ctx, &mut form, &mut tree, vec![click(checkbox)], true);
            assert_eq!(
                form.subtitles, None,
                "disabled NONE cannot create subtitle intent"
            );
            for (index, language) in SubtitleLanguage::ALL.into_iter().enumerate() {
                frame(
                    &ctx,
                    &mut form,
                    &mut tree,
                    vec![click(ids[index + 1])],
                    true,
                );
                frame(&ctx, &mut form, &mut tree, vec![], true);
                assert_eq!(form.subtitles.unwrap().language, language);
                assert_eq!(form.subtitles.unwrap().auto_generated, index > 0);
                for (choice, label) in labels.iter().enumerate() {
                    let (id, n) = node(&tree, label, Role::Button);
                    assert_eq!(id, ids[choice], "selection retains actual ID");
                    assert_eq!(
                        n.toggled(),
                        Some(if choice == index + 1 {
                            Toggled::True
                        } else {
                            Toggled::False
                        })
                    );
                    let bounds = n.bounds().expect("actual choice bounds");
                    assert!(
                        (bounds.y1 - bounds.y0 - 32.0).abs() < 0.5,
                        "actual32px choice {label}"
                    );
                }
                let (id, n) = node(&tree, c::SUBTITLE_AUTO, Role::CheckBox);
                assert_eq!(id, checkbox);
                assert!(!n.is_disabled());
                let checkbox_bounds = n.bounds().expect("actual native checkbox bounds");
                assert!((checkbox_bounds.y1 - checkbox_bounds.y0 - 32.0).abs() < 0.5);
                assert_eq!(build(&form).unwrap().subtitles, form.subtitles);
                if index == 0 {
                    frame(&ctx, &mut form, &mut tree, vec![click(checkbox)], true);
                    assert!(form.subtitles.unwrap().auto_generated);
                    assert_eq!(
                        node(&tree, c::SUBTITLE_AUTO, Role::CheckBox).1.toggled(),
                        Some(Toggled::True)
                    );
                }
            }
            let saved = form.subtitles;
            frame(&ctx, &mut form, &mut tree, vec![], false);
            assert!(
                node(&tree, c::SUBTITLE_AUTO, Role::CheckBox)
                    .1
                    .is_disabled()
            );
            for id in &ids {
                frame(&ctx, &mut form, &mut tree, vec![click(*id)], false);
            }
            frame(&ctx, &mut form, &mut tree, vec![click(checkbox)], false);
            assert_eq!(
                form.subtitles, saved,
                "entire disabled form cannot change language/automatic"
            );
            frame(&ctx, &mut form, &mut tree, vec![click(ids[0])], true);
            frame(&ctx, &mut form, &mut tree, vec![], true);
            assert_eq!(form.subtitles, None);
            assert_eq!(
                node(&tree, c::SUBTITLE_NONE, Role::Button).1.toggled(),
                Some(Toggled::True)
            );
            assert!(
                node(&tree, c::SUBTITLE_AUTO, Role::CheckBox)
                    .1
                    .is_disabled()
            );
        }
    }
    // Append inside accessible_options.rs tests. Real TextEdit input, not parser-only mirror.
    #[test]
    fn actual_advanced_invalid_valid_empty_preserve_identity_and_plan_gate() {
        use egui::{
            Event, Key, Modifiers,
            accesskit::{Action, ActionRequest, Invalid, NodeId},
        };
        // Rate has a unit-aware actual Form regression below; retain both count fields here.
        for (index, bad, good) in [(1, "21", "0"), (2, "0", "1")] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.enable_accesskit();
            let mut form = DownloadForm {
                advanced_open: true,
                ..Default::default()
            };
            let id = egui::Id::new(("download-option-input", CATALOG[index].label));
            let node_id = NodeId(id.value());
            let mut time = 0.0;
            let mut frame = |form: &mut DownloadForm, enabled, events| {
                time += 0.05;
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(640.0, 4000.0),
                        )),
                        time: Some(time),
                        focused: true,
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| form.show(ui, enabled));
                    },
                )
            };
            for _ in 0..3 {
                frame(&mut form, true, vec![]);
            }
            frame(
                &mut form,
                true,
                vec![Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target: node_id,
                    data: None,
                })],
            );
            let select_all = || Event::Key {
                key: Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                },
            };
            let invalid = frame(&mut form, true, vec![select_all(), Event::Text(bad.into())]);
            assert_eq!(form.advanced_values[index], bad);
            let core_error = AdvancedOption {
                key: CATALOG[index].id.into(),
                value: bad.into(),
            }
            .validate()
            .unwrap_err();
            let node = &invalid
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(n, _)| *n == node_id)
                .expect("actual advanced Edit invalid update")
                .1;
            assert_eq!(node.invalid(), Some(Invalid::True));
            assert_eq!(node.description(), Some(core_error.0));
            let rect = node.bounds().unwrap();
            assert!((rect.height() - 32.0).abs() < 0.1);
            assert!(
                rect.width() <= 360.5,
                "advanced input has a bounded outer width"
            );
            assert!(
                build(&form).is_err(),
                "actual plan rejects invalid advanced value"
            );
            assert!(invalid.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(r) if r.rect.min.x as f64 == rect.x0 && r.rect.min.y as f64 == rect.y0 && r.stroke.color == crate::ui_theme::DANGER_MARKER && r.stroke.width == 1.5)), "actual field danger outline");
            let valid = frame(
                &mut form,
                true,
                vec![select_all(), Event::Text(good.into())],
            );
            let node = &valid
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(n, _)| *n == node_id)
                .expect("actual advanced Edit valid update")
                .1;
            assert_eq!(node.invalid(), None);
            assert_eq!(node.description(), None);
            let valid_rect = node.bounds().unwrap();
            assert!(!valid.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(r) if r.rect.min.x as f64 == valid_rect.x0 && r.rect.min.y as f64 == valid_rect.y0 && r.stroke.color == crate::ui_theme::DANGER_MARKER && r.stroke.width == 1.5)), "valid field removes its danger overlay");
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            assert!(build(&form).is_ok());
            let empty = frame(
                &mut form,
                true,
                vec![
                    select_all(),
                    Event::Key {
                        key: Key::Backspace,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
            assert_eq!(form.advanced_values[index], "");
            let node = &empty
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(n, _)| *n == node_id)
                .expect("actual advanced Edit empty update")
                .1;
            assert_eq!(node.invalid(), None);
            assert_eq!(node.description(), None);
            assert!(
                build(&form).is_ok(),
                "empty retains original omitted-option policy"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            frame(&mut form, false, vec![Event::Text(bad.into())]);
            assert_eq!(
                form.advanced_values[index], "",
                "disabled cannot mutate input"
            );
        }
    }
    #[test]
    fn actual_quality_height_invalid_then_valid_updates_accesskit_and_input_frame() {
        use egui::{
            Event, Key, Modifiers,
            accesskit::{Action, ActionRequest, Invalid, NodeId},
        };
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut form = DownloadForm {
            quality: QualityMode::Height,
            ..Default::default()
        };
        let id = egui::Id::new(("download-option-input", c::QUALITY_HEIGHT));
        let node_id = NodeId(id.value());
        let mut time = 0.0;
        let mut frame = |form: &mut DownloadForm, enabled, events| {
            time += 0.05;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 600.0),
                    )),
                    time: Some(time),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| form.show(ui, enabled));
                },
            )
        };
        for _ in 0..3 {
            frame(&mut form, true, vec![]);
        }
        frame(
            &mut form,
            true,
            vec![Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: node_id,
                data: None,
            })],
        );
        let select_all = || Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            },
        };
        let invalid = frame(&mut form, true, vec![select_all(), Event::Text("0".into())]);
        assert_eq!(form.height, "0");
        let error = form.quality().expect_err("real core rejected height");
        let (_, node) = invalid
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(n, _)| *n == node_id)
            .expect("actual changed height node");
        assert_eq!(node.invalid(), Some(Invalid::True));
        assert_eq!(node.description(), Some(error.0));
        let b = node.bounds().unwrap();
        let rect = egui::Rect::from_min_max(
            egui::pos2(b.x0 as f32, b.y0 as f32),
            egui::pos2(b.x1 as f32, b.y1 as f32),
        );
        let danger = crate::ui_theme::DANGER_MARKER;
        assert!(invalid.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Rect(r)if r.rect==rect&&r.stroke.color==danger&&r.stroke.width==1.5)),"actual scoped input danger stroke");
        let valid = frame(
            &mut form,
            true,
            vec![select_all(), Event::Text("1080".into())],
        );
        assert_eq!(form.height, "1080");
        assert_eq!(form.quality().unwrap(), Quality::Height(1080));
        let (_, node) = valid
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(n, _)| *n == node_id)
            .expect("actual cleared height node");
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        let cursor = egui::text_edit::TextEditState::load(&ctx, id)
            .unwrap()
            .cursor
            .char_range()
            .unwrap();
        assert_eq!(
            cursor.primary.index, 4,
            "caret follows actual replacement, original logical ID unchanged"
        );
        assert!(!valid.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Rect(r)if r.rect==rect&&r.stroke.color==danger&&r.stroke.width==1.5)),"valid input removes danger overlay");
        let saved = form.height.clone();
        let disabled = frame(&mut form, false, vec![Event::Text("0".into())]);
        assert_eq!(form.height, saved, "disabled keeps exact value");
        let (_, node) = disabled
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(n, _)| *n == node_id)
            .expect("actual disabled same-ID node");
        assert!(node.is_disabled());
    }
    // Append inside accessible_options.rs tests; real Form/ScrollArea, not synthetic rects.
    #[test]
    fn actual_height_error_context_is_revealed_and_later_wheel_does_not_resnap() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut form = DownloadForm {
            quality: QualityMode::Height,
            ..Default::default()
        };
        let id = egui::Id::new(("download-option-input", c::QUALITY_HEIGHT));
        let mut time = 0.0;
        let mut frame = |form: &mut DownloadForm, events| {
            time += 0.05;
            let mut offset = egui::Vec2::ZERO;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                time: Some(time),
                focused: true,
                events,
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(2.0);
            let out = ctx.run(input, |ctx| {
                egui::TopBottomPanel::top("height-context-nav")
                    .exact_height(48.0)
                    .show(ctx, |ui| {
                        ui.label("owned navigation");
                    });
                egui::TopBottomPanel::bottom("height-context-footer")
                    .exact_height(100.0)
                    .show(ctx, |ui| {
                        ui.label("owned footer");
                    });
                egui::CentralPanel::default().show(ctx, |ui| {
                    let output = egui::ScrollArea::vertical()
                        .id_salt("height-context-body")
                        .show(ui, |ui| {
                            ui.allocate_space(egui::vec2(1.0, 80.0));
                            form.show(ui, true);
                            if let Some(target) = form.height_reveal_target(ctx) {
                                ui.scroll_to_rect_animation(
                                    target,
                                    None,
                                    egui::style::ScrollAnimation::none(),
                                );
                            }
                        });
                    offset = output.state.offset;
                    let max_y = (output.content_size.y - output.inner_rect.height()).max(0.0);
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("height-context-max-offset"), max_y)
                    });
                });
            });
            (out, offset)
        };
        for _ in 0..5 {
            frame(&mut form, vec![]);
        }
        frame(
            &mut form,
            vec![egui::Event::AccessKitActionRequest(
                egui::accesskit::ActionRequest {
                    action: egui::accesskit::Action::Focus,
                    target: egui::accesskit::NodeId(id.value()),
                    data: None,
                },
            )],
        );
        let modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::NONE
        };
        frame(
            &mut form,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
                egui::Event::Text("0".into()),
            ],
        );
        let error = form.quality().unwrap_err().0;
        let mut settled = frame(&mut form, vec![]);
        for _ in 0..12 {
            settled = frame(&mut form, vec![]);
        }
        let footer =
            egui::containers::panel::PanelState::load(&ctx, egui::Id::new("height-context-footer"))
                .unwrap()
                .rect;
        let (glyph, clip) = settled
            .0
            .shapes
            .iter()
            .find_map(|s| {
                if let egui::epaint::Shape::Text(t) = &s.shape
                    && t.galley.text() == error
                {
                    return Some((t.visual_bounding_rect(), s.clip_rect));
                }
                None
            })
            .expect("actual Height error explanation paint");
        assert!(
            clip.contains_rect(glyph) && glyph.bottom() <= footer.top(),
            "full error explanation before fixed footer: glyph={glyph:?} clip={clip:?} footer={footer:?}"
        );
        let before = settled.1;
        let max_y = ctx
            .data(|data| data.get_temp::<f32>(egui::Id::new("height-context-max-offset")))
            .unwrap();
        assert!(
            (before.y - max_y).abs() < 1.0,
            "actual fixture starts at lower scroll boundary: before={before:?} max={max_y}"
        );
        frame(
            &mut form,
            vec![
                egui::Event::PointerMoved(egui::pos2(200.0, 100.0)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 100.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let mut after = frame(&mut form, vec![]);
        for _ in 0..20 {
            after = frame(&mut form, vec![]);
        }
        assert!(
            after.1.y < before.y - 20.0,
            "actual user wheel moves away from lower boundary: before={before:?} after={:?} pending={:?}",
            after.1,
            form.height_reveal_target(&ctx)
        );
        let moved = after.1;
        for _ in 0..10 {
            after = frame(&mut form, vec![]);
        }
        assert!(
            (after.1.y - moved.y).abs() < 1.0,
            "focused invalid input does not repeatedly reveal after wheel"
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        assert_eq!(form.height, "0");
    }
    #[test]
    fn actual_advanced_help_disclosure_preserves_inputs_validation_and_plan() {
        use egui::{
            Event, Key, Modifiers,
            accesskit::{Action, ActionRequest, Invalid, NodeId, Role},
        };
        fn frame(
            ctx: &egui::Context,
            form: &mut DownloadForm,
            enabled: bool,
            events: Vec<Event>,
            time: &mut f64,
        ) -> egui::FullOutput {
            *time += 0.05;
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(360.0, 4000.0),
                    )),
                    focused: true,
                    time: Some(*time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| form.show(ui, enabled));
                },
            )
        }
        fn button(output: &egui::FullOutput, caption: &str) -> Vec<NodeId> {
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Button && node.label() == Some(caption))
                .map(|(id, _)| *id)
                .collect()
        }
        fn input(output: &egui::FullOutput, id: NodeId) -> &egui::accesskit::Node {
            &output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .expect("actual current field")
                .1
        }
        let event = |target, action| {
            Event::AccessKitActionRequest(ActionRequest {
                target,
                action,
                data: None,
            })
        };
        for dark in [false, true] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            ctx.enable_accesskit();
            let mut form = DownloadForm::default();
            let mut time = 0.0;
            let initial = frame(&ctx, &mut form, true, vec![], &mut time);
            let open = button(&initial, c::ADVANCED_OPEN);
            assert_eq!(open.len(), 1);
            frame(
                &ctx,
                &mut form,
                true,
                vec![event(open[0], Action::Click)],
                &mut time,
            );
            // Wait for the actual opening animation before inspecting painted help.
            for _ in 0..16 {
                frame(&ctx, &mut form, true, vec![], &mut time);
            }
            let baseline = frame(&ctx, &mut form, true, vec![], &mut time);
            let original_plan = argv(&form);
            for entry in CATALOG {
                let id = NodeId(egui::Id::new(("download-option-input", entry.label)).value());
                assert_eq!(input(&baseline, id).invalid(), None);
                assert_eq!(
                    input(&baseline, id).description(),
                    if entry.id == CATALOG[0].id {
                        Some(c::RATE_UNIT_HELP)
                    } else {
                        None
                    }
                );
                for required in [entry.summary, entry.label] {
                    assert!(
                        baseline
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.value() == Some(required)),
                        "visible basic help {required}; actual labels: {:?}",
                        baseline
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes
                            .iter()
                            .filter_map(|(_, node)| node.label())
                            .collect::<Vec<_>>()
                    );
                }
                let default_example = format!("{}: {}", c::HELP_DEFAULT, entry.default);
                assert!(
                    baseline
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.value() == Some(default_example.as_str()))
                );
            }
            let headers = button(&baseline, c::ADVANCED_OPTION_DETAILS);
            assert_eq!(headers.len(), 3);
            let id = NodeId(egui::Id::new(("download-option-input", CATALOG[0].label)).value());
            frame(
                &ctx,
                &mut form,
                true,
                vec![event(id, Action::Focus)],
                &mut time,
            );
            let invalid = frame(
                &ctx,
                &mut form,
                true,
                vec![Event::Text("0".into())],
                &mut time,
            );
            let error = ValidationError(c::RATE_NUMBER_ERROR);
            assert_eq!(form.rate.text, "0");
            assert_eq!(
                form.advanced_values[0], "",
                "invalid rate preserves prior omitted policy"
            );
            assert_eq!(input(&invalid, id).invalid(), Some(Invalid::True));
            assert_eq!(input(&invalid, id).description(), Some(error.0));
            assert!(build(&form).is_err());
            let nodes = &invalid
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            let errors: Vec<_> = nodes
                .iter()
                .filter(|(_, n)| n.role() == Role::Label && n.value() == Some(error.0))
                .collect();
            assert_eq!(errors.len(), 1);
            let error_bounds = errors[0].1.bounds().unwrap();
            let field_bounds = input(&invalid, id).bounds().unwrap();
            assert!(
                error_bounds.y0 >= field_bounds.y1 - 0.5
                    && error_bounds.y0 - field_bounds.y1 <= 24.0,
                "error adjacent to actual input"
            );
            let before = form.advanced_values.clone();
            let before_rate = form.rate.text.clone();
            frame(
                &ctx,
                &mut form,
                true,
                headers
                    .iter()
                    .map(|header| event(*header, Action::Expand))
                    .collect(),
                &mut time,
            );
            for _ in 0..16 {
                frame(&ctx, &mut form, true, vec![], &mut time);
            }
            let expanded = frame(&ctx, &mut form, true, vec![], &mut time);
            for entry in CATALOG {
                assert!(
                    expanded
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, n)| n.value() == Some(entry.tradeoff)),
                    "actual expanded explanation"
                );
            }
            for entry in CATALOG {
                for text in [
                    format!("{}: {}", c::HELP_EXAMPLE, entry.example),
                    format!("{}: {}", c::HELP_CONFLICT, entry.conflicts),
                ] {
                    assert!(
                        expanded
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, n)| n.value() == Some(text.as_str())),
                        "expanded help retains {text}"
                    );
                }
            }
            assert_eq!(form.advanced_values, before);
            assert_eq!(form.rate.text, before_rate);
            assert_eq!(input(&expanded, id).invalid(), Some(Invalid::True));
            assert_eq!(input(&expanded, id).description(), Some(error.0));
            assert!(build(&form).is_err());
            let disabled = frame(
                &ctx,
                &mut form,
                false,
                vec![Event::Text("1".into()), event(headers[0], Action::Collapse)],
                &mut time,
            );
            assert!(input(&disabled, id).is_disabled());
            assert_eq!(form.advanced_values, before);
            assert_eq!(form.rate.text, before_rate);
            assert_eq!(input(&disabled, id).description(), Some(error.0));
            frame(
                &ctx,
                &mut form,
                true,
                headers
                    .iter()
                    .map(|header| event(*header, Action::Collapse))
                    .collect(),
                &mut time,
            );
            for _ in 0..16 {
                frame(&ctx, &mut form, true, vec![], &mut time);
            }
            let closed = frame(&ctx, &mut form, true, vec![], &mut time);
            assert_eq!(input(&closed, id).description(), Some(error.0));
            assert_eq!(form.advanced_values, before);
            assert_eq!(form.rate.text, before_rate);
            frame(
                &ctx,
                &mut form,
                true,
                vec![event(id, Action::Focus)],
                &mut time,
            );
            let clear = Event::Key {
                key: Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            };
            let corrected = frame(
                &ctx,
                &mut form,
                true,
                vec![clear, Event::Text("1.048576".into())],
                &mut time,
            );
            assert_eq!(input(&corrected, id).invalid(), None);
            assert_eq!(input(&corrected, id).description(), Some(c::RATE_UNIT_HELP));
            assert_eq!(form.advanced_values[0], "1048576");
            let corrected_plan = argv(&form);
            frame(
                &ctx,
                &mut form,
                true,
                headers
                    .iter()
                    .map(|header| event(*header, Action::Expand))
                    .collect(),
                &mut time,
            );
            for _ in 0..16 {
                frame(&ctx, &mut form, true, vec![], &mut time);
            }
            assert_eq!(argv(&form), corrected_plan);
            assert_ne!(corrected_plan, original_plan);
            let current = frame(&ctx, &mut form, true, vec![], &mut time);
            let reset = button(&current, c::ADVANCED_RESET);
            assert_eq!(reset.len(), 1);
            frame(
                &ctx,
                &mut form,
                true,
                vec![event(reset[0], Action::Click)],
                &mut time,
            );
            let reset_output = frame(&ctx, &mut form, true, vec![], &mut time);
            assert!(form.advanced_values.iter().all(String::is_empty));
            assert_eq!(argv(&form), original_plan);
            assert_eq!(input(&reset_output, id).invalid(), None);
            assert_eq!(
                input(&reset_output, id).description(),
                Some(c::RATE_UNIT_HELP)
            );
            assert!(form.rate.text.is_empty());
        }
    }

    #[test]
    fn exact_rate_units_round_trip_boundaries_without_float_or_rounding() {
        for unit in RateUnit::ALL {
            for bytes in [1, 999, 1000, 1_048_576, 999_999_999_999, 1_000_000_000_000] {
                assert_eq!(unit.parse(&unit.format(bytes)).unwrap(), Some(bytes));
            }
            for invalid in [
                "0",
                ".",
                "1.",
                "-1",
                " 1",
                "1,5",
                "１",
                "1e3",
                "18446744073709551615",
                "18446744073709551616",
            ] {
                assert!(unit.parse(invalid).is_err(), "{}: {invalid}", unit.label());
            }
        }
        assert_eq!(RateUnit::Megabytes.parse("1.25").unwrap(), Some(1_250_000));
        assert_eq!(RateUnit::Megabytes.parse("0.000001").unwrap(), Some(1));
        assert!(RateUnit::Megabytes.parse("0.0000001").is_err());
        assert_eq!(RateUnit::Megabytes.parse("0.000001000").unwrap(), Some(1));
        assert!(RateUnit::Gigabytes.parse("1000.000000001").is_err());
        assert!(RateUnit::Bytes.parse("1.1").is_err());
    }

    #[test]
    fn actual_rate_input_invalid_preserves_canonical_and_blocks_plan_then_preset_recovers() {
        use egui::{
            Event, Key, Modifiers,
            accesskit::{Action, ActionRequest, Invalid, NodeId, Role},
        };
        use std::collections::BTreeMap;
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut form = DownloadForm {
            advanced_open: true,
            ..Default::default()
        };
        let id = egui::Id::new(("download-option-input", CATALOG[0].label));
        let node_id = NodeId(id.value());
        let mut tree = BTreeMap::new();
        let mut time = 0.0;
        let mut frame = |form: &mut DownloadForm, enabled, events| {
            time += 0.05;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(360.0, 4000.0),
                    )),
                    focused: true,
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| form.show(ui, enabled));
                },
            );
            for (id, node) in &output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
            {
                tree.insert(*id, node.clone());
            }
            (output, tree.clone())
        };
        let action = |target, action| {
            Event::AccessKitActionRequest(ActionRequest {
                target,
                action,
                data: None,
            })
        };
        let select_all = || Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            },
        };
        for _ in 0..3 {
            frame(&mut form, true, vec![]);
        }
        frame(&mut form, true, vec![action(node_id, Action::Focus)]);
        frame(&mut form, true, vec![Event::Text("1.25".into())]);
        assert_eq!(form.advanced_values[0], "1250000");
        let accepted_plan = argv(&form);
        for invalid in [
            "1.",
            "0.0000001",
            "1000000.000001",
            "18446744073709551615",
            "1,5",
        ] {
            let (output, nodes) = frame(
                &mut form,
                true,
                vec![select_all(), Event::Text(invalid.into())],
            );
            assert_eq!(
                form.rate.text, invalid,
                "actual TextEdit draft retains input"
            );
            assert_eq!(
                form.advanced_values[0], "1250000",
                "invalid UI never replaces accepted bytes"
            );
            assert!(
                build(&form).is_err(),
                "even retained valid bytes must not allow Start"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            let node = nodes.get(&node_id).unwrap();
            assert_eq!(node.invalid(), Some(Invalid::True));
            assert_eq!(node.description(), Some(c::RATE_NUMBER_ERROR));
            assert!(
                nodes.values().any(|node| node.role() == Role::ComboBox
                    && node.value() == Some(c::RATE_MBPS)
                    && node.is_disabled()),
                "invalid draft disables unit conversion"
            );
            let rect = node.bounds().unwrap();
            assert!((rect.height() - 32.0).abs() < 0.1);
            assert!(
                rect.width() <= 360.5,
                "rate control preserves bounded input width"
            );
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(r) if r.rect.min.x as f64 == rect.x0 && r.rect.min.y as f64 == rect.y0 && r.stroke.color == crate::ui_theme::DANGER_MARKER && r.stroke.width == 1.5)));
        }
        let (_, nodes) = frame(
            &mut form,
            true,
            vec![select_all(), Event::Text("1.25".into())],
        );
        assert_eq!(nodes.get(&node_id).unwrap().invalid(), None);
        assert_eq!(argv(&form), accepted_plan);
        for (unit_label, expected_text) in [
            (c::RATE_KBPS, "1250"),
            (c::RATE_GBPS, "0.00125"),
            (c::RATE_MBPS, "1.25"),
        ] {
            let (_, nodes) = frame(&mut form, true, vec![]);
            let combo = *nodes
                .iter()
                .find(|(_, n)| {
                    n.role() == Role::ComboBox && n.value() == Some(form.rate.unit.label())
                })
                .unwrap()
                .0;
            frame(&mut form, true, vec![action(combo, Action::Click)]);
            let (_, nodes) = frame(&mut form, true, vec![]);
            assert!(egui::Popup::is_any_open(&ctx));
            let choice = *nodes
                .iter()
                .find(|(_, n)| n.label() == Some(unit_label) && n.supports_action(Action::Click))
                .unwrap()
                .0;
            frame(&mut form, true, vec![action(choice, Action::Click)]);
            frame(&mut form, true, vec![]);
            assert!(!egui::Popup::is_any_open(&ctx));
            assert_eq!(form.rate.text, expected_text);
            assert_eq!(form.advanced_values[0], "1250000");
            assert_eq!(
                argv(&form),
                accepted_plan,
                "unit changes preserve exact plan"
            );
        }
        frame(&mut form, true, vec![action(node_id, Action::Focus)]);
        frame(
            &mut form,
            true,
            vec![select_all(), Event::Text("1.".into())],
        );
        let (_, nodes) = frame(&mut form, true, vec![]);
        let preset = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some(c::RATE_PRESET_FIVE))
            .unwrap()
            .0;
        frame(&mut form, true, vec![action(*preset, Action::Click)]);
        assert_eq!(form.rate.text, "5");
        assert_eq!(form.advanced_values[0], "5000000");
        assert!(build(&form).is_ok());
        frame(&mut form, true, vec![action(node_id, Action::Focus)]);
        let (_, nodes) = frame(
            &mut form,
            true,
            vec![
                select_all(),
                Event::Key {
                    key: Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        assert_eq!(form.rate.text, "");
        assert_eq!(form.advanced_values[0], "");
        assert_eq!(nodes.get(&node_id).unwrap().invalid(), None);
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        assert!(!argv(&form).iter().any(|a| a == "--limit-rate"));
        frame(&mut form, true, vec![Event::Text("5".into())]);
        frame(
            &mut form,
            false,
            vec![select_all(), Event::Text("2".into())],
        );
        assert_eq!(form.rate.text, "5", "disabled input never mutates rate");
        let (_, nodes) = frame(&mut form, true, vec![]);
        let reset = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some(c::ADVANCED_RESET))
            .unwrap()
            .0;
        frame(&mut form, true, vec![action(*reset, Action::Click)]);
        assert_eq!(form.rate.text, "");
        assert!(form.advanced_values.iter().all(String::is_empty));
        assert!(!argv(&form).iter().any(|a| a == "--limit-rate"));
    }
    // Actual DownloadForm + body ScrollArea; focus must reveal offscreen controls once.
    #[test]
    fn actual_rate_presets_and_unit_focus_reveal_full_bounds_without_wheel_resnap() {
        use egui::accesskit::{Action, ActionRequest, NodeId, Role};
        for (role, caption) in [
            (Role::Button, c::RATE_UNLIMITED),
            (Role::Button, c::RATE_PRESET_ONE),
            (Role::Button, c::RATE_PRESET_FIVE),
            (Role::Button, c::RATE_PRESET_TEN),
            (Role::Button, c::ADVANCED_RESET),
            (Role::ComboBox, c::RATE_MBPS),
        ] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.enable_accesskit();
            let mut form = DownloadForm {
                advanced_open: true,
                ..Default::default()
            };
            let original_plan = argv(&form);
            let mut time = 0.0;
            let mut frame = |form: &mut DownloadForm, seed_bottom: bool, events| {
                time += 0.05;
                let mut body = egui::Rect::NOTHING;
                let mut offset = egui::Vec2::ZERO;
                let mut input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    time: Some(time),
                    focused: true,
                    events,
                    ..Default::default()
                };
                input
                    .viewports
                    .get_mut(&egui::ViewportId::ROOT)
                    .unwrap()
                    .native_pixels_per_point = Some(2.0);
                let output = ctx.run(input, |ctx| {
                    egui::TopBottomPanel::top("rate-focus-nav")
                        .exact_height(48.0)
                        .show(ctx, |ui| {
                            ui.label("owned navigation");
                        });
                    egui::TopBottomPanel::bottom("rate-focus-footer")
                        .exact_height(100.0)
                        .show(ctx, |ui| {
                            ui.label("owned footer");
                        });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let mut scroll = egui::ScrollArea::vertical().id_salt("rate-focus-body");
                        if seed_bottom {
                            scroll = scroll.vertical_scroll_offset(2000.0);
                        }
                        let observed = scroll.show(ui, |ui| {
                            ui.allocate_space(egui::vec2(1.0, 240.0));
                            form.show(ui, true);
                            ui.allocate_space(egui::vec2(1.0, 1000.0));
                        });
                        body = observed.inner_rect;
                        offset = observed.state.offset;
                    });
                });
                (output, body, offset)
            };
            let mut initial = frame(&mut form, true, vec![]);
            for _ in 0..4 {
                initial = frame(&mut form, true, vec![]);
            }
            let matches: Vec<_> = initial
                .0
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == role
                        && if role == Role::ComboBox {
                            node.value() == Some(caption)
                        } else {
                            node.label() == Some(caption)
                        }
                })
                .collect();
            assert_eq!(
                matches.len(),
                1,
                "unique actual rate focus target {caption}"
            );
            let target: NodeId = matches[0].0;
            let rect = |output: &egui::FullOutput| {
                let bounds = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == target)
                    .unwrap()
                    .1
                    .bounds()
                    .unwrap();
                egui::Rect::from_min_max(
                    egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                    egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                )
            };
            assert!(
                !initial.1.contains_rect(rect(&initial.0)),
                "fixture starts with actual control outside body"
            );
            frame(
                &mut form,
                false,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    target,
                    action: Action::Focus,
                    data: None,
                })],
            );
            let mut settled = frame(&mut form, false, vec![]);
            for _ in 0..16 {
                settled = frame(&mut form, false, vec![]);
            }
            assert_eq!(
                ctx.memory(|memory| memory.focused()).map(|id| id.value()),
                Some(target.0)
            );
            let target_rect = rect(&settled.0);
            assert!(
                settled.1.contains_rect(target_rect),
                "focused {caption} entire bounds must be in real ScrollArea body: control={target_rect:?}, body={:?}",
                settled.1
            );
            assert!((target_rect.height() - 32.0).abs() < 0.1);
            assert_eq!(
                argv(&form),
                original_plan,
                "focus never applies a preset or changes unit"
            );
            assert!(
                !egui::Popup::is_any_open(&ctx),
                "focus does not open unit popup"
            );
            let focused_offset = settled.2;
            let pointer = settled.1.center();
            frame(
                &mut form,
                false,
                vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -100.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            let mut moved = frame(&mut form, false, vec![]);
            for _ in 0..20 {
                moved = frame(&mut form, false, vec![]);
            }
            assert!(
                moved.2.y > focused_offset.y + 20.0,
                "manual wheel can move away from focused control"
            );
            let wheel_offset = moved.2;
            for _ in 0..10 {
                moved = frame(&mut form, false, vec![]);
            }
            assert!(
                (moved.2.y - wheel_offset.y).abs() < 1.0,
                "held focus must not resnap manual scroll"
            );
            assert_eq!(
                ctx.memory(|memory| memory.focused()).map(|id| id.value()),
                Some(target.0)
            );
            assert_eq!(argv(&form), original_plan);
        }
    }
}

#[cfg(test)]
mod range_and_unit_semantics_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Invalid, Node, NodeId, Role};
    use std::collections::BTreeMap;

    struct Fixture {
        ctx: egui::Context,
        nodes: BTreeMap<NodeId, Node>,
    }
    impl Fixture {
        fn new() -> Self {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            Self {
                ctx,
                nodes: BTreeMap::new(),
            }
        }
        fn frame(
            &mut self,
            form: &mut DownloadForm,
            duration: Option<u64>,
            events: Vec<egui::Event>,
        ) {
            self.ctx.request_repaint();
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 1200.0),
                    )),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if let Some(duration) = duration {
                            form.show_media_range(ui, duration);
                        } else {
                            form.show(ui, true);
                        }
                    });
                },
            );
            self.nodes.extend(
                output
                    .platform_output
                    .accesskit_update
                    .expect("actual semantics tree")
                    .nodes,
            );
        }
        fn input(&self, label: &str) -> &Node {
            self.nodes
                .get(&NodeId(
                    egui::Id::new(("download-option-input", label)).value(),
                ))
                .expect("original stable field node")
        }
        fn click(&self, label: &str) -> egui::Event {
            let target = *self
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.label() == Some(label) && node.supports_action(Action::Click)
                })
                .expect("actual labelled action")
                .0;
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target,
                data: None,
            })
        }
    }
    #[test]
    fn unit_purpose_name_retains_value_conversion_id_and_invalid_edit_gating() {
        let mut fixture = Fixture::new();
        let mut form = DownloadForm {
            advanced_open: true,
            ..Default::default()
        };
        form.advanced_values[0] = "1250000".into();
        fixture.frame(&mut form, None, vec![]);
        let combo = *fixture
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::ComboBox && node.label() == Some(c::RATE_UNIT_LABEL)
            })
            .expect("purpose-labelled actual ComboBox")
            .0;
        assert_eq!(fixture.nodes[&combo].value(), Some(c::RATE_MBPS));
        assert_eq!(fixture.nodes[&combo].description(), Some(c::RATE_UNIT_HELP));
        fixture.frame(
            &mut form,
            None,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target: combo,
                data: None,
            })],
        );
        fixture.frame(&mut form, None, vec![]);
        let action = fixture.click(c::RATE_KBPS);
        fixture.frame(&mut form, None, vec![action]);
        fixture.frame(&mut form, None, vec![]);
        assert_eq!(fixture.nodes[&combo].label(), Some(c::RATE_UNIT_LABEL));
        assert_eq!(fixture.nodes[&combo].value(), Some(c::RATE_KBPS));
        assert_eq!(form.rate.text, "1250");
        assert_eq!(form.advanced_values[0], "1250000");
        form.rate.text = "1,".into();
        fixture.frame(&mut form, None, vec![]);
        assert!(fixture.nodes[&combo].is_disabled());
        assert_eq!(fixture.nodes[&combo].label(), Some(c::RATE_UNIT_LABEL));
        assert_eq!(
            form.advanced_values[0], "1250000",
            "invalid edit must not mutate accepted canonical bytes"
        );
    }
    #[test]
    fn media_range_marks_only_actual_invalid_fields_and_clears_after_correction() {
        for (start, end, invalid_start, invalid_end) in [
            ("bad", "20", true, false),
            ("10", "", false, true),
            ("20", "10", false, true),
            ("bad", "01:99:99", true, true),
        ] {
            let mut fixture = Fixture::new();
            let mut form = DownloadForm {
                start: start.into(),
                end: end.into(),
                ..Default::default()
            };
            let original = (form.start.clone(), form.end.clone(), form.range());
            fixture.frame(&mut form, Some(100_000), vec![]);
            assert_eq!(
                fixture.input(c::RANGE_START).invalid(),
                invalid_start.then_some(Invalid::True)
            );
            assert_eq!(
                fixture.input(c::RANGE_END).invalid(),
                invalid_end.then_some(Invalid::True)
            );
            assert_eq!(
                (form.start.clone(), form.end.clone(), form.range()),
                original,
                "semantics must not rewrite range input or domain result"
            );
            form.start = "10".into();
            form.end = "20".into();
            fixture.frame(&mut form, Some(100_000), vec![]);
            for label in [c::RANGE_START, c::RANGE_END] {
                assert_eq!(fixture.input(label).invalid(), None);
                assert_eq!(fixture.input(label).description(), None);
            }
            assert_eq!(form.media_range(100_000), Ok((10_000, 20_000)));
        }
    }
    #[test]
    fn media_duration_error_is_attached_only_to_out_of_bounds_values() {
        let mut fixture = Fixture::new();
        let mut form = DownloadForm {
            start: "5".into(),
            end: "15".into(),
            ..Default::default()
        };
        fixture.frame(&mut form, Some(10_000), vec![]);
        assert_eq!(fixture.input(c::RANGE_START).invalid(), None);
        assert_eq!(fixture.input(c::RANGE_END).invalid(), Some(Invalid::True));
        assert_eq!(
            fixture.input(c::RANGE_END).description(),
            Some(c::MEDIA_RANGE_INVALID)
        );
        form.start = "20".into();
        form.end = "30".into();
        fixture.frame(&mut form, Some(10_000), vec![]);
        assert_eq!(fixture.input(c::RANGE_START).invalid(), Some(Invalid::True));
        assert_eq!(fixture.input(c::RANGE_END).invalid(), Some(Invalid::True));
        form.start.clear();
        form.end.clear();
        fixture.frame(&mut form, Some(0), vec![]);
        assert_eq!(fixture.input(c::RANGE_START).invalid(), None);
        assert_eq!(
            fixture.input(c::RANGE_END).invalid(),
            None,
            "zero duration with whole-video defaults is a media/group error, not malformed empty fields"
        );
        assert!(form.media_range(0).is_err());
    }
    #[test]
    fn actual_advanced_disclosure_uses_same_range_association_without_duration_guessing() {
        let mut fixture = Fixture::new();
        let mut form = DownloadForm {
            advanced_open: true,
            ..Default::default()
        };
        form.start = "10".into();
        form.end = "bad".into();
        fixture.frame(&mut form, None, vec![]);
        let action = fixture.click(c::OPTIONAL_MEDIA);
        fixture.frame(&mut form, None, vec![action]);
        fixture.frame(&mut form, None, vec![]);
        assert_eq!(fixture.input(c::RANGE_START).invalid(), None);
        assert_eq!(fixture.input(c::RANGE_END).invalid(), Some(Invalid::True));
        assert_eq!(
            fixture.input(c::RANGE_END).description(),
            Some(TimeRange::parse_time("bad").unwrap_err().0)
        );
        form.end = "20".into();
        fixture.frame(&mut form, None, vec![]);
        assert_eq!(fixture.input(c::RANGE_END).invalid(), None);
        assert_eq!(fixture.input(c::RANGE_END).description(), None);
        assert!(form.range().is_ok());
    }
}

#[cfg(test)]
mod naming_context_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled};
    use std::collections::HashMap;

    struct Fixture {
        ctx: egui::Context,
        form: DownloadForm,
        nodes: HashMap<NodeId, Node>,
    }
    impl Fixture {
        fn new(theme: egui::Theme) -> Self {
            let ctx = egui::Context::default();
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.set_theme(theme);
            ctx.enable_accesskit();
            Self {
                ctx,
                form: DownloadForm::default(),
                nodes: HashMap::new(),
            }
        }
        fn frame(&mut self, enabled: bool, events: Vec<egui::Event>) {
            let time = self.ctx.input(|i| i.time) + 0.1;
            let output = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 1600.0),
                    )),
                    time: Some(time),
                    focused: true,
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| self.form.show(ui, enabled));
                },
            );
            if let Some(update) = output.platform_output.accesskit_update {
                self.nodes.extend(update.nodes);
            }
        }
        fn node(&self, label: &str, role: Role) -> (NodeId, &Node) {
            let matches: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, n)| n.label() == Some(label) && n.role() == role)
                .collect();
            assert_eq!(matches.len(), 1, "unique actual {label}");
            (*matches[0].0, matches[0].1)
        }
        fn action(&mut self, id: NodeId, action: Action, enabled: bool) {
            self.frame(
                enabled,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action,
                    target: id,
                    data: None,
                })],
            );
            self.frame(enabled, vec![]);
        }
        fn options(&self) -> DownloadOptions {
            self.form
                .build("https://fixture.invalid/watch", Path::new("fixture"), None)
                .unwrap()
        }
    }

    #[test]
    fn actual_naming_checkbox_default_click_disabled_and_snapshot_plan_roundtrip() {
        use dlpad_core::options::FinalNameStyle;
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let mut f = Fixture::new(theme);
            f.frame(true, vec![]);
            let (id, node) = f.node(c::PRESERVE_VIDEO_TITLE, Role::CheckBox);
            assert!(node.supports_action(Action::Click));
            assert!(!node.is_disabled());
            assert_eq!(node.toggled(), Some(Toggled::True));
            let named = f.options();
            assert_eq!(named.final_name_style, Some(FinalNameStyle::TitleWithJobId));
            let initial_args = named.plan().unwrap().args;
            let initial_preview = crate::accessible_plan::preview(&named).unwrap();
            let output_dir = named.output_dir.clone();
            for wanted in [false, true, false, true] {
                f.action(id, Action::Click, true);
                let (same, node) = f.node(c::PRESERVE_VIDEO_TITLE, Role::CheckBox);
                assert_eq!(same, id, "actual checkbox identity survives toggling");
                assert_eq!(
                    node.toggled(),
                    Some(if wanted {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                );
                let options = f.options();
                assert_eq!(
                    options.final_name_style,
                    wanted.then_some(FinalNameStyle::TitleWithJobId)
                );
                let json = options.to_stored_json().unwrap();
                let value: serde_json::Value = serde_json::from_str(&json).unwrap();
                if wanted {
                    assert_eq!(value["final_name_style"], "TitleWithJobId");
                } else {
                    assert!(
                        value.get("final_name_style").is_none(),
                        "off stores exact legacy absence"
                    );
                }
                let restored = DownloadOptions::from_stored_json(&json).unwrap();
                assert_eq!(restored.final_name_style, options.final_name_style);
                assert_eq!(restored.output_dir, output_dir);
                assert_eq!(
                    restored.plan().unwrap().args,
                    initial_args,
                    "naming cannot change execution arguments or templates"
                );
                assert_eq!(
                    crate::accessible_plan::preview(&restored).unwrap(),
                    initial_preview,
                    "actual redacted preview stays legacy-compatible"
                );
            }
            let before = f.options().to_stored_json().unwrap();
            f.frame(false, vec![]);
            assert!(
                f.node(c::PRESERVE_VIDEO_TITLE, Role::CheckBox)
                    .1
                    .is_disabled()
            );
            f.action(id, Action::Click, false);
            assert_eq!(
                f.options().to_stored_json().unwrap(),
                before,
                "disabled accessibility request cannot mutate naming or other options"
            );
            assert_eq!(f.node(c::PRESERVE_VIDEO_TITLE, Role::CheckBox).0, id);
            f.action(id, Action::Click, true);
            assert!(
                f.options().final_name_style.is_none(),
                "reenabled original checkbox remains actionable"
            );
        }
    }

    #[test]
    fn actual_naming_click_during_invalid_range_keeps_edit_and_validation_gate() {
        use egui::{Event, Key, Modifiers};
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let mut f = Fixture::new(theme);
            f.frame(true, vec![]);
            let id = f.node(c::PRESERVE_VIDEO_TITLE, Role::CheckBox).0;
            let disclosure = f.node(c::OPTIONAL_MEDIA, Role::Button).0;
            f.action(disclosure, Action::Click, true);
            let end = NodeId(egui::Id::new(("download-option-input", c::RANGE_END)).value());
            assert!(
                f.nodes.contains_key(&end),
                "production range TextEdit is present"
            );
            f.action(end, Action::Focus, true);
            f.frame(true, vec![Event::Text("00:".into())]);
            assert_eq!(f.form.end, "00:", "actual TextEdit accepts unfinished edit");
            assert!(
                f.form
                    .build("https://fixture.invalid/watch", Path::new("fixture"), None)
                    .is_err()
            );
            f.action(id, Action::Click, true);
            assert!(!f.form.preserve_title);
            assert_eq!(
                f.form.end, "00:",
                "naming selection preserves pending input"
            );
            assert!(
                f.form
                    .build("https://fixture.invalid/watch", Path::new("fixture"), None)
                    .is_err(),
                "naming must not bypass invalid-range plan gate"
            );
            f.action(end, Action::Focus, true);
            f.frame(
                true,
                vec![
                    Event::Key {
                        key: Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Modifiers {
                            ctrl: true,
                            command: true,
                            ..Modifiers::NONE
                        },
                    },
                    Event::Text("5".into()),
                ],
            );
            assert_eq!(f.form.end, "5");
            // Start remains a Main/Candidate contract: this module exercises its actual form-build prerequisite only.
            // Supply the required matching start as fixture setup after the real end edit.
            f.form.start = "0".into();
            let legacy = f.options().plan().unwrap().args;
            assert!(
                f.form
                    .build("not a URL", Path::new("fixture"), None)
                    .is_err()
            );
            f.action(id, Action::Click, true);
            assert!(f.form.preserve_title);
            assert_eq!(f.form.end, "5");
            assert!(
                f.form
                    .build("not a URL", Path::new("fixture"), None)
                    .is_err(),
                "naming does not repair or bypass invalid URL"
            );
            assert_eq!(
                f.options().plan().unwrap().args,
                legacy,
                "corrected draft keeps selected range argv"
            );
        }
    }
}
