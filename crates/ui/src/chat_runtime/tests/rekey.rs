//! The session file a new chat reports: how it is read out of a `get_state`
//! reply, and the move it asks for in the registry and in the host.
//!
//! `gentle-core` keeps reply data as plain JSON, so the field name this shell
//! needs (`sessionFile`, from pi's `RpcSessionState`) is pinned here.

use gentle_app::ChatId;
use gentle_backend::chat_host::ChatKey;
use gentle_core::rpc::{RpcCommand, RpcResponse};

use super::{open, runtime, send};
use crate::chat_runtime::{session_file_of, Outcome};

fn reply(command: &str, data: serde_json::Value) -> RpcResponse {
    RpcResponse {
        id: None,
        command: command.to_owned(),
        outcome: Ok(data),
    }
}

#[test]
fn the_state_reply_names_the_session_file_of_a_new_chat() {
    let data = serde_json::json!({ "sessionFile": "p.jsonl", "sessionId": "s" });

    assert_eq!(session_file_of(&reply("get_state", data)), Some("p.jsonl"));
}

#[test]
fn a_reply_that_names_no_session_file_is_not_one() {
    assert_eq!(
        session_file_of(&reply(
            "get_messages",
            serde_json::json!({ "sessionFile": "p.jsonl" })
        )),
        None,
        "only get_state reports the session of the chat"
    );
    assert_eq!(
        session_file_of(&reply("get_state", serde_json::json!({ "sessionId": "s" }))),
        None
    );
    assert_eq!(
        session_file_of(&reply(
            "get_state",
            serde_json::json!({ "sessionFile": "" })
        )),
        None
    );
    let failed = RpcResponse {
        id: None,
        command: "get_state".to_owned(),
        outcome: Err("nope".to_owned()),
    };
    assert_eq!(session_file_of(&failed), None);
}

#[test]
fn a_new_chat_that_reports_its_session_file_becomes_the_stored_chat() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(open(&chat));
    let started = spawner.started();

    let moved = runtime
        .rekey(&chat, "p.jsonl")
        .expect("a registered new chat rekeys");

    assert_eq!(moved.from, started[0]);
    assert_eq!(moved.to, ChatKey::Saved("p.jsonl".into()));
    assert_eq!(moved.id, ChatId::of_session("p.jsonl"));
    // The host kept the child and moved it to the stored key: a command to the
    // stored chat reaches the same child.
    assert_eq!(runtime.execute(send(&moved.id)), Outcome::Sent);
    assert_eq!(spawner.started(), started, "no second child");
    assert_eq!(spawner.written_to(&started[0]), vec![RpcCommand::GetState]);
}

#[test]
fn only_a_registered_new_chat_rekeys() {
    let (runtime, _spawner) = runtime();

    assert_eq!(runtime.rekey(&ChatId::fresh(), "p.jsonl"), None);
    assert_eq!(runtime.rekey(&ChatId::of_session("a"), "p.jsonl"), None);
}
