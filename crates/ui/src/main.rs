//! `gentle-desk`: the iced UI. It renders `gentle-app` state and turns clicks
//! into intents; every decision lives in `gentle-app`, every process detail in
//! `gentle-backend`. Command line arguments are forwarded to `gentle-shell`
//! (e.g. `--isolated --model claude-bridge/claude-sonnet-5-5`).
//!
//! Layout: `app` (state and wiring), `view/` (one file per component),
//! `message`, `subscription`, `backend` (startup wrapper), `refresh_flow` (the
//! iced side of the session refresh: its decisions live in `gentle-runtime`) and
//! `session_refresh` (its iced transport). The UI-agnostic runtime — `chat_runtime`
//! (one child per chat, and the app's commands over it), `chat_keys` (the two
//! names of one chat), the refresh flow, the refresh core and the pure helpers
//! (`text`, `time_ago`, `sessions`, `cache_path`, `clock`) — lives in the
//! `gentle-runtime` crate.

mod app;
mod backend;
mod message;
mod refresh_flow;
mod session_refresh;
mod subscription;
mod thread_cache;
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
