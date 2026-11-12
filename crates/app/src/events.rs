//! Folding RPC events from the backend into the app state.

use gentle_core::chat::{messages_from_history, reduce_chat_at};
use gentle_core::rpc::{RpcCommand, RpcEvent, RpcResponse};

use crate::chip;
use crate::input::Command;
use crate::notices::{notice_from, push_notice};
use crate::state::{AppState, ModelsStatus, NoticeLevel};

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
        (command, Ok(data)) => {
            // The chip's four questions parse the same in both shells.
            chip::apply_ok(state, command, data);
        }
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
    }
}
