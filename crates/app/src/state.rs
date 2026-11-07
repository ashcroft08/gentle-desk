//! The data the UI renders.

use gentle_core::chat::ChatState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

/// One row of the session list, flattened for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionItem {
    /// Session file path, passed back to `switch_session`.
    pub path: String,
    pub title: String,
    pub cwd: String,
    pub modified_ms: u64,
    pub message_count: u32,
}

/// A transient message from an extension (`notify`), shown outside the thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: u64,
    pub level: NoticeLevel,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    pub chat: ChatState,
    pub notices: Vec<Notice>,
    pub sessions: Vec<SessionItem>,
    pub current_session: Option<String>,
    /// The helper whose thread is open in the panel, if any.
    pub selected_helper: Option<String>,
    /// Ids are never reused, so a dismissed notice cannot be confused with a new one.
    pub(crate) next_notice_id: u64,
}
