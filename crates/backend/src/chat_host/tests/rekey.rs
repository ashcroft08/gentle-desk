//! A new chat learns its session path after its child is up: the slot moves
//! from the `New` key to the `Saved` one without touching the process, and the
//! events that child sends afterwards carry the new key.

use super::*;
use fake::FakeSpawner;

fn fresh(spawner: &Arc<FakeSpawner>) -> (ChatHost, ChatKey) {
    let host = plain(spawner);
    let key = ChatKey::new_chat();
    host.open(key.clone()).unwrap();
    (host, key)
}

#[test]
fn events_after_a_rekey_carry_the_new_key() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);
    let child = spawner.child(&new);
    child.push(ProcessEvent::Stderr("one".into()));
    assert_eq!(next(&host).key, new, "before the rekey the old key tags it");

    assert!(host.rekey(&new, saved("s")));
    child.push(ProcessEvent::Stderr("two".into()));

    assert_eq!(next(&host).key, saved("s"));
}

#[test]
fn a_chat_whose_child_already_died_cannot_be_rekeyed() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);
    spawner.child(&new).push(ProcessEvent::Closed);
    assert_eq!(next(&host).event, ProcessEvent::Closed);

    assert!(!host.rekey(&new, saved("s")));
}

#[test]
fn a_rekey_moves_the_chat_without_touching_its_process() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);

    assert!(host.rekey(&new, saved("s")));

    assert_eq!(host.live_keys(), vec![saved("s")]);
    assert!(host.is_alive(&saved("s")));
    assert!(!host.is_alive(&new));
    assert_eq!(spawner.child(&new).shutdowns(), 0);
    assert_eq!(spawner.spawned_keys(), vec![new], "no second child");
}

#[test]
fn the_foreground_and_working_marks_follow_the_chat() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);
    host.set_working(&new, true);

    assert!(host.rekey(&new, saved("s")));

    assert_eq!(host.foreground(), Some(saved("s")));
    assert_eq!(host.working_count(), 1);
}

#[test]
fn commands_reach_the_child_under_its_new_key() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);
    host.rekey(&new, saved("s"));

    host.send(&saved("s"), &RpcCommand::GetState).unwrap();

    assert_eq!(spawner.child(&new).commands(), vec![RpcCommand::GetState]);
    assert!(host.send(&new, &RpcCommand::GetState).is_err());
}

#[test]
fn a_rekey_is_refused_for_an_unknown_chat_or_a_taken_key() {
    let spawner = Arc::new(FakeSpawner::new());
    let (host, new) = fresh(&spawner);
    host.open(saved("s")).unwrap();

    assert!(!host.rekey(&new, saved("s")), "the target key is taken");
    assert!(!host.rekey(&saved("ghost"), saved("t")), "unknown chat");
    assert_eq!(host.live_keys(), vec![saved("s"), new]);
}

#[test]
fn a_chat_whose_child_is_still_starting_cannot_be_rekeyed() {
    let (spawner, gate) = FakeSpawner::gated();
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));
    let new = ChatKey::new_chat();
    let opener = {
        let (host, new) = (Arc::clone(&host), new.clone());
        std::thread::spawn(move || host.open(new))
    };
    spawner.wait_for_spawns(1);

    assert!(!host.rekey(&new, saved("s")));

    gate.release();
    assert_eq!(opener.join().unwrap().unwrap(), Opened::Started);
    assert!(host.is_alive(&new));
}
