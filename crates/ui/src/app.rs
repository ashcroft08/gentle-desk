//! The application: owns the state, runs `gentle-app` commands, wires the
//! components together. Drawing lives in `view/`, process startup in
//! `app/boot`, subscriptions in `subscription`, the session refresh flow (which
//! pass runs when) in `refresh_flow`, and the messages of both in
//! `app/sessions_update`, `app/chat_update` and `app/chat_command`.

use std::sync::Arc;

use gentle_app::{AppState, ChatId, Input, Intent};
use gentle_core::chat::ChatState;
use iced::widget::{column, row, scrollable};
use iced::{Element, Subscription, Task};

use crate::chat_runtime::ChatRuntime;
use crate::clock;
use crate::message::Message;
use crate::refresh_flow::Flow;
use crate::subscription::{self, EventSource};
use crate::thread_cache::ThreadCache;
use crate::view::{
    banner, composer, dialog, header, helper_thread, helpers, notices, sidebar, thread,
};

mod boot;
mod chat_command;
mod chat_update;
mod dispatch;
mod sessions_update;

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
    /// A failure that ends the session (launcher missing, the foreground child died).
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
                    scrollable::AbsoluteOffset {
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

    fn send_prompt(&mut self) -> Task<Message> {
        let prompt = std::mem::take(&mut self.input);
        // One turn at a time, and only into a chat that has a child: before the
        // first chat is open there is nowhere for the prompt to go.
        let ready = self.app.chats.foreground().is_some() && !Self::visible_chat(&self.app).working;
        if prompt.trim().is_empty() || !ready {
            self.input = prompt; // keep what the user typed
            return Task::none();
        }
        self.intent(Intent::SendPrompt(prompt))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        subscription::subscription(&self.events, self.flow.watch(), self.flow.tick_delay())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let chat = Self::visible_chat(&self.app);
        let mut page = column![
            header::view().map(Message::Header),
            notices::view(&self.app.notices).map(Message::Notices),
            thread::view(&chat.messages, &self.cache).map(Message::Thread),
        ];
        if let Some(error) = &self.fatal {
            page = page.push(banner::view(error));
        }
        if let Some(pending) = chat.pending_dialogs.first() {
            page = page.push(dialog::view(pending, &self.dialog_text).map(Message::Dialog));
        }
        let can_send = !self.input.trim().is_empty() && self.fatal.is_none();
        let page =
            page.push(composer::view(&self.input, chat.working, can_send).map(Message::Composer));
        let sessions = sidebar::view(sidebar::Props {
            visible: self.app.visible_sessions(),
            total: self.app.sessions.len(),
            query: &self.app.session_query,
            list_status: self.app.list_status,
            current: self.current_session(),
            now_ms: clock::now_ms(),
            offset: self.sidebar_offset,
        });
        let mut layout = row![sessions.map(Message::Sidebar), page];
        // The open helper's thread replaces the list; a task that vanished falls back to it.
        let open = self
            .app
            .selected_helper
            .as_ref()
            .and_then(|id| chat.helpers.tasks.iter().find(|task| &task.id == id));
        if let Some(task) = open {
            layout = layout.push(helper_thread::view(task).map(Message::HelperThread));
        } else if helpers::is_visible(&chat.helpers) {
            layout = layout.push(helpers::view(&chat.helpers).map(Message::Helpers));
        }
        layout.into()
    }

    /// The chat the window shows: the foreground one of the per-chat shell, or
    /// the empty single thread before the first chat is opened.
    fn visible_chat(app: &AppState) -> &ChatState {
        app.chats
            .foreground()
            .and_then(|chat| app.chats.state(chat))
            .unwrap_or(&app.chat)
    }

    /// The session file the sidebar marks as open.
    fn current_session(&self) -> Option<&str> {
        self.app
            .chats
            .foreground()
            .and_then(|chat| chat.session_path())
            .or(self.app.current_session.as_deref())
    }
}
