//! Nothing the user does to navigate aborts a chat that is mid-turn (S1, S6).
//!
//! These are the guarantees the reported failure broke: the user was working in
//! a chat, opened an old one to try it, came back, and found the turn dead
//! (`"Operation aborted"`) and the `working` mark gone. The navigation rules
//! themselves live in `crates/app/src/chats/store.rs` and `intents.rs`; what
//! these tests pin is that no path the user takes to another chat — or back —
//! ever asks for the working chat's child to be stopped.

use gentle_app::{update_chats, AppState, ChatActivity, ChatCommand, ChatId, Effects, Input, Intent};
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

/// A chat running a turn, which is the state the user's work was in.
fn working_chat() -> AppState {
    let (state, _) = open(AppState::default(), &chat("work"));
    let (state, _) = event(state, &chat("work"), RpcEvent::AgentStart);
    state
}

fn stopped(effects: &Effects) -> Vec<&ChatCommand> {
    effects
        .chat_commands
        .iter()
        .filter(|command| matches!(command, ChatCommand::CloseProcess { .. }))
        .collect()
}

#[test]
fn opening_another_chat_stops_nothing() {
    let state = working_chat();

    let (_state, effects) = update_chats(state, Input::Intent(Intent::OpenChat(chat("old"))));

    assert_eq!(
        stopped(&effects),
        Vec::<&ChatCommand>::new(),
        "opening a chat starts its own child and never stops the working one"
    );
}

#[test]
fn switching_back_to_a_working_chat_stops_nothing() {
    let state = working_chat();
    let (state, _) = open(state, &chat("old"));

    let (state, effects) = update_chats(state, Input::Intent(Intent::OpenChat(chat("work"))));

    assert_eq!(stopped(&effects), Vec::<&ChatCommand>::new());
    assert_eq!(state.chats.foreground(), Some(&chat("work")));
}

#[test]
fn closing_the_chat_the_user_left_stops_the_other_one_only() {
    let state = working_chat();
    let (state, _) = open(state, &chat("old"));

    let (_state, effects) = update_chats(state, Input::Intent(Intent::CloseChat(chat("old"))));

    assert_eq!(
        stopped(&effects),
        vec![&ChatCommand::CloseProcess { chat: chat("old") }],
        "the close names the chat it was asked for and no other"
    );
}

#[test]
fn closing_a_working_chat_is_refused() {
    let state = working_chat();

    let (state, effects) = update_chats(state, Input::Intent(Intent::CloseChat(chat("work"))));

    assert_eq!(
        stopped(&effects),
        Vec::<&ChatCommand>::new(),
        "a working chat's child is never stopped by a close"
    );
    assert_eq!(state.chats.foreground(), Some(&chat("work")));
}

#[test]
fn a_turn_that_ends_while_the_user_is_elsewhere_keeps_its_reply() {
    let state = working_chat();
    let (state, _) = open(state, &chat("old"));

    let (state, _) = event(state, &chat("work"), RpcEvent::AgentEnd { will_retry: None });

    assert_eq!(
        state.chats.activity(&chat("work")),
        Some(ChatActivity::FinishedUnread),
        "the background chat is marked unread instead of being stopped"
    );
}
