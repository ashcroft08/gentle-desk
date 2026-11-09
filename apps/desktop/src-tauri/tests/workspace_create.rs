//! "Create a new folder" (W3c, Q1 option A, Q6), the rules of the TARGET: the
//! app makes `<parent>/<name>` under the folder the user picks and NEVER
//! overwrites anything.
//!
//! Windowless: the flow takes a picker seam, so a fake answers with the parent,
//! and every folder lives in a temp directory.

mod workspace_seam;
use workspace_seam::{entries, holder_at, wait_for, FakePicker, SETTLE};

use std::fs;
use std::path::{Path, PathBuf};

use gentle_app::ChatCommand;
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::workspace_create;
use gentle_runtime::app_config;

#[test]
fn a_new_folder_is_created_and_becomes_a_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let parent = parent_folder(dir.path());
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();
    let picker = FakePicker::picking(&parent.to_string_lossy());

    // A leading space is trimmed; a TRAILING one is refused (see the table of
    // invalid names), because Windows would drop it silently.
    let action = workspace_create::create_new(&holder, &picker, &folders, " fresh");

    let target = parent.join("fresh");
    assert!(target.is_dir(), "the folder is created");
    assert_eq!(
        entries(&parent),
        vec!["fresh".to_owned()],
        "the name is trimmed and nothing else is written"
    );
    // S7: the workspace is added and a new chat opens in it.
    let [ChatCommand::OpenProcess { chat }] = action.commands.as_slice() else {
        panic!("expected exactly one open, got {:?}", action.commands);
    };
    assert_eq!(
        folders.take(chat),
        Some(PathBuf::from(&target)),
        "the open starts in the created folder"
    );
    assert_eq!(action.view.workspace_notice, None);
    let loaded = wait_for("the created workspace to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.workspace(&target.to_string_lossy()).is_some()).then_some(loaded)
    });
    assert_eq!(loaded.notice, None);
    assert_eq!(loaded.config.workspaces.len(), 1);
}

#[test]
fn an_existing_empty_folder_is_accepted_and_nothing_is_written_into_it() {
    let dir = tempfile::tempdir().unwrap();
    let parent = parent_folder(dir.path());
    let target = parent.join("there");
    fs::create_dir_all(&target).unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    let action = workspace_create::create_new(
        &holder,
        &FakePicker::picking(&parent.to_string_lossy()),
        &folders,
        "there",
    );

    assert!(target.is_dir());
    assert!(
        entries(&target).is_empty(),
        "an empty folder stays empty: nothing is overwritten and nothing is added"
    );
    assert_eq!(action.view.workspace_notice, None);
    let [ChatCommand::OpenProcess { chat }] = action.commands.as_slice() else {
        panic!("expected exactly one open, got {:?}", action.commands);
    };
    assert_eq!(folders.take(chat), Some(target.clone()));
    assert!(holder
        .config_value()
        .workspace(&target.to_string_lossy())
        .is_some());
}

#[test]
fn an_existing_folder_with_files_is_refused_and_its_files_are_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let parent = parent_folder(dir.path());
    let target = parent.join("taken");
    fs::create_dir_all(&target).unwrap();
    let kept = target.join("keep.txt");
    fs::write(&kept, b"the user's work").unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    let action = workspace_create::create_new(
        &holder,
        &FakePicker::picking(&parent.to_string_lossy()),
        &folders,
        "taken",
    );

    let notice = action.view.workspace_notice.clone();
    let notice = notice.expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(
        notice.contains("already holds files"),
        "the reason names the folder, got {notice:?}"
    );
    assert_eq!(
        fs::read(&kept).unwrap(),
        b"the user's work",
        "the files of the refused folder are untouched"
    );
    assert_eq!(entries(&target), vec!["keep.txt".to_owned()]);
    assert!(action.commands.is_empty(), "no chat opens on a refusal");
    assert!(folders.is_empty());
    std::thread::sleep(SETTLE);
    assert!(holder.config_value().workspaces.is_empty());
    assert!(!path.exists(), "no config file is written");
}

#[test]
fn a_target_that_is_a_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let parent = parent_folder(dir.path());
    let target = parent.join("taken");
    fs::write(&target, b"a file").unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);

    let action = workspace_create::create_new(
        &holder,
        &FakePicker::picking(&parent.to_string_lossy()),
        &PendingFolders::default(),
        "taken",
    );

    let notice = action.view.workspace_notice.clone().expect("a refusal");
    assert!(notice.contains("file"), "got {notice:?}");
    assert_eq!(
        fs::read(&target).unwrap(),
        b"a file",
        "the file is untouched"
    );
    assert!(action.commands.is_empty());
    std::thread::sleep(SETTLE);
    assert!(!path.exists());
}

/// The `parent` folder of a test's temp dir, created.
fn parent_folder(dir: &Path) -> PathBuf {
    let parent = dir.join("parent");
    fs::create_dir(&parent).unwrap();
    parent
}
