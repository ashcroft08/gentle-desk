//! Reading and writing gentle-pi's three authoritative config stores over the
//! `/gentle:desktop` channel (profiles-editor T2a, S2/S3/S9).
//!
//! Every frame this file asserts is the channel spec's own, verbatim from
//! gentle-pi's `docs/desktop-rpc.md` (`fixtures/desktop-rpc-profiles.json`);
//! the line numbers below are that document's. Nothing here spawns a process:
//! the whole battery drives the fakeable [`DesktopChannel`] seam, so the two
//! rules that matter most - `get_config` params are exactly `{ "store": … }`
//! and `set_config` params are exactly `{ store, data, expectedRevision }` with
//! the revision taken from `writeTargets[].writeRevision` - are pinned without
//! a child, and the 64 KiB cap is pinned without a pipe.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::config_store::{
    parse_config_projection, parse_set_config_written, parse_store_value, read_config,
    set_config_params, write_config, AgentRoute, ConfigLayer, ConfigProjection, FallbackStep,
    GetConfigOutcome, LayerSource, OrchestratorMode, PinLayer, PinSource, PinsValue, ReadOnlyStore,
    SetConfigOutcome, SetConfigWritten, StoreId, StoreScope, StoreValue, WritableStore,
    WriteTarget, WriteTargetName, FALLBACKS_KIND, GET_CONFIG, PROFILES_KIND, SET_CONFIG,
    WORKTREE_UNRESOLVED,
};
use gentle_core::rpc::{
    AnswerError, ChannelReply, DesktopChannel, DesktopClient, DesktopError, DesktopRequest,
    LayerStatus, NoAnswer, RequestError, StoreRevision, WriteRevision, MAX_REQUEST_BYTES, SCHEMA,
};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/desktop-rpc-profiles.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("the fixture is JSON")
}

/// One documented frame, by the name the fixture gives it.
fn frame(name: &str) -> Value {
    fixture()[name].clone()
}

/// The `data` of a documented response frame, or the frame itself when the
/// document prints the projection without an envelope around it.
fn data(name: &str) -> Value {
    let frame = frame(name);
    match frame.get("data") {
        Some(data) => data.clone(),
        None => frame,
    }
}

/// The contract `capabilities` reports, with the config slice advertised.
fn capabilities() -> String {
    response(
        "desktop-1",
        json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": ["capabilities", "get_snapshot", "get_config", "set_config", "apply_profile"],
            "events": ["config_changed", "profile_applied"],
        }),
    )
}

/// A contract that does NOT advertise the config slice (an older child).
fn capabilities_without_config() -> String {
    response(
        "desktop-1",
        json!({"schema": SCHEMA, "methods": ["capabilities", "get_snapshot"]}),
    )
}

fn response(request_id: &str, data: Value) -> String {
    json!({
        "schema": SCHEMA,
        "kind": "response",
        "requestId": request_id,
        "ok": true,
        "data": data,
    })
    .to_string()
}

/// One documented frame as a child answers OUR request: every member is the
/// document's own except `requestId`, which correlates the answer with the
/// request the client wrote (the document's frame answers its own `desktop-46`
/// / `desktop-49`...).
fn answered(name: &str, request_id: &str) -> ChannelReply {
    let mut answer = frame(name);
    answer["requestId"] = json!(request_id);
    ChannelReply::Widget(answer.to_string())
}

/// A channel that answers the replies it was given, in order, and remembers
/// every request it was asked (so the exact body is assertable).
struct FakeChannel {
    replies: Mutex<VecDeque<ChannelReply>>,
    sent: Mutex<Vec<DesktopRequest>>,
}

impl FakeChannel {
    fn answering(replies: Vec<ChannelReply>) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(replies.into()),
            sent: Mutex::new(Vec::new()),
        })
    }

    /// A channel that answers the capabilities frame and then `second`.
    fn connected(second: ChannelReply) -> Arc<Self> {
        Self::answering(vec![ChannelReply::Widget(capabilities()), second])
    }

    fn sent(&self) -> Vec<DesktopRequest> {
        self.sent.lock().unwrap().clone()
    }

    fn methods(&self) -> Vec<String> {
        self.sent()
            .iter()
            .map(|request| request.method().to_owned())
            .collect()
    }
}

impl DesktopChannel for FakeChannel {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply {
        self.sent.lock().unwrap().push(request.clone());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(ChannelReply::Timeout)
    }
}

/// The written revision the documented `get_config` answer reports
/// (`desktop-rpc.md:150`): the token `set_config` requires.
fn documented_write_revision() -> WriteRevision {
    WriteRevision::parse("sha256:8f2a…").expect("a write revision")
}

mod get_config_params {
    use super::*;

    #[test]
    fn the_request_is_exactly_the_store_identifier() {
        // The documented request (`desktop-rpc.md:101-108`): one param, no
        // scope, no path, no physical path.
        let documented = frame("get_config_request");
        let params = gentle_core::rpc::config_store::get_config_params(StoreId::Profiles);

        assert_eq!(params, documented["params"]);
        assert_eq!(
            params
                .as_object()
                .expect("an object")
                .keys()
                .collect::<Vec<_>>(),
            ["store"],
            "the contract accepts exactly one key; any other is invalid_request"
        );
    }

    #[test]
    fn every_store_identifier_the_contract_names_round_trips() {
        let documented = [
            (StoreId::Profiles, "profiles"),
            (StoreId::Models, "models"),
            (StoreId::ProfileFallbacks, "profile_fallbacks"),
            (StoreId::Subagents, "subagents"),
            (StoreId::Pins, "pins"),
            (StoreId::ManagedAssets, "managed_assets"),
        ];
        for (store, name) in documented {
            assert_eq!(store.as_str(), name);
            assert_eq!(StoreId::parse(name), Some(store));
            assert_eq!(
                gentle_core::rpc::config_store::get_config_params(store),
                json!({ "store": name })
            );
        }
        assert_eq!(StoreId::parse("settings"), None);
        assert_eq!(
            StoreId::parse("profiles.json"),
            None,
            "a store is never named by a path"
        );
    }
}

mod get_config_answer {
    use super::*;

    #[test]
    fn the_documented_profiles_answer_parses_into_typed_fields() {
        // The documented success (`desktop-rpc.md:126-152`).
        let projection = parse_config_projection(&data("get_config_profiles"))
            .expect("the documented frame is this contract's");

        assert_eq!(projection.store, StoreId::Profiles);
        assert_eq!(projection.scope, StoreScope::Global);
        assert!(!projection.read_only);
        assert_eq!(projection.status, LayerStatus::Ok);
        assert_eq!(
            projection.revision,
            StoreRevision::parse("sha256:1f0c8c0f5bd34a1e27f4ba6d0f9c2f5b").unwrap()
        );
        assert_eq!(
            projection.layers,
            [ConfigLayer {
                source: LayerSource::Global,
                status: LayerStatus::Ok,
            }]
        );
        assert_eq!(
            projection.write_targets,
            [WriteTarget {
                target: WriteTargetName::Global,
                status: LayerStatus::Ok,
                // Verbatim, ellipsis and all: the document's own token.
                write_revision: documented_write_revision(),
            }]
        );

        let StoreValue::Profiles(profiles) = projection.value.expect("a value") else {
            panic!("a profiles store carries a profiles value");
        };
        assert_eq!(profiles.active.as_deref(), Some("team"));
        assert_eq!(profiles.profiles.len(), 1);
        assert_eq!(
            profiles.profiles["team"]["explore"],
            AgentRoute {
                model: Some("anthropic/claude-sonnet-4".to_owned()),
                thinking: None,
                tier: None,
            }
        );
        assert!(
            profiles.workflow.is_empty(),
            "the documented frame names no workflow"
        );
    }

    #[test]
    fn the_documented_profiles_write_document_parses_every_member() {
        // The `data` the owner serializes for a write (`desktop-rpc.md:306-320`):
        // the same shape a read answers, so one parser reads both.
        let value = parse_store_value(StoreId::Profiles, &frame("profiles_data"))
            .expect("the documented document parses");
        let StoreValue::Profiles(profiles) = value else {
            panic!("a profiles store carries a profiles value");
        };

        assert_eq!(profiles.active.as_deref(), Some("team"));
        assert_eq!(
            profiles.profiles["team"]["orchestrator"].model.as_deref(),
            Some("openai/gpt-5")
        );
        assert_eq!(
            profiles.workflow["team"],
            OrchestratorMode::FullDelegation,
            "workflow.orchestratorMode is typed, not a string"
        );
    }

    #[test]
    fn the_documented_fallbacks_document_parses_its_chains() {
        // The documented `profile_fallbacks` content (`desktop-rpc.md:335-347`).
        let value = parse_store_value(StoreId::ProfileFallbacks, &frame("fallbacks_data"))
            .expect("the documented document parses");
        let StoreValue::Fallbacks(fallbacks) = value else {
            panic!("a fallbacks store carries a fallbacks value");
        };

        assert_eq!(
            fallbacks.profiles["team"]["*"],
            [FallbackStep {
                model: "openai/gpt-shared".to_owned(),
                thinking: None,
            }],
            "`*` is the profile-wide default chain, kept as the file has it"
        );
        assert_eq!(
            fallbacks.profiles["team"]["worker"],
            [FallbackStep {
                model: "openai/gpt-fallback".to_owned(),
                thinking: Some("low".to_owned()),
            }]
        );
    }

    #[test]
    fn the_documented_pins_answer_parses_the_winner_and_the_stale_layer() {
        // The documented `pins` projection (`desktop-rpc.md:211-232`). It is
        // printed without an envelope: the document shows only the store fields.
        let projection = parse_config_projection(&data("pins_value"))
            .expect("the documented frame is this contract's");

        assert_eq!(projection.store, StoreId::Pins);
        assert_eq!(projection.scope, StoreScope::LocalAndRepo);
        assert_eq!(
            projection.layers,
            [
                ConfigLayer {
                    source: LayerSource::Local,
                    status: LayerStatus::Ok,
                },
                ConfigLayer {
                    source: LayerSource::Repo,
                    status: LayerStatus::Ok,
                },
            ]
        );
        let StoreValue::Pins(pins) = projection.value.expect("a value") else {
            panic!("a pins store carries a pins value");
        };
        assert_eq!(
            pins,
            PinsValue {
                resolved: true,
                winner: Some(PinLayer {
                    source: PinSource::Repo,
                    profile: Some("team".to_owned()),
                }),
                invalid: Vec::new(),
                stale: vec![PinLayer {
                    source: PinSource::Local,
                    profile: Some("retired".to_owned()),
                }],
                reason: None,
            }
        );
    }

    #[test]
    fn a_pin_read_outside_a_worktree_is_a_typed_unresolved_reason() {
        // The document describes this answer in prose (`desktop-rpc.md:234`)
        // and prints no frame for it: `status: "invalid"`, `value.reason` and an
        // empty `layers` array. The frame below is assembled from exactly those
        // facts, with the pins revision of the documented projection.
        let unresolved = json!({
            "store": "pins",
            "scope": "local+repo",
            "readOnly": false,
            "status": "invalid",
            "revision": "sha256:0d19c0b7f0a94c29d5c33a0b1c9e6d21",
            "value": { "reason": WORKTREE_UNRESOLVED },
            "layers": []
        });

        let projection = parse_config_projection(&unresolved).expect("the answer parses");
        assert_eq!(projection.status, LayerStatus::Invalid);
        assert!(projection.layers.is_empty(), "no layer to report");
        assert!(
            projection.write_targets.is_empty(),
            "no truthful pin file, so no target is offered and no pin write is accepted"
        );
        let StoreValue::Pins(pins) = projection.value.expect("a value") else {
            panic!("a pins store carries a pins value");
        };
        assert!(!pins.resolved);
        assert_eq!(pins.reason.as_deref(), Some(WORKTREE_UNRESOLVED));
        assert_eq!(pins.winner, None);
    }

    #[test]
    fn a_store_without_content_answers_no_value() {
        // The documented malformed-store answer (`desktop-rpc.md:190-210`).
        let projection = parse_config_projection(&data("models_invalid"))
            .expect("the documented frame is this contract's");

        assert_eq!(projection.store, StoreId::Models);
        assert!(projection.read_only);
        assert_eq!(projection.status, LayerStatus::Invalid);
        assert_eq!(projection.value, None, "no layer produced content");
        assert_eq!(
            projection.layers,
            [
                ConfigLayer {
                    source: LayerSource::Global,
                    status: LayerStatus::Invalid,
                },
                ConfigLayer {
                    source: LayerSource::Project,
                    status: LayerStatus::Ok,
                },
            ],
            "the answer still names the layer that is broken"
        );
        assert!(
            projection.write_targets.is_empty(),
            "models is not writable"
        );
    }

    #[test]
    fn the_documented_subagents_answer_keeps_its_value_unparsed() {
        // The documented merged-store answer (`desktop-rpc.md:184-204`).
        let projection = parse_config_projection(&data("subagents_malformed_value"))
            .expect("the documented frame is this contract's");

        assert_eq!(projection.store, StoreId::Subagents);
        assert_eq!(projection.status, LayerStatus::Ok);
        assert_eq!(
            projection.value,
            Some(StoreValue::Other(
                frame("subagents_malformed_value")["value"].clone()
            )),
            "this slice types the three writable stores; the others are kept as reported"
        );
        assert_eq!(projection.layers[1].status, LayerStatus::Invalid);
    }

    #[test]
    fn a_missing_required_member_is_a_typed_parse_error_never_a_panic() {
        for field in ["store", "scope", "readOnly", "status", "revision"] {
            let mut projection = data("get_config_profiles");
            projection.as_object_mut().expect("an object").remove(field);
            assert_eq!(
                parse_config_projection(&projection),
                Err(AnswerError::MissingField { field }),
                "the store projection cannot be read without {field}"
            );
        }

        // The profiles document needs its own required members.
        let mut value = data("get_config_profiles");
        value["value"].as_object_mut().unwrap().remove("profiles");
        assert_eq!(
            parse_config_projection(&value),
            Err(AnswerError::MissingField { field: "profiles" })
        );
        let mut value = data("get_config_profiles");
        value["value"].as_object_mut().unwrap().remove("kind");
        assert_eq!(
            parse_config_projection(&value),
            Err(AnswerError::MissingField { field: "kind" })
        );
    }

    #[test]
    fn a_member_of_the_wrong_shape_is_a_typed_parse_error() {
        let mut projection = data("get_config_profiles");
        projection["revision"] = json!(1);
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::WrongType { field: "revision" })
        );

        let mut projection = data("get_config_profiles");
        projection["store"] = json!("configuration");
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::UnknownValue {
                field: "store",
                value: "configuration".to_owned(),
            })
        );

        let mut projection = data("get_config_profiles");
        projection["status"] = json!("broken");
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::UnknownValue {
                field: "status",
                value: "broken".to_owned(),
            })
        );

        let mut projection = data("get_config_profiles");
        projection["value"] = json!([]);
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::NotAnObject { field: "value" })
        );

        let mut projection = data("get_config_profiles");
        projection["layers"][0]["source"] = json!("cloud");
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::UnknownValue {
                field: "source",
                value: "cloud".to_owned(),
            })
        );

        // A store this contract does not define at all.
        let mut projection = data("get_config_profiles");
        projection["value"]["kind"] = json!("gentle-pi.agent_model_profiles");
        projection["value"]["version"] = json!(2);
        assert_eq!(
            parse_config_projection(&projection),
            Err(AnswerError::UnknownValue {
                field: "version",
                value: "2".to_owned(),
            }),
            "a version this reader does not know is refused, never half-read"
        );
    }

    #[test]
    fn unknown_extra_keys_are_ignored_on_read() {
        // The channel may grow: a member this reader does not model carries no
        // meaning here, exactly as gentle-pi's own decoder treats it.
        let documented = parse_config_projection(&data("get_config_profiles")).unwrap();

        let mut grown = data("get_config_profiles");
        grown["futureScope"] = json!({"layer": "x"});
        grown["layers"][0]["path"] = json!("C:/never/returned");
        grown["writeTargets"][0]["bytes"] = json!(42);
        grown["value"]["profiles"]["team"]["explore"]["future"] = json!("kept out");
        grown["value"]["profiles"]["team"]["explore"]["thinking"] = json!("high");
        grown["value"]["futureWorkflow"] = json!({"x": {"orchestratorMode": "organic"}});

        let parsed = parse_config_projection(&grown).expect("extra members are ignored");
        assert_eq!(
            parsed.store, documented.store,
            "the store is what the answer names"
        );
        assert_eq!(parsed.scope, documented.scope);
        assert_eq!(parsed.read_only, documented.read_only);
        assert_eq!(parsed.status, documented.status);
        assert_eq!(parsed.revision, documented.revision);
        assert_eq!(parsed.layers, documented.layers);
        assert_eq!(parsed.write_targets, documented.write_targets);
        let StoreValue::Profiles(profiles) = parsed.value.expect("a value") else {
            panic!("a profiles store carries a profiles value");
        };
        assert_eq!(
            profiles.profiles["team"]["explore"].thinking.as_deref(),
            Some("high"),
            "a member this reader does model is read"
        );

        // The same at the pins and fallbacks level.
        let mut pins = data("pins_value");
        pins["value"]["future"] = json!(true);
        pins["value"]["winner"]["path"] = json!("/x");
        assert!(
            parse_config_projection(&pins).is_ok(),
            "an extra pin member is ignored"
        );

        let mut fallbacks = frame("fallbacks_data");
        fallbacks["future"] = json!(1);
        fallbacks["profiles"]["team"]["*"][0]["tier"] = json!("high");
        assert!(parse_store_value(StoreId::ProfileFallbacks, &fallbacks).is_ok());
    }

    #[test]
    fn the_two_revision_tokens_are_different_types_and_different_values() {
        // S3: `expectedRevision` is the write target's token, never the store's.
        let projection = parse_config_projection(&data("get_config_profiles")).unwrap();
        let store: StoreRevision = projection.revision.clone();
        let write: WriteRevision = projection.write_targets[0].write_revision.clone();

        assert_ne!(
            store.as_str(),
            write.as_str(),
            "the document reports two different tokens"
        );
        assert_eq!(
            projection.write_revision(WriteTargetName::Global),
            Some(&write),
            "the token a write needs is looked up by target"
        );
        assert_eq!(projection.write_revision(WriteTargetName::Repo), None);
    }
}

mod set_config_params {
    use super::*;

    #[test]
    fn the_documented_request_is_store_data_and_the_write_revision() {
        // The documented request (`desktop-rpc.md:277-288`), whose
        // `expectedRevision` is the `writeTargets[].writeRevision` of a read.
        let documented = frame("set_config_request");
        let params = set_config_params(
            WritableStore::Profiles,
            documented["params"]["data"].clone(),
            &WriteRevision::parse("sha256:8f2a0c5e2d7b4319a6f80e1c93b4d271").unwrap(),
        );

        assert_eq!(params, documented["params"]);
        assert_eq!(
            params
                .as_object()
                .expect("an object")
                .keys()
                .collect::<Vec<_>>(),
            ["data", "expectedRevision", "store"],
            "exactly these three keys are accepted; any other is invalid_request"
        );
        assert_eq!(
            params["data"], documented["params"]["data"],
            "`data` is the complete document, never a patch"
        );
    }

    #[test]
    fn a_read_only_store_is_refused_before_any_request_is_built() {
        for store in [StoreId::Models, StoreId::Subagents, StoreId::ManagedAssets] {
            assert_eq!(
                WritableStore::try_from_store(store),
                Err(ReadOnlyStore(store)),
                "set_config writes profiles, profile_fallbacks and pins only"
            );
            assert!(
                ReadOnlyStore(store).to_string().contains(store.as_str()),
                "the refusal names the store: {}",
                ReadOnlyStore(store)
            );
        }
        assert_eq!(
            WritableStore::try_from_store(StoreId::Profiles),
            Ok(WritableStore::Profiles)
        );
        assert_eq!(
            WritableStore::try_from_store(StoreId::ProfileFallbacks),
            Ok(WritableStore::ProfileFallbacks)
        );
        assert_eq!(
            WritableStore::try_from_store(StoreId::Pins),
            Ok(WritableStore::Pins)
        );

        // A read-only store cannot even reach the builder: the only way to get a
        // `WritableStore` is to be told the store is one.
        assert_eq!(WritableStore::Profiles.as_str(), "profiles");
        assert_eq!(
            WritableStore::ProfileFallbacks.as_str(),
            "profile_fallbacks"
        );
        assert_eq!(WritableStore::Pins.as_str(), "pins");
    }

    #[test]
    fn a_pin_write_names_one_layer_and_the_pin_only_has_two() {
        assert_eq!(
            WritableStore::Profiles.write_targets(),
            [WriteTargetName::Global]
        );
        assert_eq!(
            WritableStore::ProfileFallbacks.write_targets(),
            [WriteTargetName::Global]
        );
        assert_eq!(
            WritableStore::Pins.write_targets(),
            [WriteTargetName::Local, WriteTargetName::Repo],
            "the two pin files, never both in one request"
        );
        // The documented `data` for a set and for a clear (`desktop-rpc.md:355-362`).
        assert_eq!(frame("pin_local_data")["source"], "local");
        assert_eq!(frame("pin_clear_data")["profile"], Value::Null);
    }
}

mod get_config_answer_via_the_channel {
    use super::*;

    #[test]
    fn the_read_asks_capabilities_first_and_then_the_documented_request() {
        let channel = FakeChannel::connected(answered("get_config_profiles", "desktop-2"));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let outcome = read_config(&mut client, StoreId::Profiles);

        let GetConfigOutcome::Read(projection) = outcome else {
            panic!("the documented frame reads: {outcome:?}");
        };
        assert_eq!(projection.store, StoreId::Profiles);
        assert_eq!(
            channel.methods(),
            ["capabilities", GET_CONFIG],
            "a client asks the contract once, then the method it was told about"
        );
        assert_eq!(
            channel.sent()[1].params(),
            &frame("get_config_request")["params"]
        );
        assert_eq!(channel.sent()[1].request_id(), "desktop-2");
    }

    #[test]
    fn a_child_whose_capabilities_lack_the_config_slice_is_never_asked() {
        let channel =
            FakeChannel::answering(vec![ChannelReply::Widget(capabilities_without_config())]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::Unsupported
        );
        assert_eq!(
            write_config(
                &mut client,
                WritableStore::Profiles,
                frame("profiles_data"),
                &documented_write_revision(),
            ),
            SetConfigOutcome::Unsupported
        );
        assert_eq!(
            channel.methods(),
            ["capabilities"],
            "the channel is never sent a method the child did not advertise"
        );
    }

    #[test]
    fn a_host_that_is_not_interactive_degrades_silently() {
        let channel = FakeChannel::answering(vec![ChannelReply::Inactive, ChannelReply::Inactive]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::NoAnswer(NoAnswer::NotConnected),
            "a client with no resolved contract never guesses a method"
        );
        assert_eq!(channel.methods(), ["capabilities"]);
    }

    #[test]
    fn a_child_that_never_answers_degrades_to_a_timeout() {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities()),
            ChannelReply::Timeout,
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::NoAnswer(NoAnswer::TimedOut)
        );
    }

    #[test]
    fn a_closed_child_degrades_silently() {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities()),
            ChannelReply::Closed,
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::NoAnswer(NoAnswer::Closed)
        );
    }

    #[test]
    fn a_payload_this_contract_does_not_model_is_no_answer() {
        let channel =
            FakeChannel::connected(ChannelReply::Widget("{\"not\":\"an envelope\"}".into()));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::NoAnswer(NoAnswer::Malformed)
        );
    }

    #[test]
    fn a_refusal_is_the_channel_error_and_not_a_read() {
        let refusal = json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": "desktop-2",
            "ok": false,
            "error": {"code": "unknown_store", "message": "Unknown config store"},
        })
        .to_string();
        let channel = FakeChannel::connected(ChannelReply::Widget(refusal));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::Rejected(DesktopError {
                code: "unknown_store".to_owned(),
                message: "Unknown config store".to_owned(),
            })
        );
    }

    #[test]
    fn an_ok_answer_of_another_shape_is_a_typed_unexpected_answer() {
        let mut broken = data("get_config_profiles");
        broken.as_object_mut().unwrap().remove("revision");
        let channel = FakeChannel::connected(ChannelReply::Widget(response("desktop-2", broken)));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            read_config(&mut client, StoreId::Profiles),
            GetConfigOutcome::UnexpectedAnswer(AnswerError::MissingField { field: "revision" })
        );
    }
}

mod set_config_via_the_channel {
    use super::*;

    #[test]
    fn the_write_asks_capabilities_first_and_then_one_set_config() {
        let channel = FakeChannel::connected(answered("set_config_written", "desktop-2"));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let documented = frame("set_config_request");
        let outcome = write_config(
            &mut client,
            WritableStore::Profiles,
            documented["params"]["data"].clone(),
            &WriteRevision::parse(
                documented["params"]["expectedRevision"]
                    .as_str()
                    .expect("the documented token"),
            )
            .expect("a write revision"),
        );

        let SetConfigOutcome::Written(written) = outcome else {
            panic!("the documented answer is a write: {outcome:?}");
        };
        assert_eq!(
            channel.methods(),
            ["capabilities", SET_CONFIG],
            "one write, one round trip, and the contract asked once"
        );
        assert_eq!(
            channel.sent()[1].params(),
            &frame("set_config_request")["params"],
            "the body is the documented one: store, the complete data, the write revision"
        );
        assert_eq!(
            channel.sent()[1].request_id(),
            "desktop-2",
            "the request id correlates the answer"
        );
        assert!(written.changed, "the write changed the target");
    }

    #[test]
    fn the_token_a_read_reported_is_the_one_the_write_sends() {
        // S3: `expectedRevision` is the `writeTargets[].writeRevision` of the
        // store as it was read - "never `revision`" - and it is re-read per
        // store per write.
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities()),
            answered("get_config_profiles", "desktop-2"),
            answered("set_config_written", "desktop-3"),
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let GetConfigOutcome::Read(read) = read_config(&mut client, StoreId::Profiles) else {
            panic!("the documented read frame reads");
        };
        let token = read
            .write_revision(WriteTargetName::Global)
            .expect("the documented write target")
            .clone();
        let outcome = write_config(
            &mut client,
            WritableStore::Profiles,
            frame("profiles_data"),
            &token,
        );

        assert!(matches!(outcome, SetConfigOutcome::Written(_)));
        assert_eq!(channel.methods(), ["capabilities", GET_CONFIG, SET_CONFIG]);
        let sent = &channel.sent()[2];
        assert_eq!(
            sent.params()["expectedRevision"],
            json!("sha256:8f2a…"),
            "the token the read reported is the token the write sends"
        );
        assert_ne!(
            sent.params()["expectedRevision"],
            json!(read.revision.as_str()),
            "the store's own revision is never the precondition"
        );
    }

    #[test]
    fn the_documented_write_answer_reports_the_projection_and_the_next_token() {
        // The documented success (`desktop-rpc.md:391-412`): the projection of
        // the store AFTER the write, plus `writeTarget`, `writeRevision` and
        // `changed`.
        let written: SetConfigWritten =
            parse_set_config_written(&data("set_config_written")).expect("the documented answer");

        assert_eq!(written.projection.store, StoreId::Profiles);
        assert_eq!(written.write_target, WriteTargetName::Global);
        assert_eq!(
            written.write_revision,
            WriteRevision::parse("sha256:c4b0afed3b2375ee0fb10e45a9545c78").unwrap(),
            "the answer carries the token the next write needs"
        );
        assert_eq!(
            written.projection.revision,
            StoreRevision::parse("sha256:1c8065f530f01fb3c8e1dc33a2001b63").unwrap()
        );
        assert_ne!(
            written.write_revision.as_str(),
            written.projection.revision.as_str(),
            "S3: the write token is the file's, never the store's merged digest"
        );
        let StoreValue::Profiles(profiles) = written.projection.value.expect("a value") else {
            panic!("a profiles store carries a profiles value");
        };
        assert_eq!(
            profiles.profiles["team"]["explore"].model.as_deref(),
            Some("anthropic/claude-sonnet-4"),
            "the payload is re-read from the owner, never an echo of `data`"
        );
    }

    #[test]
    fn a_write_answer_without_changed_is_a_typed_unexpected_answer() {
        let mut broken = data("set_config_written");
        broken.as_object_mut().unwrap().remove("changed");
        let channel = FakeChannel::connected(ChannelReply::Widget(response("desktop-2", broken)));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            write_config(
                &mut client,
                WritableStore::Profiles,
                frame("profiles_data"),
                &documented_write_revision(),
            ),
            SetConfigOutcome::UnexpectedAnswer(AnswerError::MissingField { field: "changed" })
        );
    }

    #[test]
    fn the_documented_stale_revision_is_a_conflict() {
        let channel = FakeChannel::connected(answered("revision_conflict", "desktop-2"));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            write_config(
                &mut client,
                WritableStore::Profiles,
                frame("profiles_data"),
                &documented_write_revision(),
            ),
            SetConfigOutcome::RevisionConflict(DesktopError {
                code: "revision_conflict".to_owned(),
                message: "Config changed since expectedRevision was read".to_owned(),
            })
        );
    }

    #[test]
    fn every_documented_refusal_is_its_own_outcome() {
        // The error table (`desktop-rpc.md:934-956`) and the validation order
        // (`:376-383`): each refusal the write can answer is modeled, and an
        // unmodeled code keeps the generic `Rejected`.
        let cases = [
            ("revision_conflict", "RevisionConflict"),
            ("invalid_data", "InvalidData"),
            ("read_only", "ReadOnly"),
            ("config_unavailable", "ConfigUnavailable"),
            ("write_failed", "WriteFailed"),
            ("profile_apply_busy", "ApplyBusy"),
            ("internal_error", "Rejected"),
        ];
        for (code, expected) in cases {
            let refusal = json!({
                "schema": SCHEMA,
                "kind": "response",
                "requestId": "desktop-2",
                "ok": false,
                "error": {"code": code, "message": "fixed"},
            });
            let channel = FakeChannel::connected(ChannelReply::Widget(refusal.to_string()));
            let mut client = DesktopClient::new(Arc::clone(&channel));

            let outcome = write_config(
                &mut client,
                WritableStore::Profiles,
                frame("profiles_data"),
                &documented_write_revision(),
            );
            let variant = match outcome {
                SetConfigOutcome::Written(_) => "Written".to_owned(),
                SetConfigOutcome::RevisionConflict(error) => {
                    assert_eq!(error.code, code);
                    "RevisionConflict".to_owned()
                }
                SetConfigOutcome::InvalidData(error) => {
                    assert_eq!(error.code, code);
                    "InvalidData".to_owned()
                }
                SetConfigOutcome::ReadOnly(error) => {
                    assert_eq!(error.code, code);
                    "ReadOnly".to_owned()
                }
                SetConfigOutcome::ConfigUnavailable(error) => {
                    assert_eq!(error.code, code);
                    "ConfigUnavailable".to_owned()
                }
                SetConfigOutcome::WriteFailed(error) => {
                    assert_eq!(error.code, code);
                    "WriteFailed".to_owned()
                }
                SetConfigOutcome::ApplyBusy(error) => {
                    assert_eq!(error.code, code);
                    "ApplyBusy".to_owned()
                }
                SetConfigOutcome::Rejected(error) => {
                    assert_eq!(error.code, code);
                    "Rejected".to_owned()
                }
                other => panic!("{code} answered {other:?}"),
            };
            assert_eq!(variant, expected, "{code} is its own outcome");
        }
    }

    #[test]
    fn a_request_over_the_channel_limit_never_leaves() {
        let channel = FakeChannel::answering(vec![ChannelReply::Widget(capabilities())]);
        let mut client = DesktopClient::new(Arc::clone(&channel));
        let oversized = json!({ "kind": PROFILES_KIND, "version": 1, "profiles": {}, "blob": "x".repeat(MAX_REQUEST_BYTES) });

        let outcome = write_config(
            &mut client,
            WritableStore::Profiles,
            oversized,
            &documented_write_revision(),
        );

        let SetConfigOutcome::InvalidRequest(RequestError::TooLarge { bytes }) = outcome else {
            panic!("an oversized request is a typed request error: {outcome:?}");
        };
        assert!(bytes > MAX_REQUEST_BYTES);
        assert_eq!(
            channel.methods(),
            ["capabilities"],
            "the channel's 64 KiB cap is checked before anything is written"
        );
    }
}

mod projection_helpers {
    use super::*;

    #[test]
    fn the_open_contract_identifiers_are_the_documented_ones() {
        // The document's own strings (`desktop-rpc.md:110-119`, `:155-176`).
        assert_eq!(PROFILES_KIND, "gentle-pi.agent_model_profiles");
        assert_eq!(FALLBACKS_KIND, "gentle-pi.profile_model_fallbacks");
        assert_eq!(WORKTREE_UNRESOLVED, "worktree_unresolved");
        assert_eq!(StoreScope::Global.as_str(), "global");
        assert_eq!(StoreScope::GlobalAndProject.as_str(), "global+project");
        assert_eq!(StoreScope::LocalAndRepo.as_str(), "local+repo");
        assert_eq!(LayerStatus::Ok.as_str(), "ok");
        assert_eq!(LayerStatus::Missing.as_str(), "missing");
        assert_eq!(LayerStatus::Invalid.as_str(), "invalid");
        assert_eq!(WriteTargetName::Global.as_str(), "global");
        assert_eq!(WriteTargetName::Local.as_str(), "local");
        assert_eq!(WriteTargetName::Repo.as_str(), "repo");
        assert_eq!(OrchestratorMode::Organic.as_str(), "organic");
        assert_eq!(OrchestratorMode::FullDelegation.as_str(), "full-delegation");
        assert_eq!(PinSource::Local.as_str(), "local");
        assert_eq!(PinSource::Repo.as_str(), "repo");
    }

    #[test]
    fn an_empty_revision_is_never_a_token() {
        // S3: "Absent, empty, and non-string tokens are all the client did not
        // prove it saw the current state" - so neither token type holds one.
        assert_eq!(StoreRevision::parse(""), None);
        assert_eq!(WriteRevision::parse(""), None);
        assert!(StoreRevision::parse("sha256:00").is_some());
        assert!(WriteRevision::parse("sha256:00").is_some());
    }

    #[test]
    fn a_projection_remembers_which_store_it_is() {
        let projection: ConfigProjection =
            parse_config_projection(&data("get_config_profiles")).unwrap();
        assert_eq!(projection.store.as_str(), "profiles");
        assert_eq!(projection.store, StoreId::Profiles);
    }
}
