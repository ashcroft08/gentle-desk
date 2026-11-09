//! The workspace notice: what the last workspace action refused.
//!
//! The reducer's notices belong to chats and events (`crates/app`), and this
//! slice may not change it, so a refused "New workspace" has no reducer path to
//! travel: a name Windows would not accept, a folder that is not there, a folder
//! that already holds files (Q6). The notice is therefore SHELL-owned and
//! travels as `ViewState.workspace_notice`, never as `config_notice`: that one
//! says what became of the config FILE, and this one waits for the user to read
//! it.
//!
//! It is set by a refused action and cleared by the next workspace action that
//! works, so a stale refusal never sits under a workspace the user just added.

use super::{lock, AppStateHolder};
use crate::contract::ViewState;

impl AppStateHolder {
    /// Records `message` as the workspace notice and answers the slice to push.
    /// The revision does not move: no list changed.
    pub fn note_workspace_problem(&self, message: String) -> ViewState {
        *lock(&self.workspace_notice) = Some(message);
        self.view()
    }

    /// Clears the workspace notice: the action that just worked has nothing to
    /// report.
    pub fn clear_workspace_notice(&self) {
        *lock(&self.workspace_notice) = None;
    }

    /// The workspace notice as the view state carries it.
    pub(super) fn workspace_notice(&self) -> Option<String> {
        lock(&self.workspace_notice).clone()
    }
}
