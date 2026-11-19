//! Windowless tests of the branch chip's slice of the contract (design/
//! composer-dock S2, R3, R4): the branch the foreground chat's folder is on,
//! read once per working directory by the shell and carried by the slice beside
//! that folder.
//!
//! No git runs for the seeded cases: the cache is seeded with the answer a read
//! would have left, so what is asserted is the slice the web side really
//! receives. The cases that need a real machine run a real `git` in a scratch
//! repository of their own: the one that drives the read end to end (R3-002), the
//! repository with no commit yet (R3-001), and the folder outside any repository
//! (R3-003), which checks its own precondition instead of assuming where the
//! system temp directory lives.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Weak};
use std::time::Duration;

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{BranchView, ViewState};
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
    opens(&holder, cwd);
    holder
}

/// The holder with that session listed and its chat open, with the cache left
/// alone: the first fold that names the folder claims the real read and the shell
/// runs it on its own thread (R3-002), so this returns the slice every one of
/// those folds pushed, in the order it pushed them.
fn opens(holder: &AppStateHolder, cwd: &str) -> Vec<ViewState> {
    let chat = ChatId::of_session(SESSION_PATH);
    vec![
        holder.fold_input(Input::SessionsLoaded(vec![session(cwd)])),
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone()))),
        holder.fold_input(Input::ChatOpened { chat }),
    ]
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
    // refusal is `None` - never an error the composer would have to handle.
    //
    // Where the system temp directory lives is not this test's business (R3-003):
    // if TEMP points inside a checkout, the folder is not outside a repository at
    // all and this assertion would be about something else, so the precondition
    // is checked and named instead of assumed.
    let scratch = Scratch::new("outside");
    if let Err(reason) = outside_any_repo(scratch.path()) {
        eprintln!("skipped a_directory_outside_any_repository_answers_none: {reason}");
        return;
    }

    assert_eq!(git::read(scratch.path()), None);
}

#[test]
fn a_repository_with_no_commit_yet_carries_no_branch_at_all() {
    // R3-001: `--abbrev-ref HEAD` answers `HEAD` for an unborn branch exactly as
    // it does for a detached HEAD. The honest answer there is no chip - the
    // branch exists as a name and as nothing else, no commit has ever backed it -
    // and reaching it must not depend on a `--short HEAD` read that can only
    // fail in a repository with no commit.
    let repo = Scratch::new("unborn");
    git_in(repo.path(), &["init", "-q"]);
    let named = git_in(repo.path(), &["symbolic-ref", "--quiet", "--short", "HEAD"]);
    assert!(
        !named.trim().is_empty(),
        "this repository must really be on an unborn branch for the test to mean anything"
    );

    assert_eq!(git::read(repo.path()), None);
}

#[test]
fn the_branch_of_a_real_repository_arrives_on_the_push_its_read_makes() {
    // R3-002: the real path, end to end - `Lookup::Missing` from the open's own
    // push, the read on the shell's worker thread, `record`, and the push that
    // carries the answer. Nothing is seeded, so no other test covers this.
    let repo = repo_on("feat/chip-read");
    let cwd = repo.cwd();

    let holder = Arc::new(AppStateHolder::default());
    let (pushed, wait) = mpsc::channel::<ViewState>();
    let weak: Weak<AppStateHolder> = Arc::downgrade(&holder);
    holder.arm_branch_push(Arc::new(move || {
        if let Some(holder) = weak.upgrade() {
            let _ = pushed.send(holder.view());
        }
    }));

    // The first fold that names the folder answers from a cache that has never
    // heard of it, so its slice carries no branch and the read starts there.
    let claimed = opens(&holder, &cwd)
        .into_iter()
        .find(|view| view.foreground_folder.is_some())
        .expect("the open names the folder of the chat's working directory");
    assert_eq!(
        claimed.foreground_branch, None,
        "nothing is known about the branch before the read answers"
    );

    // The answer arrives on the push the read itself makes: no later fold, no
    // second ask, and not never.
    let late = wait
        .recv_timeout(Duration::from_secs(60))
        .expect("the branch read never pushed its answer");
    let drawn = late
        .foreground_branch
        .expect("a folder in a work tree has a branch once the read answers");
    assert_eq!(drawn.name, "feat/chip-read");
    assert_eq!(drawn.path, cwd);

    // And the answer is kept: the next read of the slice is a cache hit, not
    // another read.
    assert_eq!(holder.view().foreground_branch, Some(drawn));
}

/// A scratch directory of this test's own under the system temp directory,
/// removed when the test ends, repository inside it included.
///
/// Named for the test and the process, so two tests never share one and a run
/// never reuses what a previous run left.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gentle-desk-git-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("failed to create the scratch directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// The same directory as the text a session row carries: the read's answer
    /// and the folder chip are compared for equality, so the state is handed
    /// exactly the string `git::read` will echo back.
    fn cwd(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Runs `git` in `dir` and answers its stdout, failing loudly when git refuses:
/// a scratch repository the test could not build is a broken test, not a result.
fn git_in(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("the real-git tests need git on PATH");
    assert!(
        output.status.success(),
        "git {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// A scratch repository whose HEAD is on `branch` and whose one commit backs it:
/// a work tree git really answers for, with a branch it really names.
fn repo_on(branch: &str) -> Scratch {
    let repo = Scratch::new("repo");
    git_in(repo.path(), &["init", "-q"]);
    // The branch is named rather than taken from `init.defaultBranch`, so the
    // name asserted above is the name git was told to make.
    git_in(
        repo.path(),
        &["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")],
    );
    git_in(
        repo.path(),
        &[
            "-c",
            "user.name=gentle-desk tests",
            "-c",
            "user.email=tests@gentle-desk.invalid",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "one",
        ],
    );
    repo
}

/// Whether `dir` is provably outside every git work tree, decided without asking
/// git: no `.git` - a directory, a file, whatever a linked worktree or a submodule
/// leaves - sits between it and the filesystem root, and nothing in this
/// environment points git at a repository from outside. The reason it is not, so
/// that a skip can name it.
fn outside_any_repo(dir: &Path) -> Result<(), String> {
    if std::env::var_os("GIT_DIR").is_some() || std::env::var_os("GIT_WORK_TREE").is_some() {
        return Err(
            "GIT_DIR or GIT_WORK_TREE is set, so git would answer for a repository this directory is not in"
                .to_owned(),
        );
    }
    match dir.ancestors().find(|at| at.join(".git").exists()) {
        Some(repo) => Err(format!(
            "{} is inside the repository at {}",
            dir.display(),
            repo.display()
        )),
        None => Ok(()),
    }
}
