//! Folding RPC events from the backend into the app state.

use gentle_core::chat::{messages_from_history, reduce_chat_at};
use gentle_core::rpc::{RpcCommand, RpcEvent, RpcResponse};
use serde_json::Value;

use crate::input::Command;
use crate::notices::{notice_from, push_notice};
use crate::state::{AppState, ModelChoice, ModelsStatus, NoticeLevel};

pub(crate) fn apply_event(
    state: AppState,
    event: &RpcEvent,
    now_ms: Option<u64>,
) -> (AppState, Vec<Command>) {
    let notice = notice_from(event);
    let mut commands = Vec::new();
    let mut state = AppState {
        chat: reduce_chat_at(state.chat, event, now_ms),
        ..state
    };
    if let Some((level, text)) = notice {
        push_notice(&mut state, level, text);
    }
    match event {
        // A finished turn may have created or extended a session file.
        RpcEvent::AgentSettled => commands.push(Command::RefreshSessions),
        RpcEvent::Response(response) => apply_response(&mut state, response, &mut commands),
        _ => {}
    }
    (state, commands)
}

/// Opening a session is a two-step exchange: `switch_session`, then `get_messages`.
/// One Model object in pi's shape (`id`, `name`, `provider`, optional
/// `reasoning`) flattened for the chip. `None` when the value is not a model.
fn model_from_value(value: Option<&Value>) -> Option<ModelChoice> {
    let object = value?.as_object()?;
    let model_id = object.get("id")?.as_str()?.to_owned();
    Some(ModelChoice {
        provider: object.get("provider")?.as_str()?.to_owned(),
        model_id: model_id.clone(),
        name: object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&model_id)
            .to_owned(),
        reasoning: object.get("reasoning").and_then(Value::as_bool) == Some(true),
    })
}

fn apply_response(state: &mut AppState, response: &RpcResponse, commands: &mut Vec<Command>) {
    match (response.command.as_str(), &response.outcome) {
        ("switch_session", Ok(data)) => {
            if data.get("cancelled").and_then(|c| c.as_bool()) == Some(true) {
                state.current_session = None;
            } else {
                // The history, then the chip's refill: the session's model and
                // level, the models the child offers, and the levels it
                // supports (R3-002). The chip was cleared on the switch, so
                // these replies repopulate it.
                commands.push(Command::Send(RpcCommand::GetMessages { id: None }));
                commands.push(Command::Send(RpcCommand::GetState));
                commands.push(Command::Send(RpcCommand::GetAvailableModels));
                commands.push(Command::Send(RpcCommand::GetAvailableThinkingLevels));
            }
        }
        ("get_messages", Ok(data)) => state.chat.messages = messages_from_history(data),
        ("get_state", Ok(data)) => {
            // The session reports its model and thinking level; the chip
            // renders both. Unknown fields stay unknown.
            state.model = model_from_value(data.get("model"));
            state.thinking_level = data
                .get("thinkingLevel")
                .and_then(Value::as_str)
                .map(str::to_owned);
        }
        ("get_available_models", Ok(data)) => {
            state.models = data
                .get("models")
                .and_then(Value::as_array)
                .map(|models| {
                    models
                        .iter()
                        .filter_map(|m| model_from_value(Some(m)))
                        .collect()
                })
                .unwrap_or_default();
            state.models_status = if state.models.is_empty() {
                ModelsStatus::Empty
            } else {
                ModelsStatus::Ready
            };
        }
        ("set_model", Ok(data)) => state.model = model_from_value(Some(data)),
        ("set_thinking_level", Err(error)) => {
            // The optimistic level from the intent is gone; unknown until the
            // session reports again. Same notice as every failed command.
            state.thinking_level = None;
            push_notice(
                state,
                NoticeLevel::Error,
                format!("set_thinking_level failed: {error}"),
            );
        }
        ("get_available_models", Err(error)) => {
            state.models_status = ModelsStatus::Failed(error.clone());
            push_notice(
                state,
                NoticeLevel::Error,
                format!("get_available_models failed: {error}"),
            );
        }
        ("get_available_thinking_levels", Ok(data)) => {
            state.thinking_levels = data
                .as_array()
                .map(|levels| {
                    levels
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
        }
        (command, Err(error)) => {
            if command == "switch_session" {
                state.current_session = None;
            }
            push_notice(
                state,
                NoticeLevel::Error,
                format!("{command} failed: {error}"),
            );
        }
        _ => {}
    }
}
