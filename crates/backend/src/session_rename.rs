//! Writing a chat's own name into its pi session file (D1(a), sessions-rename-delete T2).
//!
//! A chat's title is `name.or(first_message)` and `name` is the LATEST
//! `session_info` line of the session file (sessions-m4 S4,
//! [`gentle_core::session::SessionScanner`]). A rename therefore appends one line
//! to the file pi already reads, instead of keeping a name of our own: the
//! terminal and the desk then show the same title (S2), and no path, id or child
//! moves (D1(a)).
//!
//! The append is deliberately the ONLY thing this module does to the file: the
//! existing bytes are never rewritten, never reordered and never normalized. A
//! file whose last line carries no trailing newline gets a separator first, so the
//! new line is its own line, and every line written here ends with a newline -
//! the state pi's own writes leave the file in.
//!
//! The file system is a seam ([`SessionFiles`]) so a test never needs a pi home,
//! and [`DiskFiles`] is the one real implementation.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// The one line a user reads when the name is empty or only spaces. Trimming is
/// silent for a name with text in it, because a name is typed with stray spaces
/// around it more often than not.
pub const EMPTY_NAME: &str = "a chat name cannot be empty";

/// Why a rename did not reach the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    /// The name was empty, or only whitespace.
    EmptyName,
    /// The session file could not be read or written; the message is the OS's.
    Disk(String),
}

impl std::fmt::Display for RenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => f.write_str(EMPTY_NAME),
            Self::Disk(message) => {
                write!(f, "the chat's session file could not be written: {message}")
            }
        }
    }
}

impl std::error::Error for RenameError {}

/// The two calls the append needs from a file: what its last byte is, and one
/// append of bytes at the end. Nothing here creates, truncates or rewrites a
/// file, and a missing file is an error, never a new session.
pub trait SessionFiles {
    /// Whether the file's last byte is a newline. `false` for an empty file.
    fn ends_with_newline(&self, path: &Path) -> io::Result<bool>;

    /// Appends `bytes` at the end of an EXISTING file and does not create one.
    fn append(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
}

/// The real filesystem: the append the app does to the user's pi data, and the
/// only write of that kind in the whole app.
pub struct DiskFiles;

impl SessionFiles for DiskFiles {
    fn ends_with_newline(&self, path: &Path) -> io::Result<bool> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        if length == 0 {
            return Ok(false);
        }
        file.seek(SeekFrom::Start(length - 1))?;
        let mut last = [0u8; 1];
        file.read_exact(&mut last)?;
        Ok(last[0] == b'\n')
    }

    fn append(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        // `append` alone creates nothing: without `create`/`create_new` a missing
        // file is `NotFound`, which is exactly what a rename of a chat that is
        // gone must answer. The bytes go out in ONE write, so an append never
        // leaves half a line behind.
        let mut file = OpenOptions::new().append(true).open(path)?;
        file.write_all(bytes)?;
        file.flush()
    }
}

/// Appends the new name to the session file at `path`, as one `session_info` line,
/// and changes nothing else about it.
///
/// A blank name is refused BEFORE the file is even looked at, so a refusal is
/// never a half write. The name is trimmed; whatever it holds is escaped as JSON,
/// so a name with a quote in it is still one line.
pub fn rename_session(
    files: &dyn SessionFiles,
    path: &Path,
    name: &str,
) -> Result<(), RenameError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RenameError::EmptyName);
    }
    let ends_with_newline = files
        .ends_with_newline(path)
        .map_err(|error| RenameError::Disk(error.to_string()))?;
    let line = rename_line(trimmed, ends_with_newline)?;
    files
        .append(path, line.as_bytes())
        .map_err(|error| RenameError::Disk(error.to_string()))
}

/// The bytes one rename appends: the separator the file needs, the
/// `session_info` line pi itself writes, and the newline that ends it.
fn rename_line(name: &str, ends_with_newline: bool) -> Result<String, RenameError> {
    let quoted =
        serde_json::to_string(name).map_err(|error| RenameError::Disk(error.to_string()))?;
    let separator = if ends_with_newline { "" } else { "\n" };
    Ok(format!(
        "{separator}{{\"type\":\"session_info\",\"name\":{quoted}}}\n"
    ))
}
