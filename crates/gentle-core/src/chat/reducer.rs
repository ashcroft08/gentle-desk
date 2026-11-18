//! Folding one decoded RPC event into [`ChatState`].

use serde_json::Value;

use super::state::{ChatItem, ChatMessage, ChatRole, ChatState, Dialog, ToolPayload};
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
///
/// The row is looked up, never assumed last: a tool call or a reasoning block
/// the message produced sits after it, and the text that arrives afterwards is
/// still that message's text (R2-001).
fn append_text(state: &mut ChatState, delta: &str) {
    let Some(message) = open_assistant_mut(&mut state.items) else {
        return;
    };
    message.text.push_str(delta);
}

/// The assistant text row this turn is still writing, mutable, wherever it
/// sits: text the child said before a tool call stays above that tool's row,
/// and the deltas that follow land in it.
///
/// The `None` arm is the real answer for a user row, a closed message, a
/// reasoning row and a tool row (R2-002).
fn open_assistant_mut(items: &mut [ChatItem]) -> Option<&mut ChatMessage> {
    items.iter_mut().rev().find_map(|item| match item {
        ChatItem::Text(message) if message.role == ChatRole::Assistant && message.streaming => {
            Some(message)
        }
        _ => None,
    })
}

/// Adds a row at the end of the transcript, ahead of an empty open assistant
/// bubble.
///
/// `message_start` opens an empty bubble before the thinking and the tool calls
/// of the same message arrive, and an empty bubble says nothing about where it
/// belongs: the rows that arrive meanwhile take its place. Once the bubble holds
/// text it keeps its place, and from then on the text is found by search rather
/// than assumed to be last (R2-001), so no row's position depends on this.
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
    let Some(row) = open_thinking_mut(&mut state.items) else {
        return bump(state);
    };
    row.text.push_str(delta);
}

fn close_thinking(state: &mut ChatState, content: &str, now_ms: Option<u64>) {
    let Some(row) = open_thinking_mut(&mut state.items) else {
        return bump(state);
    };
    // `thinking_end` is authoritative for the block's text, exactly as
    // `message_end` is for the message's.
    *row.text = content.to_owned();
    *row.streaming = false;
    *row.ended_at_ms = now_ms;
    *row.duration_ms = elapsed(*row.started_at_ms, now_ms);
}

/// The parts of a reasoning row the events write.
struct ThinkingRow<'a> {
    text: &'a mut String,
    streaming: &'a mut bool,
    started_at_ms: &'a mut Option<u64>,
    ended_at_ms: &'a mut Option<u64>,
    duration_ms: &'a mut Option<u64>,
}

/// The reasoning block that is still open, mutable: `thinking_delta` grows it
/// and `thinking_end` closes it. The match is total and its `None` arm is the
/// real answer for every row that is not an open block (R2-002).
fn open_thinking_mut(items: &mut [ChatItem]) -> Option<ThinkingRow<'_>> {
    items.iter_mut().rev().find_map(|item| {
        let ChatItem::Thinking {
            text,
            streaming,
            started_at_ms,
            ended_at_ms,
            duration_ms,
            ..
        } = item
        else {
            return None;
        };
        if !*streaming {
            return None;
        }
        Some(ThinkingRow {
            text,
            streaming,
            started_at_ms,
            ended_at_ms,
            duration_ms,
        })
    })
}

fn tool_row(
    tool_call_id: &str,
    tool_name: &str,
    args: Option<ToolPayload>,
    running: bool,
    started_at_ms: Option<u64>,
) -> ChatItem {
    ChatItem::Tool {
        id: format!("tool-{tool_call_id}"),
        tool_call_id: tool_call_id.to_owned(),
        tool_name: tool_name.to_owned(),
        args,
        partial_result: None,
        result: None,
        is_error: false,
        running,
        started_at_ms,
        ended_at_ms: None,
        duration_ms: None,
    }
}

/// The parts of a tool row the events write.
struct ToolRow<'a> {
    tool_name: &'a mut String,
    args: &'a mut Option<ToolPayload>,
    partial_result: &'a mut Option<ToolPayload>,
    result: &'a mut Option<ToolPayload>,
    is_error: &'a mut bool,
    running: &'a mut bool,
    started_at_ms: &'a mut Option<u64>,
    ended_at_ms: &'a mut Option<u64>,
    duration_ms: &'a mut Option<u64>,
}

/// Starts a tool row, which the model's own `toolcall_start` announces before
/// the tool runs: it carries no timing and no output until the tool does.
fn open_tool_call(state: &mut ChatState, tool_call_id: &str, tool_name: &str) {
    // The execution may have opened the row already, and two rows for one call
    // would be two truths about it.
    if tool_at(&state.items, Some(tool_call_id)).is_some() {
        return;
    }
    push_row(state, tool_row(tool_call_id, tool_name, None, false, None));
}

/// Fills in the arguments the model finished streaming (`toolcall_end`).
fn finish_tool_call(state: &mut ChatState, tool_call: &Value) {
    let Some(row) = tool_row_mut(
        &mut state.items,
        tool_call.get("id").and_then(Value::as_str),
    ) else {
        return bump(state);
    };
    if let Some(name) = tool_call.get("name").and_then(Value::as_str) {
        *row.tool_name = name.to_owned();
    }
    if let Some(args) = tool_call.get("arguments") {
        *row.args = Some(ToolPayload::new(args));
    }
}

/// The tool row an event names: by the child's call id when the event carries
/// one, and otherwise the last row nothing has described yet.
///
/// The `None` case is the recorded untested edge
/// (`odd/tasks/chat-transcript.md`, L6): a `toolcall_end` with no id falls back
/// to the row the block that just ended opened, because pi always sends the id.
fn tool_at(items: &[ChatItem], tool_call_id: Option<&str>) -> Option<usize> {
    items.iter().rposition(|item| match item {
        ChatItem::Tool {
            tool_call_id: row,
            args,
            result,
            ..
        } => names_call(tool_call_id, row, args.as_ref(), result.as_ref()),
        ChatItem::Text(_) | ChatItem::Thinking { .. } => false,
    })
}

/// Does this tool row answer the id the event names? With no id in the event,
/// only a row nothing has described yet can be the one meant.
fn names_call(
    tool_call_id: Option<&str>,
    row: &str,
    args: Option<&ToolPayload>,
    result: Option<&ToolPayload>,
) -> bool {
    match tool_call_id {
        Some(id) => row == id,
        None => args.is_none() && result.is_none(),
    }
}

/// [`tool_at`]'s row, mutable, so an event writes its own fields instead of
/// destructuring the enum at every site. Its `None` arm is the real answer for
/// every row that is not a tool row and for every row the event does not name
/// (R2-002).
fn tool_row_mut<'a>(items: &'a mut [ChatItem], tool_call_id: Option<&str>) -> Option<ToolRow<'a>> {
    items.iter_mut().rev().find_map(|item| {
        let ChatItem::Tool {
            tool_call_id: row,
            tool_name,
            args,
            partial_result,
            result,
            is_error,
            running,
            started_at_ms,
            ended_at_ms,
            duration_ms,
            ..
        } = item
        else {
            return None;
        };
        if !names_call(tool_call_id, row, args.as_ref(), result.as_ref()) {
            return None;
        }
        Some(ToolRow {
            tool_name,
            args,
            partial_result,
            result,
            is_error,
            running,
            started_at_ms,
            ended_at_ms,
            duration_ms,
        })
    })
}

fn start_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    tool_name: &str,
    args: Option<&Value>,
    now_ms: Option<u64>,
) {
    let args = args.map(ToolPayload::new);
    // One lookup decides where the execution lands (R2-003): the row the model
    // already announced, or - when no row names the call yet - the row pushed
    // below, which is created already carrying what the execution knows.
    if let Some(row) = tool_row_mut(&mut state.items, Some(tool_call_id)) {
        *row.tool_name = tool_name.to_owned();
        if args.is_some() {
            *row.args = args;
        }
        *row.running = true;
        *row.started_at_ms = now_ms;
        return;
    }
    push_row(state, tool_row(tool_call_id, tool_name, args, true, now_ms));
}

fn update_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    args: Option<&Value>,
    partial_result: Option<&Value>,
) {
    let Some(row) = tool_row_mut(&mut state.items, Some(tool_call_id)) else {
        return bump(state);
    };
    if let Some(args) = args {
        *row.args = Some(ToolPayload::new(args));
    }
    if let Some(partial_result) = partial_result {
        *row.partial_result = Some(ToolPayload::new(partial_result));
    }
}

fn end_tool_execution(
    state: &mut ChatState,
    tool_call_id: &str,
    result: Option<&Value>,
    is_error: bool,
    now_ms: Option<u64>,
) {
    let Some(row) = tool_row_mut(&mut state.items, Some(tool_call_id)) else {
        return bump(state);
    };
    if let Some(result) = result {
        *row.result = Some(ToolPayload::new(result));
        // The result says everything the running output did, so the partial one
        // goes: a call keeps one output, not a growing pair (R1).
        *row.partial_result = None;
    }
    *row.is_error = is_error;
    *row.running = false;
    *row.ended_at_ms = now_ms;
    *row.duration_ms = elapsed(*row.started_at_ms, now_ms);
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
    let Some((index, empty)) = close_open_assistant(
        &mut state.items,
        extract_text(message),
        model_error(language, message),
    ) else {
        return state;
    };

    // A message that only carried thinking/toolCall parts has nothing to show,
    // so its bubble is dropped wherever it sits (R3-002): a tool row after it
    // does not make it a message, and leaving it would leak an empty message
    // into `messages()`. It is the last text row, so dropping it frees the
    // highest `msg-<n>` id for the next assistant message: ids stay contiguous,
    // with no gap and no second bookkeeping.
    // The thinking and the tool calls it produced stay: they are rows of their
    // own.
    if empty {
        state.items.remove(index);
    }
    state
}

/// Writes the authoritative end of a message into the open assistant text row,
/// wherever that row sits, and answers where it is and whether it finalizes
/// with nothing to show.
fn close_open_assistant(
    items: &mut [ChatItem],
    authoritative: Option<String>,
    error: Option<String>,
) -> Option<(usize, bool)> {
    for (index, item) in items.iter_mut().enumerate().rev() {
        let ChatItem::Text(message) = item else {
            continue;
        };
        if message.role != ChatRole::Assistant || !message.streaming {
            continue;
        }
        // `message_end.message` is authoritative per rpc.md: resync from its
        // text content blocks when `content` is an array, otherwise keep what
        // text_delta accumulated.
        if let Some(authoritative) = authoritative {
            message.text = authoritative;
        }
        message.streaming = false;
        // A provider failure arrives as an ordinary-looking assistant message;
        // flag it (and keep it even when empty) so the UI can show it as a
        // failure.
        if let Some(error) = error {
            message.error = Some(error);
            return Some((index, false));
        }
        return Some((index, message.text.trim().is_empty()));
    }
    None
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
