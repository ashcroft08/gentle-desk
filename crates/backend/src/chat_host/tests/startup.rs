//! What `open` starts, and in which order: the child of the chat the user just
//! clicked comes up before the children a cap or the idle window took out are
//! stopped, and a chat whose child died starts fresh instead of answering
//! `Existing`.

use std::sync::Arc;
use std::time::Duration;

use super::*;
use fake::FakeSpawner;

#[test]
fn the_clicked_chat_starts_before_the_reaped_victim_is_stopped() {
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

    // Over the live cap: "a" is the least recently used background chat, so this
    // open reaps it. Its graceful shutdown can take seconds, and the chat the
    // user clicked must not wait behind it before its own child even starts.
    assert_eq!(host.open(saved("c")).unwrap(), Opened::Started);

    assert_eq!(
        spawner.journal(),
        vec![
            "spawn chat a#0",
            "spawn chat b#0",
            "spawn chat c#0",
            "shutdown chat a#0"
        ]
    );
}

#[test]
fn a_chat_whose_child_died_starts_a_fresh_child_instead_of_answering_existing() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    spawner.child(&saved("a")).push(ProcessEvent::Closed);
    assert_eq!(next(&host).event, ProcessEvent::Closed);
    assert!(!host.is_alive(&saved("a")));

    assert_eq!(host.open(saved("a")).unwrap(), Opened::Started);

    assert_eq!(spawner.spawn_count(&saved("a")), 2);
    let children = spawner.children(&saved("a"));
    assert_eq!(children[0].shutdowns(), 1, "the dead child was let go");
    assert_eq!(children[1].shutdowns(), 0, "the new child is the live one");
    assert!(host.is_alive(&saved("a")));
    assert_eq!(host.foreground(), Some(saved("a")));
}
