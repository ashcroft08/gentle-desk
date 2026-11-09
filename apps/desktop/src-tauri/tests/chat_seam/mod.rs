//! The recording fake of the chat effects' seam, shared by the effect tests.
//!
//! It answers what a test says and remembers every command it was asked, so the
//! tests name the outcome they are about instead of building one inline. No
//! process, no window, no launcher ever runs here.
#![allow(dead_code)] // each test binary uses one part of it

use std::sync::Mutex;

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_desk_lib::effects::ChatExecutor;
use gentle_runtime::chat_runtime::Outcome;

/// A stored session, as the sidebar names one: by its path, before any chat of
/// it exists. The app builds the chat id from it.
pub const SESSION_PATH: &str = "/sessions/a.jsonl";

pub fn stored() -> ChatId {
    ChatId::of_session(SESSION_PATH)
}

/// What the fake answers, named per test. It answers like a host: the answer is
/// about the open, and a send to a chat whose child came up is delivered.
pub enum Answer {
    Sent,
    Started,
    Existing,
    OpenFailed(&'static str),
}

impl Answer {
    fn of(&self, command: &ChatCommand) -> Outcome {
        let chat = command.chat().clone();
        let opened = |how| Outcome::Opened {
            chat: chat.clone(),
            how,
        };
        match (self, command) {
            (Self::OpenFailed(message), ChatCommand::OpenProcess { .. }) => Outcome::OpenFailed {
                chat,
                message: (*message).to_owned(),
            },
            (Self::OpenFailed(message), ChatCommand::Send { .. }) => Outcome::SendFailed {
                chat,
                message: (*message).to_owned(),
            },
            (Self::Started, ChatCommand::OpenProcess { .. }) => opened(Opened::Started),
            (Self::Existing, ChatCommand::OpenProcess { .. }) => opened(Opened::Existing),
            // Sent, and every send after an open: the child took it.
            _ => Outcome::Sent,
        }
    }
}

pub struct Fake {
    answer: Answer,
    asked: Mutex<Vec<ChatCommand>>,
}

impl Fake {
    pub fn answering(answer: Answer) -> Self {
        Self {
            answer,
            asked: Mutex::new(Vec::new()),
        }
    }

    /// Every command the fake was asked, in order.
    pub fn asked(&self) -> Vec<ChatCommand> {
        self.asked.lock().expect("asked mutex").clone()
    }
}

impl ChatExecutor for Fake {
    fn execute(&self, command: ChatCommand) -> Outcome {
        let outcome = self.answer.of(&command);
        self.asked.lock().expect("asked mutex").push(command);
        outcome
    }
}
