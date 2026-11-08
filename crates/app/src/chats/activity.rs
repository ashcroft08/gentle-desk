//! What one chat is doing, as the sidebar shows it (S4).
//!
//! Derived from the chat's own state, whether the user is looking at it, and
//! whether its child is still there. Nothing here sends a command, starts a
//! process or reads a clock.

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
    /// The chat holds a prompt of its own waiting for a working slot (S7).
    Queued,
    /// A child for the chat was asked for and is not up yet (a start takes
    /// seconds): the click is seen and nothing can be sent there yet.
    Opening,
    /// An extension dialog is waiting for an answer.
    NeedsAttention,
    /// The chat's child is gone: nothing runs here until it is restarted (S3).
    Ended,
}

/// The activity of one chat, given its state, whether a finished turn is still
/// unread, whether it is the foreground chat, whether its child is gone,
/// whether its own prompt is waiting for a slot (S7), and whether a child is
/// still being started for it.
///
/// The order is: a dead child wins over everything (a pending dialog cannot be
/// answered and a turn cannot finish once there is no process to talk to); then
/// a waiting dialog, the one thing the user has to act on; then a running
/// turn, which beats its own queued prompt (the slot was taken); then the
/// opening phase, which is the chat between a click and a live child; then a
/// turn that ended elsewhere, which stays [`ChatActivity::FinishedUnread`] until
/// the chat is opened — the chat the user is on never shows it.
pub(crate) fn activity_of(
    state: &ChatState,
    unread: bool,
    foreground: bool,
    ended: bool,
    queued: bool,
    opening: bool,
) -> ChatActivity {
    if ended {
        ChatActivity::Ended
    } else if !state.pending_dialogs.is_empty() {
        ChatActivity::NeedsAttention
    } else if state.working {
        ChatActivity::Working
    } else if queued {
        ChatActivity::Queued
    } else if opening {
        ChatActivity::Opening
    } else if !foreground && unread {
        ChatActivity::FinishedUnread
    } else {
        ChatActivity::Idle
    }
}
