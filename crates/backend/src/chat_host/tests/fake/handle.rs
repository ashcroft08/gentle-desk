//! One fake child: a command log, an event channel and a shutdown counter, all
//! behind one lock, plus the option to park inside a write.

use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_core::rpc::RpcCommand;

use super::super::super::child::{Child, Writer};
use super::Gate;
use crate::chat_host::tests::fake::Journal;
use crate::process::{ProcessEvent, Shutdown};

#[derive(Clone)]
pub struct FakeChild(Arc<Mutex<Inner>>);

struct Inner {
    label: String,
    commands: Vec<RpcCommand>,
    tx: Sender<ProcessEvent>,
    rx: Option<Receiver<ProcessEvent>>,
    shutdowns: usize,
    journal: Journal,
    write_gate: Option<Arc<Gate>>,
}

impl FakeChild {
    pub(super) fn new(label: &str, journal: Journal, write_gate: Option<Arc<Gate>>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self(Arc::new(Mutex::new(Inner {
            label: label.to_owned(),
            commands: Vec::new(),
            tx,
            rx: Some(rx),
            shutdowns: 0,
            journal,
            write_gate,
        })))
    }

    /// Sends one event to whatever took the child's stream, as the child would.
    pub fn push(&self, event: ProcessEvent) {
        self.0
            .lock()
            .unwrap()
            .tx
            .send(event)
            .expect("the host still reads this child");
    }

    /// The commands written to this child, in order.
    pub fn commands(&self) -> Vec<RpcCommand> {
        self.0.lock().unwrap().commands.clone()
    }

    /// How many times this child was shut down.
    pub fn shutdowns(&self) -> usize {
        self.0.lock().unwrap().shutdowns
    }
}

/// One command line into the child. It parks on the child's write gate with the
/// child's own lock released, so a parked write never blocks a shutdown.
fn write(child: &Mutex<Inner>, command: &RpcCommand) -> io::Result<()> {
    let gate = child.lock().unwrap().write_gate.clone();
    if let Some(gate) = gate {
        gate.wait();
    }
    child.lock().unwrap().commands.push(command.clone());
    Ok(())
}

/// The writing half of a fake child: the handle the host takes under its lock.
pub struct FakeWriter(Arc<Mutex<Inner>>);

impl Writer for FakeWriter {
    fn send(&self, command: &RpcCommand) -> io::Result<()> {
        write(&self.0, command)
    }
}

impl Child for FakeChild {
    fn writer(&self) -> Option<Arc<dyn Writer>> {
        Some(Arc::new(FakeWriter(Arc::clone(&self.0))))
    }

    fn take_events(&mut self) -> Receiver<ProcessEvent> {
        self.0
            .lock()
            .unwrap()
            .rx
            .take()
            .unwrap_or_else(|| mpsc::channel().1)
    }

    fn shutdown(self: Box<Self>, _grace: Duration) -> io::Result<Shutdown> {
        let (label, journal) = {
            let mut inner = self.0.lock().unwrap();
            inner.shutdowns += 1;
            (inner.label.clone(), Arc::clone(&inner.journal))
        };
        journal.lock().unwrap().push(format!("shutdown {label}"));
        Ok(Shutdown::Killed)
    }
}
