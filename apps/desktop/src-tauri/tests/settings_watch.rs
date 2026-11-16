//! The Settings files' watcher on real temp directories (rpc-settings-sync
//! T3b-1, S2): the native watch, the debounced notice, and the degradation.
//!
//! Every test makes its own config home and agent directory under a temp
//! directory outside the worktree, so the maintainer's own pi files are never
//! read, written or watched, and no test starts a child or a window.
//!
//! Every wait POLLS with a deadline and never sleeps a fixed time: NTFS hands
//! the notice of a change over seconds late (the sessions watcher measured about
//! 3 s on a developer machine), so a short fixed sleep would be the flakiest
//! thing here.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::settings::watch::{Changed, Dirs, OnChange, SettingsWatch};
/// How long a test waits for a notice before it calls itself a miss.
const LIMIT: Duration = Duration::from_secs(5);

/// The pause between two looks at the recorder.
const POLL: Duration = Duration::from_millis(20);

/// The quiet window the watcher documents, plus a margin: what a test waits to
/// prove that NO notice comes.
const QUIET: Duration = Duration::from_millis(900);

/// A run's two directories, under a temp directory of its own. Dropping it
/// removes them.
struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("a temp home"),
        }
    }

    /// The config home, which holds both profile files.
    fn config(&self) -> PathBuf {
        create(self.dir.path().join("gentle-ai"))
    }

    /// The agent directory, which holds the auth store.
    fn agent(&self) -> PathBuf {
        create(self.dir.path().join("agent"))
    }

    fn dirs(&self) -> Dirs {
        Dirs {
            config: Some(self.config()),
            agent: Some(self.agent()),
        }
    }
}

/// One directory that exists, created beforehand: the watch is registered on a
/// directory, never on a file pi has not written yet.
fn create(dir: PathBuf) -> PathBuf {
    fs::create_dir_all(&dir).expect("the temp directory is creatable");
    dir
}

/// What the watcher pushed, in order, as a recorder the test reads.
#[derive(Clone, Default)]
struct Recorder {
    seen: Arc<Mutex<Vec<Changed>>>,
}

impl Recorder {
    fn new() -> Self {
        Self::default()
    }

    fn on_change(&self) -> OnChange {
        let seen = Arc::clone(&self.seen);
        Arc::new(move |changed| lock(&seen).push(changed))
    }

    fn seen(&self) -> Vec<Changed> {
        lock(&self.seen).clone()
    }

    /// Waits until `count` notices were pushed, and answers them all.
    fn wait_for(&self, count: usize) -> Vec<Changed> {
        let started = Instant::now();
        loop {
            let seen = self.seen();
            if seen.len() >= count {
                return seen;
            }
            assert!(
                started.elapsed() < LIMIT,
                "the watcher pushed {} of {count} notices within {LIMIT:?}",
                seen.len()
            );
            std::thread::sleep(POLL);
        }
    }

    /// Everything the watcher pushed in the quiet window that follows: how a
    /// test proves that a change reports NOTHING.
    fn within_the_quiet_window(&self) -> Vec<Changed> {
        std::thread::sleep(QUIET);
        self.seen()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn profiles_only() -> Changed {
    Changed {
        profiles: true,
        account: false,
    }
}

fn account_only() -> Changed {
    Changed {
        profiles: false,
        account: true,
    }
}

/// A file pi writes - or replaces - in the config home is ONE Profiles notice:
/// the watcher never says which file it was, only which group changed.
#[test]
fn a_write_to_the_profiles_file_is_one_profiles_notice() {
    let home = Home::new();
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    fs::write(home.config().join("profiles.json"), "{}").expect("the scratch profiles file");

    assert_eq!(recorder.wait_for(1), vec![profiles_only()]);
    assert!(watching.stop(LIMIT), "the watcher thread ended");
}

/// pi never writes its config files in place: it writes a temp file and renames
/// it over the old one. The rename is the change the pane must see, which is why
/// the watch is on the directory and not on the file.
#[test]
fn an_atomic_replace_of_the_auth_file_is_one_account_notice() {
    let home = Home::new();
    let path = home.agent().join("auth.json");
    fs::write(&path, "{\"first\":true}").expect("the scratch auth file");
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    let temp = home.agent().join("auth.json.tmp");
    fs::write(&temp, "{\"second\":true}").expect("the scratch temp file");
    fs::rename(&temp, &path).expect("the rename over the auth store");

    assert_eq!(recorder.wait_for(1), vec![account_only()]);
    assert!(watching.stop(LIMIT), "the watcher thread ended");
}

/// A file the panes never read is not a change: the notice only names the three
/// files of the two groups, and the watch stays live for the ones it does name.
#[test]
fn a_file_the_panes_do_not_read_reports_nothing() {
    let home = Home::new();
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    for name in ["unrelated.json", "profiles.json.tmp", "settings.json"] {
        fs::write(home.config().join(name), "{}").expect("the scratch file");
        fs::write(home.agent().join(name), "{}").expect("the scratch file");
    }
    assert_eq!(
        recorder.within_the_quiet_window(),
        Vec::<Changed>::new(),
        "a file the panes do not read was reported"
    );

    // The same watch still reports the file it does read: the silence above is
    // the filter's, not a dead watcher's.
    fs::write(home.config().join("profile-fallbacks.json"), "{}").expect("the chain file");
    assert_eq!(recorder.wait_for(1), vec![profiles_only()]);
    assert!(watching.stop(LIMIT), "the watcher thread ended");
}

/// Five writes in a row are ONE notice: the quiet window is what keeps the panes
/// from re-reading the same file five times.
#[test]
fn five_rapid_writes_are_one_notice() {
    let home = Home::new();
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    for step in 0..5 {
        fs::write(
            home.config().join("profiles.json"),
            format!("{{\"step\":{step}}}"),
        )
        .expect("the scratch profiles file");
    }

    assert_eq!(recorder.wait_for(1), vec![profiles_only()]);
    assert_eq!(
        recorder.within_the_quiet_window(),
        vec![profiles_only()],
        "the burst was reported more than once"
    );
    assert!(watching.stop(LIMIT), "the watcher thread ended");
}

/// A file the terminal deletes and writes again is a change each time, so the
/// pane re-reads a store that went away and came back.
#[test]
fn a_delete_and_a_recreate_are_reported() {
    let home = Home::new();
    let path = home.config().join("profiles.json");
    fs::write(&path, "{}").expect("the scratch profiles file");
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    fs::remove_file(&path).expect("the profiles file is removed");
    assert_eq!(recorder.wait_for(1), vec![profiles_only()]);

    fs::write(&path, "{\"again\":true}").expect("the profiles file is written again");
    let seen = recorder.wait_for(2);
    assert_eq!(seen, vec![profiles_only(), profiles_only()]);
    assert!(watching.stop(LIMIT), "the watcher thread ended");
}

/// A directory that is not there is NOT a failure and never reaches the user:
/// no watch runs, nothing is pushed, and the run keeps the focus refresh. The
/// directory appearing later is the next start's business - nothing polls.
#[test]
fn a_directory_that_is_not_there_starts_nothing() {
    let home = Home::new();
    let recorder = Recorder::new();
    let dirs = Dirs {
        config: Some(home.dir.path().join("not-created-yet")),
        agent: Some(home.dir.path().join("nor-this-one")),
    };

    let watching = SettingsWatch::new();
    assert!(
        !watching.start(&dirs, recorder.on_change()),
        "a watch started on directories that do not exist"
    );
    assert!(!watching.is_watching());
    assert_eq!(recorder.within_the_quiet_window(), Vec::<Changed>::new());
    // Stopping a watcher that never started answers at once.
    assert!(watching.stop(LIMIT));
}

/// The stop ends the thread, and nothing is reported after it: the close runs
/// the stop before the config and the cache are written, so a late notice can
/// never ask for a read of a file that is being flushed.
#[test]
fn a_stopped_watch_reports_nothing_more() {
    let home = Home::new();
    let recorder = Recorder::new();
    let watching = SettingsWatch::new();
    assert!(
        watching.start(&home.dirs(), recorder.on_change()),
        "the watch started"
    );

    fs::write(home.config().join("profiles.json"), "{}").expect("the scratch profiles file");
    assert_eq!(recorder.wait_for(1), vec![profiles_only()]);

    assert!(watching.stop(LIMIT), "the watcher thread ended");
    assert!(!watching.is_watching());

    fs::write(home.config().join("profiles.json"), "{\"late\":true}")
        .expect("the scratch profiles file");
    assert_eq!(recorder.within_the_quiet_window(), vec![profiles_only()]);
}
