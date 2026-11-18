//! Folding one decoded RPC event into [`ChatState`].

use serde_json::Value;

use super::state::{ChatItem, ChatRole, ChatState, Dialog};
use crate::helpers::fold_widget;
use crate::i18n::{messages, Language};
use crate::rpc::{AssistantMessageEvent, RpcEvent, RpcMessage, UiRequest};

/// Folds one decoded RPC event into chat state.
///
/// Pure: same inputs, same output, no I/O and no `now()`. It takes and returns
/// the state by value so the caller decides where the new state lives, and
/// events the chat view does not use return the state untouched.
pub fn reduce_chat(language: Language, state: ChatState, event: &RpcEvent) -> ChatState {
    reduce_chat_at(language, state, event, None)
}

/// [`reduce_chat`] with the wall-clock time (epoch ms) the event arrived at,
/// which the caller supplies so this stays pure.
///
/// The clock stamps a reasoning block and a tool execution so their rows can
/// answer how long they ran; with no clock they carry no stamps and no
/// duration, because nothing may be guessed (`prompt-turn.jsonl` has no
/// timestamps). Helper bookkeeping reads it the same way.
///
/// The language comes first, as every catalog message does: the one text this
/// fold words itself is the provider's unspoken error
/// (`messages::unknown_model_error`), and the shell that holds the config hands
/// it in (0.2.0, S3).
pub fn reduce_chat_at(
    language: Language,
    mut state: ChatState,
    event: &RpcEvent,
    now_ms: Option<u64>,
) -> ChatState {
    match event {
        RpcEvent::AgentStart => state.working = true,
        RpcEvent::AgentEnd { .. } | RpcEvent::AgentSettled => state.working = false,
        RpcEvent::MessageStart(message) => state = open_assistant_message(state, message),
        RpcEvent::MessageUpdate(delta) => {
            state = apply_assistant_message_event(state, delta, now_ms)
        }
        RpcEvent::MessageEnd(message) => state = close_assistant_message(language, state, message),
        RpcEvent::ToolExecutionStart {
            tool_call_id,
            tool_name,
            args,
        } => start_tool_execution(&mut state, tool_call_id, tool_name, args.as_ref(), now_ms),
        RpcEvent::ToolExecutionUpdate {
            tool_call_id,
            args,
            partial_result,
            ..
        } => update_tool_execution(&mut state, tool_call_id, args.as_ref(), partial_result.as_ref()),
        RpcEvent::ToolExecutionEnd {
            tool_call_id,
            result,
            is_error,
            ..
        } => end_tool_execution(&mut state, tool_call_id, result.as_ref(), *is_error, now_ms),
        RpcEvent::UiRequest(request) => state = apply_extension_ui_request(state, request, now_ms),
        RpcEvent::ExtensionError { error, .. } => state.last_error = Some(error.clone()),
        // turn_start/turn_end and command responses carry nothing the chat view
        // needs: `working` is driven by the agent_* events and the assistant
        // text by message_*/text_* deltas.
        RpcEvent::TurnStart | RpcEvent::TurnEnd | RpcEvent::Response(_) => {}
    }
    state
}

fn open_assistant_message(mut state: ChatState, message: &RpcMessage) -> ChatState {
    if message.role != "assistant" {
        return state;
    }
    state.push_message(ChatRole::Assistant, String::new(), true);
    state
}

fn apply_assistant_message_event(
    mut state: ChatState,
    delta: &AssistantMessageEvent,
    now_ms: Option<u64>,
) -> ChatState {
    match delta {
        AssistantMessageEvent::TextDelta { delta, .. } => append_text(&mut state, delta),
        // text_start/text_end carry nothing new: the running text is assembled
        // from text_delta and finalized from `message_end`.
        AssistantMessageEvent::TextStart { .. } | AssistantMessageEvent::TextEnd { .. } => {}
        AssistantMessageEvent::ThinkingStart { .. } => open_thinking(&mut state, now_ms),
        AssistantMessageEvent::ThinkingDelta { delta, .. } => append_thinking(&mut state, delta),
        AssistantMessageEvent::ThinkingEnd { content, .. } => {
            close_thinking(&mut state, content, now_ms)
        }
        AssistantMessageEvent::ToolcallStart { id, tool_name, .. } => {
            open_tool_call(&mut state, id, tool_name)
        }
        AssistantMessageEvent::ToolcallEnd { tool_call, .. } => finish_tool_call(&mut state, tool_call),
        // The arguments object arrives as text; the tool row keeps the parsed
        // object `toolcall_end` and `tool_execution_start` carry, so this delta
        // reaches no row and stays activity.
        AssistantMessageEvent::ToolcallDelta { .. } => bump(&mut state),
    }
    state
}

/// Counts one event whose content the transcript keeps nowhere.
fn bump(state: &mut ChatState) {
    state.activity += 1;
}

/// Appends a delta to the assistant text row the message is still writing.
fn append_text(state: &mut ChatState, delta: &str) {
    if let Some(ChatItem::Text(last)) = state.items.last_mut() {
        if last.role == ChatRole::Assistant && last.streaming {
            last.text.push_str(delta);
        }
    }
}

/// Adds a row at the end of the transcript, keeping the open assistant text
/// bubble last.
///
/// `message_start` opens an empty bubble before the thinking and the tool calls
/// of the same message arrive, and an empty bubble says nothing about where it
/// belongs: the rows that arrive meanwhile take its place, so the message's text
/// stays the last row, which is where the child's content blocks end. Once the
/// bubble holds text it keeps its place.
fn push_row(state: &mut ChatState, row: ChatItem) {
    let placeholder = (state.items.len())
        .checked_sub(1)
        .filter(|&last| is_empty_open_assistant(&state.items[last]));
    match placeholder {
        Some(at) => state.items.insert(at, row),
        None => state.items.push(row),
    }
}

fn is_empty_open_assistant(item: &ChatItem) -> bool {
    matches!(
        item,
        ChatItem::Text(message)
            if message.role == ChatRole::Assistant
                && message.streaming
                && message.text.trim().is_empty()
    )
}

fn open_thinking(state: &mut ChatState, now_ms: Option<u64>) {
    let id = format!("think-{}", count_thinking(&state.items));
    push_row(
        state,
        ChatItem::Thinking {
            id,
            text: String::new(),
            streaming: true,
            started_at_ms: now_ms,
            ended_at_ms: None,
            duration_ms: None,
        },
    );
}

fn count_thinking(items: &[ChatItem]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, ChatItem::Thinking { .. }))
        .count()
}

fn append_thinking(state: &mut ChatState, delta: &str) {
    let Some(index) = open_thinking_at(state) else {
        return bump(state);
    };
    if let ChatItem::Thinking { text, .. } = &mut state.items[index] {
        text.push_str(delta);
    }
}

fn close_thinking(state: &mut ChatState, content: &str, now_ms: Option<u64>) {
    let Some(index) = open_thinking_at(state) else {
        return bump(state);
    };
    let ChatItem::Thinking {
        text,
        streaming,
        started_at_ms,
        ended_at_ms,
        duration_ms,
        ..
    } = &mut state.items[index]
    else {
        return;
    };
    // `thinking_end` is authoritative for the block's text, exactly as
    // `message_end` is for the message's.
    *text = content.to_owned();
    *streaming = false;
    *ended_at_ms = now_ms;
    *duration_ms = elapsed(*started_at_ms, now_ms);
}

fn open_thinking_at(state: &ChatState) -> Option<usize> {
    state
        .items
        .iter()
        .rposition(|item| matches!(item, ChatItem::Thinking { streaming: true, .. }))
}

fn tool_row(tool_call_id: &str, tool_name: &str) -> ChatItem {
    ChatItem::Tool {
        id: format!("tool-{tool_call_id}"),
        tool_call_id: tool_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
        args: None,
        partial_result: None,
        result: None,
        is_error: false,
        running: false,
        started_at_ms: None,
        ended_at_ms: None,
        duration_ms: None,
    }
}

/// Starts a tool row, which the model's own `toolcall_start` announces before
/// the tool runs: it carries no timing and no output until the tool does.
fn open_tool_call(state: &mut ChatState, tool_call_id: &str, tool_name: &str) {
    if tool_at(&state.items, Some(tool_call_id)).is_some() {
        return;
    }
    push_row(state, tool_row(tool_call_id, tool_name));
}

/// Fills in the arguments the model finished streaming (`toolcall_end`).
fn finish_tool_call(state: &mut ChatState, tool_call: &Value) {
    let Some(index) = tool_at(&state.items, tool_call.get("id").and_then(Value::as_str)) else {
        return bump(state);
    };
    let name = tool_call.get("name").and_then(Value::as_str).map(str::to_owned);
    let args = tool_call.get("arguments").cloned();
    let ChatItem::Tool {
        tool_name,
        args: row_args,
        ..
    } = &mut state.items[index]
    else {
        return;
    };
    if let Some(name) = name {
        *tool_name = name;
    }
    if args.is_some() {
        *row_args = args;
    }
}

/// The tool row an event names: by the child's call id when it carries one, and
/// otherwise the last row nothing has described yet, which is the row the block
/// that just ended opened.
fn tool_at(items: &[ChatItem], tool_call_id: Option<&str>) -> Option<usize> {
    match tool_call_id {
        Some(id) => items.iter().rposition(
            |item| matches!(item, ChatItem::Tool { tool_call_id: row, .. } if row == id),
        ),
        None => items.iter().rposition(|item| {
            matches!(
                item,
                ChatItem::Tool {
                    args: None,
                    result: None,
                    ..
                }
            )
        }),
    }
}

fn start_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    tool_name: &str,
    args: Option<&Value>,
    now_ms: Option<u64>,
) {
    if tool_at(&state.items, Some(tool_call_id)).is_none() {
        push_row(state, tool_row(tool_call_id, tool_name));
    }
    let Some(index) = tool_at(&state.items, Some(tool_call_id)) else {
        return;
    };
    let ChatItem::Tool {
        tool_name: row_name,
        args: row_args,
        running,
        started_at_ms,
        ..
    } = &mut state.items[index]
    else {
        return;
    };
    *row_name = tool_name.to_owned();
    if args.is_some() {
        *row_args = args.cloned();
    }
    *running = true;
    *started_at_ms = now_ms;
}

fn update_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    args: Option<&Value>,
    partial_result: Option<&Value>,
) {
    let Some(index) = tool_at(&state.items, Some(tool_call_id)) else {
        return bump(state);
    };
    let ChatItem::Tool {
        args: row_args,
        partial_result: row_partial,
        ..
    } = &mut state.items[index]
    else {
        return;
    };
    if args.is_some() {
        *row_args = args.cloned();
    }
    if partial_result.is_some() {
        *row_partial = partial_result.cloned();
    }
}

fn end_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    result: Option<&Value>,
    is_error: bool,
    now_ms: Option<u64>,
) {
    let Some(index) = tool_at(&state.items, Some(tool_call_id)) else {
        return bump(state);
    };
    let ChatItem::Tool {
        result: row_result,
        is_error: row_error,
        running,
        started_at_ms,
        ended_at_ms,
        duration_ms,
        ..
    } = &mut state.items[index]
    else {
        return;
    };
    if result.is_some() {
        *row_result = result.cloned();
    }
    *row_error = is_error;
    *running = false;
    *ended_at_ms = now_ms;
    *duration_ms = elapsed(*started_at_ms, now_ms);
}

/// How long something ran, when both ends of it are known.
fn elapsed(started_at_ms: Option<u64>, ended_at_ms: Option<u64>) -> Option<u64> {
    Some(ended_at_ms?.saturating_sub(started_at_ms?))
}

fn close_assistant_message(
    language: Language,
    mut state: ChatState,
    message: &RpcMessage,
) -> ChatState {
    if message.role != "assistant" {
        return state;
    }
    let Some(index) = open_assistant_at(&state.items) else {
        return state;
    };
    let trailing = index + 1 == state.items.len();
    let authoritative = extract_text(message);
    let error = model_error(language, message);
    let ChatItem::Text(last) = &mut state.items[index] else {
        return state;
    };

    // `message_end.message` is authoritative per rpc.md: resync from its text
    // content blocks when `content` is an array, otherwise keep what
    // text_delta accumulated.
    if let Some(authoritative) = authoritative {
        last.text = authoritative;
    }

    // A provider failure arrives as an ordinary-looking assistant message;
    // flag it (and keep it even when empty) so the UI can show it as a failure.
    if let Some(error) = error {
        last.error = Some(error);
        last.streaming = false;
        return state;
    }

    last.streaming = false;

    // A message that only carried thinking/toolCall parts has nothing to show,
    // so its empty bubble is dropped instead of leaving one in the thread. Only
    // the trailing one is, because `push_row` always keeps an empty bubble last:
    // a bubble with rows after it is one that already said something. Dropping
    // the row shrinks the text count, which is what reassigns the freed `msg-<n>`
    // id to the next assistant message with no gap and no extra bookkeeping. The
    // thinking and the tool calls it produced stay: they are rows of their own.
    if trailing && last.text.trim().is_empty() {
        state.items.remove(index);
    }
    state
}

/// The assistant text row this turn is still writing, wherever it sits: text the
/// child said before a tool call stays above that tool's row, and `message_end`
/// closes it from there.
fn open_assistant_at(items: &[ChatItem]) -> Option<usize> {
    items.iter().rposition(|item| {
        matches!(
            item,
            ChatItem::Text(message)
                if message.role == ChatRole::Assistant && message.streaming
        )
    })
}

/// The provider's error text when this message ended with `stopReason: "error"`.
///
/// A provider that said nothing gets the app's own two words, worded in the
/// language the shell handed the fold.
fn model_error(language: Language, message: &RpcMessage) -> Option<String> {
    if message.raw.get("stopReason").and_then(Value::as_str) != Some("error") {
        return None;
    }
    let text = message.raw.get("errorMessage").and_then(Value::as_str);
    Some(
        text.unwrap_or(messages::unknown_model_error(language))
            .to_owned(),
    )
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
