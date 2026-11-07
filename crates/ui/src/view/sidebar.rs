//! The list of saved chats, with its search box.

use gentle_app::SessionItem;
use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::{Element, Length, Theme};

use crate::time_ago::time_ago;

#[derive(Debug, Clone)]
pub enum Event {
    /// The user picked the session stored at this path.
    Open(String),
    /// The user typed in the search box.
    QueryChanged(String),
}

/// What the sidebar draws; `visible` is already filtered by `query`.
pub struct Props<'a> {
    pub visible: Vec<&'a SessionItem>,
    /// How many chats exist, so the header can say "3 of 120".
    pub total: usize,
    pub query: &'a str,
    pub current: Option<&'a str>,
    /// The wall clock (epoch ms), passed in so this stays a pure drawing of its inputs.
    pub now_ms: u64,
}

pub fn view(props: Props<'_>) -> Element<'_, Event> {
    let Props {
        visible,
        total,
        query,
        current,
        now_ms,
    } = props;
    let shown = visible.len();
    let rows = visible.into_iter().map(|session| {
        let style: fn(&Theme, button::Status) -> button::Style =
            if current == Some(session.path.as_str()) {
                button::primary
            } else {
                button::text
            };
        button(
            column![
                text(&session.title).size(14),
                text(format!(
                    "{} · {} msgs · {}",
                    time_ago(now_ms, session.modified_ms),
                    session.message_count,
                    session.project()
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
    let list: Element<'_, Event> = if total == 0 {
        text("No saved chats yet").size(13).into()
    } else if shown == 0 {
        text("No chats match").size(13).into()
    } else {
        scrollable(column(rows).spacing(2))
            .height(Length::Fill)
            .into()
    };
    let title = if query.trim().is_empty() {
        "Chats".to_owned()
    } else {
        format!("Chats ({shown} of {total})")
    };
    container(
        column![
            text(title).size(16),
            text_input("Search chats…", query)
                .size(13)
                .on_input(Event::QueryChanged),
            list,
        ]
        .spacing(8)
        .padding(12),
    )
    .width(260)
    .height(Length::Fill)
    .style(container::bordered_box)
    .into()
}
