//! The `ViewState` slice of one `AppState`: the pure rule that decides what the
//! web side sees (S2).
//!
//! Every field is a slice the state really holds, never the whole `AppState`:
//! the foreground chat named as the app names it, the tail of its thread, the
//! first dialog its extension waits on, the notices flattened, the chats that
//! are not idle, its helpers panel and the open helper's thread, and the session
//! list as a status, a total and a revision instead of rows - the rows are
//! served one window at a time (S3). The fatal launcher failure is the one
//! shell-owned line here, because the app state has no process at all
//! (`bridge::fatal`).

use gentle_app::AppState;
use gentle_runtime::app_config::Theme;
use gentle_runtime::badge::{Badge, BadgeMark};

use crate::contract::{
    ChatActivityMark, ChatActivityView, NoticeActionView, NoticeLevel, NoticeView, ThemeView,
    ViewState,
};
use crate::{dialogs, helper_thread, helpers, sessions, view};

/// The slice of `state` as it stands now, carrying the revision the holder
/// keeps for the session list - the web side compares it to know when the page
/// of rows it holds is stale - the theme this run holds, the notice of the
/// config file, when it could not be used at startup, the reason the last
/// workspace action refused, and the one fatal launcher failure of the run.
pub fn view_state(
    state: &AppState,
    sessions_revision: u32,
    config_notice: Option<String>,
    workspace_notice: Option<String>,
    fatal: Option<String>,
    theme: Theme,
) -> ViewState {
    let helpers_view = helpers::helpers(state);
    ViewState {
        theme: view_theme(theme),
        foreground_chat: state
            .chats
            .foreground()
            .map(|chat| chat.as_str().to_owned()),
        foreground_title: state
            .chats
            .foreground()
            .map(|chat| sessions::view::header_title(state, chat)),
        list_status: sessions::view::status(state),
        sessions_revision,
        total_sessions: sessions::view::total(state),
        messages: view::messages(state),
        notices: state.notices.iter().map(NoticeView::of).collect(),
        dialog: dialogs::dialog(state),
        helpers: helpers_view,
        selected_helper: state.selected_helper.clone(),
        helper_thread: helper_thread::thread(state),
        activity: activity_view(state),
        config_notice,
        workspace_notice,
        fatal,
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
