//! Typed state of the `gentle-agents.activity/v1` widget.
//!
//! Timestamps are epoch milliseconds: the wire carries ISO strings or numbers
//! and both are normalized to one representation.

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperStatus {
    Queued,
    Running,
    Waiting,
    Done,
    Failed,
    Cancelled,
}

/// The counters gentle-agents reports; they include tasks no longer listed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HelpersSummary {
    pub running: u64,
    pub queued: u64,
    pub waiting: u64,
    pub finished: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HelperThreadItem {
    Text(String),
    Thinking(String),
    Note(String),
    Tool {
        call_id: String,
        name: String,
        args: Value,
        output: Value,
        running: Option<bool>,
        is_error: Option<bool>,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HelperThread {
    pub version: u64,
    pub dropped: u64,
    pub items: Vec<HelperThreadItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HelperTask {
    pub id: String,
    pub agent: String,
    pub label: String,
    pub prompt: String,
    pub status: HelperStatus,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub ended_at_ms: Option<u64>,
    pub last_step: Option<String>,
    pub last_activity_at_ms: Option<u64>,
    pub turns: u64,
    pub tool_calls: u64,
    pub error: Option<String>,
    pub thread: HelperThread,
}

/// What the helpers panel shows for one chat. `Default` is "no helpers".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HelpersActivity {
    pub summary: HelpersSummary,
    pub tasks: Vec<HelperTask>,
}
