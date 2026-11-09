//! The bridge that owns the app state (S2 of the migration).
//!
//! [`AppStateHolder`] holds one `gentle_app::AppState` behind a mutex,
//! created at startup and managed by Tauri. Every intent the web sends is
//! folded through the pure reducer in `crates/app`, the same `update_chats`
//! the iced shell folds, and the answer to the web is a [`ViewState`] slice,
//! carried by both the command return and the `state-changed` event.
//!
//! The mapping is explicit: the contract's [`UiIntent`] turns into
//! `gentle_app::Intent` by hand, and the state turns into [`ViewState`] by
//! [`view_state`]. Nothing derives on a `crates/app` type, so the
//! two sides stay independently defined and generator-checked
//! (`tests/contract_bindings.rs`).
//!
//! The holder only answers what the reducer asked for. The per-chat commands
//! are run by `effects` and the command layer; the SHARED commands (a settled
//! turn's `Command::RefreshSessions`) go to the [`CommandSink`] installed by the
//! bridge, which asks the sessions service for that pass. Both kinds run off the
//! window's thread.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{Command, SessionItem};
use gentle_runtime::app_config::AppConfig;
use gentle_runtime::clock::now_ms;

use crate::config::ConfigSnapshot;
use crate::contract::{SessionsWindow, UiIntent, ViewState};
use crate::sessions::cache::TreeCache;
use crate::sessions::revision;
use crate::sessions::tree::Tree;
use crate::sessions::window::{self, folder_status};
use crate::state_view::view_state;

/// What runs one shared command the reducer asked for. `Command::RefreshSessions`
/// is the session list's in the app; the sink keeps the holder free of how it is
/// run (a thread, a service, nothing at all before it is installed).
pub type CommandSink = Arc<dyn Fn(Command) + Send + Sync>;

/// The app state the Tauri shell owns. Created at startup, managed by Tauri;
/// nothing else anywhere holds an `AppState`.
#[derive(Default)]
pub struct AppStateHolder {
    app: Mutex<gentle_app::AppState>,
    /// Installed once at startup: before it exists a shared command is simply
    /// dropped, which is what the earlier slices did with all of them.
    commands: Mutex<Option<CommandSink>>,
    /// Bumped whenever the folded list, its status or the config changes, and
    /// carried by every slice as `ViewState::sessions_revision`. It wraps: the
    /// web side compares two revisions for inequality, so a wrap names no old
    /// list.
    sessions_revision: AtomicU32,
    /// The app config of this run: one snapshot, loaded at startup outside
    /// every lock and replaced as a whole ([`Self::replace_config`]).
    config: Mutex<Arc<ConfigSnapshot>>,
    /// The flattened tree of the last built revision, so a page of rows never
    /// regroups 50 000 sessions again (see [`TreeCache`]).
    tree: TreeCache,
}

impl AppStateHolder {
    /// A holder for a run whose config file was read at startup, before any
    /// lock existed.
    pub fn with_config(config: ConfigSnapshot) -> Self {
        Self {
            config: Mutex::new(Arc::new(config)),
            ..Self::default()
        }
    }

    /// Installs what runs the shared commands of every later fold. One sink for
    /// the whole holder, so the intent path, the child-event pump and a refresh
    /// pass all reach the same service.
    pub fn arm_commands(&self, sink: CommandSink) {
        *lock(&self.commands) = Some(sink);
    }

    /// Folds one intent of the contract through the reducer and answers the
    /// slice to push. Also the path the `state-changed` event payload takes.
    pub fn dispatch(&self, intent: UiIntent) -> ViewState {
        self.dispatch_commands(intent).0
    }

    /// [`Self::dispatch`] with the per-chat commands the fold asked for, for the
    /// caller to run through an executor (`effects`). The web sees the same
    /// slice either way: the commands are the shell's business, not the view's.
    pub fn dispatch_commands(&self, intent: UiIntent) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        let state = self.state();
        let input = gentle_app::Input::Intent(crate::intent::to_intent(&state, intent));
        self.apply(input)
    }

    /// Folds one reducer input through the same path. This is the slot the
    /// backend and the session passes fold through (`Input::ChatEvent`,
    /// `Input::SessionsProgress`), and it is what the tests seed state with.
    pub fn fold_input(&self, input: gentle_app::Input) -> ViewState {
        self.apply(input).0
    }

    /// Folds several inputs in order, answering the slice after the last one.
    pub fn fold_inputs(&self, inputs: impl IntoIterator<Item = gentle_app::Input>) -> ViewState {
        let mut view = self.view();
        for input in inputs {
            view = self.fold_input(input);
        }
        view
    }

    /// The slice the web side holds right now, without folding anything: the
    /// answer of `get_view_state`, the first read a sidebar makes. Reading never
    /// emits, so a webview that missed the pushes still gets the whole slice.
    pub fn view(&self) -> ViewState {
        let notice = self.config_notice();
        view_state(&self.state(), self.sessions_revision(), notice)
    }

    /// One window of the flattened workspace tree as the state holds it NOW:
    /// the rows `[offset, offset + length)` of the tree, and how many rows the
    /// tree has in total. The tree is built once per revision, from snapshots
    /// taken off the lock, so a fold on the pump's thread never waits for a
    /// page.
    pub fn sessions_window(&self, offset: u32, length: u32) -> SessionsWindow {
        let revision = self.sessions_revision();
        let tree = self.tree.get_or_build(revision, || {
            let config = self.config_snapshot();
            let sessions = self.sessions_snapshot();
            Tree::build(revision, &config.config, &sessions)
        });
        window::window(&tree, offset, length, now_ms(), &folder_status)
    }

    /// Replaces the config snapshot and bumps the revision, so the page of rows
    /// the web side holds is stale at once: the tree is built from the config
    /// too. The startup notice is kept, because it describes the file this run
    /// found and this slice never writes a config back.
    pub fn replace_config(&self, config: AppConfig) {
        let mut snapshot = self.config_snapshot();
        Arc::make_mut(&mut snapshot).config = config;
        *lock(&self.config) = snapshot;
        self.sessions_revision.fetch_add(1, Ordering::Relaxed);
    }

    /// The revision of the session list this holder has reached.
    pub fn sessions_revision(&self) -> u32 {
        self.sessions_revision.load(Ordering::Relaxed)
    }

    fn apply(&self, input: gentle_app::Input) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        // One lock for the fold and the slice it answers: a revision read here
        // describes exactly the state it is pushed with. The config notice is
        // read before it, so no path ever holds the state and the config at
        // once. The shared commands run after the lock is released, because a
        // refresh pass folds back into this same holder.
        let notice = self.config_notice();
        let (view, chat_commands, commands) = {
            let mut app = lock(&self.app);
            let previous = app.clone();
            let full = revision::folds_the_list(&input);
            let (state, effects) = gentle_app::update_chats(previous.clone(), input);
            if revision::list_moved(&previous, &state, full) {
                self.sessions_revision.fetch_add(1, Ordering::Relaxed);
            }
            *app = state;
            let view = view_state(&app, self.sessions_revision(), notice);
            (view, effects.chat_commands, effects.commands)
        };
        self.run_shared(commands);
        (view, chat_commands)
    }

    /// Runs the shared commands of one fold through the sink, if any.
    fn run_shared(&self, commands: Vec<Command>) {
        let sink = lock(&self.commands).clone();
        for command in commands {
            if let Some(sink) = sink.as_ref() {
                sink(command);
            }
        }
    }

    fn state(&self) -> gentle_app::AppState {
        lock(&self.app).clone()
    }

    /// The session list the tree is built from, taken off the lock.
    fn sessions_snapshot(&self) -> Vec<SessionItem> {
        lock(&self.app).sessions.clone()
    }

    /// The config snapshot of this run, taken off the lock.
    fn config_snapshot(&self) -> Arc<ConfigSnapshot> {
        Arc::clone(&lock(&self.config))
    }

    /// The config notice as the view state carries it.
    fn config_notice(&self) -> Option<String> {
        self.config_snapshot().notice.clone()
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
