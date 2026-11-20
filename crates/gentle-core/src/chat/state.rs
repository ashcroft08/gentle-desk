//! Chat state produced by folding decoded RPC events.
//!
//! Pure data: no I/O, no clocks, no id generation beyond the deterministic
//! `msg-<n>`, `think-<index>` and `tool-<toolCallId>` schemes. The `ui`
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
    /// `msg-<n>`, where `<n>` comes from [`ChatState::next_text_id`]: an id is
    /// the row's identity, so it is never reused. A message that was dropped
    /// leaves a GAP in the numbering, which is correct - ids are unique for the
    /// life of the chat, contiguous or not, and a thinking or tool row never
    /// shifts one.
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
    /// True when a FIELD of it was lost, which [`truncated`](Self::truncated)
    /// does not say.
    ///
    /// Cutting an object's key to the bound can make it land on another key of
    /// the same object, and a JSON object holds one value per key: the field
    /// already kept stays and the one that would overwrite it cannot be kept.
    /// That is a loss, not a shortening, so it gets its own flag - a card can
    /// then tell the user something was DROPPED instead of only that something
    /// was cut short (R3-2).
    pub dropped: bool,
}

impl ToolPayload {
    /// Keeps `value`, cutting its long strings and flagging what the cut cost.
    pub fn new(value: &Value) -> Self {
        let mut value = value.clone();
        let cut = cut_strings(&mut value);
        Self {
            value,
            truncated: cut.truncated,
            dropped: cut.dropped,
        }
    }
}

/// What cutting a value cost: strings shortened, or a field lost.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Cut {
    /// Some string was longer than the bound and was shortened.
    truncated: bool,
    /// Cutting a KEY made it land on a key the same object already holds, so
    /// one of the two fields could not be kept.
    dropped: bool,
}

impl Cut {
    /// Everything both cuts cost.
    fn or(self, other: Self) -> Self {
        Self {
            truncated: self.truncated || other.truncated,
            dropped: self.dropped || other.dropped,
        }
    }
}

/// Cuts every string of `value` to [`TOOL_PAYLOAD_MAX_BYTES`] and answers what
/// that cost.
fn cut_strings(value: &mut Value) -> Cut {
    match value {
        Value::String(text) => Cut {
            truncated: cut_string(text),
            dropped: false,
        },
        Value::Array(items) => items
            .iter_mut()
            .map(cut_strings)
            .fold(Cut::default(), Cut::or),
        Value::Object(fields) => {
            // An object's KEY is a string the payload retains just as its values
            // are: walking only the values let a payload keep a key of any size
            // and still report `truncated: false`, which is the bypass the bound
            // exists to prevent (R3-001). Drained and refilled, so a cut key
            // lands where the whole one was.
            let mut cut = Cut::default();
            let entries: Vec<(String, Value)> = std::mem::take(fields).into_iter().collect();
            for (mut key, mut item) in entries {
                cut.truncated |= cut_string(&mut key);
                cut = cut.or(cut_strings(&mut item));
                // Two keys that differ only past the bound cut to the same
                // string. A map keeps one value per key, so keeping the field
                // already there means the colliding one is lost - and a loss
                // is recorded rather than hidden behind `truncated`, which
                // already means "a string was cut" (R3-2).
                if fields.contains_key(&key) {
                    cut.dropped = true;
                    continue;
                }
                fields.insert(key, item);
            }
            cut
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => Cut::default(),
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
    // Rebuilt, not truncated in place: `String::truncate` only lowers the length
    // and keeps the original allocation, so a multi-megabyte output cut to the
    // bound would still HOLD its full buffer for the life of the chat - a bound
    // that reports a small length and keeps a large allocation is no bound at all
    // (review R3-001). A fresh string allocates exactly `end` bytes.
    *text = text[..end].to_owned();
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

    /// True when a tool row LOST a field of a payload: cutting one of an
    /// object's keys made it collide with another, so the payload could not keep
    /// both. The card says a field was dropped, which
    /// [`payload_truncated`](Self::payload_truncated) cannot say - that one only
    /// reports a shortened string (R3-2).
    pub fn payload_dropped(&self) -> bool {
        match self {
            Self::Tool {
                args,
                partial_result,
                result,
                ..
            } => [args, partial_result, result]
                .into_iter()
                .flatten()
                .any(|payload| payload.dropped),
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
    /// The next `msg-<n>` number to hand out.
    ///
    /// Ids are identity, so they are never reused: this is a monotonic counter,
    /// not a count of the text rows. Counting them fell behind the moment the
    /// reducer dropped an empty assistant row from the MIDDLE of the transcript,
    /// and the next push then reused an id a later row still held, which breaks
    /// every id-based lookup and every keyed view (R3-1). Gaps after a dropped
    /// row are therefore expected, and a message pushed by hand into `items`
    /// must keep this counter ahead of it.
    pub next_text_id: u64,
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

    /// The last `limit` rows of the transcript in order, whatever their kind.
    ///
    /// The cost is the window itself: `items` is a slice, so the rows are reached
    /// from the END and the transcript's head is never touched. Nothing is filtered
    /// here (unlike `last_text_items`), so there is no `skip` over a filtered
    /// iterator - the shape that made that accessor walk the whole transcript
    /// (R3-002).
    pub fn last_items(&self, limit: usize) -> impl Iterator<Item = &ChatItem> + '_ {
        let mut tail: Vec<&ChatItem> = self.items.iter().rev().take(limit).collect();
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

    /// Appends a text row at the end of the transcript with the next
    /// `msg-<n>` id: the user's own side of the thread.
    ///
    /// The id comes from [`next_text_id`](Self::next_text_id), never from the
    /// rows already there: a dropped message leaves a gap instead of freeing its
    /// id for reuse (R3-1).
    pub fn push_message(&mut self, role: ChatRole, text: String, streaming: bool) {
        let id = format!("msg-{}", self.next_text_id);
        self.next_text_id += 1;
        self.items.push(ChatItem::Text(ChatMessage {
            id,
            role,
            text,
            streaming,
            error: None,
        }));
    }

    /// Replaces the whole transcript with these text rows: a reopened session's
    /// history, whose ids `messages_from_history` already assigned.
    ///
    /// The counter resumes after the highest id the history kept, so a message
    /// pushed afterwards cannot reuse one of them; a history with no
    /// `msg-<n>` id at all leaves it at the number of rows it installed.
    pub fn replace_messages(&mut self, messages: Vec<ChatMessage>) {
        self.next_text_id = messages
            .iter()
            .filter_map(|message| text_id_number(&message.id))
            .max()
            .map_or(messages.len() as u64, |highest| highest + 1);
        self.items = messages.into_iter().map(ChatItem::Text).collect();
    }
}

/// The `<n>` of a `msg-<n>` id, when it has that shape.
fn text_id_number(id: &str) -> Option<u64> {
    id.strip_prefix("msg-")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{cut_string, TOOL_PAYLOAD_MAX_BYTES};

    /// The bound must bound MEMORY, not only a number: `String::truncate` keeps
    /// the original allocation, so a cut string could still hold its whole buffer
    /// while reporting a small length (review R3-001 of the card artboard
    /// candidate). Capacity is the thing that proves it.
    #[test]
    fn a_cut_string_does_not_keep_its_original_allocation() {
        let mut text = "x".repeat(TOOL_PAYLOAD_MAX_BYTES * 8);

        assert!(
            text.capacity() >= TOOL_PAYLOAD_MAX_BYTES * 8,
            "precondition: the buffer starts big"
        );
        assert!(cut_string(&mut text), "an oversized string is cut");
        assert_eq!(text.len(), TOOL_PAYLOAD_MAX_BYTES, "the length is bounded");
        assert!(
            text.capacity() <= TOOL_PAYLOAD_MAX_BYTES,
            "the ALLOCATION must be bounded too, capacity was {}",
            text.capacity()
        );
    }
}
