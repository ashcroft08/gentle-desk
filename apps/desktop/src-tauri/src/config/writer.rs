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

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

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

/// The one writer of the config file. Cloned where a save is asked for: every
/// clone feeds the same thread.
#[derive(Clone)]
pub struct ConfigWriter {
    /// The queue of the writer's thread, `None` for a run with no config file.
    queue: Option<Sender<Job>>,
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
        let Some(path) = path else {
            return Self {
                queue: None,
                failure,
            };
        };
        let (queue, jobs) = mpsc::channel();
        let slot = Arc::clone(&failure);
        let spawned = std::thread::Builder::new()
            .name("config-save".to_owned())
            .spawn(move || write_loop(jobs, &path, &slot));
        if let Err(error) = spawned {
            // A run without the writer keeps its config in memory, which is what
            // a run with no config path does: never a panic at startup.
            eprintln!("config writer unavailable: {error}");
            return Self {
                queue: None,
                failure,
            };
        }
        Self {
            queue: Some(queue),
            failure,
        }
    }

    /// Asks for `config` to be written, as the snapshot the holder numbered
    /// `version`. Never waits for the disk, and never blocks on the writer
    /// either: the queue is unbounded and holds snapshots.
    pub fn save(&self, version: u64, config: &AppConfig) {
        if let Some(queue) = self.queue.as_ref() {
            // A closed queue means the thread is gone: the config then lives in
            // memory only, as it does in a run with no config path.
            let _ = queue.send(Job {
                version,
                config: config.clone(),
            });
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
}

/// The writer's thread: one save at a time, always of the latest snapshot.
fn write_loop(jobs: Receiver<Job>, path: &Path, failure: &Mutex<Failure>) {
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
            report(failure, message(&error));
        }
    }
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
