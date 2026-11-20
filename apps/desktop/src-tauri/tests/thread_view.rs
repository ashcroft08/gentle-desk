//! Windowless tests of the thread slice of the contract (T5a): every field a
//! streamed or failed assistant message needs to reach the web side.
//!
//! The turn is fed through the same reducer inputs the child-event pump folds,
//! so what is asserted is the slice the web side really receives, never a view
//! built by the test: no child, no window and no clipboard runs here.

mod pump_seam;
use pump_seam::{assistant_message, assistant_turn, errored_turn, event_at, open_stored};

use gentle_app::{ChatId, Input, Intent};
use gentle_core::chat::TOOL_PAYLOAD_MAX_BYTES;
use gentle_core::rpc::{AssistantMessageEvent, RpcEvent, RpcMessage};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    ChatItemView, MessageRole, MessageView, ThinkingItemView, ToolCallItemView, ViewState,
};
use gentle_desk_lib::view::MESSAGES_PUSHED;
use gentle_runtime::chat_runtime::Converted;
use serde_json::json;

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

/// The id any row of the transcript is keyed by, whichever shape it has.
fn id_of(item: &ChatItemView) -> &str {
    match item {
        ChatItemView::Message(message) => &message.id,
        ChatItemView::Thinking(thinking) => &thinking.id,
        ChatItemView::Tool(tool) => &tool.id,
    }
}

/// One event of `chat`'s child with NO clock at all: the path a fixture without
/// timestamps takes, where no row may invent a duration it has no evidence for.
fn untimed(chat: &ChatId, event: RpcEvent) -> Converted {
    Converted::Input(Input::ChatEvent {
        chat: chat.clone(),
        event,
    })
}

/// The assistant message a turn opens with, carrying no content yet.
fn message_start_event() -> RpcEvent {
    RpcEvent::MessageStart(RpcMessage {
        role: "assistant".to_owned(),
        raw: json!({"role": "assistant", "content": []}),
    })
}

/// The ONE tool row of a view, as the card reads it.
fn only_tool(view: &ViewState) -> &ToolCallItemView {
    let rows: Vec<&ToolCallItemView> = view
        .items
        .iter()
        .filter_map(|item| match item {
            ChatItemView::Tool(tool) => Some(tool),
            ChatItemView::Message(_) | ChatItemView::Thinking(_) => None,
        })
        .collect();
    assert_eq!(rows.len(), 1, "one tool call in the thread");
    rows[0]
}

/// The ONE reasoning row of a view, as the card reads it.
fn only_thinking(view: &ViewState) -> &ThinkingItemView {
    let rows: Vec<&ThinkingItemView> = view
        .items
        .iter()
        .filter_map(|item| match item {
            ChatItemView::Thinking(thinking) => Some(thinking),
            ChatItemView::Message(_) | ChatItemView::Tool(_) => None,
        })
        .collect();
    assert_eq!(rows.len(), 1, "one reasoning block in the thread");
    rows[0]
}

/// The message of one closed turn, as the web side receives it.
fn closed(id: &str, role: MessageRole, text: &str, error: Option<&str>) -> MessageView {
    MessageView {
        id: id.to_owned(),
        role,
        text: text.to_owned(),
        streaming: false,
        error: error.map(str::to_owned),
        images: Vec::new(),
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
            images: Vec::new(),
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

/// One turn that reasons, runs a tool and then answers, each event at the
/// wall-clock time the pump would stamp it with: a reasoning block from 1000 to
/// 1200, a tool execution from 1300 to 1500, and the text after both.
fn thinking_and_tool_turn(chat: &ChatId) -> Vec<Converted> {
    let message = |raw: serde_json::Value| RpcMessage {
        role: "assistant".to_owned(),
        raw,
    };
    let thinking =
        |event: AssistantMessageEvent, at: u64| event_at(chat, RpcEvent::MessageUpdate(event), at);
    let execution = |event: RpcEvent, at: u64| event_at(chat, event, at);
    vec![
        event_at(
            chat,
            RpcEvent::MessageStart(message(json!({"role": "assistant", "content": []}))),
            1_000,
        ),
        thinking(
            AssistantMessageEvent::ThinkingStart { content_index: 0 },
            1_000,
        ),
        thinking(
            AssistantMessageEvent::ThinkingDelta {
                content_index: 0,
                delta: "Let me look.".to_owned(),
            },
            1_100,
        ),
        thinking(
            AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "Let me look.".to_owned(),
            },
            1_200,
        ),
        execution(
            RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(json!({"command": "ls"})),
            },
            1_300,
        ),
        execution(
            RpcEvent::ToolExecutionUpdate {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: None,
                partial_result: Some(json!({"content": [{"type": "text", "text": "a\n"}]})),
            },
            1_350,
        ),
        execution(
            RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                result: Some(json!({"content": [{"type": "text", "text": "a\nb\n"}]})),
                is_error: false,
            },
            1_500,
        ),
        thinking(
            AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: "done".to_owned(),
            },
            1_600,
        ),
        event_at(
            chat,
            RpcEvent::MessageEnd(message(json!({
                "role": "assistant",
                "stopReason": "stop",
                "content": [{"type": "text", "text": "done"}],
            }))),
            2_000,
        ),
    ]
}

/// What a card needs to exist for: the reasoning, the tool call and the text as
/// ONE ordered list, each row with the id a keyed view keys it by
/// (chat-transcript T2a, L4, S3).
#[test]
fn the_transcript_reaches_the_view_as_ordered_items_with_their_ids() {
    let (holder, chat) = open_stored();

    let view = fold_turn(&holder, thinking_and_tool_turn(&chat));

    assert_eq!(
        view.items,
        vec![
            ChatItemView::Thinking(ThinkingItemView {
                id: "think-0".to_owned(),
                text: "Let me look.".to_owned(),
                // `thinking_end` closed it, so the row is no longer growing.
                streaming: false,
                // 1200 - 1000: the block's own two stamps.
                duration_ms: Some(200.0),
            }),
            ChatItemView::Tool(ToolCallItemView {
                id: "tool-call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(json!({"command": "ls"})),
                // The result superseded the running output the update reported.
                partial_result: None,
                result: Some(json!({"content": [{"type": "text", "text": "a\nb\n"}]})),
                is_error: false,
                running: false,
                // 1500 - 1300: the execution's own two stamps.
                duration_ms: Some(200.0),
                truncated: false,
                dropped: false,
            }),
            ChatItemView::Message(closed("msg-0", MessageRole::Assistant, "done", None)),
        ],
        "the rows arrive in the child's own order, each with its own id"
    );

    // The face the web reads today, for that same thread: the text row and
    // nothing else, exactly as it was before the transcript kept the rest.
    assert_eq!(
        view.messages,
        vec![closed("msg-0", MessageRole::Assistant, "done", None)],
        "the messages field is still the text projection"
    );
}

/// The wire shape a card reads: the row's kind, its payload under `data`, the
/// field names the generated bindings declare, and the child's own JSON passed
/// through as JSON rather than as a string this side built.
#[test]
fn a_tool_item_travels_with_its_kind_and_its_untyped_payloads() {
    let (holder, chat) = open_stored();

    let view = fold_turn(&holder, thinking_and_tool_turn(&chat));
    let tool = serde_json::to_value(&view.items[1]).expect("the item serializes");

    assert_eq!(
        tool,
        json!({
            "kind": "Tool",
            "data": {
                "id": "tool-call_1",
                "tool_name": "bash",
                "args": {"command": "ls"},
                "partial_result": null,
                "result": {"content": [{"type": "text", "text": "a\nb\n"}]},
                "is_error": false,
                "running": false,
                "duration_ms": 200.0,
                "truncated": false,
                "dropped": false,
            }
        })
    );
}

/// The view in the MIDDLE of a tool call, before its end arrives: the card draws
/// "Running", its body shows the output the last update reported, and there is
/// neither a result nor a duration to draw yet (chat-transcript T2a, S3).
#[test]
fn a_tool_call_seen_while_running_carries_its_running_output_and_no_result() {
    let (holder, chat) = open_stored();
    let update = |text: &str, at: u64| {
        event_at(
            &chat,
            RpcEvent::ToolExecutionUpdate {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: None,
                partial_result: Some(json!({"content": [{"type": "text", "text": text}]})),
            },
            at,
        )
    };

    let view = fold_turn(
        &holder,
        vec![
            event_at(&chat, message_start_event(), 1_000),
            event_at(
                &chat,
                RpcEvent::ToolExecutionStart {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(json!({"command": "ls"})),
                },
                1_300,
            ),
            update("a\n", 1_350),
            update("a\nb\n", 1_400),
        ],
    );

    assert_eq!(
        *only_tool(&view),
        ToolCallItemView {
            id: "tool-call_1".to_owned(),
            tool_name: "bash".to_owned(),
            args: Some(json!({"command": "ls"})),
            // The LAST update's output: a running call shows the tail of what it
            // has produced so far, not the first line it printed.
            partial_result: Some(json!({"content": [{"type": "text", "text": "a\nb\n"}]})),
            // Nothing has ended, so there is no result and no duration to draw:
            // a duration here would be a claim about a call still running.
            result: None,
            is_error: false,
            running: true,
            duration_ms: None,
            truncated: false,
            dropped: false,
        },
        "the running call is the card's row, carrying what the child last reported"
    );
}

/// A tool output longer than the core keeps reaches the card marked as CUT, and
/// a call the child failed reaches it marked as a failure: the two marks a card
/// words differently (chat-transcript T2a, L4, L14).
#[test]
fn a_tool_output_cut_to_the_bound_says_it_was_cut_and_carries_the_failure_mark() {
    let (holder, chat) = open_stored();
    let long = "x".repeat(TOOL_PAYLOAD_MAX_BYTES * 4);

    let view = fold_turn(
        &holder,
        vec![
            event_at(&chat, message_start_event(), 1_000),
            event_at(
                &chat,
                RpcEvent::ToolExecutionStart {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(json!({"command": "cat big.log"})),
                },
                1_300,
            ),
            event_at(
                &chat,
                RpcEvent::ToolExecutionEnd {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    result: Some(json!({"content": [{"type": "text", "text": long}]})),
                    is_error: true,
                },
                1_500,
            ),
        ],
    );

    let tool = only_tool(&view);
    assert_eq!(
        (tool.truncated, tool.dropped, tool.is_error),
        (true, false, true),
        "the output was cut short, no field was lost, and the child failed the call"
    );
    assert!(!tool.running, "the execution ended");
    assert_eq!(tool.duration_ms, Some(200.0), "1500 - 1300");

    // What it kept is the head of that output, cut to the bound the card folds at.
    let kept = tool
        .result
        .as_ref()
        .and_then(|result| result.get("content"))
        .and_then(|content| content.get(0))
        .and_then(|block| block.get("text"))
        .and_then(serde_json::Value::as_str)
        .expect("the text the row kept");
    assert_eq!(
        kept.len(),
        TOOL_PAYLOAD_MAX_BYTES,
        "the row keeps the bound, not the whole output"
    );
    assert!(kept.chars().all(|character| character == 'x'));
}

/// Cutting an object's KEY to the bound can make it land on another key of the
/// same object, and a JSON object keeps one value per key: the field is LOST,
/// which `truncated` cannot say, so the row carries `dropped` too
/// (chat-transcript L10, T2a).
#[test]
fn a_cut_key_that_collides_says_a_field_of_the_payload_was_dropped() {
    let (holder, chat) = open_stored();
    // The two keys differ only AFTER the bound, so cutting each one to it makes
    // them the same string.
    let prefix = "k".repeat(TOOL_PAYLOAD_MAX_BYTES);
    let mut fields = serde_json::Map::new();
    fields.insert(
        format!("{prefix}-first"),
        json!("the value the payload keeps"),
    );
    fields.insert(
        format!("{prefix}-second"),
        json!("the value that would overwrite it"),
    );

    let view = fold_turn(
        &holder,
        vec![
            event_at(&chat, message_start_event(), 1_000),
            event_at(
                &chat,
                RpcEvent::ToolExecutionStart {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(json!({"command": "env"})),
                },
                1_300,
            ),
            event_at(
                &chat,
                RpcEvent::ToolExecutionEnd {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    result: Some(serde_json::Value::Object(fields)),
                    is_error: false,
                },
                1_500,
            ),
        ],
    );

    let tool = only_tool(&view);
    assert_eq!(
        (tool.dropped, tool.truncated),
        (true, true),
        "a field of the payload was lost to the cut, and the keys were shortened too"
    );

    let kept = tool
        .result
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .expect("the result is an object");
    assert_eq!(
        kept.len(),
        1,
        "one key, one field: the colliding field is the loss"
    );
    assert!(
        kept.keys().all(|key| key.len() == TOOL_PAYLOAD_MAX_BYTES),
        "the key the row kept is cut to the bound: {:?}",
        kept.keys().collect::<Vec<&String>>()
    );
    assert!(!tool.is_error, "this call did not fail");
}

/// The reasoning row the card draws WHILE the model is still thinking: the row
/// is open, its text grows with each delta, and with no clock supplied there is
/// no duration to show (chat-transcript T2a, S3).
#[test]
fn a_thinking_row_streams_while_its_text_grows_and_invents_no_duration() {
    let (holder, chat) = open_stored();

    let view = fold_turn(
        &holder,
        vec![
            untimed(&chat, message_start_event()),
            untimed(
                &chat,
                RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
            ),
            untimed(
                &chat,
                RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingDelta {
                    content_index: 0,
                    delta: "Let me".to_owned(),
                }),
            ),
        ],
    );
    assert_eq!(
        *only_thinking(&view),
        ThinkingItemView {
            id: "think-0".to_owned(),
            text: "Let me".to_owned(),
            streaming: true,
            duration_ms: None,
        },
        "the block is open while its text is still arriving"
    );

    // The next delta grows the SAME row: one block, never a row per delta.
    let view = fold_turn(
        &holder,
        vec![untimed(
            &chat,
            RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingDelta {
                content_index: 0,
                delta: " look.".to_owned(),
            }),
        )],
    );
    assert_eq!(
        *only_thinking(&view),
        ThinkingItemView {
            id: "think-0".to_owned(),
            text: "Let me look.".to_owned(),
            streaming: true,
            duration_ms: None,
        },
        "the row keeps growing, and no clock means no duration to claim"
    );
}

/// The same row with the child's clock: while it runs there is still no
/// duration (its end stamp has not arrived), and once it closes the card has
/// "Thought 0.2s" to draw (chat-transcript T2a, S3).
#[test]
fn a_thinking_row_with_a_clock_reports_its_duration_once_it_closes() {
    let (holder, chat) = open_stored();

    let view = fold_turn(
        &holder,
        vec![
            event_at(&chat, message_start_event(), 1_000),
            event_at(
                &chat,
                RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
                1_000,
            ),
            event_at(
                &chat,
                RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingDelta {
                    content_index: 0,
                    delta: "Let me look.".to_owned(),
                }),
                1_100,
            ),
        ],
    );
    assert_eq!(
        *only_thinking(&view),
        ThinkingItemView {
            id: "think-0".to_owned(),
            text: "Let me look.".to_owned(),
            streaming: true,
            duration_ms: None,
        },
        "a start stamp alone is not a duration: the block is still running"
    );

    let view = fold_turn(
        &holder,
        vec![event_at(
            &chat,
            RpcEvent::MessageUpdate(AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "Let me look.".to_owned(),
            }),
            1_200,
        )],
    );
    assert_eq!(
        *only_thinking(&view),
        ThinkingItemView {
            id: "think-0".to_owned(),
            text: "Let me look.".to_owned(),
            streaming: false,
            duration_ms: Some(200.0),
        },
        "1200 - 1000: the block's own two stamps, and the row is closed"
    );
}

/// The push stays bounded: the ordered list is the tail of the transcript, of
/// any kind, and a text row the two lists share is the SAME row - the one
/// `messages` carries and the one `items` carries out of one builder.
#[test]
fn a_long_thread_pushes_the_same_tail_to_both_lists() {
    let (holder, chat) = open_stored();
    // Two more prompts than a window holds, then a turn that adds three rows of
    // its own: the transcript is longer than either window, and its tail is a
    // mix of kinds.
    let prompts = MESSAGES_PUSHED + 2;
    for number in 0..prompts {
        holder.fold_input(Input::Intent(Intent::SendPrompt(format!(
            "prompt {number}"
        ))));
    }
    let view = fold_turn(&holder, thinking_and_tool_turn(&chat));

    // 52 prompts, a reasoning block, a tool call and the answer: 55 rows, of
    // which the push carries the LAST 50 - the first five are cut away.
    let rows: Vec<&str> = view.items.iter().map(id_of).collect();
    assert_eq!(
        rows.len(),
        MESSAGES_PUSHED,
        "the transcript window is bounded"
    );
    assert_eq!(
        rows.first().copied(),
        Some("msg-5"),
        "the window starts at the transcript's own tail"
    );
    assert_eq!(rows.last().copied(), Some("msg-52"));
    assert_eq!(
        &rows[rows.len() - 3..],
        ["think-0", "tool-call_1", "msg-52"],
        "the tail keeps every kind of row, in order"
    );

    // The text window is the last 50 TEXT rows, which reaches further back than
    // the item window does: 53 text rows, of which msg-0..msg-2 are cut away.
    assert_eq!(view.messages.len(), MESSAGES_PUSHED);
    assert_eq!(
        view.messages.first().map(|message| message.id.as_str()),
        Some("msg-3"),
        "messages is the last 50 text rows, as it always was"
    );

    // Where the two windows overlap they agree row for row: one builder, not two.
    let shared: Vec<MessageView> = view
        .items
        .iter()
        .filter_map(|item| match item {
            ChatItemView::Message(message) => Some(message.clone()),
            ChatItemView::Thinking(_) | ChatItemView::Tool(_) => None,
        })
        .collect();
    assert_eq!(shared.len(), MESSAGES_PUSHED - 2, "two rows are not text");
    assert_eq!(
        &view.messages[view.messages.len() - shared.len()..],
        shared.as_slice(),
        "every shared text row is the same row in both lists"
    );
}
