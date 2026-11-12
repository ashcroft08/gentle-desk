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

use crate::window::WindowRecorder;

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
    /// The session watcher and the safety-net timer: the batch threads and the
    /// timer end on the stop signal within their poll interval, and the platform
    /// watch's own thread ends when its handle drops. This only covers a wedged
    /// one, as the pump's deadline does for the child events.
    pub watch: Duration,
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
            watch: Duration::from_secs(5),
            cache: Duration::from_secs(30),
            config: Duration::from_secs(5),
        }
    }
}

/// What recording where and how big the window was gets before the shutdown goes
/// on without it.
///
/// Its own budget, and the shortest of the steps: reading five numbers off a
/// window that is still up is a handful of quick calls, and a window that is
/// already gone answers with an error at once. A window manager that does not
/// answer at all costs this, and the close then goes on without the state: the
/// file keeps what it last held, which is worth more than an app that will not
/// close.
pub const WINDOW_SAVE: Duration = Duration::from_secs(2);

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
    /// Stops the session watcher and the safety-net timer, and waits at most
    /// `budget` for their threads. Before the cache flush, so no batch asks for
    /// a pass after the last one was written.
    fn stop_watch(&self, budget: Duration);
    /// Writes the session index cache the throttle held back, and seals it: no
    /// pass writes it after this, so the state written here is the one the next
    /// run restores. Blocking by design (one file write); the flow's
    /// [`Deadlines::cache`] covers it.
    fn flush_cache(&self);
    /// Writes the last queued config snapshot, at most `budget`.
    fn flush_config(&self, budget: Duration);
    /// Ends the process, after every step above ran or was abandoned.
    fn exit(&self);
}

/// The close of one run: the steps, their deadlines, the window, and the
/// one-shot gate.
pub struct CloseFlow {
    steps: Arc<dyn CloseSteps>,
    deadlines: Deadlines,
    /// The window of this run, when it has one. The record of it is a step of its
    /// own ([`WINDOW_SAVE`]) and runs before the config flush, so the state on the
    /// disk is the state of this close.
    window: Option<Arc<dyn WindowRecorder>>,
    started: AtomicBool,
}

impl CloseFlow {
    pub fn new(steps: Arc<dyn CloseSteps>, deadlines: Deadlines) -> Self {
        Self {
            steps,
            deadlines,
            window: None,
            started: AtomicBool::new(false),
        }
    }

    /// The same close with the window of this run, so where it was and how big it
    /// was reaches the config before the flush writes it. A close that owns no
    /// window (a test, or a run that could not get one) records nothing.
    pub fn with_window(self, window: Arc<dyn WindowRecorder>) -> Self {
        Self {
            window: Some(window),
            ..self
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
        let window = self.window.clone();
        let spawned = thread::Builder::new()
            .name("app-close".to_owned())
            .spawn(move || shutdown(steps, deadlines, window));
        if let Err(error) = spawned {
            // A window whose close is prevented must still be closeable: with no
            // thread to run them, the same steps run here, bounded as always.
            eprintln!("close: no thread for the shutdown ({error}); running it here");
            shutdown(Arc::clone(&self.steps), self.deadlines, self.window.clone());
        }
        true
    }
}

/// The shutdown itself: every step in order, each bounded, then the exit.
fn shutdown(
    steps: Arc<dyn CloseSteps>,
    deadlines: Deadlines,
    window: Option<Arc<dyn WindowRecorder>>,
) {
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
    let watch_steps = Arc::clone(&steps);
    let watch_budget = deadlines.watch;
    bounded("watch", watch_budget, move || {
        watch_steps.stop_watch(watch_budget)
    });
    let cache_steps = Arc::clone(&steps);
    bounded("cache", deadlines.cache, move || cache_steps.flush_cache());
    // Before the config flush, so what it writes is on the disk before the
    // process goes away. Skipped for a close that owns no window.
    if let Some(window) = window {
        bounded("window", WINDOW_SAVE, move || window.record());
    }
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
