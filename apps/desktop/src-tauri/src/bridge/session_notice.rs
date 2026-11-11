//! What the last session action refused (T2): the rename of a chat.
//!
//! The reducer's notices belong to chats and events (`crates/app`), and a refused
//! rename is neither: the chat is fine, the user simply asked for something the
//! file cannot take while its child is mid-turn, or typed no name, or a write the
//! disk took back. So the line is SHELL-owned, like the workspace notice beside
//! it (`bridge::notice`), and travels as `ViewState.sessions_notice`: one plain
//! line, cleared by the next session action that works.
//!
//! It is a line and not an error the caller has to catch: a rename that is refused
//! still answers the current slice, so the sidebar never keeps a title it did not
//! get from the state it is drawing.

use super::{lock, AppStateHolder};
use crate::contract::ViewState;

impl AppStateHolder {
    /// Records `message` as the session notice and answers the slice to push. The
    /// revision does not move: no list changed.
    pub fn note_sessions_problem(&self, message: String) -> ViewState {
        *lock(&self.sessions_notice) = Some(message);
        self.view()
    }

    /// Clears the session notice: the action that just worked has nothing to
    /// report.
    pub fn clear_sessions_notice(&self) {
        *lock(&self.sessions_notice) = None;
    }

    /// The session notice as the view state carries it.
    pub(super) fn sessions_notice(&self) -> Option<String> {
        lock(&self.sessions_notice).clone()
    }
}
