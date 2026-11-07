use super::*;
use iced::futures::executor::block_on;
use iced::futures::FutureExt;
use std::fs;

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

#[test]
fn the_refreshed_list_has_one_item_per_session() {
    let (_dir, index) = home_with_one_session();
    let items = block_on(refresh(index)).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "hola");
    assert_eq!(items[0].message_count, 1);
}

#[test]
fn a_home_without_sessions_is_an_empty_list_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let index = Arc::new(Mutex::new(SessionIndex::new(dir.path().join("sessions"))));
    assert_eq!(block_on(refresh(index)).unwrap(), vec![]);
}

#[test]
fn a_second_refresh_sees_a_session_added_in_between() {
    let (dir, index) = home_with_one_session();
    assert_eq!(block_on(refresh(index.clone())).unwrap().len(), 1);
    let project = dir.path().join("sessions").join("--C--q--");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("s2.jsonl"), format!("{HEADER}\n")).unwrap();
    assert_eq!(block_on(refresh(index)).unwrap().len(), 2);
}

/// The point of the module: waiting for a busy index must not block the caller.
#[test]
fn the_caller_is_not_blocked_while_the_index_is_busy() {
    let (_dir, index) = home_with_one_session();
    let busy = index.lock().unwrap(); // someone else is using the index
    let mut pending = Box::pin(refresh(index.clone()));
    // Polling once returns at once: the work is on another thread, waiting for the lock.
    assert!(pending.as_mut().now_or_never().is_none());
    drop(busy);
    let items = block_on(pending).unwrap();
    assert_eq!(items.len(), 1);
}
