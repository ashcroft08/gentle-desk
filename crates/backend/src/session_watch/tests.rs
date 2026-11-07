//! The real watcher: its start errors and its lifecycle.
//!
//! These tests assert what a batch *contains*, never when exactly it arrives:
//! event delivery speed belongs to the operating system, not to this crate.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use super::*;

/// How long a batch may take to arrive. Everything here is local and small; the
/// bound exists so a lost event fails the test instead of hanging it.
const WAIT: Duration = Duration::from_secs(10);

/// A home whose sessions directory is watched, plus its temporary directory.
fn watched() -> (SessionWatcher, Receiver<Batch>, TempDir) {
    let home = tempfile::tempdir().expect("a temporary home");
    let (watcher, batches) =
        SessionWatcher::start(home.path()).expect("the watcher starts on a real directory");
    (watcher, batches, home)
}

/// One project directory inside the sessions directory.
fn project(home: &Path, name: &str) -> PathBuf {
    let dir = home.join(name);
    fs::create_dir_all(&dir).expect("a project directory");
    dir
}

fn write(path: &Path) {
    fs::write(path, "{\"type\":\"message\"}\n").expect("a session file");
}

/// Two paths name the same place, however the watcher spelled it.
fn same_place(sessions: &Path, left: &Path, right: &Path) -> bool {
    let left = paths::below(sessions, left);
    left.is_some() && left == paths::below(sessions, right)
}

fn mentions(batch: &Batch, sessions: &Path, file: &Path) -> bool {
    batch
        .paths
        .iter()
        .any(|path| same_place(sessions, path, file))
}

/// Reads batches until one satisfies `done`, or `deadline` passes.
fn until(batches: &Receiver<Batch>, deadline: Instant, done: impl Fn(&Batch) -> bool) -> bool {
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        match batches.recv_timeout(left) {
            Ok(batch) => {
                if done(&batch) {
                    return true;
                }
            }
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return false,
        }
    }
    false
}

/// Waits for a batch that lists `file` or asks for a full pass.
fn wait_for(batches: &Receiver<Batch>, sessions: &Path, file: &Path) -> bool {
    until(batches, Instant::now() + WAIT, |batch| {
        batch.rescan || mentions(batch, sessions, file)
    })
}

mod batches;

#[test]
fn a_missing_sessions_directory_is_an_error() {
    let home = tempfile::tempdir().expect("a temporary home");
    let missing = home.path().join("no-such-sessions");
    let error = SessionWatcher::start(&missing).expect_err("a missing directory is an error");
    assert_eq!(error, WatchError::Missing(missing.clone()));
    let message = error.to_string();
    assert!(
        message.contains(&missing.display().to_string()) && message.contains("does not exist"),
        "unhelpful message: {message}"
    );
}

#[test]
fn a_file_instead_of_a_directory_is_an_error() {
    let home = tempfile::tempdir().expect("a temporary home");
    let file = home.path().join("sessions.txt");
    write(&file);
    let error = SessionWatcher::start(&file).expect_err("a plain file is an error");
    assert_eq!(error, WatchError::NotDirectory(file.clone()));
    let message = error.to_string();
    assert!(
        message.contains(&file.display().to_string()) && message.contains("not a directory"),
        "unhelpful message: {message}"
    );
}

#[test]
fn dropping_the_watcher_closes_the_channel_and_stops_reporting() {
    let (watcher, batches, home) = watched();
    drop(watcher);
    match batches.recv_timeout(WAIT) {
        Err(RecvTimeoutError::Disconnected) => {}
        Ok(batch) => panic!("a batch arrived after the watcher was dropped: {batch:?}"),
        Err(RecvTimeoutError::Timeout) => panic!("the watcher thread outlived its watcher"),
    }
    let file = project(home.path(), "--home-me--").join("chat.jsonl");
    write(&file);
    match batches.recv_timeout(Duration::from_secs(2)) {
        Err(RecvTimeoutError::Disconnected | RecvTimeoutError::Timeout) => {}
        Ok(batch) => panic!("a dropped watcher still reported {batch:?}"),
    }
}

#[test]
fn dropping_the_consumer_before_the_watcher_is_harmless() {
    let home = tempfile::tempdir().expect("a temporary home");
    let (watcher, batches) =
        SessionWatcher::start(home.path()).expect("the watcher starts on a real directory");
    drop(batches);
    // Events now have nowhere to go: the thread stops, and dropping the
    // watcher must not panic or wait forever.
    write(&project(home.path(), "--home-me--").join("chat.jsonl"));
    std::thread::sleep(Duration::from_millis(300));
    drop(watcher);
}
