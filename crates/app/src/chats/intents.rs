//! The intents that belong to one chat rather than to the single open thread.
//!
//! These are additive: they exist next to the single-chat intents and only act
//! when the intent names a chat (or when there is a foreground chat to act on).
//! The one-child shell never produces them, so nothing it does changes.

use gentle_core::rpc::{RpcCommand, UiAnswer};

use super::queue;
use super::ChatId;
use crate::chats::push_death_notice;
use crate::input::{ChatCommand, Command, Effects, Intent};
use crate::intents::dialog_id;
use crate::notices::drop_restart_notice;
use crate::state::{push_user_message, AppState};

/// Applies `intent` if it belongs to the per-chat model, answering the effects
/// for one chat's child (and the shared ones it asks for). Answers `None` when
/// it is the single-chat intent.
pub(crate) fn apply(state: &mut AppState, intent: &Intent) -> Option<Effects> {
    match intent {
        Intent::OpenChat(chat) => Some(for_chat(open(state, chat.clone()))),
        Intent::OpenNewChat => Some(for_chat(open(state, ChatId::fresh()))),
        Intent::RestartChat(chat) => Some(for_chat(open(state, chat.clone()))),
        Intent::SendPrompt(text) => send_prompt(state, text).map(for_chat),
        Intent::Abort => {
            // The Abort of a chat that only waits for a slot cancels its own
            // queued prompt and sends nothing (S7): no turn was started.
            let waiting = state
                .chats
                .foreground()
                .filter(|chat| state.chats.is_queued(chat))
                .cloned();
            if let Some(chat) = waiting {
                queue::cancel(&mut state.chats, &chat);
                return Some(for_chat(Vec::new()));
            }
            send_to_foreground(state, RpcCommand::Abort).map(for_chat)
        }
        Intent::CopyMessage(id) => copy_message(state, id).map(for_shell),
        Intent::AnswerDialog { id, answer } => answer_dialog(state, id, answer).map(for_chat),
        _ => None,
    }
}

/// Effects on one chat's child and nothing of the single child.
fn for_chat(chat_commands: Vec<ChatCommand>) -> Effects {
    Effects {
        commands: Vec::new(),
        chat_commands,
    }
}

/// The shared effects (clipboard, refresh) and nothing on any child.
fn for_shell(commands: Vec<Command>) -> Effects {
    Effects {
        commands,
        chat_commands: Vec::new(),
    }
}

/// Makes `chat` the foreground one and asks the host for its child. A chat that
/// already has a live child is only focused by the host, so this never sends
/// `switch_session` or `new_session`: the session travels on the command line.
/// It is also the whole of a restart: opening a chat clears the death mark, and
/// the host starts a fresh child for it through `--session` (S3).
///
/// Until the host answers, the chat is [`ChatActivity::Opening`]: the click is
/// seen and nothing can be sent there yet. A fresh child's first event (and, as
/// the net for a child that never speaks, the UI's 20 s confirmation) clears it,
/// and a failed start through `Input::ChatFailed`; an already-live child is
/// confirmed at once. So the state never sticks.
fn open(state: &mut AppState, chat: ChatId) -> Vec<ChatCommand> {
    drop_restart_notice(state, &chat);
    state.chats.open(chat.clone());
    state.chats.mark_opening(&chat);
    vec![ChatCommand::OpenProcess { chat }]
}

/// Sends a command to the foreground chat. `None` with none, so the single-chat
/// path keeps the intent.
fn send_to_foreground(state: &AppState, rpc: RpcCommand) -> Option<Vec<ChatCommand>> {
    let chat = state.chats.foreground()?;
    Some(vec![ChatCommand::Send {
        chat: chat.clone(),
        rpc,
    }])
}

/// Sends the prompt to the foreground chat, one turn at a time, as the
/// single-chat path does. Answers `None` with no foreground chat, so the
/// single-chat path keeps the prompt.
///
/// A chat whose child died receives nothing: there is no process to write to, so
/// the prompt is not even appended to its thread and the user gets the Restart
/// notice instead (S3).
///
/// At the working cap (S7) the prompt is queued, never lost and never refused:
/// the user message is appended as today, the chat is marked Queued, and the
/// send goes out the moment a slot frees. A chat that already holds a queued
/// prompt refuses a second one, as a working chat refuses a prompt: one turn
/// at a time.
fn send_prompt(state: &mut AppState, text: &str) -> Option<Vec<ChatCommand>> {
    let chat = state.chats.foreground()?.clone();
    if text.trim().is_empty() {
        return Some(Vec::new());
    }
    if state.chats.is_ended(&chat) {
        push_death_notice(state, &chat);
        return Some(Vec::new());
    }
    // A chat whose child is coming up takes no prompt (composer disabled, prompt
    // kept): there is nothing to write to yet, exactly like working or queued.
    if state.chats.is_opening(&chat) {
        return Some(Vec::new());
    }
    if state.chats.is_working(&chat) || state.chats.is_queued(&chat) {
        return Some(Vec::new());
    }
    push_user_message(state.chats.chat_mut(&chat), text);
    if queue::has_room(&state.chats) {
        return Some(vec![ChatCommand::Send {
            chat,
            rpc: RpcCommand::Prompt {
                message: text.to_owned(),
            },
        }]);
    }
    queue::enqueue(&mut state.chats, chat, text.to_owned());
    Some(Vec::new())
}

/// Copies a message of the foreground chat, if it is that chat's message.
fn copy_message(state: &AppState, id: &str) -> Option<Vec<Command>> {
    let chat = state.chats.foreground()?;
    let message = state
        .chats
        .state(chat)?
        .messages
        .iter()
        .find(|message| message.id == id)?;
    Some(vec![Command::CopyToClipboard(message.text.clone())])
}

/// Answers a dialog the foreground chat is waiting on. A dialog another chat
/// raised is left waiting for when the user returns to it (S4).
fn answer_dialog(state: &mut AppState, id: &str, answer: &UiAnswer) -> Option<Vec<ChatCommand>> {
    let chat = state.chats.foreground()?.clone();
    let dialogs = state.chats.dialogs(&chat);
    if !dialogs.iter().any(|dialog| dialog_id(dialog) == id) {
        return Some(Vec::new());
    }
    let chat_state = state.chats.chat_mut(&chat);
    chat_state.pending_dialogs.retain(|d| dialog_id(d) != id);
    Some(vec![ChatCommand::Send {
        chat,
        rpc: RpcCommand::UiResponse {
            id: id.to_owned(),
            answer: answer.clone(),
        },
    }])
}
