//! The host's working mark is fed by the reducer (S1, S2).
//!
//! The host has the protection rules — a chat that is working is never the
//! victim chosen to make room at the live cap and is never reaped as idle — but
//! rules over a mark nothing sets protect nobody. The reducer is the one that
//! reads `agent_start` / `agent_end`, so the reducer is the one that tells the
//! host, through the same command seam every other side effect takes.
//!
//! These tests pin the seam, not the rules: the rules themselves are pinned in
//! `gentle-backend`'s `chat_host::policy`.

use gentle_app::{update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent};
use gentle_core::rpc::RpcEvent;

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    let (state, _) = update_chats(state, Input::Intent(Intent::OpenChat(chat.clone())));
    update_chats(state, Input::ChatOpened { chat: chat.clone() })
}

fn event(state: AppState, chat: &ChatId, event: RpcEvent) -> (AppState, Effects) {
    update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event,
        },
    )
}

/// The working marks an effects list asks for, in order. Every other command is
/// dropped: what this file is about is the mark.
fn marks(effects: &Effects) -> Vec<(ChatId, bool)> {
    effects
        .chat_commands
        .iter()
        .filter_map(|command| match command {
            ChatCommand::SetWorking { chat, working } => Some((chat.clone(), *working)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_turn_starting_tells_the_host_the_chat_is_working() {
    let (state, _) = open(AppState::default(), &chat("a"));

    let (_state, effects) = event(state, &chat("a"), RpcEvent::AgentStart);

    assert_eq!(marks(&effects), vec![(chat("a"), true)]);
}

#[test]
fn a_turn_ending_tells_the_host_the_chat_stopped_working() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = event(state, &chat("a"), RpcEvent::AgentStart);

    let (_state, effects) = event(state, &chat("a"), RpcEvent::AgentEnd { will_retry: None });

    assert_eq!(marks(&effects), vec![(chat("a"), false)]);
}

#[test]
fn an_event_that_changes_no_mark_says_nothing() {
    let (state, _) = open(AppState::default(), &chat("a"));

    // No turn was running, so this end frees nothing and changes no mark.
    let (_state, effects) = event(state, &chat("a"), RpcEvent::AgentEnd { will_retry: None });

    assert_eq!(marks(&effects), Vec::new());
}

#[test]
fn a_settled_turn_is_reported_as_stopped_once() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = event(state, &chat("a"), RpcEvent::AgentStart);

    let (_state, effects) = event(state, &chat("a"), RpcEvent::AgentSettled);

    assert_eq!(marks(&effects), vec![(chat("a"), false)]);
}
