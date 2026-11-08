//! What a running pass reports: the partial lists and the throttle that bounds
//! them.
//!
//! The first full pass over a big home takes seconds, so it reports the list as
//! it has folded it so far. The throttle is time-based on purpose: files vary
//! in size, so "every N files" would flood the UI thread on a fast home and stay
//! silent on a slow one. One item per [`INTERVAL`] bounds the cost whatever the
//! pass does.
//!
//! Pure: no thread, no channel, no iced. The worker hands in the time it reads
//! from its own clock, so the tests drive it without waiting.

use std::time::Duration;

use gentle_app::PreparedSessions;

use super::Refreshed;

/// How often a running pass may report: twice a second is enough to watch the
/// counters move and cheap enough for a 50 000-chat home.
pub const INTERVAL: Duration = Duration::from_millis(500);

/// One item of a pass's stream.
#[derive(Debug, Clone)]
pub enum Update {
    /// The list the pass has read so far, with how far it got. Only the run's
    /// first full pass sends these.
    Progress(Progress),
    /// The pass finished: always the stream's last item.
    Refreshed(Result<Refreshed, String>),
}

/// A partial list from a running pass, ready for
/// [`Input::SessionsProgress`](gentle_app::Input::SessionsProgress).
#[derive(Debug, Clone)]
pub struct Progress {
    /// The prepared list, built on the worker thread so `update` only swaps it in.
    pub sessions: PreparedSessions,
    /// Session files folded so far; `0` for the list restored from the cache.
    pub done: usize,
    /// Session files the pass found; `0` for the list restored from the cache.
    pub total: usize,
}

/// Decides when a partial list may be sent.
#[derive(Debug)]
pub struct Throttle {
    interval: Duration,
    last: Option<Duration>,
}

impl Throttle {
    /// A throttle that lets one item through every `interval`.
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: None,
        }
    }

    /// Whether an item may be sent at `now`, recording that it was: at once for
    /// the first one, then only once `interval` has passed since the last one
    /// sent. A refused ask records nothing, so asking often cannot shorten the
    /// interval.
    pub fn allow(&mut self, now: Duration) -> bool {
        match self.last {
            None => {}
            Some(last) if now.saturating_sub(last) < self.interval => return false,
            Some(_) => {}
        }
        self.last = Some(now);
        true
    }
}

impl Default for Throttle {
    fn default() -> Self {
        Self::new(INTERVAL)
    }
}

#[cfg(test)]
mod tests;
