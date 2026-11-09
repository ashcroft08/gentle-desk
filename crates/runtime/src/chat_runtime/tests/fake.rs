//! A spawner that starts no process: it records which chats were started, what
//! was written to each and which children were shut down, so the runtime is
//! tested without a child.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::chat_host::{ChatKey, Child, Spawner, Writer};
use gentle_backend::process::{ProcessEvent, Shutdown};
use gentle_core::rpc::RpcCommand;

/// What was written to one child, shared between the child and the test.
type Written = Arc<Mutex<Vec<RpcCommand>>>;
/// The children shut down so far, shared between the children and the test.
type Stopped = Arc<Mutex<Vec<ChatKey>>>;

#[derive(Default)]
pub struct FakeSpawner {
    started: Mutex<Vec<(ChatKey, Written)>>,
    /// The folder each start was given, in the order of `started`.
    folders: Mutex<Vec<Option<PathBuf>>>,
    stopped: Stopped,
    failure: Mutex<Option<String>>,
}

impl FakeSpawner {
    /// The chats started so far, in order.
    pub fn started(&self) -> Vec<ChatKey> {
        let started = self.started.lock().unwrap();
        started.iter().map(|(key, _)| key.clone()).collect()
    }

    /// The folder every start was given, in the order of [`FakeSpawner::started`].
    pub fn folders(&self) -> Vec<Option<PathBuf>> {
        self.folders.lock().unwrap().clone()
    }

    /// The chats whose child was shut down, in order.
    pub fn stopped(&self) -> Vec<ChatKey> {
        self.stopped.lock().unwrap().clone()
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
    fn spawn(&self, key: &ChatKey, folder: Option<&Path>) -> io::Result<Box<dyn Child>> {
        if let Some(message) = self.failure.lock().unwrap().take() {
            return Err(io::Error::other(message));
        }
        self.folders
            .lock()
            .unwrap()
            .push(folder.map(Path::to_path_buf));
        let commands = Arc::new(Mutex::new(Vec::new()));
        self.started
            .lock()
            .unwrap()
            .push((key.clone(), Arc::clone(&commands)));
        // The child keeps the writing half of its own channel, so its event
        // stream stays open and the host reads it as alive until it is shut down.
        let (events, incoming) = mpsc::channel();
        Ok(Box::new(FakeChild {
            key: key.clone(),
            commands,
            stopped: Arc::clone(&self.stopped),
            events: Some(incoming),
            _alive: events,
        }))
    }
}

struct FakeChild {
    key: ChatKey,
    commands: Written,
    stopped: Stopped,
    /// Handed to the host once, on the first `take_events`.
    events: Option<Receiver<ProcessEvent>>,
    /// Keeps the child's own event stream open while it lives.
    _alive: Sender<ProcessEvent>,
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
        self.events.take().unwrap_or_else(|| mpsc::channel().1)
    }

    fn shutdown(self: Box<Self>, _grace: Duration) -> io::Result<Shutdown> {
        self.stopped.lock().unwrap().push(self.key.clone());
        Ok(Shutdown::Killed)
    }
}
