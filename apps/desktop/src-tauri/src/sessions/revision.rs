//! When the page of rows the web side holds goes stale (S2, S3).
//!
//! The session list is not pushed any more: the sidebar reads a window of it
//! (`sessions_window`) and re-reads when the slice says the list moved. That
//! answer is a revision, a wrapping counter the holder bumps whenever the folded
//! list or its status changes, and this module is the rule that decides it.
//!
//! The comparison is kept as cheap as the reducer allows: only the three session
//! inputs can move the list at all, and only they pay for comparing the items of
//! a 50 000-chat list. Everything else is answered by the status and the size.

use gentle_app::{AppState, Input};

/// Whether an input can change the folded list or its status. Only these three
/// reach `set_prepared` / `set_progress` / `set_ready` in `crates/app` (see its
/// `lib.rs` fold); every other input leaves both exactly as they were, which is
/// what lets the revision check skip the full comparison of a big list.
pub fn folds_the_list(input: &Input) -> bool {
    matches!(
        input,
        Input::SessionsPrepared(_) | Input::SessionsProgress { .. } | Input::SessionsLoaded(_)
    )
}

/// Whether the fold moved the session list: the status, the size, or - only for
/// an input that can swap the items in place (`full`) - the items themselves.
pub fn list_moved(before: &AppState, after: &AppState, full: bool) -> bool {
    if before.list_status != after.list_status || before.sessions.len() != after.sessions.len() {
        return true;
    }
    full && before.sessions != after.sessions
}
