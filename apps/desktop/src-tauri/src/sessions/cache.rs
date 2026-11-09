//! The tree of one revision, kept so a window call does not rebuild it.
//!
//! A page of 50 000 chats would otherwise pay a full grouping pass per scroll
//! tick. The tree is valid exactly as long as the holder's revision does not
//! move - it moves when the session list or the config changes - so the revision
//! is the key of this one-slot cache.
//!
//! The build runs OUTSIDE the lock: the caller's closure reads a snapshot of the
//! state and of the config, and only the finished tree is handed to the slot.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::sessions::tree::Tree;

/// The tree of the last built revision.
#[derive(Debug, Default)]
pub struct TreeCache {
    slot: Mutex<Option<Arc<Tree>>>,
}

impl TreeCache {
    /// The tree of `revision`: the cached one when it was built for this very
    /// revision, and otherwise the one `build` answers, which runs outside the
    /// lock.
    pub fn get_or_build(&self, revision: u32, build: impl FnOnce() -> Tree) -> Arc<Tree> {
        // The hit costs one lock and no pass over the session list. The lock is
        // released before `build` runs, so a rebuild never blocks a reader of the
        // slot and never does its work under it.
        if let Some(cached) = lock(&self.slot).as_ref() {
            if cached.revision() == revision {
                return Arc::clone(cached);
            }
        }
        let fresh = Arc::new(build());
        let mut slot = lock(&self.slot);
        // A thread that got here first already stored this very revision.
        if let Some(cached) = slot.as_ref() {
            if cached.revision() == fresh.revision() {
                return Arc::clone(cached);
            }
        }
        *slot = Some(Arc::clone(&fresh));
        fresh
    }
}

/// A lock that survives a poisoned mutex: a thread that died must not leave the
/// tree unreadable for the rest of the run.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
