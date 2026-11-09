//! Keeping an unusable file instead of overwriting it.
//!
//! The file is renamed to `<name>.corrupt-<unix seconds>` next to where it was,
//! and then to `<name>.corrupt-<unix seconds>-<n>` when an earlier corruption in
//! the same second already took that name. (The count starts at 2: the plain
//! name is the first choice.) An earlier aside file is never replaced.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// How many aside names to try before giving up for this run.
const ATTEMPTS: u32 = 100;
/// Added to the attempt number, so the first counted name is `<name>-2`.
const FIRST_COUNTED: u32 = 1;

/// Renames `path` to a free aside name and answers where it went.
pub(super) fn move_aside(path: &Path) -> io::Result<PathBuf> {
    let seconds = unix_seconds();
    let base = super::file_name(path);
    for attempt in 0..ATTEMPTS {
        let aside = path.with_file_name(aside_name(&base, seconds, attempt));
        if aside.exists() {
            continue;
        }
        fs::rename(path, &aside)?;
        return Ok(aside);
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no free name to keep the unusable file aside",
    ))
}

fn aside_name(base: &str, seconds: u64, attempt: u32) -> String {
    match attempt {
        0 => format!("{base}.corrupt-{seconds}"),
        taken => format!("{base}.corrupt-{seconds}-{}", taken + FIRST_COUNTED),
    }
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
