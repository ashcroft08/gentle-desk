//! The mutations that change a chat's life: open, end, rekey, and the mutable
//! handle into one chat's state.

use gentle_core::chat::ChatState;

use super::store::Chats;
use super::{queue, ChatId};
use crate::context::ContextUsage;

impl Chats {
    /// Makes `chat` the foreground one and clears its finished-unread mark,
    /// creating it empty if this is the first it is heard of. Opening a chat also
    /// clears a death and the stderr the old child left: the caller asks the host
    /// for its child, which starts a fresh one through `--session` (S3). No other
    /// chat changes.
    pub fn open(&mut self, chat: ChatId) -> &mut ChatState {
        let previous = self.foreground.replace(chat.clone());
        let left = previous.filter(|previous| previous != &chat);
        if let Some(entry) = left.and_then(|left| self.entries.get_mut(&left)) {
            entry.refresh(false);
        }
        let entry = self.entries.entry(chat).or_default();
        entry.ended = false;
        entry.stderr.clear();
        entry.refresh(true);
        &mut entry.state
    }

    /// Marks that a child is being started for `chat` right now: the chat asked
    /// for one and the start takes seconds. Cleared when the chat's own first
    /// event proves its child is alive, when the child is confirmed up (for a
    /// fresh child only through the 20 s net, see
    /// [`Input::ChatOpened`](crate::Input::ChatOpened)), its start fails
    /// ([`Input::ChatFailed`](crate::Input::ChatFailed)) or its child goes away.
    pub(crate) fn mark_opening(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.opening = true;
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

    /// Records that the opening phase of `chat` is over, whatever ended it: the
    /// chat spoke (any of its own events — [`Chats::apply`](super::store::Chats)
    /// calls this before it folds), the child is up, its start failed, or the
    /// child went away while it was being started. So the mark never sticks.
    /// Clearing a chat that is not Opening changes nothing.
    pub(crate) fn clear_opening(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let Some(entry) = self.entries.get_mut(chat) else {
            return;
        };
        entry.opening = false;
        entry.refresh(foreground);
    }

    /// Records that `chat`'s child died: it runs nothing until it is restarted.
    /// The chat keeps its thread, its place and its foreground mark.
    pub(crate) fn ended_now(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.ended = true;
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
        self.entries.insert(to.clone(), entry);
        queue::rekeyed(self, from, to);
    }

    /// One chat's state, created empty if this is the first it is heard of.
    pub(crate) fn chat_mut(&mut self, chat: &ChatId) -> &mut ChatState {
        &mut self.entries.entry(chat.clone()).or_default().state
    }

    /// Takes a chat off the open set (0.1.14): its entry, its place as the
    /// foreground chat, and any prompt of its own that was still waiting for a
    /// working slot. Nothing on disk changes and no other chat is read or
    /// written, so the session list keeps its row and the row opens the chat
    /// again - which asks the host for a fresh child, exactly as a reopen does.
    /// Answers whether this app had heard of the chat at all.
    pub fn close(&mut self, chat: &ChatId) -> bool {
        // The queue first: forgetting a waiting prompt marks the chat it belonged
        // to, so the entry has to be gone after that, never before it.
        queue::drop_of(self, chat);
        let known = self.entries.remove(chat).is_some();
        if self.foreground.as_ref() == Some(chat) {
            self.foreground = None;
        }
        known
    }

    /// Records the context reading one chat's child just reported (0.1.12, S5).
    /// Only that chat changes: a reply of a background child is kept for when
    /// the user returns to it.
    pub(crate) fn set_context(&mut self, chat: &ChatId, usage: Option<ContextUsage>) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.context = usage;
        entry.refresh(foreground);
    }
}
