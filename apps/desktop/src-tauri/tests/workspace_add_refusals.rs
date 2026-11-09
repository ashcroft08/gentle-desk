//! "Choose an existing folder" (W3c), the paths that REFUSE: a pick that is not
//! an existing folder changes nothing and says why (Q6), and a refusal never
//! outlives the next action that works.
//!
//! Windowless: the flow takes a picker seam, so a fake answers instead of a
//! human, and the config file lives in a temp directory.

mod workspace_seam;
use workspace_seam::{entries, holder_at, text, FakePicker, SETTLE};

use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::workspace_add;

#[test]
fn a_picked_path_that_is_a_file_is_refused_with_a_message() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a-file");
    std::fs::write(&file, b"not a folder").unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();
    let before = holder.view();

    let action = workspace_add::add_existing(&holder, &FakePicker::picking(&text(&file)), &folders);

    let notice = action.view.workspace_notice.clone();
    let notice = notice.expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(
        notice.contains("file"),
        "the reason names the file, got {notice:?}"
    );
    assert_eq!(
        action.view.sessions_revision, before.sessions_revision,
        "a refusal moves no revision"
    );
    assert!(
        action.view.foreground_chat.is_none(),
        "no chat opens on a refusal"
    );
    assert!(action.commands.is_empty(), "nothing opens");
    std::thread::sleep(SETTLE);
    assert!(
        holder.config_value().workspaces.is_empty(),
        "nothing is added"
    );
    assert!(!path.exists(), "no config file is written");
}

#[test]
fn a_picked_path_that_is_not_there_is_refused_with_a_message() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone");
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    let action = workspace_add::add_existing(&holder, &FakePicker::picking(&text(&gone)), &folders);

    let notice = action.view.workspace_notice.clone();
    let notice = notice.expect("a refusal says why");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(
        notice.contains("no longer exists"),
        "the reason names the missing folder, got {notice:?}"
    );
    assert!(action.commands.is_empty(), "nothing opens");
    assert!(holder.config_value().workspaces.is_empty());
    std::thread::sleep(SETTLE);
    assert!(!path.exists(), "no config file is written");
}

#[test]
fn the_next_workspace_action_that_works_clears_the_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let repo = workspace_seam::folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let folders = PendingFolders::default();

    let gone = text(&dir.path().join("gone"));
    let refused = workspace_add::add_existing(&holder, &FakePicker::picking(&gone), &folders);
    assert!(refused.view.workspace_notice.is_some(), "refused first");

    let action = workspace_add::add_existing(&holder, &FakePicker::picking(&repo), &folders);

    assert_eq!(
        action.view.workspace_notice, None,
        "the action that worked has nothing to report"
    );
    assert!(holder.config_value().workspace(&repo).is_some());
}

#[test]
fn the_workspace_notice_is_absent_until_something_is_refused() {
    // PRESERVE: `get_view_state` answers the same slice it answered before this
    // slice, plus the one new field, which starts empty.
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_at(&dir.path().join("config.json"));

    assert_eq!(holder.view().workspace_notice, None);
    assert_eq!(holder.view().config_notice, None);
    assert_eq!(
        entries(dir.path()),
        Vec::<String>::new(),
        "reading the slice writes nothing"
    );
}
