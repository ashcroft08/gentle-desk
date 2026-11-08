//! What the shell reports about one chat: a command that could not reach its
//! child is a notice, and a new chat that learned its session file moves to the
//! stored chat it now is.

use gentle_app::{
    update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent, NoticeLevel,
};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

/// A state with one fresh chat open, and that chat's id.
fn with_a_new_chat() -> (AppState, ChatId) {
    let (state, effects) = update_chats(AppState::default(), Input::Intent(Intent::OpenNewChat));
    let ChatCommand::OpenProcess { chat } = effects.chat_commands[0].clone() else {
        panic!("a new chat asks for its child: {:?}", effects.chat_commands);
    };
    (state, chat)
}

#[test]
fn a_command_that_could_not_reach_its_child_is_an_error_notice() {
    let (state, open) = with_a_new_chat();
    let before = state.chats.state(&open).cloned();

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
        state.chats.state(&open),
        before.as_ref(),
        "a failed command leaves the chat as it was"
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
