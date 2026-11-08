//! The list of saved chats, with its search box.
//!
//! Rows have a fixed height and only the ones near the viewport exist as
//! widgets (see `windowing`), so the cost of drawing does not grow with the
//! number of chats; blank spacers keep the scrollable's full height.

use gentle_app::{ListStatus, SessionItem};
use iced::widget::{button, column, container, responsive, scrollable, text, text_input, Space};
use iced::{Element, Length, Theme};

use crate::text::truncate;
use crate::time_ago::time_ago;
use crate::windowing::window;

mod status;

/// Id of the scrollable, so the app can send it back to the top.
pub const SCROLL_ID: &str = "sidebar-scroll";

/// Height of every row, in pixels. Rows never wrap, so this is exact.
const ROW_HEIGHT: f32 = 50.0;
/// Rows built above and below the viewport so a fast scroll shows no gap.
const OVERSCAN: usize = 4;
/// Longest title shown on the single line of a row, in characters.
const TITLE_CHARS: usize = 32;

#[derive(Debug, Clone)]
pub enum Event {
    /// The user picked the session stored at this path.
    Open(String),
    /// The user typed in the search box.
    QueryChanged(String),
    /// The list was scrolled to this offset, in pixels from the top.
    Scrolled(f32),
}

/// What the sidebar draws; `visible` is already filtered by `query`.
pub struct Props<'a> {
    pub visible: Vec<&'a SessionItem>,
    /// How many chats exist, so the header can say "3 of 120".
    pub total: usize,
    pub query: &'a str,
    /// Whether the list is still coming, and how far the pass got.
    pub list_status: ListStatus,
    pub current: Option<&'a str>,
    /// The wall clock (epoch ms), passed in so this stays a pure drawing of its inputs.
    pub now_ms: u64,
    /// How far the list is scrolled, in pixels.
    pub offset: f32,
}

pub fn view(props: Props<'_>) -> Element<'_, Event> {
    let Props {
        visible,
        total,
        query,
        list_status,
        current,
        now_ms,
        offset,
    } = props;
    let texts = status::texts(list_status, total, visible.len(), query);
    let list: Element<'_, Event> = match texts.stand_in {
        Some(line) => text(line).size(13).into(),
        // `responsive` hands over the height this pane really has, which also
        // follows a resize of the window.
        None => {
            responsive(move |size| rows_in_view(&visible, current, now_ms, offset, size.height))
                .into()
        }
    };
    container(
        column![
            text(texts.title).size(16),
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

/// The scrollable with only the rows that intersect `viewport`.
fn rows_in_view<'a>(
    visible: &[&'a SessionItem],
    current: Option<&str>,
    now_ms: u64,
    offset: f32,
    viewport: f32,
) -> Element<'a, Event> {
    let w = window(visible.len(), ROW_HEIGHT, offset, viewport, OVERSCAN);
    let rows = visible[w.rows.clone()]
        .iter()
        .map(|session| row(session, current, now_ms));
    scrollable(
        column![
            Space::new().height(w.space_above),
            column(rows),
            Space::new().height(w.space_below),
        ]
        .width(Length::Fill),
    )
    .id(SCROLL_ID)
    .on_scroll(|viewport| Event::Scrolled(viewport.absolute_offset().y))
    .height(Length::Fill)
    .into()
}

fn row<'a>(session: &'a SessionItem, current: Option<&str>, now_ms: u64) -> Element<'a, Event> {
    let style: fn(&Theme, button::Status) -> button::Style =
        if current == Some(session.path.as_str()) {
            button::primary
        } else {
            button::text
        };
    let details = format!(
        "{} · {} msgs · {}",
        time_ago(now_ms, session.modified_ms),
        session.message_count,
        session.project()
    );
    button(
        column![
            text(truncate(&session.title, TITLE_CHARS))
                .size(14)
                .wrapping(text::Wrapping::None),
            text(details).size(11).wrapping(text::Wrapping::None),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .height(ROW_HEIGHT)
    .padding([6, 10])
    .style(style)
    .on_press(Event::Open(session.path.clone()))
    .into()
}
