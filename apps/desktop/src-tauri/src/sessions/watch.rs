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

/// The step a unit test runs in the middle of a registration, between the stop
/// check and the handle it registers: the window the stop-race test drives the
/// close into. Nothing in the app sets one.
#[cfg(test)]
type Window = Arc<dyn Fn() + Send + Sync>;

/// What the registration and the stop both take: whether the run stopped, and
/// the handles of the threads it accepted. One lock answers "may a thread
/// start" and "which threads exist", so the two cannot interleave: a thread
/// accepted before the stop is registered before the stop returns and the next
/// [`Watch::join`] reaches it, and one that asks after the stop starts no thread
/// at all.
#[derive(Default)]
struct State {
    stopped: bool,
    threads: Vec<JoinHandle<()>>,
}

/// The batch threads of this run, and the one flag that ends them.
#[derive(Default)]
pub struct Watch {
    stop: Arc<AtomicBool>,
    /// The threads, kept so the close can wait for them, under the lock that
    /// also holds the stop: [`Watch::join`] takes them out of it, and joins them
    /// with no lock held.
    state: Mutex<State>,
    /// What a unit test holds a registration in its window with; nothing else
    /// reads it. The window runs under `state`'s lock, like the rest of the
    /// registration.
    #[cfg(test)]
    window: Mutex<Option<Window>>,
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
        // Accept-or-stop is one decision under one lock: a registration that
        // takes it before the stop registers its handle before the stop returns,
        // and one that takes it after the stop spawns nothing. A watch started
        // after the close would outlive it, so the stop wins wherever the two
        // meet; no thread can be accepted into a join nobody runs.
        let mut state = lock(&self.state);
        if state.stopped {
            return Ok(());
        }
        let stop = Arc::clone(&self.stop);
        let spawned = std::thread::Builder::new()
            .name("session-batches".to_owned())
            .spawn(move || apply(batches, &stop, &on_batch, &on_lost));
        #[cfg(test)]
        {
            let window = lock(&self.window).clone();
            if let Some(window) = window {
                window();
            }
        }
        match spawned {
            Ok(thread) => {
                state.threads.push(thread);
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
    /// close decided this, the watcher did not fail. Takes the registration's
    /// lock, so a registration under way lands its handle before this returns
    /// and a later one starts nothing: the next [`Watch::join`] reaches every
    /// thread that was ever spawned.
    pub fn stop(&self) {
        let mut state = lock(&self.state);
        state.stopped = true;
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Waits at most `budget` for the batch threads to end, and answers whether
    /// they did. The handles are taken out under the lock and joined with no
    /// lock held, because a join blocks and nothing may hold this lock while it
    /// does. `true` when the run started no batch thread at all.
    pub fn join(&self, budget: Duration) -> bool {
        let threads: Vec<JoinHandle<()>> = {
            let mut state = lock(&self.state);
            std::mem::take(&mut state.threads)
        };
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

#[cfg(test)]
impl Watch {
    /// Makes every registration pause in its window until `window` returns.
    fn hold_registration(&self, window: Window) {
        *lock(&self.window) = Some(window);
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
            Ok(batch) => {
                // The close may have begun while this thread waited for that
                // very batch: a pass started now is one the close does not wait
                // for, and it would read the disk while the cache is flushed.
                if stop.load(Ordering::SeqCst) {
                    return;
                }
                on_batch(batch)
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc::channel as oneshot;

    /// How long a test waits for a step its own threads must reach before it
    /// calls it a miss.
    const LIMIT: Duration = Duration::from_secs(5);

    /// The join budget of the close under test: the batch thread is parked on
    /// purpose, so this only bounds the test, not the wait.
    const PROBE: Duration = Duration::from_millis(250);

    /// How long a close is given to answer while a registration still sits in
    /// its window. A close that waits for that registration never answers in
    /// it: the registration is the test's to release, so this is the one step
    /// the test bounds with a time rather than with a message.
    const EARLY: Duration = Duration::from_millis(500);

    fn batch() -> Batch {
        Batch {
            paths: Vec::new(),
            rescan: true,
        }
    }

    /// A batch thread accepted just before the stop is one the join waits for:
    /// the join never reports the batch threads ended while one of them is
    /// applying a batch.
    ///
    /// The test drives the seam S1 names: the registration is held between its
    /// stop check and the push of its handle, a batch thread of its is already
    /// applying a batch, and the stop and the join run in that window. Every
    /// hand-over is a message; the one step bounded by a time is the close that
    /// must NOT answer in that window (a close that waits for the registration
    /// cannot answer there, so nothing waits it out in the passing run).
    #[test]
    fn a_registration_in_the_stop_window_is_joined() {
        let watch = Arc::new(Watch::new());
        let (in_window, window_entered) = oneshot();
        let (window_release, hold_window) = oneshot();
        let hold_window = Mutex::new(hold_window);
        watch.hold_registration(Arc::new(move || {
            let _ = in_window.send(());
            let _ = hold_window.lock().expect("release").recv_timeout(LIMIT);
        }));

        let (batches, of_batch) = oneshot();
        let (in_batch, batch_entered) = oneshot();
        let (batch_release, hold_batch) = oneshot();
        let hold_batch = Mutex::new(hold_batch);
        let on_batch: OnBatch = Arc::new(move |_batch| {
            let _ = in_batch.send(());
            let _ = hold_batch.lock().expect("release").recv_timeout(LIMIT);
        });
        let on_lost: OnLost = Arc::new(|| {});

        let (registered, adopt_result) = oneshot();
        let registering = {
            let watch = Arc::clone(&watch);
            std::thread::spawn(move || {
                let _ = registered.send(watch.adopt(of_batch, on_batch, on_lost));
            })
        };
        window_entered
            .recv_timeout(LIMIT)
            .expect("the registration reached its window");

        // The thread this registration already spawned takes a batch and stays
        // in it: alive and applying a batch for the whole close below.
        let _ = batches.send(batch());
        batch_entered
            .recv_timeout(LIMIT)
            .expect("the batch thread applied a batch");

        let (closed, join_result) = oneshot();
        let closing = {
            let watch = Arc::clone(&watch);
            std::thread::spawn(move || {
                watch.stop();
                let _ = closed.send(watch.join(PROBE));
            })
        };
        // The stop must not answer while a registration sits in its window: the
        // thread that registration spawned is the one the join has to reach, and
        // a stop that answers now cannot reach a handle that is not registered
        // yet.
        let early = join_result.recv_timeout(EARLY);
        let answered_in_window = early.is_ok();
        // Let the registration land its handle, then hear the close out (the one
        // that answered above has nothing left to say).
        let _ = window_release.send(());
        let joined = match early {
            Ok(joined) => joined,
            Err(_) => join_result.recv_timeout(LIMIT).expect("the close answered"),
        };

        let _ = batch_release.send(());
        registering.join().expect("the registration never panicked");
        closing.join().expect("the close never panicked");
        assert!(
            adopt_result
                .recv_timeout(LIMIT)
                .expect("the registration answered")
                .is_ok(),
            "the registration of the window failed"
        );
        assert!(
            !answered_in_window,
            "the stop answered while a registration was in its window: that thread escapes the join"
        );
        // The batch thread was parked in `on_batch` for the whole close, so a
        // join that answers `true` here answered for a thread that is alive.
        assert!(
            !joined,
            "the join reported every batch thread ended while one was still applying a batch"
        );
    }

    /// A registration that starts after the stop spawns no thread at all: no
    /// batch is applied and the join has nothing to wait for.
    #[test]
    fn a_registration_after_the_stop_spawns_nothing() {
        let watch = Watch::new();
        watch.stop();
        let (batches, of_batch) = oneshot();
        let applied = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&applied);
        let on_batch: OnBatch = Arc::new(move |_batch| {
            counted.fetch_add(1, Ordering::SeqCst);
        });
        watch
            .adopt(of_batch, on_batch, Arc::new(|| {}))
            .expect("a registration after the stop is not an error");

        // Nothing keeps the receiver, so nothing is listening: had a thread
        // started, this batch would have gone to it.
        assert!(
            batches.send(batch()).is_err(),
            "a batch thread started after the stop"
        );
        assert_eq!(applied.load(Ordering::SeqCst), 0, "a batch was applied");
        assert!(
            watch.join(PROBE),
            "a join after the stop waited for nothing"
        );
    }
}
