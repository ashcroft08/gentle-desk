//! What the session list pushes: how complete it is and a window of its rows.
//!
//! Only what the app state really holds: `ListStatus` says whether the first
//! pass is still reading, and the rows are cut to the first [`ROWS_PUSHED`].
//! The full list never leaves Rust (S3): the virtualized windowing and the
//! `offset`/`length`/`query` query belong to T4, so this is a window of the
//! CURRENT list and nothing more. The row's texts come from the rules
//! `crates/runtime` already owns ([`time_ago`], [`truncate`]), never from a
//! second implementation here.

use gentle_app::{AppState, ListStatus};
use gentle_runtime::text::truncate;
use gentle_runtime::time_ago::time_ago;

use crate::contract::{ListStatusView, SessionRowView};

/// How many rows the push carries. A 50 000-chat home would otherwise ship its
/// whole list on every push for a sidebar that can only draw what fits; the
/// virtualized list of T4 replaces this window with a query.
pub const ROWS_PUSHED: usize = 50;

/// Longest title a row shows, in characters, as the iced sidebar draws it.
const TITLE_CHARS: usize = 32;

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

/// The first [`ROWS_PUSHED`] rows of the list the state says is visible, in list
/// order, with the age of each one as seen at `now_ms`.
pub fn rows(state: &AppState, now_ms: u64) -> Vec<SessionRowView> {
    state
        .visible_sessions()
        .into_iter()
        .take(ROWS_PUSHED)
        .map(|session| SessionRowView {
            id: session.path.clone(),
            title: truncate(&session.title, TITLE_CHARS),
            project: session.project().to_owned(),
            message_count: session.message_count,
            modified_ago: time_ago(now_ms, session.modified_ms),
        })
        .collect()
}

/// The list length narrows without ever aliasing: a count past `u32::MAX` reads
/// as the largest one, which no home reaches and no other list shares.
fn narrow(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}
