//! Watches the sessions directory and reports debounced batches of changes.
//!
//! A file watcher ([`notify`]) reports every create, write, rename and delete
//! under `<home>/sessions`, twice per created file and hundreds of times in a
//! burst. This layer turns that stream into [`Batch`]es a refresh can apply:
//! [`debounce`] coalesces the raw pushes into one batch with de-duplicated
//! paths, and [`classify`] decides which reported paths can matter at all.
//!
//! Only this module touches the file system and a thread: the rules it applies
//! are pure and tested on their own (`debounce/tests.rs`, `classify/tests.rs`).
//! Nothing here reads the index; the caller decides what a batch means.
//!
//! Known limit (measured on Windows/NTFS): a write made through a handle that
//! stays open and is never flushed or closed is not reported for at least 15 s;
//! once the writer closes or syncs, the event arrives in about 265 ms. pi appends
//! with `appendFileSync` (open, write, close), so it is reported promptly. Any
//! other writer that keeps the file open is only noticed by the slow
//! reconciliation pass.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub mod classify;
pub mod debounce;
mod paths;

pub use debounce::{Batch, Debouncer};

/// Why a [`SessionWatcher`] could not start.
///
/// Every case is one the caller has to survive: no watcher means the list falls
/// back to polling, so the message is for the log, not a dead end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchError {
    /// The sessions directory does not exist (yet).
    Missing(PathBuf),
    /// The sessions path exists but is not a directory.
    NotDirectory(PathBuf),
    /// The platform watcher could not be created.
    Create(String),
    /// The watcher could not watch the sessions directory.
    Register {
        /// The directory the watch was registered on.
        dir: PathBuf,
        /// The underlying error, as the platform backend reported it.
        message: String,
    },
    /// The thread that debounces events could not be started.
    Thread(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(dir) => write!(
                f,
                "the sessions directory does not exist: {}",
                dir.display()
            ),
            Self::NotDirectory(dir) => {
                write!(f, "the sessions path is not a directory: {}", dir.display())
            }
            Self::Create(message) => write!(f, "cannot create a file watcher: {message}"),
            Self::Register { dir, message } => {
                write!(f, "cannot watch {}: {message}", dir.display())
            }
            Self::Thread(message) => {
                write!(f, "cannot start the session watcher thread: {message}")
            }
        }
    }
}

impl std::error::Error for WatchError {}

/// A running watcher: drop it to stop watching and stop its thread.
pub struct SessionWatcher {
    watcher: Option<RecommendedWatcher>,
    thread: Option<JoinHandle<()>>,
}

impl SessionWatcher {
    /// Starts watching `sessions_dir` recursively and returns the batches to
    /// apply, in the order they become due.
    ///
    /// The directory has to exist: a missing or non-directory path is an error,
    /// which the caller answers by keeping the polling fallback and retrying
    /// later: the list is never wrong, only slower to update. After that a
    /// background thread does the work and the caller only reads the receiver.
    pub fn start(sessions_dir: &Path) -> Result<(Self, Receiver<Batch>), WatchError> {
        match fs::metadata(sessions_dir) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(WatchError::NotDirectory(sessions_dir.to_path_buf())),
            Err(_) => return Err(WatchError::Missing(sessions_dir.to_path_buf())),
        }
        let (raw_tx, raw_rx) = mpsc::channel::<notify::Result<notify::Event>>();
        // The platform callback only forwards: classifying and debouncing happen
        // on the thread below, so the callback stays as short as possible.
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = raw_tx.send(event);
        })
        .map_err(|error| WatchError::Create(error.to_string()))?;
        watcher
            .watch(sessions_dir, RecursiveMode::Recursive)
            .map_err(|error| WatchError::Register {
                dir: sessions_dir.to_path_buf(),
                message: error.to_string(),
            })?;

        let (batch_tx, batch_rx) = mpsc::channel();
        let dir = sessions_dir.to_path_buf();
        let thread = std::thread::Builder::new()
            .name("session-watch".to_owned())
            .spawn(move || debounce_events(&dir, &raw_rx, &batch_tx))
            .map_err(|error| WatchError::Thread(error.to_string()))?;
        Ok((
            Self {
                watcher: Some(watcher),
                thread: Some(thread),
            },
            batch_rx,
        ))
    }
}

/// How long the thread waits when no batch is open. An event or the closing
/// channel wakes it earlier, so this only bounds an idle sleep.
const IDLE_WAIT: Duration = Duration::from_secs(60);

/// The one thread that owns the [`Debouncer`]: it turns raw events into the
/// batches the caller refreshes from. A `notify` error counts as a rescan: the
/// events it would have reported are gone.
fn debounce_events(
    dir: &Path,
    raw: &Receiver<notify::Result<notify::Event>>,
    batches: &Sender<Batch>,
) {
    let started = Instant::now();
    let mut debouncer = Debouncer::default();
    loop {
        let wait = debouncer.deadline().map_or(IDLE_WAIT, |deadline| {
            deadline.saturating_sub(started.elapsed())
        });
        match raw.recv_timeout(wait) {
            Ok(Ok(event)) => {
                let classified = classify::classify(dir, &event);
                debouncer.push(started.elapsed(), classified.paths, classified.rescan);
            }
            // A notify error means events may have been lost: reconcile.
            Ok(Err(_)) => debouncer.push(started.elapsed(), Vec::new(), true),
            Err(RecvTimeoutError::Timeout) => {}
            // The watcher is gone: nothing more will arrive.
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let Some(batch) = debouncer.take_if_due(started.elapsed()) else {
            continue;
        };
        if batches.send(batch).is_err() {
            // The consumer dropped its receiver: stop instead of piling up.
            return;
        }
    }
}

impl Drop for SessionWatcher {
    fn drop(&mut self) {
        // Dropping the platform watcher drops the callback, which closes the raw
        // channel; the thread then sees it disconnected and returns.
        drop(self.watcher.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl std::fmt::Debug for SessionWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionWatcher").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
