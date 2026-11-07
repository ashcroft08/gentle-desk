//! Starting the `gentle-shell` child and the session index for one run.

use std::path::PathBuf;

use gentle_backend::home::{agent_dir, with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_backend::process::{RpcProcess, SpawnOptions};
use gentle_backend::session_cache;
use gentle_backend::session_index::SessionIndex;

use crate::cache_path;
use crate::subscription::EventSource;

/// What `start` brought up. A failure to launch is data, not a panic: the UI
/// still opens and shows it.
pub struct Started {
    pub process: Option<RpcProcess>,
    pub events: EventSource,
    /// `None` when the pi home cannot be resolved: the sidebar stays empty.
    pub index: Option<SessionIndex>,
    /// Where the index cache is read and written, when the run has one.
    pub cache_path: Option<PathBuf>,
    pub fatal: Option<String>,
}

/// Command line arguments are forwarded to `gentle-shell`; a home flag is added
/// when they name none.
pub fn start() -> Started {
    let events = EventSource::default();
    let home_env = HomeEnv::from_process();
    let args = with_home_flag(std::env::args_os().skip(1).collect(), &home_env);
    let cache_path = cache_path::cache_file(std::env::var_os("LOCALAPPDATA"));
    let index = agent_dir(&args, &home_env).map(|dir| {
        let mut index = SessionIndex::new(dir.join("sessions"));
        // A cache that is missing, stale, corrupt or from another home is
        // ignored and the first refresh does the full scan.
        if let Some(snapshot) = cache_path.as_deref().and_then(session_cache::load) {
            index.restore(snapshot);
        }
        index
    });
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
        fatal,
    }
}
