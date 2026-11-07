//! Applying what the user asked for.

use gentle_core::chat::{ChatMessage, ChatRole, ChatState, Dialog};
use gentle_core::rpc::RpcCommand;

use crate::input::{Command, Intent};
use crate::state::AppState;

pub(crate) fn apply_intent(mut state: AppState, intent: Intent) -> (AppState, Vec<Command>) {
    match intent {
        Intent::SendPrompt(text) => {
            // One turn at a time: the UI disables sending while the agent works.
            if text.trim().is_empty() || state.chat.working {
                return (state, Vec::new());
            }
            state.chat.messages.push(ChatMessage {
                id: format!("msg-{}", state.chat.messages.len()),
                role: ChatRole::User,
                text: text.clone(),
                streaming: false,
                error: None,
            });
            (
                state,
                vec![Command::Send(RpcCommand::Prompt { message: text })],
            )
        }
        Intent::Abort => (state, vec![Command::Send(RpcCommand::Abort)]),
        Intent::NewChat => {
            let mut commands = Vec::new();
            if state.chat.working {
                commands.push(Command::Send(RpcCommand::Abort));
            }
            commands.push(Command::Send(RpcCommand::NewSession));
            let fresh = AppState {
                sessions: state.sessions,
                ..AppState::default()
            };
            (fresh, commands)
        }
        Intent::CopyMessage(id) => {
            let commands = state
                .chat
                .messages
                .iter()
                .find(|message| message.id == id)
                .map(|message| Command::CopyToClipboard(message.text.clone()))
                .into_iter()
                .collect();
            (state, commands)
        }
        Intent::AnswerDialog { id, answer } => {
            let known = state
                .chat
                .pending_dialogs
                .iter()
                .any(|d| dialog_id(d) == id);
            if !known {
                return (state, Vec::new());
            }
            state.chat.pending_dialogs.retain(|d| dialog_id(d) != id);
            (
                state,
                vec![Command::Send(RpcCommand::UiResponse { id, answer })],
            )
        }
        Intent::RefreshSessions => (state, vec![Command::RefreshSessions]),
        Intent::SelectHelper(id) => {
            // Only a task the panel knows about can be opened.
            if state.chat.helpers.tasks.iter().any(|task| task.id == id) {
                state.selected_helper = Some(id);
            }
            (state, Vec::new())
        }
        Intent::CloseHelper => {
            state.selected_helper = None;
            (state, Vec::new())
        }
        Intent::OpenSession(path) => {
            let mut commands = Vec::new();
            if state.chat.working {
                commands.push(Command::Send(RpcCommand::Abort));
            }
            commands.push(Command::Send(RpcCommand::SwitchSession {
                session_path: path.clone(),
            }));
            state.chat = ChatState::default();
            state.selected_helper = None;
            state.current_session = Some(path);
            (state, commands)
        }
        Intent::DismissNotice(id) => {
            state.notices.retain(|notice| notice.id != id);
            (state, Vec::new())
        }
    }
}

fn dialog_id(dialog: &Dialog) -> &str {
    match dialog {
        Dialog::Select { id, .. }
        | Dialog::Confirm { id, .. }
        | Dialog::Input { id, .. }
        | Dialog::Editor { id, .. } => id,
    }
}
