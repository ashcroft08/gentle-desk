//! The `app` layer of gentle-desk: state, intents and commands.
//!
//! Pure: [`update`] takes the state and one [`Input`] (a user intent from the
//! UI or an RPC event from the backend) and returns the next state plus the
//! [`Command`]s to execute. Nothing here spawns a process, touches the
//! clipboard or knows a GUI framework; the shell around it runs the commands.

use gentle_core::chat::{reduce_chat, ChatMessage, ChatRole, ChatState, Dialog};
use gentle_core::rpc::{RpcCommand, RpcEvent, UiAnswer, UiRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

/// A transient message from an extension (`notify`), shown outside the thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: u64,
    pub level: NoticeLevel,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    pub chat: ChatState,
    pub notices: Vec<Notice>,
    next_notice_id: u64,
}

/// What the user asked for; the UI only ever emits these.
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    SendPrompt(String),
    Abort,
    NewChat,
    /// Copy the raw markdown of one message.
    CopyMessage(String),
    AnswerDialog {
        id: String,
        answer: UiAnswer,
    },
    DismissNotice(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Intent(Intent),
    Rpc(RpcEvent),
}

/// A side effect for the shell to run.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Send(RpcCommand),
    CopyToClipboard(String),
}

/// Most notices kept; older ones are dropped first.
const MAX_NOTICES: usize = 50;

pub fn update(state: AppState, input: Input) -> (AppState, Vec<Command>) {
    match input {
        Input::Rpc(event) => (apply_event(state, &event), Vec::new()),
        Input::Intent(intent) => apply_intent(state, intent),
    }
}

fn apply_event(state: AppState, event: &RpcEvent) -> AppState {
    let notice = notice_from(event);
    let mut state = AppState {
        chat: reduce_chat(state.chat, event),
        ..state
    };
    if let Some((level, text)) = notice {
        state.notices.push(Notice {
            id: state.next_notice_id,
            level,
            text,
        });
        state.next_notice_id += 1;
        let excess = state.notices.len().saturating_sub(MAX_NOTICES);
        state.notices.drain(..excess);
    }
    state
}

/// The level and text of an extension `notify`; other fire-and-forget
/// requests (status bars, widgets, titles) are not notices.
fn notice_from(event: &RpcEvent) -> Option<(NoticeLevel, String)> {
    let RpcEvent::UiRequest(UiRequest::FireAndForget { method, raw, .. }) = event else {
        return None;
    };
    if method != "notify" {
        return None;
    }
    let text = raw.get("message")?.as_str()?.to_owned();
    let level = match raw.get("notifyType").and_then(|v| v.as_str()) {
        Some("warning") => NoticeLevel::Warning,
        Some("error") => NoticeLevel::Error,
        _ => NoticeLevel::Info,
    };
    Some((level, text))
}

fn apply_intent(mut state: AppState, intent: Intent) -> (AppState, Vec<Command>) {
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
            (AppState::default(), commands)
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
