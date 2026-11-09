//! The iced side of the refresh flow: a decision becomes a `Task` (a pass on
//! its own stream, or nothing at all) and a watcher the core started again ends
//! up in the subscription's source. The decisions themselves are tested in
//! `gentle_runtime::refresh_flow`.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gentle_backend::session_index::SessionIndex;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::work;

use super::*;

/// A home with one session file, so a full pass has something to find.
fn home() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let sessions = dir.path().join("sessions");
    let project = sessions.join("--C--p--");
    fs::create_dir_all(&project).unwrap();
    let file = project.join("s.jsonl");
    fs::write(
        &file,
        concat!(
            r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#,
            "\n",
            r#"{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{"role":"user","content":"hola"}}"#,
            "\n"
        ),
    )
    .unwrap();
    (dir, sessions)
}

fn flow_without_watcher(shared: Shared, sessions: PathBuf) -> Flow {
    Flow::new(
        Some(shared),
        Some(sessions),
        None,
        WatchSource::default(),
        None,
    )
}

/// Runs one full pass on the index, the way the window's worker does, and
/// returns the item that settles it.
fn full_pass(shared: &Shared) -> Refreshed {
    let mut last = None;
    work(&Job::Full, shared, &mut || shared.now(), &mut |update| {
        if let Update::Refreshed(result) = update {
            last = Some(result);
        }
    });
    last.expect("every pass settles")
        .expect("a full pass on a readable home")
}

/// The subscription identity of the source, which is what iced rebuilds a
/// stream for: a new watcher must be a new identity.
fn identity(source: &WatchSource) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn a_decision_to_run_a_pass_becomes_one_worker_and_waiting_becomes_no_task() {
    let (_dir, sessions) = home();
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    let mut flow = flow_without_watcher(shared, sessions);
    assert_eq!(flow.on_tick().units(), 1, "the pass runs on a worker");
    // One at a time: a second request waits, which is no task at all.
    assert_eq!(flow.on_tick().units(), 0);
}

#[test]
fn a_run_without_a_home_asks_iced_for_nothing() {
    let mut flow = Flow::new(None, None, None, WatchSource::default(), None);
    assert_eq!(flow.on_tick().units(), 0);
    assert_eq!(flow.pump().units(), 0);
    let rescan = Batch {
        paths: Vec::new(),
        rescan: true,
    };
    assert_eq!(flow.on_batch(rescan).units(), 0);
    // Nothing to persist either.
    flow.flush();
}

#[test]
fn the_watcher_a_settled_pass_started_reports_on_a_new_subscription() {
    let (_dir, sessions) = home();
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    let mut flow = flow_without_watcher(shared.clone(), sessions);
    let before = identity(flow.watch());

    assert_eq!(flow.on_tick().units(), 1);
    let prepared = flow.settle(Ok(full_pass(&shared))).expect("a list");
    assert_eq!(prepared.items().len(), 1);
    // The full pass started the watcher: its batches went to the source, and a
    // new identity is what makes iced read them instead of the old stream.
    assert_ne!(identity(flow.watch()), before);
    // The pass that covers what the watcher may have missed is queued.
    assert_eq!(flow.pump().units(), 1);
}
