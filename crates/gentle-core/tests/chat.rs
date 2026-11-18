//! Behaviour tests for the chat reducer, ported from the reference app's
//! `chatReducer.test.ts` and replayed against the recorded RPC fixtures.
//!
//! Every test asserts the whole observable state, so a rule that leaks into a
//! field it should not touch fails here too.

use gentle_core::chat::{
    reduce_chat, reduce_chat_at, ChatItem, ChatMessage, ChatRole, ChatState, Dialog, ToolPayload,
    TOOL_PAYLOAD_MAX_BYTES,
};
use gentle_core::i18n::Language;
use gentle_core::rpc::{
    decode_line, AssistantMessageEvent, Decoded, RpcEvent, RpcMessage, RpcResponse, UiRequest,
};
use serde_json::{json, Value};

fn assistant_role() -> String {
    "assistant".to_owned()
}

fn assistant_start() -> RpcEvent {
    RpcEvent::MessageStart(RpcMessage {
        role: assistant_role(),
        raw: json!({"role": "assistant", "content": []}),
    })
}

fn assistant_end(content: Value) -> RpcEvent {
    RpcEvent::MessageEnd(RpcMessage {
        role: assistant_role(),
        raw: json!({"role": "assistant", "content": content}),
    })
}

fn assistant_end_without_content() -> RpcEvent {
    RpcEvent::MessageEnd(RpcMessage {
        role: assistant_role(),
        raw: json!({"role": "assistant"}),
    })
}

fn update(event: AssistantMessageEvent) -> RpcEvent {
    RpcEvent::MessageUpdate(event)
}

fn text_delta(delta: &str) -> RpcEvent {
    update(AssistantMessageEvent::TextDelta {
        content_index: 0,
        delta: delta.to_owned(),
    })
}

fn assistant(id: &str, text: &str, streaming: bool) -> ChatMessage {
    ChatMessage {
        id: id.to_owned(),
        role: ChatRole::Assistant,
        text: text.to_owned(),
        streaming,
        error: None,
    }
}

fn user(id: &str, text: &str) -> ChatMessage {
    ChatMessage {
        id: id.to_owned(),
        role: ChatRole::User,
        text: text.to_owned(),
        streaming: false,
        error: None,
    }
}

fn reducer_run(events: Vec<RpcEvent>) -> ChatState {
    reducer_run_from(ChatState::default(), events)
}

/// Folds `events` into `state`, with no clock: the state a fixture replays to.
fn reducer_run_from(state: ChatState, events: Vec<RpcEvent>) -> ChatState {
    events
        .into_iter()
        .fold(state, |state, event| {
            reduce_chat(Language::En, state, &event)
        })
}

/// Folds `events` with the wall-clock time each one arrived at, which is how a
/// card gets its "Done 0.2s".
fn reducer_run_at(events: Vec<(RpcEvent, Option<u64>)>) -> ChatState {
    events
        .into_iter()
        .fold(ChatState::default(), |state, (event, now_ms)| {
            reduce_chat_at(Language::En, state, &event, now_ms)
        })
}

/// The transcript a state that holds only text has: the text items, in order.
fn texts(messages: Vec<ChatMessage>) -> Vec<ChatItem> {
    messages.into_iter().map(ChatItem::Text).collect()
}

/// Replays a recorded fixture through the real decoder and the real fold.
fn replay(raw: &str) -> ChatState {
    raw.lines().filter(|line| !line.trim().is_empty()).fold(
        ChatState::default(),
        |state, line| match decode_line(line) {
            Decoded::Event(event) => reduce_chat(Language::En, state, &event),
            Decoded::Unknown(raw) => panic!("fixture line decoded as unknown: {raw}"),
        },
    )
}

mod working {
    use super::*;

    #[test]
    fn agent_start_sets_working_true() {
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &RpcEvent::AgentStart),
            ChatState {
                working: true,
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn agent_end_sets_working_false() {
        let working = ChatState {
            working: true,
            ..ChatState::default()
        };
        assert_eq!(
            reduce_chat(
                Language::En,
                working,
                &RpcEvent::AgentEnd { will_retry: None }
            ),
            ChatState::default()
        );
    }

    #[test]
    fn agent_settled_sets_working_false() {
        let working = ChatState {
            working: true,
            ..ChatState::default()
        };
        assert_eq!(
            reduce_chat(Language::En, working, &RpcEvent::AgentSettled),
            ChatState::default()
        );
    }
}

mod assistant_messages {
    use super::*;

    #[test]
    fn message_start_for_an_assistant_message_opens_an_empty_streaming_text_item() {
        let state = reduce_chat(Language::En, ChatState::default(), &assistant_start());
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![assistant("msg-0", "", true)]),
                // The id `message_start` took came from the counter, which is
                // now ahead of it: ids are never handed out twice (R3-1).
                next_text_id: 1,
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn message_start_for_a_non_assistant_message_is_a_no_op() {
        let event = RpcEvent::MessageStart(RpcMessage {
            role: "toolResult".to_owned(),
            raw: json!({"role": "toolResult", "content": []}),
        });
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &event),
            ChatState::default()
        );
    }

    #[test]
    fn text_delta_appends_to_the_open_assistant_message() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Hello "),
            text_delta("world"),
        ]);
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![assistant("msg-0", "Hello world", true)]),
                next_text_id: 1,
                ..ChatState::default()
            }
        );
    }

    /// R2-001 (also R4-002): a message's text is ONE row, wherever it sits.
    /// Once a tool call has been made, the text the same message says next must
    /// still land in that row instead of being dropped for not being last.
    #[test]
    fn text_that_arrives_after_a_tool_call_lands_in_the_same_message() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Looking. "),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            text_delta("Found it."),
            text_delta(" Done."),
        ]);
        assert_eq!(
            state.items,
            vec![
                ChatItem::Text(assistant("msg-0", "Looking. Found it. Done.", true)),
                ChatItem::Tool {
                    id: "tool-call_1".to_owned(),
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: None,
                    partial_result: None,
                    result: None,
                    is_error: false,
                    running: false,
                    started_at_ms: None,
                    ended_at_ms: None,
                    duration_ms: None,
                },
            ]
        );
        assert_eq!(state.activity, 0, "nothing was dropped, so nothing counts");
    }

    /// R3-002: an authoritative end whose content has no text leaves no empty
    /// message, even when a tool row sits after the bubble: the row is dropped
    /// wherever it is, not only when it is trailing.
    #[test]
    fn message_end_drops_an_empty_bubble_a_tool_row_follows() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Looking."),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            assistant_end(json!([
                {"type": "toolCall", "id": "call_1", "name": "bash", "arguments": {}}
            ])),
        ]);
        assert_eq!(
            state.items,
            vec![ChatItem::Tool {
                id: "tool-call_1".to_owned(),
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: None,
                partial_result: None,
                result: None,
                is_error: false,
                running: false,
                started_at_ms: None,
                ended_at_ms: None,
                duration_ms: None,
            }],
            "the empty bubble leaks into the thread"
        );
        assert!(state.messages().is_empty(), "got {:?}", state.messages());
    }

    #[test]
    fn text_delta_with_no_open_assistant_message_is_a_no_op() {
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &text_delta("orphaned")),
            ChatState::default()
        );
    }

    #[test]
    fn text_delta_does_not_append_to_a_user_message() {
        let state = ChatState {
            items: texts(vec![user("msg-0", "hi")]),
            ..ChatState::default()
        };
        let state = reduce_chat(Language::En, state, &text_delta("orphaned"));
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![user("msg-0", "hi")]),
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn message_end_resyncs_text_from_the_authoritative_content_blocks() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Hel"),
            assistant_end(
                json!([{"type": "text", "text": "Hello "}, {"type": "text", "text": "world"}]),
            ),
        ]);
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![assistant("msg-0", "Hello world", false)]),
                next_text_id: 1,
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn message_end_without_content_blocks_keeps_the_accumulated_delta_text() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("kept"),
            assistant_end_without_content(),
        ]);
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![assistant("msg-0", "kept", false)]),
                next_text_id: 1,
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn a_tool_only_assistant_message_keeps_its_empty_text_bubble_while_streaming() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 0,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
        ]);
        assert_eq!(
            state,
            ChatState {
                // The tool call takes the place the empty bubble held: the
                // bubble is still the end of the transcript, because nothing
                // has been said in it yet.
                items: vec![
                    ChatItem::Tool {
                        id: "tool-call_1".to_owned(),
                        tool_call_id: "call_1".to_owned(),
                        tool_name: "bash".to_owned(),
                        args: None,
                        partial_result: None,
                        result: None,
                        is_error: false,
                        running: false,
                        started_at_ms: None,
                        ended_at_ms: None,
                        duration_ms: None,
                    },
                    ChatItem::Text(assistant("msg-0", "", true)),
                ],
                next_text_id: 1,
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn message_end_drops_an_assistant_message_that_finalizes_with_no_text() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 0,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            update(AssistantMessageEvent::ToolcallEnd {
                content_index: 0,
                tool_call: json!({"id": "call_1", "name": "bash", "arguments": {}}),
            }),
            assistant_end(json!([
                {"type": "toolCall", "id": "call_1", "name": "bash", "arguments": {}}
            ])),
        ]);
        // The turn said nothing, so its empty bubble goes; the tool call it
        // made is the transcript.
        assert_eq!(state.items.len(), 1, "got {:?}", state.items);
        assert!(state.messages().is_empty(), "got {:?}", state.messages());
    }

    #[test]
    fn message_end_drops_an_assistant_message_whose_authoritative_text_is_only_whitespace() {
        let state = reducer_run(vec![
            assistant_start(),
            assistant_end(json!([{"type": "text", "text": "   "}])),
        ]);
        assert!(state.items.is_empty(), "got {:?}", state.items);
    }

    #[test]
    fn message_end_drops_an_assistant_message_whose_accumulated_text_is_only_whitespace() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("  \n\t"),
            assistant_end_without_content(),
        ]);
        assert!(state.items.is_empty(), "got {:?}", state.items);
    }

    /// R3-1: an id is the row's identity, so it is never reused. The message
    /// that finalizes empty is dropped and the `msg-<n>` it held becomes a GAP;
    /// the next message takes the counter's next value. This deliberately
    /// supersedes the pinned `a_dropped_message_frees_its_id_...`, which
    /// asserted the reuse that is the bug.
    #[test]
    fn a_dropped_message_leaves_a_gap_and_does_not_free_its_id() {
        let mut state = ChatState::default();
        state.push_message(ChatRole::User, "hi".to_owned(), false);
        state = reduce_chat(Language::En, state, &assistant_start());
        assert_eq!(state.messages()[1].id, "msg-1");

        state = reduce_chat(Language::En, state, &assistant_end(json!([])));
        assert_eq!(state.items, texts(vec![user("msg-0", "hi")]));

        state = reduce_chat(Language::En, state, &assistant_start());
        assert_eq!(state.messages().len(), 2);
        assert_eq!(
            state
                .messages()
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["msg-0", "msg-2"],
            "the dropped message's id is a gap, not a free slot"
        );
        assert_eq!(state.messages()[1].text, "");
        assert!(state.messages()[1].streaming);
    }

    /// R3-1: ids are unique for the chat's life, gaps included. Counting the
    /// text rows to number the next one fell behind the moment a drop removed
    /// a row from the MIDDLE of the transcript, and the next push reused an id
    /// a later row still holds - which breaks every id-based lookup and every
    /// keyed view of the thread.
    #[test]
    fn a_drop_from_the_middle_of_the_transcript_cannot_reuse_a_living_id() {
        let mut state = ChatState::default();
        // The bubble `message_start` opens is still open when the user's side
        // of the next prompt is pushed after it.
        state = reduce_chat(Language::En, state, &assistant_start());
        state.push_message(ChatRole::User, "hi".to_owned(), false);
        // The turn said nothing, so its empty bubble goes from the middle.
        state = reduce_chat(Language::En, state, &assistant_end(json!([])));
        assert_eq!(state.messages(), vec![user("msg-1", "hi")]);

        state = reduce_chat(Language::En, state, &assistant_start());
        let messages = state.messages();
        let ids: Vec<&str> = messages.iter().map(|message| message.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["msg-1", "msg-2"],
            "the id the drop freed is not handed out again: {ids:?}"
        );
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "two rows share an id: {ids:?}");
    }

    #[test]
    fn message_end_for_a_non_assistant_message_is_a_no_op() {
        let before = ChatState {
            items: texts(vec![assistant("msg-0", "text", true)]),
            ..ChatState::default()
        };
        let event = RpcEvent::MessageEnd(RpcMessage {
            role: "toolResult".to_owned(),
            raw: json!({"role": "toolResult", "content": [{"type": "text", "text": "out"}]}),
        });
        assert_eq!(reduce_chat(Language::En, before.clone(), &event), before);
    }

    #[test]
    fn message_end_with_no_open_assistant_message_is_a_no_op() {
        let before = ChatState {
            items: texts(vec![user("msg-0", "hi")]),
            ..ChatState::default()
        };
        assert_eq!(
            reduce_chat(
                Language::En,
                before.clone(),
                &assistant_end(json!([{"type": "text", "text": "late"}]))
            ),
            before
        );
    }
}

/// The counter every thinking / toolcall / tool_execution event used to add to
/// now only counts what the transcript does not keep (L4): an event that lands
/// in an item is drawn from the item, so counting it too would be a second
/// truth.
mod activity {
    use super::*;

    fn before() -> ChatState {
        ChatState {
            items: texts(vec![assistant("msg-0", "", true)]),
            ..ChatState::default()
        }
    }

    fn assert_does_not_bump(before: &ChatState, event: RpcEvent) {
        let after = reduce_chat(Language::En, before.clone(), &event);
        assert_eq!(after.activity, before.activity, "{event:?}");
    }

    fn assert_bumps(before: &ChatState, event: RpcEvent) {
        let after = reduce_chat(Language::En, before.clone(), &event);
        assert_eq!(after.activity, before.activity + 1, "{event:?}");
        assert_eq!(after.items, before.items, "{event:?}");
    }

    #[test]
    fn the_thinking_events_that_reach_the_transcript_do_not_add_to_activity() {
        let before = before();
        let after_start = reduce_chat(
            Language::En,
            before,
            &update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
        );
        assert_eq!(after_start.activity, 0);
        let after_delta = reduce_chat(
            Language::En,
            after_start,
            &update(AssistantMessageEvent::ThinkingDelta {
                content_index: 0,
                delta: "...".to_owned(),
            }),
        );
        assert_eq!(after_delta.activity, 0);
        let after_end = reduce_chat(
            Language::En,
            after_delta,
            &update(AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "...".to_owned(),
            }),
        );
        assert_eq!(after_end.activity, 0);
    }

    #[test]
    fn the_toolcall_events_that_reach_a_tool_item_do_not_add_to_activity() {
        let before = before();
        let after_start = reduce_chat(
            Language::En,
            before,
            &update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
        );
        assert_eq!(after_start.activity, 0);
        let after_end = reduce_chat(
            Language::En,
            after_start,
            &update(AssistantMessageEvent::ToolcallEnd {
                content_index: 1,
                tool_call: json!({"id": "call_1", "name": "bash", "arguments": {"command": "ls"}}),
            }),
        );
        assert_eq!(after_end.activity, 0);
        assert!(
            matches!(after_end.items.first(), Some(ChatItem::Tool { .. })),
            "the arguments landed in the tool item: {:?}",
            after_end.items
        );
    }

    /// The arguments arrive as text that the tool item has no place for (the
    /// item keeps the object `toolcall_end`/`tool_execution_start` carries), so
    /// a toolcall delta is still only counted, exactly as it was before L4.
    #[test]
    fn a_toolcall_delta_lands_nowhere_and_still_adds_one() {
        let before = before();
        assert_bumps(
            &before,
            update(AssistantMessageEvent::ToolcallDelta {
                content_index: 1,
                delta: "{}".to_owned(),
            }),
        );
    }

    #[test]
    fn tool_execution_events_that_reach_a_tool_item_do_not_add_to_activity() {
        let before = before();
        let started = reduce_chat(
            Language::En,
            before,
            &RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(json!({"command": "ls"})),
            },
        );
        assert_eq!(started.activity, 0);
        let updated = reduce_chat(
            Language::En,
            started,
            &RpcEvent::ToolExecutionUpdate {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: None,
                partial_result: Some(json!({"content": [{"type": "text", "text": "a\n"}]})),
            },
        );
        assert_eq!(updated.activity, 0);
        let ended = reduce_chat(
            Language::En,
            updated,
            &RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                result: Some(json!({"content": [{"type": "text", "text": "a\n"}]})),
                is_error: false,
            },
        );
        assert_eq!(ended.activity, 0);
        assert_eq!(ended.items.len(), 2, "the tool item, then the bubble");
    }

    /// An event naming a tool call the transcript never heard of still only
    /// counts: nothing is invented for it.
    #[test]
    fn a_tool_execution_end_for_an_unknown_call_still_adds_one() {
        let before = before();
        assert_bumps(
            &before,
            RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_gone".to_owned(),
                tool_name: "bash".to_owned(),
                result: Some(json!({})),
                is_error: false,
            },
        );
    }

    /// A reasoning delta or end with no open block reaches no row, so it keeps
    /// counting exactly as it did before the transcript kept thinking.
    #[test]
    fn thinking_events_with_no_open_block_still_add_one() {
        let before = before();
        for event in [
            AssistantMessageEvent::ThinkingDelta {
                content_index: 0,
                delta: "...".to_owned(),
            },
            AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "...".to_owned(),
            },
        ] {
            assert_bumps(&before, update(event));
        }
    }

    #[test]
    fn text_start_and_text_end_do_not_bump_activity() {
        let before = before();
        for event in [
            AssistantMessageEvent::TextStart { content_index: 0 },
            AssistantMessageEvent::TextEnd {
                content_index: 0,
                content: "done".to_owned(),
            },
        ] {
            assert_does_not_bump(&before, update(event));
        }
        assert_eq!(
            reduce_chat(
                Language::En,
                before.clone(),
                &update(AssistantMessageEvent::TextStart { content_index: 0 })
            ),
            before
        );
    }
}

/// The one ordered transcript (L4): the thinking, the tool calls and the text
/// in the order the child produced them, each row carrying what its own card is
/// drawn from. `ChatItem::Text` keeps the message shape and its ids, so nothing
/// that only knows messages has to learn a new one.
mod transcript {
    use super::*;

    /// A thinking block the child finished, with no clock to time it by.
    fn thought(id: &str, text: &str) -> ChatItem {
        ChatItem::Thinking {
            id: id.to_owned(),
            text: text.to_owned(),
            streaming: false,
            started_at_ms: None,
            ended_at_ms: None,
            duration_ms: None,
        }
    }

    /// The turn `fixtures/prompt-turn.jsonl` records, now that nothing about it
    /// is thrown away: the thinking, the tool call with its command AND its
    /// result, and the final text, in the order the child sent them.
    #[test]
    fn a_prompt_turn_keeps_the_thinking_the_tool_call_and_the_final_text_in_order() {
        let state = replay(include_str!("fixtures/prompt-turn.jsonl"));
        assert_eq!(
            state.items,
            vec![
                thought("think-0", "Let me check the file first."),
                ChatItem::Tool {
                    id: "tool-call_abc123".to_owned(),
                    tool_call_id: "call_abc123".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(ToolPayload::new(&json!({"command": "cat README.md"}))),
                    // The running output the fixture also sent is gone: the
                    // result supersedes it, and a call keeps one output (R1).
                    partial_result: None,
                    result: Some(ToolPayload::new(&json!({
                        "content": [{"type": "text", "text": "# Gentle Shell\n"}]
                    }))),
                    is_error: false,
                    running: false,
                    started_at_ms: None,
                    ended_at_ms: None,
                    duration_ms: None,
                },
                ChatItem::Text(assistant(
                    "msg-0",
                    "The README describes Gentle Shell.",
                    false
                )),
            ]
        );
        // A reader that only knows messages still sees exactly the text.
        assert_eq!(
            state.messages(),
            vec![assistant(
                "msg-0",
                "The README describes Gentle Shell.",
                false
            )]
        );
        // One id scheme per shape, all of them deterministic (L4).
        assert_eq!(
            state.items.iter().map(ChatItem::id).collect::<Vec<_>>(),
            vec!["think-0", "tool-call_abc123", "msg-0"]
        );
        assert!(!state.working);
        assert!(state.pending_dialogs.is_empty());
        assert_eq!(state.last_error, None);
        // The toolcall delta's arguments text lands in no item (the tool item
        // keeps the object), so it is the one event the fixture still counts.
        assert_eq!(state.activity, 1);
    }

    /// The projection the three readers of the old `messages` field take is
    /// exactly the text rows, in order, with the ids they always had: thinking
    /// and tool rows never shift an id.
    #[test]
    fn the_text_projection_holds_exactly_the_messages_the_state_used_to_keep() {
        let mut state = ChatState::default();
        state.push_message(ChatRole::User, "hola".to_owned(), false);
        state = reducer_run_from(
            state,
            vec![
                assistant_start(),
                update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
                update(AssistantMessageEvent::ThinkingEnd {
                    content_index: 0,
                    content: "mulling it over".to_owned(),
                }),
                update(AssistantMessageEvent::ToolcallStart {
                    content_index: 1,
                    id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                }),
                text_delta("Hola."),
                assistant_end(json!([{"type": "text", "text": "Hola."}])),
            ],
        );
        assert_eq!(state.items.len(), 4, "got {:?}", state.items);
        assert_eq!(
            state.messages(),
            vec![user("msg-0", "hola"), assistant("msg-1", "Hola.", false)]
        );
    }

    /// A message the shell appends takes the next text id, whatever rows the
    /// transcript already holds.
    #[test]
    fn a_message_pushed_by_the_shell_takes_the_next_text_id() {
        let mut state = ChatState::default();
        state.push_message(ChatRole::User, "hola".to_owned(), false);
        state.items.push(thought("think-0", "hmm"));
        state.push_message(ChatRole::User, "otra".to_owned(), false);
        assert_eq!(
            state.messages(),
            vec![user("msg-0", "hola"), user("msg-1", "otra")]
        );
    }

    /// A reopened session's history replaces the whole transcript: it is text,
    /// and its ids are the ones `messages_from_history` assigned.
    #[test]
    fn a_history_reply_replaces_the_whole_transcript() {
        let mut state = ChatState::default();
        state.items.push(thought("think-0", "gone"));
        state.replace_messages(vec![
            user("msg-0", "stored"),
            assistant("msg-1", "kept", false),
        ]);
        assert_eq!(
            state.items,
            texts(vec![user("msg-0", "stored"), assistant("msg-1", "kept", false)])
        );
    }

    /// R3-1: the id counter resumes after the highest id the history kept, so a
    /// message pushed after a reopened session cannot take an id the history
    /// already shows: ids stay unique across the reload.
    #[test]
    fn a_history_reply_leaves_the_id_counter_after_the_ids_it_kept() {
        let mut state = ChatState::default();
        state = reducer_run_from(state, vec![assistant_start()]);
        assert_eq!(state.next_text_id, 1, "the first message took `msg-0`");

        state.replace_messages(vec![
            user("msg-0", "stored"),
            assistant("msg-1", "kept", false),
        ]);
        state.push_message(ChatRole::User, "otra".to_owned(), false);
        assert_eq!(
            state
                .messages()
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["msg-0", "msg-1", "msg-2"],
            "the pushed message follows the history instead of reusing its ids"
        );
    }

    /// A thinking block records when it started, when it ended and how long it
    /// took: the "Thought 1.2s" T3 draws.
    #[test]
    fn a_thinking_block_records_its_start_its_end_and_its_duration() {
        let state = reducer_run_at(vec![
            (assistant_start(), Some(1_000)),
            (
                update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
                Some(1_000),
            ),
            (
                update(AssistantMessageEvent::ThinkingDelta {
                    content_index: 0,
                    delta: "Let me look.".to_owned(),
                }),
                Some(1_100),
            ),
            (
                update(AssistantMessageEvent::ThinkingEnd {
                    content_index: 0,
                    content: "Let me look.".to_owned(),
                }),
                Some(1_200),
            ),
        ]);
        assert_eq!(
            state.items,
            vec![
                ChatItem::Thinking {
                    id: "think-0".to_owned(),
                    text: "Let me look.".to_owned(),
                    streaming: false,
                    started_at_ms: Some(1_000),
                    ended_at_ms: Some(1_200),
                    duration_ms: Some(200),
                },
                ChatItem::Text(assistant("msg-0", "", true)),
            ]
        );
    }

    /// No clock, no duration: `prompt-turn.jsonl` carries no timestamps, and a
    /// card must not invent the "Done 0.2s" it has no evidence for.
    #[test]
    fn with_no_clock_a_thinking_block_records_no_time() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
            update(AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "Let me look.".to_owned(),
            }),
        ]);
        assert_eq!(state.items[0], thought("think-0", "Let me look."));
    }

    /// The "Done 0.2s" of a tool card: the execution's own start and end, and
    /// the output it ended with - the running output it also reported is gone,
    /// because the result supersedes it (R1).
    #[test]
    fn a_tool_execution_records_its_start_its_end_and_its_duration() {
        let state = reducer_run_at(vec![
            (assistant_start(), Some(5_000)),
            (
                update(AssistantMessageEvent::ToolcallStart {
                    content_index: 0,
                    id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                }),
                Some(5_000),
            ),
            (
                update(AssistantMessageEvent::ToolcallEnd {
                    content_index: 0,
                    tool_call: json!({"id": "call_1", "name": "bash", "arguments": {"command": "ls"}}),
                }),
                Some(5_000),
            ),
            (
                RpcEvent::ToolExecutionStart {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(json!({"command": "ls"})),
                },
                Some(5_000),
            ),
            (
                RpcEvent::ToolExecutionUpdate {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: None,
                    partial_result: Some(json!({"content": [{"type": "text", "text": "a\n"}]})),
                },
                Some(5_100),
            ),
            (
                RpcEvent::ToolExecutionEnd {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    result: Some(json!({"content": [{"type": "text", "text": "a\nb\n"}]})),
                    is_error: false,
                },
                Some(5_200),
            ),
        ]);
        assert_eq!(
            state.items.first(),
            Some(&ChatItem::Tool {
                id: "tool-call_1".to_owned(),
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(ToolPayload::new(&json!({"command": "ls"}))),
                partial_result: None,
                result: Some(ToolPayload::new(
                    &json!({"content": [{"type": "text", "text": "a\nb\n"}]})
                )),
                is_error: false,
                running: false,
                started_at_ms: Some(5_000),
                ended_at_ms: Some(5_200),
                duration_ms: Some(200),
            })
        );
    }

    /// The "Running 4.0s" of the same card: started, not ended, no duration to
    /// claim yet.
    #[test]
    fn a_tool_call_that_has_not_ended_is_still_running_with_no_duration() {
        let state = reducer_run_at(vec![
            (assistant_start(), Some(9_000)),
            (
                RpcEvent::ToolExecutionStart {
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: Some(json!({"command": "sleep 4"})),
                },
                Some(9_000),
            ),
        ]);
        match &state.items[0] {
            ChatItem::Tool {
                running,
                started_at_ms,
                ended_at_ms,
                duration_ms,
                result,
                ..
            } => {
                assert!(running);
                assert_eq!(*started_at_ms, Some(9_000));
                assert_eq!(*ended_at_ms, None);
                assert_eq!(*duration_ms, None);
                assert_eq!(*result, None);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    /// The bubble `message_start` opens is empty, so it says nothing about where
    /// it belongs: the thinking and the tool call of the same message take its
    /// place, and the text that follows stays last.
    #[test]
    fn thinking_and_tool_calls_take_the_place_of_the_empty_bubble() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
            update(AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "hmm".to_owned(),
            }),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            text_delta("done"),
        ]);
        assert_eq!(state.items.len(), 3, "got {:?}", state.items);
        assert_eq!(state.items[0], thought("think-0", "hmm"));
        assert!(matches!(state.items[1], ChatItem::Tool { .. }));
        assert_eq!(state.items[2], ChatItem::Text(assistant("msg-0", "done", true)));
    }

    /// Each reasoning block of a turn is its own row, and the ids count them.
    #[test]
    fn a_second_reasoning_block_gets_the_next_id() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
            update(AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "first".to_owned(),
            }),
            update(AssistantMessageEvent::ThinkingStart { content_index: 1 }),
            update(AssistantMessageEvent::ThinkingEnd {
                content_index: 1,
                content: "second".to_owned(),
            }),
        ]);
        assert_eq!(
            state.items,
            vec![
                thought("think-0", "first"),
                thought("think-1", "second"),
                ChatItem::Text(assistant("msg-0", "", true)),
            ]
        );
    }

    /// Text that arrived before a tool call keeps its place: the bubble has
    /// something to say by then, so the tool call goes after it.
    #[test]
    fn text_that_arrived_before_a_tool_call_keeps_its_place() {        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Looking."),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
        ]);
        assert_eq!(
            state.items,
            vec![
                ChatItem::Text(assistant("msg-0", "Looking.", true)),
                ChatItem::Tool {
                    id: "tool-call_1".to_owned(),
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: None,
                    partial_result: None,
                    result: None,
                    is_error: false,
                    running: false,
                    started_at_ms: None,
                    ended_at_ms: None,
                    duration_ms: None,
                },
            ]
        );
    }
    /// An event that names no tool call must never touch a row it cannot
    /// identify (R3-001): with two calls in flight, the ending arguments of one
    /// could be written onto the other call's row, and the transcript would
    /// present a wrong attribution as fact. The row it means is unknown, so it
    /// reaches no row and only counts, exactly as an unknown id does.
    #[test]
    fn a_toolcall_end_with_no_id_touches_neither_of_the_calls_in_flight() {
        let announced = |id: &str, name: &str| ChatItem::Tool {
            id: format!("tool-{id}"),
            tool_call_id: id.to_owned(),
            tool_name: name.to_owned(),
            args: None,
            partial_result: None,
            result: None,
            is_error: false,
            running: false,
            started_at_ms: None,
            ended_at_ms: None,
            duration_ms: None,
        };
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 0,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_2".to_owned(),
                tool_name: "read".to_owned(),
            }),
            update(AssistantMessageEvent::ToolcallEnd {
                content_index: 1,
                tool_call: json!({"name": "read", "arguments": {"path": "a.txt"}}),
            }),
        ]);
        assert_eq!(
            state.items,
            vec![
                announced("call_1", "bash"),
                announced("call_2", "read"),
                ChatItem::Text(assistant("msg-0", "", true)),
            ],
            "neither call in flight may gain the arguments of an unnamed event"
        );
        assert_eq!(
            state.activity, 1,
            "the event lands in no row, so it counts: {:?}",
            state.items
        );
    }

    /// One lookup decides where an execution lands: the row the model's own
    /// `toolcall_start` already opened is filled in, never opened twice.
    #[test]
    fn a_tool_execution_fills_the_row_the_model_announced() {
        let state = reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 0,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(json!({"command": "ls"})),
            },
        ]);
        assert_eq!(
            state.items.len(),
            2,
            "one tool row and the open bubble: {:?}",
            state.items
        );
        assert_eq!(
            state.items[0],
            ChatItem::Tool {
                id: "tool-call_1".to_owned(),
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(ToolPayload::new(&json!({"command": "ls"}))),
                partial_result: None,
                result: None,
                is_error: false,
                running: true,
                started_at_ms: None,
                ended_at_ms: None,
                duration_ms: None,
            }
        );
    }

    /// Text the child said before a tool call stays where it was said, and
    /// `message_end` closes it from there: a tool row after it must not leave the
    /// bubble streaming forever.
    #[test]
    fn message_end_closes_text_that_a_tool_call_followed() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("Let me look."),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            assistant_end(json!([
                {"type": "text", "text": "Let me look."},
                {"type": "toolCall", "id": "call_1", "name": "bash", "arguments": {"command": "ls"}}
            ])),
        ]);
        assert_eq!(
            state.items,
            vec![
                ChatItem::Text(assistant("msg-0", "Let me look.", false)),
                ChatItem::Tool {
                    id: "tool-call_1".to_owned(),
                    tool_call_id: "call_1".to_owned(),
                    tool_name: "bash".to_owned(),
                    args: None,
                    partial_result: None,
                    result: None,
                    is_error: false,
                    running: false,
                    started_at_ms: None,
                    ended_at_ms: None,
                    duration_ms: None,
                },
            ]
        );
    }

    /// R3-002 of the transcript review: the pushed window is the tail of the
    /// TEXT projection, oldest first, so a reader can take it from the END of
    /// the transcript instead of walking all of it twice. A thinking row is not
    /// a text row, so the tail has to skip one on its way back; `limit` at the
    /// number of text rows is the whole projection, and `0` is nothing.
    #[test]
    fn the_last_text_rows_are_the_tail_of_the_text_projection_in_order() {
        let state = ChatState {
            items: vec![
                ChatItem::Text(assistant("msg-0", "first", false)),
                thought("think-0", "thinking"),
                ChatItem::Text(assistant("msg-1", "second", false)),
                thought("think-1", "thinking again"),
                ChatItem::Text(assistant("msg-2", "third", false)),
                thought("think-2", "a thought after the last text"),
            ],
            ..ChatState::default()
        };
        let ids = |limit: usize| -> Vec<&str> {
            state
                .last_text_items(limit)
                .map(|message| message.id.as_str())
                .collect()
        };

        assert_eq!(
            ids(2),
            vec!["msg-1", "msg-2"],
            "N < total is the tail of the text, still oldest first"
        );
        assert_eq!(
            ids(3),
            vec!["msg-0", "msg-1", "msg-2"],
            "N == total is the whole text projection"
        );
        assert_eq!(ids(0), Vec::<&str>::new(), "N == 0 is no row at all");
    }
}

/// R1: a tool row keeps its arguments and its result for the life of the chat,
/// so each string of a payload is bounded, the row says when one was cut, and
/// the running output goes as soon as the result that supersedes it arrives.
mod tool_payloads {
    use super::*;

    /// A tool call the model announced, whose execution ended with `result`.
    fn executed(args: Value, partial: Option<Value>, result: Value) -> ChatState {
        reducer_run(vec![
            assistant_start(),
            update(AssistantMessageEvent::ToolcallStart {
                content_index: 0,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            }),
            RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(args),
            },
            RpcEvent::ToolExecutionUpdate {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: None,
                partial_result: partial,
            },
            RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                result: Some(result),
                is_error: false,
            },
        ])
    }

    /// The output text of a tool row's result payload.
    fn output(state: &ChatState) -> (&str, bool) {
        let ChatItem::Tool { result, .. } = &state.items[0] else {
            panic!("no tool row: {:?}", state.items)
        };
        let payload = result.as_ref().expect("the result the tool ended with");
        let text = payload.value["content"][0]["text"]
            .as_str()
            .expect("the output text");
        (text, payload.truncated)
    }

    /// A large output is cut at the documented bound and the row says so, so a
    /// card can report the cut instead of presenting the head as the whole.
    #[test]
    fn a_large_tool_output_is_cut_to_the_bound_and_the_row_says_so() {
        let long = "x".repeat(TOOL_PAYLOAD_MAX_BYTES * 4);
        let state = executed(
            json!({"command": "cat big.log"}),
            None,
            json!({"content": [{"type": "text", "text": long}]}),
        );
        let (kept, truncated) = output(&state);
        assert!(
            truncated,
            "a cut payload must be flagged, not presented whole"
        );
        assert_eq!(kept.len(), TOOL_PAYLOAD_MAX_BYTES, "cut to the bound");
        assert!(kept.chars().all(|c| c == 'x'));
        assert!(
            state.items[0].payload_truncated(),
            "the row says a payload was cut: {:?}",
            state.items[0]
        );
    }

    /// The bound is where it says it is: a string at it is kept whole, and a
    /// short argument is never flagged.
    #[test]
    fn a_payload_string_at_the_bound_is_kept_whole() {
        let exact = "y".repeat(TOOL_PAYLOAD_MAX_BYTES);
        let state = executed(
            json!({"command": "cat exact.log"}),
            None,
            json!({"content": [{"type": "text", "text": exact}]}),
        );
        let (kept, truncated) = output(&state);
        assert!(!truncated);
        assert_eq!(kept.len(), TOOL_PAYLOAD_MAX_BYTES);
        let ChatItem::Tool { args, .. } = &state.items[0] else {
            panic!("no tool row")
        };
        assert_eq!(
            args.as_ref()
                .map(|payload| (&payload.value, payload.truncated)),
            Some((&json!({"command": "cat exact.log"}), false))
        );
        assert!(!state.items[0].payload_truncated());
    }

    /// A cut lands on a character boundary: half a character is not text.
    #[test]
    fn a_cut_payload_lands_on_a_character_boundary() {
        // `a` shifted the multi-byte characters by one, so the bound falls in
        // the middle of one of them.
        let long = format!("a{}", "é".repeat(TOOL_PAYLOAD_MAX_BYTES));
        let state = executed(
            json!({}),
            None,
            json!({"content": [{"type": "text", "text": long}]}),
        );
        let (kept, truncated) = output(&state);
        assert!(truncated);
        assert!(kept.len() < TOOL_PAYLOAD_MAX_BYTES);
        assert!(kept.starts_with('a'));
        assert!(kept.chars().all(|c| c == 'a' || c == 'é'));
    }

    /// R3-001 of the transcript review: the bound is on every string a payload
    /// RETAINS, and an object's key is one of them. A key longer than the bound
    /// is cut and flagged, so no payload can keep an arbitrarily large key while
    /// its `truncated` flag claims it holds the whole of a bounded payload.
    #[test]
    fn an_oversized_object_key_is_cut_to_the_bound_and_the_row_says_so() {
        let long_key = "k".repeat(TOOL_PAYLOAD_MAX_BYTES * 4);
        let mut fields = serde_json::Map::new();
        fields.insert(long_key, json!("the value the payload keeps"));
        let state = executed(json!({"command": "env"}), None, Value::Object(fields));
        let ChatItem::Tool { result, .. } = &state.items[0] else {
            panic!("no tool row: {:?}", state.items)
        };
        let payload = result.as_ref().expect("the result the tool ended with");
        assert!(
            payload.truncated,
            "a cut KEY is a cut: the payload must not claim to be whole"
        );

        let field = payload.value.as_object().expect("an object");
        let keys: Vec<&String> = field.keys().collect();
        assert_eq!(keys.len(), 1, "the one field is kept: {:?}", payload.value);
        let key: &String = keys[0];
        assert_eq!(
            key.len(),
            TOOL_PAYLOAD_MAX_BYTES,
            "the key is cut to the bound, like any other string"
        );
        assert!(key.chars().all(|c| c == 'k'));
        assert_eq!(
            field.get(key),
            Some(&json!("the value the payload keeps")),
            "the value under the cut key is kept"
        );
        assert!(
            state.items[0].payload_truncated(),
            "the row says a payload was cut: {:?}",
            state.items[0]
        );
    }

    /// R3-2 (the recorded R1-002 advisory is the same root cause): cutting an
    /// object's KEY to the bound can make it land on another key of the same
    /// object, and a JSON object keeps one value per key. Losing a FIELD is not
    /// the same as shortening a string, so the payload says so separately: a
    /// card must be able to tell the user something was dropped, not merely
    /// that something was shortened.
    #[test]
    fn a_cut_key_that_collides_with_another_cut_key_reports_the_dropped_field() {
        // The two keys differ only AFTER the bound, so cutting each one to it
        // makes them the same string.
        let prefix = "k".repeat(TOOL_PAYLOAD_MAX_BYTES);
        let mut fields = serde_json::Map::new();
        fields.insert(format!("{prefix}-first"), json!("the value the payload keeps"));
        fields.insert(
            format!("{prefix}-second"),
            json!("the value that would overwrite it"),
        );
        let state = executed(json!({"command": "env"}), None, Value::Object(fields));
        let ChatItem::Tool { result, .. } = &state.items[0] else {
            panic!("no tool row: {:?}", state.items)
        };
        let payload = result.as_ref().expect("the result the tool ended with");
        assert!(
            payload.truncated,
            "the keys were cut, so the payload is not the whole of what came"
        );
        assert!(
            payload.dropped,
            "a field was lost to the cut; `truncated` alone says only that something was shortened"
        );
        let kept = payload.value.as_object().expect("an object");
        assert_eq!(kept.len(), 1, "one key, one field: {:?}", payload.value);
        assert_eq!(
            kept.get(prefix.as_str()),
            Some(&json!("the value the payload keeps")),
            "the field already kept stays; the colliding one is the loss"
        );
        assert!(
            state.items[0].payload_dropped(),
            "the row says a field was dropped: {:?}",
            state.items[0]
        );
        assert!(
            state.items[0].payload_truncated(),
            "the shortening is still reported too: {:?}",
            state.items[0]
        );
    }

    /// The two losses stay distinguishable: a payload whose strings were only
    /// shortened kept every field, and its row must not claim one is missing.
    #[test]
    fn a_shortened_string_drops_no_field() {
        let long = "x".repeat(TOOL_PAYLOAD_MAX_BYTES * 2);
        let state = executed(
            json!({"command": "cat big.log"}),
            None,
            json!({"content": [{"type": "text", "text": long}]}),
        );
        let ChatItem::Tool { result, .. } = &state.items[0] else {
            panic!("no tool row: {:?}", state.items)
        };
        let payload = result.as_ref().expect("the result the tool ended with");
        assert!(payload.truncated);
        assert!(!payload.dropped, "nothing was lost, only shortened");
        assert!(
            !state.items[0].payload_dropped(),
            "the row claims a loss that did not happen"
        );
    }

    /// The output so far is redundant once the result arrives: the call keeps
    /// the one output that is final, and no second copy of it besides.
    #[test]
    fn the_running_output_is_dropped_once_the_result_arrives() {
        let state = executed(
            json!({"command": "ls"}),
            Some(json!({"content": [{"type": "text", "text": "a\n"}]})),
            json!({"content": [{"type": "text", "text": "a\nb\n"}]}),
        );
        assert_eq!(
            state.items.first(),
            Some(&ChatItem::Tool {
                id: "tool-call_1".to_owned(),
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                args: Some(ToolPayload::new(&json!({"command": "ls"}))),
                partial_result: None,
                result: Some(ToolPayload::new(
                    &json!({"content": [{"type": "text", "text": "a\nb\n"}]})
                )),
                is_error: false,
                running: false,
                started_at_ms: None,
                ended_at_ms: None,
                duration_ms: None,
            })
        );
    }
}

mod dialogs {
    use super::*;

    fn select() -> UiRequest {        UiRequest::Select {
            id: "dlg-1".to_owned(),
            title: "Allow?".to_owned(),
            options: vec!["Allow".to_owned(), "Block".to_owned()],
            timeout: Some(10000),
        }
    }

    #[test]
    fn a_select_request_pushes_a_select_dialog() {
        let state = reduce_chat(
            Language::En,
            ChatState::default(),
            &RpcEvent::UiRequest(select()),
        );
        assert_eq!(
            state,
            ChatState {
                pending_dialogs: vec![Dialog::Select {
                    id: "dlg-1".to_owned(),
                    title: "Allow?".to_owned(),
                    options: vec!["Allow".to_owned(), "Block".to_owned()],
                }],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn a_confirm_request_pushes_a_confirm_dialog_with_its_message() {
        let event = RpcEvent::UiRequest(UiRequest::Confirm {
            id: "dlg-2".to_owned(),
            title: "Clear session?".to_owned(),
            message: "All messages will be lost.".to_owned(),
            timeout: None,
        });
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &event),
            ChatState {
                pending_dialogs: vec![Dialog::Confirm {
                    id: "dlg-2".to_owned(),
                    title: "Clear session?".to_owned(),
                    message: "All messages will be lost.".to_owned(),
                }],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn an_input_request_pushes_an_input_dialog_with_its_placeholder() {
        let event = RpcEvent::UiRequest(UiRequest::Input {
            id: "dlg-3".to_owned(),
            title: "Enter a value".to_owned(),
            placeholder: Some("type something...".to_owned()),
            timeout: None,
        });
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &event),
            ChatState {
                pending_dialogs: vec![Dialog::Input {
                    id: "dlg-3".to_owned(),
                    title: "Enter a value".to_owned(),
                    placeholder: Some("type something...".to_owned()),
                }],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn an_editor_request_pushes_an_editor_dialog_with_its_prefill() {
        let event = RpcEvent::UiRequest(UiRequest::Editor {
            id: "dlg-4".to_owned(),
            title: "Edit".to_owned(),
            prefill: Some("line 1".to_owned()),
        });
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &event),
            ChatState {
                pending_dialogs: vec![Dialog::Editor {
                    id: "dlg-4".to_owned(),
                    title: "Edit".to_owned(),
                    prefill: Some("line 1".to_owned()),
                }],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn dialogs_queue_in_arrival_order() {
        let state = reducer_run(vec![
            RpcEvent::UiRequest(select()),
            RpcEvent::UiRequest(UiRequest::Editor {
                id: "dlg-4".to_owned(),
                title: "Edit".to_owned(),
                prefill: None,
            }),
        ]);
        assert_eq!(state.pending_dialogs.len(), 2);
        assert_eq!(
            state.pending_dialogs[0],
            Dialog::Select {
                id: "dlg-1".to_owned(),
                title: "Allow?".to_owned(),
                options: vec!["Allow".to_owned(), "Block".to_owned()],
            }
        );
        assert_eq!(
            state.pending_dialogs[1],
            Dialog::Editor {
                id: "dlg-4".to_owned(),
                title: "Edit".to_owned(),
                prefill: None,
            }
        );
    }

    #[test]
    fn fire_and_forget_requests_are_ignored() {
        let event = RpcEvent::UiRequest(UiRequest::FireAndForget {
            id: "dlg-5".to_owned(),
            method: "notify".to_owned(),
            raw: json!({"type": "extension_ui_request", "id": "dlg-5", "method": "notify", "message": "hi"}),
        });
        assert_eq!(
            reduce_chat(Language::En, ChatState::default(), &event),
            ChatState::default()
        );
    }
}

mod errors_and_unhandled_events {
    use super::*;

    const ERROR: &str = "Cannot read properties of undefined (reading 'foo')";

    #[test]
    fn extension_error_sets_last_error_and_touches_nothing_else() {
        let before = ChatState {
            items: texts(vec![assistant("msg-0", "", true)]),
            working: true,
            activity: 4,
            ..ChatState::default()
        };
        let state = reduce_chat(
            Language::En,
            before,
            &RpcEvent::ExtensionError {
                extension_path: "/ext.ts".to_owned(),
                event: "tool_call".to_owned(),
                error: ERROR.to_owned(),
            },
        );
        assert_eq!(
            state,
            ChatState {
                items: texts(vec![assistant("msg-0", "", true)]),
                working: true,
                activity: 4,
                last_error: Some(ERROR.to_owned()),
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn turn_events_and_command_responses_are_no_ops() {
        let before = ChatState {
            items: texts(vec![assistant("msg-0", "hi", false)]),
            working: true,
            activity: 2,
            ..ChatState::default()
        };
        assert_eq!(
            reduce_chat(Language::En, before.clone(), &RpcEvent::TurnStart),
            before
        );
        assert_eq!(
            reduce_chat(Language::En, before.clone(), &RpcEvent::TurnEnd),
            before
        );
        let response = RpcEvent::Response(RpcResponse {
            id: None,
            command: "prompt".to_owned(),
            outcome: Ok(json!({"disposition": "started"})),
        });
        assert_eq!(reduce_chat(Language::En, before.clone(), &response), before);
    }
}

mod fixtures {
    use super::*;

    #[test]
    fn a_turn_whose_assistant_message_only_calls_a_tool_keeps_the_tool_call_and_no_empty_message() {
        let state = replay(include_str!("fixtures/tool-only-message.jsonl"));
        assert_eq!(
            state.items,
            vec![ChatItem::Tool {
                id: "tool-call_1".to_owned(),
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                // `toolcall_end` carried the arguments; this fixture's
                // tool_execution events carry neither args nor a result.
                args: Some(ToolPayload::new(&json!({"command": "ls"}))),
                partial_result: None,
                result: None,
                is_error: false,
                running: false,
                started_at_ms: None,
                ended_at_ms: None,
                duration_ms: None,
            }]
        );
        assert!(state.messages().is_empty(), "got {:?}", state.messages());
        assert!(!state.working);
        assert!(state.pending_dialogs.is_empty());
        assert_eq!(state.last_error, None);
        // One toolcall delta lands in no item, so it is the whole count.
        assert_eq!(state.activity, 1);
    }

    #[test]
    fn a_select_dialog_mid_turn_surfaces_and_leaves_the_agent_working() {
        let state = replay(include_str!("fixtures/select-dialog.jsonl"));
        assert!(state.working);
        assert_eq!(
            state.pending_dialogs,
            vec![Dialog::Select {
                id: "dlg-1".to_owned(),
                title: "Allow dangerous command?".to_owned(),
                options: vec!["Allow".to_owned(), "Block".to_owned()],
            }]
        );
        assert_eq!(
            state.items,
            texts(vec![assistant("msg-0", "", true)])
        );
        assert_eq!(state.activity, 0);
        assert_eq!(state.last_error, None);
    }

    #[test]
    fn an_extension_error_settles_into_last_error_once_the_agent_settles() {
        let state = replay(include_str!("fixtures/extension-error.jsonl"));
        assert_eq!(
            state.last_error,
            Some("Cannot read properties of undefined (reading 'foo')".to_owned())
        );
        assert!(!state.working);
        assert!(state.items.is_empty());
        assert!(state.pending_dialogs.is_empty());
    }
}

mod model_errors {
    use super::*;

    /// The shape pi emits when the provider fails: an assistant message whose
    /// content is the provider's text, flagged by `stopReason: "error"`.
    fn failed_end(content: Value) -> RpcEvent {
        RpcEvent::MessageEnd(RpcMessage {
            role: assistant_role(),
            raw: json!({
                "role": "assistant",
                "content": content,
                "stopReason": "error",
                "errorMessage": "Claude quota exceeded — rate limit"
            }),
        })
    }

    #[test]
    fn a_failed_assistant_message_is_flagged_with_the_error_message() {
        let state = reducer_run(vec![
            assistant_start(),
            failed_end(json!([{"type": "text", "text": "Fable 5.1 requires usage credits."}])),
        ]);
        let mut expected = assistant("msg-0", "Fable 5.1 requires usage credits.", false);
        expected.error = Some("Claude quota exceeded — rate limit".to_owned());
        assert_eq!(state.messages(), vec![expected]);
    }

    #[test]
    fn a_failed_message_with_no_text_is_kept_not_dropped() {
        let state = reducer_run(vec![assistant_start(), failed_end(json!([]))]);
        let mut expected = assistant("msg-0", "", false);
        expected.error = Some("Claude quota exceeded — rate limit".to_owned());
        assert_eq!(state.messages(), vec![expected]);
    }

    #[test]
    fn a_normal_stop_reason_is_not_an_error() {
        let end = RpcEvent::MessageEnd(RpcMessage {
            role: assistant_role(),
            raw: json!({"role": "assistant", "content": [{"type": "text", "text": "hi"}], "stopReason": "stop"}),
        });
        let state = reducer_run(vec![assistant_start(), end]);
        assert_eq!(state.messages(), vec![assistant("msg-0", "hi", false)]);
    }

    #[test]
    fn an_error_without_a_message_still_flags_the_message() {
        let end = RpcEvent::MessageEnd(RpcMessage {
            role: assistant_role(),
            raw: json!({"role": "assistant", "content": [], "stopReason": "error"}),
        });
        let state = reducer_run(vec![assistant_start(), end]);
        assert_eq!(
            state.messages()[0].error.as_deref(),
            Some("unknown model error")
        );
    }

    /// The one text this fold words itself follows the language it was folded in
    /// (0.2.0, S3): the shell hands it in with the state, so a Spanish run reads
    /// the catalog's Spanish sentence and not the English one.
    #[test]
    fn an_error_without_a_message_is_worded_in_the_language_it_was_folded_in() {
        let end = RpcEvent::MessageEnd(RpcMessage {
            role: assistant_role(),
            raw: json!({"role": "assistant", "content": [], "stopReason": "error"}),
        });
        let opened = reduce_chat(Language::Es, ChatState::default(), &assistant_start());
        let state = reduce_chat(Language::Es, opened, &end);
        assert_eq!(
            state.messages()[0].error.as_deref(),
            Some("error desconocido del modelo")
        );
    }
}

/// The desktop RPC channel is control traffic: its widget never becomes part of
/// the conversation, and it never disturbs the helpers widget either.
mod desktop_rpc_widget {
    use super::*;

    const DESKTOP: &str = include_str!("fixtures/desktop-rpc.jsonl");
    const HELPERS: &str = include_str!("fixtures/helpers-activity.jsonl");

    fn first_event(raw: &str) -> RpcEvent {
        let line = raw
            .lines()
            .find(|line| !line.trim().is_empty())
            .expect("a fixture line");
        match decode_line(line) {
            Decoded::Event(event) => event,
            Decoded::Unknown(raw) => panic!("not an event: {raw}"),
        }
    }

    #[test]
    fn every_desktop_rpc_frame_leaves_the_chat_state_untouched() {
        let state = ChatState::default();
        for line in DESKTOP.lines().filter(|line| !line.trim().is_empty()) {
            let event = match decode_line(line) {
                Decoded::Event(event) => event,
                Decoded::Unknown(raw) => panic!("not an event: {raw}"),
            };
            assert_eq!(
                reduce_chat(Language::En, state.clone(), &event),
                state,
                "the desktop channel is control traffic: {line}"
            );
        }
    }

    #[test]
    fn a_desktop_frame_does_not_clear_the_helpers_widget() {
        let with_helpers = reduce_chat(Language::En, ChatState::default(), &first_event(HELPERS));
        assert!(
            !with_helpers.helpers.tasks.is_empty(),
            "the helpers fixture must fold"
        );

        let clearing = DESKTOP
            .lines()
            .rfind(|line| !line.trim().is_empty())
            .expect("the clearing frame");
        let event = match decode_line(clearing) {
            Decoded::Event(event) => event,
            Decoded::Unknown(raw) => panic!("not an event: {raw}"),
        };
        assert_eq!(
            reduce_chat(Language::En, with_helpers.clone(), &event),
            with_helpers
        );
    }
}
