//! Windowless tests of the what's-new step of a run (what's-new S1, S3, S5, S6):
//! the version the last run stored, the one fetch of `latest.json` through the
//! injectable fetcher, the notes pushed through the same emit step the other
//! slices use, and the version stored - on the dismiss, or at once when there is
//! nothing to show. The version this run RUNS as travels with every slice too
//! (settings-dialog S5), because the About section draws it.
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

use gentle_core::i18n::{messages, Language};
use gentle_desk_lib::commands::{read_whats_new, Bridge, Emit, Fetcher};
use gentle_desk_lib::contract::{ViewState, WhatsNewRead, WhatsNewView};
use gentle_runtime::app_config::{self, AppConfig};
use gentle_runtime::whats_new::{Latest, UpdateRun};

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
        Self::saying(stored, None)
    }

    /// The same world, with `language` as the app's own words' preference: the
    /// line a read on demand answers is worded where it is produced, so a test
    /// of that line says which language it asked for.
    fn saying(stored: Option<&str>, language: Option<Language>) -> Self {
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
        let mut config = AppConfig {
            language,
            ..AppConfig::default()
        };
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
    let (bridge, _emit, views) = app_run_seamed();
    (bridge, views)
}

/// The same run with its emit seam in hand, which a read on demand pushes
/// through exactly as the app's own step does.
fn app_run_seamed() -> (Bridge, Emit, Arc<Mutex<Vec<ViewState>>>) {
    let (emit, views) = recorder();
    (Bridge::new().with_emit(Arc::clone(&emit)), emit, views)
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
            // The modal says "since 0.1.7": the version the config stored is the
            // one this run followed, and it reaches the slice with the notes.
            previous_version: Some("0.1.7".to_owned()),
            notes: "The sidebar folds.".to_owned(),
        }),
        "the slice carries the version, the version it follows and the notes"
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
fn a_first_run_stores_its_version_and_fetches_nothing() {
    // The config every build before this feature wrote: no version stored.
    // (R3-001) The first run records the version it runs as, so the NEXT
    // update is detected; it still shows no modal and fetches nothing.
    let harness = Harness::new(None);
    let (bridge, views) = app_run();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));

    bridge.start_whats_new("0.1.8", Some(ENDPOINT), fetch);

    assert!(
        lock(&asked).is_empty(),
        "a first run is not an update, so nothing is fetched"
    );
    let view = lock(&views)
        .first()
        .expect("the store pushes its slice")
        .clone();
    assert_eq!(
        view.whats_new, None,
        "a first run shows no modal; the store only clears the slice"
    );
    wait_for_stored(&harness, "0.1.8");
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "the first run stores the version it runs as"
    );
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

/// The version this run runs as reaches the web side on every slice, which is
/// what the Settings dialog's About section draws (settings-dialog S5). A run
/// that never named its version carries none, so the section never guesses one.
#[test]
fn the_version_this_run_runs_as_reaches_every_slice() {
    let _harness = Harness::new(None);
    let (bridge, views) = app_run();
    assert_eq!(
        bridge.holder.view().app_version,
        None,
        "a run that never named its version carries none"
    );
    let (fetch, _asked) = fetcher(latest("0.1.9", "Unread notes."));

    // A first run pushes one slice as it stores the version, so the push below
    // is the same emit step every other slice travels on.
    bridge.start_whats_new("0.1.9", Some(ENDPOINT), fetch);

    let view = lock(&views)
        .first()
        .expect("the first run pushes its slice")
        .clone();
    assert_eq!(
        view.app_version.as_deref(),
        Some("0.1.9"),
        "the pushed slice carries the version this run runs as"
    );
    assert_eq!(
        bridge.holder.view().app_version.as_deref(),
        Some("0.1.9"),
        "and so does the first read of a web view"
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

// --- The read on demand (T5, S9, D6) ---------------------------------------
//
// The About pane's "What is new in this version": one read of the RUNNING
// version's notes, asked by the user, through the same fetcher seam and the same
// address the startup step uses. It is not an update run, so nothing it does -
// including its dismissal - writes the version the last run was, and the startup
// step keeps detecting exactly what it detected before.

#[test]
fn a_read_on_demand_shows_the_notes_of_this_version() {
    let harness = Harness::saying(Some("0.1.8"), Some(Language::Es));
    let (bridge, emit, views) = app_run_seamed();
    // The run of the stored version: no update, so no startup fetch and no modal.
    let (startup, startup_asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);
    assert!(
        lock(&startup_asked).is_empty(),
        "a run of the stored version asks the release server nothing"
    );
    assert!(lock(&views).is_empty(), "and pushes nothing");

    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    let read = read_whats_new(&bridge.holder, &emit, Some(ENDPOINT), fetch.as_ref())
        .expect("the read answers the page");

    assert_eq!(read, WhatsNewRead::Shown);
    let view = lock(&views)
        .last()
        .cloned()
        .expect("the notes reach the web side through the same push as every slice");
    assert_eq!(
        view.whats_new,
        Some(WhatsNewView {
            version: "0.1.8".to_owned(),
            // A read on demand follows nothing, so the modal draws no "since".
            previous_version: None,
            notes: "The sidebar folds.".to_owned(),
        }),
        "the running version's notes, and only those"
    );
    assert_eq!(
        lock(&asked).as_slice(),
        [ENDPOINT],
        "one fetch, from the endpoint the updater config names"
    );
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "a read on demand writes no version"
    );
}

#[test]
fn a_read_on_demand_never_shows_another_versions_notes() {
    // Running 0.1.8 while the manifest announces 0.1.9: those notes are the NEXT
    // release's, and the news of a version this run is not is never drawn as if
    // it were this one's.
    let harness = Harness::saying(Some("0.1.8"), Some(Language::Es));
    let (bridge, emit, views) = app_run_seamed();
    let (startup, _asked) = fetcher(latest("0.1.8", "Unread notes."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);

    let (fetch, asked) = fetcher(latest("0.1.9", "What the release after this one brings."));
    let read = read_whats_new(&bridge.holder, &emit, Some(ENDPOINT), fetch.as_ref())
        .expect("the read answers the page");

    assert_eq!(
        read,
        WhatsNewRead::Unavailable(messages::whats_new_another_version(Language::Es).to_owned()),
        "the page is told there are no notes for this version, in its own language"
    );
    assert_eq!(lock(&asked).len(), 1, "the manifest is asked once");
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "nothing reaches the modal"
    );
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "a read on demand writes no version"
    );
}

#[test]
fn a_read_on_demand_of_a_release_with_no_text_has_nothing_to_show() {
    let _harness = Harness::saying(Some("0.1.8"), Some(Language::Es));
    let (bridge, emit, views) = app_run_seamed();
    let (startup, _asked) = fetcher(latest("0.1.8", "Unread notes."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);

    let (fetch, _asked) = fetcher(latest("0.1.8", "   "));
    let read = read_whats_new(&bridge.holder, &emit, Some(ENDPOINT), fetch.as_ref())
        .expect("the read answers the page");

    assert_eq!(
        read,
        WhatsNewRead::Unavailable(messages::whats_new_no_text(Language::Es).to_owned()),
        "a release body of whitespace is no news, not an empty modal"
    );
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "nothing reaches the modal"
    );
}

#[test]
fn a_read_on_demand_with_nothing_to_ask_answers_the_short_line() {
    // A run whose address is not configured, and a run that never named its
    // version: neither has anything to fetch, and neither pushes a slice.
    let _harness = Harness::saying(Some("0.1.8"), None);
    let (bridge, emit, views) = app_run_seamed();
    let (startup, _asked) = fetcher(latest("0.1.8", "Unread notes."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);

    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    let read = read_whats_new(&bridge.holder, &emit, None, fetch.as_ref())
        .expect("the read answers the page");
    assert_eq!(
        read,
        WhatsNewRead::Unavailable(messages::whats_new_unreadable(Language::En).to_owned()),
        "no address is a read that could not happen"
    );
    assert!(
        lock(&asked).is_empty(),
        "there is no endpoint, so nothing is fetched"
    );
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "the run with no address shows nothing"
    );

    let (unnamed, unnamed_emit, unnamed_views) = app_run_seamed();
    let (fetch, asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    let read = read_whats_new(
        &unnamed.holder,
        &unnamed_emit,
        Some(ENDPOINT),
        fetch.as_ref(),
    )
    .expect("the read answers the page");
    assert_eq!(
        read,
        WhatsNewRead::Unavailable(messages::whats_new_unreadable(Language::En).to_owned()),
        "a run that never named its version has none to read notes for"
    );
    assert!(
        lock(&asked).is_empty(),
        "nothing is fetched for a version the run does not know"
    );
    assert!(lock(&unnamed_views).is_empty(), "and nothing is pushed");
}

#[test]
fn a_manual_read_is_not_an_update_run_and_its_dismiss_stores_nothing() {
    // The reachable shape of the danger: this run follows an update (the config
    // stores 0.1.7, the run is 0.1.8) and the notes of the update are up. The
    // About pane's read replaces them, and ITS dismissal must store nothing -
    // otherwise the modal would stop coming back for a version nobody dismissed.
    let harness = Harness::new(Some("0.1.7"));
    let (bridge, emit, views) = app_run_seamed();
    let (startup, _asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);
    wait_for_notes(&views, "The sidebar folds.");

    let (fetch, _asked) = fetcher(latest("0.1.8", "Read when the user asked."));
    let read = read_whats_new(&bridge.holder, &emit, Some(ENDPOINT), fetch.as_ref())
        .expect("the read answers the page");
    assert_eq!(read, WhatsNewRead::Shown);
    let view = wait_for_notes(&views, "Read when the user asked.");
    assert_eq!(
        view.whats_new.map(|notes| notes.previous_version),
        Some(None),
        "a read on demand follows nothing, so no \"since\" line is drawn"
    );

    let dismissed = bridge
        .store_last_run_version()
        .expect("the dismiss answers the page");
    assert_eq!(dismissed.whats_new, None, "the dismiss clears the notes");
    assert_eq!(
        bridge.holder.update_run(),
        UpdateRun::Updated,
        "this run still follows the version the config stored: nothing was written"
    );
    // The store, when one happens, travels through the config writer's thread;
    // this waits long enough for a store that should NOT happen to show up.
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.7"),
        "a read on demand is not an update run: its dismissal stores no version"
    );

    // And the startup step is what it was: the next run of the same version
    // still follows an update and still shows the notes.
    let (second, second_views) = app_run();
    let (fetch, _asked) = fetcher(latest("0.1.8", "The sidebar folds."));
    second.start_whats_new("0.1.8", Some(ENDPOINT), fetch);
    wait_for_notes(&second_views, "The sidebar folds.");
}

#[test]
fn a_read_on_demand_that_could_not_reach_the_release_server_says_so() {
    let harness = Harness::saying(Some("0.1.8"), Some(Language::Es));
    let (bridge, emit, views) = app_run_seamed();
    let (startup, _asked) = fetcher(latest("0.1.8", "Unread notes."));
    bridge.start_whats_new("0.1.8", Some(ENDPOINT), startup);

    let (fetch, asked) = failing_fetcher("no network");
    let read = read_whats_new(&bridge.holder, &emit, Some(ENDPOINT), fetch.as_ref())
        .expect("the read answers the page");

    assert_eq!(
        read,
        WhatsNewRead::Unavailable(messages::whats_new_unreadable(Language::Es).to_owned()),
        "offline is one quiet line, never an empty modal"
    );
    assert_eq!(lock(&asked).len(), 1, "the fetch was tried once");
    assert!(
        lock(&views).iter().all(|view| view.whats_new.is_none()),
        "nothing reaches the modal"
    );
    assert_eq!(
        harness.stored().as_deref(),
        Some("0.1.8"),
        "a read on demand writes no version"
    );
}
