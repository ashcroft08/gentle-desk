//! Windowless test of the close flow's cache step (T8a): the session index cache
//! the throttle held back reaches the disk when the window closes, the throttle
//! is not weakened while the window runs, and the close's flush is the last write
//! of the file even when a pass is still in flight (S2).
//!
//! A temp pi home and a temp cache path; the user's own home is never read.

mod session_seam;
use session_seam::{flow_of, Home};

use std::fs;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_backend::session_cache;
use gentle_backend::session_index::{RefreshStats, SessionIndex};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::close::{BridgeSteps, CloseFlow, Deadlines};
use gentle_desk_lib::config::ConfigWriter;
use gentle_desk_lib::effects::RuntimeExecutor;
use gentle_desk_lib::events::EventPump;
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::sessions::startup;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{self, Refreshed, Shared};

/// How long a test waits for the close to exit.
const LIMIT: Duration = Duration::from_secs(10);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// One pass, run to its end on this thread: a pass reports synchronously, so no
/// wait is needed. The real clock is the pass's clock.
fn pass(job: Job, shared: &Shared) -> Refreshed {
    pass_at(job, shared, shared.now())
}

/// The same pass at `now`, the clock the pass's cache write reads: a test moves
/// time instead of waiting out the throttle's interval.
fn pass_at(job: Job, shared: &Shared, now: Duration) -> Refreshed {
    let mut settled = None;
    session_refresh::work(&job, shared, &mut || now, &mut |update| {
        if let session_refresh::Update::Refreshed(result) = update {
            settled = Some(result);
        }
    });
    settled
        .expect("a pass answers a list")
        .expect("the pass read the home")
}

/// Whether a pass found anything to persist, by the rule the cache write uses.
fn changed(stats: &RefreshStats) -> bool {
    stats.rescanned + stats.appended + stats.removed > 0
}

/// A clock past the throttle's interval: a pass that runs at it has a write the
/// throttle allows. Far enough past the interval that a slow machine only makes
/// it more due, never less.
const PAST_THE_INTERVAL: Duration = Duration::from_secs(3600);

/// What the close leaves on the disk is the last state: a pass whose final write
/// lands after the close flushed writes nothing over it (S2).
///
/// The order is the one the two writers are in when the close wins: the pass
/// reads a session the close's snapshot does not have and reaches its write
/// afterwards, at a clock the throttle would let write.
#[test]
fn a_pass_that_writes_after_the_close_leaves_the_flushed_cache_alone() {
    let home = Home::new();
    home.session("p", "a.jsonl", "uno");
    let flow = flow_of(&home);
    let run = startup::run_pass(flow.shared());
    let (service, _holder, _recorder) = session_seam::build(flow, run);
    let shared = service
        .shared()
        .expect("the home of this test has an index");
    let cache = home.cache();

    // The first pass writes; a change inside the interval is held back, so the
    // close has a state of its own to write.
    let first = pass(Job::Full, &shared);
    assert!(changed(&first.stats), "the first pass found the session");
    let written = fs::read_to_string(&cache).expect("the first change wrote the cache");
    home.session("p", "b.jsonl", "dos");
    let held = pass(Job::Full, &shared);
    assert!(
        changed(&held.stats),
        "the second pass found the new session"
    );
    assert_eq!(
        fs::read_to_string(&cache).unwrap(),
        written,
        "a change within the interval rewrote the cache before the close"
    );

    let holder = Arc::new(AppStateHolder::default());
    let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
    let executor = Arc::new(RuntimeExecutor::new(
        Arc::clone(&pump),
        Arc::new(PendingFolders::default()),
        Arc::clone(&holder),
    ));
    let exited = Arc::new(Mutex::new(false));
    let steps = BridgeSteps::new(
        executor,
        pump,
        service.shared(),
        ConfigWriter::spawn(None),
        Box::new({
            let exited = Arc::clone(&exited);
            move || *lock(&exited) = true
        }),
    );
    let close = CloseFlow::new(Arc::new(steps), Deadlines::default());
    assert!(close.request(), "the close starts");
    let deadline = Instant::now() + LIMIT;
    while !*lock(&exited) {
        assert!(Instant::now() < deadline, "the close never exited");
        std::thread::sleep(Duration::from_millis(5));
    }
    let flushed = fs::read_to_string(&cache).expect("the close wrote the pending state");
    assert_ne!(flushed, written, "the close wrote nothing");

    // The in-flight pass, at a clock the throttle would let write: the close's
    // seal is the only thing that keeps it off the disk.
    home.session("p", "c.jsonl", "tres");
    let late = pass_at(Job::Full, &shared, PAST_THE_INTERVAL);
    assert!(
        changed(&late.stats),
        "the late pass found nothing: this test would prove nothing"
    );
    assert_eq!(
        fs::read_to_string(&cache).unwrap(),
        flushed,
        "a pass wrote the cache after the close's flush"
    );
}

#[test]
fn the_cache_the_throttle_held_back_reaches_the_disk_when_the_close_runs() {
    let home = Home::new();
    home.session("p", "a.jsonl", "uno");
    let file = home.sessions().join("p").join("a.jsonl");
    let flow = flow_of(&home);
    let run = startup::run_pass(flow.shared());
    let (service, _holder, _recorder) = session_seam::build(flow, run);
    let shared = service
        .shared()
        .expect("the home of this test has an index");
    let cache = home.cache();

    // The first pass is a change with nothing written before it, so it writes.
    pass(Job::Full, &shared);
    let written = fs::read_to_string(&cache).expect("the first change wrote the cache");

    // A second change within the throttle's interval is held back, as it is
    // while the window runs: the close must not weaken that rule.
    home.session("p", "a.jsonl", "uno y dos");
    pass(Job::Paths(vec![file]), &shared);
    assert_eq!(
        fs::read_to_string(&cache).unwrap(),
        written,
        "a change within the interval rewrote the cache before the close"
    );

    // The close: the real steps of the bridge over an executor that never
    // opened a chat, with the flow's own cache as the step it flushes.
    let holder = Arc::new(AppStateHolder::default());
    let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
    let executor = Arc::new(RuntimeExecutor::new(
        Arc::clone(&pump),
        Arc::new(PendingFolders::default()),
        Arc::clone(&holder),
    ));
    let exited = Arc::new(Mutex::new(false));
    let steps = BridgeSteps::new(
        executor,
        pump,
        service.shared(),
        ConfigWriter::spawn(None),
        Box::new({
            let exited = Arc::clone(&exited);
            move || *lock(&exited) = true
        }),
    );
    let close = CloseFlow::new(Arc::new(steps), Deadlines::default());
    assert!(close.request(), "the close starts");
    let deadline = Instant::now() + LIMIT;
    while !*lock(&exited) {
        assert!(Instant::now() < deadline, "the close never exited");
        std::thread::sleep(Duration::from_millis(5));
    }

    let flushed = fs::read_to_string(&cache).unwrap();
    assert_ne!(
        flushed, written,
        "the close wrote nothing: the last state never reached the disk"
    );

    // And it is the LAST state: a fresh index restored from it has nothing left
    // to read, which is what the next run needs.
    let mut fresh = SessionIndex::new(home.sessions());
    if let Some(snapshot) = session_cache::load(&cache) {
        fresh.restore(snapshot);
    }
    let restored = Shared::new(Arc::new(Mutex::new(fresh)), None);
    assert_eq!(
        pass(Job::Full, &restored).stats.bytes_read,
        0,
        "the flushed cache was not in sync with the session files"
    );
}
