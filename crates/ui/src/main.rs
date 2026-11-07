//! `gentle-desk`: the iced UI. It renders `gentle-app` state and turns clicks
//! into intents; every decision lives in `gentle-app`, every process detail in
//! `gentle-backend`. Command line arguments are forwarded to `gentle-shell`
//! (e.g. `--isolated --model claude-bridge/claude-sonnet-5-5`).

mod sessions;
mod thread;

use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_app::{update, AppState, Command, Input, Intent, Notice, NoticeLevel};
use gentle_backend::home::{agent_dir, with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::process::{ProcessEvent, RpcProcess, SpawnOptions};
use gentle_backend::session_index::SessionIndex;
use gentle_core::chat::{ChatMessage, ChatRole, Dialog};
use gentle_core::rpc::{Decoded, UiAnswer};
use iced::widget::{button, column, container, markdown, row, scrollable, text, text_input, Space};
use iced::{clipboard, stream, window, Element, Length, Subscription, Task, Theme};
use thread::ThreadCache;

/// Id of the prompt box, focused at start so typing works at once.
const COMPOSER: &str = "composer";

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(|_: &App| Theme::Dark)
        .title("gentle-desk")
        .window(window::Settings {
            // The process must be shut down before exiting: see `Message::CloseRequested`.
            exit_on_close_request: false,
            size: iced::Size::new(900.0, 520.0),
            position: window::Position::Centered,
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Process(ProcessEvent),
    Intent(Intent),
    InputChanged(String),
    Send,
    DialogTextChanged(String),
    CloseRequested,
    LinkClicked(markdown::Uri),
}

/// The output stream of the child, handed to the subscription once.
#[derive(Clone)]
struct EventSource(Arc<Mutex<Option<Receiver<ProcessEvent>>>>);

impl Hash for EventSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // One source per run: a constant identity keeps the subscription alive.
        "gentle-shell-events".hash(state);
    }
}

struct App {
    app: AppState,
    cache: ThreadCache,
    input: String,
    dialog_text: String,
    process: Option<RpcProcess>,
    events: EventSource,
    /// `None` when the pi home cannot be resolved: the sidebar stays empty.
    index: Option<SessionIndex>,
    /// A failure that ends the session (launcher missing, child died).
    fatal: Option<String>,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let events = EventSource(Arc::default());
        let home_env = HomeEnv::from_process();
        let args = with_home_flag(std::env::args_os().skip(1).collect(), &home_env);
        let index = agent_dir(&args, &home_env).map(|dir| SessionIndex::new(dir.join("sessions")));
        let started = launcher::resolve(&LauncherEnv::from_process())
            .map_err(|e| e.to_string())
            .and_then(|launcher| {
                let options = SpawnOptions {
                    args,
                    // pi's rpc mode only serves extension UI requests with an interactive host.
                    env: vec![("GENTLE_SHELL_INTERACTIVE_HOST".into(), "1".into())],
                    cwd: None,
                };
                RpcProcess::spawn(&launcher, &options)
                    .map_err(|e| format!("cannot start {}: {e}", launcher.program.display()))
            });
        let (process, fatal) = match started {
            Ok(mut process) => {
                *events.0.lock().unwrap() = Some(process.take_events());
                (Some(process), None)
            }
            Err(error) => (None, Some(error)),
        };
        let mut app = Self {
            app: AppState::default(),
            cache: ThreadCache::default(),
            input: String::new(),
            dialog_text: String::new(),
            process,
            events,
            index,
            fatal,
        };
        let loaded = app.dispatch(Input::Intent(Intent::RefreshSessions));
        (
            app,
            Task::batch([loaded, iced::widget::operation::focus(COMPOSER)]),
        )
    }

    /// Feeds an input to the app layer and runs the commands it returns.
    fn dispatch(&mut self, input: Input) -> Task<Message> {
        let (state, commands) = update(std::mem::take(&mut self.app), input);
        self.app = state;
        self.cache.sync(&self.app.chat);
        let mut tasks = Vec::new();
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

    /// Re-reads the session index (a `stat` per unchanged file) and publishes the list.
    fn refresh_sessions(&mut self) -> Task<Message> {
        let Some(index) = &mut self.index else {
            return Task::none();
        };
        if let Err(error) = index.refresh() {
            eprintln!("cannot read sessions: {error}");
            return Task::none();
        }
        let items = index.entries().iter().map(sessions::to_item).collect();
        self.dispatch(Input::SessionsLoaded(items))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Process(ProcessEvent::Output(Decoded::Event(event))) => {
                self.dispatch(Input::Rpc(event))
            }
            Message::Process(ProcessEvent::Output(Decoded::Unknown(_))) => Task::none(),
            Message::Process(ProcessEvent::Stderr(line)) => {
                eprintln!("[gentle-shell] {line}");
                Task::none()
            }
            Message::Process(ProcessEvent::Closed) => {
                self.fatal
                    .get_or_insert_with(|| "gentle-shell exited".into());
                Task::none()
            }
            Message::Intent(intent) => {
                self.dialog_text.clear();
                self.dispatch(Input::Intent(intent))
            }
            Message::InputChanged(value) => {
                self.input = value;
                Task::none()
            }
            Message::Send => {
                let prompt = std::mem::take(&mut self.input);
                let sendable = !prompt.trim().is_empty() && !self.app.chat.working;
                if !sendable {
                    self.input = prompt; // keep what the user typed
                    return Task::none();
                }
                self.dispatch(Input::Intent(Intent::SendPrompt(prompt)))
            }
            Message::DialogTextChanged(value) => {
                self.dialog_text = value;
                Task::none()
            }
            Message::CloseRequested => {
                if let Some(process) = self.process.take() {
                    let _ = process.shutdown(Duration::from_secs(3));
                }
                iced::exit()
            }
            Message::LinkClicked(uri) => {
                eprintln!("link clicked: {uri}");
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let events = Subscription::run_with(self.events.clone(), |source| {
            let receiver = source.0.lock().unwrap().take();
            stream::channel(1024, async move |mut output| {
                use iced::futures::{executor::block_on, SinkExt};
                let Some(receiver) = receiver else { return };
                // A plain thread blocks on the std channel; `send` applies
                // backpressure instead of dropping events.
                std::thread::spawn(move || {
                    for event in receiver {
                        if block_on(output.send(Message::Process(event))).is_err() {
                            break;
                        }
                    }
                });
                std::future::pending::<()>().await;
            })
        });
        Subscription::batch([
            events,
            // Sessions change outside this window too (other terminals, other apps).
            iced::time::every(Duration::from_secs(5))
                .map(|_| Message::Intent(Intent::RefreshSessions)),
            window::close_requests().map(|_| Message::CloseRequested),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("gentle-desk").size(18),
            Space::new().width(Length::Fill),
            button("New chat").on_press(Message::Intent(Intent::NewChat)),
        ]
        .align_y(iced::Alignment::Center)
        .padding(12);

        let notices = column(self.app.notices.iter().map(notice_view)).spacing(4);

        let thread = self
            .app
            .chat
            .messages
            .iter()
            .enumerate()
            .map(|(index, message)| self.message_view(index, message));
        let thread = scrollable(column(thread).spacing(16).padding(20).width(Length::Fill))
            .anchor_bottom()
            .height(Length::Fill);

        let mut page = column![header, notices, thread];
        if let Some(error) = &self.fatal {
            page = page.push(
                container(text(format!("✗ {error}")))
                    .padding(10)
                    .width(Length::Fill)
                    .style(container::danger),
            );
        }
        if let Some(dialog) = self.app.chat.pending_dialogs.first() {
            page = page.push(self.dialog_view(dialog));
        }
        let main = page.push(self.composer());
        row![self.sidebar(), main].into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let current = self.app.current_session.as_deref();
        let rows = self.app.sessions.iter().map(|session| {
            let style: fn(&Theme, button::Status) -> button::Style =
                if current == Some(session.path.as_str()) {
                    button::primary
                } else {
                    button::text
                };
            let project = session.cwd.rsplit(['\\', '/']).next().unwrap_or("");
            button(
                column![
                    text(&session.title).size(14),
                    text(format!("{} msgs · {project}", session.message_count)).size(11),
                ]
                .spacing(2),
            )
            .width(Length::Fill)
            .style(style)
            .on_press(Message::Intent(Intent::OpenSession(session.path.clone())))
            .into()
        });
        let list: Element<'_, Message> = if self.app.sessions.is_empty() {
            text("No saved chats yet").size(13).into()
        } else {
            scrollable(column(rows).spacing(2))
                .height(Length::Fill)
                .into()
        };
        container(column![text("Chats").size(16), list].spacing(8).padding(12))
            .width(260)
            .height(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn message_view<'a>(&'a self, index: usize, message: &'a ChatMessage) -> Element<'a, Message> {
        match (message.role, &message.error) {
            (ChatRole::User, _) => container(text(&message.text))
                .padding(10)
                .style(container::rounded_box)
                .into(),
            (ChatRole::Assistant, Some(error)) => {
                container(column![text("✗ Model error").size(14), text(error).size(14),].spacing(4))
                    .padding(10)
                    .width(Length::Fill)
                    .style(container::danger)
                    .into()
            }
            (ChatRole::Assistant, None) => {
                let body: Element<'_, Message> = match self.cache.content(index) {
                    Some(content) => {
                        markdown::view(content.items(), &Theme::Dark).map(Message::LinkClicked)
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
                            .on_press(Message::Intent(Intent::CopyMessage(message.id.clone()))),
                    ]
                    .spacing(4)
                    .into()
                }
            }
        }
    }

    fn dialog_view<'a>(&'a self, dialog: &'a Dialog) -> Element<'a, Message> {
        let answer = |id: &str, answer| {
            Message::Intent(Intent::AnswerDialog {
                id: id.to_owned(),
                answer,
            })
        };
        let body: Element<'_, Message> = match dialog {
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
            Dialog::Input { id, title, .. } | Dialog::Editor { id, title, .. } => {
                let (placeholder, prefill) = match dialog {
                    Dialog::Input { placeholder, .. } => (placeholder.clone(), None),
                    Dialog::Editor { prefill, .. } => (None, prefill.clone()),
                    _ => (None, None),
                };
                let value = if self.dialog_text.is_empty() {
                    prefill.unwrap_or_default()
                } else {
                    self.dialog_text.clone()
                };
                column![
                    text(title).size(16),
                    text_input(placeholder.as_deref().unwrap_or(""), &value)
                        .on_input(Message::DialogTextChanged)
                        .on_submit(answer(id, UiAnswer::Value(value.clone()))),
                    row![
                        button("Submit").on_press(answer(id, UiAnswer::Value(value))),
                        button("Cancel").on_press(answer(id, UiAnswer::Cancelled)),
                    ]
                    .spacing(8),
                ]
                .spacing(8)
                .into()
            }
        };
        container(body)
            .padding(12)
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn composer(&self) -> Element<'_, Message> {
        let working = self.app.chat.working;
        let action = if working {
            button("Stop").on_press(Message::Intent(Intent::Abort))
        } else {
            button("Send").on_press_maybe(
                (!self.input.trim().is_empty() && self.fatal.is_none()).then_some(Message::Send),
            )
        };
        row![
            text_input("Message gentle-shell…", &self.input)
                .id(COMPOSER)
                .on_input(Message::InputChanged)
                .on_submit(Message::Send),
            action,
        ]
        .spacing(8)
        .padding(12)
        .into()
    }
}

fn notice_view(notice: &Notice) -> Element<'_, Message> {
    let style: fn(&Theme) -> container::Style = match notice.level {
        NoticeLevel::Info => container::rounded_box,
        NoticeLevel::Warning => container::warning,
        NoticeLevel::Error => container::danger,
    };
    container(
        row![
            text(&notice.text).size(14).width(Length::Fill),
            button(text("×").size(14))
                .style(button::text)
                .on_press(Message::Intent(Intent::DismissNotice(notice.id))),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12])
    .width(Length::Fill)
    .style(style)
    .into()
}
