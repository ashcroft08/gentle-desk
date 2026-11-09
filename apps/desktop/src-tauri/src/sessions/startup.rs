//! What the session list needs from a run: the index, its cache and the flow
//! that decides which pass runs when.
//!
//! This is the sessions half of [`gentle_runtime::startup`], assembled from the
//! same public pieces (`home`, `cache_path`, `session_cache`, `SessionIndex`,
//! `Shared`) for the same reason `effects::runtime` assembles the chat host that
//! way: the runtime's `start` also brings up the chat host and the watcher, and
//! those are other slices. Nothing here starts a watcher. TODO(watcher): start
//! it on boot and adopt its batches once the watcher slice lands.
//!
//! A home that cannot be resolved is data, not a failure: the flow has no index,
//! runs no pass, and the list keeps the reducer's initial loading state rather
//! than claiming the home is empty.

use std::sync::{Arc, Mutex};

use gentle_backend::home::{agent_dir, with_home_flag, HomeEnv};
use gentle_backend::session_cache;
use gentle_backend::session_index::SessionIndex;
use gentle_runtime::cache_path;
use gentle_runtime::refresh_flow::Flow;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{self, Shared, Update};

use super::service::{RunPass, SessionsService};
use crate::bridge::AppStateHolder;

/// The message a pass of a run without a home reports. It never reaches a user:
/// a flow without an index answers `Idle` to every request, so no pass starts.
const NO_HOME: &str = "no pi home to read sessions from";

/// The sessions of this run: the flow of its index, and the service that runs
/// the passes it asks for.
pub fn service(holder: Arc<AppStateHolder>) -> SessionsService {
    let flow = flow();
    let run = run_pass(flow.shared());
    SessionsService::new(holder, flow, run)
}

/// The refresh flow of this run: the index restored from the cache when it is
/// valid, the directory a pass enumerates, and no watcher.
fn flow() -> Flow {
    let home_env = HomeEnv::from_process();
    let args = with_home_flag(std::env::args_os().skip(1).collect(), &home_env);
    // A missing sessions directory is an empty list, not an error: the flow
    // still runs the pass that settles the list to complete.
    let sessions_dir = agent_dir(&args, &home_env).map(|dir| dir.join("sessions"));
    let cache = cache_path::cache_file(std::env::var_os("LOCALAPPDATA"));
    let shared = sessions_dir.clone().map(|sessions| {
        let mut index = SessionIndex::new(sessions);
        // A cache that is missing, stale, corrupt or from another home is
        // ignored and the first pass does the full scan.
        if let Some(snapshot) = cache.as_deref().and_then(session_cache::load) {
            index.restore(snapshot);
        }
        Shared::new(Arc::new(Mutex::new(index)), cache)
    });
    Flow::new(shared, sessions_dir, None, None)
}

/// The real pass: [`session_refresh::work`] on the index, reporting to whoever
/// asked for it.
pub fn run_pass(shared: Option<Shared>) -> RunPass {
    Arc::new(
        move |job: Job, emit: &mut dyn FnMut(Update)| match shared.as_ref() {
            Some(shared) => {
                session_refresh::work(&job, shared, &mut || shared.now(), &mut |update| {
                    emit(update)
                });
            }
            None => emit(Update::Refreshed(Err(NO_HOME.to_owned()))),
        },
    )
}
