//! Coalesces the paths a file watcher reports into `Batch`es.
//!
//! A created session file produces at least two events (create, then modify),
//! and a burst produces hundreds; refreshing per event would repeat work. The
//! debouncer waits for a quiet window after the last push and never holds a
//! batch longer than its max wait, so a busy home still updates.
//!
//! Time is passed in as an offset from any fixed start: nothing here reads a
//! clock, so the whole rule is testable without sleeping.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

/// A batch waits this long after the last push before it is due.
pub const QUIET: Duration = Duration::from_millis(250);

/// A burst that never goes quiet is emitted at most this long after its start.
pub const MAX_WAIT: Duration = Duration::from_secs(1);

/// Above this many distinct paths a batch is a full pass instead: a burst that
/// size is cheaper to reconcile by enumerating the sessions once.
pub const MAX_BATCH_PATHS: usize = 2000;

/// The changes one refresh should apply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    /// The session files to look at, de-duplicated and sorted.
    pub paths: Vec<PathBuf>,
    /// Do a full pass: the watcher may have missed events.
    pub rescan: bool,
}

/// Collects pushes until a quiet window passes, or the max wait expires.
#[derive(Debug)]
pub struct Debouncer {
    quiet: Duration,
    max_wait: Duration,
    /// When the open batch started, and when it last received a push.
    first: Option<Duration>,
    last: Option<Duration>,
    paths: BTreeSet<PathBuf>,
    rescan: bool,
    /// More than [`MAX_BATCH_PATHS`] distinct paths arrived: collapse.
    overflow: bool,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(QUIET, MAX_WAIT)
    }
}

impl Debouncer {
    /// `quiet` and `max_wait` are parameters so tests can use small values.
    pub fn new(quiet: Duration, max_wait: Duration) -> Self {
        Self {
            quiet,
            max_wait,
            first: None,
            last: None,
            paths: BTreeSet::new(),
            rescan: false,
            overflow: false,
        }
    }

    /// Adds `paths` (and whether a full pass is needed) seen at `now`.
    pub fn push(&mut self, now: Duration, paths: Vec<PathBuf>, rescan: bool) {
        // Nothing to act on: an empty classified event does not open a batch.
        if paths.is_empty() && !rescan {
            return;
        }
        // A `now` earlier than the previous push only widens the window.
        self.first = Some(self.first.map_or(now, |first| first.min(now)));
        self.last = Some(self.last.map_or(now, |last| last.max(now)));
        self.rescan |= rescan;
        if self.overflow {
            return;
        }
        for path in paths {
            if self.paths.insert(path) && self.paths.len() > MAX_BATCH_PATHS {
                // A burst this large is cheaper to reconcile with a full pass.
                self.overflow = true;
                self.paths.clear();
                break;
            }
        }
    }

    /// The earliest instant the open batch is due, if one is open.
    pub fn deadline(&self) -> Option<Duration> {
        let first = self.first?;
        let last = self.last?;
        let quiet = last.saturating_add(self.quiet);
        let capped = first.saturating_add(self.max_wait);
        Some(quiet.min(capped))
    }

    /// The batch if `now` has reached its deadline; nothing is cleared otherwise.
    pub fn take_if_due(&mut self, now: Duration) -> Option<Batch> {
        if self.deadline()? > now {
            return None;
        }
        self.take()
    }

    /// The open batch now, whatever the time.
    pub fn take(&mut self) -> Option<Batch> {
        self.first.take()?;
        self.last = None;
        let overflow = std::mem::take(&mut self.overflow);
        let rescan = std::mem::take(&mut self.rescan) || overflow;
        let paths = if overflow {
            Vec::new()
        } else {
            std::mem::take(&mut self.paths).into_iter().collect()
        };
        Some(Batch { paths, rescan })
    }
}

#[cfg(test)]
mod tests;
