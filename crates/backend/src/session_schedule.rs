//! When the next session refresh may start.
//!
//! Both cadences the refresh strategy needs are one function: a pass may start
//! no sooner than its base interval and no sooner than `K` times the last pass,
//! so a large home refreshes less often instead of spending more of the CPU.
//!
//! - while a file watcher runs, the full pass is only a safety net that
//!   reconciles what the watcher missed ([`RECONCILE_BASE`]);
//! - without a watcher, the full pass is the only update, so it keeps the
//!   shipped 5 s cadence ([`POLL_BASE`]).
//!
//! Nothing here reads the clock or the file system: the caller measures the
//! last pass and decides what to do with the delay.

use std::time::Duration;

/// Base interval of the safety-net full pass while a watcher runs.
pub const RECONCILE_BASE: Duration = Duration::from_secs(60);

/// Base interval of the full pass when no watcher runs (the shipped cadence).
pub const POLL_BASE: Duration = Duration::from_secs(5);

/// A pass may use at most `1 / K` of a core once it is long enough to stretch.
pub const K: u32 = 20;

/// The next refresh may start in this long.
///
/// Never sooner than `base`, and never sooner than `k` times the last pass, so
/// a slow home refreshes less often instead of spending more of the CPU. `k`
/// below 1 is treated as 1: a pass never starts back to back with the previous
/// one, and durations too large to multiply saturate instead of overflowing.
pub fn next_delay(last_pass: Duration, base: Duration, k: u32) -> Duration {
    base.max(last_pass.saturating_mul(k.max(1)))
}

/// The delay before the next *full* pass.
///
/// `watching` is whether a file watcher currently feeds the list: with one, the
/// full pass only reconciles missed events ([`RECONCILE_BASE`]); without one it
/// is the only update and keeps the fast poll ([`POLL_BASE`]).
pub fn full_pass_delay(watching: bool, last_pass: Duration) -> Duration {
    let base = if watching { RECONCILE_BASE } else { POLL_BASE };
    next_delay(last_pass, base, K)
}

#[cfg(test)]
mod tests;
