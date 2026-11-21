//! The bridge's mapping of the web side's intents onto the reducer's.
//!
//! Explicit, so the contract stays free of `crates/app` types (S2), and kept
//! apart from the holder so the holder owns state and nothing about how one id
//! travels back.

use gentle_app::ChatId;

use crate::contract::UiIntent;

/// Maps one intent of the contract to the reducer's intent, `None` for the ones
/// the reducer has no answer for: folding a workspace's chats away, choosing the
/// theme, the language and the notifications mode are the config's business
/// (D8), so the holder answers them before asking this
/// (`bridge::AppStateHolder::config_intent`).
pub fn to_intent(state: &gentle_app::AppState, intent: &UiIntent) -> Option<gentle_app::Intent> {
    Some(match intent {
        UiIntent::SendPrompt { text } => gentle_app::Intent::SendPrompt(text.clone()),
        UiIntent::Abort => gentle_app::Intent::Abort,
        UiIntent::NewChat => gentle_app::Intent::NewChat,
        UiIntent::OpenChat { chat_id } => gentle_app::Intent::OpenChat(chat_for(state, chat_id)),
        UiIntent::OpenNewChat => gentle_app::Intent::OpenNewChat,
        // The composer's own ask for a new chat's child (welcome-screen T4, R2).
        // It names a chat the state already allocated, and the reducer is what
        // makes it fire once: a second ask for the same chat answers nothing.
        UiIntent::EnsureStarted { chat_id } => {
            gentle_app::Intent::EnsureStarted(chat_for(state, chat_id))
        }
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
        // So is the notifications mode (notifications S4): a field of the config,
        // answered by `bridge::notifications` the same way.
        UiIntent::SetNotifications { .. } => return None,
        // So is the chosen home (shell-choice): a field of the config.
        UiIntent::SetShellHome { .. } => return None,
        // And so is the version the user skipped (0.8.0 settings-polish S1): a
        // field of the config, answered by `bridge::settings` the same way.
        UiIntent::SkipUpdate { .. } | UiIntent::UnskipUpdate => return None,
        // The relaunch is the bridge's: it needs the window handle.
        UiIntent::RestartApp => return None,
        // So are the install and the look for the launcher.
        UiIntent::InstallShell | UiIntent::CheckShellAgain => return None,
        UiIntent::RestartChat { chat_id } => {
            gentle_app::Intent::RestartChat(chat_for(state, chat_id))
        }
        // Close chat names a chat the same way the open does, and the reducer
        // refuses a chat that is running a turn (the delete flow's own rule).
        UiIntent::CloseChat { chat_id } => gentle_app::Intent::CloseChat(chat_for(state, chat_id)),
        // The message id is not a chat: the reducer finds it in the foreground
        // chat's thread and answers the copy command the shell runs.
        UiIntent::CopyMessage { message_id } => gentle_app::Intent::CopyMessage(message_id.clone()),
        UiIntent::DismissNotice { id } => gentle_app::Intent::DismissNotice(u64::from(*id)),
        // The dialog id and its answer are the child's own protocol: the
        // reducer drops the dialog it just answered, so a repeated answer to
        // the same id reaches nothing.
        UiIntent::AnswerDialog { id, answer } => gentle_app::Intent::AnswerDialog {
            id: id.clone(),
            answer: ui_answer(answer.clone()),
        },
        // The helper id belongs to the foreground chat's helpers, which the
        // reducer checks, so an id that names no task selects nothing.
        UiIntent::SelectHelper { helper_id } => gentle_app::Intent::SelectHelper(helper_id.clone()),
        // The stop names one helper of the foreground chat, which the reducer
        // checks (running, this chat's own) before it asks anything of a child.
        UiIntent::StopHelper { helper_id } => gentle_app::Intent::StopHelper(helper_id.clone()),
        UiIntent::CloseHelper => gentle_app::Intent::CloseHelper,
        UiIntent::RequestModels => gentle_app::Intent::RequestModels,
        UiIntent::SelectModel { provider, model_id } => {
            gentle_app::Intent::SelectModel {
                provider: provider.clone(),
                model_id: model_id.clone(),
            }
        }
        UiIntent::SelectThinkingLevel { level } => {
            gentle_app::Intent::SelectThinkingLevel { level: level.clone() }
        }
        // The Settings panes' intents (0.1.12) are the shell's: they read a
        // file or run the shell's own check, so the bridge answers them before
        // it asks this (`Bridge::dispatch`).
        // `RefreshChatProfile` (profiles-editor T2b-2, L11) is the same: it asks
        // ONE chat's own child for its configuration, so the shell answers it
        // with a command of its own (`AppStateHolder::dispatch_commands`).
        UiIntent::RefreshChatProfile { .. }
        | UiIntent::SaveChatProfileMode { .. }
        | UiIntent::ApplyChatProfile { .. }
        | UiIntent::SaveChatProfileChain { .. }
        | UiIntent::SaveChatProfilePin { .. }
        // So are the two session-mode intents (composer-dock T3b, L7): the mode
        // is that chat's OWN child's, asked through a per-chat command of the
        // shell's own (`AppStateHolder::dispatch_commands`).
        | UiIntent::ReadSessionMode { .. }
        | UiIntent::SetSessionMode { .. }
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

/// The session mode the contract names, as the channel spells it (`normal`,
/// `auto`, `yolo`, `super-yolo`): the two enums carry the same four values, so a
/// request never translates the mode it asks for (composer-dock T3b, L7).
///
/// The closed choice is the contract's and can only name a mode that exists;
/// what the owner REPORTS is kept raw in [`SessionModeView`](crate::contract::SessionModeView)
/// and is never mapped through this, because a mode this build does not know must
/// still be shown as it arrived.
pub(crate) fn session_mode_of(
    mode: crate::contract::SessionModeChoice,
) -> gentle_core::rpc::permission::SessionMode {
    use crate::contract::SessionModeChoice;
    use gentle_core::rpc::permission::SessionMode;
    match mode {
        SessionModeChoice::Normal => SessionMode::Normal,
        SessionModeChoice::Auto => SessionMode::Auto,
        SessionModeChoice::Yolo => SessionMode::Yolo,
        SessionModeChoice::SuperYolo => SessionMode::SuperYolo,
    }
}

/// The apply mode the contract names, as the channel spells it (`session`,
/// `default`): the two enums carry the same values, so no request translates the
/// mode it asks for (profiles-editor T3b).
pub(crate) fn apply_mode_of(
    mode: crate::contract::ApplyModeView,
) -> gentle_core::rpc::profile_apply::ApplyMode {
    use crate::contract::ApplyModeView;
    match mode {
        ApplyModeView::Session => gentle_core::rpc::profile_apply::ApplyMode::Session,
        ApplyModeView::Default => gentle_core::rpc::profile_apply::ApplyMode::Default,
    }
}

/// The confirmation token of what the page showed, or `None` when the string is
/// no token at all (empty, or longer than the channel's own 128 characters).
///
/// A string that is not a token is DROPPED rather than sent: the apply is then
/// asked again without one, which is exactly the rule that an answer other than
/// what was shown re-asks with a fresh token and never writes (profiles-editor
/// T3b, L30).
pub(crate) fn token_of(
    confirmation: Option<&str>,
) -> Option<gentle_core::rpc::profile_apply::ConfirmationToken> {
    confirmation.and_then(gentle_core::rpc::profile_apply::ConfirmationToken::parse)
}

/// The chain the page built, as the store's own steps (profiles-editor T5): the
/// model it typed and the effort it chose, where `None` is NO effort rather than
/// an empty one - the store refuses an empty `thinking`, and the page's empty
/// input means "no effort".
pub(crate) fn chain_steps(
    steps: &[crate::contract::ChatProfileStepView],
) -> Vec<gentle_core::rpc::config_store::FallbackStep> {
    steps
        .iter()
        .map(|step| gentle_core::rpc::config_store::FallbackStep {
            model: step.model.clone(),
            thinking: step.thinking.clone(),
        })
        .collect()
}

/// The pin layer the page asked to write, or `None` for a string that names no
/// layer at all (profiles-editor T6).
///
/// The channel names exactly two (`local`, `repo`, `desktop-rpc.md:322-345`), so
/// anything else is a page bug and is DROPPED rather than guessed at: no request
/// leaves for a layer that does not exist, and no default is inferred.
pub(crate) fn pin_source_of(source: &str) -> Option<gentle_core::rpc::config_store::PinSource> {
    gentle_core::rpc::config_store::PinSource::parse(source)
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
