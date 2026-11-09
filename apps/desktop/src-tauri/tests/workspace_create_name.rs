//! "Create a new folder" (W3c, Q6), the rules of the NAME and of the PARENT: a
//! name Windows would not accept is refused before any dialog opens, and a
//! parent that is not there is refused instead of built.
//!
//! Windowless: the flow takes a picker seam, so a fake answers with the parent,
//! and every folder lives in a temp directory.

mod workspace_seam;
use workspace_seam::{entries, holder_at, FakePicker, SETTLE};

use std::fs;

use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::workspace_create;

/// The names the rules of `folder_name` refuse: empty and blank, a separator, a
/// `..`, a device (bare and with an extension), a trailing dot, a trailing
/// space, and two of the characters Windows forbids.
const BAD_NAMES: [&str; 12] = [
    "", "   ", "a/b", "a\\b", "..", "a..b", "CON", "nul.txt", "name.", "name ", "bad:name",
    "bad?name",
];

#[test]
fn every_invalid_name_is_refused_before_any_dialog_and_creates_nothing() {
    for name in BAD_NAMES {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("parent");
        fs::create_dir(&parent).unwrap();
        let path = dir.path().join("config.json");
        let holder = holder_at(&path);
        let picker = FakePicker::picking(&parent.to_string_lossy());

        let action =
            workspace_create::create_new(&holder, &picker, &PendingFolders::default(), name);

        let notice = action.view.workspace_notice.clone();
        let notice = notice.unwrap_or_else(|| panic!("{name:?} was accepted, nothing refused"));
        assert!(notice.is_ascii(), "{name:?} gave {notice:?}");
        assert!(!notice.is_empty(), "{name:?} gave an empty reason");
        assert_eq!(
            picker.calls(),
            0,
            "{name:?}: an invalid name never opens the dialog"
        );
        assert!(
            entries(&parent).is_empty(),
            "{name:?}: nothing is created, got {:?}",
            entries(&parent)
        );
        assert!(action.commands.is_empty(), "{name:?}: nothing opens");
        std::thread::sleep(SETTLE);
        assert!(!path.exists(), "{name:?}: no config file is written");
        assert!(
            holder.config_value().workspaces.is_empty(),
            "{name:?}: nothing is added"
        );
    }
}

#[test]
fn a_parent_folder_that_is_not_there_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("gone");
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    let action = workspace_create::create_new(
        &holder,
        &FakePicker::picking(&parent.to_string_lossy()),
        &folders,
        "fresh",
    );

    let notice = action.view.workspace_notice.clone().expect("a refusal");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(
        !parent.exists(),
        "create_dir never makes the parent: the chain is not built"
    );
    assert!(action.commands.is_empty());
    std::thread::sleep(SETTLE);
    assert!(!path.exists());
}

#[test]
fn a_cancelled_parent_pick_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();
    let before = holder.view();

    let action = workspace_create::create_new(&holder, &FakePicker::cancelled(), &folders, "fresh");

    assert_eq!(action.view, before, "the slice is untouched");
    assert!(action.commands.is_empty());
    assert!(folders.is_empty());
    std::thread::sleep(SETTLE);
    assert!(!path.exists(), "no config file is written");
    assert_eq!(
        entries(dir.path()),
        Vec::<String>::new(),
        "nothing is created"
    );
}
