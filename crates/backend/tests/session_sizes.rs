//! How big each chat's session file is, as the session index reports it (S2).
//!
//! The removal offer says "2 chats (1.7 MB)", so the number has to be the
//! file's own length and has to follow the file as it grows. The index already
//! caches that length per file (it is what decides whether a file was appended
//! to or rewritten); this test pins that the entry the caller reads carries it.
//!
//! No real pi home: a temp directory holds the session files, and the size the
//! index reports is compared with the file's own metadata.

use std::fs;
use std::path::{Path, PathBuf};

use gentle_backend::session_index::SessionIndex;

fn header(id: &str) -> String {
    format!(
        r#"{{"type":"session","version":3,"id":"{id}","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\{id}"}}"#
    )
}

fn user(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("a temp directory"),
        }
    }

    fn sessions(&self) -> PathBuf {
        self.dir.path().join("sessions")
    }

    /// Writes `lines` as the whole file `<project>/<name>.jsonl`.
    fn write(&self, project: &str, name: &str, lines: &[String]) -> PathBuf {
        let dir = self.sessions().join(project);
        fs::create_dir_all(&dir).expect("the project folder");
        let path = dir.join(format!("{name}.jsonl"));
        fs::write(
            &path,
            lines
                .iter()
                .map(|line| format!("{line}\n"))
                .collect::<String>(),
        )
        .expect("the session file");
        path
    }

    fn index(&self) -> SessionIndex {
        SessionIndex::new(self.sessions())
    }
}

/// The size the index reports for `path`, or `None` when it lists no such row.
fn bytes_of(index: &SessionIndex, path: &Path) -> Option<u64> {
    index
        .entries()
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.bytes)
}

fn on_disk(path: &Path) -> u64 {
    fs::metadata(path).expect("the file is there").len()
}

#[test]
fn every_entry_carries_the_length_of_its_own_session_file() {
    let home = Home::new();
    let small = home.write("--C--dev--a--", "2026-01-01T00-00-00_s1", &[header("a")]);
    let large = home.write(
        "--C--dev--b--",
        "2026-01-02T00-00-00_s2",
        &[header("b"), user("a much longer first message")],
    );
    let mut index = home.index();

    index.refresh().expect("the sessions folder is read");

    assert_eq!(
        bytes_of(&index, &small),
        Some(on_disk(&small)),
        "one chat's size is its own file's"
    );
    assert_eq!(
        bytes_of(&index, &large),
        Some(on_disk(&large)),
        "and the next chat's is its own, never the first one's"
    );
    assert!(
        on_disk(&large) > on_disk(&small),
        "the two files really differ, so neither number is a coincidence"
    );
}

#[test]
fn the_size_follows_a_file_the_session_grows_by() {
    let home = Home::new();
    let path = home.write("--C--dev--a--", "2026-01-01T00-00-00_s1", &[header("a")]);
    let mut index = home.index();
    index.refresh().expect("the sessions folder is read");
    let before = bytes_of(&index, &path).expect("the chat is listed");

    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("the session file");
    std::io::Write::write_all(&mut file, format!("{}\n", user("hola")).as_bytes())
        .expect("the appended line");
    drop(file);
    index.refresh().expect("the grown file is folded");

    assert_eq!(bytes_of(&index, &path), Some(on_disk(&path)));
    assert!(
        bytes_of(&index, &path).expect("the chat is listed") > before,
        "a chat that was appended to is bigger than it was"
    );
}

#[test]
fn a_session_file_the_index_never_saw_reports_no_size() {
    let home = Home::new();
    home.write("--C--dev--a--", "2026-01-01T00-00-00_s1", &[header("a")]);
    let mut index = home.index();
    index.refresh().expect("the sessions folder is read");

    assert_eq!(
        bytes_of(&index, Path::new("gone.jsonl")),
        None,
        "no row, no size"
    );
}
