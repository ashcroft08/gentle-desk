//! The close flow: what happens when the window is asked to close (m19 T8a).
//!
//! The ROLE is the iced shell's
//! (`crates/ui/src/app/sessions_update.rs::on_close_requested`), never its code:
//! there the close request flushed the session cache, shut every child down and
//! exited. Two things change here:
//!
//! - the whole shutdown runs OFF the window's thread, so a child that takes its
//!   grace period (2 s, the backend's own) and a 29 MB cache write never freeze
//!   the window;
//! - every step is bounded by its own deadline ([`Deadlines`]), so a child or a
//!   disk that hangs cannot keep the app open forever.
//!
//! [`flow`] owns the order and the deadlines, [`bridge`] the real steps. The
//! only entry point is [`CloseFlow::request`], which runs at most once.

pub mod bridge;
pub mod flow;

pub use bridge::BridgeSteps;
pub use flow::{CloseFlow, CloseSteps, Deadlines, SPAWN_ATTEMPTS};

/// What a chat command answers once the close has stopped new work. One line,
/// the same one the reducer's failed-open path carries.
pub const CLOSING: &str = "the app is closing";

/// The last step of the orderly close: leave, or start the app again when the
/// run asked for a relaunch (a home chosen at the welcome question only applies
/// to a new run). The flag is read when the step RUNS, after everything was
/// flushed, so a relaunch asked for while the close was under way still counts.
pub fn exit_step(
    relaunch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    restart: impl Fn() + Send + Sync + 'static,
    exit: impl Fn() + Send + Sync + 'static,
) -> Box<dyn Fn() + Send + Sync> {
    Box::new(move || {
        if relaunch.load(std::sync::atomic::Ordering::SeqCst) {
            restart();
        } else {
            exit();
        }
    })
}
