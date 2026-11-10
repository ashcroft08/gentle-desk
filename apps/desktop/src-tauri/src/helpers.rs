//! The helpers slice of the contract, read from the app state (T7b-1).
//!
//! The foreground chat's delegated helpers, exactly as the iced panel draws
//! them (`crates/ui/src/view/helpers.rs`): one card per task, with the counters
//! line above it. Every text here comes from `gentle_runtime::helper_text`, so
//! both windows say the same thing about the same task.
//!
//! The task's THREAD is not here: the said/thought/ran view is T7b-2, and a
//! slice that carried the transcripts would push a helper's whole activity on
//! every event of the chat.

use gentle_app::AppState;
use gentle_core::helpers::{HelperStatus, HelpersActivity};
use gentle_runtime::helper_text::{status_label, summary_line, task_title};

use crate::contract::{HelperRowView, HelperStatusView, HelpersView};

/// The foreground chat's helpers, or `None` when no chat is foreground: a chat
/// with no helpers yet still answers a panel (the counters line and the empty
/// list), so only "no chat" is absent.
pub(crate) fn helpers(state: &AppState) -> Option<HelpersView> {
    let chat = state.chats.foreground()?;
    state.chats.state(chat).map(|chat| view(&chat.helpers))
}

/// One chat's activity as the panel shows it.
fn view(helpers: &HelpersActivity) -> HelpersView {
    let summary = helpers.summary;
    HelpersView {
        summary_line: summary_line(&summary),
        running: narrow(summary.running),
        queued: narrow(summary.queued),
        waiting: narrow(summary.waiting),
        finished: narrow(summary.finished),
        tasks: helpers.tasks.iter().map(row).collect(),
    }
}

/// One task as its card shows it. The title and the status text are the rules'
/// (`helper_text`), never re-spelled here.
fn row(task: &gentle_core::helpers::HelperTask) -> HelperRowView {
    HelperRowView {
        id: task.id.clone(),
        title: task_title(task),
        agent: task.agent.clone(),
        status: view_status(task.status),
        status_label: status_label(task.status).to_owned(),
        turns: narrow(task.turns),
        tool_calls: narrow(task.tool_calls),
        last_step: task.last_step.clone(),
        error: task.error.clone(),
    }
}

/// The contract's status enum matches the helper state's statuses one to one.
/// Shared with the helper thread slice, whose header carries the same status.
pub(crate) fn view_status(status: HelperStatus) -> HelperStatusView {
    match status {
        HelperStatus::Queued => HelperStatusView::Queued,
        HelperStatus::Running => HelperStatusView::Running,
        HelperStatus::Waiting => HelperStatusView::Waiting,
        HelperStatus::Done => HelperStatusView::Done,
        HelperStatus::Failed => HelperStatusView::Failed,
        HelperStatus::Cancelled => HelperStatusView::Cancelled,
    }
}

/// A counter the app keeps as `u64` in the contract's `u32`. The saturating
/// value would mark a clamp, never a silent alias: no chat delegates four
/// billion helpers.
pub(crate) fn narrow(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
