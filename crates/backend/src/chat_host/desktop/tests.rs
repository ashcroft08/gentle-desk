//! The desktop RPC channel over a real [`ChatHost`](super::super::ChatHost):
//! a fake child receives the prompt and pushes the widget frame, so the prompt
//! routing, the correlation and the bounded wait are proven with no process
//! behind them. The maintainer's real-child probe is T1 of the feature doc.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use gentle_core::rpc::helper_stop::{stop_helper, StopOutcome};
use gentle_core::rpc::{
    decode_line, ChannelReply, DesktopChannel, DesktopClient, DesktopOutcome, DesktopRequest,
    NoAnswer,
};
use serde_json::json;

use super::*;
use crate::chat_host::tests::fake::{FakeChild, FakeSpawner, WAIT};
use crate::chat_host::{Limits, ManualClock};
use crate::process::ProcessEvent;

/// The answer the channel spec documents for `get_config` (`desktop-46`).
const DOC_ANSWER: &str = concat!(
    r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-46","#,
    r#""ok":true,"data":{"store":"profiles","scope":"global","readOnly":false,"#,
    r#""status":"ok","revision":"sha256:1f0c8c0f5bd34a1e27f4ba6d0f9c2f5b"}}"#,
);

/// A widget the desktop channel does not own.
const HELPERS_WIDGET: &str = concat!(
    r#"{"type":"extension_ui_request","id":"ui-2","method":"setWidget","#,
    r#""widgetKey":"gentle-agents","widgetLines":["{}"]}"#,
);

fn chat() -> ChatKey {
    ChatKey::Saved(PathBuf::from("c:/sessions/desktop.jsonl"))
}

fn host_with(spawner: &Arc<FakeSpawner>) -> Arc<ChatHost> {
    Arc::new(ChatHost::with_parts(
        Limits::default(),
        spawner.clone(),
        Arc::new(ManualClock::new()),
    ))
}

/// One interactive host with the chat's child already up.
fn live(interactive: bool) -> (Arc<FakeSpawner>, Arc<ChatHost>) {
    let spawner = Arc::new(if interactive {
        FakeSpawner::new().with_interactive()
    } else {
        FakeSpawner::new()
    });
    let host = host_with(&spawner);
    host.open(chat()).expect("the child starts");
    (spawner, host)
}

fn get_config() -> DesktopRequest {
    DesktopRequest::new("desktop-46", "get_config", json!({"store": "profiles"}))
        .expect("the documented request")
}

/// One `setWidget` frame, as pi builds it, carrying `payload` as its one line.
fn frame(payload: &str) -> ProcessEvent {
    let frame = json!({
        "type": "extension_ui_request",
        "id": "ui-1",
        "method": "setWidget",
        "widgetKey": "gentle-desktop-rpc",
        "widgetLines": [payload],
    });
    ProcessEvent::Output(decode_line(&frame.to_string()))
}

fn output(line: &str) -> ProcessEvent {
    ProcessEvent::Output(decode_line(line))
}

/// Waits until the child has been written the prompt, so a frame pushed next
/// is never faster than the registration that must catch it.
fn wait_for_prompt(child: &FakeChild) {
    let deadline = Instant::now() + WAIT;
    while child.commands().is_empty() {
        assert!(Instant::now() < deadline, "the prompt was never written");
        thread::sleep(Duration::from_millis(1));
    }
}

/// Asks on its own thread and hands back the reply once the test says so.
fn ask_in_background(
    channel: HostDesktopChannel,
    request: DesktopRequest,
) -> thread::JoinHandle<ChannelReply> {
    thread::spawn(move || DesktopChannel::exchange(&channel, &request))
}

#[test]
fn a_request_that_names_its_own_deadline_is_bounded_by_it() {
    // packages-status T4, S7: pi's `refresh: true` is bounded at 15 s by the
    // shell, so the explicit check names its own bound instead of failing inside
    // the channel's 5 s default. The named deadline is the one that is used, on a
    // channel whose own default is untouched.
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());
    let channel = host.desktop_channel(chat());
    let deadline = Duration::from_millis(50);

    let asked = Instant::now();
    let reply = DesktopChannel::exchange_within(&channel, &get_config(), deadline);
    let bounded_for = asked.elapsed();

    assert_eq!(reply, ChannelReply::Timeout);
    assert!(
        bounded_for < Duration::from_secs(1),
        "the ask must be bounded by the deadline it NAMED, took {bounded_for:?}"
    );
    // The prompt was still written: the bound cuts the WAIT, never the request.
    wait_for_commands(&child, 1);
    assert_eq!(child.commands(), vec![get_config().command()]);
    assert!(
        lock(&host.desktop.waits).is_empty(),
        "a timed-out exchange must leave no waiter behind"
    );
}

#[test]
fn the_prompt_reaches_the_child_and_its_widget_answers_it() {
    let (spawner, host) = live(true);
    let request = get_config();
    let child = spawner.child(&chat());

    let asker = ask_in_background(
        host.desktop_channel(chat()).with_deadline(WAIT),
        request.clone(),
    );
    wait_for_prompt(&child);

    // An ordinary pi prompt, the documented `/gentle:desktop {json}` message.
    assert_eq!(child.commands(), vec![request.command()]);

    child.push(frame(DOC_ANSWER));

    assert_eq!(
        asker.join().expect("the asker ends"),
        ChannelReply::Widget(DOC_ANSWER.to_owned())
    );
    // Observing it did not steal it: the app's one stream still carries the
    // frame, and the reducer drops it as another extension's widget.
    let tagged = host
        .recv_timeout(WAIT)
        .expect("the frame still reaches the app");
    assert_eq!(tagged.key, chat());
    assert_eq!(tagged.event, frame(DOC_ANSWER));
}

#[test]
fn a_write_that_blocks_past_the_deadline_times_out_and_leaves_no_waiter() {
    let (blocked, gate) = FakeSpawner::with_blocked_writes(chat());
    let spawner = Arc::new(blocked.with_interactive());
    let host = host_with(&spawner);
    host.open(chat()).expect("the child starts");
    let deadline = Duration::from_millis(50);
    let channel = host.desktop_channel(chat()).with_deadline(deadline);

    let (replies, replied) = std::sync::mpsc::channel();
    let asker = thread::spawn(move || {
        let _ = replies.send(DesktopChannel::exchange(&channel, &get_config()));
    });
    // The child has stopped reading: the write is parked inside its pipe.
    gate.wait_until_parked(1);
    let asked = Instant::now();
    let reply = replied.recv_timeout(deadline * 10);
    let blocked_for = asked.elapsed();
    // Released before asserting, so a failing run leaves no parked thread.
    gate.release();

    // A child that ignores the command degrades to today's timeout instead of
    // hanging the caller past the exchange's own deadline.
    assert_eq!(
        reply.expect("the exchange must not block past its deadline"),
        ChannelReply::Timeout
    );
    assert!(
        blocked_for < deadline * 10,
        "a blocked write must be bounded by the deadline, took {blocked_for:?}"
    );
    assert!(
        lock(&host.desktop.waits).is_empty(),
        "a timed-out exchange must leave no waiter behind"
    );
    asker.join().expect("the asker ends");
}

#[test]
fn a_child_that_ignores_the_prompt_times_out_on_a_bounded_wait() {
    let (spawner, host) = live(true);
    let request = get_config();
    let channel = host
        .desktop_channel(chat())
        .with_deadline(Duration::from_millis(50));

    assert_eq!(
        DesktopChannel::exchange(&channel, &request),
        ChannelReply::Timeout
    );
    // The prompt was written, and the child simply never answered: today's
    // behaviour, with nothing broken and nothing visible. The write runs on a
    // short-lived thread of its own inside the exchange's deadline, so on a busy
    // machine the 50 ms can be over before that thread was scheduled: wait for
    // the write (bounded) instead of reading the child at that very instant.
    let child = spawner.child(&chat());
    wait_for_commands(&child, 1);
    assert_eq!(child.commands(), vec![request.command()]);
}

#[test]
fn a_host_without_the_interactive_marker_is_inactive_and_writes_nothing() {
    let (spawner, host) = live(false);

    assert!(!host.interactive());
    assert_eq!(
        DesktopChannel::exchange(&host.desktop_channel(chat()), &get_config()),
        ChannelReply::Inactive
    );
    assert!(spawner.child(&chat()).commands().is_empty());
}

#[test]
fn a_chat_with_no_live_child_is_closed() {
    let spawner = Arc::new(FakeSpawner::new().with_interactive());
    let host = host_with(&spawner);

    assert!(host.interactive());
    assert_eq!(
        DesktopChannel::exchange(&host.desktop_channel(chat()), &get_config()),
        ChannelReply::Closed
    );
}

#[test]
fn a_widget_for_another_request_id_is_not_the_answer() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());
    let mut answer = String::from(DOC_ANSWER);
    answer = answer.replace("desktop-46", "desktop-999");

    let asker = ask_in_background(
        host.desktop_channel(chat())
            .with_deadline(Duration::from_millis(100)),
        get_config(),
    );
    wait_for_prompt(&child);
    child.push(frame(&answer));

    assert_eq!(asker.join().expect("the asker ends"), ChannelReply::Timeout);
}

#[test]
fn another_extensions_widget_is_not_the_answer() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());

    let asker = ask_in_background(
        host.desktop_channel(chat()).with_deadline(WAIT),
        get_config(),
    );
    wait_for_prompt(&child);
    child.push(output(HELPERS_WIDGET));
    child.push(frame(DOC_ANSWER));

    assert_eq!(
        asker.join().expect("the asker ends"),
        ChannelReply::Widget(DOC_ANSWER.to_owned())
    );
}

#[test]
fn a_child_that_closes_wakes_the_waiter_instead_of_waiting_out_the_deadline() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());

    let asker = ask_in_background(
        host.desktop_channel(chat()).with_deadline(WAIT),
        get_config(),
    );
    wait_for_prompt(&child);

    let pushed = Instant::now();
    child.push(ProcessEvent::Closed);

    assert_eq!(asker.join().expect("the asker ends"), ChannelReply::Closed);
    assert!(
        pushed.elapsed() < WAIT,
        "a closed child must not wait out the deadline"
    );
}

#[test]
fn a_client_over_the_real_host_degrades_silently() {
    let (_, host) = live(false);
    let mut client = DesktopClient::new(host.desktop_channel(chat()));

    assert_eq!(
        client.connect(),
        DesktopOutcome::NoAnswer(NoAnswer::Inactive)
    );
}

#[test]
fn a_client_over_a_child_that_never_answers_times_out_silently() {
    let (_, host) = live(true);
    let mut client = DesktopClient::new(
        host.desktop_channel(chat())
            .with_deadline(Duration::from_millis(50)),
    );

    assert_eq!(
        client.connect(),
        DesktopOutcome::NoAnswer(NoAnswer::TimedOut)
    );
}

#[test]
fn a_client_reads_the_documented_answer_from_the_real_host() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());

    // The client asks `capabilities` first; this child answers with the
    // documented contract, from the fake child's own frame.
    let answer = concat!(
        r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-1","#,
        r#""ok":true,"data":{"schema":"gentle-shell.desktop-rpc/v1","command":"gentle:desktop","#,
        r#""widget":"gentle-desktop-rpc","methods":["capabilities","get_config"],"#,
        r#""events":["config_changed"]}}"#,
    );
    let mut client = DesktopClient::new(host.desktop_channel(chat()).with_deadline(WAIT));
    let asker = thread::spawn(move || client.connect());

    wait_for_prompt(&child);
    child.push(frame(answer));

    assert!(matches!(
        asker.join().expect("the asker ends"),
        DesktopOutcome::Data { request_id, .. } if request_id == "desktop-1"
    ));
}

// ---------------------------------------------------------------------------
// Stopping one delegated helper (helper-stop T3): the same channel, one method
// deeper. The child is the fake one above; nothing here starts a process.
// ---------------------------------------------------------------------------

/// The contract this child reports: the subagents slice advertised.
const CAPABILITIES_ANSWER: &str = concat!(
    r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-1","#,
    r#""ok":true,"data":{"schema":"gentle-shell.desktop-rpc/v1","command":"gentle:desktop","#,
    r#""widget":"gentle-desktop-rpc","methods":["capabilities","list_subagents","#,
    r#""cancel_subagent"],"events":["agents_update"]}}"#,
);

/// The answer the channel documents for a cancel it accepted.
const CANCEL_ANSWER: &str = concat!(
    r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-2","#,
    r#""ok":true,"data":{"agent":{"id":"h1","agent":"worker","status":"cancelled"}}}"#,
);

/// The typed refusal the channel documents for an id it cannot find.
const NOT_FOUND_ANSWER: &str = concat!(
    r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-2","#,
    r#""ok":false,"error":{"code":"not_found","message":"No subagent h1 in this session"}}"#,
);

/// The contract of a child that has no subagents slice.
const CAPABILITIES_WITHOUT_CANCEL: &str = concat!(
    r#"{"schema":"gentle-shell.desktop-rpc/v1","kind":"response","requestId":"desktop-1","#,
    r#""ok":true,"data":{"schema":"gentle-shell.desktop-rpc/v1","methods":["capabilities"],"#,
    r#""events":[]}}"#,
);

/// Waits until the child was written `count` commands, so a frame pushed next is
/// never faster than the registration that must catch it.
fn wait_for_commands(child: &FakeChild, count: usize) {
    let deadline = Instant::now() + WAIT;
    while child.commands().len() < count {
        assert!(
            Instant::now() < deadline,
            "the child was only written {} commands",
            child.commands().len()
        );
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn a_helper_stop_writes_the_documented_cancel_subagent_request() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());
    let mut client = DesktopClient::new(host.desktop_channel(chat()).with_deadline(WAIT));
    let stopper = thread::spawn(move || stop_helper(&mut client, "h1"));

    // The client's first call of the session: capabilities.
    wait_for_commands(&child, 1);
    child.push(frame(CAPABILITIES_ANSWER));
    // Then the cancel itself, with the exact body the contract accepts.
    wait_for_commands(&child, 2);
    let asked = child.commands();
    let message = asked[1].prompt_text().expect("a prompt");
    let envelope = message
        .strip_prefix("/gentle:desktop ")
        .expect("the desktop command prefix");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(envelope).expect("one JSON envelope"),
        json!({
            "schema": "gentle-shell.desktop-rpc/v1",
            "requestId": "desktop-2",
            "method": "cancel_subagent",
            "params": { "id": "h1" },
        }),
        "the request body is exactly the id, and never another key"
    );

    child.push(frame(CANCEL_ANSWER));

    assert_eq!(
        stopper.join().expect("the stopper ends"),
        StopOutcome::Stopped
    );
}

#[test]
fn a_not_found_answer_is_a_typed_refusal_and_not_a_stop() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());
    let mut client = DesktopClient::new(host.desktop_channel(chat()).with_deadline(WAIT));
    let stopper = thread::spawn(move || stop_helper(&mut client, "h1"));

    wait_for_commands(&child, 1);
    child.push(frame(CAPABILITIES_ANSWER));
    wait_for_commands(&child, 2);
    child.push(frame(NOT_FOUND_ANSWER));

    match stopper.join().expect("the stopper ends") {
        StopOutcome::Rejected(error) => assert_eq!(error.code, "not_found"),
        other => panic!("expected the typed refusal, got {other:?}"),
    }
}

#[test]
fn a_child_without_the_method_is_never_written_the_cancel() {
    let (spawner, host) = live(true);
    let child = spawner.child(&chat());
    let mut client = DesktopClient::new(host.desktop_channel(chat()).with_deadline(WAIT));
    let stopper = thread::spawn(move || stop_helper(&mut client, "h1"));

    wait_for_commands(&child, 1);
    child.push(frame(CAPABILITIES_WITHOUT_CANCEL));

    assert_eq!(
        stopper.join().expect("the stopper ends"),
        StopOutcome::Unsupported
    );
    let asked = child.commands();
    assert_eq!(
        asked.len(),
        1,
        "nothing past the capabilities ask was written to the child"
    );
}

#[test]
fn a_stop_for_a_chat_with_no_child_degrades_silently() {
    // Not opened: the chat has no child, so no request is written and the caller
    // gets today's behaviour (no stop possible), never an error the user sees.
    let spawner = Arc::new(FakeSpawner::new().with_interactive());
    let host = host_with(&spawner);
    let mut client = DesktopClient::new(host.desktop_channel(chat()).with_deadline(WAIT));

    assert_eq!(
        stop_helper(&mut client, "h1"),
        StopOutcome::NoAnswer(NoAnswer::NotConnected)
    );
}
