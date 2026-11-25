//! Reading which extensions are behind over one chat's own child (packages-status
//! T1, S1, S2, S4, S5).
//!
//! The wire itself is asserted here against a channel that answers what the test
//! says and records every request it was asked; the gate and the retry rule live
//! one layer up (`commands.rs`) and are proved there. The two scripted-child tests
//! at the end are the exception: they open a REAL fake child through the host, so
//! the key a FRESH `new-N` chat is asked over (T7) is the runtime's own.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_app::{ChatCommand, ChatId};
use gentle_core::rpc::{ChannelReply, DesktopChannel, DesktopClient, DesktopRequest, SCHEMA};
use serde_json::{json, Value};

use super::fake::{Reply, Script};
use super::{open, runtime};
use crate::chat_runtime::{
    check_packages, packages_status, Outcome, PackagesRead, MAX_ITEMS, REFRESH_DEADLINE,
};

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

/// One request the fake was handed: the method, its params, and the deadline
/// the caller named for THAT ask (`None` for the channel's own default).
#[derive(Debug, Clone, PartialEq)]
struct Asked {
    method: String,
    params: Value,
    within: Option<Duration>,
}

/// A channel that records every request and answers `capabilities` with
/// `methods`, and every later request with `answer`.
struct FakeChannel {
    methods: &'static [&'static str],
    answer: Answer,
    asked: Mutex<Vec<Asked>>,
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
        self.recorded()
            .into_iter()
            .map(|asked| (asked.method, asked.params))
            .collect()
    }

    /// Every request with the deadline the caller named for it, so a test can
    /// tell an ask that took its own deadline from one on the channel's default.
    fn deadlines(&self) -> Vec<(String, Option<Duration>)> {
        self.recorded()
            .into_iter()
            .map(|asked| (asked.method, asked.within))
            .collect()
    }

    fn recorded(&self) -> Vec<Asked> {
        self.asked.lock().unwrap().clone()
    }

    /// Records `request` and answers it, the one behaviour both the plain
    /// exchange and the one with its own deadline share.
    fn answer(&self, request: &DesktopRequest, within: Option<Duration>) -> ChannelReply {
        self.asked.lock().unwrap().push(Asked {
            method: request.method().to_owned(),
            params: request.params().clone(),
            within,
        });
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

impl DesktopChannel for FakeChannel {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply {
        self.answer(request, None)
    }

    /// The seam an ask with its own deadline takes: recorded, so the test can
    /// assert which bound the caller named.
    fn exchange_within(&self, request: &DesktopRequest, deadline: Duration) -> ChannelReply {
        self.answer(request, Some(deadline))
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

/// The dialog's plain read is the ONE read of an open (S2), and a client that has
/// not resolved its contract yet asks it first - the rule `ask_config`,
/// `helper_stop`, `permission` and `profile_apply` all follow. Without it the
/// FIRST read of any chat answered `Unasked` without writing a single request,
/// which the pane words as "this shell cannot tell": the maintainer's 0.9.0
/// report, on a machine where the child was alive and advertised the method.
#[test]
fn a_plain_read_asks_the_contract_first_when_it_has_none() {
    let channel = FakeChannel::new(
        ADVERTISED,
        Answer::Data(json!({
            "packages": {
                "available": true,
                "items": [{"name": "gentle-pi", "state": "no_update_reported"}],
            }
        })),
    );

    let read = packages_status(&mut DesktopClient::new(Arc::clone(&channel)));

    assert_eq!(
        channel.asked(),
        vec![
            ("capabilities".to_owned(), json!({})),
            ("get_update_status".to_owned(), json!({"refresh": false})),
        ],
        "the contract, then the one plain read: the read is asked, never declined"
    );
    let PackagesRead::Read(status) = read else {
        panic!("a child that advertises the method answers the read: {read:?}");
    };
    assert!(status.available);
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

/// The listing `states` names, one item each, as an `available` listing.
fn listing_of(states: &[&str]) -> Value {
    let items = states
        .iter()
        .enumerate()
        .map(|(at, state)| json!({"name": format!("package-{at}"), "state": state}))
        .collect::<Vec<_>>();
    listing(json!({"available": true, "truncated": false, "items": items}))
}

/// The one `get_update_status` answer of a channel scripted with `answer`, read
/// as a listing.
fn status_of(answer: Value) -> crate::chat_runtime::PackagesStatus {
    let channel = FakeChannel::new(ADVERTISED, Answer::Data(answer));
    let PackagesRead::Read(status) = packages_status(&mut connected(&channel)) else {
        panic!("the child answered its listing");
    };
    status
}

#[test]
fn a_listing_pi_reported_on_for_every_item_is_checked_and_up_to_date() {
    // S6: pi's own check ran for every extension and nothing is behind. THIS is
    // the only shape that may be drawn as "up to date".
    let status = status_of(listing_of(&["no_update_reported", "no_update_reported"]));

    assert!(status.available);
    assert!(!status.unknown(), "the listing was read");
    assert!(
        !status.unchecked(),
        "every checkable item carries pi's own answer, so the listing IS checked"
    );
    assert_eq!(status.behind().count(), 0);
    assert!(status.up_to_date());
}

#[test]
fn one_unknown_item_is_unchecked_and_never_up_to_date() {
    // S6's defect: with no refresh in that child pi reports every checkable
    // package as `unknown`, so `available: true` with nothing behind is NOT
    // "up to date" - nothing has been checked yet.
    let status = status_of(listing_of(&["no_update_reported", "unknown"]));

    assert!(status.available, "pi produced a listing");
    assert!(!status.unknown(), "and it was read, so nothing failed");
    assert!(
        status.unchecked(),
        "one item pi has not checked makes the listing not yet checked"
    );
    assert_eq!(status.behind().count(), 0);
    assert!(
        !status.up_to_date(),
        "a listing nobody checked is never 'up to date'"
    );
}

#[test]
fn a_state_from_the_future_is_unchecked_too() {
    // A state this build has not learned is not pi's answer that the item is
    // current either: it reads as `unknown`, so it is not checked.
    assert!(status_of(listing_of(&["reinstalling"])).unchecked());
}

#[test]
fn behind_and_unchecked_together_are_two_separate_facts() {
    // S6: a machine with one package behind AND one pi has not checked reads
    // both ways at once - the count is drawn AND the listing is not yet complete.
    let status = status_of(listing_of(&["update_available", "unknown"]));

    assert_eq!(
        status
            .behind()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["package-0"]
    );
    assert!(status.unchecked());
    assert!(!status.up_to_date());
}

#[test]
fn skipped_items_alone_are_not_checkable_so_nothing_is_unchecked() {
    // S6: a skipped item is not one pi has to check, so `skipped` alone leaves
    // the listing checked - and, with nothing behind it, up to date.
    let status = status_of(listing_of(&["skipped", "no_update_reported"]));

    assert!(!status.unchecked());
    assert!(status.up_to_date());
}

#[test]
fn a_listing_the_child_could_not_produce_is_unknown_and_never_unchecked() {
    // S6 keeps the two facts apart: a read that did not produce a listing is
    // `unknown` (the read could not run), never `unchecked` (nothing has run).
    let status = status_of(listing(json!(
        {"available": false, "items": [], "truncated": false}
    )));

    assert!(status.unknown());
    assert!(!status.unchecked());
    assert!(!status.up_to_date());
}

#[test]
fn an_empty_checked_listing_is_up_to_date() {
    // No extensions installed at all is pi's checked answer too, not a guess.
    let status = status_of(listing_of(&[]));

    assert!(status.available);
    assert!(!status.unchecked());
    assert!(status.up_to_date());
}

#[test]
fn the_explicit_check_asks_for_a_refresh_on_its_own_longer_deadline() {
    // S7: the ONE ask that runs pi's own check names `refresh: true` and takes a
    // deadline of its own - the shell bounds its refresh at 15 s, so the
    // channel's 5 s default is not it.
    assert!(
        REFRESH_DEADLINE > Duration::from_secs(5),
        "the refresh is bounded longer than a plain read's 5 s: {REFRESH_DEADLINE:?}"
    );
    assert!(
        REFRESH_DEADLINE >= Duration::from_secs(15),
        "the shell's own bound is 15 s, so the ask must tolerate it"
    );
    let channel = FakeChannel::new(ADVERTISED, Answer::Data(listing_of(&[])));

    let read = check_packages(&mut connected(&channel));

    assert_eq!(
        channel.asked(),
        vec![
            ("capabilities".to_owned(), json!({})),
            ("get_update_status".to_owned(), json!({"refresh": true})),
        ],
        "the contract, then one ask - and the ask is the refresh"
    );
    assert_eq!(
        channel.deadlines(),
        vec![
            ("capabilities".to_owned(), None),
            ("get_update_status".to_owned(), Some(REFRESH_DEADLINE)),
        ],
        "the check's own deadline is what the channel was told"
    );
    let PackagesRead::Read(status) = read else {
        panic!("the child answered its listing: {read:?}");
    };
    assert!(status.up_to_date());
}

#[test]
fn a_plain_read_still_uses_the_channels_default_deadline() {
    // S7 leaves the plain read exactly as it was: no deadline of its own.
    let channel = FakeChannel::new(ADVERTISED, Answer::Data(listing_of(&[])));

    let _ = packages_status(&mut connected(&channel));

    assert_eq!(
        channel.deadlines(),
        vec![
            ("capabilities".to_owned(), None),
            ("get_update_status".to_owned(), None),
        ],
        "a plain read asks with the channel's default bound"
    );
}

#[test]
fn the_check_resolves_the_contract_first_on_its_own_long_deadline() {
    // The check is what makes the answer exist, so it asks for the contract when
    // the chat's client never resolved one - and it asks for it on the LONG
    // deadline too (packages-status T7). A freshly spawned child answers NO
    // desktop request until it is ready (measured: ~14 s after spawn), so a
    // connect on the channel's 5 s default fails on a child that is a few seconds
    // from speaking and the check would never run.
    let channel = FakeChannel::new(ADVERTISED, Answer::Data(listing_of(&[])));

    let _ = check_packages(&mut DesktopClient::new(Arc::clone(&channel)));

    assert_eq!(
        channel.deadlines(),
        vec![
            ("capabilities".to_owned(), Some(REFRESH_DEADLINE)),
            ("get_update_status".to_owned(), Some(REFRESH_DEADLINE)),
        ],
        "a child still starting is connected on the check's own long bound"
    );
}

#[test]
fn a_check_the_child_never_answers_is_the_deadline() {
    let channel = FakeChannel::new(ADVERTISED, Answer::Silence);

    assert_eq!(
        check_packages(&mut connected(&channel)),
        PackagesRead::Deadline,
        "a refresh that never answers is bounded, never a hang"
    );
}

#[test]
fn a_refused_check_is_the_owners_own_code() {
    let channel = FakeChannel::new(ADVERTISED, Answer::Refused("unsupported_method"));

    assert_eq!(
        check_packages(&mut connected(&channel)),
        PackagesRead::Refused("unsupported_method".to_owned())
    );
}

#[test]
fn a_check_of_a_chat_with_no_child_is_unasked_and_touches_nothing() {
    let (runtime, spawner) = runtime();
    let chat = gentle_app::ChatId::of_session("/sessions/never-opened.jsonl");

    assert_eq!(runtime.check_packages(&chat), PackagesRead::Unasked);
    assert!(
        spawner.started().is_empty(),
        "the explicit check never starts a child either"
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

#[test]
fn the_check_time_reads_the_shells_iso_instant_and_a_plain_number() {
    // The shell's real shape: `checkedAt` is `isoTime`'s ISO instant, not a
    // number (lib/desktop-updates-service.ts:266, read on this machine). A
    // number keeps working, and anything else is nothing, never a guess.
    let cases: [(&str, Option<f64>); 5] = [
        // 2026-11-22T02:10:03Z = 2026-11-22T00:00:00Z (1_795_305_600_000) + 2h10m3s.
        ("2026-11-22T02:10:03.000Z", Some(1_795_313_403_000.0)),
        ("2026-11-22T02:10:03Z", Some(1_795_313_403_000.0)),
        ("2026-11-22T02:10:03.000+03:00", Some(1_795_302_603_000.0)),
        ("2026-11-22T02:10:03.000-08:00", Some(1_795_342_203_000.0)),
        ("not-a-time", None),
    ];
    for (checked_at, expected) in cases {
        let channel = FakeChannel::new(
            ADVERTISED,
            Answer::Data(json!({
                "packages": {
                    "available": true,
                    "items": [],
                    "checkedAt": checked_at,
                }
            })),
        );
        let read = packages_status(&mut connected(&channel));
        let PackagesRead::Read(status) = read else {
            panic!("the child answered its listing: {read:?}");
        };
        assert_eq!(status.checked_ms, expected, "checkedAt = {checked_at}");
    }
}

/// packages-status T7, S7: a FRESH chat is askable over its own child.
///
/// The runtime keys a `new-N` chat by the key it registered when the chat was
/// opened, so the check reaches the child that chat really started. This is what
/// the gate had stopped naming - it re-derived a session path from the id text,
/// and a chat with no session file yet has none - and the wire proves it: the
/// refresh leaves for THAT child, on the same key, with the check's own params.
#[test]
fn a_fresh_chats_check_reaches_its_own_child() {
    let script: Script = Arc::new(|method: &str, params: &Value| match method {
        "capabilities" => Reply::Data(json!({
            "schema": SCHEMA,
            "command": "gentle:desktop",
            "widget": "gentle-desktop-rpc",
            "methods": ["capabilities", "get_update_status"],
            "events": [],
        })),
        "get_update_status" => {
            assert_eq!(
                params,
                &json!({"refresh": true}),
                "the explicit check asks pi to run its own checks"
            );
            Reply::Data(listing(json!({
                "available": true,
                "truncated": false,
                "items": [{
                    "name": "gentle-pi",
                    "current": "4.9.0",
                    "state": "update_available",
                }],
            })))
        }
        _ => Reply::Refused {
            code: "unsupported_method",
        },
    });
    let (runtime, spawner, _events) = super::profile::answering(script);
    let chat = ChatId::fresh();

    let opened = runtime.execute(open(&chat));
    assert!(
        matches!(opened, Outcome::Opened { .. }),
        "the fake child started: {opened:?}"
    );

    let read = runtime.check_packages(&chat);

    let PackagesRead::Read(status) = read else {
        panic!("a fresh chat's own child answered its listing: {read:?}");
    };
    assert_eq!(
        status
            .behind()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["gentle-pi"],
        "the listing is the one that child produced"
    );
    let key = spawner
        .started()
        .first()
        .cloned()
        .expect("the child started");
    let written = spawner.written_to(&key);
    assert!(
        written
            .iter()
            .filter_map(|command| command.prompt_text())
            .any(|prompt| prompt.contains(r#""method":"get_update_status""#)
                && prompt.contains(r#""refresh":true"#)),
        "the ask left for that chat's own child, on its own key"
    );
}

/// packages-status T7: the chat in front is the one asked whenever the host has
/// it; when it does not, another open chat that HAS a child is asked instead of
/// nothing.
#[test]
fn the_chat_in_front_wins_and_another_live_chat_is_the_fallback() {
    let (runtime, spawner) = runtime();
    let other = ChatId::fresh();
    runtime.execute(open(&other));
    assert_eq!(spawner.started().len(), 1, "one child, for the open chat");

    assert_eq!(
        runtime.packages_chat(Some(&other)),
        Some(other.clone()),
        "the chat in front is asked, exactly as before"
    );
    assert_eq!(
        runtime.packages_chat(None),
        Some(other.clone()),
        "no chat in front still names an open chat that can answer"
    );
    let stranger = ChatId::of_text("new-424242").expect("the fresh form");
    assert_eq!(
        runtime.packages_chat(Some(&stranger)),
        Some(other),
        "a chat this run never opened cannot answer, so another open one is asked"
    );
}

/// packages-status T7: with no chat open at all, no chat is named - the zero-chat
/// rule the pane's honest sentence rests on.
#[test]
fn no_open_chat_names_no_chat() {
    let (runtime, spawner) = runtime();

    assert_eq!(runtime.packages_chat(None), None);
    assert_eq!(runtime.packages_chat(Some(&ChatId::fresh())), None);
    assert!(spawner.started().is_empty(), "naming a chat starts nothing");
}

/// packages-status T7: a FRESH chat the runtime opened is asked by the command
/// layer too - `ChatCommand` is what the app runs, and the id it holds is the
/// text the gate reads back.
#[test]
fn opening_a_fresh_chat_registers_the_id_the_gate_reads_back() {
    let (runtime, _spawner) = runtime();
    let chat = ChatId::fresh();
    runtime.execute(ChatCommand::OpenProcess { chat: chat.clone() });

    assert_eq!(
        ChatId::of_text(chat.as_str()),
        Some(chat.clone()),
        "the id text the UI holds is the fresh chat itself"
    );
    assert_eq!(runtime.packages_chat(Some(&chat)), Some(chat));
}
