//! Renaming a chat (T2, D1(a), S4): the shell's side of the one write gentle-desk
//! makes to the user's pi data.
//!
//! It is a COMMAND and not a `UiIntent` variant, for the same reason the two
//! workspace commands are: it waits for a human (the row's title becomes an input
//! and the rename runs when they press Enter) and it writes the disk. A reducer
//! answers in one step; this answers the same [`ViewState`] much later, so it is a
//! command the web side awaits.
//!
//! The append itself lives in [`gentle_backend::session_rename`], which owns pi
//! data access: one `session_info` line, and nothing else about the file changes.
//! What this module adds is the rule that must hold BEFORE that write: a chat
//! whose child is running a turn is not renamed, because the child appends to the
//! same file as the turn runs and two appenders could interleave (the rule D2(b)
//! gives delete). A refusal is a notice, never a crashed command.
//!
//! The write runs on a blocking thread - a file write takes as long as the disk
//! takes - and the pass that reads the new name back is asked for after it, so the
//! row and the header carry the new title on the next push with no path change
//! (S2).

use std::path::Path;
use std::sync::Arc;

use gentle_app::ChatActivity;
use gentle_backend::session_rename::{self, DiskFiles, SessionFiles};
use tauri::AppHandle;

use crate::bridge::AppStateHolder;
use crate::commands::{emit_state, Bridge};
use crate::contract::ViewState;

/// The one line a user reads when they asked to rename a chat that is mid-turn.
pub const WORKING_CHILD: &str = "This chat is working. Rename it when the turn ends.";

/// Whether the chat behind a session file has a child that is running a turn right
/// now. The app already knows it (the host's working mark, as the reducer reports
/// it), so this is a seam only to keep the command testable without a child.
pub trait WorkingChats {
    fn is_working(&self, session: &str) -> bool;
}

/// The real answer: the activity the reducer holds for that chat. A chat the app
/// has never heard of is idle, and an idle chat's child appends nothing until the
/// user sends a prompt, so only `Working` refuses the rename.
impl WorkingChats for AppStateHolder {
    fn is_working(&self, session: &str) -> bool {
        self.session_activity(session) == ChatActivity::Working
    }
}

/// What one rename came to: the slice to push, and whether the session file was
/// written - the caller then asks for the pass that reads the new name back.
pub struct Renamed {
    pub view: ViewState,
    pub written: bool,
}

/// Renames the chat stored at `session` to `name`, or says why not in the slice.
///
/// Answers the slice the user must see either way: a refusal carries its reason in
/// `ViewState.sessions_notice` and leaves the file untouched, and a rename clears
/// whatever refusal was there before.
pub fn rename(
    holder: &AppStateHolder,
    files: &dyn SessionFiles,
    working: &dyn WorkingChats,
    session: &str,
    name: &str,
) -> Renamed {
    if working.is_working(session) {
        return Renamed {
            view: holder.note_sessions_problem(WORKING_CHILD.to_owned()),
            written: false,
        };
    }
    match session_rename::rename_session(files, Path::new(session), name) {
        Ok(()) => {
            holder.clear_sessions_notice();
            Renamed {
                view: holder.view(),
                written: true,
            }
        }
        Err(error) => Renamed {
            view: holder.note_sessions_problem(error.to_string()),
            written: false,
        },
    }
}

/// Renames one chat (S4): the row sends the session path it was given and the name
/// typed in its title, and the answer is the slice the sidebar draws.
///
/// The write happens off the window's thread. A rename that lands asks the session
/// list for the pass that reads the name back, which is what moves the row and the
/// header; a refusal asks for nothing, because no file changed.
#[tauri::command]
#[specta::specta]
pub async fn rename_session(
    session_id: String,
    name: String,
    bridge: tauri::State<'_, Bridge>,
    app: AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    bridge.accepting_work()?;
    let holder = Arc::clone(&bridge.holder);
    let renamed = tauri::async_runtime::spawn_blocking(move || {
        rename(
            holder.as_ref(),
            &DiskFiles,
            holder.as_ref(),
            &session_id,
            &name,
        )
    })
    .await
    .map_err(|error| format!("the rename could not run: {error}"))?;
    emit_state(&app, &renamed.view)?;
    if renamed.written {
        bridge.sessions.request();
    }
    Ok(renamed.view)
}
