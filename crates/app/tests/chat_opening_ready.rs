//! A chat whose child is *starting* stays Opening until the child shows life:
//! the start of a fresh child takes seconds (a real gentle-shell answers after
//! ~3 s), so `ChatOpened` is not the moment it is up. The chat's own first
//! decoded event clears the phase; the 20 s net (`ChatOpened` again) is there
//! for a child that starts but never speaks, and is idempotent.

use gentle_app::{update_chats, AppState, ChatActivity, ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::RpcEvent;

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, name: &str) -> AppState {
    update_chats(state, Input::Intent(Intent::OpenChat(chat(name)))).0
}

fn event(state: AppState, name: &str, event: RpcEvent) -> AppState {
    update_chats(
        state,
        Input::ChatEventAt {
            chat: chat(name),
            event,
            now_ms: 7,
        },
    )
    .0
}

fn opening_confirmed(state: AppState, name: &str) -> AppState {
    update_chats(state, Input::ChatOpened { chat: chat(name) }).0
}

#[test]
fn a_started_child_that_is_not_speaking_yet_leaves_the_chat_opening() {
    // Real children print on stderr while they come up (warnings, setup
    // reminders). Stderr is not life: no first event has arrived, so no
    // confirmation clears the phase and the chat stays Opening.
    let state = open(AppState::default(), "a");
    let state = update_chats(
        state,
        Input::ChatStderr {
            chat: chat("a"),
            line: "already has content and was not set up ...".into(),
        },
    )
    .0;

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "a stderr line while starting is not the child answering"
    );
}

#[test]
fn the_first_own_event_of_a_starting_chat_ends_the_opening_phase() {
    let state = open(AppState::default(), "a");
    // No `ChatOpened` arrived: the host confirmed the start milliseconds after
    // the click, while the child itself takes seconds. Its first event is what
    // proves it is alive.
    let state = event(state, "a", RpcEvent::AgentSettled);

    assert!(
        !state.chats.is_opening(&chat("a")),
        "the chat speaks, so it is not opening any more"
    );
    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}

#[test]
fn a_prompt_is_refused_while_opening_and_accepted_after_the_first_event() {
    let state = open(AppState::default(), "a");

    let (state, effects) = update_chats(state, Input::Intent(Intent::SendPrompt("hola".into())));
    assert_eq!(
        effects,
        gentle_app::Effects::default(),
        "nothing to write to"
    );
    assert!(
        state
            .chats
            .state(&chat("a"))
            .expect("chat a")
            .messages
            .is_empty(),
        "the prompt is not appended while the child is coming up"
    );

    let state = event(state, "a", RpcEvent::AgentSettled);
    let (state, effects) = update_chats(state, Input::Intent(Intent::SendPrompt("hola".into())));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::Send {
            chat: chat("a"),
            rpc: gentle_core::rpc::RpcCommand::Prompt {
                message: "hola".into()
            }
        }],
        "the first event made the chat answerable"
    );
    assert_eq!(
        state
            .chats
            .state(&chat("a"))
            .expect("chat a")
            .messages
            .len(),
        1
    );
}

#[test]
fn the_twenty_second_confirmation_clears_the_phase_and_changes_nothing_after() {
    // The safety net the UI runs 20 s after a started open sends the same
    // `ChatOpened`. It clears a phase that no event has cleared yet, and a
    // second one (or one after the first event) changes nothing.
    let state = open(AppState::default(), "a");

    let state = opening_confirmed(state, "a");
    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));

    let settled = opening_confirmed(state.clone(), "a");
    assert_eq!(
        settled, state,
        "clearing an already-clear chat changes nothing"
    );
    let state = event(settled, "a", RpcEvent::AgentSettled);
    let settled = opening_confirmed(state.clone(), "a");
    assert_eq!(
        settled, state,
        "the net is idempotent once the chat has answered too"
    );
}

#[test]
fn a_chat_confirmed_without_waits_is_not_stuck_opening() {
    // A focus-only open (`Existing`) or a discarded start is confirmed at once
    // by the UI, with no event needed: the child has been alive all along (or
    // never was the one to speak).
    let state = open(AppState::default(), "a");

    let state = opening_confirmed(state, "a");

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}
