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

impl SessionItem {
    /// The project this chat belongs to: the last segment of its working directory.
    pub fn project(&self) -> &str {
        let separators = [char::from(47u8), char::from(92u8)]; // / and the backslash
        self.cwd
            .trim_end_matches(separators)
            .rsplit(separators)
            .next()
            .unwrap_or("")
    }
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
    /// Set through `Input::SessionsLoaded` or `Input::SessionsPrepared`, which
    /// keep the search data in step.
    pub sessions: Vec<SessionItem>,
    /// What the user typed in the sidebar's search box.
    pub session_query: String,
    /// Folded title and project of each session, parallel to `sessions`.
    pub(crate) search_keys: Vec<String>,
    /// Indices into `sessions` of the chats matching `session_query`.
    pub(crate) visible: Vec<usize>,
    pub current_session: Option<String>,
    /// The helper whose thread is open in the panel, if any.
    pub selected_helper: Option<String>,
    /// Ids are never reused, so a dismissed notice cannot be confused with a new one.
    pub(crate) next_notice_id: u64,
}
