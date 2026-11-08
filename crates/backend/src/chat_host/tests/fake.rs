//! A spawner whose children are only data: a test pushes their events and reads
//! what they were asked to do, with no process behind them.

use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use gentle_core::rpc::RpcCommand;

use crate::chat_host::child::Child;
use crate::chat_host::spawner::Spawner;
use crate::chat_host::ChatKey;
use crate::process::{ProcessEvent, Shutdown};

pub const WAIT: Duration = Duration::from_secs(5);

/// One fake child: a command log, an event channel and a shutdown counter.
#[derive(Clone)]
pub struct FakeChild(Arc<Mutex<Inner>>);

struct Inner {
    commands: Vec<RpcCommand>,
    tx: Sender<ProcessEvent>,
    rx: Option<Receiver<ProcessEvent>>,
    shutdowns: usize,
}

impl FakeChild {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self(Arc::new(Mutex::new(Inner {
            commands: Vec::new(),
            tx,
            rx: Some(rx),
            shutdowns: 0,
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

impl Child for FakeChild {
    fn send(&mut self, command: &RpcCommand) -> io::Result<()> {
        self.0.lock().unwrap().commands.push(command.clone());
        Ok(())
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
        self.0.lock().unwrap().shutdowns += 1;
        Ok(Shutdown::Killed)
    }
}

/// A gate that holds every spawn until it is released.
#[derive(Default)]
pub struct Gate {
    open: Mutex<bool>,
    released: Condvar,
}

impl Gate {
    pub fn wait(&self) {
        let mut open = self.open.lock().unwrap();
        while !*open {
            open = self.released.wait(open).unwrap();
        }
    }

    pub fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.released.notify_all();
    }
}

/// The spawner itself: what was asked for, and which child answered.
pub struct FakeSpawner {
    state: Mutex<SpawnState>,
    gate: Option<Arc<Gate>>,
}

#[derive(Default)]
struct SpawnState {
    spawned: Vec<ChatKey>,
    children: HashMap<ChatKey, Vec<FakeChild>>,
    failures: HashMap<ChatKey, String>,
}

impl FakeSpawner {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(SpawnState::default()),
            gate: None,
        }
    }

    /// A spawner whose every spawn blocks until the gate is released.
    pub fn gated() -> (Self, Arc<Gate>) {
        let gate = Arc::new(Gate::default());
        (
            Self {
                state: Mutex::new(SpawnState::default()),
                gate: Some(Arc::clone(&gate)),
            },
            gate,
        )
    }

    /// Makes every spawn of `key` fail with `message`.
    pub fn fail(&self, key: ChatKey, message: &str) {
        self.state
            .lock()
            .unwrap()
            .failures
            .insert(key, message.to_owned());
    }

    /// The keys it was asked to start, in order.
    pub fn spawned_keys(&self) -> Vec<ChatKey> {
        self.state.lock().unwrap().spawned.clone()
    }

    pub fn spawn_count(&self, key: &ChatKey) -> usize {
        self.spawned_keys()
            .iter()
            .filter(|seen| *seen == key)
            .count()
    }

    /// Every child started for `key`, in start order.
    pub fn children(&self, key: &ChatKey) -> Vec<FakeChild> {
        self.state
            .lock()
            .unwrap()
            .children
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    /// The last child started for `key`.
    pub fn child(&self, key: &ChatKey) -> FakeChild {
        self.children(key)
            .pop()
            .unwrap_or_else(|| panic!("no child was started for {key}"))
    }

    /// Waits until `count` children have been requested, so a test can act on a
    /// start that is still in flight.
    pub fn wait_for_spawns(&self, count: usize) {
        let deadline = Instant::now() + WAIT;
        while self.spawned_keys().len() < count {
            assert!(
                Instant::now() < deadline,
                "only {} of {count} spawns were requested",
                self.spawned_keys().len()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Spawner for FakeSpawner {
    fn spawn(&self, key: &ChatKey) -> io::Result<Box<dyn Child>> {
        {
            let mut state = self.state.lock().unwrap();
            if let Some(message) = state.failures.get(key) {
                return Err(io::Error::other(message.clone()));
            }
            state.spawned.push(key.clone());
            let child = FakeChild::new();
            state
                .children
                .entry(key.clone())
                .or_default()
                .push(child.clone());
        }
        if let Some(gate) = &self.gate {
            gate.wait();
        }
        Ok(Box::new(self.child(key)))
    }
}
