//! Framework-free startup of one run: the chat host (one `gentle-shell` child
//! per chat), the session index and its cache, and the session watcher.
//!
//! Everything here is plain data: std receivers, no window framework, so both
//! the iced window and, later, the Tauri app can use it. A failure to launch is
//! data, not a panic: the window still opens and shows it.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use gentle_backend::chat_host::{
    ChatHost, Clock, Limits, ProcessSpawner, Spawner, SystemClock, TaggedEvent,
};
use gentle_backend::home::{agent_dir, with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::session_cache;
use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_watch::{Batch, SessionWatcher, WatchError};

use crate::cache_path;
use crate::chat_runtime::ChatRuntime;

/// What [`start`] brought up. A failure to launch is data, not a panic: the
/// window still opens and shows it.
pub struct Started {
    /// The chat host with the registry of chat names, shared with the event
    /// source and with every command. `None` when gentle-shell cannot be
    /// resolved at all, which is the run's fatal banner.
    pub chat: Option<Arc<ChatRuntime>>,
    /// The tagged events of every child, when the host is up (`chat` is
    /// `Some`); to be read by a single consumer.
    pub events: Option<Receiver<TaggedEvent>>,
    /// `None` when the pi home cannot be resolved: the sidebar stays empty.
    pub index: Option<SessionIndex>,
    /// Where the index cache is read and written, when the run has one.
    pub cache_path: Option<PathBuf>,
    /// The session directory being watched, when the home is known.
    pub sessions_dir: Option<PathBuf>,
    /// `None` when the home is unknown or the watcher could not start: the list
    /// then falls back to polling and retries on every full pass.
    pub watcher: Option<SessionWatcher>,
    /// The batches of `watcher`, when it runs.
    pub batches: Option<Receiver<Batch>>,
    /// Why the watcher did not start at boot, already logged once here: the
    /// refresh flow seeds its "log only on a new message" state with it.
    pub watch_error: Option<String>,
    pub fatal: Option<String>,
}

/// Command line arguments are forwarded to every `gentle-shell` child; a home
/// flag is added when they name none.
pub fn start() -> Started {
    let home_env = HomeEnv::from_process();
    let args = with_home_flag(std::env::args_os().skip(1).collect(), &home_env);
    let cache_path = cache_path::cache_file(std::env::var_os("LOCALAPPDATA"));
    let agent = agent_dir(&args, &home_env);
    let sessions_dir = agent.map(|dir| dir.join("sessions"));
    let index = sessions_dir.clone().map(|sessions| {
        let mut index = SessionIndex::new(sessions);
        // A cache that is missing, stale, corrupt or from another home is
        // ignored and the first refresh does the full scan.
        if let Some(snapshot) = cache_path.as_deref().and_then(session_cache::load) {
            index.restore(snapshot);
        }
        index
    });
    // A watcher that cannot start is no dead end: the polling fallback covers
    // the list and every full pass tries again.
    let mut watch_error = None;
    let (watcher, batches) = match sessions_dir.as_deref() {
        Some(dir) => match try_start_watcher(dir) {
            Ok(started) => (Some(started.0), Some(started.1)),
            Err(error) => {
                eprintln!("session watcher unavailable: {error}");
                watch_error = Some(error.to_string());
                (None, None)
            }
        },
        None => (None, None),
    };
    let (chat, events, fatal) = match start_host(args) {
        Ok(host) => {
            let (runtime, tagged) = ChatRuntime::new(host);
            let runtime = Arc::new(runtime);
            (Some(runtime), Some(tagged), None)
        }
        Err(error) => (None, None, Some(error)),
    };
    Started {
        chat,
        events,
        index,
        cache_path,
        sessions_dir,
        watcher,
        batches,
        watch_error,
        fatal,
    }
}

/// The host itself: real children, the searcher's limits, and the launcher plus
/// the forwarded arguments of this window.
fn start_host(args: Vec<OsString>) -> Result<ChatHost, String> {
    let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|e| e.to_string())?;
    let spawner = ProcessSpawner::new(launcher).with_args(args);
    Ok(ChatHost::with_parts(
        Limits::default(),
        Arc::new(spawner) as Arc<dyn Spawner>,
        Arc::new(SystemClock) as Arc<dyn Clock>,
    ))
}

/// Starts watching `sessions_dir`, for the first time and for every retry after
/// a failure. `Err` is worth logging once, not once per pass: the caller decides.
pub fn try_start_watcher(
    sessions_dir: &Path,
) -> Result<(SessionWatcher, Receiver<Batch>), WatchError> {
    SessionWatcher::start(sessions_dir)
}
