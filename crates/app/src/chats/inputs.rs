//! Inputs about one chat that are not events from its child: a command that did
//! not reach it, and the session file a new chat learned.

use crate::chats::ChatId;
use crate::notices::push_notice;
use crate::state::{AppState, NoticeLevel};

/// A per-chat command failed: an error notice naming the chat, and nothing else
/// changes. The chat keeps its thread and its place, and the other children are
/// untouched.
pub(crate) fn failed(mut state: AppState, chat: &ChatId, message: &str) -> AppState {
    push_notice(
        &mut state,
        NoticeLevel::Error,
        format!("{}: {message}", chat.label()),
    );
    state
}

/// A new chat became the stored chat its child reported: its state, its
/// foreground mark and a finished turn nobody has read all move with it.
pub(crate) fn rekeyed(mut state: AppState, from: &ChatId, to: ChatId) -> AppState {
    state.chats.rekey(from, to);
    state
}
