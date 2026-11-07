//! Folding one decoded RPC event into [`ChatState`].

use serde_json::Value;

use super::state::{ChatMessage, ChatRole, ChatState, Dialog};
use crate::helpers::fold_widget;
use crate::rpc::{AssistantMessageEvent, RpcEvent, RpcMessage, UiRequest};

/// Folds one decoded RPC event into chat state.
///
/// Pure: same inputs, same output, no I/O and no `now()`. It takes and returns
/// the state by value so the caller decides where the new state lives, and
/// events the chat view does not use return the state untouched.
pub fn reduce_chat(state: ChatState, event: &RpcEvent) -> ChatState {
    reduce_chat_at(state, event, None)
}

/// [`reduce_chat`] with the wall-clock time (epoch ms) the event arrived at,
/// which the caller supplies so this stays pure. Only helper bookkeeping uses it.
pub fn reduce_chat_at(state: ChatState, event: &RpcEvent, now_ms: Option<u64>) -> ChatState {
    match event {
        RpcEvent::AgentStart => ChatState {
            working: true,
            ..state
        },
        RpcEvent::AgentEnd { .. } | RpcEvent::AgentSettled => ChatState {
            working: false,
            ..state
        },
        RpcEvent::MessageStart(message) => open_assistant_message(state, message),
        RpcEvent::MessageUpdate(delta) => apply_assistant_message_event(state, delta),
        RpcEvent::MessageEnd(message) => close_assistant_message(state, message),
        RpcEvent::ToolExecutionStart { .. }
        | RpcEvent::ToolExecutionUpdate { .. }
        | RpcEvent::ToolExecutionEnd { .. } => ChatState {
            activity: state.activity + 1,
            ..state
        },
        RpcEvent::UiRequest(request) => apply_extension_ui_request(state, request, now_ms),
        RpcEvent::ExtensionError { error, .. } => ChatState {
            last_error: Some(error.clone()),
            ..state
        },
        // turn_start/turn_end and command responses carry nothing the chat view
        // needs: `working` is driven by the agent_* events and the assistant
        // text by message_*/text_* deltas.
        RpcEvent::TurnStart | RpcEvent::TurnEnd | RpcEvent::Response(_) => state,
    }
}

fn open_assistant_message(state: ChatState, message: &RpcMessage) -> ChatState {
    if message.role != "assistant" {
        return state;
    }
    let mut messages = state.messages;
    messages.push(ChatMessage {
        id: format!("msg-{}", messages.len()),
        role: ChatRole::Assistant,
        text: String::new(),
        streaming: true,
        error: None,
    });
    ChatState { messages, ..state }
}

fn apply_assistant_message_event(state: ChatState, delta: &AssistantMessageEvent) -> ChatState {
    match delta {
        AssistantMessageEvent::TextDelta { delta, .. } => append_to_last_assistant(state, delta),
        // text_start/text_end carry nothing new: the running text is assembled
        // from text_delta and finalized from `message_end`.
        AssistantMessageEvent::ThinkingStart { .. }
        | AssistantMessageEvent::ThinkingDelta { .. }
        | AssistantMessageEvent::ThinkingEnd { .. }
        | AssistantMessageEvent::ToolcallStart { .. }
        | AssistantMessageEvent::ToolcallDelta { .. }
        | AssistantMessageEvent::ToolcallEnd { .. } => ChatState {
            activity: state.activity + 1,
            ..state
        },
        AssistantMessageEvent::TextStart { .. } | AssistantMessageEvent::TextEnd { .. } => state,
    }
}

fn append_to_last_assistant(state: ChatState, delta: &str) -> ChatState {
    let mut messages = state.messages;
    match messages.last_mut() {
        Some(last) if last.role == ChatRole::Assistant => last.text.push_str(delta),
        _ => return ChatState { messages, ..state },
    }
    ChatState { messages, ..state }
}

fn close_assistant_message(state: ChatState, message: &RpcMessage) -> ChatState {
    if message.role != "assistant" {
        return state;
    }
    let mut messages = state.messages;
    let Some(last) = messages.last_mut() else {
        return ChatState { messages, ..state };
    };
    if last.role != ChatRole::Assistant {
        return ChatState { messages, ..state };
    }

    // `message_end.message` is authoritative per rpc.md: resync from its text
    // content blocks when `content` is an array, otherwise keep what
    // text_delta accumulated.
    if let Some(authoritative) = extract_text(message) {
        last.text = authoritative;
    }

    // A provider failure arrives as an ordinary-looking assistant message;
    // flag it (and keep it even when empty) so the UI can show it as a failure.
    if let Some(error) = model_error(message) {
        last.error = Some(error);
        last.streaming = false;
        return ChatState { messages, ..state };
    }

    // A message that only ever carried thinking/toolCall parts has nothing to
    // show, so drop it instead of leaving an empty bubble in the thread.
    // Dropping the last element shrinks `messages.len()` back, which is what
    // reassigns the freed `msg-<n>` id to the next assistant message with no
    // gap and no extra bookkeeping.
    if last.text.trim().is_empty() {
        messages.pop();
        return ChatState { messages, ..state };
    }

    last.streaming = false;
    ChatState { messages, ..state }
}

/// The provider's error text when this message ended with `stopReason: "error"`.
fn model_error(message: &RpcMessage) -> Option<String> {
    if message.raw.get("stopReason").and_then(Value::as_str) != Some("error") {
        return None;
    }
    let text = message.raw.get("errorMessage").and_then(Value::as_str);
    Some(text.unwrap_or("unknown model error").to_owned())
}

/// The concatenated `{"type":"text"}` blocks of `content`, or `None` when
/// `content` is absent or not an array.
fn extract_text(message: &RpcMessage) -> Option<String> {
    let content = message.raw.get("content")?.as_array()?;
    let mut text = String::new();
    for block in content {
        if let Some(block) = block.as_object() {
            let is_text = block.get("type").and_then(Value::as_str) == Some("text");
            if is_text {
                if let Some(part) = block.get("text").and_then(Value::as_str) {
                    text.push_str(part);
                }
            }
        }
    }
    Some(text)
}

fn apply_extension_ui_request(
    state: ChatState,
    request: &UiRequest,
    now_ms: Option<u64>,
) -> ChatState {
    let dialog = match request {
        UiRequest::Select {
            id, title, options, ..
        } => Dialog::Select {
            id: id.clone(),
            title: title.clone(),
            options: options.clone(),
        },
        UiRequest::Confirm {
            id, title, message, ..
        } => Dialog::Confirm {
            id: id.clone(),
            title: title.clone(),
            message: message.clone(),
        },
        UiRequest::Input {
            id,
            title,
            placeholder,
            ..
        } => Dialog::Input {
            id: id.clone(),
            title: title.clone(),
            placeholder: placeholder.clone(),
        },
        UiRequest::Editor {
            id, title, prefill, ..
        } => Dialog::Editor {
            id: id.clone(),
            title: title.clone(),
            prefill: prefill.clone(),
        },
        // notify, setStatus, setTitle, set_editor_text: fire-and-forget, no
        // dialog is waiting for an answer.
        // Widgets, status bars, titles: no dialog is waiting. Only gentle-agents'
        // `setWidget` carries state (the helpers), the rest is chrome.
        UiRequest::FireAndForget { method, raw, .. } => {
            if method != "setWidget" {
                return state;
            }
            return ChatState {
                helpers: fold_widget(state.helpers, raw, now_ms),
                ..state
            };
        }
    };
    let mut pending_dialogs = state.pending_dialogs;
    pending_dialogs.push(dialog);
    ChatState {
        pending_dialogs,
        ..state
    }
}
