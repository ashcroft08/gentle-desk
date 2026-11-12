//! What a chat asks for right after its child came up: its history if it is
//! stored, its state if it is new, nothing at all otherwise — and, once the
//! child is live, the model questions the chip needs (0.1.11).

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

use crate::chat_runtime::request_after_open;

/// The follow-up requests a chat asks, in order.
#[test]
fn a_stored_chat_that_just_started_asks_its_history_state_and_models() {
    let chat = ChatId::of_session("p.jsonl");

    assert_eq!(
        request_after_open(&chat, Opened::Started),
        vec![
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetMessages { id: None },
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableModels,
            },
        ]
    );
}

#[test]
fn a_new_chat_that_just_started_asks_its_state_and_models() {
    let chat = ChatId::fresh();

    assert_eq!(
        request_after_open(&chat, Opened::Started),
        vec![
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableModels,
            },
        ]
    );
}

#[test]
fn a_chat_that_was_already_up_still_starting_or_discarded_asks_for_nothing() {
    let chat = ChatId::of_session("p.jsonl");

    assert!(request_after_open(&chat, Opened::Existing).is_empty());
    // Still starting: the child that comes up is the one the first click asked
    // for, and it will answer `get_messages` when it is this chat's turn.
    assert!(request_after_open(&chat, Opened::Starting).is_empty());
    assert!(request_after_open(&chat, Opened::Discarded).is_empty());
}
