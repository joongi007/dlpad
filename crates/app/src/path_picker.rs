//! Bounded GUI coordinator. Native dialog/OS I/O live on one transient STA worker.
use eframe::egui;
use std::path::{Path, PathBuf};
#[cfg(all(windows, feature = "cpu"))]
#[path = "folder_picker.rs"]
mod native_dialog;
#[cfg(all(windows, feature = "cpu"))]
#[path = "folder_picker_worker.rs"]
mod native_worker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Download,
    Library,
    LibraryExport { generation: u64 },
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
    #[cfg(all(windows, feature = "cpu"))]
    pub(crate) fn owner(&self) -> Option<std::sync::Arc<winit::window::Window>> {
        self.owner.clone()
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
                crate::catalog::CHOOSE_SAVE_FOLDER.into(),
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

/// Presentation only. Never use this string to replace an authoritative path.
pub(crate) fn display_path(path: &Path) -> String {
    let display = path.to_string_lossy();
    #[cfg(windows)]
    {
        windows_path_display(&display)
    }
    #[cfg(not(windows))]
    {
        display.into_owned()
    }
}
#[cfg(any(windows, test))]
fn windows_path_display(value: &str) -> String {
    let Some(rest) = value.strip_prefix(r"\\?\") else {
        return value.to_owned();
    };
    // Extended drive paths have exactly an ASCII drive letter, colon and slash.
    let bytes = rest.as_bytes();
    if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\' {
        return rest.to_owned();
    }
    if rest
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(r"UNC\"))
    {
        let tail = &rest[4..];
        let mut parts = tail.split('\\');
        let server = parts.next().unwrap_or_default();
        let share = parts.next().unwrap_or_default();
        if !server.is_empty()
            && !share.is_empty()
            && !matches!(server, "." | ".." | "?")
            && !matches!(share, "." | "..")
        {
            return format!(r"\\{tail}");
        }
    }
    // GLOBALROOT, Volume GUID and other device namespaces keep their spelling.
    value.to_owned()
}

pub(crate) fn folder_field(
    ui: &mut egui::Ui,
    value: &mut String,
    enabled: bool,
    available: bool,
) -> bool {
    let input_id = ui.make_persistent_id("folder-path-input");
    folder_field_with_id(ui, value, enabled, available, input_id)
}

pub(crate) fn folder_field_with_id(
    ui: &mut egui::Ui,
    value: &mut String,
    enabled: bool,
    available: bool,
    input_id: egui::Id,
) -> bool {
    let label = ui.label(
        egui::RichText::new(crate::catalog::SAVE_ROOT)
            .small()
            .weak(),
    );
    let mut browse = false;
    // Preserve a useful path viewport in narrow inspector cards. Wide library
    // rows keep the original inline browse action.
    let stacked = ui.available_width() < 360.0;
    let mut fields = |ui: &mut egui::Ui| {
        let reserve = if stacked {
            0.0
        } else {
            112.0 + ui.spacing().item_spacing.x
        };
        let width =
            (ui.available_width() - reserve - crate::ui_components::TEXT_INPUT_HORIZONTAL_MARGIN)
                .max(32.0);
        let input = ui
            .add_enabled_ui(enabled, |ui| {
                crate::ui_components::text_input_with_id(ui, value, 32768, width, input_id)
            })
            .inner
            .labelled_by(label.id);
        let input = crate::ui_components::context_help(input, value.as_str());
        let focus_requested = input.ctx.input(|state| {
            state.num_accesskit_action_requests(input.id, egui::accesskit::Action::Focus) > 0
        });
        if input.gained_focus() || (input.has_focus() && focus_requested) {
            input.scroll_to_me(None);
        }
        let button = crate::ui_components::secondary_sized(
            ui,
            crate::catalog::CHOOSE_FOLDER,
            enabled && available,
            egui::vec2(112.0, 32.0),
        );
        if button.gained_focus() {
            button.scroll_to_me(None);
        }
        browse = button.clicked();
    };
    if stacked {
        ui.vertical(&mut fields);
    } else {
        ui.horizontal(&mut fields);
    }
    if !available {
        ui.weak(crate::catalog::FOLDER_PICKER_UNAVAILABLE);
    }
    browse
}

/// Cancellation is authoritative even when a selected result was already queued.
#[cfg(any(all(windows, feature = "cpu"), test))]
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
#[cfg(all(test, windows, feature = "cpu"))]
pub(crate) fn registered_folder_proxy_identity_for_test() -> Result<(usize, usize), String> {
    native_worker::registered_proxy_identity_for_test()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_discards_already_completed_selection_without_masking_failure() {
        assert_eq!(
            cancelled_selection(true, Ok(Some(PathBuf::from("selected")))),
            Ok(None)
        );
        assert_eq!(cancelled_selection(true, Err(())), Err(()));
        assert_eq!(cancelled_selection(true, Ok(None)), Ok(None));
    }
    #[test]
    fn normal_selection_and_native_cancel_keep_distinct_results() {
        let selected = PathBuf::from("selected");
        assert_eq!(
            cancelled_selection(false, Ok(Some(selected.clone()))),
            Ok(Some(selected))
        );
        assert_eq!(cancelled_selection(false, Ok(None)), Ok(None));
        assert_eq!(cancelled_selection(false, Err(())), Err(()));
    }
    #[test]
    fn presentation_strips_only_standard_extended_drive_and_unc_prefixes() {
        for (input, expected) in [
            (r"\\?\C:\영상\원본.mp4", r"C:\영상\원본.mp4"),
            (r"\\?\z:\", r"z:\"),
            (r"\\?\UNC\서버\공유\영상.mp4", r"\\서버\공유\영상.mp4"),
            (r"\\?\unc\server\share", r"\\server\share"),
            (r"C:\일반\경로", r"C:\일반\경로"),
            (r"\\server\share\file", r"\\server\share\file"),
        ] {
            assert_eq!(windows_path_display(input), expected);
        }
    }
    #[test]
    fn presentation_preserves_device_namespaces_and_malformed_extended_paths() {
        for input in [
            r"\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy1\file",
            r"\\?\Volume{12345678-1234-1234-1234-123456789012}\file",
            r"\\.\PhysicalDrive0",
            r"\\?\C:relative",
            r"\\?\1:\file",
            r"\\?\UNC\server",
            r"\\?\UNC\\share",
            r"\\?\UNC\server\\file",
            r"\\?\UNC\..\share",
            r"\\?\UNC\server\..\file",
            "",
            "/tmp/영상",
        ] {
            assert_eq!(windows_path_display(input), input);
        }
    }
    #[test]
    fn display_mapping_never_overwrites_original_path() {
        let original = PathBuf::from(r"\\?\C:\원본\영상.mp4");
        let saved = original.clone();
        let display = display_path(&original);
        #[cfg(windows)]
        assert_eq!(display, r"C:\원본\영상.mp4");
        #[cfg(not(windows))]
        assert_eq!(display, original.to_string_lossy());
        assert_eq!(original, saved);
        assert_eq!(original.as_os_str(), saved.as_os_str());
    }

    #[test]
    fn actual_folder_browse_uses_shared_secondary_pointer_states_and_refuses_disabled() {
        use egui::accesskit::{Action, ActionRequest, Node, NodeId, Role};
        use std::collections::{HashMap, HashSet};
        fn rect(node: &Node) -> egui::Rect {
            let b = node.bounds().expect("current folder button bounds");
            egui::Rect::from_min_max(
                egui::pos2(b.x0 as f32, b.y0 as f32),
                egui::pos2(b.x1 as f32, b.y1 as f32),
            )
        }
        fn background(out: &egui::FullOutput, target: egui::Rect) -> egui::Color32 {
            out.shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Rect(r)
                        if r.rect == target && r.fill != egui::Color32::TRANSPARENT =>
                    {
                        assert!(
                            s.clip_rect.contains_rect(r.rect),
                            "entire Browse paint is visible"
                        );
                        Some(r.fill)
                    }
                    _ => None,
                })
                .expect("actual opaque Browse button background")
        }
        fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }
        }
        for dark in [false, true] {
            for width in [320.0, 800.0] {
                let ctx = egui::Context::default();
                crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
                ctx.set_theme(if dark {
                    egui::ThemePreference::Dark
                } else {
                    egui::ThemePreference::Light
                });
                ctx.enable_accesskit();
                let mut value = String::from("C:\\원본\\영상");
                let original = value.clone();
                let mut nodes = HashMap::<NodeId, Node>::new();
                let mut root = None;
                let mut time = 0.0;
                {
                    let mut frame = |enabled: bool, available: bool, events| {
                        time += 0.1;
                        let mut clicked = false;
                        let out = ctx.run(
                            egui::RawInput {
                                focused: true,
                                time: Some(time),
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 240.0),
                                )),
                                events,
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().frame(egui::Frame::NONE).show(
                                    ctx,
                                    |ui| {
                                        clicked = folder_field_with_id(
                                            ui,
                                            &mut value,
                                            enabled,
                                            available,
                                            egui::Id::new("actual-browse-editor"),
                                        );
                                    },
                                );
                            },
                        );
                        let update = out
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .expect("actual folder provider update");
                        if let Some(tree) = &update.tree {
                            root = Some(tree.root);
                        }
                        for (id, node) in &update.nodes {
                            nodes.insert(*id, node.clone());
                        }
                        let mut live = HashSet::new();
                        let mut pending = vec![root.expect("actual root")];
                        while let Some(id) = pending.pop() {
                            if live.insert(id)
                                && let Some(node) = nodes.get(&id)
                            {
                                pending.extend(node.children().iter().copied());
                            }
                        }
                        nodes.retain(|id, _| live.contains(id));
                        let matches: Vec<_> = nodes
                            .iter()
                            .filter(|(_, n)| {
                                n.role() == Role::Button
                                    && n.label() == Some(crate::catalog::CHOOSE_FOLDER)
                            })
                            .collect();
                        assert_eq!(matches.len(), 1, "one actual Korean folder action");
                        let (id, button) = matches[0];
                        (clicked, *id, button.clone(), out)
                    };
                    let (clicked, id, button, idle) = frame(true, true, vec![]);
                    assert!(!clicked && !button.is_disabled());
                    let bounds = rect(&button);
                    assert!(bounds.width() >= 112.0 && bounds.height() >= 32.0);
                    assert!(
                        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 240.0))
                            .contains_rect(bounds)
                    );
                    assert!(idle.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text() == crate::catalog::CHOOSE_FOLDER && s.clip_rect.contains_rect(t.galley.rect.translate(t.pos.to_vec2())))), "actual full Korean Browse label is painted");
                    let normal_fill = background(&idle, bounds);
                    let expected = if dark {
                        crate::ui_tokens::DARK_SECONDARY
                    } else {
                        crate::ui_tokens::LIGHT_SECONDARY
                    };
                    assert_eq!(
                        normal_fill,
                        egui::Color32::from_rgb(expected[0], expected[1], expected[2]),
                        "secondary surface role, not a raw Button default"
                    );
                    let pos = bounds.center();
                    let (clicked, same, _, hover) =
                        frame(true, true, vec![egui::Event::PointerMoved(pos)]);
                    assert!(!clicked);
                    assert_eq!(same, id);
                    assert_eq!(
                        hover.platform_output.cursor_icon,
                        egui::CursorIcon::PointingHand
                    );
                    let hover_fill = background(&hover, bounds);
                    assert_ne!(normal_fill, hover_fill);
                    let (clicked, same, _, press) = frame(
                        true,
                        true,
                        vec![egui::Event::PointerMoved(pos), pointer(pos, true)],
                    );
                    assert!(!clicked);
                    assert_eq!(same, id);
                    assert_ne!(
                        background(&press, bounds),
                        hover_fill,
                        "held press differs from hover"
                    );
                    let (clicked, same, _, _) = frame(
                        true,
                        true,
                        vec![egui::Event::PointerMoved(pos), pointer(pos, false)],
                    );
                    assert!(clicked);
                    assert_eq!(same, id);
                    for (enabled, available) in [(true, false), (false, true)] {
                        let (clicked, same, disabled, _) = frame(enabled, available, vec![]);
                        assert!(!clicked);
                        assert_eq!(same, id);
                        assert!(disabled.is_disabled());
                        let (_, _, _, hover) =
                            frame(enabled, available, vec![egui::Event::PointerMoved(pos)]);
                        assert_ne!(
                            hover.platform_output.cursor_icon,
                            egui::CursorIcon::PointingHand
                        );
                        let (clicked, _, _, _) =
                            frame(enabled, available, vec![pointer(pos, true)]);
                        assert!(!clicked);
                        let (clicked, _, _, _) = frame(
                            enabled,
                            available,
                            vec![
                                pointer(pos, false),
                                egui::Event::AccessKitActionRequest(ActionRequest {
                                    action: Action::Click,
                                    target: id,
                                    data: None,
                                }),
                            ],
                        );
                        assert!(
                            !clicked,
                            "disabled pointer/reader action must not request a dialog"
                        );
                    }
                }
                assert_eq!(
                    value, original,
                    "Browse never mutates the editable Korean path"
                );
            }
        }
    }

    #[test]
    fn actual_folder_field_resizes_without_losing_editor_and_uses_available_width() {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut value = String::from("C:\\원본");
        let mut time = 0.0;
        let mut frame =
            |width: f32, enabled: bool, available: bool, events: Vec<egui::Event>, focus: bool| {
                time += 0.1;
                let mut editor = None;
                let output = ctx.run(
                    egui::RawInput {
                        time: Some(time),
                        focused: true,
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 600.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::NONE)
                            .show(ctx, |ui| {
                                let id = ui.make_persistent_id("folder-path-input");
                                editor = Some(id);
                                if focus {
                                    ctx.memory_mut(|m| m.request_focus(id));
                                }
                                assert!(
                                    !folder_field(ui, &mut value, enabled, available),
                                    "no synthetic browse action"
                                );
                            });
                    },
                );
                let update = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .expect("actual AccessKit update");
                let find = |role| {
                    update
                        .nodes
                        .iter()
                        .find(|(_, n)| n.role() == role)
                        .expect("actual folder role")
                };
                let (input_id, input) = find(egui::accesskit::Role::TextInput);
                let (_, button) = find(egui::accesskit::Role::Button);
                let rect = |node: &egui::accesskit::Node| {
                    let b = node.bounds().expect("actual node bounds");
                    egui::Rect::from_min_max(
                        egui::pos2(b.x0 as f32, b.y0 as f32),
                        egui::pos2(b.x1 as f32, b.y1 as f32),
                    )
                };
                (
                    editor.unwrap(),
                    *input_id,
                    rect(input),
                    rect(button),
                    input.is_disabled(),
                    button.is_disabled(),
                )
            };
        let first = frame(268.0, true, true, vec![], true);
        let narrow = frame(268.0, true, true, vec![], false);
        assert_eq!(first.0, narrow.0);
        assert!(
            (narrow.2.width() - 268.0).abs() <= 1.0,
            "narrow actual editor uses parent width: {:?}",
            narrow.2
        );
        assert!(
            narrow.3.top() >= narrow.2.bottom(),
            "browse is below actual editor"
        );
        assert!(narrow.2.right() <= 268.5 && narrow.3.right() <= 268.5);
        let wide = frame(
            720.0,
            true,
            true,
            vec![egui::Event::Text("X".into())],
            false,
        );
        assert_eq!(
            (first.0, first.1),
            (wide.0, wide.1),
            "resize preserves actual persistent editor identity"
        );
        assert!(
            ctx.memory(|m| m.has_focus(wide.0)),
            "resize preserves actual focus"
        );
        assert!((wide.2.top() - wide.3.top()).abs() <= 1.0);
        assert!(wide.2.right() <= wide.3.left() && wide.3.right() <= 720.5);
        let disabled = frame(268.0, false, true, vec![], false);
        assert!(disabled.4 && disabled.5);
        let unavailable = frame(268.0, true, false, vec![], false);
        assert!(!unavailable.4 && unavailable.5);
        assert!(
            value.contains('X'),
            "actual text event reached the same focused editor after resize"
        );
    }
}
