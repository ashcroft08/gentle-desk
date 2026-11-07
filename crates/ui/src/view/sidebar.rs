//! The list of saved chats.

use gentle_app::SessionItem;
use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length, Theme};

use crate::time_ago::time_ago;

#[derive(Debug, Clone)]
pub enum Event {
    /// The user picked the session stored at this path.
    Open(String),
}

/// `now_ms` is the wall clock (epoch ms), passed in so this stays a pure drawing of its inputs.
pub fn view<'a>(
    sessions: &'a [SessionItem],
    current: Option<&'a str>,
    now_ms: u64,
) -> Element<'a, Event> {
    let rows = sessions.iter().map(|session| {
        let style: fn(&Theme, button::Status) -> button::Style =
            if current == Some(session.path.as_str()) {
                button::primary
            } else {
                button::text
            };
        let project = session.cwd.rsplit(['\\', '/']).next().unwrap_or("");
        button(
            column![
                text(&session.title).size(14),
                text(format!(
                    "{} · {} msgs · {project}",
                    time_ago(now_ms, session.modified_ms),
                    session.message_count
                ))
                .size(11),
            ]
            .spacing(2),
        )
        .width(Length::Fill)
        .style(style)
        .on_press(Event::Open(session.path.clone()))
        .into()
    });
    let list: Element<'_, Event> = if sessions.is_empty() {
        text("No saved chats yet").size(13).into()
    } else {
        scrollable(column(rows).spacing(2))
            .height(Length::Fill)
            .into()
    };
    container(column![text("Chats").size(16), list].spacing(8).padding(12))
        .width(260)
        .height(Length::Fill)
        .style(container::bordered_box)
        .into()
}
