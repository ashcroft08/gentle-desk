//! The watcher of the Settings files (rpc-settings-sync T3b-1, S2): the raw
//! file-system stream of pi's config directories, classified by file name and
//! coalesced into one notice per quiet window.
//!
//! gentle-pi (and the terminal) edit `profiles.json`, its sibling
//! `profile-fallbacks.json` and `auth.json` while the desk runs. The Settings
//! panes could only see those edits on a remount or on window focus, so this
//! module watches the PARENT directories of those three files and reports that
//! they changed; the panes decide what to re-read (T3b-2).
//!
//! pi REPLACES its config files atomically (write a temp file, then rename it
//! over the old one), so the watch is on the directory and NOT on the file: the
//! file the watcher registered may be gone a moment later, while the directory
//! is the one thing that stays. The events are then filtered by file NAME -
//! anything else the directories hold is ignored.
//!
//! Nothing here runs a subprocess and nothing here reads the files: the watcher
//! only says WHICH GROUP changed ([`Group`]). That is what keeps
//! `auth check --no-refresh` from running in the background while the Settings
//! dialog is closed.
//!
//! Degradation (S3's standing rule, applied to this watcher): a directory that
//! does not exist, a platform watcher that cannot be created, or a path that
//! cannot be watched NEVER fails the run and never reaches the user. A run with
//! no watch keeps today's behaviour, the focus refresh. A directory that appears
//! later is picked up by the next start of the app, and nothing polls for one.
//!
//! The rules are pure ([`classify`], [`Debouncer`]) and tested without a file
//! system or a clock; [`SettingsWatch`] is the thin `notify` shell around them,
//! in the same shape as the sessions watcher (`sessions/watch.rs`,
//! `crates/backend/src/session_watch.rs`).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use gentle_runtime::profiles::{FALLBACKS_FILE, PROFILES_FILE};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use super::SettingsPaths;

/// The name gentle-pi keeps its auth store under, in the agent directory.
pub const AUTH_FILE: &str = "auth.json";

/// How long the stream has to be quiet before a batch of events is reported: pi
/// writes a file in several steps (create, write, rename) and the platform
/// reports each one, so a burst is one change to the user.
pub const QUIET: Duration = Duration::from_millis(300);

/// How long the debouncing thread waits when no batch is open. An event or the
/// closing channel wakes it earlier, so this only bounds an idle sleep.
const IDLE_WAIT: Duration = Duration::from_secs(60);

/// One group of Settings files: the files a single pane reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// `profiles.json` and its sibling `profile-fallbacks.json`, both in the
    /// config home: the Profiles pane's files.
    Profiles,
    /// `auth.json` in the agent directory: the Account pane's file.
    Account,
}

/// Which groups a batch of changes touched. This is what crosses the contract
/// (`contract::SettingsFilesChanged`), so it names groups, never paths.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Changed {
    pub profiles: bool,
    pub account: bool,
}

impl Changed {
    /// Whether the batch named at least one group.
    pub fn any(self) -> bool {
        self.profiles || self.account
    }

    fn note(&mut self, group: Group) {
        match group {
            Group::Profiles => self.profiles = true,
            Group::Account => self.account = true,
        }
    }
}

/// The group a file NAME names, `None` for every other name a watched directory
/// holds. Exact names only: `profiles.json.tmp`, the file pi writes before it
/// renames, is not `profiles.json`, and neither is a differently cased one.
pub fn classify(name: &str) -> Option<Group> {
    if name == PROFILES_FILE || name == FALLBACKS_FILE {
        Some(Group::Profiles)
    } else if name == AUTH_FILE {
        Some(Group::Account)
    } else {
        None
    }
}

/// The two directories to watch: the config home that holds the profile files,
/// and the agent directory that holds the auth store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dirs {
    pub config: Option<PathBuf>,
    pub agent: Option<PathBuf>,
}

impl Dirs {
    /// The directories `SettingsPaths` resolves: the folder of `profiles.json`
    /// (its sibling `profile-fallbacks.json` lives there too) and the agent
    /// directory. Both are `None` when no home resolves.
    pub fn of(paths: &SettingsPaths) -> Self {
        Self {
            config: paths
                .profiles
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
            // The agent directory itself is the folder `auth.json` lives in, so
            // it needs no parent step.
            agent: paths
                .agent
                .clone()
                .or_else(|| folder(paths.auth.as_deref())),
        }
    }

    /// The directories to register, each once: a run whose config home IS its
    /// agent directory watches that directory once.
    fn watched(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        for dir in [self.config.as_ref(), self.agent.as_ref()]
            .into_iter()
            .flatten()
        {
            if !dirs.contains(dir) {
                dirs.push(dir.clone());
            }
        }
        dirs
    }
}

/// The folder a file lives in, if the path names one.
fn folder(file: Option<&Path>) -> Option<PathBuf> {
    file.and_then(Path::parent).map(Path::to_path_buf)
}

/// Coalesces the raw events of one or both directories into notices.
///
/// Time is passed in as an offset from any fixed start, so the whole rule is
/// testable without sleeping. The groups are tracked independently: a burst that
/// touches both files names both, and a burst that touches one names only it.
#[derive(Debug)]
pub struct Debouncer {
    quiet: Duration,
    /// When the open batch last received an event; `None` when no batch is open.
    last: Option<Duration>,
    changed: Changed,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(QUIET)
    }
}

impl Debouncer {
    /// `quiet` is a parameter so a test can use a small value.
    pub fn new(quiet: Duration) -> Self {
        Self {
            quiet,
            last: None,
            changed: Changed::default(),
        }
    }

    /// Notes one raw event: the group it names, or `None` for a file this
    /// watcher does not care about (which opens no batch).
    ///
    /// A batch that is already DUE when the event arrives is answered here and
    /// the event opens the NEXT one: the thread that owns the debouncer can be
    /// delayed past a deadline, and an event that arrives after the quiet window
    /// is not part of the old batch - merging it would delay the notice the old
    /// batch already owed until the new event's own quiet window.
    pub fn push(&mut self, now: Duration, group: Option<Group>) -> Option<Changed> {
        // The due check comes FIRST, from the instant this event arrived, and it
        // does not depend on the event naming a group: the notice the open batch
        // already owes goes out with this push even when the event is noise.
        let flushed = self.take_if_due(now);
        // Only an event that names a group opens (or extends) a batch. A `now`
        // earlier than the previous push only widens the window.
        if let Some(group) = group {
            self.last = Some(self.last.map_or(now, |last| last.max(now)));
            self.changed.note(group);
        }
        flushed
    }

    /// The instant the open batch is due, if one is open.
    pub fn deadline(&self) -> Option<Duration> {
        self.last.map(|last| last + self.quiet)
    }

    /// The batch if `now` has reached its deadline; nothing is cleared otherwise.
    pub fn take_if_due(&mut self, now: Duration) -> Option<Changed> {
        if self.deadline()? > now {
            return None;
        }
        self.last = None;
        Some(std::mem::take(&mut self.changed))
    }
}

/// What the watcher does with one debounced batch: push the notice to the web
/// side. Called on the watcher's own thread, never on the window's.
pub type OnChange = Arc<dyn Fn(Changed) + Send + Sync>;

/// Why a watch could not start. Every case is one the run survives: no watch
/// means the panes keep the focus refresh, so this never reaches the user.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WatchError {
    /// Not one of the run's directories could be watched: none exists yet, or
    /// the platform refused every one of them.
    Unwatched,
    /// The platform watcher could not be created at all.
    Create(String),
    /// The thread that debounces the events could not be started.
    Thread(String),
}

/// One running watch: the platform watch, which stops when it is dropped, and
/// the thread that debounces its events.
struct SettingsWatcher {
    watcher: Option<RecommendedWatcher>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for SettingsWatcher {
    fn drop(&mut self) {
        // Dropping the platform watcher drops its callback, which closes the raw
        // channel; the thread then sees it disconnected and returns.
        drop(self.watcher.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts watching every directory of `dirs` that can be watched, and the thread
/// that turns its events into notices.
///
/// A directory that is not there is skipped rather than failed: pi may not have
/// created its config home yet. When NOT ONE directory could be registered there
/// is nothing to watch, and the caller keeps today's behaviour.
fn start(dirs: &Dirs, on_change: OnChange) -> Result<SettingsWatcher, WatchError> {
    let (raw_tx, raw_rx) = mpsc::channel::<notify::Result<notify::Event>>();
    // The platform callback only forwards: classifying and debouncing run on the
    // thread below, so the callback stays as short as possible.
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = raw_tx.send(event);
    })
    .map_err(|error| WatchError::Create(error.to_string()))?;
    let mut watching = 0;
    for dir in dirs.watched() {
        // Non-recursive: the three files sit directly in these two directories,
        // and a recursive watch would report every chat of the home as well.
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_ok() {
            watching += 1;
        }
    }
    if watching == 0 {
        return Err(WatchError::Unwatched);
    }
    let thread = std::thread::Builder::new()
        .name("settings-watch".to_owned())
        .spawn(move || debounce_events(&raw_rx, &on_change))
        .map_err(|error| WatchError::Thread(error.to_string()))?;
    Ok(SettingsWatcher {
        watcher: Some(watcher),
        thread: Some(thread),
    })
}

/// The one thread that owns the [`Debouncer`] of a watch: it reads the raw
/// stream, keeps the group each event names, and calls `on_change` once per
/// quiet window.
fn debounce_events(raw: &Receiver<notify::Result<notify::Event>>, on_change: &OnChange) {
    let started = Instant::now();
    let mut debouncer = Debouncer::default();
    loop {
        let wait = debouncer.deadline().map_or(IDLE_WAIT, |deadline| {
            deadline.saturating_sub(started.elapsed())
        });
        match raw.recv_timeout(wait) {
            Ok(Ok(event)) => {
                for path in &event.paths {
                    let name = path.file_name().and_then(|name| name.to_str());
                    // The push answers a batch that was already due before this
                    // event: that notice goes out NOW, and this event opens the
                    // next batch.
                    if let Some(changed) =
                        debouncer.push(started.elapsed(), name.and_then(classify))
                    {
                        on_change(changed);
                    }
                }
            }
            // A platform error names no file, so it names no group either: this
            // watcher cannot guess which pane to wake, and a full re-read of both
            // is what the focus refresh already does.
            Ok(Err(_)) => {}
            Err(RecvTimeoutError::Timeout) => {}
            // The watch was dropped: nothing more will arrive.
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if let Some(changed) = debouncer.take_if_due(started.elapsed()) {
            on_change(changed);
        }
    }
}

/// How long a second [`SettingsWatch::start`] waits for the watch it replaces
/// before it goes on with the new one. The same shape as the close's stop: on a
/// thread of its own, waited for at most this, so no start can hang on a watch
/// someone else wedged.
const REPLACE_WAIT: Duration = Duration::from_secs(1);

/// The watcher of this run, as the bridge holds it and the close stops it.
///
/// The app starts it ONCE, at startup, and the close stops it once: nothing
/// retries, and nothing polls for a directory that appears later. A second
/// [`SettingsWatch::start`] replaces the running watch with the one it is given
/// (documented there); a run whose directories were not there at startup keeps
/// the focus refresh until the next start of the app.
#[derive(Default)]
pub struct SettingsWatch {
    /// The watch of this run; taken out by the stop under this lock and dropped
    /// outside it, because its drop joins a thread.
    running: Mutex<Option<SettingsWatcher>>,
}

impl SettingsWatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the watcher on `dirs`, with `on_change` as its push. Answers
    /// whether one runs: a run that could not watch anything answers `false`
    /// and is otherwise identical.
    ///
    /// A watch that is already running is REPLACED, not kept: the `dirs` and the
    /// `on_change` given here are the ones that take effect, and the old watch
    /// stops reporting. The old one is stopped the way the close stops it - on a
    /// thread of its own, waited for at most a fixed short wait - so a start
    /// never hangs on it: a wedged watch is abandoned, and this start goes on
    /// without it. The stop and the start both run outside the lock, because the
    /// stop joins a thread and the start creates a platform watch and a thread.
    pub fn start(&self, dirs: &Dirs, on_change: OnChange) -> bool {
        self.stop(REPLACE_WAIT);
        match start(dirs, on_change) {
            Ok(watcher) => {
                *lock(&self.running) = Some(watcher);
                true
            }
            // No watch is no failure: the panes keep the focus refresh, and
            // nothing user-visible is said about it.
            Err(_) => false,
        }
    }

    /// Whether a watch runs right now.
    pub fn is_watching(&self) -> bool {
        lock(&self.running).is_some()
    }

    /// Stops the watcher and waits at most `budget` for its thread, answering
    /// whether it ended. The drop of the platform watch closes the raw channel,
    /// so the thread returns at its next look; it runs on a thread of its own so
    /// that a wedged disk can never hold the close past the step's deadline.
    pub fn stop(&self, budget: Duration) -> bool {
        let Some(watcher) = lock(&self.running).take() else {
            return true;
        };
        let (done, wait) = mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("settings-watch-stop".to_owned())
            .spawn(move || {
                drop(watcher);
                let _ = done.send(());
            });
        match spawned {
            Err(error) => {
                eprintln!("settings watch stop unavailable: {error}");
                false
            }
            Ok(_) => wait.recv_timeout(budget).is_ok(),
        }
    }
}

/// A lock that survives a poisoned mutex: a thread that died must not leave the
/// watcher unreadable for the rest of the run.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// The file names of the two panes map to their groups, and everything else
    /// - a temp file pi writes before a rename included - maps to nothing.
    #[test]
    fn only_the_three_file_names_name_a_group() {
        assert_eq!(classify("profiles.json"), Some(Group::Profiles));
        assert_eq!(classify("profile-fallbacks.json"), Some(Group::Profiles));
        assert_eq!(classify("auth.json"), Some(Group::Account));
        for other in [
            "profiles.json.tmp",
            "auth.json.tmp",
            "profile-fallbacks.json~",
            "PROFILES.JSON",
            "Auth.json",
            "profiles.json.bak",
            "settings.json",
            "profiles",
            "",
        ] {
            assert_eq!(classify(other), None, "{other} named a group");
        }
    }

    /// A burst of events for one file is ONE notice, and it is due only after
    /// the quiet window that follows the LAST of them.
    #[test]
    fn a_burst_becomes_one_notice_when_it_goes_quiet() {
        let mut debouncer = Debouncer::default();
        for millis in [0, 40, 90] {
            debouncer.push(at(millis), Some(Group::Profiles));
        }
        assert_eq!(debouncer.deadline(), Some(at(90) + QUIET));
        assert_eq!(debouncer.take_if_due(at(90) + QUIET - at(1)), None);
        assert_eq!(
            debouncer.take_if_due(at(90) + QUIET),
            Some(Changed {
                profiles: true,
                account: false,
            })
        );
        assert_eq!(debouncer.deadline(), None, "the batch stays open");
        assert_eq!(debouncer.take_if_due(at(5_000)), None);
    }

    /// An event that names no group (a temp file, any other name in a watched
    /// folder) still reads the clock: when the open batch is already due, the
    /// notice it owes goes out with that push instead of waiting for the loop's
    /// next turn.
    #[test]
    fn an_ignored_event_after_the_deadline_still_flushes_the_due_batch() {
        let mut debouncer = Debouncer::default();
        assert_eq!(debouncer.push(at(0), Some(Group::Profiles)), None);
        let deadline = debouncer.deadline().expect("the batch is open");

        let late = deadline + at(500);
        assert_eq!(
            debouncer.push(late, None),
            Some(Changed {
                profiles: true,
                account: false,
            }),
            "an ignored event past the deadline left the due batch waiting"
        );
        assert_eq!(debouncer.deadline(), None, "no batch is left open");
    }

    /// An ignored event before the deadline changes nothing: it neither flushes
    /// the batch nor moves its quiet window, so noise in the folder can never
    /// delay a real change's notice.
    #[test]
    fn an_ignored_event_before_the_deadline_leaves_the_batch_alone() {
        let mut debouncer = Debouncer::default();
        assert_eq!(debouncer.push(at(0), Some(Group::Profiles)), None);
        let deadline = debouncer.deadline().expect("the batch is open");

        assert_eq!(debouncer.push(at(100), None), None);
        assert_eq!(
            debouncer.deadline(),
            Some(deadline),
            "an ignored event moved the quiet window"
        );
        assert_eq!(
            debouncer.take_if_due(deadline),
            Some(Changed {
                profiles: true,
                account: false,
            })
        );
    }

    /// An ignored event with no batch open has nothing to flush and opens none.
    #[test]
    fn an_ignored_event_with_no_batch_open_does_nothing() {
        let mut debouncer = Debouncer::default();
        assert_eq!(debouncer.push(at(1_000), None), None);
        assert_eq!(debouncer.deadline(), None);
    }

    /// Two bursts with the quiet window between them are two notices, not one.
    #[test]
    fn two_bursts_separated_by_the_quiet_window_are_two_notices() {
        let mut debouncer = Debouncer::default();
        assert_eq!(debouncer.push(at(0), Some(Group::Account)), None);
        assert!(debouncer.take_if_due(at(0) + QUIET).is_some());
        assert_eq!(debouncer.push(at(1_000), Some(Group::Account)), None);
        assert!(debouncer.take_if_due(at(1_000) + QUIET).is_some());
    }

    /// An event that arrives AFTER the open batch's deadline is the next batch's:
    /// the notice the old batch already owed goes out first, and the new event
    /// opens a batch of its own. The thread that owns the debouncer can be
    /// delayed past a deadline, and merging then would delay that notice until
    /// the new event's quiet window.
    #[test]
    fn an_event_after_the_deadline_flushes_the_old_batch_first() {
        let mut debouncer = Debouncer::default();
        assert_eq!(debouncer.push(at(0), Some(Group::Profiles)), None);
        let deadline = debouncer.deadline().expect("the batch is open");

        // Half a second past the deadline the account file changes: the profiles
        // notice is already owed and is answered with this very push.
        let late = deadline + at(500);
        assert_eq!(
            debouncer.push(late, Some(Group::Account)),
            Some(Changed {
                profiles: true,
                account: false,
            }),
            "an event past the deadline merged into the batch that was already due"
        );
        assert_eq!(
            debouncer.deadline(),
            Some(late + QUIET),
            "the late event did not open a batch of its own"
        );
        assert_eq!(
            debouncer.take_if_due(late + QUIET),
            Some(Changed {
                profiles: false,
                account: true,
            })
        );
    }

    /// The groups are tracked independently: one file's burst names its own
    /// group only, a burst that touches both names both, and a group already
    /// seen in the open batch never turns back off.
    #[test]
    fn the_groups_of_one_batch_are_tracked_independently() {
        let mut debouncer = Debouncer::default();
        debouncer.push(at(0), Some(Group::Profiles));
        assert_eq!(
            debouncer.take_if_due(QUIET),
            Some(Changed {
                profiles: true,
                account: false,
            })
        );

        debouncer.push(at(500), Some(Group::Profiles));
        debouncer.push(at(510), Some(Group::Account));
        debouncer.push(at(520), Some(Group::Profiles));
        let mixed = debouncer.take_if_due(at(520) + QUIET);
        assert_eq!(
            mixed,
            Some(Changed {
                profiles: true,
                account: true,
            })
        );
        assert!(mixed.expect("the mixed batch").any());
    }

    /// An event for a file this watcher ignores opens no batch at all, so it can
    /// never wake the panes.
    #[test]
    fn an_ignored_file_opens_no_batch() {
        let mut debouncer = Debouncer::default();
        debouncer.push(at(0), None);
        assert_eq!(debouncer.deadline(), None);
        assert_eq!(debouncer.take_if_due(at(10_000)), None);
        assert!(!Changed::default().any(), "an empty batch names a group");
    }

    /// The directories come from the same resolution the panes read through: the
    /// folder of `profiles.json` for the profiles, the agent directory for the
    /// auth store, and no watch at all for a run whose home did not resolve.
    #[test]
    fn the_directories_are_the_folders_the_panes_read() {
        let paths = SettingsPaths {
            profiles: Some(PathBuf::from("C:/cfg/profiles.json")),
            fallbacks: Some(PathBuf::from("C:/cfg/profile-fallbacks.json")),
            auth: Some(PathBuf::from("C:/agent/auth.json")),
            agent: Some(PathBuf::from("C:/agent")),
            sessions: Some(PathBuf::from("C:/agent/sessions")),
            home_args: Vec::new(),
        };

        assert_eq!(
            Dirs::of(&paths),
            Dirs {
                config: Some(PathBuf::from("C:/cfg")),
                agent: Some(PathBuf::from("C:/agent")),
            }
        );
        assert_eq!(Dirs::of(&SettingsPaths::default()), Dirs::default());
    }

    /// An auth path with no agent directory named beside it still resolves to its
    /// folder: the store is the only thing a run without one knows about.
    #[test]
    fn the_auth_store_names_its_own_folder() {
        let paths = SettingsPaths {
            auth: Some(PathBuf::from("C:/agent/auth.json")),
            ..SettingsPaths::default()
        };
        assert_eq!(
            Dirs::of(&paths),
            Dirs {
                config: None,
                agent: Some(PathBuf::from("C:/agent")),
            }
        );
    }

    /// A config home that is also the agent directory is registered once: the
    /// platform watches a directory, not a pair of names.
    #[test]
    fn one_directory_is_registered_once() {
        let both = Dirs {
            config: Some(PathBuf::from("C:/one")),
            agent: Some(PathBuf::from("C:/one")),
        };
        assert_eq!(both.watched(), vec![PathBuf::from("C:/one")]);

        let split = Dirs {
            config: Some(PathBuf::from("C:/cfg")),
            agent: Some(PathBuf::from("C:/agent")),
        };
        assert_eq!(
            split.watched(),
            vec![PathBuf::from("C:/cfg"), PathBuf::from("C:/agent")]
        );
        assert_eq!(Dirs::default().watched(), Vec::<PathBuf>::new());
    }
}
