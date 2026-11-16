//! The identity of one chat's child.
//!
//! Opaque on purpose: the app layer owns the scheme, so the UI can map its own
//! chat keys to a [`ChatId`] and back without knowing the backend's process
//! types (`gentle-backend::chat_host::ChatKey` lives in a crate this one neither
//! depends on nor needs to).

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// One chat, as the app layer names it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChatId(String);

impl ChatId {
    /// A brand-new chat: it has no session file yet, so its child is started
    /// without one. Ids are never reused, so a reopened chat is a new chat.
    pub fn fresh() -> Self {
        Self(format!(
            "{FRESH_PREFIX}{}",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    /// The chat of a stored session. The path is the identity: opening the same
    /// session twice is opening the same chat.
    pub fn of_session(path: &str) -> Self {
        Self(format!("{SESSION_PREFIX}{path}"))
    }

    /// The session file of a stored chat, or `None` for a chat that has none
    /// yet. The UI reads the path back through this, so it never parses the
    /// scheme itself.
    pub fn session_path(&self) -> Option<&str> {
        self.0.strip_prefix(SESSION_PREFIX)
    }

    /// The session file the id TEXT names, if it names one: a command that starts
    /// from an id the UI holds reads the scheme here and nowhere else.
    ///
    /// Both live forms answer their file. The chat header sends the [`ChatId`]
    /// text itself (`foreground_chat`, the `of_session` form), and a sidebar row
    /// still sends the bare session path; the rename both share writes the same
    /// session file either way. A fresh chat ([`ChatId::fresh`]) has no file yet,
    /// so it answers `None` - there is nothing to append a name to - and so does
    /// an id that carries no path at all. Anything else is read as the path it
    /// already was, so a string the app does not recognise is refused by the disk
    /// in the disk's own words, exactly as before.
    pub fn session_path_of(id: &str) -> Option<&str> {
        if is_fresh(id) {
            return None;
        }
        let path = id.strip_prefix(SESSION_PREFIX).unwrap_or(id);
        (!path.is_empty()).then_some(path)
    }

    /// The id as text, the way the UI holds it.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A short name for a notice or a log line: the file name of a stored
    /// chat's session, or the id of a new chat. Never the whole path.
    pub fn label(&self) -> String {
        match self.session_path() {
            Some(path) => path
                .rsplit([char::from(47u8), char::from(92u8)]) // / and the backslash
                .next()
                .unwrap_or(path)
                .to_owned(),
            None => self.0.clone(),
        }
    }
}

static NEXT: AtomicU64 = AtomicU64::new(1);

const SESSION_PREFIX: &str = "session:";
const FRESH_PREFIX: &str = "new-";

/// Whether `id` is exactly [`ChatId::fresh`]'s form: the fresh prefix and the
/// counter, so a path that merely starts with the same letters is not read as a
/// chat that has no file yet.
fn is_fresh(id: &str) -> bool {
    id.strip_prefix(FRESH_PREFIX).is_some_and(|counter| {
        !counter.is_empty() && counter.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests;

impl fmt::Display for ChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
