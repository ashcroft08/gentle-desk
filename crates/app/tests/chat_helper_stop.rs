//! Stopping one running helper from its thread bar (helper-stop T2, S1/S2).
//!
//! The intent names one helper of the FOREGROUND chat and nothing else: the
//! answer is one command for that chat's own child (S6), the chat keeps running
//! and its other helpers are untouched (S1). Only a helper that is RUNNING can be
//! stopped - the thread bar's square is enabled exactly then - and every other id
//! (an unknown task, another chat's task, a helper that already finished or is
//! still queued) changes nothing, the rule `SelectHelper` already follows.
//!
//! The child's own acceptance comes back as `Input::HelperStopped`: that helper
//! reads `Cancelled` (the `Ended` mark both windows draw) and nothing else
//! changes. A stop that did not happen sends no input at all, so the helper keeps
//! the mark it had: silent degradation (S7).

use gentle_app::{
    update, update_chats, AppState, ChatActivity, ChatCommand, ChatId, Effects, Input, Intent,
};
use gentle_core::helpers::HelperStatus;
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
    let state = update_chats(state, Input::ChatOpened { chat: chat.clone() }).0;
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

/// The effects of one intent on the per-chat shell.
fn forked(state: AppState, intent: Intent) -> (AppState, Effects) {
    update_chats(state, Input::Intent(intent))
}

fn stop(state: AppState, id: &str) -> (AppState, Effects) {
    forked(state, Intent::StopHelper(id.to_owned()))
}

/// The status the state holds for one helper of one chat.
fn status(state: &AppState, chat: &ChatId, id: &str) -> Option<HelperStatus> {
    state
        .chats
        .state(chat)?
        .helpers
        .tasks
        .iter()
        .find(|task| task.id == id)
        .map(|task| task.status)
}

#[test]
fn stopping_a_running_helper_asks_that_chats_own_child_to_cancel_it() {
    let (state, chat) = opened(SESSION_A, &[("a1", "running"), ("a2", "waiting")]);

    let (state, effects) = stop(state, "a1");

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::StopHelper {
            chat: chat.clone(),
            helper_id: "a1".to_owned(),
        }],
        "one stop, for the foreground chat's own child and that helper"
    );
    assert_eq!(
        effects.commands,
        Vec::new(),
        "a helper's stop is no single-child command"
    );
    assert_eq!(
        status(&state, &chat, "a1"),
        Some(HelperStatus::Running),
        "the reducer marks nothing until the child answers (S7)"
    );
}

#[test]
fn a_helper_that_is_not_running_is_not_stopped() {
    // Only a running helper can be stopped (the square's own rule): a queued,
    // waiting or finished one answers nothing at all.
    for status in ["queued", "waiting", "done", "failed", "cancelled"] {
        let (state, _chat) = opened(SESSION_A, &[("a1", "running"), ("a2", status)]);

        let (_state, effects) = stop(state, "a2");

        assert_eq!(
            effects.chat_commands,
            Vec::new(),
            "a {status} helper is not running, so nothing is asked of the child"
        );
    }
}

#[test]
fn an_unknown_id_stops_nothing() {
    let (state, _chat) = opened(SESSION_A, &[("a1", "running")]);

    let (_state, effects) = stop(state, "ghost");

    assert_eq!(
        effects.chat_commands,
        Vec::new(),
        "an id that names no task is ignored, exactly as SelectHelper ignores one"
    );
}

#[test]
fn another_chats_helper_is_not_this_chats_to_stop() {
    // The stop names a task of the FOREGROUND chat: a helper that belongs to a
    // chat the user left can never be stopped from the one on screen (S6).
    let (state, _chat_a) = opened(SESSION_A, &[("a1", "running")]);
    let (_, chat_b) = opened(SESSION_B, &[("b1", "running")]);
    let state = update_chats(state, Input::Intent(Intent::OpenChat(chat_b.clone()))).0;
    let state = update_chats(
        state,
        Input::ChatOpened {
            chat: chat_b.clone(),
        },
    )
    .0;

    let (_state, effects) = stop(state, "a1");

    assert_eq!(
        effects.chat_commands,
        Vec::new(),
        "no other chat's child is asked to stop anything"
    );
}

#[test]
fn a_stop_leaves_the_open_thread_and_the_selection_alone() {
    let (state, _chat) = opened(SESSION_A, &[("a1", "running"), ("a2", "waiting")]);
    let (state, _) = forked(state, Intent::SelectHelper("a2".to_owned()));

    let (state, _) = stop(state, "a1");

    assert_eq!(
        state.selected_helper.as_deref(),
        Some("a2"),
        "stopping a helper is not opening it, and never closes the one that is open"
    );
}

#[test]
fn the_acceptance_marks_that_helper_cancelled_and_the_chat_keeps_running() {
    let (state, chat) = opened(SESSION_A, &[("a1", "running"), ("a2", "waiting")]);
    let (state, _) = update_chats(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event: RpcEvent::AgentStart,
        },
    );
    assert_eq!(
        state.chats.activity(&chat),
        Some(ChatActivity::Working),
        "the chat is mid-turn before the stop"
    );

    let (state, effects) = update_chats(
        state,
        Input::HelperStopped {
            chat: chat.clone(),
            helper_id: "a1".to_owned(),
        },
    );

    assert_eq!(
        status(&state, &chat, "a1"),
        Some(HelperStatus::Cancelled),
        "the stopped helper reads Cancelled, which both windows draw as the Ended mark"
    );
    assert_eq!(
        status(&state, &chat, "a2"),
        Some(HelperStatus::Waiting),
        "its sibling is untouched: one helper, never the chat (S1)"
    );
    assert_eq!(
        state.chats.activity(&chat),
        Some(ChatActivity::Working),
        "the chat keeps running: stopping a helper stops no turn"
    );
    assert_eq!(
        state.chats.foreground(),
        Some(&chat),
        "and the chat is still the one on screen"
    );
    assert_eq!(effects.chat_commands, Vec::new());
    assert_eq!(effects.commands, Vec::new());
}

#[test]
fn an_acceptance_the_state_cannot_place_changes_nothing() {
    // A chat or a task the state does not know: the stop already happened in the
    // child, and a task it stopped reporting has no card to re-mark. Nothing is
    // created and nothing panics.
    let (state, chat) = opened(SESSION_A, &[("a1", "running")]);

    let (unknown_task, _) = update_chats(
        state.clone(),
        Input::HelperStopped {
            chat: chat.clone(),
            helper_id: "ghost".to_owned(),
        },
    );
    assert_eq!(
        status(&unknown_task, &chat, "a1"),
        Some(HelperStatus::Running)
    );

    let (unknown_chat, _) = update_chats(
        state,
        Input::HelperStopped {
            chat: ChatId::of_session(SESSION_B),
            helper_id: "a1".to_owned(),
        },
    );
    assert!(
        unknown_chat
            .chats
            .state(&ChatId::of_session(SESSION_B))
            .is_none(),
        "an acceptance for a chat the app never opened creates no chat"
    );
}

#[test]
fn without_a_foreground_chat_the_single_child_is_asked_nothing() {
    // The one-child shell (`gentle-term`) has no desktop channel: a stop there
    // sends no command and marks nothing, and the legacy answers it shares with
    // the per-chat shell keep working.
    let state = update(
        AppState::default(),
        Input::Rpc(widget(&[("t1", "running")])),
    )
    .0;

    let (state, commands) = update(state, Input::Intent(Intent::StopHelper("t1".into())));

    assert_eq!(commands, Vec::new());
    assert_eq!(
        state
            .chat
            .helpers
            .tasks
            .iter()
            .find(|task| task.id == "t1")
            .map(|task| task.status),
        Some(HelperStatus::Running),
        "the single child's helper keeps its mark"
    );
}
