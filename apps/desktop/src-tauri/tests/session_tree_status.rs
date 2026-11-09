//! Windowless tests of whether a workspace can host a new chat, and of WHEN that
//! question is asked: the folder of a workspace is a file-system question, so it
//! is asked for the header rows of the served page only, never for the whole
//! list.

mod session_tree_seam;
use session_tree_seam::{chat, config, folded, header, session};

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use gentle_desk_lib::sessions::tree::Tree;
use gentle_desk_lib::sessions::window::{self, MAX_WINDOW};
use gentle_runtime::app_config::{workspace_status, WorkspaceStatus};

const NOW: u64 = 1_000;

/// The reason the runtime's own rule gives for `path`: the oracle the served
/// header's reason is compared against.
fn reason_of(path: &str) -> String {
    workspace_status(Path::new(path))
        .reason()
        .expect("the fixture is not a usable folder")
        .to_owned()
}

#[test]
fn an_existing_folder_is_available_and_a_gone_one_carries_its_reason() {
    let dir = tempfile::tempdir().unwrap();
    let here = dir.path().to_string_lossy().into_owned();
    let gone = dir.path().join("removed").to_string_lossy().into_owned();
    let tree = Tree::build(1, &config(&[&here, &gone]), &[]);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &window::folder_status);

    let available = header(&page.rows[0]);
    assert!(available.available, "the folder is there");
    assert_eq!(available.reason, None);
    let missing = header(&page.rows[1]);
    assert!(!missing.available, "the folder is gone");
    let reason = missing.reason.as_deref().expect("a reason is served");
    assert_eq!(reason, reason_of(&gone));
    assert!(reason.is_ascii() && !reason.is_empty(), "got {reason:?}");
}

#[test]
fn a_file_where_a_folder_was_is_reported_with_its_own_reason() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"x").unwrap();
    let path = file.to_string_lossy().into_owned();
    let tree = Tree::build(1, &config(&[&path]), &[]);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &window::folder_status);

    let row = header(&page.rows[0]);
    assert!(!row.available);
    assert_eq!(row.reason.as_deref(), Some(reason_of(&path).as_str()));
}

#[test]
fn the_folder_is_asked_only_for_the_headers_the_page_holds() {
    // Three workspaces with one chat each: rows W C W C W C.
    let sessions = [
        session(r"C:\a", 300),
        session(r"C:\b", 200),
        session(r"C:\c", 100),
    ];
    let tree = Tree::build(1, &config(&[]), &sessions);
    let probes = AtomicUsize::new(0);
    let counting = |path: &str| {
        probes.fetch_add(1, Ordering::Relaxed);
        workspace_status(Path::new(path))
    };

    // A page of chats only: no header is served, so no folder is asked.
    let chats = window::window(&tree, 1, 1, NOW, &counting);
    assert_eq!(chat(&chats.rows[0]).id, "sessions/300.jsonl");
    assert_eq!(probes.load(Ordering::Relaxed), 0, "no header, no probe");

    // A page of one header: exactly one folder is asked.
    let one = window::window(&tree, 2, 1, NOW, &counting);
    assert!(one.total > 0);
    assert_eq!(header(&one.rows[0]).path, r"C:\b");
    assert_eq!(probes.load(Ordering::Relaxed), 1, "one header, one probe");

    // The whole tree fits one page: one probe per header, never per chat.
    let all = window::window(&tree, 0, MAX_WINDOW, NOW, &counting);
    assert_eq!(all.rows.len(), 6);
    assert_eq!(
        probes.load(Ordering::Relaxed),
        1 + 3,
        "three headers in all"
    );
}

#[test]
fn a_probe_is_never_asked_for_a_collapsed_group_chat() {
    let sessions = [session(r"C:\folded", 100)];
    let tree = Tree::build(1, &folded(&[r"C:\folded"]), &sessions);
    let probes = AtomicUsize::new(0);
    let counting = |_path: &str| {
        probes.fetch_add(1, Ordering::Relaxed);
        WorkspaceStatus::Available
    };

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &counting);

    assert_eq!(page.rows.len(), 1, "the header of the folded group");
    assert_eq!(probes.load(Ordering::Relaxed), 1);
}
