//! Every mutation of the map except writing a command: open, close, idle reap
//! and shutdown.
//!
//! The two slow steps — starting a child and stopping one — always happen
//! outside the lock. That is what keeps one chat's ten-second start from
//! freezing another chat's send, and it is why a start result is checked
//! against its generation before it is adopted.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use super::super::policy::{self, Admission};
use super::super::{ChatKey, Child, HostError, Opened};
use super::state::{Slot, Tag};
use super::ChatHost;

impl ChatHost {
    /// Opens `key`: starts its child if it has none, and makes it the
    /// foreground chat either way. Never stops another child, and never sends
    /// `switch_session` or `new_session` — the session goes on the command line
    /// when the child is started.
    ///
    /// Reopening a chat whose child died starts a fresh child.
    pub fn open(&self, key: ChatKey) -> Result<Opened, HostError> {
        let now = self.clock.now();
        let mut stops: Vec<Box<dyn Child>> = Vec::new();
        let generation;
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
                state.foreground = Some(key);
                return Ok(Opened::Existing);
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
        let started = self.spawner.spawn(&key);
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
        let tag: Tag = Arc::new(Mutex::new(key.clone()));
        let forwarded = {
            let mut state = self.state();
            if let Err(child) = state.adopt(
                &key,
                generation,
                child,
                Arc::clone(&alive),
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
            self.forward(tag, alive, events);
        }
        Ok(Opened::Started)
    }

    /// Stops one chat's child, if it has one, and forgets the chat. A start
    /// that is still in flight for it is discarded when it arrives.
    pub fn close(&self, key: &ChatKey) -> bool {
        let slot = self.state().take(key);
        match slot {
            Some(slot) => {
                slot.shut_down(self.grace);
                true
            }
            None => false,
        }
    }

    /// Stops the idle background children and answers which ones went. The app
    /// calls this on a timer; [`ChatHost::open`] applies the same rule first.
    pub fn reap_idle(&self) -> Vec<ChatKey> {
        let now = self.clock.now();
        let reaped: Vec<ChatKey>;
        let mut stops: Vec<Box<dyn Child>> = Vec::new();
        {
            let mut state = self.state();
            reaped = policy::reap_idle(&self.limits, &state.views(), now);
            for key in &reaped {
                if let Some(slot) = state.take(key) {
                    stops.extend(slot.into_child());
                }
            }
        }
        self.stop_all(stops);
        reaped
    }

    /// Stops every child. `Drop` does the same, so no child outlives the host.
    pub fn shutdown(&mut self) {
        let slots: Vec<Slot> = {
            let mut state = self.state();
            state.foreground = None;
            state.slots.drain().map(|(_, slot)| slot).collect()
        };
        for slot in slots {
            slot.shut_down(self.grace);
        }
    }

    /// Shuts a child down with the host's grace: stdin first, then the whole
    /// process tree.
    pub(super) fn stop(&self, child: Box<dyn Child>) {
        let _ = child.shutdown(self.grace);
    }

    pub(super) fn stop_all(&self, children: Vec<Box<dyn Child>>) {
        for child in children {
            self.stop(child);
        }
    }
}
