//! Windowless tests of the branch chip's slice of the contract (design/
//! composer-dock S2, R3, R4): the branch the foreground chat's folder is on,
//! read once per working directory by the shell and carried by the slice beside
//! that folder.
//!
//! No git runs here: the cache is seeded with the answer a read would have left,
//! so what is asserted is the slice the web side really receives. The ONE case
//! that needs a real machine - a folder outside any repository - is asserted
//! against `git::read` itself.

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::BranchView;
use gentle_desk_lib::git;

const SESSION_PATH: &str = "/sessions/a.jsonl";
const CWD: &str = r"C:\Users\josep\dev\gentle-desk";

/// One session file of the list, as the first pass reports it.
fn session(cwd: &str) -> SessionItem {
    SessionItem {
        path: SESSION_PATH.to_owned(),
        title: "a chat".to_owned(),
        cwd: cwd.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The branch of a work tree whose root is `path`.
fn branch(name: &str, path: &str) -> BranchView {
    BranchView {
        name: name.to_owned(),
        path: path.to_owned(),
    }
}

/// The holder with that session listed and its chat open.
///
/// The cache is seeded BEFORE the chat opens, so every push the open makes is a
/// cache hit: nothing here waits for a process, and `record` is the very call
/// the shell's branch read ends with.
fn open_in(cwd: &str, answer: Option<BranchView>) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.branches().record(cwd, answer);
    holder.fold_input(Input::SessionsLoaded(vec![session(cwd)]));
    let chat = ChatId::of_session(SESSION_PATH);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat });
    holder
}

#[test]
fn the_open_chat_carries_the_branch_its_folder_is_on() {
    let holder = open_in(CWD, Some(branch("feat/0.7.0-transcript", CWD)));

    let drawn = holder
        .view()
        .foreground_branch
        .expect("a folder in a work tree has a branch");

    assert_eq!(drawn.name, "feat/0.7.0-transcript");
    assert_eq!(drawn.path, CWD);
}

#[test]
fn a_folder_git_had_nothing_to_say_about_carries_no_branch_at_all() {
    // R3: no repository is no chip - not an empty one, not a dash, not a word.
    let holder = open_in(CWD, None);

    assert_eq!(holder.view().foreground_branch, None);
    // The folder is still named: only the branch is absent.
    assert!(holder.view().foreground_folder.is_some());
}

#[test]
fn a_branch_read_from_another_directory_is_not_drawn_on_this_folder() {
    // A read lands after the user moved to another chat: its answer names the
    // folder they left, so it is dropped rather than drawn under the wrong one.
    let holder = open_in(CWD, Some(branch("main", r"C:\Users\josep\dev\other")));

    assert_eq!(holder.view().foreground_branch, None);
}

#[test]
fn a_chat_with_no_listed_session_carries_no_branch() {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::Intent(Intent::OpenNewChat));

    let view = holder.view();

    assert!(view.foreground_chat.is_some());
    assert_eq!(view.foreground_branch, None);
}

#[test]
fn no_open_chat_means_no_branch() {
    let holder = AppStateHolder::default();

    assert_eq!(holder.view().foreground_branch, None);
}

#[test]
fn a_directory_outside_any_repository_answers_none() {
    // The real read: git refuses in a folder no repository holds, and every
    // refusal is `None` - never an error the composer would have to handle. The
    // scratch directory lives outside the worktree and is removed here.
    let scratch = std::env::temp_dir().join(format!("gentle-desk-git-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("failed to create the scratch directory");

    let answer = git::read(&scratch);

    let _ = std::fs::remove_dir(&scratch);
    assert_eq!(answer, None);
}
