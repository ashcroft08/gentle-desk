//! The host against a fake spawner: what `open` starts, what it leaves alone,
//! where commands go and how every child's events come back.

use std::path::PathBuf;
use std::sync::Arc;

use gentle_core::rpc::{Decoded, RpcCommand, RpcEvent};

use super::*;
use crate::process::ProcessEvent;

mod fake;
use fake::{FakeSpawner, WAIT};

fn saved(name: &str) -> ChatKey {
    ChatKey::Saved(PathBuf::from(name))
}

fn host(limits: Limits, spawner: &Arc<FakeSpawner>, clock: &Arc<ManualClock>) -> ChatHost {
    ChatHost::with_parts(limits, spawner.clone(), clock.clone())
}

/// The default limits and a clock no test has to move.
fn plain(spawner: &Arc<FakeSpawner>) -> ChatHost {
    host(Limits::default(), spawner, &Arc::new(ManualClock::new()))
}

fn next(host: &ChatHost) -> TaggedEvent {
    host.recv_timeout(WAIT).expect("an event")
}

#[test]
fn a_new_chat_key_is_distinct_every_time() {
    assert_ne!(ChatKey::new_chat(), ChatKey::new_chat());
    assert_ne!(ChatKey::new_chat(), saved("a"));
}

#[test]
fn opening_a_chat_starts_one_child_and_makes_it_foreground() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);

    assert_eq!(host.open(saved("a")).unwrap(), Opened::Started);

    assert_eq!(spawner.spawn_count(&saved("a")), 1);
    assert_eq!(host.foreground(), Some(saved("a")));
    assert!(host.is_alive(&saved("a")));
    assert_eq!(host.live_keys(), vec![saved("a")]);
}

#[test]
fn reopening_a_chat_only_makes_it_foreground_again() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    assert_eq!(host.open(saved("a")).unwrap(), Opened::Existing);

    assert_eq!(spawner.spawn_count(&saved("a")), 1);
    assert_eq!(host.foreground(), Some(saved("a")));
    assert_eq!(host.live_keys(), vec![saved("a"), saved("b")]);
}

#[test]
fn opening_another_chat_leaves_the_first_child_running() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    host.open(saved("b")).unwrap();

    assert_eq!(host.live_keys(), vec![saved("a"), saved("b")]);
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 0);
    assert!(host.is_alive(&saved("a")));
    assert_eq!(host.foreground(), Some(saved("b")));
}

#[test]
fn opening_chats_never_sends_a_session_switch() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);

    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();
    host.open(saved("a")).unwrap();

    // The session is chosen at spawn time; no child is ever told to replace its
    // session, which is what the extension context used to die of.
    for key in [saved("a"), saved("b")] {
        assert_eq!(spawner.child(&key).commands(), Vec::new(), "{key}");
    }
}

#[test]
fn a_command_reaches_only_its_chats_child() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    let prompt = RpcCommand::Prompt {
        message: "hola".into(),
    };
    host.send(&saved("b"), &prompt).unwrap();

    assert_eq!(spawner.child(&saved("b")).commands(), vec![prompt]);
    assert_eq!(spawner.child(&saved("a")).commands(), Vec::new());
}

#[test]
fn a_prompt_with_images_reaches_its_chats_child_as_it_was_sent() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    // The host writes what the shell built, images and all: the encoding is the
    // codec's (`crates/gentle-core/tests/codec.rs`), the routing is this.
    let prompt = RpcCommand::prompt_with_images(
        "look",
        vec![gentle_core::rpc::ImageContent {
            mime_type: "image/png".to_owned(),
            data: "aGk=".to_owned(),
        }],
    );
    host.send(&saved("a"), &prompt).unwrap();

    assert_eq!(spawner.child(&saved("a")).commands(), vec![prompt]);
}

#[test]
fn sending_to_a_chat_the_host_never_opened_is_refused() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);

    let refused = host.send(&saved("a"), &RpcCommand::GetState).unwrap_err();

    assert!(matches!(refused, SendError::NoChild(key) if key == saved("a")));
}

#[test]
fn every_childs_events_arrive_on_one_channel_tagged_with_its_chat() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    spawner
        .child(&saved("a"))
        .push(ProcessEvent::Output(Decoded::Event(RpcEvent::AgentStart)));
    spawner
        .child(&saved("b"))
        .push(ProcessEvent::Stderr("boom".into()));

    let mut seen = vec![next(&host), next(&host)];
    seen.sort_by(|left, right| left.key.cmp(&right.key));
    assert_eq!(
        seen,
        vec![
            TaggedEvent {
                key: saved("a"),
                event: ProcessEvent::Output(Decoded::Event(RpcEvent::AgentStart)),
                expected: false,
            },
            TaggedEvent {
                key: saved("b"),
                event: ProcessEvent::Stderr("boom".into()),
                expected: false,
            },
        ]
    );
}

#[test]
fn a_child_that_closed_is_no_longer_live_and_reopening_restarts_it() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    spawner.child(&saved("a")).push(ProcessEvent::Closed);

    assert_eq!(
        next(&host),
        TaggedEvent {
            key: saved("a"),
            event: ProcessEvent::Closed,
            expected: false,
        }
    );
    assert!(!host.is_alive(&saved("a")));
    assert_eq!(host.open(saved("a")).unwrap(), Opened::Started);
    assert_eq!(spawner.spawn_count(&saved("a")), 2);
    assert_eq!(spawner.children(&saved("a"))[0].shutdowns(), 1);
}

#[test]
fn closing_a_chat_stops_only_its_child() {
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    assert!(host.close(&saved("b")));
    assert!(!host.close(&saved("b")));

    assert_eq!(spawner.child(&saved("b")).shutdowns(), 1);
    assert_eq!(spawner.child(&saved("a")).shutdowns(), 0);
    assert_eq!(host.live_keys(), vec![saved("a")]);
    assert_eq!(host.foreground(), None);
}

#[test]
fn a_failed_spawn_is_reported_and_leaves_no_slot_behind() {
    let spawner = Arc::new(FakeSpawner::new());
    spawner.fail(saved("a"), "the session file is gone");
    let host = plain(&spawner);

    let refused = host.open(saved("a")).unwrap_err();

    assert_eq!(
        refused,
        HostError::Spawn {
            key: saved("a"),
            message: "the session file is gone".into(),
        }
    );
    assert_eq!(host.live_keys(), Vec::new());
    assert_eq!(host.foreground(), None);
    assert!(!host.is_alive(&saved("a")));
    assert_eq!(
        host.send(&saved("a"), &RpcCommand::GetState)
            .unwrap_err()
            .to_string(),
        "chat a has no live child"
    );
}

mod caps;
mod closes;
mod folder;
mod isolation;
mod real;
mod rekey;
mod serialization;
mod startup;
