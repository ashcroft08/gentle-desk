//! The handle contract the host needs from one chat's child.
//!
//! [`RpcProcess`] is the real thing; the trait exists so the host can be tested
//! against a child that is only data, with no process behind it. It is
//! deliberately the smallest surface the host uses: a writer for command lines,
//! the event stream, and a shutdown with a grace period.
//!
//! The writer is a handle of its own on purpose. Writing a command blocks when
//! the child stops reading its pipe, so the host takes a writer while it holds
//! its map lock and writes after releasing it: one stalled chat then costs one
//! stalled writer instead of every other chat's send, open, close and reap.

use std::io;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

use gentle_core::rpc::RpcCommand;

use crate::process::{ProcessEvent, RpcProcess, RpcWriter, Shutdown};

/// One command line out to one child, usable with the host's lock released.
///
/// A writer belongs to a single child, so two commands to that child keep their
/// order while commands to different children never wait for each other.
pub trait Writer: Send + Sync {
    fn send(&self, command: &RpcCommand) -> io::Result<()>;
}

pub trait Child: Send {
    /// A handle that writes one command line, or `None` when the child takes no
    /// more commands.
    fn writer(&self) -> Option<Arc<dyn Writer>>;
    /// Hands over the ordered event stream. What stays behind is a closed
    /// stream, so the host is its only consumer.
    fn take_events(&mut self) -> Receiver<ProcessEvent>;
    /// Closes the child's stdin, waits up to `grace`, then kills its whole
    /// process tree. Dropping the handle does the same.
    fn shutdown(self: Box<Self>, grace: Duration) -> io::Result<Shutdown>;
}

impl Writer for RpcWriter {
    fn send(&self, command: &RpcCommand) -> io::Result<()> {
        RpcWriter::send(self, command)
    }
}

impl Child for RpcProcess {
    fn writer(&self) -> Option<Arc<dyn Writer>> {
        RpcProcess::writer(self).map(|writer| Arc::new(writer) as Arc<dyn Writer>)
    }

    fn take_events(&mut self) -> Receiver<ProcessEvent> {
        RpcProcess::take_events(self)
    }

    fn shutdown(self: Box<Self>, grace: Duration) -> io::Result<Shutdown> {
        RpcProcess::shutdown(*self, grace)
    }
}
