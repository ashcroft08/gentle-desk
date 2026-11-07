//! The prompt box with its Send / Stop button.

use iced::widget::{button, row, text_input};
use iced::Element;

/// Id of the prompt box, so the app can focus it at start.
pub const ID: &str = "composer";

#[derive(Debug, Clone)]
pub enum Event {
    Changed(String),
    Send,
    Stop,
}

/// `can_send` is the app's decision (non-blank text, a live child).
pub fn view(input: &str, working: bool, can_send: bool) -> Element<'_, Event> {
    let action = if working {
        button("Stop").on_press(Event::Stop)
    } else {
        button("Send").on_press_maybe(can_send.then_some(Event::Send))
    };
    row![
        text_input("Message gentle-shell…", input)
            .id(ID)
            .on_input(Event::Changed)
            .on_submit(Event::Send),
        action,
    ]
    .spacing(8)
    .padding(12)
    .into()
}
