//! A chat's event that arrives with its wall-clock time: the timestamp reaches
//! that chat's helper bookkeeping, also for a chat in the background, and a
//! timeless `ChatEvent` still behaves as before.

use gentle_app::{update_chats, AppState, ChatId, Input, Intent};
use gentle_core::helpers::HelperStatus;
use gentle_core::rpc::{decode_line, Decoded, RpcEvent};

const RUNNING: &str =
    r#"[{"summary":{"id":"t1","status":"running","createdAt":"2026-09-22T10:00:00.000Z"}}]"#;
const COUNTS_1: &str = r#"{"running":1,"queued":0,"waiting":0,"finished":0}"#;
const COUNTS_DONE: &str = r#"{"running":0,"queued":0,"waiting":0,"finished":1}"#;
const NOW: u64 = 1_790_074_800_000;

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn widget(summary: &str, tasks: &str) -> RpcEvent {
    let lines =
        format!(r#"{{"schema":"gentle-agents.activity/v1","summary":{summary},"tasks":{tasks}}}"#);
    let raw = serde_json::json!({
        "type": "extension_ui_request", "id": "w", "method": "setWidget",
        "widgetKey": "gentle-agents", "widgetLines": [lines]
    });
    match decode_line(&raw.to_string()) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

fn run(state: AppState, inputs: Vec<Input>) -> AppState {
    inputs
        .into_iter()
        .fold(state, |state, input| update_chats(state, input).0)
}

fn timed(chat: &ChatId, event: RpcEvent, now_ms: u64) -> Input {
    Input::ChatEventAt {
        chat: chat.clone(),
        event,
        now_ms,
    }
}

fn plain(chat: &ChatId, event: RpcEvent) -> Input {
    Input::ChatEvent {
        chat: chat.clone(),
        event,
    }
}

fn first_task(state: &AppState, chat: &ChatId) -> (HelperStatus, Option<u64>) {
    let task = &state
        .chats
        .state(chat)
        .expect("the chat exists")
        .helpers
        .tasks[0];
    (task.status, task.ended_at_ms)
}

#[test]
fn the_arrival_time_stamps_a_task_a_background_chat_stops_reporting() {
    let (a, b) = (chat("a"), chat("b"));
    let state = run(
        AppState::default(),
        vec![
            Input::Intent(Intent::OpenChat(a.clone())),
            plain(&b, widget(COUNTS_1, RUNNING)),
            // The user is on chat "a": "b" is in the background when it reports.
            timed(&b, widget(COUNTS_DONE, "[]"), NOW),
        ],
    );

    assert_eq!(first_task(&state, &b), (HelperStatus::Done, Some(NOW)));
}

#[test]
fn a_timeless_event_leaves_the_end_time_unknown_as_before() {
    let a = chat("a");
    let state = run(
        AppState::default(),
        vec![
            plain(&a, widget(COUNTS_1, RUNNING)),
            plain(&a, widget(COUNTS_DONE, "[]")),
        ],
    );

    assert_eq!(first_task(&state, &a), (HelperStatus::Done, None));
}

#[test]
fn a_timed_event_behaves_like_a_plain_one_for_everything_else() {
    let a = chat("a");
    let start = RpcEvent::AgentStart;

    let timed_state = run(AppState::default(), vec![timed(&a, start.clone(), NOW)]);
    let plain_state = run(AppState::default(), vec![plain(&a, start)]);

    assert_eq!(timed_state, plain_state);
    assert!(timed_state.chats.state(&a).unwrap().working);
}

#[test]
fn a_timed_event_never_touches_another_chat() {
    let (a, b) = (chat("a"), chat("b"));
    let state = run(
        AppState::default(),
        vec![
            plain(&a, widget(COUNTS_1, RUNNING)),
            timed(&b, widget(COUNTS_1, RUNNING), NOW),
            timed(&b, widget(COUNTS_DONE, "[]"), NOW),
        ],
    );

    assert_eq!(first_task(&state, &a), (HelperStatus::Running, None));
    assert_eq!(first_task(&state, &b), (HelperStatus::Done, Some(NOW)));
}
