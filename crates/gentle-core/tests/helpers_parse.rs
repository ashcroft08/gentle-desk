//! Decoding `gentle-agents.activity/v1`, ported from the reference app's
//! `helpersActivity.test.ts`.

use gentle_core::helpers::{
    parse_helpers_activity, HelperStatus, HelperThreadItem, HelpersActivity, HelpersSummary,
};
use serde_json::json;

fn lines(value: serde_json::Value) -> Vec<String> {
    vec![value.to_string()]
}

fn payload(tasks: serde_json::Value) -> Vec<String> {
    lines(json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": 1, "queued": 0, "waiting": 0, "finished": 0},
        "tasks": tasks,
    }))
}

fn task(summary: serde_json::Value) -> serde_json::Value {
    json!({"summary": summary, "thread": {"version": 0, "dropped": 0, "items": []}})
}

fn minimal(id: &str) -> serde_json::Value {
    json!({"id": id, "status": "running", "createdAt": "2026-09-22T10:00:00.000Z"})
}

const CREATED_MS: u64 = 1_790_071_200_000; // 2026-09-22T10:00:00.000Z (checked with node Date.parse)

#[test]
fn a_valid_payload_decodes_into_typed_state() {
    let frame = json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": 1, "queued": 1, "waiting": 0, "finished": 0},
        "tasks": [{
            "summary": {
                "id": "task-1", "agent": "researcher", "label": "Research the API",
                "prompt": "Look into the auth flow", "status": "running",
                "createdAt": "2026-09-22T10:00:00.000Z", "startedAt": "2026-09-22T10:00:01.000Z",
                "lastStep": "Reading rpc.md", "turns": 2, "toolCalls": 1, "error": "none yet"
            },
            "thread": {"version": 2, "dropped": 1, "items": [
                {"kind": "text", "text": "Starting research."},
                {"kind": "thinking", "text": "hmm"},
                {"kind": "note", "text": "fyi"},
                {"kind": "tool", "callId": "call-1", "name": "Read", "args": {"path": "rpc.md"},
                 "output": "...", "running": false, "isError": false}
            ]}
        }]
    });
    let activity = parse_helpers_activity(&lines(frame)).unwrap();
    assert_eq!(
        activity.summary,
        HelpersSummary {
            running: 1,
            queued: 1,
            waiting: 0,
            finished: 0
        }
    );
    let task = &activity.tasks[0];
    assert_eq!(
        (task.id.as_str(), task.agent.as_str()),
        ("task-1", "researcher")
    );
    assert_eq!(task.label, "Research the API");
    assert_eq!(task.prompt, "Look into the auth flow");
    assert_eq!(task.status, HelperStatus::Running);
    assert_eq!(task.created_at_ms, CREATED_MS);
    assert_eq!(task.started_at_ms, Some(CREATED_MS + 1000));
    assert_eq!(task.ended_at_ms, None);
    assert_eq!(task.last_step.as_deref(), Some("Reading rpc.md"));
    assert_eq!((task.turns, task.tool_calls), (2, 1));
    assert_eq!(task.error.as_deref(), Some("none yet"));
    assert_eq!((task.thread.version, task.thread.dropped), (2, 1));
    assert_eq!(
        task.thread.items[0],
        HelperThreadItem::Text("Starting research.".into())
    );
    assert_eq!(
        task.thread.items[1],
        HelperThreadItem::Thinking("hmm".into())
    );
    assert_eq!(task.thread.items[2], HelperThreadItem::Note("fyi".into()));
    assert_eq!(
        task.thread.items[3],
        HelperThreadItem::Tool {
            call_id: "call-1".into(),
            name: "Read".into(),
            args: json!({"path": "rpc.md"}),
            output: json!("..."),
            running: Some(false),
            is_error: Some(false),
        }
    );
}

#[test]
fn every_helper_status_is_recognized() {
    let statuses = [
        "queued",
        "running",
        "waiting",
        "done",
        "failed",
        "cancelled",
    ];
    let tasks: Vec<_> = statuses
        .iter()
        .enumerate()
        .map(|(i, s)| task(json!({"id": format!("t{i}"), "status": s, "createdAt": "2026-09-22T10:00:00.000Z"})))
        .collect();
    let activity = parse_helpers_activity(&payload(json!(tasks))).unwrap();
    let decoded: Vec<_> = activity.tasks.iter().map(|t| t.status).collect();
    assert_eq!(
        decoded,
        [
            HelperStatus::Queued,
            HelperStatus::Running,
            HelperStatus::Waiting,
            HelperStatus::Done,
            HelperStatus::Failed,
            HelperStatus::Cancelled
        ]
    );
}

#[test]
fn epoch_milliseconds_are_accepted_wherever_a_timestamp_is() {
    let mut summary = minimal("t");
    summary["createdAt"] = json!(CREATED_MS);
    summary["lastActivityAt"] = json!(CREATED_MS + 5000);
    let activity = parse_helpers_activity(&payload(json!([task(summary)]))).unwrap();
    assert_eq!(activity.tasks[0].created_at_ms, CREATED_MS);
    assert_eq!(
        activity.tasks[0].last_activity_at_ms,
        Some(CREATED_MS + 5000)
    );
}

#[test]
fn missing_numeric_fields_default_to_zero() {
    let frame = lines(json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": 2},
        "tasks": [task(minimal("t"))],
    }));
    let activity = parse_helpers_activity(&frame).unwrap();
    assert_eq!(
        activity.summary,
        HelpersSummary {
            running: 2,
            queued: 0,
            waiting: 0,
            finished: 0
        }
    );
    assert_eq!(
        (activity.tasks[0].turns, activity.tasks[0].tool_calls),
        (0, 0)
    );
}

#[test]
fn a_missing_summary_or_thread_becomes_empty() {
    let frame = lines(json!({
        "schema": "gentle-agents.activity/v1",
        "tasks": [{"summary": minimal("t")}],
    }));
    let activity = parse_helpers_activity(&frame).unwrap();
    assert_eq!(activity.summary, HelpersSummary::default());
    assert_eq!(activity.tasks[0].thread.items.len(), 0);
    assert_eq!(activity.tasks[0].thread.version, 0);
}

#[test]
fn unknown_fields_are_ignored() {
    let mut summary = minimal("t");
    summary["futureField"] = json!(true);
    let frame = lines(json!({
        "schema": "gentle-agents.activity/v1",
        "extra": 1,
        "summary": {"running": 0, "queued": 0, "waiting": 0, "finished": 0, "more": 9},
        "tasks": [{"summary": summary, "thread": {"version": 1, "dropped": 0, "x": 1,
            "items": [{"kind": "text", "text": "a", "extra": true}]}, "other": 1}],
    }));
    let activity = parse_helpers_activity(&frame).unwrap();
    assert_eq!(
        activity.tasks[0].thread.items,
        [HelperThreadItem::Text("a".into())]
    );
}

#[test]
fn unrecognized_or_incomplete_thread_items_are_dropped() {
    let frame = payload(
        json!([{"summary": minimal("t"), "thread": {"version": 1, "dropped": 0, "items": [
            {"kind": "hologram", "text": "?"},
            {"kind": "text"},
            {"kind": "tool", "name": "Read"},
            {"kind": "text", "text": "kept"},
            "not an object",
        ]}}]),
    );
    let activity = parse_helpers_activity(&frame).unwrap();
    assert_eq!(
        activity.tasks[0].thread.items,
        [HelperThreadItem::Text("kept".into())]
    );
}

#[test]
fn tasks_missing_required_fields_are_dropped() {
    let tasks = json!([
        task(json!({"status": "running", "createdAt": "2026-09-22T10:00:00.000Z"})),
        task(json!({"id": "no-status", "createdAt": "2026-09-22T10:00:00.000Z"})),
        task(json!({"id": "bad-status", "status": "exploded", "createdAt": "2026-09-22T10:00:00.000Z"})),
        task(json!({"id": "no-created", "status": "running"})),
        {"thread": {}},
        task(minimal("ok")),
    ]);
    let activity = parse_helpers_activity(&payload(tasks)).unwrap();
    let ids: Vec<_> = activity.tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["ok"]);
}

#[test]
fn unusable_frames_return_none_so_the_caller_keeps_its_state() {
    assert_eq!(
        parse_helpers_activity(&["{not valid json".to_owned()]),
        None
    );
    assert_eq!(parse_helpers_activity(&lines(json!({"tasks": []}))), None);
    assert_eq!(
        parse_helpers_activity(&lines(json!({"schema": "other/v1", "tasks": []}))),
        None
    );
    assert_eq!(parse_helpers_activity(&lines(json!([1, 2]))), None);
}

#[test]
fn no_lines_at_all_is_an_explicit_clear() {
    assert_eq!(
        parse_helpers_activity(&[]),
        Some(HelpersActivity::default())
    );
}

#[test]
fn multiple_lines_are_joined_before_parsing() {
    let whole = payload(json!([task(minimal("t"))])).remove(0);
    let (a, b) = whole.split_at(whole.len() / 2);
    let activity = parse_helpers_activity(&[a.to_owned(), b.to_owned()]).unwrap();
    assert_eq!(activity.tasks.len(), 1);
}

#[test]
fn out_of_range_or_negative_epochs_are_dropped_not_fatal() {
    // Beyond JavaScript's Date range (±8.64e15 ms), negative, and fractional garbage.
    for bad in [json!(9_000_000_000_000_000_u64), json!(-5), json!(1e300)] {
        let mut summary = minimal("t");
        summary["startedAt"] = bad;
        let activity = parse_helpers_activity(&payload(json!([task(summary)]))).unwrap();
        assert_eq!(activity.tasks[0].started_at_ms, None);
    }
}

#[test]
fn a_task_whose_required_created_at_is_out_of_range_is_dropped() {
    let mut summary = minimal("t");
    summary["createdAt"] = json!(9_000_000_000_000_000_u64);
    let activity = parse_helpers_activity(&payload(json!([task(summary)]))).unwrap();
    assert!(activity.tasks.is_empty());
}

#[test]
fn a_number_literal_that_overflows_makes_the_whole_frame_unusable() {
    // JavaScript turns `1e400` into Infinity and drops the field; serde_json
    // rejects the number, so the frame is malformed and the previous state stays.
    let raw = r#"{"schema":"gentle-agents.activity/v1","summary":{},"tasks":[{"summary":{"id":"t","status":"running","createdAt":"2026-09-22T10:00:00.000Z","startedAt":1e400}}]}"#;
    assert_eq!(parse_helpers_activity(&[raw.to_owned()]), None);
}
