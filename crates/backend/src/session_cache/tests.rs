use super::*;
use std::path::PathBuf;

use gentle_core::session::SessionScanner;

use crate::session_snapshot::{FileSnapshot, IndexSnapshot};

const HEADER: &str = r#"{"type":"session","version":3,"id":"s","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;

fn sample(sessions_dir: &Path, name: &str) -> IndexSnapshot {
    let mut scanner = SessionScanner::new();
    scanner.push_line(HEADER);
    IndexSnapshot {
        sessions_dir: sessions_dir.to_path_buf(),
        files: vec![FileSnapshot {
            path: sessions_dir.join(format!("--C--p--/{name}.jsonl")),
            size: 42,
            mtime_secs: 1_700_000_000,
            mtime_nanos: 123_456_789,
            parsed_len: 42,
            scanner,
        }],
    }
}

fn cache_path(dir: &Path) -> PathBuf {
    dir.join("gentle-desk").join("session-index-v1.json")
}

#[test]
fn a_missing_cache_file_is_no_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(load(&cache_path(dir.path())), None);
}

#[test]
fn garbage_is_no_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"{not json").unwrap();
    assert_eq!(load(&path), None);
}

#[test]
fn another_version_is_no_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let document = serde_json::json!({
        "version": CACHE_VERSION + 1,
        "snapshot": sample(dir.path(), "s"),
    });
    fs::write(&path, document.to_string()).unwrap();
    assert_eq!(load(&path), None);
}

#[test]
fn save_then_load_round_trips_the_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    let snapshot = sample(dir.path(), "s");
    save(&path, &snapshot).unwrap();
    assert_eq!(load(&path), Some(snapshot));
}

#[test]
fn save_replaces_the_previous_file_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = cache_path(dir.path());
    save(&path, &sample(dir.path(), "old")).unwrap();
    let current = sample(dir.path(), "new");
    save(&path, &current).unwrap();

    let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(leftovers, ["session-index-v1.json"]);
    assert_eq!(load(&path), Some(current));
}

#[test]
fn save_creates_the_missing_directories() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a").join("b").join("session-index-v1.json");
    let snapshot = sample(dir.path(), "s");
    save(&path, &snapshot).unwrap();
    assert_eq!(load(&path), Some(snapshot));
}
