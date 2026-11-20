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

    // The first fold that names the folder starts the read. What that fold's own
    // slice says about the branch is not asserted: the read runs on a worker, so
    // it may already have recorded its answer by the time the slice is built.
    // What matters is that the answer arrives on the push, checked below.
    assert!(
        opens(&holder, &cwd)
            .into_iter()
            .any(|view| view.foreground_folder.is_some()),
        "the open names the folder of the chat's working directory"
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
        // One counter per call: two tests run in parallel and a shared name
        // would make the second `git init` see the first's half-built .git.
        static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "gentle-desk-git-{tag}-{}-{seq}",
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
            // Isolated from the machine: no signing, and no hooks from a global
            // `core.hooksPath` (an empty value names no hook directory).
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=",
            "-c",
            "user.name=gentle-desk tests",
            "-c",
            "user.email=tests@gentle-desk.invalid",
            "commit",
            "-q",
            "--allow-empty",
            "--no-verify",
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

/// The open through the CONTRACT'S OWN intent, the way the web side asks: the
/// path where the open's refresh lives (`AppStateHolder::dispatch_commands`).
fn dispatch_open(holder: &AppStateHolder, cwd: &str) -> ViewState {
    use gentle_desk_lib::contract::UiIntent;
    holder.fold_input(Input::SessionsLoaded(vec![session(cwd)]));
    // The chat id the WEB sends is the session's raw path: `chat_for` prefixes
    // it, so handing it the id again would double the prefix and open a chat
    // no row names.
    holder
        .dispatch_commands(UiIntent::OpenChat {
            chat_id: SESSION_PATH.to_owned(),
        })
        .0
}

#[test]
fn opening_the_chat_again_asks_git_again() {
    // composer-dock L12: the branch is read once per folder per RUN, and an
    // open drops that answer so a branch changed outside this app - the
    // terminal - is seen without a restart.
    let repo = repo_on("feat/one");
    let cwd = repo.cwd();
    let holder = Arc::new(AppStateHolder::default());

    let (pushed, wait) = mpsc::channel::<ViewState>();
    let weak: Weak<AppStateHolder> = Arc::downgrade(&holder);
    holder.arm_branch_push(Arc::new(move || {
        if let Some(holder) = weak.upgrade() {
            let _ = pushed.send(holder.view());
        }
    }));

    let first = dispatch_open(&holder, &cwd);
    assert_eq!(
        first.foreground_branch.as_ref().map(|b| b.name.clone()),
        None,
        "the first open answers before the read lands"
    );
    eprintln!(
        "DBG lookup={:?} folder={:?}",
        holder.branches().lookup(&cwd),
        first.foreground_folder.clone()
    );
    let again = holder.fold_input(Input::SessionsLoaded(vec![session(&cwd)]));
    eprintln!("DBG2 refold folder={:?}", again.foreground_folder.clone());
    eprintln!(
        "DBG3 fg={:?} total={}",
        first.foreground_chat.clone(),
        first.total_sessions
    );
    let read = wait
        .recv_timeout(Duration::from_secs(60))
        .expect("the first read never pushed");
    assert_eq!(
        read.foreground_branch.as_ref().map(|b| b.name.clone()),
        Some("feat/one".to_owned()),
        "the first open's answer is the branch the repo was on"
    );

    // The branch changes OUTSIDE the app, the way a terminal would, and the
    // user comes BACK to the chat: opening it again is the moment they look.
    // A branch that EXISTS: the read answers None for an unborn one by design
    // (fbffd19), so the second branch carries its own commit.
    git_in(repo.path(), &["branch", "feat/two"]);
    git_in(
        repo.path(),
        &["symbolic-ref", "HEAD", "refs/heads/feat/two"],
    );

    let second = dispatch_open(&holder, &cwd);
    assert_eq!(
        second.foreground_branch.as_ref().map(|b| b.name.clone()),
        None,
        "the fresh read is answered on its push, not in the open's slice"
    );
    let re_read = wait
        .recv_timeout(Duration::from_secs(60))
        .expect("the second read never pushed");
    assert_eq!(
        re_read.foreground_branch.as_ref().map(|b| b.name.clone()),
        Some("feat/two".to_owned()),
        "the second open sees the branch the terminal changed"
    );
}

#[test]
fn a_new_chat_names_the_workspace_folder_until_its_session_exists() {
    // composer-dock L12: a chat opened in a workspace lives there from the
    // start, so the folder chip names the workspace's folder while the chat's
    // own session row does not exist, and the branch is read from it.
    use gentle_desk_lib::contract::UiIntent;

    let holder = AppStateHolder::default();
    let (view, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: r"C:\dev\wsp".to_owned(),
    });
    assert_eq!(commands.len(), 1, "the workspace opens one chat");
    let folder = view
        .foreground_folder
        .expect("the new chat names the workspace folder");
    assert_eq!(folder.name, "wsp");
    assert_eq!(folder.path, r"C:\dev\wsp");

    // A chat that closed takes the hint with it: absent, never a placeholder.
    let chat = match &commands[0] {
        gentle_app::ChatCommand::OpenProcess { chat } => chat.clone(),
        other => panic!("the workspace opens a chat, got {other:?}"),
    };
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });
    let view = holder.view();
    assert!(
        view.foreground_folder.is_none(),
        "a closed chat names no folder"
    );
}

#[test]
fn the_session_row_wins_over_the_workspace_hint() {
    // The hint is only a stand-in: once the chat's session row is listed, the
    // folder the row carries is the one the chip names.
    use gentle_desk_lib::contract::UiIntent;

    let holder = AppStateHolder::default();
    let (_, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: r"C:\dev\wsp".to_owned(),
    });
    let chat = match &commands[0] {
        gentle_app::ChatCommand::OpenProcess { chat } => chat.clone(),
        other => panic!("the workspace opens a chat, got {other:?}"),
    };

    // The session the child created: its cwd is the chat's own truth.
    holder.fold_input(Input::ChatRekeyed {
        from: chat.clone(),
        to: ChatId::of_session(chat.as_str()),
    });
    holder.fold_input(Input::SessionsLoaded(vec![SessionItem {
        path: chat.as_str().to_owned(),
        title: "the new chat".to_owned(),
        cwd: r"C:\dev\real".to_owned(),
        modified_ms: 0,
        message_count: 1,
    }]));
    let view = holder.view();
    let folder = view
        .foreground_folder
        .expect("the session row names the folder");
    assert_eq!(folder.path, r"C:\dev\real");
}
