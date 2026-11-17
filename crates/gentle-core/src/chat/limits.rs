//! The caps that bound how many chat children may live and how many may work.
//!
//! They live here, in the pure crate both sides already depend on, because two
//! crates enforce them and they must never disagree: `gentle-backend`'s
//! `Limits` decides which child may be stopped to make room, and `gentle-app`'s
//! queue decides which prompt waits. A second copy of these numbers is how the
//! reducer ended up capping at a constant that agreed with the host by accident
//! instead of by design.
//!
//! The cost each cap bounds is the user's, not this process's memory alone:
//! every live child is a Node process, and every working child is a model call
//! spending the usage limit.

use std::time::Duration;

/// How many chat children may be alive at once.
///
/// A child is spawned on demand and kept while its chat is worth reopening
/// cheaply, so this is the memory bound: six Node processes is a footprint a
/// desktop app can hold, and the idle window below releases the ones the user
/// stopped touching.
pub const MAX_LIVE: usize = 6;

/// How many of the live children may be running a model call at once.
///
/// The expensive axis: parallel model calls drain the usage limit faster, and
/// every additional working chat is another stream editing the same folder. A
/// prompt over this cap waits instead of being refused.
pub const MAX_WORKING: usize = 3;

/// How long a background child may sit without activity, and without a running
/// model call, before it is reaped.
///
/// A reap costs one start-up when the chat is reopened through `--session`, so
/// the window is generous: fifteen minutes of untouched background chat is
/// still worth keeping alive.
pub const IDLE_AFTER: Duration = Duration::from_secs(15 * 60);
