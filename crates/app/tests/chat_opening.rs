//! A chat whose child is being started (the opening phase): the sidebar shows
//! it, the composer takes no prompt yet, and the phase ends the moment the
//! child is confirmed up (`ChatOpened`) or its start fails (`ChatFailed`),
//! so it can never stick.

use gentle_app::{
    update_chats, AppState, ChatActivity, ChatCommand, ChatId, Effects, Input, Intent,
    NoticeAction, NoticeLevel,
};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, name: &str) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::OpenChat(chat(name))))
}

fn opened(state: AppState, name: &str) -> AppState {
    update_chats(state, Input::ChatOpened { chat: chat(name) }).0
}

fn failed(state: AppState, name: &str) -> AppState {
    update_chats(
        state,
        Input::ChatFailed {
            chat: chat(name),
            message: "cannot start the child".into(),
        },
    )
    .0
}

fn prompt(state: AppState, text: &str) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::SendPrompt(text.into())))
}

#[test]
fn an_open_marks_the_chat_opening_and_asks_the_host_for_its_child() {
    let (state, effects) = open(AppState::default(), "a");

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }]
    );
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "the child takes seconds, so the chat says it is coming up"
    );
}

#[test]
fn the_child_being_up_ends_the_opening_phase() {
    let (state, _) = open(AppState::default(), "a");

    let state = opened(state, "a");

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}

#[test]
fn a_failed_open_ends_the_opening_phase_and_reports_itself() {
    let (state, _) = open(AppState::default(), "a");

    let state = failed(state, "a");

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Idle),
        "the opening mark never sticks after a failure (the failure notice is the report)"
    );
    let notice = state.notices.last().expect("the failure reports itself");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert!(notice.text.contains("cannot start the child"));
    assert!(notice.text.contains(&chat("a").label()));
}

#[test]
fn a_focus_only_reopen_clears_at_once_through_chat_opened() {
    // A chat whose child is already up opens instantly on the host side, but
    // the app only knows it from the same `ChatOpened` confirmation.
    let (state, _) = open(AppState::default(), "a");
    let state = opened(state, "a");

    let (state, effects) = open(state, "a");

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }],
        "the host is asked even for a live child: it answers Existing"
    );
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "until it answers, the chat is still opening"
    );

    let state = opened(state, "a");

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}

#[test]
fn a_prompt_to_an_opening_chat_is_refused_and_leaves_no_thread_message() {
    let (state, _) = open(AppState::default(), "a");

    let (state, effects) = prompt(state, "hola");

    assert_eq!(effects, Effects::default(), "there is no child to write to");
    assert!(
        state
            .chats
            .state(&chat("a"))
            .expect("chat a")
            .messages
            .is_empty(),
        "the prompt is not appended while the child is coming up"
    );
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "the refusal changes nothing the sidebar shows"
    );
}

#[test]
fn a_chat_that_dies_while_opening_is_ended_not_opening() {
    let (state, _) = open(AppState::default(), "a");

    let (state, _) = update_chats(
        state,
        Input::ChatClosed {
            chat: chat("a"),
            expected: false,
        },
    );

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Ended));
    let notice = state.notices.last().expect("a notice");
    assert_eq!(
        notice.action,
        Some(NoticeAction::RestartChat(chat("a"))),
        "a death while opening still offers the Restart"
    );
}

#[test]
fn a_chat_the_host_stopped_while_opening_returns_to_idle() {
    let (state, _) = open(AppState::default(), "a");

    let (state, _) = update_chats(
        state,
        Input::ChatClosed {
            chat: chat("a"),
            expected: true,
        },
    );

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
    assert!(
        state.notices.is_empty(),
        "a stop the host asked for is silent"
    );
}

#[test]
fn working_still_wins_over_opening() {
    // The child answered between the click and the confirmation: its own
    // events say what it is doing, and that outranks the opening phase.
    let (state, _) = open(AppState::default(), "a");
    let (state, _) = update_chats(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: gentle_core::rpc::RpcEvent::AgentStart,
        },
    );

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Working)
    );
    let state = opened(state, "a");
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Working),
        "the missing confirmation of the opening phase takes nothing from the turn"
    );
}

#[test]
fn a_new_chat_is_opening_from_its_first_click() {
    let (state, _) = update_chats(AppState::default(), Input::Intent(Intent::OpenNewChat));

    let fresh = state
        .chats
        .foreground()
        .cloned()
        .expect("the new chat is the foreground one");
    assert_eq!(state.chats.activity(&fresh), Some(ChatActivity::Opening));
    // And it follows its chat through the rekey to the stored chat.
    let stored = ChatId::of_session("/s/new.jsonl");
    let state = update_chats(
        state,
        Input::ChatRekeyed {
            from: fresh.clone(),
            to: stored.clone(),
        },
    )
    .0;
    assert_eq!(state.chats.activity(&stored), Some(ChatActivity::Opening));
}
