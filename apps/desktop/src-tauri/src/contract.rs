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
    /// The install is running. `line` is the last line the command printed, as
    /// the command printed it (normalised only where npm redraws a line), so the
    /// screen can say what is happening. `None` until it says anything: nothing
    /// here turns that line into a percentage or an estimate, because nobody
    /// knows one.
    Running {
        line: Option<String>,
    },
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
    /// The foreground chat's messages, oldest first. Message text is the model's
    /// whole markdown reply, so the slice carries the last
    /// [`MESSAGES_PUSHED`](crate::view::MESSAGES_PUSHED) of them and never a
    /// whole long session.
    pub messages: Vec<MessageView>,
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
    /// empty config, and a file that could not be written leaves the change in
    /// memory only. `None` when the file was read, or was not there, and nothing
    /// failed.
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
    /// Why the last workspace action was refused, as one line of plain text: a
    /// name Windows would not accept, a folder that is not there, a folder that
    /// already holds files (Q6). `None` when nothing was refused, and cleared by
    /// the next workspace action that works. A shell-owned line, never the
    /// config's: the config notice says what became of the file, this one says
    /// what the user asked for that could not be done.
    pub workspace_notice: Option<String>,
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
