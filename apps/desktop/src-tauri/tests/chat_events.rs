//! Windowless tests of the child-event pump: a chat's translated events arrive
//! on a plain channel, the pump folds each one through the reducer and pushes
//! the slice, and nothing here starts a child or a window.
//!
//! The source is the seam: `Converted` values fed by hand, which is exactly what
//! the translating thread produces from a real child. Every pushed slice is
//! recorded instead of emitted.

mod pump_seam;
use pump_seam::{assistant_message, open_stored, Pump};

use gentle_desk_lib::contract::{ChatActivityMark, ChatActivityView, MessageRole, MessageView};
use gentle_runtime::chat_runtime::Converted;

#[test]
fn an_assistant_event_is_pushed_as_a_message_of_the_foreground_chat() {
    let (holder, chat) = open_stored();
    let pump = Pump::start(&holder);

    for converted in assistant_message(&chat, "hello") {
        pump.send(converted);
    }
    let pushed = pump.finish();

    let last = pushed.last().expect("one push per event");
    assert_eq!(last.foreground_chat.as_deref(), Some(chat.as_str()));
    assert_eq!(
        last.messages,
        vec![MessageView {
            id: "msg-0".to_owned(),
            role: MessageRole::Assistant,
            text: "hello".to_owned(),
            // The turn has not ended yet: the pump pushed the deltas as they
            // arrived, so the message is still open.
            streaming: true,
            error: None,
            images: Vec::new(),
        }],
        "the next slice carries what the child said"
    );
}

#[test]
fn a_stderr_line_is_reflected_only_when_the_state_keeps_it() {
    let (holder, chat) = open_stored();
    let pump = Pump::start(&holder);

    pump.send(Converted::Stderr {
        chat: chat.clone(),
        line: "  cannot open the session  ".to_owned(),
    });
    // A blank line is not kept by the state's bounded ring, so it says nothing.
    pump.send(Converted::Stderr {
        chat: chat.clone(),
        line: "   ".to_owned(),
    });
    pump.send(Converted::Closed {
        chat: chat.clone(),
        expected: false,
    });
    let pushed = pump.finish();

    // The state exposes a kept line only as the reason of the death notice: it
    // is not a slice of its own, and nothing here invents one.
    let notice = &pushed.last().expect("the close is pushed").notices[0];
    assert_eq!(
        notice.text,
        "a.jsonl: the child that runs this chat exited (cannot open the session). \
         Restart to keep talking."
    );

    // A blank line never enters the ring: another chat's death has no reason.
    let (holder, chat) = open_stored();
    let pump = Pump::start(&holder);
    pump.send(Converted::Stderr {
        chat: chat.clone(),
        line: "   ".to_owned(),
    });
    pump.send(Converted::Closed {
        chat,
        expected: false,
    });
    let pushed = pump.finish();
    assert_eq!(
        pushed.last().expect("the close is pushed").notices[0].text,
        "a.jsonl: the child that runs this chat exited. Restart to keep talking."
    );
}

#[test]
fn a_closed_child_ends_its_chat_and_pushes_once() {
    let (holder, chat) = open_stored();
    let before = holder.view();
    assert!(before.activity.is_empty(), "the chat is open and idle");

    let pump = Pump::start(&holder);
    pump.send(Converted::Closed {
        chat: chat.clone(),
        expected: false,
    });
    let pushed = pump.finish();

    assert_eq!(pushed.len(), 1, "one child's end, one push");
    assert_eq!(
        pushed[0].activity,
        vec![ChatActivityView {
            chat_id: chat.as_str().to_owned(),
            activity: ChatActivityMark::Ended,
        }],
        "a dead child ends its chat"
    );
}

#[test]
fn the_pump_stops_when_its_source_closes() {
    let (holder, _) = open_stored();
    let pump = Pump::start(&holder);

    // `finish` drops the source and fails the test if the thread outlives it.
    assert!(
        pump.finish().is_empty(),
        "nothing was fed, nothing is pushed"
    );
}
