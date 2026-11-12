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
//!
//! The window's thread - where the close request arrives - only ever asks for
//! the shutdown's thread. It never runs a step itself: a shutdown that ran there
//! would hold the window for the sum of every deadline, which is the freeze this
//! close exists to avoid. A request that cannot get a thread runs nothing and
//! answers so, leaving the window the close it already had.

use std::io;
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

/// How many times a close asks for the thread its shutdown runs on.
///
/// A refused spawn is usually the moment, not the process: the run is out of
/// threads for an instant, and the ask that follows a pause gets one. So the
/// close tries again instead of settling for the first refusal, and the whole
/// retry - [`SPAWN_ATTEMPTS`] asks and the pauses between them - costs the
/// caller about a tenth of a second, far less than any step of the shutdown.
/// A close that is still refused runs nothing at all: see
/// [`CloseFlow::request`].
pub const SPAWN_ATTEMPTS: u32 = 3;

/// The pause between two asks for the shutdown's thread: long enough for a
/// thread that was just released to become available, and short enough that the
/// caller's wait for the whole retry stays around a tenth of a second - no part
/// of the shutdown runs while it waits.
const SPAWN_PAUSE: Duration = Duration::from_millis(50);

/// What starts the thread a shutdown runs on. The flow's own spawner names the
/// thread `app-close`; a test injects one that refuses, to prove what a close
/// does when no thread can be had ([`CloseFlow::with_spawner`]). The work is
/// shared rather than moved, so a refused attempt leaves it to the next one.
type Spawner = Arc<dyn Fn(&Arc<dyn Fn() + Send + Sync>) -> io::Result<()> + Send + Sync>;

/// The spawner of the app: one named thread, or the reason there is none.
fn spawn_close_thread(work: &Arc<dyn Fn() + Send + Sync>) -> io::Result<()> {
    let work = Arc::clone(work);
    thread::Builder::new()
        .name("app-close".to_owned())
        .spawn(move || work())
        .map(|_thread| ())
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
    /// How the shutdown's thread is started. [`spawn_close_thread`] in the app;
    /// a test that must see what happens without one replaces it.
    spawn: Spawner,
}

impl CloseFlow {
    pub fn new(steps: Arc<dyn CloseSteps>, deadlines: Deadlines) -> Self {
        Self {
            steps,
            deadlines,
            window: None,
            started: AtomicBool::new(false),
            spawn: Arc::new(spawn_close_thread),
        }
    }

    /// The same close that starts its shutdown's thread with `spawn` instead of
    /// the app's own. Only a test uses it, and only to force the refusal the
    /// app meets when the process has no thread left to give: the close must run
    /// nothing on the caller then.
    pub fn with_spawner(self, spawn: Spawner) -> Self {
        Self { spawn, ..self }
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
    ///
    /// This runs on the window's own thread, where the close request arrives, so
    /// it must not run a step of the shutdown: every deadline of [`Deadlines`]
    /// adds up to about a minute, and a window held for that long is the freeze
    /// this close exists to avoid. When the thread cannot be had, the request
    /// asks [`SPAWN_ATTEMPTS`] times, logs the refusal, and answers `false`
    /// having run nothing: the started gate goes back with it, so the caller
    /// keeps the close it already had and the process ends there, as it did
    /// before there was a flow.
    pub fn request(&self) -> bool {
        if self.started.swap(true, Ordering::SeqCst) {
            return false;
        }
        let steps = Arc::clone(&self.steps);
        let deadlines = self.deadlines;
        let window = self.window.clone();
        let work: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            shutdown(Arc::clone(&steps), deadlines, window.clone());
        });
        let mut refusal = None;
        for attempt in 1..=SPAWN_ATTEMPTS {
            match (self.spawn)(&work) {
                Ok(()) => return true,
                Err(error) => {
                    refusal = Some(error);
                    if attempt < SPAWN_ATTEMPTS {
                        thread::sleep(SPAWN_PAUSE);
                    }
                }
            }
        }
        // No thread for the shutdown, and running it here would freeze the
        // window for the sum of every deadline: so nothing runs, and the gate
        // goes back to the state a close that never happened leaves it in. The
        // caller sees `false`, does not prevent the window's close, and the
        // process ends the way it does without a flow at all.
        self.started.store(false, Ordering::SeqCst);
        let error = refusal.expect("a close that asked for its thread has a reason it has none");
        eprintln!(
            "close: no thread for the shutdown after {SPAWN_ATTEMPTS} asks ({error}); running nothing, the window's own close stands"
        );
        false
    }

    /// Whether a shutdown was started: from the first request that got its
    /// thread, this stays true, and the window's close must be prevented until
    /// the flow's exit step ends the process. It is false when no thread could
    /// be had, so a window whose shutdown never started keeps the close it would
    /// have had without a flow.
    pub fn shutdown_started(&self) -> bool {
        self.started.load(Ordering::SeqCst)
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
