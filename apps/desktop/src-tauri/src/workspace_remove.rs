//! Removing a workspace, and the chats it holds (S1, S2, S4, S5, S7).
//!
//! One rule, in the order the maintainer ruled on 2026-11-16: a workspace whose
//! chat is running a turn is refused WHOLE (S1) with the working-chat wording the
//! app already uses; otherwise the open, idle children of the workspace are
//! closed first (S5), then their session files are deleted trash-first (S4), and
//! only then is the bookmark forgotten. A workspace with no chat is forgotten
//! without touching the disk at all, and a delete the disk refuses stops the
//! removal before the bookmark goes: a workspace that still holds a chat is not
//! removed.
//!
//! The working flag is read TWICE: once for the whole removal, and again before
//! each file, because the closes take time and a chat that starts a turn in that
//! gap must not lose its file under its own turn (S1). The second read refuses
//! the rest of the flow with the same wording and the same outcome as the first.
//!
//! Why that order (S5): the file must not be deleted under a child that still
//! holds it open, so the close is run TO ITS END before the first `remove_file`,
//! and the chat leaves the open set on the way, which drops the cached session
//! the sidebar would otherwise keep drawing.
//!
//! These are COMMANDS and not `UiIntent` variants, as the delete and the rename
//! beside them: the removal waits for a human (the row's menu opens the removal
//! dialog) and it writes the disk. The offer is a command too: it reads the
//! session index for the one workspace the dialog is about, which is what lets
//! the dialog say "2 chats (1.7 MB)" before anything happens (S2).

use std::path::PathBuf;
use std::sync::Arc;

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::session_delete::{self, DiskRemovals, SessionRemovals};
use gentle_core::i18n::messages;
use gentle_runtime::app_config::folder_key;
use gentle_runtime::session_refresh::Shared;
use tauri::AppHandle;

use crate::bridge::AppStateHolder;
use crate::chat_rename::WorkingChats;
use crate::commands::{emit_state, Bridge};
use crate::contract::{UiIntent, ViewState, WorkspaceRemovalView};

/// One chat of a workspace as the removal sees it: the session file it deletes
/// and the size the offer adds up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatFile {
    pub path: PathBuf,
    pub bytes: u64,
}

impl ChatFile {
    /// The session path as text: the id the app names a stored chat by, which is
    /// what [`WorkingChats`] and [`CloseFirst`] take.
    pub fn session(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

/// The chats of one workspace: their session files and sizes, from the session
/// index (S2). A seam, so a test needs no pi home and no index.
pub trait WorkspaceChats {
    fn chats(&self, workspace: &str) -> Vec<ChatFile>;
}

/// The real answer: the index of this run, filtered by the folder each session
/// header reports. The identity is [`folder_key`] - the rule the sidebar groups
/// by - so a removal offers and deletes exactly the chats its row shows.
pub struct IndexChats<'a>(pub Option<&'a Shared>);

impl WorkspaceChats for IndexChats<'_> {
    fn chats(&self, workspace: &str) -> Vec<ChatFile> {
        // A run whose pi home is unknown has no index and so no chats: nothing
        // is invented for it.
        let Some(shared) = self.0 else {
            return Vec::new();
        };
        let key = folder_key(workspace);
        let index = shared
            .index
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        index
            .entries()
            .into_iter()
            .filter(|entry| folder_key_of(&entry.summary.cwd).as_deref() == Some(key.as_str()))
            .map(|entry| ChatFile {
                path: entry.path,
                bytes: entry.bytes,
            })
            .collect()
    }
}

/// Closing a chat before its file goes (S5). The app's own Close chat, so the
/// chat leaves the open set and its cached session is dropped; a test records
/// what was closed and in which order.
pub trait CloseFirst {
    fn close(&self, session: &str);
}

/// The shell's own close, as a value a blocking thread can carry: the holder the
/// reducer folds into, and what runs a chat command to its end
/// ([`Bridge::close_first`] gives the real one).
pub struct ShellClose {
    holder: Arc<AppStateHolder>,
    run: Arc<dyn Fn(Vec<ChatCommand>) + Send + Sync>,
}

impl ShellClose {
    pub fn new(
        holder: Arc<AppStateHolder>,
        run: Arc<dyn Fn(Vec<ChatCommand>) + Send + Sync>,
    ) -> Self {
        Self { holder, run }
    }
}

impl CloseFirst for ShellClose {
    fn close(&self, session: &str) {
        let chat_id = ChatId::of_session(session).as_str().to_owned();
        let (_, commands) = self
            .holder
            .dispatch_commands(UiIntent::CloseChat { chat_id });
        // Blocking, by design: the state a close leaves behind is the one the
        // delete below relies on.
        (self.run)(commands);
    }
}

/// What one removal came to: the slice to push, and whether session files were
/// removed - the caller then asks for the pass that reads the list back.
pub struct Removed {
    pub view: ViewState,
    pub removed: bool,
}

/// The offer of the workspace at `path` (S1, S2): how many chats it holds and
/// how big they are, which is what the dialog's consequence line says.
///
/// The path and the label are the row's - the config's spelling when the config
/// holds the folder, the sidebar's label when the tree carries it - so the dialog
/// names the workspace the way the sidebar does.
pub fn offer(
    holder: &AppStateHolder,
    chats: &dyn WorkspaceChats,
    path: &str,
) -> WorkspaceRemovalView {
    let key = folder_key(path);
    let stored = holder
        .config_value()
        .workspace(path)
        .map(|entry| entry.path.clone());
    let listed = chats.chats(path);
    WorkspaceRemovalView {
        path: stored.unwrap_or_else(|| path.to_owned()),
        label: holder
            .workspace_label(&key)
            .unwrap_or_else(|| last_segment(path)),
        key,
        chats: u32::try_from(listed.len()).unwrap_or(u32::MAX),
        bytes: listed.iter().map(|chat| chat.bytes).sum::<u64>() as f64,
    }
}

/// Removes the workspace at `path`, once `confirmed`, or says why not in the
/// slice.
///
/// Without the confirmation nothing happens at all: no child is closed, no file
/// is deleted and no notice records a question the user never answered. A
/// refusal answers its reason in `ViewState.workspace_notice` - the working-chat
/// refusal (S1) or the disk's own failed delete - and leaves the bookmark where
/// it was.
pub fn remove(
    holder: &AppStateHolder,
    chats: &dyn WorkspaceChats,
    files: &dyn SessionRemovals,
    working: &dyn WorkingChats,
    close: &dyn CloseFirst,
    path: &str,
    confirmed: bool,
) -> Removed {
    if !confirmed || path.trim().is_empty() {
        return Removed::only(holder);
    }
    // A path that names no workspace of this run - neither a folder the config
    // holds nor one the history names - has no row to remove.
    if holder.config_value().workspace(path).is_none() && !holder.knows_workspace(path) {
        return Removed::only(holder);
    }
    let listed = chats.chats(path);
    // S1: one chat mid-turn refuses the whole removal, before anything moves.
    if listed
        .iter()
        .any(|chat| working.is_working(&chat.session()))
    {
        let refusal = messages::remove_workspace_working_refusal(holder.language());
        return Removed::refused(holder, refusal);
    }
    // S5: the open, idle children of the workspace go first...
    for chat in &listed {
        close.close(&chat.session());
    }
    // ...then their files, trash-first (S4)...
    for chat in &listed {
        // S1, read again for THIS file: the gate above answered before the closes,
        // and a chat that started a turn in the gap would otherwise lose the file
        // under its own turn. A chat that is working now refuses the rest of the
        // flow whole, with the same wording and nothing further deleted.
        if working.is_working(&chat.session()) {
            let refusal = messages::remove_workspace_working_refusal(holder.language());
            return Removed::refused(holder, refusal);
        }
        if let Err(error) = session_delete::delete_session(files, &chat.path) {
            let line = messages::workspace_removal_failed(holder.language(), &error.to_string());
            return Removed::refused(holder, line);
        }
    }
    // ...and the bookmark last, which is what makes the gate true: a run that
    // reaches here has a workspace with no chat left.
    holder.update_config(|config| config.remove_workspace(path));
    holder.clear_workspace_notice();
    Removed {
        view: holder.view(),
        removed: !listed.is_empty(),
    }
}

impl Removed {
    /// An action that changes nothing: no confirmation, or a path no row names.
    fn only(holder: &AppStateHolder) -> Self {
        Self {
            view: holder.view(),
            removed: false,
        }
    }

    /// An action that was refused: nothing more is closed or deleted, and the
    /// reason reaches the user as the view's workspace notice.
    fn refused(holder: &AppStateHolder, message: String) -> Self {
        Self {
            view: holder.note_workspace_problem(message),
            removed: false,
        }
    }
}

/// The identity a folder's chats share, or `None` for a session that reports no
/// folder: no workspace is invented for it, so none can be removed either.
fn folder_key_of(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        None
    } else {
        Some(folder_key(path))
    }
}

/// The last segment of a folder path, for a workspace no row labels: the same
/// name the sidebar would draw, read without needing the tree.
fn last_segment(path: &str) -> String {
    let separators = [char::from(47u8), char::from(92u8)]; // / and the backslash
    path.trim_end_matches(separators)
        .rsplit(separators)
        .next()
        .unwrap_or(path)
        .to_owned()
}

/// The removal offer of the workspace at `path` (S1, S2): what the dialog says
/// before anything happens.
#[tauri::command]
#[specta::specta]
pub async fn workspace_removal_offer(
    path: String,
    bridge: tauri::State<'_, Bridge>,
) -> Result<WorkspaceRemovalView, String> {
    bridge.accepting_work()?;
    // Reading the index is a lock over cached rows, never a pass over the disk.
    let shared = bridge.sessions.shared();
    Ok(offer(&bridge.holder, &IndexChats(shared.as_ref()), &path))
}

/// Removes one workspace (S1, S2, S4, S5): the row menu's one destructive path,
/// which the web side sends only from its dialog, `confirmed`.
///
/// Everything that blocks - closing a child (up to the host's 2 s grace) and the
/// deletes - runs off the window's thread. A removal that lands asks the session
/// list for the pass that heals it, which is what takes the chats' rows off the
/// tree; a removal of a workspace with no chat changes the config, whose revision
/// the page already follows.
#[tauri::command]
#[specta::specta]
pub async fn remove_workspace(
    path: String,
    confirmed: bool,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.accepting_work()?;
    let language = bridge.holder.language();
    let holder = Arc::clone(&bridge.holder);
    let shared = bridge.sessions.shared();
    let close = bridge.close_first();
    let removed = tauri::async_runtime::spawn_blocking(move || {
        remove(
            holder.as_ref(),
            &IndexChats(shared.as_ref()),
            &DiskRemovals,
            holder.as_ref(),
            &close,
            &path,
            confirmed,
        )
    })
    .await
    .map_err(|error| messages::workspace_removal_failed(language, &error.to_string()))?;
    emit_state(&app, &removed.view)?;
    if removed.removed {
        bridge.sessions.request();
    }
    Ok(removed.view)
}
