//! Actual Context/AccessKit action tests. No OS ValuePattern or physical IME claim.
use eframe::egui;
use egui::accesskit::{Action, ActionData, ActionRequest, Node, NodeId, Role};
use egui::text::{CCursor, CCursorRange};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
enum Mode {
    Common,
    Disabled,
    Noninteractive,
    Readonly,
    Password,
}
struct Fixture {
    ctx: egui::Context,
    value: String,
    other: String,
    readonly: &'static str,
    mode: Mode,
    limit: usize,
    time: f64,
    nodes: HashMap<NodeId, Node>,
    root: Option<NodeId>,
    ids: Option<[egui::Id; 2]>,
}
impl Fixture {
    fn new(mode: Mode) -> Self {
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        crate::ui_theme::configure_dlpad_theme(&ctx);
        ctx.enable_accesskit();
        let mut f = Self {
            ctx,
            value: "A한글Z".into(),
            other: "B한글Z".into(),
            readonly: "읽기 전용",
            mode,
            limit: 64,
            time: 0.0,
            nodes: HashMap::new(),
            root: None,
            ids: None,
        };
        f.frame(vec![]);
        f
    }
    fn frame(&mut self, events: Vec<egui::Event>) -> ([egui::Response; 2], egui::FullOutput) {
        self.time += 0.1;
        let mut responses = None;
        let out = self.ctx.run(
            egui::RawInput {
                focused: true,
                time: Some(self.time),
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 400.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let first = ui
                        .add_enabled_ui(!matches!(self.mode, Mode::Disabled), |ui| {
                            match self.mode {
                                Mode::Common | Mode::Disabled => {
                                    crate::ui_components::text_input_with_id(
                                        ui,
                                        &mut self.value,
                                        self.limit,
                                        300.0,
                                        egui::Id::new("accesskit-value-first"),
                                    )
                                }
                                Mode::Noninteractive => ui.add(
                                    egui::TextEdit::singleline(&mut self.value)
                                        .id(egui::Id::new("accesskit-value-first"))
                                        .interactive(false),
                                ),
                                Mode::Readonly => ui.add(
                                    egui::TextEdit::singleline(&mut self.readonly)
                                        .id(egui::Id::new("accesskit-value-first")),
                                ),
                                Mode::Password => ui.add(
                                    egui::TextEdit::singleline(&mut self.value)
                                        .id(egui::Id::new("accesskit-value-first"))
                                        .password(true),
                                ),
                            }
                        })
                        .inner;
                    let other = crate::ui_components::text_input_with_id(
                        ui,
                        &mut self.other,
                        64,
                        300.0,
                        egui::Id::new("accesskit-value-other"),
                    );
                    // Purpose labels only: values and actions still come from the actual editor.
                    ctx.accesskit_node_builder(first.id, |node| node.set_label("첫 입력"));
                    ctx.accesskit_node_builder(other.id, |node| node.set_label("다른 입력"));
                    responses = Some([first, other]);
                });
            },
        );
        let responses = responses.expect("actual editor render");
        self.ids = Some([responses[0].id, responses[1].id]);
        let update = out
            .platform_output
            .accesskit_update
            .as_ref()
            .expect("actual AccessKit update");
        if let Some(tree) = &update.tree {
            self.root = Some(tree.root);
        }
        self.nodes.extend(update.nodes.iter().cloned());
        let mut live = HashSet::new();
        let mut todo = vec![self.root.expect("actual root")];
        while let Some(id) = todo.pop() {
            assert!(live.len() < 256, "bounded current tree");
            if live.insert(id) {
                todo.extend(self.nodes[&id].children().iter().copied());
            }
        }
        self.nodes.retain(|id, _| live.contains(id));
        (responses, out)
    }
    fn node(&self, index: usize) -> (NodeId, &Node) {
        let name = ["첫 입력", "다른 입력"][index];
        let mut found = self.nodes.iter().filter(|(_, n)| {
            n.label() == Some(name) && matches!(n.role(), Role::TextInput | Role::PasswordInput)
        });
        let (id, node) = found.next().expect("current actual TextInput node");
        assert!(found.next().is_none(), "unique actual input");
        (*id, node)
    }
    fn action(&self, index: usize, action: Action, data: Option<ActionData>) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target: self.node(index).0,
            data,
        })
    }
    fn set(&mut self, index: usize, value: &str) -> ([egui::Response; 2], egui::FullOutput) {
        let event = self.action(
            index,
            Action::SetValue,
            Some(ActionData::Value(value.into())),
        );
        self.frame(vec![event])
    }
    fn focus(&mut self, index: usize) {
        let event = self.action(index, Action::Focus, None);
        self.frame(vec![event]);
        self.frame(vec![]);
        assert!(self.ctx.memory(|m| m.has_focus(self.ids.unwrap()[index])));
    }
    fn range(&self, index: usize) -> Option<CCursorRange> {
        egui::TextEdit::load_state(&self.ctx, self.ids.unwrap()[index])
            .expect("actual editor state")
            .cursor
            .char_range()
    }
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
#[test]
fn actual_accesskit_set_value_unfocused_hangul_sanitizes_and_limits_unicode() {
    let mut f = Fixture::new(Mode::Common);
    let (responses, _) = f.set(0, "A한\r\n글\nZ");
    assert_eq!(f.value, "A한  글 Z");
    assert!(responses[0].changed());
    assert!(!responses[0].has_focus());
    assert_eq!(f.node(0).1.value(), Some("A한  글 Z"));
    let (same, _) = f.set(0, "A한  글 Z");
    assert!(!same[0].changed());
    f.limit = 4;
    f.frame(vec![]);
    let (limited, _) = f.set(0, "😀한글AB");
    assert_eq!(f.value, "😀한글A");
    assert_eq!(f.value.chars().count(), 4);
    assert!(limited[0].changed());
}
#[test]
fn actual_accesskit_set_value_preserves_other_field_focus_selection_and_typing() {
    let mut f = Fixture::new(Mode::Common);
    f.focus(1);
    let id = f.ids.unwrap()[1];
    let mut state = egui::TextEdit::load_state(&f.ctx, id).unwrap();
    state
        .cursor
        .set_char_range(Some(CCursorRange::two(CCursor::new(3), CCursor::new(1))));
    state.store(&f.ctx, id);
    f.frame(vec![]);
    let range = f.range(1);
    let (responses, _) = f.set(0, "새 값");
    assert_eq!(f.value, "새 값");
    assert_eq!(f.other, "B한글Z");
    assert!(responses[1].has_focus());
    assert!(!responses[0].has_focus());
    assert_eq!(f.range(1), range);
    f.frame(vec![egui::Event::Text("X".into())]);
    assert_eq!(f.other, "BXZ");
    assert_eq!(f.value, "새 값");
    f.frame(vec![egui::Event::Paste("한\n글".into())]);
    assert_eq!(f.other, "BX한 글Z");
}
#[test]
fn actual_accesskit_set_value_refuses_disabled_noninteractive_and_readonly() {
    for mode in [Mode::Disabled, Mode::Noninteractive, Mode::Readonly] {
        let mut f = Fixture::new(mode);
        let original_node = f.node(0).1.value().map(str::to_owned);
        let (responses, _) = f.set(0, "거절해야 함");
        assert_eq!(f.value, "A한글Z");
        assert_eq!(f.readonly, "읽기 전용");
        assert!(!responses[0].changed());
        assert!(!responses[0].has_focus());
        assert_eq!(f.node(0).1.value(), original_node.as_deref());
    }
}
#[test]
fn actual_accesskit_set_value_password_never_exposes_plaintext_nodes_or_paint() {
    let mut f = Fixture::new(Mode::Password);
    let secret = "synthetic-password-한글";
    let (_, out) = f.set(0, secret);
    assert!(
        f.value == secret,
        "enabled mutable password receives setter"
    );
    assert!(
        f.nodes
            .values()
            .all(|n| n.value().is_none_or(|v| !v.contains(secret))
                && n.label().is_none_or(|v| !v.contains(secret))
                && n.description().is_none_or(|v| !v.contains(secret))),
        "no secret in accessibility metadata"
    );
    assert!(
        out.shapes
            .iter()
            .all(|s| !matches!(&s.shape, egui::Shape::Text(t) if t.galley.text().contains(secret))),
        "no secret in actual painter"
    );
}
#[test]
fn actual_accesskit_set_value_undo_restores_original_text() {
    let mut f = Fixture::new(Mode::Common);
    f.focus(0);
    f.time += 1.0;
    f.frame(vec![]);
    f.set(0, "교체 문자열");
    assert_eq!(f.value, "교체 문자열");
    f.time += 1.0;
    f.frame(vec![]);
    f.frame(vec![key(egui::Key::Z, egui::Modifiers::COMMAND)]);
    assert_eq!(f.value, "A한글Z");
}
#[test]
fn actual_accesskit_set_value_during_preedit_rejects_late_commit_overwrite() {
    let mut f = Fixture::new(Mode::Common);
    f.focus(0);
    f.frame(vec![
        egui::Event::Ime(egui::ImeEvent::Enabled),
        egui::Event::Ime(egui::ImeEvent::Preedit("조합".into())),
    ]);
    f.set(0, "외부 교체");
    assert_eq!(f.value, "외부 교체");
    f.frame(vec![
        egui::Event::Ime(egui::ImeEvent::Commit("늦은 조합".into())),
        egui::Event::Ime(egui::ImeEvent::Disabled),
    ]);
    assert_eq!(f.value, "외부 교체");
}

#[test]
fn actual_accesskit_set_value_capabilities_follow_mutability_and_parent_enabled() {
    for mode in [Mode::Common, Mode::Password] {
        let f = Fixture::new(mode);
        let node = f.node(0).1;
        assert!(!node.is_disabled());
        assert!(!node.is_read_only());
        assert!(node.supports_action(Action::SetValue));
    }
    for mode in [Mode::Noninteractive, Mode::Readonly] {
        let f = Fixture::new(mode);
        let node = f.node(0).1;
        assert!(node.is_read_only());
        assert!(!node.supports_action(Action::SetValue));
    }
    let f = Fixture::new(Mode::Disabled);
    let node = f.node(0).1;
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::SetValue));
}

#[test]
fn actual_accesskit_set_value_rejects_wrong_payload_foreign_node_and_oversize() {
    let mut f = Fixture::new(Mode::Common);
    let numeric = f.action(0, Action::SetValue, Some(ActionData::NumericValue(123.0)));
    let (responses, _) = f.frame(vec![numeric]);
    assert_eq!(f.value, "A한글Z");
    assert!(!responses[0].changed());
    let root = f.root.expect("current actual non-input root");
    assert_ne!(root, f.node(0).0);
    assert_ne!(root, f.node(1).0);
    let (responses, _) = f.frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
        action: Action::SetValue,
        target: root,
        data: Some(ActionData::Value("not an editor".into())),
    })]);
    assert_eq!(f.value, "A한글Z");
    assert_eq!(f.other, "B한글Z");
    assert!(responses.iter().all(|response| !response.changed()));
    let oversized = "한".repeat((2 * 1024 * 1024) / 3 + 1);
    assert!(oversized.len() > 2 * 1024 * 1024);
    let (responses, _) = f.set(0, &oversized);
    // Do not format the injected oversized value in failure diagnostics.
    assert!(
        f.value == "A한글Z",
        "oversize refusal preserves original text"
    );
    assert!(!responses[0].changed());
    assert!(
        f.node(0).1.value() == Some("A한글Z"),
        "actual node retains original after refusal"
    );
}

#[test]
fn actual_accesskit_set_value_target_and_focused_keyboard_each_dispatch_once() {
    let mut f = Fixture::new(Mode::Common);
    f.focus(1);
    f.frame(vec![key(egui::Key::End, egui::Modifiers::NONE)]);
    let setter = f.action(
        0,
        Action::SetValue,
        Some(ActionData::Value("새 첫 값".into())),
    );
    let (responses, _) = f.frame(vec![setter, egui::Event::Text("X".into())]);
    assert_eq!(f.value, "새 첫 값");
    assert_eq!(f.other, "B한글ZX");
    assert!(responses[0].changed() && responses[1].changed());
    assert!(responses[1].has_focus());
    assert!(!responses[0].has_focus());
    let first = f.value.clone();
    let (responses, _) = f.set(1, "다른 대상");
    assert_eq!(f.value, first);
    assert_eq!(f.other, "다른 대상");
    assert!(!responses[0].changed() && responses[1].changed());
}
