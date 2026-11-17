//! Stopping one helper (helper-stop S8): one stop client per chat, and a fresh
//! one when the chat is (re)opened.
//!
//! The client is what carries the chat's `requestId` counter and the
//! capabilities its child reported, so these two facts - the same client for one
//! chat, a different one for another chat, and a new one after an open - are
//! what makes S8's "ONE requestId counter per chat" true. No child starts here:
//! the client only holds the channel, and nothing is written until a stop is
//! asked for.

use std::sync::Arc;

use gentle_app::ChatId;

use super::{open, runtime};

#[test]
fn one_chat_keeps_one_stop_client_and_another_chat_gets_its_own() {
    let (runtime, _spawner) = runtime();
    let one = ChatId::fresh();
    let other = ChatId::fresh();
    runtime.execute(open(&one));
    runtime.execute(open(&other));

    let first = runtime.stopper(&one).expect("an open chat has a key");
    let again = runtime.stopper(&one).expect("the same chat is still open");
    let theirs = runtime.stopper(&other).expect("the other chat is open too");

    assert!(
        Arc::ptr_eq(&first, &again),
        "one client per chat: a second stop of the same chat is numbered by the same counter"
    );
    assert!(
        !Arc::ptr_eq(&first, &theirs),
        "two chats never share a counter"
    );
}

#[test]
fn reopening_a_live_chat_keeps_its_stop_client() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(open(&chat));
    let before = runtime.stopper(&chat).expect("the chat was opened");

    // Clicking an already-open chat only focuses it: the same child answers, so
    // the client's counter and the capabilities it cached still belong to it.
    // Dropping them would restart the requestId at desktop-1 (S8).
    runtime.execute(open(&chat));
    let after = runtime.stopper(&chat).expect("the chat is still open");

    assert_eq!(spawner.started().len(), 1, "one child, not two");
    assert!(
        Arc::ptr_eq(&before, &after),
        "a focus-only open keeps the client, so the counter keeps growing"
    );
}

#[test]
fn a_new_child_gets_a_fresh_stop_client() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(open(&chat));
    let before = runtime.stopper(&chat).expect("the chat was opened");

    // Close chat stops the child; opening the chat again starts a NEW one, and
    // capabilities belong to the child that reported them.
    runtime.execute(super::closing(&chat));
    runtime.execute(open(&chat));
    let after = runtime.stopper(&chat).expect("the chat is open again");

    assert_eq!(spawner.started().len(), 2, "the reopen started a new child");
    assert!(
        !Arc::ptr_eq(&before, &after),
        "a new child is asked for its own capabilities, so it gets a new client"
    );
}

#[test]
fn a_chat_the_host_never_opened_has_no_stop_client() {
    let (runtime, _spawner) = runtime();

    assert!(
        runtime.stopper(&ChatId::fresh()).is_none(),
        "no child, no channel: the stop degrades to nothing (S7)"
    );
}
