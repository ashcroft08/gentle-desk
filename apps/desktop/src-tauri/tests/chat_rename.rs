//! Renaming a chat (T2): the command's own rules, windowless.
//!
//! What reaches the file is `gentle-backend`'s append, tested there; what this
//! file tests is the rule AROUND it that the command owes the user: a chat whose
//! child is running a turn is not renamed, and every refusal arrives as one line
//! of the pushed slice instead of an error the user cannot see. A fake filesystem
//! answers the append and a fake activity answers the live-child question, so no
//! pi home, child or window is involved.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_backend::session_rename::{SessionFiles, EMPTY_NAME};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::chat_rename::{self, WorkingChats, WORKING_CHILD};

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;
const MESSAGE: &str = r#"{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{"role":"user","content":"first chat"}}"#;
const SESSION: &str = "C:\\pi\\sessions\\--dev-p--\\2026_s1.jsonl";

/// The session file's text before any rename.
fn before() -> String {
    format!("{HEADER}\n{MESSAGE}\n")
}

/// The line a rename must have appended.
fn appended(name: &str) -> String {
    format!("{{\"type\":\"session_info\",\"name\":\"{name}\"}}\n")
}

/// The session files a test holds, in memory, plus how many writes they took.
#[derive(Default)]
struct FakeFiles {
    files: Mutex<HashMap<PathBuf, String>>,
    writes: Mutex<usize>,
}

impl FakeFiles {
    /// The one session file of these tests, already holding `text`.
    fn holding(text: &str) -> Self {
        let files = FakeFiles::default();
        lock(&files.files).insert(PathBuf::from(SESSION), text.to_owned());
        files
    }

    fn text(&self) -> String {
        lock(&self.files)
            .get(&PathBuf::from(SESSION))
            .cloned()
            .unwrap_or_default()
    }

    fn writes(&self) -> usize {
        *lock(&self.writes)
    }
}

impl SessionFiles for FakeFiles {
    fn ends_with_newline(&self, path: &Path) -> io::Result<bool> {
        let files = lock(&self.files);
        let text = files
            .get(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such session file"))?;
        Ok(text.ends_with('\n'))
    }

    fn append(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut files = lock(&self.files);
        let text = files
            .get_mut(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such session file"))?;
        text.push_str(std::str::from_utf8(bytes).expect("utf-8 bytes"));
        *lock(&self.writes) += 1;
        Ok(())
    }
}

/// The chats whose child is running a turn, as the app's activity marks them.
struct FakeWork {
    working: Vec<String>,
}

impl FakeWork {
    fn working(session: &str) -> Self {
        Self {
            working: vec![session.to_owned()],
        }
    }

    fn idle() -> Self {
        Self {
            working: Vec::new(),
        }
    }
}

impl WorkingChats for FakeWork {
    fn is_working(&self, session: &str) -> bool {
        self.working.iter().any(|known| known == session)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn a_working_chat_is_refused_with_one_notice_and_its_file_is_untouched() {
    let holder = AppStateHolder::default();
    let before_view = holder.view();
    let files = FakeFiles::holding(&before());

    let renamed = chat_rename::rename(
        &holder,
        &files,
        &FakeWork::working(SESSION),
        SESSION,
        "New name",
    );

    assert!(!renamed.written, "nothing was written");
    assert_eq!(files.text(), before(), "the session file is byte for byte");
    assert_eq!(files.writes(), 0, "the append never ran");
    assert_eq!(
        renamed.view.sessions_notice.as_deref(),
        Some(WORKING_CHILD),
        "the reason reaches the user as a notice"
    );
    assert_eq!(
        renamed.view.sessions_revision, before_view.sessions_revision,
        "no list moved"
    );
}

#[test]
fn an_idle_chat_is_renamed_and_the_earlier_notice_goes_away() {
    let holder = AppStateHolder::default();
    holder.note_sessions_problem("an earlier refusal".to_owned());
    assert_eq!(
        holder.view().sessions_notice.as_deref(),
        Some("an earlier refusal")
    );
    let files = FakeFiles::holding(&before());

    let renamed = chat_rename::rename(&holder, &files, &FakeWork::idle(), SESSION, "Season two");

    assert!(
        renamed.written,
        "the command asks for the pass that reads it back"
    );
    assert_eq!(
        files.text(),
        format!("{}{}", before(), appended("Season two")),
        "one session_info line was appended"
    );
    assert_eq!(files.writes(), 1, "one write");
    assert_eq!(
        renamed.view.sessions_notice, None,
        "the stale refusal is gone"
    );
}

#[test]
fn a_blank_name_is_refused_with_the_reason_and_the_file_is_untouched() {
    let holder = AppStateHolder::default();
    let files = FakeFiles::holding(&before());

    let renamed = chat_rename::rename(&holder, &files, &FakeWork::idle(), SESSION, "   ");

    assert!(!renamed.written);
    assert_eq!(files.text(), before(), "the session file is byte for byte");
    assert_eq!(
        renamed.view.sessions_notice.as_deref(),
        Some(EMPTY_NAME),
        "the backend's own line is what the user reads"
    );
}

#[test]
fn a_file_that_could_not_be_written_says_so_instead_of_claiming_a_rename() {
    let holder = AppStateHolder::default();
    let files = FakeFiles::default();

    let renamed = chat_rename::rename(&holder, &files, &FakeWork::idle(), SESSION, "New name");

    assert!(!renamed.written);
    let notice = renamed.view.sessions_notice.expect("the disk reason");
    assert!(
        notice.contains("session file could not be written"),
        "got {notice:?}"
    );
    assert_eq!(files.writes(), 0, "the append never ran");
}
