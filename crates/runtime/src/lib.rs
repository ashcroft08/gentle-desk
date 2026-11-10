//! `gentle-runtime`: the UI-agnostic runtime of the window. One child per chat
//! ([`chat_runtime`]), the session refresh and its cache ([`session_refresh`]),
//! and the framework-free startup of one run ([`startup`]).
//!
//! Nothing here draws: the iced window (`crates/ui`) and, later, the Tauri app
//! both sit on top of this crate.

pub mod app_config;
pub mod badge;
pub mod cache_path;
pub mod chat_keys;
pub mod chat_runtime;
pub mod clock;
pub mod helper_text;
pub mod refresh_flow;
pub mod refresh_queue;
pub mod save_throttle;
pub mod session_refresh;
pub mod sessions;
pub mod startup;
pub mod text;
pub mod time_ago;
pub mod workspaces;
