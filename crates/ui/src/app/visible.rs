//! Which chat the window shows, and what it may do with it.
//!
//! One place answers "the chat on screen" so the frame, the composer and the
//! cache all agree, and one place decides whether that chat can be talked to.

use gentle_app::AppState;
use gentle_core::chat::ChatState;

/// The chat the window shows: the foreground one of the per-chat shell, or the
/// empty single thread before the first chat is opened.
pub(super) fn chat(app: &AppState) -> &ChatState {
    app.chats
        .foreground()
        .and_then(|chat| app.chats.state(chat))
        .unwrap_or(&app.chat)
}

/// Whether the chat the window shows has lost its child (S3). Only that chat is
/// disabled: every other chat keeps its own child and stays usable.
pub(super) fn ended(app: &AppState) -> bool {
    app.chats
        .foreground()
        .is_some_and(|chat| app.chats.is_ended(chat))
}

/// Whether the chat the window shows holds a queued prompt, waiting for its
/// working turn (S7). Like a working chat, it takes no second prompt.
pub(super) fn queued(app: &AppState) -> bool {
    app.chats
        .foreground()
        .is_some_and(|chat| app.chats.is_queued(chat))
}

/// Whether the composer may send that text right now.
///
/// Disabled for a failure that ends everything (gentle-shell could not be
/// resolved), for a chat whose child died: there is nowhere for the prompt to
/// go, and the notice beside it carries the Restart, and for a chat that
/// already has a turn running or queued (S7). Nothing else disables it, so
/// one death never takes the other chats with it.
pub(super) fn can_send(app: &AppState, input: &str, fatal: bool) -> bool {
    !input.trim().is_empty() && !fatal && !ended(app) && !chat_busy(app)
}

/// Whether the chat on screen cannot take a prompt: it works or it queues.
pub(super) fn chat_busy(app: &AppState) -> bool {
    chat(app).working || queued(app)
}

/// The session file the sidebar marks as open.
pub(super) fn current_session(app: &AppState) -> Option<&str> {
    app.chats
        .foreground()
        .and_then(|chat| chat.session_path())
        .or(app.current_session.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gentle_app::{update_chats, ChatId, Input, Intent};
    use gentle_core::rpc::RpcEvent;

    /// Two chats at the working cap: the prompt the user sends while the
    /// second one waits makes it Queued, and a queued chat sends no more.
    fn state_with_a_queued_chat() -> AppState {
        let (state, _) = update_chats(
            AppState::default(),
            Input::Intent(Intent::OpenChat(ChatId::of_session("a"))),
        );
        let (state, _) = update_chats(
            state,
            Input::ChatEvent {
                chat: ChatId::of_session("a"),
                event: RpcEvent::AgentStart,
            },
        );
        let (state, _) = update_chats(state, Input::SetWorkingCap(1));
        let (state, _) = update_chats(
            state,
            Input::Intent(Intent::OpenChat(ChatId::of_session("b"))),
        );
        let (state, _) = update_chats(state, Input::Intent(Intent::SendPrompt("hola".into())));
        state
    }

    #[test]
    fn a_queued_chat_cannot_send_until_its_turn_starts() {
        let state = state_with_a_queued_chat();

        assert!(queued(&state), "b holds a queued prompt");
        assert!(!can_send(&state, "más", false), "queued is like working");
    }
}
