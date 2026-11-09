//! Windowless tests of the workspace intents (W3b): a new chat started in the
//! folder of a workspace, the folder table the executor takes it from, and the
//! collapse state the config remembers between runs.
//!
//! A temp directory holds the config file and the folders, so the user's own
//! config and home are never read. The save runs on the writer's own thread, so
//! a test that reads the file back waits for it, bounded.

mod session_tree_seam;
use session_tree_seam::{header, session, shape};

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use gentle_app::{ChatCommand, Input};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::{ChatActivityMark, ChatActivityView, UiIntent};
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::sessions::window::MAX_WINDOW;
use gentle_desk_lib::workspace;
use gentle_runtime::app_config::{self, AppConfig};

/// Every wait of these tests: a save of one small file, never a hang.
const WAIT: Duration = Duration::from_secs(10);

/// How long a table or a file is watched after the change that should not have
/// touched it.
const SETTLE: Duration = Duration::from_millis(100);

/// How many workspaces the concurrency test folds at the same time: half of them
/// in the config, half known only from their sessions.
const WORKSPACES: usize = 8;

/// How many times the concurrency test repeats its race.
const ROUNDS: usize = 20;

#[test]
fn a_new_chat_in_a_workspace_opens_a_new_chat_and_holds_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let holder = AppStateHolder::default();
    let folders = PendingFolders::default();

    let (view, commands) =
        holder.dispatch_commands(UiIntent::NewChatInWorkspace { path: repo.clone() });

    // The new chat of the per-chat model, never the legacy single-child `NewChat`.
    let [ChatCommand::OpenProcess { chat }] = commands.as_slice() else {
        panic!("expected exactly one open, got {commands:?}");
    };
    assert!(
        chat.as_str().starts_with("new-"),
        "a chat that did not exist before, got {}",
        chat.as_str()
    );
    assert_eq!(
        view.foreground_chat.as_deref(),
        Some(chat.as_str()),
        "the new chat is the foreground one"
    );
    assert_eq!(
        view.activity,
        vec![ChatActivityView {
            chat_id: chat.as_str().to_owned(),
            activity: ChatActivityMark::Opening,
        }]
    );

    // What the intent path does before the command runs: the folder waits for
    // exactly the chat the reducer allocated.
    workspace::remember_folder(&folders, &commands, &repo);

    assert_eq!(
        folders.take(chat),
        Some(PathBuf::from(&repo)),
        "the open takes the folder the workspace named"
    );
    assert_eq!(folders.take(chat), None, "a folder is taken once");
    assert!(folders.is_empty(), "no entry waits after its open took it");
}

#[test]
fn an_open_of_another_chat_finds_no_folder_left_behind() {
    let dir = tempfile::tempdir().unwrap();
    let first = folder(dir.path(), "first");
    let second = folder(dir.path(), "second");
    let holder = AppStateHolder::default();
    let folders = PendingFolders::default();

    let (_, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: first.clone(),
    });
    let [ChatCommand::OpenProcess { chat }] = commands.as_slice() else {
        panic!("expected exactly one open, got {commands:?}");
    };
    workspace::remember_folder(&folders, &commands, &first);
    // The first chat's open runs here.
    assert_eq!(folders.take(chat), Some(PathBuf::from(&first)));

    let (_, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: second.clone(),
    });
    let [ChatCommand::OpenProcess { chat: next }] = commands.as_slice() else {
        panic!("expected exactly one open, got {commands:?}");
    };
    workspace::remember_folder(&folders, &commands, &second);

    assert_ne!(next, chat, "a second workspace opens a chat of its own");
    assert_eq!(
        folders.take(next),
        Some(PathBuf::from(&second)),
        "the second chat takes its own folder"
    );
    assert_eq!(
        folders.take(chat),
        None,
        "the first chat's folder was consumed by its own open"
    );
}

#[test]
fn the_plain_new_chat_names_no_folder() {
    // PRESERVE: `OpenNewChat` is unchanged by this slice - it starts the same new
    // chat and names no folder, so the child inherits this process's folder.
    let holder = AppStateHolder::default();
    let folders = PendingFolders::default();

    let (view, commands) = holder.dispatch_commands(UiIntent::OpenNewChat);

    let [ChatCommand::OpenProcess { chat }] = commands.as_slice() else {
        panic!("expected exactly one open, got {commands:?}");
    };
    assert_eq!(view.foreground_chat.as_deref(), Some(chat.as_str()));
    assert_eq!(folders.take(chat), None, "no intent named a folder for it");
}

#[test]
fn a_workspace_that_names_no_folder_starts_nothing() {
    let holder = AppStateHolder::default();
    let folders = PendingFolders::default();
    let before = holder.view();

    let (view, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: "   ".to_owned(),
    });

    assert!(
        commands.is_empty(),
        "no chat is opened for an empty path, got {commands:?}"
    );
    assert_eq!(view, before, "the state is untouched");
    assert!(folders.is_empty());
}

#[test]
fn folding_a_workspace_persists_it_and_the_tree_skips_its_chats() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    app_config::save(&path, &saved).unwrap();

    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.clone())));
    holder.fold_input(Input::SessionsLoaded(vec![session(&repo, 100)]));

    let before = holder.view();
    let page = holder.sessions_window(0, MAX_WINDOW);
    assert_eq!(shape(&page.rows), "WC", "the workspace and its chat");
    assert!(!header(&page.rows[0]).collapsed);

    let after = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });

    assert!(
        after.sessions_revision > before.sessions_revision,
        "the tree moved, so the revision says so"
    );
    let page = holder.sessions_window(0, MAX_WINDOW);
    assert_eq!(shape(&page.rows), "W", "a folded workspace serves no chat");
    let row = header(&page.rows[0]);
    assert!(row.collapsed);
    assert_eq!(
        row.chat_count, 1,
        "folding never changes what the group says about itself"
    );

    // The file, read back the way the app reads it.
    let loaded = wait_for("the fold to reach the config file", || {
        let loaded = app_config::load(&path);
        let folded = loaded
            .config
            .workspace(&repo)
            .is_some_and(|entry| entry.collapsed);
        folded.then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
}

#[test]
fn folding_a_derived_workspace_adds_it_to_the_config() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    // No config file at all: the folder is known only from a session header.
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.clone())));
    holder.fold_input(Input::SessionsLoaded(vec![session(&repo, 100)]));
    assert_eq!(
        holder.config_value().workspaces.len(),
        0,
        "the folder is derived from the history, not added"
    );

    holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });

    assert_eq!(
        holder.config_value().workspaces.len(),
        1,
        "a folded workspace is added, or its state could not be remembered"
    );
    let loaded = wait_for("the added workspace to reach the file", || {
        let loaded = app_config::load(&path);
        (loaded.config.workspaces.len() == 1).then_some(loaded)
    });
    assert_eq!(loaded.config.workspaces[0].path, repo);
    assert!(loaded.config.workspaces[0].collapsed);
    assert_eq!(
        shape(&holder.sessions_window(0, MAX_WINDOW).rows),
        "W",
        "the tree folds it from now on"
    );
}

#[test]
fn two_quick_folds_leave_the_last_value_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    // The file starts FOLDED, so the last fold of the test is a visible change:
    // nothing here can pass because the file never moved.
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    assert!(saved.set_collapsed(&repo, true));
    app_config::save(&path, &saved).unwrap();

    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.clone())));

    holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });
    holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: false,
    });

    // One writer, one order: whatever it wrote in between, the last value is
    // what the file holds, and it stays there.
    wait_for("the last fold to reach the file", || {
        let loaded = app_config::load(&path);
        let unfolded = loaded
            .config
            .workspace(&repo)
            .is_some_and(|entry| !entry.collapsed);
        unfolded.then_some(())
    });
    std::thread::sleep(SETTLE);
    let loaded = app_config::load(&path);
    assert!(
        loaded
            .config
            .workspace(&repo)
            .is_some_and(|entry| !entry.collapsed),
        "the file holds the last fold, got {:?}",
        loaded.config
    );
}

#[test]
fn a_save_that_fails_sets_the_notice_and_the_run_keeps_the_value() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    // The config's parent is a FILE, so no config file can be created there.
    let blocked = dir.path().join("blocked");
    fs::write(&blocked, b"a file, not a folder").unwrap();
    let path = blocked.join("config.json");

    let writer = ConfigWriter::spawn(Some(path.clone()));
    let holder = Arc::new(AppStateHolder::default());
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    holder.replace_config(saved);
    holder.arm_config_saves(writer.clone());
    // The bridge's wiring, in one line: the failure becomes the config notice.
    let holder_for_failure = Arc::clone(&holder);
    writer.on_failure(Arc::new(move |message: String| {
        holder_for_failure.note_config_problem(message);
    }));

    let before = holder.view();
    let after = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });

    assert!(
        after.sessions_revision > before.sessions_revision,
        "the fold is in memory at once"
    );
    let notice = wait_for("the failed save to be reported", || {
        holder.view().config_notice.clone()
    });
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(
        notice.contains("could not be saved"),
        "the reason names the save, got {notice:?}"
    );
    assert!(!path.exists(), "a failed save creates no file");
    assert!(
        header(&holder.sessions_window(0, MAX_WINDOW).rows[0]).collapsed,
        "the run keeps the value it holds in memory"
    );
}

#[test]
fn a_workspace_the_tree_does_not_know_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.clone())));
    holder.fold_input(Input::SessionsLoaded(vec![session(&repo, 100)]));
    let before = holder.view();

    let unknown = text(&dir.path().join("nowhere"));
    let after = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: unknown,
        collapsed: true,
    });
    assert_eq!(after, before, "an unknown folder is not added");

    let after = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: "   ".to_owned(),
        collapsed: true,
    });
    assert_eq!(after, before, "an empty path changes nothing");

    std::thread::sleep(SETTLE);
    assert!(!path.exists(), "nothing was written");
    assert!(holder.config_value().workspaces.is_empty());
}

#[test]
fn a_run_with_no_config_path_keeps_the_fold_in_memory() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let holder = AppStateHolder::default();
    holder.arm_config_saves(ConfigWriter::spawn(None));
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    holder.replace_config(saved);
    let before = holder.view();

    let after = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });

    assert!(after.sessions_revision > before.sessions_revision);
    assert_eq!(
        after.config_notice, None,
        "a run with no config path has nothing to fail at"
    );
    assert!(header(&holder.sessions_window(0, MAX_WINDOW).rows[0]).collapsed);
}

#[test]
fn folds_of_different_workspaces_at_the_same_time_all_survive() {
    // Eight folds released together, four of them on a workspace the config
    // holds and four on one known only from a session header. A read, an edit
    // and a write that are not one step lose each other's change here.
    for round in 0..ROUNDS {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let folders: Vec<String> = (0..WORKSPACES)
            .map(|index| folder(dir.path(), &format!("ws{index}")))
            .collect();
        // Half of the workspaces are in the config file already...
        let mut saved = AppConfig::default();
        for path_text in &folders[..WORKSPACES / 2] {
            assert!(saved.add_workspace(path_text));
        }
        app_config::save(&path, &saved).unwrap();

        let holder = Arc::new(AppStateHolder::with_config(ConfigSnapshot::at(Some(&path))));
        holder.arm_config_saves(ConfigWriter::spawn(Some(path.clone())));
        // ...and the other half only from the history.
        let sessions: Vec<_> = folders[WORKSPACES / 2..]
            .iter()
            .enumerate()
            .map(|(index, cwd)| session(cwd, index as u64 + 1))
            .collect();
        holder.fold_input(Input::SessionsLoaded(sessions));

        let barrier = Arc::new(Barrier::new(WORKSPACES));
        let threads: Vec<_> = folders
            .iter()
            .map(|path_text| {
                let holder = Arc::clone(&holder);
                let barrier = Arc::clone(&barrier);
                let path_text = path_text.clone();
                thread::spawn(move || {
                    barrier.wait();
                    holder.dispatch(UiIntent::SetWorkspaceCollapsed {
                        path: path_text,
                        collapsed: true,
                    });
                })
            })
            .collect();
        for handle in threads {
            handle.join().expect("a fold thread never panics");
        }

        // In memory: every fold landed, whatever order the threads ran in.
        let config = holder.config_value();
        for path_text in &folders {
            assert!(
                config
                    .workspace(path_text)
                    .is_some_and(|entry| entry.collapsed),
                "round {round}: {path_text} lost its fold, holder says {:?}",
                config.workspaces
            );
        }

        // And on disk, once the one writer settled.
        let loaded = wait_for("every fold to reach the file", || {
            let loaded = app_config::load(&path);
            let all = folders.iter().all(|path_text| {
                loaded
                    .config
                    .workspace(path_text)
                    .is_some_and(|entry| entry.collapsed)
            });
            all.then_some(loaded)
        });
        assert_eq!(
            loaded.notice, None,
            "round {round}: the file stayed readable"
        );
        assert_eq!(
            loaded.config.workspaces.len(),
            WORKSPACES,
            "round {round}: one entry per workspace, got {:?}",
            loaded.config.workspaces
        );
    }
}

/// A folder of `dir`, created, as the text the web side sends.
fn folder(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    fs::create_dir_all(&path).unwrap();
    text(&path)
}

/// A path as the text the contract carries it.
fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Some(value) = probe() {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "waited {WAIT:?} for {what} and it never came"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
