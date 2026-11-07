//! The session index cache file: one versioned JSON document, written
//! atomically.
//!
//! The file is only ever a speedup. [`load`] returns `None` for anything it
//! does not recognise (missing, unreadable, invalid JSON, another version) and
//! the caller then does the full scan.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::session_snapshot::IndexSnapshot;

/// Bumped whenever the snapshot shape changes; an older file is ignored.
pub const CACHE_VERSION: u32 = 1;

#[derive(Serialize)]
struct DocumentRef<'a> {
    version: u32,
    snapshot: &'a IndexSnapshot,
}

#[derive(Deserialize)]
struct Document {
    version: u32,
    snapshot: IndexSnapshot,
}

/// The snapshot stored at `path`, or `None` if there is no usable one.
pub fn load(path: &Path) -> Option<IndexSnapshot> {
    let data = fs::read(path).ok()?;
    let document: Document = serde_json::from_slice(&data).ok()?;
    (document.version == CACHE_VERSION).then_some(document.snapshot)
}

/// Writes `snapshot` to `path`, creating its directory. The document goes to a
/// temporary file next to it and is renamed over the target, so a reader never
/// sees a half-written file and a failure leaves the previous one in place.
pub fn save(path: &Path, snapshot: &IndexSnapshot) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!("{}.tmp", file_name(path)));
    if let Err(error) = write(&temp, snapshot) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || "session-index".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn write(path: &Path, snapshot: &IndexSnapshot) -> io::Result<()> {
    let document = DocumentRef {
        version: CACHE_VERSION,
        snapshot,
    };
    let data = serde_json::to_vec(&document)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut file = fs::File::create(path)?;
    file.write_all(&data)?;
    Ok(())
}

#[cfg(test)]
mod tests;
