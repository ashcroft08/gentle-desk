//! The handle contract the host needs from one chat's child.
//!
//! [`RpcProcess`] is the real thing; the trait exists so the host can be tested
//! against a child that is only data, with no process behind it. It is
//! deliberately the smallest surface the host uses: write a command, take the
//! event stream, shut the child down with a grace period.

use std::io;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use gentle_core::rpc::RpcCommand;

use crate::process::{ProcessEvent, RpcProcess, Shutdown};

pub trait Child: Send {
    /// Writes one command line to the child.
    fn send(&mut self, command: &RpcCommand) -> io::Result<()>;
    /// Hands over the ordered event stream. What stays behind is a closed
    /// stream, so the host is its only consumer.
    fn take_events(&mut self) -> Receiver<ProcessEvent>;
    /// Closes the child's stdin, waits up to `grace`, then kills its whole
    /// process tree. Dropping the handle does the same.
    fn shutdown(self: Box<Self>, grace: Duration) -> io::Result<Shutdown>;
}

impl Child for RpcProcess {
    fn send(&mut self, command: &RpcCommand) -> io::Result<()> {
        RpcProcess::send(self, command)
    }

    fn take_events(&mut self) -> Receiver<ProcessEvent> {
        RpcProcess::take_events(self)
    }

    fn shutdown(self: Box<Self>, grace: Duration) -> io::Result<Shutdown> {
        RpcProcess::shutdown(*self, grace)
    }
}
