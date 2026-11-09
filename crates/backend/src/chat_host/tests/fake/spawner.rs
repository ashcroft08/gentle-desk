//! The spawner itself: what was asked for, which child answered, and the journal
//! of every start and stop.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::super::child::Child;
use super::super::super::spawner::Spawner;
use super::super::super::ChatKey;
use super::{FakeChild, Gate, Journal, WAIT};

pub struct FakeSpawner {
    state: Mutex<SpawnState>,
    spawn_gate: Option<Arc<Gate>>,
    /// The chat whose children park inside every write, with the gate they park on.
    blocked_writes: Option<(ChatKey, Arc<Gate>)>,
    journal: Journal,
}

#[derive(Default)]
struct SpawnState {
    spawned: Vec<ChatKey>,
    children: HashMap<ChatKey, Vec<FakeChild>>,
    /// The folder each child was given, per chat, in start order.
    folders: HashMap<ChatKey, Vec<Option<PathBuf>>>,
    failures: HashMap<ChatKey, String>,
}

impl FakeSpawner {
    pub fn new() -> Self {
        Self::with_parts(None, None)
    }

    /// A spawner whose every spawn blocks until the gate is released.
    pub fn gated() -> (Self, Arc<Gate>) {
        let gate = Arc::new(Gate::default());
        (Self::with_parts(Some(Arc::clone(&gate)), None), gate)
    }

    /// A spawner whose children of `key` park inside every write until the gate
    /// is released: the pipe of a child that cannot keep up.
    pub fn with_blocked_writes(key: ChatKey) -> (Self, Arc<Gate>) {
        let gate = Arc::new(Gate::default());
        let blocked = Some((key, Arc::clone(&gate)));
        (Self::with_parts(None, blocked), gate)
    }

    fn with_parts(
        spawn_gate: Option<Arc<Gate>>,
        blocked_writes: Option<(ChatKey, Arc<Gate>)>,
    ) -> Self {
        Self {
            state: Mutex::new(SpawnState::default()),
            spawn_gate,
            blocked_writes,
            journal: Journal::default(),
        }
    }

    /// Every start and stop, in order, across all the children.
    pub fn journal(&self) -> Vec<String> {
        self.journal.lock().unwrap().clone()
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

    /// The folder every child of `key` was asked to start in, in start order.
    pub fn folders_of(&self, key: &ChatKey) -> Vec<Option<PathBuf>> {
        self.state
            .lock()
            .unwrap()
            .folders
            .get(key)
            .cloned()
            .unwrap_or_default()
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
    fn spawn(&self, key: &ChatKey, folder: Option<&Path>) -> io::Result<Box<dyn Child>> {
        {
            let mut state = self.state.lock().unwrap();
            if let Some(message) = state.failures.get(key) {
                return Err(io::Error::other(message.clone()));
            }
            state.spawned.push(key.clone());
            state
                .folders
                .entry(key.clone())
                .or_default()
                .push(folder.map(Path::to_path_buf));
            let index = state.children.get(key).map_or(0, Vec::len);
            let label = format!("{key}#{index}");
            let writes = match &self.blocked_writes {
                Some((blocked, gate)) if blocked == key => Some(Arc::clone(gate)),
                _ => None,
            };
            let child = FakeChild::new(&label, Arc::clone(&self.journal), writes);
            self.journal.lock().unwrap().push(format!("spawn {label}"));
            state.children.entry(key.clone()).or_default().push(child);
        }
        if let Some(gate) = &self.spawn_gate {
            gate.wait();
        }
        Ok(Box::new(self.child(key)))
    }
}
