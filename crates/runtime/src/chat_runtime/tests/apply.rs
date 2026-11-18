//! Applying one profile over the chat's OWN child (profiles-editor T3b, S4, S8,
//! S9, L30): `apply_profile` with `mode: session` (use it in this chat) and
//! `mode: default` (make it the global default), the screen's own
//! `confirmation_required` round trip, and the answers that write nothing.
//!
//! Two facts decide the shape of what these tests pin:
//!
//! - the apply is a request to the chat's own child, on the chat's ONE desktop
//!   client, so its `requestId` continues the same sequence the read uses (S8);
//! - the answer's own dialog text is the CHILD's, and it is carried verbatim
//!   (L30 ruling 2): nothing here rewrites, translates or re-words it.
//!
//! Nothing here starts a child: the scripted fake answers the channel, so the
//! wire itself - the ids, the order and the exact bodies - is what is asserted.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_app::ChatCommand;
use gentle_core::rpc::profile_apply::{
    ApplyMode, ApplyProfileOutcome, ConfirmationToken, NoteLevel, ProfileConfirmation,
};
use gentle_core::rpc::SCHEMA;
use serde_json::{json, Value};

use super::fake::{FakeSpawner, Reply, Script};
use super::profile::{answering, stored_chat, wire};
use crate::chat_runtime::StoreRead;

use super::{open, ChatRuntime, Outcome};

/// The frames the pure core pinned (profiles-editor T2a), read here so the
/// runtime is tested against the SAME doc-quoted answers.
fn fixtures() -> Value {
    serde_json::from_str(include_str!(
        "../../../../gentle-core/tests/fixtures/desktop-rpc-profiles.json"
    ))
    .expect("the fixtures are JSON")
}

/// The documented `applied` answer, naming the mode the request asked for: the
/// child echoes it, and a `session` apply echoes `session`.
fn applied(mode: ApplyMode) -> Reply {
    let mut data = fixtures()["applied"]["data"].clone();
    data["mode"] = json!(mode.as_str());
    Reply::Data(data)
}

/// The documented `confirmation_required` answer: the screen's own title, its
/// own message and its own token, untouched (L30 ruling 2).
fn asks() -> Reply {
    Reply::Data(fixtures()["confirmation_required"]["data"].clone())
}

/// The profiles store the child holds: the documented document, `active` as the
/// test names it.
fn profiles(active: &str) -> Value {
    let mut value = fixtures()["profiles_data"].clone();
    value["active"] = json!(active);
    value
}

/// One `get_config` answer for `store`, with the profiles store read from
/// `profiles`.
fn store_data(fixtures: &Value, store: &str, profiles: &Value) -> Value {
    match store {
        "profiles" => json!({
            "store": "profiles",
            "scope": "global",
            "readOnly": false,
            "status": "ok",
            "revision": "sha256:profiles",
            "value": profiles,
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
            "revision": "sha256:fallbacks",
            "value": fixtures["fallbacks_data"],
            "layers": [{ "source": "global", "status": "ok" }],
            "writeTargets": [{
                "target": "global",
                "status": "ok",
                "writeRevision": "sha256:fallbacks-write",
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
            "writeTargets": [],
        }),
        other => panic!("no documented answer for the {other} store"),
    }
}

/// Every `apply_profile` params the child was written, in order.
type Written = Arc<Mutex<Vec<Value>>>;

/// A child that answers `apply_profile` with `answers`, in order. `advertise` is
/// whether its contract lists the method at all.
fn scripted(answers: Vec<Reply>) -> (Script, Written) {
    scripted_with(answers, true)
}

fn scripted_with(answers: Vec<Reply>, advertise: bool) -> (Script, Written) {
    let fixtures = fixtures();
    let held = Arc::new(Mutex::new(profiles("team")));
    let queue = Arc::new(Mutex::new(answers.into_iter().collect::<VecDeque<_>>()));
    let written: Written = Arc::new(Mutex::new(Vec::new()));
    let store = Arc::clone(&held);
    let answers = Arc::clone(&queue);
    let seen = Arc::clone(&written);
    let methods = if advertise {
        json!(["capabilities", "get_config", "apply_profile"])
    } else {
        json!(["capabilities", "get_config"])
    };
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": methods,
            "events": ["config_changed", "profile_applied"],
        })),
        "get_config" => Reply::Data(store_data(
            &fixtures,
            params["store"].as_str().unwrap_or_default(),
            &store.lock().unwrap(),
        )),
        "apply_profile" => {
            seen.lock().unwrap().push(params.clone());
            answers
                .lock()
                .unwrap()
                .pop_front()
                .expect("one answer per apply_profile")
        }
        other => panic!("the apply never asks {other}"),
    });
    (script, written)
}

/// The chat these tests apply for, already open on the scripted child.
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
    runtime.execute(open(&chat));
    (runtime, spawner, chat, events)
}

/// The outcome of one apply.
fn applied_by(outcome: &Outcome) -> &crate::chat_runtime::ChatProfileApply {
    match outcome {
        Outcome::ChatApply { attempt, .. } => attempt,
        other => panic!("expected an apply, got {other:?}"),
    }
}

#[test]
fn the_session_apply_asks_that_chats_own_child_and_carries_no_confirmation() {
    let (script, written) = scripted(vec![applied(ApplyMode::Session)]);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    let attempt = applied_by(&outcome);
    assert!(
        matches!(&attempt.result, ApplyProfileOutcome::Applied(apply) if apply.mode == ApplyMode::Session),
        "the documented ok answer is an apply: {attempt:?}"
    );
    assert_eq!(
        written.lock().unwrap().clone(),
        vec![json!({"profile": "team", "mode": "session"})],
        "a first apply carries exactly the profile and the mode (desktop-rpc.md:844)"
    );
    let (_chat, key) = stored_chat();
    assert_eq!(
        wire(&spawner, &key)
            .iter()
            .map(|asked| asked.method.as_str())
            .collect::<Vec<_>>(),
        [
            "capabilities",
            "apply_profile",
            "get_config",
            "get_config",
            "get_config"
        ],
        "the apply, then the same three reads a plain read makes"
    );
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
        "S8: the chat's ONE client numbers the apply and the re-read"
    );
}

#[test]
fn a_session_apply_re_reads_the_profile_configuration_the_tab_draws() {
    let (script, _written) = scripted(vec![applied(ApplyMode::Session)]);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    let Some(read) = &applied_by(&outcome).read else {
        panic!("an apply that ran re-reads the configuration: {outcome:?}");
    };
    assert!(
        matches!(read.profiles, StoreRead::Read(_)),
        "the re-read answers the profiles store: {read:?}"
    );
    assert!(
        matches!(read.fallbacks, StoreRead::Read(_)) && matches!(read.pins, StoreRead::Read(_)),
        "all three, exactly as a plain read reads them"
    );
}

#[test]
fn the_default_apply_answers_the_channels_own_dialog_verbatim() {
    let (script, _written) = scripted(vec![asks()]);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Default,
        confirmation: None,
    });

    let attempt = applied_by(&outcome);
    let ApplyProfileOutcome::ConfirmationRequired(confirmation) = &attempt.result else {
        panic!("the documented answer is the screen's dialog: {attempt:?}");
    };
    let documented = fixtures()["confirmation_required"]["data"]["confirmation"].clone();
    assert_eq!(
        confirmation,
        &ProfileConfirmation {
            profile: "team".to_owned(),
            mode: ApplyMode::Default,
            title: documented["title"].as_str().unwrap_or_default().to_owned(),
            message: documented["message"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            token: ConfirmationToken::parse("3f1c…").expect("the documented token"),
        },
        "L30 ruling 2: the child's title, message and token, exactly as they arrived"
    );
    assert!(
        attempt.read.is_none(),
        "nothing was written, so there is nothing new to read: the tab keeps what it drew"
    );
}

#[test]
fn the_repeat_carries_the_token_the_dialog_issued() {
    let (script, written) = scripted(vec![asks(), applied(ApplyMode::Default)]);
    let (runtime, spawner, chat, _events) = opened(script);

    let first = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Default,
        confirmation: None,
    });
    let ApplyProfileOutcome::ConfirmationRequired(confirmation) = &applied_by(&first).result else {
        panic!("the first answer is the dialog: {first:?}");
    };
    let second = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Default,
        confirmation: Some(confirmation.token.clone()),
    });

    assert!(
        matches!(&applied_by(&second).result, ApplyProfileOutcome::Applied(_)),
        "the token the child issued is what approves the same apply: {second:?}"
    );
    assert_eq!(
        written.lock().unwrap().clone(),
        vec![
            json!({"profile": "team", "mode": "default"}),
            json!({"profile": "team", "mode": "default", "confirmation": "3f1c…"}),
        ],
        "the repeat carries the SAME profile, the SAME mode and the token"
    );
    let (_chat, key) = stored_chat();
    let ids: Vec<String> = wire(&spawner, &key)
        .iter()
        .filter(|asked| asked.method == "apply_profile")
        .map(|asked| asked.id.clone())
        .collect();
    assert_eq!(
        ids,
        ["desktop-2", "desktop-3"],
        "S8: the contract is cached, the dialog wrote nothing (so nothing is re-read), and          the two applies number from one counter"
    );
}

#[test]
fn a_profile_the_store_does_not_define_is_its_own_result_and_writes_nothing() {
    let (script, written) = scripted(vec![Reply::Refused {
        code: "unknown_profile",
    }]);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "gone".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    let attempt = applied_by(&outcome);
    assert!(
        matches!(
            &attempt.result,
            ApplyProfileOutcome::UnknownProfile(error) if error.code == "unknown_profile"
        ),
        "the owner's own answer, not an apply: {attempt:?}"
    );
    assert_eq!(
        written.lock().unwrap().len(),
        1,
        "one attempt, never a retry"
    );
    assert!(
        attempt.read.is_none(),
        "nothing was written, so nothing is re-read"
    );
    let (_chat, key) = stored_chat();
    assert!(
        !wire(&spawner, &key)
            .iter()
            .any(|asked| asked.method == "apply_profile"
                && asked.params.get("confirmation").is_some()),
        "a name the store does not define is never approved with a token"
    );
}

#[test]
fn an_apply_already_running_is_its_own_result_and_writes_nothing() {
    let (script, _written) = scripted(vec![Reply::Refused {
        code: "profile_apply_busy",
    }]);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Default,
        confirmation: None,
    });

    assert!(
        matches!(
            &applied_by(&outcome).result,
            ApplyProfileOutcome::Busy(error) if error.code == "profile_apply_busy"
        ),
        "{outcome:?}"
    );
}

#[test]
fn a_child_that_does_not_advertise_the_apply_is_never_asked() {
    let (script, written) = scripted_with(Vec::new(), false);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    assert!(
        matches!(
            &applied_by(&outcome).result,
            ApplyProfileOutcome::Unsupported
        ),
        "{outcome:?}"
    );
    assert!(
        written.lock().unwrap().is_empty(),
        "the channel is never sent a method the child did not advertise"
    );
    let (_chat, key) = stored_chat();
    assert!(
        !wire(&spawner, &key)
            .iter()
            .any(|asked| asked.method == "apply_profile"),
        "not one apply_profile request left the client"
    );
}

#[test]
fn a_chat_with_no_child_answers_the_not_connected_reason() {
    let (runtime, _spawner) = super::runtime();
    let chat = gentle_app::ChatId::of_session("never-opened.jsonl");

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    let attempt = applied_by(&outcome);
    assert!(
        matches!(
            &attempt.result,
            ApplyProfileOutcome::NoAnswer(gentle_core::rpc::NoAnswer::NotConnected)
        ),
        "{attempt:?}"
    );
    assert!(attempt.read.is_none(), "and there is nothing to read");
}

#[test]
fn the_outcome_carries_the_attempt_for_the_chat_it_was_made_for() {
    let (script, _written) = scripted(vec![applied(ApplyMode::Session)]);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: ApplyMode::Session,
        confirmation: None,
    });

    let Outcome::ChatApply {
        chat: answered_for,
        attempt,
    } = outcome
    else {
        panic!("expected an apply, got {outcome:?}");
    };
    assert_eq!(
        answered_for, chat,
        "the answer names the chat it belongs to"
    );
    let ApplyProfileOutcome::Applied(apply) = &attempt.result else {
        panic!("the documented ok answer is an apply: {attempt:?}");
    };
    assert_eq!(
        apply.notes,
        vec![gentle_core::rpc::profile_apply::ProfileNote {
            level: NoteLevel::Info,
            text: fixtures()["applied"]["data"]["notes"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        }],
        "the screen's own note travels verbatim (L30 ruling 2)"
    );
    assert!(attempt.read.is_some(), "an apply that ran re-reads");
}
