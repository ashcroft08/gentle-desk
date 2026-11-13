//! The map `ChatId -> chat`, the foreground chat, and every read and write the
//! rest of the app does on it.
//!
//! Additive on purpose: it lives next to [`AppState::chat`](crate::AppState) —
//! the single thread the one-child shell still renders — and touches nothing of
//! it. Opening a chat only moves the foreground mark: no other chat is stopped,
//! cleared or read, which is what lets a chat keep working in the background.

use std::collections::BTreeMap;

use gentle_core::chat::ChatState;

use super::activity::{activity_of, ChatActivity};
use super::{queue, ChatId};
use crate::context::ContextUsage;

/// Every chat the app has heard of, and the one the user is looking at.
#[derive(Debug, Clone, PartialEq)]
pub struct Chats {
    pub(crate) entries: BTreeMap<ChatId, Entry>,
    pub(crate) foreground: Option<ChatId>,
    /// Prompts waiting for a working slot, oldest first, across every chat (S7).
    /// Its state and methods live in `queue`, together with the cap.
    pub(crate) queue: Vec<(ChatId, String)>,
    /// The working cap; `0` means no cap is enforced (S7).
    pub(crate) max_working: usize,
}

/// One chat: its state, whether a finished turn is still unread, whether its
/// child is gone, whether one is being started for it, whether a prompt of its
/// own is waiting for a slot (S7), the last lines its child printed on stderr,
/// and the activity the sidebar shows for it.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Entry {
    pub(crate) state: ChatState,
    pub(crate) unread: bool,
    /// The context reading this chat's child last reported (0.1.12, S5).
    /// `None` until the chat's first `get_session_stats` reply, and it belongs
    /// to this chat alone: another chat's turn never overwrites it.
    pub(crate) context: Option<ContextUsage>,
    /// The chat's child is getting one now: set when the chat is asked for and
    /// cleared when the child is up (or the start failed). Cleared the same way
    /// when the chat's child goes away, so it can never stick.
    pub(crate) opening: bool,
    /// The chat's child exited and was not restarted (S3). Cleared by opening or
    /// restarting the chat, which is what asks the host for a fresh child.
    pub(crate) ended: bool,
    /// The chat holds a prompt of its own waiting for a working slot (S7).
    pub(crate) queued: bool,
    /// The last [`STDERR_RING_LINES`] non-empty lines the chat's child printed
    /// on stderr, oldest first, each cut at [`STDERR_LINE_CHARS`], so a death
    /// notice can say why the child died. Bounded; a fresh child starts with a
    /// clean ring.
    pub(crate) stderr: Vec<String>,
    pub(crate) activity: ChatActivity,
}

/// How many stderr lines the ring of one chat keeps.
const STDERR_RING_LINES: usize = 3;
/// How many characters of one stderr line the ring keeps.
const STDERR_LINE_CHARS: usize = 200;

impl Entry {
    /// Re-derives the activity, clearing a finished turn the user is now
    /// looking at.
    pub(crate) fn refresh(&mut self, foreground: bool) {
        if foreground {
            self.unread = false;
        }
        self.activity = activity_of(
            &self.state,
            self.unread,
            foreground,
            self.ended,
            self.queued,
            self.opening,
        );
    }
}

/// The bounded stderr ring of each chat, so a death notice can say why the
/// child died (S3).
impl Chats {
    /// Keeps one non-empty stderr line of that chat's child, oldest last, cut
    /// at [`STDERR_LINE_CHARS`], and keeps at most the last [`STDERR_RING_LINES`]
    /// of them. It never grows unbounded.
    pub(crate) fn record_stderr(&mut self, chat: &ChatId, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        let entry = self.entries.entry(chat.clone()).or_default();
        entry
            .stderr
            .push(line.chars().take(STDERR_LINE_CHARS).collect());
        let excess = entry.stderr.len().saturating_sub(STDERR_RING_LINES);
        entry.stderr.drain(..excess);
    }

    /// The lines that chat's child left on stderr, oldest first.
    pub(crate) fn stderr_of(&self, chat: &ChatId) -> &[String] {
        self.entries.get(chat).map_or(&[], |entry| &entry.stderr)
    }
}

impl Default for Chats {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            foreground: None,
            queue: Default::default(),
            max_working: queue::DEFAULT_MAX_WORKING,
        }
    }
}
