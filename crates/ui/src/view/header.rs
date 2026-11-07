//! The title bar of the chat pane.

use iced::widget::{button, row, text, Space};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Event {
    NewChat,
}

pub fn view<'a>() -> Element<'a, Event> {
    row![
        text("gentle-desk").size(18),
        Space::new().width(Length::Fill),
        button("New chat").on_press(Event::NewChat),
    ]
    .align_y(iced::Alignment::Center)
    .padding(12)
    .into()
}
