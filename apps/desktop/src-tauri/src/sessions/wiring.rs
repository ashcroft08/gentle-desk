//! How the refresh of a desktop run is wired: where a watcher is started from,
//! and how long the safety net waits between two passes.

use std::path::Path;
use std::sync::Arc;

use gentle_runtime::refresh_flow::Flow;

use super::timer::TickDelay;
use super::watch::StartWatch;

/// What a desktop run adds around the passes. A run driven by hand (the tests of
/// the passes alone) has none of it: no watcher is started or retried, and no
/// safety net runs.
#[derive(Clone)]
pub struct Wiring {
    /// The seam a full pass starts a watcher through.
    pub start: StartWatch,
    /// The safety net's delay, asked of the flow.
    pub delay: TickDelay,
}

impl Wiring {
    /// The real one: the backend's watcher through
    /// [`gentle_runtime::startup::try_start_watcher`], and the flow's own delay,
    /// which is `crates/backend`'s `session_schedule` (`POLL_BASE` 5 s while the
    /// list polls, `RECONCILE_BASE` 60 s while a watcher runs, both stretched to
    /// `K` times the last full pass).
    pub fn real() -> Self {
        Self {
            start: Arc::new(|dir: &Path| gentle_runtime::startup::try_start_watcher(dir)),
            delay: Arc::new(|flow: &Flow| flow.tick_delay()),
        }
    }
}
