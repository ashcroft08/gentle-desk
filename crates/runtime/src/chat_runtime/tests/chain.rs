//! Saving ONE profile's fallback chain over the chat's own child (profiles-editor
//! T5, S2, S3, L30-3).
//!
//! The write is a `set_config` of the WHOLE `profile_fallbacks` document, so what
//! these tests pin is the document that travels: the store's `value` as the child
//! sent it - an unknown document member and another profile's chain included -
//! with only `profiles.<profile>.<agent>` replaced, and the precondition taken
//! from THAT store's `writeTargets[global].writeRevision`. Then the same three
//! reads a plain read makes, so the tab draws the truth the store now holds.
//!
//! Nothing here starts a child: the scripted fake answers the channel, so the
//! wire itself - ids, order and bodies - is what is asserted.

use std::sync::{Arc, Mutex};

use gentle_app::ChatCommand;
use gentle_core::rpc::config_store::{ChainInvalid, FallbackStep, SetConfigOutcome};
use gentle_core::rpc::SCHEMA;
use serde_json::{json, Value};

use super::fake::{FakeSpawner, Reply, Script};
use super::profile::{answering, stored_chat, wire};
use crate::chat_runtime::{ChainWriteResult, ChatRuntime, Outcome};

/// The fallbacks document the scripted child stores: an unknown member of the
/// document's own, two profiles, and - for the profile being edited - the
/// profile-wide chain, one agent's chain and an unknown member inside a step of
/// the chain that is NOT edited.
fn document() -> Value {
    json!({
        "kind": "gentle-pi.profile_model_fallbacks",
        "version": 1,
        "future": {"seed": true},
        "profiles": {
            "team": {
                "*": [{"model": "openai/gpt-shared"}],
                "worker": [{"model": "openai/gpt-fallback", "thinking": "low"}]
            },
            "busy": {
                "*": [{"model": "openai/gpt-busy", "futureStep": {"weight": 2}}]
            }
        }
    })
}

/// The profiles store, only ever re-read here: it needs to be a document the
/// reader accepts, and it holds the profile whose chain is edited.
fn profiles_document() -> Value {
    json!({
        "kind": "gentle-pi.agent_model_profiles",
        "version": 1,
        "active": "team",
        "profiles": {
            "team": {"orchestrator": {"model": "p/m"}, "worker": {"model": "p/w"}},
            "busy": {"orchestrator": {"model": "p/b"}}
        }
    })
}

/// The `get_config` answer of one store, as the child holds it.
fn store_answer(store: &str, value: &Value, revision: &str, write: &str) -> Value {
    match store {
        "profiles" => json!({
            "store": "profiles",
            "scope": "global",
            "readOnly": false,
            "status": "ok",
            "revision": "sha256:profiles-read",
            "value": value,
            "layers": [{ "source": "global", "status": "ok" }],
            "writeTargets": [{
                "target": "global",
                "status": "ok",
                "writeRevision": "sha256:profiles-write",
            }],
        }),
        "profile_fallbacks" => json!({
            "store": "profile_fallbacks",
            "scope": "global",
            "readOnly": false,
            "status": "ok",
            "revision": revision,
            "value": value,
            "layers": [{ "source": "global", "status": "ok" }],
            "writeTargets": [{
                "target": "global",
                "status": "ok",
                "writeRevision": write,
            }],
        }),
        "pins" => json!({
            "store": "pins",
            "scope": "local+repo",
            "readOnly": false,
            "status": "ok",
            "revision": "sha256:pins",
            "value": {"resolved": false, "invalid": [], "stale": []},
            "layers": [],
            "writeTargets": [],
        }),
        other => panic!("the save never asks {other:?}"),
    }
}

/// The `set_config` answer: the store after the write, the target and the token
/// the next write needs, as the channel documents it.
fn written_answer(value: &Value) -> Value {
    let mut answer = store_answer(
        "profile_fallbacks",
        value,
        "sha256:fallbacks-after",
        "sha256:fallbacks-next",
    );
    answer["writeTarget"] = json!("global");
    answer["writeRevision"] = json!("sha256:fallbacks-next");
    answer["changed"] = json!(true);
    answer
}

/// The child a chain save talks to: the script it answers with, the fallbacks
/// document it stores, and every `set_config` params it was given.
type ScriptedChild = (Script, Arc<Mutex<Value>>, Arc<Mutex<Vec<Value>>>);

/// A child whose fallbacks store IS `stored`: `get_config` answers it as it
/// stands, `set_config` records the params and stores `data` (or refuses with
/// `refusal`). The other two stores answer their documented shape.
fn scripted(stored: Value, refusal: Option<&'static str>) -> ScriptedChild {
    let fallbacks = Arc::new(Mutex::new(stored));
    let profiles = Arc::new(Mutex::new(profiles_document()));
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let held = Arc::clone(&fallbacks);
    let definitions = Arc::clone(&profiles);
    let seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": ["capabilities", "get_config", "set_config"],
            "events": ["config_changed"],
        })),
        "get_config" => Reply::Data(match params["store"].as_str() {
            Some("profiles") => store_answer(
                "profiles",
                &definitions.lock().unwrap(),
                "sha256:profiles-read",
                "sha256:profiles-write",
            ),
            Some("profile_fallbacks") => store_answer(
                "profile_fallbacks",
                &held.lock().unwrap(),
                "sha256:fallbacks-read",
                "sha256:fallbacks-write",
            ),
            Some("pins") => store_answer("pins", &Value::Null, "sha256:pins", "sha256:none"),
            other => panic!("the save never asks {other:?}"),
        }),
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
    (script, fallbacks, writes)
}

/// The chat these tests write for, already open on the scripted child.
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

/// One chain as the owner writes it.
fn steps(pairs: &[(&str, Option<&str>)]) -> Vec<FallbackStep> {
    pairs
        .iter()
        .map(|(model, thinking)| FallbackStep {
            model: (*model).to_owned(),
            thinking: thinking.map(str::to_owned),
        })
        .collect()
}

/// The command one test sends: the profile and the agent key are the store's own.
fn save(
    chat: &gentle_app::ChatId,
    profile: &str,
    agent: &str,
    chain: Vec<FallbackStep>,
) -> ChatCommand {
    ChatCommand::SetChatProfileChain {
        chat: chat.clone(),
        profile: profile.to_owned(),
        agent: agent.to_owned(),
        steps: chain,
    }
}

/// The result of one `exit` of the chain command.
fn result(outcome: &Outcome) -> &ChainWriteResult {
    match outcome {
        Outcome::ChatChain { result, .. } => result,
        other => panic!("expected a chain write, got {other:?}"),
    }
}

/// The stores one chain save asked, in order, as `method:store` pairs.
fn asked_stores(spawner: &FakeSpawner) -> Vec<String> {
    let (_chat, key) = stored_chat();
    wire(spawner, &key)
        .iter()
        .map(|asked| match asked.method.as_str() {
            "get_config" => format!(
                "get_config:{}",
                asked.params["store"].as_str().unwrap_or_default()
            ),
            other => other.to_owned(),
        })
        .collect()
}

#[test]
fn the_edited_chain_is_the_only_member_that_changes_and_the_token_is_the_write_one() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(
        &chat,
        "team",
        "worker",
        steps(&[("openai/gpt-next", Some("high")), ("openai/gpt-last", None)]),
    ));

    assert_eq!(
        result(&outcome),
        &ChainWriteResult::Saved,
        "the documented ok answer is a save: {outcome:?}"
    );
    let sent = writes.lock().unwrap().clone();
    assert_eq!(sent.len(), 1, "one write, and only one");
    assert_eq!(
        sent[0]["store"],
        json!("profile_fallbacks"),
        "the store the chains live in is the fallbacks one"
    );
    assert_eq!(
        sent[0]["expectedRevision"],
        json!("sha256:fallbacks-write"),
        "S3: the precondition is this store's writeTargets[global].writeRevision"
    );
    assert_ne!(
        sent[0]["expectedRevision"],
        json!("sha256:fallbacks-read"),
        "the store's own revision is never the token"
    );
    assert_ne!(
        sent[0]["expectedRevision"],
        json!("sha256:profiles-write"),
        "and another store's token is not this store's"
    );
    let mut expected = document();
    expected["profiles"]["team"]["worker"] = json!([
        {"model": "openai/gpt-next", "thinking": "high"},
        {"model": "openai/gpt-last"}
    ]);
    assert_eq!(
        sent[0]["data"], expected,
        "every other member is the child's own, and one chain is replaced (S2)"
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

    runtime.execute(save(
        &chat,
        "team",
        "worker",
        steps(&[("openai/gpt-next", None)]),
    ));

    assert_eq!(
        asked_stores(&spawner),
        [
            "capabilities",
            "get_config:profile_fallbacks",
            "set_config",
            "get_config:profiles",
            "get_config:profile_fallbacks",
            "get_config:pins"
        ],
        "one read to build the write, the write, then the read of all three stores"
    );
    let (_chat, key) = stored_chat();
    let ids: Vec<String> = wire(&spawner, &key)
        .iter()
        .map(|asked| asked.id.clone())
        .collect();
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
fn the_reread_shows_the_chain_the_store_now_holds() {
    let (script, _stored, _writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(
        &chat,
        "team",
        "*",
        steps(&[("openai/gpt-next", Some("medium"))]),
    ));

    let Outcome::ChatChain { read, .. } = outcome else {
        panic!("expected the chain write");
    };
    let crate::chat_runtime::StoreRead::Read(fallbacks) = read.fallbacks else {
        panic!("the re-read answers the fallbacks store: {read:?}");
    };
    assert_eq!(
        fallbacks.profiles["team"]["*"][0].model, "openai/gpt-next",
        "the chain the tab draws is the one the child stored"
    );
    assert_eq!(
        fallbacks.profiles["team"]["*"][0].thinking.as_deref(),
        Some("medium")
    );
    assert_eq!(
        fallbacks.profiles["team"]["worker"][0].model, "openai/gpt-fallback",
        "the profile's other chain came back with the document"
    );
}

#[test]
fn the_profile_wide_chain_is_written_like_any_other() {
    let (script, stored, writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(
        &chat,
        "team",
        "*",
        steps(&[
            ("openai/gpt-shared", Some("high")),
            ("openai/gpt-second", None),
        ]),
    ));

    assert_eq!(result(&outcome), &ChainWriteResult::Saved, "{outcome:?}");
    assert_eq!(
        writes.lock().unwrap()[0]["data"]["profiles"]["team"]["*"],
        json!([
            {"model": "openai/gpt-shared", "thinking": "high"},
            {"model": "openai/gpt-second"}
        ]),
        "`*` is a chain key like an agent's: the terminal edits it the same way (L30-3)"
    );
    assert_eq!(
        stored.lock().unwrap()["profiles"]["team"]["worker"][0]["thinking"],
        json!("low"),
        "the other chain kept the member this app does not model"
    );
}

#[test]
fn a_profile_the_fallbacks_store_does_not_hold_writes_nothing_at_all() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, "gone", "*", steps(&[("p/m", None)])));

    assert_eq!(result(&outcome), &ChainWriteResult::UnknownProfile);
    assert!(
        writes.lock().unwrap().is_empty(),
        "a profile the store records no chain for is never written"
    );
    assert!(
        !asked_stores(&spawner)
            .iter()
            .any(|asked| asked == "set_config"),
        "no set_config leaves for a profile that does not exist"
    );
    let Outcome::ChatChain { read, .. } = outcome else {
        panic!("expected the chain write");
    };
    assert!(
        matches!(read.profiles, crate::chat_runtime::StoreRead::Read(_)),
        "the tab still re-reads the truth it draws: {read:?}"
    );
}

#[test]
fn an_agent_the_profile_has_no_chain_for_writes_nothing_at_all() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, "team", "reviewer", steps(&[("p/m", None)])));

    assert_eq!(
        result(&outcome),
        &ChainWriteResult::UnknownChain,
        "the view only offers chains the store records; a new key is not invented here"
    );
    assert!(writes.lock().unwrap().is_empty(), "nothing was written");
}

#[test]
fn a_chain_the_owners_rules_refuse_is_never_sent() {
    let nine: Vec<FallbackStep> = (1..=9)
        .map(|index| FallbackStep {
            model: format!("p/model-{index}"),
            thinking: None,
        })
        .collect();
    let cases: Vec<(Vec<FallbackStep>, ChainInvalid)> = vec![
        (nine, ChainInvalid::TooLong { steps: 9 }),
        (
            steps(&[("p/a", None), ("p/a", Some("high"))]),
            ChainInvalid::RepeatedModel {
                model: "p/a".to_owned(),
            },
        ),
        (
            steps(&[("p/a", None), ("", None)]),
            ChainInvalid::MissingModel { index: 1 },
        ),
        (Vec::new(), ChainInvalid::Empty),
    ];

    for (chain, expected) in cases {
        let (script, stored, writes) = scripted(document(), None);
        let (runtime, spawner, chat, _events) = opened(script);

        let outcome = runtime.execute(save(&chat, "team", "worker", chain));

        assert_eq!(
            result(&outcome),
            &ChainWriteResult::Invalid(expected.clone()),
            "{expected}"
        );
        assert!(
            writes.lock().unwrap().is_empty(),
            "{expected}: a chain the owner would refuse is never sent, so an `invalid_data` is never our fault"
        );
        assert!(
            !asked_stores(&spawner)
                .iter()
                .any(|asked| asked == "set_config"),
            "{expected}: no write left the app"
        );
        assert_eq!(
            *stored.lock().unwrap(),
            document(),
            "{expected}: the store is untouched"
        );
    }
}

#[test]
fn a_chain_the_store_already_holds_is_not_written_again() {
    let (script, _stored, writes) = scripted(document(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(
        &chat,
        "team",
        "worker",
        steps(&[("openai/gpt-fallback", Some("low"))]),
    ));

    assert_eq!(
        result(&outcome),
        &ChainWriteResult::Unchanged,
        "the chain is already the stored one: nothing to save, and no notice"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "an equal chain is never written (no double write, no pointless one)"
    );
}

#[test]
fn every_refusal_writes_nothing_more_and_re_reads() {
    for code in ["revision_conflict", "invalid_data", "profile_apply_busy"] {
        let (script, stored, writes) = scripted(document(), Some(code));
        let (runtime, spawner, chat, _events) = opened(script);

        let outcome = runtime.execute(save(&chat, "team", "worker", steps(&[("p/new", None)])));

        let ChainWriteResult::Failed(failure) = result(&outcome) else {
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
            *stored.lock().unwrap(),
            document(),
            "{code}: nothing was written"
        );
        assert_eq!(
            asked_stores(&spawner)
                .iter()
                .filter(|asked| asked.starts_with("get_config:"))
                .count(),
            4,
            "{code}: the chat is re-read after the attempt"
        );
        let Outcome::ChatChain { read, .. } = outcome else {
            panic!("expected the chain write");
        };
        assert!(
            matches!(read.fallbacks, crate::chat_runtime::StoreRead::Read(_)),
            "{code}: the failure still re-reads the truth the tab shows"
        );
    }
}

#[test]
fn a_child_that_does_not_advertise_set_config_is_never_written() {
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "methods": ["capabilities", "get_config"],
        })),
        "get_config" => Reply::Data(match params["store"].as_str() {
            Some("profiles") => store_answer(
                "profiles",
                &profiles_document(),
                "sha256:profiles-read",
                "sha256:profiles-write",
            ),
            Some("profile_fallbacks") => store_answer(
                "profile_fallbacks",
                &document(),
                "sha256:fallbacks-read",
                "sha256:fallbacks-write",
            ),
            Some("pins") => store_answer("pins", &Value::Null, "sha256:pins", "sha256:none"),
            other => panic!("a store is always named: {other:?}"),
        }),
        other => panic!("the save never asks {other}"),
    });
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, "team", "worker", steps(&[("p/new", None)])));

    assert_eq!(
        result(&outcome),
        &ChainWriteResult::failed(SetConfigOutcome::Unsupported),
        "an older child is not guessed at (support is never inferred)"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "nothing was written to a child that never advertised the method"
    );
}

#[test]
fn a_chat_with_no_child_writes_nothing_and_answers_the_not_connected_reason() {
    let (runtime, spawner, _events) = answering(Arc::new(|method: &str, _params: &Value| {
        panic!("a chat with no child is never asked {method}")
    }));
    let chat = gentle_app::ChatId::fresh();

    let outcome = runtime.execute(save(&chat, "team", "worker", steps(&[("p/new", None)])));

    assert_eq!(
        result(&outcome),
        &ChainWriteResult::failed(SetConfigOutcome::NoAnswer(
            gentle_core::rpc::NoAnswer::NotConnected
        ))
    );
    assert!(spawner.started().is_empty(), "no child was started for it");
    let Outcome::ChatChain { read, .. } = outcome else {
        panic!("expected the chain write");
    };
    assert_eq!(
        *read,
        crate::chat_runtime::ChatConfigRead::unasked(),
        "the re-read of a chat with no child is nothing read, not an error"
    );
}
