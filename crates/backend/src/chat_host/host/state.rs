//! The host's own data: the map, the foreground mark and the spawn generations.
//!
//! Nothing here starts or stops a child. It only records what is live, what is
//! reserved by an open that is still starting, and what a rule may consider —
//! so the decision code upstream stays a decision over plain facts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::policy::ChatView;
use super::super::{ChatKey, Child};

/// Everything the mutex guards.
#[derive(Default)]
pub(in crate::chat_host) struct State {
    pub(super) slots: HashMap<ChatKey, Slot>,
    pub(super) foreground: Option<ChatKey>,
    generations: u64,
}

impl State {
    /// A number no earlier reservation of the same key had, so a start result
    /// that arrives late can be told apart from a newer one.
    pub(super) fn next_generation(&mut self) -> u64 {
        self.generations += 1;
        self.generations
    }

    /// Removes a slot and drops the foreground mark if it pointed at it.
    pub(super) fn take(&mut self, key: &ChatKey) -> Option<Slot> {
        let slot = self.slots.remove(key)?;
        if self.foreground.as_ref() == Some(key) {
            self.foreground = None;
        }
        Some(slot)
    }

    /// The chats a rule may consider, in key order so a decision never depends
    /// on hash iteration order.
    pub(super) fn views(&self) -> Vec<ChatView> {
        let mut views: Vec<ChatView> = self
            .slots
            .iter()
            .filter_map(|(key, slot)| slot.view(key, self.foreground.as_ref() == Some(key)))
            .collect();
        views.sort_by(|left, right| left.key.cmp(&right.key));
        views
    }

    /// Adopts a started child, unless its chat was closed or reopened meanwhile
    /// (then the generation no longer matches). Returns the child back so the
    /// caller can stop it outside the lock.
    pub(super) fn adopt(
        &mut self,
        key: &ChatKey,
        generation: u64,
        child: Box<dyn Child>,
        alive: Arc<AtomicBool>,
        stopped: Arc<AtomicBool>,
        tag: Tag,
    ) -> Result<(), Box<dyn Child>> {
        let Some(slot) = self.slots.get_mut(key) else {
            return Err(child);
        };
        if slot.generation != generation || !slot.is_spawning() {
            return Err(child);
        }
        slot.status = Status::Live {
            child,
            alive,
            stopped,
            tag,
        };
        Ok(())
    }

    /// Removes a reservation whose start already returned, without touching a
    /// child (there is none yet). Answers whether the reservation was still ours.
    pub(super) fn drop_reservation(&mut self, key: &ChatKey, generation: u64) -> bool {
        let ours = self
            .slots
            .get(key)
            .is_some_and(|slot| slot.generation == generation && slot.is_spawning());
        if ours {
            self.take(key);
        }
        ours
    }
}

pub(super) struct Slot {
    generation: u64,
    pub(super) status: Status,
    working: bool,
    last_active: Instant,
}

impl Slot {
    pub(super) fn spawning(generation: u64, now: Instant) -> Self {
        Self {
            generation,
            status: Status::Spawning,
            working: false,
            last_active: now,
        }
    }

    pub(super) fn is_spawning(&self) -> bool {
        matches!(self.status, Status::Spawning)
    }

    pub(super) fn is_dead(&self) -> bool {
        matches!(&self.status, Status::Live { alive, .. } if !alive.load(Ordering::SeqCst))
    }

    /// The chat as a rule sees it, or `None` when its child is gone: a dead
    /// child never holds a slot against a cap.
    fn view(&self, key: &ChatKey, foreground: bool) -> Option<ChatView> {
        let pending = self.is_spawning();
        if !pending && self.is_dead() {
            return None;
        }
        Some(ChatView {
            key: key.clone(),
            working: self.working,
            foreground,
            pending,
            last_active: self.last_active,
        })
    }

    pub(super) fn mark(&mut self, working: Option<bool>, now: Instant) {
        if let Some(working) = working {
            self.working = working;
        }
        self.last_active = now;
    }

    pub(super) fn child_mut(&mut self) -> Option<&mut Box<dyn Child>> {
        match &mut self.status {
            Status::Live { child, .. } => Some(child),
            Status::Spawning => None,
        }
    }

    pub(super) fn into_child(self) -> Option<Box<dyn Child>> {
        match self.status {
            Status::Live { child, stopped, .. } => {
                // From here on, this child's closure is one the host asked for:
                // the forwarder reads this flag for the `Closed` it stamps.
                stopped.store(true, Ordering::SeqCst);
                Some(child)
            }
            Status::Spawning => None,
        }
    }

    /// Points the events of this slot's child at a new key. Only a live slot
    /// has events to tag; answers whether there was one.
    pub(super) fn retag(&self, key: ChatKey) -> bool {
        match &self.status {
            Status::Live { tag, .. } => {
                *tag.lock().unwrap_or_else(|p| p.into_inner()) = key;
                true
            }
            Status::Spawning => false,
        }
    }

    pub(super) fn shut_down(self, grace: Duration) {
        if let Some(child) = self.into_child() {
            let _ = child.shutdown(grace);
        }
    }
}

pub(super) enum Status {
    /// Reserved by an open that is starting the child right now.
    Spawning,
    Live {
        child: Box<dyn Child>,
        alive: Arc<AtomicBool>,
        /// Set when the host takes the child out to stop it, before the stop
        /// begins, so the `Closed` that follows is reported as expected.
        stopped: Arc<AtomicBool>,
        /// The key stamped on every event this child sends. A cell, not a copy,
        /// so a rekey changes what the forwarder already running reads.
        tag: Tag,
    },
}

/// The chat key a child's events are tagged with, shared with its forwarder.
pub(in crate::chat_host) type Tag = Arc<Mutex<ChatKey>>;
