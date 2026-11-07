//! Parsed markdown for the message thread.
//!
//! iced renders markdown from a parsed [`markdown::Content`]. Re-parsing every
//! message on every streamed delta would be wasteful, so each assistant
//! message keeps its `Content` and only the new tail of its text is pushed in.
//! When the text is *not* an extension of what was parsed (`message_end`
//! resyncs a message from its authoritative text), the message is re-parsed.

use gentle_core::chat::{ChatRole, ChatState};
use iced::widget::markdown;

struct Entry {
    /// The text `content` was built from.
    raw: String,
    content: markdown::Content,
}

/// One slot per chat message; `None` for user messages (shown as plain text).
#[derive(Default)]
pub struct ThreadCache {
    entries: Vec<Option<Entry>>,
}

impl ThreadCache {
    /// Brings the cache in line with `chat`.
    pub fn sync(&mut self, chat: &ChatState) {
        // A shorter thread (new chat, dropped message) invalidates the tail.
        self.entries.truncate(chat.messages.len());
        for (index, message) in chat.messages.iter().enumerate() {
            let wanted = message.role == ChatRole::Assistant;
            match self.entries.get_mut(index) {
                Some(Some(entry)) if wanted => {
                    match message.text.strip_prefix(entry.raw.as_str()) {
                        Some("") => {}
                        Some(tail) => {
                            entry.content.push_str(tail);
                            entry.raw = message.text.clone();
                        }
                        None => *entry = parse(&message.text),
                    }
                }
                Some(slot) => *slot = wanted.then(|| parse(&message.text)),
                None => self.entries.push(wanted.then(|| parse(&message.text))),
            }
        }
    }

    pub fn content(&self, index: usize) -> Option<&markdown::Content> {
        self.entries
            .get(index)?
            .as_ref()
            .map(|entry| &entry.content)
    }
}

fn parse(text: &str) -> Entry {
    Entry {
        raw: text.to_owned(),
        content: markdown::Content::parse(text),
    }
}

#[cfg(test)]
mod tests;
