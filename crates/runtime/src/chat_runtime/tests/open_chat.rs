//! What a chat asks for right after its child came up: its history if it is
//! stored, its state if it is new, and nothing at all otherwise.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

use crate::chat_runtime::request_after_open;

#[test]
fn a_stored_chat_that_just_started_asks_for_its_history() {
    let chat = ChatId::of_session("p.jsonl");

    assert_eq!(
        request_after_open(&chat, Opened::Started),
        Some(ChatCommand::Send {
            chat,
            rpc: RpcCommand::GetMessages { id: None },
        })
    );
}

#[test]
fn a_new_chat_that_just_started_asks_for_its_state() {
    let chat = ChatId::fresh();

    assert_eq!(
        request_after_open(&chat, Opened::Started),
        Some(ChatCommand::Send {
            chat,
            rpc: RpcCommand::GetState,
        })
    );
}

#[test]
fn a_chat_that_was_already_up_still_starting_or_discarded_asks_for_nothing() {
    let chat = ChatId::of_session("p.jsonl");

    assert_eq!(request_after_open(&chat, Opened::Existing), None);
    // Still starting: the child that comes up is the one the first click asked
    // for, and it will answer `get_messages` when it is this chat's turn.
    assert_eq!(request_after_open(&chat, Opened::Starting), None);
    assert_eq!(request_after_open(&chat, Opened::Discarded), None);
}
