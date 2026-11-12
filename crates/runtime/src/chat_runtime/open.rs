//! What the shell asks a chat for right after its child came up.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

/// The requests a chat needs once its child is live, in order.
///
/// A child started for a stored chat has the session on its command line but
/// not its history in the window, so it sends that history; a new chat has no
/// session file yet, so it asks for its state and learns the path from the
/// reply. Either way it also asks the two questions the model chip needs:
/// the session's model and thinking level (`get_state`), and the models the
/// child offers (`get_available_models`). A chat whose child was already up
/// has its thread already, one whose child is still starting is served by the
/// start already on its way, and a start that was discarded has no child at
/// all: none of them asks for anything. Nothing here is a `switch_session`
/// or a `new_session` (S1).
pub fn request_after_open(chat: &ChatId, how: Opened) -> Vec<ChatCommand> {
    if how != Opened::Started {
        return Vec::new();
    }
    let ask = |rpc| ChatCommand::Send {
        chat: chat.clone(),
        rpc,
    };
    match chat.session_path() {
        Some(_) => vec![
            ask(RpcCommand::GetMessages { id: None }),
            ask(RpcCommand::GetState),
            ask(RpcCommand::GetAvailableModels),
        ],
        None => vec![
            ask(RpcCommand::GetState),
            ask(RpcCommand::GetAvailableModels),
        ],
    }
}
