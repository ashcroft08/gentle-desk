//! Helpers state through the reducer: retention, ordering and the widget
//! rules, ported from the reference app's `chatReducer.test.ts`.

use gentle_core::chat::{reduce_chat, reduce_chat_at, ChatState};
use gentle_core::helpers::{HelperStatus, HelpersSummary};
use gentle_core::rpc::{decode_line, Decoded};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/helpers-activity.jsonl");

fn widget_event(key: &str, lines: Option<Vec<String>>) -> gentle_core::rpc::RpcEvent {
    let mut raw = json!({"type": "extension_ui_request", "id": "ui", "method": "setWidget", "widgetKey": key});
    if let Some(lines) = lines {
        raw["widgetLines"] = json!(lines);
    }
    match decode_line(&raw.to_string()) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

fn frame(summary: Value, tasks: Value) -> Vec<String> {
    vec![
        json!({"schema": "gentle-agents.activity/v1", "summary": summary, "tasks": tasks})
            .to_string(),
    ]
}

fn counts(running: u64, queued: u64, waiting: u64, finished: u64) -> Value {
    json!({"running": running, "queued": queued, "waiting": waiting, "finished": finished})
}

fn task(id: &str, status: &str, extra: Value) -> Value {
    let mut summary = json!({
        "id": id, "agent": "researcher", "label": id, "prompt": "p", "status": status,
        "createdAt": "2026-09-22T10:00:00.000Z", "turns": 1, "toolCalls": 0
    });
    for (key, value) in extra.as_object().unwrap() {
        summary[key] = value.clone();
    }
    json!({"summary": summary, "thread": {"version": 2, "dropped": 0, "items": [{"kind": "text", "text": "Working on it."}]}})
}

fn apply(state: ChatState, key: &str, lines: Vec<String>, now: Option<u64>) -> ChatState {
    reduce_chat_at(state, &widget_event(key, Some(lines)), now)
}

fn statuses(state: &ChatState) -> Vec<(&str, HelperStatus)> {
    state
        .helpers
        .tasks
        .iter()
        .map(|t| (t.id.as_str(), t.status))
        .collect()
}

const NOW: u64 = 1_790_074_800_000; // 2026-09-22T11:00:00.000Z

#[test]
fn a_new_chat_has_no_helpers() {
    assert_eq!(
        ChatState::default().helpers.summary,
        HelpersSummary::default()
    );
    assert!(ChatState::default().helpers.tasks.is_empty());
}

#[test]
fn a_frame_sets_summary_and_tasks_ordered_running_waiting_queued_finished() {
    let tasks = json!([
        task("q", "queued", json!({})),
        task("d", "done", json!({"endedAt": "2026-09-22T10:05:00.000Z"})),
        task("w", "waiting", json!({})),
        task("r", "running", json!({})),
    ]);
    let state = apply(
        ChatState::default(),
        "gentle-agents",
        frame(counts(1, 1, 1, 1), tasks),
        None,
    );
    assert_eq!(
        state.helpers.summary,
        HelpersSummary {
            running: 1,
            queued: 1,
            waiting: 1,
            finished: 1
        }
    );
    let order: Vec<_> = state.helpers.tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(order, ["r", "w", "q", "d"]);
}

#[test]
fn finished_tasks_are_ordered_by_end_time_newest_first() {
    let tasks = json!([
        task(
            "old",
            "done",
            json!({"endedAt": "2026-09-22T10:01:00.000Z"})
        ),
        task(
            "new",
            "failed",
            json!({"endedAt": "2026-09-22T10:09:00.000Z"})
        ),
        task(
            "mid",
            "cancelled",
            json!({"endedAt": "2026-09-22T10:05:00.000Z"})
        ),
    ]);
    let state = apply(
        ChatState::default(),
        "gentle-agents",
        frame(counts(0, 0, 0, 3), tasks),
        None,
    );
    let order: Vec<_> = state.helpers.tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(order, ["new", "mid", "old"]);
}

#[test]
fn a_task_dropped_by_gentle_agents_is_kept_as_done_stamped_with_the_arrival_time() {
    let running = apply(
        ChatState::default(),
        "gentle-agents",
        frame(
            counts(1, 0, 0, 0),
            json!([task("t1", "running", json!({}))]),
        ),
        None,
    );
    let state = apply(
        running,
        "gentle-agents",
        frame(counts(0, 0, 0, 1), json!([])),
        Some(NOW),
    );
    assert_eq!(
        state.helpers.summary,
        HelpersSummary {
            running: 0,
            queued: 0,
            waiting: 0,
            finished: 1
        }
    );
    assert_eq!(state.helpers.tasks.len(), 1);
    let kept = &state.helpers.tasks[0];
    assert_eq!(
        (kept.id.as_str(), kept.status, kept.ended_at_ms),
        ("t1", HelperStatus::Done, Some(NOW))
    );
    assert_eq!(kept.thread.items.len(), 1, "its thread survives");
}

#[test]
fn without_a_clock_the_promoted_task_simply_has_no_end_time() {
    let running = reduce_chat(
        ChatState::default(),
        &widget_event(
            "gentle-agents",
            Some(frame(
                counts(1, 0, 0, 0),
                json!([task("t1", "running", json!({}))]),
            )),
        ),
    );
    let state = reduce_chat(
        running,
        &widget_event("gentle-agents", Some(frame(counts(0, 0, 0, 1), json!([])))),
    );
    assert_eq!(statuses(&state), [("t1", HelperStatus::Done)]);
    assert_eq!(state.helpers.tasks[0].ended_at_ms, None);
}

#[test]
fn queued_and_waiting_tasks_are_promoted_to_done_too() {
    let both = frame(
        counts(0, 1, 1, 0),
        json!([
            task("q", "queued", json!({})),
            task("w", "waiting", json!({}))
        ]),
    );
    let state = apply(ChatState::default(), "gentle-agents", both, None);
    let state = apply(
        state,
        "gentle-agents",
        frame(counts(0, 0, 0, 2), json!([])),
        Some(NOW),
    );
    assert!(state
        .helpers
        .tasks
        .iter()
        .all(|t| t.status == HelperStatus::Done));
}

#[test]
fn an_existing_end_time_is_not_overwritten_on_promotion() {
    let first = frame(
        counts(1, 0, 0, 0),
        json!([task(
            "t",
            "running",
            json!({"endedAt": "2026-09-22T10:30:00.000Z"})
        )]),
    );
    let state = apply(ChatState::default(), "gentle-agents", first, None);
    let state = apply(
        state,
        "gentle-agents",
        frame(counts(0, 0, 0, 1), json!([])),
        Some(NOW),
    );
    assert_eq!(state.helpers.tasks[0].ended_at_ms, Some(1_790_073_000_000)); // 10:30:00Z
}

#[test]
fn a_terminal_status_is_never_rewritten_when_the_task_is_dropped() {
    let first = frame(
        counts(0, 0, 0, 2),
        json!([
            task(
                "f",
                "failed",
                json!({"endedAt": "2026-09-22T10:10:00.000Z", "error": "boom"})
            ),
            task(
                "c",
                "cancelled",
                json!({"endedAt": "2026-09-22T10:20:00.000Z"})
            ),
        ]),
    );
    let state = apply(ChatState::default(), "gentle-agents", first, None);
    let state = apply(
        state,
        "gentle-agents",
        frame(counts(0, 0, 0, 2), json!([])),
        Some(NOW),
    );
    assert_eq!(
        statuses(&state),
        [("c", HelperStatus::Cancelled), ("f", HelperStatus::Failed)]
    );
    assert_eq!(state.helpers.tasks[1].error.as_deref(), Some("boom"));
}

#[test]
fn a_task_that_reappears_replaces_its_retained_record() {
    let running = apply(
        ChatState::default(),
        "gentle-agents",
        frame(
            counts(1, 0, 0, 0),
            json!([task("t1", "running", json!({"turns": 0}))]),
        ),
        None,
    );
    let dropped = apply(
        running,
        "gentle-agents",
        frame(counts(0, 0, 0, 1), json!([])),
        Some(NOW),
    );
    assert_eq!(dropped.helpers.tasks[0].status, HelperStatus::Done);
    let back = apply(
        dropped,
        "gentle-agents",
        frame(
            counts(1, 0, 0, 0),
            json!([task("t1", "running", json!({"turns": 3}))]),
        ),
        Some(NOW + 1),
    );
    assert_eq!(back.helpers.tasks.len(), 1);
    assert_eq!(back.helpers.tasks[0].status, HelperStatus::Running);
    assert_eq!(back.helpers.tasks[0].ended_at_ms, None);
    assert_eq!(back.helpers.tasks[0].turns, 3);
}

#[test]
fn a_malformed_frame_keeps_the_previous_helpers() {
    let good = apply(
        ChatState::default(),
        "gentle-agents",
        frame(counts(1, 0, 0, 0), json!([task("t", "running", json!({}))])),
        None,
    );
    let after = apply(
        good.clone(),
        "gentle-agents",
        vec!["{not valid json".into()],
        Some(NOW),
    );
    assert_eq!(after.helpers, good.helpers);
}

#[test]
fn other_widget_keys_are_ignored() {
    let state = apply(
        ChatState::default(),
        "some-other-extension",
        frame(counts(9, 9, 9, 9), json!([task("t", "running", json!({}))])),
        None,
    );
    assert_eq!(state.helpers, ChatState::default().helpers);
}

#[test]
fn an_explicit_clear_zeroes_the_counters_but_finished_work_stays_listed() {
    let running = apply(
        ChatState::default(),
        "gentle-agents",
        frame(counts(1, 0, 0, 0), json!([task("t", "running", json!({}))])),
        None,
    );
    for clear in [
        widget_event("gentle-agents", None),
        widget_event("gentle-agents", Some(vec![])),
    ] {
        let state = reduce_chat_at(running.clone(), &clear, Some(NOW));
        assert_eq!(state.helpers.summary, HelpersSummary::default());
        assert_eq!(statuses(&state), [("t", HelperStatus::Done)]);
    }
}

#[test]
fn non_string_widget_lines_are_skipped() {
    let raw = json!({"type": "extension_ui_request", "id": "u", "method": "setWidget", "widgetKey": "gentle-agents",
        "widgetLines": [1, null, frame(counts(1, 0, 0, 0), json!([task("t", "running", json!({}))]))[0]]});
    let Decoded::Event(event) = decode_line(&raw.to_string()) else {
        panic!("event")
    };
    let state = reduce_chat(ChatState::default(), &event);
    assert_eq!(statuses(&state), [("t", HelperStatus::Running)]);
}

#[test]
fn the_recorded_gentle_agents_stream_folds_as_the_reference_app_does() {
    let mut state = ChatState::default();
    let mut seen = Vec::new();
    for line in FIXTURE.lines() {
        if let Decoded::Event(event) = decode_line(line) {
            state = reduce_chat_at(state, &event, Some(NOW));
            seen.push((
                state.helpers.summary,
                statuses(&state)
                    .iter()
                    .map(|(id, s)| format!("{id}:{s:?}"))
                    .collect::<Vec<_>>(),
            ));
        }
    }
    // Frame 1: one running, one queued; the malformed frame and the other
    // extension's widget change nothing; frame 4 has task-1 done, task-2 waiting.
    assert_eq!(seen[0].1, ["task-1:Running", "task-2:Queued"]);
    assert_eq!(
        seen[2], seen[1],
        "a malformed frame keeps the previous state"
    );
    assert_eq!(seen[4], seen[3], "another extension's widget is ignored");
    assert_eq!(
        seen[3].0,
        HelpersSummary {
            running: 0,
            queued: 0,
            waiting: 1,
            finished: 1
        }
    );
    assert_eq!(seen[3].1, ["task-2:Waiting", "task-1:Done"]);
}
