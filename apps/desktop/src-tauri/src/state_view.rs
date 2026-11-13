//! The `ViewState` slice of one `AppState`: the pure rule that decides what the
//! web side sees (S2).
//!
//! Every field is a slice the state really holds, never the whole `AppState`:
//! the foreground chat named as the app names it, the tail of its thread, the
//! first dialog its extension waits on, the notices flattened, the chats that
//! are not idle, its helpers panel and the open helper's thread, and the session
//! list as a status, a total and a revision instead of rows - the rows are
//! served one window at a time (S3). What the app state does NOT carry comes in
//! beside it ([`ShellParts`]): the config's notice, the fatal launcher failure
//! (the state has no process at all, `bridge::fatal`), the line a refused
//! workspace action left (`bridge::notice`), the images a chat holds or sent,
//! which the shell reads and the reducer never sees (`bridge::images`), the
//! version this run runs as, and the notes it follows (`bridge::whats_new`).

use gentle_app::{AppState, ModelsStatus};
use gentle_runtime::app_config::Theme;
use gentle_runtime::badge::{Badge, BadgeMark};

use crate::contract::{
    ChatActivityMark, ChatActivityView, ContextUsageView, ImageView, ModelChipView,
    ModelChoiceView, ModelListStatusView, NoticeActionView, NoticeLevel, NoticeView, ShellView,
    ThemeView, ViewState, WhatsNewView,
};
use crate::images::Images;
use crate::{dialogs, helper_thread, helpers, sessions, view};

/// Everything the slice carries that the app state does not: the shell's own
/// lines, the theme of this run, what the app found about the shell, and the
/// images the chats hold.
///
/// One value rather than six arguments, so [`view_state`] reads as what it is
/// (the state plus what the shell knows) instead of a wall of parameters.
pub struct ShellParts<'a> {
    /// What became of the config file: read, or not.
    pub config_notice: Option<String>,
    /// Why the last workspace action was refused.
    pub workspace_notice: Option<String>,
    /// Why the last session action was refused (T2's rename).
    pub sessions_notice: Option<String>,
    /// The notes of the version this run follows (what's-new S3b), `None` until
    /// the fetch answers with this version and after the modal is dismissed.
    pub whats_new: Option<WhatsNewView>,
    /// The version this run runs as (settings-dialog S5), `None` until the
    /// bridge names it at startup: the About section draws it, and it is what
    /// the dismiss stores as the last run's.
    pub app_version: Option<String>,
    /// The run's one fatal launcher failure.
    pub fatal: Option<String>,
    /// The theme this run holds, as the app config remembers it.
    pub theme: Theme,
    /// The shell and home this run sees.
    pub shell: ShellView,
    /// The images each chat holds or sent.
    pub images: &'a Images,
}

/// The slice of `state` as it stands now, carrying the revision the holder
/// keeps for the session list (the web side compares it to know when the page of
/// rows it holds is stale) and everything the shell owns ([`ShellParts`]).
pub fn view_state(state: &AppState, sessions_revision: u32, shell: ShellParts<'_>) -> ViewState {
    let ShellParts {
        config_notice,
        workspace_notice,
        sessions_notice,
        whats_new,
        app_version,
        fatal,
        theme,
        shell,
        images,
    } = shell;
    let helpers_view = helpers::helpers(state);
    let foreground = state
        .chats
        .foreground()
        .map(|chat| chat.as_str().to_owned());
    ViewState {
        theme: view_theme(theme),
        shell,
        foreground_chat: foreground.clone(),
        foreground_title: state
            .chats
            .foreground()
            .map(|chat| sessions::view::header_title(state, chat)),
        list_status: sessions::view::status(state),
        sessions_revision,
        total_sessions: sessions::view::total(state),
        messages: view::messages(state, images),
        notices: state.notices.iter().map(NoticeView::of).collect(),
        dialog: dialogs::dialog(state),
        helpers: helpers_view,
        selected_helper: state.selected_helper.clone(),
        helper_thread: helper_thread::thread(state),
        activity: activity_view(state),
        images: foreground
            .map(|chat| {
                images
                    .pending(&chat)
                    .into_iter()
                    .map(ImageView::from)
                    .collect()
            })
            .unwrap_or_default(),
        images_notice: images.notice(),
        model_chip: model_chip_view(state),
        context_usage: context_usage_view(state),
        config_notice,
        workspace_notice,
        sessions_notice,
        whats_new,
        app_version,
        fatal,
    }
}

/// The composer's model chip from the app state, one to one. A no-foreground
/// state shows nothing, so every field is simply the app's value.
fn model_chip_view(state: &AppState) -> ModelChipView {
    ModelChipView {
        status: match &state.models_status {
            ModelsStatus::Unknown => ModelListStatusView::Unknown,
            ModelsStatus::Loading => ModelListStatusView::Loading,
            ModelsStatus::Ready => ModelListStatusView::Ready,
            ModelsStatus::Empty => ModelListStatusView::Empty,
            ModelsStatus::Failed(error) => ModelListStatusView::Failed(error.clone()),
        },
        models: state
            .models
            .iter()
            .map(|m| ModelChoiceView {
                provider: m.provider.clone(),
                model_id: m.model_id.clone(),
                name: m.name.clone(),
                reasoning: m.reasoning,
            })
            .collect(),
        current: state.model.as_ref().map(|m| ModelChoiceView {
            provider: m.provider.clone(),
            model_id: m.model_id.clone(),
            name: m.name.clone(),
            reasoning: m.reasoning,
        }),
        thinking_level: state.thinking_level.clone(),
        thinking_levels: state.thinking_levels.clone(),
    }
}

/// How full the foreground chat's context is (0.1.12, S5): the reading its own
/// child last reported. No foreground chat, or a chat that has not answered yet,
/// is every field null - which the chip draws as nothing, never as `0%`.
fn context_usage_view(state: &AppState) -> ContextUsageView {
    let usage = state
        .chats
        .foreground()
        .and_then(|chat| state.chats.context(chat));
    match usage {
        // The contract narrows both counts to `u32` (a model's window and a
        // session's token count are far below it); a number that somehow did not
        // fit is unknown rather than a wrapped lie.
        Some(usage) => ContextUsageView {
            tokens: usage.tokens.and_then(|tokens| u32::try_from(tokens).ok()),
            percent: usage.percent,
            window: usage
                .context_window
                .and_then(|window| u32::try_from(window).ok()),
        },
        None => ContextUsageView::default(),
    }
}

/// The theme as the contract names it: the two enums carry the same six names,
/// so the value never travels as a string that could drift from the stylesheet.
fn view_theme(theme: Theme) -> ThemeView {
    match theme {
        Theme::System => ThemeView::System,
        Theme::Dark => ThemeView::Dark,
        Theme::Gentle => ThemeView::Gentle,
        Theme::GentlemanCute => ThemeView::GentlemanCute,
        Theme::GentlemanSexy => ThemeView::GentlemanSexy,
        Theme::Light => ThemeView::Light,
    }
}

/// The marks of the chats with something to show, in id order. The rule that
/// turns a chat's activity into a mark is `crates/runtime`'s (`Badge::on`), so
/// an idle chat is simply absent and this list stays as small as the chats that
/// are working, waiting or failed.
fn activity_view(state: &AppState) -> Vec<ChatActivityView> {
    state
        .chats
        .ids()
        .into_iter()
        .filter_map(|chat| {
            let badge = Badge::on(state.chats.activity(chat)?)?;
            Some(ChatActivityView {
                chat_id: chat.as_str().to_owned(),
                activity: view_mark(badge.mark),
            })
        })
        .collect()
}

/// The badge mark as the contract names it: the two enums carry the same six
/// meanings, and the contract's names are the ones already on the wire.
fn view_mark(mark: BadgeMark) -> ChatActivityMark {
    match mark {
        BadgeMark::Working => ChatActivityMark::Working,
        BadgeMark::NewReply => ChatActivityMark::NewReply,
        BadgeMark::Queued => ChatActivityMark::Queued,
        BadgeMark::Opening => ChatActivityMark::Opening,
        BadgeMark::NeedsAttention => ChatActivityMark::NeedsAttention,
        BadgeMark::Ended => ChatActivityMark::Ended,
    }
}

impl NoticeView {
    /// Flattens one app notice for display.
    pub(crate) fn of(notice: &gentle_app::Notice) -> Self {
        Self {
            id: view_notice_id(notice),
            text: notice.text.clone(),
            level: view_notice_level(notice.level),
            action: notice.action.as_ref().map(view_action),
        }
    }
}

/// The one action a notice may offer, as the contract names it. The chat id is
/// the one the restart intent takes back, exactly as `foreground_chat` carries
/// it, so a background chat's notice fixes that chat.
fn view_action(action: &gentle_app::NoticeAction) -> NoticeActionView {
    match action {
        gentle_app::NoticeAction::RestartChat(chat) => NoticeActionView::RestartChat {
            chat_id: chat.as_str().to_owned(),
        },
    }
}

/// The id the dismiss intent refers back to. The app's ids are a never-reused
/// counter that stays far below `u32::MAX` in practice; the contract narrows
/// both sides of the id to `u32`, and the saturating value would mark a
/// collapse, never silently aliasing another notice (each id is unique).
fn view_notice_id(notice: &gentle_app::Notice) -> u32 {
    u32::try_from(notice.id).unwrap_or(u32::MAX)
}

/// The contract's level enum matches the app's notice kinds one to one.
fn view_notice_level(level: gentle_app::NoticeLevel) -> NoticeLevel {
    match level {
        gentle_app::NoticeLevel::Info => NoticeLevel::Info,
        gentle_app::NoticeLevel::Warning => NoticeLevel::Warning,
        gentle_app::NoticeLevel::Error => NoticeLevel::Error,
    }
}
