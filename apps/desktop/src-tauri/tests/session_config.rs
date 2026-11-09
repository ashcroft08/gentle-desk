//! Windowless tests of the app config at the bridge: the file is read once at
//! startup, a damaged file is kept aside and reported through the view state, and
//! replacing the config moves the revision so the rows the web side holds go
//! stale.
//!
//! A temp directory holds the config file; the user's own config is never read.

mod session_tree_seam;
use session_tree_seam::{config, header};

use std::fs;

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::ConfigSnapshot;
use gentle_desk_lib::contract::SessionTreeRow;
use gentle_desk_lib::sessions::window::MAX_WINDOW;

#[test]
fn a_damaged_config_file_is_kept_aside_and_the_notice_reaches_the_view_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, b"{ this is not the document this app writes").unwrap();

    let snapshot = ConfigSnapshot::at(Some(&path));
    let holder = AppStateHolder::with_config(snapshot);

    let notice = holder
        .view()
        .config_notice
        .expect("a damaged file is reported");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(!notice.is_empty());
    assert!(
        !path.exists(),
        "the damaged file is moved aside, never overwritten"
    );
    let aside: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        aside.iter().any(|name| name.contains("corrupt")),
        "the file was kept, got {aside:?}"
    );
}

#[test]
fn a_config_file_that_reads_carries_no_notice_and_its_workspaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let folder = dir.path().join("repo");
    fs::create_dir_all(&folder).unwrap();
    let folder_text = folder.to_string_lossy().into_owned();
    let mut saved = config(&[&folder_text]);
    assert!(saved.add_workspace(&dir.path().to_string_lossy()));
    gentle_runtime::app_config::save(&path, &saved).unwrap();

    let snapshot = ConfigSnapshot::at(Some(&path));
    assert_eq!(snapshot.notice, None, "a readable file says nothing");
    assert_eq!(snapshot.config.workspaces.len(), 2);

    let holder = AppStateHolder::with_config(snapshot);
    assert_eq!(holder.view().config_notice, None);
    let page = holder.sessions_window(0, MAX_WINDOW);
    assert!(!page.rows.is_empty(), "the added folders are workspaces");
}

#[test]
fn a_run_with_no_config_path_holds_an_empty_config_without_a_notice() {
    let snapshot = ConfigSnapshot::at(None);

    assert_eq!(snapshot, ConfigSnapshot::default());
    assert_eq!(snapshot.notice, None);
    assert!(snapshot.config.workspaces.is_empty());

    let holder = AppStateHolder::with_config(snapshot);
    assert_eq!(holder.view().config_notice, None);
    assert_eq!(holder.sessions_window(0, MAX_WINDOW).total, 0);
}

#[test]
fn replacing_the_config_bumps_the_revision_and_changes_the_rows() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("repo");
    fs::create_dir_all(&folder).unwrap();
    let holder = AppStateHolder::default();
    let before = holder.view().sessions_revision;
    assert_eq!(holder.sessions_window(0, MAX_WINDOW).total, 0);

    let folder_text = folder.to_string_lossy().into_owned();
    holder.replace_config(config(&[&folder_text]));

    let after = holder.view();
    assert!(
        after.sessions_revision > before,
        "the config moved and the revision says so"
    );
    let page = holder.sessions_window(0, MAX_WINDOW);
    assert_eq!(page.total, 1, "the new workspace is a header row");
    assert!(matches!(page.rows[0], SessionTreeRow::Workspace(_)));

    // A second change moves it again: the revision never stands still.
    holder.replace_config(config(&[]));
    assert!(holder.view().sessions_revision > after.sessions_revision);
    assert_eq!(holder.sessions_window(0, MAX_WINDOW).total, 0);
}

#[test]
fn the_rows_of_a_config_workspace_are_served_with_their_folder_checked() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("repo");
    fs::create_dir_all(&folder).unwrap();
    let folder_text = folder.to_string_lossy().into_owned();
    let holder = AppStateHolder::with_config(ConfigSnapshot {
        config: config(&[&folder_text]),
        notice: None,
    });

    let page = holder.sessions_window(0, MAX_WINDOW);

    let row = header(&page.rows[0]);
    assert_eq!(row.path, folder.to_string_lossy());
    assert_eq!(row.chat_count, 0);
    assert!(row.available, "a folder that exists can host a chat");
}

#[test]
fn a_config_notice_moves_no_revision_and_no_list_state() {
    // The notice is a slice of its own: a run that starts with a damaged config
    // file still reads, refreshes and dispatches as it did.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, b"not json").unwrap();
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    let before = holder.view();

    let after = holder.dispatch(gentle_desk_lib::contract::UiIntent::NewChat);

    assert_eq!(
        after.sessions_revision, before.sessions_revision,
        "a new chat does not move the list"
    );
    assert_eq!(
        after.config_notice, before.config_notice,
        "the notice stays"
    );
    assert_eq!(after.list_status, before.list_status);
    assert!(!path.exists(), "the damaged file was kept aside at startup");
}
