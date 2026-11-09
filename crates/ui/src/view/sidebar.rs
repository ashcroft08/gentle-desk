//! The list of saved chats, with its search box.
//!
//! Rows have a fixed height and only the ones near the viewport exist as
//! widgets (see `windowing`), so the cost of drawing does not grow with the
//! number of chats; blank spacers keep the scrollable's full height.

use gentle_app::{ChatActivity, ListStatus, SessionItem};
use iced::widget::{
    button, column, container, responsive, row, scrollable, text, text_input, Space,
};
use iced::{Element, Font, Length, Theme};
use std::collections::BTreeMap;

use crate::windowing::window;
use gentle_runtime::text::truncate;
use gentle_runtime::time_ago::time_ago;

mod badge;
mod status;

/// Id of the scrollable, so the app can send it back to the top.
pub const SCROLL_ID: &str = "sidebar-scroll";

/// Height of every row, in pixels. Rows never wrap, so this is exact.
const ROW_HEIGHT: f32 = 50.0;
/// Rows built above and below the viewport so a fast scroll shows no gap.
const OVERSCAN: usize = 4;
/// Longest title shown on the single line of a row, in characters.
const TITLE_CHARS: usize = 32;
/// The title of a row with a badge is drawn bold, so a reply waiting to be
/// read reads as heavier from across the list.
const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::DEFAULT
};

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
    /// The activities the rows' badges draw (S4), keyed by session path: one
    /// plain snapshot of the chats that are not idle, not the whole app state.
    /// A row absent from it is idle and draws nothing. Asking the snapshot per
    /// drawn row keeps the cost of a frame by the rows actually drawn, so it
    /// stays constant however many chats the list lists.
    pub activities: BTreeMap<String, ChatActivity>,
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
        activities,
    } = props;
    let texts = status::texts(list_status, total, visible.len(), query);
    let list: Element<'_, Event> = match texts.stand_in {
        Some(line) => text(line).size(13).into(),
        // `responsive` hands over the height this pane really has, which also
        // follows a resize of the window.
        None => responsive(move |size| {
            rows_in_view(&visible, current, now_ms, offset, &activities, size.height)
        })
        .into(),
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
    activities: &BTreeMap<String, ChatActivity>,
    viewport: f32,
) -> Element<'a, Event> {
    let w = window(visible.len(), ROW_HEIGHT, offset, viewport, OVERSCAN);
    let rows = visible[w.rows.clone()].iter().map(|session| {
        row_of(
            session,
            current,
            now_ms,
            activities.get(&session.path).copied().unwrap_or_default(),
        )
    });
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

fn row_of<'a>(
    session: &'a SessionItem,
    current: Option<&str>,
    now_ms: u64,
    activity: ChatActivity,
) -> Element<'a, Event> {
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
    let badge = badge::Badge::on(activity);
    // A badge costs the title the room of a mark, and no more: the row keeps
    // its height and its one line either way.
    let room = badge.map(|_| badge::TITLE_ROOM).unwrap_or(0);
    let title = text(truncate(&session.title, TITLE_CHARS - room))
        .size(14)
        .wrapping(text::Wrapping::None)
        .font_maybe(badge.filter(|badge| badge.bold_title).map(|_| BOLD));
    let title: Element<'_, Event> = match badge {
        Some(badge) => row![
            title,
            text(badge::glyph(badge.mark))
                .size(13)
                .style(badge_colour(badge.role)),
        ]
        .spacing(6)
        .into(),
        None => title.into(),
    };
    button(column![title, text(details).size(11).wrapping(text::Wrapping::None)].spacing(2))
        .width(Length::Fill)
        .height(ROW_HEIGHT)
        .padding([6, 10])
        .style(style)
        .on_press(Event::Open(session.path.clone()))
        .into()
}

/// The theme family a badge and its mark draw in; the row's own texts stay
/// themed beside it.
fn badge_colour(role: badge::ColourRole) -> fn(&Theme) -> text::Style {
    use badge::ColourRole;
    match role {
        ColourRole::Accent => text::primary,
        ColourRole::Success => text::success,
        ColourRole::Muted => text::secondary,
        ColourRole::Warning => text::warning,
        ColourRole::Error => text::danger,
    }
}
