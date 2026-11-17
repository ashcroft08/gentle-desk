//! The timer that releases the idle chat children (S7).
//!
//! The host holds the rules — which child is idle, which must never be stopped —
//! but rules do nothing on their own: without a driver, live children accumulate
//! silently until the live cap is the only thing keeping their number down. This
//! module is that driver, and nothing else: one thread, one tick, one call.
//!
//! A reap is safe to miss and safe to repeat. The child that goes away reports
//! its own `Closed` through the event stream, so the app learns about the reap
//! the same way it learns about a death, and the chat reopens through `--session`
//! when the user comes back.

use std::sync::Arc;
use std::time::Duration;

use crate::effects::RuntimeExecutor;

/// How often the timer looks. The window it enforces is minutes long, so a
/// minute of slack costs nothing and keeps the thread quiet.
const TICK: Duration = Duration::from_secs(60);

/// Runs the reap on a thread of its own, so no tick ever waits on the window's.
///
/// The loop ends by itself when the run is closed: the close flow stops every
/// child anyway, and a timer that outlived it would only log. A thread that
/// cannot start is a log line, never a fatal: a chat that is never reaped is the
/// behavior of every earlier release.
pub fn start(executor: Arc<RuntimeExecutor>) {
    let spawned = std::thread::Builder::new()
        .name("chat-idle-reaper".to_owned())
        .spawn(move || loop {
            std::thread::sleep(TICK);
            if executor.is_closed() {
                return;
            }
            executor.reap_idle();
        });
    if let Err(error) = spawned {
        eprintln!("chat idle reaper unavailable: {error}");
    }
}
