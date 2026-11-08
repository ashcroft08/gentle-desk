//! How often the persisted session cache may be rewritten.
//!
//! The cache is one JSON document holding every session (about 29 MB at 50 000
//! chats), so writing it per change would cost more than the refresh that
//! produced the change. The rule is a minimum interval between two writes:
//! the first change after a save only marks the cache dirty, and the next pass
//! that finds it dirty after the interval writes it. [`SaveThrottle::flush`]
//! writes it regardless, for when the window closes.
//!
//! Pure: the caller passes the time in, so the whole rule is testable without
//! sleeping or a clock.

use std::time::Duration;

/// How long to wait between two writes of the persisted cache.
pub const MIN_INTERVAL: Duration = Duration::from_secs(30);

/// Whether the cache is dirty and when it was last written, in the caller's
/// time base.
#[derive(Debug, Clone)]
pub struct SaveThrottle {
    min_interval: Duration,
    dirty: bool,
    last_saved: Option<Duration>,
}

impl SaveThrottle {
    /// `min_interval` is a parameter so tests can use a small value; production
    /// uses [`MIN_INTERVAL`].
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            dirty: false,
            last_saved: None,
        }
    }

    /// Something the cache holds has changed since the last write.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Whether a save is due at `now`.
    ///
    /// True for a change that was never followed by a write, and otherwise
    /// once `min_interval` has passed since the last one. A clock that went
    /// backwards only delays the save: the elapsed time saturates at zero.
    pub fn should_save(&self, now: Duration) -> bool {
        if !self.dirty {
            return false;
        }
        match self.last_saved {
            None => true,
            Some(last) => now.saturating_sub(last) >= self.min_interval,
        }
    }

    /// Records that the cache was written at `now`.
    pub fn saved(&mut self, now: Duration) {
        self.dirty = false;
        self.last_saved = Some(now);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for SaveThrottle {
    fn default() -> Self {
        Self::new(MIN_INTERVAL)
    }
}

#[cfg(test)]
mod tests;
