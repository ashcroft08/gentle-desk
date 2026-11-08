//! Folding one chat's translated event into the app: its decoded output, its
//! stderr, the session file a new chat learns, and the child going away.

use gentle_app::{ChatId, Input};
use gentle_core::rpc::RpcEvent;
use iced::Task;

use super::App;
use crate::chat_runtime::{session_file_of, Converted};
use crate::message::Message;

impl App {
    pub(super) fn on_chat(&mut self, converted: Converted) -> Task<Message> {
        match converted {
            Converted::Input(input) => self.on_chat_input(input),
            Converted::Stderr { chat, line } => {
                eprintln!("[{}] {line}", chat.label());
                // The app keeps the last lines per chat (bounded) so a death
                // notice can say why the child died (S3).
                self.dispatch(Input::ChatStderr { chat, line })
            }
            Converted::Closed { chat, expected } => self.on_chat_closed(chat, expected),
        }
    }

    /// Folds one event of one chat. A `get_state` reply may turn a new chat into
    /// the stored chat its child created, which happens first: from then on the
    /// event belongs to the stored chat, not to the name it was sent under.
    fn on_chat_input(&mut self, input: Input) -> Task<Message> {
        let Input::ChatEventAt {
            chat,
            event,
            now_ms,
        } = input
        else {
            return self.dispatch(input);
        };
        let changes_working = matches!(
            event,
            RpcEvent::AgentStart | RpcEvent::AgentEnd { .. } | RpcEvent::AgentSettled
        );
        let mut chat = chat;
        let mut tasks = Vec::new();
        if let Some(path) = session_file_of_event(&event) {
            if let Some((from, to)) = self.rekey(&chat, path) {
                chat = to.clone();
                tasks.push(self.dispatch(Input::ChatRekeyed { from, to }));
            }
        }
        tasks.push(self.dispatch(Input::ChatEventAt {
            chat: chat.clone(),
            event,
            now_ms,
        }));
        if changes_working {
            self.mark_working(&chat);
        }
        Task::batch(tasks)
    }

    /// The move a new chat's session file asks for, with the runtime renaming the
    /// chat in its registry and in the host. Answers the old and new ids.
    fn rekey(&self, chat: &ChatId, path: &str) -> Option<(ChatId, ChatId)> {
        let moved = self.chat.as_ref()?.rekey(chat, path)?;
        Some((chat.clone(), moved.id))
    }

    /// Keeps the host's working mark in step with what the app derived from the
    /// chat's own events: a working chat is never reaped and never counted as
    /// room for another one.
    fn mark_working(&mut self, chat: &ChatId) {
        let Some(runtime) = &self.chat else {
            return;
        };
        let Some(state) = self.app.chats.state(chat) else {
            return;
        };
        runtime.set_working(chat, state.working);
    }
}

/// The session file a reply reports, for the one command that reports it.
fn session_file_of_event(event: &RpcEvent) -> Option<&str> {
    match event {
        RpcEvent::Response(response) => session_file_of(response),
        _ => None,
    }
}
