//! Deleting a chat (T3, D2(b), S3, S4) and the workspace row it can take with it
//! (T4, D3(a)): the shell's side of the ONE file gentle-desk ever removes from the
//! user's pi data.
//!
//! It is a COMMAND and not a `UiIntent` variant, for the reasons the rename
//! beside it is: it waits for a human (the row's menu opens a confirm dialog that
//! names the real file) and it writes the disk, so the web side awaits an answer
//! that arrives long after the click.
//!
//! Two rules must hold BEFORE the file goes, and both are refusals, never errors:
//! the delete runs only with the confirmation the dialog sends ([`delete`]'s
//! `confirmed`), and a chat whose child is running a turn is refused with one
//! line, exactly as a rename is (`chat_rename`'s [`WorkingChats`]). The child is
//! never stopped from here: stopping a chat belongs to its own rules, so the
//! first refusal asks the user to stop it (D2(b), S-T3.5).
//!
//! The removal itself lives in [`gentle_backend::session_delete`], which owns pi
//! data access: one `remove_file` of exactly that path, and nothing else. The
//! list heals by itself afterwards - the index drops a file it no longer finds
//! and the watcher classifies the removal (S3) - so this module only ASKS for the
//! pass that reads the list back, as a rename does.
//!
//! When the deleted chat was the last chat of a workspace only the history names
//! (a DERIVED folder, not one the config holds), its row goes away with that
//! refreshed list (D3(a), T4). That is reported only when the workspace was the
//! current one, because a row the user is not looking at needs no line of its
//! own; the removal is already visible on the next push.

use std::path::Path;
use std::sync::Arc;

use gentle_backend::session_delete::{self, DiskRemovals, SessionRemovals};
use gentle_runtime::app_config::folder_key;
use tauri::AppHandle;

use crate::bridge::AppStateHolder;
use crate::chat_rename::WorkingChats;
use crate::commands::{emit_state, Bridge};
use crate::contract::ViewState;

/// The one line a user reads when they asked to delete a chat that is mid-turn.
/// The delete never stops the child itself (S-T3.5): the user does. Its rule and
/// its words are shared with Close chat (0.1.14), which says the same sentence
/// with its own verb, so a row menu never grows a second wording.
pub fn working_child() -> String {
    format!("{}, then delete it.", gentle_app::WORKING_CHAT)
}

/// What the user reads when the last chat of a derived workspace is deleted and
/// that workspace was the current one: its row is gone from the tree.
pub fn workspace_gone(label: &str) -> String {
    format!("{label} has no chats left, so its workspace row is gone.")
}

/// What one delete came to: the slice to push, and whether the session file was
/// removed - the caller then asks for the pass that heals the list.
pub struct Deleted {
    pub view: ViewState,
    pub removed: bool,
}

/// Deletes the chat stored at `session`, once `confirmed`, or says why not in the
/// slice.
///
/// Without the confirmation nothing happens at all: no file is looked at and no
/// notice records a question the user never answered. A refusal answers the
/// reason in `ViewState.sessions_notice` and leaves the file where it is, and a
/// delete that lands clears whatever refusal was there before.
pub fn delete(
    holder: &AppStateHolder,
    files: &dyn SessionRemovals,
    working: &dyn WorkingChats,
    session: &str,
    confirmed: bool,
) -> Deleted {
    if !confirmed {
        return Deleted {
            view: holder.view(),
            removed: false,
        };
    }
    if working.is_working(session) {
        return Deleted {
            view: holder.note_sessions_problem(working_child()),
            removed: false,
        };
    }
    // Read before the file goes: the workspace a delete leaves empty can only be
    // named while the chat is still in the list the tree is built from.
    let vanished = vanishing_workspace(holder, session);
    match session_delete::delete_session(files, Path::new(session)) {
        Ok(()) => {
            holder.clear_sessions_notice();
            if let Some(label) = vanished {
                holder.note_workspace_problem(workspace_gone(&label));
            }
            Deleted {
                view: holder.view(),
                removed: true,
            }
        }
        Err(error) => Deleted {
            view: holder.note_sessions_problem(error.to_string()),
            removed: false,
        },
    }
}

/// Deletes one chat (S4): the row sends the session path the confirm dialog
/// showed, and the web side sends `confirmed` only from that dialog's Delete.
///
/// The removal happens off the window's thread. A delete that lands asks the
/// session list for the pass that heals it, which is what takes the chat's row -
/// and the row of a workspace it leaves empty - off the tree; a refusal asks for
/// nothing, because no file changed.
#[tauri::command]
#[specta::specta]
pub async fn delete_session(
    session_id: String,
    confirmed: bool,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.accepting_work()?;
    let holder = Arc::clone(&bridge.holder);
    let deleted = tauri::async_runtime::spawn_blocking(move || {
        delete(
            holder.as_ref(),
            &DiskRemovals,
            holder.as_ref(),
            &session_id,
            confirmed,
        )
    })
    .await
    .map_err(|error| format!("the delete could not run: {error}"))?;
    emit_state(&app, &deleted.view)?;
    if deleted.removed {
        bridge.sessions.request();
    }
    Ok(deleted.view)
}

/// The label of the derived workspace that deleting `session` leaves with no
/// chat, when that workspace is the CURRENT one (T4, D3(a)); `None` for every
/// other delete.
///
/// A DERIVED workspace is one only the session cwds name: a folder the config
/// holds keeps its row with no chat, so nothing vanishes there. The current
/// workspace is the one of the foreground chat, which is the rule the sidebar
/// names the current one by (P5); a run with no foreground chat has none here to
/// lose, and the delete needs no line either.
fn vanishing_workspace(holder: &AppStateHolder, session: &str) -> Option<String> {
    let listed = holder.listed_sessions();
    let gone = listed.iter().find(|item| item.path == session)?;
    let key = folder_key_of(&gone.cwd)?;
    let config = holder.config_value();
    let in_config = config
        .workspaces
        .iter()
        .any(|entry| folder_key_of(&entry.path).as_deref() == Some(key.as_str()));
    if in_config {
        return None;
    }
    let chats = listed
        .iter()
        .filter(|item| folder_key_of(&item.cwd).as_deref() == Some(key.as_str()))
        .count();
    if chats != 1 {
        return None;
    }
    let foreground = holder.foreground_session()?;
    let current = listed
        .iter()
        .find(|item| item.path == foreground)
        .and_then(|item| folder_key_of(&item.cwd))?;
    if current != key {
        return None;
    }
    holder.workspace_label(&key)
}

/// The identity a folder's chats share, or `None` for a session that reports no
/// folder: no workspace is invented for it, so none can vanish.
fn folder_key_of(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        None
    } else {
        Some(folder_key(path))
    }
}
