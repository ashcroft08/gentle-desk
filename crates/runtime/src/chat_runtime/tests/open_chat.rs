//! What a chat asks for right after its child came up: its history if it is
//! stored, its state if it is new, nothing at all otherwise — and, whichever
//! way the child came up, the questions the model chip needs (0.1.11), so a
//! chat that is already running answers them again instead of leaving the
//! chip on a loading line forever.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

use crate::chat_runtime::request_after_open;

/// The follow-up requests a chat asks, in order.
#[test]
fn a_stored_chat_that_just_started_asks_its_history_state_and_both_lists() {
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
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableModels,
            },
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableThinkingLevels,
            },
        ]
    );
}

#[test]
fn a_new_chat_that_just_started_asks_its_state_and_both_lists() {
    let chat = ChatId::fresh();

    assert_eq!(
        request_after_open(&chat, Opened::Started),
        vec![
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableModels,
            },
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableThinkingLevels,
            },
        ]
    );
}

/// A chat whose child was already up has its thread on screen, but its chip
/// was cleared when it left the foreground: the state and both lists are asked
/// again (R3-003, R4-chip-stuck).
#[test]
fn an_already_running_chat_still_asks_its_state_and_both_lists() {
    let chat = ChatId::of_session("p.jsonl");

    assert_eq!(
        request_after_open(&chat, Opened::Existing),
        vec![
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableModels,
            },
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableThinkingLevels,
            },
        ]
    );
}

#[test]
fn a_still_starting_or_discarded_chat_asks_for_nothing() {
    let chat = ChatId::of_session("p.jsonl");

    // Still starting: the child that comes up is the one the first click asked
    // for, and it will answer `get_messages` when it is this chat's turn.
    assert!(request_after_open(&chat, Opened::Starting).is_empty());
    assert!(request_after_open(&chat, Opened::Discarded).is_empty());
}
