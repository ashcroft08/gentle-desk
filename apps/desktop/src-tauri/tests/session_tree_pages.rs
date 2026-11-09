//! Windowless tests of what a page of the flattened tree holds when the page is
//! not the whole tree: a page past the end, a length above the cap, and a
//! workspace that has no chat at all.

mod session_tree_seam;
use session_tree_seam::{available, chat, chat_ids, config, header, session, shape};

use gentle_desk_lib::sessions::tree::Tree;
use gentle_desk_lib::sessions::window::{self, MAX_WINDOW};

/// A pinned clock, so a chat row's age text is the runtime's rule only.
const NOW: u64 = 3 * 60 * 60_000;

fn tree(
    config: &gentle_runtime::app_config::AppConfig,
    sessions: &[gentle_app::SessionItem],
) -> Tree {
    Tree::build(1, config, sessions)
}

#[test]
fn a_page_beyond_the_total_keeps_the_total_and_holds_no_rows() {
    let sessions = [session(r"C:\a", 300)];
    let tree = tree(&config(&[]), &sessions);

    let page = window::window(&tree, 40, 10, NOW, &available);

    assert_eq!(page.total, 2, "the total of the whole tree");
    assert!(page.rows.is_empty(), "and no row past its end");
}

#[test]
fn a_length_above_the_cap_is_cut_to_two_hundred_rows() {
    let mut config = config(&[]);
    let mut sessions = Vec::new();
    for folder in 0..60 {
        let path = format!(r"C:\dev\f{folder}");
        assert!(config.add_workspace(&path));
        for chat_index in 0..4 {
            sessions.push(session(&path, (folder * 10 + chat_index) as u64));
        }
    }
    let tree = tree(&config, &sessions);
    let total = tree.len();
    assert_eq!(total, 60 + 240, "60 headers and 240 chats");

    let page = window::window(&tree, 0, 5_000, NOW, &available);

    assert_eq!(page.total as usize, total);
    assert_eq!(
        page.rows.len(),
        MAX_WINDOW as usize,
        "a page, never the tree"
    );
}

#[test]
fn a_workspace_with_no_chat_is_a_header_with_a_zero_count() {
    let sessions = [session(r"C:\has-chats", 300)];
    let tree = tree(&config(&[r"C:\empty"]), &sessions);

    let page = window::window(&tree, 0, MAX_WINDOW, NOW, &available);

    assert_eq!(
        shape(&page.rows),
        "WCW",
        "the folder with no chat comes last"
    );
    let empty = header(&page.rows[2]);
    assert_eq!(empty.chat_count, 0);
    assert_eq!(empty.path, r"C:\empty");
    assert_eq!(shape(&page.rows[..2]), "WC");
    assert_eq!(chat(&page.rows[1]).id, "sessions/300.jsonl");
    assert_eq!(
        chat_ids(&page.rows),
        ["sessions/300.jsonl".to_owned()],
        "a folder with no chat serves no chat row"
    );
}
