//! The Tauri commands of the bridge and the effects they run.
//!
//! One command folds the intent, pushes the slice and then runs what the fold
//! asked for - one worker thread per chat command, because a child takes
//! seconds to start and a write to it blocks. Each outcome is folded back
//! through the reducer and pushed as the next `state-changed`, so the web side
//! sees the chat opening, then confirmed, failed or sent, without ever waiting
//! on a child.
//!
//! The two "New workspace" commands of `workspace_commands` end here too: both
//! of them open a new chat through the same fold, so they finish through
//! [`Bridge::finish`] and the same worker threads, and a workspace added by hand
//! behaves like one that was there.

use std::sync::Arc;

use gentle_app::Command;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::bridge::{AppStateHolder, CommandSink};
use crate::clipboard::{self, Clipboard, SystemClipboard};
use crate::config::{ConfigSnapshot, ConfigWriter};
use crate::contract::{StateChanged, UiIntent, ViewState};
use crate::effects::{self, Ran, RuntimeExecutor};
use crate::events::{EventPump, Push};
use crate::link_opener::{LinkOpener, SystemLinkOpener};
use crate::pending::PendingFolders;
use crate::picker::{FolderPicker, NativeFolderPicker};
use crate::sessions::{startup, SessionsService};
use crate::workspace::{self, WorkspaceAction};

/// Everything the bridge owns and hands to the worker threads: the state, the
/// pump the children's events come back through, the session list's passes, the
/// seam that runs a chat command and the folders the workspace intents named.
/// The only managed state of the app.
pub struct Bridge {
    pub holder: Arc<AppStateHolder>,
    pub executor: Arc<RuntimeExecutor>,
    pub pump: Arc<EventPump>,
    pub sessions: Arc<SessionsService>,
    /// The folders of the workspace intents, waiting for the open of their
    /// chat: written by the intent path, taken by the executor.
    folders: Arc<PendingFolders>,
    /// The clipboard the Copy action writes to (T5a). The command sink is armed
    /// with it at startup, which is how a fold's `Command::CopyToClipboard`
    /// reaches it without the holder ever knowing a clipboard exists.
    clipboard: Arc<dyn Clipboard>,
    /// The browser opener the `open_link` command uses (T5a).
    opener: Arc<dyn LinkOpener>,
    /// The folder dialog of the two "New workspace" commands (Q1): the real
    /// one, kept by its own type because the setup step has to give it the
    /// window that makes the dialog modal. The flows take the seam, not this
    /// field.
    picker: Arc<NativeFolderPicker>,
    /// The one writer of the config file: a save never runs on this thread.
    writer: ConfigWriter,
}

impl Bridge {
    /// One holder, one pump folding into it, the sessions of this run, the
    /// executor that feeds the pump its host when the first command needs one,
    /// and the one writer of the config file.
    ///
    /// The app config file is read HERE, before the holder exists, so the one
    /// disk read of the config happens outside every lock; both the read and the
    /// writer take their path from the same place.
    pub fn new() -> Self {
        // The two seams of the thread slice: the system clipboard the Copy
        // action writes to, and the system browser a link opens in.
        let clipboard: Arc<dyn Clipboard> = Arc::new(SystemClipboard::new());
        let opener: Arc<dyn LinkOpener> = Arc::new(SystemLinkOpener::new());
        let path = ConfigSnapshot::path_from_env();
        let holder = Arc::new(AppStateHolder::with_config(ConfigSnapshot::at(
            path.as_deref(),
        )));
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        let sessions = Arc::new(startup::service(Arc::clone(&holder)));
        let writer = ConfigWriter::spawn(path);
        holder.arm_config_saves(writer.clone());
        let folders = Arc::new(PendingFolders::default());
        let bridge = Self {
            holder,
            executor: Arc::new(RuntimeExecutor::new(
                Arc::clone(&pump),
                Arc::clone(&folders),
            )),
            pump,
            sessions,
            folders,
            clipboard,
            opener,
            picker: Arc::new(NativeFolderPicker::new()),
            writer,
        };
        // The reducer doesn't refresh the list itself: it answers
        // `Command::RefreshSessions` and this is where that one is run. A copy
        // is run by the same sink, off the thread that folded it, so the two
        // services are taken from the bridge they belong to.
        bridge.holder.arm_commands(command_sink(
            Some(Arc::clone(&bridge.sessions)),
            Arc::clone(&bridge.clipboard),
        ));
        bridge
    }

    /// The browser opener, as the `open_link` command takes it: cloned out of
    /// the bridge before the open runs, so no lock is held while it does.
    pub fn opener(&self) -> Arc<dyn LinkOpener> {
        Arc::clone(&self.opener)
    }

    /// The folder dialog as the seam the workspace flows take.
    pub fn picker(&self) -> Arc<dyn FolderPicker> {
        self.picker.clone()
    }

    /// The folders waiting for the open that takes them.
    pub fn folders(&self) -> Arc<PendingFolders> {
        Arc::clone(&self.folders)
    }

    /// The window's emit step of the event pump, installed once at startup, so
    /// the pumped slices reach the web side (only Tauri's setup has the handle).
    /// The same place arms what a failed config save says, because that reason
    /// reaches the user through the same push.
    pub fn arm_events(&self, app: &AppHandle<tauri::Wry>) {
        self.pump.arm(push_to(app));
        // The folder dialog is owned by the window, so it is modal to the app
        // instead of to the desktop. The window exists only from this step, and
        // it is the same step that arms the pump, for the same reason.
        if let Some(window) = app.get_webview_window("main") {
            self.picker.set_parent(window);
        }
        let holder = Arc::clone(&self.holder);
        let failure_app = app.clone();
        self.writer.on_failure(Arc::new(move |message: String| {
            let view = holder.note_config_problem(message);
            push(&failure_app, view);
        }));
    }

    /// Announces the list's emit step and asks for the first full pass. The
    /// iced shell asks for that pass as its first task
    /// (`crates/ui/src/app/boot.rs`), so the window opens with a list that is
    /// reading rather than a list that claims to be empty.
    ///
    /// A push that lands before the webview listens is no longer lost: the
    /// sidebar's first read is `get_view_state`, a command that answers the
    /// slice the state holds and emits nothing, so it starts from the truth
    /// whatever it missed. The pass is armed here because the setup step is the
    /// only place that has the handle, exactly as the child events are.
    pub fn start_sessions(&self, app: &AppHandle<tauri::Wry>) {
        let push = push_to(app);
        self.sessions.arm(push);
        self.sessions.request();
    }

    /// Folds one intent, pushes the slice it answers, and runs the per-chat
    /// commands on worker threads. Returns the slice the command answers with,
    /// the one the web side may render before any child has answered.
    pub fn dispatch(
        &self,
        intent: UiIntent,
        app: &AppHandle<tauri::Wry>,
    ) -> Result<ViewState, String> {
        // The folder of a chat that starts in a workspace travels BESIDE the
        // command: the reducer names the chat and `crates/app` carries no folder
        // at all (S2). A blank path names none, so nothing is opened for it.
        let folder = match &intent {
            UiIntent::NewChatInWorkspace { path } => Some(path.clone()),
            _ => None,
        };
        let (view, commands) = self.holder.dispatch_commands(intent);
        emit_state(app, &view)?;
        if let Some(folder) = folder {
            // Before the first command runs on its thread: the executor takes it
            // from here, once.
            workspace::remember_folder(&self.folders, &commands, &folder);
        }
        self.run_chat_commands(commands, app);
        Ok(view)
    }

    /// Runs the per-chat commands one fold asked for, one worker thread per chat
    /// command, because a child takes seconds to start and a write to it blocks.
    /// Never waits for one: each outcome folds back through the reducer and
    /// pushes the next `state-changed`.
    pub(crate) fn run_chat_commands(
        &self,
        commands: Vec<gentle_app::ChatCommand>,
        app: &AppHandle<tauri::Wry>,
    ) {
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
    }

    /// Pushes the slice one workspace action answers, then runs the new chat it
    /// opened: exactly what [`Bridge::dispatch`] does with the commands of the
    /// reducer, so a workspace added by hand behaves like one that was there.
    pub(crate) fn finish(
        &self,
        action: WorkspaceAction,
        app: &AppHandle<tauri::Wry>,
    ) -> Result<ViewState, String> {
        emit_state(app, &action.view)?;
        self.run_chat_commands(action.commands, app);
        Ok(action.view)
    }
}

/// A bridge with nothing run yet: the same one [`Bridge::new`] builds.
impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

/// What runs the shared commands of the reducer: the session list's pass and
/// the message Copy here, and nothing for the one that belongs to the one-child
/// shell (a send to the child this app does not use).
///
/// `sessions` is `None` in a test that only exercises the clipboard: the list's
/// branch then answers nothing, and the copy branch is the one under test.
pub fn command_sink(
    sessions: Option<Arc<SessionsService>>,
    clipboard: Arc<dyn Clipboard>,
) -> CommandSink {
    Arc::new(move |command| match command {
        Command::RefreshSessions => {
            if let Some(sessions) = sessions.as_ref() {
                sessions.request();
            }
        }
        // Never on the thread that folded the intent, and never under a lock:
        // the write happens on a thread of its own (`clipboard`).
        Command::CopyToClipboard(text) => {
            clipboard::copy_off_thread(Arc::clone(&clipboard), text);
        }
        Command::Send(_) => {}
    })
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

/// The window's emit step as the [`Push`] a pump or the session list arms: the
/// same closure at each of the three places one is installed.
fn push_to(app: &AppHandle<tauri::Wry>) -> Push {
    let app = app.clone();
    Box::new(move |view| push(&app, view))
}

/// The one push of the bridge: the typed event the web side listens on, the
/// same payload type the command returns.
fn emit_state(app: &AppHandle<tauri::Wry>, view: &ViewState) -> Result<(), String> {
    StateChanged(view.clone())
        .emit(app)
        .map_err(|error| error.to_string())
}

/// One command this slice exposes: fold the intent, push the new slice and
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
