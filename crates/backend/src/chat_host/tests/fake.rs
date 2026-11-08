//! A spawner whose children are only data: a test pushes their events and reads
//! what they were asked to do, with no process behind them.
//!
//! - [`gate`] — a condvar gate, used to hold a spawn or a write mid-flight.
//! - [`handle`] — one fake child: its command log, its events, its shutdowns.
//! - [`spawner`] — the spawner itself: what was asked for and which child
//!   answered, plus the journal of every start and stop in order.

use std::sync::{Arc, Mutex};
use std::time::Duration;

mod gate;
mod handle;
mod spawner;

pub use gate::Gate;
pub use handle::FakeChild;
pub use spawner::FakeSpawner;

/// How long a test waits for something that must happen, so a stuck host fails
/// instead of hanging the suite.
pub const WAIT: Duration = Duration::from_secs(5);

/// What the spawner and every child of it append to, so a test can assert the
/// order of starts and stops and not only their counts.
pub type Journal = Arc<Mutex<Vec<String>>>;
