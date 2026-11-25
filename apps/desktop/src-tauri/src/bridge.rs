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
//!
//! A fold has exactly one delivery, and which one it is depends on the entry
//! point that took it: [`AppStateHolder::dispatch_commands`] ANSWERS its
//! per-chat commands to its caller (the intent path runs them), and
//! [`AppStateHolder::deliver_chat_commands`] HANDS the ones a child event's fold
//! asked for to the [`ChatCommandSink`] the bridge armed, off the pump's thread.
//! Nothing both answers and delivers, so nothing is run twice.
//!
//! The config snapshot and everything that writes it live in the `config`
//! module below: one atomic edit, one version per snapshot, one writer.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{Command, SessionItem};
use gentle_core::i18n::Language;
use gentle_runtime::app_config::{self, folder_key};
use gentle_runtime::clock::now_ms;

use crate::config::{ConfigSnapshot, ConfigWriter};
use crate::contract::{
    AuthView, BranchView, ChatProfileView, SessionsWindow, UiIntent, UpdatesView, UsageView,
    ViewState,
};
use crate::images::Images;
use crate::sessions::cache::TreeCache;
use crate::sessions::revision;
use crate::sessions::tree::Tree;
use crate::sessions::window::{self, folder_status};
use crate::state_view::{view_state, ShellParts};
use crate::theme::WindowTheme;
use crate::workspace;

mod chat_profile;

pub use chat_profile::ChatWork;
pub use transcript::{SessionRead, SessionRows, TranscriptPush};
mod config;
mod fatal;
mod images;
mod language;
mod notice;
mod notifications;
mod session_mode;
mod session_notice;
mod settings;
mod shell;
mod theme;
mod transcript;
mod welcome;
mod whats_new;

/// What runs one shared command the reducer asked for. `Command::RefreshSessions`
/// is the session list's in the app; the sink keeps the holder free of how it is
/// run (a thread, a service, nothing at all before it is installed).
pub type CommandSink = Arc<dyn Fn(Command) + Send + Sync>;

/// What runs the per-chat commands ONE CHILD EVENT'S fold asked for
/// (chat-event-commands, S1-S4). The child-event pump is the only fold that
/// hands them over, because it is the only one whose caller owns no executor.
///
/// It is called ON the folding thread - the pump behind every chat's events -
/// so it must never block and must never write to a child's pipe (S3): the sink
/// the bridge installs starts one `chat-command` worker thread per command,
/// exactly as the intent path does with the commands it is answered.
pub type ChatCommandSink = Arc<dyn Fn(Vec<gentle_app::ChatCommand>) + Send + Sync>;

/// Installed once at startup: what pushes the slice a branch read answered on a
/// worker thread ([`AppStateHolder::arm_branch_push`]). Before it exists the
/// answer is still cached, and the next fold pushes it.
pub type BranchPush = Arc<dyn Fn() + Send + Sync>;

/// Installed once at startup: what runs the explicit extensions check once per
/// app run (packages-status T4, S7), when the FIRST chat's child is ready
/// ([`AppStateHolder::note_child_ready`]). It starts its own thread and returns:
/// it is called while a child event is being folded, so it must never block, and
/// nothing runs while no chat has a child (which is the whole reason the trigger
/// is a child's own event and never a launch step).
pub type PackagesTrigger = Arc<dyn Fn() + Send + Sync>;

/// The app state the Tauri shell owns. Created at startup, managed by Tauri;
/// nothing else anywhere holds an `AppState`.
#[derive(Default)]
pub struct AppStateHolder {
    app: Mutex<gentle_app::AppState>,
    /// Installed once at startup: before it exists a shared command is simply
    /// dropped, which is what the earlier slices did with all of them.
    commands: Mutex<Option<CommandSink>>,
    /// Installed once at startup: what runs the per-chat commands a CHILD
    /// EVENT's fold asked for. Before it exists such a fold's commands are
    /// dropped, which is what a run with no window does.
    chat_commands: Mutex<Option<ChatCommandSink>>,
    /// Bumped whenever the folded list, its status or the config changes, and
    /// carried by every slice as `ViewState::sessions_revision`. It wraps: the
    /// web side compares two revisions for inequality, so a wrap names no old
    /// list.
    sessions_revision: AtomicU32,
    /// The app config of this run: one snapshot, loaded at startup outside
    /// every lock and replaced as a whole by `config::update_config` or
    /// `config::replace_config`.
    config: Mutex<Arc<ConfigSnapshot>>,
    /// The number of the snapshot `config` holds, bumped under the same lock as
    /// the snapshot itself: the writer keeps the file ordered by it, so a caller
    /// that is descheduled between its edit and its save can never rewrite an
    /// older state.
    config_version: AtomicU64,
    /// Installed once at startup: the one writer of the config file. Before it
    /// exists a config change is kept in memory and nothing is written, which is
    /// what a run with no config path does for its whole life.
    saver: Mutex<Option<ConfigWriter>>,
    /// Installed once at startup: what makes the window's own chrome (the title
    /// bar) follow the chosen theme. Before it exists the theme is still the
    /// user's, and the page is colored by it either way.
    window_theme: Mutex<Option<Arc<dyn WindowTheme>>>,
    /// The folder of the user's pi when it was found at startup (`bridge::shell`).
    pi_path: Mutex<Option<String>>,
    /// Set when the launcher was looked for and not found (`bridge::shell`); a
    /// run that never looked has it unset, so it never claims one is missing.
    launcher_missing: std::sync::atomic::AtomicBool,
    /// Where installing gentle-shell stands (`bridge::shell`).
    install: Mutex<crate::contract::InstallView>,
    /// The home choice this run started with (`bridge::shell`).
    started_with: Mutex<Option<gentle_backend::home::HomeChoice>>,
    /// The flattened tree of the last built revision, so a page of rows never
    /// regroups 50 000 sessions again (see [`TreeCache`]).
    tree: TreeCache,
    /// What the last workspace action refused, as `ViewState.workspace_notice`
    /// carries it: a shell-owned line, because no reducer path records one
    /// (`bridge::notice`).
    workspace_notice: Mutex<Option<String>>,
    /// What the last session action refused (a rename), as
    /// `ViewState.sessions_notice` carries it: the same kind of shell-owned line
    /// (`bridge::session_notice`).
    sessions_notice: Mutex<Option<String>>,
    /// The notes of the version this run follows, or the notes the About pane
    /// read on demand, as `ViewState.whats_new` carries them: pushed by the
    /// what's-new step of the run and cleared by the dismiss that stores the
    /// version, or by the dismiss of a read on demand, which stores nothing
    /// (`bridge::whats_new`).
    whats_new: Mutex<Option<whats_new::Notes>>,
    /// The version this run runs as, `None` until the what's-new step names it:
    /// what the dismiss stores, never the manifest's version (S5), and what
    /// every slice carries as `ViewState.app_version` for the Settings
    /// dialog's About section (settings-dialog S5).
    app_version: Mutex<Option<String>>,
    /// The images each chat holds: attached in the composer, read here, taken by
    /// that chat's send and served back as previews (`bridge::images`).
    images: Images,
    /// The Profile tab's view of one chat (profiles-editor T2b-2, L11): what that
    /// chat's own child answered and which profile its session file binds. It is
    /// that chat's and only that chat's, and it is dropped when the chat closes
    /// (`bridge::chat_profile`).
    chat_profile: Mutex<Option<ChatProfileView>>,
    /// The session-file seam the Profile tab's binding half is read through
    /// (L9): the run's own reader and the sessions store a chat's path must stay
    /// inside, armed together at startup like the command sink. Until it is
    /// armed no session file is read, so a view says the chat uses the default -
    /// which is what a chat with no file says too.
    session_files: Mutex<Option<chat_profile::SessionFiles>>,
    /// Each chat's profile WORK (profiles-editor T3-mode, L18), one entry per
    /// chat: whether a read of that chat's configuration is in flight, whether a
    /// mode save is (the switch is disabled while it is: the slice carries only
    /// that one), and whether the last save failed. Shell-owned, and dropped with
    /// the chat. The two slots are what the one-in-flight cap is made of
    /// ([`AppStateHolder::dispatch_commands`],
    /// [`AppStateHolder::deliver_chat_commands`]).
    profile_work: Mutex<std::collections::HashMap<String, chat_profile::ProfileWork>>,
    /// Each chat's SESSION permission mode (composer-dock T3b, L7), one entry per
    /// chat that answered: the mode its own child last reported, the refusal of
    /// the last attempt that wrote nothing, and how many writes it answered.
    /// Shell-owned, and dropped with the chat (`bridge::session_mode`).
    session_modes: Mutex<session_mode::SessionModes>,
    /// The folder a chat was opened in before it has a session row (composer-dock
    /// L12): a NEW chat in a workspace lives in the workspace's folder from the
    /// start, so its chip names it until the session row takes over. Shell-owned,
    /// and dropped with the chat.
    folder_hints: Mutex<std::collections::BTreeMap<String, crate::contract::FolderView>>,
    /// How many times each chat's session mode was forgotten, so a command that
    /// started before a close and answers after a reopen is told apart from one
    /// of the new chat (`bridge::session_mode`).
    session_epochs: Mutex<std::collections::BTreeMap<String, u64>>,
    /// The rows read from each stored chat's session FILE, painted while that
    /// chat's child is still starting and dropped the moment the child's answer
    /// is folded (`bridge::transcript`, chat-open-latency T1). Shell-owned, like
    /// the session mode: the reducer's transcript is the authority and this is
    /// the frame that fills the seconds before it arrives.
    transcripts: Mutex<transcript::Transcripts>,
    /// Installed once at startup: what pushes the answer of a file read that
    /// ran on its own thread ([`AppStateHolder::arm_transcript_push`]).
    transcript_push: Mutex<Option<TranscriptPush>>,
    /// The Account pane's slice (0.1.12, T3): the provider names the shell's
    /// store lists and what its own check said about each.
    auth: Mutex<AuthView>,
    /// The Updates pane's slice (0.1.12, S2, S7): the last check's answer.
    updates: Mutex<UpdatesView>,
    /// The Extensions block the explicit check HELD (packages-status T4, S7): what
    /// `check_packages` made pi answer, kept so the pushed slice
    /// (`ViewState.updates.packages`) and the gear's dot carry it without Settings
    /// being open, and so the dialog's own load shows the checked answer instead
    /// of re-deriving a "not checked yet" from a plain read. `None` until the
    /// first check answers.
    packages: Mutex<Option<crate::contract::PackagesView>>,
    /// Installed once at startup: what starts the once-per-run extensions check
    /// when the first chat's child is ready ([`Self::note_child_ready`]).
    packages_trigger: Mutex<Option<PackagesTrigger>>,
    /// Set the first time a child's own event proves a chat is alive, so the
    /// explicit check runs once per app run and no later event starts it again.
    /// A check that failed or timed out does NOT retry on its own: the user's
    /// "Comprobar de nuevo" is what runs it again (S7).
    packages_check_started: std::sync::atomic::AtomicBool,
    /// The Usage pane's slice: the subscription usage the shell's own sources
    /// reported when the pane last asked, and whether that read is running.
    usage: Mutex<UsageView>,
    /// The run's one fatal failure, as `ViewState.fatal` carries it: the
    /// launcher could not be resolved, so no chat can start (`bridge::fatal`).
    fatal: Mutex<Option<String>>,
    /// The branch each working directory was on, read once per directory off the
    /// window's thread and reused by every push after it (`crate::git`), with
    /// the push a late answer travels on.
    branches: Arc<crate::git::Branches>,
    branch_push: Mutex<Option<BranchPush>>,
}

impl AppStateHolder {
    /// A holder for a run whose config file was read at startup, before any
    /// lock existed.
    pub fn with_config(config: ConfigSnapshot) -> Self {
        let language = config.config.language;
        let holder = Self {
            config: Mutex::new(Arc::new(config)),
            ..Self::default()
        };
        // The reducer words its own notices, so it starts the run in the
        // language the config holds: `None` is English (D2).
        lock(&holder.app).language = language;
        // The working cap is the host's, dispatched once here so the reducer
        // never enforces a number of its own: over it a prompt waits, it is
        // never refused.
        holder.fold_input(gentle_app::Input::SetWorkingCap(
            gentle_backend::chat_host::Limits::default().max_working,
        ));
        holder
    }

    /// Installs what runs the shared commands of every later fold. One sink for
    /// the whole holder, so the intent path, the child-event pump and a refresh
    /// pass all reach the same service.
    pub fn arm_commands(&self, sink: CommandSink) {
        *lock(&self.commands) = Some(sink);
    }

    /// Installs what runs the per-chat commands of a child event's fold. Armed
    /// in the same startup step that arms the pump's push, because the pump is
    /// what folds a child event and hands its commands over
    /// ([`Self::deliver_chat_commands`]).
    pub fn arm_chat_commands(&self, sink: ChatCommandSink) {
        *lock(&self.chat_commands) = Some(sink);
    }

    /// Installs what pushes the slice a branch read answered (design/composer-dock
    /// S2, R3, R4). The read runs on its own thread so no push ever waits for a
    /// `git` process; this is how what it found reaches the window.
    pub fn arm_branch_push(&self, push: BranchPush) {
        *lock(&self.branch_push) = Some(push);
    }

    /// Installs what runs the once-per-run extensions check (packages-status T4,
    /// S7). Armed with the window's emit step: a run with no window would start a
    /// check nothing can show.
    pub fn arm_packages_trigger(&self, trigger: PackagesTrigger) {
        *lock(&self.packages_trigger) = Some(trigger);
    }

    /// The first chat's child is ready: a chat's own decoded event arrived, which
    /// is the app's proof that some child is alive and answerable.
    ///
    /// The trigger runs ONCE per app run (the flag is taken before it is called),
    /// so a second chat, a second event or a re-opened chat never asks again. It
    /// never runs at launch and never starts a child: this is only ever reached
    /// from an event that child already sent. A failed check is not retried here -
    /// only the user's own "Comprobar de nuevo" asks again (S7).
    pub fn note_child_ready(&self) {
        if self.packages_check_started.load(Ordering::SeqCst) {
            return;
        }
        let trigger = lock(&self.packages_trigger).clone();
        let Some(trigger) = trigger else {
            return;
        };
        if self.packages_check_started.swap(true, Ordering::SeqCst) {
            return;
        }
        trigger();
    }

    /// The branch cache, for a caller outside a fold: the tests seed it so an
    /// answer can be asserted without a repository or a process.
    pub fn branches(&self) -> &crate::git::Branches {
        &self.branches
    }

    /// Records the folder the workspace handed a NEW chat (composer-dock L12):
    /// the hint is what its folder chip names until its own session row exists,
    /// and answers `true` when the chat the answered slice names took it.
    ///
    /// The chat is the SLICE's foreground one and no longer a command: since
    /// welcome-screen T4 (R2) a new chat asks for no child when it is allocated,
    /// so the fold that opens it answers no `OpenProcess` to read the chat off,
    /// and the slice it answers is where that chat is named.
    fn note_folder_hint(&self, folder: &str, chat: Option<&str>) -> bool {
        let Some(chat) = chat else {
            return false;
        };
        let name = folder
            .rsplit(['\\', '/'])
            .find(|segment| !segment.is_empty())
            .unwrap_or(folder)
            .to_owned();
        let view = crate::contract::FolderView {
            name,
            path: folder.to_owned(),
        };
        lock(&self.folder_hints).insert(chat.to_owned(), view);
        true
    }

    /// The folder hint the FOREGROUND chat carries, `None` when it has none:
    /// a hint is only a stand-in for the chat's own session row.
    fn folder_hint_for(&self, app: &gentle_app::AppState) -> Option<crate::contract::FolderView> {
        let chat = app.chats.foreground()?;
        lock(&self.folder_hints).get(chat.as_str()).cloned()
    }

    /// The folder hint as the slice's shell part carries it: the chat it belongs
    /// to, and the view. The foreground gate is `view_state`'s.
    fn folder_hint(&self) -> Option<(String, crate::contract::FolderView)> {
        let chat = {
            let app = lock(&self.app);
            app.chats.foreground()?.as_str().to_owned()
        };
        let hint = lock(&self.folder_hints).get(&chat)?.clone();
        Some((chat, hint))
    }

    /// Drops a chat's folder hint with the chat: its session row, or the next
    /// chat, names the folder from now on.
    fn forget_folder_hint(&self, chat: &gentle_app::ChatId) {
        lock(&self.folder_hints).remove(chat.as_str());
    }

    /// Asks the branch question again for the foreground chat's folder, and
    /// answers whether the cache really held an answer to drop (composer-dock
    /// L12). The next slice starts the fresh read, which pushes on its own.
    fn refresh_branch(&self) -> bool {
        let folder = {
            let app = lock(&self.app);
            crate::state_view::foreground_folder(&app)
                .or_else(|| self.folder_hint_for(&app))
                .map(|folder| folder.path)
        };
        let Some(folder) = folder else {
            return false;
        };
        self.branches.forget(&folder);
        true
    }

    /// Folds one intent of the contract through the reducer and answers the
    /// slice to push. Also the path the `state-changed` event payload takes.
    /// Folds one intent of the contract through the reducer and answers the
    /// slice to push. Also the path the `state-changed` event payload takes.
    pub fn dispatch(&self, intent: UiIntent) -> ViewState {
        self.dispatch_commands(intent).0
    }

    /// [`Self::dispatch`] with the per-chat commands the fold asked for, for the
    /// caller to run through an executor (`effects`). The web sees the same
    /// slice either way: the commands are the shell's business, not the view's.
    pub fn dispatch_commands(&self, intent: UiIntent) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        // The intents the reducer does not fold, answered first: folding a
        // workspace and choosing the theme are the config's, and a workspace
        // that names no folder starts nothing at all.
        if let Some(view) = self.config_intent(&intent) {
            return (view, Vec::new());
        }
        let state = self.state();
        // The Profile tab's read (profiles-editor T2b-2, L11) is the shell's, and
        // it is a per-chat command: ONE chat's own child is where that chat's
        // profiles and its worktree's pin live. The answer is the slice as it
        // stands, and the read pushes what it found when it lands (`bridge::chat_profile`).
        //
        // At most ONE read per chat is in flight (L18): an ask that arrives while
        // one runs is DROPPED here - answered with the slice as it stands and no
        // command - because the tab's own ask is already coalesced and the point
        // of the cap is to leave that chat's child free (the measured storm was
        // 4,570 asks in 25 s and ~4,300 worker threads).
        if let UiIntent::RefreshChatProfile { chat_id } = &intent {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::ReadChatProfile { chat };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // Saving one profile's mode (profiles-editor T3-mode, S2, S3) is the
        // same kind of per-chat command: the chat's own child holds the
        // document, the token and the worktree that a write needs. The write
        // runs on a worker thread, so the slice this answers with already says
        // `saving`, which is what disables the switch until the answer lands.
        //
        // One save per chat too (L18): a second SaveChatProfileMode while one is
        // in flight answers the slice as it stands and writes nothing, which is
        // also what the review asked for (R1-002/R3-002).
        if let UiIntent::SaveChatProfileMode {
            chat_id,
            profile,
            mode,
        } = &intent
        {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::SetChatProfileMode {
                chat,
                profile: profile.clone(),
                mode: crate::intent::mode_of(*mode),
            };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // Applying a profile (profiles-editor T3b, S4, S8, S9) is the same kind
        // of per-chat command: the chat's own child is the only one that can bind
        // the profile to THIS chat (`session`) or run the channel's global apply
        // (`default`, which may answer its own confirmation first). The command
        // takes the chat's apply slot, so one apply at a time is in flight - the
        // same cap the read and the mode save keep (L18, L29) - and the slice this
        // answers with already says `applying`, which is what disables the buttons.
        if let UiIntent::ApplyChatProfile {
            chat_id,
            profile,
            mode,
            confirmation,
        } = &intent
        {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::ApplyChatProfile {
                chat,
                profile: profile.clone(),
                mode: crate::intent::apply_mode_of(*mode),
                confirmation: crate::intent::token_of(confirmation.as_deref()),
            };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // Saving ONE fallback chain of the used profile (profiles-editor T5,
        // L30-3) is the same kind of per-chat command and takes the SAME save slot
        // (L18): a chain save and a mode save of one chat can never be in flight
        // together, and a second ask is dropped with the slice as it stands. The
        // chain is the page's own draft, converted to the store's own step shape
        // here: `null` thinking is no effort, never an empty string the owner
        // would refuse.
        if let UiIntent::SaveChatProfileChain {
            chat_id,
            profile,
            agent,
            steps,
        } = &intent
        {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::SetChatProfileChain {
                chat,
                profile: profile.clone(),
                agent: agent.clone(),
                steps: crate::intent::chain_steps(steps),
            };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // Writing ONE pin layer of the chat's worktree (profiles-editor T6, S2,
        // S3) is the same kind of per-chat command and takes the SAME save slot
        // (L18): a pin save, a chain save and a mode save of one chat can never be
        // in flight together, and a second ask is dropped with the slice as it
        // stands. A layer string that names neither `local` nor `repo` is a page
        // bug: no command is built for it at all.
        if let UiIntent::SaveChatProfilePin {
            chat_id,
            source,
            profile,
        } = &intent
        {
            let Some(source) = crate::intent::pin_source_of(source) else {
                return (self.view(), Vec::new());
            };
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::SetChatProfilePin {
                chat,
                source,
                profile: profile.clone(),
            };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // The session permission mode (composer-dock T3b, L7) is the same kind of
        // per-chat command: the mode belongs to the child of the chat it names,
        // so it is asked of that chat's own child and no other. One read draws no
        // control until it answers, and one write's shown mode changes only on
        // the confirmed answer the run folds back.
        //
        // One session-mode command per chat is in flight at most (the same cap
        // the profile commands keep): a second ask is DROPPED with the slice as
        // it stands rather than queued against that chat's child.
        if let UiIntent::ReadSessionMode { chat_id } = &intent {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::SessionMode { chat, change: None };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        // A change of that mode is the same command with the mode in it: the
        // closed choice of the contract maps to the channel's own wire string,
        // and the saved default is never touched because `persist` is never sent
        // (L7).
        if let UiIntent::SetSessionMode { chat_id, mode } = &intent {
            let chat = crate::intent::chat_for(&state, chat_id);
            let command = gentle_app::ChatCommand::SessionMode {
                chat,
                change: Some(crate::intent::session_mode_of(*mode)),
            };
            if !self.begin_chat_command(&command) {
                return (self.view(), Vec::new());
            }
            return (self.view(), vec![command]);
        }
        match crate::intent::to_intent(&state, &intent) {
            Some(reduced) => {
                let opened = opens_a_chat(&reduced);
                // A NEW chat in a workspace lives in that folder from the start
                // (composer-dock L12): the hint is what its folder chip names
                // until its own session row exists, and what the branch is read
                // from. Recorded only for the chat the fold opens, never one
                // more.
                let hint = match (&intent, &reduced) {
                    (UiIntent::NewChatInWorkspace { path }, gentle_app::Intent::OpenNewChat)
                        if !path.trim().is_empty() =>
                    {
                        Some(path.clone())
                    }
                    _ => None,
                };
                let answer = self.apply(gentle_app::Input::Intent(reduced));
                let hinted = hint.map(|folder| {
                    self.note_folder_hint(&folder, answer.0.foreground_chat.as_deref())
                });
                // Opening a stored chat paints its thread from its session file
                // while its child starts (chat-open-latency T1): the read is
                // claimed here and run on a worker, because this path is
                // synchronous and a file is the last thing it may wait for.
                if opened {
                    self.start_transcript_read();
                }
                // An open also asks the branch question again (composer-dock
                // L12): the answer the cache holds may name a branch the user
                // changed outside this app, and opening the chat is when the
                // user looks.
                let refreshed = opened && self.refresh_branch();
                if hinted == Some(true) || refreshed {
                    // The hint or the fresh read change what the slice draws,
                    // so the slice this intent answers is built AFTER them.
                    (self.view(), answer.1)
                } else {
                    answer
                }
            }
            // Every intent has an answer: the reducer's, or the config's above.
            None => (self.view(), Vec::new()),
        }
    }

    /// Folds one reducer input through the same path, answering the slice only.
    /// This is the slot the session passes fold through
    /// (`Input::SessionsProgress`) and what the shell's own code seeds state
    /// with, so it runs nothing: the per-chat commands the fold asked for are
    /// dropped here. [`Self::fold_event`] folds the pump's events and pairs with
    /// [`Self::deliver_chat_commands`], which runs them.
    pub fn fold_input(&self, input: gentle_app::Input) -> ViewState {
        self.apply(input).0
    }

    /// Folds ONE CHILD EVENT through the same reducer and answers the slice to
    /// push together with the per-chat commands the fold asked for
    /// (chat-event-commands, S1-S4). The child-event pump is the only caller:
    /// it pushes the slice and then hands the commands to
    /// [`Self::deliver_chat_commands`], which is the order the intent path keeps
    /// (show, then run).
    ///
    /// The delivery of those commands is that one pair's and no other entry
    /// point's: [`Self::fold_input`] runs nothing and [`Self::dispatch_commands`]
    /// answers its commands to its caller, which runs them for the intent path.
    /// One fold therefore has one runner by construction - never both - so no
    /// command can run twice (S4).
    pub fn fold_event(
        &self,
        input: gentle_app::Input,
    ) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        self.apply(input)
    }

    /// Hands the per-chat commands of one child event's fold to the armed sink,
    /// if any. Called on the folding thread and never under a state lock: the
    /// sink starts a worker per command and returns, so this thread is never the
    /// one waiting on a child's pipe (S3).
    pub fn deliver_chat_commands(&self, commands: Vec<gentle_app::ChatCommand>) {
        // The same one-in-flight cap the intent path keeps (L18): a profile
        // command whose work is already in flight for its chat is DROPPED here
        // too, so a child-event fold cannot queue a second read behind the first
        // and nothing that holds no slot is ever filtered out.
        let commands: Vec<gentle_app::ChatCommand> = commands
            .into_iter()
            .filter(|command| self.begin_chat_command(command))
            .collect();
        if commands.is_empty() {
            return;
        }
        let sink = lock(&self.chat_commands).clone();
        if let Some(sink) = sink {
            sink(commands);
        }
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
    ///
    /// The two shell-owned notice lines are read before the state lock, as the
    /// config notice is, so no path holds the state and a notice at once.
    pub fn view(&self) -> ViewState {
        let shell = self.shell_parts();
        view_state(&self.state(), self.sessions_revision(), shell)
    }

    /// The activity the reducer holds for the chat stored at `session` (S4): the
    /// chat of a session file it has never heard of is idle. Read by the rename
    /// command, which must not append to a file whose child is mid-turn
    /// (`chat_rename`).
    pub fn session_activity(&self, session: &str) -> gentle_app::ChatActivity {
        lock(&self.app).activity_of_session(session)
    }

    /// The session list as the state holds it right now. Read by the delete
    /// command (`chat_delete`): deciding whether the folder of the deleted chat
    /// was the last chat of a workspace the config does not name needs the whole
    /// list (T4).
    pub fn listed_sessions(&self) -> Vec<SessionItem> {
        self.sessions_snapshot()
    }

    /// The session file of the chat the app holds as the foreground one: the
    /// current workspace is the one that file lives in (T4, D3(a)). `None` for
    /// no foreground chat, and for a chat that has no session file yet.
    pub fn foreground_session(&self) -> Option<String> {
        lock(&self.app)
            .chats
            .foreground()
            .and_then(|chat| chat.session_path().map(str::to_owned))
    }

    /// The branch the foreground chat's folder is on (design/composer-dock S2,
    /// R3, R4), from the cache the one read left.
    ///
    /// It never runs `git` on the caller's thread: a directory nobody has read
    /// yet claims its one read here and a worker thread runs it, so this answers
    /// `None` for exactly one push and the read's answer arrives on its own
    /// ([`Self::arm_branch_push`]).
    fn branch_view(&self) -> Option<BranchView> {
        let cwd = {
            // The directory comes from the same session row the folder chip
            // reads - or, for a chat opened before its session existed, from
            // the workspace folder hint (composer-dock L12) - under one short
            // lock: the read itself happens with none.
            let app = lock(&self.app);
            crate::state_view::foreground_folder(&app)
                .or_else(|| self.folder_hint_for(&app))
                .map(|folder| folder.path)
        }?;
        match self.branches.lookup(&cwd) {
            crate::git::Lookup::Read(branch) => branch,
            crate::git::Lookup::Waiting => None,
            crate::git::Lookup::Missing => {
                self.start_branch_read(cwd);
                None
            }
        }
    }

    /// Starts the ONE read of a working directory's branch, off this thread and
    /// with no lock held: a `git` process is the last thing a push should wait
    /// for. What it answers is cached and pushed.
    fn start_branch_read(&self, cwd: String) {
        let branches = Arc::clone(&self.branches);
        let push = lock(&self.branch_push).clone();
        std::thread::spawn(move || {
            let branch = crate::git::read(std::path::Path::new(&cwd));
            branches.record(&cwd, branch);
            if let Some(push) = push {
                push();
            }
        });
    }

    /// The label the sidebar draws for the workspace `key` names, when the tree
    /// of the current revision carries it: the delete's notice names the
    /// workspace that vanished, and the naming rule is the tree's (T4).
    pub fn workspace_label(&self, key: &str) -> Option<String> {
        self.tree()
            .header_of(key)
            .map(|header| header.label.clone())
    }

    /// One window of the flattened workspace tree as the state holds it NOW:
    /// the rows `[offset, offset + length)` of the tree, and how many rows the
    /// tree has in total. The tree is built once per revision, from snapshots
    /// taken off the lock, so a fold on the pump's thread never waits for a
    /// page.
    pub fn sessions_window(&self, offset: u32, length: u32) -> SessionsWindow {
        let language = self.language();
        window::window(&self.tree(), offset, length, now_ms(), language, &|path| {
            folder_status(path, language)
        })
    }

    /// Whether the workspace tree already holds `path` as a workspace, because a
    /// session lives there: a folder that only the history names has no state in
    /// the config to remember yet (the W1 limit).
    pub fn knows_workspace(&self, path: &str) -> bool {
        self.tree().knows(&folder_key(path))
    }

    /// The revision of the session list this holder has reached.
    pub fn sessions_revision(&self) -> u32 {
        self.sessions_revision.load(Ordering::Relaxed)
    }

    fn apply(&self, input: gentle_app::Input) -> (ViewState, Vec<gentle_app::ChatCommand>) {
        // A chat the user CLOSED leaves the open set, and it takes the rows read
        // from its session file with it, together with the incarnation those rows
        // were read for (chat-open-latency T1, S4): the same id coming back is a
        // NEW chat, so a read of the old one may never be installed on it
        // (R3-001 of `chat-transcript.md`). It is acted on after the fold, not
        // before it, because a close the reducer REFUSES - a chat mid-turn - keeps
        // the chat open and keeps its rows.
        let closing = match &input {
            gentle_app::Input::Intent(gentle_app::Intent::CloseChat(chat)) => Some(chat.clone()),
            _ => None,
        };
        // A chat that closes takes the Profile tab's view with it (profiles-editor
        // T2b-2, L11-4): the view is that chat's, and a read still in flight for
        // it may not leave a stale one behind for the next frame to draw. Done
        // before the slice is taken, so the push that reports the close carries
        // no view of it.
        if let gentle_app::Input::ChatClosed { chat, .. } = &input {
            self.forget_chat_profile(chat);
            // The session mode is that chat's own reading of its own child, and
            // it may not outlive it either (composer-dock T3b, L7).
            self.forget_session_mode(chat);
            // Nor does the folder the workspace handed it before its session
            // existed (composer-dock L12).
            self.forget_folder_hint(chat);
        }
        // One lock for the fold and the slice it answers: a revision read here
        // describes exactly the state it is pushed with. Everything the shell
        // owns is read before it, so no path ever holds the state and another
        // lock at once. The shared commands run after the lock is released,
        // because a refresh pass folds back into this same holder.
        let shell = self.shell_parts();
        let (view, chat_commands, commands) = {
            let mut app = lock(&self.app);
            speak(&mut app, shell.language);
            let previous = app.clone();
            let full = revision::folds_the_list(&input);
            let (state, effects) = gentle_app::update_chats(previous.clone(), input);
            if revision::list_moved(&previous, &state, full) {
                self.sessions_revision.fetch_add(1, Ordering::Relaxed);
            }
            *app = state;
            let view = view_state(&app, self.sessions_revision(), shell);
            (view, effects.chat_commands, effects.commands)
        };
        self.run_shared(commands);
        if let Some(chat) = closing.filter(|chat| !self.is_open(chat)) {
            self.forget_transcript(&chat);
        }
        (view, chat_commands)
    }

    /// Everything the slice carries beside the state: the lines and slices the
    /// shell owns, read together and before any state lock.
    fn shell_parts(&self) -> ShellParts<'_> {
        ShellParts {
            config_notice: self.config_notice(),
            workspace_notice: self.workspace_notice(),
            sessions_notice: self.sessions_notice(),
            whats_new: self.whats_new(),
            app_version: self.app_version(),
            fatal: self.fatal(),
            theme: self.theme_value(),
            language: self.language_value(),
            notifications: self.notifications_value(),
            shell: self.shell_view(),
            chat_profile: self.chat_profile(),
            session_mode: self.session_mode(),
            transcript: self.provisional_transcript(),
            branch: self.branch_view(),
            folder_hint: self.folder_hint(),
            auth: self.auth(),
            updates: self.updates(),
            usage: self.usage(),
            images: &self.images,
            welcome: self.welcome_view(),
        }
    }

    /// View A's data (welcome-screen T1, S1, S3, R3, R4): the last workspace, its
    /// recent chats and the other configured workspaces, read from the config and
    /// the tree of the revision the holder is at.
    ///
    /// No pass over the session list runs here: the grouping is the one the tree
    /// already cached for this revision (welcome-screen T1), so a push pays the
    /// workspaces of the config and three chat rows, never 50 000 sessions. The
    /// one thing it asks the disk is whether the last workspace's folder is still
    /// there - the same probe a served page runs for its headers, asked for ONE
    /// folder - and it asks it with no lock held: the config, the tree and the
    /// language are snapshots read before it.
    fn welcome_view(&self) -> crate::contract::WelcomeView {
        let config = self.config_snapshot();
        let tree = self.tree();
        let language = config.config.language.unwrap_or(Language::En);
        welcome::welcome(
            &config.config,
            tree.workspaces(),
            tree.sessions(),
            &|path| folder_status(path, language),
        )
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

    /// The intents the reducer has no answer for, answered here: folding a
    /// workspace's chats away is the config's (D8), choosing the theme is the
    /// config's (the v1 Appearance setting), choosing the language is the
    /// config's (0.2.0, S4), choosing the notifications mode is the config's
    /// (notifications S4), the version the user skipped is the config's (0.8.0
    /// settings-polish S1), and a workspace that names no folder starts nothing.
    /// Answers the slice to push, `None` for the reducer's intents.
    fn config_intent(&self, intent: &UiIntent) -> Option<ViewState> {
        match intent {
            UiIntent::SetWorkspaceCollapsed { path, collapsed } => {
                Some(workspace::collapse(self, path, *collapsed))
            }
            UiIntent::SetTheme { theme } => Some(self.set_theme(*theme)),
            UiIntent::SetLanguage { language } => Some(self.set_language(*language)),
            UiIntent::SetNotifications { mode } => Some(self.set_notifications(*mode)),
            UiIntent::SetShellHome { choice } => Some(self.set_shell_home(*choice)),
            UiIntent::SkipUpdate { version } => Some(self.skip_update(version)),
            UiIntent::UnskipUpdate => Some(self.unskip_update()),
            UiIntent::NewChatInWorkspace { path } if path.trim().is_empty() => Some(self.view()),
            _ => None,
        }
    }

    /// The theme this run holds, taken off the config lock before any state
    /// lock is taken, as the config notice is.
    fn theme_value(&self) -> app_config::Theme {
        self.config_snapshot().config.theme
    }

    /// The language this run holds, taken off the config lock before any state
    /// lock is taken, as the theme is. `None` while no preference is stored: this
    /// never detects the system language, and it never writes one, so a load
    /// leaves the file exactly as it found it.
    fn language_value(&self) -> Option<Language> {
        self.config_snapshot().config.language
    }

    /// When this run notifies the user, taken off the config lock before any
    /// state lock is taken, as the theme is. The default while no choice is
    /// stored, which is what the config's own lenient read answers.
    fn notifications_value(&self) -> app_config::Notifications {
        self.config_snapshot().config.notifications
    }

    /// The flattened tree of the revision the state is at: built once per
    /// revision, from snapshots read off every lock.
    fn tree(&self) -> Arc<Tree> {
        let revision = self.sessions_revision();
        self.tree.get_or_build(revision, || {
            let config = self.config_snapshot();
            let sessions = self.sessions_snapshot();
            Tree::build(revision, &config.config, &sessions)
        })
    }

    /// The images the chats hold, for a caller that reads or changes them
    /// outside a fold (`bridge::images`).
    pub fn images(&self) -> &Images {
        &self.images
    }

    /// The folded state, with the language this run speaks already in it: the
    /// read path (`view`) words the texts it derives the same way a fold does.
    fn state(&self) -> gentle_app::AppState {
        let language = self.language_value();
        let mut state = lock(&self.app).clone();
        speak(&mut state, language);
        state
    }

    /// The language this run speaks, as the shell's own code asks for it: the
    /// stored preference, or English while there is none (D2).
    pub(crate) fn language(&self) -> Language {
        self.language_value().unwrap_or(Language::En)
    }

    /// The session list the tree is built from, taken off the lock.
    fn sessions_snapshot(&self) -> Vec<SessionItem> {
        lock(&self.app).sessions.clone()
    }

    /// The config notice as the view state carries it.
    fn config_notice(&self) -> Option<String> {
        self.config_snapshot().notice.clone()
    }
}

/// Gives `state` the language this run speaks, read off the config.
/// This is the ONE place the reducer learns the language (0.2.0, S3): every
/// notice `crates/app` produces is worded through the field this sets, so
/// nothing there reads a config or a locale. `None` — no stored preference — is
/// English (D2: Rust never detects, the web reports a Spanish system once and
/// the stored choice wins from then on).
fn speak(state: &mut gentle_app::AppState, language: Option<Language>) {
    state.language = language;
}

/// Whether one intent is an OPENING of a chat: the two the web sends that put a
/// chat (and its session file) on screen, which is when the thread's first frame
/// is claimed (`bridge::transcript`). A fresh chat is one of them and has no file
/// yet, so its claim answers nothing.
fn opens_a_chat(intent: &gentle_app::Intent) -> bool {
    matches!(
        intent,
        gentle_app::Intent::OpenChat(_) | gentle_app::Intent::RestartChat(_)
    )
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
