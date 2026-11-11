//! Renaming a chat by appending one `session_info` line to its pi session file
//! (D1(a), T2).
//!
//! No real pi home: a fake filesystem answers the two calls the append needs, so
//! every test says exactly what reached the file. The bytes are read back through
//! the SAME scanner the session list uses (`gentle_core::session::SessionScanner`)
//! and through a real file in a temp directory, which is the only write these
//! tests make.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_backend::session_rename::{self, DiskFiles, RenameError, SessionFiles, EMPTY_NAME};
use gentle_core::session::SessionScanner;

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;
const MESSAGE: &str = r#"{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{"role":"user","content":"first chat"}}"#;
const OLD_NAME: &str =
    r#"{"type":"session_info","id":"n1","timestamp":"2026-01-01T00:02:00.000Z","name":"Old name"}"#;

/// The line one rename appends, as the file must carry it.
fn appended(name: &str) -> String {
    format!("{{\"type\":\"session_info\",\"name\":\"{name}\"}}\n")
}

/// The name the session list would read from `text`: the SCANNER's answer, never
/// this file's idea of the line.
fn read_back(text: &str) -> Option<String> {
    let mut scanner = SessionScanner::new();
    for line in text.split('\n') {
        scanner.push_line(line);
    }
    scanner.summary().expect("a valid session file").name
}

/// One in-memory session file, and the calls the append really made.
#[derive(Default)]
struct FakeFiles {
    files: Mutex<HashMap<PathBuf, String>>,
    calls: Mutex<Vec<String>>,
}

impl FakeFiles {
    fn holding(path: &str, text: &str) -> Self {
        let files = FakeFiles::default();
        lock(&files.files).insert(PathBuf::from(path), text.to_owned());
        files
    }

    fn text(&self, path: &str) -> String {
        lock(&self.files)
            .get(&PathBuf::from(path))
            .cloned()
            .unwrap_or_default()
    }

    /// Every call the seam received, in order, as `method:path`.
    fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl SessionFiles for FakeFiles {
    fn ends_with_newline(&self, path: &Path) -> io::Result<bool> {
        lock(&self.calls).push(format!("ends:{}", path.display()));
        let files = lock(&self.files);
        let text = files.get(path).ok_or_else(not_found)?;
        Ok(text.ends_with('\n'))
    }

    fn append(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        lock(&self.calls).push(format!("append:{}", path.display()));
        let mut files = lock(&self.files);
        let text = files.get_mut(path).ok_or_else(not_found)?;
        text.push_str(std::str::from_utf8(bytes).expect("utf-8 bytes"));
        Ok(())
    }
}

fn not_found() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "no such session file")
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn an_empty_or_blank_name_is_refused_and_the_file_is_never_touched() {
    let path = "C:\\pi\\sessions\\--p--\\s1.jsonl";
    let before = format!("{HEADER}\n{MESSAGE}\n");
    let files = FakeFiles::holding(path, &before);

    for blank in ["", "   ", "\t\n"] {
        let error = session_rename::rename_session(&files, Path::new(path), blank)
            .expect_err("a blank name is refused");
        assert_eq!(error, RenameError::EmptyName, "blank {blank:?}");
        assert_eq!(error.to_string(), EMPTY_NAME, "the one line the user reads");
    }

    assert_eq!(files.text(path), before, "not one byte reached the file");
    assert!(
        files.calls().is_empty(),
        "a refusal never asks the disk a question, got {:?}",
        files.calls()
    );
}

#[test]
fn the_rename_appends_one_session_info_line_and_the_scanner_reads_the_new_name() {
    let path = "C:\\pi\\sessions\\--p--\\s1.jsonl";
    let before = format!("{HEADER}\n{MESSAGE}\n{OLD_NAME}\n");
    let files = FakeFiles::holding(path, &before);
    assert_eq!(read_back(&before).as_deref(), Some("Old name"));

    session_rename::rename_session(&files, Path::new(path), "  Season two  ")
        .expect("the append runs");

    let after = files.text(path);
    assert_eq!(
        after,
        format!("{before}{}", appended("Season two")),
        "the name is trimmed, the line is one line, and nothing else changed"
    );
    assert!(
        after.ends_with('\n'),
        "the appended line ends with a newline"
    );
    let last = after.lines().last().expect("the appended line");
    assert_eq!(
        last, r#"{"type":"session_info","name":"Season two"}"#,
        "the exact shape pi itself writes"
    );
    assert_eq!(
        after.lines().count(),
        before.lines().count() + 1,
        "exactly one line was added"
    );
    assert_eq!(
        read_back(&after).as_deref(),
        Some("Season two"),
        "the LATEST session_info wins, so the list shows the new name"
    );
    assert_eq!(
        files.calls(),
        vec![format!("ends:{path}"), format!("append:{path}"),],
        "one question and one write"
    );
}

#[test]
fn a_file_that_does_not_end_with_a_newline_gets_a_line_of_its_own() {
    let path = "C:\\pi\\sessions\\--p--\\s1.jsonl";
    // The last line is complete but the file stops without a newline.
    let before = format!("{HEADER}\n{MESSAGE}");
    let files = FakeFiles::holding(path, &before);

    session_rename::rename_session(&files, Path::new(path), "Cut short").expect("the append runs");

    let after = files.text(path);
    assert_eq!(
        after,
        format!("{before}\n{}", appended("Cut short")),
        "a separator keeps the appended line out of the line that was there"
    );
    assert_eq!(
        after.lines().nth(1),
        Some(MESSAGE),
        "the last line of the file is untouched"
    );
    assert_eq!(read_back(&after).as_deref(), Some("Cut short"));
}

#[test]
fn a_missing_session_file_is_reported_and_nothing_is_created() {
    let path = "C:\\pi\\sessions\\--p--\\gone.jsonl";
    let files = FakeFiles::default();

    let error = session_rename::rename_session(&files, Path::new(path), "New name")
        .expect_err("a file that is not there cannot be renamed");

    assert!(
        matches!(error, RenameError::Disk(_)),
        "the reason is the disk's, got {error:?}"
    );
    assert_eq!(files.text(path), "", "no file is created");
    assert_eq!(
        files.calls(),
        vec![format!("ends:{path}")],
        "the write is never attempted"
    );
}

#[test]
fn the_real_filesystem_append_keeps_the_existing_bytes_and_reads_back() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path: PathBuf = dir.path().join("s1.jsonl");
    let before = format!("{HEADER}\n{MESSAGE}");
    std::fs::write(&path, &before).expect("the session file");

    session_rename::rename_session(&DiskFiles, &path, "Renamed on disk").expect("the append runs");

    let after = std::fs::read_to_string(&path).expect("the session file");
    assert_eq!(after, format!("{before}\n{}", appended("Renamed on disk")));
    assert_eq!(read_back(&after).as_deref(), Some("Renamed on disk"));

    // The same file renamed again keeps the last name as the latest one.
    session_rename::rename_session(&DiskFiles, &path, "And again").expect("the second append");
    let twice = std::fs::read_to_string(&path).expect("the session file");
    assert_eq!(twice, format!("{after}{}", appended("And again")));
    assert_eq!(read_back(&twice).as_deref(), Some("And again"));
}
