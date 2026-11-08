//! The Restart death notice only lives while the death is unfixed: a restart or
//! a reopen of that same chat removes exactly its notice, and every other
//! notice — another chat's death, a plain extension notice — stays.

use gentle_app::{update_chats, AppState, ChatId, Input, Intent, NoticeAction};
use gentle_core::rpc::{RpcEvent, UiRequest};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, chat: &ChatId) -> AppState {
    update_chats(state, Input::Intent(Intent::OpenChat(chat.clone()))).0
}

fn restart(state: AppState, chat: &ChatId) -> AppState {
    update_chats(state, Input::Intent(Intent::RestartChat(chat.clone()))).0
}

fn died(state: AppState, chat: &ChatId) -> AppState {
    update_chats(
        state,
        Input::ChatClosed {
            chat: chat.clone(),
            expected: false,
        },
    )
    .0
}

fn extension_notice(state: AppState) -> AppState {
    update_chats(
        state,
        Input::Rpc(RpcEvent::UiRequest(UiRequest::FireAndForget {
            id: "n1".into(),
            method: "notify".into(),
            raw: serde_json::json!({"message": "extension says something", "notifyType": "error"}),
        })),
    )
    .0
}

fn restart_action(state: &AppState) -> Option<&ChatId> {
    state
        .notices
        .iter()
        .find_map(|notice| match &notice.action {
            Some(NoticeAction::RestartChat(chat)) => Some(chat),
            _ => None,
        })
}

#[test]
fn restart_removes_its_own_death_notice() {
    let state = open(AppState::default(), &chat("a"));
    let state = died(state, &chat("a"));
    assert_eq!(
        restart_action(&state),
        Some(&chat("a")),
        "the death left its Restart notice"
    );

    let state = restart(state, &chat("a"));

    assert_eq!(
        restart_action(&state),
        None,
        "the fix took its own notice away"
    );
    assert!(state.notices.is_empty(), "no other notice was there");
}

#[test]
fn another_chats_death_notice_stays() {
    let state = open(AppState::default(), &chat("a"));
    let state = open(state, &chat("b"));
    let state = died(state, &chat("a"));
    let state = died(state, &chat("b"));

    let state = restart(state, &chat("a"));

    assert_eq!(
        restart_action(&state),
        Some(&chat("b")),
        "chat b's death is still unfixed: {}",
        state.notices.len()
    );
}

#[test]
fn an_extension_notice_stays() {
    let state = open(AppState::default(), &chat("a"));
    let state = extension_notice(state);
    let state = died(state, &chat("a"));
    assert_eq!(state.notices.len(), 2, "the death added its own notice");

    let state = restart(state, &chat("a"));

    assert_eq!(state.notices.len(), 1, "only the extension notice is left");
    let notice = &state.notices[0];
    assert_eq!(notice.text, "extension says something");
    assert!(notice.action.is_none());
}

#[test]
fn opening_a_healthy_chat_leaves_notices_untouched() {
    let state = open(AppState::default(), &chat("a"));
    let state = died(state, &chat("a"));
    let before = state.notices.clone();

    let state = open(state, &chat("b"));

    assert_eq!(
        state.notices, before,
        "opening a chat that never ended fixes nothing and drops nothing"
    );
    assert_eq!(
        restart_action(&state),
        Some(&chat("a")),
        "chat a's death is still unfixed"
    );
}

#[test]
fn a_chat_that_dies_again_after_a_restart_gets_a_fresh_notice() {
    let state = open(AppState::default(), &chat("a"));
    let state = died(state, &chat("a"));
    let state = restart(state, &chat("a"));

    let state = died(state, &chat("a"));

    assert_eq!(
        state.notices.len(),
        1,
        "one fresh notice: {:?}",
        state.notices
    );
    assert_eq!(
        restart_action(&state),
        Some(&chat("a")),
        "the new death carries Restart again"
    );
}
