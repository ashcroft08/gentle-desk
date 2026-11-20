//! What the shell reports about one chat: a command that could not reach its
//! child is a notice, and a new chat that learned its session file moves to the
//! stored chat it now is.

use gentle_app::{
    update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent, NoticeLevel,
};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

/// A state with one fresh chat open and started, and that chat's id: the click
/// allocates it and the composer's own ask is what starts its child
/// (welcome-screen T4, R2).
fn with_a_new_chat() -> (AppState, ChatId) {
    let (state, effects) = update_chats(AppState::default(), Input::Intent(Intent::OpenNewChat));
    assert_eq!(
        effects,
        Effects::default(),
        "a new chat asks for no child by itself"
    );
    let chat = state
        .chats
        .foreground()
        .cloned()
        .expect("the new chat is the foreground one");
    let (state, effects) = update_chats(state, Input::Intent(Intent::EnsureStarted(chat.clone())));
    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat.clone() }],
        "the composer's ask starts it"
    );
    (state, chat)
}

#[test]
fn a_command_that_could_not_reach_its_child_is_an_error_notice() {
    let (state, open) = with_a_new_chat();
    // Only the thread is claimed to be unchanged: the transient opening mark
    // of the failed open is cleared by `ChatFailed`, so the phase never sticks.
    let before = state.chats.state(&open).expect("chat").messages();

    let (state, effects) = update_chats(
        state,
        Input::ChatFailed {
            chat: open.clone(),
            message: "cannot start the child: no launcher".into(),
        },
    );

    assert_eq!(effects, Effects::default(), "a notice asks for nothing");
    let notice = state.notices.last().expect("a notice");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert!(
        notice.text.contains("no launcher"),
        "the reason is shown: {}",
        notice.text
    );
    assert!(
        notice.text.contains(&open.label()),
        "the chat it happened to is named: {}",
        notice.text
    );
    assert_eq!(
        state.chats.state(&open).expect("chat").messages(),
        before,
        "a failed command leaves the chat's thread as it was"
    );
}

#[test]
fn a_new_chat_that_reports_its_session_file_becomes_the_stored_chat() {
    let (state, open) = with_a_new_chat();
    let stored = chat("C:/s/p.jsonl");

    let (state, effects) = update_chats(
        state,
        Input::ChatRekeyed {
            from: open.clone(),
            to: stored.clone(),
        },
    );

    assert_eq!(effects, Effects::default());
    assert_eq!(state.chats.foreground(), Some(&stored));
    assert_eq!(state.chats.state(&open), None, "the old identity is gone");
    assert_eq!(state.chats.len(), 1, "one chat, not two");
}
