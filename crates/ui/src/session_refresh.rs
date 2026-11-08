//! Reading the session index off the UI thread: the iced transport over the
//! core moved to `gentle-runtime`.
//!
//! The core lives in [`gentle_runtime::session_refresh`] (the pass, its
//! throttle and its cache rules); this module keeps what is iced-bound:
//! [`stream`], the pass wrapped in an iced channel the refresh flow pumps, and
//! the test harnesses that drain it. The names here re-export the runtime's
//! so the rest of the window keeps one path.

mod worker;

pub use gentle_runtime::session_refresh::{flush, Progress, Refreshed, Shared, Update};
pub use worker::stream;

#[cfg(test)]
use gentle_app::SessionItem;
#[cfg(test)]
use gentle_backend::session_index::SessionIndex;
#[cfg(test)]
use gentle_runtime::refresh_queue::Job;
#[cfg(test)]
pub(crate) use gentle_runtime::session_refresh::{work, SharedIndex};
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
// Imports the test modules reach through `use super::*`, as before the move.
use std::sync::{Arc, Mutex};
#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
pub(crate) use worker::run;

#[cfg(test)]
mod stream_tests;

#[cfg(test)]
mod test_home;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod worker_tests;

/// Refreshes the index on a worker thread and returns the list to show.
///
/// A full pass with a fresh throttle, so the list is never held back by how
/// often this run persisted the cache. The shipped path is [`stream`] through
/// `refresh_flow`; this wrapper exists for the module's own tests.
#[cfg(test)]
pub async fn refresh(
    index: SharedIndex,
    cache_path: Option<PathBuf>,
) -> Result<Vec<SessionItem>, String> {
    let refreshed = run(Job::Full, Shared::new(index, cache_path)).await?;
    Ok(refreshed.sessions.items().to_vec())
}
