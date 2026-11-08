//! A gate that holds its callers until it is released, and says how many are
//! parked so a test can act on a call that is already inside it.

use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use super::WAIT;

#[derive(Default)]
pub struct Gate {
    state: Mutex<GateState>,
    changed: Condvar,
}

#[derive(Default)]
struct GateState {
    open: bool,
    parked: usize,
}

impl Gate {
    pub fn wait(&self) {
        let mut state = self.state.lock().unwrap();
        if state.open {
            return;
        }
        state.parked += 1;
        self.changed.notify_all();
        while !state.open {
            state = self.changed.wait(state).unwrap();
        }
        state.parked -= 1;
    }

    /// Waits until `count` callers are parked in [`Gate::wait`].
    pub fn wait_until_parked(&self, count: usize) {
        let deadline = Instant::now() + WAIT;
        let mut state = self.state.lock().unwrap();
        while state.parked < count {
            assert!(
                Instant::now() < deadline,
                "only {} of {count} callers parked on the gate",
                state.parked
            );
            let (next, _) = self
                .changed
                .wait_timeout(state, Duration::from_millis(5))
                .unwrap();
            state = next;
        }
    }

    pub fn release(&self) {
        self.state.lock().unwrap().open = true;
        self.changed.notify_all();
    }
}
