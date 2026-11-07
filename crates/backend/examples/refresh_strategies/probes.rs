//! The changes both candidates are measured against: an existing session that
//! gets a new message, and a brand new session file.
//!
//! Shared so every candidate opts the same bytes into existence: an append that
//! the scanner counts as one message, and a header-only file it lists.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gentle_backend::session_index::SessionIndex;

/// One real session file, for the "the user chats" append trials.
pub fn first_session(index: &SessionIndex) -> PathBuf {
    index
        .entries()
        .first()
        .map(|entry| entry.path.clone())
        .expect("the sessions directory holds no sessions")
}

/// The message count the list reports for `target`, or 0 while the file is not
/// in the list at all.
pub fn message_count(index: &SessionIndex, target: &Path) -> u32 {
    index
        .entries()
        .iter()
        .find(|entry| entry.path == target)
        .map_or(0, |entry| entry.summary.message_count)
}

/// Appends one complete message entry stamped now, so the session's activity
/// moves to now and its message count grows by one.
pub fn append_message(target: &Path) {
    let line = format!(
        "{{\"type\":\"message\",\"id\":\"probe-{}\",\"timestamp\":\"2026-01-01T00:00:00.000Z\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",\"text\":\"probe\"}}],\"timestamp\":{}}}}}\n",
        now_ms(),
        now_ms()
    );
    let mut file = OpenOptions::new()
        .append(true)
        .open(target)
        .expect("cannot open a session file for appending");
    file.write_all(line.as_bytes())
        .expect("cannot append to a session file");
}

/// Writes a header-only session file: valid, and listed once the index sees it.
pub fn write_session_file(path: &Path) {
    let header = format!(
        "{{\"type\":\"session\",\"version\":3,\"id\":\"probe-{}\",\"timestamp\":\"2026-01-01T00:00:00.000Z\",\"cwd\":\"C:/dev/probe\"}}\n",
        now_ms()
    );
    fs::write(path, header).expect("cannot write a probe session file");
}

/// The first project directory under `sessions`: pi keeps one per cwd, and the
/// probe files go next to real sessions.
pub fn first_project_dir(sessions: &Path) -> PathBuf {
    let mut dirs: Vec<PathBuf> = fs::read_dir(sessions)
        .expect("cannot read the sessions directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs.into_iter()
        .next()
        .expect("the sessions directory holds no project directory")
}

/// Milliseconds since the epoch: names of probe files carry it, so every run
/// creates files nobody has seen before (a burst must be creations, not
/// overwrites of the previous run's probes).
pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis())
}
