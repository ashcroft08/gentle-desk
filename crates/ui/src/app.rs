//! The application: owns the state, runs `gentle-app` commands, wires the
//! components together. Drawing lives in `view/`, process startup in
//! `backend`, subscriptions in `subscription`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_app::{update, AppState, Command, Input, Intent};
use gentle_backend::process::{ProcessEvent, RpcProcess};
use gentle_core::chat::Dialog;
use gentle_core::rpc::Decoded;
use iced::widget::{column, row, scrollable};
use iced::{clipboard, Element, Subscription, Task};

use crate::backend;
use crate::clock;
use crate::message::Message;
use crate::session_refresh::{self, SharedIndex};
use crate::subscription::{self, EventSource};
use crate::thread_cache::ThreadCache;
use crate::view::{
    banner, composer, dialog, header, helper_thread, helpers, notices, sidebar, thread,
};

pub struct App {
    app: AppState,
    cache: ThreadCache,
    input: String,
    dialog_text: String,
    process: Option<RpcProcess>,
    events: EventSource,
    index: Option<SharedIndex>,
    /// Where the index cache is kept; `None` means this run persists nothing.
    cache_path: Option<PathBuf>,
    /// A refresh is running on its worker thread; another one waits for it.
    refreshing: bool,
    /// How far the chat list is scrolled, in pixels (window geometry, not app state).
    sidebar_offset: f32,
    /// A failure that ends the session (launcher missing, child died).
    fatal: Option<String>,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let started = backend::start();
        let mut app = Self {
            app: AppState::default(),
            cache: ThreadCache::default(),
            input: String::new(),
            dialog_text: String::new(),
            process: started.process,
            events: started.events,
            index: started.index.map(|index| Arc::new(Mutex::new(index))),
            cache_path: started.cache_path,
            refreshing: false,
            sidebar_offset: 0.0,
            fatal: started.fatal,
        };
        let loaded = app.dispatch(Input::Intent(Intent::RefreshSessions));
        let focus = iced::widget::operation::focus(composer::ID);
        (app, Task::batch([loaded, focus]))
    }

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
                Command::RefreshSessions => tasks.push(self.refresh_sessions()),
            }
        }
        Task::batch(tasks)
    }

    /// Starts a refresh of the session index on a worker thread; the list
    /// arrives later as `Message::SessionsRefreshed`. One at a time.
    fn refresh_sessions(&mut self) -> Task<Message> {
        let Some(index) = &self.index else {
            return Task::none();
        };
        if self.refreshing {
            return Task::none();
        }
        self.refreshing = true;
        Task::perform(
            session_refresh::refresh(Arc::clone(index), self.cache_path.clone()),
            Message::SessionsRefreshed,
        )
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
            Message::Tick => self.intent(Intent::RefreshSessions),
            Message::SessionsRefreshed(result) => {
                self.refreshing = false;
                match result {
                    Ok(items) => self.dispatch(Input::SessionsLoaded(items)),
                    Err(error) => {
                        eprintln!("{error}");
                        Task::none()
                    }
                }
            }
            Message::CloseRequested => {
                if let Some(process) = self.process.take() {
                    let _ = process.shutdown(Duration::from_secs(3));
                }
                iced::exit()
            }
        }
    }

    fn on_process(&mut self, event: ProcessEvent) -> Task<Message> {
        match event {
            ProcessEvent::Output(Decoded::Event(event)) => {
                self.dispatch(Input::RpcAt(event, clock::now_ms()))
            }
            ProcessEvent::Output(Decoded::Unknown(_)) => Task::none(),
            ProcessEvent::Stderr(line) => {
                eprintln!("[gentle-shell] {line}");
                Task::none()
            }
            ProcessEvent::Closed => {
                self.fatal
                    .get_or_insert_with(|| "gentle-shell exited".into());
                Task::none()
            }
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
        subscription::subscription(&self.events)
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
