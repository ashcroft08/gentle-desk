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

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use gentle_core::session::{SessionScanner, SessionSummary};

use crate::session_snapshot::{join_mtime, split_mtime, FileSnapshot, IndexSnapshot};

mod disk;

use disk::{read_new_lines, session_files};

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
    /// Session files found on disk.
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

struct Cached {
    size: u64,
    mtime: SystemTime,
    /// Bytes already folded into `scanner`: up to the last complete line.
    parsed_len: u64,
    scanner: SessionScanner,
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

    /// Brings the cache in line with the disk. A missing sessions directory is
    /// an empty list, not an error; an unreadable single file is skipped.
    pub fn refresh(&mut self) -> io::Result<RefreshStats> {
        let mut stats = RefreshStats::default();
        let found = session_files(&self.sessions_dir)?;
        stats.files = found.len();
        let alive: HashSet<&PathBuf> = found.iter().collect();
        let before = self.cache.len();
        self.cache.retain(|path, _| alive.contains(path));
        stats.removed = before - self.cache.len();

        for path in &found {
            let Ok(meta) = fs::metadata(path) else {
                continue;
            };
            let (size, mtime) = (meta.len(), meta.modified().unwrap_or(UNIX_EPOCH));
            let start = match self.cache.get(path) {
                Some(c) if c.size == size && c.mtime == mtime => continue,
                // Grown: only the appended bytes need parsing.
                Some(c) if size > c.size => Some(c.parsed_len),
                // Shrunk, or same size with a different mtime: rewritten.
                _ => None,
            };
            let outcome = match start {
                Some(offset) => {
                    stats.appended += 1;
                    let cached = self.cache.get_mut(path).expect("checked above");
                    read_new_lines(path, offset, &mut cached.scanner)
                        .map(|(parsed, read)| (parsed, read, None))
                }
                None => {
                    stats.rescanned += 1;
                    let mut scanner = SessionScanner::new();
                    read_new_lines(path, 0, &mut scanner)
                        .map(|(parsed, read)| (parsed, read, Some(scanner)))
                }
            };
            let Ok((parsed_len, read, fresh)) = outcome else {
                self.cache.remove(path); // unreadable now: retry on the next refresh
                continue;
            };
            stats.bytes_read += read;
            match fresh {
                Some(scanner) => {
                    self.cache.insert(
                        path.clone(),
                        Cached {
                            size,
                            mtime,
                            parsed_len,
                            scanner,
                        },
                    );
                }
                None => {
                    let cached = self.cache.get_mut(path).expect("checked above");
                    cached.size = size;
                    cached.mtime = mtime;
                    cached.parsed_len = parsed_len;
                }
            }
        }
        Ok(stats)
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
mod tests;
