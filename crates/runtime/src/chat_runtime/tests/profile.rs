//! Reading one chat's profile configuration over its own child (profiles-editor
//! T2b-1, S9): capabilities once, then `get_config` for `profiles`,
//! `profile_fallbacks` and `pins`, each typed from the answer the channel
//! documents, and each store's own answer kept apart from the other two.
//!
//! Nothing here starts a child: the scripted fake spawner answers the channel,
//! so the wire itself - the request ids, their order and their exact bodies - is
//! what these tests assert.

use std::sync::Arc;

use gentle_app::ChatCommand;
use gentle_backend::chat_host::{
    ChatHost, ChatKey, Clock, Limits, ManualClock, Spawner, TaggedEvent,
};
use gentle_core::rpc::{GetConfigOutcome, NoAnswer, COMMAND, SCHEMA};
use serde_json::{json, Value};

use super::fake::{FakeSpawner, Reply, Script};
use super::{closing, open};
use crate::chat_runtime::{ChatConfigRead, ChatRuntime, Outcome, StoreRead};

/// One interactive host whose children answer `script`, plus the chat event
/// stream kept alive: the desktop channel observes a child's frames on that
/// stream, so a test that drops it gets one answer and no more.
pub(super) fn answering(
    script: Script,
) -> (
    ChatRuntime,
    Arc<FakeSpawner>,
    std::sync::mpsc::Receiver<TaggedEvent>,
) {
    let spawner = Arc::new(FakeSpawner::default().answering(script));
    let host = ChatHost::with_parts(
        Limits::default(),
        Arc::clone(&spawner) as Arc<dyn Spawner>,
        Arc::new(ManualClock::new()) as Arc<dyn Clock>,
    );
    let (runtime, events) = ChatRuntime::new(host);
    (runtime, spawner, events)
}

/// The stored chat these tests read: a session path, so the host knows its key
/// before anything is opened.
pub(super) fn stored_chat() -> (gentle_app::ChatId, ChatKey) {
    (
        gentle_app::ChatId::of_session("s.jsonl"),
        ChatKey::Saved("s.jsonl".into()),
    )
}

/// The frames the pure core pinned (profiles-editor T2a), read here so the
/// runtime is tested against the SAME doc-quoted answers.
fn fixtures() -> Value {
    serde_json::from_str(include_str!(
        "../../../../gentle-core/tests/fixtures/desktop-rpc-profiles.json"
    ))
    .expect("the fixtures are JSON")
}

/// What the child answers `capabilities` with: the slice this read needs.
fn capabilities() -> Value {
    json!({
        "schema": SCHEMA,
        "command": "gentle:desktop",
        "widget": "gentle-desktop-rpc",
        "methods": ["capabilities", "get_config", "cancel_subagent"],
        "events": ["config_changed"],
    })
}

/// The `get_config` answer for one store, built from the fixtures' own values:
/// `profiles` reuses the documented projection whole, the other two wrap the
/// documented documents in a projection of their own store.
fn store_data(fixtures: &Value, store: &str) -> Value {
    match store {
        "profiles" => fixtures["get_config_profiles"]["data"].clone(),
        "profile_fallbacks" => json!({
            "store": "profile_fallbacks",
            "scope": "global",
            "readOnly": false,
            "status": "ok",
            "revision": "sha256:2b0c9e0f2f5a4b1c8d7e6f5a4b3c2d1e",
            "value": fixtures["fallbacks_data"],
            "layers": [{ "source": "global", "status": "ok" }],
            "writeTargets": [{
                "target": "global",
                "status": "ok",
                "writeRevision": "sha256:8f2a0c5e2d7b4319a6f80e1c93b4d271",
            }],
        }),
        "pins" => json!({
            "store": "pins",
            "scope": "local+repo",
            "readOnly": false,
            "status": "ok",
            "revision": fixtures["pins_value"]["revision"],
            "value": fixtures["pins_value"]["value"],
            "layers": fixtures["pins_value"]["layers"],
            "writeTargets": [
                {
                    "target": "local",
                    "status": "ok",
                    "writeRevision": "sha256:1b1f0d6e9a3c47f2b8d5e0a7c4f19b62",
                },
                {
                    "target": "repo",
                    "status": "ok",
                    "writeRevision": "sha256:2c2a1e7f0b4d49381a6e5c2f9d7b30a4",
                },
            ],
        }),
        other => panic!("no documented answer for the {other} store"),
    }
}

/// A child that answers the whole read the way the channel documents it.
fn documented() -> Script {
    let fixtures = fixtures();
    Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(capabilities()),
        "get_config" => Reply::Data(store_data(
            &fixtures,
            params["store"].as_str().unwrap_or_default(),
        )),
        // A stop of a helper answers its own method; the S8 test uses it.
        "cancel_subagent" => Reply::Data(json!({
            "agent": { "id": "h1", "agent": "worker", "status": "cancelled" },
        })),
        other => panic!("the read never asks {other}"),
    })
}

/// One desktop request as it went out: its id, its method and its params.
#[derive(Debug, PartialEq)]
pub(super) struct Asked {
    pub(super) id: String,
    pub(super) method: String,
    pub(super) params: Value,
}

impl Asked {
    pub(super) fn of(id: &str, method: &str, params: Value) -> Self {
        Self {
            id: id.to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

/// Every desktop request the child of `key` was written, in order.
pub(super) fn wire(spawner: &FakeSpawner, key: &ChatKey) -> Vec<Asked> {
    spawner
        .written_to(key)
        .iter()
        .filter_map(|command| {
            let message = command.prompt_text()?;
            let envelope = message.strip_prefix(COMMAND)?.trim_start();
            let request: Value = serde_json::from_str(envelope).ok()?;
            Some(Asked {
                id: request["requestId"].as_str()?.to_owned(),
                method: request["method"].as_str()?.to_owned(),
                params: request["params"].clone(),
            })
        })
        .collect()
}

#[test]
fn the_read_asks_capabilities_once_then_the_three_stores_in_order() {
    let (runtime, spawner, _events) = answering(documented());
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    let read = runtime.read_chat_profile(&chat);

    assert!(
        matches!(read.profiles, StoreRead::Read(_)),
        "the documented profiles answer reads: {read:?}"
    );
    assert_eq!(
        wire(&spawner, &key),
        vec![
            Asked::of("desktop-1", "capabilities", json!({})),
            Asked::of("desktop-2", "get_config", json!({ "store": "profiles" })),
            Asked::of(
                "desktop-3",
                "get_config",
                json!({ "store": "profile_fallbacks" })
            ),
            Asked::of("desktop-4", "get_config", json!({ "store": "pins" })),
        ],
        "capabilities once, then the three stores in order, each exactly {{ store }}"
    );
}

#[test]
fn each_store_is_typed_from_its_documented_answer() {
    let (runtime, _spawner, _events) = answering(documented());
    let (chat, _key) = stored_chat();
    runtime.execute(open(&chat));

    let outcome = runtime.execute(ChatCommand::ReadChatProfile { chat: chat.clone() });

    let Outcome::ChatProfile {
        chat: answered,
        read,
    } = outcome
    else {
        panic!("expected the chat's profile read, got {outcome:?}");
    };
    assert_eq!(answered, chat);
    let StoreRead::Read(profiles) = read.profiles else {
        panic!("profiles: {:?}", read.profiles);
    };
    assert_eq!(profiles.active.as_deref(), Some("team"));
    assert_eq!(
        profiles.profiles["team"]["explore"].model.as_deref(),
        Some("anthropic/claude-sonnet-4"),
        "the agent's routing row is typed, not left as JSON"
    );
    let StoreRead::Read(fallbacks) = read.fallbacks else {
        panic!("fallbacks: {:?}", read.fallbacks);
    };
    assert_eq!(
        fallbacks.profiles["team"]["worker"][0].model,
        "openai/gpt-fallback"
    );
    assert_eq!(
        fallbacks.profiles["team"]["worker"][0].thinking.as_deref(),
        Some("low")
    );
    let StoreRead::Read(pins) = read.pins else {
        panic!("pins: {:?}", read.pins);
    };
    assert!(pins.value.resolved);
    let winner = pins
        .value
        .winner
        .clone()
        .expect("a resolved pin has a winner");
    assert_eq!(winner.source.as_str(), "repo");
    assert_eq!(winner.profile.as_deref(), Some("team"));
    assert_eq!(
        pins.slot(gentle_core::rpc::config_store::PinSource::Repo),
        crate::chat_runtime::PinSlot::Pinned("team".to_owned()),
        "the layer that decides here, and the pin it holds"
    );
    assert_eq!(
        pins.layers.len(),
        2,
        "both pin files the worktree has, so a pin may be edited per layer (T6)"
    );
    assert!(
        pins.writable(gentle_core::rpc::config_store::PinSource::Local)
            && pins.writable(gentle_core::rpc::config_store::PinSource::Repo),
        "both layers offer a write target inside a resolvable worktree"
    );
}

#[test]
fn a_store_that_refuses_leaves_the_other_two_intact() {
    let fixtures = fixtures();
    let (runtime, spawner, _events) = answering(Arc::new(move |method: &str, params: &Value| {
        match method {
            "capabilities" => Reply::Data(capabilities()),
            // The owner refuses the profiles store: a REAL answer about one
            // store, so it must not stop the other two.
            "get_config" if params["store"] == "profiles" => Reply::Refused {
                code: "profiles_unavailable",
            },
            "get_config" => Reply::Data(store_data(
                &fixtures,
                params["store"].as_str().unwrap_or_default(),
            )),
            other => panic!("the read never asks {other}"),
        }
    }));
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    let read = runtime.read_chat_profile(&chat);

    let StoreRead::Unread(GetConfigOutcome::Rejected(error)) = read.profiles else {
        panic!("expected the typed refusal, got {:?}", read.profiles);
    };
    assert_eq!(error.code, "profiles_unavailable");
    assert!(
        matches!(read.fallbacks, StoreRead::Read(_)),
        "the refusal of one store leaves the next one read: {:?}",
        read.fallbacks
    );
    assert!(matches!(read.pins, StoreRead::Read(_)));
    assert_eq!(
        wire(&spawner, &key).len(),
        4,
        "a refusal is an answer, so all three stores were still asked"
    );
}

#[test]
fn the_first_no_answer_skips_the_remaining_stores() {
    let (runtime, spawner, _events) = answering(documented());
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));
    // The child goes away and the key stays: the chat is known, nothing is live.
    runtime.execute(closing(&chat));

    let read = runtime.read_chat_profile(&chat);

    assert!(
        matches!(
            read.profiles,
            StoreRead::Unread(GetConfigOutcome::NoAnswer(_))
        ),
        "a closed child is a no-answer, not a refusal: {:?}",
        read.profiles
    );
    assert_eq!(
        read.fallbacks,
        StoreRead::Skipped,
        "one no-answer must cost ONE deadline, so the rest are never asked"
    );
    assert_eq!(read.pins, StoreRead::Skipped);
    assert!(
        wire(&spawner, &key).is_empty(),
        "an answerless child is not written three times"
    );
}

#[test]
fn a_chat_the_host_never_opened_answers_without_writing_anything() {
    let (runtime, spawner, _events) = answering(documented());
    let chat = gentle_app::ChatId::fresh();

    let read = runtime.read_chat_profile(&chat);

    assert_eq!(
        read,
        ChatConfigRead::unasked(),
        "no child, nothing asked: every store answers the same not-connected"
    );
    assert_eq!(
        read.profiles,
        StoreRead::Unread(GetConfigOutcome::NoAnswer(NoAnswer::NotConnected))
    );
    assert!(spawner.started().is_empty(), "no child was started for it");
}

#[test]
fn one_stop_and_one_profile_read_share_the_chats_request_counter() {
    let (runtime, spawner, _events) = answering(documented());
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));

    assert!(
        runtime.stop_helper(&chat, "h1"),
        "the documented cancel is accepted"
    );
    let read = runtime.read_chat_profile(&chat);
    assert!(matches!(read.profiles, StoreRead::Read(_)), "{read:?}");

    let asked = wire(&spawner, &key);
    let ids: Vec<&str> = asked.iter().map(|asked| asked.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "desktop-1",
            "desktop-2",
            "desktop-3",
            "desktop-4",
            "desktop-5"
        ],
        "S8: one chat, ONE counter - the stop and the read never collide"
    );
    let methods: Vec<&str> = asked.iter().map(|asked| asked.method.as_str()).collect();
    assert_eq!(
        methods,
        [
            "capabilities",
            "cancel_subagent",
            "get_config",
            "get_config",
            "get_config"
        ],
        "capabilities is asked once: both requests share the chat's one client"
    );
}

#[test]
fn focusing_an_open_chat_keeps_the_counter_growing_across_reads() {
    let (runtime, spawner, _events) = answering(documented());
    let (chat, key) = stored_chat();
    runtime.execute(open(&chat));
    let first = runtime.read_chat_profile(&chat);
    assert!(matches!(first.profiles, StoreRead::Read(_)), "{first:?}");

    // Clicking the chat again only focuses it: the same child answers, so the
    // client - its counter and the capabilities it cached - stays (#184).
    assert_eq!(
        runtime.execute(open(&chat)),
        Outcome::Opened {
            chat: chat.clone(),
            how: gentle_backend::chat_host::Opened::Existing,
        }
    );
    let second = runtime.read_chat_profile(&chat);
    assert!(matches!(second.profiles, StoreRead::Read(_)), "{second:?}");

    let asked = wire(&spawner, &key);
    let ids: Vec<&str> = asked.iter().map(|asked| asked.id.as_str()).collect();
    assert_eq!(
        ids,
        (1..=7)
            .map(|number| format!("desktop-{number}"))
            .collect::<Vec<String>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>(),
        "a focus-only open restarts neither the counter nor the capabilities"
    );
    assert_eq!(
        asked
            .iter()
            .filter(|asked| asked.method == "capabilities")
            .count(),
        1,
        "capabilities belongs to the child that reported it, and that child stayed"
    );
}
