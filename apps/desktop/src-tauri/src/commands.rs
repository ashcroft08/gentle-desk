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
//!
//! Nothing here needs a window: the one thing the running app gets from Tauri is
//! the EMIT step ([`Emit`]), installed at startup and replaceable by a recorder,
//! and the two things a machine answers - whether the `gentle-shell` launcher
//! resolves ([`LauncherProbe`]) and what installs it ([`Installer`]) - are seams
//! of their own. A test therefore drives the real intent path windowless.

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::Command;
use gentle_core::i18n::messages;
use gentle_runtime::whats_new::{self, Decision, Latest, UpdateRun};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::bridge::{AppStateHolder, ChatWork, CommandSink};
use crate::clipboard::{self, Clipboard, SystemClipboard};
use crate::close::{self, BridgeSteps, CloseFlow};
use crate::config::{ConfigSnapshot, ConfigWriter};
use crate::contract::{
    AttachmentView, SettingsFilesChanged, SettingsFilesChangedView, StateChanged,
    TerminalActionView, UiIntent, UpdatesStatusView, UpdatesView, UsageStatusView, UsageView,
    ViewState,
};
use crate::effects::{self, Ran, RuntimeExecutor};
use crate::events::{EventPump, Push};
use crate::link_opener::{LinkOpener, SystemLinkOpener};
use crate::pending::PendingFolders;
use crate::picker::{FilePicker, FolderPicker, NativeFilePicker, NativeFolderPicker};
use crate::reaper;
use crate::sessions::{startup, SessionsService};
use crate::settings::{
    self, Changed, Dirs, FileRead, ProviderProbe, SettingsSeams, SettingsWatch, VersionProbe,
    WebGet,
};
use crate::settings_snapshot::{self, Reads, SettingsConfig, SettingsSnapshot};
use crate::shell_install::{CommandInstaller, Installer};
use crate::theme::SystemWindowTheme;
use crate::window::{self, SystemWindowRecorder};
use crate::workspace::{self, WorkspaceAction};

/// What shows one slice to the web side: the typed `state-changed` event in the
/// running app, a recorder in a test. `Send + Sync` because the install pushes
/// from a thread of its own and the pump pushes from its two.
pub type Emit = Arc<dyn Fn(&ViewState) -> Result<(), String> + Send + Sync>;

/// What reads `latest.json` for the what's-new step (S3): the endpoint the
/// updater config names, and the manifest it answers, or the reason it could not
/// be read. The real one is the shell's own fetch of that URL; a test gives a
/// closure, so no test here ever reaches the network.
pub type Fetcher = Arc<dyn Fn(&str) -> Result<Latest, String> + Send + Sync>;

/// Whether the `gentle-shell` launcher resolves on this machine right now: the
/// real probe for the app ([`launcher_resolves`]), a flag in a test. Asked at
/// startup, on "Check again" and after a successful install, and for nothing
/// else.
pub type LauncherProbe = Arc<dyn Fn() -> bool + Send + Sync>;

/// What pushes one debounced notice of the Settings files to the web side: the
/// typed `settings-files-changed` event in the running app, a recorder in a test.
/// `Send + Sync` because the watcher's own thread is what calls it.
pub type SettingsNotify = Arc<dyn Fn(Changed) -> Result<(), String> + Send + Sync>;

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
    /// The file dialog the composer's clip asks (design/composer-dock T4, S3):
    /// the real one, sharing the folder picker's window parent so both dialogs
    /// are modal to the app. The command takes the seam, not this field.
    file_picker: Arc<NativeFilePicker>,
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
    /// The one push of the bridge, as the running app installed it (`arm_emit`)
    /// or as a test gave it ([`Bridge::with_emit`]). Before it is installed
    /// nothing is shown, which is a run with no window.
    emit: Mutex<Emit>,
    /// Whether the `gentle-shell` launcher resolves (shell-choice S1-S2): the
    /// real probe unless a test gave one ([`Bridge::with_launcher`]).
    launcher: LauncherProbe,
    /// The Settings pane's seams and paths (0.1.12): the files, the provider
    /// check, the version probe and one GET. Nothing runs until the pane asks or
    /// the update check starts.
    settings: SettingsSeams,
    /// Where a later "Check again" reads the desk's own manifest from: the same
    /// endpoint the updater names, named once at startup by
    /// [`Bridge::start_updates`].
    desk_endpoint: Mutex<Option<String>>,
    /// The watcher of pi's Settings files (rpc-settings-sync T3b-1, S2): the
    /// debounced notice that the panes' files changed on disk. Started once at
    /// boot and stopped by the close; a run whose directories could not be
    /// watched keeps the panes' focus refresh.
    settings_watch: Arc<SettingsWatch>,
    /// The one push of that watcher, installed with the window's emit step: a
    /// no-op until then, so a notice that came before the window existed is not
    /// a failure.
    settings_notify: Mutex<SettingsNotify>,
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
        // The two dialogs share one window parent: only the setup step has a
        // window, and it installs it on the folder picker, which is the one the
        // file picker was built from.
        let picker = Arc::new(NativeFolderPicker::new());
        let file_picker = Arc::new(NativeFilePicker::with_parent(picker.parent()));
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
            picker,
            file_picker,
            writer,
            close: Mutex::new(None),
            relaunch: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            installer: Arc::new(CommandInstaller::npm()),
            // Nothing to push to until the window exists: the setup step
            // installs the real emit, exactly as it arms the pump.
            emit: Mutex::new(Arc::new(|_view: &ViewState| Ok(()))),
            launcher: Arc::new(launcher_resolves),
            // The Settings pane's seams read the environment once, here: the
            // files it reads and the flags the shell runs with.
            settings: SettingsSeams::new(home_choice),
            desk_endpoint: Mutex::new(None),
            settings_watch: Arc::new(SettingsWatch::new()),
            // Nothing to push to until the window exists: the setup step installs
            // the real event, exactly as it installs the state's emit.
            settings_notify: Mutex::new(Arc::new(|_changed: Changed| Ok(()))),
        };
        // Whether the launcher resolves is asked once here, through the seam, so
        // a machine with no gentle-shell is told at the first frame, not at the
        // first chat.
        bridge.holder.set_launcher_found((bridge.launcher)());
        // The reducer doesn't refresh the list itself: it answers
        // `Command::RefreshSessions` and this is where that one is run. A copy
        // is run by the same sink, off the thread that folded it, so the two
        // services are taken from the bridge they belong to.
        bridge.holder.arm_commands(command_sink(
            Some(Arc::clone(&bridge.sessions)),
            Arc::clone(&bridge.clipboard),
        ));
        // The Profile tab's binding half is read from a chat's own session file
        // (profiles-editor T2b-2, L9), through the same file seam the Settings
        // panes read through, and only inside this run's sessions store (the
        // folder the session list itself enumerates, T2b-3): armed once here,
        // like the command sink above. A run with no store reads no file.
        if let Some(store) = bridge.settings.paths.sessions.clone() {
            bridge
                .holder
                .arm_session_files(Arc::clone(&bridge.settings.files), store);
        }
        bridge
    }

    /// The app's own emit step, installed once at startup: only the setup step
    /// has the window handle, and this is where every push of this bridge gets
    /// it. The pump and the session list are armed with the same seam
    /// ([`Bridge::arm_events`]), because they push the same payload.
    pub fn arm_emit(&self, app: &AppHandle<tauri::Wry>) {
        let app = app.clone();
        *lock(&self.emit) = Arc::new(move |view: &ViewState| emit_state(&app, view));
    }

    /// The same seam, given outright instead of built from a window: a recorder
    /// in a test that runs the real dispatch windowless.
    pub fn with_emit(mut self, emit: Emit) -> Self {
        self.emit = Mutex::new(emit);
        self
    }

    /// The launcher probe this bridge asks, instead of the real one. It is asked
    /// here once, exactly as [`Bridge::new`] asks the real one, so the answer of
    /// this probe is what the run starts with, and again wherever the app asks
    /// ("Check again" and the completion of an install).
    pub fn with_launcher(mut self, probe: LauncherProbe) -> Self {
        self.holder.set_launcher_found(probe());
        self.launcher = probe;
        self
    }

    /// What installs gentle-shell, instead of the real `npm install -g`.
    pub fn with_installer(mut self, installer: Arc<dyn Installer>) -> Self {
        self.installer = installer;
        self
    }

    /// The Settings pane's file reader, instead of the machine's: a test reads a
    /// scratch profiles.json or auth.json without touching a real home.
    pub fn with_settings_files(mut self, files: FileRead) -> Self {
        self.settings.files = files;
        self
    }

    /// The per-provider check, instead of the real `auth check` run: no test
    /// starts a shell.
    pub fn with_provider_probe(mut self, probe: ProviderProbe) -> Self {
        self.settings.probe = probe;
        self
    }

    /// The version probe, instead of the real `gentle-shell --version`.
    pub fn with_version_probe(mut self, versions: VersionProbe) -> Self {
        self.settings.versions = versions;
        self
    }

    /// One GET, instead of the real one: no test of the update check reaches the
    /// network.
    pub fn with_web_get(mut self, web: WebGet) -> Self {
        self.settings.web = web;
        self
    }

    /// The usage sources' run, instead of `node usage-json.ts`: no test starts a
    /// script or needs node installed.
    pub fn with_node_probe(mut self, node: crate::settings::NodeProbe) -> Self {
        self.settings.node = node;
        self
    }

    /// Whether [`UiIntent::RestartApp`] asked for a relaunch: the flag the close
    /// flow's last step reads (`close::exit_step`).
    pub fn relaunch_requested(&self) -> bool {
        self.relaunch.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The emit seam as it stands, cloned out so no lock is held while a push
    /// runs. `Send + Sync`, so a worker thread can push with it too.
    fn emitter(&self) -> Emit {
        lock(&self.emit).clone()
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

    /// The file dialog as the seam the clip's command takes.
    pub fn file_picker(&self) -> Arc<dyn FilePicker> {
        self.file_picker.clone()
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
        // The bridge's own pushes need the same handle the pump's do, so the two
        // are installed in the same step.
        self.arm_emit(app);
        // The Settings files' watcher pushes an event of its own, from the window
        // handle this step is the only place to get.
        self.arm_settings_notify(app);
        let emit = self.emitter();
        // The pump folds a child's event and hands the chat commands that fold
        // asked for to this sink (chat-event-commands, S1-S4): the same runner
        // the intent path uses, on the same worker threads, and never on the
        // pump's own thread. Armed before the pump's push, because arming the
        // push is what may start it.
        self.arm_chat_commands(&emit);
        // The branch chip's read runs off the window (design/composer-dock S2,
        // R3, R4), so what it answers reaches the web the way every other late
        // answer does: a push from the thread that read it. The holder is held
        // weakly, exactly as the chat-command sink holds it, because the sink
        // lives inside the holder it pushes.
        let branch_holder = Arc::downgrade(&self.holder);
        let branch_emit = Arc::clone(&emit);
        self.holder.arm_branch_push(Arc::new(move || {
            if let Some(holder) = branch_holder.upgrade() {
                let view = holder.view();
                push(&branch_emit, view);
            }
        }));
        // The thread's first frame is read from the chat's own session file off
        // the window (chat-open-latency T1), so its answer reaches the web the
        // same way: a push from the thread that read it, and only when the read
        // may still be drawn (`note_transcript` answers nothing for one a close or
        // the child's own answer superseded).
        let transcript_holder = Arc::downgrade(&self.holder);
        let transcript_emit = Arc::clone(&emit);
        self.holder.arm_transcript_push(Arc::new(move |read| {
            if let Some(holder) = transcript_holder.upgrade() {
                if let Some(view) = holder.note_transcript(&read) {
                    push(&transcript_emit, view);
                }
            }
        }));
        self.pump.arm(push_via(emit.clone()));
        // The folder dialog is owned by the window, so it is modal to the app
        // instead of to the desktop. The window exists only from this step, and
        // it is the same step that arms the pump, for the same reason.
        if let Some(window) = app.get_webview_window("main") {
            self.picker.set_parent(window);
        }
        let holder = Arc::clone(&self.holder);
        let failure_emit = emit;
        self.writer.on_failure(Arc::new(move |message: String| {
            let view = holder.note_config_problem(message);
            push(&failure_emit, view);
        }));
    }

    /// Arms what runs the per-chat commands a CHILD EVENT's fold asked for: the
    /// pump's delivery (chat-event-commands, S1-S4). It is [`run_chat_commands`],
    /// the same runner the intent path hands its own commands to, so a command
    /// has one implementation and each fold one delivery - the pump's or the
    /// intent path's, never both.
    fn arm_chat_commands(&self, emit: &Emit) {
        // Weak handles, because the holder owns this sink and the pump the
        // executor holds is held by the same holder: a strong handle to either
        // would keep the whole run alive. Neither can be gone while the pump
        // folds - the bridge holds both for the run's whole life - and a sink
        // that finds neither simply runs nothing.
        let holder = Arc::downgrade(&self.holder);
        let executor = Arc::downgrade(&self.executor);
        let emit = Arc::clone(emit);
        self.holder.arm_chat_commands(Arc::new(move |commands| {
            let (Some(holder), Some(executor)) = (holder.upgrade(), executor.upgrade()) else {
                return;
            };
            run_chat_commands(&holder, executor, &emit, commands);
        }));
    }

    /// Installs the real push of the Settings files' watcher: the typed
    /// `settings-files-changed` event, from the same handle every other push
    /// gets. Only the setup step has the window, so it is armed beside the state
    /// emit.
    fn arm_settings_notify(&self, app: &AppHandle<tauri::Wry>) {
        let app = app.clone();
        *lock(&self.settings_notify) =
            Arc::new(move |changed: Changed| emit_settings_files_changed(&app, changed));
    }

    /// Starts the watcher of pi's Settings files (rpc-settings-sync T3b-1, S2),
    /// right beside the session list's: the directory of `profiles.json` and
    /// `profile-fallbacks.json`, and the agent directory of `auth.json`, both
    /// resolved by the same [`SettingsPaths`](crate::settings::SettingsPaths) the
    /// panes read through.
    ///
    /// The watcher runs NO subprocess and calls NO refresh command: it pushes the
    /// typed event, and the panes that are mounted decide to re-read through
    /// their own commands (T3b-2). That is what keeps `auth check --no-refresh`
    /// from running in the background while the Settings dialog is closed.
    ///
    /// Answers whether a watch runs. A run that could not watch anything is not
    /// told to the user: the panes keep the focus refresh they have today. The
    /// close stops it through [`Bridge::arm_close`]'s steps, in the same bounded
    /// step as the session watcher, and a second call replaces the running watch
    /// with the directories and the push of this one (`SettingsWatch::start`).
    pub fn start_settings_watch(&self) -> bool {
        let dirs = Dirs::of(&self.settings.paths);
        let notify = lock(&self.settings_notify).clone();
        let on_change: settings::watch::OnChange = Arc::new(move |changed| {
            if let Err(error) = notify(changed) {
                // The web side missed one notice: the panes keep the focus
                // refresh, so this is a log line and nothing else.
                eprintln!("settings-files-changed could not be emitted: {error}");
            }
        });
        self.settings_watch.start(&dirs, on_change)
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
    pub fn start_sessions(&self) {
        // The list pushes the same payload through the same seam the step before
        // installed: it needs no handle of its own.
        self.sessions.arm(push_via(self.emitter()));
        self.sessions.boot();
    }

    /// Starts the timer that releases the idle chat children (S7). One thread for
    /// the whole run: the host's window and protection rules decide what goes,
    /// and the reaper only gives them a clock.
    pub fn start_reaper(&self) {
        reaper::start(Arc::clone(&self.executor));
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
        // must not ask for a pass after the last state was written. The Settings
        // files' watcher stops in the same step (rpc-settings-sync T3b-1), for the
        // same reason: its notice must not ask a pane to read after the flush.
        .with_sessions(Arc::clone(&self.sessions))
        .with_settings_watch(Arc::clone(&self.settings_watch));
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
    pub fn dispatch(&self, intent: UiIntent) -> Result<ViewState, String> {
        self.accepting_work()?;
        // The one push of this call, cloned out before any thread is given it.
        let emit = self.emitter();
        // Installing gentle-shell: only the permission screen's button sends
        // this, and the install runs on a thread of its own, so the window keeps
        // answering and shows its progress through the next push.
        if matches!(intent, UiIntent::InstallShell) {
            if self.holder.begin_install() {
                let holder = Arc::clone(&self.holder);
                let installer = Arc::clone(&self.installer);
                // The completion probe and the emit both travel with the
                // thread: neither needs the window, and the handle is never
                // cloned to reach it.
                let launcher = Arc::clone(&self.launcher);
                let emit = Arc::clone(&emit);
                let started = std::thread::Builder::new()
                    .name("shell-install".to_owned())
                    .spawn(move || {
                        // The language is read when the install starts, on the
                        // thread that owns it: a switch applies from the next
                        // attempt on (0.2.0, S3).
                        let language = holder.language();
                        let view =
                            after_install(&holder, installer.install(language), launcher.as_ref());
                        push(&emit, view);
                    });
                if started.is_err() {
                    let failure = messages::install_could_not_start(self.holder.language());
                    self.holder.finish_install(Err(failure.to_owned()));
                }
            }
            let view = self.holder.view();
            emit_view(&emit, &view)?;
            return Ok(view);
        }
        if matches!(intent, UiIntent::CheckShellAgain) {
            let view = self.holder.recheck_launcher(self.launcher.as_ref());
            emit_view(&emit, &view)?;
            return Ok(view);
        }
        // The Settings panes' intents (0.1.12) are the shell's, not the
        // reducer's: they read a file or run the shell's own check. Each answers
        // the slice it pushed.
        match &intent {
            UiIntent::RefreshAuth => return self.refresh_auth(),
            UiIntent::CheckUpdates => return self.check_updates(),
            UiIntent::RefreshUsage => return self.refresh_usage(),
            _ => {}
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
        // state: the reducer names the chat and `crates/app` carries no folder
        // at all (S2). A blank path names none, so nothing is opened for it, and
        // the chat it belongs to is the one the answered slice names - a new chat
        // asks for no child until its composer does (welcome-screen T4, R2), so
        // the folder waits for that ask.
        let folder = match &intent {
            UiIntent::NewChatInWorkspace { path } => Some(path.clone()),
            _ => None,
        };
        let (view, commands) = self.holder.dispatch_commands(intent);
        if let Some(folder) = folder {
            workspace::remember_folder(&self.folders, view.foreground_chat.as_deref(), &folder);
        }
        emit_view(&emit, &view)?;
        self.run_chat_commands(commands);
        Ok(view)
    }

    /// Attaches the images the drop named: the paths come from Tauri's own drop
    /// event (a browser `File` has no path), and the files are read here, once,
    /// before anything is held. Answers the slice to push, so a refusal reaches
    /// the composer as `images_notice` in the same answer.
    pub fn attach_images(&self, paths: &[String]) -> Result<ViewState, String> {
        self.accepting_work()?;
        let view = self.holder.attach_images(paths);
        emit_view(&self.emitter(), &view)?;
        Ok(view)
    }

    /// Takes one attached image away before its send. Nothing is sent: the send
    /// that follows carries the images the chat still holds.
    pub fn remove_image(&self, id: &str) -> Result<ViewState, String> {
        self.accepting_work()?;
        let view = self.holder.remove_image(id);
        emit_view(&self.emitter(), &view)?;
        Ok(view)
    }

    /// Runs the per-chat commands one fold asked for, one worker thread per chat
    /// command, because a child takes seconds to start and a write to it blocks.
    /// Never waits for one: each outcome folds back through the reducer and
    /// pushes the next `state-changed`.
    pub(crate) fn run_chat_commands(&self, commands: Vec<gentle_app::ChatCommand>) {
        run_chat_commands(
            &self.holder,
            Arc::clone(&self.executor),
            &self.emitter(),
            commands,
        );
    }

    /// The close a workspace removal runs before it deletes (S5), as a value a
    /// blocking thread can carry: the reducer's Close chat folded here, and the
    /// command it answers run TO ITS END, because the session file the removal
    /// deletes next must not be one a child still holds open.
    ///
    /// Unlike [`run_chat_commands`](Self::run_chat_commands) this never hands a
    /// command to a worker thread and never returns before it settled: the order
    /// the maintainer ruled (close, then delete, then forget) is the caller's
    /// thread's.
    pub(crate) fn close_first(&self) -> crate::workspace_remove::ShellClose {
        let holder = Arc::clone(&self.holder);
        let executor = Arc::clone(&self.executor);
        let emit = self.emitter();
        let run = move |commands: Vec<gentle_app::ChatCommand>| {
            for command in commands {
                let ran = effects::run_one(command, executor.as_ref());
                fold_and_push(&holder, &emit, ran);
            }
        };
        crate::workspace_remove::ShellClose::new(Arc::clone(&self.holder), Arc::new(run))
    }

    /// Pushes the slice one workspace action answers, then runs the new chat it
    /// opened: exactly what [`Bridge::dispatch`] does with the commands of the
    /// reducer, so a workspace added by hand behaves like one that was there.
    pub(crate) fn finish(&self, action: WorkspaceAction) -> Result<ViewState, String> {
        self.accepting_work()?;
        emit_view(&self.emitter(), &action.view)?;
        self.run_chat_commands(action.commands);
        Ok(action.view)
    }

    /// The gate of the two intent paths: while the close runs, nothing new is
    /// folded or opened here either, so nothing queues behind a window that is
    /// going away. The executor has the same gate for the commands it runs; this
    /// one stops the work before it is even folded.
    pub(crate) fn accepting_work(&self) -> Result<(), String> {
        if self.executor.is_closed() {
            return Err(close::closing(self.holder.language()));
        }
        Ok(())
    }

    /// The what's-new step of this run (what's-new S1, S3, S5, S6): the version
    /// this run runs as, what the config stored, and - only when this run follows
    /// an update - one fetch of `latest.json` through the seam.
    ///
    /// Nothing here waits for the network: the fetch runs on a thread of its own
    /// and pushes the slice it answers through the same emit step every other
    /// step uses, so the first frame never depends on it. A run of the stored
    /// version asks nothing and pushes nothing at all; a first run records the
    /// version it runs as before it returns (R3-001), so the next update is
    /// detected, and shows no modal itself.
    pub fn start_whats_new(&self, app_version: &str, endpoint: Option<&str>, fetch: Fetcher) {
        self.holder.set_app_version(app_version);
        if self.holder.update_run() == UpdateRun::FirstRun {
            // A first run - any config from before this feature stored a
            // version - records the version it runs as before it returns:
            // returning without storing would keep the run FirstRun forever,
            // so no later update would ever be detected.
            if let Some(view) = self.holder.store_last_run_version() {
                push(&self.emitter(), view);
            }
            return;
        }
        if self.holder.update_run() != UpdateRun::Updated {
            return;
        }
        let emit = self.emitter();
        let Some(endpoint) = endpoint.map(str::to_owned) else {
            // No endpoint in the config to read the notes from: the modal is best
            // effort, the bookkeeping is not, so the version is stored now (S5).
            if let Some(view) = self.holder.store_last_run_version() {
                push(&emit, view);
            }
            return;
        };
        let holder = Arc::clone(&self.holder);
        let version = app_version.to_owned();
        let thread_emit = Arc::clone(&emit);
        let spawned = std::thread::Builder::new()
            .name("whats-new".to_owned())
            .spawn(move || {
                let decision = whats_new::decide(fetch(&endpoint), &version);
                if let Some(view) = after_fetch(&holder, decision) {
                    push(&thread_emit, view);
                }
            });
        if spawned.is_err() {
            // No thread to fetch on: the fetch is skipped rather than delayed onto
            // the thread that draws, and the version is stored now (S5).
            if let Some(view) = self.holder.store_last_run_version() {
                push(&emit, view);
            }
        }
    }

    /// The dismiss of the what's-new modal (S5): this run's own version is stored
    /// as the version the last run was, the notes are cleared and the slice that
    /// carries neither is pushed. A run that never named its version has nothing
    /// to store, and answers the slice it holds.
    pub fn store_last_run_version(&self) -> Result<ViewState, String> {
        self.accepting_work()?;
        let emit = self.emitter();
        match self.holder.store_last_run_version() {
            Some(view) => {
                emit_view(&emit, &view)?;
                Ok(view)
            }
            None => Ok(self.holder.view()),
        }
    }

    /// Opens a visible terminal window on one of the jobs the desk hands to the
    /// shell (0.1.12, S1, S2, T2, T3): the profiles screen, its login or logout,
    /// or its own update. It is a COMMAND, like `open_link`: no intent folds and
    /// nothing of the slice changes, so no push follows it.
    ///
    /// The window runs with the home flags this run's children get, so the
    /// terminal reads and writes the home the pane is showing, and the refusal
    /// (no launcher, no console) reaches the button that asked.
    pub fn open_terminal(&self, action: TerminalActionView) -> Result<(), String> {
        self.accepting_work()?;
        crate::terminal::open(
            self.holder.language(),
            action,
            &self.settings.paths.home_args,
        )
    }

    /// The Account pane's read (0.1.12, T3): the provider names the auth store
    /// lists, and one `auth check --no-refresh` per provider, pushed as the next
    /// slice.
    ///
    /// The checks run on the intent's own thread - one short process each, and
    /// the pane draws the answer it asked for - and nothing here refreshes a
    /// token: `--no-refresh` is part of the command.
    pub fn refresh_auth(&self) -> Result<ViewState, String> {
        self.accepting_work()?;
        let emit = self.emitter();
        let view = self.holder.note_auth(settings::auth_view(
            &self.settings.files,
            self.settings.paths.auth.as_deref(),
            &self.settings.probe,
            self.holder.language(),
        ));
        emit_view(&emit, &view)?;
        Ok(view)
    }

    /// One update check (0.1.12, S2, S3, S7): "Check again" from the pane. The
    /// answer to the click says the check is running, and the check's own thread
    /// pushes the versions it found. A check only ever CHECKS: no update is
    /// installed from here.
    pub fn check_updates(&self) -> Result<ViewState, String> {
        self.accepting_work()?;
        let emit = self.emitter();
        let view = self.holder.note_updates_checking();
        emit_view(&emit, &view)?;
        self.run_update_check(emit);
        Ok(view)
    }

    /// The automatic check of this run (S3): once, when the app opens, and never
    /// on the thread that draws. The endpoint is kept so a later "Check again"
    /// reads the same manifest.
    pub fn start_updates(&self, desk_endpoint: Option<&str>) {
        *lock(&self.desk_endpoint) = desk_endpoint.map(str::to_owned);
        self.run_update_check(self.emitter());
    }

    /// The Settings dialog's one read (settings-snapshot): every page's data
    /// in one object, gathered once per open and never on the window's thread.
    ///
    /// The reads are the three the panes used to run on their own - the auth
    /// store and its per-provider check, the usage sources' one run, and the
    /// update check - plus the half of the slice this run already holds, which
    /// costs nothing. Every seam is cloned out BEFORE the load starts, so no
    /// lock is held while a file, a process or a socket is read, and the reads
    /// themselves run on a blocking worker, so the window keeps drawing.
    ///
    /// A refusal is only the load's own: a read that fails says why in its own
    /// section ([`SettingsSnapshot`] carries each one's status) and leaves the
    /// rest of the object whole.
    pub async fn settings_snapshot(&self) -> Result<SettingsSnapshot, String> {
        self.accepting_work()?;
        let view = self.holder.view();
        let reads = Reads {
            files: Arc::clone(&self.settings.files),
            auth: self.settings.paths.auth.clone(),
            agent: self.settings.paths.agent.clone(),
            probe: Arc::clone(&self.settings.probe),
            versions: Arc::clone(&self.settings.versions),
            web: Arc::clone(&self.settings.web),
            node: Arc::clone(&self.settings.node),
            desk_version: view.app_version.clone(),
            desk_manifest: lock(&self.desk_endpoint).clone(),
            language: self.holder.language(),
        };
        let config = SettingsConfig {
            theme: view.theme,
            language: view.language,
            notifications: view.notifications,
            shell: view.shell,
            app_version: view.app_version,
            open_chats: view.open_chats,
            config_notice: view.config_notice,
        };
        tauri::async_runtime::spawn_blocking(move || settings_snapshot::gather(config, &reads))
            .await
            .map_err(|error| format!("the settings snapshot could not run: {error}"))
    }

    /// The Usage pane's read: where the shell's own usage sources stand, and -
    /// when they are installed - what the one document they print says.
    ///
    /// The locator and the run are both work: the answer to the click says the
    /// read is running, and the read's own thread pushes what it found, so the
    /// window keeps drawing while `node` works (up to the seam's 30 s budget).
    /// Nothing here looks at a chat, so no open chat can hold this read back,
    /// and nothing is written: the sources own their own credentials.
    pub fn refresh_usage(&self) -> Result<ViewState, String> {
        self.accepting_work()?;
        let emit = self.emitter();
        let view = self.holder.note_usage_reading();
        emit_view(&emit, &view)?;
        let files = Arc::clone(&self.settings.files);
        let node = Arc::clone(&self.settings.node);
        let home = self.settings.paths.agent.clone();
        let holder = Arc::clone(&self.holder);
        let thread_emit = Arc::clone(&emit);
        let spawned = std::thread::Builder::new()
            .name("usage-read".to_owned())
            .spawn(move || {
                let answered = settings::usage_view(
                    &files,
                    home.as_deref(),
                    &node,
                    gentle_runtime::clock::now_ms(),
                    holder.language(),
                );
                push(&thread_emit, holder.note_usage(answered));
            });
        if spawned.is_err() {
            // No thread to read on: the pane says the read could not run rather
            // than staying on "Reading...".
            let view = self.holder.note_usage(UsageView {
                status: UsageStatusView::Failed(
                    messages::usage_read_could_not_start(self.holder.language()).to_owned(),
                ),
                ..UsageView::default()
            });
            push(&emit, view);
        }
        Ok(view)
    }

    /// The check itself, on a thread of its own: the two versions the shell
    /// prints, the desk's own manifest and pi's registry entry, gathered outside
    /// every lock and pushed when they are all in.
    fn run_update_check(&self, emit: Emit) {
        let desk = self.holder.app_version();
        let manifest = lock(&self.desk_endpoint).clone();
        let web = Arc::clone(&self.settings.web);
        let versions = Arc::clone(&self.settings.versions);
        let holder = Arc::clone(&self.holder);
        let thread_emit = Arc::clone(&emit);
        let spawned = std::thread::Builder::new()
            .name("update-check".to_owned())
            .spawn(move || {
                let checked = settings::updates_view(
                    desk.as_deref(),
                    manifest.as_deref(),
                    &web,
                    &versions,
                    holder.language(),
                );
                push(&thread_emit, holder.note_updates(checked));
            });
        if spawned.is_err() {
            // No thread to check on: the version probe and the two GETs are not
            // moved onto the thread that draws, so the pane says the check ran
            // and learned nothing rather than staying on "checking".
            let view = self.holder.note_updates(UpdatesView {
                status: UpdatesStatusView::Ready,
                ..UpdatesView::default()
            });
            push(&emit, view);
        }
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

/// What follows the fetch: the notes to push, or this run's version stored and the
/// slice that carries no notes (S3, S5). `None` only for a run that never named
/// its version, which the step does before it fetches anything.
fn after_fetch(holder: &AppStateHolder, decision: Decision) -> Option<ViewState> {
    match decision {
        Decision::Show { version, notes } => Some(holder.note_whats_new(version, notes)),
        Decision::Store => holder.store_last_run_version(),
    }
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

/// Runs the per-chat commands one fold asked for, one worker thread per chat
/// command, because a child takes seconds to start and a write to it blocks.
/// Never waits for one: each outcome folds back through the reducer and pushes
/// the next `state-changed`.
///
/// This is the ONE runner of the per-chat commands (chat-event-commands, T1):
/// [`Bridge::run_chat_commands`] hands it the commands an intent's fold answered
/// with, and the sink [`Bridge::arm_chat_commands`] installs hands it the ones a
/// child event's fold asked for. The two paths differ in who asks, never in what
/// running a command means.
pub fn run_chat_commands(
    holder: &Arc<AppStateHolder>,
    executor: Arc<RuntimeExecutor>,
    emit: &Emit,
    commands: Vec<gentle_app::ChatCommand>,
) {
    for command in commands {
        let worker = Arc::clone(holder);
        let executor = Arc::clone(&executor);
        // The seam is `Send + Sync`, so a worker thread pushes with it and
        // no window handle is cloned anywhere.
        let emit_arc = Arc::clone(emit);
        // The worker's clone moves into the closure; the failure branch keeps
        // its own so a failed spawn still has an emit to push through.
        let emit = emit_arc.clone();
        // A thread that cannot be created never runs the command, and
        // `command` itself moves into the worker: the command names, below,
        // both the profile work slot it took (the one-in-flight cap, L18) and
        // the web answer it can no longer give.
        let unstarted = command.clone();
        // A session-mode command remembers which incarnation of its chat it
        // started in: a chat that closes and opens again is a new chat, and the
        // old child's answer may not be installed on it (composer-dock T3b, R3-001).
        let epoch = match &command {
            gentle_app::ChatCommand::SessionMode { chat, .. } => {
                Some((chat.clone(), holder.session_epoch(chat)))
            }
            _ => None,
        };
        let spawned = std::thread::Builder::new()
            .name("chat-command".to_owned())
            .spawn(move || {
                let mut ran = effects::run_one(command, executor.as_ref());
                // The session-mode answer is noted with the epoch it started in,
                // checked under the fold's own lock; the rest of the run folds
                // as every command's does.
                let answers = std::mem::take(&mut ran.session_mode);
                fold_and_push(&worker, &emit, ran);
                for (chat, change, outcome) in answers {
                    let started_in = epoch.as_ref().map(|(_, epoch)| *epoch);
                    if let Some(view) =
                        worker.note_session_mode_in(&chat, change, &outcome, started_in)
                    {
                        push(&emit, view);
                    }
                }
            });
        if let Err(error) = spawned {
            // A failed OS call is worth a log line, and neither side may be
            // left behind: the profile work slot the command took (the
            // one-in-flight cap, L18) is given back, and the web side gets the
            // failure it can draw ([`fold_spawn_failure`], L17(a), L18).
            if let Some(work) = ChatWork::of(&unstarted) {
                holder.note_chat_work_unstarted(unstarted.chat(), work);
            }
            eprintln!("chat command thread unavailable: {error}");
            fold_spawn_failure(holder, &emit_arc, &unstarted);
        }
    }
}

/// Folds what a chat command that could NOT be started leaves, and pushes it
/// (profiles-editor L17(a), L18).
///
/// A thread that cannot be created drops the command it was for, so the answer
/// can only be the fold's. The two commands the page is WAITING on answer it: a
/// read that never ran becomes the chat's unavailable view, which the tab draws
/// with its Try again, and a mode save that never ran becomes the end state of a
/// write that got no answer, which re-enables the switch and says the mode could
/// not be saved. Nothing else is drawn, no retry is made, and nothing is
/// started: the command's own chat may already be closed, where it folds
/// nothing at all.
pub fn fold_spawn_failure(
    holder: &Arc<AppStateHolder>,
    emit: &Emit,
    command: &gentle_app::ChatCommand,
) {
    let view = match command {
        gentle_app::ChatCommand::ReadChatProfile { chat } => holder.note_chat_profile_unread(chat),
        gentle_app::ChatCommand::SetChatProfileMode { chat, .. } => {
            holder.note_chat_mode_unstarted(chat)
        }
        // An apply that never started applied nothing: both buttons come back
        // and one short line says why (profiles-editor T3b, L17(a)).
        gentle_app::ChatCommand::ApplyChatProfile { chat, .. } => {
            holder.note_chat_apply_unstarted(chat)
        }
        // A chain write that never ran owes the page the same answer a write that
        // got no answer gives: nothing was written, the editor is usable again, and
        // one short line says so (profiles-editor T5).
        gentle_app::ChatCommand::SetChatProfileChain { chat, .. } => {
            holder.note_chat_chain_unstarted(chat)
        }
        // A pin write that never ran owes the page the same answer: nothing was
        // written, the picker is usable again, and one short line says so
        // (profiles-editor T6).
        gentle_app::ChatCommand::SetChatProfilePin { chat, .. } => {
            holder.note_chat_pin_unstarted(chat)
        }
        // A session mode that never reached a child owes the page the same answer
        // a write that got no answer gives: nothing was written, the mode stays,
        // and one short line names it. A READ that never ran says nothing at all
        // (composer-dock T3b, L7).
        gentle_app::ChatCommand::SessionMode { chat, change } => {
            holder.note_session_mode_unstarted(chat, *change)
        }
        // Every other command has already answered the web side with the slice
        // its intent folded, and none of them is a slice the failure changes.
        _ => None,
    };
    if let Some(view) = view {
        push(emit, view);
    }
}

/// Folds what one command came to and pushes each step, then arms the 20 s net
/// of a fresh child against the same state and the same emit seam.
fn fold_and_push(holder: &Arc<AppStateHolder>, emit: &Emit, ran: Ran) {
    for input in ran.inputs {
        push(emit, holder.fold_input(input));
    }
    // One chat's profile configuration is the shell's own view, noted on its
    // own (profiles-editor T2b-2, L11-1). Every read of the run is
    // noted, in the order it answered (T2b-3): the last one is the view the
    // shell holds. A read that reached no store, or whose chat closed while the
    // read was in flight, notes nothing and pushes nothing (L11-4).
    for (chat, read) in ran.chat_profile {
        if let Some(view) = holder.note_chat_profile(&chat, &read) {
            push(emit, view);
        }
    }
    // One profile's mode was saved, or it was not (profiles-editor T3-mode): the
    // answer enables the switch again, says whether the mode could not be saved,
    // and installs the configuration the attempt re-read.
    for (chat, result, read) in ran.chat_mode {
        if let Some(view) = holder.note_chat_mode(&chat, &result, &read) {
            push(emit, view);
        }
    }
    // One saved profile was applied, or the child is asking its own question, or
    // the answer wrote nothing and says why (profiles-editor T3b): the buttons
    // come back, the dialog is drawn verbatim, and an attempt that ran installs
    // the configuration it re-read.
    for (chat, attempt) in ran.chat_apply {
        if let Some(view) = holder.note_chat_apply(&chat, &attempt) {
            push(emit, view);
        }
    }
    // One fallback chain was saved, or it was not (profiles-editor T5): the same
    // answer as a mode save's, in the chain editor's own notice, with the
    // configuration the attempt re-read.
    for (chat, result, read) in ran.chat_chain {
        if let Some(view) = holder.note_chat_chain(&chat, &result, &read) {
            push(emit, view);
        }
    }
    // One pin layer was written, or it was not (profiles-editor T6): the same
    // answer as a chain save's, in the pin editor's own notice, with the
    // configuration the attempt re-read.
    for (chat, result, read) in ran.chat_pin {
        if let Some(view) = holder.note_chat_pin(&chat, &result, &read) {
            push(emit, view);
        }
    }
    // One chat's session permission mode was read, or one change of it was
    // answered (composer-dock T3b, L7): the label draws the owner's own string,
    // and an attempt that wrote nothing keeps the mode and says its code.
    for (chat, change, outcome) in ran.session_mode {
        if let Some(view) = holder.note_session_mode(&chat, change, &outcome) {
            push(emit, view);
        }
    }
    if let Some(chat) = ran.confirm_after {
        let holder = Arc::clone(holder);
        let emit = Arc::clone(emit);
        effects::opening_net(chat, move |input| {
            let view = holder.fold_input(input);
            let _ = emit(&view);
        });
    }
}

/// The push through the installed seam, as the result an intent path answers
/// with: a walk that cannot be shown is a failed walk.
fn emit_view(emit: &Emit, view: &ViewState) -> Result<(), String> {
    emit(view)
}

/// A push whose failure is only a log line: the web side still has the
/// command's answer, so there is nothing to fall back to.
fn push(emit: &Emit, view: ViewState) {
    if let Err(error) = emit(&view) {
        eprintln!("state-changed could not be emitted: {error}");
    }
}

/// The emit seam as the [`Push`] a pump or the session list arms: the same
/// closure at each place one is installed.
fn push_via(emit: Emit) -> Push {
    Box::new(move |view| push(&emit, view))
}

/// The one push of the bridge: the typed event the web side listens on, the
/// same payload type the command returns.
pub(crate) fn emit_state(app: &AppHandle<tauri::Wry>, view: &ViewState) -> Result<(), String> {
    StateChanged(view.clone())
        .emit(app)
        .map_err(|error| error.to_string())
}

/// The one push of the Settings files' watcher: the typed event the web side
/// listens on, carrying which groups changed and no path at all.
pub(crate) fn emit_settings_files_changed(
    app: &AppHandle<tauri::Wry>,
    changed: Changed,
) -> Result<(), String> {
    SettingsFilesChanged(SettingsFilesChangedView {
        profiles: changed.profiles,
        account: changed.account,
    })
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
    // Kept although the dispatch no longer needs it: the web side's binding for
    // this command is generated from what it serializes, and the handle is one
    // of Tauri's own arguments, never a payload field.
    _app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().dispatch(intent)
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
    _app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().attach_images(&paths)
}

/// Takes one attached image away before its send, named by the id
/// [`crate::contract::ImageView`] carried. Nothing is sent.
#[tauri::command]
#[specta::specta]
pub fn remove_image(
    id: String,
    bridge: tauri::State<'_, Bridge>,
    _app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().remove_image(&id)
}

/// The files the human picked for this message (design/composer-dock T4, S3):
/// the clip's one command. It answers the chosen paths as strings, and the empty
/// list for a cancelled dialog, so a cancel changes nothing on the page.
///
/// The dialog can stay open for minutes, so it runs off the window's thread and
/// under no lock, exactly as the folder picker does. The title is the web side's
/// own word for the dialog ([`Bridge`] carries no words), in the language the
/// composer is drawn in, so one dictionary names both the button and its dialog.
#[tauri::command]
#[specta::specta]
pub async fn pick_files(
    title: String,
    bridge: tauri::State<'_, Bridge>,
) -> Result<Vec<String>, String> {
    let picker = bridge.inner().file_picker();
    tauri::async_runtime::spawn_blocking(move || crate::attachments::pick(picker.as_ref(), &title))
        .await
        .map_err(|error| format!("the file dialog could not run: {error}"))
}

/// What each path the user chose or dropped is (design/composer-dock T4, R5,
/// R6): an image still travels as an image, every other file becomes a chip that
/// carries its path, and one that cannot be attached answers why.
///
/// `folder` is the foreground chat's working directory, which is what decides
/// `outside`. Nothing here sends, writes or opens a dialog: it reads the
/// metadata of the named files once and answers, so it needs no holder and no
/// lock. It runs off the window's thread because it touches the disk (a path on a
/// network drive can take a while), like every other read here.
#[tauri::command]
#[specta::specta]
pub async fn describe_attachments(
    paths: Vec<String>,
    folder: Option<String>,
) -> Vec<AttachmentView> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::attachments::describe(&paths, folder.as_deref())
    })
    .await
    .unwrap_or_default()
}

/// The dismiss of the what's-new modal (what's-new S5): store the version this run
/// runs as and clear the notes it pushed. The version is the run's own, never the
/// manifest's. It takes no argument: the page says it put the modal away, and the
/// shell is the one that knows which version it is.
#[tauri::command]
#[specta::specta]
pub fn store_last_run_version(
    bridge: tauri::State<'_, Bridge>,
    _app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.inner().store_last_run_version()
}

/// The Settings dialog's one read (settings-snapshot): every page's data in
/// one plain object, asked once when the dialog opens.
///
/// It takes no argument and needs no window: the gather's seams and the half of
/// the slice it carries all come from the bridge, and the answer is the same
/// shape the web side draws. Nothing of a chat travels through it (a chat's
/// profiles, session mode and images are read through that chat's own child), so
/// opening Settings never waits on a child and never starts one.
#[tauri::command]
#[specta::specta]
pub async fn settings_snapshot(
    bridge: tauri::State<'_, Bridge>,
) -> Result<SettingsSnapshot, String> {
    bridge.inner().settings_snapshot().await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gentle_app::{ChatId, Input, Intent};
    use gentle_core::rpc::config_store::{self, StoreId, StoreValue};
    use gentle_runtime::chat_runtime::{ChatConfigRead, StoreRead};

    use super::{fold_and_push, Emit, Ran};
    use crate::bridge::AppStateHolder;
    use crate::contract::ViewState;

    /// One chat's profile read, as the channel's own parser answers it: the
    /// profiles store alone is all a fold test needs to know it reached a store.
    fn read() -> ChatConfigRead {
        let value = serde_json::json!({
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "profiles": {"team": {"worker": {"model": "p/m"}}},
        });
        let parsed =
            config_store::parse_store_value(StoreId::Profiles, &value).expect("a profile document");
        ChatConfigRead {
            profiles: StoreRead::Read(match parsed {
                StoreValue::Profiles(document) => document,
                _ => panic!("the profiles store answered"),
            }),
            ..ChatConfigRead::unasked()
        }
    }

    /// `chat` opened and confirmed: the app's open set holds it, and it is in
    /// front.
    fn opened(holder: &AppStateHolder, chat: &ChatId) {
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
        holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    }

    /// One slice recorder: the pushes of a step, in order.
    fn recorder() -> (Emit, Arc<Mutex<Vec<ViewState>>>) {
        let pushed: Arc<Mutex<Vec<ViewState>>> = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&pushed);
        let emit: Emit = Arc::new(move |view: &ViewState| {
            kept.lock().expect("pushed mutex").push(view.clone());
            Ok(())
        });
        (emit, pushed)
    }

    /// The finding R4-late-read-reinstalls-view's first order: the SAME run
    /// folds the chat's close and carries the read that answered while it was
    /// closing. The user's close already took the chat out of the open set
    /// (`Intent::CloseChat`), the host's expected `Closed` arrives in this run,
    /// and `fold_and_push` folds the inputs before it notes the read.
    #[test]
    fn a_read_that_arrives_with_the_close_of_the_same_run_installs_nothing() {
        let holder = Arc::new(AppStateHolder::default());
        let chat = ChatId::of_session("/sessions/a.jsonl");
        opened(&holder, &chat);
        holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
        let (emit, pushed) = recorder();

        fold_and_push(
            &holder,
            &emit,
            Ran {
                inputs: vec![Input::ChatClosed {
                    chat: chat.clone(),
                    expected: true,
                }],
                chat_profile: vec![(chat.clone(), read())],
                ..Ran::default()
            },
        );

        assert_eq!(
            pushed.lock().expect("pushed mutex").len(),
            1,
            "only the close's own slice is pushed"
        );
        // Reopening the same id is how the stale view would reappear: the row
        // opens the chat again and nothing reads it yet.
        holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
        assert!(
            holder.view().chat_profile.is_none(),
            "the late read installed no view for the closed chat"
        );
    }

    /// The finding R4-ran-single-slot-drops-read: `Ran` carries every read one
    /// run answered, and the fold notes them in order instead of keeping only
    /// the last.
    #[test]
    fn two_reads_of_one_run_are_noted_in_order() {
        let holder = Arc::new(AppStateHolder::default());
        let first = ChatId::of_session("/sessions/a.jsonl");
        let second = ChatId::of_session("/sessions/b.jsonl");
        opened(&holder, &first);
        opened(&holder, &second);
        // The first chat is in front, so its own read is the one the slice
        // carries; the second is open behind it.
        holder.fold_input(Input::Intent(Intent::OpenChat(first.clone())));
        let (emit, pushed) = recorder();

        fold_and_push(
            &holder,
            &emit,
            Ran {
                chat_profile: vec![(first.clone(), read()), (second.clone(), read())],
                ..Ran::default()
            },
        );

        let slices = pushed.lock().expect("pushed mutex").clone();
        assert_eq!(
            slices.len(),
            2,
            "each read of the run is noted and pushes its own slice"
        );
        assert_eq!(
            slices[0]
                .chat_profile
                .as_ref()
                .map(|view| view.chat_id.clone()),
            Some(first.as_str().to_owned()),
            "the first read of the run lands first"
        );
        // The second read landed after it: it is the view the shell holds, and
        // seeing it needs only the chat it belongs to in front.
        holder.fold_input(Input::Intent(Intent::OpenChat(second.clone())));
        assert_eq!(
            holder.view().chat_profile.map(|view| view.chat_id),
            Some(second.as_str().to_owned()),
            "the second read of the same run was not dropped"
        );
    }
}
