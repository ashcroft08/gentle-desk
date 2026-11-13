use gentle_core::rpc::*;
use serde_json::json;

fn event(line: &str) -> RpcEvent {
    match decode_line(line) {
        Decoded::Event(event) => event,
        Decoded::Unknown(raw) => panic!("expected an event, got unknown: {raw}"),
    }
}

fn is_unknown(line: &str) -> bool {
    matches!(decode_line(line), Decoded::Unknown(_))
}

mod encode {
    use super::*;

    #[test]
    fn prompt_is_one_lf_terminated_json_line() {
        let line = encode_command(&RpcCommand::Prompt {
            message: "hi\nthere".into(),
        });
        assert_eq!(line, "{\"message\":\"hi\\nthere\",\"type\":\"prompt\"}\n");
        assert_eq!(line.matches('\n').count(), 1);
    }

    /// One image in pi's own shape: the content block an attachment is.
    fn image(mime: &str, data: &str) -> ImageContent {
        ImageContent {
            mime_type: mime.to_owned(),
            data: data.to_owned(),
        }
    }

    #[test]
    fn a_prompt_with_images_carries_pis_images_field() {
        let line = encode_command(&RpcCommand::prompt_with_images(
            "look at this",
            vec![image("image/png", "aGk="), image("image/jpeg", "AA==")],
        ));

        assert_eq!(
            line,
            concat!(
                "{\"images\":[{\"data\":\"aGk=\",\"mimeType\":\"image/png\",\"type\":\"image\"},",
                "{\"data\":\"AA==\",\"mimeType\":\"image/jpeg\",\"type\":\"image\"}],",
                "\"message\":\"look at this\",\"type\":\"prompt\"}\n"
            )
        );
        assert_eq!(line.matches('\n').count(), 1);
    }

    #[test]
    fn a_prompt_with_no_images_is_byte_identical_to_the_old_line() {
        let old = encode_command(&RpcCommand::Prompt {
            message: "hi\nthere".into(),
        });
        let built = encode_command(&RpcCommand::prompt("hi\nthere"));
        let empty = encode_command(&RpcCommand::prompt_with_images("hi\nthere", Vec::new()));

        assert_eq!(built, old, "the constructor sends the old line");
        assert_eq!(empty, old, "no images is the old line, byte for byte");
        assert!(
            !empty.contains("images"),
            "the field is omitted, never sent empty: {empty}"
        );
    }

    #[test]
    fn simple_commands_carry_no_extra_fields() {
        assert_eq!(encode_command(&RpcCommand::Abort), "{\"type\":\"abort\"}\n");
        assert_eq!(
            encode_command(&RpcCommand::NewSession),
            "{\"type\":\"new_session\"}\n"
        );
        assert_eq!(
            encode_command(&RpcCommand::GetState),
            "{\"type\":\"get_state\"}\n"
        );
        assert_eq!(
            encode_command(&RpcCommand::GetMessages { id: None }),
            "{\"type\":\"get_messages\"}\n"
        );
    }

    /// pi's model commands, byte for byte (rpc-types.d.ts:44-58, 230-243).
    #[test]
    fn model_commands_carry_pis_exact_fields() {
        assert_eq!(
            encode_command(&RpcCommand::GetAvailableModels),
            "{\"type\":\"get_available_models\"}\n"
        );
        assert_eq!(
            encode_command(&RpcCommand::SetModel {
                provider: "anthropic".into(),
                model_id: "claude-opus-4-5".into(),
            }),
            "{\"modelId\":\"claude-opus-4-5\",\"provider\":\"anthropic\",\"type\":\"set_model\"}\n"
        );
        assert_eq!(
            encode_command(&RpcCommand::SetThinkingLevel {
                level: "high".into()
            }),
            "{\"level\":\"high\",\"type\":\"set_thinking_level\"}\n"
        );
        assert_eq!(
            encode_command(&RpcCommand::GetAvailableThinkingLevels),
            "{\"type\":\"get_available_thinking_levels\"}\n"
        );
    }

    /// pi's own ask for the session's statistics (rpc-mode.js:488-490): the
    /// one request whose reply carries `contextUsage`.
    #[test]
    fn the_session_stats_ask_is_pis_own_line() {
        assert_eq!(
            encode_command(&RpcCommand::GetSessionStats),
            "{\"type\":\"get_session_stats\"}\n"
        );
    }

    #[test]
    fn get_messages_and_switch_session_carry_their_fields() {
        let with_id = encode_command(&RpcCommand::GetMessages {
            id: Some("h1".into()),
        });
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&with_id).unwrap(),
            json!({"type": "get_messages", "id": "h1"})
        );
        let switch = encode_command(&RpcCommand::SwitchSession {
            session_path: "C:\\s\\a.jsonl".into(),
        });
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&switch).unwrap(),
            json!({"type": "switch_session", "sessionPath": "C:\\s\\a.jsonl"})
        );
    }

    #[test]
    fn each_ui_response_variant() {
        let encode = |answer| {
            let line = encode_command(&RpcCommand::UiResponse {
                id: "d1".into(),
                answer,
            });
            serde_json::from_str::<serde_json::Value>(&line).unwrap()
        };
        assert_eq!(
            encode(UiAnswer::Value("Allow".into())),
            json!({"type": "extension_ui_response", "id": "d1", "value": "Allow"})
        );
        assert_eq!(
            encode(UiAnswer::Confirmed(true)),
            json!({"type": "extension_ui_response", "id": "d1", "confirmed": true})
        );
        assert_eq!(
            encode(UiAnswer::Cancelled),
            json!({"type": "extension_ui_response", "id": "d1", "cancelled": true})
        );
    }
}

mod line_splitter {
    use super::*;

    #[test]
    fn splits_a_chunk_with_two_complete_lines() {
        let mut splitter = LineSplitter::new();
        assert_eq!(splitter.push(b"a\nb\n"), vec!["a", "b"]);
    }

    #[test]
    fn buffers_a_partial_line_across_pushes() {
        let mut splitter = LineSplitter::new();
        assert!(splitter.push(b"hel").is_empty());
        assert_eq!(splitter.push(b"lo\nwor"), vec!["hello"]);
        assert_eq!(splitter.push(b"ld\n"), vec!["world"]);
    }

    #[test]
    fn strips_a_trailing_carriage_return() {
        let mut splitter = LineSplitter::new();
        assert_eq!(splitter.push(b"a\r\nb\r\n"), vec!["a", "b"]);
    }

    #[test]
    fn does_not_split_on_unicode_line_separators() {
        let mut splitter = LineSplitter::new();
        assert_eq!(
            splitter.push("a\u{2028}b\u{2029}c\n".as_bytes()),
            vec!["a\u{2028}b\u{2029}c"]
        );
    }

    #[test]
    fn keeps_a_multibyte_character_cut_across_chunks_intact() {
        let bytes = "ñ\n".as_bytes();
        let mut splitter = LineSplitter::new();
        assert!(splitter.push(&bytes[..1]).is_empty());
        assert_eq!(splitter.push(&bytes[1..]), vec!["ñ"]);
    }

    #[test]
    fn flush_returns_the_remainder_once_and_empty_flush_returns_nothing() {
        let mut splitter = LineSplitter::new();
        assert!(splitter.flush().is_empty());
        splitter.push(b"tail\r");
        assert_eq!(splitter.flush(), vec!["tail"]);
        assert!(splitter.flush().is_empty());
    }
}

mod decode {
    use super::*;

    #[test]
    fn blank_malformed_and_untyped_lines_are_unknown() {
        assert!(is_unknown(""));
        assert!(is_unknown("   "));
        assert!(is_unknown("{not json"));
        assert!(is_unknown("[1,2]"));
        assert!(is_unknown("{\"no\":\"type\"}"));
        assert!(is_unknown("{\"type\":7}"));
    }

    #[test]
    fn unmodeled_event_types_are_unknown_and_keep_the_raw_line() {
        let line = "{\"type\":\"compaction_start\"}";
        assert_eq!(decode_line(line), Decoded::Unknown(line.to_owned()));
    }

    #[test]
    fn lifecycle_events() {
        assert_eq!(event("{\"type\":\"agent_start\"}"), RpcEvent::AgentStart);
        assert_eq!(
            event("{\"type\":\"agent_settled\"}"),
            RpcEvent::AgentSettled
        );
        assert_eq!(event("{\"type\":\"turn_start\"}"), RpcEvent::TurnStart);
        assert_eq!(event("{\"type\":\"turn_end\"}"), RpcEvent::TurnEnd);
        assert_eq!(
            event("{\"type\":\"agent_end\"}"),
            RpcEvent::AgentEnd { will_retry: None }
        );
        assert_eq!(
            event("{\"type\":\"agent_end\",\"willRetry\":true}"),
            RpcEvent::AgentEnd {
                will_retry: Some(true)
            }
        );
    }

    #[test]
    fn message_start_and_end_keep_role_and_raw() {
        match event(
            "{\"type\":\"message_start\",\"message\":{\"role\":\"assistant\",\"content\":[]}}",
        ) {
            RpcEvent::MessageStart(message) => {
                assert_eq!(message.role, "assistant");
                assert_eq!(message.raw["content"], json!([]));
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(is_unknown(
            "{\"type\":\"message_end\",\"message\":{\"content\":[]}}"
        ));
    }

    #[test]
    fn message_update_text_delta() {
        let line = "{\"type\":\"message_update\",\"assistantMessageEvent\":{\"type\":\"text_delta\",\"contentIndex\":0,\"delta\":\"Hi\"}}";
        assert_eq!(
            event(line),
            RpcEvent::MessageUpdate(AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: "Hi".into()
            })
        );
    }

    #[test]
    fn tool_execution_events() {
        assert_eq!(
            event(
                "{\"type\":\"tool_execution_start\",\"toolCallId\":\"t1\",\"toolName\":\"bash\"}"
            ),
            RpcEvent::ToolExecutionStart {
                tool_call_id: "t1".into(),
                tool_name: "bash".into()
            }
        );
        assert_eq!(
            event("{\"type\":\"tool_execution_end\",\"toolCallId\":\"t1\",\"toolName\":\"bash\",\"isError\":true}"),
            RpcEvent::ToolExecutionEnd { tool_call_id: "t1".into(), tool_name: "bash".into(), is_error: true }
        );
        assert!(is_unknown(
            "{\"type\":\"tool_execution_start\",\"toolName\":\"bash\"}"
        ));
    }

    #[test]
    fn select_dialog_and_fire_and_forget_requests() {
        assert_eq!(
            event("{\"type\":\"extension_ui_request\",\"id\":\"d1\",\"method\":\"select\",\"title\":\"Allow?\",\"options\":[\"Allow\",\"Block\"],\"timeout\":10000}"),
            RpcEvent::UiRequest(UiRequest::Select {
                id: "d1".into(),
                title: "Allow?".into(),
                options: vec!["Allow".into(), "Block".into()],
                timeout: Some(10000),
            })
        );
        match event("{\"type\":\"extension_ui_request\",\"id\":\"n1\",\"method\":\"notify\",\"message\":\"hi\"}") {
            RpcEvent::UiRequest(UiRequest::FireAndForget { id, method, raw }) => {
                assert_eq!((id.as_str(), method.as_str()), ("n1", "notify"));
                assert_eq!(raw["message"], "hi");
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(is_unknown(
            "{\"type\":\"extension_ui_request\",\"id\":\"x\",\"method\":\"mystery\"}"
        ));
    }

    #[test]
    fn extension_error() {
        assert_eq!(
            event("{\"type\":\"extension_error\",\"extensionPath\":\"/e.js\",\"event\":\"tool_call\",\"error\":\"boom\"}"),
            RpcEvent::ExtensionError { extension_path: "/e.js".into(), event: "tool_call".into(), error: "boom".into() }
        );
    }

    #[test]
    fn response_envelopes() {
        assert_eq!(
            event("{\"type\":\"response\",\"command\":\"prompt\",\"success\":true,\"data\":{\"disposition\":\"started\"}}"),
            RpcEvent::Response(RpcResponse {
                id: None,
                command: "prompt".into(),
                outcome: Ok(json!({"disposition": "started"})),
            })
        );
        assert_eq!(
            event("{\"type\":\"response\",\"id\":\"h1\",\"command\":\"get_messages\",\"success\":false,\"error\":\"nope\"}"),
            RpcEvent::Response(RpcResponse { id: Some("h1".into()), command: "get_messages".into(), outcome: Err("nope".into()) })
        );
    }
}

mod fixtures {
    use super::*;

    const FIXTURES: [(&str, &str); 5] = [
        (
            "extension-error",
            include_str!("fixtures/extension-error.jsonl"),
        ),
        (
            "helpers-activity",
            include_str!("fixtures/helpers-activity.jsonl"),
        ),
        ("prompt-turn", include_str!("fixtures/prompt-turn.jsonl")),
        (
            "select-dialog",
            include_str!("fixtures/select-dialog.jsonl"),
        ),
        (
            "tool-only-message",
            include_str!("fixtures/tool-only-message.jsonl"),
        ),
    ];

    #[test]
    fn every_recorded_line_decodes_to_an_event() {
        let mut total = 0;
        for (name, raw) in FIXTURES {
            for (index, line) in raw
                .lines()
                .filter(|line| !line.trim().is_empty())
                .enumerate()
            {
                total += 1;
                assert!(
                    matches!(decode_line(line), Decoded::Event(_)),
                    "{name}:{} fell back to unknown: {line}",
                    index + 1
                );
            }
        }
        assert_eq!(
            total, 41,
            "fixture line count changed; update this guard together with the fixtures"
        );
    }
}
