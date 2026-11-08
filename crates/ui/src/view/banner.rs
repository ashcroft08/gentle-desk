//! The error bar for a failure that ends the whole session: gentle-shell could
//! not be resolved. A chat's child dying is a per-chat notice with a Restart, not
//! this bar (S3).

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
