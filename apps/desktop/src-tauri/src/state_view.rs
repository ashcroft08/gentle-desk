//! The `ViewState` slice of one `AppState`: the pure rule that decides what the
//! web side sees (S2).
//!
//! Every field is a slice the state really holds, never the whole `AppState`:
//! the foreground chat named as the app names it, the tail of its thread, the
//! first dialog its extension waits on, the notices flattened, the chats that
//! are not idle, its helpers panel and the open helper's thread, and the session
//! list as a status, a total and a revision instead of rows - the rows are
//! served one window at a time (S3). What the app state does NOT carry comes in
//! beside it ([`ShellParts`]): the config's notice, the fatal launcher failure
//! (the state has no process at all, `bridge::fatal`), the line a refused
//! workspace action left (`bridge::notice`), the images a chat holds or sent,
//! which the shell reads and the reducer never sees (`bridge::images`), the
//! version this run runs as, and the notes it follows (`bridge::whats_new`).

use gentle_app::{AppState, ModelsStatus};
use gentle_core::chat::ChatMessage;
use gentle_core::i18n::Language;
use gentle_runtime::app_config::Theme;
use gentle_runtime::badge::{Badge, BadgeMark};

use crate::contract::{
    AuthView, BranchView, ChatActivityMark, ChatActivityView, ChatProfileView, ContextUsageView,
    FolderView, ImageView, LanguageView, ModelChipView, ModelChoiceView, ModelListStatusView,
    NoticeActionView, NoticeLevel, NoticeView, SessionModeView, ShellView, ThemeView, UpdatesView,
    UsageView, ViewState, WelcomeView, WhatsNewView,
};
use crate::images::Images;
use crate::{dialogs, helper_thread, helpers, sessions, view};

/// Everything the slice carries that the app state does not: the shell's own
/// lines, the theme of this run, what the app found about the shell, and the
/// images the chats hold.
///
/// One value rather than six arguments, so [`view_state`] reads as what it is
/// (the state plus what the shell knows) instead of a wall of parameters.
pub struct ShellParts<'a> {
    /// What became of the config file: read, or not.
    pub config_notice: Option<String>,
    /// Why the last workspace action was refused.
    pub workspace_notice: Option<String>,
    /// Why the last session action was refused (T2's rename).
    pub sessions_notice: Option<String>,
    /// The notes of the version this run follows (what's-new S3b), `None` until
    /// the fetch answers with this version and after the modal is dismissed.
    pub whats_new: Option<WhatsNewView>,
    /// The version this run runs as (settings-dialog S5), `None` until the
    /// bridge names it at startup: the About section draws it, and it is what
    /// the dismiss stores as the last run's.
    pub app_version: Option<String>,
    /// The run's one fatal launcher failure.
    pub fatal: Option<String>,
    /// The theme this run holds, as the app config remembers it.
    pub theme: Theme,
    /// The language this run holds, as the app config remembers it: `None` while
    /// no preference is stored, which is when the web side starts from the
    /// system's own locale.
    pub language: Option<Language>,
    /// The shell and home this run sees.
    pub shell: ShellView,
    /// The Profile tab's view of one chat (profiles-editor T2b-2, L11), as the
    /// shell holds it: it belongs to the chat its own `chat_id` names.
    pub chat_profile: Option<ChatProfileView>,
    /// The session permission mode of one chat (composer-dock T3b, L7), as the
    /// shell holds it: the chat it was read for, and what that chat's own child
    /// answered. The chat is carried beside the view for the same reason
    /// [`chat_profile`](Self::chat_profile) carries its own: the two are compared
    /// to the foreground chat here before anything is drawn.
    pub session_mode: Option<(String, SessionModeView)>,
    /// The rows read from the foreground chat's session FILE while its child
    /// starts (`bridge::transcript`, chat-open-latency T1), with the chat they
    /// were read for. Whether they may be drawn is this module's rule, below.
    pub transcript: Option<(String, Vec<ChatMessage>)>,
    /// The branch the foreground chat's folder is on (design/composer-dock S2,
    /// R3, R4), as the shell's git cache holds it. It arrives with the directory
    /// it was read from, so a read that lands after the user moved to another
    /// chat is dropped instead of labelling the wrong folder.
    pub branch: Option<BranchView>,
    /// Where the shell's authentication stands (0.1.12, T3).
    pub auth: AuthView,
    /// The versions this machine runs (0.1.12, S2, S7).
    pub updates: UpdatesView,
    /// The subscription usage the shell's own sources report, read on the Usage
    /// pane's behalf.
    pub usage: UsageView,
    /// The images each chat holds or sent.
    pub images: &'a Images,
    /// The folder a NEW chat was opened in (composer-dock L12), with the chat it
    /// belongs to: the chip names the workspace's folder while the chat's own
    /// session row does not exist yet. The chat is carried for the same gate the
    /// other per-chat slices keep here.
    pub folder_hint: Option<(String, crate::contract::FolderView)>,
    /// View A's data (welcome-screen T1, S1, S3, R3, R4), built by the shell from
    /// the config and the tree of this revision: the last workspace, its recent
    /// chats and the other configured workspaces. It is read whatever the
    /// foreground chat is, because a push that CLOSES the last chat is the one
    /// that has to bring the screen back; whether it is carried at all is this
    /// module's foreground gate ([`view_state`]).
    pub welcome: WelcomeView,
}

/// The slice of `state` as it stands now, carrying the revision the holder
/// keeps for the session list (the web side compares it to know when the page of
/// rows it holds is stale) and everything the shell owns ([`ShellParts`]).
pub fn view_state(state: &AppState, sessions_revision: u32, shell: ShellParts<'_>) -> ViewState {
    let ShellParts {
        config_notice,
        workspace_notice,
        sessions_notice,
        whats_new,
        app_version,
        fatal,
        theme,
        language,
        shell,
        chat_profile,
        session_mode,
        transcript,
        branch,
        auth,
        updates,
        usage,
        images,
        folder_hint,
        welcome,
    } = shell;
    let helpers_view = helpers::helpers(state);
    let foreground = state
        .chats
        .foreground()
        .map(|chat| chat.as_str().to_owned());
    // The Profile tab's view is the foreground chat's and nobody else's, the same
    // rule the images follow (profiles-editor T2b-2, L11): a read that lands after
    // the user moved to another chat never draws on the wrong one.
    let chat_profile = match (chat_profile, foreground.as_deref()) {
        (Some(view), Some(chat)) if view.chat_id == chat => Some(view),
        _ => None,
    };
    // The session mode is the foreground chat's and nobody else's, by the same
    // rule (composer-dock T3b, L7): an answer that lands after the user moved to
    // another chat never labels the wrong one.
    let session_mode = match (session_mode, foreground.as_deref()) {
        (Some((chat_id, view)), Some(chat)) if chat_id == chat => Some(view),
        _ => None,
    };
    // The folder and the branch it is on are one answer (design/composer-dock S2,
    // R3, R4): the branch is kept only while it was read from the very directory
    // the folder chip names, so a read still in flight for the chat the user just
    // left never labels the one they are looking at.
    let foreground_folder = foreground_folder(state).or_else(|| {
        // A chat opened before its session exists names the folder the
        // workspace handed it (composer-dock L12): the hint is the fallback,
        // only for the chat it belongs to.
        match (&folder_hint, foreground.as_deref()) {
            (Some((chat, view)), Some(foreground)) if chat == foreground => Some(view.clone()),
            _ => None,
        }
    });
    let foreground_branch = branch_of(foreground_folder.as_ref(), branch);
    // The rows read from the chat's session file are the thread only while that
    // chat's child has answered nothing and the reducer holds no row of its own:
    // the child's answer always wins (S4, chat-open-latency T1).
    let provisional = provisional_rows(state, transcript);
    let (messages, items, transcript_provisional) = match provisional {
        Some(rows) => {
            let (messages, items) = view::provisional(&rows);
            (messages, items, true)
        }
        None => (
            view::messages(state, images),
            view::items(state, images),
            false,
        ),
    };
    // View A is the screen of NO foreground chat and of no other state
    // (welcome-screen R1): with a chat on screen the web side draws none of it,
    // so the slice does not carry it either.
    let welcome = foreground.is_none().then_some(welcome);
    ViewState {
        theme: view_theme(theme),
        language: view_language(language),
        shell,
        foreground_chat: foreground.clone(),
        foreground_title: state
            .chats
            .foreground()
            .map(|chat| sessions::view::header_title(state, chat)),
        foreground_folder,
        foreground_branch,
        list_status: sessions::view::status(state),
        sessions_revision,
        total_sessions: sessions::view::total(state),
        // Every chat whose child this run started, idle ones included: the
        // Updates pane's chats-closed gate reads exactly this (0.1.12, S4).
        open_chats: u32::try_from(state.chats.len()).unwrap_or(u32::MAX),
        messages,
        items,
        transcript_provisional,
        notices: state.notices.iter().map(NoticeView::of).collect(),
        dialog: dialogs::dialog(state),
        helpers: helpers_view,
        selected_helper: state.selected_helper.clone(),
        helper_thread: helper_thread::thread(state),
        activity: activity_view(state),
        images: foreground
            .map(|chat| {
                images
                    .pending(&chat)
                    .into_iter()
                    .map(ImageView::from)
                    .collect()
            })
            .unwrap_or_default(),
        images_notice: images.notice(),
        model_chip: model_chip_view(state),
        context_usage: context_usage_view(state),
        chat_profile,
        session_mode,
        auth,
        updates,
        usage,
        config_notice,
        workspace_notice,
        sessions_notice,
        whats_new,
        app_version,
        fatal,
        welcome,
    }
}

/// The folder the foreground chat runs in (design/composer-dock S2, R4), read
/// from the session row the header's title also reads.
///
/// The name is `SessionItem::project`, the very rule a sidebar row applies, so
/// the composer's chip and the row can never name the same folder differently;
/// the whole directory travels beside it, for the hover only. A chat whose
/// session file is not listed yet (one started a moment ago) has no folder to
/// name, so it has none - absent, never a placeholder (R3's rule).
///
/// `pub(crate)` because the shell reads the same directory to ask git about it
/// (`bridge::AppStateHolder::branch_view`): one lookup, so the branch chip and the
/// folder chip can never name two different folders.
pub(crate) fn foreground_folder(state: &AppState) -> Option<FolderView> {
    let chat = state.chats.foreground()?;
    let session = state
        .sessions
        .iter()
        .find(|session| Some(session.path.as_str()) == chat.session_path())?;
    Some(FolderView {
        name: session.project().to_owned(),
        path: session.cwd.clone(),
    })
}

/// The branch chip's slice: the cached answer, kept only while it was read from
/// the very directory the folder chip beside it names (design/composer-dock S2,
/// R3, R4).
///
/// Both strings come from one field of one session row, so comparing them for
/// equality is exact: an answer whose directory is not the folder's is a read
/// that landed after the user moved on, and it is dropped rather than drawn.
fn branch_of(folder: Option<&FolderView>, branch: Option<BranchView>) -> Option<BranchView> {
    let folder = folder?;
    branch.filter(|branch| branch.path == folder.path)
}

/// The composer's model chip from the app state, one to one. A no-foreground
/// state shows nothing, so every field is simply the app's value.
fn model_chip_view(state: &AppState) -> ModelChipView {
    ModelChipView {
        status: match &state.models_status {
            ModelsStatus::Unknown => ModelListStatusView::Unknown,
            ModelsStatus::Loading => ModelListStatusView::Loading,
            ModelsStatus::Ready => ModelListStatusView::Ready,
            ModelsStatus::Empty => ModelListStatusView::Empty,
            ModelsStatus::Failed(error) => ModelListStatusView::Failed(error.clone()),
        },
        models: state
            .models
            .iter()
            .map(|m| ModelChoiceView {
                provider: m.provider.clone(),
                model_id: m.model_id.clone(),
                name: m.name.clone(),
                reasoning: m.reasoning,
            })
            .collect(),
        current: state.model.as_ref().map(|m| ModelChoiceView {
            provider: m.provider.clone(),
            model_id: m.model_id.clone(),
            name: m.name.clone(),
            reasoning: m.reasoning,
        }),
        thinking_level: state.thinking_level.clone(),
        thinking_levels: state.thinking_levels.clone(),
    }
}

/// How full the foreground chat's context is (0.1.12, S5): the reading its own
/// child last reported. No foreground chat, or a chat that has not answered yet,
/// is every field null - which the chip draws as nothing, never as `0%`.
fn context_usage_view(state: &AppState) -> ContextUsageView {
    let usage = state
        .chats
        .foreground()
        .and_then(|chat| state.chats.context(chat));
    match usage {
        // The contract narrows both counts to `u32` (a model's window and a
        // session's token count are far below it); a number that somehow did not
        // fit is unknown rather than a wrapped lie.
        Some(usage) => ContextUsageView {
            tokens: usage.tokens.and_then(|tokens| u32::try_from(tokens).ok()),
            percent: usage.percent,
            window: usage
                .context_window
                .and_then(|window| u32::try_from(window).ok()),
        },
        None => ContextUsageView::default(),
    }
}

/// The theme as the contract names it: the two enums carry the same six names,
/// so the value never travels as a string that could drift from the stylesheet.
fn view_theme(theme: Theme) -> ThemeView {
    match theme {
        Theme::System => ThemeView::System,
        Theme::Dark => ThemeView::Dark,
        Theme::Gentle => ThemeView::Gentle,
        Theme::GentlemanCute => ThemeView::GentlemanCute,
        Theme::GentlemanSexy => ThemeView::GentlemanSexy,
        Theme::Light => ThemeView::Light,
    }
}

/// The language as the contract names it: the two tags are the ones the config
/// stores and the web's dictionaries are keyed by, so the value never travels as
/// a string that could drift from them. `None` while no preference is stored,
/// which is what the web side paints the system's own locale for.
fn view_language(language: Option<Language>) -> Option<LanguageView> {
    language.map(|language| match language {
        Language::En => LanguageView::En,
        Language::Es => LanguageView::Es,
    })
}

/// The marks of the chats with something to show, in id order. The rule that
/// turns a chat's activity into a mark is `crates/runtime`'s (`Badge::on`), so
/// an idle chat is simply absent and this list stays as small as the chats that
/// are working, waiting or failed.
fn activity_view(state: &AppState) -> Vec<ChatActivityView> {
    state
        .chats
        .ids()
        .into_iter()
        .filter_map(|chat| {
            let badge = Badge::on(state.chats.activity(chat)?)?;
            Some(ChatActivityView {
                chat_id: chat.as_str().to_owned(),
                activity: view_mark(badge.mark),
            })
        })
        .collect()
}

/// The badge mark as the contract names it: the two enums carry the same six
/// meanings, and the contract's names are the ones already on the wire.
fn view_mark(mark: BadgeMark) -> ChatActivityMark {
    match mark {
        BadgeMark::Working => ChatActivityMark::Working,
        BadgeMark::NewReply => ChatActivityMark::NewReply,
        BadgeMark::Queued => ChatActivityMark::Queued,
        BadgeMark::Opening => ChatActivityMark::Opening,
        BadgeMark::NeedsAttention => ChatActivityMark::NeedsAttention,
        BadgeMark::Ended => ChatActivityMark::Ended,
    }
}

/// The rows read from the foreground chat's session FILE, kept only while they
/// may be drawn (chat-open-latency T1, S4).
///
/// Four things have to hold at once, and all four are the app state's own (read
/// here, in the one place that has it):
///
/// - the rows were read for the chat that is on screen NOW, so a read that landed
///   after the user moved to another chat never draws on the wrong one;
/// - they hold something to draw;
/// - the chat's thread holds NO row of its own, because a row the reducer has is
///   the chat's own truth and the file's is a fallback;
/// - the chat's child has not answered `get_messages`, because that answer IS the
///   history and a file read that lands after it is never drawn - nor, further
///   back, re-installed ([`AppStateHolder::note_transcript`]).
fn provisional_rows(
    state: &AppState,
    transcript: Option<(String, Vec<ChatMessage>)>,
) -> Option<Vec<ChatMessage>> {
    let (chat_id, rows) = transcript?;
    if rows.is_empty() {
        return None;
    }
    let chat = state
        .chats
        .foreground()
        .filter(|chat| chat.as_str() == chat_id)?;
    if state.chats.history_loaded(chat) {
        return None;
    }
    if !state.chats.state(chat)?.items.is_empty() {
        return None;
    }
    Some(rows)
}

impl NoticeView {
    /// Flattens one app notice for display.
    pub(crate) fn of(notice: &gentle_app::Notice) -> Self {
        Self {
            id: view_notice_id(notice),
            text: notice.text.clone(),
            level: view_notice_level(notice.level),
            action: notice.action.as_ref().map(view_action),
        }
    }
}

/// The one action a notice may offer, as the contract names it. The chat id is
/// the one the restart intent takes back, exactly as `foreground_chat` carries
/// it, so a background chat's notice fixes that chat.
fn view_action(action: &gentle_app::NoticeAction) -> NoticeActionView {
    match action {
        gentle_app::NoticeAction::RestartChat(chat) => NoticeActionView::RestartChat {
            chat_id: chat.as_str().to_owned(),
        },
    }
}

/// The id the dismiss intent refers back to. The app's ids are a never-reused
/// counter that stays far below `u32::MAX` in practice; the contract narrows
/// both sides of the id to `u32`, and the saturating value would mark a
/// collapse, never silently aliasing another notice (each id is unique).
fn view_notice_id(notice: &gentle_app::Notice) -> u32 {
    u32::try_from(notice.id).unwrap_or(u32::MAX)
}

/// The contract's level enum matches the app's notice kinds one to one.
fn view_notice_level(level: gentle_app::NoticeLevel) -> NoticeLevel {
    match level {
        gentle_app::NoticeLevel::Info => NoticeLevel::Info,
        gentle_app::NoticeLevel::Warning => NoticeLevel::Warning,
        gentle_app::NoticeLevel::Error => NoticeLevel::Error,
    }
}
