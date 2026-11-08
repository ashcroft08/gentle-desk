//! Inputs about one chat that are not events from its child: a command that did
//! not reach it, the session file a new chat learned, and its child going away.

use crate::chats::ChatId;
use crate::notices::{push_action_notice, push_notice};
use crate::state::{AppState, NoticeAction, NoticeLevel};

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

/// One chat's child is gone.
///
/// An expected stop — the host reaped it, made room for another chat, closed it
/// or shut down — leaves the chat just idle and says nothing: its next open
/// starts a fresh child through `--session`. An unexpected death of any chat,
/// foreground or background, ends that chat and leaves one notice naming it,
/// with the Restart that brings it back (S3). Nothing here is fatal for the
/// window: every other chat keeps its child.
pub(crate) fn closed(mut state: AppState, chat: &ChatId, expected: bool) -> AppState {
    if expected {
        state.chats.clear_ended(chat);
    } else {
        state.chats.ended_now(chat);
        push_death_notice(&mut state, chat);
    }
    state
}

/// The one notice a death leaves. Extensions repeat themselves, so the notice is
/// deduplicated by text: a chat that dies twice shows one row, and that row still
/// carries the Restart.
pub(crate) fn push_death_notice(state: &mut AppState, chat: &ChatId) {
    push_action_notice(
        state,
        NoticeLevel::Error,
        format!(
            "{}: the child that runs this chat exited. Restart to keep talking.",
            chat.label()
        ),
        NoticeAction::RestartChat(chat.clone()),
    );
}
