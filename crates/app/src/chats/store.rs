//! The map `ChatId -> chat`, the foreground chat, and every read and write the
//! rest of the app does on it.
//!
//! Additive on purpose: it lives next to [`AppState::chat`](crate::AppState) —
//! the single thread the one-child shell still renders — and touches nothing of
//! it. Opening a chat only moves the foreground mark: no other chat is stopped,
//! cleared or read, which is what lets a chat keep working in the background.

use std::collections::BTreeMap;

use gentle_core::chat::{messages_from_history, reduce_chat_at, ChatState, Dialog};
use gentle_core::rpc::RpcEvent;

use super::activity::{activity_of, ChatActivity};
use super::ChatId;
use crate::input::Command;

/// Every chat the app has heard of, and the one the user is looking at.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chats {
    entries: BTreeMap<ChatId, Entry>,
    foreground: Option<ChatId>,
}

/// One chat: its state, whether a finished turn is still unread, whether its
/// child is gone, and the activity the sidebar shows for it.
#[derive(Debug, Clone, Default, PartialEq)]
struct Entry {
    state: ChatState,
    unread: bool,
    /// The chat's child exited and was not restarted (S3). Cleared by opening or
    /// restarting the chat, which is what asks the host for a fresh child.
    ended: bool,
    activity: ChatActivity,
}

impl Entry {
    /// Re-derives the activity, clearing a finished turn the user is now
    /// looking at.
    fn refresh(&mut self, foreground: bool) {
        if foreground {
            self.unread = false;
        }
        self.activity = activity_of(&self.state, self.unread, foreground, self.ended);
    }
}

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

    /// The dialogs waiting for an answer in one chat, oldest first. A background
    /// chat's dialog is queued here and never lost: only the user answers it.
    pub fn dialogs(&self, chat: &ChatId) -> &[Dialog] {
        self.entries
            .get(chat)
            .map_or(&[], |entry| entry.state.pending_dialogs.as_slice())
    }

    /// Makes `chat` the foreground one and clears its finished-unread mark,
    /// creating it empty if this is the first it is heard of. Opening a chat also
    /// clears a death: the caller asks the host for its child, which starts a
    /// fresh one through `--session` (S3). No other chat changes.
    pub fn open(&mut self, chat: ChatId) -> &mut ChatState {
        let previous = self.foreground.replace(chat.clone());
        let left = previous.filter(|previous| previous != &chat);
        if let Some(entry) = left.and_then(|left| self.entries.get_mut(&left)) {
            entry.refresh(false);
        }
        let entry = self.entries.entry(chat).or_default();
        entry.ended = false;
        entry.refresh(true);
        &mut entry.state
    }

    /// Records that `chat`'s child died: it runs nothing until it is restarted.
    /// The chat keeps its thread, its place and its foreground mark.
    pub(crate) fn ended_now(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.ended = true;
        entry.refresh(foreground);
    }

    /// Records that `chat` has a child again: an expected stop, or a restart the
    /// host honoured. The chat is idle and its thread is untouched.
    pub(crate) fn clear_ended(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let Some(entry) = self.entries.get_mut(chat) else {
            return;
        };
        entry.ended = false;
        entry.refresh(foreground);
    }

    /// Moves a chat from the id it had to the one it just learned, for a new chat
    /// that now has a session path. The foreground follows it, and a finished
    /// turn nobody has read is still waiting afterwards.
    pub fn rekey(&mut self, from: &ChatId, to: ChatId) {
        let Some(mut entry) = self.entries.remove(from) else {
            return;
        };
        if let Some(existing) = self.entries.get(&to) {
            entry.unread |= existing.unread;
        }
        if self.foreground.as_ref() == Some(from) {
            self.foreground = Some(to.clone());
        }
        entry.refresh(self.foreground.as_ref() == Some(&to));
        self.entries.insert(to, entry);
    }

    /// One chat's state, created empty if this is the first it is heard of.
    pub(crate) fn chat_mut(&mut self, chat: &ChatId) -> &mut ChatState {
        &mut self.entries.entry(chat.clone()).or_default().state
    }

    /// Folds one event into one chat and answers the commands it asks for.
    /// Nothing of any other chat is read or written.
    pub(crate) fn apply(
        &mut self,
        chat: &ChatId,
        event: &RpcEvent,
        now_ms: Option<u64>,
    ) -> Vec<Command> {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.state = reduce_chat_at(std::mem::take(&mut entry.state), event, now_ms);
        // A turn that ends while the user is elsewhere leaves a reply to read.
        let ended = matches!(event, RpcEvent::AgentEnd { .. } | RpcEvent::AgentSettled);
        if ended && !foreground {
            entry.unread = true;
        }
        entry.refresh(foreground);
        match event {
            // A finished turn may have created or extended a session file.
            RpcEvent::AgentSettled => vec![Command::RefreshSessions],
            // A stored chat's history arrives in one reply to `get_messages`.
            RpcEvent::Response(response) => {
                if let ("get_messages", Ok(data)) = (response.command.as_str(), &response.outcome) {
                    entry.state.messages = messages_from_history(data);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
}
