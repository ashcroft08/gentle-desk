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
use gentle_runtime::chat_runtime::{
    ChainWriteResult, ChatConfigRead, ChatProfileApply, ModeWriteResult, Outcome, PinWriteResult,
};

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
    /// The chat's child accepted the stop of one helper (helper-stop T3).
    HelperStopped,
    /// No stop happened: no child, an inactive channel, a timeout or a refusal
    /// (helper-stop S7).
    HelperNotStopped,
    /// The chat's own child answered its profile configuration with this read
    /// (profiles-editor T2b-2).
    ChatProfile(Box<ChatConfigRead>),
    /// One profile's orchestration mode was saved (or not) and this is the
    /// configuration the attempt re-read (profiles-editor T3-mode).
    ChatMode(Box<ModeWriteResult>, Box<ChatConfigRead>),
    /// One saved profile was applied over that chat's own child, or the answer is
    /// the screen's dialog / a typed refusal (profiles-editor T3b).
    ChatApply(Box<ChatProfileApply>),
    /// One profile's fallback chain was saved (or not) and this is the
    /// configuration the attempt re-read (profiles-editor T5).
    ChatChain(Box<ChainWriteResult>, Box<ChatConfigRead>),
    /// ONE pin layer of that chat's worktree was written (or not) and this is the
    /// configuration the attempt re-read (profiles-editor T6).
    ChatPin(Box<PinWriteResult>, Box<ChatConfigRead>),
    /// The user's Close chat stopped that chat's child, as the host reports it
    /// (profiles-editor T2b-3).
    Closed,
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
            (
                Self::HelperStopped,
                ChatCommand::StopHelper {
                    helper_id: stopped, ..
                },
            ) => Outcome::Stopped {
                chat,
                helper_id: stopped.clone(),
            },
            (
                Self::HelperNotStopped,
                ChatCommand::StopHelper {
                    helper_id: stopped, ..
                },
            ) => Outcome::NotStopped {
                chat,
                helper_id: stopped.clone(),
            },
            (Self::ChatProfile(read), ChatCommand::ReadChatProfile { .. }) => {
                Outcome::ChatProfile {
                    chat,
                    read: read.clone(),
                }
            }
            (Self::ChatMode(result, read), ChatCommand::SetChatProfileMode { .. }) => {
                Outcome::ChatMode {
                    chat,
                    result: (**result).clone(),
                    read: read.clone(),
                }
            }
            (Self::ChatApply(attempt), ChatCommand::ApplyChatProfile { .. }) => {
                Outcome::ChatApply {
                    chat,
                    attempt: attempt.clone(),
                }
            }
            (Self::ChatChain(result, read), ChatCommand::SetChatProfileChain { .. }) => {
                Outcome::ChatChain {
                    chat,
                    result: (**result).clone(),
                    read: read.clone(),
                }
            }
            (Self::ChatPin(result, read), ChatCommand::SetChatProfilePin { .. }) => {
                Outcome::ChatPin {
                    chat,
                    result: (**result).clone(),
                    read: read.clone(),
                }
            }
            (Self::Closed, ChatCommand::CloseProcess { .. }) => Outcome::Closed { chat },
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
