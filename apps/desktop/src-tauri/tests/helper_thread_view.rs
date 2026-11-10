//! Windowless tests of the open helper's thread slice of the contract (T7b-2):
//! what the selected helper of the foreground chat said, thought and ran, and
//! the header iced draws above it.
//!
//! The frames are fed through the same reducer input the child-event pump folds
//! (`Input::ChatEvent` with a `setWidget` request), so what is asserted is the
//! slice the web side really receives: no child, no window and no launcher runs
//! here. Every text is checked against the runtime rule that built it, never
//! against a spelling re-derived in this file.

use gentle_app::{ChatId, Input, Intent};
use gentle_core::helpers::HelperStatus;
use gentle_core::rpc::{RpcEvent, UiRequest};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    HelperStatusView, HelperThreadItemView, ToolStatusView, UiIntent, ViewState,
};
use gentle_desk_lib::helper_thread::THREAD_ITEMS_PUSHED;
use gentle_runtime::helper_text::format::{args_line, output_preview};
use gentle_runtime::helper_text::status_label;
use gentle_runtime::text::truncate;
use serde_json::{json, Value};

const SESSION_PATH: &str = "/sessions/a.jsonl";

/// The ellipsis `gentle_runtime::text::truncate` ends a cut text with, written as
/// an escape so this file stays ASCII.
const ELLIPSIS: char = '\u{2026}';

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

/// One `gentle-agents` task carrying `thread`.
fn task(id: &str, status: &str, thread: Value) -> Value {
    json!({
        "summary": {
            "id": id,
            "agent": "gentle-ai-explore",
            "label": format!("{status} helper"),
            "prompt": format!("Do the {status} thing"),
            "status": status,
            "createdAt": "2026-09-22T10:00:00.000Z",
            "turns": 2,
            "toolCalls": 1,
        },
        "thread": thread,
    })
}

fn thread(dropped: u64, items: Vec<Value>) -> Value {
    json!({"version": 1, "dropped": dropped, "items": items})
}

fn said(text: &str) -> Value {
    json!({"kind": "text", "text": text})
}

fn thought(text: &str) -> Value {
    json!({"kind": "thinking", "text": text})
}

fn note(text: &str) -> Value {
    json!({"kind": "note", "text": text})
}

fn tool(
    name: &str,
    args: Value,
    output: Value,
    running: Option<bool>,
    is_error: Option<bool>,
) -> Value {
    let mut item =
        json!({"kind": "tool", "callId": "c1", "name": name, "args": args, "output": output});
    if let Some(running) = running {
        item["running"] = json!(running);
    }
    if let Some(is_error) = is_error {
        item["isError"] = json!(is_error);
    }
    item
}

/// The `setWidget` request carrying one activity frame.
fn widget(tasks: Value) -> RpcEvent {
    let lines = json!({
        "schema": "gentle-agents.activity/v1",
        "summary": {"running": 0, "queued": 0, "waiting": 0, "finished": 1},
        "tasks": tasks,
    });
    let raw = json!({
        "type": "extension_ui_request", "id": "w", "method": "setWidget",
        "widgetKey": "gentle-agents", "widgetLines": [lines.to_string()],
    });
    RpcEvent::UiRequest(UiRequest::FireAndForget {
        id: "w".to_owned(),
        method: "setWidget".to_owned(),
        raw,
    })
}

/// A chat whose one helper is `id`, with `thread` as its activity.
fn with_task(id: &str, body: Value) -> (AppStateHolder, ChatId) {
    let (holder, chat) = open_stored();
    fold(&holder, &chat, widget(json!([task(id, "running", body)])));
    (holder, chat)
}

fn select(holder: &AppStateHolder, helper_id: &str) -> ViewState {
    holder.dispatch(UiIntent::SelectHelper {
        helper_id: helper_id.to_owned(),
    })
}

#[test]
fn opening_a_helper_carries_the_header_iced_draws() {
    let prompt = "Do the running thing";
    let (holder, _chat) = with_task("r", thread(0, vec![said("Reading the crate.")]));

    let view = select(&holder, "r");
    let open = view
        .helper_thread
        .expect("the selected helper has a thread");

    assert_eq!(view.selected_helper.as_deref(), Some("r"));
    assert_eq!(open.id, "r");
    assert_eq!(open.title, "running helper", "the title is the rule's");
    assert_eq!(open.agent, "gentle-ai-explore");
    assert_eq!(open.status, HelperStatusView::Running);
    assert_eq!(open.status_label, status_label(HelperStatus::Running));
    assert_eq!(open.prompt, prompt);
}

#[test]
fn every_item_kind_maps_to_its_own_drawn_variant() {
    let (holder, _chat) = with_task(
        "r",
        thread(
            0,
            vec![
                said("The bridge is thin."),
                thought("which modules move"),
                note("left a TODO"),
            ],
        ),
    );

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert_eq!(
        open.items,
        vec![
            HelperThreadItemView::Said {
                text: "The bridge is thin.".to_owned()
            },
            HelperThreadItemView::Thought {
                text: "which modules move".to_owned()
            },
            HelperThreadItemView::Note {
                text: "left a TODO".to_owned()
            },
        ]
    );
}

#[test]
fn a_tool_call_carries_the_rules_status_arguments_and_output() {
    let args = json!({"path": "crates/runtime/src/lib.rs", "recursive": true});
    let output = "14 intents\n6 state events";
    let (holder, _chat) = with_task(
        "r",
        thread(
            0,
            vec![
                tool("read", args.clone(), json!(output), None, None),
                tool(
                    "grep",
                    json!({"pattern": "Intent"}),
                    json!(""),
                    Some(true),
                    None,
                ),
                tool(
                    "bash",
                    json!({"cmd": "cargo test"}),
                    json!(""),
                    None,
                    Some(true),
                ),
            ],
        ),
    );

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert_eq!(
        open.items,
        vec![
            HelperThreadItemView::Ran {
                name: "read".to_owned(),
                status: ToolStatusView::Ok,
                args_line: args_line(&args),
                output_preview: output_preview(&json!(output)),
            },
            HelperThreadItemView::Ran {
                name: "grep".to_owned(),
                status: ToolStatusView::Running,
                args_line: args_line(&json!({"pattern": "Intent"})),
                output_preview: output_preview(&json!("")),
            },
            HelperThreadItemView::Ran {
                name: "bash".to_owned(),
                status: ToolStatusView::Failed,
                args_line: args_line(&json!({"cmd": "cargo test"})),
                output_preview: output_preview(&json!("")),
            },
        ]
    );
}

#[test]
fn long_arguments_and_outputs_are_truncated_by_the_runtime_rules() {
    let args = json!({"command": "x".repeat(400)});
    let long_output = (1..=30)
        .map(|line| format!("line {line} of a long output"))
        .collect::<Vec<_>>()
        .join("\n");
    let (holder, _chat) = with_task(
        "r",
        thread(
            0,
            vec![tool("bash", args.clone(), json!(long_output), None, None)],
        ),
    );

    let open = select(&holder, "r").helper_thread.expect("a thread");
    let HelperThreadItemView::Ran {
        args_line: sent_args,
        output_preview: sent_output,
        ..
    } = &open.items[0]
    else {
        panic!("the item is a tool call");
    };

    assert_eq!(sent_args, &args_line(&args));
    assert_eq!(sent_output, &output_preview(&json!(long_output)));
    assert!(sent_args.ends_with(ELLIPSIS), "the argument line is cut");
    assert!(sent_output.ends_with(ELLIPSIS), "the output preview is cut");
    assert!(
        sent_output.chars().count() < long_output.chars().count(),
        "the preview is shorter than the output it shows"
    );
}

#[test]
fn a_long_thread_keeps_its_tail_and_counts_the_rest_as_dropped() {
    let items: Vec<Value> = (0..THREAD_ITEMS_PUSHED + 70)
        .map(|index| said(&format!("item {index}")))
        .collect();
    let (holder, _chat) = with_task("r", thread(3, items));

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert_eq!(open.items.len(), THREAD_ITEMS_PUSHED);
    assert_eq!(
        open.items[0],
        HelperThreadItemView::Said {
            text: "item 70".to_owned()
        },
        "the tail of the thread, not its start"
    );
    assert_eq!(
        open.dropped,
        3 + 70,
        "the task's own dropped count plus the items this push left out"
    );
}

#[test]
fn a_helper_with_no_activity_has_an_empty_thread() {
    let (holder, _chat) = with_task("r", thread(0, Vec::new()));

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert!(open.items.is_empty());
    assert_eq!(open.dropped, 0);
}

#[test]
fn a_long_prompt_is_truncated_as_iced_echoes_it() {
    let prompt = "y".repeat(400);
    let (holder, chat) = open_stored();
    let mut with_prompt = task("r", "running", thread(0, vec![said("hi")]));
    with_prompt["summary"]["prompt"] = json!(prompt);
    fold(&holder, &chat, widget(json!([with_prompt])));

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert_eq!(open.prompt, truncate(&prompt, 300));
}

#[test]
fn no_selection_is_no_thread() {
    let (holder, _chat) = with_task("r", thread(0, vec![said("Reading the crate.")]));

    assert_eq!(holder.view().helper_thread, None);
}

#[test]
fn a_selection_of_another_chats_helper_is_never_a_thread() {
    // The slice is the FOREGROUND chat's, as the iced thread is: another chat's
    // helper waits for the user to return to that chat.
    let (holder, _chat) = with_task("r", thread(0, vec![said("Reading the crate.")]));
    select(&holder, "r");

    let view = holder.dispatch(UiIntent::OpenChat {
        chat_id: "/sessions/b.jsonl".to_owned(),
    });

    assert_eq!(view.selected_helper, None);
    assert_eq!(view.helper_thread, None);
}

#[test]
fn closing_the_helper_clears_the_thread() {
    let (holder, _chat) = with_task("r", thread(0, vec![said("Reading the crate.")]));
    select(&holder, "r");

    let view = holder.dispatch(UiIntent::CloseHelper);

    assert_eq!(view.selected_helper, None);
    assert_eq!(view.helper_thread, None);
}

#[test]
fn the_title_is_the_rules_title_even_when_the_task_has_no_label() {
    let (holder, chat) = open_stored();
    let mut unlabelled = task("r", "running", thread(0, vec![said("hi")]));
    unlabelled["summary"]["label"] = json!("");
    fold(&holder, &chat, widget(json!([unlabelled])));

    let open = select(&holder, "r").helper_thread.expect("a thread");

    assert_eq!(
        open.title, "Do the running thing",
        "the prompt line names it"
    );
}

#[test]
fn a_realistic_thread_slice_stays_a_few_kilobytes() {
    // 40 items is a busy helper's thread: what it said between tools, what it
    // thought, its notes, and its tool calls with their arguments and output.
    let mut items = Vec::new();
    for index in 0..10 {
        items.push(said(&format!(
            "Step {index}: reading the modules of the runtime crate and noting what the \
             bridge has to expose to the web side. The bridge stays thin."
        )));
        items.push(thought(&format!(
            "Which of the {index} modules can move without an iced import?"
        )));
        items.push(note("TODO: check the helper thread caps"));
        items.push(tool(
            "read",
            json!({"path": "crates/runtime/src/helper_text.rs", "limit": 200}),
            json!("//! The helpers (delegated subagents) panel as plain text:\n//! the status label, the task title and the counters line both windows draw."),
            None,
            None,
        ));
    }
    let count = items.len();
    let (holder, _chat) = with_task("r", thread(0, items));

    let open = select(&holder, "r").helper_thread.expect("a thread");
    let bytes = serde_json::to_string(&open)
        .expect("the slice serializes")
        .len();

    assert!(
        count <= THREAD_ITEMS_PUSHED,
        "the fixture fits in one push: {count} items"
    );
    assert_eq!(open.items.len(), count);
    assert!(bytes < 8192, "the forty-item thread slice is {bytes} bytes");
}
