//! Folding RPC events from the backend into the app state.

use gentle_core::chat::{messages_from_history, reduce_chat_at};
use gentle_core::rpc::{RpcCommand, RpcEvent, RpcResponse};

use crate::input::Command;
use crate::notices::{notice_from, push_notice};
use crate::state::{AppState, NoticeLevel};

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
                commands.push(Command::Send(RpcCommand::GetMessages { id: None }));
            }
        }
        ("get_messages", Ok(data)) => state.chat.messages = messages_from_history(data),
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
