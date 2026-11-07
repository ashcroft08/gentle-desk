//! The error bar for a failure that ends the session (launcher missing, child died).

use iced::widget::{container, text};
use iced::{Element, Length};

/// Emits no events, so it fits any parent message type.
pub fn view<'a, M: 'a>(error: &'a str) -> Element<'a, M> {
    container(text(format!("✗ {error}")))
        .padding(10)
        .width(Length::Fill)
        .style(container::danger)
        .into()
}
