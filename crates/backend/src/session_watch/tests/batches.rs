//! What real file changes produce: files in a temporary directory.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn a_new_session_file_is_reported_with_its_path() {
    let (watcher, batches, home) = watched();
    let file = project(home.path(), "--home-me--").join("chat.jsonl");
    write(&file);
    assert!(
        wait_for(&batches, home.path(), &file),
        "the created file was never reported"
    );
    drop(watcher);
}

#[test]
fn an_append_to_a_session_file_is_reported() {
    let (watcher, batches, home) = watched();
    let file = project(home.path(), "--home-me--").join("chat.jsonl");
    write(&file);
    assert!(
        wait_for(&batches, home.path(), &file),
        "the create was lost"
    );
    let mut append = fs::OpenOptions::new()
        .append(true)
        .open(&file)
        .expect("the session file");
    append.write_all(b"{\"type\":\"message\"}\n").unwrap();
    assert!(
        wait_for(&batches, home.path(), &file),
        "the append was never reported"
    );
    drop(watcher);
}

#[test]
fn a_removed_session_file_is_reported() {
    let (watcher, batches, home) = watched();
    let file = project(home.path(), "--home-me--").join("chat.jsonl");
    write(&file);
    assert!(
        wait_for(&batches, home.path(), &file),
        "the create was lost"
    );
    fs::remove_file(&file).expect("remove the session file");
    assert!(
        wait_for(&batches, home.path(), &file),
        "the removal was never reported"
    );
    drop(watcher);
}

#[test]
fn a_file_in_a_project_created_after_the_watch_started_is_reported() {
    let (watcher, batches, home) = watched();
    let file = project(home.path(), "--home-new--").join("chat.jsonl");
    write(&file);
    assert!(
        wait_for(&batches, home.path(), &file),
        "the file in the new project directory was never reported"
    );
    drop(watcher);
}

#[test]
fn a_burst_of_two_hundred_files_is_fully_reported() {
    let (watcher, batches, home) = watched();
    let dir = project(home.path(), "--home-me--");
    let files: Vec<PathBuf> = (0..200)
        .map(|index| dir.join(format!("chat-{index:03}.jsonl")))
        .collect();
    for file in &files {
        write(file);
    }
    let mut seen = BTreeSet::new();
    let mut rescan = false;
    let deadline = Instant::now() + WAIT;
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let batch = match batches.recv_timeout(left) {
            Ok(batch) => batch,
            Err(_) => break,
        };
        rescan |= batch.rescan;
        for path in &batch.paths {
            if let Some(file) = files
                .iter()
                .find(|file| same_place(home.path(), path, file))
            {
                seen.insert(file.clone());
            }
        }
        if rescan || seen.len() == files.len() {
            break;
        }
    }
    // A full pass is the documented answer to a burst too large to list: then
    // every file appears through the rescan instead of through its own path.
    assert!(
        rescan || seen.len() == files.len(),
        "{} of {} created files were never reported",
        files.len() - seen.len(),
        files.len()
    );
    drop(watcher);
}

#[test]
fn files_that_cannot_be_sessions_produce_no_batch() {
    let home = tempfile::tempdir().expect("a temporary home");
    // The project directory exists before the watch starts: only the two files
    // below are new, and neither may ask for work.
    let dir = project(home.path(), "--home-me--");
    let (watcher, batches) =
        SessionWatcher::start(home.path()).expect("the watcher starts on a real directory");
    write(&dir.join("notes.txt"));
    write(&home.path().join("at-the-root.jsonl"));
    match batches.recv_timeout(Duration::from_secs(2)) {
        Err(RecvTimeoutError::Timeout) => {}
        Ok(batch) => panic!("a batch was reported for files that are not sessions: {batch:?}"),
        Err(RecvTimeoutError::Disconnected) => panic!("the watcher stopped on its own"),
    }
    drop(watcher);
}

#[test]
fn a_batch_only_ever_lists_sorted_distinct_session_paths() {
    let (watcher, batches, home) = watched();
    let dir = project(home.path(), "--home-me--");
    write(&dir.join("b.jsonl"));
    write(&dir.join("a.jsonl"));
    write(&dir.join("notes.txt"));
    let sessions = [dir.join("a.jsonl"), dir.join("b.jsonl")];
    let mut seen = BTreeSet::new();
    let mut counted = 0;
    let deadline = Instant::now() + WAIT;
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let batch = match batches.recv_timeout(left) {
            Ok(batch) => batch,
            Err(_) => break,
        };
        counted += 1;
        assert!(
            batch.paths.windows(2).all(|pair| pair[0] < pair[1]),
            "batch paths are not sorted and distinct: {:?}",
            batch.paths
        );
        for path in &batch.paths {
            assert!(
                path.to_string_lossy().ends_with(".jsonl"),
                "a file that cannot be a session was listed: {path:?}"
            );
            if let Some(file) = sessions
                .iter()
                .find(|file| same_place(home.path(), path, file))
            {
                seen.insert(file.clone());
            }
        }
        if seen.len() == sessions.len() {
            break;
        }
    }
    assert!(counted > 0, "no batch was reported for two session files");
    assert_eq!(seen.len(), sessions.len(), "not both files were reported");
    drop(watcher);
}
