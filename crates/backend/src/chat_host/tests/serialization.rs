//! Spawns are serialized per chat (S2): a rapid second click never starts a
//! second child, and a start result that arrives after its chat was closed is
//! discarded instead of adopted.

use std::sync::Arc;
use std::thread;

use gentle_core::rpc::RpcCommand;

use super::*;
use fake::FakeSpawner;

#[test]
fn concurrent_opens_of_one_chat_start_exactly_one_child() {
    let (spawner, gate) = FakeSpawner::gated();
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));

    let first = {
        let host = Arc::clone(&host);
        thread::spawn(move || host.open(saved("a")))
    };
    spawner.wait_for_spawns(1);

    // A second click while the first child is still starting: no second child.
    assert_eq!(host.open(saved("a")).unwrap(), Opened::Existing);
    gate.release();
    assert_eq!(first.join().unwrap().unwrap(), Opened::Started);

    assert_eq!(spawner.spawn_count(&saved("a")), 1);
    assert!(host.is_alive(&saved("a")));
}

#[test]
fn a_spawn_that_finishes_after_its_chat_was_closed_is_discarded() {
    let (spawner, gate) = FakeSpawner::gated();
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));

    let first = {
        let host = Arc::clone(&host);
        thread::spawn(move || host.open(saved("a")))
    };
    spawner.wait_for_spawns(1);
    assert!(host.close(&saved("a")));
    gate.release();

    assert_eq!(first.join().unwrap().unwrap(), Opened::Discarded);
    assert_eq!(spawner.spawn_count(&saved("a")), 1);
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 1);
    assert_eq!(host.live_keys(), Vec::new());
    assert_eq!(host.foreground(), None);
}

#[test]
fn a_command_to_a_chat_that_is_still_starting_is_refused() {
    let (spawner, gate) = FakeSpawner::gated();
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));

    let first = {
        let host = Arc::clone(&host);
        thread::spawn(move || host.open(saved("a")))
    };
    spawner.wait_for_spawns(1);

    let refused = host.send(&saved("a"), &RpcCommand::GetState).unwrap_err();

    assert!(matches!(refused, SendError::Starting(key) if key == saved("a")));
    gate.release();
    first.join().unwrap().unwrap();
    assert!(host.send(&saved("a"), &RpcCommand::GetState).is_ok());
}

#[test]
fn reopening_a_chat_while_its_child_is_starting_does_not_start_another() {
    let (spawner, gate) = FakeSpawner::gated();
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));

    let first = {
        let host = Arc::clone(&host);
        thread::spawn(move || host.open(saved("a")))
    };
    spawner.wait_for_spawns(1);
    host.open(saved("a")).unwrap();
    gate.release();
    first.join().unwrap().unwrap();

    assert_eq!(spawner.spawn_count(&saved("a")), 1);
    assert_eq!(spawner.children(&saved("a")).len(), 1);
}
