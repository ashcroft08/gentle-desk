use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_watch::Batch;

use super::*;
use crate::session_refresh::{self, Update};
use crate::startup;

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
    Flow::new(Some(shared), Some(sessions), None, None)
}

/// Runs one full pass on the flow's index, the way the window's worker does,
/// and returns the item that settles it. No transport: the decision is all
/// this module owns.
fn full_pass(flow: &Flow) -> Refreshed {
    let shared = flow.shared().expect("a run with an index");
    let mut last = None;
    session_refresh::work(&Job::Full, &shared, &mut || shared.now(), &mut |update| {
        if let Update::Refreshed(refreshed) = update {
            last = Some(refreshed);
        }
    });
    last.expect("every pass settles")
        .expect("a full pass on a readable home")
}

#[test]
fn the_first_full_pass_starts_one_worker_and_the_next_waits() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(
        flow.on_tick(),
        Next::Run(Job::Full),
        "the pass runs on a worker"
    );
    // One at a time: a second request waits for the first to settle.
    assert_eq!(flow.on_tick(), Next::Idle);
}

#[test]
fn a_finished_full_pass_hands_the_list_over_and_remembers_its_duration() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(flow.on_tick(), Next::Run(Job::Full));

    let refreshed = full_pass(&flow);
    assert_eq!(refreshed.sessions.items().len(), 1);
    let took = refreshed.took;
    let settled = flow.settle(Ok(refreshed));
    let prepared = settled.sessions.expect("a list to show");
    assert_eq!(prepared.items().len(), 1);
    assert_eq!(flow.last_full_pass, took);
    // No watcher runs, so this pass asks the caller to start one.
    assert!(settled.retry_watcher);
    assert!(!flow.is_watching());
}

#[test]
fn a_failed_pass_reports_nothing_and_frees_the_queue() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    let _ = flow.on_tick();
    let settled = flow.settle(Err("unreadable".to_owned()));
    assert!(settled.sessions.is_none());
    assert!(!settled.retry_watcher);
    // The queue is free again: the next request starts at once.
    assert_eq!(flow.on_tick(), Next::Run(Job::Full));
}

#[test]
fn a_batch_of_paths_asks_for_a_targeted_pass_and_a_rescan_for_a_full_one() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(
        flow.on_batch(batch(vec![path("a.jsonl")], false)),
        Next::Run(Job::Paths(vec![path("a.jsonl")]))
    );
    // The targeted pass waits while the first one runs.
    assert!(flow.settle(Err("failed".to_owned())).sessions.is_none());
    assert_eq!(flow.on_batch(batch(Vec::new(), true)), Next::Run(Job::Full));
}

#[test]
fn without_a_watcher_the_list_polls_every_few_seconds() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert!(!flow.is_watching());
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);
    // A settled turn is not covered by any watcher, so it asks for a pass.
    assert_eq!(flow.on_refresh_command(), Next::Run(Job::Full));
}

#[test]
fn a_run_without_a_home_does_nothing_at_all() {
    let mut flow = Flow::new(None, None, None, None);
    assert_eq!(flow.on_tick(), Next::Idle);
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);
    assert!(!flow.is_watching());
    assert!(flow.shared().is_none(), "nothing to flush either");
}

#[test]
fn losing_the_watcher_falls_back_to_a_full_pass() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    assert_eq!(flow.on_watcher_lost(), Next::Run(Job::Full));
    assert!(!flow.is_watching());
}

#[test]
fn the_safety_net_waits_for_the_last_full_pass() {
    let (_dir, sessions) = home();
    let mut flow = flow_without_watcher(sessions);
    flow.last_full_pass = Duration::from_secs(10);
    assert_eq!(flow.tick_delay(), Duration::from_secs(200));
}

/// The case the flow exists for: with a watcher, a settled turn is already
/// covered (the watcher reports the appended file), so it asks for nothing.
#[test]
fn a_settled_turn_still_asks_for_a_pass_with_a_watcher_and_the_safety_net_slows_down() {
    let (_dir, sessions) = home();
    let (watcher, _batches) = startup::try_start_watcher(&sessions).expect("a watcher");
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    let mut flow = Flow::new(Some(shared), Some(sessions), Some(watcher), None);

    assert!(flow.is_watching());
    // A settled turn is the user's own write: the watcher's notice of it can come
    // seconds late (measured on Windows), so the turn end asks for a pass itself.
    assert_eq!(flow.on_refresh_command(), Next::Run(Job::Full));
    // The timer is still the slow safety net while a watcher runs.
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
        Some(shared),
        Some(missing.clone()),
        None,
        Some("the sessions directory does not exist".to_owned()),
    );

    assert_eq!(flow.on_tick(), Next::Run(Job::Full));
    let settled = flow.settle(Ok(full_pass(&flow)));
    assert!(settled.sessions.is_some());
    assert!(
        settled.retry_watcher,
        "every full pass tries the watcher again"
    );
    // What the caller gets for that request: the directory is not there yet.
    flow.watcher_failed("the sessions directory does not exist".to_owned());
    assert!(!flow.is_watching(), "still nothing to watch");
    assert_eq!(flow.tick_delay(), session_schedule::POLL_BASE);

    // The directory appears: the next full pass starts the watcher again.
    fs::create_dir_all(&missing).unwrap();
    assert_eq!(flow.on_tick(), Next::Run(Job::Full));
    let settled = flow.settle(Ok(full_pass(&flow)));
    assert!(settled.sessions.is_some());
    assert!(settled.retry_watcher);
    let (watcher, _batches) = startup::try_start_watcher(&missing).expect("a watcher");
    flow.watcher_started(watcher);
    assert!(flow.is_watching());
    // Events may have been missed before the watcher started: one pass covers it.
    assert_eq!(flow.pump(), Next::Run(Job::Full));
    assert_eq!(flow.tick_delay(), session_schedule::RECONCILE_BASE);
}

/// The bridge takes the watcher out of the flow so it can drop it with no lock
/// held; the flow then polls, exactly as it does after losing the watcher.
#[test]
fn taking_the_watcher_out_leaves_the_list_polling() {
    let (_dir, sessions) = home();
    let (watcher, _batches) = startup::try_start_watcher(&sessions).expect("a watcher");
    let shared = Shared::new(
        Arc::new(Mutex::new(SessionIndex::new(sessions.clone()))),
        None,
    );
    let mut flow = Flow::new(Some(shared), Some(sessions), Some(watcher), None);
    assert!(flow.is_watching());

    let taken = flow.take_watcher();
    assert!(taken.is_some(), "the handle leaves the flow");
    assert!(
        !flow.is_watching(),
        "and no watcher feeds the list any more"
    );
    assert_eq!(
        flow.tick_delay(),
        session_schedule::POLL_BASE,
        "the fast poll"
    );
    // Dropping it here is the point: no lock is held, so its thread joins freely.
    drop(taken);
    assert!(flow.take_watcher().is_none(), "nothing left to take");
    // The fallback pass is the caller's, as after any lost watcher.
    assert_eq!(flow.on_watcher_lost(), Next::Run(Job::Full));
}
