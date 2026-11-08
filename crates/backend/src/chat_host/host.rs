//! The map `ChatKey -> live child`: the struct, how it is built, how every
//! child's events fan into one tagged stream, and the accessors the app reads.
//!
//! Mutations live next door: [`lifecycle`] starts, reopens, closes, reaps and
//! stops children, [`routing`] writes commands and records which chats are
//! working, and [`state`] is the map itself with its generation bookkeeping.
//!
//! One mutex guards the map, the foreground chat and the spawn generations, and
//! it is never held while a child starts or stops, nor while a command is
//! written: all three can take seconds, and a held lock there would stall every
//! other chat. So an open reserves its slot, releases the lock to start the
//! child, and re-checks its generation when the start returns; a
//! [`ChatHost::close`] in between makes the generation stale and the child that
//! arrives is stopped instead of adopted (S2). A send takes the child's writer
//! under the lock and writes only after releasing it, so a child that stops
//! reading its pipe stalls its own chat and nothing else.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use super::policy::{self, Clock, Limits, SystemClock};
use super::spawner::{ProcessSpawner, Spawner};
use super::{ChatKey, TaggedEvent};
use crate::launcher::LauncherError;
use crate::process::ProcessEvent;

use state::{State, Tag};

mod lifecycle;
mod rekey;
mod routing;
mod state;

pub use routing::SendError;

/// How long a child gets to leave on its own before it is killed with its
/// descendants: the grace [`crate::process::RpcProcess::shutdown`] already uses.
const DEFAULT_GRACE: Duration = Duration::from_secs(2);

/// One child per chat, started on demand and stopped only when it is idle, when
/// a cap forces it out, when its chat is closed, or when the host shuts down.
pub struct ChatHost {
    pub(super) limits: Limits,
    pub(super) spawner: Arc<dyn Spawner>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) grace: Duration,
    state: Mutex<State>,
    events_tx: Sender<TaggedEvent>,
    events_rx: Mutex<Receiver<TaggedEvent>>,
}

impl ChatHost {
    /// A host whose children are real `gentle-shell` processes resolved from
    /// the process environment.
    pub fn real(limits: Limits) -> Result<Self, LauncherError> {
        Ok(Self::with_parts(
            limits,
            Arc::new(ProcessSpawner::resolve()?),
            Arc::new(SystemClock),
        ))
    }

    /// Every dependency injected: the caps, where children come from and where
    /// "now" is read. Tests inject the last two.
    pub fn with_parts(limits: Limits, spawner: Arc<dyn Spawner>, clock: Arc<dyn Clock>) -> Self {
        let (events_tx, events_rx) = mpsc::channel();
        Self {
            limits,
            spawner,
            clock,
            grace: DEFAULT_GRACE,
            state: Mutex::new(State::default()),
            events_tx,
            events_rx: Mutex::new(events_rx),
        }
    }

    /// Overrides how long a child gets to exit before it is killed (2 s).
    pub fn with_grace(mut self, grace: Duration) -> Self {
        self.grace = grace;
        self
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// The live chats, in key order (children still starting included).
    pub fn live_keys(&self) -> Vec<ChatKey> {
        let state = self.state();
        let mut keys: Vec<ChatKey> = state
            .slots
            .iter()
            .filter(|(_, slot)| !slot.is_dead())
            .map(|(key, _)| key.clone())
            .collect();
        keys.sort();
        keys
    }

    /// Whether the chat has a child that is up. A child that exited is not
    /// alive, and the next [`ChatHost::open`] of its chat starts a fresh one.
    pub fn is_alive(&self, key: &ChatKey) -> bool {
        let state = self.state();
        state.slots.get(key).is_some_and(|slot| {
            matches!(slot.status, state::Status::Live { .. }) && !slot.is_dead()
        })
    }

    /// The chat the user is on, if any.
    pub fn foreground(&self) -> Option<ChatKey> {
        self.state().foreground.clone()
    }

    /// How many chats are working right now.
    pub fn working_count(&self) -> usize {
        policy::working_count(&self.state().views())
    }

    /// Hands the tagged event stream to another owner. What stays behind is a
    /// closed stream, so there is never a second consumer.
    pub fn take_events(&mut self) -> Receiver<TaggedEvent> {
        let mut guard = self.events_rx.lock().unwrap_or_else(|p| p.into_inner());
        std::mem::replace(&mut *guard, mpsc::channel().1)
    }

    /// The next tagged event, for a caller that polls instead of owning the
    /// stream.
    pub fn recv_timeout(&self, wait: Duration) -> Result<TaggedEvent, RecvTimeoutError> {
        self.events_rx
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .recv_timeout(wait)
    }

    pub(super) fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Forwards one child's event stream onto the host's channel, tagging every
    /// event with its chat, and clears the child's alive flag when it goes.
    pub(super) fn forward(&self, tag: Tag, alive: Arc<AtomicBool>, events: Receiver<ProcessEvent>) {
        let sender = self.events_tx.clone();
        thread::Builder::new()
            .name("chat-host-events".to_owned())
            .spawn(move || {
                while let Ok(event) = events.recv() {
                    if event == ProcessEvent::Closed {
                        alive.store(false, Ordering::SeqCst);
                    }
                    let key = tag.lock().unwrap_or_else(|p| p.into_inner()).clone();
                    let tagged = TaggedEvent { key, event };
                    if sender.send(tagged).is_err() {
                        break; // the host is gone
                    }
                }
                alive.store(false, Ordering::SeqCst);
            })
            .ok();
    }
}

impl Drop for ChatHost {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl std::fmt::Debug for ChatHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatHost")
            .field("limits", &self.limits)
            .field("live", &self.live_keys())
            .finish_non_exhaustive()
    }
}
