//! The caps and the idle reap, at the host level: which child an open stops to
//! make room, and which refusal it answers instead.

use std::sync::Arc;
use std::time::Duration;

use super::*;
use fake::FakeSpawner;

/// A second of test time per step, so the LRU order is never a tie.
const STEP: Duration = Duration::from_secs(1);

fn two_live() -> (Arc<FakeSpawner>, Arc<ManualClock>) {
    (Arc::new(FakeSpawner::new()), Arc::new(ManualClock::new()))
}

#[test]
fn opening_over_the_live_cap_stops_the_least_recently_used_idle_chat() {
    let (spawner, clock) = two_live();
    let host = host(
        Limits::new(2, 2, Duration::from_secs(600)),
        &spawner,
        &clock,
    );
    host.open(saved("a")).unwrap();
    clock.advance(STEP);
    host.open(saved("b")).unwrap();
    clock.advance(STEP);

    assert_eq!(host.open(saved("c")).unwrap(), Opened::Started);

    // "a" was the least recently used background chat; "b" was the foreground.
    assert_eq!(host.live_keys(), vec![saved("b"), saved("c")]);
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 1);
    assert_eq!(spawner.child(&saved("b")).shutdowns(), 0);
    assert_eq!(host.foreground(), Some(saved("c")));
}

#[test]
fn opening_over_the_live_cap_never_stops_a_working_chat() {
    let (spawner, clock) = two_live();
    let host = host(
        Limits::new(1, 1, Duration::from_secs(600)),
        &spawner,
        &clock,
    );
    host.open(saved("a")).unwrap();
    assert!(host.set_working(&saved("a"), true));
    clock.advance(STEP);

    let refused = host.open(saved("b")).unwrap_err();

    assert_eq!(
        refused,
        HostError::TooManyLive(TooManyLive { cap: 1, live: 1 })
    );
    assert!(host.is_alive(&saved("a")));
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 0);
    assert_eq!(spawner.spawn_count(&saved("b")), 0);
}

#[test]
fn opening_over_the_live_cap_never_stops_the_foreground_chat() {
    let (spawner, clock) = two_live();
    let host = host(
        Limits::new(1, 2, Duration::from_secs(600)),
        &spawner,
        &clock,
    );
    host.open(saved("a")).unwrap();
    clock.advance(STEP);

    let refused = host.open(saved("b")).unwrap_err();

    // The only live child is the one the user is on: nothing may be stopped.
    assert_eq!(
        refused,
        HostError::TooManyLive(TooManyLive { cap: 1, live: 1 })
    );
    assert_eq!(host.live_keys(), vec![saved("a")]);
    assert_eq!(host.foreground(), Some(saved("a")));
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 0);
    assert_eq!(spawner.spawn_count(&saved("b")), 0);
}

#[test]
fn opening_reaps_the_idle_background_children_first() {
    let (spawner, clock) = two_live();
    let limits = Limits::new(4, 2, Duration::from_secs(10));
    let host = host(limits, &spawner, &clock);
    for name in ["a", "b", "c"] {
        host.open(saved(name)).unwrap();
        clock.advance(STEP);
    }
    // "a" and "b" are background and now long idle; "c" is the foreground.
    clock.advance(Duration::from_secs(60));

    assert_eq!(host.open(saved("d")).unwrap(), Opened::Started);

    assert_eq!(
        host.live_keys(),
        vec![saved("c"), saved("d")],
        "the idle background children went, the foreground stayed"
    );
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 1);
    assert_eq!(spawner.child(&saved("b")).shutdowns(), 1);
    assert_eq!(spawner.child(&saved("c")).shutdowns(), 0);
}

#[test]
fn reaping_idle_children_leaves_the_foreground_and_the_working_ones() {
    let (spawner, clock) = two_live();
    let limits = Limits::new(4, 2, Duration::from_secs(10));
    let host = host(limits, &spawner, &clock);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();
    host.open(saved("c")).unwrap();
    assert!(host.set_working(&saved("b"), true));
    clock.advance(Duration::from_secs(60));

    assert_eq!(host.reap_idle(), vec![saved("a")]);

    assert_eq!(host.live_keys(), vec![saved("b"), saved("c")]);
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 1);
}

#[test]
fn a_reaped_chat_reopens_as_a_fresh_child() {
    let (spawner, clock) = two_live();
    let host = host(Limits::new(4, 2, Duration::from_secs(10)), &spawner, &clock);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();
    clock.advance(Duration::from_secs(60));
    assert_eq!(host.reap_idle(), vec![saved("a")]);

    assert_eq!(host.open(saved("a")).unwrap(), Opened::Started);
    assert_eq!(spawner.spawn_count(&saved("a")), 2);
}

#[test]
fn the_working_cap_gates_a_new_model_call() {
    let (spawner, clock) = two_live();
    let limits = Limits::new(4, 2, Duration::from_secs(600));
    let host = host(limits, &spawner, &clock);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    assert!(host.may_start_work());
    assert!(host.set_working(&saved("a"), true));
    assert!(host.may_start_work());
    assert!(host.set_working(&saved("b"), true));
    assert!(!host.may_start_work());
    assert_eq!(host.working_count(), 2);
    assert!(host.set_working(&saved("a"), false));
    assert!(host.may_start_work());
    assert!(!host.set_working(&saved("c"), true));
}
