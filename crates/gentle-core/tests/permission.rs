//! The session permission mode over the `/gentle:desktop` channel
//! (composer-dock T3a): `get_permission_mode` and `set_permission_mode`, typed.
//!
//! The answer shapes are the ones a real `gentle-shell --mode rpc` child
//! returned when probed (composer-dock L7, L8).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use gentle_core::rpc::permission::{
    get_permission_params, read_permission, set_permission_params, write_permission,
    PermissionOutcome, PermissionState, SessionMode, GET_PERMISSION_MODE, SET_PERMISSION_MODE,
};
use gentle_core::rpc::{
    AnswerError, ChannelReply, DesktopChannel, DesktopClient, DesktopError, DesktopRequest,
    NoAnswer, SCHEMA,
};
use serde_json::{json, Value};

fn response(request_id: &str, data: Value) -> String {
    json!({"schema": SCHEMA, "kind": "response", "requestId": request_id, "ok": true, "data": data})
        .to_string()
}

fn refusal(request_id: &str, code: &str, message: &str) -> String {
    json!({
        "schema": SCHEMA, "kind": "response", "requestId": request_id, "ok": false,
        "error": {"code": code, "message": message},
    })
    .to_string()
}

fn capabilities() -> ChannelReply {
    ChannelReply::Widget(response(
        "desktop-1",
        json!({
            "schema": SCHEMA,
            "methods": ["capabilities", "get_permission_mode", "set_permission_mode"],
            "events": ["permission_mode_changed"],
        }),
    ))
}

fn capabilities_without_permission() -> ChannelReply {
    ChannelReply::Widget(response(
        "desktop-1",
        json!({"schema": SCHEMA, "methods": ["capabilities", "get_snapshot"]}),
    ))
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

fn client(channel: &Arc<FakeChannel>) -> DesktopClient<Arc<FakeChannel>> {
    DesktopClient::new(Arc::clone(channel))
}

#[test]
fn the_four_wire_modes_parse_and_print_back_exactly() {
    for wire in ["normal", "auto", "yolo", "super-yolo"] {
        let mode = SessionMode::parse(wire).expect("a mode the channel names");
        assert_eq!(mode.as_str(), wire);
    }
    assert_eq!(SessionMode::parse("semi-auto"), None);
    assert_eq!(SessionMode::parse("SUPER-YOLO"), None);
}

#[test]
fn only_yolo_and_super_yolo_need_a_confirmation() {
    assert!(!SessionMode::Normal.needs_confirmation());
    assert!(!SessionMode::Auto.needs_confirmation());
    assert!(SessionMode::Yolo.needs_confirmation());
    assert!(SessionMode::SuperYolo.needs_confirmation());
}

#[test]
fn the_get_request_carries_no_params_and_the_set_request_only_the_mode() {
    assert_eq!(get_permission_params(), json!({}));
    // `persist` is never sent: the UI changes this session, never the default.
    assert_eq!(
        set_permission_params(SessionMode::SuperYolo),
        json!({"mode": "super-yolo"})
    );
}

#[test]
fn a_read_answers_the_mode_and_the_default_as_the_channel_named_them() {
    let channel = FakeChannel::answering(vec![
        capabilities(),
        ChannelReply::Widget(response(
            "desktop-2",
            json!({"mode": "yolo", "default_mode": "super-yolo"}),
        )),
    ]);
    let outcome = read_permission(&mut client(&channel));
    assert_eq!(
        outcome,
        PermissionOutcome::Answered(PermissionState {
            mode: "yolo".to_owned(),
            default_mode: "super-yolo".to_owned(),
        })
    );
    let sent = channel.sent();
    assert_eq!(sent[1].method(), GET_PERMISSION_MODE);
    assert_eq!(sent[1].params(), &json!({}));
}

#[test]
fn a_value_the_mapping_does_not_know_still_travels_raw() {
    let channel = FakeChannel::answering(vec![
        capabilities(),
        ChannelReply::Widget(response(
            "desktop-2",
            json!({"mode": "semi-auto", "default_mode": "normal"}),
        )),
    ]);
    match read_permission(&mut client(&channel)) {
        PermissionOutcome::Answered(state) => assert_eq!(state.mode, "semi-auto"),
        other => panic!("expected an answer, got {other:?}"),
    }
}

#[test]
fn a_write_sends_the_mode_alone_and_answers_the_state_re_read_after_it() {
    let channel = FakeChannel::answering(vec![
        capabilities(),
        ChannelReply::Widget(response(
            "desktop-2",
            json!({"mode": "normal", "default_mode": "super-yolo", "persisted": false, "changed": true}),
        )),
    ]);
    let outcome = write_permission(&mut client(&channel), SessionMode::Normal);
    assert_eq!(
        outcome,
        PermissionOutcome::Answered(PermissionState {
            mode: "normal".to_owned(),
            default_mode: "super-yolo".to_owned(),
        })
    );
    let sent = channel.sent();
    assert_eq!(sent[1].method(), SET_PERMISSION_MODE);
    assert_eq!(sent[1].params(), &json!({"mode": "normal"}));
}

#[test]
fn a_refusal_keeps_its_code_so_the_ui_can_say_why() {
    let channel = FakeChannel::answering(vec![
        capabilities(),
        ChannelReply::Widget(refusal("desktop-2", "permission_denied", "Not allowed")),
    ]);
    assert_eq!(
        write_permission(&mut client(&channel), SessionMode::Yolo),
        PermissionOutcome::Rejected(DesktopError {
            code: "permission_denied".to_owned(),
            message: "Not allowed".to_owned(),
        })
    );
}

#[test]
fn a_channel_without_the_methods_is_unsupported_and_writes_nothing_more() {
    let channel = FakeChannel::answering(vec![capabilities_without_permission()]);
    assert_eq!(
        read_permission(&mut client(&channel)),
        PermissionOutcome::Unsupported
    );
    assert_eq!(
        channel.sent().len(),
        1,
        "only `capabilities` left the client"
    );
}

#[test]
fn no_answer_is_told_apart_from_a_refusal() {
    let channel = FakeChannel::answering(vec![capabilities(), ChannelReply::Timeout]);
    assert_eq!(
        read_permission(&mut client(&channel)),
        PermissionOutcome::NoAnswer(NoAnswer::TimedOut)
    );
}

#[test]
fn an_ok_answer_without_the_mode_is_an_unexpected_answer_never_a_panic() {
    let channel = FakeChannel::answering(vec![
        capabilities(),
        ChannelReply::Widget(response("desktop-2", json!({"default_mode": "normal"}))),
    ]);
    assert_eq!(
        read_permission(&mut client(&channel)),
        PermissionOutcome::UnexpectedAnswer(AnswerError::MissingField { field: "mode" })
    );
}
