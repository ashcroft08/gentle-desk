//! The host calls that are not a command: the idle reap and the shutdown of
//! every child before the window exits (S5, S7).

use std::time::Duration;

use gentle_app::ChatId;
use gentle_backend::chat_host::{ChatKey, Limits};

use super::{open, runtime, runtime_with_clock};

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
