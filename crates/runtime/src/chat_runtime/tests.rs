use std::sync::Arc;
use std::time::Duration;

use gentle_app::{ChatCommand, ChatId, Input};
use gentle_backend::chat_host::{
    ChatHost, ChatKey, Limits, ManualClock, Opened, Spawner, TaggedEvent,
};
use gentle_backend::process::ProcessEvent;
use gentle_core::rpc::{Decoded, RpcCommand, RpcEvent};

use super::{ChatRuntime, Converted, Outcome};

mod calls;
mod fake;
mod folder;
mod open_chat;
mod rekey;
use fake::FakeSpawner;

fn runtime_with(limits: Limits) -> (ChatRuntime, Arc<FakeSpawner>) {
    let (runtime, spawner, _clock) = runtime_with_clock(limits);
    (runtime, spawner)
}

fn runtime_with_clock(limits: Limits) -> (ChatRuntime, Arc<FakeSpawner>, Arc<ManualClock>) {
    let spawner = Arc::new(FakeSpawner::default());
    let clock = Arc::new(ManualClock::new());
    let host = ChatHost::with_parts(
        limits,
        Arc::clone(&spawner) as Arc<dyn Spawner>,
        Arc::clone(&clock) as Arc<dyn gentle_backend::chat_host::Clock>,
    );
    let (runtime, _events) = ChatRuntime::new(host);
    (runtime, spawner, clock)
}

fn runtime() -> (ChatRuntime, Arc<FakeSpawner>) {
    runtime_with(Limits::default())
}

fn open(chat: &ChatId) -> ChatCommand {
    ChatCommand::OpenProcess { chat: chat.clone() }
}

fn send(chat: &ChatId) -> ChatCommand {
    ChatCommand::Send {
        chat: chat.clone(),
        rpc: RpcCommand::GetState,
    }
}

/// The user's Close chat: this chat's child is stopped and nothing of another
/// chat's changes.
fn closing(chat: &ChatId) -> ChatCommand {
    ChatCommand::CloseProcess { chat: chat.clone() }
}

fn tagged(key: ChatKey, event: ProcessEvent) -> TaggedEvent {
    TaggedEvent {
        key,
        event,
        expected: false,
    }
}

#[test]
fn opening_a_stored_chat_starts_a_child_for_its_session_path() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    let outcome = runtime.execute(open(&chat));

    assert_eq!(
        outcome,
        Outcome::Opened {
            chat,
            how: Opened::Started
        }
    );
    assert_eq!(spawner.started(), vec![ChatKey::Saved("s.jsonl".into())]);
}

#[test]
fn opening_a_fresh_chat_starts_a_child_under_a_new_key() {
    let (runtime, spawner) = runtime();

    runtime.execute(open(&ChatId::fresh()));

    assert!(matches!(spawner.started()[..], [ChatKey::New(_)]));
}

#[test]
fn opening_a_chat_again_only_focuses_it() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(open(&chat));

    assert_eq!(
        outcome,
        Outcome::Opened {
            chat,
            how: Opened::Existing
        }
    );
    assert_eq!(spawner.started().len(), 1, "one child, not two");
}

#[test]
fn a_child_that_cannot_start_is_a_message_and_not_a_panic() {
    let (runtime, spawner) = runtime();
    spawner.fail_next("no launcher");
    let chat = ChatId::of_session("s.jsonl");

    let outcome = runtime.execute(open(&chat));

    let Outcome::OpenFailed {
        chat: failed,
        message,
    } = outcome
    else {
        panic!("expected OpenFailed, got {outcome:?}");
    };
    assert_eq!(failed, chat);
    assert!(message.contains("no launcher"), "message was: {message}");
}

#[test]
fn a_full_host_refuses_the_open_with_a_message() {
    let (runtime, spawner) = runtime_with(Limits::new(1, 1, Duration::from_secs(600)));
    runtime.execute(open(&ChatId::of_session("a")));
    let second = ChatId::of_session("b");

    let outcome = runtime.execute(open(&second));

    let Outcome::OpenFailed { chat, message } = outcome else {
        panic!("expected OpenFailed, got {outcome:?}");
    };
    assert_eq!(chat, second);
    assert!(!message.is_empty());
    assert_eq!(spawner.started().len(), 1, "no second child");
}

#[test]
fn a_command_reaches_only_the_child_of_its_chat() {
    let (runtime, spawner) = runtime();
    let (a, b) = (ChatId::of_session("a"), ChatId::of_session("b"));
    runtime.execute(open(&a));
    runtime.execute(open(&b));

    assert_eq!(runtime.execute(send(&b)), Outcome::Sent);

    assert_eq!(
        spawner.written_to(&ChatKey::Saved("b".into())),
        vec![RpcCommand::GetState]
    );
    assert!(spawner.written_to(&ChatKey::Saved("a".into())).is_empty());
}

#[test]
fn a_command_for_a_chat_without_a_child_is_a_message() {
    let (runtime, _spawner) = runtime();
    let chat = ChatId::of_session("never-opened");

    let outcome = runtime.execute(send(&chat));

    let Outcome::SendFailed {
        chat: failed,
        message,
    } = outcome
    else {
        panic!("expected SendFailed, got {outcome:?}");
    };
    assert_eq!(failed, chat);
    assert!(!message.is_empty());
}

#[test]
fn a_command_for_a_fresh_chat_nobody_opened_is_a_message_too() {
    let (runtime, _spawner) = runtime();

    let outcome = runtime.execute(send(&ChatId::fresh()));

    assert!(matches!(outcome, Outcome::SendFailed { .. }), "{outcome:?}");
}

#[test]
fn a_decoded_event_becomes_a_timed_input_for_its_chat() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(open(&chat));
    let key = spawner.started().remove(0);

    let converted = runtime.convert(
        tagged(
            key,
            ProcessEvent::Output(Decoded::Event(RpcEvent::AgentStart)),
        ),
        7,
    );

    let Some(Converted::Input(Input::ChatEventAt {
        chat: from,
        event,
        now_ms,
    })) = converted
    else {
        panic!("expected a timed chat event, got {converted:?}");
    };
    assert_eq!((from, event, now_ms), (chat, RpcEvent::AgentStart, 7));
}

#[test]
fn stderr_and_closed_are_kept_for_the_shell_to_show() {
    let (runtime, _spawner) = runtime();
    let key = ChatKey::Saved("s.jsonl".into());
    let chat = ChatId::of_session("s.jsonl");

    assert_eq!(
        runtime.convert(tagged(key.clone(), ProcessEvent::Stderr("boom".into())), 1),
        Some(Converted::Stderr {
            chat: chat.clone(),
            line: "boom".into()
        })
    );
    assert_eq!(
        runtime.convert(tagged(key, ProcessEvent::Closed), 1),
        Some(Converted::Closed {
            chat,
            expected: false
        })
    );
}

#[test]
fn a_close_the_host_asked_for_is_marked_expected() {
    let (runtime, _spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    // The host stamps an expected stop on the event; the shell must keep it, or
    // a reaped chat would look like a death worth a notice (S3).
    let stopped = TaggedEvent {
        key: ChatKey::Saved("s.jsonl".into()),
        event: ProcessEvent::Closed,
        expected: true,
    };

    assert_eq!(
        runtime.convert(stopped, 1),
        Some(Converted::Closed {
            chat,
            expected: true
        })
    );
}

#[test]
fn an_event_of_a_chat_nobody_registered_is_dropped() {
    let (runtime, _spawner) = runtime();

    let converted = runtime.convert(
        tagged(
            ChatKey::new_chat(),
            ProcessEvent::Output(Decoded::Event(RpcEvent::AgentStart)),
        ),
        1,
    );

    assert_eq!(converted, None);
}

#[test]
fn a_stdout_line_that_is_not_a_known_event_is_dropped() {
    let (runtime, _spawner) = runtime();
    let key = ChatKey::Saved("s.jsonl".into());

    let converted = runtime.convert(
        tagged(
            key,
            ProcessEvent::Output(Decoded::Unknown(r#"{"type":"mystery"}"#.into())),
        ),
        1,
    );

    assert_eq!(converted, None);
}
