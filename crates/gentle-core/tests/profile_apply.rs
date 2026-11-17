//! Applying one saved profile over the `/gentle:desktop` channel
//! (profiles-editor T2a: S8/S9 and L8's ruling).
//!
//! Two modes exist and they are not interchangeable (`desktop-rpc.md:839-905`):
//! `default` runs the screen's **Make default for new sessions**, which is the
//! only path that can answer `confirmation_required` with a token and a dialog
//! the human must be shown; `session` binds the profile to this RPC session and
//! never asks. L8 established that last fact from gentle-pi's own source, so a
//! `session` answer that asks anyway is reported as an unexpected answer instead
//! of being modeled as a possible round trip.
//!
//! Every literal here is the channel spec's own frame
//! (`fixtures/desktop-rpc-profiles.json`); the two frames the document describes
//! in prose instead of printing are assembled from exactly those sentences and
//! say so. Nothing spawns a process.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::profile_apply::{
    apply_profile, apply_profile_params, parse_apply_profile_answer, ApplyMode,
    ApplyProfileOutcome, ConfirmationToken, NoteLevel, ProfileAnswer, ProfileAnswerError,
    ProfileApply, ProfileConfirmation, ProfileNote, ProfileParamsError, APPLY_PROFILE,
    MAX_NAME_CHARS,
};
use gentle_core::rpc::{
    AnswerError, ChannelReply, DesktopChannel, DesktopClient, DesktopError, DesktopRequest,
    NoAnswer, RequestError, MAX_REQUEST_BYTES, SCHEMA,
};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/desktop-rpc-profiles.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("the fixture is JSON")
}

fn frame(name: &str) -> Value {
    fixture()[name].clone()
}

/// The `data` of a documented response frame.
fn data(name: &str) -> Value {
    frame(name)["data"].clone()
}

/// One documented frame as a child answers OUR request: every member is the
/// document's own except `requestId`, which correlates it with the request.
fn answered(name: &str, request_id: &str) -> ChannelReply {
    let mut answer = frame(name);
    answer["requestId"] = json!(request_id);
    ChannelReply::Widget(answer.to_string())
}

/// The contract `capabilities` reports, with the profiles slice advertised.
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

/// A contract that does NOT advertise `apply_profile` (an older child).
fn capabilities_without_apply() -> String {
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

fn refusal(code: &str) -> ChannelReply {
    ChannelReply::Widget(
        json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": "desktop-2",
            "ok": false,
            "error": {"code": code, "message": "fixed"},
        })
        .to_string(),
    )
}

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

/// The documented token (`desktop-rpc.md:869`), verbatim, ellipsis and all.
fn documented_token() -> ConfirmationToken {
    ConfirmationToken::parse("3f1c…").expect("the documented token")
}

mod params {
    use super::*;

    #[test]
    fn the_documented_request_is_the_profile_and_the_default_mode() {
        // The documented request (`desktop-rpc.md:844`).
        let documented = frame("apply_profile_request");
        let params =
            apply_profile_params("team", ApplyMode::Default, None).expect("a valid profile name");

        assert_eq!(params, documented["params"]);
        assert_eq!(
            params
                .as_object()
                .expect("an object")
                .keys()
                .collect::<Vec<_>>(),
            ["mode", "profile"],
            "a first apply carries no confirmation: it is answered with one"
        );
    }

    #[test]
    fn the_session_mode_is_its_own_typed_value() {
        let params =
            apply_profile_params("team", ApplyMode::Session, None).expect("a valid profile name");

        assert_eq!(params, json!({"profile": "team", "mode": "session"}));
        assert_eq!(ApplyMode::Session.as_str(), "session");
        assert_eq!(ApplyMode::Default.as_str(), "default");
        assert_eq!(ApplyMode::parse("session"), Some(ApplyMode::Session));
        assert_eq!(ApplyMode::parse("default"), Some(ApplyMode::Default));
        assert_eq!(
            ApplyMode::parse("this-chat"),
            None,
            "any other mode is invalid_request"
        );
    }

    #[test]
    fn the_confirmation_token_is_carried_only_on_a_repeat() {
        // The documented round trip (`desktop-rpc.md:869-871`): the token of a
        // `confirmation_required` answer for the same profile and mode.
        let params = apply_profile_params("team", ApplyMode::Default, Some(&documented_token()))
            .expect("a valid profile name");

        assert_eq!(
            params,
            json!({"profile": "team", "mode": "default", "confirmation": "3f1c…"})
        );
        assert_eq!(
            params
                .as_object()
                .expect("an object")
                .keys()
                .collect::<Vec<_>>(),
            ["confirmation", "mode", "profile"],
            "exactly these three keys; any other is invalid_request"
        );
    }

    #[test]
    fn a_profile_name_the_channel_would_refuse_is_never_sent() {
        assert_eq!(
            apply_profile_params("", ApplyMode::Default, None),
            Err(ProfileParamsError::EmptyProfile)
        );
        let longest = "p".repeat(MAX_NAME_CHARS);
        assert!(
            apply_profile_params(&longest, ApplyMode::Default, None).is_ok(),
            "128 characters is the channel's limit, so 128 is accepted"
        );
        let too_long = "p".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(
            apply_profile_params(&too_long, ApplyMode::Default, None),
            Err(ProfileParamsError::ProfileTooLong {
                chars: MAX_NAME_CHARS + 1
            })
        );
        // Counted in characters, not bytes: 64 two-byte characters are within
        // the contract's 128.
        let accented = "é".repeat(MAX_NAME_CHARS);
        assert!(apply_profile_params(&accented, ApplyMode::Session, None).is_ok());
    }

    #[test]
    fn an_empty_or_oversized_token_is_not_a_token() {
        assert_eq!(ConfirmationToken::parse(""), None);
        assert_eq!(
            ConfirmationToken::parse(&"t".repeat(MAX_NAME_CHARS + 1)),
            None
        );
        assert_eq!(
            ConfirmationToken::parse(&"t".repeat(MAX_NAME_CHARS))
                .expect("a token")
                .as_str(),
            "t".repeat(MAX_NAME_CHARS)
        );
    }

    #[test]
    fn a_request_that_cannot_be_built_is_never_written() {
        let channel = FakeChannel::answering(vec![ChannelReply::Widget(capabilities())]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "", ApplyMode::Default, None),
            ApplyProfileOutcome::InvalidParams(ProfileParamsError::EmptyProfile)
        );
        assert_eq!(
            channel.methods(),
            Vec::<String>::new(),
            "nothing was written, not even the contract"
        );
    }
}

mod answer {
    use super::*;

    #[test]
    fn the_documented_confirmation_required_answer_parses_the_dialog() {
        // The documented answer (`desktop-rpc.md:856-874`): the screen's own
        // dialog, its token, and nothing written.
        let answer = parse_apply_profile_answer(&data("confirmation_required"))
            .expect("the documented answer");

        assert_eq!(
            answer,
            ProfileAnswer::ConfirmationRequired(ProfileConfirmation {
                profile: "team".to_owned(),
                mode: ApplyMode::Default,
                title: "Apply profile \"team\"?".to_owned(),
                message: concat!(
                    "Profile \"team\" has agent routing entries. Applying it will replace ",
                    "global routing in … with this profile's routes: worker: openai/alpha ",
                    "(added). Continue?"
                )
                .to_owned(),
                token: documented_token(),
            })
        );
    }

    #[test]
    fn the_documented_applied_answer_parses_its_notes() {
        // The documented success (`desktop-rpc.md:884-897`).
        let answer = parse_apply_profile_answer(&data("applied")).expect("the documented answer");

        assert_eq!(
            answer,
            ProfileAnswer::Applied(ProfileApply {
                profile: "team".to_owned(),
                mode: ApplyMode::Default,
                notes: vec![ProfileNote {
                    level: NoteLevel::Info,
                    text: "el Gentleman applied profile \"team\" — 1 agent updated. …".to_owned(),
                }],
            })
        );
    }

    #[test]
    fn a_not_applied_answer_is_its_own_outcome() {
        // The document describes this status in prose (`desktop-rpc.md:891-893`):
        // `not_applied` when the screen's last notice was a warning or there was
        // none, with `notes` saying why. The frame is assembled from that
        // sentence and the document's own note shape.
        let not_applied = json!({
            "profile": "team",
            "mode": "default",
            "status": "not_applied",
            "notes": [
                {"level": "warning", "text": "pinned repository: this clone re-pinned instead"},
            ],
        });

        let answer = parse_apply_profile_answer(&not_applied).expect("the answer parses");
        assert_eq!(
            answer,
            ProfileAnswer::NotApplied(ProfileApply {
                profile: "team".to_owned(),
                mode: ApplyMode::Default,
                notes: vec![ProfileNote {
                    level: NoteLevel::Warning,
                    text: "pinned repository: this clone re-pinned instead".to_owned(),
                }],
            })
        );
    }

    #[test]
    fn a_session_apply_that_asks_for_a_confirmation_is_an_unexpected_answer() {
        // L8/L9: `session` never asks (`desktop-rpc.md:880`), so this answer is
        // reported, never acted on: there is no token to resend.
        let asking = json!({
            "profile": "team",
            "mode": "session",
            "status": "confirmation_required",
            "confirmation": {"title": "Apply profile \"team\"?", "message": "…", "token": "3f1c…"},
        });

        assert_eq!(
            parse_apply_profile_answer(&asking),
            Err(ProfileAnswerError::ConfirmationForSession)
        );
    }

    #[test]
    fn an_answer_of_another_shape_is_a_typed_error_never_a_panic() {
        let mut broken = data("applied");
        broken.as_object_mut().unwrap().remove("status");
        assert_eq!(
            parse_apply_profile_answer(&broken),
            Err(ProfileAnswerError::Field(AnswerError::MissingField {
                field: "status"
            }))
        );

        let mut broken = data("applied");
        broken["status"] = json!("applied-ish");
        assert_eq!(
            parse_apply_profile_answer(&broken),
            Err(ProfileAnswerError::Field(AnswerError::UnknownValue {
                field: "status",
                value: "applied-ish".to_owned(),
            }))
        );

        let mut broken = data("applied");
        broken["mode"] = json!("global");
        assert_eq!(
            parse_apply_profile_answer(&broken),
            Err(ProfileAnswerError::Field(AnswerError::UnknownValue {
                field: "mode",
                value: "global".to_owned(),
            }))
        );

        let mut broken = data("confirmation_required");
        broken["confirmation"]
            .as_object_mut()
            .unwrap()
            .remove("token");
        assert_eq!(
            parse_apply_profile_answer(&broken),
            Err(ProfileAnswerError::Field(AnswerError::MissingField {
                field: "token"
            }))
        );

        let mut broken = data("applied");
        broken["notes"][0]["level"] = json!("fatal");
        assert_eq!(
            parse_apply_profile_answer(&broken),
            Err(ProfileAnswerError::Field(AnswerError::UnknownValue {
                field: "level",
                value: "fatal".to_owned(),
            }))
        );

        // An apply with no notices at all is one of the document's own
        // `not_applied` cases (`:891-893`), so an absent list reads as empty.
        let mut quiet = data("applied");
        quiet.as_object_mut().unwrap().remove("notes");
        assert_eq!(
            parse_apply_profile_answer(&quiet),
            Ok(ProfileAnswer::Applied(ProfileApply {
                profile: "team".to_owned(),
                mode: ApplyMode::Default,
                notes: Vec::new(),
            }))
        );
    }

    #[test]
    fn unknown_extra_keys_are_ignored_on_read() {
        let mut grown = data("applied");
        grown["future"] = json!({"anything": true});
        grown["notes"][0]["detail"] = json!("kept out");

        let answer = parse_apply_profile_answer(&grown).expect("extra members are ignored");
        let ProfileAnswer::Applied(applied) = answer else {
            panic!("still an applied answer");
        };
        assert_eq!(applied.notes.len(), 1);
        assert_eq!(applied.notes[0].level, NoteLevel::Info);
    }
}

mod over_the_channel {
    use super::*;

    #[test]
    fn the_apply_asks_capabilities_first_and_then_the_documented_request() {
        let channel = FakeChannel::connected(answered("applied", "desktop-2"));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let outcome = apply_profile(&mut client, "team", ApplyMode::Default, None);

        assert!(matches!(outcome, ApplyProfileOutcome::Applied(_)));
        assert_eq!(
            channel.methods(),
            ["capabilities", APPLY_PROFILE],
            "a client asks the contract once, then the method it was told about"
        );
        assert_eq!(
            channel.sent()[1].params(),
            &frame("apply_profile_request")["params"],
            "the body is the documented one"
        );
        assert_eq!(
            channel.sent()[1]
                .params()
                .as_object()
                .expect("an object")
                .keys()
                .collect::<Vec<_>>(),
            ["mode", "profile"],
            "a first apply carries exactly profile and mode: the answer is what asks"
        );
    }

    #[test]
    fn the_documented_round_trip_asks_then_repeats_with_the_token() {
        // The whole documented path (`desktop-rpc.md:856-877`): the answer is a
        // dialog, the human approves, the request is resent with the token.
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities()),
            answered("confirmation_required", "desktop-2"),
            answered("applied", "desktop-3"),
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let first = apply_profile(&mut client, "team", ApplyMode::Default, None);
        let ApplyProfileOutcome::ConfirmationRequired(confirmation) = first else {
            panic!("the documented first answer is a dialog: {first:?}");
        };
        let second = apply_profile(
            &mut client,
            "team",
            ApplyMode::Default,
            Some(&confirmation.token),
        );

        assert!(matches!(second, ApplyProfileOutcome::Applied(_)));
        assert_eq!(
            channel.methods(),
            ["capabilities", APPLY_PROFILE, APPLY_PROFILE],
            "the contract is cached: one round trip per attempt, not per module"
        );
        assert_eq!(
            channel.sent()[2].params(),
            &json!({"profile": "team", "mode": "default", "confirmation": "3f1c…"}),
            "the repeat carries the token the child itself issued"
        );
    }

    #[test]
    fn every_documented_refusal_is_its_own_outcome() {
        let cases = [
            ("unknown_profile", "UnknownProfile"),
            ("profile_apply_busy", "Busy"),
            ("profiles_unavailable", "ProfilesUnavailable"),
            ("invalid_request", "Rejected"),
            ("internal_error", "Rejected"),
        ];
        for (code, expected) in cases {
            let channel = FakeChannel::connected(refusal(code));
            let mut client = DesktopClient::new(Arc::clone(&channel));

            let outcome = apply_profile(&mut client, "team", ApplyMode::Default, None);
            let variant = match outcome {
                ApplyProfileOutcome::UnknownProfile(error) => {
                    assert_eq!(error.code, code);
                    "UnknownProfile".to_owned()
                }
                ApplyProfileOutcome::Busy(error) => {
                    assert_eq!(error.code, code);
                    "Busy".to_owned()
                }
                ApplyProfileOutcome::ProfilesUnavailable(error) => {
                    assert_eq!(error.code, code);
                    "ProfilesUnavailable".to_owned()
                }
                ApplyProfileOutcome::Rejected(error) => {
                    assert_eq!(error.code, code);
                    "Rejected".to_owned()
                }
                other => panic!("{code} answered {other:?}"),
            };
            assert_eq!(variant, expected, "{code} is its own outcome");
        }
    }

    #[test]
    fn a_child_whose_capabilities_lack_the_method_is_never_asked() {
        let channel =
            FakeChannel::answering(vec![ChannelReply::Widget(capabilities_without_apply())]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "team", ApplyMode::Session, None),
            ApplyProfileOutcome::Unsupported
        );
        assert_eq!(
            channel.methods(),
            ["capabilities"],
            "the channel is never sent a method the child did not advertise"
        );
    }

    #[test]
    fn a_child_that_never_answers_degrades_to_a_timeout() {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities()),
            ChannelReply::Timeout,
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "team", ApplyMode::Session, None),
            ApplyProfileOutcome::NoAnswer(NoAnswer::TimedOut),
            "a chat that does not answer cannot be reported as applied"
        );
    }

    #[test]
    fn a_payload_this_contract_does_not_model_is_no_answer() {
        let channel =
            FakeChannel::connected(ChannelReply::Widget("{\"not\":\"an envelope\"}".into()));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "team", ApplyMode::Session, None),
            ApplyProfileOutcome::NoAnswer(NoAnswer::Malformed)
        );
    }

    #[test]
    fn an_ok_answer_this_contract_does_not_model_is_a_typed_unexpected_answer() {
        let asking = json!({
            "profile": "team",
            "mode": "session",
            "status": "confirmation_required",
            "confirmation": {"title": "t", "message": "m", "token": "3f1c…"},
        });
        let channel = FakeChannel::connected(ChannelReply::Widget(response("desktop-2", asking)));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "team", ApplyMode::Session, None),
            ApplyProfileOutcome::UnexpectedAnswer(ProfileAnswerError::ConfirmationForSession)
        );
    }

    #[test]
    fn the_profile_name_limit_is_reached_long_before_the_channel_limit() {
        // The channel caps one envelope at 64 KiB, and this method's own name
        // limit is 128 characters: a request that could ever reach the envelope
        // cap cannot be built here at all, so nothing is written for one.
        let channel = FakeChannel::answering(vec![ChannelReply::Widget(capabilities())]);
        let mut client = DesktopClient::new(Arc::clone(&channel));
        let oversized = "p".repeat(MAX_REQUEST_BYTES);

        let outcome = apply_profile(&mut client, &oversized, ApplyMode::Default, None);

        assert_eq!(
            outcome,
            ApplyProfileOutcome::InvalidParams(ProfileParamsError::ProfileTooLong {
                chars: MAX_REQUEST_BYTES
            }),
            "the profile name is 128 characters at most, long before 64 KiB"
        );
        assert_eq!(channel.methods(), Vec::<String>::new());
    }

    #[test]
    fn a_desktop_error_still_reaches_the_caller_untouched() {
        let channel = FakeChannel::connected(refusal("unknown_profile"));
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "ghost", ApplyMode::Session, None),
            ApplyProfileOutcome::UnknownProfile(DesktopError {
                code: "unknown_profile".to_owned(),
                message: "fixed".to_owned(),
            })
        );
    }

    #[test]
    fn a_client_with_no_contract_never_guesses_a_method() {
        // The channel's envelope limits are `DesktopRequest`'s, not this
        // module's: a request it refuses never left the client.
        let channel = FakeChannel::answering(Vec::new());
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            apply_profile(&mut client, "team", ApplyMode::Default, None),
            ApplyProfileOutcome::NoAnswer(NoAnswer::NotConnected)
        );
        assert_eq!(
            channel.methods(),
            ["capabilities"],
            "no contract, no method: support is never inferred"
        );
        // `RequestError` is still part of this outcome so a caller reads one
        // enum, not two.
        let _: fn(RequestError) -> ApplyProfileOutcome = ApplyProfileOutcome::InvalidRequest;
    }
}
