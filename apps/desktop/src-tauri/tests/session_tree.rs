//! Windowless tests of the flattened workspace tree (S3, D1, D7, D8): one
//! header row per workspace, then the chats of the workspaces that are not
//! collapsed, served a page at a time with the header rows counted in the total.
//!
//! The grouping, the naming and the order are `gentle-runtime`'s
//! (`crates/runtime/src/workspaces.rs`); these tests read the rows the app serves
//! and never re-decide them.

mod session_tree_seam;
use session_tree_seam::{available, chat_ids, config, folded, header, session, shape};

use gentle_app::SessionItem;
use gentle_desk_lib::sessions::tree::Tree;
use gentle_desk_lib::sessions::window::{self, MAX_WINDOW};

/// A pinned clock, so a chat row's age text is the runtime's rule only.
const NOW: u64 = 3 * 60 * 60_000;

fn tree(config: &gentle_runtime::app_config::AppConfig, sessions: &[SessionItem]) -> Tree {
    Tree::build(1, config, sessions)
}

#[test]
fn a_workspace_header_comes_first_and_its_own_chats_follow_it() {
    // Two folders, newest activity first, so the order is decided by the
    // runtime's rule: `later` (300) then `first` (100).
    let sessions = [
        session(r"C:\dev\later", 300),
        session(r"C:\dev\first", 100),
        session(r"C:\dev\later", 200),
    ];
    let tree = tree(&config(&[]), &sessions);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &available);

    assert_eq!(
        shape(&page.rows),
        "WCCWC",
        "header, its chats, header, chat"
    );
    let labels: Vec<&str> = page
        .rows
        .iter()
        .filter_map(|row| match row {
            gentle_desk_lib::contract::SessionTreeRow::Workspace(workspace) => {
                Some(workspace.label.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(labels, ["later", "first"]);
    assert_eq!(
        chat_ids(&page.rows),
        [
            "sessions/300.jsonl".to_owned(),
            "sessions/200.jsonl".to_owned(),
            "sessions/100.jsonl".to_owned(),
        ],
        "each chat sits under its own folder"
    );
}

#[test]
fn a_collapsed_workspace_shows_its_header_and_none_of_its_chats() {
    let sessions = [
        session(r"C:\dev\shown", 300),
        session(r"C:\dev\folded", 200),
    ];
    let tree = tree(&folded(&[r"C:\dev\folded"]), &sessions);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &available);

    assert_eq!(shape(&page.rows), "WCW", "the folded header has no chat");
    let folded = header(&page.rows[2]);
    assert!(folded.collapsed);
    assert_eq!(
        folded.chat_count, 1,
        "the header still says how many it holds"
    );
    assert_eq!(
        chat_ids(&page.rows),
        ["sessions/300.jsonl".to_owned()],
        "only the expanded folder serves chats"
    );
}

#[test]
fn the_total_counts_the_workspace_headers() {
    let sessions = [
        session(r"C:\a", 300),
        session(r"C:\b", 200),
        session(r"C:\a", 100),
    ];
    let tree = tree(&config(&[]), &sessions);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &available);

    assert_eq!(page.total, 5, "two headers and three chats");
    assert_eq!(page.rows.len(), 5);
}

#[test]
fn a_page_that_starts_on_a_chat_and_ends_on_the_next_header_tiles_the_tree() {
    let sessions = [
        session(r"C:\a", 300),
        session(r"C:\b", 200),
        session(r"C:\a", 100),
    ];
    let tree = tree(&config(&[]), &sessions);

    // Row 0 is a's header, 1 and 2 are its chats, 3 is b's header, 4 its chat.
    let boundary = window::window(&tree, 1, 3, NOW, &available);

    assert_eq!(shape(&boundary.rows), "CCW", "a page crosses the boundary");
    assert_eq!(boundary.total, 5, "the total is the whole tree");
    assert_eq!(
        chat_ids(&boundary.rows),
        [
            "sessions/300.jsonl".to_owned(),
            "sessions/100.jsonl".to_owned()
        ]
    );
}
