//! Sending what the user typed.
//!
//! One turn at a time, into a chat that can receive it: before the first chat is
//! open there is nowhere for the prompt to go, and a chat whose child died has to
//! be restarted first (S3). A refused prompt stays in the box.

use gentle_app::Intent;
use iced::Task;

use super::{visible, App};
use crate::message::Message;

impl App {
    pub(super) fn send_prompt(&mut self) -> Task<Message> {
        let prompt = std::mem::take(&mut self.input);
        // One turn at a time, and only into a chat that has a child: before the
        // first chat is open there is nowhere for the prompt to go.
        let has_chat = self.app.chats.foreground().is_some();
        let working = visible::chat(&self.app).working;
        if !has_chat || working || !visible::can_send(&self.app, &prompt, self.fatal.is_some()) {
            self.input = prompt; // keep what the user typed
            return Task::none();
        }
        self.intent(Intent::SendPrompt(prompt))
    }
}
