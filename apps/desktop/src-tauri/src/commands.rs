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

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::Command;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::bridge::{AppStateHolder, CommandSink};
use crate::clipboard::{self, Clipboard, SystemClipboard};
use crate::close::{self, BridgeSteps, CloseFlow, CLOSING};
use crate::config::{ConfigSnapshot, ConfigWriter};
use crate::contract::{StateChanged, UiIntent, ViewState};
use crate::effects::{self, Ran, RuntimeExecutor};
use crate::events::{EventPump, Push};
use crate::link_opener::{LinkOpener, SystemLinkOpener};
use crate::pending::PendingFolders;
use crate::picker::{FolderPicker, NativeFolderPicker};
use crate::sessions::{startup, SessionsService};
use crate::shell_install::{CommandInstaller, Installer};
use crate::theme::SystemWindowTheme;
use crate::window::{self, SystemWindowRecorder};
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
    /// The close of this run, armed once at startup because only the setup step
    /// has the handle that ends the process (`arm_close`).
    close: Mutex<Option<Arc<CloseFlow>>>,
    /// Set by [`UiIntent::RestartApp`]: the close that follows ends by starting
    /// the app again instead of leaving.
    relaunch: Arc<std::sync::atomic::AtomicBool>,
    /// What installs gentle-shell, once the user asked for it (shell-choice S4).
    installer: Arc<dyn Installer>,
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
        let snapshot = ConfigSnapshot::at(path.as_deref());
        // Read once: the home the user chose applies from this start on, and a
        // change made while the app runs waits for the next one.
        let home_choice = snapshot.config.shell_home;
        let holder = Arc::new(AppStateHolder::with_config(snapshot));
        holder.set_started_with(home_choice);
        // Whether the launcher resolves is asked once here too, so a machine with
        // no gentle-shell is told at the first frame, not at the first chat.
        holder.set_launcher_found(launcher_resolves());
        // Only the existence of pi's folder is asked, once, outside every lock.
        holder.set_pi_path(
            gentle_backend::home::detect_pi(
                &gentle_backend::home::HomeEnv::from_process(),
                |dir| dir.is_dir(),
            )
            .map(|dir| dir.to_string_lossy().into_owned()),
        );
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        let sessions = Arc::new(startup::service(Arc::clone(&holder), home_choice));
        let writer = ConfigWriter::spawn(path);
        holder.arm_config_saves(writer.clone());
        let folders = Arc::new(PendingFolders::default());
        let bridge = Self {
            holder: Arc::clone(&holder),
            executor: Arc::new(
                RuntimeExecutor::new(Arc::clone(&pump), Arc::clone(&folders), Arc::clone(&holder))
                    .with_home_choice(home_choice),
            ),
            pump,
            sessions,
            folders,
            clipboard,
            opener,
            picker: Arc::new(NativeFolderPicker::new()),
            writer,
            close: Mutex::new(None),
            relaunch: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            installer: Arc::new(CommandInstaller::npm()),
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

    /// Announces the list's emit step and boots the refresh of this run: the
    /// watcher, the safety net and the first full pass. The iced shell booted
    /// the same three (`crates/ui/src/app/boot.rs`: the watcher from
    /// `backend::start`, then `flow.on_tick()`), so the window opens with a list
    /// that is reading rather than a list that claims to be empty, and a session
    /// file the terminal writes shows up without a restart.
    ///
    /// A push that lands before the webview listens is no longer lost: the
    /// sidebar's first read is `get_view_state`, a command that answers the
    /// slice the state holds and emits nothing, so it starts from the truth
    /// whatever it missed. The boot is armed here because the setup step is the
    /// only place that has the handle, exactly as the child events are.
    pub fn start_sessions(&self, app: &AppHandle<tauri::Wry>) {
        self.sessions.arm(push_to(app));
        self.sessions.boot();
    }

    /// The window's own chrome follows the chosen theme from the first frame:
    /// this installs the real sink (only the setup step has the window) and
    /// applies the theme this run's config gave it, before the page is up. A run
    /// with no window installs nothing and changes nothing: the page is colored
    /// by the same theme either way.
    pub fn arm_window_theme(&self, app: &AppHandle<tauri::Wry>) {
        if let Some(window) = app.get_webview_window("main") {
            self.holder
                .arm_window_theme(Arc::new(SystemWindowTheme::new(window)));
        }
        self.holder.apply_window_theme();
    }

    /// Where the window opens, from the config of this run and the screens that
    /// exist right now: slice 1's rule decides (`app_config::placement`), and the
    /// window module runs the calls that answer it. Only the setup step has the
    /// window, so it runs there, once, before the page is up.
    ///
    /// The window is created hidden (`tauri.conf.json`), so this is also the step
    /// that shows it: `window::place` ends by showing a window that is still
    /// hidden, whatever the config said or whichever step failed. A config that
    /// could not be read is no stored state (`config_value` answers the document
    /// the run holds, with no `window`), which centres and shows. A run with no
    /// main window has nothing that could stay hidden, which is the one path that
    /// returns before the show - and it has no window to show.
    pub fn place_window(&self, app: &AppHandle<tauri::Wry>) {
        let Some(main) = app.get_webview_window("main") else {
            return;
        };
        let stored = self.holder.config_value().window;
        window::place(&main, stored, &window::screens(&main));
    }

    /// The close of this run, armed once at startup. The exit is the app's own
    /// (`AppHandle::exit`), kept behind a closure so `close` names no Tauri type
    /// and a test can run the same steps with a flag. This is the only place the
    /// close flow is built: it must be one per run, because its one-shot gate is
    /// what ignores a second close request.
    pub fn arm_close(&self, app: &AppHandle<tauri::Wry>) {
        let steps = BridgeSteps::new(
            Arc::clone(&self.executor),
            Arc::clone(&self.pump),
            self.sessions.shared(),
            self.writer.clone(),
            close::exit_step(
                Arc::clone(&self.relaunch),
                {
                    let app = app.clone();
                    move || app.restart()
                },
                {
                    let app = app.clone();
                    move || app.exit(0)
                },
            ),
        )
        // The watcher and the safety net stop before the cache write: a batch
        // must not ask for a pass after the last state was written.
        .with_sessions(Arc::clone(&self.sessions));
        // Where and how big the window was is a step of the close of its own,
        // before the config flush: only this arm has the window, and a run
        // without one closes without recording anything.
        let flow = CloseFlow::new(Arc::new(steps), close::Deadlines::default());
        let flow = match app.get_webview_window("main") {
            Some(main) => flow.with_window(Arc::new(SystemWindowRecorder::new(
                main,
                Arc::clone(&self.holder),
            ))),
            None => flow,
        };
        *lock(&self.close) = Some(Arc::new(flow));
    }

    /// The close of this run, if the close handler is armed. Cloned out, so the
    /// caller never holds the lock while the shutdown starts.
    pub fn close_flow(&self) -> Option<Arc<CloseFlow>> {
        lock(&self.close).clone()
    }

    /// Folds one intent, pushes the slice it answers, and runs the per-chat
    /// commands on worker threads. Returns the slice the command answers with,
    /// the one the web side may render before any child has answered.
    pub fn dispatch(
        &self,
        intent: UiIntent,
        app: &AppHandle<tauri::Wry>,
    ) -> Result<ViewState, String> {
        self.accepting_work()?;
        // Installing gentle-shell: only the permission screen's button sends
        // this, and the install runs on a thread of its own, so the window keeps
        // answering and shows its progress through the next push.
        if matches!(intent, UiIntent::InstallShell) {
            if self.holder.begin_install() {
                let holder = Arc::clone(&self.holder);
                let installer = Arc::clone(&self.installer);
                let app = app.clone();
                let started = std::thread::Builder::new()
                    .name("shell-install".to_owned())
                    .spawn(move || {
                        let view = after_install(&holder, installer.install(), &launcher_resolves);
                        push(&app, view);
                    });
                if started.is_err() {
                    self.holder
                        .finish_install(Err("could not start the install".to_owned()));
                }
            }
            let view = self.holder.view();
            emit_state(app, &view)?;
            return Ok(view);
        }
        if matches!(intent, UiIntent::CheckShellAgain) {
            let view = self.holder.recheck_launcher(&launcher_resolves);
            emit_state(app, &view)?;
            return Ok(view);
        }
        // A relaunch is the close flow with a different last step: the config
        // writer and the session cache are flushed first, as for any close.
        if matches!(intent, UiIntent::RestartApp) {
            self.relaunch
                .store(true, std::sync::atomic::Ordering::SeqCst);
            if let Some(close) = self.close_flow() {
                close.request();
            }
            return Ok(self.holder.view());
        }
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

    /// Attaches the images the drop named: the paths come from Tauri's own drop
    /// event (a browser `File` has no path), and the files are read here, once,
    /// before anything is held. Answers the slice to push, so a refusal reaches
    /// the composer as `images_notice` in the same answer.
    pub fn attach_images(
        &self,
        paths: &[String],
        app: &AppHandle<tauri::Wry>,
    ) -> Result<ViewState, String> {
        self.accepting_work()?;
        let view = self.holder.attach_images(paths);
        emit_state(app, &view)?;
        Ok(view)
    }

    /// Takes one attached image away before its send. Nothing is sent: the send
    /// that follows carries the images the chat still holds.
    pub fn remove_image(&self, id: &str, app: &AppHandle<tauri::Wry>) -> Result<ViewState, String> {
        self.accepting_work()?;
        let view = self.holder.remove_image(id);
        emit_state(app, &view)?;
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
        self.accepting_work()?;
        emit_state(app, &action.view)?;
        self.run_chat_commands(action.commands, app);
        Ok(action.view)
    }

    /// The gate of the two intent paths: while the close runs, nothing new is
    /// folded or opened here either, so nothing queues behind a window that is
    /// going away. The executor has the same gate for the commands it runs; this
    /// one stops the work before it is even folded.
    fn accepting_work(&self) -> Result<(), String> {
        if self.executor.is_closed() {
            return Err(CLOSING.to_owned());
        }
        Ok(())
    }
}

/// Whether the `gentle-shell` launcher resolves on this machine right now.
fn launcher_resolves() -> bool {
    gentle_backend::launcher::resolve(&gentle_backend::launcher::LauncherEnv::from_process())
        .is_ok()
}

/// What follows an install, whoever ran it: the outcome reaches the install state
/// machine as it stands, and only a SUCCESSFUL one re-probes the launcher, because
/// that is the one case where the launcher may now be there. The install screen
/// would otherwise stay up until the user pressed "Check again", right after an
/// install that had just worked (S4). A failure says nothing about the machine, so
/// nothing is probed and the view keeps claiming what it knew. Answers the slice to
/// push.
pub fn after_install(
    holder: &AppStateHolder,
    outcome: Result<(), String>,
    probe: &dyn Fn() -> bool,
) -> ViewState {
    let installed = outcome.is_ok();
    holder.finish_install(outcome);
    if installed {
        holder.recheck_launcher(probe);
    }
    holder.view()
}

/// A bridge with nothing run yet: the same one [`Bridge::new`] builds.
impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

/// A lock that survives a poisoned mutex: a thread that died must not leave the
/// close unarmed or unreadable for the rest of the run.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
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

/// The images a drop attached, as one command: the paths are the files the user
/// dragged, and they come from Tauri's own drop event, because the webview's
/// `File` has no path. Nothing but paths travel up: the shell reads the files,
/// checks them, and answers the slice with a thumbnail's name and URL.
#[tauri::command]
#[specta::specta]
pub fn attach_images(
    paths: Vec<String>,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().attach_images(&paths, &app)
}

/// Takes one attached image away before its send, named by the id
/// [`crate::contract::ImageView`] carried. Nothing is sent.
#[tauri::command]
#[specta::specta]
pub fn remove_image(
    id: String,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().remove_image(&id, &app)
}
