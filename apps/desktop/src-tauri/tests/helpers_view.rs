//! Windowless tests of the helpers slice of the contract (T7b-1): the
//! foreground chat's helpers panel, one card per task, and the selection the
//! reducer keeps for it.
//!
//! The stand-in frame the fake shell publishes for its `helpers` prompt
//! (`tools/fake-gentle-shell/helpers-frame.mjs`) is fed through the same reducer
//! input the child-event pump folds (`Input::ChatEvent` with a `setWidget`
//! request), so what is asserted is the slice the web side really receives, not
//! a view built by the test: no child, no window and no launcher runs here.

use gentle_app::{ChatId, Input, Intent};
use gentle_core::helpers::{HelperStatus, HelpersSummary};
use gentle_core::rpc::{RpcEvent, UiRequest};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{HelperStatusView, UiIntent, ViewState};
use gentle_runtime::helper_text::{status_label, summary_line};

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

/// Folds one event the child sent, the way the pump's translation does.
fn fold(holder: &AppStateHolder, chat: &ChatId, event: RpcEvent) -> ViewState {
    holder.fold_input(Input::ChatEvent {
        chat: chat.clone(),
        event,
    })
}

/// One `gentle-agents` task, with the fields the stand-in frame adds per status.
fn task(id: &str, status: &str, extra: serde_json::Value) -> serde_json::Value {
    let mut summary = serde_json::json!({
        "id": id,
        "agent": "gentle-ai-explore",
        "label": format!("{status} helper"),
        "prompt": format!("Do the {status} thing"),
        "status": status,
        "createdAt": "2026-09-22T10:00:00.000Z",
        "turns": 2,
        "toolCalls": 1,
    });
    let (Some(fields), Some(extra)) = (summary.as_object_mut(), extra.as_object()) else {
        panic!("a task is an object");
    };
    for (key, value) in extra {
        fields.insert(key.clone(), value.clone());
    }
    serde_json::json!({
        "summary": summary,
        "thread": {
            "version": 1,
            "dropped": 0,
            "items": [{"kind": "text", "text": format!("Working as {status}.")}],
        },
    })
}

/// The `setWidget` request carrying one activity frame.
fn widget(tasks: serde_json::Value, summary: serde_json::Value) -> RpcEvent {
    let lines = serde_json::json!({
        "schema": "gentle-agents.activity/v1",
        "summary": summary,
        "tasks": tasks,
    });
    let raw = serde_json::json!({
        "type": "extension_ui_request", "id": "w", "method": "setWidget",
        "widgetKey": "gentle-agents", "widgetLines": [lines.to_string()],
    });
    RpcEvent::UiRequest(UiRequest::FireAndForget {
        id: "w".to_owned(),
        method: "setWidget".to_owned(),
        raw,
    })
}

/// The five helpers the stand-in publishes: one per interesting state, with
/// counters 1 / 1 / 1 / 2.
fn stand_in() -> RpcEvent {
    let tasks = serde_json::json!([
        task("r", "running", serde_json::json!({"lastStep": "grep"})),
        task("w", "waiting", serde_json::json!({})),
        task("q", "queued", serde_json::json!({})),
        task(
            "d",
            "done",
            serde_json::json!({"endedAt": "2026-09-22T10:05:00.000Z"})
        ),
        task(
            "f",
            "failed",
            serde_json::json!({"endedAt": "2026-09-22T10:06:00.000Z", "error": "model quota exceeded"}),
        ),
    ]);
    widget(
        tasks,
        serde_json::json!({"running": 1, "queued": 1, "waiting": 1, "finished": 2}),
    )
}

/// A chat with the stand-in's five helpers on screen.
fn with_helpers() -> (AppStateHolder, ChatId) {
    let (holder, chat) = open_stored();
    fold(&holder, &chat, stand_in());
    (holder, chat)
}

fn select(holder: &AppStateHolder, helper_id: &str) -> ViewState {
    holder.dispatch(UiIntent::SelectHelper {
        helper_id: helper_id.to_owned(),
    })
}

#[test]
fn one_row_per_helper_carries_its_status_and_the_rules_label() {
    let (holder, _chat) = with_helpers();

    let rows = holder
        .view()
        .helpers
        .expect("an open chat has a panel")
        .tasks;
    let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        ids,
        ["r", "w", "q", "f", "d"],
        "one card per helper, in the order the state keeps them: running, waiting, queued, then finished newest first"
    );
    let expected = [
        HelperStatusView::Running,
        HelperStatusView::Waiting,
        HelperStatusView::Queued,
        HelperStatusView::Failed,
        HelperStatusView::Done,
    ];
    let statuses: Vec<HelperStatusView> = rows.iter().map(|row| row.status).collect();
    assert_eq!(statuses, expected);
    let labels: Vec<&str> = rows.iter().map(|row| row.status_label.as_str()).collect();
    assert_eq!(
        labels,
        ["running", "waiting", "queued", "failed", "done"],
        "every label is the text rule's"
    );
}

#[test]
fn a_cancelled_helper_is_a_status_of_its_own() {
    let (holder, chat) = open_stored();
    fold(
        &holder,
        &chat,
        widget(
            serde_json::json!([task("c", "cancelled", serde_json::json!({}))]),
            serde_json::json!({"running": 0, "queued": 0, "waiting": 0, "finished": 1}),
        ),
    );

    let view = holder.view();
    let row = &view.helpers.expect("a panel").tasks[0];

    assert_eq!(row.status, HelperStatusView::Cancelled);
    assert_eq!(row.status_label, status_label(HelperStatus::Cancelled));
}

#[test]
fn a_card_carries_the_title_agent_counts_step_and_error_of_its_task() {
    let (holder, _chat) = with_helpers();

    let view = holder.view();
    let rows = &view.helpers.expect("a panel").tasks;
    let running = &rows[0];
    assert_eq!(running.title, "running helper", "the label is the title");
    assert_eq!(running.agent, "gentle-ai-explore");
    assert_eq!((running.turns, running.tool_calls), (2, 1));
    assert_eq!(running.last_step.as_deref(), Some("grep"));
    assert_eq!(running.error, None);
    let failed = &rows[3];
    assert_eq!(failed.error.as_deref(), Some("model quota exceeded"));
    assert_eq!(failed.last_step, None);
}

#[test]
fn the_counters_line_is_the_rules_text() {
    let (holder, _chat) = with_helpers();

    let helpers = holder.view().helpers.expect("a panel");

    assert_eq!(
        (
            helpers.running,
            helpers.queued,
            helpers.waiting,
            helpers.finished
        ),
        (1, 1, 1, 2),
        "the four counts travel beside the line the tab's badge reads"
    );
    assert_eq!(
        helpers.summary_line,
        summary_line(&HelpersSummary {
            running: 1,
            queued: 1,
            waiting: 1,
            finished: 2,
        }),
        "the line is the runtime rule's, never a reopened-in-TS counter"
    );
}

#[test]
fn a_background_chats_helpers_never_reach_the_foreground_panel() {
    // The slice is the FOREGROUND chat's, as the iced panel draws it: another
    // chat's helpers wait for the user to return to that chat.
    let (holder, _chat) = open_stored();
    let other = ChatId::of_session("/sessions/b.jsonl");
    fold(&holder, &other, stand_in());

    let view = holder.view();

    let helpers = view.helpers.expect("the foreground chat still has a panel");
    assert!(
        helpers.tasks.is_empty(),
        "the foreground chat delegated nothing"
    );
    assert_eq!((helpers.running, helpers.finished), (0, 0));
    assert_eq!(
        helpers.summary_line,
        summary_line(&HelpersSummary::default())
    );
}

#[test]
fn a_chat_with_no_helpers_still_has_a_panel_and_says_so_with_zeros() {
    let (holder, _chat) = open_stored();

    let helpers = holder
        .view()
        .helpers
        .expect("an open chat with no helpers still answers the empty panel");

    assert!(helpers.tasks.is_empty());
    assert_eq!(
        helpers.summary_line,
        summary_line(&HelpersSummary::default())
    );
}

#[test]
fn no_open_chat_has_neither_helpers_nor_a_selection() {
    let holder = AppStateHolder::default();

    let view = holder.view();

    assert_eq!(view.helpers, None, "no chat is no panel at all");
    assert_eq!(view.selected_helper, None);
}

#[test]
fn selecting_a_helper_is_what_the_slice_reports() {
    let (holder, _chat) = with_helpers();

    let view = select(&holder, "r");

    assert_eq!(
        view.selected_helper.as_deref(),
        Some("r"),
        "the reducer's selection reaches the web side"
    );
}

#[test]
fn closing_the_selection_returns_to_the_list() {
    let (holder, _chat) = with_helpers();
    select(&holder, "r");

    let view = holder.dispatch(UiIntent::CloseHelper);

    assert_eq!(view.selected_helper, None);
}

#[test]
fn selecting_an_id_the_panel_does_not_list_changes_nothing() {
    // The reducer's own rule, asserted as it is: an id that names no task of the
    // foreground chat leaves the open helper open.
    let (holder, _chat) = with_helpers();
    select(&holder, "r");

    let view = select(&holder, "ghost");

    assert_eq!(
        view.selected_helper.as_deref(),
        Some("r"),
        "an unknown id never clears a task that is really listed"
    );
}

#[test]
fn a_selection_of_another_chats_helper_is_never_reported() {
    // The selection names a task of ONE chat. Opening another chat makes it the
    // foreground one, and the reducer forgets the selection with it, so the
    // panel of the new chat never marks a card it does not draw.
    let (holder, _chat) = with_helpers();
    select(&holder, "r");

    let view = holder.dispatch(UiIntent::OpenChat {
        chat_id: "/sessions/b.jsonl".to_owned(),
    });

    assert_eq!(
        view.selected_helper, None,
        "a selection never outlives the chat whose helper it names"
    );
    assert!(
        view.helpers.expect("a panel").tasks.is_empty(),
        "the new chat delegated nothing of its own yet"
    );
}

#[test]
fn the_helpers_slice_is_a_handful_of_short_lines() {
    // The panel is not a transcript: this slice carries no thread item at all
    // (T7b-2 brings the said/thought/ran view). The stand-in's five helpers are
    // the widest frame the slice is checked against, and the budget keeps a
    // later change from quietly pushing whole transcripts on every event.
    let (holder, _chat) = with_helpers();

    let helpers = holder.view().helpers.expect("a panel");
    let bytes = serde_json::to_string(&helpers)
        .expect("the slice serializes")
        .len();

    assert!(bytes < 1024, "the five-helper slice is {bytes} bytes");
}
