//! Reading which extensions are behind over one chat's own child (packages-status
//! T1, S1, S2, S4, S5).
//!
//! Nothing here starts a child: the channel is a fake that answers what the test
//! says and records every request it was asked, so the wire itself - one
//! `get_update_status` ask, its params, and the typed answer it became - is what
//! these tests assert.

use std::sync::{Arc, Mutex};

use gentle_core::rpc::{ChannelReply, DesktopChannel, DesktopClient, DesktopRequest, SCHEMA};
use serde_json::{json, Value};

use super::runtime;
use crate::chat_runtime::{packages_status, PackagesRead, MAX_ITEMS};

/// What a scripted child answers the one method this module asks.
#[derive(Debug, Clone)]
enum Answer {
    /// An `ok: true` answer carrying this `data`.
    Data(Value),
    /// An `ok: false` answer with this error code.
    Refused(&'static str),
    /// No frame at all: the answer never comes.
    Silence,
}

/// A channel that records every request and answers `capabilities` with
/// `methods`, and every later request with `answer`.
struct FakeChannel {
    methods: &'static [&'static str],
    answer: Answer,
    asked: Mutex<Vec<(String, Value)>>,
}

impl FakeChannel {
    fn new(methods: &'static [&'static str], answer: Answer) -> Arc<Self> {
        Arc::new(Self {
            methods,
            answer,
            asked: Mutex::new(Vec::new()),
        })
    }

    /// The methods this channel was asked, in order, with their params.
    fn asked(&self) -> Vec<(String, Value)> {
        self.asked.lock().unwrap().clone()
    }
}

impl DesktopChannel for FakeChannel {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply {
        self.asked
            .lock()
            .unwrap()
            .push((request.method().to_owned(), request.params().clone()));
        let id = request.request_id();
        if request.method() == "capabilities" {
            return reply_ok(id, json!({"methods": self.methods, "events": []}));
        }
        match &self.answer {
            Answer::Data(data) => reply_ok(id, data.clone()),
            Answer::Refused(code) => reply_refused(id, code),
            Answer::Silence => ChannelReply::Timeout,
        }
    }
}

/// One `ok: true` answer envelope, correlated with the request it answers.
fn reply_ok(request_id: &str, data: Value) -> ChannelReply {
    ChannelReply::Widget(
        json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": request_id,
            "ok": true,
            "data": data,
        })
        .to_string(),
    )
}

/// One `ok: false` answer envelope, correlated with the request it answers.
fn reply_refused(request_id: &str, code: &str) -> ChannelReply {
    ChannelReply::Widget(
        json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": request_id,
            "ok": false,
            "error": {"code": code, "message": "the owner words this itself"},
        })
        .to_string(),
    )
}

/// A client of `channel` whose `capabilities` was already resolved, exactly as
/// the chat's one client is once its child answered.
fn connected(channel: &Arc<FakeChannel>) -> DesktopClient<Arc<FakeChannel>> {
    let mut client = DesktopClient::new(Arc::clone(channel));
    client.connect();
    client
}

/// A child that advertises `get_update_status` and nothing else.
const ADVERTISED: &[&str] = &["capabilities", "get_update_status"];

#[test]
fn a_child_that_does_not_advertise_the_method_is_unasked_and_asked_nothing() {
    // An older shell (S5): its capabilities name other methods, so the question
    // is never sent.
    let channel = FakeChannel::new(&["capabilities", "get_config"], Answer::Data(json!({})));

    let read = packages_status(&mut connected(&channel));

    assert_eq!(read, PackagesRead::Unasked);
    assert_eq!(
        channel.asked(),
        vec![("capabilities".to_owned(), json!({}))],
        "only the contract was asked; get_update_status never left"
    );
}

#[test]
fn a_client_that_never_resolved_its_contract_is_unasked_and_asks_nothing() {
    let channel = FakeChannel::new(ADVERTISED, Answer::Data(json!({})));

    let read = packages_status(&mut DesktopClient::new(Arc::clone(&channel)));

    assert_eq!(read, PackagesRead::Unasked);
    assert!(
        channel.asked().is_empty(),
        "a client with no contract knows no method exists, so it writes nothing"
    );
}

#[test]
fn a_child_that_does_not_answer_in_time_is_the_deadline() {
    let channel = FakeChannel::new(ADVERTISED, Answer::Silence);

    assert_eq!(
        packages_status(&mut connected(&channel)),
        PackagesRead::Deadline,
        "no frame and no child are one bounded failure, never a state"
    );
}

#[test]
fn a_refused_read_is_the_owners_own_code() {
    let channel = FakeChannel::new(ADVERTISED, Answer::Refused("not_found"));

    assert_eq!(
        packages_status(&mut connected(&channel)),
        PackagesRead::Refused("not_found".to_owned()),
        "the refusal is the child's code, never a sentence this app wrote"
    );
}

/// One `get_update_status` answer whose packages slice is `packages`.
fn listing(packages: Value) -> Value {
    json!({"packages": packages})
}

#[test]
fn only_update_available_is_behind_and_every_other_state_is_not() {
    let channel = FakeChannel::new(
        ADVERTISED,
        Answer::Data(listing(json!({
            "available": true,
            "truncated": false,
            "items": [
                {"name": "gentle-pi", "current": "4.9.0", "state": "update_available"},
                {"name": "usage-sources", "current": "0.2.0", "state": "no_update_reported"},
                {"name": "skipped-one", "state": "skipped"},
                {"name": "unknown-one", "state": "unknown"},
                {"name": "a-state-from-the-future", "state": "reinstalling"}
            ]
        }))),
    );

    let PackagesRead::Read(status) = packages_status(&mut connected(&channel)) else {
        panic!("the child answered its listing");
    };

    assert!(status.available);
    assert!(
        !status.unknown(),
        "the listing was read, so it is not unknown"
    );
    assert_eq!(
        status
            .behind()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["gentle-pi"],
        "`update_available` is the ONLY state that reads as behind (S1)"
    );
    assert_eq!(
        status.items.len(),
        5,
        "every named extension is carried, and none is turned into a guess"
    );
}

#[test]
fn a_listing_the_child_could_not_produce_is_unknown_and_never_up_to_date() {
    // pi's own mark for a packages slice it could not read.
    for answer in [
        listing(json!({"available": false, "items": [], "truncated": false})),
        // An answer that carries no packages slice at all is the same state: a
        // listing this app did not get.
        json!({"something": "else"}),
        // An answer that contradicts itself - no listing, yet extensions named -
        // is not knowledge either, so nothing is drawn as behind.
        listing(json!({
            "available": false,
            "items": [{"name": "gentle-pi", "current": "4.9.0", "state": "update_available"}]
        })),
    ] {
        let channel = FakeChannel::new(ADVERTISED, Answer::Data(answer.clone()));

        let PackagesRead::Read(status) = packages_status(&mut connected(&channel)) else {
            panic!("the child answered");
        };

        assert!(status.unknown(), "{answer}");
        assert!(!status.available, "{answer}");
        assert_eq!(status.behind().count(), 0, "{answer}");
    }
}

#[test]
fn the_contracts_own_hundred_item_bound_is_kept_and_says_it_cut() {
    let items = (0..=MAX_ITEMS)
        .map(|at| json!({"name": format!("package-{at}"), "state": "no_update_reported"}))
        .collect::<Vec<_>>();
    let channel = FakeChannel::new(
        ADVERTISED,
        Answer::Data(listing(json!({
            "available": true,
            "truncated": false,
            "items": items
        }))),
    );

    let PackagesRead::Read(status) = packages_status(&mut connected(&channel)) else {
        panic!("the child answered its listing");
    };

    assert_eq!(status.items.len(), MAX_ITEMS, "the contract's own bound");
    assert!(
        status.truncated,
        "a listing this app cut says so, even when the child said it did not"
    );

    // And the child's own mark travels as it is.
    let channel = FakeChannel::new(
        ADVERTISED,
        Answer::Data(listing(
            json!({"available": true, "truncated": true, "items": []}),
        )),
    );
    let PackagesRead::Read(status) = packages_status(&mut connected(&channel)) else {
        panic!("the child answered its listing");
    };
    assert!(status.truncated);
}

#[test]
fn a_chat_with_no_child_is_unasked_and_touches_nothing() {
    let (runtime, spawner) = runtime();
    let chat = gentle_app::ChatId::of_session("/sessions/never-opened.jsonl");

    assert_eq!(runtime.read_packages(&chat), PackagesRead::Unasked);
    assert!(
        spawner.started().is_empty(),
        "a read from Settings never starts a child"
    );
}
#[test]
fn the_listing_is_one_plain_read_of_get_update_status() {
    let channel = FakeChannel::new(
        ADVERTISED,
        Answer::Data(json!({
            "packages": {
                "available": true,
                "truncated": false,
                "checkedAt": 1_730_000_000_000.0,
                "command": "pi update --extension",
                "items": [
                    {"name": "gentle-pi", "kind": "npm", "scope": "user",
                     "current": "4.9.0", "latest": null, "pinned": false,
                     "state": "update_available"}
                ]
            }
        })),
    );

    let read = packages_status(&mut connected(&channel));

    assert_eq!(
        channel.asked(),
        vec![
            ("capabilities".to_owned(), json!({})),
            // ONE plain read: the checks are pi's, and `refresh` is never asked
            // for (the channel defines it as RUNNING them, longer than the
            // channel's own deadline).
            ("get_update_status".to_owned(), json!({"refresh": false})),
        ],
        "the contract, then one plain read and nothing else"
    );
    let PackagesRead::Read(status) = read else {
        panic!("the child answered its listing: {read:?}");
    };
    assert!(status.available);
    assert_eq!(status.checked_ms, Some(1_730_000_000_000.0));
    assert!(!status.truncated);
    assert_eq!(
        status
            .behind()
            .map(|item| (item.name.as_str(), item.current.as_deref()))
            .collect::<Vec<_>>(),
        vec![("gentle-pi", Some("4.9.0"))]
    );
}
