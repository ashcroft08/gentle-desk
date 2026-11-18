//! Chat state produced by folding decoded RPC events.
//!
//! Pure data: no I/O, no clocks, no id generation beyond the deterministic
//! `msg-<index>`, `think-<index>` and `tool-<toolCallId>` schemes. The `ui`
//! layer renders this and the `app` layer feeds it lines read from
//! `gentle-shell --mode rpc`.

use serde_json::Value;

use crate::helpers::HelpersActivity;

/// Who wrote a chat message.
///
/// Only the two roles a chat thread displays are modeled: the reducer opens
/// assistant messages from `message_start` and a tool result is never a message
/// (it is the tool row's own `result`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// One rendered message of the open thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// `msg-<how many text rows came before it>`, so ids stay contiguous over
    /// the text: a dropped message frees its id for the next one, and a thinking
    /// or tool row never shifts it.
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

/// One row of the open thread, in the order the child produced it.
///
/// This is the ONE transcript (`odd/tasks/chat-transcript.md`, L4): what a
/// thread draws and the only place a row lives. A text row is the
/// [`ChatMessage`] that was the whole transcript before the chat kept thinking
/// and tool calls, so its shape and its ids keep their meaning.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatItem {
    Text(ChatMessage),
    /// A reasoning block: `thinking_start` opens it, `thinking_delta` grows it
    /// and `thinking_end` closes it with the authoritative text.
    Thinking {
        /// `think-<how many reasoning blocks came before it>`.
        id: String,
        text: String,
        /// True between `thinking_start` and `thinking_end`.
        streaming: bool,
        /// The clock the block started at, when the caller supplied one.
        started_at_ms: Option<u64>,
        ended_at_ms: Option<u64>,
        /// How long the block ran, which is `None` when either stamp is: the
        /// `prompt-turn` fixture carries no timestamps, and a card must not
        /// invent the "Thought 1.2s" it has no evidence for.
        duration_ms: Option<u64>,
    },
    /// One tool call: the `toolcall_*` events announce it, the
    /// `tool_execution_*` events carry the arguments, the output so far and the
    /// result it ended with. Every payload is the child's own JSON and stays
    /// untyped here, because each tool's shape is its own.
    Tool {
        /// `tool-<toolCallId>`.
        id: String,
        /// The child's own id for the call, which every later event names.
        tool_call_id: String,
        tool_name: String,
        /// The call's arguments as the child sent them.
        args: Option<Value>,
        /// The output the tool had produced at its last update.
        partial_result: Option<Value>,
        /// The result the tool ended with.
        result: Option<Value>,
        /// The child's own verdict on the result.
        is_error: bool,
        /// True between `tool_execution_start` and `tool_execution_end`: the
        /// card's "Running" instead of its "Done".
        running: bool,
        /// The clock the execution started at, when the caller supplied one.
        started_at_ms: Option<u64>,
        ended_at_ms: Option<u64>,
        duration_ms: Option<u64>,
    },
}

impl ChatItem {
    /// The row's own id, whichever shape it has.
    pub fn id(&self) -> &str {
        match self {
            Self::Text(message) => &message.id,
            Self::Thinking { id, .. } | Self::Tool { id, .. } => id,
        }
    }
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
    /// The thread, oldest row first: the ONE transcript.
    pub items: Vec<ChatItem>,
    /// Driven by `agent_start` / `agent_end` / `agent_settled`.
    pub working: bool,
    /// Blocking extension dialogs in arrival order, oldest first.
    pub pending_dialogs: Vec<Dialog>,
    /// Monotonic count of the thinking / toolcall / tool_execution events whose
    /// content landed in no row of the transcript: the rows are drawn from
    /// themselves, so counting the events that reached one would be a second
    /// truth (L4).
    pub activity: u64,
    /// Last `extension_error` the child reported.
    pub last_error: Option<String>,
    /// Delegated helpers of this chat, from the `gentle-agents` widget.
    pub helpers: HelpersActivity,
}

impl ChatState {
    /// The text rows of the transcript, oldest first, borrowed.
    pub fn text_items(&self) -> impl DoubleEndedIterator<Item = &ChatMessage> + '_ {
        self.items.iter().filter_map(|item| match item {
            ChatItem::Text(message) => Some(message),
            ChatItem::Thinking { .. } | ChatItem::Tool { .. } => None,
        })
    }

    /// The derived text projection of the transcript, oldest first: exactly the
    /// messages a state held when `messages` was a field of its own (L5).
    ///
    /// A reader that only knows messages — the terminal renderer, the thread
    /// slice the web receives, the images bound to a sent message — reads this
    /// instead of keeping a second copy of the thread.
    pub fn messages(&self) -> Vec<ChatMessage> {
        self.text_items().cloned().collect()
    }

    /// Appends a text row at the end of the transcript with the next contiguous
    /// `msg-<n>` id: the user's own side of the thread.
    pub fn push_message(&mut self, role: ChatRole, text: String, streaming: bool) {
        self.items.push(ChatItem::Text(ChatMessage {
            id: format!("msg-{}", self.text_items().count()),
            role,
            text,
            streaming,
            error: None,
        }));
    }

    /// Replaces the whole transcript with these text rows: a reopened session's
    /// history, whose ids `messages_from_history` already assigned.
    pub fn replace_messages(&mut self, messages: Vec<ChatMessage>) {
        self.items = messages.into_iter().map(ChatItem::Text).collect();
    }
}
