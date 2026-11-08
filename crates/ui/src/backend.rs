//! Starting the chat host (one `gentle-shell` child per chat), the session index
//! and the session watcher for one run, in the shape the window needs.
//!
//! The startup itself is framework-free and lives in
//! [`gentle_runtime::startup`]; this thin wrapper turns its plain receivers
//! into the window's event sources.

use std::path::PathBuf;
use std::sync::Arc;

use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_watch::SessionWatcher;
use gentle_runtime::chat_runtime::ChatRuntime;
use gentle_runtime::startup;

use crate::subscription::EventSource;
use crate::watch_source::WatchSource;

/// What `start` brought up. A failure to launch is data, not a panic: the UI
/// still opens and shows it.
pub struct Started {
    /// The chat host with the registry of chat names, shared with the event
    /// subscription and with every command. `None` when gentle-shell cannot be
    /// resolved at all, which is the run's fatal banner.
    pub chat: Option<Arc<ChatRuntime>>,
    /// The tagged events of every child, filled when the host is up.
    pub events: EventSource,
    /// `None` when the pi home cannot be resolved: the sidebar stays empty.
    pub index: Option<SessionIndex>,
    /// Where the index cache is read and written, when the run has one.
    pub cache_path: Option<PathBuf>,
    /// The session directory being watched, when the home is known.
    pub sessions_dir: Option<PathBuf>,
    /// `None` when the home is unknown or the watcher could not start: the list
    /// then falls back to polling and retries on every full pass.
    pub watcher: Option<SessionWatcher>,
    /// The batches of `watcher`, handed to the subscription.
    pub watch: WatchSource,
    /// Why the watcher did not start at boot, already logged once here: the
    /// refresh flow seeds its "log only on a new message" state with it.
    pub watch_error: Option<String>,
    pub fatal: Option<String>,
}

/// Command line arguments are forwarded to every `gentle-shell` child; a home
/// flag is added when they name none.
pub fn start() -> Started {
    let raw = startup::start();
    let events = EventSource::default();
    if let (Some(chat), Some(tagged)) = (&raw.chat, raw.events) {
        events.fill(chat.clone(), tagged);
    }
    let mut watch = WatchSource::default();
    if let Some(batches) = raw.batches {
        watch.fill(batches);
    }
    Started {
        chat: raw.chat,
        events,
        index: raw.index,
        cache_path: raw.cache_path,
        sessions_dir: raw.sessions_dir,
        watcher: raw.watcher,
        watch,
        watch_error: raw.watch_error,
        fatal: raw.fatal,
    }
}

/// Starts watching `sessions_dir`, for the first time and for every retry after
/// a failure. `Err` is worth logging once, not once per pass: the caller decides.
pub use gentle_runtime::startup::try_start_watcher;
