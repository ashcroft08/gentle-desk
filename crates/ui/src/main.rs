//! `gentle-desk`: the iced UI. It renders `gentle-app` state and turns clicks
//! into intents; every decision lives in `gentle-app`, every process detail in
//! `gentle-backend`. Command line arguments are forwarded to `gentle-shell`
//! (e.g. `--isolated --model claude-bridge/claude-sonnet-5-5`).
//!
//! Layout: `app` (state and wiring), `view/` (one file per component),
//! `message`, `subscription`, `backend` (startup), `refresh_flow` (which session
//! refresh runs when) and its pure halves `refresh_queue` and `save_throttle`,
//! plus the small pure helpers `thread_cache`, `sessions` and `cache_path`.

mod app;
mod backend;
mod cache_path;
// Wired by the chat host dispatch (m18 T2 step 2b); until then only its tests use it.
#[allow(dead_code)]
mod chat_keys;
// Wired by the chat host dispatch (m18 T2 step 2b-2); until then only its tests use it.
#[allow(dead_code)]
mod chat_runtime;
mod clock;
mod message;
mod refresh_flow;
mod refresh_queue;
mod save_throttle;
mod session_refresh;
mod sessions;
mod subscription;
mod text;
mod thread_cache;
mod time_ago;
mod view;
mod watch_source;
mod windowing;

use iced::{window, Theme};

use app::App;

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(|_: &App| Theme::Dark)
        .title("gentle-desk")
        .window(window::Settings {
            // The process must be shut down before exiting: see `Message::CloseRequested`.
            exit_on_close_request: false,
            size: iced::Size::new(1100.0, 560.0),
            position: window::Position::Centered,
            ..window::Settings::default()
        })
        .run()
}
