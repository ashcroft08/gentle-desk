//! Feeding one input to the app layer and running everything it asks for: the
//! shared commands (clipboard, refresh) and one task per chat command.

use gentle_app::{update_chats, Command, Effects, Input};
use gentle_core::chat::Dialog;
use iced::{clipboard, Task};

use super::App;
use crate::message::Message;
use crate::view::dialog;

impl App {
    /// Feeds an input to the app layer and runs the effects it returns.
    pub(super) fn dispatch(&mut self, input: Input) -> Task<Message> {
        let dialog_before = Self::visible_chat(&self.app)
            .pending_dialogs
            .first()
            .cloned();
        let (state, effects) = update_chats(std::mem::take(&mut self.app), input);
        self.app = state;
        let chat = Self::visible_chat(&self.app);
        self.cache.sync(chat);
        let mut tasks = Vec::new();
        // A new text dialog takes the keyboard so the user can type at once.
        let dialog_now = Self::visible_chat(&self.app)
            .pending_dialogs
            .first()
            .cloned();
        if dialog_now != dialog_before
            && matches!(
                dialog_now,
                Some(Dialog::Input { .. } | Dialog::Editor { .. })
            )
        {
            tasks.push(iced::widget::operation::focus(dialog::INPUT_ID));
        }
        tasks.push(self.run(effects));
        Task::batch(tasks)
    }

    /// Runs both halves of an answer. The single child's send is gone with that
    /// child: every command to a process is a `ChatCommand` now, and a
    /// `Command::Send` that only the kept single-chat intents still produce has
    /// no process to reach.
    fn run(&mut self, effects: Effects) -> Task<Message> {
        let mut tasks = Vec::new();
        for command in effects.commands {
            match command {
                Command::CopyToClipboard(text) => tasks.push(clipboard::write(text)),
                Command::RefreshSessions => tasks.push(self.flow.on_refresh_command()),
                Command::Send(_) => {}
            }
        }
        tasks.push(self.run_chat_commands(effects.chat_commands));
        Task::batch(tasks)
    }
}
