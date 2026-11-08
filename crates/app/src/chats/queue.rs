//! The working cap and the FIFO queue of prompts that wait for a slot (S7).
//!
//! Over the cap nothing is lost and nothing is refused (user decision,
//! "Ponerlo en cola"): the prompt waits here and goes out the moment a slot
//! frees. The cap is plain data (`Chats::max_working`, set by
//! `Input::SetWorkingCap` at boot from the host's `Limits::max_working`);
//! `0` means no cap. Because both the check and the slot-taking happen in one
//! reducer step, two prompts starting together cannot both take the same slot
//! the way the host's `may_start_work` / `set_working` pair could.

use gentle_core::rpc::RpcCommand;

use super::{ChatId, Chats};
use crate::input::ChatCommand;

/// The cap [`Chats::default`] carries before the boot input names the host's.
pub(crate) const DEFAULT_MAX_WORKING: usize = 2;

impl Default for Chats {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            foreground: None,
            queue: Default::default(),
            max_working: DEFAULT_MAX_WORKING,
        }
    }
}

/// Reads and marks on `Chats` that only the cap knows the meaning of. They live
/// here, next to the queue, so `store` stays the plain map it was.
impl Chats {
    /// Whether that chat holds a prompt waiting for a working slot (S7).
    pub fn is_queued(&self, chat: &ChatId) -> bool {
        self.entries.get(chat).is_some_and(|entry| entry.queued)
    }

    /// Marks (or clears) that chat's own waiting-for-a-slot state.
    pub(crate) fn set_queued(&mut self, chat: &ChatId, queued: bool) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.queued = queued;
        entry.refresh(foreground);
    }

    /// Takes the slot the released prompt needs right away (S7): the chat counts
    /// as working from this step on, so no other prompt reads a freed slot twice.
    pub(crate) fn mark_working(&mut self, chat: &ChatId) {
        let foreground = self.foreground.as_ref() == Some(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.state.working = true;
        entry.queued = false;
        entry.refresh(foreground);
    }

    /// Clears the working mark because the chat cannot go on: its child is
    /// gone. Answers whether it was working, so the caller knows its slot freed.
    pub(crate) fn stop_working(&mut self, chat: &ChatId) -> bool {
        let Some(entry) = self.entries.get_mut(chat) else {
            return false;
        };
        let was = entry.state.working;
        entry.state.working = false;
        was
    }
}

/// Applies the boot's cap from the UI.
pub(crate) fn set_cap(chats: &mut Chats, cap: usize) {
    chats.max_working = cap;
}

/// Whether another prompt may start a turn under the cap.
pub(crate) fn has_room(chats: &Chats) -> bool {
    chats.max_working == 0 || chats.working_count() < chats.max_working
}

/// Places a prompt at the end of the queue and marks its chat Queued. Its user
/// message is already in the thread: the chat shows its own words.
pub(crate) fn enqueue(chats: &mut Chats, chat: ChatId, message: String) {
    chats.queue.push((chat.clone(), message));
    chats.set_queued(&chat, true);
}

/// Cancels `chat`'s own queued prompt, if it waits (the Abort on a Queued chat).
/// Nothing is sent: no turn was started. Answers whether there was one.
pub(crate) fn cancel(chats: &mut Chats, chat: &ChatId) -> bool {
    remove_of(chats, chat)
}

/// Forgets `chat`'s queued prompt without freeing anything: an ended chat can
/// never send it (its death notice already told the user).
pub(crate) fn drop_of(chats: &mut Chats, chat: &ChatId) -> bool {
    remove_of(chats, chat)
}

/// Fills every freed slot with the oldest queued prompt, marking that chat
/// Working right away, so the next reducer step sees the slot as taken.
pub(crate) fn release(chats: &mut Chats) -> Vec<ChatCommand> {
    let mut sent = Vec::new();
    while has_room(chats) {
        let Some((chat, message)) = chats.queue.first() else {
            break;
        };
        let (chat, message) = (chat.clone(), message.clone());
        chats.queue.remove(0);
        chats.mark_working(&chat);
        sent.push(ChatCommand::Send {
            chat,
            rpc: RpcCommand::Prompt { message },
        });
    }
    sent
}

/// Removes that chat's first queued prompt, if any.
fn remove_of(chats: &mut Chats, chat: &ChatId) -> bool {
    let Some(at) = chats.queue.iter().position(|(queued, _)| queued == chat) else {
        return false;
    };
    chats.queue.remove(at);
    chats.set_queued(chat, false);
    true
}

/// A queued prompt follows its chat: a new chat that just learned its session
/// file is still the same chat (S7).
pub(crate) fn rekeyed(chats: &mut Chats, from: &ChatId, to: ChatId) {
    for (queued, _) in &mut chats.queue {
        if queued == from {
            *queued = to.clone();
        }
    }
}
