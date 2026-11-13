//! The data the UI renders.

use std::collections::BTreeMap;

use gentle_core::chat::{ChatMessage, ChatRole, ChatState};

use crate::chats::{ChatActivity, ChatId, Chats};
use crate::list_status::ListStatus;

/// One model pi offers, flattened for the chip's dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    pub provider: String,
    pub model_id: String,
    pub name: String,
    /// Whether the model does reasoning, from pi's own `reasoning` flag.
    pub reasoning: bool,
}

/// Where the model list stands: the chip renders each state differently.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ModelsStatus {
    /// Not asked yet.
    #[default]
    Unknown,
    /// Asked, no reply yet.
    Loading,
    /// The reply listed models.
    Ready,
    /// The reply listed none.
    Empty,
    /// The ask failed; the string is what the child said.
    Failed(String),
}

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

/// The one thing the user can do about a notice, when the notice is about
/// something the app can fix by itself (S3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeAction {
    /// Start that chat's child again and reopen the chat.
    RestartChat(ChatId),
}

/// A transient message from an extension (`notify`), or the report of a chat's
/// death, shown outside the thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: u64,
    pub level: NoticeLevel,
    pub text: String,
    /// The action that fixes what the notice reports, if there is one.
    pub action: Option<NoticeAction>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    /// The single chat the one-child shell renders. Untouched by the per-chat
    /// model, which keeps its own states in [`AppState::chats`].
    pub chat: ChatState,
    /// One state per chat, plus the foreground one. Empty in a shell that runs
    /// one child for every chat it does not have.
    pub chats: Chats,
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
    /// Whether `sessions` is the whole list or a partial read of the first pass.
    /// The sidebar's empty state depends on it, so it starts `Loading`.
    pub list_status: ListStatus,
    pub current_session: Option<String>,
    /// The helper whose thread is open in the panel, if any.
    pub selected_helper: Option<String>,
    /// Ids are never reused, so a dismissed notice cannot be confused with a new one.
    pub(crate) next_notice_id: u64,
    /// What pi offers, for the chip's dropdown. Cleared when the foreground
    /// chat changes: a list is per child, not global.
    pub models: Vec<ModelChoice>,
    pub models_status: ModelsStatus,
    /// The session's current model, from `get_state` / a `set_model` reply.
    pub model: Option<ModelChoice>,
    /// The session's current thinking level; `None` is unknown, not off.
    pub thinking_level: Option<String>,
    /// The levels the current model supports, from
    /// `get_available_thinking_levels`.
    pub thinking_levels: Vec<String>,
}

impl AppState {
    /// The activity a sidebar row shows for the chat of the session stored at
    /// `path` (S4). The rows ask one path at a time — the ones being drawn —
    /// and a chat the app has never heard of is idle, so every path in the
    /// list has an answer without the app tracking every session file.
    pub fn activity_of_session(&self, path: &str) -> ChatActivity {
        self.chats
            .activity(&ChatId::of_session(path))
            .unwrap_or_default()
    }

    /// The activities of the rows' badges, as one plain snapshot keyed by the
    /// path of the session they belong to: only the chats that are not idle.
    /// Rows do not find themselves in it the moment they are idle, so it stays
    /// small however many sessions the list lists, and the sidebar reads no
    /// further than the activities.
    pub fn activity_by_path(&self) -> BTreeMap<String, ChatActivity> {
        self.chats
            .ids()
            .iter()
            .filter_map(|chat| {
                let activity = self
                    .chats
                    .activity(chat)
                    .filter(|a| *a != ChatActivity::Idle)?;
                Some((chat.session_path()?.to_owned(), activity))
            })
            .collect()
    }
}

/// Appends what the user typed as their side of the thread.
///
/// Ids stay contiguous — `msg-<index>` — the same scheme the reducer uses when it
/// opens and closes an assistant message, so the whole thread keeps one id space
/// whichever chat it belongs to.
pub(crate) fn push_user_message(state: &mut ChatState, text: &str) {
    state.messages.push(ChatMessage {
        id: format!("msg-{}", state.messages.len()),
        role: ChatRole::User,
        text: text.to_owned(),
        streaming: false,
        error: None,
    });
}
