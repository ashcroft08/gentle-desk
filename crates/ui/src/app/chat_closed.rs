//! A chat's child going away, folded into the window.
//!
//! A death of any chat, foreground or background, becomes a per-chat notice with
//! a Restart: it never becomes the banner that used to end the whole session
//! (S3). A stop the host asked for (idle reap, cap, close) is only a log line,
//! because the chat is idle and its next open starts a child anyway.

use gentle_app::{ChatId, Input};
use iced::Task;

use super::App;
use crate::message::Message;

impl App {
    /// Folds one child's stream ending. The app layer owns what it means; this
    /// only names the case in the log so a run can be read afterwards.
    pub(super) fn on_chat_closed(&mut self, chat: ChatId, expected: bool) -> Task<Message> {
        if expected {
            eprintln!("[{}] child stopped by the host", chat.label());
        } else {
            eprintln!("[{}] child exited", chat.label());
        }
        self.dispatch(Input::ChatClosed { chat, expected })
    }
}
