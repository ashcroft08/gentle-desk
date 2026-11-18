//! Editing ONE pin layer over the chat's own child (profiles-editor T6, S2, S3).
//!
//! The `pins` store is written one LAYER at a time: the payload is exactly
//! `{source, profile}`, with `null` clearing that layer and the other layer's
//! file untouched by construction (`desktop-rpc.md:322-372`). What these tests pin
//! is therefore three things: the payload names one layer and one profile, the
//! precondition is THAT layer's own `writeTargets[].writeRevision` - `pins` has
//! two targets, so the other token is the one that would answer
//! `revision_conflict` for ever - and every failure writes nothing more and
//! re-reads the truth the tab draws.
//!
//! Nothing here starts a child: the scripted fake answers the channel, so the
//! wire itself - ids, order and bodies - is what is asserted.

use std::sync::{Arc, Mutex};

use gentle_app::ChatCommand;
use gentle_core::rpc::config_store::{PinInvalid, PinSource, SetConfigOutcome};
use gentle_core::rpc::{LayerStatus, SCHEMA};
use serde_json::{json, Value};

use super::fake::{FakeSpawner, Reply, Script};
use super::profile::{answering, stored_chat, wire};
use crate::chat_runtime::{ChatRuntime, Outcome, PinSlot, PinWriteResult, StoreRead};

/// The profiles the scripted child defines: a pin may only name one of these.
fn defined() -> Vec<&'static str> {
    vec!["team", "fast"]
}

/// The profiles store, only ever re-read here.
fn profiles_answer() -> Value {
    json!({
        "store": "profiles",
        "scope": "global",
        "readOnly": false,
        "status": "ok",
        "revision": "sha256:profiles-read",
        "value": {
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "active": "team",
            "profiles": {
                "team": {"orchestrator": {"model": "p/m"}},
                "fast": {"orchestrator": {"model": "p/f"}}
            }
        },
        "layers": [{ "source": "global", "status": "ok" }],
        "writeTargets": [{
            "target": "global",
            "status": "ok",
            "writeRevision": "sha256:profiles-write",
        }],
    })
}

/// The fallbacks store, only ever re-read here.
fn fallbacks_answer() -> Value {
    json!({
        "store": "profile_fallbacks",
        "scope": "global",
        "readOnly": false,
        "status": "ok",
        "revision": "sha256:fallbacks-read",
        "value": {
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "profiles": {"team": {"*": [{"model": "p/shared"}]}}
        },
        "layers": [{ "source": "global", "status": "ok" }],
        "writeTargets": [{
            "target": "global",
            "status": "ok",
            "writeRevision": "sha256:fallbacks-write",
        }],
    })
}

/// The two pin files, as the child holds them: `None` is a file that does not
/// exist, `Some(name)` the profile it names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PinFiles {
    local: Option<String>,
    repo: Option<String>,
}

impl PinFiles {
    fn of(pairs: &[(&str, Option<&str>)]) -> Self {
        let mut files = Self::default();
        for (layer, name) in pairs {
            let slot = if *layer == "local" {
                &mut files.local
            } else {
                &mut files.repo
            };
            *slot = name.map(str::to_owned);
        }
        files
    }

    fn get(&self, layer: PinSource) -> Option<&String> {
        match layer {
            PinSource::Local => self.local.as_ref(),
            PinSource::Repo => self.repo.as_ref(),
        }
    }

    /// The token of one layer's file: the digest of that file's own content, so
    /// writing the other layer can never move it (`desktop-rpc.md:265-273`).
    fn token(&self, layer: PinSource) -> String {
        match self.get(layer) {
            Some(name) => format!("sha256:{}-{name}", layer.as_str()),
            None => format!("sha256:{}-missing", layer.as_str()),
        }
    }
}

/// A pin file that names a profile the store still defines, as the owner
/// resolves it.
fn resolves(files: &PinFiles, layer: PinSource) -> Option<String> {
    let name = files.get(layer)?;
    defined()
        .iter()
        .find(|known| **known == name.as_str())
        .map(|_| name.clone())
}

/// The `pins` answer for `files`, computed the way the owner does it: the repo
/// layer outranks the local one, a name the profiles store no longer defines is
/// `stale`, and a VALID layer another one outranks is not named at all
/// (`desktop-rpc.md:211-234`).
fn pins_answer(files: &PinFiles) -> Value {
    let winner = match (
        resolves(files, PinSource::Repo),
        resolves(files, PinSource::Local),
    ) {
        (Some(repo), _) => Some((PinSource::Repo, repo)),
        (None, Some(local)) => Some((PinSource::Local, local)),
        (None, None) => None,
    };
    let mut stale: Vec<Value> = Vec::new();
    for layer in [PinSource::Local, PinSource::Repo] {
        if let Some(name) = files.get(layer) {
            if !defined().contains(&name.as_str()) {
                stale.push(json!({ "source": layer.as_str(), "profile": name }));
            }
        }
    }
    let mut value = json!({
        "resolved": winner.is_some(),
        "invalid": [],
        "stale": stale,
    });
    if let Some((layer, profile)) = &winner {
        value["winner"] = json!({ "source": layer.as_str(), "profile": profile });
    }
    json!({
        "store": "pins",
        "scope": "local+repo",
        "readOnly": false,
        "status": if winner.is_some() { "ok" } else { "invalid" },
        "revision": format!("sha256:pins-{}{}", files.token(PinSource::Local), files.token(PinSource::Repo)),
        "value": value,
        "layers": [
            {
                "source": "local",
                "status": if files.local.is_some() { "ok" } else { "missing" },
            },
            {
                "source": "repo",
                "status": if files.repo.is_some() { "ok" } else { "missing" },
            },
        ],
        "writeTargets": [
            {
                "target": "local",
                "status": if files.local.is_some() { "ok" } else { "missing" },
                "writeRevision": files.token(PinSource::Local),
            },
            {
                "target": "repo",
                "status": if files.repo.is_some() { "ok" } else { "missing" },
                "writeRevision": files.token(PinSource::Repo),
            },
        ],
    })
}

/// The pins answer of a session whose working directory is no resolvable Git
/// worktree: no layer is reported and no target is offered, so no pin write is
/// accepted (`desktop-rpc.md:273`, `:381`).
fn no_worktree_answer() -> Value {
    json!({
        "store": "pins",
        "scope": "local+repo",
        "readOnly": false,
        "status": "invalid",
        "revision": "sha256:pins-none",
        "value": {"resolved": false, "invalid": [], "stale": [], "reason": "worktree_unresolved"},
        "layers": [],
        "writeTargets": [],
    })
}

/// A child whose pin files ARE `files`: `get_config` answers what they amount to,
/// `set_config` records the params and writes the one layer (or refuses with
/// `refusal`). The other two stores answer their documented shape.
type ScriptedChild = (Script, Arc<Mutex<PinFiles>>, Arc<Mutex<Vec<Value>>>);

fn scripted(files: PinFiles, refusal: Option<&'static str>) -> ScriptedChild {
    let pins = Arc::new(Mutex::new(files));
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let held = Arc::clone(&pins);
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
            Some("profiles") => profiles_answer(),
            Some("profile_fallbacks") => fallbacks_answer(),
            Some("pins") => pins_answer(&held.lock().unwrap()),
            other => panic!("the pin save never asks {other:?}"),
        }),
        "set_config" => {
            seen.lock().unwrap().push(params.clone());
            if let Some(code) = refusal {
                return Reply::Refused { code };
            }
            // The owner's own write of one layer: the other file is untouched.
            let data = &params["data"];
            let layer = data["source"].as_str().expect("a layer is named");
            let profile = data["profile"].as_str().map(str::to_owned);
            {
                let mut files = held.lock().unwrap();
                match layer {
                    "local" => files.local = profile,
                    "repo" => files.repo = profile,
                    other => panic!("the pin store names two layers, not {other}"),
                }
            }
            let mut answer = pins_answer(&held.lock().unwrap());
            answer["writeTarget"] = json!(layer);
            answer["writeRevision"] = json!(held
                .lock()
                .unwrap()
                .token(PinSource::parse(layer).expect("a documented layer")));
            answer["changed"] = json!(true);
            Reply::Data(answer)
        }
        other => panic!("the pin save never asks {other}"),
    });
    (script, pins, writes)
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

/// The command one test sends.
fn save(chat: &gentle_app::ChatId, source: PinSource, profile: Option<&str>) -> ChatCommand {
    ChatCommand::SetChatProfilePin {
        chat: chat.clone(),
        source,
        profile: profile.map(str::to_owned),
    }
}

/// The result of one `exit` of the pin command.
fn result(outcome: &Outcome) -> &PinWriteResult {
    match outcome {
        Outcome::ChatPin { result, .. } => result,
        other => panic!("expected a pin write, got {other:?}"),
    }
}

/// The stores one pin save asked, in order, as `method:store` pairs.
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
fn the_pin_write_names_one_layer_and_that_layers_token() {
    let (script, _files, writes) = scripted(PinFiles::of(&[("local", Some("fast"))]), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::Saved,
        "the documented ok answer is a save: {outcome:?}"
    );
    let sent = writes.lock().unwrap().clone();
    assert_eq!(sent.len(), 1, "one write, and only one");
    assert_eq!(sent[0]["store"], json!("pins"));
    assert_eq!(
        sent[0]["data"],
        json!({ "source": "repo", "profile": "team" }),
        "S2: exactly `{{source, profile}}`, and the profile it should hold"
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
    assert_eq!(
        sent[0]["expectedRevision"],
        json!("sha256:repo-missing"),
        "S3: the precondition is THIS layer's own entry, whose file did not exist yet"
    );
    assert_ne!(
        sent[0]["expectedRevision"],
        json!("sha256:local-fast"),
        "the other layer's token is not this write's: it would conflict for ever"
    );
}

#[test]
fn the_local_layer_is_preconditioned_on_the_local_token() {
    let (script, _files, writes) = scripted(PinFiles::of(&[("repo", Some("team"))]), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    runtime.execute(save(&chat, PinSource::Local, Some("fast")));

    assert_eq!(
        writes.lock().unwrap()[0]["expectedRevision"],
        json!("sha256:local-missing"),
        "the local layer's own file"
    );
    assert_eq!(
        writes.lock().unwrap()[0]["data"],
        json!({ "source": "local", "profile": "fast" })
    );
}

#[test]
fn a_clear_names_the_layer_and_a_null_profile() {
    let (script, files, writes) = scripted(PinFiles::of(&[("local", Some("team"))]), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Local, None));

    assert_eq!(result(&outcome), &PinWriteResult::Saved, "{outcome:?}");
    assert_eq!(
        writes.lock().unwrap()[0]["data"],
        json!({ "source": "local", "profile": null }),
        "the explicit clear the owner documents: the literal null, not an absent key"
    );
    assert_eq!(
        files.lock().unwrap().local,
        None,
        "the owner's own clear removed that one file"
    );
}

#[test]
fn the_write_is_followed_by_the_same_three_reads_a_plain_read_makes() {
    let (script, _files, _writes) = scripted(PinFiles::default(), None);
    let (runtime, spawner, chat, _events) = opened(script);

    runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        asked_stores(&spawner),
        [
            "capabilities",
            "get_config:pins",
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
fn the_reread_shows_the_pin_the_store_now_holds() {
    let (script, _files, _writes) = scripted(PinFiles::default(), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("fast")));

    let Outcome::ChatPin { read, .. } = outcome else {
        panic!("expected the pin write");
    };
    let StoreRead::Read(pins) = read.pins else {
        panic!("the re-read answers the pins store: {read:?}");
    };
    assert_eq!(
        pins.slot(PinSource::Repo),
        PinSlot::Pinned("fast".to_owned()),
        "the layer the tab draws is the one the child stored"
    );
    assert_eq!(
        pins.slot(PinSource::Local),
        PinSlot::Empty,
        "and the other layer is exactly as it was: no file, no pin"
    );
}

#[test]
fn the_read_carries_each_layers_own_state_and_which_layers_are_writable() {
    // The documented case: the repo layer wins with `team`, the local one names a
    // profile the store dropped (`desktop-rpc.md:211-234`).
    let (script, _files, _writes) = scripted(
        PinFiles::of(&[("local", Some("retired")), ("repo", Some("team"))]),
        None,
    );
    let (runtime, _spawner, chat, _events) = opened(script);

    let Outcome::ChatProfile { read, .. } =
        runtime.execute(ChatCommand::ReadChatProfile { chat: chat.clone() })
    else {
        panic!("expected the chat's profile read");
    };
    let StoreRead::Read(pins) = read.pins else {
        panic!("the pins store answered: {read:?}");
    };

    assert_eq!(
        pins.layers,
        vec![
            crate::chat_runtime::PinLayerState {
                source: PinSource::Local,
                status: LayerStatus::Ok,
            },
            crate::chat_runtime::PinLayerState {
                source: PinSource::Repo,
                status: LayerStatus::Ok,
            },
        ],
        "one row per pin file the store read, in its own order"
    );
    assert_eq!(
        pins.slot(PinSource::Repo),
        PinSlot::Pinned("team".to_owned()),
        "the layer that decides here"
    );
    assert_eq!(
        pins.slot(PinSource::Local),
        PinSlot::Stale("retired".to_owned()),
        "a layer that names a profile the profiles store dropped is named as stale"
    );
    assert!(pins.writable(PinSource::Local) && pins.writable(PinSource::Repo));
}

#[test]
fn a_valid_layer_another_one_outranks_is_not_reported() {
    let (script, _files, _writes) = scripted(
        PinFiles::of(&[("local", Some("fast")), ("repo", Some("team"))]),
        None,
    );
    let (runtime, _spawner, chat, _events) = opened(script);

    let read = runtime.read_chat_profile(&chat);

    let StoreRead::Read(pins) = read.pins else {
        panic!("the pins store answered: {read:?}");
    };
    assert_eq!(
        pins.slot(PinSource::Repo),
        PinSlot::Pinned("team".to_owned())
    );
    assert_eq!(
        pins.slot(PinSource::Local),
        PinSlot::Unreported,
        "a VALID local pin the repo one outranks is not named by the value at all, \
         so the tab must not claim to know it"
    );
}

#[test]
fn a_layer_this_worktree_offers_no_target_for_is_never_written() {
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "methods": ["capabilities", "get_config", "set_config"],
        })),
        "get_config" => Reply::Data(match params["store"].as_str() {
            Some("profiles") => profiles_answer(),
            Some("profile_fallbacks") => fallbacks_answer(),
            Some("pins") => no_worktree_answer(),
            other => panic!("a store is always named: {other:?}"),
        }),
        "set_config" => {
            seen.lock().unwrap().push(params.clone());
            Reply::Refused {
                code: "config_unavailable",
            }
        }
        other => panic!("the pin save never asks {other}"),
    });
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::Unavailable,
        "no working directory in a resolvable worktree, so no pin file is offered"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "a request the owner could only answer `config_unavailable` is never sent"
    );
    assert!(
        !asked_stores(&spawner)
            .iter()
            .any(|asked| asked == "set_config"),
        "nothing left the app"
    );
    let Outcome::ChatPin { read, .. } = outcome else {
        panic!("expected the pin write");
    };
    assert!(
        matches!(read.pins, StoreRead::Read(_)),
        "the tab still re-reads the truth it draws: {read:?}"
    );
}

#[test]
fn a_blank_profile_is_never_sent() {
    let (script, _files, writes) = scripted(PinFiles::default(), None);
    let (runtime, spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("  ")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::Invalid(PinInvalid::BlankProfile),
        "a name the owner would refuse is caught here, before a request exists"
    );
    assert!(writes.lock().unwrap().is_empty(), "nothing was written");
    assert_eq!(
        asked_stores(&spawner),
        [
            "capabilities",
            "get_config:profiles",
            "get_config:profile_fallbacks",
            "get_config:pins"
        ],
        "and nothing of the write left the app: the re-read is the only conversation"
    );
}

#[test]
fn the_pin_the_store_already_holds_is_not_written_again() {
    let (script, _files, writes) = scripted(PinFiles::of(&[("repo", Some("team"))]), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::Unchanged,
        "the layer already names this profile: nothing to save, and no notice"
    );
    assert!(
        writes.lock().unwrap().is_empty(),
        "an equal pin is never written"
    );
}

#[test]
fn a_clear_of_a_layer_with_no_file_is_not_written_again() {
    let (script, _files, writes) = scripted(PinFiles::of(&[("repo", Some("team"))]), None);
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Local, None));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::Unchanged,
        "clearing a layer whose file does not exist is already the desired state"
    );
    assert!(writes.lock().unwrap().is_empty());
}

#[test]
fn every_refusal_writes_nothing_more_and_re_reads() {
    for code in [
        "revision_conflict",
        "invalid_data",
        "config_unavailable",
        "profile_apply_busy",
    ] {
        let files = PinFiles::of(&[("local", Some("fast"))]);
        let (script, held, writes) = scripted(files.clone(), Some(code));
        let (runtime, spawner, chat, _events) = opened(script);

        let outcome = runtime.execute(save(&chat, PinSource::Local, Some("team")));

        let PinWriteResult::Failed(failure) = result(&outcome) else {
            panic!("{code}: expected a typed failure, got {outcome:?}");
        };
        match (code, failure.as_ref()) {
            ("revision_conflict", SetConfigOutcome::RevisionConflict(_))
            | ("invalid_data", SetConfigOutcome::InvalidData(_))
            | ("config_unavailable", SetConfigOutcome::ConfigUnavailable(_))
            | ("profile_apply_busy", SetConfigOutcome::ApplyBusy(_)) => {}
            (_, other) => panic!("{code} is not {other:?}"),
        }
        assert_eq!(
            writes.lock().unwrap().len(),
            1,
            "{code}: the refusal is an answer, so it is not retried"
        );
        assert_eq!(*held.lock().unwrap(), files, "{code}: nothing was written");
        assert_eq!(
            asked_stores(&spawner)
                .iter()
                .filter(|asked| asked.starts_with("get_config:"))
                .count(),
            4,
            "{code}: the chat is re-read after the attempt"
        );
        let Outcome::ChatPin { read, .. } = outcome else {
            panic!("expected the pin write");
        };
        assert!(
            matches!(read.pins, StoreRead::Read(_)),
            "{code}: the failure still re-reads the truth the tab shows"
        );
    }
}

#[test]
fn a_child_that_does_not_advertise_set_config_is_never_written() {
    let writes: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&writes);
    let script: Script = Arc::new(move |method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "methods": ["capabilities", "get_config"],
        })),
        "get_config" => Reply::Data(match params["store"].as_str() {
            Some("profiles") => profiles_answer(),
            Some("profile_fallbacks") => fallbacks_answer(),
            Some("pins") => pins_answer(&PinFiles::default()),
            other => panic!("a store is always named: {other:?}"),
        }),
        "set_config" => {
            seen.lock().unwrap().push(params.clone());
            Reply::Refused {
                code: "unknown_method",
            }
        }
        other => panic!("the pin save never asks {other}"),
    });
    let (runtime, _spawner, chat, _events) = opened(script);

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::failed(SetConfigOutcome::Unsupported),
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

    let outcome = runtime.execute(save(&chat, PinSource::Repo, Some("team")));

    assert_eq!(
        result(&outcome),
        &PinWriteResult::failed(SetConfigOutcome::NoAnswer(
            gentle_core::rpc::NoAnswer::NotConnected
        ))
    );
    assert!(spawner.started().is_empty(), "no child was started for it");
    let Outcome::ChatPin { read, .. } = outcome else {
        panic!("expected the pin write");
    };
    assert_eq!(
        *read,
        crate::chat_runtime::ChatConfigRead::unasked(),
        "the re-read of a chat with no child is nothing read, not an error"
    );
}
