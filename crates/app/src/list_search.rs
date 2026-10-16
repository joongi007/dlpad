//! Explicit bounded search drafts; no I/O or executable authority.
use crate::catalog as c;
use eframe::egui;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Request {
    pub generation: u64,
    pub query: String,
}
#[derive(Default)]
pub(crate) struct Search {
    pub draft: String,
    pub current: Request,
    pub queued: Option<Request>,
    published: u64,
    published_query: String,
    invalid: bool,
    result_reveal: Option<u64>,
}
impl Search {
    pub fn enqueue(&mut self, query: &str) {
        self.result_reveal = None;
        let query = query.trim();
        if query.len() > 256 || query.contains('\0') {
            self.invalid = true;
            return;
        }
        let Some(generation) = self.current.generation.checked_add(1) else {
            self.invalid = true;
            return;
        };
        self.invalid = false;
        self.current = Request {
            generation,
            query: query.to_owned(),
        };
        self.queued = Some(self.current.clone());
    }
    pub fn accepts(&self, request: &Request) -> bool {
        self.current == *request
    }
    pub fn published(&mut self, request: &Request) {
        if self.accepts(request) {
            let first_publication = self.published != request.generation;
            self.published = request.generation;
            self.published_query.clone_from(&request.query);
            if first_publication && request.generation > 0 && self.draft.trim() == request.query {
                self.result_reveal = Some(request.generation);
            }
        }
    }
    pub fn cancel_result_reveal(&mut self) {
        self.result_reveal = None;
    }
    pub fn reveal_result(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let generation = self.result_reveal.take();
        // Return this one publication's UI reveal permission, including when its
        // ID already fits. Callers may reveal a fitting surrounding group without
        // altering the actual Label rect, focus, selection, or execution authority.
        let reveal = generation == Some(self.current.generation)
            && !self.stale()
            && !self.invalid
            && ui.is_enabled();
        if reveal && !ui.clip_rect().contains_rect(response.rect) {
            response.scroll_to_me(None);
        }
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_description(format!(
                "{}: {}",
                c::LIST_SEARCH_APPLIED,
                self.published_query
            ));
        });
        reveal
    }
    pub fn stale(&self) -> bool {
        self.published != self.current.generation || self.draft.trim() != self.published_query
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: &'static str,
        label: &'static str,
        enabled: bool,
    ) {
        if !enabled || !ui.is_enabled() || self.stale() {
            self.cancel_result_reveal();
        }
        let caption = ui.label(label);
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                let width = (ui.available_width() - 200.0).clamp(80.0, 360.0);
                let response = crate::ui_components::text_input_with_id(
                    ui,
                    &mut self.draft,
                    256,
                    width,
                    egui::Id::new(id),
                )
                .labelled_by(caption.id);
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_description(c::LIST_SEARCH_HELP)
                });
                if response.enabled()
                    && ui.input_mut(|input| {
                        input.consume_key(egui::Modifiers::COMMAND, egui::Key::F)
                    })
                {
                    response.request_focus();
                }
                let enter = response.enabled()
                    && response.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                let submit = ui
                    .scope_builder(
                        egui::UiBuilder::new().id(egui::Id::new((id, "submit"))),
                        |ui| crate::ui_components::secondary(ui, c::LIST_SEARCH, true),
                    )
                    .inner;
                if enter || submit.clicked() {
                    let query = self.draft.clone();
                    self.enqueue(&query);
                }
                let clear = ui
                    .scope_builder(
                        egui::UiBuilder::new().id(egui::Id::new((id, "clear"))),
                        |ui| {
                            crate::ui_components::ghost(
                                ui,
                                c::LIST_SEARCH_CLEAR,
                                !self.draft.is_empty() || !self.current.query.is_empty(),
                            )
                        },
                    )
                    .inner;
                if clear.clicked() {
                    self.draft.clear();
                    self.enqueue("");
                }
            });
        });
        if self.invalid {
            ui.colored_label(ui.visuals().error_fg_color, c::LIST_SEARCH_LIMIT);
        }
        ui.weak(if self.published_query.is_empty() {
            c::LIST_SEARCH_ALL.to_owned()
        } else {
            format!("{}: {}", c::LIST_SEARCH_APPLIED, self.published_query)
        });
        if self.stale() {
            ui.weak(c::LIST_SEARCH_STALE);
        }
        crate::ui_components::context_help(caption, c::LIST_SEARCH_HELP);
    }
}

#[cfg(test)]
pub(crate) mod interaction_tests {
    use super::*;
    pub(crate) struct Fixture {
        pub ctx: egui::Context,
        nodes: std::collections::HashMap<egui::accesskit::NodeId, egui::accesskit::Node>,
        time: f64,
        root: Option<egui::accesskit::NodeId>,
        painted: std::collections::HashSet<String>,
        pub screen: egui::Vec2,
    }
    impl Fixture {
        pub fn new() -> Self {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            crate::ui_theme::configure_dlpad_theme(&ctx);
            ctx.enable_accesskit();
            Self {
                ctx,
                nodes: Default::default(),
                time: 0.0,
                root: None,
                painted: Default::default(),
                screen: egui::vec2(800.0, 600.0),
            }
        }
        pub fn frame(&mut self, events: Vec<egui::Event>, mut show: impl FnMut(&mut egui::Ui)) {
            self.time += 0.05;
            let output = self.ctx.run(
                egui::RawInput {
                    focused: true,
                    time: Some(self.time),
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.screen)),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| show(ui));
                },
            );
            self.painted = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    egui::Shape::Text(text)
                        if shape.clip_rect.contains_rect(text.visual_bounding_rect()) =>
                    {
                        Some(text.galley.text().to_owned())
                    }
                    _ => None,
                })
                .collect();
            let update = output
                .platform_output
                .accesskit_update
                .expect("actual list tree");
            if let Some(tree) = update.tree {
                self.root = Some(tree.root);
            }
            for (id, node) in update.nodes {
                self.nodes.insert(id, node);
            }
            let mut live = std::collections::HashSet::new();
            let mut pending = vec![self.root.expect("actual current root")];
            while let Some(id) = pending.pop() {
                assert!(live.len() < 512, "bounded actual current tree");
                if live.insert(id) {
                    pending.extend(self.nodes[&id].children().iter().copied());
                }
            }
            self.nodes.retain(|id, _| live.contains(id));
        }
        pub fn fully_painted(&self, text: &str) -> bool {
            self.painted.contains(text)
        }
        pub fn is_nonfocusable_label(&self, text: &str) -> bool {
            self.nodes.values().any(|node| {
                node.role() == egui::accesskit::Role::Label
                    && (node.label() == Some(text) || node.value() == Some(text))
                    && !node.supports_action(egui::accesskit::Action::Focus)
            })
        }
        /// Inject only a failure-path request addressed to an actual disabled node.
        /// This does not claim the OS permits Invoke on a disabled button.
        pub fn disabled_button_click(&self, label: &str) -> egui::Event {
            let mut nodes = self.nodes.iter().filter(|(_, node)| {
                node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
            });
            let (id, node) = nodes.next().expect("actual disabled button node");
            assert!(
                nodes.next().is_none(),
                "unique actual disabled action target"
            );
            assert!(
                node.is_disabled(),
                "failure-path target is actually disabled"
            );
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target: *id,
                data: None,
            })
        }

        pub fn action(
            &self,
            role: egui::accesskit::Role,
            label: Option<&str>,
            action: egui::accesskit::Action,
        ) -> egui::Event {
            let mut nodes = self.nodes.iter().filter(|(_, node)| {
                node.role() == role && label.is_none_or(|label| node.label() == Some(label))
            });
            let (id, node) = nodes.next().expect("actual control node");
            assert!(nodes.next().is_none(), "unique actual action target");
            assert!(!node.is_disabled(), "actual action is enabled");
            egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
                action,
                target: *id,
                data: None,
            })
        }
    }
    pub(crate) fn enter() -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn actual_list_search_input_enter_clear_and_draft_scope_are_visible() {
        let mut f = Fixture::new();
        let mut search = Search::default();
        f.frame(vec![], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        let shortcut = egui::Event::Key {
            key: egui::Key::F,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        };
        f.frame(vec![shortcut], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        assert!(
            f.ctx
                .memory(|memory| memory.has_focus(egui::Id::new("search-fixture"))),
            "Ctrl+F reaches the real current search input"
        );
        f.frame(vec![egui::Event::Text("  한글%_  ".into())], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        assert!(search.queued.is_none(), "typing is not a database query");
        assert!(
            search.stale(),
            "old displayed results are marked while editing"
        );
        f.frame(vec![enter()], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        let request = search.queued.take().expect("real Enter dispatch");
        assert_eq!(request.query, "한글%_");
        search.published(&request);
        let clear = f.action(
            egui::accesskit::Role::Button,
            Some(c::LIST_SEARCH_CLEAR),
            egui::accesskit::Action::Click,
        );
        f.frame(vec![clear], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        assert!(search.draft.is_empty());
        let cleared = search.queued.take().expect("real Clear dispatch");
        assert!(cleared.query.is_empty() && cleared.generation > request.generation);
        assert!(search.stale());
        search.published(&cleared);
        assert!(!search.stale());
        f.frame(vec![], |ui| {
            search.show(ui, "search-fixture", c::LIBRARY_SEARCH_LABEL, true)
        });
        assert!(
            f.nodes
                .values()
                .any(|node| node.role() == egui::accesskit::Role::Label
                    && (node.label() == Some(c::LIST_SEARCH_ALL)
                        || node.value() == Some(c::LIST_SEARCH_ALL))),
            "published query scope is present in the actual current Label value/name"
        );
        assert!(
            f.painted.contains(c::LIST_SEARCH_ALL),
            "published scope is actually painted inside its clip"
        );
    }
}
