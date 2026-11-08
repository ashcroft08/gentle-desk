//! What one chat is doing, as the sidebar shows it (S4).
//!
//! Derived from the chat's own state and from whether the user is looking at it.
//! Nothing here sends a command, starts a process or reads a clock.

use gentle_core::chat::ChatState;

/// The state of one chat in the list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChatActivity {
    /// Nothing running and nothing waiting to be read.
    #[default]
    Idle,
    /// Running a model call, in the foreground or in the background.
    Working,
    /// A turn ended while the user was on another chat.
    FinishedUnread,
    /// An extension dialog is waiting for an answer.
    NeedsAttention,
}

/// The activity of one chat, given its state, whether a finished turn is still
/// unread and whether it is the foreground chat.
///
/// A waiting dialog wins over everything: it is the one thing the user has to
/// act on, and it is lost if a background chat's dialog is not surfaced. A turn
/// that ends elsewhere stays [`ChatActivity::FinishedUnread`] until the chat is
/// opened; the chat the user is on never shows it.
pub(crate) fn activity_of(state: &ChatState, unread: bool, foreground: bool) -> ChatActivity {
    if !state.pending_dialogs.is_empty() {
        ChatActivity::NeedsAttention
    } else if state.working {
        ChatActivity::Working
    } else if !foreground && unread {
        ChatActivity::FinishedUnread
    } else {
        ChatActivity::Idle
    }
}
