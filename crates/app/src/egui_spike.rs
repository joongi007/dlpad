//! Alternative native S00 probe. No downloader or product settings are touched.
#[allow(dead_code)] // The shared catalog also contains product adapter strings.
mod catalog;
use catalog as c;
use eframe::egui;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Downloads,
    Library,
    Settings,
}
struct Spike {
    url: String,
    ime: String,
    page: Page,
    fixture: bool,
    scale: u16,
    preference: egui::ThemePreference,
    trace_enabled: bool,
    ime_trace: std::collections::VecDeque<String>,
    trace_sequence: u64,
    commit_fallback: bool,
    ime_active: bool,
}
impl Spike {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_korean_font(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.interact_size.y = 32.0;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
        cc.egui_ctx.set_style(style);
        cc.egui_ctx.set_theme(egui::ThemePreference::System);
        let fixture = std::env::args().any(|a| a == "--fixture");
        Self {
            url: String::new(),
            ime: String::new(),
            page: if std::env::args().any(|arg| arg == "--settings") {
                Page::Settings
            } else if fixture {
                Page::Library
            } else {
                Page::Downloads
            },
            fixture,
            scale: 100,
            preference: egui::ThemePreference::System,
            trace_enabled: fixture && std::env::args().any(|arg| arg == "--ime-events"),
            ime_trace: std::collections::VecDeque::with_capacity(16),
            trace_sequence: 0,
            commit_fallback: fixture && std::env::args().any(|arg| arg == "--ime-commit-fallback"),
            ime_active: false,
        }
    }
}
impl eframe::App for Spike {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.trace_enabled {
            ctx.input(|input| {
                for event in &input.events {
                    let egui::Event::Ime(event) = event else {
                        continue;
                    };
                    let description = match event {
                        egui::ImeEvent::Enabled => "Enabled".to_owned(),
                        egui::ImeEvent::Disabled => "Disabled".to_owned(),
                        egui::ImeEvent::Preedit(value) => {
                            format!("Preedit {:?}", value.chars().take(80).collect::<String>())
                        }
                        egui::ImeEvent::Commit(value) => {
                            format!("Commit {:?}", value.chars().take(80).collect::<String>())
                        }
                    };
                    self.trace_sequence = self.trace_sequence.saturating_add(1);
                    if self.ime_trace.len() == 16 {
                        self.ime_trace.pop_front();
                    }
                    self.ime_trace
                        .push_back(format!("{} {description}", self.trace_sequence));
                }
            });
        }
        // Experimental XIM fallback: commit-only transports need a fresh cursor epoch.
        // Raw trace above stays unchanged so this experiment cannot hide the input evidence.
        if self.commit_fallback {
            ctx.input_mut(|input| {
                let events = std::mem::take(&mut input.events);
                for event in events {
                    if let egui::Event::Ime(ime) = &event {
                        match ime {
                            egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_) => {
                                self.ime_active = true
                            }
                            egui::ImeEvent::Disabled => self.ime_active = false,
                            egui::ImeEvent::Commit(_) => {
                                if !self.ime_active {
                                    input.events.push(egui::Event::Ime(egui::ImeEvent::Enabled));
                                }
                                self.ime_active = false;
                            }
                        }
                    }
                    input.events.push(event);
                }
            });
        }
        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.heading("dlpad");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.page, Page::Downloads, c::DOWNLOADS);
                ui.selectable_value(&mut self.page, Page::Library, c::LIBRARY);
                ui.selectable_value(&mut self.page, Page::Settings, c::SETTINGS);
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(c::METRICS);
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Downloads => {
                let label = ui.label(c::URL);
                ui.add(
                    egui::TextEdit::singleline(&mut self.url)
                        .hint_text(c::PLACEHOLDER)
                        .desired_width(f32::INFINITY),
                )
                .labelled_by(label.id);
                ui.add_space(12.0);
                ui.label(c::EMPTY);
                ui.label(c::STATUS);
            }
            Page::Library => {
                ui.heading(if self.fixture { c::FIXTURE } else { c::EMPTY });
                let count = if self.fixture { 10_000 } else { 0 };
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, 32.0, count, |ui, range| {
                        for index in range {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:05}  한글 영상 · 08:30:00", index + 1));
                                ui.label(c::LOCAL);
                            });
                        }
                    });
            }
            Page::Settings => {
                ui.heading(c::THEME);
                let previous = self.preference;
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.preference,
                        egui::ThemePreference::System,
                        c::SYSTEM,
                    );
                    ui.selectable_value(
                        &mut self.preference,
                        egui::ThemePreference::Light,
                        c::LIGHT,
                    );
                    ui.selectable_value(&mut self.preference, egui::ThemePreference::Dark, c::DARK);
                });
                if previous != self.preference {
                    ctx.set_theme(self.preference);
                }
                ui.add_space(12.0);
                ui.label(c::SCALE);
                let old_scale = self.scale;
                ui.horizontal(|ui| {
                    for scale in [100, 150, 200] {
                        ui.selectable_value(&mut self.scale, scale, format!("{scale}%"));
                    }
                });
                if old_scale != self.scale {
                    ctx.set_zoom_factor(f32::from(self.scale) / 100.0);
                }
                ui.add_space(12.0);
                let label = ui.label(c::IME);
                ui.label(c::IME_HELP);
                ui.add(egui::TextEdit::singleline(&mut self.ime).desired_width(f32::INFINITY))
                    .labelled_by(label.id);
                ui.label(&self.ime);
                if self.trace_enabled {
                    ui.separator();
                    ui.label("S00 IME event trace (latest 16; fixture only)");
                    for event in &self.ime_trace {
                        ui.monospace(event);
                    }
                }
            }
        });
    }
}

fn load_korean_font(ctx: &egui::Context) {
    // Probe-only known paths; production needs OS font discovery with coverage checks.
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("DLPAD_TEST_FONT") {
        candidates.push(std::path::PathBuf::from(path));
    }
    candidates.extend(
        [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "C:/Windows/Fonts/malgun.ttf",
        ]
        .into_iter()
        .map(std::path::PathBuf::from),
    );
    for path in candidates {
        if !std::fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() <= 32 * 1024 * 1024) {
            continue;
        }
        if let Ok(data) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("korean".into(), egui::FontData::from_owned(data).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "korean".into());
            ctx.set_fonts(fonts);
            return;
        }
    }
    eprintln!("Korean font unavailable; set DLPAD_TEST_FONT for this probe.");
}
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(c::TITLE)
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        c::TITLE,
        options,
        Box::new(|cc| Ok(Box::new(Spike::new(cc)))),
    )
}
