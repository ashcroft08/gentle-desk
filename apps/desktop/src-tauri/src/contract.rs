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
    /// Start the child of a chat that died again and reopen it (S3).
    RestartChat {
        chat_id: String,
    },
    DismissNotice {
        id: u32,
    },
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
/// it, the id the sidebar passes back), the notices, and the per-chat activity
/// the list rows show. The push size is the notices plus the chats that are not
/// idle, never the whole `AppState` (S2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ViewState {
    pub foreground_chat: Option<String>,
    pub notices: Vec<NoticeView>,
    pub activity: Vec<ChatActivityView>,
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
