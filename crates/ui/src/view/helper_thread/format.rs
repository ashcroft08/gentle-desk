//! Turning a helper's thread items into short, readable text.

use serde_json::Value;

use crate::text::truncate;

/// How a tool call of a helper stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Failed,
    Ok,
}

/// Longest argument line, in characters.
const ARGS_CHARS: usize = 100;
/// Most lines and characters of a tool output shown.
const OUTPUT_LINES: usize = 12;
const OUTPUT_CHARS: usize = 600;

pub fn tool_status(running: Option<bool>, is_error: Option<bool>) -> ToolStatus {
    if running == Some(true) {
        ToolStatus::Running
    } else if is_error == Some(true) {
        ToolStatus::Failed
    } else {
        ToolStatus::Ok
    }
}

/// A tool call's arguments on one line, e.g. `path=rpc.md, recursive=true`.
pub fn args_line(args: &Value) -> String {
    let line = match args {
        Value::Null => String::new(),
        Value::Object(map) => map
            .iter()
            .map(|(key, value)| format!("{key}={}", one_line(value)))
            .collect::<Vec<_>>()
            .join(", "),
        other => one_line(other),
    };
    truncate(&line, ARGS_CHARS)
}

/// The first lines of a tool's output, bounded in size.
pub fn output_preview(output: &Value) -> String {
    let text = match output {
        Value::Null => return String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    let lines: Vec<&str> = text.lines().collect();
    let shown = if lines.len() > OUTPUT_LINES {
        format!("{}\n…", lines[..OUTPUT_LINES].join("\n"))
    } else {
        text
    };
    truncate(&shown, OUTPUT_CHARS)
}

/// A string as is, anything else as compact JSON; never more than one line.
fn one_line(value: &Value) -> String {
    let text = match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    text.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests;
