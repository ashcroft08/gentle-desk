//! Inputs about one chat that are not events from its child: a command that did
//! not reach it, the stderr its child printed, the session file a new chat
//! learned, and its child going away.

use crate::chats::{queue, ChatId};
use crate::input::Effects;
use crate::notices::{push_action_notice, push_notice};
use crate::state::{AppState, NoticeAction, NoticeLevel};

/// A per-chat command failed: an error notice naming the chat, and nothing else
/// changes. The chat keeps its thread and its place, and the other children are
/// untouched. Whether it was a failed start or a failed send, the chat's opening
/// phase is over, so the opening mark never sticks.
pub(crate) fn failed(mut state: AppState, chat: &ChatId, message: &str) -> AppState {
    state.chats.clear_opening(chat);
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
pub(crate) fn closed(mut state: AppState, chat: &ChatId, expected: bool) -> (AppState, Effects) {
    // A working chat whose child goes away frees the slot it held, whatever
    // stopped the child: both an expected stop and a death free it.
    let freed = state.chats.stop_working(chat);
    // The opening phase was for a child that is now gone, whatever stopped it.
    state.chats.clear_opening(chat);
    if expected {
        state.chats.clear_ended(chat);
    } else {
        state.chats.ended_now(chat);
        // Its queued prompt can never be sent: the death notice already
        // told the user.
        queue::drop_of(&mut state.chats, chat);
        push_death_notice(&mut state, chat);
    }
    let chat_commands = if freed {
        queue::release(&mut state.chats)
    } else {
        Vec::new()
    };
    (
        state,
        Effects {
            commands: Vec::new(),
            chat_commands,
        },
    )
}

/// The one notice a death leaves. Extensions repeat themselves, so the notice is
/// deduplicated by text: a chat that dies twice shows one row, and that row still
/// carries the Restart. The last stderr lines the child left, if any, are the
/// reason, so the user learns why it died (a stored working directory that is
/// gone says so, for instance).
pub(crate) fn push_death_notice(state: &mut AppState, chat: &ChatId) {
    let reason = state.chats.stderr_of(chat).join(" · ");
    let text = if reason.is_empty() {
        format!(
            "{}: the child that runs this chat exited. Restart to keep talking.",
            chat.label()
        )
    } else {
        format!(
            "{}: the child that runs this chat exited ({}). Restart to keep talking.",
            chat.label(),
            reason
        )
    };
    push_action_notice(
        state,
        NoticeLevel::Error,
        text,
        NoticeAction::RestartChat(chat.clone()),
    );
}

/// Keeps one stderr line of that chat's child in its bounded ring (S3): the
/// reason a death notice reports, if the child ever dies. It is not a notice
/// while the child is alive.
pub(crate) fn stderr(mut state: AppState, chat: &ChatId, line: &str) -> AppState {
    state.chats.record_stderr(chat, line);
    state
}
