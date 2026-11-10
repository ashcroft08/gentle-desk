//! Selecting a helper of the foreground chat (T7b-1).
//!
//! The selection is app state (`AppState::selected_helper`) and the panel of a
//! shell that runs one child per chat reads it as the foreground chat's own
//! choice: a task of another chat can never be the selected one. The one-child
//! shell keeps its single-chat rule untouched, which the last test pins.

use gentle_app::{update, update_chats, AppState, ChatId, Input, Intent};
use gentle_core::rpc::{decode_line, Decoded, RpcEvent};
use serde_json::json;

const SESSION_A: &str = "/sessions/a.jsonl";
const SESSION_B: &str = "/sessions/b.jsonl";

/// The `gentle-agents` widget frame a child publishes: one task per pair.
fn widget(tasks: &[(&str, &str)]) -> RpcEvent {
    let tasks: Vec<serde_json::Value> = tasks
        .iter()
        .map(|(id, status)| {
            json!({"summary": {"id": id, "status": status, "createdAt": "2026-09-22T10:00:00.000Z"}})
        })
        .collect();
    let lines = json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": tasks.len(), "queued": 0, "waiting": 0, "finished": 0},
        "tasks": tasks,
    });
    let raw = json!({
        "type": "extension_ui_request", "id": "w", "method": "setWidget",
        "widgetKey": "gentle-agents", "widgetLines": [lines.to_string()],
    });
    match decode_line(&raw.to_string()) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

/// The stored chat at `path`, opened and holding `tasks` as its helpers.
fn opened(path: &str, tasks: &[(&str, &str)]) -> (AppState, ChatId) {
    let chat = ChatId::of_session(path);
    let state = update_chats(
        AppState::default(),
        Input::Intent(Intent::OpenChat(chat.clone())),
    )
    .0;
    let state = update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event: widget(tasks),
        },
    )
    .0;
    (state, chat)
}

fn forked(state: AppState, intent: Intent) -> AppState {
    update_chats(state, Input::Intent(intent)).0
}

fn select(state: AppState, id: &str) -> AppState {
    forked(state, Intent::SelectHelper(id.to_owned()))
}

#[test]
fn selecting_the_foreground_chats_helper_remembers_it() {
    let (state, _chat) = opened(SESSION_A, &[("a1", "running"), ("a2", "waiting")]);

    let state = select(state, "a1");

    assert_eq!(state.selected_helper.as_deref(), Some("a1"));
}

#[test]
fn an_unknown_id_leaves_the_selection_unchanged() {
    // The legacy rule: only a task the panel knows about can be opened.
    let (state, _chat) = opened(SESSION_A, &[("a1", "running"), ("a2", "waiting")]);

    assert_eq!(select(state.clone(), "ghost").selected_helper, None);
    let selected = select(state, "a2");
    assert_eq!(
        select(selected, "ghost").selected_helper.as_deref(),
        Some("a2"),
        "an id that names no task never clears the task that is open"
    );
}

#[test]
fn closing_returns_to_the_list() {
    let (state, _chat) = opened(SESSION_A, &[("a1", "running")]);

    let state = forked(select(state, "a1"), Intent::CloseHelper);

    assert_eq!(state.selected_helper, None);
}

#[test]
fn switching_the_foreground_chat_drops_the_selection() {
    // The selection names a task of ONE chat, so every way of making another
    // chat the foreground one forgets it.
    let cases = [
        Intent::OpenChat(ChatId::of_session(SESSION_B)),
        Intent::OpenNewChat,
        Intent::RestartChat(ChatId::of_session(SESSION_B)),
    ];
    for intent in cases {
        let (state, _chat) = opened(SESSION_A, &[("a1", "running")]);
        let selected = select(state, "a1");

        let switched = forked(selected, intent.clone());

        assert_eq!(
            switched.selected_helper, None,
            "{intent:?} must not leave a selection pointing at another chat's helper"
        );
    }
}

#[test]
fn without_a_foreground_chat_the_single_chat_rule_is_unchanged() {
    // The one-child shell (`gentle-term`) folds its widget into `AppState::chat`
    // and has no foreground chat, so the legacy rule still answers.
    let state = update(
        AppState::default(),
        Input::Rpc(widget(&[("t1", "running")])),
    )
    .0;

    let selected = update(state, Input::Intent(Intent::SelectHelper("t1".into()))).0;
    assert_eq!(selected.selected_helper.as_deref(), Some("t1"));
    let ghost = update(
        selected.clone(),
        Input::Intent(Intent::SelectHelper("ghost".into())),
    )
    .0;
    assert_eq!(ghost.selected_helper.as_deref(), Some("t1"));
    let closed = update(selected, Input::Intent(Intent::CloseHelper)).0;
    assert_eq!(closed.selected_helper, None);
}
