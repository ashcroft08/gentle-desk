//! The data a [`SessionIndex`](crate::session_index::SessionIndex) persists
//! between runs: per file, the size and mtime it was parsed from plus the
//! scanner state, so a later run can resume without reading the file again.
//!
//! Plain data with no I/O: reading and writing the file lives in
//! [`session_cache`](crate::session_cache).

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gentle_core::session::SessionScanner;
use serde::{Deserialize, Serialize};

/// Everything the index knew, tagged with the directory it was built for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexSnapshot {
    /// The `<home>/sessions` directory this snapshot describes.
    pub sessions_dir: PathBuf,
    /// Sorted by path, so the same index always writes the same bytes.
    pub files: Vec<FileSnapshot>,
}

/// One cached session file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: PathBuf,
    /// File size the scanner is in sync with.
    pub size: u64,
    /// Modification time, seconds since `UNIX_EPOCH`.
    pub mtime_secs: u64,
    /// The sub-second part of the modification time.
    pub mtime_nanos: u32,
    /// Bytes already folded into `scanner`: up to the last complete line.
    pub parsed_len: u64,
    pub scanner: SessionScanner,
}

/// Splits a modification time into the two numbers a snapshot stores.
pub(crate) fn split_mtime(time: SystemTime) -> (u64, u32) {
    let since = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    (since.as_secs(), since.subsec_nanos())
}

/// The inverse of [`split_mtime`]. An impossible time cannot match any file, so
/// a corrupt snapshot costs a rescan instead of a panic.
pub(crate) fn join_mtime(secs: u64, nanos: u32) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::new(secs, nanos.min(999_999_999)))
        .unwrap_or(UNIX_EPOCH)
}
