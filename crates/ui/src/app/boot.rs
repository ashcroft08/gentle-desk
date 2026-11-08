//! Process startup: the child, the session index and its cache, the watcher,
//! and the window's first task.

use std::sync::{Arc, Mutex};

use gentle_app::AppState;
use iced::Task;

use super::App;
use crate::backend;
use crate::message::Message;
use crate::refresh_flow::Flow;
use crate::session_refresh::Shared;
use crate::thread_cache::ThreadCache;
use crate::view::composer;

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let started = backend::start();
        let shared = started
            .index
            .map(|index| Shared::new(Arc::new(Mutex::new(index)), started.cache_path.clone()));
        let mut app = Self {
            app: AppState::default(),
            cache: ThreadCache::default(),
            input: String::new(),
            dialog_text: String::new(),
            process: started.process,
            events: started.events,
            flow: Flow::new(
                shared,
                started.sessions_dir,
                started.watcher,
                started.watch,
                started.watch_error,
            ),
            sidebar_offset: 0.0,
            fatal: started.fatal,
        };
        // The window opens empty: the first full pass fills it, first with what
        // the restored cache already knew and then with its final list.
        let first_list = app.flow.on_tick();
        let focus = iced::widget::operation::focus(composer::ID);
        (app, Task::batch([first_list, focus]))
    }
}
