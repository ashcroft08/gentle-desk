//! The application: owns the state, runs `gentle-app` commands, wires the
//! components together.
//!
//! Drawing lives in `view/` and the frame in `app/frame`; process startup in
//! `app/boot`; subscriptions in `subscription`; the session refresh flow (which
//! pass runs when) in `refresh_flow`, and the messages of both in
//! `app/sessions_update`, `app/chat_update`, `app/chat_closed`, `app/chat_command`
//! and `app/send_prompt`.

use std::sync::Arc;

use gentle_app::{AppState, ChatId, Input, Intent};
use iced::{Subscription, Task};

use crate::chat_runtime::ChatRuntime;
use crate::message::Message;
use crate::refresh_flow::Flow;
use crate::subscription::{self, EventSource};
use crate::thread_cache::ThreadCache;
use crate::view::{composer, dialog, header, helper_thread, helpers, notices, sidebar, thread};

mod boot;
mod chat_closed;
mod chat_command;
mod chat_update;
mod dispatch;
mod frame;
mod send_prompt;
mod sessions_update;
mod visible;

pub struct App {
    app: AppState,
    cache: ThreadCache,
    input: String,
    dialog_text: String,
    /// The chat host: one child per chat, shared with the event subscription and
    /// with every command. `None` when gentle-shell could not be resolved.
    chat: Option<Arc<ChatRuntime>>,
    events: EventSource,
    /// The session index, the watcher and the passes they run.
    flow: Flow,
    /// How far the chat list is scrolled, in pixels (window geometry, not app state).
    sidebar_offset: f32,
    /// A failure that ends the whole session: gentle-shell could not be resolved
    /// at all. A chat's child dying is a per-chat notice with a Restart (S3), not
    /// this.
    fatal: Option<String>,
}

impl App {
    fn intent(&mut self, intent: Intent) -> Task<Message> {
        self.dispatch(Input::Intent(intent))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Chat(converted) => self.on_chat(converted),
            Message::ChatOutcome(outcome) => self.on_chat_outcome(outcome),
            Message::Header(header::Event::NewChat) => self.intent(Intent::OpenNewChat),
            Message::Sidebar(sidebar::Event::Open(path)) => {
                self.intent(Intent::OpenChat(ChatId::of_session(&path)))
            }
            Message::Sidebar(sidebar::Event::QueryChanged(query)) => {
                // A new search starts from the top of its (shorter or longer) list.
                self.sidebar_offset = 0.0;
                let to_top = iced::widget::operation::scroll_to(
                    sidebar::SCROLL_ID,
                    iced::widget::scrollable::AbsoluteOffset {
                        x: None,
                        y: Some(0.0),
                    },
                );
                Task::batch([self.intent(Intent::SetSessionQuery(query)), to_top])
            }
            Message::Sidebar(sidebar::Event::Scrolled(offset)) => {
                self.sidebar_offset = offset;
                Task::none()
            }
            Message::Thread(thread::Event::Copy(id)) => self.intent(Intent::CopyMessage(id)),
            Message::Thread(thread::Event::LinkClicked(uri)) => {
                eprintln!("link clicked: {uri}");
                Task::none()
            }
            Message::Notices(notices::Event::Dismiss(id)) => self.intent(Intent::DismissNotice(id)),
            // The death notice's one action: start that chat's child again and
            // reopen it, so no chat ends up dead until the app restarts (S3).
            Message::Notices(notices::Event::Restart(chat)) => {
                self.intent(Intent::RestartChat(chat))
            }
            Message::Dialog(dialog::Event::Answer { id, answer }) => {
                self.dialog_text.clear();
                // The keyboard goes back to the prompt box once the dialog is gone.
                let answered = self.intent(Intent::AnswerDialog { id, answer });
                Task::batch([answered, iced::widget::operation::focus(composer::ID)])
            }
            Message::Dialog(dialog::Event::TextChanged(value)) => {
                self.dialog_text = value;
                Task::none()
            }
            Message::Composer(composer::Event::Changed(value)) => {
                self.input = value;
                Task::none()
            }
            Message::Composer(composer::Event::Send) => self.send_prompt(),
            Message::Composer(composer::Event::Stop) => self.intent(Intent::Abort),
            Message::Helpers(helpers::Event::Select(id)) => self.intent(Intent::SelectHelper(id)),
            Message::HelperThread(helper_thread::Event::Back) => self.intent(Intent::CloseHelper),
            Message::Tick => self.on_tick(),
            Message::ReapDone => Task::none(),
            Message::SessionsChanged(batch) => self.on_sessions_changed(batch),
            Message::WatcherLost => self.on_watcher_lost(),
            Message::SessionsRefreshed(result) => self.on_sessions_refreshed(result),
            Message::SessionsProgress(progress) => self.on_sessions_progress(progress),
            Message::CloseRequested => self.on_close_requested(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        subscription::subscription(&self.events, self.flow.watch(), self.flow.tick_delay())
    }
}
