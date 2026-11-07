//! Folding a `setWidget` request into the helpers state.

use serde_json::Value;

use super::merge::merge_tasks;
use super::parse::parse_helpers_activity;
use super::types::HelpersActivity;

/// The one widget key gentle-agents publishes its activity under.
const WIDGET_KEY: &str = "gentle-agents";

/// Applies a raw `extension_ui_request` (method `setWidget`) to `current`.
///
/// Other widget keys are other extensions' business and change nothing. A
/// frame that cannot be decoded changes nothing either. `now_ms` stamps tasks
/// that gentle-agents stopped reporting (see [`merge_tasks`]).
pub fn fold_widget(current: HelpersActivity, raw: &Value, now_ms: Option<u64>) -> HelpersActivity {
    if raw.get("widgetKey").and_then(Value::as_str) != Some(WIDGET_KEY) {
        return current;
    }
    // Absent `widgetLines` is the documented way to clear a widget; entries
    // that are not strings are skipped.
    let lines: Vec<String> = raw
        .get("widgetLines")
        .and_then(Value::as_array)
        .map(|lines| {
            lines
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let Some(parsed) = parse_helpers_activity(&lines) else {
        return current;
    };
    HelpersActivity {
        summary: parsed.summary,
        tasks: merge_tasks(&current.tasks, parsed.tasks, now_ms),
    }
}
