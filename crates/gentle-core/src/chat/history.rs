//! Rebuilding the thread from a `get_messages` response.

use serde_json::Value;

use super::state::{ChatMessage, ChatRole};
use crate::i18n::{messages, Language};

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
        let role = match entry.get("role").and_then(Value::as_str) {
            Some("user") => ChatRole::User,
            Some("assistant") => ChatRole::Assistant,
            _ => continue,
        };
        let text = text_of(&entry["content"]);
        let error = (role == ChatRole::Assistant)
            .then(|| model_error(language, entry))
            .flatten();
        if text.trim().is_empty() && error.is_none() {
            continue;
        }
        messages.push(ChatMessage {
            id: format!("msg-{}", messages.len()),
            role,
            text,
            streaming: false,
            error,
        });
    }
    messages
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
