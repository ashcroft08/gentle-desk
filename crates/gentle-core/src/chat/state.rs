//! Chat state produced by folding decoded RPC events.
//!
//! Pure data: no I/O, no clocks, no id generation beyond the deterministic
//! `msg-<index>` scheme. The `ui` layer renders this and the `app` layer feeds
//! it lines read from `gentle-shell --mode rpc`.

use crate::helpers::HelpersActivity;

/// Who wrote a chat message.
///
/// Only the two roles a chat thread displays are modeled: the reducer opens
/// assistant messages from `message_start` and tool results never become
/// messages (a tool-only assistant turn is dropped instead).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// One rendered message of the open thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// `msg-<index in state.messages>`, so ids stay contiguous: a dropped
    /// message frees its id for the next one.
    pub id: String,
    pub role: ChatRole,
    /// Accumulated text while streaming, the authoritative text once closed.
    pub text: String,
    /// True between `message_start` and `message_end`.
    pub streaming: bool,
    /// Set when the provider failed this turn (`stopReason: "error"`): the
    /// text is then the provider's own and must be shown as a failure.
    pub error: Option<String>,
}

/// A blocking extension dialog waiting for an answer.
///
/// The variant is the wire `method`; `timeout` is not carried because it only
/// affects when the extension itself gives up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Select {
        id: String,
        title: String,
        options: Vec<String>,
    },
    Confirm {
        id: String,
        title: String,
        message: String,
    },
    Input {
        id: String,
        title: String,
        placeholder: Option<String>,
    },
    Editor {
        id: String,
        title: String,
        prefill: Option<String>,
    },
}

/// Everything a chat window shows for one open session.
///
/// `Default` is the initial state of a freshly opened chat.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    /// Driven by `agent_start` / `agent_end` / `agent_settled`.
    pub working: bool,
    /// Blocking extension dialogs in arrival order, oldest first.
    pub pending_dialogs: Vec<Dialog>,
    /// Monotonic count of thinking / toolcall / tool_execution activity.
    pub activity: u64,
    /// Last `extension_error` the child reported.
    pub last_error: Option<String>,
    /// Delegated helpers of this chat, from the `gentle-agents` widget.
    pub helpers: HelpersActivity,
}
