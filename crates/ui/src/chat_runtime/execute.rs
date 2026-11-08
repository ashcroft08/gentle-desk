//! Running one per-chat command against the host.

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::Opened;
use gentle_core::rpc::RpcCommand;

use super::ChatRuntime;

/// What running a [`ChatCommand`] came to. Plain data: a failure is a message
/// for the shell to show, never a panic and never fatal for the whole app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The chat's child is up (started now, or already was).
    Opened { chat: ChatId, how: Opened },
    /// The child could not be started or the live cap left no room.
    OpenFailed { chat: ChatId, message: String },
    /// The command reached the chat's child.
    Sent,
    /// The command did not reach a child (none, still starting, or a broken pipe).
    SendFailed { chat: ChatId, message: String },
}

impl ChatRuntime {
    /// Runs one command. Blocking: an open waits for the child to start.
    pub fn execute(&self, command: ChatCommand) -> Outcome {
        match command {
            ChatCommand::OpenProcess { chat } => self.open(chat),
            ChatCommand::Send { chat, rpc } => self.send(chat, &rpc),
        }
    }

    fn open(&self, chat: ChatId) -> Outcome {
        // The registry lock ends before `open`, which waits for the child.
        let key = {
            let mut keys = self.keys();
            keys.key_of(&chat).or_else(|| keys.register_new(&chat))
        };
        let Some(key) = key else {
            return Outcome::OpenFailed {
                chat,
                message: "this chat has no process to open".to_owned(),
            };
        };
        match self.host.open(key) {
            Ok(how) => Outcome::Opened { chat, how },
            Err(error) => Outcome::OpenFailed {
                chat,
                message: error.to_string(),
            },
        }
    }

    fn send(&self, chat: ChatId, rpc: &RpcCommand) -> Outcome {
        let key = self.keys().key_of(&chat);
        let Some(key) = key else {
            return Outcome::SendFailed {
                chat,
                message: "this chat has no process yet".to_owned(),
            };
        };
        match self.host.send(&key, rpc) {
            Ok(()) => Outcome::Sent,
            Err(error) => Outcome::SendFailed {
                chat,
                message: error.to_string(),
            },
        }
    }
}
