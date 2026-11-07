//! A temporary pi home for the session-index tests: one sessions directory with
//! one project directory, plus the small edits a test makes to a session file.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use super::SessionIndex;

pub fn header(id: &str) -> String {
    format!(
        r#"{{"type":"session","version":3,"id":"{id}","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\{id}"}}"#
    )
}

pub fn user(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

pub struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn sessions(&self) -> std::path::PathBuf {
        self.root().join("sessions")
    }

    pub fn project(&self) -> std::path::PathBuf {
        self.sessions().join("--C--a--")
    }

    /// Writes the whole file `<project>/<name>.jsonl`.
    pub fn write(&self, name: &str, lines: &[String]) -> std::path::PathBuf {
        fs::create_dir_all(self.project()).unwrap();
        let path = self.project().join(format!("{name}.jsonl"));
        fs::write(
            &path,
            lines.iter().map(|l| format!("{l}\n")).collect::<String>(),
        )
        .unwrap();
        path
    }

    pub fn index(&self) -> SessionIndex {
        SessionIndex::new(self.sessions())
    }
}

/// Appends `line` as a complete line; returns the bytes written.
pub fn append(path: &Path, line: &str) -> usize {
    let text = format!("{line}\n");
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
    text.len()
}

/// mtime granularity can hide a rewrite; force a distinct one.
pub fn bump_mtime(path: &Path, seconds: u64) {
    let file = File::options().write(true).open(path).unwrap();
    let mtime = fs::metadata(path).unwrap().modified().unwrap() + Duration::from_secs(seconds);
    file.set_modified(mtime).unwrap();
}

/// The message count the cache holds for `path`, or `None` when not cached.
pub fn messages(index: &SessionIndex, path: &Path) -> Option<u32> {
    index
        .entries()
        .iter()
        .find(|entry| entry.path.as_path() == path)
        .map(|entry| entry.summary.message_count)
}
