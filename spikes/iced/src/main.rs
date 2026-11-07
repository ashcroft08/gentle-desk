//! iced spike: message thread + markdown + streaming over the real core.

use std::time::Duration;

use gentle_core::chat::{ChatRole, ChatState};
use iced::widget::{column, container, markdown, scrollable, text};
use iced::{stream, Element, Length, Subscription, Theme};

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .theme(|_: &App| Theme::Dark)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    State(ChatState),
    LinkClicked(markdown::Uri),
}

/// Parsed markdown for one message; `raw_len` is how much of its text was already pushed.
struct Rendered {
    raw_len: usize,
    content: markdown::Content,
}

#[derive(Default)]
struct App {
    state: ChatState,
    rendered: Vec<Rendered>,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::LinkClicked(uri) => eprintln!("link: {uri}"),
            Message::State(state) => {
                // Incremental markdown: only the new tail of a streaming message is parsed in.
                self.rendered.truncate(state.messages.len());
                for (index, message) in state.messages.iter().enumerate() {
                    match self.rendered.get_mut(index) {
                        Some(rendered) if message.text.len() >= rendered.raw_len => {
                            rendered.content.push_str(&message.text[rendered.raw_len..]);
                            rendered.raw_len = message.text.len();
                        }
                        Some(rendered) => {
                            *rendered = parse(&message.text);
                        }
                        None => self.rendered.push(parse(&message.text)),
                    }
                }
                self.state = state;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = Theme::Dark;
        let thread = self.state.messages.iter().zip(&self.rendered).map(
            |(message, rendered)| -> Element<'_, Message> {
                match message.role {
                    ChatRole::User => container(text(&message.text))
                        .padding(10)
                        .style(container::rounded_box)
                        .into(),
                    ChatRole::Assistant => {
                        markdown::view(rendered.content.items(), &theme).map(Message::LinkClicked)
                    }
                }
            },
        );
        scrollable(column(thread).spacing(16).padding(20).width(Length::Fill))
            .anchor_bottom()
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            stream::channel(256, async |tx| {
                spike_common::spawn_feed(
                    move |state| {
                        let _ = tx.clone().try_send(Message::State(state));
                    },
                    3,
                    Duration::from_millis(25),
                );
                std::future::pending::<()>().await;
            })
        })
    }
}

fn parse(text: &str) -> Rendered {
    Rendered {
        raw_len: text.len(),
        content: markdown::Content::parse(text),
    }
}
