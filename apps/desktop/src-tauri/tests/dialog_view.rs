//! Windowless tests of the dialog slice of the contract (T7a): the foreground
//! chat's first pending extension dialog, and the answer that clears it.
//!
//! The requests are fed through the same reducer input the child-event pump
//! folds (`Input::ChatEvent` with a `ui_request`), so what is asserted is the
//! slice the web side really receives, never a view built by the test: no child,
//! no window and no launcher runs here.

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::{RpcCommand, RpcEvent, UiAnswer, UiRequest};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{DialogAnswer, DialogView, UiIntent, ViewState};

const SESSION_PATH: &str = "/sessions/a.jsonl";

/// The holder with the stored chat opened and confirmed, so it is the
/// foreground chat and it is idle.
fn open_stored() -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    let chat = ChatId::of_session(SESSION_PATH);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// Folds one request the child sent, the way the pump's translation does.
fn ask(holder: &AppStateHolder, chat: &ChatId, request: UiRequest) -> ViewState {
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: RpcEvent::UiRequest(request),
    })
}

fn confirm(id: &str) -> UiRequest {
    UiRequest::Confirm {
        id: id.to_owned(),
        title: "Allow this command?".to_owned(),
        message: "Run `ls`?".to_owned(),
        timeout: None,
    }
}

fn select(id: &str) -> UiRequest {
    UiRequest::Select {
        id: id.to_owned(),
        title: "Pick a letter".to_owned(),
        options: vec!["a".to_owned(), "b".to_owned()],
        timeout: None,
    }
}

fn vote(holder: &AppStateHolder, id: &str, answer: DialogAnswer) -> (ViewState, Vec<ChatCommand>) {
    holder.dispatch_commands(UiIntent::AnswerDialog {
        id: id.to_owned(),
        answer,
    })
}

/// The one answer the child is waiting for, as the command it travels in.
fn answered(chat: &ChatId, id: &str, answer: UiAnswer) -> Vec<ChatCommand> {
    vec![ChatCommand::Send {
        chat: chat.clone(),
        rpc: RpcCommand::UiResponse {
            id: id.to_owned(),
            answer,
        },
    }]
}

#[test]
fn a_confirm_request_reaches_the_view_with_its_title_and_message() {
    let (holder, chat) = open_stored();

    let view = ask(&holder, &chat, confirm("d1"));

    assert_eq!(
        view.dialog,
        Some(DialogView::Confirm {
            id: "d1".to_owned(),
            title: "Allow this command?".to_owned(),
            message: "Run `ls`?".to_owned(),
        }),
        "the confirm dialog arrives with the two texts it draws"
    );
}

#[test]
fn a_select_request_carries_its_options_in_the_order_the_child_sent_them() {
    let (holder, chat) = open_stored();

    let view = ask(&holder, &chat, select("d2"));

    assert_eq!(
        view.dialog,
        Some(DialogView::Select {
            id: "d2".to_owned(),
            title: "Pick a letter".to_owned(),
            options: vec!["a".to_owned(), "b".to_owned()],
        }),
        "a select dialog arrives with every option, in order"
    );
}

#[test]
fn an_input_request_carries_its_placeholder_when_it_has_one() {
    let (holder, chat) = open_stored();

    let view = ask(
        &holder,
        &chat,
        UiRequest::Input {
            id: "d3".to_owned(),
            title: "Your name".to_owned(),
            placeholder: Some("name".to_owned()),
            timeout: None,
        },
    );

    assert_eq!(
        view.dialog,
        Some(DialogView::Input {
            id: "d3".to_owned(),
            title: "Your name".to_owned(),
            placeholder: Some("name".to_owned()),
        }),
        "an input dialog arrives with the hint its field shows"
    );
}

#[test]
fn an_editor_request_carries_its_prefill_when_it_has_one() {
    let (holder, chat) = open_stored();

    let view = ask(
        &holder,
        &chat,
        UiRequest::Editor {
            id: "d4".to_owned(),
            title: "Edit the note".to_owned(),
            prefill: Some("a note".to_owned()),
        },
    );

    assert_eq!(
        view.dialog,
        Some(DialogView::Editor {
            id: "d4".to_owned(),
            title: "Edit the note".to_owned(),
            prefill: Some("a note".to_owned()),
        }),
        "an editor dialog arrives with the text its box starts with"
    );
}

#[test]
fn only_the_first_dialog_of_the_queue_is_pushed() {
    let (holder, chat) = open_stored();
    ask(&holder, &chat, confirm("d1"));

    let view = ask(&holder, &chat, select("d2"));

    assert_eq!(
        view.dialog,
        Some(DialogView::Confirm {
            id: "d1".to_owned(),
            title: "Allow this command?".to_owned(),
            message: "Run `ls`?".to_owned(),
        }),
        "a second dialog waits: the view shows one at a time, the oldest first"
    );
}

#[test]
fn answering_the_first_dialog_shows_the_next_one() {
    let (holder, chat) = open_stored();
    ask(&holder, &chat, confirm("d1"));
    ask(&holder, &chat, select("d2"));

    let (view, commands) = vote(&holder, "d1", DialogAnswer::Cancelled);

    assert_eq!(commands, answered(&chat, "d1", UiAnswer::Cancelled));
    assert_eq!(
        view.dialog,
        Some(DialogView::Select {
            id: "d2".to_owned(),
            title: "Pick a letter".to_owned(),
            options: vec!["a".to_owned(), "b".to_owned()],
        }),
        "the next waiting dialog takes the place of the one just answered"
    );
}

#[test]
fn answering_the_last_dialog_leaves_no_dialog_behind() {
    let (holder, chat) = open_stored();
    ask(&holder, &chat, confirm("d1"));

    let (view, commands) = vote(&holder, "d1", DialogAnswer::Cancelled);

    assert_eq!(commands, answered(&chat, "d1", UiAnswer::Cancelled));
    assert_eq!(view.dialog, None, "the answered dialog is gone");
}

#[test]
fn the_answer_carries_the_shape_the_child_waits_for() {
    let cases = [
        (
            DialogAnswer::Confirmed { confirmed: true },
            UiAnswer::Confirmed(true),
        ),
        (
            DialogAnswer::Value {
                value: "b".to_owned(),
            },
            UiAnswer::Value("b".to_owned()),
        ),
        (DialogAnswer::Cancelled, UiAnswer::Cancelled),
    ];
    for (answer, expected) in cases {
        let (holder, chat) = open_stored();
        ask(&holder, &chat, select("d2"));

        let (_view, commands) = vote(&holder, "d2", answer.clone());

        assert_eq!(
            commands,
            answered(&chat, "d2", expected),
            "the {answer:?} answer is the one the child's protocol takes"
        );
    }
}

#[test]
fn a_second_answer_to_the_same_dialog_sends_nothing() {
    // A double click on Submit is one answer, not two: the reducer drops the
    // dialog it just answered, so the second answer names nothing.
    let (holder, chat) = open_stored();
    ask(&holder, &chat, confirm("d1"));

    let (_view, first) = vote(&holder, "d1", DialogAnswer::Confirmed { confirmed: true });
    let (view, second) = vote(&holder, "d1", DialogAnswer::Confirmed { confirmed: true });

    assert_eq!(first, answered(&chat, "d1", UiAnswer::Confirmed(true)));
    assert!(
        second.is_empty(),
        "the dialog is no longer pending, so nothing is sent twice"
    );
    assert_eq!(view.dialog, None);
}

#[test]
fn an_answer_naming_no_pending_dialog_changes_nothing() {
    let (holder, chat) = open_stored();
    ask(&holder, &chat, confirm("d1"));

    let (view, commands) = vote(&holder, "nope", DialogAnswer::Cancelled);

    assert!(
        commands.is_empty(),
        "an id the chat is not waiting on answers nothing"
    );
    assert_eq!(
        view.dialog,
        Some(DialogView::Confirm {
            id: "d1".to_owned(),
            title: "Allow this command?".to_owned(),
            message: "Run `ls`?".to_owned(),
        }),
        "the dialog that is really pending stays pending"
    );
}

#[test]
fn a_dialog_of_a_background_chat_is_not_the_foreground_dialog() {
    // The dialog slice is the FOREGROUND chat's, as the iced shell draws it:
    // a background chat's question waits for the user to return to it (S4).
    let (holder, chat) = open_stored();
    let other = ChatId::of_session("/sessions/b.jsonl");
    ask(&holder, &other, confirm("b1"));

    assert_eq!(
        holder.view().dialog,
        None,
        "a background chat's dialog is never drawn over another chat"
    );
    ask(&holder, &chat, confirm("d1"));
    assert_eq!(
        holder.view().dialog,
        Some(DialogView::Confirm {
            id: "d1".to_owned(),
            title: "Allow this command?".to_owned(),
            message: "Run `ls`?".to_owned(),
        }),
        "the foreground chat's own dialog is the one drawn"
    );
}

#[test]
fn a_run_whose_launcher_is_unresolvable_has_no_banner_until_it_is_told_so() {
    // The banner is the shell's own fact, not the reducer's: the app state has
    // no launcher at all (S2), so nothing here may invent one.
    let (holder, _chat) = open_stored();

    assert_eq!(holder.view().fatal, None);

    let view = holder.note_fatal("gentle-shell is not available".to_owned());

    assert_eq!(
        view.fatal.as_deref(),
        Some("gentle-shell is not available"),
        "the bar says the one thing it knows: the launcher is not there"
    );
    assert_eq!(
        holder.view().fatal.as_deref(),
        Some("gentle-shell is not available"),
        "every later slice carries the same bar"
    );
}
