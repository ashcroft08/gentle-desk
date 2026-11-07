//! Reading the session index off the UI thread.
//!
//! The refresh is a `stat` per unchanged file but a full parse of new or
//! rewritten ones, so on a big home it must never run inside the window's
//! event loop. The index is shared behind a mutex; the work runs on its own
//! thread and the caller only awaits the finished list.
//!
//! The same thread persists the index afterwards, but only when the refresh
//! changed something: an unchanged home must not rewrite the cache file.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gentle_app::SessionItem;
use gentle_backend::session_cache;
use gentle_backend::session_index::{RefreshStats, SessionIndex};

use crate::sessions;

pub type SharedIndex = Arc<Mutex<SessionIndex>>;

/// Refreshes the index on a worker thread and returns the list to show.
/// `Err` carries a message fit for the log (unreadable directory, thread lost).
pub async fn refresh(
    index: SharedIndex,
    cache_path: Option<PathBuf>,
) -> Result<Vec<SessionItem>, String> {
    let (done, result) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let outcome = match index.lock() {
            Err(_) => Err("the session index is poisoned".to_owned()),
            Ok(mut index) => match index.refresh() {
                Err(error) => Err(format!("cannot read sessions: {error}")),
                Ok(stats) => {
                    store(&index, cache_path.as_deref(), &stats);
                    Ok(index.entries().iter().map(sessions::to_item).collect())
                }
            },
        };
        let _ = done.send(outcome);
    });
    result
        .await
        .unwrap_or_else(|_| Err("the session refresh thread ended unexpectedly".to_owned()))
}

/// Writes the index to the cache after a refresh that changed something. A
/// cache that cannot be written only costs the next run its speedup, so the
/// failure is logged and the refresh still delivers its list.
fn store(index: &SessionIndex, cache_path: Option<&Path>, stats: &RefreshStats) {
    let changed = stats.rescanned + stats.appended + stats.removed > 0;
    let Some(path) = cache_path.filter(|_| changed) else {
        return;
    };
    if let Err(error) = session_cache::save(path, &index.snapshot()) {
        eprintln!("cannot write the session index cache: {error}");
    }
}

#[cfg(test)]
mod tests;
