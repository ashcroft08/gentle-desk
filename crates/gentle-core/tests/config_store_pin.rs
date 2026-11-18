//! Editing ONE pin layer as a `set_config` of the `pins` store (profiles-editor
//! T6, S2, S3).
//!
//! The `pins` store is the one store whose `data` is a LAYER and not a whole
//! document: exactly `{source, profile}`, where `profile` is a defined profile
//! name or the literal `null` to clear that layer (`desktop-rpc.md:322-372`).
//! Two rules follow and both are pinned here before a request exists:
//!
//! - the request names ONE layer, so the other layer's file is untouched by
//!   construction - there is no document to rebuild and nothing of the other
//!   layer can leak into this one's payload;
//! - exactly two keys are accepted, so an extra member is `invalid_data`: the
//!   builder here can only ever produce those two.
//!
//! The token is that layer's OWN `writeTargets[].writeRevision`
//! (`desktop-rpc.md:246-273`): `pins` has two targets - `local` and `repo` - and
//! a write preconditioned on the other layer's token would answer
//! `revision_conflict` for ever.
//!
//! The frames are the doc-quoted ones of `fixtures/desktop-rpc-profiles.json`
//! (`desktop-rpc.md:354-360`), so the payload is compared with the channel's own
//! two examples.

use gentle_core::rpc::config_store::{
    get_config_params, parse_config_projection, pin_data, set_config_params, validate_pin,
    PinInvalid, PinSource, StoreId, WritableStore, WriteTargetName,
};
use gentle_core::rpc::WriteRevision;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/desktop-rpc-profiles.json");

/// One documented frame, by the name the fixture gives it.
fn frame(name: &str) -> Value {
    serde_json::from_str::<Value>(FIXTURE).expect("the fixture is JSON")[name].clone()
}

/// The `pins` read the fixture documents, typed: two layers and their tokens.
fn pins_read() -> gentle_core::rpc::ConfigProjection {
    let mut answer = frame("pins_value");
    answer["writeTargets"] = json!([
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
    ]);
    answer["readOnly"] = json!(false);
    parse_config_projection(&answer).expect("the documented pins projection")
}

/// The very token one layer's entry carries.
fn token_of(source: PinSource) -> WriteRevision {
    pins_read()
        .write_revision(source.write_target())
        .expect("the documented read offers that layer")
        .clone()
}

#[test]
fn a_pin_names_its_layer_and_its_profile_and_nothing_else() {
    let data = pin_data(PinSource::Local, Some("team")).expect("a profile name is a pin");

    assert_eq!(
        data,
        frame("pin_local_data"),
        "the documented payload of a set: exactly `{{source, profile}}`"
    );
    assert_eq!(
        data.as_object()
            .expect("an object")
            .keys()
            .collect::<Vec<_>>(),
        ["profile", "source"],
        "an extra key is `invalid_data`, so this builder can only produce the two"
    );
}

#[test]
fn a_clear_names_the_layer_and_a_null_profile() {
    let data = pin_data(PinSource::Repo, None).expect("a clear is a pin request too");

    assert_eq!(
        data,
        frame("pin_clear_data"),
        "the documented payload of a clear: `null` is the explicit removal, not an absent key"
    );
    assert!(
        data["profile"].is_null(),
        "the clear carries the literal null the owner documents, never a dropped member"
    );
}

#[test]
fn a_generated_pin_keeps_the_other_layer_out_of_the_request() {
    // The `pins` store is written one LAYER at a time (`desktop-rpc.md:322-345`),
    // so "the other layer survives" is a property of the request's own shape: no
    // request can carry the other layer, whatever this app read.
    for (source, other) in [(PinSource::Local, "repo"), (PinSource::Repo, "local")] {
        let data = pin_data(source, Some("team")).expect("a profile name is a pin");

        assert_eq!(data["source"], json!(source.as_str()));
        assert!(
            !data.to_string().contains(other),
            "{source:?}: the request names one layer, so the {other} layer's file is untouched"
        );
    }
}

#[test]
fn a_pin_write_is_preconditioned_on_that_layers_own_token() {
    let local = token_of(PinSource::Local);
    let repo = token_of(PinSource::Repo);
    assert_ne!(local, repo, "the two pin files have two write revisions");

    let params = set_config_params(
        WritableStore::Pins,
        pin_data(PinSource::Local, Some("team")).expect("a pin"),
        &local,
    );

    assert_eq!(
        params,
        json!({
            "store": "pins",
            "data": { "source": "local", "profile": "team" },
            "expectedRevision": local.as_str(),
        }),
        "S3: the token is the target the write touches, never the store's `revision`"
    );
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
fn the_pins_store_offers_the_two_layer_targets_and_no_other() {
    assert_eq!(
        WritableStore::Pins.write_targets(),
        [WriteTargetName::Local, WriteTargetName::Repo],
        "`pins` has one file per layer (`desktop-rpc.md:246-262`)"
    );
    assert_eq!(PinSource::Local.write_target(), WriteTargetName::Local);
    assert_eq!(PinSource::Repo.write_target(), WriteTargetName::Repo);
}

#[test]
fn the_read_a_pin_write_starts_from_names_the_pins_store_only() {
    assert_eq!(
        get_config_params(StoreId::Pins),
        json!({ "store": "pins" }),
        "a layer is named by the write, never by the read"
    );
}

#[test]
fn a_pin_the_owner_would_refuse_never_becomes_a_request() {
    for blank in ["", " ", "\t\n"] {
        assert_eq!(
            validate_pin(Some(blank)),
            Err(PinInvalid::BlankProfile),
            "{blank:?}: an empty profile is `invalid_data`, so it is caught here"
        );
        assert_eq!(
            pin_data(PinSource::Repo, Some(blank)),
            Err(PinInvalid::BlankProfile),
            "{blank:?}: and the builder cannot produce the payload either"
        );
    }
    assert_eq!(validate_pin(None), Ok(()), "a clear names no profile");
    assert_eq!(
        validate_pin(Some("team")),
        Ok(()),
        "a name the profiles store defines is the owner's own check"
    );
}
