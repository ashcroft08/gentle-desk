//! Windowless test of the close's session step (T8b): the watcher and the
//! safety net of a running refresh stop inside the deadline the close gives
//! them, through the bridge's own steps (`BridgeSteps`), not through a stub.
//!
//! Nothing here opens a window or a child: the executor never had a chat, the
//! event pump never had a host, and the config writer has no path.

mod session_seam;
mod watch_seam;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::close::{BridgeSteps, CloseFlow, Deadlines};
use gentle_desk_lib::config::ConfigWriter;
use gentle_desk_lib::effects::RuntimeExecutor;
use gentle_desk_lib::events::EventPump;
use gentle_desk_lib::pending::PendingFolders;

use session_seam::Home;
use watch_seam::wired;

/// How long the session threads get: their own poll interval is a tenth of a
/// second, so this only covers a wedged one.
const WATCH: Duration = Duration::from_secs(3);

/// Every deadline short: the close of this run waits on nothing but the session
/// threads.
fn deadlines() -> Deadlines {
    Deadlines {
        children: Duration::from_secs(3),
        pump: Duration::from_secs(2),
        watch: WATCH,
        cache: Duration::from_secs(2),
        config: Duration::from_secs(2),
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn the_close_stops_the_session_watcher_and_its_timer() {
    let home = Home::new();
    home.session("p", "a.jsonl", "first chat");
    // A safety net of 20 ms: by the time the close runs, both session threads
    // are live and firing.
    let wired = wired(&home, Duration::from_millis(20));
    wired.service.boot();
    wired.wait_passes(2);
    wired.wait_starts(1);

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
        None,
        ConfigWriter::spawn(None),
        Box::new({
            let exited = Arc::clone(&exited);
            move || *lock(&exited) = true
        }),
    )
    .with_sessions(Arc::clone(&wired.service));
    let close = CloseFlow::new(Arc::new(steps), deadlines());

    let started = Instant::now();
    assert!(close.request(), "the close starts");
    let deadline = Instant::now() + WATCH;
    while !*lock(&exited) {
        assert!(Instant::now() < deadline, "the close never exited");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        started.elapsed() < WATCH,
        "the close waited for the session threads' whole deadline"
    );

    // The threads are gone, not merely told to go.
    let stopped = wired.passes();
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        wired.passes(),
        stopped,
        "a session thread outlived the close"
    );
}
