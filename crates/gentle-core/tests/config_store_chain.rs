//! Editing ONE profile's fallback chain as a `set_config` of the whole
//! `profile_fallbacks` document (profiles-editor T5, S2, S3, L30-3).
//!
//! The document a write sends is the store's `value` EXACTLY as the owner sent
//! it - every member this reader does not model survives - with only
//! `profiles.<profile>.<agent>` replaced. The three rules the owner's own
//! normalizer enforces are checked HERE, before a request exists: at most eight
//! steps, each step naming a model, and no model twice in one chain. The owner
//! also refuses an EMPTY chain (`desktop-rpc.md:382`), so that is one of ours
//! too: a local rejection is never the owner's `invalid_data`.
//!
//! The frames are the doc-quoted ones of `fixtures/desktop-rpc-profiles.json`
//! (`desktop-rpc.md:331-345`), so the edited document is compared with the
//! channel's own canonical shape.

use gentle_core::rpc::config_store::{
    parse_config_verbatim, parse_store_value, set_config_params, validate_chain,
    with_profile_chain, ChainInvalid, FallbackStep, StoreId, StoreValue, WritableStore,
    WriteTargetName, MAX_CHAIN_STEPS,
};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/desktop-rpc-profiles.json");

/// One documented frame, by the name the fixture gives it.
fn frame(name: &str) -> Value {
    serde_json::from_str::<Value>(FIXTURE).expect("the fixture is JSON")[name].clone()
}

/// A fallbacks document carrying every trap this rule must survive: an unknown
/// member of its own, an unknown member of a chain map, an unknown member of
/// ANOTHER profile's step, and a second profile.
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

#[test]
fn the_edited_chain_is_replaced_and_every_other_member_survives() {
    let original = document();

    let edited = with_profile_chain(
        &original,
        "team",
        "worker",
        &steps(&[("openai/gpt-next", Some("high")), ("openai/gpt-last", None)]),
    )
    .expect("the document holds that profile");

    assert_eq!(
        edited,
        json!({
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "future": {"seed": true},
            "profiles": {
                "team": {
                    "*": [{"model": "openai/gpt-shared"}],
                    "worker": [
                        {"model": "openai/gpt-next", "thinking": "high"},
                        {"model": "openai/gpt-last"}
                    ]
                },
                "busy": {
                    "*": [{"model": "openai/gpt-busy", "futureStep": {"weight": 2}}]
                }
            }
        }),
        "S2: `data` is the COMPLETE content, so only the edited chain may differ"
    );
}

#[test]
fn the_profile_wide_chain_is_edited_exactly_like_an_agents() {
    let original = document();

    let edited = with_profile_chain(
        &original,
        "team",
        "*",
        &steps(&[("openai/gpt-shared", Some("medium"))]),
    )
    .expect("the document holds that profile");

    assert_eq!(
        edited["profiles"]["team"]["*"],
        json!([{"model": "openai/gpt-shared", "thinking": "medium"}])
    );
    assert_eq!(
        edited["profiles"]["busy"]["*"], original["profiles"]["busy"]["*"],
        "another profile's `*` is not this write's business"
    );
}

#[test]
fn a_profile_the_document_does_not_hold_is_never_created() {
    let original = document();

    assert_eq!(
        with_profile_chain(&original, "gone", "*", &steps(&[("p/m", None)])),
        None,
        "a name the fallbacks store does not record writes nothing at all"
    );
}

#[test]
fn a_shape_this_rule_does_not_model_is_never_rewritten() {
    assert_eq!(
        with_profile_chain(&json!({"value": "not a document"}), "team", "*", &[]),
        None
    );
    assert_eq!(
        with_profile_chain(&json!({"profiles": []}), "team", "*", &[]),
        None,
        "a `profiles` that is not an object is not a document to write"
    );
    assert_eq!(
        with_profile_chain(&json!({"profiles": {"team": []}}), "team", "*", &[]),
        None,
        "a chain map that is not an object is not one to edit"
    );
}

#[test]
fn a_chain_of_eight_distinct_steps_is_accepted() {
    let eight: Vec<String> = (1..=8).map(|index| format!("p/model-{index}")).collect();
    let owned: Vec<(&str, Option<&str>)> =
        eight.iter().map(|model| (model.as_str(), None)).collect();

    assert_eq!(
        validate_chain(&steps(&owned)),
        Ok(()),
        "the owner's own cap is {MAX_CHAIN_STEPS} steps (`desktop-rpc.md:345`)"
    );
}

#[test]
fn a_chain_of_nine_steps_is_refused() {
    let nine: Vec<String> = (1..=9).map(|index| format!("p/model-{index}")).collect();
    let owned: Vec<(&str, Option<&str>)> =
        nine.iter().map(|model| (model.as_str(), None)).collect();

    assert_eq!(
        validate_chain(&steps(&owned)),
        Err(ChainInvalid::TooLong { steps: 9 })
    );
}

#[test]
fn a_chain_that_repeats_a_model_is_refused() {
    assert_eq!(
        validate_chain(&steps(&[
            ("p/a", None),
            ("p/b", None),
            ("p/a", Some("high"))
        ])),
        Err(ChainInvalid::RepeatedModel {
            model: "p/a".to_owned()
        }),
        "a model appears at most once per chain"
    );
}

#[test]
fn a_step_that_names_no_model_is_refused() {
    assert_eq!(
        validate_chain(&steps(&[("p/a", None), ("  ", Some("low"))])),
        Err(ChainInvalid::MissingModel { index: 1 }),
        "every step is `{{model, thinking?}}`: a blank model is no step"
    );
}

#[test]
fn an_empty_chain_is_refused() {
    assert_eq!(
        validate_chain(&[]),
        Err(ChainInvalid::Empty),
        "the owner refuses an empty chain (`desktop-rpc.md:382`), so this one never leaves"
    );
}

#[test]
fn the_write_names_the_fallbacks_store_and_the_documents_write_revision() {
    let read = parse_config_verbatim(&json!({
        "store": "profile_fallbacks",
        "scope": "global",
        "readOnly": false,
        "status": "ok",
        "revision": "sha256:fallbacks-read",
        "value": document(),
        "layers": [{"source": "global", "status": "ok"}],
        "writeTargets": [{
            "target": "global",
            "status": "ok",
            "writeRevision": "sha256:fallbacks-write"
        }]
    }))
    .expect("a documented answer");
    let data = with_profile_chain(
        read.verbatim.as_ref().expect("the read carried its value"),
        "team",
        "worker",
        &steps(&[("openai/gpt-next", None)]),
    )
    .expect("the document holds that profile");
    let expected = read
        .projection
        .write_revision(WriteTargetName::Global)
        .expect("the read offers the global write target");
    assert_ne!(
        read.projection.revision.as_str(),
        expected.as_str(),
        "S3: the store's revision is never the write token"
    );

    let params = set_config_params(WritableStore::ProfileFallbacks, data, expected);

    assert_eq!(params["store"], json!("profile_fallbacks"));
    assert_eq!(params["expectedRevision"], json!("sha256:fallbacks-write"));
    assert_eq!(
        params
            .as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        ["data", "expectedRevision", "store"],
        "the params are exactly the contract's three"
    );
}

#[test]
fn the_documented_frame_edited_keeps_the_owners_canonical_shape() {
    let value = frame("fallbacks_data");

    let edited = with_profile_chain(
        &value,
        "team",
        "worker",
        &steps(&[
            ("openai/gpt-next", Some("high")),
            ("anthropic/claude-haiku-4-5", None),
        ]),
    )
    .expect("the documented document holds its own profile");

    assert_eq!(
        edited,
        json!({
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "profiles": {
                "team": {
                    "*": [{"model": "openai/gpt-shared"}],
                    "worker": [
                        {"model": "openai/gpt-next", "thinking": "high"},
                        {"model": "anthropic/claude-haiku-4-5"}
                    ]
                }
            }
        }),
        "the documented shape (`desktop-rpc.md:331-345`) is what this write produces"
    );
}

#[test]
fn a_chain_the_owner_sends_back_parses_as_the_same_chain() {
    let original = document();
    let mine = steps(&[("openai/gpt-next", Some("high")), ("openai/gpt-last", None)]);
    let edited =
        with_profile_chain(&original, "team", "worker", &mine).expect("the document holds it");

    let parsed = parse_store_value(StoreId::ProfileFallbacks, &edited)
        .expect("the document this write produces is one the reader reads");

    let StoreValue::Fallbacks(document) = parsed else {
        panic!("a fallbacks store carries a fallbacks value");
    };
    assert_eq!(document.profiles["team"]["worker"], mine);
    assert_eq!(
        document.profiles["team"]["*"][0].model, "openai/gpt-shared",
        "the other chain came back untouched"
    );
}
