//! Rust-to-web contract types.
//!
//! The TypeScript declarations that the Svelte view compiles against are
//! GENERATED from the definitions below into
//! `apps/desktop/src/contract/bindings.ts`. A change on the Rust side must
//! land together with a regenerated file; `tests/contract_bindings.rs` fails
//! the build when the committed file no longer matches.
//!
//! These types are the SLICES the web state renders (S2 of the migration),
//! never the whole `AppState`: the mapping to and from `gentle-app` lives in
//! `bridge`, so nothing here derives on a `crates/app` type.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChatBadge {
    pub chat_id: String,
    pub activity: ChatActivityMark,
    pub unread: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ChatActivityMark {
    Working,
    Queued,
    Opening,
    NewReply,
    NeedsAttention,
    Ended,
}

/// One chat that has something to show, as a list row sees it: the id the app
/// names it by (the same text `ViewState::foreground_chat` carries) and its
/// mark. Only chats that are not idle are listed, so the push stays as small as
/// the working set, and the mark itself is chosen by the rule `crates/runtime`
/// owns ([`gentle_runtime::badge`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatActivityView {
    pub chat_id: String,
    pub activity: ChatActivityMark,
}

/// The six themes the whole window can be set to (the v1 Appearance setting),
/// named exactly as `design/system.md` and the artboard's picker name them: one
/// spelling for the config file, the `data-theme` attribute the page paints, and
/// the label the picker draws, so no side translates a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ThemeView {
    /// The reader's own setting, and the theme of a run that chose none.
    #[serde(rename = "system")]
    System,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "Gentle")]
    Gentle,
    #[serde(rename = "Gentleman-Cute")]
    GentlemanCute,
    #[serde(rename = "Gentleman-Sexy")]
    GentlemanSexy,
    #[serde(rename = "light")]
    Light,
}

/// The two languages the app's own words can be written in (0.2.0, S4), named
/// exactly as the config file stores them and as the web's dictionaries are
/// keyed: one spelling for the stored value, the contract and the copy module,
/// so no side translates a language's own name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum LanguageView {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es")]
    Es,
}

/// When the app notifies the user about a chat that needs them (notifications
/// S4), named exactly as the config file stores it: one spelling for the stored
/// value, the contract and the Settings pane, so no side translates a mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum NotificationsMode {
    /// A tone and a system notification, only while the window is not in front.
    /// The mode of a run that chose none.
    #[serde(rename = "when-unfocused")]
    WhenUnfocused,
    /// Nothing: no tone, no system notification, whatever the window is doing.
    #[serde(rename = "off")]
    Off,
}

/// The two homes the welcome screen offers for `gentle-shell` (shell-choice S2),
/// spelled the way the app config stores them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum HomeChoiceView {
    /// Use the user's own pi (`~/.pi/agent`).
    #[serde(rename = "link")]
    Link,
    /// Keep a space of its own (`~/.gentle-shell/agent`).
    #[serde(rename = "isolated")]
    Isolated,
}

/// Where installing gentle-shell stands (shell-choice S4). It only leaves `Idle`
/// when the user gave permission on the permission screen.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum InstallView {
    #[default]
    Idle,
    Running,
    Done,
    /// The install ended badly; `message` is its exit code and the last lines
    /// of what it printed, bounded.
    Failed {
        message: String,
    },
}

/// What the app knows about the shell and the home it runs on: what it found at
/// start and what the user chose (shell-choice S1-S3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ShellView {
    /// The folder of the user's pi, when it exists on this machine. Only its
    /// existence was checked, never its contents.
    pub pi_path: Option<String>,
    /// The home the user chose, `None` until they do.
    pub choice: Option<HomeChoiceView>,
    /// Whether the welcome question is due: pi was found, nothing is chosen
    /// yet, and the `gentle-shell` launcher resolves. All three must hold, so
    /// the question is only ever asked of a run that can start: with no pi there
    /// is nothing to choose between, and with no launcher there is nothing to
    /// run, so a machine still missing gentle-shell sees the install screen and
    /// is asked nothing.
    pub ask: bool,
    /// Whether the `gentle-shell` launcher resolves on this machine. `true`
    /// until the app has looked, so a run that did not look never claims it is
    /// missing.
    pub launcher_found: bool,
    /// Where installing gentle-shell stands.
    pub install: InstallView,
    /// Whether the home chosen differs from the one this run started with. A
    /// choice only applies to a new run, so while this is true the sidebar says
    /// "restart to apply" and offers the restart.
    pub restart_needed: bool,
}

/// What the user asked for, as the web side sends it. Only the intents the
/// current slice needs; the rest are added when their views arrive. The
/// explicit mapping into `gentle-app`'s [`gentle_app::Intent`] keeps this
/// enum free of every `crates/app` type.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum UiIntent {
    SendPrompt {
        text: String,
    },
    Abort,
    NewChat,
    /// Make that chat the foreground one, asking the host for its child.
    OpenChat {
        chat_id: String,
    },
    /// The same for a chat that did not exist before.
    OpenNewChat,
    /// Start the child of that chat, if this run never started one for it
    /// (welcome-screen T4, R2).
    ///
    /// A new chat is ALLOCATED when it opens and its process starts on this ask:
    /// the composer sends it the first time its box is focused for that chat, or
    /// when the first key lands in it, so opening a new session (or the app
    /// itself) starts no child and costs no process. Until the child answers, the
    /// chat is Opening and its composer is disabled with the reason it has today;
    /// the draft is kept and an Enter sends nothing.
    ///
    /// The ask is IDEMPOTENT by the state: a chat whose child was asked for
    /// already - Opening, working, queued, idle or ended - sends no command and
    /// changes nothing, so a focus and a keystroke can never start two children.
    /// A stored chat starts its child when it is opened, so it never needs this.
    EnsureStarted {
        chat_id: String,
    },
    /// Start a NEW chat in the workspace at `path`: what the header's New
    /// session button (D2) and a workspace's own new-chat icon (D3) send. The
    /// web side decides which workspace is the current one and names it; the
    /// chat started here is the same new chat `OpenNewChat` starts, only in
    /// that folder, and never the legacy single-child `NewChat`.
    NewChatInWorkspace {
        path: String,
    },
    /// Fold or unfold the chats of the workspace at `path` in the sidebar (D8):
    /// the state is remembered in the app config, so it survives the run.
    SetWorkspaceCollapsed {
        path: String,
        collapsed: bool,
    },
    /// Choose the window's theme (the v1 Appearance setting). The choice is
    /// remembered in the app config, so it survives the run, and the window's
    /// own chrome follows it ([`gentle_runtime::app_config::Theme`]).
    SetTheme {
        theme: ThemeView,
    },
    /// Choose the language the app's own words are written in (0.2.0, S2): the
    /// one tag the picker sent, stored in the app config so it survives the run,
    /// and re-emitted as the very slice this call answers with - the words change
    /// in this same tick, with no restart and no webview reload ([`LanguageView`]).
    /// The same language again changes nothing and saves nothing.
    SetLanguage {
        language: LanguageView,
    },
    /// Choose when the app notifies the user about a chat that needs them
    /// (notifications S4): while the window is not in front (the default), or
    /// never. The choice is remembered in the app config, so it survives the
    /// run, and re-emitted as the slice this call answers with. The same mode
    /// again changes nothing and saves nothing ([`NotificationsMode`]).
    SetNotifications {
        mode: NotificationsMode,
    },
    /// Choose which home `gentle-shell` runs on (the welcome screen and the
    /// sidebar footer). Remembered in the app config; it applies from the next
    /// start of the app.
    SetShellHome {
        choice: HomeChoiceView,
    },
    /// Start the app again after the orderly close (config and cache flushed
    /// first): a home chosen at the welcome question applies to a new run.
    RestartApp,
    /// The user gave permission to install gentle-shell (the button of the
    /// permission screen). Nothing installs without this intent.
    InstallShell,
    /// Look for the `gentle-shell` launcher once more ("Check again").
    CheckShellAgain,
    /// Start the child of a chat that died again and reopen it (S3).
    RestartChat {
        chat_id: String,
    },
    /// Close one chat (0.1.14): stop that chat's child and take it off the open
    /// set, which is what the Updates pane's chats-closed gate counts. NOT a
    /// delete: nothing on disk changes and the session list keeps its row, so
    /// the row opens the chat again. A chat that is running a turn is refused,
    /// with the same rule and words the delete flow refuses with.
    CloseChat {
        chat_id: String,
    },
    /// Copy one message's raw markdown: the message is named by the id the
    /// thread was given ([`MessageView::id`]), and the copy itself is the
    /// shell's side effect, never the view's.
    CopyMessage {
        message_id: String,
    },
    DismissNotice {
        id: u32,
    },
    /// Answer the foreground chat's pending dialog, named by [`DialogView`]'s
    /// `id`. An id that is no longer pending answers nothing, so a second click
    /// on the same dialog never reaches the child twice.
    AnswerDialog {
        id: String,
        answer: DialogAnswer,
    },
    /// Open the thread of one delegated helper (T7b-2). The id is the one
    /// [`HelperRowView::id`] carried, and the reducer only selects a helper of
    /// the foreground chat: an id that names no task changes nothing.
    SelectHelper {
        helper_id: String,
    },
    /// Back from a helper's thread to the helpers list: the same intent the
    /// iced panel's Back sends (`crates/ui/src/app.rs`).
    CloseHelper,
    /// Stop the RUNNING helper whose thread is open (helper-stop S2): one helper,
    /// never the chat. The id is the one [`HelperRowView::id`] carried, and the
    /// reducer only stops a helper of the foreground chat that is running and is
    /// that chat's own: any other id changes nothing and asks nothing of the
    /// child.
    StopHelper {
        helper_id: String,
    },
    /// Ask the child what models it offers: what the chip's dropdown shows
    /// (0.1.11). The answer travels back on
    /// [`ViewState::model_chip`].
    RequestModels,
    /// Make a model the session's model (RPC `set_model`). The chip updates
    /// from the reply's Model object, never from the choice alone.
    SelectModel {
        provider: String,
        model_id: String,
    },
    /// Set the session's thinking level (RPC `set_thinking_level`).
    SelectThinkingLevel {
        level: String,
    },
    /// Re-read THIS chat's own profile configuration (profiles-editor T2b-2,
    /// L11): the profiles, their fallback chains and the pin of that chat's
    /// worktree, asked of that chat's OWN child, which is the only one that knows
    /// them. The Profile tab sends it when it opens and when it asks again.
    /// Nothing is written, and a read that reached no store leaves the view as it
    /// was. The answer travels back on [`ViewState::chat_profile`], for that chat
    /// alone.
    RefreshChatProfile {
        chat_id: String,
    },
    /// Save ONE profile's orchestration mode, the way the terminal does: the
    /// value `profiles.json` stores under `workflow.<profile>.orchestratorMode`,
    /// which EVERY chat using that profile reads, never this chat alone (L20,
    /// L22).
    ///
    /// The write is the whole `profiles` document as this chat's own child read
    /// it, with only that profile's entry changed, preconditioned on the read's
    /// `writeTargets[global].writeRevision` (S2, S3), and it is followed by a
    /// fresh read of that chat's configuration. A write that did not happen - a
    /// stale token, a document the owner refuses, an apply in flight, a child
    /// that does not answer - changes nothing and leaves
    /// [`ChatProfileView::save_failed`] set, which the tab says in one short
    /// line and clears on the next successful read.
    SaveChatProfileMode {
        chat_id: String,
        /// The profiles store's own key for the profile (the one this chat uses).
        profile: String,
        /// The mode to store. `organic` is stored as the ABSENCE of the entry.
        mode: OrchestratorModeView,
    },
    /// Apply one saved profile over THIS chat's own child (profiles-editor T3b,
    /// S4, S8, S9): the terminal's other two outcomes, which a save never
    /// reaches.
    ///
    /// `Use in this chat` ([`ApplyModeView::Session`]) binds the profile to the
    /// chat the intent names and to no other, so the other open chats keep the
    /// routing they started with. `Make default` ([`ApplyModeView::Default`]) is
    /// the channel's own global apply (L30 ruling 1): it writes the store's
    /// `active` and claims the routing of every new chat, and it is the only mode
    /// that answers `confirmation_required` - the screen's own dialog, whose
    /// title and message are shown AS THEY ARRIVED (L30 ruling 2) and whose
    /// `token` comes back in [`confirmation`](Self::ApplyChatProfile::confirmation)
    /// when, and only when, the human accepts.
    ///
    /// The answer travels back on [`ViewState::chat_profile`]: the buttons are
    /// disabled while it is in flight, the dialog is drawn verbatim, every answer
    /// that wrote nothing leaves one short notice
    /// ([`ChatProfileView::apply_notice`]), and every answer that RAN leaves the
    /// child's own notes, verbatim, in
    /// [`ChatProfileView::apply_answer`] - the only thing an `applied` answer that
    /// asked nothing says.
    ApplyChatProfile {
        chat_id: String,
        /// The profiles store's own key for the profile to apply.
        profile: String,
        mode: ApplyModeView,
        /// The token of the dialog that was SHOWN, and nothing else: a first
        /// apply carries none, and an empty or unrecognisable string is not a
        /// token at all - it is dropped, so the apply asks again and a fresh
        /// dialog (with a fresh token) answers instead of writing.
        confirmation: Option<String>,
    },
    /// Save ONE fallback chain of the used profile, the way the terminal edits
    /// `profile-fallbacks.json` (profiles-editor T5, S2, S3, L30-3): the chain of
    /// one agent, or the profile-wide `*` one, stored under
    /// `profiles.<profile>.<agent>`. Every chat using that profile reads it, never
    /// this chat alone.
    ///
    /// The write is the whole `profile_fallbacks` document as this chat's own
    /// child read it, with only that chain replaced, preconditioned on THAT
    /// store's `writeTargets[global].writeRevision` (S2, S3), and it is followed
    /// by a fresh read of that chat's configuration. The chain is checked against
    /// the owner's own rules before anything is sent (at most eight steps, a model
    /// per step, no model twice, never empty); a write that did not happen - a
    /// stale token, a document the owner refuses, an apply in flight, a child that
    /// does not answer - changes nothing and leaves
    /// [`ChatProfileView::chain_failed`] set, which the tab says in one short line
    /// and clears on the next successful read.
    SaveChatProfileChain {
        chat_id: String,
        /// The profiles store's own key for the profile (the one this chat uses).
        profile: String,
        /// The chain's own key: an agent's name, or `*` for the profile-wide one.
        agent: String,
        /// The chain as the user built it, in the order gentle-pi would move
        /// through it. An empty list is a page bug, not a way to delete a chain:
        /// the owner keeps no empty chain, so the shell refuses it too.
        steps: Vec<ChatProfileStepView>,
    },
    /// Write ONE pin layer of this chat's WORKTREE, the way the terminal pins a
    /// profile (profiles-editor T6, S2, S3): the `pins` store's `data` for that
    /// layer, exactly `{source, profile}`, where a `null` profile CLEARS the
    /// layer.
    ///
    /// The pin is per LAYER and per worktree, never per profile: `local` is the
    /// clone-wide file in the Git common dir, `repo` the committable one of this
    /// chat's worktree. The write is preconditioned on THAT layer's own
    /// `writeTargets[].writeRevision` (S3), it is followed by a fresh read of that
    /// chat's configuration, and it writes nothing of the other layer - the
    /// channel's pin payload names one file, so there is no document to rebuild.
    ///
    /// A name the profiles store does not define, a stale token, an apply in
    /// flight, a chat with no resolvable worktree and a child that does not
    /// answer all change nothing and leave [`ChatProfileView::pin_failed`] set,
    /// which the tab says in one short line and clears on the next successful
    /// read.
    SaveChatProfilePin {
        chat_id: String,
        /// The layer to write: `local` or `repo`, and nothing else. Anything
        /// else is a page bug and is dropped, so no request leaves for it.
        source: String,
        /// A profile name the store defines, or `null` to clear that layer.
        profile: Option<String>,
    },
    /// Read the SESSION permission mode of one chat (composer-dock T3b, L7):
    /// `get_permission_mode` over that chat's OWN child, which is the only one
    /// that knows the mode of the session it runs. Nothing is written.
    ///
    /// The answer travels back on [`ViewState::session_mode`], for that chat
    /// alone, and it is the owner's own string, never a name this app coined.
    /// A read that did not answer draws no mode at all: the control is absent
    /// rather than guessed at.
    ReadSessionMode {
        chat_id: String,
    },
    /// Ask the SESSION permission mode of one chat to change (composer-dock T3b,
    /// L7): `set_permission_mode { mode }` over that chat's own child, and never
    /// `persist` - this changes the mode of THIS session and leaves the saved
    /// default exactly as it was.
    ///
    /// The shown mode changes only on the confirmed answer, never optimistically
    /// (L7), so the answer that travels back on [`ViewState::session_mode`] is
    /// the state the owner re-read after the attempt. A refused or unanswered
    /// write keeps the mode it had and leaves its own `refusal`, which the page
    /// shows; `yolo` and `super-yolo` are the page's confirmation step to ask
    /// for, never this intent's.
    SetSessionMode {
        chat_id: String,
        mode: SessionModeChoice,
    },
    /// Re-read the provider names and re-run one `auth check` per provider
    /// (0.1.12, T3). The answer travels back on [`ViewState::auth`].
    RefreshAuth,
    /// Check the three versions again (0.1.12, S2, S7), the same check the app
    /// runs once when it opens. It only ever CHECKS: no update is installed
    /// from here. The answer travels back on [`ViewState::updates`].
    ///
    /// Unlike the check that runs when the app opens, this one always reports
    /// the real state: a version the user skipped is offered here with
    /// [`UpdatesView::skipped`] beside it, so the pane can say so and offer to
    /// stop skipping (0.8.0 settings-polish S2).
    CheckUpdates,
    /// Skip the version the Updates pane offered (0.8.0 settings-polish S1): the
    /// version the check reported, remembered in the app config so it survives a
    /// restart, and beside the rows of every later push as
    /// [`UpdatesView::skipped`]. From then on the check that runs when the app
    /// opens does not offer it again; a version NEWER than the skipped one is
    /// offered as usual (S2).
    ///
    /// Versions are compared as versions, never as text, and a `version` this
    /// app cannot read as a strict `x.y.z` stores nothing - the same rule the
    /// stored value is read with - so a bad value hides no update.
    SkipUpdate {
        version: String,
    },
    /// Stop skipping that version (S2): the next check offers it again. Answers
    /// the re-emitted slice like every other config intent, and nothing is
    /// installed or uninstalled by it.
    UnskipUpdate,
    /// Read the subscription usage the shell's own sources report, and run them
    /// if they are installed: what the Usage pane asks for when it appears and
    /// on "Check again". This is the ONE run the pane makes, on the intent's own
    /// thread, because a run takes seconds and the keyboard must not wait.
    /// Nothing of it reads a chat, so no open chat can hold it back. The answer
    /// travels back on [`ViewState::usage`].
    RefreshUsage,
}

/// One mode the page may ask the session to switch to (composer-dock T3b, S1).
///
/// A closed set, named exactly as the channel's own wire spells the four modes
/// (`normal`, `auto`, `yolo`, `super-yolo`), so neither side translates a value
/// and no string a page sends can become a mode nobody offers. Which of them
/// needs a confirmation step first is the page's business and its design's
/// (`yolo` and `super-yolo`), never this enum's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum SessionModeChoice {
    /// Asks before a tool the mode would otherwise allow.
    #[serde(rename = "normal")]
    Normal,
    /// Chooses by the owner's own rules without asking.
    #[serde(rename = "auto")]
    Auto,
    /// Never asks: the session runs unattended.
    #[serde(rename = "yolo")]
    Yolo,
    /// The widest of the four: never asks, and takes the owner's own widest
    /// permission set. The status text reads `SUPER-YOLO`; the wire is
    /// `super-yolo`.
    #[serde(rename = "super-yolo")]
    SuperYolo,
}

/// The session's permission mode as the slice carries it (composer-dock T3b,
/// L7): what the chat's own child reported, and what became of the last attempt
/// to change it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SessionModeView {
    /// The mode in effect, in the channel's OWN string, exactly as it arrived.
    /// A value this build does not know is shown as-is, never hidden and never
    /// mapped onto a mode the app does know, so an upstream mode the page has
    /// not learned yet still says what the session really runs.
    pub mode: String,
    /// The owner's own code for the last attempt that did not happen
    /// (`permission_denied`, `permission_unavailable`, `invalid_request`, ...),
    /// or `no_answer`/`unsupported` when the child did not answer at all. `None`
    /// while nothing was refused, and cleared by the next answered exchange.
    /// The mode above is left exactly as it was: a failure shows its code and
    /// changes nothing (L7).
    pub refusal: Option<String>,
    /// How many WRITE attempts this chat has answered since it opened. A read is
    /// no attempt, and the page tells one confirmed change from the next by it.
    pub attempt: u32,
}

/// One model pi offers, flattened for the chip's dropdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ModelChoiceView {
    pub provider: String,
    pub model_id: String,
    pub name: String,
    /// Whether the model does reasoning, pi's own `reasoning` flag.
    pub reasoning: bool,
}

/// Where the model list stands, matching `gentle_app::ModelsStatus` one to one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum ModelListStatusView {
    Unknown,
    Loading,
    Ready,
    Empty,
    Failed(String),
}

/// What the composer's model chip shows: the models the child offers, the
/// session's current model and thinking level, and the levels it supports.
/// All of it is the child's truth, never the page's guess: the chip renders a
/// state, it does not invent one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ModelChipView {
    pub status: ModelListStatusView,
    pub models: Vec<ModelChoiceView>,
    pub current: Option<ModelChoiceView>,
    /// The session's thinking level; `None` is unknown, not off.
    pub thinking_level: Option<String>,
    /// The levels the current model supports, in pi's order.
    pub thinking_levels: Vec<String>,
}

/// How the orchestrator runs a turn, named exactly as gentle-pi's
/// `profiles.json` `workflow.<profile>.orchestratorMode` names them (`organic`,
/// `full-delegation`), so neither side translates a value. The value a view
/// carries is the PROFILE's, which every chat using that profile runs with
/// (profiles-editor L20, L22); the session-only switch that used to carry it to
/// children at spawn time is gone with the Settings pane (T2d).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum OrchestratorModeView {
    /// gentle-pi's own default: the orchestrator works and delegates as it
    /// chooses. A run that never chose shows this one.
    #[default]
    #[serde(rename = "organic")]
    Organic,
    #[serde(rename = "full-delegation")]
    FullDelegation,
}

/// Which of the terminal's two APPLY outcomes a request asks for, named exactly
/// as the channel's `apply_profile` params name them (`session`, `default`), so
/// neither side translates the value (profiles-editor T3b, S8, S9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ApplyModeView {
    /// `Use in this chat`: bind the profile to this chat's OWN child, write no
    /// store, and never ask for a confirmation. The other open chats keep the
    /// routing they started with.
    #[serde(rename = "session")]
    Session,
    /// `Make default`: the channel's own global apply, which writes the store's
    /// `active` and may answer `confirmation_required` first (L30 ruling 1).
    #[serde(rename = "default")]
    Default,
}

/// The one short line an apply that wrote nothing leaves, in the cases the
/// channel names (profiles-editor T3b, S4).
///
/// It is a TYPE, not a sentence: the page words each case in its own language
/// (D4), and Rust never writes a user-facing line for a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ChatProfileApplyNoticeView {
    /// No profile by that name is saved, and nothing was written.
    UnknownProfile,
    /// Another apply is already running: nothing was written, and it can be
    /// retried after it finishes.
    Busy,
    /// The store moved between the read and the write: nothing was written.
    RevisionConflict,
    /// The owner refused the data it was given: nothing was written.
    InvalidData,
    /// No answer at all: a deadline, a child that is gone, a host that is not
    /// interactive. Nothing was written.
    NoAnswer,
    /// This chat's child does not advertise `apply_profile` at all. Nothing was
    /// sent.
    Unsupported,
    /// The apply ran and its own notices did not end well: the screen's path
    /// rolled its write back, so nothing stands.
    NotApplied,
    /// Any other refusal, reported as the one line a case this slice does not
    /// model deserves.
    Refused,
}

/// The screen's own dialog, carried to the page EXACTLY as the child wrote it
/// (profiles-editor T3b, L30 ruling 2).
///
/// `title` and `message` are the child's text, in the child's language, never
/// translated and never re-worded: this is the same dialog the terminal shows.
/// `token` is what the acceptance carries back, and it is bound to the content it
/// was issued for, so a resend with anything else re-asks instead of writing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileApplyConfirmationView {
    /// The profile the dialog is about, as the child echoed it.
    pub profile: String,
    /// The mode it asks about (`default` today: `session` never asks).
    pub mode: ApplyModeView,
    /// The child's own title, verbatim.
    pub title: String,
    /// The child's own message (the diff and the question), verbatim.
    pub message: String,
    /// The token to carry back if, and only if, the human accepts.
    pub token: String,
}

/// The child's own ANSWER to an apply that RAN - `applied` or `not_applied`,
/// which both ran - carried to the page as it wrote it (profiles-editor T3b, S4,
/// L32).
///
/// Its notes are the child's text, in the child's language, in its own voice: the
/// approved probe of L32 has them start with `el Gentleman applied profile ...`,
/// and the one for a profile whose routing already matches says so instead of
/// claiming a change. The page draws them VERBATIM and wraps them in no sentence
/// of its own, exactly as it draws the child's dialog (L30 ruling 2) - a
/// maintainer's real-window finding (2026-11-17) was that an apply that ran left
/// the tab with nothing at all said.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileApplyAnswerView {
    /// One entry per note the child wrote, in the child's own order, each with its
    /// OWN line breaks kept (a note is two or three lines long).
    pub notes: Vec<String>,
}

/// One agent of one profile, as the chat's Profile tab draws it (profiles-editor
/// T2b-2, S1): the key the store names the agent by and the route it stores.
/// Every route member is optional because the file allows a partial row, and a
/// missing one is drawn as nothing rather than as a guess.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileAgentView {
    /// The agent's name, the store's own key (`orchestrator`, `worker`, ...).
    pub agent: String,
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub tier: Option<String>,
}

/// One step of a fallback chain: the model, and the effort gentle-pi switches to
/// with it when the step names one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileStepView {
    pub model: String,
    pub thinking: Option<String>,
}

/// One fallback chain of one profile: the agent it is for, and the models in the
/// order gentle-pi would move through them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileChainView {
    /// The store's own key: an agent's name, or `*` for the profile-wide chain.
    pub agent: String,
    pub steps: Vec<ChatProfileStepView>,
}

/// One profile definition, so the tab can draw a profile this chat is not bound
/// to. The chains are the ones the sibling `profile_fallbacks` store records for
/// it, and an empty list means the store names none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileDefinitionView {
    pub name: String,
    /// Every agent of the profile, in the order the store reports them.
    pub agents: Vec<ChatProfileAgentView>,
    /// Every chain this profile has, `*` included, in the store's own order.
    pub chains: Vec<ChatProfileChainView>,
    /// The mode this profile's `workflow` entry stores in `profiles.json`, which
    /// applies to EVERY chat that uses the profile, not to this one alone (L20,
    /// L22). An absent entry is `Organic`, gentle-pi's own default, so the field
    /// is defaulted rather than optional: the page always has a word to draw.
    #[serde(default)]
    pub mode: OrchestratorModeView,
}

/// Where one pin layer stands, in the three states the `pins` value itself
/// names (`desktop-rpc.md:211-234`).
///
/// A layer that holds a VALID profile another layer outranks is not reported by
/// that value at all, so it carries no mark: the tab shows the winner and the two
/// broken lists, and invents no third mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ChatProfilePinMarkView {
    /// The layer that decides which profile this chat's worktree uses.
    Winner,
    /// The layer names a profile the profiles store no longer defines.
    Stale,
    /// The layer is not a pin at all.
    Invalid,
}

/// The state of ONE pin FILE, as the `pins` store's own `layers[]` reports it
/// (`desktop-rpc.md:196-234`). It is per file and independent of what the file
/// amounts to: a layer can read `ok` and still be outranked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ChatProfilePinStatusView {
    /// The file exists and holds a pin the owner could read.
    Ok,
    /// No file of that layer exists.
    Missing,
    /// The file exists and is not a pin.
    Invalid,
}

/// One pin layer: the file it lives in, the profile it names, and what became of
/// it.
///
/// Everything an EDITOR needs is here too (profiles-editor T6): the file's own
/// state, whether a valid pin in it is merely outranked (and so not reported at
/// all), and whether a write may replace it - which is what tells the tab to draw
/// a picker instead of a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfilePinLayerView {
    /// `local` or `repo`: the file the pin is kept in.
    pub source: String,
    /// The state of that FILE, from the store's `layers[]`.
    pub status: ChatProfilePinStatusView,
    /// The profile the layer names, `None` when it names none - and `None` for a
    /// layer whose valid pin another layer outranks, whose content is not
    /// reported by the value at all.
    pub profile: Option<String>,
    /// What the value makes of this layer, `None` when it does not name it: a
    /// valid pin another layer outranks carries no mark.
    pub mark: Option<ChatProfilePinMarkView>,
    /// Whether this layer reads `ok` and the value does not name it, i.e. a
    /// VALID pin here is outranked by another layer (`desktop-rpc.md:211-234`).
    /// Its content is then UNKNOWN to this client, and the editor must not claim
    /// to know it.
    pub outranked: bool,
    /// Whether a write may replace this layer's file: its `writeTargets` entry is
    /// present, which outside a resolvable Git worktree it is not
    /// (`desktop-rpc.md:273`). Only a writable layer is editable.
    pub writable: bool,
}

/// What the `pins` store answered for THIS chat's own worktree: the pins belong
/// to a folder, so they are read from the chat's child and never globally.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfilePinsView {
    /// Whether a layer named a profile the profiles store still defines.
    pub resolved: bool,
    /// The layer that decides here, when one does.
    pub winner: Option<ChatProfilePinLayerView>,
    /// Every layer the answer reports, one per pin FILE this worktree has (the
    /// store's `layers[]`), each with its own mark, its own state and whether it
    /// may be written: the winner first, then the stale ones, then the ones that
    /// are not a pin at all, and a layer the value does not name last.
    pub layers: Vec<ChatProfilePinLayerView>,
    /// The owner's own code for why nothing resolved; `worktree_unresolved`
    /// today.
    pub reason: Option<String>,
    /// Whether `reason` is the no-worktree case: no working directory inside a
    /// resolvable Git worktree, so there is no pin file to read or write (state 5
    /// of the artboard).
    pub worktree_unresolved: bool,
}

/// How one of the three stores a chat's profile is read from answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ChatProfileAvailabilityView {
    /// The store answered with its content.
    Read,
    /// The store answered, and the answer carried no content this contract
    /// reads: a refusal the owner words itself, or an unusable payload.
    Unavailable,
    /// The store was NEVER ASKED, because an earlier one got no answer at all
    /// and asking could only wait out another deadline. It is state 10 of the
    /// artboard - this chat's child is not answering - and it reaches the page
    /// without a channel error.
    Skipped,
}

/// How each of the three stores answered, so the page can draw "unavailable"
/// without knowing one channel error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileStoresView {
    /// `profiles`: the definitions and the global `active` marker.
    pub profiles: ChatProfileAvailabilityView,
    /// `profile_fallbacks`: the chains per profile and agent.
    pub fallbacks: ChatProfileAvailabilityView,
    /// `pins`: the layers of this chat's own worktree.
    pub pins: ChatProfileAvailabilityView,
}

/// The Profile tab's view of ONE chat (profiles-editor T2b-2, L8, L9, L11): which
/// profile this chat uses, the definitions it could switch to, their fallback
/// chains, and the pin of its own worktree.
///
/// It carries codes and names only - the words the tab says are its own, in the
/// page (T2b-3) - and no revision and no write token: the shell keeps the token
/// a write needs to itself, inside the chat's own client, and this view only
/// says whether a save is in flight and whether the last one failed
/// (profiles-editor T3-mode).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ChatProfileView {
    /// The chat this view belongs to. [`ViewState::chat_profile`] carries it only
    /// while that chat is the foreground one.
    pub chat_id: String,
    /// The profile this chat BOUND, `None` for a chat that uses the default: a
    /// chat with no session file, a session file with no bind entry, a cleared
    /// one, and an unreadable one are all the default.
    pub bound: Option<String>,
    /// Whether `bound` came from the chat's session file - which is the only way
    /// it can be read (L8: no channel read exposes the binding), so the tab
    /// labels the answer as the approximation the maintainer accepted in L9.
    /// `false` for a chat that bound nothing.
    pub bound_from_session_file: bool,
    /// The layer the session file recorded the binding for: `user`, `local`,
    /// `repo` or `global`, as the record itself names them.
    pub bound_origin: Option<String>,
    /// The profiles store's `active`: the GLOBAL default, which a valid pin
    /// outranks (L8). `None` when that store was not read, and when it names no
    /// active profile.
    pub default_profile: Option<String>,
    /// Every profile the store defines, in the order it reports them.
    pub profiles: Vec<ChatProfileDefinitionView>,
    /// The pins as this chat's own worktree resolves them, `None` when that store
    /// was not read ([`ChatProfileStoresView::pins`] then says why).
    pub pins: Option<ChatProfilePinsView>,
    /// How each of the three stores answered.
    pub stores: ChatProfileStoresView,
    /// Whether a save of this profile's mode is in flight right now: the switch
    /// is DISABLED while it is, so a second click cannot write twice
    /// (profiles-editor T3-mode). Shell-owned: it is cleared by the save's own
    /// answer, and a chat that closes drops it with the view.
    pub saving: bool,
    /// Whether the last attempt to save this profile's mode FAILED, and nothing
    /// was written: a stale `expectedRevision`, a document the owner refuses, an
    /// apply in flight, a child that did not answer, or a method it does not
    /// advertise. The tab says so in one short line next to the switch and draws
    /// the re-read truth. Cleared by the next successful read, by a save that
    /// worked, and never set by the read itself.
    pub save_failed: bool,
    /// Whether an APPLY of the used profile is in flight right now (profiles-editor
    /// T3b): both buttons - `Use in this chat` and `Make default` - are DISABLED
    /// while it is, so one click cannot become two applies. Shell-owned and
    /// shell-side capped like the save: a second ask is dropped, never queued
    /// (L18, L29), and a chat that closes drops it with the view.
    pub applying: bool,
    /// The one short line the last apply that WROTE NOTHING left, `None` when it
    /// worked, when it is still in flight, and when the child is asking its own
    /// confirmation. Cleared by the next apply's own answer and by the next
    /// successful read.
    pub apply_notice: Option<ChatProfileApplyNoticeView>,
    /// The child's own answer to the last apply that RAN, `None` for an attempt
    /// still in flight, one that wrote nothing (its
    /// [`apply_notice`](Self::apply_notice) says why), and one the child is asking
    /// about (its [`apply_confirmation`](Self::apply_confirmation) is the words).
    ///
    /// `applied` and `not_applied` both RAN, and both said what they did: a
    /// profile whose routing already matches answers `applied` with the child's
    /// own "already matches" line, and a dialog never appears for it, so this is
    /// the ONLY thing the user learns from clicking. A `not_applied` answer
    /// carries both the short notice that nothing stands and these notes, which
    /// say why.
    ///
    /// Cleared by the next apply's own answer and by the next successful read, the
    /// same one lifecycle the notice keeps: no second rule, no timer.
    pub apply_answer: Option<ChatProfileApplyAnswerView>,
    /// The screen's own confirmation, when the child answered
    /// `confirmation_required` and nothing was written yet. Its `title` and
    /// `message` are the CHILD's text and are drawn verbatim (L30 ruling 2);
    /// accepting sends the SAME profile and mode back with its `token`, and
    /// cancelling sends nothing at all.
    pub apply_confirmation: Option<ChatProfileApplyConfirmationView>,
    /// Whether the last attempt to save ONE of this profile's fallback chains
    /// FAILED, and nothing was written (profiles-editor T5, L30-3): a stale
    /// `expectedRevision`, a document the owner refuses, an apply in flight, a
    /// child that did not answer, a chain the owner's own rules refuse, or a
    /// profile/chain key the store does not record. It is the PARALLEL of
    /// [`Self::save_failed`] - one notice per control, never one for both - and
    /// its lifecycle is the same: cleared by the next successful read, by a save
    /// that worked, and never set by the read itself. A chain save holds the SAME
    /// one-save slot as the mode switch, so [`Self::saving`] disables both.
    pub chain_failed: bool,
    /// Whether the last attempt to write ONE pin layer FAILED, and nothing was
    /// written (profiles-editor T6): a stale `expectedRevision`, a name the
    /// profiles store does not define, no resolvable Git worktree, a child that
    /// did not answer, a method it does not advertise. It is the PARALLEL of
    /// [`Self::save_failed`] and [`Self::chain_failed`] - one notice per control,
    /// never one for every write - and its lifecycle is the same: cleared by the
    /// next successful read, by a save that worked, and never set by the read
    /// itself. A pin save holds the SAME one-save slot as the mode switch and the
    /// chain editor, so [`Self::saving`] disables all three.
    pub pin_failed: bool,
}

/// What one "Open in terminal" button asks the shell to do (0.1.12, S1, S2,
/// T2, T3). It is a COMMAND and not an intent, like `open_link`: nothing of the
/// app state changes when a terminal window opens, and the `argv` the shell is
/// run with is the shell's business ([`crate::terminal`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum TerminalActionView {
    /// The shell's profiles screen, as its opening prompt: the chat's own
    /// Profile tab sends every edit, create and switch there (L9, and T2d moved
    /// the button out of Settings).
    Profiles,
    /// The shell's own `auth login`.
    Login,
    /// The shell's own `auth logout`.
    Logout,
    /// The shell's own `update`, which forwards to pi's updater (L5): run from a
    /// terminal, and only offered with every chat closed (S4).
    Update,
    /// The same shell update with pi's `--extensions` flag passed straight
    /// through, so only the packages the shell has installed are touched: safe
    /// to run any time, and offered without a check (nothing can say which
    /// extensions are behind).
    UpdateExtensions,
    /// The shell's own install of the usage sources
    /// (`pi install git:github.com/ashcroft08/pi-usage-sources`): the one way
    /// the Usage pane offers to add them, EXPLICIT and in a terminal window the
    /// user reads - never a silent install from the desk.
    InstallUsageSources,
}

/// How one provider's `auth check` stands: the four states the shell reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ProviderStatusView {
    Ready,
    NotReady,
    /// The store lists the provider, but pi's registry does not know the name
    /// (a custom provider), so the check cannot speak: the credentials exist,
    /// nobody verified them.
    Unverified,
    Invalid,
    Error,
}

/// One provider the shell's auth store lists, with what its own check said. The
/// NAME is all that is read from the store: no key, token or account identity
/// travels up (S1's refinement, L11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AuthProviderView {
    pub name: String,
    pub status: ProviderStatusView,
    /// The shell's own reason, when it gave one.
    pub reason: Option<String>,
}

/// Where the auth read stands.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum AuthStatusView {
    /// Not read yet.
    #[default]
    Unknown,
    /// Read: every provider listed was checked.
    Ready,
    /// The store could not be read; the string is why.
    Failed(String),
}

/// The Account pane (0.1.12, S1, T3): one row per connected provider, as the
/// shell's own check answered.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AuthView {
    pub status: AuthStatusView,
    pub providers: Vec<AuthProviderView>,
}

/// One version row of the Updates pane: what is installed, what is published,
/// and whether the second is newer. `update_available: None` means the
/// comparison cannot be made - a probe that failed, a registry that did not
/// answer, or gentle-shell's channel, which the desk cannot tell - and it is
/// never drawn as "up to date".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct VersionRowView {
    pub current: Option<String>,
    pub latest: Option<String>,
    pub update_available: Option<bool>,
}

/// Where the update check stands. It runs once when the app opens (S3) and again
/// only when the user asks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum UpdatesStatusView {
    /// Not checked yet.
    #[default]
    Unknown,
    Checking,
    /// The check ran; each row says what it found on its own.
    Ready,
}

/// The Updates pane (0.1.12, S2, S7): the three pieces this machine runs, and
/// the one check the app made for them. It is read-only: no update is installed
/// from the contract.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct UpdatesView {
    pub status: UpdatesStatusView,
    /// The version the user skipped (0.8.0 settings-polish S1, S2), `None`
    /// while nothing is skipped, and always the config's own value - so it is
    /// here before the first check has run and right after a restart.
    ///
    /// It travels BESIDE the rows and never instead of them: a row keeps what is
    /// published, and only the check the app runs when it opens stops offering
    /// that version (`update_available: false`). A check the user asked for
    /// reports the real state with this field set, which is what the pane draws
    /// its offer to stop skipping from. A value that is not a strict `x.y.z` is
    /// no skip at all, so it never hides anything.
    ///
    /// Absent from the push while nothing is skipped, like the config's own
    /// optional fields: a run that skipped nothing sends what older builds sent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub skipped: Option<String>,
    /// The desk itself, against the release manifest the updater reads.
    pub desk: VersionRowView,
    /// pi, against the npm registry's published version.
    pub pi: VersionRowView,
    /// gentle-shell. Its channel decides where a newer version lives and the
    /// desk cannot tell the channel, so this row never claims an update.
    pub shell: VersionRowView,
}

/// How one provider of the usage document stands: the three statuses the
/// shell's own sources print. `NoData` is drawn as "no data" and never as a
/// zero percent, and `Failed` carries the source's own redacted reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum UsageProviderStatusView {
    Ok,
    NoData,
    Failed,
}

/// One usage window: its label as the provider names it ("5h", "week"), the
/// share of it used, and the countdown to its reset when the provider reports
/// one. `resets_in` is already a phrase ("resets in 4h", "resets now"), built
/// by [`gentle_runtime::usage::reset_label`]: `None` is a provider that reports
/// no reset time, never a countdown into the past.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UsageWindowView {
    pub label: String,
    pub used_percent: f64,
    pub resets_in: Option<String>,
}

/// One named limit of a provider, with its windows. `limit_reached` is the
/// provider's own mark: the pane draws it beside the limit's name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UsageLimitView {
    pub name: String,
    pub limit_reached: bool,
    pub windows: Vec<UsageWindowView>,
}

/// One provider of the subscription usage view: its name, how its last read
/// stands, the plan when it reports one, the source's own failure reason, and
/// its limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UsageProviderView {
    pub provider: String,
    pub status: UsageProviderStatusView,
    pub plan: Option<String>,
    pub error: Option<String>,
    pub limits: Vec<UsageLimitView>,
}

/// Where the subscription usage stands, in the four states the pane draws plus
/// the two waiting ones. `Installed` is the only one that carries providers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum UsageStatusView {
    /// Not read yet.
    #[default]
    Unknown,
    /// The read is running: the pane says so quietly and never blocks.
    Reading,
    /// The sources are installed and answered: `providers` is what they said.
    Installed,
    /// The sources are not installed for this shell; the string is why (a
    /// plain sentence, never a file name).
    Missing(String),
    /// Something that had to be read could not be: the shell's settings, or the
    /// installed sources' own folder. The string is why.
    Unreadable(String),
    /// The sources ran and their answer could not be used: a run that failed, a
    /// document of another shape. The string is why.
    Failed(String),
}

/// The Usage pane's slice (settings-0.1.12 S6's deferred half): the
/// subscription usage the shell's own sources report, read-only and never
/// written. Nothing here touches a chat.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
pub struct UsageView {
    pub status: UsageStatusView,
    pub providers: Vec<UsageProviderView>,
}

/// How full the foreground chat's context is (0.1.12, S5): pi's own
/// `contextUsage` numbers, one reading per chat. Every field is nullable and a
/// null is drawn as nothing at all - never as `0` - because pi answers null
/// right after a compaction.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
pub struct ContextUsageView {
    /// The tokens the session's context holds.
    pub tokens: Option<u32>,
    /// The share of the window in use, in percent, as pi computed it.
    pub percent: Option<f64>,
    /// The model's window, the denominator of the percentage.
    pub window: Option<u32>,
}

/// The folder the foreground chat runs in (design/composer-dock S2, R4): the
/// last segment of its working directory - the name its sidebar row draws - and
/// the whole directory, which is only ever the hover.
///
/// The name is the app's own rule ([`SessionItem::project`](gentle_app::SessionItem::project)),
/// the one the session rows use, so the composer's chip and the sidebar row can
/// never name the same folder differently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct FolderView {
    /// The last segment of the chat's working directory, as the sidebar draws it.
    pub name: String,
    /// The whole working directory: the chip's `title`, never its text.
    pub path: String,
}

/// The branch the foreground chat's working directory is on (design/composer-dock
/// S2, R3, R4): the branch's name, and the directory it was read from, which is
/// only ever the hover.
///
/// There is no "no repository" variant on purpose: a folder outside a work tree,
/// a bare one, a machine without git and every other refusal are all `None` on
/// [`ViewState::foreground_branch`], and a `None` draws no chip at all - no
/// gap, no dash, no placeholder (R3).
///
/// A detached HEAD carries the SHORT sha as its name, because the word git
/// answers there (`HEAD`) names no branch and the chip never lies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct BranchView {
    /// The branch's name as git spells it, or the short sha of a detached HEAD.
    pub name: String,
    /// The working directory the branch was read from: the chip's `title`, never
    /// its text. It is also what ties the answer to the folder beside it.
    pub path: String,
}

/// How complete the session list is, matching `gentle_app::ListStatus` one to
/// one. `Loading` carries how far the first pass got, so the sidebar can say it
/// is still reading; only `Complete` lets an empty list call itself empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum ListStatusView {
    Loading { done: u32, total: u32 },
    Complete,
}

/// One row of the session list, flattened for display. Rows are served a
/// WINDOW at a time ([`SessionsWindow`], S3); the whole list is never pushed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SessionRowView {
    /// The session file path: the id `OpenChat` takes back, which is exactly
    /// what `gentle_app::ChatId::of_session` builds a stored chat's id from.
    pub id: String,
    pub title: String,
    pub project: String,
    pub message_count: u32,
    /// How long ago the session was last written, as the text the sidebar shows
    /// (the rule is [`gentle_runtime::time_ago`]).
    pub modified_ago: String,
}

/// One workspace row of the flattened tree: the folder the user works in, with
/// the state the sidebar draws and whether new chats can start there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WorkspaceRowView {
    /// The folder identity two spellings share
    /// ([`gentle_runtime::app_config::folder_key`]).
    pub key: String,
    /// The name extended with the parent segments it takes to tell two
    /// workspaces with the same name apart.
    pub label: String,
    /// The canonical spelling of the folder.
    pub path: String,
    /// The last segment of the path.
    pub name: String,
    /// Whether the chats of the workspace are folded away (D8).
    pub collapsed: bool,
    /// How many chats the workspace holds, folded or not.
    pub chat_count: u32,
    /// Whether the folder is there right now, so new chats can start in it.
    pub available: bool,
    /// Why a folder that is not available cannot host a chat.
    pub reason: Option<String>,
}

/// The removal offer of one workspace (S1, S2): the chats a removal would
/// delete, and how much they weigh. The count is what the dialog's consequence
/// line names in bold and the bytes are what stands beside it ("2 chats
/// (1.7 MB)"), so the sentence is drawn from these two numbers and never from
/// text built here.
///
/// Both come from the session index, filtered by the folder each session header
/// reports ([`gentle_runtime::app_config::folder_key`]), which is the same
/// identity the sidebar groups by: a removal offers exactly the chats the row
/// shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WorkspaceRemovalView {
    /// The folder identity two spellings share, as [`WorkspaceRowView::key`]
    /// carries it.
    pub key: String,
    /// The canonical spelling of the folder: the config's, when the config holds
    /// it, else the spelling the caller named.
    pub path: String,
    /// The label the sidebar draws for the workspace, so the dialog names it the
    /// way the row does.
    pub label: String,
    /// How many chats the workspace holds: the removal deletes exactly these.
    pub chats: u32,
    /// Their total size in bytes, from the session index. A `f64` because the
    /// generated bindings forbid 64-bit integers on the wire (a `number` that
    /// could lose precision): a byte count is exact in a double far past any
    /// real session store, and the web side is what turns it into "1.7 MB". The
    /// sum of the chats' sizes is always written, so the generated `| null` of
    /// this contract's other measured values never arrives here.
    pub bytes: f64,
}

/// One row of the flattened workspace tree the sidebar draws: a workspace
/// header, then - only when the workspace is not collapsed - its chats.
///
/// The two kinds share one row height on the web side, so the tree is one
/// virtualized list of rows rather than a nested one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind")]
pub enum SessionTreeRow {
    Workspace(WorkspaceRowView),
    Chat(SessionRowView),
}

/// One page of the workspace tree as `sessions_window` answers it (S3): the rows
/// `[offset, offset + length)` of the flattened tree, and how many rows the tree
/// has in total - workspace headers included - so the sidebar knows how far it
/// can scroll.
///
/// The length is capped ([`MAX_WINDOW`](crate::sessions::window::MAX_WINDOW)),
/// so this type never carries the whole list of a 50 000-chat home.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SessionsWindow {
    /// The count of the flattened rows: headers and chats, collapsed groups
    /// skipped, never the page's length.
    pub total: u32,
    pub rows: Vec<SessionTreeRow>,
}

/// Who wrote one message of the thread. The two roles a chat thread displays,
/// matching `gentle_core`'s chat roles one to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum MessageRole {
    User,
    Assistant,
}

/// One image a chat holds, as the composer and the thread draw it.
///
/// It is a NAME and a URL, never a byte of the image: the shell reads the file
/// and serves its thumbnail from the drop's own path, so nothing but a path ever
/// travels up from the page (S4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ImageView {
    /// The id the remove intent takes back.
    pub id: String,
    /// The file's own name, for the thumbnail's accessible text.
    pub name: String,
    /// Where the webview loads this thumbnail from: the shell's own scheme,
    /// served from the file the user dropped.
    pub preview_url: String,
}

/// Where one file the composer attached stands (design/composer-dock T4, R5,
/// R6): an image still travels as an image, every other file travels as a path,
/// and one that cannot be attached says why instead of becoming a chip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum AttachmentKindView {
    /// One of the four image types a message carries: it goes to
    /// `attach_images`, so it still travels as an image and not as a path (R6).
    Image,
    /// Every other file: it becomes a chip, and its absolute path is what the
    /// message carries.
    File,
    /// A path that cannot be attached at all, and the reason, which is the one
    /// line the composer draws in place of a chip.
    Rejected(AttachmentRejectView),
}

/// Why one path the user chose or dropped was not attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
pub enum AttachmentRejectView {
    /// Nothing exists at that path.
    NotFound,
    /// A folder, which is not a file to open.
    IsDirectory,
    /// Something is there but could not be read.
    Unreadable,
}

/// One file the composer attached, as the web side draws it: the whole path
/// (absolute, as the app shows paths elsewhere), the name the chip draws, what
/// it is, and whether it lives outside the chat's folder (R5).
///
/// Nothing here is decided twice: the image rule is [`crate::images::Mime`]'s own
/// and the path is classified once, in [`crate::attachments`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct AttachmentView {
    /// The absolute path, with no Windows verbatim prefix: what the message
    /// carries and what the agent's own tools open. It is also the chip's hover.
    pub path: String,
    /// The last segment of that path: what the chip draws.
    pub name: String,
    /// What this file is, or why it was not attached.
    pub kind: AttachmentKindView,
    /// Whether the file lives outside the chat's folder, so the agent may not be
    /// able to open it. A path with no chat folder is never outside.
    pub outside: bool,
}

/// One message of the foreground chat's thread, flattened for display.
///
/// The three fields after the text are what a streamed thread needs and the
/// text alone cannot offer (T5a): the id the Copy intent names back, whether
/// the message is still growing (no Copy, no "done" mark before it ends), and
/// the provider's failure text when this turn was a model error, which is drawn
/// as a failure instead of a normal reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct MessageView {
    /// `msg-<n>` in the chat's own scheme: what [`UiIntent::CopyMessage`] takes.
    pub id: String,
    pub role: MessageRole,
    pub text: String,
    /// True between the message's start and its end.
    pub streaming: bool,
    /// The provider's own error text when this turn ended in `stopReason:
    /// error`. The text above is then the provider's message, never a reply.
    pub error: Option<String>,
    /// The images this message was sent with, drawn as the same thumbnails the
    /// composer showed (S2). Empty for every message that carried none, which
    /// is every assistant reply and every prompt typed without a drop.
    pub images: Vec<ImageView>,
}

/// One row of the foreground chat's transcript, in the order the child produced
/// it: the text messages, the reasoning blocks and the tool calls the core keeps
/// as ONE ordered list (chat-transcript T2a, L4, S3).
///
/// It is a list of rows and not a tree, and every variant carries the id a keyed
/// view keys the row by. The text variant carries exactly the [`MessageView`]
/// this same push puts in [`ViewState::messages`], built by the same rule, so the
/// field the web reads today and the ordered list can never disagree about a row
/// they both carry while T2 moves the web from one to the other.
///
/// A tool's arguments and output stay the child's own untyped JSON: each tool's
/// shape is its own, and how to show a command or an output is the card's
/// decision, never this contract's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", content = "data")]
pub enum ChatItemView {
    /// A text row: a prompt or a reply, exactly as [`MessageView`] carries it.
    Message(MessageView),
    /// A reasoning block, drawn as a collapsible row of its own.
    Thinking(ThinkingItemView),
    /// One tool call, drawn as one card.
    Tool(ToolCallItemView),
}

/// One reasoning block of the transcript: its text, whether it is still
/// arriving, and how long it ran when the child's clock was known.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ThinkingItemView {
    /// `think-<n>` in the chat's own scheme: the key a list keys the row by.
    pub id: String,
    /// The reasoning text the child wrote.
    pub text: String,
    /// True between the block's start and its end: the row is open while it is,
    /// because the text is still growing.
    pub streaming: bool,
    /// How long the block ran, in milliseconds, from its own start and end
    /// stamps. `None` when either stamp is missing (the `prompt-turn` fixture
    /// carries no timestamps), because a row never invents the duration it has
    /// no evidence for.
    ///
    /// An `f64` rather than the core's `u64` because the generated bindings
    /// refuse a 64-bit integer on the wire - a `number` could lose precision -
    /// and every real millisecond count is exact in a double.
    pub duration_ms: Option<f64>,
}

/// One tool call of the transcript, as its card draws it: the tool's own name,
/// the arguments it was given, the output it had produced while running, the
/// result it ended with, how it stands, and the two marks that say a payload did
/// not arrive whole.
///
/// The three payloads are the child's JSON, untyped and NOT flattened into text:
/// how to show a command, an output or an argument is the card's decision. Each
/// of them is `null` when the call carried none, which the generated bindings
/// state as `unknown` - the wire carries the JSON value itself, and `null` is a
/// value of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ToolCallItemView {
    /// `tool-<toolCallId>`: the key a list keys the row by.
    pub id: String,
    /// The tool's own name ("bash", "read", ...), as the child sent it.
    pub tool_name: String,
    /// The arguments the call was made with, when it carried any.
    #[specta(type = specta_typescript::Unknown)]
    pub args: Option<serde_json::Value>,
    /// The output the tool had produced at its last update, dropped the moment
    /// the `result` that supersedes it arrives: one call keeps one output, so
    /// this is what a RUNNING call has to show and `null` once it ends.
    #[specta(type = specta_typescript::Unknown)]
    pub partial_result: Option<serde_json::Value>,
    /// The result the tool ended with, `null` until it ends.
    #[specta(type = specta_typescript::Unknown)]
    pub result: Option<serde_json::Value>,
    /// The child's own verdict on the result: the card's failure mark.
    pub is_error: bool,
    /// True between the execution's start and its end: the card's "Running"
    /// instead of its "Done".
    pub running: bool,
    /// How long the execution ran, in milliseconds, from its own two stamps;
    /// `None` while it runs and when the clock was not supplied. A `f64` for the
    /// reason [`ThinkingItemView::duration_ms`] gives.
    pub duration_ms: Option<f64>,
    /// True when a payload of this call had a string longer than the core keeps
    /// and was cut to the bound: the card says the output was cut short instead
    /// of drawing the head as the whole of it.
    pub truncated: bool,
    /// True when a FIELD of a payload was lost - cutting one of an object's keys
    /// made it collide with another - which [`truncated`](Self::truncated)
    /// cannot say: the card words the two losses differently.
    pub dropped: bool,
}

/// The first blocking dialog the foreground chat's extension is waiting on,
/// flattened for display: exactly the fields the iced shell draws
/// (`crates/ui/src/view/dialog.rs`), never a `crates/gentle-core` type.
///
/// The app queues dialogs oldest first, and a dialog blocks its chat until it is
/// answered, so the view draws the FIRST pending one at a time; the next arrives
/// in a later slice, once this one is answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind")]
pub enum DialogView {
    /// Yes / No / Cancel, with the question above the buttons.
    Confirm {
        id: String,
        title: String,
        message: String,
    },
    /// One button per option, in the order the extension sent them.
    Select {
        id: String,
        title: String,
        options: Vec<String>,
    },
    /// One line of text; `placeholder` is the hint an empty field shows.
    Input {
        id: String,
        title: String,
        placeholder: Option<String>,
    },
    /// A multiline box; `prefill` is the text it starts with.
    Editor {
        id: String,
        title: String,
        prefill: Option<String>,
    },
}

/// How the user answered a dialog, as the web side sends it back. The three
/// shapes are the child's own protocol (`gentle_core::rpc::UiAnswer`), named
/// here so the contract stays free of that type (S2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum DialogAnswer {
    /// What a select option or a typed value answers.
    Value { value: String },
    /// What a confirm dialog's Yes (`true`) or No (`false`) answers.
    Confirmed { confirmed: bool },
    /// Escape, Cancel or a closed dialog: the extension decides what that means.
    Cancelled,
}

/// The level of a notice, matching `gentle_app`'s notice kinds one to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

/// The one thing the user can do about a notice, when the notice reports
/// something the app can fix by itself (S3). It names its own target, so a
/// notice about a background chat is fixed without opening that chat first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum NoticeActionView {
    /// Start that chat's child again and reopen it: what a death notice offers
    /// as its one action ([`UiIntent::RestartChat`], which takes this id back).
    RestartChat { chat_id: String },
}

/// One notice, flattened for display: id, text, level, and the one action that
/// fixes it when there is one. The id is the one the dismiss intent refers back
/// to. The app lays notices out newest last and keeps the most recent ones,
/// which the view reflects as received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct NoticeView {
    pub id: u32,
    pub text: String,
    pub level: NoticeLevel,
    /// `None` for a notice that only reports something (an extension's own
    /// message), so the view never draws a button that does nothing.
    pub action: Option<NoticeActionView>,
}

/// The status of one delegated helper, matching `gentle_core`'s helper statuses
/// one to one. The TEXT of a status is never built here: it is
/// [`gentle_runtime::helper_text::status_label`]'s, carried beside the enum in
/// [`HelperRowView::status_label`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum HelperStatusView {
    Queued,
    Running,
    Waiting,
    Done,
    Failed,
    Cancelled,
}

/// One card of the helpers panel, flattened for display: what the iced panel
/// draws for a task (`crates/ui/src/view/helpers.rs`) and nothing more. The
/// helper's thread is NOT here - the said/thought/ran view is T7b-2 - so the
/// slice stays a handful of short lines per task, not a transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct HelperRowView {
    /// The id [`UiIntent::SelectHelper`] takes back.
    pub id: String,
    /// The card's headline, from [`gentle_runtime::helper_text::task_title`].
    pub title: String,
    pub agent: String,
    pub status: HelperStatusView,
    /// The status as text, from
    /// [`gentle_runtime::helper_text::status_label`]: it is the card's
    /// accessible name, so the text, not the colour, carries the state.
    pub status_label: String,
    pub turns: u32,
    pub tool_calls: u32,
    /// The step the helper is on or was last on, when it reported one.
    pub last_step: Option<String>,
    /// The failure the helper ended with, when it failed.
    pub error: Option<String>,
}

/// The helpers panel of the foreground chat: the counters line both windows
/// draw, the four counts the working badge is derived from, and one row per
/// task.
///
/// The counters include tasks no longer listed (gentle-agents reports them
/// that way), so the line and the rows never disagree about the counts. The
/// line itself is [`gentle_runtime::helper_text::summary_line`]'s, computed
/// once here; the counts travel beside it because the tab's badge depends on
/// `running` and the web side never parses text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct HelpersView {
    /// The runtime's own counters line, four counts separated as that
    /// rule separates them.
    pub summary_line: String,
    pub running: u32,
    pub queued: u32,
    pub waiting: u32,
    pub finished: u32,
    /// The tasks in the order the state keeps them: running, waiting, queued,
    /// then the finished ones newest first (`gentle_core::helpers::merge_tasks`
    /// orders them that way), so the panel draws what iced draws.
    pub tasks: Vec<HelperRowView>,
}

/// How one tool call of a helper's thread stands. The two booleans the helper
/// state carries are folded into these three by
/// [`gentle_runtime::helper_text::format::tool_status`]; the WORD each state is
/// drawn with stays the view's own, as it is in iced (`running`, `failed`,
/// `ok`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ToolStatusView {
    Running,
    Failed,
    Ok,
}

/// One item of a helper's thread, flattened for display: what the helper said,
/// what it thought, a note it left, or one tool call it ran. Every text is
/// already built by `gentle_runtime::helper_text` and its `format` module, so no
/// `serde_json::Value` crosses the contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind")]
pub enum HelperThreadItemView {
    /// A piece of the helper's own reply, drawn as markdown like the chat
    /// thread's assistant messages.
    Said { text: String },
    /// The helper's reasoning, drawn as a muted block.
    Thought { text: String },
    /// A note the helper left, drawn as plain muted text.
    Note { text: String },
    /// One tool call: its name, how it stands, its arguments on one line and the
    /// first lines of its output, both already truncated by the rules.
    Ran {
        name: String,
        status: ToolStatusView,
        args_line: String,
        output_preview: String,
    },
}

/// The open helper's thread: the header both windows draw and the items the
/// thread scrolls through.
///
/// Only the SELECTED helper of the FOREGROUND chat is here, so this slice never
/// carries another task's transcript. It is bounded too: the push carries the
/// last [`THREAD_ITEMS_PUSHED`](crate::helper_thread::THREAD_ITEMS_PUSHED) items
/// and [`dropped`](Self::dropped) counts the ones left out, so the thread's own
/// "earlier items were dropped" line stays true.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct HelperThreadView {
    /// The id [`HelperRowView::id`] carried and [`UiIntent::SelectHelper`] takes
    /// back.
    pub id: String,
    /// The thread's headline, from [`gentle_runtime::helper_text::task_title`].
    pub title: String,
    pub agent: String,
    pub status: HelperStatusView,
    /// The status as text, from
    /// [`gentle_runtime::helper_text::status_label`]: it is the thread's
    /// accessible name for the status mark, so the text, not the colour, carries
    /// the state.
    pub status_label: String,
    /// The task's prompt, truncated to the length iced echoes at the top.
    pub prompt: String,
    /// How many earlier items the thread does not show: the ones the helper
    /// state itself dropped, plus the ones this push left out of a long thread.
    pub dropped: u32,
    /// The items the thread draws, oldest first.
    pub items: Vec<HelperThreadItemView>,
}

/// The notes of the version this run follows (what's-new S3b): the version the run
/// runs as and the plain text its release carried. It is pushed once, through the
/// same `state-changed` step the notices travel on, and cleared when the modal's
/// dismiss stores the version (S5), so the web side draws it for exactly the run
/// that follows an update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WhatsNewView {
    /// The version this run runs as, never the manifest's (S5).
    pub version: String,
    /// The version the last run stored, which this run followed: the modal says
    /// "since <it>". `None` when the config stored nothing, so the modal never
    /// invents a starting point.
    pub previous_version: Option<String>,
    /// The release notes as plain text: no markup and no locale, so the modal
    /// decides how it lays them out (T2).
    pub notes: String,
}

/// View A's data (welcome-screen T1, S1, S3, R3, R4): the workspace the user
/// worked in last, that workspace's recent chats and the other configured
/// workspaces as quiet links. It is the screen the app shows when NO chat is in
/// the foreground.
///
/// Every field is read from what the sidebar already groups and draws - the
/// config's workspaces and the session list - so the welcome names no folder the
/// sidebar does not name, no chat the sidebar does not list, and counts nothing
/// of its own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WelcomeView {
    /// The workspace of the most recent session, `None` only when the config
    /// holds no workspace at all: there is nothing to be last in.
    pub last: Option<WelcomeWorkspace>,
    /// Up to three chats of [`last`](Self::last), newest first. Empty for a
    /// workspace with no chat, which says so instead of drawing an empty list.
    pub recents: Vec<WelcomeChat>,
    /// The other CONFIGURED workspaces, in the order the config lists them,
    /// without [`last`](Self::last): the quiet links at the foot of the screen.
    pub others: Vec<WelcomeLink>,
}

/// The workspace the user worked in last (welcome-screen R3): the configured
/// workspace that owns the most recent session of the list, with what the
/// sidebar already draws for it and nothing more.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WelcomeWorkspace {
    /// The canonical spelling of the folder, which
    /// [`UiIntent::NewChatInWorkspace`] takes back: the same `path` a sidebar
    /// workspace row carries.
    pub path: String,
    /// The label the sidebar row draws: the folder's name, extended with the
    /// parent segments it takes to tell two workspaces of the same name apart.
    pub label: String,
    /// Whether new chats can start in this folder right now: the very
    /// `available` a sidebar workspace row carries
    /// ([`WorkspaceRowView::available`]), from the same probe. `false` is a
    /// folder that is gone and a path that holds something else instead, so the
    /// primary action is disabled exactly when the sidebar disables it.
    pub available: bool,
    /// Why it cannot, in the shell's own words, exactly as
    /// [`WorkspaceRowView::reason`] carries them for that same folder: the
    /// sidebar's "this folder no longer exists" line for a folder that is gone,
    /// and its other refusal for a path that is not a folder. `None` while the
    /// folder is available, which is when there is nothing to warn about.
    pub reason: Option<String>,
    /// How many sessions the sidebar's tree holds for this workspace, folded
    /// away or not.
    pub chat_count: u32,
    /// The most recent session's timestamp, in epoch milliseconds: what the
    /// screen turns into `hace 21 min`, `None` for a workspace with no chat.
    ///
    /// An `f64` rather than the session's `u64` because the generated bindings
    /// refuse a 64-bit integer on the wire - a `number` could lose precision -
    /// and every real millisecond count is exact in a double (the same rule
    /// [`ThinkingItemView::duration_ms`] follows).
    pub last_activity_ms: Option<f64>,
}

/// One recent chat of the last workspace (welcome-screen R4): a row of the
/// sidebar, with the id that opens it and the age the screen words itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WelcomeChat {
    /// Exactly the string [`UiIntent::OpenChat`] takes back: the RAW session
    /// file path, never the id `gentle_app::ChatId` builds from it (the intent
    /// prefixes it once).
    pub chat_id: String,
    /// The session's stored title, whole: the name the sidebar row beside it
    /// draws for the same session, uncut here because this column is wider - the
    /// page ellipsizes it for the width it really has, so a long name keeps the
    /// room it is given instead of the truncation a narrow sidebar row makes.
    pub title: String,
    /// When the session was last written, in epoch milliseconds: the same value
    /// a chat row's `modified_ago` is worded from, carried as the number so the
    /// screen can measure it against the clock it holds.
    ///
    /// An `f64` for the same reason [`WelcomeWorkspace::last_activity_ms`] is:
    /// the generated bindings refuse a 64-bit integer on the wire, and a real
    /// millisecond count is exact in a double.
    pub modified_ms: f64,
}

/// One other configured workspace, as the quiet link under the recents
/// (welcome-screen T1): the folder, the label the sidebar draws for it, and
/// whether a new chat can start there right now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WelcomeLink {
    /// The canonical spelling of the folder, as [`UiIntent::NewChatInWorkspace`]
    /// takes it back.
    pub path: String,
    /// The label the sidebar row draws.
    pub label: String,
    /// Whether new chats can start in this folder right now: the very
    /// `available` a sidebar workspace row carries
    /// ([`WorkspaceRowView::available`]), from the same probe. `false` is a
    /// folder that is gone and a path that holds something else instead, and a
    /// link that is not available must not offer a new chat: the intent alone
    /// would not refuse it.
    pub available: bool,
}

/// The state the current views render: the foreground chat (as the app names
/// it, the id the sidebar passes back) with its thread, the notices, the
/// per-chat activity the list rows show, and the session list as a status plus
/// a revision. The rows themselves are not here: the sidebar asks
/// `sessions_window` for the page it draws (S3).
///
/// The push size is the notices, the chats that are not idle, the tail of one
/// thread and the foreground chat's helper cards, never the whole `AppState`
/// and never the whole list (S2).
///
/// A chat's stderr reason lines are not a slice of their own: the app keeps them
/// in a bounded ring but exposes them only as the reason of the death notice, so
/// they reach the web side inside [`NoticeView::text`] and nowhere else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ViewState {
    /// The chosen appearance of the whole window, as the app config remembers
    /// it ([`ThemeView`]): the page paints its `data-theme` from it, and the
    /// window's own chrome follows it. `System` until the user chooses.
    pub theme: ThemeView,
    /// The language the app's own words are written in, as the app config
    /// remembers it ([`LanguageView`]): `null` while no preference is stored,
    /// which is when the web side starts from the system's own locale, and the
    /// `"en"`/`"es"` tag once one is. Like the theme, the change is pushed
    /// through the same re-emit, so switching needs no restart.
    pub language: Option<LanguageView>,
    /// When the app notifies the user about a chat that needs them, as the app
    /// config remembers it ([`NotificationsMode`]): `WhenUnfocused` while no
    /// choice is stored, which is the maintainer's default. The web side reads
    /// it before it decides whether to play a tone or send a toast
    /// (notifications S4, S1).
    pub notifications: NotificationsMode,
    /// The shell and home this run sees (shell-choice).
    pub shell: ShellView,
    pub foreground_chat: Option<String>,
    /// The title the chat header draws for the foreground chat: the stored
    /// session's own title, which is the text a sidebar row draws (and the same
    /// source), untruncated here because the header cuts it for its own width and
    /// keeps the whole name for whoever asks. A chat with no session file yet is
    /// named [`NEW_CHAT_TITLE`](crate::sessions::view::NEW_CHAT_TITLE), and no
    /// foreground chat has no title.
    pub foreground_title: Option<String>,
    /// The folder the foreground chat runs in (design/composer-dock S2, R4): the
    /// last segment of its working directory and the whole path, which the
    /// composer draws as its folder chip. Read from the same session row the
    /// header's title reads, so the two can never name one folder differently.
    /// `None` for a chat whose session is not listed yet: an unknown folder is
    /// absent rather than a placeholder.
    pub foreground_folder: Option<FolderView>,
    /// The branch that folder is on (design/composer-dock S2, R3, R4), read once
    /// per working directory by the shell (`git`) and kept only while it was read
    /// from the very directory `foreground_folder` names. `None` when there is no
    /// branch to draw - no repository, a bare one, git not installed, an answer
    /// that has not landed yet - and the composer then draws no chip at all.
    pub foreground_branch: Option<BranchView>,
    /// How complete the session list is. It starts `Loading`, so the sidebar
    /// never claims there are no chats before the first pass ends.
    pub list_status: ListStatusView,
    /// Bumped every time the folded list or its status changes (wrapping), so
    /// the web side knows the page of rows it holds is stale and re-queries
    /// `sessions_window`. A push that does not touch the list leaves it alone.
    pub sessions_revision: u32,
    /// How many sessions the app holds, whatever the search box says: the
    /// unfiltered count, next to the filtered total each window answers with.
    pub total_sessions: u32,
    /// How many chats the app has OPEN right now, whatever they are doing:
    /// every chat whose child this run started, the foreground one included.
    ///
    /// The Updates pane's gate reads it (0.1.12, S4): an update runs only with
    /// every chat closed, and an idle chat in the background is still a child
    /// the update would land under, so the rule counts them all rather than the
    /// foreground one alone.
    pub open_chats: u32,
    /// The foreground chat's messages, oldest first. Message text is the model's
    /// whole markdown reply, so the slice carries the last
    /// [`MESSAGES_PUSHED`](crate::view::MESSAGES_PUSHED) of them and never a
    /// whole long session.
    pub messages: Vec<MessageView>,
    /// The foreground chat's transcript, oldest row first: every row the chat
    /// keeps - the text messages, the reasoning blocks and the tool calls - in
    /// the order the child produced them (chat-transcript T2a, L4).
    ///
    /// It is the tail, and the SAME limit [`messages`](Self::messages) uses:
    /// the last [`MESSAGES_PUSHED`](crate::view::MESSAGES_PUSHED) rows of ANY
    /// kind, so a chat of hundreds of tool calls pushes a bounded list and the
    /// payloads the core keeps cannot turn one refresh into megabytes. A text
    /// row that both lists carry is the same row: `messages` is this list's text
    /// rows out of one builder, which is what lets the web move from the one to
    /// the other without the two disagreeing on the way.
    pub items: Vec<ChatItemView>,
    /// Whether [`items`](Self::items) and [`messages`](Self::messages) are the
    /// PROVISIONAL rows read from the chat's own session FILE, painted while its
    /// child is still starting (`chat-open-latency.md` T1, T2).
    ///
    /// True only while the thread draws those rows, and those rows are drawn only
    /// for the foreground chat, only while its child has answered nothing and its
    /// thread holds no row of its own. The child's `get_messages` answer replaces
    /// them and always wins (S4), so this flag turns false in the same push that
    /// carries the real rows - never one frame later.
    ///
    /// While it is true a row may be DRAWN but not ACTED on: the rows are the
    /// file's, not the child's, so the thread gives them no Copy and nothing else
    /// that assumes the chat owns them.
    pub transcript_provisional: bool,
    pub notices: Vec<NoticeView>,
    /// The foreground chat's first pending blocking dialog, if an extension is
    /// waiting for an answer. `None` when no chat is foreground or nothing is
    /// pending, which is also what lets the composer work.
    pub dialog: Option<DialogView>,
    /// The foreground chat's delegated helpers: the counters line a chat with no
    /// helpers still shows, and one row per task. `None` only when no chat is
    /// foreground (the window draws no Chat | Helpers tabs then).
    pub helpers: Option<HelpersView>,
    /// The helper whose thread is open in the panel, as the reducer remembers
    /// it. It only ever names a card [`helpers`](Self::helpers) lists: the
    /// reducer selects a task of the foreground chat and forgets the selection
    /// when another chat becomes the foreground one. `None` for the list
    /// itself, and always with no foreground chat.
    pub selected_helper: Option<String>,
    /// The open helper's thread, and only that one: the selected task of the
    /// foreground chat, with the header iced draws and its items. `None` when no
    /// helper is selected, when no chat is foreground, or when the selected id
    /// names no task of that chat (a task the next frame no longer reports).
    pub helper_thread: Option<HelperThreadView>,
    pub activity: Vec<ChatActivityView>,
    /// The one failure that ends the whole run instead of one chat: the
    /// `gentle-shell` launcher could not be resolved, so no chat can ever
    /// start. Drawn as the bar above everything
    /// (`crates/ui/src/view/banner.rs`), never as a notice. `None` for every
    /// run whose launcher resolved, which is every run that has not tried yet.
    pub fatal: Option<String>,
    /// What the user should be told about the app config file: a file that
    /// could not be used at startup is kept aside and the app starts with an
    /// empty config, a file that could not be written leaves the change in
    /// memory only, and a run whose settings folder could not be found says so
    /// because nothing will be written at all. `None` when the file was read, or
    /// was not there, and every save worked.
    pub config_notice: Option<String>,
    /// The foreground chat's images, waiting for its send: drawn in the composer
    /// as thumbnails with a remove button, and never sent by themselves (S2).
    /// `None`/empty for a chat with none, and for no chat at all.
    pub images: Vec<ImageView>,
    /// Why the last drop was refused, as one line of plain text: a file that is
    /// not one of the four image types, one that is no longer there, or one that
    /// would pass the eight-image or 64 MB cap (S1). `None` when nothing was
    /// refused. A shell-owned line like [`ViewState::workspace_notice`]: the
    /// shell reads the files, so the shell is what says no.
    pub images_notice: Option<String>,
    /// What the composer's model chip shows, for the foreground chat's child
    /// (0.1.11). A no-foreground-chat run shows nothing, so the chip is part of
    /// the chat panel, not the window.
    pub model_chip: ModelChipView,
    /// How full the foreground chat's context is (0.1.12, S5), from that chat's
    /// own `get_session_stats` reply. Every field is null when there is no
    /// reading, and null is never drawn as zero.
    pub context_usage: ContextUsageView,
    /// The Profile tab's view of the FOREGROUND chat (profiles-editor T2b-2,
    /// L11): which profile this chat uses, the definitions, their chains and the
    /// pin of its own worktree, as that chat's own child answered it.
    ///
    /// Shell-owned, like [`auth`](Self::auth): the reducer knows nothing
    /// about a session file or a child's config store. A read belongs to the one
    /// chat [`ChatProfileView::chat_id`] names and is carried only while that
    /// chat is the foreground one, so a read that lands after the user moved on
    /// never draws on the wrong chat. `None` until a read answered, for a chat
    /// that is not the foreground one, and for every chat without a view of its
    /// own.
    pub chat_profile: Option<ChatProfileView>,
    /// The SESSION permission mode of the FOREGROUND chat (composer-dock T3b,
    /// L7): the mode that chat's own child reported, in the channel's own string,
    /// with the refusal of the last attempt that wrote nothing and the number of
    /// write attempts answered so far.
    ///
    /// Shell-owned, like [`chat_profile`](Self::chat_profile): the reducer knows
    /// nothing about a permission mode. It is carried only while its mode is
    /// KNOWN and the chat it was read for is the foreground one, so a late answer
    /// never draws on the wrong chat and a chat nothing answered draws no control
    /// at all. `None` until an exchange answers, and for every chat that is not
    /// the foreground one.
    pub session_mode: Option<SessionModeView>,
    /// Where the shell's authentication stands (0.1.12, T3): provider names and
    /// their own check, never a key.
    pub auth: AuthView,
    /// The versions this machine runs and the newer ones published (0.1.12,
    /// S2, S7). Checked once when the app opens.
    pub updates: UpdatesView,
    /// The subscription usage the shell's own sources report, read when the
    /// Usage section is opened and never from a chat.
    pub usage: UsageView,
    /// Why the last workspace action was refused, as one line of plain text: a
    /// name Windows would not accept, a folder that is not there, a folder that
    /// already holds files (Q6). `None` when nothing was refused, and cleared by
    /// the next workspace action that works. A shell-owned line, never the
    /// config's: the config notice says what became of the file, this one says
    /// what the user asked for that could not be done. It also carries the one
    /// report a delete leaves: the current workspace had its last chat removed,
    /// so its row is gone from the tree (T4, D3(a)).
    pub workspace_notice: Option<String>,
    /// Why the last session action was refused, as one line of plain text: a chat
    /// that is working (its child is appending to the same session file) on a
    /// rename or a delete, a name that is empty or only spaces, or a session file
    /// the disk would not take or no longer holds. `None` when nothing was
    /// refused, and cleared by the next session action that works. A shell-owned
    /// line, like [`workspace_notice`](Self::workspace_notice): no reducer path
    /// records one.
    pub sessions_notice: Option<String>,
    /// The notes of the version this run follows (what's-new S3b), pushed once the
    /// fetch of `latest.json` came back with this version. `None` for every run
    /// that follows no update, for one whose notes could not be read, and once the
    /// modal is dismissed: the dismiss is what stores the version and clears it
    /// (S5), which is what makes the modal show once per version.
    pub whats_new: Option<WhatsNewView>,
    /// The version this run RUNS as, which the bridge names once at startup
    /// (`bridge::whats_new`): what the Settings dialog's About section draws
    /// (settings-dialog S5) and the version the dismiss stores as the last run's,
    /// never the manifest's. `None` for a run that never named one, so the section
    /// says it does not know instead of guessing.
    pub app_version: Option<String>,
    /// View A's data (welcome-screen T1, S1, S3, R3, R4): the workspace the user
    /// worked in last, its recent chats and the other configured workspaces.
    ///
    /// `Some` ONLY while no chat is in the foreground, which is the one state
    /// that draws the welcome at all: with a chat on screen the field costs
    /// nothing and the web side draws none of it. It is never `None` for a
    /// missing workspace or an empty list - those are cases of the view itself
    /// ([`WelcomeView::last`]), not an absent screen.
    pub welcome: Option<WelcomeView>,
}

/// The state slice after each dispatch, emitted to the web side (typed
/// tauri-specta event). The name on the wire is kebab-case by `NAME` below.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StateChanged(pub ViewState);

// Manual `Event` implementation instead of the derive: the `derive` feature
// is not enabled (it would pull the proc-macro crate in); the trait needs
// only the wire name, every other method has a default.
impl tauri_specta::Event for StateChanged {
    const NAME: &'static str = "state-changed";
}

/// Which of the Settings files changed on disk since the last notice
/// (rpc-settings-sync T3b-1, S2): a group is on when at least one file of it was
/// written, replaced or deleted during one quiet window of the native watcher.
///
/// The payload names GROUPS, never paths: the panes own the files they read, and
/// the app side never tells the web side a location. A batch that touches both
/// files of the Profiles group and the auth store carries both, so one notice
/// covers it.
///
/// The Profiles group is kept although no pane subscribes to it since
/// profiles-editor T2d removed Settings > Profiles: it is the watcher's own
/// description of the config home (the same directories, the same debouncer, one
/// group per file set), and the chat's Profile tab may take it when the pin of a
/// worktree starts to be read live (T6). Dropping it would touch the watcher's
/// tested core for no observable change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SettingsFilesChangedView {
    /// `profiles.json` or its sibling `profile-fallbacks.json` in the config
    /// home: the files gentle-pi and the terminal write. Watched, and with no web
    /// listener since T2d.
    pub profiles: bool,
    /// `auth.json` in the agent directory: the Account pane's file.
    pub account: bool,
}

/// One notice that pi's Settings files changed (typed tauri-specta event, named
/// by [`SETTINGS_FILES_CHANGED`] below). It is emitted on its own rather than on
/// the `state-changed` slice: the files are NOT part of `ViewState`, and the
/// panes are the ones that decide to re-read through their own refresh
/// commands, so the notice carries no state at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SettingsFilesChanged(pub SettingsFilesChangedView);

/// The name of the [`SettingsFilesChanged`] event on the wire.
pub const SETTINGS_FILES_CHANGED: &str = "settings-files-changed";

impl tauri_specta::Event for SettingsFilesChanged {
    const NAME: &'static str = SETTINGS_FILES_CHANGED;
}

/// Sample contract command. Trivial by design: it exists so the generator
/// proves that command names, camel-cased argument names and return types are
/// typed over the same Rust definition, not re-declared by hand.
#[tauri::command]
#[specta::specta]
pub fn get_badge(chat_id: String) -> ChatBadge {
    ChatBadge {
        chat_id,
        activity: ChatActivityMark::Working,
        unread: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The event the watcher's batch travels on is pinned by its wire shape: the
    /// name the web side listens on and the two flags it reads, and nothing else
    /// - no path, no file name, no revision.
    #[test]
    fn the_settings_files_changed_event_is_two_group_flags() {
        assert_eq!(SETTINGS_FILES_CHANGED, "settings-files-changed");
        let changed = SettingsFilesChanged(SettingsFilesChangedView {
            profiles: true,
            account: false,
        });
        assert_eq!(
            serde_json::to_value(changed).expect("the event serializes"),
            serde_json::json!({ "profiles": true, "account": false })
        );
        assert_eq!(
            serde_json::from_str::<SettingsFilesChangedView>(
                r#"{"profiles":false,"account":true}"#
            )
            .expect("the payload the web side reads"),
            SettingsFilesChangedView {
                profiles: false,
                account: true,
            }
        );
    }
}
