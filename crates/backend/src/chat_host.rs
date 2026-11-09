//! One `gentle-shell --mode rpc` child per chat, owned by one manager.
//!
//! Chatting inside a single child and replacing its session (`switch_session`,
//! and `new_session` too) is what let a stale extension context kill the
//! window: this layer removes the trigger class by starting one child per chat
//! instead, with `--session <path>` when the chat already has a session file and
//! without it when the chat is new.
//!
//! What lives here is the I/O side, split by responsibility:
//!
//! - `host` - the map `ChatKey -> live child`, and every mutation of it (open,
//!   send, close, reap, shut down).
//! - [`spawner`] - turns a chat key into a started child.
//! - [`child`] - the handle contract the host needs, implemented by
//!   [`RpcProcess`](crate::process::RpcProcess).
//! - [`policy`] - every rule about reaping and caps, as pure functions over a
//!   `now` the caller supplies, so the clock is injectable and nothing in it
//!   starts or stops a process.
//!
//! The invariant the rest of the app relies on: opening a chat never stops
//! another one, so a chat that is working finishes in the background. The only
//! children that stop behind the user's back are the idle ones the [`policy`]
//! reaps and the ones a cap forces out.
//!
//! Integration note for the UI: every [`ProcessEvent`] of every child arrives on
//! one channel, each tagged with the [`ChatKey`] it came from.

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::process::ProcessEvent;

pub mod child;
mod folder;
mod host;
pub mod policy;
pub mod spawner;

pub use child::{Child, Writer};
pub use host::{ChatHost, SendError};
pub use policy::{Admission, ChatView, Clock, Limits, ManualClock, SystemClock, TooManyLive};
pub use spawner::{ProcessSpawner, Spawner};

/// Which chat a child belongs to: its session file, or an id for a chat that
/// has none yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChatKey {
    /// A chat with a stored session: its child is spawned with `--session <path>`.
    Saved(PathBuf),
    /// A new chat with no session file yet: its child is spawned without
    /// `--session`, and its session path becomes known once the child writes it.
    New(u64),
}

impl ChatKey {
    /// A key that belongs to no session file yet. Ids are unique per process and
    /// never reused, so a reopened "new" chat is a different chat.
    pub fn new_chat() -> Self {
        Self::New(NEXT_NEW.fetch_add(1, Ordering::Relaxed))
    }
}

static NEXT_NEW: AtomicU64 = AtomicU64::new(1);

impl fmt::Display for ChatKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Saved(path) => write!(f, "chat {}", path.display()),
            Self::New(id) => write!(f, "new chat {id}"),
        }
    }
}

/// One event of one child, tagged with the chat it belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedEvent {
    pub key: ChatKey,
    pub event: ProcessEvent,
    /// True when the host stopped this child itself - the idle reap, a cap
    /// victim, a close, or the shutdown of every child - instead of the child
    /// ending on its own. Only meaningful on [`ProcessEvent::Closed`]; it is how
    /// the shell tells a stop the user asked for from a death worth a notice.
    pub expected: bool,
}

/// What [`ChatHost::open`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    /// The chat already had a live child: it only became the foreground one.
    Existing,
    /// The chat's child is still starting, so there is nothing to write to yet.
    /// The open changed nothing beyond the foreground mark: the one child on its
    /// way is the one this chat will have.
    Starting,
    /// A child was started for the chat.
    Started,
    /// The chat was closed while its child was starting, so the started child
    /// was stopped instead of adopted.
    Discarded,
}

/// Why an [`ChatHost::open`] failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// The live cap is reached and no child may be stopped to make room: every
    /// one of them is working, foreground or still starting.
    TooManyLive(TooManyLive),
    /// The child could not be started (missing launcher, unreadable session,
    /// and so on). The chat has no child afterwards.
    Spawn { key: ChatKey, message: String },
    /// The folder the chat was asked to open in cannot host a child: it is not
    /// there, or it is a file. Nothing was started and nothing was reserved, so
    /// the host is exactly as it was before the call.
    Folder { message: String },
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyLive(too_many) => write!(f, "{too_many}"),
            Self::Spawn { key, message } => {
                write!(f, "cannot start the child of {key}: {message}")
            }
            Self::Folder { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for HostError {}

#[cfg(test)]
mod tests;
