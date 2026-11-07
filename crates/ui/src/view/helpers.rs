//! The helpers (delegated subagents) panel: counters and one card per task.

use gentle_core::helpers::{HelperStatus, HelperTask, HelpersActivity, HelpersSummary};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length, Theme};

/// Whether there is anything to show; the panel stays out of the way otherwise.
pub fn is_visible(helpers: &HelpersActivity) -> bool {
    !helpers.tasks.is_empty()
}

/// Draws the panel. It emits no events, so it fits any parent message type.
pub fn view<'a, M: 'a>(helpers: &'a HelpersActivity) -> Element<'a, M> {
    let cards = helpers.tasks.iter().map(card);
    container(
        column![
            text("Helpers").size(16),
            text(summary_line(&helpers.summary)).size(12),
            scrollable(column(cards).spacing(8)).height(Length::Fill),
        ]
        .spacing(8)
        .padding(12),
    )
    .width(300)
    .height(Length::Fill)
    .style(container::bordered_box)
    .into()
}

fn card<'a, M: 'a>(task: &'a HelperTask) -> Element<'a, M> {
    let status_style: fn(&Theme) -> text::Style = match task.status {
        HelperStatus::Running => text::primary,
        HelperStatus::Waiting | HelperStatus::Queued => text::secondary,
        HelperStatus::Done => text::success,
        HelperStatus::Failed | HelperStatus::Cancelled => text::danger,
    };
    let mut body = column![
        text(task_title(task)).size(14),
        text(status_label(task.status)).size(12).style(status_style),
        text(format!(
            "{} · {} turns · {} tool calls",
            task.agent, task.turns, task.tool_calls
        ))
        .size(11),
    ]
    .spacing(2);
    if let Some(step) = &task.last_step {
        body = body.push(text(step).size(11));
    }
    if let Some(error) = &task.error {
        body = body.push(text(error).size(11).style(text::danger));
    }
    container(body)
        .padding(8)
        .width(Length::Fill)
        .style(container::rounded_box)
        .into()
}

/// "1 running · 0 queued · 0 waiting · 2 finished"
fn summary_line(summary: &HelpersSummary) -> String {
    format!(
        "{} running · {} queued · {} waiting · {} finished",
        summary.running, summary.queued, summary.waiting, summary.finished
    )
}

fn status_label(status: HelperStatus) -> &'static str {
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
fn task_title(task: &HelperTask) -> String {
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
