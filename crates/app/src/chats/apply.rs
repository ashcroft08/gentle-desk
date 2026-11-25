//! Folds one event into one chat and answers the commands it asks for, with the
//! release of the working queue the fold itself may free (S7).

use gentle_core::chat::{messages_from_history, reduce_chat_at};
use gentle_core::i18n::Language;
use gentle_core::rpc::{RpcCommand, RpcEvent};

use super::queue;
use super::store::Chats;
use super::ChatId;
use crate::context;
use crate::input::{ChatCommand, Command};

impl Chats {
    /// Folds one event into one chat and answers the commands it asks for.
    /// Nothing of any other chat is read or written — except the queue (S7):
    /// a turn that just ended frees a slot, and filling it is one more step of
    /// the same fold, with the check and the mark side by side.
    pub(crate) fn apply(
        &mut self,
        language: Language,
        chat: &ChatId,
        event: &RpcEvent,
        now_ms: Option<u64>,
    ) -> (Vec<Command>, Vec<ChatCommand>) {
        // A chat this run CLOSED keeps nothing of the child it stopped: a reply
        // that was already in flight is stale, and folding it would put the chat
        // back on the open set - the count `open_chats` carries, and the gate the
        // Updates pane's terminal buttons read (packages-status S7). An id the app
        // never heard of keeps today's rule, below: its event makes the chat
        // exist.
        if self.is_closed(chat) {
            return (Vec::new(), Vec::new());
        }
        // The chat's own decoded event is the proof its child is alive: whatever
        // end of the opening phase awaits (`ChatOpened` at once, the 20 s net,
        // the first request's reply), any event of this chat means it already
        // tells the user something. Cleared here so the phase cannot stick.
        self.clear_opening(chat);
        let foreground = self.foreground.as_ref() == Some(chat);
        // Read before the reduce: only a turn that was really running freed a
        // slot; end events of an idle chat free nothing.
        let was_working = self.is_working(chat);
        let entry = self.entries.entry(chat.clone()).or_default();
        entry.state = reduce_chat_at(language, std::mem::take(&mut entry.state), event, now_ms);
        // A turn that ends while the user is elsewhere leaves a reply to read.
        let ended = matches!(event, RpcEvent::AgentEnd { .. } | RpcEvent::AgentSettled);
        if ended && !foreground {
            entry.unread = true;
        }
        entry.refresh(foreground);
        // The host learns the mark the reducer owns, so its protection rules
        // stop guessing: a chat running a turn is never the victim chosen to
        // make room at the live cap, and never reaped as idle. Only a real
        // transition is sent, so an event that changes nothing says nothing.
        let mut released = Vec::new();
        let now_working = self.is_working(chat);
        if now_working != was_working {
            released.push(ChatCommand::SetWorking {
                chat: chat.clone(),
                working: now_working,
            });
        }
        // The turn that just ended freed a slot: the oldest queued prompt goes
        // out, if any waited.
        if ended && was_working {
            released.extend(queue::release(self));
        }
        // A settled turn may also have moved the context: this chat's own
        // child is the only one that can answer for it, so its `get_session_stats`
        // is asked now (0.1.12, S5). pi pushes nothing, so nothing else would.
        if matches!(event, RpcEvent::AgentSettled) {
            released.push(ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetSessionStats,
            });
        }
        let commands = match event {
            // A finished turn may have created or extended a session file.
            RpcEvent::AgentSettled => vec![Command::RefreshSessions],
            // A stored chat's history arrives in one reply to `get_messages`;
            // its context reading in one reply to `get_session_stats`.
            RpcEvent::Response(response) => {
                match (response.command.as_str(), &response.outcome) {
                    ("get_messages", Ok(data)) => {
                        let entry = self.entries.entry(chat.clone()).or_default();
                        entry
                            .state
                            .replace_messages(messages_from_history(language, data));
                        // The child's answer is the ONLY history this chat has: what
                        // the shell read from the same session file before it arrived
                        // is no longer drawn, and no read still in flight may
                        // re-install itself (chat-open-latency T1, S4).
                        entry.history_loaded = true;
                    }
                    // A reply without a reading (an older child, a failure)
                    // changes nothing: the chip keeps what it knew.
                    ("get_session_stats", Ok(data)) => {
                        let usage = context::from_reply(data);
                        if usage.is_some() {
                            self.set_context(chat, usage);
                        }
                    }
                    _ => {}
                }
                Vec::new()
            }
            _ => Vec::new(),
        };
        (commands, released)
    }
}
