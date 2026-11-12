//! Windowless tests of the close flow (T8a): the ordered shutdown a close
//! request starts, off the window's thread, each step bounded by its own
//! deadline.
//!
//! The seam is [`CloseSteps`]: a recorder here, the bridge's real pieces in
//! `BridgeSteps` (`close_gate.rs` runs those over an idle run). Nothing in this
//! file starts a child or opens a window; the process-level proof lives in
//! `fake_shell.rs`.

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::close::{CloseFlow, CloseSteps, Deadlines, SPAWN_ATTEMPTS};

/// How long a test waits for a shutdown that its own deadlines bound.
const LIMIT: Duration = Duration::from_secs(10);

/// The step names, in the order the flow runs them.
const ORDER: [&str; 7] = [
    "work", "children", "pump", "watch", "cache", "config", "exit",
];

/// Every deadline short: the tests never wait for a real one to run out.
fn deadlines() -> Deadlines {
    Deadlines {
        children: Duration::from_secs(3),
        pump: Duration::from_secs(2),
        watch: Duration::from_secs(2),
        cache: Duration::from_secs(2),
        config: Duration::from_secs(2),
    }
}

/// The steps of a close, recorded. With `hang` naming one of them, that step
/// never returns: the flow must abandon it and go on.
struct Recorded {
    ran: Arc<Mutex<Vec<&'static str>>>,
    hang: Option<&'static str>,
}

impl Recorded {
    fn new(hang: Option<&'static str>) -> (Self, Arc<Mutex<Vec<&'static str>>>) {
        let ran = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                ran: Arc::clone(&ran),
                hang,
            },
            ran,
        )
    }

    fn record(&self, step: &'static str) {
        lock(&self.ran).push(step);
        if self.hang == Some(step) {
            // Long enough to outlive every deadline, and the thread is abandoned.
            std::thread::sleep(Duration::from_secs(60));
        }
    }
}

impl CloseSteps for Recorded {
    fn stop_work(&self) {
        self.record("work");
    }

    fn stop_children(&self) {
        self.record("children");
    }

    fn join_pump(&self, _budget: Duration) {
        self.record("pump");
    }

    fn stop_watch(&self, _budget: Duration) {
        self.record("watch");
    }

    fn flush_cache(&self) {
        self.record("cache");
    }

    fn flush_config(&self, _budget: Duration) {
        self.record("config");
    }

    fn exit(&self) {
        self.record("exit");
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Waits for the shutdown to reach `step`, failing the test instead of hanging.
fn wait_for(ran: &Arc<Mutex<Vec<&'static str>>>, step: &str) {
    let deadline = Instant::now() + LIMIT;
    while !lock(ran).contains(&step) {
        assert!(
            Instant::now() < deadline,
            "waited {LIMIT:?} for the close to reach {step}, ran {:?}",
            lock(ran)
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn every_step_runs_once_in_order_and_the_process_exits_last() {
    let (steps, ran) = Recorded::new(None);
    let flow = CloseFlow::new(Arc::new(steps), deadlines());

    assert!(
        flow.request(),
        "the first close request is the one that starts the shutdown"
    );
    wait_for(&ran, "exit");

    assert_eq!(
        *lock(&ran),
        ORDER.to_vec(),
        "the shutdown runs every step once, in order, and exits last"
    );
}

#[test]
fn a_second_close_request_while_shutting_down_runs_nothing_again() {
    let (steps, ran) = Recorded::new(None);
    let flow = CloseFlow::new(Arc::new(steps), deadlines());

    assert!(flow.request());
    assert!(
        !flow.request(),
        "a second close request is ignored, not run twice"
    );
    wait_for(&ran, "exit");
    assert!(
        !flow.request(),
        "nor does a later one restart a shutdown that already finished"
    );
    std::thread::sleep(Duration::from_millis(200));

    assert_eq!(*lock(&ran), ORDER.to_vec(), "one shutdown, never two");
}

/// A spawner as the flow asks for one: start a thread for the shutdown's work,
/// or say why there is none.
type Spawn = Arc<dyn Fn(&Arc<dyn Fn() + Send + Sync>) -> io::Result<()> + Send + Sync>;

/// A spawner that never starts a thread: what a close meets when the process
/// has no thread to give it. It counts the asks, so the test sees the retry.
fn refusing_spawner(asks: Arc<AtomicUsize>) -> Spawn {
    Arc::new(move |_work: &Arc<dyn Fn() + Send + Sync>| {
        asks.fetch_add(1, Ordering::SeqCst);
        Err(io::Error::new(io::ErrorKind::WouldBlock, "no thread"))
    })
}

#[test]
fn a_close_that_cannot_get_a_thread_runs_nothing_on_the_caller() {
    // The window's thread is where `CloseRequested` arrives. A shutdown run here
    // would hold it for the sum of every deadline (about a minute), which is the
    // freeze the close exists to avoid: with no thread for it, the close runs no
    // step at all and the window keeps the close it already had.
    let (steps, ran) = Recorded::new(None);
    let asks = Arc::new(AtomicUsize::new(0));
    let flow = CloseFlow::new(Arc::new(steps), deadlines())
        .with_spawner(refusing_spawner(Arc::clone(&asks)));

    let started = Instant::now();
    let did_start = flow.request();
    let took = started.elapsed();

    assert!(
        !did_start,
        "a close that could not start its thread did not start the shutdown"
    );
    assert!(
        lock(&ran).is_empty(),
        "no step ran, least of all on the caller's thread: {:?}",
        lock(&ran)
    );
    assert!(
        !flow.shutdown_started(),
        "no shutdown is underway, so the window's own close is still free to run"
    );
    assert_eq!(
        asks.load(Ordering::SeqCst),
        SPAWN_ATTEMPTS as usize,
        "the close asks for its thread a bounded number of times, no more and no fewer"
    );
    assert!(
        took < Duration::from_secs(2),
        "the retries are a pause, not a wait: {took:?}"
    );
}

#[test]
fn a_close_that_could_not_get_a_thread_leaves_a_later_request_able_to_start_it() {
    // The refusal is often the moment, not the process: a later close request
    // must still be able to get its thread, so the started gate goes back.
    let (steps, ran) = Recorded::new(None);
    let asks = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&asks);
    let spawn: Spawn = Arc::new(move |work: &Arc<dyn Fn() + Send + Sync>| {
        // The first close's attempts are refused; then a thread is there.
        if counter.fetch_add(1, Ordering::SeqCst) < SPAWN_ATTEMPTS as usize {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "no thread"));
        }
        let work = Arc::clone(work);
        std::thread::Builder::new()
            .name("app-close".to_owned())
            .spawn(move || work())
            .map(|_thread| ())
    });
    let flow = CloseFlow::new(Arc::new(steps), deadlines()).with_spawner(spawn);

    assert!(!flow.request(), "the first close got no thread");
    assert!(lock(&ran).is_empty(), "and ran nothing instead");

    assert!(
        flow.request(),
        "a later request starts the shutdown the earlier one could not"
    );
    wait_for(&ran, "exit");
    assert_eq!(
        *lock(&ran),
        ORDER.to_vec(),
        "the shutdown that did start runs every step once, in order"
    );
}

#[test]
fn a_step_that_never_returns_is_abandoned_so_the_app_still_exits() {
    // The cache write is the step that blocks on a slow disk: with a child of a
    // 29 MB document stuck, the window must still close.
    let (steps, ran) = Recorded::new(Some("cache"));
    let flow = CloseFlow::new(Arc::new(steps), deadlines());

    let started = Instant::now();
    assert!(flow.request());
    wait_for(&ran, "exit");
    let took = started.elapsed();

    // The steps after the hung one ran anyway, and the exit came without waiting
    // for it: the hung step's own deadline is what bounds the close.
    assert_eq!(*lock(&ran), ORDER.to_vec());
    assert!(
        took < Duration::from_secs(5),
        "the shutdown waited for a step that never returns: {took:?}"
    );
}
