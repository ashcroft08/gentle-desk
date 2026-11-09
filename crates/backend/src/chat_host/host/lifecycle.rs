//! Every mutation of the map except opening a chat ([`super::open`]) and writing
//! a command: close, idle reap and shutdown.
//!
//! The slow step here - stopping a child - always happens outside the lock, so a
//! graceful shutdown never freezes another chat's send.

use super::super::policy;
use super::super::ChatKey;
use super::super::Child;
use super::state::Slot;
use super::ChatHost;

impl ChatHost {
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
    /// Takes `&self` because the map is behind a mutex: a shell that shares the
    /// host (the event reader holds one reference too) can still close every
    /// child before the window exits (S5).
    pub fn shutdown(&self) {
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
