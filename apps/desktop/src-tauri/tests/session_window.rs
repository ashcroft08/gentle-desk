//! Windowless tests of the windowed tree (S3): a temp pi home, a real refresh
//! pass, and the pages read back out of the holder the app keeps. No Tauri app,
//! no window, no real home of the user.
//!
//! The home holds one project folder, so the tree the holder serves is one
//! workspace header followed by its chats: the pages have to tile the chats and
//! count the header, and no page may ever come back longer than the cap.

mod session_seam;
use session_seam::{settled, Home};

use gentle_desk_lib::contract::{SessionRowView, SessionTreeRow};
use gentle_desk_lib::sessions::window::MAX_WINDOW;

/// The chat rows of a page.
fn chats(rows: &[SessionTreeRow]) -> Vec<&SessionRowView> {
    rows.iter()
        .filter_map(|row| match row {
            SessionTreeRow::Chat(chat) => Some(chat),
            SessionTreeRow::Workspace(_) => None,
        })
        .collect()
}

/// The chat ids of a page, in the order it holds them.
fn ids(rows: &[SessionTreeRow]) -> Vec<String> {
    chats(rows).iter().map(|chat| chat.id.clone()).collect()
}

#[test]
fn the_pages_tile_the_tree_and_a_page_past_the_end_is_empty() {
    let home = Home::new();
    home.many(25);
    let (holder, _) = settled(&home);

    let first = holder.sessions_window(0, 11);
    let second = holder.sessions_window(11, 11);
    let third = holder.sessions_window(22, 11);
    let overflow = holder.sessions_window(40, 11);

    assert_eq!(first.total, 26, "25 chats and the one workspace header");
    assert_eq!(second.total, 26);
    assert!(matches!(first.rows[0], SessionTreeRow::Workspace(_)));
    assert_eq!(ids(&first.rows).len(), 10, "the header takes one row");
    assert_eq!(second.rows.len(), 11);
    assert_eq!(third.rows.len(), 4, "the tail of the tree");

    assert_eq!(overflow.total, 26, "a page past the end keeps the total");
    assert!(overflow.rows.is_empty(), "and holds no rows");

    let mut seen = ids(&first.rows);
    seen.extend(ids(&second.rows));
    seen.extend(ids(&third.rows));
    let before = seen.len();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), before, "no chat is served twice");
    assert_eq!(seen.len(), 25, "every chat of the home is served");
}

#[test]
fn a_length_above_the_cap_is_cut_to_two_hundred() {
    let home = Home::new();
    home.many(250);
    let (holder, _) = settled(&home);

    let wide = holder.sessions_window(0, 5_000);

    assert_eq!(wide.total, 251, "250 chats and the header");
    assert_eq!(
        wide.rows.len(),
        MAX_WINDOW as usize,
        "a page, never the whole list"
    );
    assert_eq!(holder.sessions_window(250, 5_000).rows.len(), 1);
}

#[test]
fn a_settled_list_serves_its_chats_under_the_folder_they_were_written_in() {
    let home = Home::new();
    home.session("p", "a.jsonl", "first chat");
    let (holder, _) = settled(&home);

    let page = holder.sessions_window(0, MAX_WINDOW);

    let SessionTreeRow::Workspace(workspace) = &page.rows[0] else {
        panic!("the first row is the workspace header");
    };
    assert_eq!(workspace.chat_count, 1);
    assert_eq!(workspace.name, "p");
    let chats = chats(&page.rows);
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0].title, "first chat");
}
