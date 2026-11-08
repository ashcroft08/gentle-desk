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
        Self(format!("new-{}", NEXT.fetch_add(1, Ordering::Relaxed)))
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

    /// The id as text, the way the UI holds it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

static NEXT: AtomicU64 = AtomicU64::new(1);

const SESSION_PREFIX: &str = "session:";

impl fmt::Display for ChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
