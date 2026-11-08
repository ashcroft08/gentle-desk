//! The intents that belong to one chat rather than to the single open thread.
//!
//! These are additive: they exist next to the single-chat intents and only act
//! when the intent names a chat (or, for `SendPrompt`, when there is a foreground
//! chat). The one-child shell never produces them, so nothing it does changes.

use gentle_core::rpc::RpcCommand;

use super::ChatId;
use crate::input::{ChatCommand, Intent};
use crate::state::{push_user_message, AppState};

/// Applies `intent` if it belongs to the per-chat model, answering the effects
/// for one chat's child. Answers `None` when it is the single-chat intent.
pub(crate) fn apply(state: &mut AppState, intent: &Intent) -> Option<Vec<ChatCommand>> {
    match intent {
        Intent::OpenChat(chat) => Some(open(state, chat.clone())),
        Intent::OpenNewChat => Some(open(state, ChatId::fresh())),
        Intent::SendPrompt(text) => send_prompt(state, text),
        _ => None,
    }
}

/// Makes `chat` the foreground one and asks the host for its child. A chat that
/// already has a live child is only focused by the host, so this never sends
/// `switch_session` or `new_session`: the session travels on the command line.
fn open(state: &mut AppState, chat: ChatId) -> Vec<ChatCommand> {
    state.chats.open(chat.clone());
    vec![ChatCommand::OpenProcess { chat }]
}

/// Sends the prompt to the foreground chat, one turn at a time, as the
/// single-chat path does. Answers `None` with no foreground chat, so the
/// single-chat path keeps the prompt.
fn send_prompt(state: &mut AppState, text: &str) -> Option<Vec<ChatCommand>> {
    let chat = state.chats.foreground()?.clone();
    if text.trim().is_empty() {
        return Some(Vec::new());
    }
    let chat_state = state.chats.chat_mut(&chat);
    if chat_state.working {
        return Some(Vec::new());
    }
    push_user_message(chat_state, text);
    Some(vec![ChatCommand::Send {
        chat,
        rpc: RpcCommand::Prompt {
            message: text.to_owned(),
        },
    }])
}
