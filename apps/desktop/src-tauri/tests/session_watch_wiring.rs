//! Windowless tests of the watcher wiring (T8b): what the flow asks for, and
//! what each of the threads around it does about it.
//!
//! The debouncer itself is `gentle_backend`'s (`session_watch::debounce`, with
//! its own tests): here a test feeds the batch the debouncer would have handed
//! over, so the rules of THIS slice are deterministic and need no file notice to
//! arrive in time. The real `notify` watcher end to end is
//! `session_watch_live.rs`.
//!
//! One test per rule: a batch is one pass, a burst is one pass, a change during
//! a pass is one follow-up, a lost watcher falls back to polling and retries,
//! the safety net fires on its own, the threads stop, and a run with no home
//! does nothing at all.

mod session_seam;
mod watch_seam;

use std::time::Duration;

use gentle_backend::session_schedule;
use gentle_backend::session_watch::Batch;
use watch_seam::{homeless, session_file, wired, wired_gated, SETTLE};

/// A safety net that never fires: only the batch under test may run a pass.
const NEVER: Duration = Duration::from_secs(600);

/// A safety net of a test, short enough to see it fire and long enough that a
/// pass or two fit between two firings.
const OFTEN: Duration = Duration::from_millis(20);

/// The deadline a close gives its thread stops here.
const CLOSE: Duration = Duration::from_secs(5);

#[test]
fn one_batch_of_one_changed_file_runs_exactly_one_pass_and_shows_it() {
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    let wired = wired(&home, NEVER);
    wired.service.boot();
    wired.wait_passes(1);
    assert_eq!(wired.source().starts(), 1, "boot starts one watcher");
    wired.wait_total(1);
    let before = wired.passes();

    // The terminal wrote a session file: the watcher reports it as one batch.
    home.session("p", "b.jsonl", "written by the terminal");
    wired
        .source()
        .batches()
        .send(Batch {
            paths: vec![session_file(&home, "p", "b.jsonl")],
            rescan: false,
        })
        .expect("the batch thread takes a batch");

    wired.wait_passes(before + 1);
    wired.wait_total(2);
    // Exactly one: nothing else runs while the test watches.
    std::thread::sleep(SETTLE);
    assert_eq!(wired.passes(), before + 1, "one batch, one pass");
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_burst_coalesced_into_one_batch_is_one_pass() {
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    let wired = wired(&home, NEVER);
    wired.service.boot();
    wired.wait_passes(1);
    wired.wait_total(1);
    let before = wired.passes();

    // The debouncer's rule is the backend's; this is the one batch it hands over
    // for a burst that never went quiet.
    let mut paths = Vec::new();
    for index in 0..40 {
        let name = format!("burst{index}.jsonl");
        home.session("p", &name, "written by the terminal");
        paths.push(session_file(&home, "p", &name));
    }
    wired
        .source()
        .batches()
        .send(Batch {
            paths,
            rescan: false,
        })
        .expect("the batch thread takes a batch");

    wired.wait_passes(before + 1);
    wired.wait_total(41);
    std::thread::sleep(SETTLE);
    assert_eq!(wired.passes(), before + 1, "a burst is one pass");
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_change_during_a_pass_queues_exactly_one_follow_up() {
    let home = session_seam::Home::new();
    let (gate, wired) = wired_gated(&home, NEVER);
    wired.service.boot();
    // The first pass holds the index.
    gate.enter();
    let file = session_file(&home, "p", "a.jsonl");
    for _ in 0..3 {
        wired
            .source()
            .batches()
            .send(Batch {
                paths: vec![file.clone()],
                rescan: false,
            })
            .expect("the batch thread takes a batch");
    }
    std::thread::sleep(SETTLE);
    assert_eq!(
        wired.passes(),
        1,
        "nothing starts while a pass holds the index"
    );

    // The pass settles: the three changes are one queued pass, not three.
    gate.release();
    gate.enter();
    std::thread::sleep(SETTLE);
    assert_eq!(wired.passes(), 2, "one follow-up pass, not one per change");

    gate.release();
    std::thread::sleep(SETTLE);
    assert_eq!(wired.passes(), 2, "and nothing after it");
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_lost_watcher_falls_back_to_polling_and_retries() {
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    // The pass is gated so the test holds the flow in the state it wants to see:
    // while the boot pass runs, a watcher is up; once the watch is lost, the
    // cadence must be the fast one until a watcher starts again.
    let (gate, wired) = wired_gated(&home, NEVER);
    wired.service.boot();
    gate.enter();
    // While a watcher runs, the timer is the slow reconciliation pass.
    wired.wait_delay(0, true, session_schedule::RECONCILE_BASE);
    assert_eq!(wired.source().starts(), 1, "boot starts one watcher");

    // The watch ends on its own, as it does when the platform watcher dies.
    wired.source().lose();
    // With no watcher the list polls on the shipped 5 s cadence.
    wired.wait_delay(0, false, session_schedule::POLL_BASE);
    let seen = wired.delays().len();

    // The pass settles: the fallback's full pass runs, retries the watcher, and
    // the watcher that started asks for the one pass that covers what it could
    // not have seen.
    gate.release();
    gate.enter();
    wired.wait_delay(seen, true, session_schedule::RECONCILE_BASE);
    assert_eq!(
        wired.source().starts(),
        2,
        "the fallback retried the watcher"
    );
    std::thread::sleep(SETTLE);
    assert_eq!(
        wired.passes(),
        2,
        "the fallback and its cover, nothing more"
    );
    gate.release();
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn the_safety_net_fires_a_pass_when_nothing_else_does() {
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    let wired = wired(&home, OFTEN);
    // No watcher: the safety net is the only source of a pass after the boot.
    wired.source().refuse("no watcher in this test");
    wired.service.boot();
    wired.wait_passes(1);
    // Two more, and nothing but the timer asked for them.
    wired.wait_passes(3);
    assert!(
        wired.source().starts() >= 2,
        "every full pass retries the watcher while none runs, the flow's own rule"
    );
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn the_close_stops_the_watcher_and_the_timer_within_its_deadline() {
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    let wired = wired(&home, OFTEN);
    wired.service.boot();
    // The timer is firing and a watcher is up: both threads are live.
    wired.wait_passes(2);
    wired.wait_starts(1);

    let started = std::time::Instant::now();
    assert!(wired.service.stop(CLOSE), "every thread ended in time");
    assert!(
        started.elapsed() < CLOSE,
        "the stop waited out its whole deadline"
    );

    // Nothing ticks and nothing batches any more: the threads are gone, not
    // merely told to go.
    let stopped = wired.passes();
    let _ = wired.source().batches().send(Batch {
        paths: vec![session_file(&home, "p", "a.jsonl")],
        rescan: false,
    });
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(wired.passes(), stopped, "a thread outlived the close");
    // A second stop has nothing left to wait for.
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_home_with_no_sessions_folder_is_an_empty_list_that_keeps_polling() {
    // No session was ever written, so the directory a watcher needs is not
    // there: the pass still settles an empty COMPLETE list (the rule the list
    // slice already had, never a false "no chats" while loading), the watcher
    // cannot start, and the polling fallback keeps retrying it.
    let home = session_seam::Home::new();
    let wired = wired(&home, OFTEN);
    wired
        .source()
        .refuse("the sessions directory does not exist");
    wired.service.boot();
    wired.wait_total(0);
    wired.wait_starts(2);
    wired.wait_passes(2);
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_run_without_a_home_runs_nothing_at_all() {
    let wired = homeless(OFTEN);
    wired.service.boot();
    // Long enough for the timer to have fired several times.
    std::thread::sleep(Duration::from_millis(150));

    assert_eq!(wired.passes(), 0, "no index, no pass");
    assert!(wired.recorder.pushes().is_empty(), "and nothing is pushed");
    assert_eq!(
        wired.source().starts(),
        0,
        "and there is no directory to watch"
    );
    assert!(wired.service.stop(CLOSE));
}

#[test]
fn a_run_driven_by_hand_starts_no_watcher_and_no_timer() {
    // The passes alone, the seam the other tests of the list use: a request is
    // one pass, and nothing else ever asks for one.
    let home = session_seam::Home::new();
    home.session("p", "a.jsonl", "first chat");
    let (service, _holder, recorder) = session_seam::service(&home);
    service.request();
    let pushes = recorder.wait_for_complete();
    assert_eq!(pushes.last().expect("a list").total_sessions, 1);
    std::thread::sleep(SETTLE);
    assert_eq!(
        recorder.count(),
        pushes.len(),
        "a run with no wiring started a timer"
    );
}
