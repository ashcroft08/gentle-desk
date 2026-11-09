//! The windowed read of the session list (S3): `offset`/`length`/`query` over
//! the list the app state holds, never the whole list over the wire.
//!
//! The matching is NOT re-implemented here. A query narrows the list by folding
//! the same intent the iced sidebar's search box folds
//! ([`Intent::SetSessionQuery`]) on a COPY of the state, and then reading
//! [`AppState::visible_sessions`], the rule that decides which saved chats the
//! sidebar shows. The copy is thrown away, so answering a window never changes
//! the query the state holds.
//!
//! The window is the plain slice of that filtered list: a home of 50 000 chats
//! costs one filter plus the rows the sidebar really draws, and the length is
//! capped at [`MAX_WINDOW`] so no caller can ask for the whole list back.

use gentle_app::{update_chats, AppState, Input, Intent};

use crate::contract::SessionsWindow;
use crate::sessions::view;

/// The most rows one window answers with. A window is a page the sidebar draws,
/// not a way to ship the whole list: a longer length is cut to this.
pub const MAX_WINDOW: u32 = 200;

/// The rows `[offset, offset + length)` of the list `query` narrows, and how
/// many rows that query matches in total. Ages are measured at `now_ms`.
pub fn window(
    state: &AppState,
    query: &str,
    offset: u32,
    length: u32,
    now_ms: u64,
) -> SessionsWindow {
    let filtered = narrowed(state, query);
    let visible = filtered.visible_sessions();
    let total = view::narrow(visible.len());
    let length = length.min(MAX_WINDOW) as usize;
    let rows = visible
        .into_iter()
        .skip(offset as usize)
        .take(length)
        .map(|session| view::row(session, now_ms))
        .collect();
    SessionsWindow { total, rows }
}

/// The list `query` narrows, decided by the app: the intent the search box
/// folds, on a copy, so the live state keeps what the user typed.
fn narrowed(state: &AppState, query: &str) -> AppState {
    let input = Input::Intent(Intent::SetSessionQuery(query.to_owned()));
    update_chats(state.clone(), input).0
}
