//! Actual egui widget regressions, suitable for ORIGINAL-red/vendor-green comparison.
//! No native event injection: these are the observed adapter's translated events.
#[cfg(test)]
mod tests {
    use eframe::egui::{
        self, Event, ImeEvent, Key, Modifiers,
        text::{CCursor, CCursorRange},
    };
    struct Editor {
        ctx: egui::Context,
        text: String,
        other: String,
        id: egui::Id,
        other_id: egui::Id,
        time: f64,
    }
    impl Editor {
        fn new(text: &str, primary: usize, secondary: usize) -> Self {
            let mut editor = Self {
                ctx: Default::default(),
                text: text.into(),
                other: String::new(),
                id: egui::Id::new("tested-entry"),
                other_id: egui::Id::new("other-entry"),
                time: 0.0,
            };
            editor.frame(vec![]);
            editor.ctx.memory_mut(|m| m.request_focus(editor.id));
            // Settle gained_focus before installing the tested selection.
            editor.frame(vec![]);
            let mut state = egui::TextEdit::load_state(&editor.ctx, editor.id).unwrap();
            state.cursor.set_char_range(Some(CCursorRange {
                primary: CCursor::new(primary),
                secondary: CCursor::new(secondary),
                h_pos: None,
            }));
            state.store(&editor.ctx, editor.id);
            editor
        }
        fn frame(&mut self, events: Vec<Event>) -> CCursorRange {
            self.time += 0.05;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut cursor = None;
            let _ = self.ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    cursor = egui::TextEdit::singleline(&mut self.text)
                        .id(self.id)
                        .show(ui)
                        .cursor_range;
                    egui::TextEdit::singleline(&mut self.other)
                        .id(self.other_id)
                        .show(ui);
                });
            });
            cursor
                .or_else(|| {
                    egui::TextEdit::load_state(&self.ctx, self.id)
                        .unwrap()
                        .cursor
                        .char_range()
                })
                .unwrap_or_else(|| {
                    assert!(
                        !self.ctx.memory(|m| m.has_focus(self.id)),
                        "focused editor must expose a real cursor"
                    );
                    CCursorRange::default()
                })
        }
        fn settle(&mut self) {
            self.time += 10.0;
            self.frame(vec![]);
        }
        fn start(&mut self, text: &str) {
            self.frame(vec![
                ime(ImeEvent::Enabled),
                ime(ImeEvent::Preedit(text.into())),
            ]);
        }
        fn terminal(&mut self, commit: &str) -> CCursorRange {
            self.frame(vec![
                ime(ImeEvent::Disabled),
                ime(ImeEvent::Commit(commit.into())),
            ])
        }
    }
    fn ime(event: ImeEvent) -> Event {
        Event::Ime(event)
    }
    fn key(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        }
    }
    #[test]
    fn exact_native_translated_cancel_restores_original_text_and_selection() {
        for (text, p, s) in [
            ("", 0, 0),
            ("AB", 1, 1),
            ("ABC", 3, 0),
            ("ABC", 0, 3),
            ("가😀나다", 3, 1),
        ] {
            let mut editor = Editor::new(text, p, s);
            editor.start("한글");
            let cursor = editor.terminal("");
            assert_eq!(editor.text, text, "original {text:?}, selection {p}:{s}");
            assert_eq!((cursor.primary.index, cursor.secondary.index), (p, s));
        }
    }
    #[test]
    fn exact_native_nonempty_commit_replaces_selection_once() {
        for (text, p, s, expected, at) in [
            ("", 0, 0, "한글", 2),
            ("AB", 1, 1, "A한글B", 3),
            ("ABC", 0, 3, "한글", 2),
            ("ABC", 3, 0, "한글", 2),
            ("가😀나다", 3, 1, "가한글다", 3),
            ("가😀나다", 1, 3, "가한글다", 3),
        ] {
            let mut editor = Editor::new(text, p, s);
            editor.start("조");
            let cursor = editor.terminal("한글");
            assert_eq!(editor.text, expected);
            assert_eq!((cursor.primary.index, cursor.secondary.index), (at, at));
            editor.frame(vec![ime(ImeEvent::Disabled)]);
            assert_eq!(editor.text, expected);
        }
    }
    #[test]
    fn multiple_preedits_and_disabled_commit_split_frames_restore_selection() {
        let mut editor = Editor::new("ABC", 3, 0);
        editor.start("ㅎ");
        editor.frame(vec![ime(ImeEvent::Preedit("한".into()))]);
        editor.settle();
        editor.frame(vec![ime(ImeEvent::Preedit("한글".into()))]);
        editor.frame(vec![ime(ImeEvent::Disabled)]);
        let cursor = editor.frame(vec![ime(ImeEvent::Commit(String::new()))]);
        assert_eq!(editor.text, "ABC");
        assert_eq!((cursor.primary.index, cursor.secondary.index), (3, 0));
    }
    #[test]
    fn unrelated_disabled_and_focus_change_do_not_erase_other_input() {
        let mut editor = Editor::new("AB", 1, 1);
        editor.frame(vec![Event::Text("x".into()), ime(ImeEvent::Disabled)]);
        assert_eq!(editor.text, "AxB");
        editor.start("한글");
        editor.ctx.memory_mut(|m| m.request_focus(editor.other_id));
        editor.frame(vec![
            Event::Text("other".into()),
            ime(ImeEvent::Disabled),
            ime(ImeEvent::Commit(String::new())),
        ]);
        assert_eq!(editor.other, "other");
        editor.ctx.memory_mut(|m| m.request_focus(editor.id));
        editor.frame(vec![ime(ImeEvent::Disabled)]);
        assert!(!editor.text.contains("other"));
    }
    #[test]
    fn externally_replaced_text_is_not_overwritten_by_stale_cancel_snapshot() {
        let mut editor = Editor::new("ABC", 3, 0);
        editor.start("한글");
        editor.text = "external😀".into();
        editor.terminal("");
        assert_eq!(editor.text, "external😀");
    }
    #[test]
    fn oversized_preedit_does_not_clobber_original_selection() {
        let mut editor = Editor::new("ABC", 3, 0);
        editor.start(&"한".repeat(1024 * 1024 + 1));
        editor.terminal("");
        assert!(
            editor.text == "ABC",
            "oversized preedit clobbered original text"
        );
    }
    #[test]
    fn cancel_after_elapsed_undo_checkpoint_does_not_resurrect_uncommitted_preedit() {
        let mut editor = Editor::new("", 0, 0);
        editor.settle();
        editor.frame(vec![Event::Text("ABC".into())]);
        editor.settle();
        editor.start("한글");
        editor.settle();
        editor.frame(vec![ime(ImeEvent::Preedit("한글말".into()))]);
        editor.settle();
        editor.terminal("");
        assert_eq!(editor.text, "ABC");
        editor.settle();
        editor.frame(vec![key(Key::Z)]);
        assert_eq!(editor.text, "");
        editor.frame(vec![key(Key::Y)]);
        assert_eq!(editor.text, "ABC");
        assert!(!editor.text.contains("한"));
    }
    #[test]
    fn whole_native_translated_batch_cancels_or_commits_without_double_replace() {
        for (commit, expected) in [("", "ABC"), ("한글", "한글")] {
            let mut editor = Editor::new("ABC", 3, 0);
            editor.frame(vec![
                ime(ImeEvent::Enabled),
                ime(ImeEvent::Preedit("조".into())),
                ime(ImeEvent::Disabled),
                ime(ImeEvent::Commit(commit.into())),
            ]);
            assert_eq!(editor.text, expected);
        }
    }
    #[test]
    fn split_frame_nonempty_commit_keeps_selection_replacement_correct() {
        for (text, p, s, expected, at) in [
            ("가😀나다", 3, 1, "가한글다", 3),
            ("가😀나다", 1, 3, "가한글다", 3),
            ("ABC", 3, 0, "한글", 2),
            ("ABC", 0, 3, "한글", 2),
        ] {
            let mut editor = Editor::new(text, p, s);
            editor.start("조");
            editor.frame(vec![ime(ImeEvent::Disabled)]);
            let cursor = editor.frame(vec![ime(ImeEvent::Commit("한글".into()))]);
            assert_eq!(editor.text, expected);
            assert_eq!((cursor.primary.index, cursor.secondary.index), (at, at));
        }
    }
}
