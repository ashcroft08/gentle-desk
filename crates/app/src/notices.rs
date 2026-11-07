//! Extension `notify` messages as notices.

use gentle_core::rpc::{RpcEvent, UiRequest};

use crate::state::{AppState, Notice, NoticeLevel};

/// Most notices kept; older ones are dropped first.
const MAX_NOTICES: usize = 50;

pub(crate) fn push_notice(state: &mut AppState, level: NoticeLevel, text: String) {
    // Extensions repeat their notices (every session switch re-announces itself).
    if state
        .notices
        .iter()
        .any(|notice| notice.level == level && notice.text == text)
    {
        return;
    }
    state.notices.push(Notice {
        id: state.next_notice_id,
        level,
        text,
    });
    state.next_notice_id += 1;
    let excess = state.notices.len().saturating_sub(MAX_NOTICES);
    state.notices.drain(..excess);
}

/// The level and text of an extension `notify`; other fire-and-forget
/// requests (status bars, widgets, titles) are not notices.
pub(crate) fn notice_from(event: &RpcEvent) -> Option<(NoticeLevel, String)> {
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
