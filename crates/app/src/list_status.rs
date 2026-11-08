//! Whether the sidebar's list is still coming, and what a partial list knows.
//!
//! The first full pass over the session files takes long on a big home, so it
//! reports progress. A partial list arrives through
//! [`Input::SessionsProgress`](crate::Input::SessionsProgress): it replaces the
//! list exactly like `SessionsPrepared`, but says the list is still incomplete.
//! The final `SessionsPrepared` is the only thing that settles it.

use crate::session_prepare::PreparedSessions;
use crate::state::AppState;

/// How complete the sidebar's list is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStatus {
    /// The first full pass is running: `done` of `total` session files read.
    Loading { done: usize, total: usize },
    /// The list is the whole truth: every later change is a new full pass.
    Ready,
}

/// A fresh [`AppState`] has not read a single session file yet, so it starts
/// `Loading` and the sidebar never claims "No saved chats yet" before the
/// first pass ends.
impl Default for ListStatus {
    fn default() -> Self {
        Self::Loading { done: 0, total: 0 }
    }
}

impl AppState {
    /// Takes a partial list from the running pass. Once the final list has
    /// arrived a straggler changes nothing: it is older than what is shown.
    pub(crate) fn set_progress(&mut self, prepared: PreparedSessions, done: usize, total: usize) {
        if self.list_status == ListStatus::Ready {
            return;
        }
        // Same list, same keys: only the counters move.
        self.set_prepared(prepared);
        self.list_status = ListStatus::Loading { done, total };
    }

    /// The final list settled: the sidebar may now say it is empty.
    pub(crate) fn set_ready(&mut self) {
        self.list_status = ListStatus::Ready;
    }
}
