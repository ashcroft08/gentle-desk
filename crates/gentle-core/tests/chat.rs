//! Behaviour tests for the chat reducer, ported from the reference app's
//! `chatReducer.test.ts` and replayed against the recorded RPC fixtures.
//!
//! Every test asserts the whole observable state, so a rule that leaks into a
//! field it should not touch fails here too.

use gentle_core::chat::{reduce_chat, ChatMessage, ChatRole, ChatState, Dialog};
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
    }
}

fn user(id: &str, text: &str) -> ChatMessage {
    ChatMessage {
        id: id.to_owned(),
        role: ChatRole::User,
        text: text.to_owned(),
        streaming: false,
    }
}

fn reducer_run(events: Vec<RpcEvent>) -> ChatState {
    events
        .into_iter()
        .fold(ChatState::default(), |state, event| {
            reduce_chat(state, &event)
        })
}

mod working {
    use super::*;

    #[test]
    fn agent_start_sets_working_true() {
        assert_eq!(
            reduce_chat(ChatState::default(), &RpcEvent::AgentStart),
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
            reduce_chat(working, &RpcEvent::AgentEnd { will_retry: None }),
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
            reduce_chat(working, &RpcEvent::AgentSettled),
            ChatState::default()
        );
    }
}

mod assistant_messages {
    use super::*;

    #[test]
    fn message_start_for_an_assistant_message_opens_an_empty_streaming_message() {
        let state = reduce_chat(ChatState::default(), &assistant_start());
        assert_eq!(
            state,
            ChatState {
                messages: vec![assistant("msg-0", "", true)],
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
            reduce_chat(ChatState::default(), &event),
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
                messages: vec![assistant("msg-0", "Hello world", true)],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn text_delta_with_no_open_assistant_message_is_a_no_op() {
        assert_eq!(
            reduce_chat(ChatState::default(), &text_delta("orphaned")),
            ChatState::default()
        );
    }

    #[test]
    fn text_delta_does_not_append_to_a_user_message() {
        let state = ChatState {
            messages: vec![user("msg-0", "hi")],
            ..ChatState::default()
        };
        let state = reduce_chat(state, &text_delta("orphaned"));
        assert_eq!(
            state,
            ChatState {
                messages: vec![user("msg-0", "hi")],
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
                messages: vec![assistant("msg-0", "Hello world", false)],
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
                messages: vec![assistant("msg-0", "kept", false)],
                ..ChatState::default()
            }
        );
    }

    #[test]
    fn a_tool_only_assistant_message_stays_present_while_streaming() {
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
                messages: vec![assistant("msg-0", "", true)],
                activity: 1,
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
        assert!(state.messages.is_empty(), "got {:?}", state.messages);
        assert_eq!(state.activity, 2);
    }

    #[test]
    fn message_end_drops_an_assistant_message_whose_authoritative_text_is_only_whitespace() {
        let state = reducer_run(vec![
            assistant_start(),
            assistant_end(json!([{"type": "text", "text": "   "}])),
        ]);
        assert!(state.messages.is_empty(), "got {:?}", state.messages);
    }

    #[test]
    fn message_end_drops_an_assistant_message_whose_accumulated_text_is_only_whitespace() {
        let state = reducer_run(vec![
            assistant_start(),
            text_delta("  \n\t"),
            assistant_end_without_content(),
        ]);
        assert!(state.messages.is_empty(), "got {:?}", state.messages);
    }

    #[test]
    fn a_dropped_message_frees_its_id_so_the_next_message_stays_contiguous() {
        let mut state = ChatState {
            messages: vec![user("msg-0", "hi")],
            ..ChatState::default()
        };
        state = reduce_chat(state, &assistant_start());
        assert_eq!(state.messages[1].id, "msg-1");

        state = reduce_chat(state, &assistant_end(json!([])));
        assert_eq!(state.messages, vec![user("msg-0", "hi")]);

        state = reduce_chat(state, &assistant_start());
        assert_eq!(state.messages.len(), 2);
        assert_eq!(state.messages[1].id, "msg-1");
        assert_eq!(state.messages[1].text, "");
        assert!(state.messages[1].streaming);
    }

    #[test]
    fn message_end_for_a_non_assistant_message_is_a_no_op() {
        let before = ChatState {
            messages: vec![assistant("msg-0", "text", true)],
            ..ChatState::default()
        };
        let event = RpcEvent::MessageEnd(RpcMessage {
            role: "toolResult".to_owned(),
            raw: json!({"role": "toolResult", "content": [{"type": "text", "text": "out"}]}),
        });
        assert_eq!(reduce_chat(before.clone(), &event), before);
    }

    #[test]
    fn message_end_with_no_open_assistant_message_is_a_no_op() {
        let before = ChatState {
            messages: vec![user("msg-0", "hi")],
            ..ChatState::default()
        };
        assert_eq!(
            reduce_chat(
                before.clone(),
                &assistant_end(json!([{"type": "text", "text": "late"}]))
            ),
            before
        );
    }
}

mod activity {
    use super::*;

    fn before() -> ChatState {
        ChatState {
            messages: vec![assistant("msg-0", "", true)],
            ..ChatState::default()
        }
    }

    fn assert_bumps(before: &ChatState, event: RpcEvent) {
        let after = reduce_chat(before.clone(), &event);
        assert_eq!(after.activity, before.activity + 1);
        assert_eq!(after.messages, before.messages);
    }

    #[test]
    fn thinking_and_toolcall_deltas_each_add_one() {
        let before = before();
        for event in [
            AssistantMessageEvent::ThinkingStart { content_index: 0 },
            AssistantMessageEvent::ThinkingDelta {
                content_index: 0,
                delta: "...".to_owned(),
            },
            AssistantMessageEvent::ThinkingEnd {
                content_index: 0,
                content: "...".to_owned(),
            },
            AssistantMessageEvent::ToolcallStart {
                content_index: 1,
                id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            },
            AssistantMessageEvent::ToolcallDelta {
                content_index: 1,
                delta: "{}".to_owned(),
            },
            AssistantMessageEvent::ToolcallEnd {
                content_index: 1,
                tool_call: json!({}),
            },
        ] {
            assert_bumps(&before, update(event));
        }
    }

    #[test]
    fn tool_execution_events_each_add_one() {
        let before = before();
        assert_bumps(
            &before,
            RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            },
        );
        assert_bumps(
            &before,
            RpcEvent::ToolExecutionUpdate {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            },
        );
        assert_bumps(
            &before,
            RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                is_error: false,
            },
        );
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
            assert_eq!(reduce_chat(before.clone(), &update(event)), before);
        }
    }

    #[test]
    fn activity_accumulates_across_events() {
        let state = reducer_run(vec![
            RpcEvent::ToolExecutionStart {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
            },
            RpcEvent::ToolExecutionEnd {
                tool_call_id: "call_1".to_owned(),
                tool_name: "bash".to_owned(),
                is_error: true,
            },
            update(AssistantMessageEvent::ThinkingStart { content_index: 0 }),
        ]);
        assert_eq!(state.activity, 3);
        assert!(state.messages.is_empty());
    }
}

mod dialogs {
    use super::*;

    fn select() -> UiRequest {
        UiRequest::Select {
            id: "dlg-1".to_owned(),
            title: "Allow?".to_owned(),
            options: vec!["Allow".to_owned(), "Block".to_owned()],
            timeout: Some(10000),
        }
    }

    #[test]
    fn a_select_request_pushes_a_select_dialog() {
        let state = reduce_chat(ChatState::default(), &RpcEvent::UiRequest(select()));
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
            reduce_chat(ChatState::default(), &event),
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
            reduce_chat(ChatState::default(), &event),
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
            reduce_chat(ChatState::default(), &event),
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
            reduce_chat(ChatState::default(), &event),
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
            messages: vec![assistant("msg-0", "", true)],
            working: true,
            activity: 4,
            ..ChatState::default()
        };
        let state = reduce_chat(
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
                messages: vec![assistant("msg-0", "", true)],
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
            messages: vec![assistant("msg-0", "hi", false)],
            working: true,
            activity: 2,
            ..ChatState::default()
        };
        assert_eq!(reduce_chat(before.clone(), &RpcEvent::TurnStart), before);
        assert_eq!(reduce_chat(before.clone(), &RpcEvent::TurnEnd), before);
        let response = RpcEvent::Response(RpcResponse {
            id: None,
            command: "prompt".to_owned(),
            outcome: Ok(json!({"disposition": "started"})),
        });
        assert_eq!(reduce_chat(before.clone(), &response), before);
    }
}

mod fixtures {
    use super::*;

    fn replay(raw: &str) -> ChatState {
        raw.lines().filter(|line| !line.trim().is_empty()).fold(
            ChatState::default(),
            |state, line| match decode_line(line) {
                Decoded::Event(event) => reduce_chat(state, &event),
                Decoded::Unknown(raw) => panic!("fixture line decoded as unknown: {raw}"),
            },
        )
    }

    #[test]
    fn a_prompt_turn_with_thinking_a_tool_call_and_streamed_text_ends_with_only_the_final_text() {
        let state = replay(include_str!("fixtures/prompt-turn.jsonl"));
        assert_eq!(
            state.messages,
            vec![assistant(
                "msg-0",
                "The README describes Gentle Shell.",
                false
            )]
        );
        assert!(!state.working);
        assert!(state.pending_dialogs.is_empty());
        assert_eq!(state.last_error, None);
        // 3 thinking + 3 toolcall + 3 tool_execution events.
        assert_eq!(state.activity, 9);
    }

    #[test]
    fn a_turn_whose_assistant_message_only_calls_a_tool_leaves_no_empty_message() {
        let state = replay(include_str!("fixtures/tool-only-message.jsonl"));
        assert!(state.messages.is_empty(), "got {:?}", state.messages);
        assert!(!state.working);
        assert!(state.pending_dialogs.is_empty());
        assert_eq!(state.last_error, None);
        // 3 toolcall + 2 tool_execution events.
        assert_eq!(state.activity, 5);
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
        assert_eq!(state.messages, vec![assistant("msg-0", "", true)]);
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
        assert!(state.messages.is_empty());
        assert!(state.pending_dialogs.is_empty());
    }
}
