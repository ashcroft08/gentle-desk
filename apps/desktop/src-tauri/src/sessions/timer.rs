//! The safety-net timer: one full pass every `tick_delay`, and nothing else.
//!
//! The ROLE is the iced shell's (`crates/ui/src/subscription.rs`:
//! `iced::time::every(delay)`), never its code. iced rebuilt that subscription
//! whenever `Flow::tick_delay` changed, which is how the cadence follows the
//! watcher: 5 s while the list polls (`session_schedule::POLL_BASE`), 60 s while
//! a watcher reconciles (`RECONCILE_BASE`), both stretched to `K` times the last
//! full pass. Here the thread re-reads the delay every [`IDLE`] slice and
//! restarts its window when the answer changed, so a lost (or restarted)
//! watcher moves the cadence at once.
//!
//! The delay itself is the flow's decision, handed in as [`TickDelay`]: the app
//! passes [`gentle_runtime::refresh_flow::Flow::tick_delay`], a test passes a
//! short value so it never waits out the real 5 s poll.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use gentle_runtime::refresh_flow::Flow;

/// How long the safety net waits before each pass: the flow's own
/// [`Flow::tick_delay`] in the app, a short value in a test.
pub type TickDelay = Arc<dyn Fn(&Flow) -> Duration + Send + Sync>;

/// The delay as the timer thread reads it: `None` when the run is gone.
pub type DelaySource = Arc<dyn Fn() -> Option<Duration> + Send + Sync>;

/// What the timer asks for when it fires.
pub type OnTick = Arc<dyn Fn() + Send + Sync>;

/// How long the thread sleeps between two looks at the delay, and therefore the
/// longest a changed cadence or a close waits. Named for the same reason as the
/// batch thread's `IDLE`: a blocked sleep could never be woken in time.
const IDLE: Duration = Duration::from_millis(100);

/// The safety net of this run, and the one flag that ends it.
#[derive(Default)]
pub struct TickTimer {
    stop: Arc<AtomicBool>,
    /// Wakes the sleeping thread for the close and for a changed delay.
    wake: Mutex<Option<Sender<()>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl TickTimer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the timer. A second start, like a start after the stop, runs
    /// nothing: one safety net per run.
    pub fn start(&self, delay: DelaySource, tick: OnTick) {
        let mut slot = lock(&self.thread);
        if slot.is_some() || self.is_stopped() {
            return;
        }
        let (wake, woken) = channel();
        *lock(&self.wake) = Some(wake);
        let stop = Arc::clone(&self.stop);
        let spawned = std::thread::Builder::new()
            .name("session-timer".to_owned())
            .spawn(move || run(&stop, &woken, &delay, &tick));
        match spawned {
            Ok(thread) => *slot = Some(thread),
            // A run with no safety net still refreshes on its watcher and on a
            // settled turn: a log line, never a panic.
            Err(error) => eprintln!("session timer unavailable: {error}"),
        }
    }

    /// Whether the run has already stopped: no timer starts after that.
    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    /// Ends the timer at its next look, and wakes it if it is asleep through a
    /// whole delay.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(wake) = lock(&self.wake).as_ref() {
            let _ = wake.send(());
        }
    }

    /// Waits at most `budget` for the thread to end, and answers whether it did.
    /// The handle is taken out under the lock and joined with no lock held,
    /// because a join blocks. `true` when the run started no timer at all.
    pub fn join(&self, budget: Duration) -> bool {
        let Some(thread) = lock(&self.thread).take() else {
            return true;
        };
        let (done, wait) = channel();
        let spawned = std::thread::Builder::new()
            .name("session-timer-join".to_owned())
            .spawn(move || {
                let _ = thread.join();
                let _ = done.send(());
            });
        match spawned {
            Err(error) => {
                eprintln!("session timer join unavailable: {error}");
                false
            }
            Ok(_) => wait.recv_timeout(budget).is_ok(),
        }
    }
}

/// The timer itself: wait the flow's delay, ask for a pass, and do it again
/// with whatever delay the flow answers next.
fn run(stop: &AtomicBool, woken: &Receiver<()>, delay: &DelaySource, tick: &OnTick) {
    let Some(mut wait) = (delay)() else {
        return;
    };
    let mut due = Instant::now() + wait;
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        // A slice, the close, or the deadline: whichever comes first.
        let slice = due.saturating_duration_since(Instant::now()).min(IDLE);
        match woken.recv_timeout(slice) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
        let Some(relearned) = (delay)() else {
            return;
        };
        // A lost or restarted watcher changes the cadence: the window restarts,
        // as iced's rebuilt `time::every` did.
        if relearned != wait {
            wait = relearned;
            due = Instant::now() + wait;
        }
        if Instant::now() < due {
            continue;
        }
        tick();
        let Some(next) = (delay)() else {
            return;
        };
        wait = next;
        due = Instant::now() + wait;
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
