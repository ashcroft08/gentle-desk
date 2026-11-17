//! Stopping one delegated helper over the desktop channel (helper-stop T3, S2/S3/S7).
//!
//! The call is the channel's own contract, not a new protocol: a client asks
//! `capabilities` once (its first call of a session), then writes
//! `cancel_subagent` with exactly `{ "id": "<helper id>" }` - the one params shape
//! `desktop-rpc.md` accepts (any other key, a non-string or an empty id is
//! `invalid_request`). One helper at a time, never the chat.
//!
//! Everything that is not a correlated `ok` answer degrades silently (S7): a host
//! that is not interactive, a child that never answers, a closed child, a payload
//! this contract does not model, a refused `not_found`, and a child whose
//! capabilities do not list the method. Nothing here reaches for a process: the
//! whole battery drives the fakeable [`DesktopChannel`] seam.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::helper_stop::{
    cancel_subagent_params, stop_helper, StopOutcome, CANCEL_SUBAGENT,
};
use gentle_core::rpc::{
    ChannelReply, DesktopChannel, DesktopClient, DesktopError, DesktopOutcome, DesktopRequest,
    NoAnswer, SCHEMA,
};
use serde_json::{json, Value};

/// The contract `capabilities` reports, with the subagents slice advertised.
fn capabilities() -> String {
    response(
        "desktop-1",
        json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": ["capabilities", "get_snapshot", "list_subagents", "cancel_subagent"],
            "events": ["agents_update"],
        }),
    )
}

/// A contract that does NOT advertise `cancel_subagent` (an older child).
fn capabilities_without_cancel() -> String {
    response(
        "desktop-1",
        json!({"schema": SCHEMA, "methods": ["capabilities", "get_snapshot"]}),
    )
}

/// One `ok: true` answer envelope for `request_id`.
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

/// The post-action state the channel documents for a queued cancel.
fn cancelled_agent() -> String {
    response(
        "desktop-2",
        json!({"agent": {"id": "h1", "agent": "worker", "status": "cancelled"}}),
    )
}

/// The typed `not_found` refusal the channel documents.
fn not_found() -> String {
    json!({
        "schema": SCHEMA,
        "kind": "response",
        "requestId": "desktop-2",
        "ok": false,
        "error": {"code": "not_found", "message": "No subagent h1 in this session"},
    })
    .to_string()
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

#[test]
fn the_stop_asks_capabilities_first_and_then_the_documented_cancel_request() {
    let channel = FakeChannel::connected(ChannelReply::Widget(cancelled_agent()));
    let mut client = DesktopClient::new(Arc::clone(&channel));

    let outcome = stop_helper(&mut client, "h1");

    assert_eq!(outcome, StopOutcome::Stopped);
    assert_eq!(
        channel.methods(),
        ["capabilities", CANCEL_SUBAGENT],
        "a client asks the contract once, then the method it was told about"
    );
    // The exact prompt the child reads: the command, then one compact envelope.
    assert_eq!(
        channel.sent()[1].prompt(),
        concat!(
            "/gentle:desktop {",
            "\"method\":\"cancel_subagent\",",
            "\"params\":{\"id\":\"h1\"},",
            "\"requestId\":\"desktop-2\",",
            "\"schema\":\"gentle-shell.desktop-rpc/v1\"}"
        ),
        "the request body is the channel's own, params included"
    );
}

#[test]
fn the_params_are_exactly_the_helper_id() {
    let params = cancel_subagent_params("h1");

    assert_eq!(params, json!({"id": "h1"}));
    let object = params.as_object().expect("an object");
    assert_eq!(
        object.keys().collect::<Vec<_>>(),
        ["id"],
        "only the id key is accepted; any other is invalid_request"
    );
    assert_eq!(object["id"].as_str(), Some("h1"));
}

#[test]
fn every_stop_of_one_client_keeps_one_growing_request_id_counter() {
    // S8: the ids of one chat never repeat, which is what lets a hub correlate
    // two answers that arrive close together.
    let channel = FakeChannel::answering(vec![
        ChannelReply::Widget(capabilities()),
        ChannelReply::Widget(cancelled_agent()),
        ChannelReply::Widget(response("desktop-3", json!({"agent": {"id": "h2"}}))),
    ]);
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(stop_helper(&mut client, "h1"), StopOutcome::Stopped);
    assert_eq!(stop_helper(&mut client, "h2"), StopOutcome::Stopped);

    assert_eq!(
        channel.methods(),
        ["capabilities", "cancel_subagent", "cancel_subagent"],
        "the capabilities answer is cached: it is asked once per child"
    );
    let ids: Vec<String> = channel
        .sent()
        .iter()
        .map(|request| request.request_id().to_owned())
        .collect();
    assert_eq!(ids, ["desktop-1", "desktop-2", "desktop-3"]);
}

#[test]
fn a_host_that_is_not_interactive_degrades_silently() {
    let channel = FakeChannel::answering(vec![ChannelReply::Inactive, ChannelReply::Inactive]);
    let mut client = DesktopClient::new(Arc::clone(&channel));

    // The channel itself says the host is not interactive, and a client with no
    // resolved capabilities never guesses a method (it writes nothing at all
    // here). Either way the caller learns only that no stop happened.
    assert_eq!(
        client.connect(),
        DesktopOutcome::NoAnswer(NoAnswer::Inactive)
    );
    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::NotConnected)
    );
    assert_eq!(
        channel.methods(),
        ["capabilities", "capabilities"],
        "the stop retried the contract once and wrote nothing past it"
    );
}

#[test]
fn a_client_that_never_resolved_its_capabilities_never_guesses() {
    let channel = FakeChannel::answering(Vec::new());
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::NotConnected),
        "a client that never resolved its capabilities refuses to guess a method"
    );
    assert_eq!(channel.methods(), ["capabilities"]);
}

#[test]
fn a_child_that_never_answers_the_cancel_degrades_to_a_timeout() {
    let channel = FakeChannel::answering(vec![
        ChannelReply::Widget(capabilities()),
        ChannelReply::Timeout,
    ]);
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::TimedOut)
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
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::Closed)
    );
}

#[test]
fn a_payload_this_contract_does_not_model_is_no_answer() {
    let channel = FakeChannel::connected(ChannelReply::Widget("{\"not\":\"an envelope\"}".into()));
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::Malformed)
    );
}

#[test]
fn a_not_found_refusal_is_not_a_stop() {
    let channel = FakeChannel::connected(ChannelReply::Widget(not_found()));
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::Rejected(DesktopError {
            code: "not_found".to_owned(),
            message: "No subagent h1 in this session".to_owned(),
        })
    );
}

#[test]
fn a_child_whose_capabilities_lack_the_method_is_never_asked() {
    let channel = FakeChannel::answering(vec![ChannelReply::Widget(capabilities_without_cancel())]);
    let mut client = DesktopClient::new(Arc::clone(&channel));

    assert_eq!(stop_helper(&mut client, "h1"), StopOutcome::Unsupported);
    assert_eq!(
        channel.methods(),
        ["capabilities"],
        "the channel is never sent a method the child did not advertise"
    );
}
