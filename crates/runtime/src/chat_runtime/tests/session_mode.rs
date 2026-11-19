//! Reading and changing one chat's session permission mode over that chat's own
//! child (composer-dock T3b, L7).
//!
//! The mode is the SESSION's, so the question goes to the chat's own child
//! through the client the chat already has: `get_permission_mode` reads it and
//! `set_permission_mode { mode }` asks for a change, with `persist` never sent
//! (L7: the UI changes this session, never the saved default).
//!
//! Nothing here starts a child: the scripted fake spawner answers the channel, so
//! the wire itself - the request ids, their order and their exact bodies - is
//! what these tests assert.

use std::sync::Arc;

use gentle_app::{ChatCommand, ChatId};
use gentle_core::rpc::permission::{PermissionOutcome, SessionMode};
use gentle_core::rpc::{NoAnswer, SCHEMA};
use serde_json::{json, Value};

use super::fake::Reply;
use super::open;
use super::profile::{answering, stored_chat, wire, Asked};
use crate::chat_runtime::Outcome;

/// One desktop request for a mode, read (`None`) or written (`Some`).
fn mode(chat: &ChatId, change: Option<SessionMode>) -> ChatCommand {
    ChatCommand::SessionMode {
        chat: chat.clone(),
        change,
    }
}

/// What a child answers `capabilities` with: exactly the methods `methods` names.
fn capabilities(methods: &[&str]) -> Value {
    json!({
        "schema": SCHEMA,
        "command": "gentle:desktop",
        "widget": "gentle-desktop-rpc",
        "methods": methods,
        "events": [],
    })
}

/// The two modes this channel's own state answer carries.
fn state(mode: &str) -> Value {
    json!({ "mode": mode, "default_mode": "normal" })
}

/// The two methods this slice asks for, as `capabilities` lists them.
const METHODS: [&str; 3] = [
    "capabilities",
    "get_permission_mode",
    "set_permission_mode",
];

#[test]
fn the_read_asks_capabilities_once_then_the_mode() {
    let (runtime, spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            "capabilities" => Reply::Data(capabilities(&METHODS)),
            "get_permission_mode" => Reply::Data(state("auto")),
            other => panic!("the read never asks {other}"),
        }));
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(mode(&chat, None));

    let Outcome::SessionMode {
        chat: answered,
        change,
        outcome,
    } = outcome
    else {
        panic!("expected the chat's session mode, got {outcome:?}");
    };
    assert_eq!(answered, chat);
    assert_eq!(change, None, "a read asks for no change");
    let PermissionOutcome::Answered(reported) = outcome else {
        panic!("the documented read answers the state, got {outcome:?}");
    };
    assert_eq!(reported.mode, "auto");
    assert_eq!(reported.default_mode, "normal");
    assert_eq!(
        wire(&spawner, &key),
        vec![
            Asked::of("desktop-1", "capabilities", json!({})),
            Asked::of("desktop-2", "get_permission_mode", json!({})),
        ],
        "capabilities once, then the read, the way the chat's client numbers them"
    );
}

#[test]
fn a_write_asks_the_mode_alone_and_never_persist() {
    let (runtime, spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            "capabilities" => Reply::Data(capabilities(&METHODS)),
            "set_permission_mode" => Reply::Data(json!({
                "mode": "yolo",
                "default_mode": "normal",
                "persisted": false,
                "changed": true,
            })),
            other => panic!("the write never asks {other}"),
        }));
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(mode(&chat, Some(SessionMode::Yolo)));

    let Outcome::SessionMode {
        change,
        outcome: PermissionOutcome::Answered(reported),
        ..
    } = outcome
    else {
        panic!("expected the re-read state, got {outcome:?}");
    };
    assert_eq!(change, Some(SessionMode::Yolo));
    assert_eq!(
        reported.mode, "yolo",
        "the answer is the state the owner re-read, never an echo of the request"
    );
    assert_eq!(
        wire(&spawner, &key),
        vec![
            Asked::of("desktop-1", "capabilities", json!({})),
            // Exactly the mode: `persist` is never sent (L7).
            Asked::of("desktop-2", "set_permission_mode", json!({ "mode": "yolo" })),
        ]
    );
}

#[test]
fn the_wire_string_of_a_mode_is_the_channels_own() {
    let (runtime, spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            "capabilities" => Reply::Data(capabilities(&METHODS)),
            "set_permission_mode" => Reply::Data(state("super-yolo")),
            other => panic!("the write never asks {other}"),
        }));
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    runtime.execute(mode(&chat, Some(SessionMode::SuperYolo)));

    assert_eq!(
        wire(&spawner, &key)[1],
        Asked::of(
            "desktop-2",
            "set_permission_mode",
            json!({ "mode": "super-yolo" })
        ),
        "the status text says SUPER-YOLO, the wire says super-yolo"
    );
}

#[test]
fn a_refused_change_is_the_owners_own_code() {
    let (runtime, _spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            "capabilities" => Reply::Data(capabilities(&METHODS)),
            "set_permission_mode" => Reply::Refused {
                code: "permission_denied",
            },
            other => panic!("the write never asks {other}"),
        }));
    let (chat, _key) = stored_chat();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(mode(&chat, Some(SessionMode::Yolo)));

    let Outcome::SessionMode {
        outcome: PermissionOutcome::Rejected(error),
        ..
    } = outcome
    else {
        panic!("a refusal is a real answer, got {outcome:?}");
    };
    assert_eq!(
        error.code, "permission_denied",
        "the child's own code travels, so the UI can show the refusal (L7)"
    );
}

#[test]
fn a_method_the_child_does_not_advertise_is_unsupported() {
    let (runtime, spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            "capabilities" => Reply::Data(capabilities(&["capabilities"])),
            other => panic!("an unsupported method is never written: {other}"),
        }));
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(mode(&chat, None));

    assert_eq!(
        outcome,
        Outcome::SessionMode {
            chat: chat.clone(),
            change: None,
            outcome: PermissionOutcome::Unsupported,
        },
        "support is never inferred: the child's own capabilities decide"
    );
    assert_eq!(
        wire(&spawner, &key),
        vec![Asked::of("desktop-1", "capabilities", json!({}))],
        "a method the child does not list is not asked"
    );
}

#[test]
fn a_chat_with_no_child_reads_and_writes_nothing() {
    let (runtime, spawner, _events) =
        answering(Arc::new(|method: &str, _params: &Value| match method {
            other => panic!("a chat with no child is never asked: {other}"),
        }));
    let chat = ChatId::fresh();

    for change in [None, Some(SessionMode::Auto)] {
        assert_eq!(
            runtime.execute(mode(&chat, change)),
            Outcome::SessionMode {
                chat: chat.clone(),
                change,
                outcome: PermissionOutcome::NoAnswer(NoAnswer::NotConnected),
            },
            "no child, no channel: the answer is the not-connected reason, silently"
        );
    }
    assert!(spawner.started().is_empty(), "no child was started for it");
}
