//! The host calls that are not a command: the idle reap, the shutdown of every
//! child before the window exits (S5, S7), and the stop of one chat's child
//! when the user closes that chat (0.1.14).

use std::time::Duration;

use gentle_app::ChatId;
use gentle_backend::chat_host::{ChatKey, Limits};

use super::{closing, open, runtime, runtime_with_clock, send};
use crate::chat_runtime::Outcome;

#[test]
fn a_working_chat_is_never_reaped_and_an_idle_one_is() {
    let (runtime, spawner, clock) = runtime_with_clock(Limits::new(4, 2, Duration::from_secs(60)));
    let (a, b, c) = (
        ChatId::of_session("a"),
        ChatId::of_session("b"),
        ChatId::of_session("c"),
    );
    for chat in [&a, &b, &c] {
        runtime.execute(open(chat));
    }

    assert!(runtime.set_working(&a, true));
    clock.advance(Duration::from_secs(120));
    let reaped = runtime.reap_idle();

    assert_eq!(
        reaped,
        vec![ChatKey::Saved("b".into())],
        "the idle background chat goes; the working one and the foreground one stay"
    );
    assert_eq!(spawner.stopped(), vec![ChatKey::Saved("b".into())]);
}

#[test]
fn a_chat_the_host_does_not_know_is_never_marked_working() {
    let (runtime, _spawner) = runtime();

    assert!(!runtime.set_working(&ChatId::of_session("never-opened"), true));
}

#[test]
fn every_child_is_stopped_when_the_shell_shuts_down() {
    let (runtime, spawner) = runtime();
    let (a, b) = (ChatId::of_session("a"), ChatId::of_session("b"));
    runtime.execute(open(&a));
    runtime.execute(open(&b));

    runtime.shutdown();

    let mut stopped = spawner.stopped();
    stopped.sort();
    assert_eq!(
        stopped,
        vec![ChatKey::Saved("a".into()), ChatKey::Saved("b".into())]
    );
}

#[test]
fn closing_one_chat_stops_only_that_child_and_the_others_keep_theirs() {
    let (runtime, spawner) = runtime();
    let (a, b, c) = (
        ChatId::of_session("a"),
        ChatId::of_session("b"),
        ChatId::of_session("c"),
    );
    for chat in [&a, &b, &c] {
        runtime.execute(open(chat));
    }

    let outcome = runtime.execute(closing(&b));

    assert_eq!(outcome, Outcome::Closed { chat: b.clone() });
    assert_eq!(
        spawner.stopped(),
        vec![ChatKey::Saved("b".into())],
        "the closed chat's child is the only one stopped"
    );
    // The two chats the user kept are untouched: their children still take
    // commands. The closed one has no child left to write to.
    assert_eq!(runtime.execute(send(&a)), Outcome::Sent);
    assert_eq!(runtime.execute(send(&c)), Outcome::Sent);
    assert!(matches!(
        runtime.execute(send(&b)),
        Outcome::SendFailed { .. }
    ));
}

#[test]
fn a_closed_chat_starts_a_fresh_child_when_it_is_opened_again() {
    let (runtime, spawner) = runtime();
    let a = ChatId::of_session("a");
    runtime.execute(open(&a));

    runtime.execute(closing(&a));
    let again = runtime.execute(open(&a));

    assert!(
        matches!(again, Outcome::Opened { .. }),
        "closing a chat is not a death: its next open starts a child through --session"
    );
    assert_eq!(
        spawner.started(),
        vec![ChatKey::Saved("a".into()), ChatKey::Saved("a".into())],
        "a second start for the same session path"
    );
}

#[test]
fn closing_a_chat_with_no_child_stops_nothing_and_is_not_an_error() {
    let (runtime, spawner) = runtime();

    let outcome = runtime.execute(closing(&ChatId::of_session("never-opened")));

    assert_eq!(
        outcome,
        Outcome::Closed {
            chat: ChatId::of_session("never-opened")
        },
        "a chat with no child is simply closed"
    );
    assert!(spawner.stopped().is_empty());
}
