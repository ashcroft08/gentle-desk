//! The helpers (delegated subagents) panel as plain text: the status label, the
//! task title and the counters line both windows draw, plus, in [`format`],
//! the short text of one tool call.
//!
//! Nothing here draws: the iced panel (`crates/ui/src/view/helpers.rs` and
//! `crates/ui/src/view/helper_thread.rs`) and, later, the Tauri app both sit on
//! top of this module. The colour a status is drawn in stays with the window.

pub mod format;

use gentle_core::helpers::{HelperStatus, HelperTask, HelpersSummary};

/// "1 running · 0 queued · 0 waiting · 2 finished"
pub fn summary_line(summary: &HelpersSummary) -> String {
    format!(
        "{} running · {} queued · {} waiting · {} finished",
        summary.running, summary.queued, summary.waiting, summary.finished
    )
}

pub fn status_label(status: HelperStatus) -> &'static str {
    match status {
        HelperStatus::Queued => "queued",
        HelperStatus::Running => "running",
        HelperStatus::Waiting => "waiting",
        HelperStatus::Done => "done",
        HelperStatus::Failed => "failed",
        HelperStatus::Cancelled => "cancelled",
    }
}

/// The label if there is one, else the first line of the prompt, else the agent, else the id.
pub fn task_title(task: &HelperTask) -> String {
    let first_line = |text: &str| {
        text.lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(str::to_owned)
    };
    first_line(&task.label)
        .or_else(|| first_line(&task.prompt))
        .or_else(|| first_line(&task.agent))
        .unwrap_or_else(|| task.id.clone())
}

#[cfg(test)]
mod tests;
