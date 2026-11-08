//! A spawner that starts no process: it records which chats were started and
//! what was written to each, so the runtime is tested without a child.

use std::io;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::chat_host::{ChatKey, Child, Spawner, Writer};
use gentle_backend::process::{ProcessEvent, Shutdown};
use gentle_core::rpc::RpcCommand;

/// What was written to one child, shared between the child and the test.
type Written = Arc<Mutex<Vec<RpcCommand>>>;

#[derive(Default)]
pub struct FakeSpawner {
    started: Mutex<Vec<(ChatKey, Written)>>,
    failure: Mutex<Option<String>>,
}

impl FakeSpawner {
    /// The chats started so far, in order.
    pub fn started(&self) -> Vec<ChatKey> {
        let started = self.started.lock().unwrap();
        started.iter().map(|(key, _)| key.clone()).collect()
    }

    /// Makes the next start fail with `message`.
    pub fn fail_next(&self, message: &str) {
        *self.failure.lock().unwrap() = Some(message.to_owned());
    }

    /// What was written to the child of `key`.
    pub fn written_to(&self, key: &ChatKey) -> Vec<RpcCommand> {
        let started = self.started.lock().unwrap();
        started
            .iter()
            .find(|(known, _)| known == key)
            .map(|(_, commands)| commands.lock().unwrap().clone())
            .unwrap_or_default()
    }
}

impl Spawner for FakeSpawner {
    fn spawn(&self, key: &ChatKey) -> io::Result<Box<dyn Child>> {
        if let Some(message) = self.failure.lock().unwrap().take() {
            return Err(io::Error::other(message));
        }
        let commands = Arc::new(Mutex::new(Vec::new()));
        self.started
            .lock()
            .unwrap()
            .push((key.clone(), Arc::clone(&commands)));
        Ok(Box::new(FakeChild { commands }))
    }
}

struct FakeChild {
    commands: Written,
}

struct FakeWriter(Written);

impl Writer for FakeWriter {
    fn send(&self, command: &RpcCommand) -> io::Result<()> {
        self.0.lock().unwrap().push(command.clone());
        Ok(())
    }
}

impl Child for FakeChild {
    fn writer(&self) -> Option<Arc<dyn Writer>> {
        Some(Arc::new(FakeWriter(Arc::clone(&self.commands))))
    }

    fn take_events(&mut self) -> Receiver<ProcessEvent> {
        mpsc::channel().1
    }

    fn shutdown(self: Box<Self>, _grace: Duration) -> io::Result<Shutdown> {
        Ok(Shutdown::Killed)
    }
}
