//! The helpers (delegated subagents) panel: counters and one card per task.

use gentle_core::helpers::{HelperStatus, HelperTask, HelpersActivity};
use gentle_runtime::helper_text::{status_label, summary_line, task_title};
use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum Event {
    /// The user opened the thread of the task with this id.
    Select(String),
}

/// Whether there is anything to show; the panel stays out of the way otherwise.
pub fn is_visible(helpers: &HelpersActivity) -> bool {
    !helpers.tasks.is_empty()
}

pub fn view(helpers: &HelpersActivity) -> Element<'_, Event> {
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
    .width(340)
    .height(Length::Fill)
    .style(container::bordered_box)
    .into()
}

fn card(task: &HelperTask) -> Element<'_, Event> {
    let mut body = column![
        text(task_title(task)).size(14),
        text(status_label(task.status))
            .size(12)
            .style(status_style(task.status)),
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
        body = body.push(
            text(error)
                .size(11)
                .style(status_style(HelperStatus::Failed)),
        );
    }
    // A transparent button inside the same card the thread view uses, so the
    // card keeps the theme's own background and the status colours stay readable.
    container(
        button(body)
            .width(Length::Fill)
            .padding(0)
            .style(button::text)
            .on_press(Event::Select(task.id.clone())),
    )
    .padding(8)
    .width(Length::Fill)
    .style(container::rounded_box)
    .into()
}

/// The colour a status is drawn in; shared with the thread view.
pub fn status_style(status: HelperStatus) -> fn(&Theme) -> text::Style {
    match status {
        HelperStatus::Running => |_| colored(0x7a, 0xa2, 0xf7),
        HelperStatus::Waiting | HelperStatus::Queued => |_| colored(0xc0, 0xc5, 0xd0),
        HelperStatus::Done => |_| colored(0x7e, 0xdc, 0x9a),
        HelperStatus::Failed | HelperStatus::Cancelled => |_| colored(0xff, 0x7b, 0x7b),
    }
}

/// Fixed light tones: the theme's own success/primary colours are too dark on its dark cards.
fn colored(r: u8, g: u8, b: u8) -> text::Style {
    text::Style {
        color: Some(iced::Color::from_rgb8(r, g, b)),
    }
}

#[cfg(test)]
mod tests;
