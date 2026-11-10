//! A helper's thread: what it said, thought and ran.

use gentle_core::helpers::{HelperTask, HelperThreadItem};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Font, Length};

use super::helpers::status_style;
use gentle_runtime::helper_text::format::{args_line, output_preview, tool_status, ToolStatus};
use gentle_runtime::helper_text::{status_label, task_title};
use gentle_runtime::text::truncate;

/// Longest prompt echoed at the top, in characters.
const PROMPT_CHARS: usize = 300;

#[derive(Debug, Clone)]
pub enum Event {
    /// Back to the helpers list.
    Back,
}

pub fn view(task: &HelperTask) -> Element<'_, Event> {
    let header = row![
        button(text("← Helpers").size(13))
            .style(button::text)
            .on_press(Event::Back),
        text(task_title(task)).size(15),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Fill the width (minus the scrollbar) so long lines wrap instead of being clipped.
    let mut items = column![].spacing(8).width(Length::Fill).padding([0, 14]);
    if task.thread.dropped > 0 {
        items = items.push(
            text(format!(
                "{} earlier items were dropped",
                task.thread.dropped
            ))
            .size(11)
            .style(text::secondary),
        );
    }
    if task.thread.items.is_empty() {
        items = items.push(text("No activity yet").size(12).style(text::secondary));
    }
    for item in &task.thread.items {
        items = items.push(item_view(item));
    }

    container(
        column![
            header,
            text(status_label(task.status))
                .size(12)
                .style(status_style(task.status)),
            text(truncate(&task.prompt, PROMPT_CHARS))
                .size(11)
                .style(text::secondary),
            scrollable(items).anchor_bottom().height(Length::Fill),
        ]
        .spacing(8)
        .padding(12),
    )
    .width(340)
    .height(Length::Fill)
    .style(container::bordered_box)
    .into()
}

fn item_view(item: &HelperThreadItem) -> Element<'_, Event> {
    match item {
        HelperThreadItem::Text(body) => text(body).size(13).into(),
        HelperThreadItem::Thinking(body) => text(format!("thinking: {body}"))
            .size(12)
            .style(text::secondary)
            .into(),
        HelperThreadItem::Note(body) => text(format!("note: {body}"))
            .size(12)
            .style(text::secondary)
            .into(),
        HelperThreadItem::Tool {
            name,
            args,
            output,
            running,
            is_error,
            ..
        } => {
            let status = tool_status(*running, *is_error);
            let (label, style): (&str, fn(&iced::Theme) -> text::Style) = match status {
                ToolStatus::Running => ("running", text::primary),
                ToolStatus::Failed => ("failed", text::danger),
                ToolStatus::Ok => ("ok", text::success),
            };
            let mut body = column![
                row![text(name).size(13), text(label).size(11).style(style)].spacing(8),
                text(args_line(args)).size(11).font(Font::MONOSPACE),
            ]
            .spacing(2);
            let preview = output_preview(output);
            if !preview.is_empty() {
                body = body.push(
                    text(preview)
                        .size(11)
                        .font(Font::MONOSPACE)
                        .style(text::secondary),
                );
            }
            container(body)
                .padding(6)
                .width(Length::Fill)
                .style(container::rounded_box)
                .into()
        }
    }
}
