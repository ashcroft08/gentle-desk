//! Summaries of pi session files.
//!
//! A session is an append-only JSONL file: a `session` header line followed by
//! entries. [`SessionScanner`] folds those lines one at a time into a
//! [`SessionSummary`], so a caller that already scanned a file can feed only
//! the bytes appended since (the incremental index in `gentle-backend` does).
//! Pure: no file system, no clock.
//!
//! A summary and a scanner both serialize, so a caller may persist its state
//! and resume scanning where it stopped after a restart.

use serde::{Deserialize, Serialize};

pub use crate::time::parse_iso_ms;

/// Most characters kept of a session name or first message: enough to search by
/// and to fill a list row, small enough that thousands of chats stay cheap in
/// memory and in the persisted cache.
pub const MAX_TITLE_CHARS: usize = 200;

/// What a session list shows for one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub cwd: String,
    /// Header timestamp, epoch milliseconds.
    pub created_ms: Option<u64>,
    /// Latest `session_info` name, trimmed; `None` if never set or cleared.
    pub name: Option<String>,
    /// Text of the first user message that has any.
    pub first_message: Option<String>,
    /// Every `message` entry, whatever its role.
    pub message_count: u32,
    /// Latest user/assistant message time, epoch milliseconds.
    pub last_activity_ms: Option<u64>,
}

/// Incremental line-by-line builder of a [`SessionSummary`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionScanner {
    summary: Option<SessionSummary>,
    /// The first parsable entry was not a `session` header: not a session file.
    rejected: bool,
}

impl SessionScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Folds one line in. Blank and malformed lines are skipped, like pi does.
    pub fn push_line(&mut self, line: &str) {
        if self.rejected || line.trim().is_empty() {
            return;
        }
        let Ok(serde_json::Value::Object(entry)) = serde_json::from_str(line) else {
            return;
        };
        let kind = entry.get("type").and_then(|v| v.as_str());
        let Some(summary) = self.summary.as_mut() else {
            if kind == Some("session") {
                self.summary = Some(SessionSummary {
                    id: str_field(&entry, "id").unwrap_or_default().to_owned(),
                    cwd: str_field(&entry, "cwd").unwrap_or_default().to_owned(),
                    created_ms: str_field(&entry, "timestamp").and_then(parse_iso_ms),
                    name: None,
                    first_message: None,
                    message_count: 0,
                    last_activity_ms: None,
                });
            } else {
                self.rejected = true;
            }
            return;
        };
        match kind {
            Some("session_info") => {
                summary.name = str_field(&entry, "name")
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(bounded);
            }
            Some("message") => fold_message(summary, &entry),
            _ => {}
        }
    }

    /// The summary so far; `None` until a valid header was seen.
    pub fn summary(&self) -> Option<SessionSummary> {
        self.summary.clone()
    }
}

fn fold_message(summary: &mut SessionSummary, entry: &serde_json::Map<String, serde_json::Value>) {
    summary.message_count += 1;
    let Some(message) = entry.get("message").and_then(|m| m.as_object()) else {
        return;
    };
    let role = message.get("role").and_then(|r| r.as_str());
    if !message.contains_key("content") || !matches!(role, Some("user" | "assistant")) {
        return;
    }
    // The message's own epoch-ms timestamp beats the entry's ISO one.
    let activity = message
        .get("timestamp")
        .and_then(|t| t.as_u64())
        .or_else(|| str_field(entry, "timestamp").and_then(parse_iso_ms));
    if let Some(activity) = activity {
        summary.last_activity_ms = Some(
            summary
                .last_activity_ms
                .map_or(activity, |a| a.max(activity)),
        );
    }
    if role == Some("user") && summary.first_message.is_none() {
        let text = message_text(&message["content"]);
        if !text.is_empty() {
            summary.first_message = Some(bounded(&text));
        }
    }
}

/// At most [`MAX_TITLE_CHARS`] characters, cut on a character boundary.
fn bounded(text: &str) -> String {
    text.chars().take(MAX_TITLE_CHARS).collect()
}

/// A string content as is; a block list as its `text` blocks joined by a space.
fn message_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

fn str_field<'a>(
    entry: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    entry.get(key).and_then(|v| v.as_str())
}
