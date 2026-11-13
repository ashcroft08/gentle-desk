//! The read-only queries on `Chats`: what the rest of the app may look at.

use gentle_core::chat::{ChatState, Dialog};

use super::activity::ChatActivity;
use super::store::Chats;
use super::ChatId;
use crate::context::ContextUsage;

impl Chats {
    /// The chat the user is on, if any.
    pub fn foreground(&self) -> Option<&ChatId> {
        self.foreground.as_ref()
    }

    /// Every chat the app knows, in id order.
    pub fn ids(&self) -> Vec<&ChatId> {
        self.entries.keys().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// One chat's thread and working state.
    pub fn state(&self, chat: &ChatId) -> Option<&ChatState> {
        self.entries.get(chat).map(|entry| &entry.state)
    }

    /// What the sidebar shows for one chat.
    pub fn activity(&self, chat: &ChatId) -> Option<ChatActivity> {
        self.entries.get(chat).map(|entry| entry.activity)
    }

    /// Whether that chat's child is gone, so its composer cannot reach
    /// anything until it is restarted (S3).
    pub fn is_ended(&self, chat: &ChatId) -> bool {
        self.entries.get(chat).is_some_and(|entry| entry.ended)
    }

    /// Whether a child is being started for that chat right now: it cannot take
    /// a prompt yet (composer disabled, prompt kept), like a working or queued
    /// chat.
    pub fn is_opening(&self, chat: &ChatId) -> bool {
        self.entries.get(chat).is_some_and(|entry| entry.opening)
    }

    /// Every chat whose child is gone, in id order: the chats a Restart can
    /// bring back.
    pub fn ended(&self) -> Vec<ChatId> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.ended)
            .map(|(chat, _)| chat.clone())
            .collect()
    }

    /// How many chats are running a model call right now, so the shell can
    /// consult the host's working cap before starting another one.
    pub fn working_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.state.working)
            .count()
    }

    /// Whether that chat is running a model call right now.
    pub(crate) fn is_working(&self, chat: &ChatId) -> bool {
        self.entries
            .get(chat)
            .is_some_and(|entry| entry.state.working)
    }

    /// The dialogs waiting for an answer in one chat, oldest first. A background
    /// chat's dialog is queued here and never lost: only the user answers it.
    pub fn dialogs(&self, chat: &ChatId) -> &[Dialog] {
        self.entries
            .get(chat)
            .map_or(&[], |entry| entry.state.pending_dialogs.as_slice())
    }

    /// The context reading that chat's child last reported (0.1.12, S5), `None`
    /// for a chat that has not answered `get_session_stats` yet.
    pub fn context(&self, chat: &ChatId) -> Option<&ContextUsage> {
        self.entries
            .get(chat)
            .and_then(|entry| entry.context.as_ref())
    }
}
