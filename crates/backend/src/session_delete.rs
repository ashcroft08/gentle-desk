//! Removing a chat's pi session file (T3, D2(b), S3).
//!
//! Deleting a chat removes exactly ONE file: the session `.jsonl` the row names.
//! Nothing else is touched - no folder is walked and no sibling is looked at - so
//! the delete cannot take another chat with it. The list heals by itself
//! afterwards: the index drops a file it no longer finds and the watcher
//! classifies the removal (S3), which is why nothing here refreshes anything.
//!
//! The file goes to the OS TRASH first (S4, the maintainer's 2026-11-16 ruling:
//! "papelera, como pi"), and unlinking the file is only the fallback for a
//! system that will not take it - a volume with no recycle bin, a file another
//! process holds open. pi's own terminal delete does the same thing in the same
//! order, so a chat deleted here is recoverable the same way.
//!
//! The two filesystem calls are seams ([`SessionRemovals`] for one file's
//! removal, [`TrashMoves`] for the move into the trash) so a test never needs a
//! pi home and never needs a real recycle bin; [`DiskRemovals`] and [`OsTrash`]
//! are the two real implementations.

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

/// The one call a delete needs from the OS trash (S4): move the file at exactly
/// this path into it, and nothing else. A move that fails is reported with the
/// system's own reason, which is what makes it the fallback's turn.
pub trait TrashMoves {
    fn move_to_trash(&self, path: &Path) -> io::Result<()>;
}

/// The real trash: the system's own, which on Windows is the recycle bin (the
/// `trash` crate's native API), exactly as pi's terminal delete moves it.
pub struct OsTrash;

impl TrashMoves for OsTrash {
    fn move_to_trash(&self, path: &Path) -> io::Result<()> {
        trash::delete(path).map_err(|error| io::Error::other(error.to_string()))
    }
}

/// The real filesystem: the file goes to the trash first and is unlinked only
/// when the trash refuses it (S4), and nothing else about the disk changes.
pub struct DiskRemovals;

impl SessionRemovals for DiskRemovals {
    fn remove(&self, path: &Path) -> io::Result<()> {
        remove_file_trash_first(&OsTrash, path)
    }
}

/// Removes the one file at `path`, trash-first (S4): the move into the trash IS
/// the removal, and `unlink` is the fallback for a system that will not take it.
///
/// The fallback's reason is the one reported, never the trash's, so a file that
/// is not there is still a missing file whichever of the two noticed it first.
pub fn remove_file_trash_first(trash: &dyn TrashMoves, path: &Path) -> io::Result<()> {
    match trash.move_to_trash(path) {
        Ok(()) => Ok(()),
        Err(_) => fs::remove_file(path),
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
