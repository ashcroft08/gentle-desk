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
/// child is gone, whether a prompt of its own is waiting for a slot (S7), and
/// the activity the sidebar shows for it.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Entry {
    pub(crate) state: ChatState,
    pub(crate) unread: bool,
    /// The chat's child exited and was not restarted (S3). Cleared by opening or
    /// restarting the chat, which is what asks the host for a fresh child.
    pub(crate) ended: bool,
    /// The chat holds a prompt of its own waiting for a working slot (S7).
    pub(crate) queued: bool,
    pub(crate) activity: ChatActivity,
}

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
        );
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
