//! Windowless tests of the thread slice of the contract (T5a): every field a
//! streamed or failed assistant message needs to reach the web side.
//!
//! The turn is fed through the same reducer inputs the child-event pump folds,
//! so what is asserted is the slice the web side really receives, never a view
//! built by the test: no child, no window and no clipboard runs here.

mod pump_seam;
use pump_seam::{assistant_message, assistant_turn, errored_turn, open_stored};

use gentle_app::{Input, Intent};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{MessageRole, MessageView, ViewState};
use gentle_runtime::chat_runtime::Converted;

/// Folds one built turn in order and answers the slice after its last input.
fn fold_turn(holder: &AppStateHolder, turn: Vec<Converted>) -> ViewState {
    let mut view = holder.view();
    for converted in turn {
        match converted {
            Converted::Input(input) => view = holder.fold_input(input),
            _ => panic!("the turn builders of the seam produce inputs only"),
        }
    }
    view
}

/// The message of one closed turn, as the web side receives it.
fn closed(id: &str, role: MessageRole, text: &str, error: Option<&str>) -> MessageView {
    MessageView {
        id: id.to_owned(),
        role,
        text: text.to_owned(),
        streaming: false,
        error: error.map(str::to_owned),
    }
}

#[test]
fn an_open_assistant_message_reaches_the_view_streaming_with_its_id() {
    let (holder, chat) = open_stored();

    let view = fold_turn(&holder, assistant_message(&chat, "hello"));

    assert_eq!(
        view.messages,
        vec![MessageView {
            id: "msg-0".to_owned(),
            role: MessageRole::Assistant,
            text: "hello".to_owned(),
            // The message is open: its deltas keep arriving, so the thread knows
            // not to offer Copy yet.
            streaming: true,
            error: None,
        }],
        "the open message arrives with the id the copy intent takes back"
    );
}

#[test]
fn the_message_stops_streaming_when_the_child_ends_it() {
    let (holder, chat) = open_stored();

    let view = fold_turn(&holder, assistant_turn(&chat, "hello"));

    assert_eq!(
        view.messages,
        vec![closed("msg-0", MessageRole::Assistant, "hello", None)],
        "the closed message is the same message, no longer streaming"
    );
}

#[test]
fn a_provider_failure_carries_its_error_text_and_is_not_streaming() {
    let (holder, chat) = open_stored();

    let view = fold_turn(
        &holder,
        errored_turn(&chat, "Provider says no.", "quota exceeded (fake)"),
    );

    assert_eq!(
        view.messages,
        vec![closed(
            "msg-0",
            MessageRole::Assistant,
            "Provider says no.",
            Some("quota exceeded (fake)")
        )],
        "a stopReason error turn marks the message, it does not hide it"
    );
}

#[test]
fn a_user_turn_carries_its_own_id_and_never_streams() {
    let (holder, _chat) = open_stored();

    let view = holder.fold_input(Input::Intent(Intent::SendPrompt("hello".to_owned())));

    assert_eq!(
        view.messages,
        vec![closed("msg-0", MessageRole::User, "hello", None)],
        "the user's own turn is a finished message with an id of its own"
    );
}

/// The assistant's message follows the user's in the id scheme, so the thread
/// can copy either one by the id it renders.
#[test]
fn a_turn_after_the_users_gets_the_next_id() {
    let (holder, chat) = open_stored();
    holder.fold_input(Input::Intent(Intent::SendPrompt("hello".to_owned())));

    let view = fold_turn(&holder, assistant_turn(&chat, "hi"));

    assert_eq!(
        view.messages,
        vec![
            closed("msg-0", MessageRole::User, "hello", None),
            closed("msg-1", MessageRole::Assistant, "hi", None),
        ]
    );
}
