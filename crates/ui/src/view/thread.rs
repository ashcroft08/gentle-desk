//! The message thread: user bubbles, markdown replies, model errors.

use gentle_core::chat::{ChatMessage, ChatRole};
use iced::widget::{button, column, container, markdown, scrollable, text};
use iced::{Element, Length, Theme};

use crate::thread_cache::ThreadCache;

#[derive(Debug, Clone)]
pub enum Event {
    /// Copy the raw markdown of the message with this id.
    Copy(String),
    LinkClicked(markdown::Uri),
}

pub fn view<'a>(messages: &'a [ChatMessage], cache: &'a ThreadCache) -> Element<'a, Event> {
    let rows = messages
        .iter()
        .enumerate()
        .map(|(index, message)| message_view(index, message, cache));
    scrollable(column(rows).spacing(16).padding(20).width(Length::Fill))
        .anchor_bottom()
        .height(Length::Fill)
        .into()
}

fn message_view<'a>(
    index: usize,
    message: &'a ChatMessage,
    cache: &'a ThreadCache,
) -> Element<'a, Event> {
    match (message.role, &message.error) {
        (ChatRole::User, _) => container(text(&message.text))
            .padding(10)
            .style(container::rounded_box)
            .into(),
        (ChatRole::Assistant, Some(error)) => {
            container(column![text("✗ Model error").size(14), text(error).size(14)].spacing(4))
                .padding(10)
                .width(Length::Fill)
                .style(container::danger)
                .into()
        }
        (ChatRole::Assistant, None) => {
            let body: Element<'_, Event> = match cache.content(index) {
                Some(content) => {
                    markdown::view(content.items(), &Theme::Dark).map(Event::LinkClicked)
                }
                None => text(&message.text).into(),
            };
            if message.streaming {
                body
            } else {
                column![
                    body,
                    button(text("Copy").size(12))
                        .style(button::text)
                        .on_press(Event::Copy(message.id.clone())),
                ]
                .spacing(4)
                .into()
            }
        }
    }
}
