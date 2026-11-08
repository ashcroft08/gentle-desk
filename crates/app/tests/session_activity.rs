//! The activity the sidebar shows for one chat, answered by session path
//! through the app layer's public interface (S4).
//!
//! A row in the sidebar only has the path of the session it lists, so the pure
//! query is `AppState::activity_of_session` — unknown chat, idle — with the
//! per-chat state behind it driving working, finished-unread, needs-attention
//! and ended.

use gentle_app::{update_chats, AppState, ChatActivity, ChatId, Input, Intent};
use gentle_core::rpc::{RpcEvent, UiRequest};

fn session(name: &str) -> String {
    format!("/sessions/{name}.jsonl")
}

/// The query the sidebar asks before anything has happened: every path it does
/// not list yet — and there are far more of those than live chats — is idle.
#[test]
fn a_session_path_the_app_never_saw_is_idle() {
    let state = AppState::default();

    assert_eq!(
        state.activity_of_session(&session("never-opened")),
        ChatActivity::Idle
    );
}

// The store's events and intents are the only way something happens; each
// one below is already observed on `Chats`, but the sidebar reads the answer
// through the path, so it is pinned here in that shape too.

fn event(path: &str, event: RpcEvent) -> Input {
    Input::ChatEvent {
        chat: ChatId::of_session(path),
        event,
    }
}

fn opened(state: AppState, path: &str) -> AppState {
    update_chats(
        state,
        Input::Intent(Intent::OpenChat(ChatId::of_session(path))),
    )
    .0
}

#[test]
fn a_chat_running_a_turn_is_working() {
    let path = session("a");
    let (state, _) = update_chats(AppState::default(), event(&path, RpcEvent::AgentStart));

    assert_eq!(state.activity_of_session(&path), ChatActivity::Working);
}

#[test]
fn a_background_chat_that_finished_is_finished_unread_just_while_unread() {
    let a = session("a");
    let b = session("b");

    // a works and settles after the user moved to b; b is read on return.
    let state = opened(opened(AppState::default(), &a), &b);
    let (state, _) = update_chats(state, event(&a, RpcEvent::AgentStart));
    let (state, _) = update_chats(state, event(&a, RpcEvent::AgentSettled));

    assert_eq!(
        state.activity_of_session(&a),
        ChatActivity::FinishedUnread,
        "the reply waits to be read as long as the chat is not open"
    );
    assert_eq!(state.activity_of_session(&b), ChatActivity::Idle);

    let state = opened(state, &a);

    assert_eq!(
        state.activity_of_session(&a),
        ChatActivity::Idle,
        "opening the chat reads the reply, so the row has nothing to say"
    );
}

#[test]
fn a_chat_with_a_pending_dialog_needs_attention() {
    let a = session("a");
    let b = session("b");
    let dialog = RpcEvent::UiRequest(UiRequest::Confirm {
        id: "d1".into(),
        title: "Permission".into(),
        message: "go on?".into(),
        timeout: None,
    });

    let state = opened(opened(AppState::default(), &a), &b);
    let (state, _) = update_chats(state, event(&a, dialog));

    assert_eq!(
        state.activity_of_session(&a),
        ChatActivity::NeedsAttention,
        "a background chat's dialog is queued and asks for the user"
    );
}

#[test]
fn a_chat_whose_child_died_is_ended_until_it_is_reopened() {
    let a = session("a");

    let state = opened(AppState::default(), &a);
    let (state, _) = update_chats(
        state,
        Input::ChatClosed {
            chat: ChatId::of_session(&a),
            expected: false,
        },
    );

    assert_eq!(state.activity_of_session(&a), ChatActivity::Ended);

    let state = opened(state, &a);

    assert_eq!(
        state.activity_of_session(&a),
        ChatActivity::Idle,
        "opening the chat asks for a fresh child, so the end is cleared"
    );
}

#[test]
fn a_chat_the_host_stopped_on_purpose_is_just_idle() {
    let a = session("a");

    let state = opened(AppState::default(), &a);
    let (state, _) = update_chats(
        state,
        Input::ChatClosed {
            chat: ChatId::of_session(&a),
            expected: true,
        },
    );

    assert_eq!(state.activity_of_session(&a), ChatActivity::Idle);
}

/// The rows also read one plain snapshot instead of the app state, so the
/// sidebar never reaches past the activities: the snapshot is answered by
/// session path, and an idle or never-opened chat is simply not in it.
#[test]
fn the_activity_snapshot_by_path_holds_only_the_chats_that_are_not_idle() {
    let a = session("a");
    let b = session("b");
    let c = session("c");

    // a is left working behind b, which is open again and read again: idle.
    let state = opened(AppState::default(), &a);
    let state = opened(state, &b);
    let (state, _) = update_chats(state, event(&a, RpcEvent::AgentStart));
    let state = opened(state, &b);

    let snapshot = state.activity_by_path();

    assert_eq!(snapshot.get(&a), Some(&ChatActivity::Working));
    assert_eq!(snapshot.get(&b), None, "an idle chat costs nothing");
    assert_eq!(snapshot.get(&c), None, "a chat never opened is not in it");
    assert_eq!(snapshot.len(), 1);
}
