//! The per-file half of the session index: one cache entry per session file
//! and the single place that brings an entry in line with the disk.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use gentle_core::session::SessionScanner;

use super::disk::read_new_lines;
use super::RefreshStats;

/// A cache entry: the file's identity the last time it was folded in, plus the
/// scanner holding everything parsed so far.
pub struct Cached {
    pub size: u64,
    pub mtime: SystemTime,
    /// Bytes already folded into `scanner`: up to the last complete line.
    pub parsed_len: u64,
    pub scanner: SessionScanner,
}

/// What [`update`] did, so both refresh paths count it the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Update {
    /// Size and mtime match the entry: nothing was read.
    Unchanged,
    /// There is no usable file behind the path: the entry was left alone.
    Missing,
    /// A read succeeded, from the previous end unless it was a rescan.
    Read { rescanned: bool, bytes_read: u64 },
    /// The read failed and the entry was dropped, so the next pass retries.
    Failed { rescanned: bool },
}

impl Update {
    /// Adds this update to `stats`. A failed read still counts as an attempt,
    /// exactly as a full refresh has always counted it.
    pub fn count(self, stats: &mut RefreshStats) {
        let (rescanned, bytes_read) = match self {
            Self::Unchanged | Self::Missing => return,
            Self::Read {
                rescanned,
                bytes_read,
            } => (rescanned, bytes_read),
            Self::Failed { rescanned } => (rescanned, 0),
        };
        stats.bytes_read += bytes_read;
        if rescanned {
            stats.rescanned += 1;
        } else {
            stats.appended += 1;
        }
    }
}

/// Brings `entry` in line with the file at `path`: unchanged, grown (only the
/// appended bytes), rewritten or shrunk (from the start), new (from the start),
/// or gone. An unreadable file drops the entry.
pub fn update(entry: &mut Option<Cached>, path: &Path) -> Update {
    let Ok(meta) = fs::metadata(path) else {
        return Update::Missing;
    };
    let (size, mtime) = (meta.len(), meta.modified().unwrap_or(UNIX_EPOCH));
    let start = match entry.as_ref() {
        Some(cached) if cached.size == size && cached.mtime == mtime => return Update::Unchanged,
        // Grown: only the appended bytes need parsing.
        Some(cached) if size > cached.size => Some(cached.parsed_len),
        // Shrunk, or same size with a different mtime: rewritten.
        _ => None,
    };
    let outcome = match start {
        Some(offset) => {
            let cached = entry.as_mut().expect("checked above");
            read_new_lines(path, offset, &mut cached.scanner)
                .map(|(parsed_len, bytes_read)| (parsed_len, bytes_read, None))
        }
        None => {
            let mut scanner = SessionScanner::new();
            read_new_lines(path, 0, &mut scanner)
                .map(|(parsed_len, bytes_read)| (parsed_len, bytes_read, Some(scanner)))
        }
    };
    let Ok((parsed_len, bytes_read, fresh)) = outcome else {
        *entry = None; // unreadable now: retry on the next pass
        return Update::Failed {
            rescanned: start.is_none(),
        };
    };
    match fresh {
        Some(scanner) => {
            *entry = Some(Cached {
                size,
                mtime,
                parsed_len,
                scanner,
            });
        }
        None => {
            let cached = entry.as_mut().expect("checked above");
            cached.size = size;
            cached.mtime = mtime;
            cached.parsed_len = parsed_len;
        }
    }
    Update::Read {
        rescanned: start.is_none(),
        bytes_read,
    }
}
