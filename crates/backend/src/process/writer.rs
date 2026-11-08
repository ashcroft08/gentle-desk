//! The writing half of one `gentle-shell --mode rpc` child.
//!
//! It is a handle of its own because writing a command blocks when the child
//! stops reading its pipe: an owner can take a writer while it holds its own
//! lock, release it, and only then write. Everything that writes to one child
//! shares this handle, so two commands to that child keep the order they were
//! taken in, and a write can never race the end-of-file
//! [`RpcProcess::stop`](super::RpcProcess) sends: both go through the same slot.

use std::io::{self, Write};
use std::process::ChildStdin;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::{encode_command, RpcCommand};

/// A handle that writes command lines to one child's stdin.
#[derive(Clone)]
pub struct RpcWriter {
    stdin: Arc<Mutex<Option<ChildStdin>>>,
}

impl RpcWriter {
    /// A writer over a slot that is `None` once stdin is closed.
    pub(super) fn new(stdin: Arc<Mutex<Option<ChildStdin>>>) -> Self {
        Self { stdin }
    }

    /// Writes one command line to the child's stdin.
    pub fn send(&self, command: &RpcCommand) -> io::Result<()> {
        let mut stdin = self
            .stdin
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let stdin = stdin.as_mut().ok_or_else(stdin_closed)?;
        stdin.write_all(encode_command(command).as_bytes())?;
        stdin.flush()
    }
}

/// What a write answers once the end-of-file went out.
pub(super) fn stdin_closed() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "stdin is closed")
}
