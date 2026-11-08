//! What the shell asks a chat for right after its child came up.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

/// The one request a chat needs once its child is live.
///
/// A child started for a stored chat has the session on its command line but
/// not its history in the window, so it sends that history; a new chat has no
/// session file yet, so it asks for its state and learns the path from the
/// reply. A chat whose child was already up has its thread already, and a start
/// that was discarded has no child at all: neither asks for anything. Neither
/// request replaces the child's session: nothing here is a `switch_session` or a
/// `new_session` (S1).
pub fn request_after_open(chat: &ChatId, how: Opened) -> Option<ChatCommand> {
    if how != Opened::Started {
        return None;
    }
    let rpc = match chat.session_path() {
        Some(_) => RpcCommand::GetMessages { id: None },
        None => RpcCommand::GetState,
    };
    Some(ChatCommand::Send {
        chat: chat.clone(),
        rpc,
    })
}
