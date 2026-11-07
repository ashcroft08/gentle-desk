//! Starting the `gentle-shell` child, the session index and the session watcher
//! for one run.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use gentle_backend::home::{agent_dir, with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::process::{RpcProcess, SpawnOptions};
use gentle_backend::session_cache;
use gentle_backend::session_index::SessionIndex;
use gentle_backend::session_watch::{Batch, SessionWatcher, WatchError};

use crate::cache_path;
use crate::subscription::EventSource;
use crate::watch_source::WatchSource;

/// What `start` brought up. A failure to launch is data, not a panic: the UI
/// still opens and shows it.
pub struct Started {
    pub process: Option<RpcProcess>,
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

/// Command line arguments are forwarded to `gentle-shell`; a home flag is added
/// when they name none.
pub fn start() -> Started {
    let events = EventSource::default();
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
    let mut watch = WatchSource::default();
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
    if let Some(batches) = batches {
        watch.fill(batches);
    }
    let spawned = launcher::resolve(&LauncherEnv::from_process())
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
    let (process, fatal) = match spawned {
        Ok(mut process) => {
            events.fill(process.take_events());
            (Some(process), None)
        }
        Err(error) => (None, Some(error)),
    };
    Started {
        process,
        events,
        index,
        cache_path,
        sessions_dir,
        watcher,
        watch,
        watch_error,
        fatal,
    }
}

/// Starts watching `sessions_dir`, for the first time and for every retry after
/// a failure. `Err` is worth logging once, not once per pass: the caller decides.
pub fn try_start_watcher(
    sessions_dir: &Path,
) -> Result<(SessionWatcher, Receiver<Batch>), WatchError> {
    SessionWatcher::start(sessions_dir)
}
