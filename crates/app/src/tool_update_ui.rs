//! Candidate native tool-version UI. No worker, download, filesystem or activation here.
use crate::{catalog as c, managed::ToolId, ui_components as controls, ui_tokens as t};
use dlpad_engine::tools::{InstallManifest, Ownership};
use eframe::egui;

pub(crate) const MAX_INSTALLED_VERSIONS: usize = 32;
const MAX_ID_BYTES: usize = 128;
const TOOLS: [ToolId; 3] = [ToolId::YtDlp, ToolId::Deno, ToolId::Ffmpeg];
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VersionKey {
    pub version: String,
    pub target: String,
}
impl VersionKey {
    fn of(manifest: &InstallManifest) -> Self {
        Self {
            version: manifest.version.clone(),
            target: manifest.target.clone(),
        }
    }
    fn valid(&self) -> bool {
        bounded(&self.version, MAX_ID_BYTES) && bounded(&self.target, MAX_ID_BYTES)
    }
    fn label(&self) -> String {
        format!("{} · {}", self.version, self.target)
    }
}
#[derive(Clone, Debug)]
pub(crate) enum Eligibility {
    Ready,
    Checking,
    Unavailable(&'static str),
}
impl Eligibility {
    fn ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
    fn reason(&self) -> Option<&'static str> {
        match self {
            Self::Ready => None,
            Self::Checking => Some(c::TOOL_UPDATE_SELECTION_CHECKING),
            Self::Unavailable(v) => Some(*v),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum CurrentVersion {
    Unknown,
    External,
    Managed(VersionKey),
}
#[derive(Clone, Debug)]
pub(crate) enum Status {
    Idle,
    Checking,
    Installing,
    Selecting,
    Offline,
    Unsupported,
    Unverified,
    Failed(&'static str),
}
impl Status {
    fn busy(&self) -> bool {
        matches!(self, Self::Checking | Self::Installing | Self::Selecting)
    }
    fn label(&self) -> Option<&'static str> {
        match self {
            Self::Idle => None,
            Self::Checking => Some(c::TOOL_UPDATE_CHECKING),
            Self::Installing => Some(c::TOOL_UPDATE_INSTALLING),
            Self::Selecting => Some(c::TOOL_UPDATE_SELECTING),
            Self::Offline => Some(c::TOOL_UPDATE_OFFLINE),
            Self::Unsupported => Some(c::TOOL_UPDATE_UNSUPPORTED),
            Self::Unverified => Some(c::TOOL_UPDATE_UNVERIFIED),
            Self::Failed(v) => Some(*v),
        }
    }
}
/// Worker maps a captured release into a canonical tool/version/target display key.
/// Ready requires a worker-held captured yt-dlp pin or an exact approved compiled Catalog match.
/// UI JSON or a saved receipt can never supply that authority.
#[derive(Clone, Debug)]
pub(crate) struct ObservedVersion {
    pub key: VersionKey,
    pub install: Eligibility,
}
#[derive(Clone, Debug)]
pub(crate) struct InstalledVersion {
    pub manifest: InstallManifest,
    pub select: Eligibility,
}
#[derive(Clone, Debug)]
pub(crate) struct ToolSnapshot {
    pub tool: ToolId,
    pub generation: u64,
    pub current: CurrentVersion,
    pub observed: Option<ObservedVersion>,
    pub installed: Vec<InstalledVersion>,
    pub status: Status,
}
impl ToolSnapshot {
    fn valid(&self) -> bool {
        if self.installed.len() > MAX_INSTALLED_VERSIONS {
            return false;
        }
        if let CurrentVersion::Managed(key) = &self.current
            && !key.valid()
        {
            return false;
        }
        if self.observed.as_ref().is_some_and(|v| !v.key.valid()) {
            return false;
        }
        for (index, v) in self.installed.iter().enumerate() {
            let m = &v.manifest;
            if m.tool != self.tool.id()
                || !matches!(m.ownership, Ownership::Managed)
                || !VersionKey::of(m).valid()
                || !bounded(&m.executable, 256)
                || !bounded(&m.source_url, 2048)
                || !bounded(&m.license, 4096)
                || m.sha256.len() != 64
                || !m.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || self.installed[..index].iter().any(|other| {
                    other.manifest.version == m.version && other.manifest.target == m.target
                })
            {
                return false;
            }
        }
        true
    }
}
/// One bounded intent per frame. Root rechecks snapshot generation, Catalog, owned manifest,
/// runtime/hash/root-lock and cancellation; these display snapshots convey no supply authority.
#[derive(Clone, Debug)]
pub(crate) enum Intent {
    Check {
        tool: ToolId,
        generation: u64,
    },
    InstallApprovedObserved {
        tool: ToolId,
        generation: u64,
        version: VersionKey,
    },
    UseInstalled {
        tool: ToolId,
        generation: u64,
        manifest: InstallManifest,
    },
}
#[derive(Default)]
pub(crate) struct Panel {
    embedded: bool,
    tool: ToolId,
    selected: [Option<VersionKey>; 3],
}
fn bounded(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn slot(tool: ToolId) -> usize {
    match tool {
        ToolId::YtDlp => 0,
        ToolId::Deno => 1,
        ToolId::Ffmpeg => 2,
    }
}
fn stable_control(
    ui: &mut egui::Ui,
    tool: ToolId,
    key: &str,
    label: &str,
    primary: bool,
    enabled: bool,
) -> egui::Response {
    ui.scope_builder(
        egui::UiBuilder::new()
            .id(ui.make_persistent_id(("tool-update-action", tool.id(), key)))
            .global_scope(true),
        |ui| {
            if primary {
                controls::primary(ui, label, enabled)
            } else {
                controls::secondary(ui, label, enabled)
            }
        },
    )
    .inner
}
#[cfg(test)]
fn probe(ui: &egui::Ui, key: &'static str, response: &egui::Response) {
    let frame = ui.ctx().cumulative_frame_nr();
    ui.ctx().data_mut(|data| {
        data.insert_temp(
            egui::Id::new(("tool-update-probe", key)),
            (frame, response.clone()),
        )
    });
}
impl Panel {
    /// Reuse the existing Tools screen's selected tab without adding another navigation row.
    pub(crate) fn show_for_tool(
        &mut self,
        ui: &mut egui::Ui,
        tool: ToolId,
        snapshots: &[ToolSnapshot; 3],
        mutate: bool,
        closing: bool,
        blocked_reason: Option<&'static str>,
    ) -> Option<Intent> {
        self.tool = tool;
        self.embedded = true;
        let intent = self.show(ui, snapshots, mutate, closing, blocked_reason);
        self.embedded = false;
        intent
    }
    /// Root's mutate gate covers global downloads/worker leases; closing freezes every action.
    /// Root dispatch rejection must keep original snapshots and surface its reason next frame.
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        snapshots: &[ToolSnapshot; 3],
        mutate: bool,
        closing: bool,
        blocked_reason: Option<&'static str>,
    ) -> Option<Intent> {
        ui.label(c::TOOL_UPDATE_TITLE);
        if !self.embedded {
            ui.horizontal_wrapped(|ui| {
                for tool in TOOLS {
                    let response = ui
                        .scope_builder(
                            egui::UiBuilder::new()
                                .id(ui.make_persistent_id(("tool-update-tab", tool.id())))
                                .global_scope(true),
                            |ui| {
                                ui.add_enabled_ui(!closing, |ui| {
                                    controls::choice(ui, self.tool == tool, tool.label())
                                })
                                .inner
                            },
                        )
                        .inner;
                    if response.clicked() {
                        self.tool = tool;
                    }
                    #[cfg(test)]
                    if tool == ToolId::YtDlp {
                        probe(ui, "tab", &response);
                    }
                }
            });
        }
        ui.add_space(t::CONTROL_GAP);
        if TOOLS
            .iter()
            .any(|tool| snapshots.iter().filter(|s| s.tool == *tool).count() != 1)
        {
            ui.colored_label(ui.visuals().error_fg_color, c::TOOL_UPDATE_INVALID_SNAPSHOT);
            return None;
        }
        let s = snapshots
            .iter()
            .find(|s| s.tool == self.tool)
            .expect("three unique tool snapshots");
        if !s.valid() {
            ui.colored_label(ui.visuals().error_fg_color, c::TOOL_UPDATE_INVALID_SNAPSHOT);
            return None;
        }
        let mut intent = None;
        let enabled =
            mutate && !closing && !s.status.busy() && !matches!(s.status, Status::Unsupported);
        if !self.embedded {
            ui.horizontal_wrapped(|ui| {
                ui.label(c::TOOL_UPDATE_CURRENT);
                ui.label(match &s.current {
                    CurrentVersion::Unknown => c::TOOL_UPDATE_CURRENT_UNKNOWN.to_owned(),
                    CurrentVersion::External => c::TOOL_UPDATE_EXTERNAL.to_owned(),
                    CurrentVersion::Managed(v) => v.label(),
                });
            });
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(c::TOOL_UPDATE_OBSERVED);
            ui.label(
                s.observed
                    .as_ref()
                    .map_or_else(|| c::TOOL_UPDATE_NOT_CHECKED.to_owned(), |v| v.key.label()),
            );
        });
        if let Some(reason) = s.status.label() {
            ui.label(reason);
        }
        if closing {
            ui.label(c::CLOSE_WAIT_HELP);
        } else if !mutate && let Some(reason) = blocked_reason {
            ui.label(reason);
        }
        let already_installed = s.observed.as_ref().is_some_and(|v| {
            s.installed
                .iter()
                .any(|i| VersionKey::of(&i.manifest) == v.key)
        });
        ui.horizontal_wrapped(|ui| {
            let check = stable_control(ui, s.tool, "check", c::TOOL_UPDATE_CHECK, false, enabled);
            #[cfg(test)]
            probe(ui, "check", &check);
            if check.clicked() {
                intent = Some(Intent::Check {
                    tool: s.tool,
                    generation: s.generation,
                });
            }
            let install_enabled = enabled
                && !matches!(s.status, Status::Offline | Status::Unverified)
                && !already_installed
                && s.observed.as_ref().is_some_and(|v| v.install.ready());
            let install = stable_control(
                ui,
                s.tool,
                "install",
                c::TOOL_UPDATE_INSTALL,
                true,
                install_enabled,
            );
            #[cfg(test)]
            probe(ui, "install", &install);
            if install.clicked()
                && intent.is_none()
                && let Some(observed) = &s.observed
            {
                intent = Some(Intent::InstallApprovedObserved {
                    tool: s.tool,
                    generation: s.generation,
                    version: observed.key.clone(),
                });
            }
        });
        if already_installed {
            ui.label(c::TOOL_UPDATE_ALREADY_INSTALLED);
        }
        if let Some(reason) = s.observed.as_ref().and_then(|v| v.install.reason()) {
            ui.label(reason);
        }
        ui.label(c::TOOL_UPDATE_INSTALL_ONLY_HELP);
        ui.add_space(t::SECTION_GAP);
        let selected = &mut self.selected[slot(s.tool)];
        if selected.as_ref().is_some_and(|key| {
            !s.installed
                .iter()
                .any(|v| VersionKey::of(&v.manifest) == *key)
        }) {
            *selected = None;
        }
        let label = ui.label(c::TOOL_UPDATE_INSTALLED);
        let caption = selected
            .as_ref()
            .map_or_else(|| c::TOOL_UPDATE_CHOOSE.to_owned(), VersionKey::label);
        let response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .id(ui.make_persistent_id(("tool-update-installed", s.tool.id())))
                    .global_scope(true),
                |ui| {
                    ui.add_enabled_ui(enabled && !s.installed.is_empty(), |ui| {
                        let combo = egui::ComboBox::from_id_salt("versions")
                            .selected_text(caption)
                            .width(ui.available_width().min(360.0))
                            .truncate()
                            .height(240.0)
                            .show_ui(ui, |ui| {
                                for row in &s.installed {
                                    let key = VersionKey::of(&row.manifest);
                                    let response = ui
                                        .scope_builder(
                                            egui::UiBuilder::new()
                                                .id(ui.make_persistent_id((
                                                    "version",
                                                    s.tool.id(),
                                                    &key.version,
                                                    &key.target,
                                                )))
                                                .global_scope(true),
                                            |ui| {
                                                ui.add_enabled_ui(row.select.ready(), |ui| {
                                                    controls::choice(
                                                        ui,
                                                        selected.as_ref() == Some(&key),
                                                        &key.label(),
                                                    )
                                                })
                                                .inner
                                            },
                                        )
                                        .inner;
                                    if response.clicked() {
                                        *selected = Some(key);
                                        ui.close();
                                    }
                                }
                            })
                            .response
                            .labelled_by(label.id);
                        controls::combo_response(ui, combo)
                    })
                    .inner
                },
            )
            .inner;
        #[cfg(not(test))]
        let _ = response;
        #[cfg(test)]
        probe(ui, "combo", &response);
        if s.installed.is_empty() {
            ui.label(c::TOOL_UPDATE_NO_INSTALLED);
        }
        let chosen = selected.as_ref().and_then(|key| {
            s.installed
                .iter()
                .find(|v| VersionKey::of(&v.manifest) == *key)
        });
        let current = chosen.is_some_and(
            |v| matches!(&s.current,CurrentVersion::Managed(k)if*k==VersionKey::of(&v.manifest)),
        );
        let use_enabled = enabled && !current && chosen.is_some_and(|v| v.select.ready());
        let use_version = stable_control(
            ui,
            s.tool,
            "use",
            c::TOOL_UPDATE_USE_SELECTED,
            false,
            use_enabled,
        );
        #[cfg(test)]
        probe(ui, "use", &use_version);
        if use_version.clicked()
            && intent.is_none()
            && let Some(chosen) = chosen
        {
            intent = Some(Intent::UseInstalled {
                tool: s.tool,
                generation: s.generation,
                manifest: chosen.manifest.clone(),
            });
        }
        if current {
            ui.label(c::TOOL_UPDATE_VERSION_ACTIVE);
        }
        if let Some(reason) = chosen.and_then(|v| v.select.reason()) {
            ui.label(reason);
        }
        if selected.is_none() && !s.installed.is_empty() {
            ui.label(c::TOOL_UPDATE_SELECT_FIRST);
        }
        ui.label(c::TOOL_UPDATE_USE_HELP);
        intent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Role};
    fn key(version: &str) -> VersionKey {
        VersionKey {
            version: version.into(),
            target: "x86_64-pc-windows-msvc".into(),
        }
    }
    fn installed(tool: ToolId, version: &str) -> InstalledVersion {
        InstalledVersion {
            manifest: InstallManifest {
                tool: tool.id().into(),
                version: version.into(),
                target: key(version).target,
                source_url: "https://supply.invalid/owned-fixture".into(),
                sha256: "a".repeat(64),
                license: "synthetic".into(),
                ownership: Ownership::Managed,
                executable: "tool.exe".into(),
            },
            select: Eligibility::Ready,
        }
    }
    fn snapshots() -> [ToolSnapshot; 3] {
        TOOLS.map(|tool| ToolSnapshot {
            tool,
            generation: 17,
            current: CurrentVersion::Managed(key("1.0.0")),
            observed: Some(ObservedVersion {
                key: key("3.0.0"),
                install: Eligibility::Ready,
            }),
            installed: vec![installed(tool, "1.0.0"), installed(tool, "2.0.0")],
            status: Status::Idle,
        })
    }
    struct Fixture {
        ctx: egui::Context,
        panel: Panel,
        snapshots: [ToolSnapshot; 3],
        time: f64,
        size: egui::Vec2,
        closing: bool,
        mutate: bool,
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
                panel: Panel::default(),
                snapshots: snapshots(),
                time: 0.0,
                size: egui::vec2(640.0, 800.0),
                closing: false,
                mutate: true,
            }
        }
        fn frame(&mut self, events: Vec<egui::Event>) -> (egui::FullOutput, Option<Intent>) {
            self.time += 0.05;
            let mut intent = None;
            let out = self.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.size)),
                    focused: true,
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("actual-tool-update-parent")
                            .show(ui, |ui| {
                                intent = self.panel.show(
                                    ui,
                                    &self.snapshots,
                                    self.mutate,
                                    self.closing,
                                    Some(c::TOOL_RUNNING),
                                );
                            });
                    });
                },
            );
            (out, intent)
        }
        fn response(&self, name: &'static str) -> egui::Response {
            let (frame, response) = self
                .ctx
                .data(|data| {
                    data.get_temp::<(u64, egui::Response)>(egui::Id::new((
                        "tool-update-probe",
                        name,
                    )))
                })
                .expect("actual rendered control probe");
            assert_eq!(
                frame + 1,
                self.ctx.cumulative_frame_nr(),
                "probe must come from this frame"
            );
            response
        }
    }
    fn action(response: &egui::Response, action: Action) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            target: response.id.value().into(),
            action,
            data: None,
        })
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn rect_shapes<'a>(
        shape: &'a egui::Shape,
        rect: egui::Rect,
        out: &mut Vec<&'a egui::epaint::RectShape>,
    ) {
        match shape {
            egui::Shape::Rect(r) if rect.contains_rect(r.rect) => out.push(r),
            egui::Shape::Vec(v) => {
                for s in v {
                    rect_shapes(s, rect, out)
                }
            }
            _ => {}
        }
    }
    fn fill(out: &egui::FullOutput, r: &egui::Response) -> egui::Color32 {
        let mut shapes = Vec::new();
        for s in &out.shapes {
            rect_shapes(&s.shape, r.rect, &mut shapes);
        }
        shapes
            .iter()
            .find(|s| s.rect == r.rect)
            .expect("actual command fill painted")
            .fill
    }
    fn has_text(shape: &egui::Shape, text: &str) -> bool {
        match shape {
            egui::Shape::Text(t) => t.galley.text() == text,
            egui::Shape::Vec(v) => v.iter().any(|s| has_text(s, text)),
            _ => false,
        }
    }
    #[test]
    fn actual_update_controls_paint_distinct_hover_pressed_focus_and_selected_states() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark);
            let (idle, intent) = f.frame(vec![]);
            assert!(intent.is_none());
            let check = f.response("check");
            assert!(check.enabled());
            assert!(check.rect.height() >= t::CONTROL_HEIGHT);
            let idle_fill = fill(&idle, &check);
            let tab = f.response("tab");
            assert!(
                idle.platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(id, n)| *id == tab.id.value().into()
                        && n.toggled() == Some(egui::accesskit::Toggled::True))
            );
            f.frame(vec![egui::Event::PointerMoved(check.rect.center())]);
            let (hover, _) = f.frame(vec![]);
            let hovered = f.response("check");
            assert_eq!(hovered.id, check.id);
            assert!(hovered.hovered());
            let hover_fill = fill(&hover, &hovered);
            assert_ne!(idle_fill, hover_fill);
            let (down, intent) = f.frame(vec![pointer(check.rect.center(), true)]);
            let pressed = f.response("check");
            assert!(intent.is_none());
            assert!(pressed.is_pointer_button_down_on());
            assert_ne!(fill(&down, &pressed), hover_fill);
            let (_, intent) = f.frame(vec![pointer(check.rect.center(), false)]);
            assert!(matches!(
                intent,
                Some(Intent::Check {
                    tool: ToolId::YtDlp,
                    generation: 17
                })
            ));
            let (focused, intent) = f.frame(vec![
                egui::Event::PointerGone,
                action(&check, Action::Focus),
            ]);
            assert!(intent.is_none());
            let actual = f.response("check");
            assert!(actual.has_focus());
            assert_eq!(actual.id, check.id);
            let mut shapes = Vec::new();
            for s in &focused.shapes {
                rect_shapes(&s.shape, actual.rect, &mut shapes);
            }
            assert!(
                shapes
                    .iter()
                    .any(|s| s.stroke.width >= 1.5 && s.stroke.color.a() == 255),
                "actual opaque focus stroke"
            );
            let install = f.response("install");
            let (_, intent) = f.frame(vec![action(&install, Action::Click)]);
            match intent {
                Some(Intent::InstallApprovedObserved {
                    tool: ToolId::YtDlp,
                    generation: 17,
                    version,
                }) => assert_eq!(version, key("3.0.0")),
                _ => panic!(
                    "enabled observed-install control must emit exact approved snapshot version"
                ),
            }
        }
    }
    #[test]
    fn actual_installed_choice_emits_exact_owned_manifest_after_reorder_and_closes_popup() {
        let mut f = Fixture::new(false);
        f.frame(vec![]);
        let combo = f.response("combo");
        f.frame(vec![action(&combo, Action::Click)]);
        let (opened, _) = f.frame(vec![]);
        assert!(egui::Popup::is_any_open(&f.ctx));
        let label = key("2.0.0").label();
        let target = opened
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, n)| {
                n.role() == Role::Button
                    && n.label() == Some(label.as_str())
                    && n.supports_action(Action::Click)
            })
            .expect("actual installed version choice")
            .0;
        f.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action: Action::Click,
            data: None,
        })]);
        f.frame(vec![]);
        assert!(!egui::Popup::is_any_open(&f.ctx));
        assert_eq!(f.panel.selected[0], Some(key("2.0.0")));
        let use_version = f.response("use");
        let expected = serde_json::to_vec(&f.snapshots[0].installed[1].manifest).unwrap();
        let (_, intent) = f.frame(vec![action(&use_version, Action::Click)]);
        match intent {
            Some(Intent::UseInstalled {
                tool: ToolId::YtDlp,
                generation: 17,
                manifest,
            }) => assert_eq!(serde_json::to_vec(&manifest).unwrap(), expected),
            _ => panic!("actual selection must emit exact installed identity"),
        }
        f.snapshots[0].installed.swap(0, 1);
        f.snapshots[0]
            .installed
            .insert(0, installed(ToolId::YtDlp, "0.9.0"));
        f.frame(vec![]);
        assert_eq!(f.response("use").id, use_version.id);
        assert_eq!(f.panel.selected[0], Some(key("2.0.0")));
        let (_, intent) = f.frame(vec![action(&use_version, Action::Click)]);
        match intent {
            Some(Intent::UseInstalled { manifest, .. }) => {
                assert_eq!(serde_json::to_vec(&manifest).unwrap(), expected)
            }
            _ => panic!("reorder cannot retarget selected action"),
        }
        f.snapshots[0]
            .installed
            .retain(|v| v.manifest.version != "2.0.0");
        f.frame(vec![]);
        assert!(f.panel.selected[0].is_none());
        assert!(!f.response("use").enabled());
    }
    #[test]
    fn actual_update_unverified_offline_unsupported_and_close_wait_never_emit_disabled_installs() {
        for status in [
            Status::Checking,
            Status::Offline,
            Status::Unsupported,
            Status::Unverified,
        ] {
            let mut f = Fixture::new(false);
            f.snapshots[0].status = status.clone();
            let (out, _) = f.frame(vec![]);
            let install = f.response("install");
            assert!(!install.enabled());
            assert!(
                out.shapes
                    .iter()
                    .any(|s| has_text(&s.shape, status.label().unwrap())),
                "mandatory reason actually painted"
            );
            let (_, intent) = f.frame(vec![action(&install, Action::Click)]);
            assert!(intent.is_none());
        }
        let mut error = Fixture::new(false);
        error.snapshots[0].status = Status::Failed(c::TOOL_FAILED);
        let (out, _) = error.frame(vec![]);
        assert!(
            out.shapes
                .iter()
                .any(|s| has_text(&s.shape, c::TOOL_FAILED))
        );
        assert!(
            error.response("check").enabled(),
            "error recovery keeps explicit recheck available"
        );
        let mut f = Fixture::new(false);
        f.snapshots[0].observed.as_mut().unwrap().install =
            Eligibility::Unavailable(c::TOOL_UPDATE_UNVERIFIED);
        let (out, _) = f.frame(vec![]);
        let install = f.response("install");
        assert!(!install.enabled());
        assert!(
            out.shapes
                .iter()
                .any(|s| has_text(&s.shape, c::TOOL_UPDATE_UNVERIFIED))
        );
        f.snapshots[0].observed.as_mut().unwrap().install = Eligibility::Ready;
        f.closing = true;
        let (out, _) = f.frame(vec![]);
        assert!(
            out.shapes
                .iter()
                .any(|s| has_text(&s.shape, c::CLOSE_WAIT_HELP))
        );
        for name in ["tab", "check", "install", "combo", "use"] {
            let response = f.response(name);
            assert!(!response.enabled());
            let (_, intent) = f.frame(vec![action(&response, Action::Click)]);
            assert!(intent.is_none());
        }
        assert_eq!(
            f.snapshots[0].current.clone().managed_version_for_test(),
            Some(key("1.0.0"))
        );
    }
    #[test]
    fn actual_narrow_200_percent_focus_reveals_full_command_and_version_control() {
        for dark in [false, true] {
            let mut f = Fixture::new(dark);
            f.ctx.set_zoom_factor(2.0);
            f.size = egui::vec2(280.0, 300.0);
            f.panel.selected[0] = Some(key("2.0.0"));
            f.frame(vec![]);
            for name in ["check", "install", "combo", "use"] {
                let before = f.response(name);
                assert!(before.rect.height() >= t::CONTROL_HEIGHT);
                f.frame(vec![
                    egui::Event::PointerGone,
                    action(&before, Action::Focus),
                ]);
                for _ in 0..3 {
                    f.frame(vec![]);
                }
                let (out, intent) = f.frame(vec![]);
                assert!(intent.is_none());
                let r = f.response(name);
                assert_eq!(r.id, before.id);
                assert!(r.has_focus());
                let clip = out
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::Shape::Rect(rect) if rect.rect == r.rect => Some(s.clip_rect),
                        _ => None,
                    })
                    .expect("actual focused control background");
                assert!(
                    clip.contains_rect(r.rect),
                    "200% narrow control fully inside body after focus reveal: {name} rect={:?} clip={clip:?}",
                    r.rect
                );
                assert!(r.rect.width() <= f.size.x, "no horizontal overflow");
            }
        }
    }
    #[test]
    fn invalid_or_overflowed_snapshot_refuses_all_intents_without_selecting_another_version() {
        let mut f = Fixture::new(false);
        f.frame(vec![]);
        let original_check = f.response("check");
        f.snapshots[0].installed = (0..=MAX_INSTALLED_VERSIONS)
            .map(|index| installed(ToolId::YtDlp, &format!("1.{index}.0")))
            .collect();
        let (out, intent) = f.frame(vec![action(&original_check, Action::Click)]);
        assert!(intent.is_none());
        assert!(
            out.shapes
                .iter()
                .any(|s| has_text(&s.shape, c::TOOL_UPDATE_INVALID_SNAPSHOT))
        );
        assert!(f.panel.selected[0].is_none());
    }
    impl CurrentVersion {
        fn managed_version_for_test(self) -> Option<VersionKey> {
            if let Self::Managed(k) = self {
                Some(k)
            } else {
                None
            }
        }
    }
}
