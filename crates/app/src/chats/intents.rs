//! The intents that belong to one chat rather than to the single open thread.
//!
//! These are additive: they exist next to the single-chat intents and only act
//! when the intent names a chat (or when there is a foreground chat to act on).
//! The one-child shell never produces them, so nothing it does changes.

use gentle_core::helpers::HelperStatus;
use gentle_core::rpc::{RpcCommand, UiAnswer};

use super::queue;
use super::ChatId;
use crate::chats::{close_working, push_death_notice};
use crate::input::{ChatCommand, Command, Effects, Intent};
use crate::intents::dialog_id;
use crate::notices::{drop_restart_notice, push_notice};
use crate::state::{push_user_message, AppState, ModelsStatus, NoticeLevel};

/// Applies `intent` if it belongs to the per-chat model, answering the effects
/// for one chat's child (and the shared ones it asks for). Answers `None` when
/// it is the single-chat intent.
pub(crate) fn apply(state: &mut AppState, intent: &Intent) -> Option<Effects> {
    match intent {
        Intent::OpenChat(chat) => Some(for_chat(open(state, chat.clone()))),
        Intent::OpenNewChat => Some(for_chat(allocate(state, ChatId::fresh()))),
        Intent::EnsureStarted(chat) => Some(for_chat(ensure_started(state, chat))),
        Intent::RestartChat(chat) => Some(for_chat(open(state, chat.clone()))),
        Intent::CloseChat(chat) => Some(for_chat(close_chat(state, chat))),
        Intent::CloseAllChats => Some(for_chat(close_all_chats(state))),
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
        Intent::SelectHelper(id) => select_helper(state, id),
        Intent::StopHelper(id) => stop_helper(state, id),
        // The chip's three asks go to the foreground chat's child, like any
        // other prompt (0.1.11): the one-child send the fallthrough would
        // produce is dropped by the chats shell.
        Intent::RequestModels => {
            state.models_status = ModelsStatus::Loading;
            send_to_foreground(state, RpcCommand::GetAvailableModels).map(for_chat)
        }
        Intent::SelectModel { provider, model_id } => send_to_foreground(
            state,
            RpcCommand::SetModel {
                provider: provider.clone(),
                model_id: model_id.clone(),
            },
        )
        .map(for_chat),
        Intent::SelectThinkingLevel { level } => {
            state.thinking_level = Some(level.clone());
            send_to_foreground(
                state,
                RpcCommand::SetThinkingLevel {
                    level: level.clone(),
                },
            )
            .map(for_chat)
        }
        Intent::CloseHelper => close_helper(state),
        _ => None,
    }
}

/// Selects a helper of the FOREGROUND chat, named by the id the panel drew. A
/// task of another chat is not a candidate, exactly as a helper that is not
/// listed is not: an id that names no task leaves the selection as it was.
/// `None` with no foreground chat, so the single-chat rule keeps the intent.
fn select_helper(state: &mut AppState, id: &str) -> Option<Effects> {
    let chat = state.chats.foreground()?;
    let known = state
        .chats
        .state(chat)
        .is_some_and(|chat| chat.helpers.tasks.iter().any(|task| task.id == id));
    if known {
        state.selected_helper = Some(id.to_owned());
    }
    Some(for_chat(Vec::new()))
}

/// Back to the list of the foreground chat; `None` with no foreground chat,
/// like [`select_helper`].
fn close_helper(state: &mut AppState) -> Option<Effects> {
    state.chats.foreground()?;
    state.selected_helper = None;
    Some(for_chat(Vec::new()))
}

/// Stops one helper of the FOREGROUND chat (helper-stop S2), named by the id the
/// panel drew.
///
/// Only a helper that is RUNNING can be stopped: that is exactly when the thread
/// bar's square is enabled, and a stop of a queued, waiting or finished helper
/// would ask the child to cancel something that is not working. Every other id -
/// one this chat never listed, or a task of another chat - changes nothing, the
/// same rule [`select_helper`] follows for an unknown id: no command, no notice.
///
/// The answer is one command for THIS chat's own child (S6): the chat keeps
/// running and its other helpers are untouched (S1). Whether the child accepted
/// comes back as [`Input::HelperStopped`](crate::Input::HelperStopped), which
/// marks that helper; a stop that did not happen sends nothing (S7).
fn stop_helper(state: &mut AppState, id: &str) -> Option<Effects> {
    let chat = state.chats.foreground()?.clone();
    let running = state.chats.state(&chat).is_some_and(|chat| {
        chat.helpers
            .tasks
            .iter()
            .any(|task| task.id == id && task.status == HelperStatus::Running)
    });
    if !running {
        return Some(for_chat(Vec::new()));
    }
    Some(for_chat(vec![ChatCommand::StopHelper {
        chat,
        helper_id: id.to_owned(),
    }]))
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

/// Everything an open does to the chat ITSELF, and nothing about its child: it
/// becomes the foreground one (which clears the death and the stderr an old
/// child left), the helper panel's selection goes - a selection never names a
/// task of a chat the user left - and the chip's data is forgotten, because it
/// belongs to the previous chat's child.
fn present(state: &mut AppState, chat: &ChatId) {
    drop_restart_notice(state, chat);
    state.chats.open(chat.clone());
    state.selected_helper = None;
    state.models = Vec::new();
    state.models_status = ModelsStatus::Unknown;
    state.model = None;
    state.thinking_level = None;
    state.thinking_levels = Vec::new();
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
    present(state, &chat);
    // The ask is made here, so this chat's one start is behind it.
    state.chats.mark_started(&chat);
    vec![ChatCommand::OpenProcess { chat }]
}

/// Allocates a NEW chat and asks for NO child (welcome-screen T4, R2): the chat
/// is the foreground one, and it is [`ChatActivity::Opening`] from here on - so
/// its composer keeps the state and the reason it has today, and an Enter in it
/// sends nothing while the draft stays where it is - but the process itself
/// starts on the composer's own ask ([`Intent::EnsureStarted`]). Opening a chat
/// at launch therefore costs no child at all, and the seconds it takes to come
/// up are spent when the user looks at the box, never while the app starts.
fn allocate(state: &mut AppState, chat: ChatId) -> Vec<ChatCommand> {
    present(state, &chat);
    state.chats.mark_opening(&chat);
    Vec::new()
}

/// Starts `chat`'s child if it was never asked for, and answers nothing at all
/// when it was (welcome-screen T4, R2).
///
/// This is the composer's own ask - the first time its box is focused for that
/// chat, or the first key that lands in it - and it is idempotent by the state:
/// a chat the app started already (Opening, working, queued, idle or ended)
/// sends no command and changes nothing, so the focus and the first keystroke of
/// one box can never start two children. The chat is NOT made the foreground one
/// here: it is the chat the composer belongs to, and moving the foreground is the
/// open's own act, never a side effect of a keystroke.
fn ensure_started(state: &mut AppState, chat: &ChatId) -> Vec<ChatCommand> {
    if state.chats.is_started(chat) {
        return Vec::new();
    }
    state.chats.mark_started(chat);
    vec![ChatCommand::OpenProcess { chat: chat.clone() }]
}

/// Closes `chat` (0.1.14): its child is stopped and the chat leaves the open
/// set, while the session list keeps its row and the file on disk is untouched.
///
/// A chat that is running a turn is refused with the delete flow's own rule and
/// words ([`close_working`], the same sentence `chat_delete` refuses with): its
/// child is mid-turn, and stopping it is the user's (S4), so the row menu asks
/// for that first rather than pulling the child out from under the turn.
fn close_chat(state: &mut AppState, chat: &ChatId) -> Vec<ChatCommand> {
    match close_if_idle(state, chat) {
        Some(commands) => commands,
        None => {
            let refusal = close_working(state.language());
            push_notice(state, NoticeLevel::Warning, refusal);
            Vec::new()
        }
    }
}

/// Closes EVERY open chat at once (close-chats T1, S2, S3), each one through the
/// very close `CloseChat` takes ([`close_if_idle`]: the stop command, the removal
/// from the open set, the closed set that keeps it closed and the foreground
/// fallback), and with the same guard: a chat that is working is SKIPPED,
/// silently. That is deliberate - the turn in flight is the user's to stop, and
/// this action only closes what is not running - so a skipped chat answers no
/// command, no notice and no change at all, and the caller reads the count the
/// open set still holds (close-chats S3's sentence).
///
/// The order is the open set's own id order with the FOREGROUND chat LAST: closing
/// a chat clears the foreground mark, so closing the one in front first would
/// leave the view with no chat to draw while the others go. Last, the user keeps
/// a chat in front for every step and the slice ends where a single close ends,
/// with no foreground chat at all.
fn close_all_chats(state: &mut AppState) -> Vec<ChatCommand> {
    let mut commands = Vec::new();
    for chat in closing_order(state) {
        if let Some(close) = close_if_idle(state, &chat) {
            commands.extend(close);
        }
    }
    commands
}

/// The open chats of `state` in the order a bulk close takes them: a stable id
/// order with the foreground chat last (see [`close_all_chats`]).
fn closing_order(state: &AppState) -> Vec<ChatId> {
    let foreground = state.chats.foreground().cloned();
    let mut chats: Vec<ChatId> = state.chats.ids().into_iter().cloned().collect();
    // `false` sorts first, so only the foreground entry moves to the end.
    chats.sort_by_key(|chat| foreground.as_ref() == Some(chat));
    chats
}

/// The close of one chat with its guard already answered: `None` when the chat
/// is working, because which words say so - or nothing at all - is the caller's
/// rule, and never a second copy of the close itself.
fn close_if_idle(state: &mut AppState, chat: &ChatId) -> Option<Vec<ChatCommand>> {
    if state.chats.is_working(chat) {
        return None;
    }
    // A chat this run never opened has no child to stop and no place to leave,
    // so it answers no command.
    if !state.chats.close(chat) {
        return Some(Vec::new());
    }
    Some(vec![ChatCommand::CloseProcess { chat: chat.clone() }])
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
        .text_items()
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
