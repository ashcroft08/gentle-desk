//! Rust-to-web contract types.
//!
//! The TypeScript declarations that the Svelte view compiles against are
//! GENERATED from the definitions below into
//! `apps/desktop/src/contract/bindings.ts`. A change on the Rust side must
//! land together with a regenerated file; `tests/contract_bindings.rs` fails
//! the build when the committed file no longer matches.
//!
//! These types are the SLICES the web state renders (S2 of the migration),
//! never the whole `AppState`: the mapping to and from `gentle-app` lives in
//! `bridge`, so nothing here derives on a `crates/app` type.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChatBadge {
    pub chat_id: String,
    pub activity: ChatActivityMark,
    pub unread: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ChatActivityMark {
    Working,
    Queued,
    Opening,
    NewReply,
    NeedsAttention,
    Ended,
}

/// One chat that has something to show, as a list row sees it: the id the app
/// names it by (the same text `ViewState::foreground_chat` carries) and its
/// mark. Only chats that are not idle are listed, so the push stays as small as
/// the working set, and the mark itself is chosen by the rule `crates/runtime`
/// owns ([`gentle_runtime::badge`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatActivityView {
    pub chat_id: String,
    pub activity: ChatActivityMark,
}

/// What the user asked for, as the web side sends it. Only the intents the
/// current slice needs; the rest are added when their views arrive. The
/// explicit mapping into `gentle-app`'s [`gentle_app::Intent`] keeps this
/// enum free of every `crates/app` type.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum UiIntent {
    SendPrompt {
        text: String,
    },
    Abort,
    NewChat,
    /// Make that chat the foreground one, asking the host for its child.
    OpenChat {
        chat_id: String,
    },
    /// The same for a chat that did not exist before.
    OpenNewChat,
    /// Start a NEW chat in the workspace at `path`: what the header's New
    /// session button (D2) and a workspace's own new-chat icon (D3) send. The
    /// web side decides which workspace is the current one and names it; the
    /// chat started here is the same new chat `OpenNewChat` starts, only in
    /// that folder, and never the legacy single-child `NewChat`.
    NewChatInWorkspace {
        path: String,
    },
    /// Fold or unfold the chats of the workspace at `path` in the sidebar (D8):
    /// the state is remembered in the app config, so it survives the run.
    SetWorkspaceCollapsed {
        path: String,
        collapsed: bool,
    },
    /// Start the child of a chat that died again and reopen it (S3).
    RestartChat {
        chat_id: String,
    },
    DismissNotice {
        id: u32,
    },
}

/// How complete the session list is, matching `gentle_app::ListStatus` one to
/// one. `Loading` carries how far the first pass got, so the sidebar can say it
/// is still reading; only `Complete` lets an empty list call itself empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum ListStatusView {
    Loading { done: u32, total: u32 },
    Complete,
}

/// One row of the session list, flattened for display. Rows are served a
/// WINDOW at a time ([`SessionsWindow`], S3); the whole list is never pushed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SessionRowView {
    /// The session file path: the id `OpenChat` takes back, which is exactly
    /// what `gentle_app::ChatId::of_session` builds a stored chat's id from.
    pub id: String,
    pub title: String,
    pub project: String,
    pub message_count: u32,
    /// How long ago the session was last written, as the text the sidebar shows
    /// (the rule is [`gentle_runtime::time_ago`]).
    pub modified_ago: String,
}

/// One workspace row of the flattened tree: the folder the user works in, with
/// the state the sidebar draws and whether new chats can start there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WorkspaceRowView {
    /// The folder identity two spellings share
    /// ([`gentle_runtime::app_config::folder_key`]).
    pub key: String,
    /// The name extended with the parent segments it takes to tell two
    /// workspaces with the same name apart.
    pub label: String,
    /// The canonical spelling of the folder.
    pub path: String,
    /// The last segment of the path.
    pub name: String,
    /// Whether the chats of the workspace are folded away (D8).
    pub collapsed: bool,
    /// How many chats the workspace holds, folded or not.
    pub chat_count: u32,
    /// Whether the folder is there right now, so new chats can start in it.
    pub available: bool,
    /// Why a folder that is not available cannot host a chat.
    pub reason: Option<String>,
}

/// One row of the flattened workspace tree the sidebar draws: a workspace
/// header, then - only when the workspace is not collapsed - its chats.
///
/// The two kinds share one row height on the web side, so the tree is one
/// virtualized list of rows rather than a nested one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind")]
pub enum SessionTreeRow {
    Workspace(WorkspaceRowView),
    Chat(SessionRowView),
}

/// One page of the workspace tree as `sessions_window` answers it (S3): the rows
/// `[offset, offset + length)` of the flattened tree, and how many rows the tree
/// has in total - workspace headers included - so the sidebar knows how far it
/// can scroll.
///
/// The length is capped ([`MAX_WINDOW`](crate::sessions::window::MAX_WINDOW)),
/// so this type never carries the whole list of a 50 000-chat home.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SessionsWindow {
    /// The count of the flattened rows: headers and chats, collapsed groups
    /// skipped, never the page's length.
    pub total: u32,
    pub rows: Vec<SessionTreeRow>,
}

/// Who wrote one message of the thread. The two roles a chat thread displays,
/// matching `gentle_core`'s chat roles one to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum MessageRole {
    User,
    Assistant,
}

/// One message of the foreground chat's thread, flattened for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct MessageView {
    pub role: MessageRole,
    pub text: String,
}

/// The level of a notice, matching `gentle_app`'s notice kinds one to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

/// One notice, flattened for display: id, text, level. The id the dismiss
/// intent refers back to. The app lays notices out newest last and keeps the
/// most recent ones, which the view reflects as received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct NoticeView {
    pub id: u32,
    pub text: String,
    pub level: NoticeLevel,
}

/// The state the current views render: the foreground chat (as the app names
/// it, the id the sidebar passes back) with its thread, the notices, the
/// per-chat activity the list rows show, and the session list as a status plus
/// a revision. The rows themselves are not here: the sidebar asks
/// `sessions_window` for the page it draws (S3).
///
/// The push size is the notices, the chats that are not idle and the tail of one
/// thread, never the whole `AppState` and never the whole list (S2).
///
/// A chat's stderr reason lines are not a slice of their own: the app keeps them
/// in a bounded ring but exposes them only as the reason of the death notice, so
/// they reach the web side inside [`NoticeView::text`] and nowhere else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ViewState {
    pub foreground_chat: Option<String>,
    /// How complete the session list is. It starts `Loading`, so the sidebar
    /// never claims there are no chats before the first pass ends.
    pub list_status: ListStatusView,
    /// Bumped every time the folded list or its status changes (wrapping), so
    /// the web side knows the page of rows it holds is stale and re-queries
    /// `sessions_window`. A push that does not touch the list leaves it alone.
    pub sessions_revision: u32,
    /// How many sessions the app holds, whatever the search box says: the
    /// unfiltered count, next to the filtered total each window answers with.
    pub total_sessions: u32,
    /// The foreground chat's messages, oldest first. Message text is the model's
    /// whole markdown reply, so the slice carries the last
    /// [`MESSAGES_PUSHED`](crate::view::MESSAGES_PUSHED) of them and never a
    /// whole long session.
    pub messages: Vec<MessageView>,
    pub notices: Vec<NoticeView>,
    pub activity: Vec<ChatActivityView>,
    /// What the user should be told about the app config file: a file that
    /// could not be used at startup is kept aside and the app starts with an
    /// empty config, and a file that could not be written leaves the change in
    /// memory only. `None` when the file was read, or was not there, and nothing
    /// failed.
    pub config_notice: Option<String>,
}

/// The state slice after each dispatch, emitted to the web side (typed
/// tauri-specta event). The name on the wire is kebab-case by `NAME` below.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StateChanged(pub ViewState);

// Manual `Event` implementation instead of the derive: the `derive` feature
// is not enabled (it would pull the proc-macro crate in); the trait needs
// only the wire name, every other method has a default.
impl tauri_specta::Event for StateChanged {
    const NAME: &'static str = "state-changed";
}

/// Sample contract command. Trivial by design: it exists so the generator
/// proves that command names, camel-cased argument names and return types are
/// typed over the same Rust definition, not re-declared by hand.
#[tauri::command]
#[specta::specta]
pub fn get_badge(chat_id: String) -> ChatBadge {
    ChatBadge {
        chat_id,
        activity: ChatActivityMark::Working,
        unread: false,
    }
}
