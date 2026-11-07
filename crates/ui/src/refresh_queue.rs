//! What session refresh runs next: one pass at a time, nothing dropped.
//!
//! A refresh occupies the index until it returns, so whatever arrives while it
//! runs has to be remembered: the paths a watcher reports, or a full pass. A
//! full pass already covers every path, so it supersedes them; above
//! [`MAX_PENDING_PATHS`] distinct paths the queue collapses into a full pass,
//! which is cheaper than applying thousands of paths one by one.
//!
//! Pure: no clock, no channel, no thread. The caller decides when to start the
//! next job and reports back with [`RefreshQueue::finish`] when the worker
//! delivers its list.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Above this many distinct pending paths a full pass is the cheaper pass.
pub const MAX_PENDING_PATHS: usize = 2000;

/// The work one refresh pass does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    /// Enumerate the sessions directory and bring everything in line.
    Full,
    /// Look only at these session files.
    Paths(Vec<PathBuf>),
}

/// The passes waiting for the index, and whether one is running.
#[derive(Debug, Default)]
pub struct RefreshQueue {
    busy: bool,
    full: bool,
    paths: BTreeSet<PathBuf>,
}

impl RefreshQueue {
    /// Remembers `paths` for the next pass. Nothing is lost while a pass runs.
    pub fn request_paths(&mut self, paths: Vec<PathBuf>) {
        // A full pass already covers every path, so collecting them is pointless.
        if self.full {
            return;
        }
        self.paths.extend(paths);
        if self.paths.len() > MAX_PENDING_PATHS {
            self.request_full();
        }
    }

    /// Asks for a full pass, which covers every pending path.
    pub fn request_full(&mut self) {
        self.full = true;
        self.paths.clear();
    }

    /// The next pass to run, or `None` while one runs or nothing is pending.
    pub fn start_next(&mut self) -> Option<Job> {
        if self.busy {
            return None;
        }
        let job = if std::mem::take(&mut self.full) {
            Job::Full
        } else if self.paths.is_empty() {
            return None;
        } else {
            Job::Paths(std::mem::take(&mut self.paths).into_iter().collect())
        };
        self.busy = true;
        Some(job)
    }

    /// The running pass is done; the next request may start.
    pub fn finish(&mut self) {
        self.busy = false;
    }

    /// Whether a pass has the index right now. The flow only needs
    /// `start_next`'s answer; the tests read this to name the state.
    #[allow(dead_code)]
    pub fn is_busy(&self) -> bool {
        self.busy
    }
}

#[cfg(test)]
mod tests;
