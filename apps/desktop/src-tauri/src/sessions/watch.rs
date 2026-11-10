//! The session watcher of the desktop run: the thread that applies its batches,
//! and how that thread stops.
//!
//! The ROLE is the iced shell's (`crates/ui/src/watch_source.rs` and
//! `refresh_flow::start_watcher`), never its code: there every batch of
//! `gentle_backend::session_watch` became `Message::SessionsChanged` and the end
//! of the stream became `Message::WatcherLost`; here a batch goes straight to
//! the flow and the end of the stream is the lost fallback. Draining one blocks,
//! so it runs on a thread of its own - never on the window's, never on a pass's.
//!
//! [`StartWatch`] is the seam: `gentle_runtime::startup::try_start_watcher` in
//! the app, a channel a test fills in place of `notify`.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use gentle_backend::session_watch::{Batch, SessionWatcher, WatchError};

/// Starts watching a sessions directory: the batches to apply, and the handle
/// whose drop stops the platform watch and its own thread.
pub type StartWatch =
    Arc<dyn Fn(&Path) -> Result<(SessionWatcher, Receiver<Batch>), WatchError> + Send + Sync>;

/// What the run does with one batch of changed session files.
pub type OnBatch = Arc<dyn Fn(Batch) + Send + Sync>;

/// What the run does when a watch ended on its own: poll until one starts again.
pub type OnLost = Arc<dyn Fn() + Send + Sync>;

/// How long a batch thread waits for the next batch before it looks at the stop
/// signal. The channel stays open for the whole life of a watch, so a blocking
/// receive could never be woken for the close; a tenth of a second costs nothing
/// and bounds the close's wait (the child event pump's `IDLE` for the same
/// reason).
const IDLE: Duration = Duration::from_millis(100);

/// The batch threads of this run, and the one flag that ends them.
#[derive(Default)]
pub struct Watch {
    stop: Arc<AtomicBool>,
    /// The threads, kept so the close can wait for them. They are taken out
    /// (under this lock) by [`Watch::join`], which joins them with no lock held.
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl Watch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts `dir` through the `start` seam and the thread that applies its
    /// batches. Keep the watcher it answers: dropping it stops the platform
    /// watch and the thread behind it.
    pub fn start(
        &self,
        dir: &Path,
        start: &StartWatch,
        on_batch: OnBatch,
        on_lost: OnLost,
    ) -> Result<SessionWatcher, WatchError> {
        let (watcher, batches) = start(dir)?;
        self.adopt(batches, on_batch, on_lost)?;
        Ok(watcher)
    }

    /// Applies `batches` on a thread of its own until the watch ends or the run
    /// stops. Called by [`Watch::start`], and on its own by a test.
    pub fn adopt(
        &self,
        batches: Receiver<Batch>,
        on_batch: OnBatch,
        on_lost: OnLost,
    ) -> Result<(), WatchError> {
        // A watch started after the close would outlive it: refuse instead of
        // leaving a thread nobody waits for.
        if self.is_stopped() {
            return Ok(());
        }
        let stop = Arc::clone(&self.stop);
        let spawned = std::thread::Builder::new()
            .name("session-batches".to_owned())
            .spawn(move || apply(batches, &stop, &on_batch, &on_lost));
        match spawned {
            Ok(thread) => {
                lock(&self.threads).push(thread);
                Ok(())
            }
            Err(error) => Err(WatchError::Thread(error.to_string())),
        }
    }

    /// Whether the run has already stopped: no watch starts after that.
    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    /// Ends every batch thread at its next look, and reports no watch lost: the
    /// close decided this, the watcher did not fail.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Waits at most `budget` for the batch threads to end, and answers whether
    /// they did. The handles are taken out under the lock and joined with no
    /// lock held, because a join blocks and nothing may hold this lock while it
    /// does. `true` when the run started no batch thread at all.
    pub fn join(&self, budget: Duration) -> bool {
        let threads: Vec<JoinHandle<()>> = std::mem::take(&mut *lock(&self.threads));
        if threads.is_empty() {
            return true;
        }
        let (done, wait) = channel();
        let spawned = std::thread::Builder::new()
            .name("session-batches-join".to_owned())
            .spawn(move || {
                for thread in threads {
                    let _ = thread.join();
                }
                let _ = done.send(());
            });
        match spawned {
            // A join thread that cannot start is a miss, exactly like a timeout:
            // the caller's deadline decides, and the close goes on.
            Err(error) => {
                eprintln!("session batch join unavailable: {error}");
                false
            }
            Ok(_) => wait.recv_timeout(budget).is_ok(),
        }
    }
}

/// One batch thread: a batch at a time, then the lost fallback when the watch
/// ended without the run asking it to.
fn apply(batches: Receiver<Batch>, stop: &AtomicBool, on_batch: &OnBatch, on_lost: &OnLost) {
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        match batches.recv_timeout(IDLE) {
            Ok(batch) => on_batch(batch),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    // The watcher stopped: the list polls until one starts again.
    if !stop.load(Ordering::SeqCst) {
        on_lost();
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
