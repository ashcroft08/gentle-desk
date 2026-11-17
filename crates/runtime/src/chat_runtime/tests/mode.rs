//! Saving ONE profile's orchestration mode over the chat's own child
//! (profiles-editor T3-mode, S2, S3, L22, L24).
//!
//! The write is a `set_config` of the WHOLE profiles document, so what these
//! tests pin is the document that travels: the store's `value` as the child sent
//! it - an unknown member included - with only `workflow.<profile>` changed, and
//! the precondition taken from the read's `writeTargets[global].writeRevision`.
//! Then the same three reads a plain read makes, so the tab draws the truth the
//! store now holds instead of what the page hoped for.
//!
//! Nothing here starts a child: the scripted fake answers the channel, so the
//! wire itself - ids, order and bodies - is what is asserted.

use std::sync::{Arc, Mutex};

use gentle_app::ChatCommand;
use gentle_core::rpc::config_store::{OrchestratorMode, SetConfigOutcome};
use gentle_core::rpc::{NoAnswer, SCHEMA};
use serde_json::{json, Value};

use super::fake::{FakeSpawner, Reply, Script};
use super::profile::{answering, stored_chat, wire, Asked};
use crate::chat_runtime::{ChatRuntime, ModeWriteResult, Outcome};

/// The profile document the scripted child stores: one unknown member of its
/// own, an unknown member of a routing row, two profiles, and a `workflow` entry
/// for the one that is not being switched.
fn document() -> Value {
    json!({
        "kind": "gentle-pi.agent_model_profiles",
        "version": 1,
        "active": "team",
        "future": {"seed": true},
        "profiles": {
            "team": {
                "orchestrator": {"model": "p/m", "future": "kept"},
                "worker": {"model": "p/w", "tier": "cheap"}
            },
            "busy": {"orchestrator": {"model": "p/b"}}
        },
        "workflow": {"busy": {"orchestratorMode": "full-delegation"}}
    })
}

/// The `get_config` answer of the profiles store, as the child holds it.
fn profiles_answer(document: &Value) -> Value {
    json!({
        "store": "profiles",
        "scope": "global",
        "readOnly": false,
        "status": "ok",
        "revision": "sha256:read",
        "value": document,
        "layers": [{ "source": "global", "status": "ok" }],
        "writeTargets": [{
            "target": "global",
            "status": "ok",
            "writeRevision": "sha256:write",
        }],
    })
}

/// The `set_config` answer: the store after the write, the target and the token
/// the next write needs, exactly as L24's real child answered it.
fn written_answer(document: &Value) -> Value {
    let mut answer = profiles_answer(document);
    answer["revision"] = json!("sha256:after");
    answer["writeTargets"][0]["writeRevision"] = json!("sha256:next");
    answer["writeTarget"] = json!("global");
    answer["writeRevision"] = json!("sha256:next");
    answer["changed"] = json!(true);
    answer
}

/// The child a mode save talks to: the script it answers with, the document it
/// stores, and every `set_config` params it was given.
type ScriptedChild = (Script, Arc<Mutex<Value>>, Arc<Mutex<Vec<Value>>>);

/// A child whose profiles store IS `stored`: `get_config` answers it as it
/// stands, `set_config` records the params and stores `data` (or refuses with
/// `refusal`). The other two stores answer their documented shape.
fn scripted(stored: Value, refusal: Option<&'static str>) -> ScriptedChild {
    let document = Arc::new(Mutex::new(stored));
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let held = Arc::clone(&document);
    let seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": ["capabilities", "get_config", "set_config"],
            "events": ["config_changed"],
        })),
        "get_config" => match params["store"].as_str() {
            Some("profiles") => Reply::Data(profiles_answer(&held.lock().unwrap())),
            Some("profile_fallbacks") => Reply::Data(json!({
                "store": "profile_fallbacks",
                "scope": "global",
                "readOnly": false,
                "status": "ok",
                "revision": "sha256:fallbacks",
                "value": {"kind": "gentle-pi.profile_model_fallbacks", "version": 1, "profiles": {}},
                "layers": [{ "source": "global", "status": "ok" }],
                "writeTargets": [{
                    "target": "global",
                    "status": "ok",
                    "writeRevision": "sha256:fallbacks-write",
                }],
            })),
            Some("pins") => Reply::Data(json!({
                "store": "pins",
                "scope": "local+repo",
                "readOnly": false,
                "status": "ok",
                "revision": "sha256:pins",
                "value": {"resolved": false, "invalid": [], "stale": []},
                "layers": [],
                "writeTargets": [],
            })),
            other => panic!("the save never asks {other:?}"),
        },
        "set_config" => {
            seen.lock().unwrap().push(params.clone());
            if let Some(code) = refusal {
                return Reply::Refused { code };
            }
            let data = params["data"].clone();
            *held.lock().unwrap() = data.clone();
            Reply::Data(written_answer(&data))
        }
        other => panic!("the save never asks {other}"),
    });
    (script, document, writes)
}

/// The chat these tests write for, already open on the scripted child. The event
/// stream is handed back because the desktop channel observes a child's frames on
/// it: a test that drops it gets one answer and no more.
fn opened(
    script: Script,
) -> (
    ChatRuntime,
    Arc<FakeSpawner>,
    gentle_app::ChatId,
    std::sync::mpsc::Receiver<gentle_backend::chat_host::TaggedEvent>,
) {
    let (runtime, spawner, events) = answering(script);
    let (chat, _key) = stored_chat();
    runtime.execute(super::open(&chat));
    (runtime, spawner, chat, events)
}

/// The result of one `exit` of the mode command.
fn result(outcome: &Outcome) -> &ModeWriteResult {
    match outcome {
        Outcome::ChatMode { result, .. } => result,
        other => panic!("expected a mode write, got {other:?}"),
    }
}

#[test]
fn full_delegation_sends_only_that_profiles_entry_and_the_write_revision() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert_eq!(
        result(&outcome),
        &ModeWriteResult::Saved,
        "the documented ok answer is a save: {outcome:?}"
    );
    let sent = writes.lock().unwrap().clone();
    assert_eq!(sent.len(), 1, "one write, and only one");
    assert_eq!(
        sent[0]["store"],
        json!("profiles"),
        "the store the mode lives in is the whole document's"
    );
    assert_eq!(
        sent[0]["expectedRevision"],
        json!("sha256:write"),
        "S3: the precondition is writeTargets[global].writeRevision, never revision"
    );
    assert_ne!(
        sent[0]["expectedRevision"],
        json!("sha256:read"),
        "the store's own revision is never the token"
    );
    let mut expected = document();
    expected["workflow"]["team"] = json!({"orchestratorMode": "full-delegation"});
    assert_eq!(
        sent[0]["data"], expected,
        "every other member is the child's own, and one entry is set (S2)"
    );
    assert_eq!(
        sent[0]
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        ["data", "expectedRevision", "store"],
        "the params are exactly the contract's three"
    );
}

#[test]
fn the_write_is_followed_by_the_same_three_reads_a_plain_read_makes() {
    let (script, _stored, _writes) = scripted(document(), None);
    let (runtime, spawner, chat, _events) = opened(script);

    runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    let (_chat, key) = stored_chat();
    let asked = wire(&spawner, &key);
    let methods: Vec<&str> = asked.iter().map(|asked| asked.method.as_str()).collect();
    assert_eq!(
        methods,
        [
            "capabilities",
            "get_config",
            "set_config",
            "get_config",
            "get_config",
            "get_config"
        ],
        "one read to build the write, the write, then the read of all three stores"
    );
    let stores: Vec<&str> = asked
        .iter()
        .filter(|asked| asked.method == "get_config")
        .map(|asked| asked.params["store"].as_str().unwrap_or_default())
        .collect();
    assert_eq!(
        stores,
        ["profiles", "profiles", "profile_fallbacks", "pins"],
        "the re-read is ReadChatProfile's own, in its own order"
    );
    let ids: Vec<&str> = asked.iter().map(|asked| asked.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "desktop-1",
            "desktop-2",
            "desktop-3",
            "desktop-4",
            "desktop-5",
            "desktop-6"
        ],
        "S8: the chat's ONE client numbers the read, the write and the re-read"
    );
}

#[test]
fn the_reread_shows_the_mode_the_store_now_holds() {
    let (script, _stored, _writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    let Outcome::ChatMode { read, .. } = outcome else {
        panic!("expected the mode write");
    };
    let crate::chat_runtime::StoreRead::Read(profiles) = read.profiles else {
        panic!("the re-read answers the profiles store: {read:?}");
    };
    assert_eq!(
        profiles.workflow.get("team"),
        Some(&OrchestratorMode::FullDelegation),
        "the mode the tab draws is the one the child stored"
    );
    assert_eq!(
        profiles.workflow.get("busy"),
        Some(&OrchestratorMode::FullDelegation),
        "the other profile's entry came back with the document"
    );
}

#[test]
fn organic_removes_the_entry_and_a_workflow_it_empties() {
    let mut stored = document();
    stored["workflow"] = json!({"team": {"orchestratorMode": "full-delegation"}});
    let (script, document_after, writes) = scripted(stored, None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::Organic,
    });

    assert_eq!(result(&outcome), &ModeWriteResult::Saved, "{outcome:?}");
    let sent = writes.lock().unwrap().clone();
    assert_eq!(
        sent[0]["data"].get("workflow"),
        None,
        "organic is absence, and the emptied workflow is removed (L24)"
    );
    assert_eq!(
        document_after.lock().unwrap().get("workflow"),
        None,
        "the store the child holds lost the key entirely"
    );
}

#[test]
fn a_profile_the_document_does_not_define_writes_nothing_at_all() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "gone".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert_eq!(result(&outcome), &ModeWriteResult::UnknownProfile);
    assert!(
        writes.lock().unwrap().is_empty(),
        "a name the store does not define is never written"
    );
    let (_chat, key) = stored_chat();
    assert_eq!(
        wire(&spawner, &key)
            .iter()
            .filter(|asked| asked.method == "set_config")
            .count(),
        0
    );
    let Outcome::ChatMode { read, .. } = outcome else {
        panic!("expected the mode write");
    };
    assert!(
        matches!(read.profiles, crate::chat_runtime::StoreRead::Read(_)),
        "the tab still re-reads the truth it draws: {read:?}"
    );
}

#[test]
fn a_mode_the_document_already_records_is_not_written_again() {
    let mut stored = document();
    stored["workflow"]["team"] = json!({"orchestratorMode": "full-delegation"});
    let (script, _stored, writes) = scripted(stored, None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert_eq!(
        result(&outcome),
        &ModeWriteResult::Unchanged,
        "the chosen mode is already the stored one: nothing to save, and no notice"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "an equal mode is never written (no double write, no pointless one)"
    );
}

#[test]
fn every_refusal_writes_nothing_more_and_re_reads() {
    for code in ["revision_conflict", "invalid_data", "profile_apply_busy"] {
        let (script, document_after, writes) = scripted(document(), Some(code));
        let (runtime, spawner, chat, _events) = opened(script);

        let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
            chat: chat.clone(),
            profile: "team".to_owned(),
            mode: OrchestratorMode::FullDelegation,
        });

        let ModeWriteResult::Failed(failure) = result(&outcome) else {
            panic!("{code}: expected a typed failure, got {outcome:?}");
        };
        match (code, failure.as_ref()) {
            ("revision_conflict", SetConfigOutcome::RevisionConflict(_))
            | ("invalid_data", SetConfigOutcome::InvalidData(_))
            | ("profile_apply_busy", SetConfigOutcome::ApplyBusy(_)) => {}
            (_, other) => panic!("{code} is not {other:?}"),
        }
        assert_eq!(
            writes.lock().unwrap().len(),
            1,
            "{code}: the refusal is an answer, so it is not retried"
        );
        assert_eq!(
            *document_after.lock().unwrap(),
            document(),
            "{code}: nothing was written"
        );
        let (_chat, key) = stored_chat();
        assert_eq!(
            wire(&spawner, &key)
                .iter()
                .filter(|asked| asked.method == "get_config")
                .count(),
            4,
            "{code}: the chat is re-read after the attempt"
        );
        let Outcome::ChatMode { read, .. } = outcome else {
            panic!("expected the mode write");
        };
        assert!(
            matches!(read.profiles, crate::chat_runtime::StoreRead::Read(_)),
            "{code}: the failure still re-reads the truth the tab shows"
        );
    }
}

#[test]
fn a_child_that_does_not_advertise_set_config_is_never_written() {
    let document_state = Arc::new(Mutex::new(document()));
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let held = Arc::clone(&document_state);
    let _seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "methods": ["capabilities", "get_config"],
        })),
        "get_config" => Reply::Data(match params["store"].as_str() {
            Some("profiles") => profiles_answer(&held.lock().unwrap()),
            Some(store) => json!({
                "store": store,
                "scope": "global",
                "readOnly": false,
                "status": "ok",
                "revision": "sha256:other",
                "layers": [],
            }),
            None => panic!("a store is always named"),
        }),
        other => panic!("the save never asks {other}"),
    });
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert_eq!(
        result(&outcome),
        &ModeWriteResult::failed(SetConfigOutcome::Unsupported),
        "an older child is not guessed at (support is never inferred)"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "nothing was written to a child that never advertised the method"
    );
    let (_chat, key) = stored_chat();
    assert!(
        wire(&spawner, &key)
            .iter()
            .all(|asked| asked.method != "set_config"),
        "the method is never sent when capabilities do not list it"
    );
}

#[test]
fn a_chat_with_no_child_writes_nothing_and_answers_the_not_connected_reason() {
    let (runtime, spawner, _events) = answering(Arc::new(|method: &str, _params: &Value| {
        panic!("a chat with no child is never asked {method}")
    }));
    let chat = gentle_app::ChatId::fresh();

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert_eq!(
        result(&outcome),
        &ModeWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected))
    );
    assert!(spawner.started().is_empty(), "no child was started for it");
    let Outcome::ChatMode { read, .. } = outcome else {
        panic!("expected the mode write");
    };
    assert_eq!(
        *read,
        crate::chat_runtime::ChatConfigRead::unasked(),
        "the re-read of a chat with no child is nothing read, not an error"
    );
}

#[test]
fn a_child_that_goes_away_mid_exchange_stops_reading_and_writes_nothing_further() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, spawner, chat, _events) = opened(script);
    // The child goes away and its key stays: the chat is known, nothing is live.
    runtime.execute(super::closing(&chat));

    let outcome = runtime.execute(ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    });

    assert!(
        matches!(result(&outcome), ModeWriteResult::Failed(_)),
        "a closed child is a no-answer, not a refusal: {outcome:?}"
    );
    assert!(writes.lock().unwrap().is_empty(), "nothing was written");
    let (_chat, key) = stored_chat();
    assert!(
        wire(&spawner, &key)
            .iter()
            .all(|asked: &Asked| asked.method != "set_config"),
        "one deadline, and no write after it"
    );
    let Outcome::ChatMode { read, .. } = outcome else {
        panic!("expected the mode write");
    };
    assert!(
        matches!(read.profiles, crate::chat_runtime::StoreRead::Skipped),
        "one deadline: the dead child ends the batch, so the re-read is skipped, not retried: {read:?}"
    );
}
