//! A temporary pi home for the refresh tests: a sessions directory with one
//! project per name, one file per session, and the small reads both test files
//! share (the list a pass produced, a snapshot to restore).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gentle_app::PreparedSessions;
use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_snapshot::IndexSnapshot;

use super::{Refreshed, SharedIndex};

pub const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;

pub fn message(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

/// A pi home with a `sessions/` directory and one file per session.
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

    pub fn sessions(&self) -> PathBuf {
        self.root().join("sessions")
    }

    pub fn project(&self, name: &str) -> PathBuf {
        let dir = self.sessions().join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A session file with a header and one user message.
    pub fn session(&self, project: &str, name: &str, first: &str) -> PathBuf {
        let path = self.project(project).join(name);
        let body = format!("{HEADER}\n{}\n", message(first));
        fs::write(&path, body).unwrap();
        path
    }

    /// Appends `line` and returns the bytes written, as a pass should read them.
    pub fn append(&self, path: &Path, line: &str) -> u64 {
        let text = format!("{line}\n");
        let mut body = fs::read_to_string(path).unwrap();
        body.push_str(&text);
        fs::write(path, body).unwrap();
        text.len() as u64
    }

    pub fn index(&self) -> SessionIndex {
        SessionIndex::new(self.sessions())
    }

    pub fn cache(&self) -> PathBuf {
        self.root().join("data").join("session-index-v1.json")
    }
}

pub fn index_in(sessions: PathBuf) -> SharedIndex {
    Arc::new(Mutex::new(SessionIndex::new(sessions)))
}

/// The titles of a prepared list, in order.
pub fn names(sessions: &PreparedSessions) -> Vec<String> {
    sessions
        .items()
        .iter()
        .map(|item| item.title.clone())
        .collect()
}

/// The titles a finished pass produced, in order.
pub fn titles(refreshed: &Refreshed) -> Vec<String> {
    names(&refreshed.sessions)
}

/// The snapshot of the home as it is now, read by one full pass.
pub fn warm_snapshot(home: &Home) -> IndexSnapshot {
    let mut index = home.index();
    index.refresh().unwrap();
    index.snapshot()
}
