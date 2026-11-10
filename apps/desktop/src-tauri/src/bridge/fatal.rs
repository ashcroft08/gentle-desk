//! The one fatal failure of a run: the `gentle-shell` launcher is not there.
//!
//! The app state knows no launcher (`crates/app` carries no process at all, S2),
//! so this slice is the SHELL's: the executor resolves the launcher on the first
//! command that needs a child, and when that resolution fails every chat is
//! impossible, which is the iced shell's global banner
//! (`crates/ui/src/view/banner.rs`), never a per-chat notice alone.
//!
//! It is set once and never cleared: a launcher that could not be resolved at
//! the first command cannot appear later, because the failing resolution is kept
//! in the executor (`effects::runtime`) and every later command answers the same
//! way.

use super::{lock, AppStateHolder};
use crate::contract::ViewState;

impl AppStateHolder {
    /// Records the run's fatal reason and answers the slice to push. The
    /// revision does not move: no list changed.
    pub fn note_fatal(&self, message: String) -> ViewState {
        *lock(&self.fatal) = Some(message);
        self.view()
    }

    /// The fatal reason as the view state carries it.
    pub(super) fn fatal(&self) -> Option<String> {
        lock(&self.fatal).clone()
    }
}
