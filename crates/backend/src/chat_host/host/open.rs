//! Opening a chat: reserving its slot and starting its child in the folder the
//! caller named.
//!
//! The two slow steps of the old single open still happen here - starting a
//! child and stopping the ones a cap or the idle window took out - and they still
//! happen outside the lock, so one chat's ten-second start never freezes another
//! chat's send.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use super::super::folder::refusal;
use super::super::policy::{self, Admission};
use super::super::{ChatKey, Child, HostError, Opened};
use super::state::{Slot, Tag};
use super::ChatHost;

impl ChatHost {
    /// Opens `key` in the folder of this process, which is what a caller that
    /// names no folder inherits.
    pub fn open(&self, key: ChatKey) -> Result<Opened, HostError> {
        self.open_in(key, None)
    }

    /// Opens `key` in `folder`: starts its child there if it has none, and makes
    /// it the foreground chat either way. Never stops another child, and never
    /// sends `switch_session` or `new_session` - the session goes on the command
    /// line when the child is started.
    ///
    /// The folder is the working folder of a child that is actually started: a
    /// chat that is already up keeps the folder it started in, and this call
    /// only makes it foreground. A folder that is not there, or that is a file,
    /// fails the open before anything is reserved, so the host is exactly as it
    /// was before the call.
    ///
    /// Reopening a chat whose child died starts a fresh child, in `folder`.
    pub fn open_in(&self, key: ChatKey, folder: Option<PathBuf>) -> Result<Opened, HostError> {
        let now = self.clock.now();
        let mut stops: Vec<Box<dyn Child>> = Vec::new();
        let generation;
        // A folder only matters for a child that is actually started, and a
        // folder that cannot host one fails the open before the map changes.
        // Looking at the disk can be slow (a network drive that went away), so
        // it happens with the lock released: the state is read for an instant
        // and the folder is checked after. A chat that comes up in that gap
        // only means a refusal that did not strictly need to happen.
        let running = self
            .state()
            .slots
            .get(&key)
            .is_some_and(|slot| !slot.is_dead());
        if !running {
            if let Some(refused) = folder.as_deref().and_then(refusal) {
                return Err(refused);
            }
        }
        {
            let mut state = self.state();
            // A child that exited on its own leaves nothing usable behind: drop
            // it so reopening the chat is a fresh start.
            if state.slots.get(&key).is_some_and(Slot::is_dead) {
                if let Some(slot) = state.take(&key) {
                    stops.extend(slot.into_child());
                }
            }
            if let Some(slot) = state.slots.get_mut(&key) {
                slot.mark(None, now);
                let starting = slot.is_spawning();
                state.foreground = Some(key);
                // A child still starting is not a live one: a caller that would
                // write to it has to know the difference (S2, R3-existing-false-promise).
                return Ok(if starting {
                    Opened::Starting
                } else {
                    Opened::Existing
                });
            }
            // Idle background children go first: nothing is lost by stopping a
            // child the user has not touched for a long time.
            for victim in policy::reap_idle(&self.limits, &state.views(), now) {
                if let Some(slot) = state.take(&victim) {
                    stops.extend(slot.into_child());
                }
            }
            match policy::admit(&self.limits, &state.views()) {
                Ok(Admission::Open) => {}
                Ok(Admission::Reap(victim)) => {
                    if let Some(slot) = state.take(&victim) {
                        stops.extend(slot.into_child());
                    }
                }
                Err(too_many) => {
                    drop(state);
                    // The idle ones reaped above are idle whether or not this
                    // open is refused, so they still go.
                    self.stop_all(stops);
                    return Err(HostError::TooManyLive(too_many));
                }
            }
            generation = state.next_generation();
            state
                .slots
                .insert(key.clone(), Slot::spawning(generation, now));
            state.foreground = Some(key.clone());
        }
        // Starting and stopping a child take seconds: never under the lock.
        //
        // And the clicked chat's child starts before the children that were
        // taken out are stopped: each of those graceful shutdowns can take the
        // whole grace period, and the chat the user just clicked must not sit
        // `Spawning` behind them (R4-002). The cap is exceeded by one process for
        // as long as a shutdown takes, which is the cheaper trade.
        let started = self.spawner.spawn(&key, folder.as_deref());
        self.stop_all(stops);

        let child = match started {
            Ok(child) => child,
            Err(error) => {
                let ours = self.state().drop_reservation(&key, generation);
                return if ours {
                    Err(HostError::Spawn {
                        key,
                        message: error.to_string(),
                    })
                } else {
                    Ok(Opened::Discarded) // closed while starting
                };
            }
        };

        let alive = Arc::new(AtomicBool::new(true));
        let stopped = Arc::new(AtomicBool::new(false));
        let tag: Tag = Arc::new(Mutex::new(key.clone()));
        let forwarded = {
            let mut state = self.state();
            if let Err(child) = state.adopt(
                &key,
                generation,
                child,
                Arc::clone(&alive),
                Arc::clone(&stopped),
                Arc::clone(&tag),
            ) {
                drop(state);
                // The chat was closed or reopened while its child started (S2):
                // discard the start result instead of adopting it.
                self.stop(child);
                return Ok(Opened::Discarded);
            }
            state
                .slots
                .get_mut(&key)
                .and_then(Slot::child_mut)
                .map(|child| child.take_events())
        };
        if let Some(events) = forwarded {
            self.forward(tag, alive, stopped, events);
        }
        Ok(Opened::Started)
    }
}
