//! The bridge's mapping of the web side's intents onto the reducer's.
//!
//! Explicit, so the contract stays free of `crates/app` types (S2), and kept
//! apart from the holder so the holder owns state and nothing about how one id
//! travels back.

use gentle_app::ChatId;

use crate::contract::UiIntent;

/// Maps one intent of the contract to the reducer's intent, `None` for the ones
/// the reducer has no answer for: folding a workspace's chats away, choosing the
/// theme and choosing the language are the config's business (D8), so the holder
/// answers them before asking this (`bridge::AppStateHolder::config_intent`).
pub fn to_intent(state: &gentle_app::AppState, intent: UiIntent) -> Option<gentle_app::Intent> {
    Some(match intent {
        UiIntent::SendPrompt { text } => gentle_app::Intent::SendPrompt(text),
        UiIntent::Abort => gentle_app::Intent::Abort,
        UiIntent::NewChat => gentle_app::Intent::NewChat,
        UiIntent::OpenChat { chat_id } => gentle_app::Intent::OpenChat(chat_for(state, &chat_id)),
        UiIntent::OpenNewChat => gentle_app::Intent::OpenNewChat,
        // The folder is not the reducer's business: `crates/app` carries no
        // folder at all (S2), so it travels beside the command in the bridge's
        // pending-folder table (`pending`), keyed by the chat allocated here.
        UiIntent::NewChatInWorkspace { .. } => gentle_app::Intent::OpenNewChat,
        UiIntent::SetWorkspaceCollapsed { .. } => return None,
        // The chosen theme is the app config's too (the v1 Appearance setting),
        // so the holder answers it before asking this.
        UiIntent::SetTheme { .. } => return None,
        // So is the chosen language (0.2.0, S4): a field of the config, answered
        // by `bridge::language` and re-emitted as the slice the web side draws.
        UiIntent::SetLanguage { .. } => return None,
        // So is the chosen home (shell-choice): a field of the config.
        UiIntent::SetShellHome { .. } => return None,
        // The relaunch is the bridge's: it needs the window handle.
        UiIntent::RestartApp => return None,
        // So are the install and the look for the launcher.
        UiIntent::InstallShell | UiIntent::CheckShellAgain => return None,
        UiIntent::RestartChat { chat_id } => {
            gentle_app::Intent::RestartChat(chat_for(state, &chat_id))
        }
        // Close chat names a chat the same way the open does, and the reducer
        // refuses a chat that is running a turn (the delete flow's own rule).
        UiIntent::CloseChat { chat_id } => gentle_app::Intent::CloseChat(chat_for(state, &chat_id)),
        // The message id is not a chat: the reducer finds it in the foreground
        // chat's thread and answers the copy command the shell runs.
        UiIntent::CopyMessage { message_id } => gentle_app::Intent::CopyMessage(message_id),
        UiIntent::DismissNotice { id } => gentle_app::Intent::DismissNotice(u64::from(id)),
        // The dialog id and its answer are the child's own protocol: the
        // reducer drops the dialog it just answered, so a repeated answer to
        // the same id reaches nothing.
        UiIntent::AnswerDialog { id, answer } => gentle_app::Intent::AnswerDialog {
            id,
            answer: ui_answer(answer),
        },
        // The helper id belongs to the foreground chat's helpers, which the
        // reducer checks, so an id that names no task selects nothing.
        UiIntent::SelectHelper { helper_id } => gentle_app::Intent::SelectHelper(helper_id),
        // The stop names one helper of the foreground chat, which the reducer
        // checks (running, this chat's own) before it asks anything of a child.
        UiIntent::StopHelper { helper_id } => gentle_app::Intent::StopHelper(helper_id),
        UiIntent::CloseHelper => gentle_app::Intent::CloseHelper,
        UiIntent::RequestModels => gentle_app::Intent::RequestModels,
        UiIntent::SelectModel { provider, model_id } => {
            gentle_app::Intent::SelectModel { provider, model_id }
        }
        UiIntent::SelectThinkingLevel { level } => {
            gentle_app::Intent::SelectThinkingLevel { level }
        }
        // The Settings panes' intents (0.1.12) are the shell's: they read a
        // file or run the shell's own check, so the bridge answers them before
        // it asks this (`Bridge::dispatch`).
        // `RefreshChatProfile` (profiles-editor T2b-2, L11) is the same: it asks
        // ONE chat's own child for its configuration, so the shell answers it
        // with a command of its own (`AppStateHolder::dispatch_commands`).
        UiIntent::RefreshChatProfile { .. }
        | UiIntent::SaveChatProfileMode { .. }
        | UiIntent::RefreshAuth
        | UiIntent::CheckUpdates
        | UiIntent::RefreshUsage => return None,
    })
}

/// The contract's answer shape as `gentle_core`'s, one to one.
fn ui_answer(answer: crate::contract::DialogAnswer) -> gentle_core::rpc::UiAnswer {
    use crate::contract::DialogAnswer;
    match answer {
        DialogAnswer::Value { value } => gentle_core::rpc::UiAnswer::Value(value),
        DialogAnswer::Confirmed { confirmed } => gentle_core::rpc::UiAnswer::Confirmed(confirmed),
        DialogAnswer::Cancelled => gentle_core::rpc::UiAnswer::Cancelled,
    }
}

/// The mode the contract names, as the channel spells it (`organic`,
/// `full-delegation`): the two enums carry the same values, so a write never
/// translates the string the store holds (profiles-editor T3-mode).
pub(crate) fn mode_of(
    mode: crate::contract::OrchestratorModeView,
) -> gentle_core::rpc::config_store::OrchestratorMode {
    use crate::contract::OrchestratorModeView;
    match mode {
        OrchestratorModeView::Organic => gentle_core::rpc::config_store::OrchestratorMode::Organic,
        OrchestratorModeView::FullDelegation => {
            gentle_core::rpc::config_store::OrchestratorMode::FullDelegation
        }
    }
}

/// The app's chat for an id the web side sent back.
///
/// `pub(crate)` because the Profile tab's read (profiles-editor T2b-2) names one
/// chat the same way: it is the ONE rule that reads an id back, so a chat the
/// view named travels back to the chat it belongs to.
///
/// The web only ever holds ids the state named - [`gentle_app::AppState`] names
/// every chat by `ChatId::as_str` and every session row by its path - so the
/// lookup is the round trip of an id the view was given, and a chat of no
/// session file (`new-N`, which no path can rebuild) travels back unchanged. An
/// id the state has not heard of is read as the session path of a stored chat,
/// which is how the app builds that chat's id: the sidebar names a row by its
/// path long before any chat exists.
pub(crate) fn chat_for(state: &gentle_app::AppState, chat_id: &str) -> ChatId {
    state
        .chats
        .ids()
        .into_iter()
        .find(|chat| chat.as_str() == chat_id)
        .cloned()
        .unwrap_or_else(|| ChatId::of_session(chat_id))
}
