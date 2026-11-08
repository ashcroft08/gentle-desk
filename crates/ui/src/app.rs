//! The application: owns the state, runs `gentle-app` commands, wires the
//! components together. Drawing lives in `view/`, process startup in
//! `app/boot`, subscriptions in `subscription`, the session refresh flow (which
//! pass runs when) in `refresh_flow`, and the messages of both in
//! `app/process_update` and `app/sessions_update`.

use gentle_app::{update, AppState, Command, Input, Intent};
use gentle_backend::process::RpcProcess;
use gentle_core::chat::Dialog;
use iced::widget::{column, row, scrollable};
use iced::{clipboard, Element, Subscription, Task};

use crate::clock;
use crate::message::Message;
use crate::refresh_flow::Flow;
use crate::subscription::{self, EventSource};
use crate::thread_cache::ThreadCache;
use crate::view::{
    banner, composer, dialog, header, helper_thread, helpers, notices, sidebar, thread,
};

mod boot;
mod process_update;
mod sessions_update;

pub struct App {
    app: AppState,
    cache: ThreadCache,
    input: String,
    dialog_text: String,
    process: Option<RpcProcess>,
    events: EventSource,
    /// The session index, the watcher and the passes they run.
    flow: Flow,
    /// How far the chat list is scrolled, in pixels (window geometry, not app state).
    sidebar_offset: f32,
    /// A failure that ends the session (launcher missing, child died).
    fatal: Option<String>,
}

impl App {
    /// Feeds an input to the app layer and runs the commands it returns.
    fn dispatch(&mut self, input: Input) -> Task<Message> {
        let dialog_before = self.app.chat.pending_dialogs.first().cloned();
        let (state, commands) = update(std::mem::take(&mut self.app), input);
        self.app = state;
        self.cache.sync(&self.app.chat);
        let mut tasks = Vec::new();
        // A new text dialog takes the keyboard so the user can type at once.
        let dialog_now = self.app.chat.pending_dialogs.first();
        if dialog_now != dialog_before.as_ref()
            && matches!(
                dialog_now,
                Some(Dialog::Input { .. } | Dialog::Editor { .. })
            )
        {
            tasks.push(iced::widget::operation::focus(dialog::INPUT_ID));
        }
        for command in commands {
            match command {
                Command::Send(rpc) => {
                    let sent = match &mut self.process {
                        Some(process) => process.send(&rpc).map_err(|e| e.to_string()),
                        None => Err("gentle-shell is not running".into()),
                    };
                    if let Err(error) = sent {
                        self.fatal = Some(format!("cannot reach gentle-shell: {error}"));
                    }
                }
                Command::CopyToClipboard(text) => tasks.push(clipboard::write(text)),
                Command::RefreshSessions => tasks.push(self.flow.on_refresh_command()),
            }
        }
        Task::batch(tasks)
    }

    fn intent(&mut self, intent: Intent) -> Task<Message> {
        self.dispatch(Input::Intent(intent))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Process(event) => self.on_process(event),
            Message::Header(header::Event::NewChat) => self.intent(Intent::NewChat),
            Message::Sidebar(sidebar::Event::Open(path)) => self.intent(Intent::OpenSession(path)),
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
            Message::SessionsChanged(batch) => self.on_sessions_changed(batch),
            Message::WatcherLost => self.on_watcher_lost(),
            Message::SessionsRefreshed(result) => self.on_sessions_refreshed(result),
            Message::SessionsProgress(progress) => self.on_sessions_progress(progress),
            Message::CloseRequested => self.on_close_requested(),
        }
    }

    fn send_prompt(&mut self) -> Task<Message> {
        let prompt = std::mem::take(&mut self.input);
        if prompt.trim().is_empty() || self.app.chat.working {
            self.input = prompt; // keep what the user typed
            return Task::none();
        }
        self.intent(Intent::SendPrompt(prompt))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        subscription::subscription(&self.events, self.flow.watch(), self.flow.tick_delay())
    }

    pub fn view(&self) -> Element<'_, Message> {
        let chat = &self.app.chat;
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
            current: self.app.current_session.as_deref(),
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
}
