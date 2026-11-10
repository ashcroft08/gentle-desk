//! What the session list slice of a `ViewState` carries: how complete the list
//! is, how many sessions it holds, and how one row reads.
//!
//! The rows themselves are NOT part of the push: they are served one window at
//! a time by [`window`](crate::sessions::window) (S3), so a 50 000-chat home
//! never ships its list to the window. The row's texts come from the rules
//! `crates/runtime` already owns ([`time_ago`], [`truncate`]), never from a
//! second implementation here.

use gentle_app::{AppState, ChatId, ListStatus, SessionItem};
use gentle_runtime::text::truncate;
use gentle_runtime::time_ago::time_ago;

use crate::contract::{ListStatusView, SessionRowView};

/// Longest title a row shows, in characters, as the iced sidebar draws it.
const TITLE_CHARS: usize = 32;

/// What the header calls a chat whose session file does not exist yet: a chat
/// started a moment ago has no stored title to draw, and a blank header would
/// read as no chat at all. The words are the shell's own (the iced header's
/// button said the same), never the session's.
pub const NEW_CHAT_TITLE: &str = "New chat";

/// How complete the list is, as the contract says it. The counters narrow to
/// `u32`: a home has files, not billions of them, and a saturating value would
/// name the same list, never another one.
pub fn status(state: &AppState) -> ListStatusView {
    match state.list_status {
        ListStatus::Loading { done, total } => ListStatusView::Loading {
            done: narrow(done),
            total: narrow(total),
        },
        ListStatus::Ready => ListStatusView::Complete,
    }
}

/// How many sessions the state holds, whatever the search box says: the count
/// the sidebar's header shows, and the one that tells an empty home from a
/// query that matches nothing.
pub fn total(state: &AppState) -> u32 {
    narrow(state.sessions.len())
}

/// One row of the list, flattened for display, with its age as seen at
/// `now_ms`. The one mapping of a session to a row: the window that serves the
/// rows uses it, so every row of every page reads the same way.
pub fn row(session: &SessionItem, now_ms: u64) -> SessionRowView {
    SessionRowView {
        id: session.path.clone(),
        title: truncate(&session.title, TITLE_CHARS),
        project: session.project().to_owned(),
        message_count: session.message_count,
        modified_ago: time_ago(now_ms, session.modified_ms),
    }
}

/// The title the chat header draws for `chat`: the stored session's own title,
/// which is the text a sidebar row draws, untruncated here because the header cuts
/// it for its own width and keeps the whole name for whoever asks. A chat whose
/// session file does not exist yet reads [`NEW_CHAT_TITLE`]. The lookup goes
/// through the same list the rows come from, so the header and the row can never
/// name the same chat differently.
pub fn header_title(state: &AppState, chat: &ChatId) -> String {
    chat.session_path()
        .and_then(|path| state.sessions.iter().find(|session| session.path == path))
        .map_or_else(
            || NEW_CHAT_TITLE.to_owned(),
            |session| session.title.clone(),
        )
}

/// A list length narrows without ever aliasing: a count past `u32::MAX` reads
/// as the largest one, which no home reaches and no other list shares.
pub(crate) fn narrow(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}
