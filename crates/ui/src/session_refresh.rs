//! Reading the session index off the UI thread.
//!
//! The refresh is a `stat` per unchanged file but a full parse of new or
//! rewritten ones, so on a big home it must never run inside the window's
//! event loop. The index is shared behind a mutex; the work runs on its own
//! thread, builds the prepared list there too (the folding is O(N) and pure),
//! and the caller only awaits the finished list.
//!
//! Two kinds of pass exist: a full one that enumerates the sessions directory,
//! and a targeted one that only looks at the paths a file watcher reported
//! (`RefreshQueue` decides which runs when). The first full pass of a run also
//! reports the list as it goes ([`progress`]), because on a big home it takes
//! seconds and the sidebar would otherwise show nothing.
//!
//! The same thread persists the index afterwards, but never per change: the
//! cache is one document holding every session (`SaveThrottle`), so it is
//! written at most every [`save_throttle::MIN_INTERVAL`] and on close.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gentle_app::PreparedSessions;
#[cfg(test)]
use gentle_app::SessionItem;
use gentle_backend::session_cache;
use gentle_backend::session_index::{RefreshStats, SessionIndex};

use crate::save_throttle::SaveThrottle;

mod progress;
mod worker;

pub use progress::{Progress, Update};
pub use worker::stream;

#[cfg(test)]
pub(crate) use worker::run;

#[cfg(test)]
pub(crate) use worker::work;

#[cfg(test)]
mod stream_tests;

#[cfg(test)]
mod test_home;

pub type SharedIndex = Arc<Mutex<SessionIndex>>;

/// What the worker threads need: the index, where its cache goes, how often
/// that cache may be rewritten, and whether the first full pass already ran.
/// Cheap to clone, so every pass gets its own copy and the app keeps one for
/// [`flush`].
#[derive(Clone)]
pub struct Shared {
    pub index: SharedIndex,
    /// Where the index cache is read and written; `None` means nothing persists.
    pub cache_path: Option<PathBuf>,
    throttle: Arc<Mutex<SaveThrottle>>,
    /// The time base of the throttle: the clock inside never leaves this struct.
    started: Instant,
    /// Whether the run's first full pass still has to report its progress.
    first_pass: Arc<AtomicBool>,
}

impl Shared {
    /// A run that starts throttling now.
    pub fn new(index: SharedIndex, cache_path: Option<PathBuf>) -> Self {
        Self {
            index,
            cache_path,
            throttle: Arc::new(Mutex::new(SaveThrottle::default())),
            started: Instant::now(),
            first_pass: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Time since this run started, as the throttle's time base.
    fn now(&self) -> Duration {
        self.started.elapsed()
    }

    /// Whether the run's first full pass still has to report its progress.
    fn is_first_pass(&self) -> bool {
        self.first_pass.load(Ordering::Relaxed)
    }

    /// A full pass ran: later passes stay silent, since the list they would
    /// report is already complete.
    fn first_pass_done(&self) {
        self.first_pass.store(false, Ordering::Relaxed);
    }

    /// Marks the cache dirty when the pass changed anything and writes it when
    /// the throttle allows. A failed write is only logged: the next run starts
    /// cold, which costs time, never accuracy.
    fn save_if_due(&self, stats: &RefreshStats) {
        let Some(path) = self.cache_path.as_deref() else {
            return;
        };
        let Ok(mut throttle) = self.throttle.lock() else {
            return;
        };
        // An unchanged pass has nothing new to persist.
        if stats.rescanned + stats.appended + stats.removed > 0 {
            throttle.mark_dirty();
        }
        let now = self.now();
        if !throttle.should_save(now) {
            return;
        }
        let snapshot = match self.index.lock() {
            Ok(index) => index.snapshot(),
            Err(_) => return,
        };
        match session_cache::save(path, &snapshot) {
            Ok(()) => throttle.saved(now),
            Err(error) => eprintln!("cannot write the session index cache: {error}"),
        }
    }
}

/// A finished pass, ready to hand to `gentle-app`.
#[derive(Debug, Clone)]
pub struct Refreshed {
    /// The whole list, with its search data already built on the worker thread.
    pub sessions: PreparedSessions,
    /// How long the index pass itself took (the preparation is not counted).
    pub took: Duration,
    /// Whether this was a full pass (`true`) or a targeted one.
    pub full: bool,
    /// What the pass read and parsed: the tests assert a targeted pass touches
    /// only its paths, and it tells a slow home from a fast one.
    #[allow(dead_code)]
    pub stats: RefreshStats,
}

/// Writes the cache now if it holds something unsaved, for use on close.
pub fn flush(shared: &Shared) {
    let Some(path) = shared.cache_path.as_deref() else {
        return;
    };
    let Ok(mut throttle) = shared.throttle.lock() else {
        return;
    };
    if !throttle.is_dirty() {
        return;
    }
    let snapshot = match shared.index.lock() {
        Ok(index) => index.snapshot(),
        Err(_) => return,
    };
    let now = shared.now();
    match session_cache::save(path, &snapshot) {
        Ok(()) => throttle.saved(now),
        Err(error) => eprintln!("cannot write the session index cache: {error}"),
    }
}

/// Refreshes the index on a worker thread and returns the list to show.
///
/// A full pass with a fresh throttle, so the list is never held back by how
/// often this run persisted the cache. The shipped path is [`stream`] through
/// `refresh_flow`; this wrapper exists for the module's own tests.
#[cfg(test)]
pub async fn refresh(
    index: SharedIndex,
    cache_path: Option<PathBuf>,
) -> Result<Vec<SessionItem>, String> {
    let refreshed = run(
        crate::refresh_queue::Job::Full,
        Shared::new(index, cache_path),
    )
    .await?;
    Ok(refreshed.sessions.items().to_vec())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod worker_tests;
