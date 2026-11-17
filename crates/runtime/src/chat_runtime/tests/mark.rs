//! The working mark through the command seam (S1, S2).
//!
//! `ChatCommand::SetWorking` is what the reducer asks for when a chat starts or
//! ends a turn, so this is the path that decides whether the host's protection
//! rules ever see a working chat. The rules themselves are pinned in
//! `gentle-backend`'s `chat_host::policy` and `chat_host::tests::caps`; what
//! these tests add is that the seam production uses actually reaches them.

use std::time::Duration;

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::{ChatKey, Limits};

use super::{open, runtime_with};
use crate::chat_runtime::Outcome;

fn mark(chat: &ChatId, working: bool) -> ChatCommand {
    ChatCommand::SetWorking {
        chat: chat.clone(),
        working,
    }
}

/// The live cap is two, so a third open has to stop somebody: the foreground
/// chat is protected, and so is the one the command just marked.
#[test]
fn a_working_chat_is_protected_at_the_live_cap_through_the_command_seam() {
    let (runtime, spawner) = runtime_with(Limits::new(2, 2, Duration::from_secs(60)));
    let (a, b, c) = (
        ChatId::of_session("a"),
        ChatId::of_session("b"),
        ChatId::of_session("c"),
    );
    runtime.execute(open(&a));
    runtime.execute(open(&b));

    assert_eq!(
        runtime.execute(mark(&a, true)),
        Outcome::Working {
            chat: a.clone(),
            working: true
        }
    );
    let outcome = runtime.execute(open(&c));

    assert!(
        matches!(outcome, Outcome::OpenFailed { .. }),
        "the cap leaves no room and the working chat may not make it: {outcome:?}"
    );
    assert_eq!(
        spawner.stopped(),
        Vec::new(),
        "the chat in the middle of a turn was never stopped"
    );
}

/// The same chat, once its turn ended, is exactly the victim the cap picks: the
/// protection came from the mark and not from the chat being spared.
#[test]
fn clearing_the_mark_hands_the_same_chat_back_to_the_cap() {
    let (runtime, spawner) = runtime_with(Limits::new(2, 2, Duration::from_secs(60)));
    let (a, b, c) = (
        ChatId::of_session("a"),
        ChatId::of_session("b"),
        ChatId::of_session("c"),
    );
    runtime.execute(open(&a));
    runtime.execute(open(&b));
    runtime.execute(mark(&a, true));
    let stopped_before = spawner.stopped();

    assert_eq!(
        stopped_before,
        Vec::new(),
        "nothing was stopped while it worked"
    );

    runtime.execute(mark(&a, false));
    let outcome = runtime.execute(open(&c));

    assert!(
        matches!(outcome, Outcome::Opened { .. }),
        "the room came from the chat whose turn ended: {outcome:?}"
    );
    assert_eq!(
        spawner.stopped(),
        vec![ChatKey::Saved("a".into())],
        "the chat that stopped working is the one the cap reaps"
    );
}

#[test]
fn marking_a_chat_the_host_does_not_know_answers_without_starting_anything() {
    let (runtime, spawner) = runtime_with(Limits::default());
    let never = ChatId::of_session("never-opened");

    assert_eq!(
        runtime.execute(mark(&never, true)),
        Outcome::Working {
            chat: never.clone(),
            working: true
        }
    );
    assert!(
        spawner.started().is_empty(),
        "a mark is not a reason to start a child"
    );
}
