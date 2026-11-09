//! The `ViewState` slice of one `AppState`: the pure rule that decides what the
//! web side sees (S2).
//!
//! Every field is a slice the state really holds, never the whole `AppState`:
//! the foreground chat named as the app names it, the tail of its thread, the
//! notices flattened, the chats that are not idle, and the session list as a
//! status, a total and a revision instead of rows - the rows are served one
//! window at a time (S3).

use gentle_app::AppState;
use gentle_runtime::badge::{Badge, BadgeMark};

use crate::contract::{ChatActivityMark, ChatActivityView, NoticeLevel, NoticeView, ViewState};
use crate::{sessions, view};

/// The slice of `state` as it stands now, carrying the revision the holder
/// keeps for the session list: the web side compares it to know when the page
/// of rows it holds is stale.
pub fn view_state(state: &AppState, sessions_revision: u32) -> ViewState {
    ViewState {
        foreground_chat: state
            .chats
            .foreground()
            .map(|chat| chat.as_str().to_owned()),
        list_status: sessions::view::status(state),
        sessions_revision,
        total_sessions: sessions::view::total(state),
        messages: view::messages(state),
        notices: state.notices.iter().map(NoticeView::of).collect(),
        activity: activity_view(state),
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
        }
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
