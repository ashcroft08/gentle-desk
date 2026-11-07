//! What goes into `update` and what comes out of it.

use gentle_core::rpc::{RpcCommand, RpcEvent, UiAnswer};

use crate::state::SessionItem;

/// What the user asked for; the UI only ever emits these.
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    SendPrompt(String),
    Abort,
    NewChat,
    /// Copy the raw markdown of one message.
    CopyMessage(String),
    AnswerDialog {
        id: String,
        answer: UiAnswer,
    },
    DismissNotice(u64),
    /// Reload the session list.
    RefreshSessions,
    /// Switch to a saved session and show its history.
    OpenSession(String),
    /// Open the thread of the helper with this id.
    SelectHelper(String),
    /// Back from a helper's thread to the helpers list.
    CloseHelper,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Intent(Intent),
    Rpc(RpcEvent),
    /// An RPC event with the wall-clock time (epoch ms) it arrived at. The shell
    /// supplies it so `update` stays pure; only helper bookkeeping uses it.
    RpcAt(RpcEvent, u64),
    /// The shell finished listing sessions.
    SessionsLoaded(Vec<SessionItem>),
}

/// A side effect for the shell to run.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Send(RpcCommand),
    CopyToClipboard(String),
    /// Refresh the session index and report back with `Input::SessionsLoaded`.
    RefreshSessions,
}
