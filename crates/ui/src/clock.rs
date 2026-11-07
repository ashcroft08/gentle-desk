//! The wall clock, kept out of `gentle-app` so its `update` stays pure.

use std::time::{SystemTime, UNIX_EPOCH};

/// Now, as epoch milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}
