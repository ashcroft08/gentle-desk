//! Telling a stop from a death: the host stamps `expected` on the `Closed` event
//! of a child it stopped itself (idle reap, cap victim, close, shutdown), so the
//! window can stay silent for those and offer a Restart only for a real death.

use std::sync::Arc;
use std::time::Duration;

use super::*;
use fake::FakeSpawner;

fn take_closed(host: &ChatHost) -> TaggedEvent {
    let event = next(host);
    assert_eq!(
        event.event,
        ProcessEvent::Closed,
        "the child reported its stream ending"
    );
    event
}

#[test]
fn a_child_that_exits_on_its_own_closes_unexpected() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    // The child ends without anyone asking it to: the one case the user has to
    // be told about, with a way to start it again.
    spawner.child(&saved("a")).push(ProcessEvent::Closed);

    let event = take_closed(&host);
    assert_eq!(event.key, saved("a"));
    assert!(!event.expected, "nobody stopped it: this is a death");
}

#[test]
fn closing_a_chat_marks_that_close_expected() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    assert!(host.close(&saved("a")));
    spawner.child(&saved("a")).push(ProcessEvent::Closed);

    assert!(take_closed(&host).expected, "the host closed it on purpose");
}

#[test]
fn the_idle_reap_marks_the_reaped_close_expected() {
    let spawner = Arc::new(FakeSpawner::new());
    let clock = Arc::new(ManualClock::new());
    let host = host(Limits::new(4, 2, Duration::from_secs(10)), &spawner, &clock);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();
    clock.advance(Duration::from_secs(60));

    assert_eq!(host.reap_idle(), vec![saved("a")]);
    spawner.child(&saved("a")).push(ProcessEvent::Closed);

    let event = take_closed(&host);
    assert_eq!(event.key, saved("a"));
    assert!(event.expected, "a reaped chat reopens through --session");
}

#[test]
fn a_child_stopped_for_room_under_the_cap_closes_expected() {
    let spawner = Arc::new(FakeSpawner::new());
    let clock = Arc::new(ManualClock::new());
    let host = host(
        Limits::new(2, 2, Duration::from_secs(600)),
        &spawner,
        &clock,
    );
    host.open(saved("a")).unwrap();
    clock.advance(Duration::from_secs(1));
    host.open(saved("b")).unwrap();
    clock.advance(Duration::from_secs(1));

    // Over the live cap: the least recently used background chat makes room.
    assert_eq!(host.open(saved("c")).unwrap(), Opened::Started);
    spawner.child(&saved("a")).push(ProcessEvent::Closed);

    let event = take_closed(&host);
    assert_eq!(event.key, saved("a"));
    assert!(event.expected, "the cap took it out, not a crash");
}

#[test]
fn shutting_the_host_down_marks_every_close_expected() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    host.shutdown();
    for key in [saved("a"), saved("b")] {
        spawner.child(&key).push(ProcessEvent::Closed);
    }

    let mut seen = vec![take_closed(&host), take_closed(&host)];
    seen.sort_by(|left, right| left.key.cmp(&right.key));
    assert!(seen.iter().all(|event| event.expected), "{seen:?}");
}
