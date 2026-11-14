//! Deleting a chat (T3, D2(b), T4, D3(a)): the command's own rules, windowless.
//!
//! What reaches the disk is `gentle-backend`'s one `remove_file`, tested there;
//! what this file tests is the rule AROUND it that the command owes the user:
//! nothing happens without the confirmation the dialog sends, a chat whose child
//! is running a turn is refused with one line, every refusal arrives as one line
//! of the pushed slice instead of an error the user cannot see, and the workspace
//! row a derived folder loses is reported only when the current workspace was the
//! one that vanished. A fake filesystem answers the removal and a fake activity
//! answers the live-child question, so no pi home, child or window is involved.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_backend::session_delete::{SessionRemovals, MISSING_SESSION};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::chat_delete::{self, working_child, workspace_gone};
use gentle_desk_lib::chat_rename::WorkingChats;
use gentle_desk_lib::sessions::view::NEW_CHAT_TITLE;
use gentle_runtime::app_config::AppConfig;

const SESSION: &str = r"C:\pi\sessions\--dev-repo--\2026_s1.jsonl";
const OTHER_SESSION: &str = r"C:\pi\sessions\--dev-notes--\2026_s2.jsonl";
const SIBLING: &str = r"C:\pi\sessions\--dev-repo--\2026_s3.jsonl";
const REPO: &str = r"C:\dev\repo";
const NOTES: &str = r"C:\dev\notes";

/// One session file of the list, as the first pass reports it.
fn session(path: &str, cwd: &str) -> SessionItem {
    SessionItem {
        path: path.to_owned(),
        title: "A chat".to_owned(),
        cwd: cwd.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The holder whose list holds `sessions`, with the chat stored at `foreground`
/// opened the way a click on its row opens it.
fn holder_with(sessions: Vec<SessionItem>, foreground: &str) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(sessions));
    holder.fold_input(Input::Intent(Intent::OpenChat(ChatId::of_session(
        foreground,
    ))));
    holder
}

/// The files a fake disk holds, and every path the seam was asked to remove.
#[derive(Default)]
struct FakeDisk {
    files: Mutex<HashSet<PathBuf>>,
    calls: Mutex<Vec<String>>,
}

impl FakeDisk {
    fn holding(paths: &[&str]) -> Self {
        let disk = FakeDisk::default();
        *lock(&disk.files) = paths.iter().map(PathBuf::from).collect();
        disk
    }

    fn holds(&self, path: &str) -> bool {
        lock(&self.files).contains(Path::new(path))
    }

    fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl SessionRemovals for FakeDisk {
    fn remove(&self, path: &Path) -> io::Result<()> {
        lock(&self.calls).push(path.display().to_string());
        if lock(&self.files).remove(path) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no such session file",
            ))
        }
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
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    let before_view = holder.view();
    let disk = FakeDisk::holding(&[SESSION]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::working(SESSION), SESSION, true);

    assert!(!deleted.removed, "nothing was removed");
    assert!(disk.holds(SESSION), "the session file is still there");
    assert!(disk.calls().is_empty(), "the removal never ran");
    assert_eq!(
        deleted.view.sessions_notice.as_deref(),
        Some(working_child().as_str()),
        "the reason reaches the user as a notice"
    );
    assert_eq!(
        working_child(),
        "This chat is working. Stop it first, then delete it.",
        "the delete flow's words are unchanged: Close chat shares their sentence, not the other way round"
    );
    assert_eq!(
        deleted.view.sessions_revision, before_view.sessions_revision,
        "no list moved"
    );
}

#[test]
fn without_the_confirmation_the_command_does_nothing_at_all() {
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    let disk = FakeDisk::holding(&[SESSION]);
    let before = holder.view();

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, false);

    assert!(!deleted.removed, "no file is removed");
    assert!(disk.holds(SESSION), "the session file is still there");
    assert!(disk.calls().is_empty(), "the removal is never asked for");
    assert_eq!(
        deleted.view.sessions_notice, None,
        "a dialog the user never confirmed is not a refusal to report"
    );
    assert_eq!(deleted.view, before, "the slice is the one it already had");
}

#[test]
fn a_confirmed_delete_removes_that_one_file_and_clears_the_earlier_notice() {
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    holder.note_sessions_problem("an earlier refusal".to_owned());
    let disk = FakeDisk::holding(&[SESSION, SIBLING]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, true);

    assert!(
        deleted.removed,
        "the command asks for the pass that heals the list"
    );
    assert!(!disk.holds(SESSION), "the one named file is gone");
    assert!(disk.holds(SIBLING), "a sibling session is untouched");
    assert_eq!(
        disk.calls(),
        vec![SESSION.to_owned()],
        "exactly one path was removed"
    );
    assert_eq!(
        deleted.view.sessions_notice, None,
        "the stale refusal is gone"
    );
}

#[test]
fn a_session_file_that_is_gone_is_reported_instead_of_claiming_a_delete() {
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    let disk = FakeDisk::default();

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, true);

    assert!(!deleted.removed);
    assert_eq!(
        deleted.view.sessions_notice.as_deref(),
        Some(MISSING_SESSION),
        "the backend's own line is what the user reads"
    );
}

#[test]
fn the_last_chat_of_a_derived_workspace_takes_its_row_with_a_notice() {
    // The folder is known only from the session cwd, and the chat is the one the
    // user is on, so the current workspace is the one that vanishes (D3(a)).
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    let disk = FakeDisk::holding(&[SESSION]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, true);

    assert!(deleted.removed);
    assert_eq!(
        deleted.view.workspace_notice,
        Some(workspace_gone("repo")),
        "the vanished workspace is named, once"
    );
}

#[test]
fn a_workspace_the_config_holds_stays_and_needs_no_notice() {
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    let mut config = AppConfig::default();
    assert!(config.add_workspace(REPO), "the folder is in the config");
    holder.replace_config(config);
    let disk = FakeDisk::holding(&[SESSION]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, true);

    assert!(deleted.removed);
    assert_eq!(
        deleted.view.workspace_notice, None,
        "an added workspace keeps its row with no chat, so nothing vanished"
    );
}

#[test]
fn a_workspace_that_is_not_the_current_one_goes_away_without_a_notice() {
    // `notes` loses its only chat, but the user is on a chat of `repo`.
    let holder = holder_with(
        vec![session(SESSION, REPO), session(OTHER_SESSION, NOTES)],
        SESSION,
    );
    let disk = FakeDisk::holding(&[OTHER_SESSION]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), OTHER_SESSION, true);

    assert!(deleted.removed);
    assert_eq!(
        deleted.view.workspace_notice, None,
        "the row of a workspace the user is not on simply goes"
    );
}

#[test]
fn a_workspace_that_still_has_a_chat_does_not_go_away() {
    let holder = holder_with(
        vec![session(SESSION, REPO), session(SIBLING, REPO)],
        SESSION,
    );
    let disk = FakeDisk::holding(&[SIBLING]);

    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SIBLING, true);

    assert!(deleted.removed);
    assert_eq!(
        deleted.view.workspace_notice, None,
        "the workspace still holds a chat, so no row vanished"
    );
}

#[test]
fn a_foreground_chat_whose_session_vanished_keeps_its_place_and_reads_as_new() {
    // The refresh the delete asks for heals the list by itself (S3): the index
    // drops the file it no longer finds. The house rule for the foreground chat
    // is unchanged by this slice, and this is it, asserted end to end.
    let holder = holder_with(vec![session(SESSION, REPO)], SESSION);
    holder.replace_config(AppConfig::default());
    let disk = FakeDisk::holding(&[SESSION]);
    let deleted = chat_delete::delete(&holder, &disk, &FakeWork::idle(), SESSION, true);
    assert!(deleted.removed);

    // What the refreshed session list folds in next: no such file.
    let healed = holder.fold_input(Input::SessionsLoaded(Vec::new()));

    assert_eq!(
        healed.foreground_chat,
        Some(ChatId::of_session(SESSION).as_str().to_owned()),
        "the chat the user was on stays the foreground one"
    );
    assert_eq!(
        healed.foreground_title.as_deref(),
        Some(NEW_CHAT_TITLE),
        "with its session gone the header reads as a new chat, which is the existing rule"
    );
}
