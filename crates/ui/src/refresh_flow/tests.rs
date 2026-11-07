use super::*;
use iced::futures::executor::block_on;
use std::fs;
use std::sync::{Arc, Mutex};

use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_watch::Batch;

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}

fn batch(paths: Vec<PathBuf>, rescan: bool) -> Batch {
    Batch { paths, rescan }
}

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

/// A flow that knows where the sessions are but has no watcher: the polling
/// fallback, which is the case a headless test can drive without a window.
fn flow_without_watcher(sessions: PathBuf) -> Flow {
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    Flow::new(
        Some(shared),
        Some(sessions),
        None,
        WatchSource::default(),
        None,
    )
}

#[test]
fn the_first_full_pass_produces_the_list_and_starts_one_worker() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(flow.on_tick().units(), 1, "the pass runs on a worker");
    // One at a time: a second request waits for the first to settle.
    assert_eq!(flow.on_tick().units(), 0);
}

#[test]
fn a_finished_full_pass_hands_the_list_over_and_remembers_its_duration() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert!(flow.on_tick().units() > 0);

    let shared = flow.shared.clone().expect("a run with an index");
    let refreshed = block_on(session_refresh::run(
        crate::refresh_queue::Job::Full,
        shared,
    ))
    .unwrap();
    assert_eq!(refreshed.sessions.items().len(), 1);
    let took = refreshed.took;
    let prepared = flow.settle(Ok(refreshed)).expect("a list to show");
    assert_eq!(prepared.items().len(), 1);
    assert_eq!(flow.last_full_pass, took);
}

#[test]
fn a_failed_pass_reports_nothing_and_frees_the_queue() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    let _ = flow.on_tick();
    assert_eq!(flow.settle(Err("unreadable".to_owned())), None);
    // The queue is free again: the next request starts at once.
    assert_eq!(flow.on_tick().units(), 1);
}

#[test]
fn a_batch_of_paths_asks_for_a_targeted_pass_and_a_rescan_for_a_full_one() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert!(flow.on_batch(batch(vec![path("a.jsonl")], false)).units() > 0);
    // The targeted pass waits while the first one runs.
    assert_eq!(flow.settle(Err("failed".to_owned())), None);
    assert!(flow.on_batch(batch(Vec::new(), true)).units() > 0);
}

#[test]
fn without_a_watcher_the_list_polls_and_with_one_it_only_reconciles() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert!(!flow.is_watching());
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);
    // A settled turn is not covered by any watcher, so it asks for a pass.
    assert_eq!(flow.on_refresh_command().units(), 1);
}

#[test]
fn a_run_without_a_home_does_nothing_at_all() {
    let mut flow = Flow::new(None, None, None, WatchSource::default(), None);
    assert_eq!(flow.on_tick().units(), 0);
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);
    assert!(!flow.is_watching());
    flow.flush();
}

#[test]
fn losing_the_watcher_falls_back_to_a_full_pass() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(flow.on_watcher_lost().units(), 1);
    assert!(!flow.is_watching());
}

#[test]
fn the_safety_net_waits_for_the_last_full_pass() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions.clone());
    flow.last_full_pass = Duration::from_secs(10);
    assert_eq!(flow.tick_delay(), Duration::from_secs(200));
}

/// The case the flow exists for: with a watcher, a settled turn is already
/// covered (the watcher reports the appended file), so it asks for nothing.
#[test]
fn a_running_watcher_covers_a_settled_turn_and_slows_the_safety_net_down() {
    let (_dir, sessions) = home();
    let (watcher, batches) = backend::try_start_watcher(&sessions).expect("a watcher");
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    let mut watch = WatchSource::default();
    watch.fill(batches);
    let mut flow = Flow::new(Some(shared), Some(sessions), Some(watcher), watch, None);

    assert!(flow.is_watching());
    assert_eq!(flow.on_refresh_command().units(), 0);
    // Nothing was requested, so there is no pass to settle.
    assert_eq!(flow.tick_delay(), session_schedule::RECONCILE_BASE);
}

/// S4: a watcher that cannot start leaves the list to polling, and every full
/// pass tries again until one starts; the pass that covers the gap is queued.
#[test]
fn a_watcher_is_retried_on_each_full_pass_and_the_restarted_one_asks_for_a_pass() {
    let (_dir, _sessions) = home();
    // The sessions directory the run expects does not exist yet.
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("sessions");
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(missing.clone()))),
        None,
    );
    let mut flow = Flow::new(
        Some(shared.clone()),
        Some(missing.clone()),
        None,
        WatchSource::default(),
        Some("the sessions directory does not exist".to_owned()),
    );

    let full = || {
        let shared = shared.clone();
        async move { session_refresh::run(crate::refresh_queue::Job::Full, shared).await }
    };
    assert_eq!(flow.on_tick().units(), 1);
    assert!(flow.settle(block_on(full())).is_some());
    assert!(!flow.is_watching(), "still nothing to watch");
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);

    // The directory appears: the next full pass starts the watcher again.
    fs::create_dir_all(&missing).unwrap();
    assert_eq!(flow.on_tick().units(), 1);
    assert!(flow.settle(block_on(full())).is_some());
    assert!(flow.is_watching());
    // Events may have been missed before the watcher started: one pass covers it.
    assert!(flow.pump().units() > 0);
    assert_eq!(flow.tick_delay(), session_schedule::RECONCILE_BASE);
}
