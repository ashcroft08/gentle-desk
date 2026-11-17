//! Stopping one running helper through the contract and the effects seam
//! (helper-stop T2/T3, S2/S7): the intent asks for one stop of the foreground
//! chat's helper, the child's acceptance marks exactly that helper `Cancelled`,
//! and a stop that did not happen folds nothing at all.
//!
//! No child and no window run here: the helpers arrive as the stand-in
//! `gentle-agents` frame (the reducer input the child-event pump folds), and the
//! executor is the recording fake of `chat_seam`.

mod chat_seam;
use chat_seam::{stored, Answer, Fake, SESSION_PATH};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::{RpcEvent, UiRequest};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    ChatActivityMark, ChatActivityView, HelperStatusView, UiIntent, ViewState,
};
use gentle_desk_lib::effects;

/// The holder with the stored chat opened and confirmed, so it is the foreground
/// chat and it is idle. `chat_seam::stored` is the same chat the reducer names.
fn open_stored() -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// The `setWidget` request carrying one activity frame of `(id, status)` pairs.
fn widget(tasks: &[(&str, &str)]) -> RpcEvent {
    let tasks: Vec<serde_json::Value> = tasks
        .iter()
        .map(|(id, status)| {
            serde_json::json!({
                "summary": {
                    "id": id,
                    "agent": "gentle-ai-worker",
                    "label": format!("{status} helper"),
                    "prompt": "Do the thing",
                    "status": status,
                    "createdAt": "2026-09-22T10:00:00.000Z",
                },
                "thread": {"version": 1, "dropped": 0, "items": []},
            })
        })
        .collect();
    let running = tasks.len();
    let lines = serde_json::json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": running, "queued": 0, "waiting": 0, "finished": 0},
        "tasks": tasks,
    });
    RpcEvent::UiRequest(UiRequest::FireAndForget {
        id: "w".to_owned(),
        method: "setWidget".to_owned(),
        raw: serde_json::json!({
            "type": "extension_ui_request", "id": "w", "method": "setWidget",
            "widgetKey": "gentle-agents", "widgetLines": [lines.to_string()],
        }),
    })
}

/// A chat with two helpers on screen: one running, one waiting.
fn with_helpers() -> (AppStateHolder, ChatId) {
    let (holder, chat) = open_stored();
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: widget(&[("h1", "running"), ("h2", "waiting")]),
    });
    (holder, chat)
}

/// The status one card of the pushed helpers slice carries.
fn card(view: &ViewState, id: &str) -> Option<HelperStatusView> {
    view.helpers
        .as_ref()?
        .tasks
        .iter()
        .find(|row| row.id == id)
        .map(|row| row.status)
}

#[test]
fn the_stop_intent_asks_for_one_stop_of_the_foreground_chats_helper() {
    let (holder, chat) = with_helpers();

    let (view, commands) = holder.dispatch_commands(UiIntent::StopHelper {
        helper_id: "h1".to_owned(),
    });

    assert_eq!(
        commands,
        vec![ChatCommand::StopHelper {
            chat,
            helper_id: "h1".to_owned(),
        }],
        "one stop, for the chat that owns the helper and that helper alone"
    );
    assert_eq!(
        card(&view, "h1"),
        Some(HelperStatusView::Running),
        "the card keeps its mark until the child answers (S7)"
    );
}

#[test]
fn a_helper_that_is_not_running_asks_for_nothing() {
    let (holder, _chat) = with_helpers();

    let (_view, commands) = holder.dispatch_commands(UiIntent::StopHelper {
        helper_id: "h2".to_owned(),
    });

    assert_eq!(
        commands,
        Vec::new(),
        "a waiting helper (like a queued or finished one) cannot be stopped"
    );
}

#[test]
fn an_unknown_helper_asks_for_nothing() {
    let (holder, _chat) = with_helpers();

    let (_view, commands) = holder.dispatch_commands(UiIntent::StopHelper {
        helper_id: "ghost".to_owned(),
    });

    assert_eq!(commands, Vec::new());
}

#[test]
fn the_childs_acceptance_marks_exactly_that_helper_cancelled() {
    let (holder, chat) = with_helpers();
    // The chat is mid-turn, which is what a running helper is doing.
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event: RpcEvent::AgentStart,
    });
    let (_, commands) = holder.dispatch_commands(UiIntent::StopHelper {
        helper_id: "h1".to_owned(),
    });

    let ran = effects::run_one(commands[0].clone(), &Fake::answering(Answer::HelperStopped));

    assert_eq!(
        ran.inputs,
        vec![Input::HelperStopped {
            chat: chat.clone(),
            helper_id: "h1".to_owned(),
        }],
        "an accepted stop folds one input and says nothing else"
    );
    let view = holder.fold_input(ran.inputs[0].clone());
    assert_eq!(
        card(&view, "h1"),
        Some(HelperStatusView::Cancelled),
        "the stopped helper reads Cancelled, drawn as the Ended mark"
    );
    assert_eq!(
        card(&view, "h2"),
        Some(HelperStatusView::Waiting),
        "its sibling is untouched (S1)"
    );
    assert_eq!(
        view.activity,
        vec![ChatActivityView {
            chat_id: chat.as_str().to_owned(),
            activity: ChatActivityMark::Working,
        }],
        "the chat keeps running: only the helper stopped"
    );
}

#[test]
fn the_open_thread_of_the_stopped_helper_reads_cancelled_too() {
    let (holder, _chat) = with_helpers();
    holder.dispatch(UiIntent::SelectHelper {
        helper_id: "h1".to_owned(),
    });

    holder.fold_input(Input::HelperStopped {
        chat: stored(),
        helper_id: "h1".to_owned(),
    });

    let thread = holder
        .view()
        .helper_thread
        .expect("the selected helper's thread");
    assert_eq!(thread.id, "h1");
    assert_eq!(thread.status, HelperStatusView::Cancelled);
}

#[test]
fn a_stop_the_channel_could_not_make_folds_nothing() {
    // Inactive channel, no child, timeout or a refusal: today's behaviour, with
    // no notice and no mark (S7).
    let (holder, chat) = with_helpers();
    let command = ChatCommand::StopHelper {
        chat: chat.clone(),
        helper_id: "h1".to_owned(),
    };

    let ran = effects::run_one(command, &Fake::answering(Answer::HelperNotStopped));

    assert_eq!(ran.inputs, Vec::new());
    assert_eq!(
        card(&holder.view(), "h1"),
        Some(HelperStatusView::Running),
        "the helper keeps the mark it had"
    );
    assert!(
        holder.view().notices.is_empty(),
        "silent degradation never leaves a notice"
    );
}

/// The path the seam's fake names a stored session with, kept asserted so a
/// change to the chat-id scheme is noticed here too.
#[test]
fn the_fixture_chat_is_the_stored_session() {
    assert_eq!(stored(), ChatId::of_session(SESSION_PATH));
}
