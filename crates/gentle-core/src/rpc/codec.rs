//! Encoding of commands and decoding of event lines.

use serde_json::{json, Map, Value};

use super::types::*;

/// One JSON command line, LF-terminated per the rpc framing rules.
pub fn encode_command(command: &RpcCommand) -> String {
    let value = match command {
        RpcCommand::Prompt { message } => json!({ "type": "prompt", "message": message }),
        // The `images` key is written only when there is something in it: a
        // prompt with nothing attached must stay the line every earlier prompt
        // was, byte for byte.
        RpcCommand::PromptWithImages { message, images } => {
            let mut map = Map::new();
            map.insert("type".into(), json!("prompt"));
            map.insert("message".into(), json!(message));
            if !images.is_empty() {
                map.insert(
                    "images".into(),
                    Value::Array(images.iter().map(image_content).collect()),
                );
            }
            Value::Object(map)
        }
        RpcCommand::Abort => json!({ "type": "abort" }),
        RpcCommand::NewSession => json!({ "type": "new_session" }),
        RpcCommand::SwitchSession { session_path } => {
            json!({ "type": "switch_session", "sessionPath": session_path })
        }
        RpcCommand::GetState => json!({ "type": "get_state" }),
        RpcCommand::GetMessages { id: None } => json!({ "type": "get_messages" }),
        RpcCommand::GetMessages { id: Some(id) } => json!({ "type": "get_messages", "id": id }),
        RpcCommand::GetAvailableModels => json!({ "type": "get_available_models" }),
        RpcCommand::SetModel { provider, model_id } => {
            json!({ "type": "set_model", "provider": provider, "modelId": model_id })
        }
        RpcCommand::SetThinkingLevel { level } => {
            json!({ "type": "set_thinking_level", "level": level })
        }
        RpcCommand::GetAvailableThinkingLevels => {
            json!({ "type": "get_available_thinking_levels" })
        }
        RpcCommand::UiResponse { id, answer } => {
            let mut map = Map::new();
            map.insert("type".into(), json!("extension_ui_response"));
            map.insert("id".into(), json!(id));
            match answer {
                UiAnswer::Value(value) => map.insert("value".into(), json!(value)),
                UiAnswer::Confirmed(confirmed) => map.insert("confirmed".into(), json!(confirmed)),
                UiAnswer::Cancelled => map.insert("cancelled".into(), json!(true)),
            };
            Value::Object(map)
        }
    };
    format!("{value}\n")
}

/// One image in pi's own content shape: an `image` content block.
fn image_content(image: &ImageContent) -> Value {
    json!({
        "type": "image",
        "data": image.data,
        "mimeType": image.mime_type,
    })
}

/// Splits raw output chunks into complete lines.
///
/// Splits on `\n` only (never on U+2028/U+2029, which are valid inside JSON
/// strings) and strips an optional trailing `\r`. It works on bytes, so a
/// multi-byte character cut across two chunks is never corrupted: `\n` cannot
/// occur inside a UTF-8 sequence.
#[derive(Debug, Default)]
pub struct LineSplitter {
    buffer: Vec<u8>,
}

impl LineSplitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a chunk, get back every complete line it produced.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);
        let mut lines = Vec::new();
        while let Some(newline) = self.buffer.iter().position(|&byte| byte == b'\n') {
            let rest = self.buffer.split_off(newline + 1);
            let mut line = std::mem::replace(&mut self.buffer, rest);
            line.pop(); // the '\n'
            lines.push(into_line(line));
        }
        lines
    }

    /// Flush whatever is left (e.g. on stream end) as one final line, if non-empty.
    pub fn flush(&mut self) -> Vec<String> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        vec![into_line(std::mem::take(&mut self.buffer))]
    }
}

fn into_line(mut bytes: Vec<u8>) -> String {
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Decodes one JSONL line. Never panics: a blank line, malformed JSON, or a
/// valid event this client does not model becomes `Decoded::Unknown`.
pub fn decode_line(line: &str) -> Decoded {
    if line.trim().is_empty() {
        return Decoded::Unknown(line.to_owned());
    }
    let Ok(Value::Object(parsed)) = serde_json::from_str::<Value>(line) else {
        return Decoded::Unknown(line.to_owned());
    };
    match parsed
        .get("type")
        .and_then(Value::as_str)
        .and_then(|kind| to_event(kind, &parsed))
    {
        Some(event) => Decoded::Event(event),
        None => Decoded::Unknown(line.to_owned()),
    }
}

fn string(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number(map: &Map<String, Value>, key: &str) -> Option<u64> {
    map.get(key).and_then(Value::as_u64)
}

fn to_event(kind: &str, parsed: &Map<String, Value>) -> Option<RpcEvent> {
    match kind {
        "agent_start" => Some(RpcEvent::AgentStart),
        "agent_settled" => Some(RpcEvent::AgentSettled),
        "turn_start" => Some(RpcEvent::TurnStart),
        "turn_end" => Some(RpcEvent::TurnEnd),
        "agent_end" => Some(RpcEvent::AgentEnd {
            will_retry: parsed.get("willRetry").and_then(Value::as_bool),
        }),
        "message_start" => to_message(parsed).map(RpcEvent::MessageStart),
        "message_end" => to_message(parsed).map(RpcEvent::MessageEnd),
        "message_update" => to_message_update(parsed).map(RpcEvent::MessageUpdate),
        "tool_execution_start" => Some(RpcEvent::ToolExecutionStart {
            tool_call_id: string(parsed, "toolCallId")?,
            tool_name: string(parsed, "toolName")?,
        }),
        "tool_execution_update" => Some(RpcEvent::ToolExecutionUpdate {
            tool_call_id: string(parsed, "toolCallId")?,
            tool_name: string(parsed, "toolName")?,
        }),
        "tool_execution_end" => Some(RpcEvent::ToolExecutionEnd {
            tool_call_id: string(parsed, "toolCallId")?,
            tool_name: string(parsed, "toolName")?,
            is_error: parsed.get("isError").and_then(Value::as_bool) == Some(true),
        }),
        "extension_ui_request" => to_ui_request(parsed).map(RpcEvent::UiRequest),
        "extension_error" => Some(RpcEvent::ExtensionError {
            extension_path: string(parsed, "extensionPath").unwrap_or_default(),
            event: string(parsed, "event").unwrap_or_default(),
            error: string(parsed, "error")?,
        }),
        "response" => to_response(parsed).map(RpcEvent::Response),
        _ => None,
    }
}

fn to_message(parsed: &Map<String, Value>) -> Option<RpcMessage> {
    let message = parsed.get("message")?;
    let role = message.as_object()?.get("role")?.as_str()?.to_owned();
    Some(RpcMessage {
        role,
        raw: message.clone(),
    })
}

fn to_message_update(parsed: &Map<String, Value>) -> Option<AssistantMessageEvent> {
    let event = parsed.get("assistantMessageEvent")?.as_object()?;
    let content_index = number(event, "contentIndex")?;
    Some(match event.get("type")?.as_str()? {
        "text_start" => AssistantMessageEvent::TextStart { content_index },
        "text_delta" => AssistantMessageEvent::TextDelta {
            content_index,
            delta: string(event, "delta")?,
        },
        "text_end" => AssistantMessageEvent::TextEnd {
            content_index,
            content: string(event, "content")?,
        },
        "thinking_start" => AssistantMessageEvent::ThinkingStart { content_index },
        "thinking_delta" => AssistantMessageEvent::ThinkingDelta {
            content_index,
            delta: string(event, "delta")?,
        },
        "thinking_end" => AssistantMessageEvent::ThinkingEnd {
            content_index,
            content: string(event, "content")?,
        },
        "toolcall_start" => AssistantMessageEvent::ToolcallStart {
            content_index,
            id: string(event, "id")?,
            tool_name: string(event, "toolName")?,
        },
        "toolcall_delta" => AssistantMessageEvent::ToolcallDelta {
            content_index,
            delta: string(event, "delta")?,
        },
        "toolcall_end" => AssistantMessageEvent::ToolcallEnd {
            content_index,
            tool_call: event.get("toolCall").cloned().unwrap_or(Value::Null),
        },
        _ => return None,
    })
}

fn to_ui_request(parsed: &Map<String, Value>) -> Option<UiRequest> {
    let id = string(parsed, "id")?;
    let method = string(parsed, "method")?;
    let timeout = number(parsed, "timeout");
    match method.as_str() {
        "select" => {
            let options = parsed
                .get("options")?
                .as_array()?
                .iter()
                .filter_map(|option| option.as_str().map(str::to_owned))
                .collect();
            Some(UiRequest::Select {
                id,
                title: string(parsed, "title")?,
                options,
                timeout,
            })
        }
        "confirm" => Some(UiRequest::Confirm {
            id,
            title: string(parsed, "title")?,
            message: string(parsed, "message")?,
            timeout,
        }),
        "input" => Some(UiRequest::Input {
            id,
            title: string(parsed, "title")?,
            placeholder: string(parsed, "placeholder"),
            timeout,
        }),
        "editor" => Some(UiRequest::Editor {
            id,
            title: string(parsed, "title")?,
            prefill: string(parsed, "prefill"),
        }),
        "notify" | "setStatus" | "setWidget" | "setTitle" | "set_editor_text" => {
            Some(UiRequest::FireAndForget {
                id,
                method,
                raw: Value::Object(parsed.clone()),
            })
        }
        _ => None,
    }
}

fn to_response(parsed: &Map<String, Value>) -> Option<RpcResponse> {
    let command = string(parsed, "command")?;
    let success = parsed.get("success")?.as_bool()?;
    let id = string(parsed, "id");
    let outcome = if success {
        Ok(parsed.get("data").cloned().unwrap_or(Value::Null))
    } else {
        Err(string(parsed, "error")?)
    };
    Some(RpcResponse {
        id,
        command,
        outcome,
    })
}
