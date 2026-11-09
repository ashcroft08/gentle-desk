//! The chat-content slice of the contract, read from the app state.
//!
//! Only what a child's events change and what the state really keeps: the
//! foreground chat's thread. A chat's stderr lines stay the state's private
//! ring and reach the view only as the reason of a death notice; dialogs,
//! helpers and the session list are other slices.

use gentle_app::AppState;
use gentle_core::chat::ChatRole;

use crate::contract::{MessageRole, MessageView};

/// How many of the foreground chat's messages the push carries.
///
/// One message's text is the model's whole markdown reply, so a long session
/// would push megabytes on every event; the thread renders the tail, and the
/// older messages of a reopened session arrive with its history (a later
/// slice), not with every push.
pub const MESSAGES_PUSHED: usize = 50;

/// The foreground chat's messages, oldest first, cut to the last
/// [`MESSAGES_PUSHED`]. No foreground chat is no thread.
pub(crate) fn messages(state: &AppState) -> Vec<MessageView> {
    let Some(thread) = state
        .chats
        .foreground()
        .and_then(|chat| state.chats.state(chat))
    else {
        return Vec::new();
    };
    let start = thread.messages.len().saturating_sub(MESSAGES_PUSHED);
    thread.messages[start..]
        .iter()
        .map(|message| MessageView {
            role: view_role(message.role),
            text: message.text.clone(),
        })
        .collect()
}

/// The contract's role enum matches the chat state's roles one to one.
fn view_role(role: ChatRole) -> MessageRole {
    match role {
        ChatRole::User => MessageRole::User,
        ChatRole::Assistant => MessageRole::Assistant,
    }
}
