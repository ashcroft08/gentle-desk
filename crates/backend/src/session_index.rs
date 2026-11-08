//! Incremental index of pi's saved sessions.
//!
//! Sessions live in `<home>/sessions/--<encoded-cwd>--/*.jsonl` and are
//! append-only. The reference app listed them by parsing every file on every
//! call. This index keeps one [`SessionScanner`] per file, keyed by path with
//! the size and mtime it was built from, so [`SessionIndex::refresh`] costs one
//! `stat` per unchanged file, reads only the appended bytes of a grown file,
//! rescans a rewritten one and forgets deleted ones.
//!
//! [`SessionIndex::snapshot`] turns that cache into data and
//! [`SessionIndex::restore`] brings it back, so a later run can start warm. The
//! restored state is never trusted on its own: [`SessionIndex::refresh`] still
//! reuses an entry only when its size and mtime match the file exactly.
//!
//! [`SessionIndex::refresh_paths`] applies the same per-file rules to the paths
//! a file watcher reports, so keeping up with a busy home costs one `stat` per
//! reported path instead of a pass over every session.
//!
//! [`SessionIndex::refresh_with_progress`] is that same full pass reporting how
//! many files it has folded, so a caller can show what the cache already knows
//! while a big home is still being read.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gentle_core::session::SessionSummary;

use crate::session_snapshot::{join_mtime, split_mtime, FileSnapshot, IndexSnapshot};

mod disk;
mod paths;
mod progress;
mod store;

use paths::session_key;
use store::{Cached, Update};

/// One row of the session list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEntry {
    pub path: PathBuf,
    pub summary: SessionSummary,
    /// Latest activity, else creation time, else the file's mtime (as pi does).
    pub modified_ms: u64,
}

/// What a [`SessionIndex::refresh`] did, so callers (and tests) can see its cost.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefreshStats {
    /// Session files found on disk: for [`SessionIndex::refresh_paths`], the
    /// valid session paths it was handed.
    pub files: usize,
    /// Bytes read from disk during this refresh.
    pub bytes_read: u64,
    /// Files seen for the first time or rewritten, parsed from the start.
    pub rescanned: usize,
    /// Files that only grew: parsed from the previous end.
    pub appended: usize,
    /// Cached files that no longer exist.
    pub removed: usize,
}

pub struct SessionIndex {
    sessions_dir: PathBuf,
    cache: HashMap<PathBuf, Cached>,
}

impl SessionIndex {
    /// `sessions_dir` is `<home>/sessions`.
    pub fn new(sessions_dir: impl Into<PathBuf>) -> Self {
        Self {
            sessions_dir: sessions_dir.into(),
            cache: HashMap::new(),
        }
    }

    /// Brings the cache in line with just `paths`, as a file watcher reports
    /// them, without enumerating the sessions directory: one `stat` per
    /// reported path instead of a pass over every session in the home.
    ///
    /// The rules per file are exactly the ones [`refresh`](Self::refresh)
    /// applies, and both share one implementation: an unchanged file is not
    /// read, a grown one only from its previous end, a rewritten or shrunk one
    /// from the start, a new one from the start, and a file that is gone is
    /// dropped from the cache (`removed`). An unreadable file is dropped too,
    /// so the next pass retries it.
    ///
    /// A path that is not a session file location is ignored: not counted, not
    /// logged, never an error. Only `<sessions_dir>/<project>/<name>.jsonl`
    /// counts, and the comparison accepts the spellings a watcher reports on
    /// Windows (verbatim `\\?\` prefix, different case, `/` separators).
    /// Duplicate paths are processed once.
    ///
    /// [`RefreshStats::files`] is the number of valid session paths considered
    /// here, after ignoring and de-duplicating, not the number of sessions the
    /// index holds. It cannot fail: nothing is enumerated.
    pub fn refresh_paths(&mut self, paths: &[PathBuf]) -> RefreshStats {
        let mut stats = RefreshStats::default();
        let mut seen = HashSet::new();
        for path in paths {
            let Some((key, folded)) = session_key(&self.sessions_dir, path) else {
                continue;
            };
            if !seen.insert(folded) {
                continue;
            }
            // A directory is not a session file, whatever it is called.
            if fs::metadata(&key).is_ok_and(|meta| meta.is_dir()) {
                continue;
            }
            stats.files += 1;
            let cached = self.cache.contains_key(&key);
            match self.update_file(&key) {
                Update::Missing => {
                    self.cache.remove(&key);
                    if cached {
                        stats.removed += 1;
                    }
                }
                update => update.count(&mut stats),
            }
        }
        stats
    }

    /// Folds one file into the cache: the one implementation `refresh` and
    /// `refresh_paths` share, so both answer with the same rules.
    fn update_file(&mut self, path: &Path) -> Update {
        let mut entry = self.cache.remove(path);
        let update = store::update(&mut entry, path);
        if let Some(cached) = entry {
            self.cache.insert(path.to_path_buf(), cached);
        }
        update
    }

    /// The cache as data, so a later run can skip the cold read.
    pub fn snapshot(&self) -> IndexSnapshot {
        let mut files: Vec<FileSnapshot> = self
            .cache
            .iter()
            .map(|(path, cached)| {
                let (mtime_secs, mtime_nanos) = split_mtime(cached.mtime);
                FileSnapshot {
                    path: path.clone(),
                    size: cached.size,
                    mtime_secs,
                    mtime_nanos,
                    parsed_len: cached.parsed_len,
                    scanner: cached.scanner.clone(),
                }
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        IndexSnapshot {
            sessions_dir: self.sessions_dir.clone(),
            files,
        }
    }

    /// Replaces the cache with `snapshot`, unless it was built for another
    /// sessions directory. The next [`refresh`](Self::refresh) then behaves as
    /// after an in-memory warm-up: unchanged files are not read at all.
    ///
    /// Every entry still has to earn its place: [`refresh`](Self::refresh)
    /// compares its size and mtime with the file and rescans on any difference,
    /// so a stale, other-home or corrupt cache only costs time, never accuracy.
    pub fn restore(&mut self, snapshot: IndexSnapshot) {
        if snapshot.sessions_dir != self.sessions_dir {
            return;
        }
        self.cache = snapshot
            .files
            .into_iter()
            // A parsed length past the end of the file cannot describe a cache
            // that is in sync, so the file gets a full scan instead.
            .filter(|file| file.parsed_len <= file.size)
            .map(|file| {
                (
                    file.path,
                    Cached {
                        size: file.size,
                        mtime: join_mtime(file.mtime_secs, file.mtime_nanos),
                        parsed_len: file.parsed_len,
                        scanner: file.scanner,
                    },
                )
            })
            .collect();
    }

    /// The valid sessions, most recently active first.
    pub fn entries(&self) -> Vec<SessionEntry> {
        let mut entries: Vec<SessionEntry> = self
            .cache
            .iter()
            .filter_map(|(path, cached)| {
                let summary = cached.scanner.summary()?;
                let modified_ms = summary
                    .last_activity_ms
                    .or(summary.created_ms)
                    .unwrap_or_else(|| millis(cached.mtime));
                Some(SessionEntry {
                    path: path.clone(),
                    summary,
                    modified_ms,
                })
            })
            .collect();
        entries.sort_by(|a, b| {
            b.modified_ms
                .cmp(&a.modified_ms)
                .then_with(|| a.path.cmp(&b.path))
        });
        entries
    }
}

fn millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

#[cfg(test)]
mod progress_tests;

#[cfg(test)]
mod refresh_paths_scope_tests;

#[cfg(test)]
mod refresh_paths_tests;

#[cfg(test)]
mod test_home;

#[cfg(test)]
mod tests;
