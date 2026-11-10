//! An extension `notify` of one chat in the multi-chat fold.
//!
//! The one-child path turns a `notify` into a notice; the multi-chat fold runs
//! for every chat of a shell with one child per chat, so it must turn it into
//! the same notice - same level, same text, same id space and the same
//! deduplication - or extension messages would reach the desktop app's notice
//! list for one chat and no chat at all for another.

use gentle_app::{update, update_chats, AppState, ChatId, Input, Intent, NoticeLevel};
use gentle_core::rpc::{decode_line, Decoded, RpcEvent};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

/// The event a child's `notify` line decodes to.
fn notify(text: &str, level: &str) -> RpcEvent {
    let line = format!(
        r#"{{"type":"extension_ui_request","id":"n","method":"notify","message":"{text}","notifyType":"{level}"}}"#
    );
    match decode_line(&line) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

fn fold(state: AppState, chat: &ChatId, event: RpcEvent) -> AppState {
    update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event,
        },
    )
    .0
}

fn timed(state: AppState, chat: &ChatId, event: RpcEvent) -> AppState {
    update_chats(
        state,
        Input::ChatEventAt {
            chat: chat.clone(),
            event,
            now_ms: 1_790_074_800_000,
        },
    )
    .0
}

fn fold_single(state: AppState, event: RpcEvent) -> AppState {
    update(state, Input::Rpc(event)).0
}

#[test]
fn a_chats_warning_notify_is_one_warning_notice_with_its_text() {
    let state = fold(
        AppState::default(),
        &chat("a"),
        notify("rate limit reached", "warning"),
    );

    assert_eq!(state.notices.len(), 1, "one notify, one notice");
    let notice = &state.notices[0];
    assert_eq!(notice.level, NoticeLevel::Warning);
    assert_eq!(notice.text, "rate limit reached");
    assert_eq!(notice.action, None, "an extension notice asks for nothing");
    assert_eq!(notice.id, 0, "the first notice of the app is id 0");
}

#[test]
fn a_chats_info_notify_is_an_info_notice() {
    let state = fold(
        AppState::default(),
        &chat("a"),
        notify("session switched", "info"),
    );

    assert_eq!(state.notices.len(), 1);
    assert_eq!(state.notices[0].level, NoticeLevel::Info);
    assert_eq!(state.notices[0].text, "session switched");
}

#[test]
fn an_event_that_is_not_a_notify_leaves_no_notice() {
    let status = match decode_line(
        r#"{"type":"extension_ui_request","id":"4","method":"setStatus","statusKey":"x","statusText":"y"}"#,
    ) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    };

    let state = fold(AppState::default(), &chat("a"), RpcEvent::AgentStart);
    let state = fold(state, &chat("a"), status);

    assert!(state.notices.is_empty(), "got {:?}", state.notices);
    assert!(
        state.chats.state(&chat("a")).is_some(),
        "the chat still folded its own events"
    );
}

#[test]
fn the_same_event_folded_twice_is_one_notice_as_in_the_one_child_path() {
    let event = notify("rate limit reached", "warning");

    let folded = fold(
        fold(AppState::default(), &chat("a"), event.clone()),
        &chat("a"),
        event.clone(),
    );
    let single = fold_single(fold_single(AppState::default(), event.clone()), event);

    assert_eq!(folded.notices.len(), 1, "an extension repeats itself");
    assert_eq!(
        folded.notices, single.notices,
        "the multi-chat fold keeps the same dedup, level, text and ids"
    );
}

#[test]
fn two_chats_keep_their_own_notify_texts() {
    let state = fold(
        AppState::default(),
        &chat("a"),
        notify("a says this", "warning"),
    );
    let state = fold(state, &chat("b"), notify("b says that", "warning"));

    let texts: Vec<&str> = state
        .notices
        .iter()
        .map(|notice| notice.text.as_str())
        .collect();
    assert_eq!(texts, vec!["a says this", "b says that"]);
}

#[test]
fn a_background_chats_notify_is_its_own_text_and_not_the_foreground_chats() {
    let state = update_chats(
        AppState::default(),
        Input::Intent(Intent::OpenChat(chat("b"))),
    )
    .0;
    let state = fold(state, &chat("a"), notify("from a", "warning"));

    assert_eq!(
        state.chats.foreground(),
        Some(&chat("b")),
        "the foreground chat does not move"
    );
    assert_eq!(state.notices.len(), 1);
    assert_eq!(
        state.notices[0].text, "from a",
        "b's title is never attached to a's message"
    );
    assert_eq!(state.notices[0].action, None);
}

#[test]
fn a_timed_event_carries_the_notice_too() {
    let state = timed(
        AppState::default(),
        &chat("a"),
        notify("rate limit reached", "warning"),
    );

    assert_eq!(state.notices.len(), 1);
    assert_eq!(state.notices[0].level, NoticeLevel::Warning);
    assert_eq!(state.notices[0].text, "rate limit reached");
}

#[test]
fn a_notify_leaves_the_error_notices_the_shell_already_had() {
    let state = fold(
        AppState::default(),
        &chat("a"),
        notify("rate limit reached", "warning"),
    );
    let state = update_chats(
        state,
        Input::ChatFailed {
            chat: chat("a"),
            message: "no launcher".into(),
        },
    )
    .0;

    assert_eq!(state.notices.len(), 2);
    assert_eq!(state.notices[0].level, NoticeLevel::Warning);
    assert_eq!(state.notices[0].text, "rate limit reached");
    assert_eq!(state.notices[1].level, NoticeLevel::Error);
    assert!(
        state.notices[1].text.contains("no launcher"),
        "the failure notice is unchanged: {:?}",
        state.notices[1].text
    );
}
