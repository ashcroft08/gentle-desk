//! Decoding of the widget's JSON lines.

use serde_json::{Map, Value};

use super::types::{
    HelperStatus, HelperTask, HelperThread, HelperThreadItem, HelpersActivity, HelpersSummary,
};
use crate::time::parse_iso_ms;

const SCHEMA: &str = "gentle-agents.activity/v1";

/// JavaScript's `Date` range: epoch values beyond it are not real timestamps.
const MAX_EPOCH_MS: f64 = 8.64e15;

/// Decodes the `widgetLines` of a `setWidget` request for `gentle-agents`.
///
/// Tolerant by design: unknown fields are ignored, a missing thread is empty,
/// a malformed task or thread item is dropped. No lines at all is the
/// documented way to clear a widget and decodes to "no helpers". `None` means
/// the frame is unusable (malformed JSON, another schema): the caller keeps
/// what it had instead of treating it as "no helpers".
pub fn parse_helpers_activity(lines: &[String]) -> Option<HelpersActivity> {
    if lines.is_empty() {
        return Some(HelpersActivity::default());
    }
    let Ok(Value::Object(frame)) = serde_json::from_str::<Value>(&lines.concat()) else {
        return None;
    };
    if str_of(&frame, "schema") != Some(SCHEMA) {
        return None;
    }
    let tasks = frame
        .get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| tasks.iter().filter_map(to_task).collect())
        .unwrap_or_default();
    Some(HelpersActivity {
        summary: to_summary(frame.get("summary")),
        tasks,
    })
}

fn str_of<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    map.get(key).and_then(Value::as_str)
}

fn owned(map: &Map<String, Value>, key: &str) -> Option<String> {
    str_of(map, key).map(str::to_owned)
}

fn count(map: &Map<String, Value>, key: &str) -> u64 {
    map.get(key).and_then(Value::as_u64).unwrap_or(0)
}

/// An ISO string or an epoch-ms number in range; anything else is dropped.
fn timestamp(map: &Map<String, Value>, key: &str) -> Option<u64> {
    match map.get(key)? {
        Value::String(text) => parse_iso_ms(text),
        Value::Number(number) => {
            let ms = number.as_f64()?;
            (ms.is_finite() && (0.0..=MAX_EPOCH_MS).contains(&ms)).then_some(ms as u64)
        }
        _ => None,
    }
}

fn to_summary(value: Option<&Value>) -> HelpersSummary {
    let Some(Value::Object(map)) = value else {
        return HelpersSummary::default();
    };
    HelpersSummary {
        running: count(map, "running"),
        queued: count(map, "queued"),
        waiting: count(map, "waiting"),
        finished: count(map, "finished"),
    }
}

fn to_status(text: &str) -> Option<HelperStatus> {
    Some(match text {
        "queued" => HelperStatus::Queued,
        "running" => HelperStatus::Running,
        "waiting" => HelperStatus::Waiting,
        "done" => HelperStatus::Done,
        "failed" => HelperStatus::Failed,
        "cancelled" => HelperStatus::Cancelled,
        _ => return None,
    })
}

fn to_task(value: &Value) -> Option<HelperTask> {
    let summary = value.get("summary")?.as_object()?;
    Some(HelperTask {
        id: owned(summary, "id")?,
        status: to_status(str_of(summary, "status")?)?,
        created_at_ms: timestamp(summary, "createdAt")?,
        agent: owned(summary, "agent").unwrap_or_default(),
        label: owned(summary, "label").unwrap_or_default(),
        prompt: owned(summary, "prompt").unwrap_or_default(),
        started_at_ms: timestamp(summary, "startedAt"),
        ended_at_ms: timestamp(summary, "endedAt"),
        last_step: owned(summary, "lastStep"),
        last_activity_at_ms: timestamp(summary, "lastActivityAt"),
        turns: count(summary, "turns"),
        tool_calls: count(summary, "toolCalls"),
        error: owned(summary, "error"),
        thread: to_thread(value.get("thread")),
    })
}

fn to_thread(value: Option<&Value>) -> HelperThread {
    let Some(Value::Object(map)) = value else {
        return HelperThread::default();
    };
    HelperThread {
        version: count(map, "version"),
        dropped: count(map, "dropped"),
        items: map
            .get("items")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(to_item).collect())
            .unwrap_or_default(),
    }
}

fn to_item(value: &Value) -> Option<HelperThreadItem> {
    let item = value.as_object()?;
    Some(match str_of(item, "kind")? {
        "text" => HelperThreadItem::Text(owned(item, "text")?),
        "thinking" => HelperThreadItem::Thinking(owned(item, "text")?),
        "note" => HelperThreadItem::Note(owned(item, "text")?),
        "tool" => HelperThreadItem::Tool {
            call_id: owned(item, "callId")?,
            name: owned(item, "name")?,
            args: item.get("args").cloned().unwrap_or(Value::Null),
            output: item.get("output").cloned().unwrap_or(Value::Null),
            running: item.get("running").and_then(Value::as_bool),
            is_error: item.get("isError").and_then(Value::as_bool),
        },
        _ => return None,
    })
}
