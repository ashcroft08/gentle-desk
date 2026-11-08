use super::*;
use iced::futures::executor::block_on;
use iced::futures::FutureExt;
use std::fs;

use crate::sessions;
use gentle_backend::session_cache;

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;
const USER: &str = r#"{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{"role":"user","content":"hola"}}"#;

fn home_with_one_session() -> (tempfile::TempDir, SharedIndex) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("sessions").join("--C--p--");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("s1.jsonl"), format!("{HEADER}\n{USER}\n")).unwrap();
    let index = Arc::new(Mutex::new(SessionIndex::new(dir.path().join("sessions"))));
    (dir, index)
}

fn cache_file(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("data").join("session-index-v1.json")
}

#[test]
fn the_refreshed_list_has_one_item_per_session() {
    let (_dir, index) = home_with_one_session();
    let items = block_on(refresh(index, None)).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "hola");
    assert_eq!(items[0].message_count, 1);
}

#[test]
fn a_home_without_sessions_is_an_empty_list_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let index = Arc::new(Mutex::new(SessionIndex::new(dir.path().join("sessions"))));
    assert_eq!(block_on(refresh(index, None)).unwrap(), vec![]);
}

#[test]
fn a_second_refresh_sees_a_session_added_in_between() {
    let (dir, index) = home_with_one_session();
    assert_eq!(block_on(refresh(index.clone(), None)).unwrap().len(), 1);
    let project = dir.path().join("sessions").join("--C--q--");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("s2.jsonl"), format!("{HEADER}\n")).unwrap();
    assert_eq!(block_on(refresh(index, None)).unwrap().len(), 2);
}

/// The point of the module: waiting for a busy index must not block the caller.
#[test]
fn the_caller_is_not_blocked_while_the_index_is_busy() {
    let (_dir, index) = home_with_one_session();
    let busy = index.lock().unwrap(); // someone else is using the index
    let mut pending = Box::pin(refresh(index.clone(), None));
    // Polling once returns at once: the work is on another thread, waiting for the lock.
    assert!(pending.as_mut().now_or_never().is_none());
    drop(busy);
    let items = block_on(pending).unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn a_refresh_that_changed_something_writes_a_reusable_cache() {
    let (dir, index) = home_with_one_session();
    let cache = cache_file(&dir);
    let items = block_on(refresh(index.clone(), Some(cache.clone()))).unwrap();

    let snapshot = session_cache::load(&cache).expect("the refresh wrote a cache");
    assert_eq!(snapshot.files.len(), 1);

    // The next run starts cold, restores the cache and reads nothing.
    let mut restored = SessionIndex::new(dir.path().join("sessions"));
    assert!(restored.refresh().unwrap().bytes_read > 0);
    let mut warm = SessionIndex::new(dir.path().join("sessions"));
    warm.restore(snapshot);
    assert_eq!(warm.refresh().unwrap().bytes_read, 0);
    let warm_items: Vec<_> = warm.entries().iter().map(sessions::to_item).collect();
    assert_eq!(warm_items, items);
}

#[test]
fn a_refresh_that_changed_nothing_leaves_the_cache_alone() {
    let (dir, index) = home_with_one_session();
    let cache = cache_file(&dir);
    block_on(refresh(index.clone(), Some(cache.clone()))).unwrap();

    // Anything the refresh wrote would replace this marker.
    fs::write(&cache, "untouched").unwrap();
    block_on(refresh(index.clone(), Some(cache.clone()))).unwrap();
    assert_eq!(fs::read_to_string(&cache).unwrap(), "untouched");
}

#[test]
fn no_cache_path_writes_nothing() {
    let (dir, index) = home_with_one_session();
    block_on(refresh(index, None)).unwrap();
    assert!(!dir.path().join("data").exists());
}

#[test]
fn an_unwritable_cache_location_does_not_fail_the_refresh() {
    let (dir, index) = home_with_one_session();
    // A file where the cache directory should be: the save cannot win.
    let blocked = dir.path().join("data");
    fs::write(&blocked, "not a directory").unwrap();
    let items = block_on(refresh(index, Some(blocked.join("session-index-v1.json")))).unwrap();
    assert_eq!(items.len(), 1);
}
