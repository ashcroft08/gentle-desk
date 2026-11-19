//! Rebuilding the thread from a `get_messages` response, and from the session
//! file a chat is stored in.

use serde_json::Value;

use super::state::{ChatMessage, ChatRole};
use crate::i18n::{messages, Language};

/// The most bytes of a session file the thread is rebuilt from.
///
/// The shell reads the file WHOLE, once, before its chat's child has answered
/// anything (`chat-open-latency.md` T1: it is what the thread paints in the
/// seconds a child takes to start), so what it reads lands in memory at once
/// rather than being streamed. 8 MiB is a long chat - about a hundred thousand
/// tool-heavy lines - and a file past it is not worth parsing into rows the
/// child's own answer replaces a moment later, so such a file draws nothing at
/// all and the thread waits for the child, exactly as it did before this.
pub const SESSION_FILE_MAX_BYTES: usize = 8 * 1024 * 1024;

/// The thread a `get_messages` response (`{"messages": [...]}`) describes.
///
/// Mirrors what the live reducer would have shown: only user and assistant
/// text, an assistant turn with no text dropped, a failed one kept and
/// flagged. System messages and tool results are not part of the thread.
///
/// The language comes first, as it does everywhere in the catalog: a failed turn
/// the provider said nothing about is worded here, and the caller holds the
/// shell's choice (0.2.0, S3).
pub fn messages_from_history(language: Language, data: &Value) -> Vec<ChatMessage> {
    let Some(entries) = data.get("messages").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut messages: Vec<ChatMessage> = Vec::new();
    for entry in entries {
        let id = format!("msg-{}", messages.len());
        messages.extend(project(language, entry, id));
    }
    messages
}

/// The thread a session FILE's text describes: the rows a stored chat paints
/// while its child is still starting (`chat-open-latency.md` T1).
///
/// A session file is an append-only JSONL file (`crate::session`): a `session`
/// header, then entries. Only the `message` entries are the thread, and each
/// one is projected by the very rule [`messages_from_history`] applies to a
/// `get_messages` entry, so the rows the file paints are the rows the child is
/// about to send - only their ids differ.
///
/// The ids of these rows are [`PROVISIONAL_PREFIX`]`<n>`, never the child's
/// `msg-<n>`: a file row is a placeholder, it is never the authority, and it is
/// never handed back to the child as one of its own.
///
/// Nothing here is an error. A line that is not JSON - the half-written line a
/// file ends with while it is being appended to - is skipped by the same rule a
/// malformed line in the middle is, and so is every entry that is not a
/// message. A file past [`SESSION_FILE_MAX_BYTES`] draws nothing.
///
/// The language a failed turn with no message of its own is worded in has no
/// caller here (the signature is the file's text and nothing else): it is
/// English, and that row is replaced by the child's own answer - worded in the
/// run's language - in the same second.
pub fn messages_from_session_file(text: &str) -> Vec<ChatMessage> {
    if text.len() > SESSION_FILE_MAX_BYTES {
        return Vec::new();
    }
    let mut messages: Vec<ChatMessage> = Vec::new();
    for line in text.lines() {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if entry.get("type").and_then(Value::as_str) != Some(MESSAGE_ENTRY) {
            continue;
        }
        let Some(message) = entry.get("message") else {
            continue;
        };
        let id = format!("{PROVISIONAL_PREFIX}{}", messages.len());
        messages.extend(project(Language::En, message, id));
    }
    messages
}

/// The prefix of a row read from a session file: distinct from the child's
/// `msg-<n>` scheme on purpose, so a provisional row is never mistaken for one
/// the reducer owns.
pub const PROVISIONAL_PREFIX: &str = "prov-";

/// The `type` of a session file's entry that carries one turn.
const MESSAGE_ENTRY: &str = "message";

/// One turn of the thread, projected by the ONE rule both readers apply.
///
/// `message` is the message OBJECT (`{"role", "content", "stopReason",
/// "errorMessage"}`): the entry itself in a `get_messages` answer, the nested
/// `message` of a session file's line. `None` for a role the thread does not
/// show and for an assistant turn with no text and no failure.
fn project(language: Language, message: &Value, id: String) -> Option<ChatMessage> {
    let role = match message.get("role").and_then(Value::as_str) {
        Some("user") => ChatRole::User,
        Some("assistant") => ChatRole::Assistant,
        _ => return None,
    };
    let text = text_of(&message["content"]);
    let error = (role == ChatRole::Assistant)
        .then(|| model_error(language, message))
        .flatten();
    if text.trim().is_empty() && error.is_none() {
        return None;
    }
    Some(ChatMessage {
        id,
        role,
        text,
        streaming: false,
        error,
    })
}

/// A string content as is; a block list as its `text` blocks concatenated.
fn text_of(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect(),
        _ => String::new(),
    }
}

fn model_error(language: Language, entry: &Value) -> Option<String> {
    if entry.get("stopReason").and_then(Value::as_str) != Some("error") {
        return None;
    }
    let text = entry.get("errorMessage").and_then(Value::as_str);
    Some(
        text.unwrap_or(messages::unknown_model_error(language))
            .to_owned(),
    )
}
