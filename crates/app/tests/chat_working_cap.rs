//! The working cap (S7).
//!
//! Under the cap a prompt is sent at once. At the cap nothing is lost and
//! nothing is refused (user decision, "Ponerlo en cola"): the user message
//! still lands in the thread, the chat shows Queued, and the prompts go out
//! FIFO, one per freed slot, the moment a working chat stops. The check and
//! the working mark happen in one reducer step, so there is no gap for two
//! chats to take the same slot (R3-working-cap-race).

use gentle_app::{
    update_chats, AppState, ChatActivity, ChatCommand, ChatId, Effects, Input, Intent,
};
use gentle_core::rpc::{RpcCommand, RpcEvent};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::OpenChat(chat.clone())))
}

fn prompt(state: AppState, text: &str) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::SendPrompt(text.into())))
}

fn working(state: AppState, chat: &ChatId) -> AppState {
    let (state, _) = update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event: RpcEvent::AgentStart,
        },
    );
    state
}

fn agent_end(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event: RpcEvent::AgentEnd { will_retry: None },
        },
    )
}

/// Starts a turn in each named chat, in order, like two real children would.
fn working_chats(names: &[&str]) -> AppState {
    let mut state = AppState::default();
    for name in names {
        let (next, _) = open(state, &chat(name));
        state = working(next, &chat(name));
    }
    state
}

/// The prompts the effects ask to send, in order.
fn sends(effects: &Effects) -> Vec<(&ChatId, &str)> {
    effects
        .chat_commands
        .iter()
        .filter_map(|command| match command {
            ChatCommand::Send {
                chat,
                rpc: RpcCommand::Prompt { message },
            } => Some((chat, message.as_str())),
            _ => None,
        })
        .collect()
}

#[test]
fn under_the_cap_a_prompt_is_sent_at_once() {
    let (state, _) = open(AppState::default(), &chat("a"));

    let (state, effects) = prompt(state, "hola");

    assert_eq!(sends(&effects), vec![(&chat("a"), "hola")]);
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Idle),
        "the app does not mark working: the child's agent_start does"
    );
}

#[test]
fn the_default_cap_is_two_working_chats() {
    let state = working_chats(&["a", "b"]);
    let (state, _) = open(state, &chat("c"));

    let (state, effects) = prompt(state, "hola");

    assert_eq!(sends(&effects), Vec::new());
    assert_eq!(
        state.chats.activity(&chat("c")),
        Some(ChatActivity::Queued),
        "the app starts with the host's default cap, not no cap"
    );
}

#[test]
fn at_the_cap_a_prompt_is_queued_and_sends_nothing() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));

    let (state, effects) = prompt(state, "hola");

    assert_eq!(sends(&effects), Vec::new(), "nothing over the cap is sent");
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Queued),
        "the sidebar says the chat has a prompt waiting"
    );
}

#[test]
fn the_thread_of_a_queued_chat_already_shows_its_message() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));

    let (state, _) = prompt(state, "hola");

    let thread = &state.chats.state(&chat("b")).expect("chat b").messages;
    assert_eq!(thread.len(), 1);
    assert_eq!(thread[0].text, "hola");
    assert_eq!(thread[0].id, "msg-0");
}

#[test]
fn a_chat_with_a_queued_prompt_takes_no_second_prompt() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "primero");

    let (state, effects) = prompt(state, "segundo");

    assert_eq!(
        sends(&effects),
        Vec::new(),
        "one turn at a time, and a queued prompt counts as one"
    );
    let thread = &state.chats.state(&chat("b")).expect("chat b").messages;
    assert_eq!(thread.len(), 1, "the second prompt is not in the thread");
}

#[test]
fn a_freed_slot_releases_the_oldest_queued_prompt_and_takes_the_slot() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "primero");
    let (state, _) = open(state, &chat("c"));
    let (state, _) = prompt(state, "segundo");

    let (state, effects) = agent_end(state, &chat("a"));

    assert_eq!(
        sends(&effects),
        vec![(&chat("b"), "primero")],
        "FIFO: the oldest queued prompt goes out first"
    );
    assert_eq!(
        state.chats.working_count(),
        1,
        "the released prompt counts as taken at once"
    );
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Working),
        "the released chat shows working, not queued"
    );
}

#[test]
fn another_freed_slot_releases_the_next_fifo_prompt() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "primero");
    let (state, _) = open(state, &chat("c"));
    let (state, _) = prompt(state, "segundo");
    let (state, _) = agent_end(state, &chat("a"));
    let (state, _) = agent_end(state, &chat("b"));

    let (state, effects) = agent_end(state, &chat("b"));

    assert_eq!(
        sends(&effects),
        Vec::new(),
        "the second slot took the second prompt already"
    );
    assert_eq!(
        state.chats.activity(&chat("c")),
        Some(ChatActivity::Working),
        "the second queued prompt went out with it"
    );
    let thread = &state.chats.state(&chat("c")).expect("chat c").messages;
    assert_eq!(thread[0].text, "segundo");
}

#[test]
fn no_release_happens_twice_for_the_slot_it_already_took() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "hola");
    let (state, first) = agent_end(state, &chat("a"));

    let (state, second) = agent_end(state, &chat("a"));

    assert_eq!(sends(&first), vec![(&chat("b"), "hola")]);
    assert_eq!(
        sends(&second),
        Vec::new(),
        "a taken slot is not handed out again"
    );
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Working),
        "b still holds the slot the first release gave it"
    );
}

#[test]
fn an_abort_cancels_the_queued_prompt_and_sends_nothing() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "hola");

    let (state, effects) = update_chats(state, Input::Intent(Intent::Abort));

    assert_eq!(effects.chat_commands, Vec::new(), "nothing is sent");
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Idle),
        "the queue does not hold the prompt any more"
    );
    // Filling the slot afterwards sends nothing: the prompt is gone.
    let (state, effects) = agent_end(state, &chat("a"));
    assert_eq!(sends(&effects), Vec::new());
    assert!(!state.chats.is_queued(&chat("b")), "and stays gone");
}

#[test]
fn an_ended_chat_drops_its_queued_prompt() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "hola");

    let (state, _) = update_chats(
        state,
        Input::ChatClosed {
            chat: chat("b"),
            expected: false,
        },
    );

    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Ended),
        "the death is what the sidebar shows"
    );
    // The dead chat's prompt can never take a slot: Filling one afterwards
    // sends nothing, since the death notice already told the user.
    let (state, _) = agent_end(state, &chat("c"));
    let (_, effects) = agent_end(state, &chat("a"));
    assert_eq!(sends(&effects), Vec::new());
}

#[test]
fn a_working_chat_whose_child_dies_releases_its_slot() {
    let (state, _) = update_chats(working_chats(&["a"]), Input::SetWorkingCap(1));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = prompt(state, "hola");

    let (state, effects) = update_chats(
        state,
        Input::ChatClosed {
            chat: chat("a"),
            expected: false,
        },
    );

    assert_eq!(
        sends(&effects),
        vec![(&chat("b"), "hola")],
        "the release happens in the same step that notices the death"
    );
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Ended),
        "the death is what the sidebar shows"
    );
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Working),
        "the released prompt took the dead chat's slot"
    );
}

#[test]
fn cap_zero_never_queues_anything() {
    let state = working_chats(&["a", "b", "c"]);
    let (state, _) = update_chats(state, Input::SetWorkingCap(0));
    let (state, _) = open(state, &chat("d"));

    let (state, effects) = prompt(state, "hola");

    assert_eq!(sends(&effects), vec![(&chat("d"), "hola")]);
    assert!(!state.chats.is_queued(&chat("d")), "no cap is enforced");
}
