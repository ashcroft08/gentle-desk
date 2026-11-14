//! Running one per-chat command against the host.

use std::path::PathBuf;

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
    /// The chat's child is gone, because this command stopped it (or because it
    /// had none). The chat itself is unchanged: its next open starts a fresh
    /// child through `--session` (0.1.14).
    Closed { chat: ChatId },
}

impl ChatRuntime {
    /// Runs one command, in the folder of this process.
    pub fn execute(&self, command: ChatCommand) -> Outcome {
        self.execute_in(command, None)
    }

    /// Runs one command with the folder a workspace named for the chat it
    /// opens. The folder only matters for a child that is actually started: an
    /// open of a chat that is already live keeps the folder it started in, and a
    /// send goes to a child that already has one. An open in a folder that
    /// cannot host a child fails the way the host says it does, before anything
    /// is started.
    pub fn execute_in(&self, command: ChatCommand, folder: Option<PathBuf>) -> Outcome {
        match command {
            ChatCommand::OpenProcess { chat } => self.open(chat, folder),
            ChatCommand::Send { chat, rpc } => self.send(chat, &rpc),
            // The child is stopped here, on the caller's thread: the host's grace
            // is up to 2 s and nothing of the window may wait on it (0.1.14).
            ChatCommand::CloseProcess { chat } => {
                self.close(&chat);
                Outcome::Closed { chat }
            }
        }
    }

    fn open(&self, chat: ChatId, folder: Option<PathBuf>) -> Outcome {
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
        match self.host.open_in(key, folder) {
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
