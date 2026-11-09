//! Windowless tests of the tree cache: the flattened tree is built once per
//! revision, so a scroll of a 50 000-chat home does not regroup the whole list
//! on every page.

mod session_tree_seam;
use session_tree_seam::{config, session};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use gentle_desk_lib::sessions::cache::TreeCache;
use gentle_desk_lib::sessions::tree::Tree;

#[test]
fn the_tree_of_one_revision_is_built_once_and_served_again() {
    let config = config(&[r"C:\dev\p"]);
    let sessions = [session(r"C:\dev\p", 300), session(r"C:\dev\p", 200)];
    let cache = TreeCache::default();
    let builds = AtomicUsize::new(0);
    let build = || {
        builds.fetch_add(1, Ordering::Relaxed);
        Tree::build(7, &config, &sessions)
    };

    let first = cache.get_or_build(7, build);
    assert_eq!(first.len(), 3, "a header and its two chats");
    assert_eq!(builds.load(Ordering::Relaxed), 1);

    // The same revision again: the cached tree, and no second pass over the
    // session list.
    let second = cache.get_or_build(7, build);
    assert_eq!(builds.load(Ordering::Relaxed), 1, "no rebuild for a page");
    assert!(Arc::ptr_eq(&first, &second), "the very same tree");

    // A new revision is a new tree: the list or the config changed.
    let third = cache.get_or_build(8, || {
        builds.fetch_add(1, Ordering::Relaxed);
        Tree::build(8, &config, &sessions)
    });
    assert_eq!(builds.load(Ordering::Relaxed), 2);
    assert_eq!(third.revision(), 8);
    assert!(!Arc::ptr_eq(&second, &third));
}

#[test]
fn a_tree_with_no_rows_is_empty_and_a_page_of_it_has_nothing() {
    let cache = TreeCache::default();
    let tree = cache.get_or_build(1, || Tree::build(1, &config(&[]), &[]));

    assert!(tree.is_empty());
    assert!(tree.page(0, 10).is_empty());
}
