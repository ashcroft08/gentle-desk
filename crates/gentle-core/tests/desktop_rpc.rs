//! The `/gentle:desktop` transport: the request envelope, the answer's
//! correlation, and the silent degradation of a child that does not answer.
//!
//! Every literal below is the channel spec's own example, verbatim from
//! `desktop-rpc.md` of gentle-pi 4.8.0 (`gentle-shell.desktop-rpc/v1`), except
//! the outer `extension_ui_request` frame: that is pi's (`fixtures/desktop-rpc.jsonl`,
//! decoded the same way every other recorded frame is).
//!
//! The real-IO half of the transport is the backend seam
//! (`crates/backend/src/chat_host/desktop.rs`); this file drives the pure half
//! through an in-memory channel, and nothing here spawns a process.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::{
    decode_line, parse_widget_payload, widget_payload, ChannelReply, Decoded, DesktopChannel,
    DesktopClient, DesktopError, DesktopOutcome, DesktopRequest, NoAnswer, RequestError,
    RpcCommand, RpcEvent, UiRequest, WidgetPayload, MAX_METHOD_CHARS, MAX_REQUEST_BYTES,
    MAX_REQUEST_ID_CHARS, SCHEMA, WIDGET_KEY,
};
use serde_json::{json, Value};

/// The prompt the spec documents: `/gentle:desktop` followed by one JSON request.
const DOC_PROMPT: &str = r#"{"id":"pi-42","type":"prompt","message":"/gentle:desktop {\"schema\":\"gentle-shell.desktop-rpc/v1\",\"requestId\":\"desktop-42\",\"method\":\"capabilities\",\"params\":{}}"}"#;

/// The `get_config` request the spec documents.
const DOC_GET_CONFIG_REQUEST: &str = r#"{
  "schema": "gentle-shell.desktop-rpc/v1",
  "requestId": "desktop-46",
  "method": "get_config",
  "params": { "store": "profiles" }
}"#;

/// The contract `capabilities` returns, exactly as the spec prints it.
const DOC_CAPABILITIES: &str = r#"{
  "schema": "gentle-shell.desktop-rpc/v1",
  "command": "gentle:desktop",
  "widget": "gentle-desktop-rpc",
  "methods": ["capabilities", "get_snapshot", "get_config", "set_config", "get_permission_mode", "set_permission_mode", "get_subscription_usage", "refresh_subscription_usage", "get_design_canvas", "list_jobs", "cancel_job", "apply_profile", "list_subagents", "cancel_subagent"],
  "events": ["config_changed", "permission_mode_changed", "rate_limit", "design_canvas", "jobs_update", "profile_applied", "agents_update"]
}"#;

/// The `failure` envelope the spec documents (`revision_conflict`).
const DOC_ERROR: &str = r#"{
  "schema": "gentle-shell.desktop-rpc/v1",
  "kind": "response",
  "requestId": "desktop-49",
  "ok": false,
  "error": { "code": "revision_conflict", "message": "Config changed since expectedRevision was read" }
}"#;

const FIXTURE: &str = include_str!("fixtures/desktop-rpc.jsonl");

fn fixture(index: usize) -> String {
    FIXTURE
        .lines()
        .filter(|line| !line.trim().is_empty())
        .nth(index)
        .unwrap_or_else(|| panic!("fixture line {index} is missing"))
        .to_owned()
}

fn value(raw: &str) -> Value {
    serde_json::from_str(raw).expect("the fixture is JSON")
}

/// The payload pi's frame carries, decoded through this workspace's own codec
/// (so the fixture proves the outer frame, not only the payload).
fn payload_of(frame: &str) -> Option<String> {
    match decode_line(frame) {
        Decoded::Event(RpcEvent::UiRequest(UiRequest::FireAndForget { method, raw, .. })) => {
            assert_eq!(method, "setWidget");
            widget_payload(&raw).map(str::to_owned)
        }
        other => panic!("expected a setWidget frame, got {other:?}"),
    }
}

/// One pi frame carrying `payload` under the desktop widget key.
fn frame_with(payload: &Value) -> Value {
    json!({
        "type": "extension_ui_request",
        "id": "ui-9",
        "method": "setWidget",
        "widgetKey": WIDGET_KEY,
        "widgetLines": [payload.to_string()],
    })
}

/// The capabilities answer the client caches, built from the spec's own
/// contract document.
fn capabilities_answer() -> Value {
    json!({
        "schema": SCHEMA,
        "kind": "response",
        "requestId": "desktop-1",
        "ok": true,
        "data": value(DOC_CAPABILITIES),
    })
}

fn ok_answer(request_id: &str, data: Value) -> Value {
    json!({"schema": SCHEMA, "kind": "response", "requestId": request_id, "ok": true, "data": data})
}

/// A channel that answers with what a test queued and records every request it
/// was asked to carry. It never touches a pipe.
#[derive(Default)]
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

    /// A channel that answers `capabilities` and nothing else.
    fn connected() -> Arc<Self> {
        Self::answering(vec![ChannelReply::Widget(
            capabilities_answer().to_string(),
        )])
    }

    fn sent(&self) -> Vec<DesktopRequest> {
        self.sent.lock().unwrap().clone()
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

mod request {
    use super::*;

    #[test]
    fn the_documented_prompt_is_what_the_builder_writes() {
        let request = DesktopRequest::new("desktop-42", "capabilities", json!({})).unwrap();

        let documented = value(DOC_PROMPT);
        let message = documented["message"].as_str().expect("a message");
        let envelope = message
            .strip_prefix("/gentle:desktop ")
            .expect("the documented command prefix");
        let written = request
            .prompt()
            .strip_prefix("/gentle:desktop ")
            .expect("the command prefix")
            .to_owned();

        // The same envelope the spec prints, member for member: the key order
        // on the wire is this workspace's own (`serde_json` sorts them).
        assert_eq!(value(&written), value(envelope));
        assert_eq!(value(&request.json()), value(envelope));
    }

    #[test]
    fn the_request_is_one_compact_envelope_under_the_command() {
        let request = DesktopRequest::new("desktop-42", "capabilities", json!({})).unwrap();

        assert_eq!(
            request.prompt(),
            concat!(
                "/gentle:desktop {",
                "\"method\":\"capabilities\",",
                "\"params\":{},",
                "\"requestId\":\"desktop-42\",",
                "\"schema\":\"gentle-shell.desktop-rpc/v1\"}"
            )
        );
        assert!(!request.prompt().contains('\n'));
    }

    #[test]
    fn the_documented_get_config_request_round_trips() {
        let request =
            DesktopRequest::new("desktop-46", "get_config", json!({"store": "profiles"})).unwrap();

        assert_eq!(value(&request.json()), value(DOC_GET_CONFIG_REQUEST));
        assert_eq!(request.method(), "get_config");
        assert_eq!(request.params(), &json!({"store": "profiles"}));
    }

    #[test]
    fn a_request_id_of_128_characters_is_accepted_and_129_refused() {
        let longest = "r".repeat(MAX_REQUEST_ID_CHARS);
        assert!(DesktopRequest::new(longest.clone(), "capabilities", json!({})).is_ok());

        let too_long = "r".repeat(MAX_REQUEST_ID_CHARS + 1);
        assert_eq!(
            DesktopRequest::new(too_long, "capabilities", json!({})),
            Err(RequestError::RequestIdTooLong {
                chars: MAX_REQUEST_ID_CHARS + 1
            })
        );
    }

    #[test]
    fn an_empty_request_id_is_refused() {
        assert_eq!(
            DesktopRequest::new("", "capabilities", json!({})),
            Err(RequestError::EmptyRequestId)
        );
    }

    #[test]
    fn a_method_of_64_characters_is_accepted_and_65_refused() {
        let longest = "a".repeat(MAX_METHOD_CHARS);
        assert!(DesktopRequest::new("desktop-1", &longest, json!({})).is_ok());

        let too_long = "a".repeat(MAX_METHOD_CHARS + 1);
        assert_eq!(
            DesktopRequest::new("desktop-1", &too_long, json!({})),
            Err(RequestError::MethodTooLong {
                chars: MAX_METHOD_CHARS + 1
            })
        );
    }

    #[test]
    fn a_method_that_is_not_lowercase_snake_case_is_refused() {
        for method in [
            "Get_Snapshot",
            "get-snapshot",
            "get.snapshot",
            "get snapshot",
            "",
        ] {
            let error = DesktopRequest::new("desktop-1", method, json!({}))
                .err()
                .unwrap_or_else(|| panic!("{method:?} was accepted"));
            assert!(
                matches!(
                    error,
                    RequestError::MethodNotSnakeCase { .. } | RequestError::EmptyMethod
                ),
                "{method:?} answered {error:?}"
            );
        }
    }

    #[test]
    fn a_request_of_64_kib_is_accepted_and_one_byte_more_is_refused() {
        let empty = DesktopRequest::new("desktop-1", "capabilities", json!({"pad": ""}))
            .unwrap()
            .json()
            .len();
        let fits = MAX_REQUEST_BYTES - empty;

        let at_the_limit = DesktopRequest::new(
            "desktop-1",
            "capabilities",
            json!({"pad": "x".repeat(fits)}),
        );
        assert!(
            at_the_limit.is_ok(),
            "the documented 64 KiB limit must pass"
        );

        let over = DesktopRequest::new(
            "desktop-1",
            "capabilities",
            json!({"pad": "x".repeat(fits + 1)}),
        );
        assert_eq!(
            over,
            Err(RequestError::TooLarge {
                bytes: MAX_REQUEST_BYTES + 1
            })
        );
    }

    #[test]
    fn the_command_is_an_ordinary_pi_prompt() {
        let request = DesktopRequest::new("desktop-42", "capabilities", json!({})).unwrap();

        assert_eq!(
            request.command(),
            RpcCommand::Prompt {
                message: request.prompt()
            }
        );
    }
}

mod answer {
    use super::*;

    #[test]
    fn the_widget_frame_carries_the_documented_answer() {
        let payload = payload_of(&fixture(1)).expect("a desktop payload");

        let Some(WidgetPayload::Response(response)) = parse_widget_payload(&payload) else {
            panic!("expected a response envelope, got {payload}");
        };
        assert_eq!(response.request_id.as_deref(), Some("desktop-46"));
        let data = response.outcome.expect("an ok answer");
        assert_eq!(data["store"], "profiles");
        assert_eq!(data["revision"], "sha256:1f0c8c0f5bd34a1e27f4ba6d0f9c2f5b");
        assert_eq!(data["writeTargets"][0]["writeRevision"], "sha256:8f2a…");
    }

    #[test]
    fn the_documented_error_is_a_typed_rejection() {
        let payload = payload_of(&fixture(2)).expect("a desktop payload");

        let Some(WidgetPayload::Response(response)) = parse_widget_payload(&payload) else {
            panic!("expected a response envelope");
        };
        assert_eq!(response.request_id.as_deref(), Some("desktop-49"));
        assert_eq!(
            response.outcome,
            Err(DesktopError {
                code: "revision_conflict".into(),
                message: "Config changed since expectedRevision was read".into(),
            })
        );
    }

    #[test]
    fn the_documented_event_is_classified_as_an_event() {
        let payload = payload_of(&fixture(3)).expect("a desktop payload");

        let Some(WidgetPayload::Event(event)) = parse_widget_payload(&payload) else {
            panic!("expected an event envelope");
        };
        assert_eq!(event.seq, 1);
        assert_eq!(event.event, "config_changed");
        assert_eq!(event.data["source"], "desktop");
        assert_eq!(event.data["store"], "profiles");
    }

    #[test]
    fn the_documented_capabilities_are_read_from_the_answer_data() {
        let payload = payload_of(&fixture(0)).expect("a desktop payload");
        let Some(WidgetPayload::Response(response)) = parse_widget_payload(&payload) else {
            panic!("expected a response envelope");
        };
        let data = response.outcome.expect("an ok answer");

        let capabilities = DesktopClient::new(FakeChannel::connected()).connect();
        let DesktopOutcome::Data { data: answer, .. } = capabilities else {
            panic!("expected the capabilities answer, got {capabilities:?}");
        };
        assert_eq!(answer, data);
        assert_eq!(
            answer["methods"].as_array().map(Vec::len),
            Some(14),
            "the spec advertises fourteen methods"
        );
    }

    #[test]
    fn another_widget_key_is_not_the_desktop_channel() {
        let frame = json!({
            "type": "extension_ui_request",
            "id": "ui-1",
            "method": "setWidget",
            "widgetKey": "gentle-agents",
            "widgetLines": [json!({"anything": true}).to_string()],
        });

        assert_eq!(payload_of(&frame.to_string()), None);
    }

    #[test]
    fn a_cleared_widget_has_no_payload() {
        assert_eq!(payload_of(&fixture(4)), None);
    }

    #[test]
    fn a_payload_that_is_not_our_schema_is_not_an_answer() {
        let foreign = json!({"schema": "some-other-extension/v1", "kind": "response"});

        assert_eq!(parse_widget_payload(&foreign.to_string()), None);
    }

    #[test]
    fn an_unparsable_or_incomplete_payload_is_not_an_answer() {
        for payload in [
            "{not valid json",
            "",
            "{}",
            r#"{"schema":"gentle-shell.desktop-rpc/v1"}"#,
            r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response"}"#,
            r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-1","ok":false}"#,
            r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"event","event":"config_changed"}"#,
        ] {
            assert_eq!(parse_widget_payload(payload), None, "{payload}");
        }
    }
}

mod correlation {
    use super::*;

    #[test]
    fn an_answer_for_the_right_request_id_is_data() {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(ok_answer("desktop-2", json!({"sequence": 4})).to_string()),
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));
        client.connect();

        let outcome = client.ask("get_snapshot", json!({}));

        assert_eq!(
            outcome,
            DesktopOutcome::Data {
                request_id: "desktop-2".into(),
                data: json!({"sequence": 4}),
            }
        );
        assert_eq!(channel.sent().len(), 2, "capabilities, then the method");
    }

    #[test]
    fn an_answer_for_another_request_id_is_no_answer() {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(ok_answer("desktop-999", json!({"sequence": 4})).to_string()),
        ]);
        let mut client = DesktopClient::new(channel);
        client.connect();

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Uncorrelated)
        );
    }

    #[test]
    fn an_error_without_a_request_id_is_no_answer() {
        let mut answer = value(DOC_ERROR);
        answer["requestId"] = Value::Null;
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(answer.to_string()),
        ]);
        let mut client = DesktopClient::new(channel);
        client.connect();

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Uncorrelated)
        );
    }

    #[test]
    fn an_event_while_waiting_is_not_the_answer() {
        let mut event = value(
            r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"event","seq":2,"event":"rate_limit","data":{}}"#,
        );
        event["requestId"] = json!("desktop-2");
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(event.to_string()),
        ]);
        let mut client = DesktopClient::new(channel);
        client.connect();

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Uncorrelated)
        );
    }

    #[test]
    fn the_documented_rejection_arrives_as_a_typed_error() {
        let mut answer = value(DOC_ERROR);
        answer["requestId"] = json!("desktop-2");
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(answer.to_string()),
        ]);
        let mut client = DesktopClient::new(channel);
        client.connect();

        assert_eq!(
            client.ask("set_config", json!({"store": "profiles"})),
            DesktopOutcome::Rejected {
                request_id: Some("desktop-2".into()),
                error: DesktopError {
                    code: "revision_conflict".into(),
                    message: "Config changed since expectedRevision was read".into(),
                },
            }
        );
    }
}

mod capabilities_first {
    use super::*;

    #[test]
    fn asking_before_capabilities_answers_not_connected_and_sends_nothing() {
        let channel = Arc::new(FakeChannel::default());
        let mut client = DesktopClient::new(Arc::clone(&channel));

        let outcome = client.ask("get_snapshot", json!({}));

        assert_eq!(outcome, DesktopOutcome::NoAnswer(NoAnswer::NotConnected));
        assert_eq!(channel.sent(), Vec::new());
    }

    #[test]
    fn a_method_outside_the_reported_contract_is_unsupported_and_never_sent() {
        let channel = FakeChannel::connected();
        let mut client = DesktopClient::new(Arc::clone(&channel));
        client.connect();

        let outcome = client.ask("open_canvas", json!({}));

        assert_eq!(
            outcome,
            DesktopOutcome::Unsupported {
                method: "open_canvas".into()
            }
        );
        assert_eq!(channel.sent().len(), 1, "only the capabilities call");
        assert!(!client.supports("open_canvas"));
    }

    #[test]
    fn every_advertised_method_of_the_spec_is_supported() {
        let mut client = DesktopClient::new(FakeChannel::connected());
        client.connect();

        let capabilities = client.capabilities().expect("a cached contract");
        assert_eq!(capabilities.methods.len(), 14);
        assert_eq!(capabilities.events.len(), 7);
        for method in [
            "get_snapshot",
            "get_config",
            "set_config",
            "get_permission_mode",
            "set_permission_mode",
            "get_subscription_usage",
            "refresh_subscription_usage",
            "get_design_canvas",
            "list_jobs",
            "cancel_job",
            "apply_profile",
            "list_subagents",
            "cancel_subagent",
        ] {
            assert!(client.supports(method), "{method}");
        }
        assert!(
            !client.supports("config_changed"),
            "an event is not a method"
        );
    }

    #[test]
    fn asking_again_for_capabilities_is_always_allowed() {
        let mut client = DesktopClient::new(FakeChannel::connected());
        client.connect();

        assert!(matches!(
            client.ask("capabilities", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::TimedOut)
        ));
    }

    #[test]
    fn a_failed_capabilities_call_leaves_the_client_unconnected() {
        let channel = FakeChannel::answering(vec![ChannelReply::Timeout]);
        let mut client = DesktopClient::new(Arc::clone(&channel));

        assert_eq!(
            client.connect(),
            DesktopOutcome::NoAnswer(NoAnswer::TimedOut)
        );
        assert_eq!(client.capabilities(), None);
        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::NotConnected)
        );
        assert_eq!(
            channel.sent().len(),
            1,
            "nothing followed the failed connect"
        );
    }

    #[test]
    fn the_client_numbers_its_requests_and_never_reuses_one() {
        let replies = vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(ok_answer("desktop-2", json!({})).to_string()),
            ChannelReply::Widget(ok_answer("desktop-3", json!({})).to_string()),
        ];
        let channel = FakeChannel::answering(replies);
        let mut client = DesktopClient::new(Arc::clone(&channel));
        client.connect();
        client.ask("get_snapshot", json!({}));
        client.ask("get_snapshot", json!({}));

        let ids: Vec<String> = channel
            .sent()
            .iter()
            .map(|r| r.request_id().to_owned())
            .collect();
        assert_eq!(ids, ["desktop-1", "desktop-2", "desktop-3"]);
    }
}

mod degradation {
    use super::*;

    fn client_with(reply: ChannelReply) -> (Arc<FakeChannel>, DesktopClient<Arc<FakeChannel>>) {
        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            reply,
        ]);
        let mut client = DesktopClient::new(Arc::clone(&channel));
        client.connect();
        (channel, client)
    }

    #[test]
    fn a_child_that_does_not_answer_times_out() {
        let (_, mut client) = client_with(ChannelReply::Timeout);

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::TimedOut)
        );
    }

    #[test]
    fn a_host_that_is_not_interactive_is_inactive() {
        let (_, mut client) = client_with(ChannelReply::Inactive);

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Inactive)
        );
    }

    #[test]
    fn a_closed_child_is_closed() {
        let (_, mut client) = client_with(ChannelReply::Closed);

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Closed)
        );
    }

    #[test]
    fn a_widget_that_does_not_parse_is_no_answer() {
        let (_, mut client) = client_with(ChannelReply::Widget("{not valid json".into()));

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::NoAnswer(NoAnswer::Malformed)
        );
    }

    #[test]
    fn a_request_that_breaks_a_limit_never_reaches_the_channel() {
        let (channel, mut client) = client_with(ChannelReply::Timeout);
        let before = channel.sent().len();

        let method = "a".repeat(MAX_METHOD_CHARS + 1);
        assert_eq!(
            client.ask(&method, json!({})),
            DesktopOutcome::Invalid(RequestError::MethodTooLong {
                chars: MAX_METHOD_CHARS + 1
            })
        );
        assert_eq!(channel.sent().len(), before);
    }

    #[test]
    fn a_frame_built_the_way_pi_builds_it_is_the_answer_the_client_reads() {
        // The same frame shape the fixture uses, built here so the test proves
        // the client reads `setWidget` + `widgetLines[0]`, not a sibling field.
        let answer = ok_answer("desktop-2", json!({"sequence": 4}));
        let frame = frame_with(&answer);
        let payload = widget_payload(&frame).expect("the desktop payload");

        let channel = FakeChannel::answering(vec![
            ChannelReply::Widget(capabilities_answer().to_string()),
            ChannelReply::Widget(payload.to_owned()),
        ]);
        let mut client = DesktopClient::new(channel);
        client.connect();

        assert_eq!(
            client.ask("get_snapshot", json!({})),
            DesktopOutcome::Data {
                request_id: "desktop-2".into(),
                data: json!({"sequence": 4}),
            }
        );
    }
}
