//! The full pass over the sessions directory, with and without progress.
//!
//! [`SessionIndex::refresh`] is the plain spellings callers use; it is
//! [`SessionIndex::refresh_with_progress`] with a callback that ignores
//! everything, so both answer with the same stats, the same entries and the
//! same cost. Keeping the pair in one file makes that one implementation, not
//! two that must agree.
//!
//! The callback gets the index itself (`&SessionIndex`), so a caller can read
//! [`entries`](SessionIndex::entries) as files are folded, without having to
//! buffer them here.

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

use super::disk::session_files;
use super::{RefreshStats, SessionIndex};

impl SessionIndex {
    /// Brings the cache in line with the disk. A missing sessions directory is
    /// an empty list, not an error; an unreadable single file is skipped.
    pub fn refresh(&mut self) -> io::Result<RefreshStats> {
        self.refresh_with_progress(|_, _, _| {})
    }

    /// [`refresh`](Self::refresh) reporting how much of the pass is done:
    /// `on_progress(index, done, total)` is called after every session file,
    /// with the files folded so far and the number of files found, and once
    /// more when the pass ends, so a caller that throttles still sees it
    /// complete. A home with no session files therefore reports `(0, 0)`, and
    /// a pass that fails before listing the files reports nothing.
    ///
    /// Every rule is [`refresh`](Self::refresh)'s: the removals happen before
    /// the first call and each call sees the entries folded so far.
    pub fn refresh_with_progress(
        &mut self,
        mut on_progress: impl FnMut(&SessionIndex, usize, usize),
    ) -> io::Result<RefreshStats> {
        let mut stats = RefreshStats::default();
        let found = session_files(&self.sessions_dir)?;
        stats.files = found.len();
        let alive: HashSet<&PathBuf> = found.iter().collect();
        let before = self.cache.len();
        self.cache.retain(|path, _| alive.contains(path));
        stats.removed = before - self.cache.len();
        let total = found.len();
        for (folded, path) in found.iter().enumerate() {
            self.update_file(path).count(&mut stats);
            on_progress(self, folded + 1, total);
        }
        on_progress(self, total, total);
        Ok(stats)
    }
}
