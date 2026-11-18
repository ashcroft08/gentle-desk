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

/// The most bytes of any one string a tool row keeps.
///
/// A tool's output is the child's own text and can be megabytes - a `read` of a
/// large file, a whole test log - and the transcript keeps it for the life of
/// the chat. 8 KiB is about two hundred lines: enough for the collapsed body a
/// card draws and its "N earlier lines" head, small enough that a chat of
/// hundreds of calls stays a chat. A string longer than this is cut, and the
/// payload says so, so a card reports the cut instead of presenting the head as
/// the whole output.
pub const TOOL_PAYLOAD_MAX_BYTES: usize = 8 * 1024;

/// One payload of a tool call - its arguments or the output it produced - kept
/// as the child's own untyped JSON, because each tool's shape is its own, and
/// bounded ([`TOOL_PAYLOAD_MAX_BYTES`]).
///
/// Constructing one is the only way a payload enters a row, so the bound cannot
/// be forgotten at a call site.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPayload {
    /// The JSON the child sent, with no single string longer than
    /// [`TOOL_PAYLOAD_MAX_BYTES`] - the string KEYS of its objects included,
    /// since a key is a string the payload retains like any other.
    pub value: Value,
    /// True when a string of it was cut to fit. The row is not lying about what
    /// it holds: what it holds is the head of a longer payload.
    pub truncated: bool,
}

impl ToolPayload {
    /// Keeps `value`, cutting its long strings and flagging the cut.
    pub fn new(value: &Value) -> Self {
        let mut value = value.clone();
        let truncated = cut_strings(&mut value);
        Self { value, truncated }
    }
}

/// Cuts every string of `value` to [`TOOL_PAYLOAD_MAX_BYTES`] and answers
/// whether any of them was cut.
fn cut_strings(value: &mut Value) -> bool {
    match value {
        Value::String(text) => cut_string(text),
        Value::Array(items) => items
            .iter_mut()
            .fold(false, |cut, item| cut | cut_strings(item)),
        Value::Object(fields) => {
            // An object's KEY is a string the payload retains just as its values
            // are: walking only the values let a payload keep a key of any size
            // and still report `truncated: false`, which is the bypass the bound
            // exists to prevent (R3-001). Drained and refilled, so a cut key
            // lands where the whole one was.
            let mut cut = false;
            let entries: Vec<(String, Value)> = std::mem::take(fields).into_iter().collect();
            for (mut key, mut item) in entries {
                cut |= cut_string(&mut key);
                cut |= cut_strings(&mut item);
                fields.insert(key, item);
            }
            cut
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

/// Cuts one string to the bound, on a character boundary: half a character is
/// not text.
fn cut_string(text: &mut String) -> bool {
    if text.len() <= TOOL_PAYLOAD_MAX_BYTES {
        return false;
    }
    let mut end = TOOL_PAYLOAD_MAX_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    true
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
    /// untyped here, because each tool's shape is its own; each is a
    /// [`ToolPayload`], which bounds what one call can cost the chat.
    Tool {
        /// `tool-<toolCallId>`.
        id: String,
        /// The child's own id for the call, which every later event names.
        tool_call_id: String,
        tool_name: String,
        /// The call's arguments as the child sent them.
        args: Option<ToolPayload>,
        /// The output the tool had produced at its last update, dropped as soon
        /// as the `result` that supersedes it arrives: one call keeps one
        /// output.
        partial_result: Option<ToolPayload>,
        /// The result the tool ended with.
        result: Option<ToolPayload>,
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

    /// True when a tool row had to cut a payload it keeps: the card says the
    /// output was cut instead of drawing the head as the whole of it.
    pub fn payload_truncated(&self) -> bool {
        match self {
            Self::Tool {
                args,
                partial_result,
                result,
                ..
            } => [args, partial_result, result]
                .into_iter()
                .flatten()
                .any(|payload| payload.truncated),
            Self::Text(_) | Self::Thinking { .. } => false,
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

    /// The LAST `limit` text rows of the transcript, oldest first, borrowed: the
    /// tail [`messages`](Self::messages) would end with, taken from the end of
    /// the thread instead of counted and skipped from its start.
    ///
    /// The cost is the rows that travel, plus the thinking and tool rows mixed in
    /// among them: the walk starts at the last row and stops as soon as the
    /// `limit`-th text row is reached, so a long session's head is never touched.
    /// Counting and skipping instead walks the whole transcript twice (R3-002).
    pub fn last_text_items(&self, limit: usize) -> impl Iterator<Item = &ChatMessage> + '_ {
        // Walked backwards from the LAST row, so the walk stops as soon as the
        // `limit`-th text row is in hand and the transcript's head is never
        // touched; then put back in the order a thread reads. At most `limit`
        // borrowed rows are held, and a `Vec` of borrowed rows is the smallest
        // thing that can reverse a window (`Take`'s own backward walk needs an
        // exact size count that a filtered row does not have).
        let mut tail: Vec<&ChatMessage> = self.text_items().rev().take(limit).collect();
        tail.reverse();
        tail.into_iter()
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
