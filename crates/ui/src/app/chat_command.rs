//! Running one per-chat command off the UI thread and folding what came of it.
//!
//! Starting a child takes seconds and writing to one blocks when that child
//! stops reading, so each command runs in its own task: a slow chat never
//! freezes the window and never holds up another chat.

use std::time::Duration;

use gentle_app::{ChatCommand, ChatId, Input};
use gentle_backend::chat_host::Opened;
use iced::Task;

use super::App;
use crate::message::Message;
use gentle_runtime::chat_runtime::{request_after_open, Converted, Outcome};

impl App {
    /// Runs the commands the app layer answered, one task each.
    pub(super) fn run_chat_commands(&mut self, commands: Vec<ChatCommand>) -> Task<Message> {
        let Some(runtime) = self.chat.clone() else {
            // No launcher at all: every command is a notice, never a silent click.
            return self.chat_failed(commands, "gentle-shell is not available");
        };
        Task::batch(commands.into_iter().map(move |command| {
            let runtime = std::sync::Arc::clone(&runtime);
            Task::perform(
                async move { runtime.execute(command) },
                Message::ChatOutcome,
            )
        }))
    }

    /// What running one command came to: the confirmation of a child that came
    /// up, or the notice of one that could not be reached.
    pub(super) fn on_chat_outcome(&mut self, outcome: Outcome) -> Task<Message> {
        match outcome {
            Outcome::Opened { chat, how } => match how {
                // A fresh child: the host's answer only says the process was
                // created, which happens in milliseconds, while the child takes
                // seconds to speak (T5a: ~2.2 s to its first line, ~3 s per
                // request). The opening phase ends when the chat's first own
                // event arrives; the 20 s net below is the backup.
                Opened::Started => {
                    let requests = self.on_opened(chat.clone(), how);
                    Task::batch([requests, opening_net(chat)])
                }
                // Already live, or its start was discarded while starting:
                // nothing waits for the child any more. Confirm at once.
                _ => {
                    let opened = self.dispatch(Input::ChatOpened { chat: chat.clone() });
                    let rest = self.on_opened(chat, how);
                    Task::batch([opened, rest])
                }
            },
            Outcome::OpenFailed { chat, message } | Outcome::SendFailed { chat, message } => {
                // A failed open ends the opening phase too: `ChatFailed` clears it.
                self.dispatch(Input::ChatFailed { chat, message })
            }
            Outcome::Sent => Task::none(),
        }
    }

    /// A child that just came up is asked for what the window has to show: a
    /// stored chat's history, or the session file of a new one.
    fn on_opened(&mut self, chat: ChatId, how: Opened) -> Task<Message> {
        let request = request_after_open(&chat, how);
        self.run_chat_commands(request.into_iter().collect())
    }

    /// Every command as a notice against its chat, for a shell with no host.
    fn chat_failed(&mut self, commands: Vec<ChatCommand>, message: &str) -> Task<Message> {
        let failed = commands
            .into_iter()
            .map(|command| {
                self.dispatch(Input::ChatFailed {
                    chat: command.chat().clone(),
                    message: message.to_owned(),
                })
            })
            .collect::<Vec<_>>();
        Task::batch(failed)
    }
}

/// The safety net of a fresh start: 20 s after the host confirmed the child
/// was created, the chat is confirmed `ChatOpened` whatever it did. A child
/// that started but never speaks would otherwise leave its chat `Opening`
/// forever; a child that already spoke was cleared by the fold, and for it
/// this confirmation changes nothing. Like `execute` above, the wait happens
/// on a background thread so the window never stalls.
fn opening_net(chat: ChatId) -> Task<Message> {
    const SILENCE_TOLERATED: Duration = Duration::from_secs(20);
    // An async channel, not a blocking one: awaiting it parks the task, while a
    // blocking `recv` inside the future would hold an executor thread for the
    // whole wait and stall the delivery of every child's events with it.
    let (elapsed, wait) = iced::futures::channel::oneshot::channel::<()>();
    let _ = std::thread::Builder::new()
        .name("chat-opening-wait".into())
        .spawn(move || {
            std::thread::sleep(SILENCE_TOLERATED);
            let _ = elapsed.send(());
        });
    let confirmed = chat;
    Task::perform(
        async move {
            let _ = wait.await;
        },
        move |()| Message::Chat(Converted::Input(Input::ChatOpened { chat: confirmed })),
    )
}
