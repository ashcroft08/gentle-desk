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

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatId, Command};

use crate::contract::{UiIntent, ViewState};
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
}

impl AppStateHolder {
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
        let input = gentle_app::Input::Intent(to_intent(&state, intent));
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

    /// The slice the web side holds right now, without folding anything.
    pub fn view(&self) -> ViewState {
        view_state(&self.state())
    }

    fn apply(&self, input: gentle_app::Input) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        let state = self.state();
        let (state, effects) = gentle_app::update_chats(state, input);
        *lock(&self.app) = state.clone();
        let view = view_state(&state);
        // After the state lock is released: a refresh request starts a pass
        // whose thread folds back into this same holder.
        self.run_shared(effects.commands);
        (view, effects.chat_commands)
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
}

/// Maps one intent of the contract to the reducer's intent. Explicit so the
/// contract stays free of `crates/app` types (S2).
fn to_intent(state: &gentle_app::AppState, intent: UiIntent) -> gentle_app::Intent {
    match intent {
        UiIntent::SendPrompt { text } => gentle_app::Intent::SendPrompt(text),
        UiIntent::Abort => gentle_app::Intent::Abort,
        UiIntent::NewChat => gentle_app::Intent::NewChat,
        UiIntent::OpenChat { chat_id } => gentle_app::Intent::OpenChat(chat_for(state, &chat_id)),
        UiIntent::OpenNewChat => gentle_app::Intent::OpenNewChat,
        UiIntent::RestartChat { chat_id } => {
            gentle_app::Intent::RestartChat(chat_for(state, &chat_id))
        }
        UiIntent::DismissNotice { id } => gentle_app::Intent::DismissNotice(u64::from(id)),
    }
}

/// The app's chat for an id the web side sent back.
///
/// The web only ever holds ids the state named - [`ViewState`] names every chat
/// by `ChatId::as_str` and every session row by its path - so the lookup is the
/// round trip of an id the view was given, and a chat of no session file
/// (`new-N`, which no path can rebuild) travels back unchanged. An id the state
/// has not heard of is read as the session path of a stored chat, which is how
/// the app builds that chat's id: the sidebar names a row by its path long
/// before any chat exists.
fn chat_for(state: &gentle_app::AppState, chat_id: &str) -> ChatId {
    state
        .chats
        .ids()
        .into_iter()
        .find(|chat| chat.as_str() == chat_id)
        .cloned()
        .unwrap_or_else(|| ChatId::of_session(chat_id))
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
