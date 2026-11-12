//! What the shell asks a chat for right after its child came up.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

/// The requests a chat needs once its child is live, in order.
///
/// A child started for a stored chat has the session on its command line but
/// not its history in the window, so it sends that history; a new chat has no
/// session file yet, so it asks for its state and learns the path from the
/// reply. A chat whose child was already up has its thread already, but the
/// chip's data was cleared when it left the foreground, so it asks the state
/// and both lists again. Either way the model chip's questions go out: the
/// session's model and thinking level (`get_state`), the models the child
/// offers (`get_available_models`), and the levels the session's model
/// supports (`get_available_thinking_levels`). One whose child is still
/// starting is served by the start already on its way, and a start that was
/// discarded has no child at all: none of them asks for anything. Nothing
/// here is a `switch_session` or a `new_session` (S1).
pub fn request_after_open(chat: &ChatId, how: Opened) -> Vec<ChatCommand> {
    let ask = |rpc| ChatCommand::Send {
        chat: chat.clone(),
        rpc,
    };
    let chip = || {
        vec![
            ask(RpcCommand::GetState),
            ask(RpcCommand::GetAvailableModels),
            ask(RpcCommand::GetAvailableThinkingLevels),
        ]
    };
    match how {
        Opened::Started => match chat.session_path() {
            Some(_) => {
                let mut asks = vec![ask(RpcCommand::GetMessages { id: None })];
                asks.extend(chip());
                asks
            }
            None => chip(),
        },
        // Already live: the thread is on screen, the chip is not.
        Opened::Existing => chip(),
        // Still starting: the start already on its way asks everything.
        Opened::Starting | Opened::Discarded => Vec::new(),
    }
}
