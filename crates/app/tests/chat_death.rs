//! A chat whose child is gone (S3): an unexpected death ends that chat and
//! leaves one recoverable notice naming it, with a Restart action; an expected
//! stop (the host reaped, capped or closed it) leaves the chat just idle and
//! silent, and its next open starts a child again.

use gentle_app::{
    update_chats, AppState, ChatActivity, ChatCommand, ChatId, Effects, Input, Intent,
    NoticeAction, NoticeLevel,
};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::OpenChat(chat.clone())))
}

/// What the real UI sends the moment the host answers an open, fresh start or
/// already-live child alike: the opening phase is over.
fn opened(state: AppState, chat: &ChatId) -> AppState {
    update_chats(state, Input::ChatOpened { chat: chat.clone() }).0
}

fn closed(state: AppState, chat: &ChatId, expected: bool) -> (AppState, Effects) {
    update_chats(
        state,
        Input::ChatClosed {
            chat: chat.clone(),
            expected,
        },
    )
}

fn prompt(state: AppState, text: &str) -> (AppState, Effects) {
    update_chats(state, Input::Intent(Intent::SendPrompt(text.into())))
}

#[test]
fn an_unexpected_death_ends_that_chat_with_one_error_notice_that_restarts_it() {
    let (state, _) = open(AppState::default(), &chat("a"));

    let (state, effects) = closed(state, &chat("a"), false);

    assert_eq!(
        effects,
        Effects::default(),
        "a death alone asks for nothing"
    );
    assert_eq!(state.chats.ended(), vec![chat("a")]);
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Ended),
        "the sidebar shows the death, not a working chat"
    );
    let notice = state.notices.last().expect("a notice");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert!(
        notice.text.contains(&chat("a").label()),
        "the notice names the chat: {}",
        notice.text
    );
    assert_eq!(
        notice.action,
        Some(NoticeAction::RestartChat(chat("a"))),
        "the notice carries the one way out"
    );
}

#[test]
fn a_background_death_ends_only_its_own_chat() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let state = opened(state, &chat("a"));
    let (state, _) = open(state, &chat("b"));
    let state = opened(state, &chat("b"));

    let (state, _) = closed(state, &chat("a"), false);

    assert_eq!(state.chats.ended(), vec![chat("a")]);
    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Ended));
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Idle),
        "the chat the user is on is untouched"
    );
    assert_eq!(state.chats.foreground(), Some(&chat("b")));
    assert_eq!(state.notices.len(), 1, "one notice for the one death");
}

#[test]
fn each_chat_that_dies_is_named_once_and_a_repeat_death_is_not_a_storm() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = open(state, &chat("b"));

    let (state, _) = closed(state, &chat("a"), false);
    let (state, _) = closed(state, &chat("b"), false);
    let (state, _) = closed(state, &chat("a"), false);

    let named: Vec<&str> = state
        .notices
        .iter()
        .filter(|notice| notice.level == NoticeLevel::Error)
        .map(|notice| notice.text.as_str())
        .collect();
    assert_eq!(named.len(), 2, "one per chat that died: {named:?}");
    assert!(named.iter().any(|text| text.contains(&chat("a").label())));
    assert!(named.iter().any(|text| text.contains(&chat("b").label())));
    assert_eq!(state.chats.ended(), vec![chat("a"), chat("b")]);
}

#[test]
fn an_expected_stop_leaves_the_chat_idle_and_says_nothing() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let before = state.chats.state(&chat("a")).cloned();

    let (state, effects) = closed(state, &chat("a"), true);

    assert_eq!(effects, Effects::default());
    assert_eq!(state.chats.ended(), Vec::new(), "a reaped chat is not dead");
    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
    assert_eq!(state.chats.foreground(), Some(&chat("a")));
    assert!(
        state.notices.is_empty(),
        "nothing to show: it reopens silently"
    );
    assert_eq!(
        state.chats.state(&chat("a")),
        before.as_ref(),
        "its thread is kept for the reopen"
    );
}

#[test]
fn restart_clears_the_end_and_asks_the_host_for_the_child_again() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = closed(state, &chat("a"), false);

    let (state, effects) = update_chats(state, Input::Intent(Intent::RestartChat(chat("a"))));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }],
        "the child is started again through --session"
    );
    assert_eq!(state.chats.ended(), Vec::new());
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "the fresh child is not up yet: the chat says it is coming up"
    );
    assert_eq!(state.chats.foreground(), Some(&chat("a")));

    let state = opened(state, &chat("a"));

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}

#[test]
fn a_prompt_to_an_ended_chat_is_a_notice_and_reaches_no_child() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let state = opened(state, &chat("a"));
    let (state, _) = prompt(state, "hola");
    let (state, _) = closed(state, &chat("a"), false);

    let (state, effects) = prompt(state, "otra");

    assert_eq!(effects, Effects::default(), "there is no child to write to");
    let messages = &state.chats.state(&chat("a")).expect("chat a").messages;
    assert_eq!(messages.len(), 1, "the prompt was not appended");
    assert_eq!(messages[0].text, "hola");
    let notice = state.notices.last().expect("a notice");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert_eq!(
        notice.action,
        Some(NoticeAction::RestartChat(chat("a"))),
        "the same Restart that fixes the death"
    );
}

#[test]
fn opening_an_ended_chat_starts_a_fresh_child_and_clears_the_end() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = closed(state, &chat("a"), false);

    let (state, effects) = open(state, &chat("a"));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }]
    );
    assert_eq!(state.chats.ended(), Vec::new());
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening),
        "the fresh child is coming up"
    );

    let state = opened(state, &chat("a"));

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}
