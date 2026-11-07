//! Reading the session index off the UI thread.
//!
//! The refresh is a `stat` per unchanged file but a full parse of new or
//! rewritten ones, so on a big home it must never run inside the window's
//! event loop. The index is shared behind a mutex; the work runs on its own
//! thread and the caller only awaits the finished list.

use std::sync::{Arc, Mutex};

use gentle_app::SessionItem;
use gentle_backend::session_index::SessionIndex;

use crate::sessions;

pub type SharedIndex = Arc<Mutex<SessionIndex>>;

/// Refreshes the index on a worker thread and returns the list to show.
/// `Err` carries a message fit for the log (unreadable directory, thread lost).
pub async fn refresh(index: SharedIndex) -> Result<Vec<SessionItem>, String> {
    let (done, result) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let outcome = match index.lock() {
            Err(_) => Err("the session index is poisoned".to_owned()),
            Ok(mut index) => index
                .refresh()
                .map(|_| index.entries().iter().map(sessions::to_item).collect())
                .map_err(|error| format!("cannot read sessions: {error}")),
        };
        let _ = done.send(outcome);
    });
    result
        .await
        .unwrap_or_else(|_| Err("the session refresh thread ended unexpectedly".to_owned()))
}

#[cfg(test)]
mod tests;
