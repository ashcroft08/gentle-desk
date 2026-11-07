//! A blocking extension dialog (confirm, select, input, editor).

use gentle_core::chat::Dialog;
use gentle_core::rpc::UiAnswer;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Event {
    Answer {
        id: String,
        answer: UiAnswer,
    },
    /// The user typed in an input/editor dialog.
    TextChanged(String),
}

/// `typed` is what the user has typed so far in an input/editor dialog.
pub fn view<'a>(dialog: &'a Dialog, typed: &'a str) -> Element<'a, Event> {
    let answer = |id: &str, answer| Event::Answer {
        id: id.to_owned(),
        answer,
    };
    let body: Element<'_, Event> = match dialog {
        Dialog::Confirm { id, title, message } => column![
            text(title).size(16),
            text(message),
            row![
                button("Yes").on_press(answer(id, UiAnswer::Confirmed(true))),
                button("No").on_press(answer(id, UiAnswer::Confirmed(false))),
                button("Cancel").on_press(answer(id, UiAnswer::Cancelled)),
            ]
            .spacing(8),
        ]
        .spacing(8)
        .into(),
        Dialog::Select { id, title, options } => {
            let choices = options.iter().map(|option| {
                button(text(option))
                    .on_press(answer(id, UiAnswer::Value(option.clone())))
                    .into()
            });
            column![
                text(title).size(16),
                column(choices).spacing(4),
                button("Cancel").on_press(answer(id, UiAnswer::Cancelled)),
            ]
            .spacing(8)
            .into()
        }
        Dialog::Input {
            id,
            title,
            placeholder,
        } => text_dialog(id, title, placeholder.as_deref().unwrap_or(""), "", typed),
        Dialog::Editor { id, title, prefill } => {
            text_dialog(id, title, "", prefill.as_deref().unwrap_or(""), typed)
        }
    };
    container(body)
        .padding(12)
        .width(Length::Fill)
        .style(container::bordered_box)
        .into()
}

/// Input and editor dialogs differ only in placeholder vs prefill.
fn text_dialog<'a>(
    id: &str,
    title: &'a str,
    placeholder: &'a str,
    prefill: &str,
    typed: &str,
) -> Element<'a, Event> {
    let value = if typed.is_empty() { prefill } else { typed }.to_owned();
    let submit = |value: String| Event::Answer {
        id: id.to_owned(),
        answer: UiAnswer::Value(value),
    };
    column![
        text(title).size(16),
        text_input(placeholder, &value)
            .on_input(Event::TextChanged)
            .on_submit(submit(value.clone())),
        row![
            button("Submit").on_press(submit(value)),
            button("Cancel").on_press(Event::Answer {
                id: id.to_owned(),
                answer: UiAnswer::Cancelled
            }),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .into()
}
