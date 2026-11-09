//! The Tauri commands of the bridge and the effects they run.
//!
//! One command folds the intent, pushes the slice and then runs what the fold
//! asked for - one worker thread per chat command, because a child takes
//! seconds to start and a write to it blocks. Each outcome is folded back
//! through the reducer and pushed as the next `state-changed`, so the web side
//! sees the chat opening, then confirmed, failed or sent, without ever waiting
//! on a child.

use std::sync::Arc;

use tauri::AppHandle;
use tauri_specta::Event;

use crate::bridge::AppStateHolder;
use crate::contract::{StateChanged, UiIntent, ViewState};
use crate::effects::{self, Ran, RuntimeExecutor};
use crate::events::{EventPump, Push};

/// Everything the bridge owns and hands to the worker threads: the state, the
/// pump the children's events come back through, and the seam that runs a chat
/// command. The only managed state of the app.
pub struct Bridge {
    pub holder: Arc<AppStateHolder>,
    pub executor: Arc<RuntimeExecutor>,
    pub pump: Arc<EventPump>,
}

impl Bridge {
    /// One holder, one pump folding into it, and the executor that feeds the
    /// pump its host when the first command needs one.
    pub fn new() -> Self {
        let holder = Arc::new(AppStateHolder::default());
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        Self {
            holder,
            executor: Arc::new(RuntimeExecutor::new(Arc::clone(&pump))),
            pump,
        }
    }

    /// The window's emit step of the event pump, installed once at startup, so
    /// the pumped slices reach the web side (only Tauri's setup has the handle).
    pub fn arm_events(&self, app: &AppHandle<tauri::Wry>) {
        let app = app.clone();
        let push: Push = Box::new(move |view| push(&app, view));
        self.pump.arm(push);
    }

    /// Folds one intent, pushes the slice it answers, and runs the per-chat
    /// commands on worker threads. Returns the slice the command answers with,
    /// the one the web side may render before any child has answered.
    pub fn dispatch(
        &self,
        intent: UiIntent,
        app: &AppHandle<tauri::Wry>,
    ) -> Result<ViewState, String> {
        let (view, commands) = self.holder.dispatch_commands(intent);
        emit_state(app, &view)?;
        for command in commands {
            let holder = Arc::clone(&self.holder);
            let executor = Arc::clone(&self.executor);
            let app = app.clone();
            let spawned = std::thread::Builder::new()
                .name("chat-command".to_owned())
                .spawn(move || {
                    let ran = effects::run_one(command, executor.as_ref());
                    fold_and_push(&holder, &app, ran);
                });
            if let Err(error) = spawned {
                // A command that never runs leaves its chat opening until its
                // own first event or a later open ends the phase: a log line,
                // never a silent click that nobody can explain.
                eprintln!("chat command thread unavailable: {error}");
            }
        }
        Ok(view)
    }
}

/// A bridge with nothing run yet: the same one [`Bridge::new`] builds.
impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Folds what one command came to and pushes each step, then arms the 20 s net
/// of a fresh child against the same state and window.
fn fold_and_push(holder: &Arc<AppStateHolder>, app: &AppHandle<tauri::Wry>, ran: Ran) {
    for input in ran.inputs {
        push(app, holder.fold_input(input));
    }
    if let Some(chat) = ran.confirm_after {
        let holder = Arc::clone(holder);
        let app = app.clone();
        effects::opening_net(chat, move |input| {
            let view = holder.fold_input(input);
            let _ = emit_state(&app, &view);
        });
    }
}

fn push(app: &AppHandle<tauri::Wry>, view: ViewState) {
    if let Err(error) = emit_state(app, &view) {
        // Nothing to fall back to: the web side still has the command's answer.
        eprintln!("state-changed could not be emitted: {error}");
    }
}

/// The one push of the bridge: the typed event the web side listens on, the
/// same payload type the command returns.
fn emit_state(app: &AppHandle<tauri::Wry>, view: &ViewState) -> Result<(), String> {
    StateChanged(view.clone())
        .emit(app)
        .map_err(|error| error.to_string())
}

/// The one command this slice exposes: fold the intent, push the new slice and
/// run what it asked for. The emitted event is typed through tauri-specta, so
/// the Svelte side gets the same [`ViewState`] for the return and the payload.
#[tauri::command]
#[specta::specta]
pub fn dispatch_intent(
    intent: UiIntent,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().dispatch(intent, &app)
}
