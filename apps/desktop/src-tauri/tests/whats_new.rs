//! Windowless tests of the what's-new step of a run (what's-new S1, S3, S5, S6):
//! the version the last run stored, the one fetch of `latest.json` through the
//! injectable fetcher, the notes pushed through the same emit step the other
//! slices use, and the version stored - on the dismiss, or at once when there is
//! nothing to show.
//!
//! No test reaches the network: the fetcher is a closure of the test, so the
//! endpoint it is asked for, how often, and what it answers are all pinned here.
//! Every test sets its own `APPDATA`, `LOCALAPPDATA`, `PI_CODING_AGENT_DIR` and
//! `GENTLE_SHELL_HOME` to a temp directory, so the real config and the user's own
//! pi are never read or written; the environment is process-global, so only one
//! test of this file runs at a time.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::commands::{Bridge, Emit, Fetcher};
use gentle_desk_lib::contract::{ViewState, WhatsNewView};
use gentle_runtime::app_config::{self, AppConfig};
use gentle_runtime::whats_new::Latest;

/// How long a test waits for a push or a stored file, so a hang fails instead.
const LIMIT: Duration = Duration::from_secs(10);

/// The endpoint the updater config names in the running app.
const ENDPOINT: &str =
    "https://github.com/ashcroft08/gentle-desk/releases/latest/download/latest.json";

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own app config, local app data, pi folder and
/// gentle-shell home, all under a temp directory outside the worktree.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
}

impl Harness {
    /// A world whose config file stores `last_run_version` as `stored`, `None`
    /// writing the document every release before this feature wrote.
    fn new(stored: Option<&str>) -> Self {
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        for dir in [
            root.join("appdata"),
            root.join("localappdata"),
            root.join("home"),
            root.join("pi"),
        ] {
            std::fs::create_dir_all(&dir).expect("the scratch directory is creatable");
        }
        std::env::set_var("APPDATA", root.join("appdata"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("PI_CODING_AGENT_DIR", root.join("pi"));
        let harness = Self {
            _serial: serial,
            root,
        };
        let mut config = AppConfig::default();
        if let Some(version) = stored {
            assert!(config.set_last_run_version(version));
        }
        app_config::save(&harness.config_file(), &config).expect("the config file is writable");
        harness
    }

    fn config_file(&self) -> PathBuf {
        app_config::config_path(Some(self.root.join("appdata").as_os_str()))
            .expect("the temp app data is a config location")
    }

    /// The version the config file holds right now.
    fn stored(&self) -> Option<String> {
        app_config::load(&self.config_file())
            .config
            .last_run_version
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in [
            "APPDATA",
            "LOCALAPPDATA",
            "GENTLE_SHELL_HOME",
            "PI_CODING_AGENT_DIR",
        ] {
            std::env::remove_var(name);
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A fresh directory of the system tempdir, never inside the worktree.
fn scratch_root() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "gentle-desk-whats-new-{}-{stamp}",
        std::process::id()
    ))
}

/// The emit seam of a test: every slice the bridge pushes, in order.
fn recorder() -> (Emit, Arc<Mutex<Vec<ViewState>>>) {
    let views = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&views);
    let emit: Emit = Arc::new(move |view: &ViewState| {
        lock(&kept).push(view.clone());
        Ok(())
    });
    (emit, views)
}

/// The real bridge of one run, with a recorder where the window would be.
fn app_run() -> (Bridge, Arc<Mutex<Vec<ViewState>>>) {
    let (emit, views) = recorder();
    (Bridge::new().with_emit(emit), views)
}

/// A lock that survives a poisoned mutex: one failing test must not fail the rest.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A fetcher that answers `manifest` and records the URLs it was asked for.
fn fetcher(manifest: Latest) -> (Fetcher, Arc<Mutex<Vec<String>>>) {
    let asked = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&asked);
    let fetch: Fetcher = Arc::new(move |url: &str| {
        lock(&seen).push(url.to_owned());
        Ok(manifest.clone())
    });
    (fetch, asked)
}

/// A fetcher that fails the way a run with no network does.
fn failing_fetcher(reason: &str) -> (Fetcher, Arc<Mutex<Vec<String>>>) {
    let asked = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&asked);
    let message = reason.to_owned();
    let fetch: Fetcher = Arc::new(move |url: &str| {
        lock(&seen).push(url.to_owned());
        Err(message.clone())
    });
    (fetch, asked)
}

/// The manifest of a release that carries `notes`.
fn latest(version: &str, notes: &str) -> Latest {
    Latest {
        version: version.to_owned(),
        notes: notes.to_owned(),
    }
}

/// Waits for the push that carries `notes`, failing instead of hanging.
fn wait_for_notes(views: &Arc<Mutex<Vec<ViewState>>>, notes: &str) -> ViewState {
    let deadline = Instant::now() + LIMIT;
    loop {
        let found = lock(views)
            .iter()
            .rev()
            .find(|view| {
                view.whats_new
                    .as_ref()
                    .is_some_and(|whats_new| whats_new.notes == notes)
            })
            .cloned();
        if let Some(view) = found {
            return view;
        }
        assert!(
            Instant::now() < deadline,
            "the notes never reached the web side: {} pushes",
            lock(views).len()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Waits until the config file holds `version`, failing instead of hanging: the
/// store travels through the config writer's own thread.
fn wait_for_stored(harness: &Harness, version: &str) {
    let deadline = Instant::now() + LIMIT;
    loop {
        let stored = harness.stored();
        if stored.as_deref() == Some(version) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the config file never held {version}: {stored:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn an_update_shows_the_notes_of_this_version_and_stores_nothing_yet() {
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));

    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    let view = wait_for_notes(&views, "The sidebar folds.");
    assert_eq!(
        view.whats_new,
        Some(WhatsNewView {
            version: "0.1.8".to_owned(),
            notes: "The sidebar folds.".to_owned(),
        }),
        "the slice carries the version and the plain-text notes"
    );
    assert_eq!(
        lock(&asked).as_slice(),
        [ENDPOINT],
        "latest.json is fetched exactly once, from the endpoint the updater config names"
    );
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.7"),
        "nothing is stored while the modal is still up"
    );
}

#[test]
fn the_dismiss_stores_this_runs_version_and_the_next_run_follows_no_update() {
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, views) = app_run();
    let (fetch, _asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);
    wait_for_notes(&views, "The sidebar folds.");

    let view = bridge
        .store_last_run_version()
        .expect("a run that named its version stores it");
    assert_eq!(view.whats_new, None, "the dismiss clears the notes");
    wait_for_stored(&harness, "0.1.8");
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "the stored version is this run's own"
    );

    // The next run of the same version: no update, so no fetch at all.
    let (second, second_views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    second.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    assert!(
        lock(&second_views).is_empty(),
        "a run of the stored version pushes nothing"
    );
    assert!(
        lock(&asked).is_empty(),
        "a run of the stored version fetches nothing"
    );
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "no run stores again"
    );
}

#[test]
fn a_fetch_that_failed_stores_the_version_and_pushes_nothing() {
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, views) = app_run();
    let (fetch, asked) = failing_fetcher("no network");

    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    wait_for_stored(&harness, "0.1.8");
    assert_eq!(
        lock(&asked).as_slice(),
        [ENDPOINT],
        "the fetch was tried once"
    );
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "a failed fetch shows nothing"
    );
}

#[test]
fn a_manifest_of_another_version_stores_this_runs_version_and_pushes_nothing() {
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.9", "Notes of a release this run is not"));

    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    wait_for_stored(&harness, "0.1.8");
    assert_eq!(lock(&asked).len(), 1, "the fetch was tried once");
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "a manifest of another version shows nothing"
    );
}

#[test]
fn a_first_run_shows_nothing_stores_nothing_and_fetches_nothing() {
    // The config every build before this feature wrote: no version stored.
    let harness = Harness::new(None);
    let (bridge, views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));

    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    assert!(
        lock(&asked).is_empty(),
        "a first run is not an update, so nothing is fetched"
    );
    assert!(lock(&views).is_empty(), "a first run shows nothing");
    assert_eq!(harness.stored(), None, "a first run stores no version");
}

#[test]
fn a_run_whose_config_names_no_endpoint_stores_the_version_and_pushes_nothing() {
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));

    bridge.start_whats_new("0.1.8", None, fetch);

    wait_for_stored(&harness, "0.1.8");
    assert!(
        lock(&asked).is_empty(),
        "there is no endpoint to ask, so nothing is fetched"
    );
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "no endpoint is no notes to show"
    );
}

/// The config file the harness writes is the document the app reads: this keeps
/// the two independent of each other, so the tests above never pass by accident.
#[test]
fn the_harness_writes_the_config_the_app_reads() {
    let harness = Harness::new(Some("0.1.7"));
    let loaded = app_config::load(&harness.config_file());
    assert_eq!(loaded.notice, None);
    assert_eq!(loaded.config.last_run_version, Some("0.1.7".to_owned()));
    assert!(
        Path::new(&harness.config_file()).exists(),
        "the config file is where the run looks for it"
    );
}
