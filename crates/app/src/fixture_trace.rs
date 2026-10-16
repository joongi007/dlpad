//! Explicit synthetic-fixture tracing. No text, keys, URLs or coordinates.
use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::mpsc::{SyncSender, sync_channel},
};

#[derive(Debug, Clone, Copy)]
pub struct EventTag {
    pub kind: &'static str,
    pub captured: bool,
}

pub struct FixtureTrace {
    sender: SyncSender<EventTag>,
    sent: usize,
}
impl FixtureTrace {
    pub fn start(root: PathBuf) -> Self {
        let (sender, receiver) = sync_channel::<EventTag>(64);
        std::thread::spawn(move || {
            // root is the canonical root of the retained InstanceGuard.
            let Ok(mut output) = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("fixture-events.jsonl"))
            else {
                return;
            };
            for (sequence, tag) in receiver.into_iter().take(64).enumerate() {
                if writeln!(
                    output,
                    "{{\"sequence\":{},\"kind\":\"{}\",\"captured\":{}}}",
                    sequence + 1,
                    tag.kind,
                    tag.captured
                )
                .is_err()
                {
                    return;
                }
            }
        });
        Self { sender, sent: 0 }
    }
    pub fn observe(&mut self, tag: EventTag) {
        if self.sent < 64 && self.sender.try_send(tag).is_ok() {
            self.sent += 1;
        }
    }
}

pub fn classify(event: iced::Event, status: iced::event::Status) -> Option<EventTag> {
    use iced::{Event, keyboard, mouse, window};
    let kind = match event {
        Event::Mouse(mouse::Event::CursorEntered) => "cursor-entered",
        Event::Mouse(mouse::Event::CursorLeft) => "cursor-left",
        Event::Mouse(mouse::Event::CursorMoved { .. }) => "cursor-moved",
        Event::Mouse(mouse::Event::ButtonPressed(_)) => "button-pressed",
        Event::Mouse(mouse::Event::ButtonReleased(_)) => "button-released",
        Event::Mouse(mouse::Event::WheelScrolled { .. }) => "wheel-scrolled",
        Event::Keyboard(keyboard::Event::KeyPressed { .. }) => "key-pressed",
        Event::Keyboard(keyboard::Event::KeyReleased { .. }) => "key-released",
        Event::Window(window::Event::Focused) => "focused",
        Event::Window(window::Event::Unfocused) => "unfocused",
        _ => return None,
    };
    Some(EventTag {
        kind,
        captured: matches!(status, iced::event::Status::Captured),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn payload_is_omitted_and_redraw_cannot_feed_back() {
        let moved = classify(
            iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position: iced::Point::new(123.0, 456.0),
            }),
            iced::event::Status::Captured,
        )
        .unwrap();
        assert_eq!(moved.kind, "cursor-moved");
        assert!(moved.captured);
        assert!(
            classify(
                iced::Event::Window(iced::window::Event::Resized(iced::Size::new(123.0, 456.0))),
                iced::event::Status::Ignored,
            )
            .is_none()
        );
    }
}
