//! Commands out to one chat's child, and the working flag in.

use std::io;
use std::sync::atomic::Ordering;

use gentle_core::rpc::RpcCommand;

use super::super::policy;
use super::super::ChatKey;
use super::state::Status;
use super::ChatHost;

impl ChatHost {
    /// Routes one command to that chat's child and nothing else.
    ///
    /// The host's lock is released before the command is written. A pipe write
    /// blocks for as long as the child stops reading, and holding the map lock
    /// across it would stall every other chat's send, open, close and reap for
    /// that long; the writer is a handle of its own, so the write happens after
    /// the lock is gone (R3-send-holds-lock).
    pub fn send(&self, key: &ChatKey, command: &RpcCommand) -> Result<(), SendError> {
        let now = self.clock.now();
        let writer = {
            let mut state = self.state();
            let slot = state
                .slots
                .get_mut(key)
                .ok_or_else(|| SendError::NoChild(key.clone()))?;
            let writer = match &slot.status {
                Status::Spawning => return Err(SendError::Starting(key.clone())),
                Status::Live { child, alive, .. } => {
                    if !alive.load(Ordering::SeqCst) {
                        return Err(SendError::NoChild(key.clone()));
                    }
                    child.writer().ok_or_else(|| SendError::Write {
                        key: key.clone(),
                        error: io::Error::new(io::ErrorKind::BrokenPipe, "stdin is closed"),
                    })?
                }
            };
            // Touched before the write: a chat whose child stops reading is the
            // one the idle reap must leave alone while its write is parked.
            slot.mark(None, now);
            writer
        };
        writer.send(command).map_err(|error| SendError::Write {
            key: key.clone(),
            error,
        })
    }

    /// Tells the host whether a chat is running a model call, as the app derives
    /// it from `agent_start` / `agent_end`. A working chat is never reaped and
    /// never used to make room. Answers `false` for a chat the host does not know.
    pub fn set_working(&self, key: &ChatKey, working: bool) -> bool {
        let now = self.clock.now();
        match self.state().slots.get_mut(key) {
            Some(slot) => {
                slot.mark(Some(working), now);
                true
            }
            None => false,
        }
    }

    /// Whether another chat may start a model call, under
    /// [`Limits::max_working`](super::super::Limits::max_working).
    pub fn may_start_work(&self) -> bool {
        !policy::working_full(&self.limits, &self.state().views())
    }
}

/// Why a command could not be routed.
#[derive(Debug)]
pub enum SendError {
    /// The chat has no live child: never opened, closed, or the child exited.
    NoChild(ChatKey),
    /// The chat's child is still starting.
    Starting(ChatKey),
    /// The command could not be written to the child.
    Write { key: ChatKey, error: std::io::Error },
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoChild(key) => write!(f, "{key} has no live child"),
            Self::Starting(key) => write!(f, "the child of {key} is still starting"),
            Self::Write { key, error } => write!(f, "cannot write to {key}: {error}"),
        }
    }
}

impl std::error::Error for SendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { error, .. } => Some(error),
            _ => None,
        }
    }
}
