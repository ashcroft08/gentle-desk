//! The one writer of the config file.
//!
//! A save is disk I/O (a temporary file and a rename, `app_config::save`), so it
//! never runs on the thread that asked for it: the caller hands the snapshot
//! over and goes on. One thread owns the file for the whole run, so two saves
//! can never overtake each other, and it always writes the LATEST snapshot: a
//! burst of changes costs one write per finished save, never one per change.
//!
//! Every snapshot arrives numbered ([`ConfigWriter::save`]). The number is what
//! keeps the file right when two callers edit the config at once: their edits
//! are ordered by the config lock, but their saves reach this queue in whatever
//! order the threads are scheduled, so an older snapshot that arrives late is
//! dropped instead of undoing the newer one.
//!
//! With no config path there is no thread and nothing is written: the run keeps
//! its config in memory, which is what a run whose app data directory is
//! unknown does for its whole life.
//!
//! A save that fails is not fatal. The run keeps the value it holds in memory
//! and the failure goes to the handler installed at startup (the bridge arms it
//! with the window's own push), which turns it into the view's config notice.
//!
//! The close flow ends the writer with [`ConfigWriter::flush_and_stop`]: taking
//! the queue's sender is what lets the thread drain what is already queued and
//! then finish, and the thread signals right before it returns, so a snapshot
//! queued when the window closes is on the disk before the process goes away.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use gentle_runtime::app_config::{self, AppConfig};

/// What to call when a save fails: one line for the user. Shared, because the
/// handler is installed after the thread has started.
pub type SaveFailure = Arc<dyn Fn(String) + Send + Sync>;

/// One snapshot to write, numbered by the holder's config version: the writer
/// never writes a number it has already passed.
struct Job {
    version: u64,
    config: AppConfig,
}

/// What every clone of a writer shares: the queue of its thread, and the signal
/// that thread sends as it finishes.
#[derive(Default)]
struct Shared {
    /// `None` before the thread exists, and once the close took it: a save after
    /// that is dropped, as it is in a run with no config path.
    queue: Mutex<Option<Sender<Job>>>,
    /// The other end of the thread's last signal. Taken by the close, once.
    finished: Mutex<Option<Receiver<()>>>,
}

/// The one writer of the config file. Cloned where a save is asked for: every
/// clone feeds the same thread.
#[derive(Clone)]
pub struct ConfigWriter {
    shared: Arc<Shared>,
    /// Where a failure goes, shared with the thread.
    failure: Arc<Mutex<Failure>>,
}

/// The failure handler, and the message of a save that failed before one was
/// installed.
#[derive(Default)]
struct Failure {
    handler: Option<SaveFailure>,
    /// A failure that arrived before the handler: kept, so it is never lost.
    pending: Option<String>,
}

impl ConfigWriter {
    /// Starts the writer of `path`. With no path there is no thread: every save
    /// is dropped and the run keeps its config in memory.
    pub fn spawn(path: Option<PathBuf>) -> Self {
        let failure = Arc::new(Mutex::new(Failure::default()));
        let shared = Arc::new(Shared::default());
        let writer = Self { shared, failure };
        let Some(path) = path else {
            return writer;
        };
        let (queue, jobs) = mpsc::channel();
        let (finished, done) = mpsc::channel();
        let slot = Arc::clone(&writer.failure);
        let spawned = std::thread::Builder::new()
            .name("config-save".to_owned())
            .spawn(move || write_loop(jobs, &path, &slot, &finished));
        if let Err(error) = spawned {
            // A run without the writer keeps its config in memory, which is what
            // a run with no config path does: never a panic at startup.
            eprintln!("config writer unavailable: {error}");
            return writer;
        }
        *lock(&writer.shared.queue) = Some(queue);
        *lock(&writer.shared.finished) = Some(done);
        writer
    }

    /// Asks for `config` to be written, as the snapshot the holder numbered
    /// `version`. Never waits for the disk, and never blocks on the writer
    /// either: the queue is unbounded and holds snapshots.
    pub fn save(&self, version: u64, config: &AppConfig) {
        let job = Job {
            version,
            config: config.clone(),
        };
        // The snapshot is copied before the lock, so a 50-workspace config is
        // never cloned while another caller waits for this one.
        let queue = lock(&self.shared.queue).clone();
        // A closed queue means the thread is gone, or the close took it: the
        // config then lives in memory only, as it does with no config path.
        if let Some(queue) = queue {
            let _ = queue.send(job);
        }
    }

    /// Installs what to call when a save fails. A failure that arrived before
    /// this point is delivered at once, so no reason is ever lost.
    pub fn on_failure(&self, handler: SaveFailure) {
        let pending = {
            let mut slot = lock(&self.failure);
            slot.handler = Some(handler);
            slot.pending.take()
        };
        if let Some(message) = pending {
            report(&self.failure, message);
        }
    }

    /// Ends the writer: every queued snapshot is received, the channel closes
    /// behind them, and the thread signals as it finishes. Answers at once for a
    /// run with no config path (there is nothing to wait for) and, bounded by
    /// `budget`, whether the last snapshot is on the disk. A thread that misses
    /// its deadline is a line on stderr: the process still ends, and the failure
    /// of a save that never ran is never reported as success.
    pub fn flush_and_stop(&self, budget: Duration) -> bool {
        // Taking the sender is what ends the thread: what is already queued is
        // still received, and the channel disconnects behind it.
        let queue = lock(&self.shared.queue).take();
        let Some(queue) = queue else {
            return true; // no path, or an earlier close already flushed it
        };
        drop(queue);
        let Some(finished) = lock(&self.shared.finished).take() else {
            return true;
        };
        match finished.recv_timeout(budget) {
            Ok(()) => true,
            // The thread ended without its last signal (it panicked): the file
            // may be older than what the run held, so this is not a success.
            Err(RecvTimeoutError::Disconnected) => {
                eprintln!("close: the config writer stopped without finishing");
                false
            }
            Err(RecvTimeoutError::Timeout) => {
                eprintln!("close: the config writer did not finish within {budget:?}");
                false
            }
        }
    }
}

/// The writer's thread: one save at a time, always of the latest snapshot.
fn write_loop(jobs: Receiver<Job>, path: &Path, failure: &Mutex<Failure>, finished: &Sender<()>) {
    let mut written: u64 = 0;
    while let Ok(mut job) = jobs.recv() {
        // Everything that arrived while the last save was running is one more
        // write of the newest value, never one per change: this thread is the
        // only one that writes the file.
        while let Ok(newer) = jobs.try_recv() {
            if newer.version > job.version {
                job = newer;
            }
        }
        // A snapshot older than what the file already holds would undo a change
        // that landed after it, so it is dropped whatever order it arrived in.
        if job.version <= written {
            continue;
        }
        written = job.version;
        if let Err(error) = app_config::save(path, &job.config) {
            let message = message(&error);
            // stderr as well as the installed handler: a save that fails during
            // the close has no window left to draw a notice in, and a lost save
            // must never be silent.
            eprintln!("{message}");
            report(failure, message);
        }
    }
    // Every queued snapshot was received and the channel closed behind them:
    // whoever waits for the close now knows the file is as new as it can be.
    let _ = finished.send(());
}

/// Hands one failure to the installed handler, or keeps it for one.
fn report(slot: &Mutex<Failure>, message: String) {
    let handler = {
        let mut guard = lock(slot);
        match guard.handler.clone() {
            Some(handler) => Some(handler),
            None => {
                guard.pending = Some(message.clone());
                None
            }
        }
    };
    // The handler runs with the lock released: it folds state and pushes a
    // slice, and neither of those belongs under this mutex.
    if let Some(handler) = handler {
        handler(message);
    }
}

/// The one line a failed save shows the user.
fn message(error: &io::Error) -> String {
    format!("The app config file could not be saved: {error}")
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
