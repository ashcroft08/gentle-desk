//! The ordered shutdown of a run, and the deadlines that bound it.
//!
//! [`CloseFlow::request`] is called from the window's close request, answers at
//! once and does the work on a thread of its own, so the window keeps repainting
//! while children stop and caches are written. A second request does nothing: the
//! started flag is taken once.
//!
//! Each step runs inside [`bounded`]: a step that misses its deadline is logged
//! and left behind, so no hung child and no hung disk keeps the app open. The
//! last step ends the process.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// What each step of a close gets before the shutdown goes on without it.
///
/// The numbers come from what the iced shell did, made finite: its child stop was
/// the backend's 2 s grace per child with no bound on the whole step, and its
/// cache flush was one unbounded write on the window's own thread.
#[derive(Debug, Clone, Copy)]
pub struct Deadlines {
    /// Every live child, the whole step: the host still gives each one its own
    /// 2 s grace and then kills its process tree. Ten seconds covers five
    /// children that all use their whole grace period.
    pub children: Duration,
    /// The child-event threads: they end on the stop signal within their poll
    /// interval, so this only covers a wedged thread.
    pub pump: Duration,
    /// The session index cache: one JSON document, about 29 MB at 50 000 chats.
    pub cache: Duration,
    /// The config writer: one small file, after the queue drains.
    pub config: Duration,
}

impl Default for Deadlines {
    fn default() -> Self {
        Self {
            children: Duration::from_secs(10),
            pump: Duration::from_secs(2),
            cache: Duration::from_secs(30),
            config: Duration::from_secs(5),
        }
    }
}

/// The steps of a close, in the order [`CloseFlow`] runs them. The real ones are
/// [`super::BridgeSteps`]; a test records them.
pub trait CloseSteps: Send + Sync {
    /// Stops accepting new work: no intent, command or child starts after it.
    fn stop_work(&self);
    /// Stops every live chat child. Blocking by design: the flow runs it on its
    /// own thread, bounded by [`Deadlines::children`].
    fn stop_children(&self);
    /// Ends the child-event threads and waits for them, at most `budget`.
    fn join_pump(&self, budget: Duration);
    /// Writes the session index cache the throttle held back. Blocking by
    /// design (one file write); the flow's [`Deadlines::cache`] covers it.
    fn flush_cache(&self);
    /// Writes the last queued config snapshot, at most `budget`.
    fn flush_config(&self, budget: Duration);
    /// Ends the process, after every step above ran or was abandoned.
    fn exit(&self);
}

/// The close of one run: the steps, their deadlines, and the one-shot gate.
pub struct CloseFlow {
    steps: Arc<dyn CloseSteps>,
    deadlines: Deadlines,
    started: AtomicBool,
}

impl CloseFlow {
    pub fn new(steps: Arc<dyn CloseSteps>, deadlines: Deadlines) -> Self {
        Self {
            steps,
            deadlines,
            started: AtomicBool::new(false),
        }
    }

    /// Asks for the shutdown. Answers whether THIS call started it: a second
    /// close request while shutting down, or after it finished, is ignored, so
    /// the steps never run twice and nothing panics.
    pub fn request(&self) -> bool {
        if self.started.swap(true, Ordering::SeqCst) {
            return false;
        }
        let steps = Arc::clone(&self.steps);
        let deadlines = self.deadlines;
        let spawned = thread::Builder::new()
            .name("app-close".to_owned())
            .spawn(move || shutdown(steps, deadlines));
        if let Err(error) = spawned {
            // A window whose close is prevented must still be closeable: with no
            // thread to run them, the same steps run here, bounded as always.
            eprintln!("close: no thread for the shutdown ({error}); running it here");
            shutdown(Arc::clone(&self.steps), self.deadlines);
        }
        true
    }
}

/// The shutdown itself: every step in order, each bounded, then the exit.
fn shutdown(steps: Arc<dyn CloseSteps>, deadlines: Deadlines) {
    // First, so nothing new is started while the rest runs.
    steps.stop_work();
    let child_steps = Arc::clone(&steps);
    bounded("children", deadlines.children, move || {
        child_steps.stop_children()
    });
    let pump_steps = Arc::clone(&steps);
    let pump_budget = deadlines.pump;
    bounded("pump", pump_budget, move || {
        pump_steps.join_pump(pump_budget)
    });
    let cache_steps = Arc::clone(&steps);
    bounded("cache", deadlines.cache, move || cache_steps.flush_cache());
    let config_steps = Arc::clone(&steps);
    let config_budget = deadlines.config;
    bounded("config", config_budget, move || {
        config_steps.flush_config(config_budget)
    });
    // The process goes away last, whatever the steps above came to: their state
    // is on the disk or it is lost, and a window that cannot be closed is worse.
    steps.exit();
}

/// Runs one blocking step on a thread of its own and waits at most `budget`.
/// `false` when it did not finish in time: the step's thread is left behind (it
/// is abandoned, not killed) and the close goes on.
fn bounded(label: &'static str, budget: Duration, step: impl FnOnce() + Send + 'static) -> bool {
    let (done, wait) = mpsc::channel();
    let spawned = thread::Builder::new()
        .name(format!("close-{label}"))
        .spawn(move || {
            step();
            let _ = done.send(());
        });
    if let Err(error) = spawned {
        eprintln!("close: {label} could not run ({error}); going on");
        return false;
    }
    match wait.recv_timeout(budget) {
        Ok(()) => true,
        Err(_) => {
            eprintln!("close: {label} did not finish within {budget:?}; going on");
            false
        }
    }
}
