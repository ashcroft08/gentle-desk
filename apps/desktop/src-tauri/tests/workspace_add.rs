//! "Choose an existing folder" (W3c, Q1 option A), the paths that WORK: a pick
//! becomes a workspace and a new chat opens there (S7).
//!
//! Windowless: the flow takes a picker seam, so a fake answers instead of a
//! human, and the config file lives in a temp directory. Every assertion is an
//! effect the user can see: the slice, the config file read back, the tree, and
//! the chat that opens in the workspace.

mod workspace_seam;
use workspace_seam::{folder, holder_at, wait_for, FakePicker};

use std::path::PathBuf;

use gentle_app::ChatCommand;
use gentle_desk_lib::contract::SessionTreeRow;
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::sessions::window::MAX_WINDOW;
use gentle_desk_lib::workspace_add;
use gentle_runtime::app_config::{self, AppConfig};

#[test]
fn a_cancelled_pick_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();
    let before = holder.view();
    let picker = FakePicker::cancelled();

    let action = workspace_add::add_existing(&holder, &picker, &folders);

    assert_eq!(picker.calls(), 1, "the one dialog of the menu item");
    assert_eq!(action.view, before, "the slice is untouched");
    assert!(
        action.commands.is_empty(),
        "no chat is opened, got {:?}",
        action.commands
    );
    assert!(folders.is_empty(), "no folder waits for an open");
    assert_eq!(
        action.view.sessions_revision, before.sessions_revision,
        "no revision moves"
    );
    std::thread::sleep(workspace_seam::SETTLE);
    assert!(!path.exists(), "no config file is written");
}

#[test]
fn a_picked_folder_is_added_and_a_new_chat_opens_there() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();
    let before = holder.view();

    let action = workspace_add::add_existing(&holder, &FakePicker::picking(&repo), &folders);

    // S7: the workspace is added AND a new chat opens in it.
    let [ChatCommand::OpenProcess { chat }] = action.commands.as_slice() else {
        panic!("expected exactly one open, got {:?}", action.commands);
    };
    assert_eq!(
        action.view.foreground_chat.as_deref(),
        Some(chat.as_str()),
        "the new chat is the foreground one"
    );
    assert_eq!(
        folders.take(chat),
        Some(PathBuf::from(&repo)),
        "the open starts in the folder that was added"
    );
    assert!(folders.is_empty());
    assert!(
        action.view.sessions_revision > before.sessions_revision,
        "the tree moved, so the row is there"
    );
    assert_eq!(action.view.workspace_notice, None, "nothing was refused");

    // The row the sidebar draws comes from the config.
    assert_eq!(
        header_path(&holder.sessions_window(0, MAX_WINDOW).rows),
        Some(repo.clone()),
        "the added workspace is a header row"
    );

    // The file, read back the way the app reads it.
    let loaded = wait_for("the added workspace to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.workspace(&repo).is_some()).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.workspaces.len(), 1);
    assert!(
        !loaded.config.workspaces[0].collapsed,
        "an added workspace is expanded"
    );
}

#[test]
fn a_known_folder_is_added_once_and_still_opens_a_chat() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    // The spelling differs (a trailing separator); the folder is the same one.
    let picked = format!("{repo}\\");
    let action = workspace_add::add_existing(&holder, &FakePicker::picking(&picked), &folders);

    assert_eq!(
        holder.config_value().workspaces.len(),
        1,
        "a known folder is one workspace, got {:?}",
        holder.config_value().workspaces
    );
    assert_eq!(
        holder.config_value().workspaces[0].path,
        repo,
        "the config keeps the canonical spelling"
    );
    let [ChatCommand::OpenProcess { chat }] = action.commands.as_slice() else {
        panic!("expected exactly one open, got {:?}", action.commands);
    };
    assert_eq!(
        folders.take(chat),
        Some(PathBuf::from(&repo)),
        "the open takes the canonical spelling"
    );
    let loaded = app_config::load(&path);
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "no duplicate reached the file"
    );
}

/// The path of the first workspace header row, if the page starts with one.
fn header_path(rows: &[SessionTreeRow]) -> Option<String> {
    match rows.first()? {
        SessionTreeRow::Workspace(row) => Some(row.path.clone()),
        SessionTreeRow::Chat(_) => None,
    }
}
