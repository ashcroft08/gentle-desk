//! Windowless test of the close's session step (T8b): the watcher and the
//! safety net of a running refresh stop inside the deadline the close gives
//! them, through the bridge's own steps (`BridgeSteps`), not through a stub.
//!
//! Nothing here opens a window or a child: the executor never had a chat, the
//! event pump never had a host, and the config writer has no path.
//!
//! The same step now ends the Settings files' watcher too (rpc-settings-sync
//! T3b-1, S2): the watch of pi's config directories stops there, before the cache
//! and the config are written, so its notice can never ask a pane to read a file
//! the close is finishing with.

mod session_seam;
mod watch_seam;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::close::{BridgeSteps, CloseFlow, CloseSteps, Deadlines};
use gentle_desk_lib::config::ConfigWriter;
use gentle_desk_lib::effects::RuntimeExecutor;
use gentle_desk_lib::events::EventPump;
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::settings::{watch::OnChange, Changed, Dirs, SettingsWatch};
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{self, Shared};

use session_seam::{flow_of, Home};
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

/// How long a test waits for the settings watcher's own notice: the platform
/// hands it over late on NTFS, so this is a deadline, never a fixed sleep.
const NOTICE: Duration = Duration::from_secs(5);

/// The quiet window the settings watcher documents, plus a margin: what a test
/// waits to prove that NO notice comes after the stop.
const QUIET: Duration = Duration::from_millis(900);

/// The two Settings directories of one test, under a temp directory of its own:
/// the config home of the profile files and the agent directory of the auth
/// store, exactly the pair `SettingsPaths` resolves in the app.
struct SettingsHome {
    dir: tempfile::TempDir,
}

impl SettingsHome {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("a temp settings home"),
        }
    }

    fn config(&self) -> PathBuf {
        self.folder("config")
    }

    fn agent(&self) -> PathBuf {
        self.folder("agent")
    }

    fn folder(&self, name: &str) -> PathBuf {
        let dir = self.dir.path().join(name);
        fs::create_dir_all(&dir).expect("the scratch settings directory");
        dir
    }

    fn dirs(&self) -> Dirs {
        Dirs {
            config: Some(self.config()),
            agent: Some(self.agent()),
        }
    }
}

/// The watcher's push, recorded: what proves it was live before the step and
/// silent after it.
fn recorder(seen: &Arc<Mutex<Vec<Changed>>>) -> OnChange {
    let seen = Arc::clone(seen);
    Arc::new(move |changed| lock(&seen).push(changed))
}

/// Writes over the profiles file and waits for the notice it must cause, so a
/// silence later is the stop's and never a watch that never worked.
fn await_one_notice(seen: &Arc<Mutex<Vec<Changed>>>, profiles: &Path) {
    fs::write(profiles, "{\"one\":true}").expect("the scratch profiles file");
    let deadline = Instant::now() + NOTICE;
    while lock(seen).is_empty() {
        assert!(
            Instant::now() < deadline,
            "the settings watcher never reported a write"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// One pass, run to its end on this thread (`close_cache.rs`'s seam): a pass
/// reports synchronously, so no wait is needed and the throttle's clock is the
/// real one.
fn pass(shared: &Shared) {
    let mut answered = false;
    session_refresh::work(&Job::Full, shared, &mut || shared.now(), &mut |update| {
        if matches!(update, session_refresh::Update::Refreshed(_)) {
            answered = true;
        }
    });
    assert!(answered, "the pass answered no list");
}

/// The close's `watch` step is where the Settings files' watcher ends, and it
/// ends BEFORE the cache is written: the step alone stops it, and the cache file
/// the next step writes still holds the pre-close state when it returns.
#[test]
fn the_watch_step_stops_the_settings_watcher_before_the_cache_is_written() {
    let home = Home::new();
    home.session("p", "a.jsonl", "uno");
    let flow = flow_of(&home);
    let shared = flow.shared().expect("the home of this test has an index");
    let cache = home.cache();

    // A change the throttle holds back, so the close has a state of its own to
    // write: without it the cache step would write nothing and this test could
    // not tell "before the flush" from "never flushed".
    pass(&shared);
    let written = fs::read_to_string(&cache).expect("the first change wrote the cache");
    home.session("p", "b.jsonl", "dos");
    pass(&shared);
    assert_eq!(
        fs::read_to_string(&cache).expect("the cache is still there"),
        written,
        "a change within the throttle's interval rewrote the cache before the close"
    );

    let settings = SettingsHome::new();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let watching = Arc::new(SettingsWatch::new());
    assert!(
        watching.start(&settings.dirs(), recorder(&seen)),
        "the settings watch started"
    );
    await_one_notice(&seen, &settings.config().join("profiles.json"));

    let holder = Arc::new(AppStateHolder::default());
    let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
    let executor = Arc::new(RuntimeExecutor::new(
        Arc::clone(&pump),
        Arc::new(PendingFolders::default()),
        Arc::clone(&holder),
    ));
    let steps = BridgeSteps::new(
        executor,
        pump,
        Some(shared),
        ConfigWriter::spawn(None),
        Box::new(|| {}),
    )
    .with_settings_watch(Arc::clone(&watching));

    // The step on its own, as the close's flow runs it: `watch`, then `cache`.
    steps.stop_watch(WATCH);
    assert!(
        !watching.is_watching(),
        "the close's watch step left the settings watcher running"
    );
    assert_eq!(
        fs::read_to_string(&cache).expect("the cache is still there"),
        written,
        "the cache was written before the watch step ended, so the stop cannot be shown to come first"
    );

    // The write the stop must precede, through the step that makes it.
    steps.flush_cache();
    assert_ne!(
        fs::read_to_string(&cache).expect("the cache step wrote nothing"),
        written,
        "the cache step wrote nothing: this test would prove nothing about order"
    );

    // And the stopped watcher reports nothing more.
    fs::write(settings.config().join("profiles.json"), "{\"late\":true}")
        .expect("the scratch profiles file");
    std::thread::sleep(QUIET);
    assert_eq!(
        lock(&seen).len(),
        1,
        "the stopped settings watcher reported a change"
    );
}

/// The whole close, through the flow, ends the Settings files' watcher too: after
/// the exit step nothing of it runs and nothing more is reported, so no notice
/// asks a pane to read a file the close has just written.
#[test]
fn the_close_ends_the_settings_watcher() {
    let settings = SettingsHome::new();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let watching = Arc::new(SettingsWatch::new());
    assert!(
        watching.start(&settings.dirs(), recorder(&seen)),
        "the settings watch started"
    );
    await_one_notice(&seen, &settings.config().join("profiles.json"));

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
    .with_settings_watch(Arc::clone(&watching));
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
        "the close waited for the settings watcher's whole deadline"
    );

    assert!(
        !watching.is_watching(),
        "the close left the settings watcher running"
    );
    fs::write(settings.config().join("profiles.json"), "{\"late\":true}")
        .expect("the scratch profiles file");
    std::thread::sleep(QUIET);
    assert_eq!(
        lock(&seen).len(),
        1,
        "a stopped settings watcher reported a change after the close"
    );
}
