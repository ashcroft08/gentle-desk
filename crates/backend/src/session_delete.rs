//! Removing a chat's pi session file (T3, D2(b), S3).
//!
//! Deleting a chat removes exactly ONE file: the session `.jsonl` the row names.
//! Nothing else is touched - no folder is walked and no sibling is looked at - so
//! the delete cannot take another chat with it. The list heals by itself
//! afterwards: the index drops a file it no longer finds and the watcher
//! classifies the removal (S3), which is why nothing here refreshes anything.
//!
//! The file system is a seam ([`SessionRemovals`]) so a test never needs a pi
//! home, and [`DiskRemovals`] is the one real implementation.

use std::fs;
use std::io;
use std::path::Path;

/// The one line a user reads when the file the delete named is not there: the
/// chat was already removed, or the path was never a session.
pub const MISSING_SESSION: &str = "that chat's session file is not there any more";

/// Why a delete did not reach the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteError {
    /// There was no file at the path: it is gone already, or it never was one.
    Missing,
    /// The file was there and could not be removed; the message is the OS's.
    Disk(String),
}

impl std::fmt::Display for DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str(MISSING_SESSION),
            Self::Disk(message) => {
                write!(f, "the chat's session file could not be removed: {message}")
            }
        }
    }
}

impl std::error::Error for DeleteError {}

/// The one call a delete needs from a file system: remove the file at exactly
/// this path. Nothing here lists, walks or creates.
pub trait SessionRemovals {
    /// Removes the file `path` names; a missing file is an error, never a
    /// success, so a delete of a chat that is already gone says so.
    fn remove(&self, path: &Path) -> io::Result<()>;
}

/// The real filesystem: one `remove_file` of the path the row named, and
/// nothing else.
pub struct DiskRemovals;

impl SessionRemovals for DiskRemovals {
    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

/// Removes the one session file at `path`, and nothing else about the disk.
///
/// An empty path is refused before the file system is asked, because it names no
/// file at all; a path the system will not remove (a folder, a file another
/// process holds) is reported with the system's own reason.
pub fn delete_session(files: &dyn SessionRemovals, path: &Path) -> Result<(), DeleteError> {
    if path.as_os_str().is_empty() {
        return Err(DeleteError::Missing);
    }
    files.remove(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => DeleteError::Missing,
        _ => DeleteError::Disk(error.to_string()),
    })
}
