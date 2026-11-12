//! The two "New workspace" commands of Q1 option A (S7, S8, W3c).
//!
//! They are COMMANDS and not `UiIntent` variants, because they do not fold: both
//! wait for a human (a folder dialog can stay open for minutes) and both touch
//! the disk. An intent is answered by the reducer in one step; these answer the
//! same `ViewState` much later, so they are commands the web side awaits.
//!
//! Each one runs its flow on a blocking thread - the dialog and the folder
//! creation never run on the window's thread and never under a lock - and then
//! pushes and runs the new chat the workspace opens, exactly as
//! [`Bridge::dispatch`] does.

use std::sync::Arc;

use crate::commands::Bridge;
use crate::contract::ViewState;
use crate::workspace::WorkspaceAction;
use crate::{workspace_add, workspace_create};

/// "Choose an existing folder": the picker, then the folder becomes a workspace
/// and a new chat opens there (S7).
///
/// A cancelled dialog changes NOTHING and answers the current slice; a pick that
/// is not an existing folder is refused and the reason reaches the user as
/// `ViewState.workspace_notice`.
#[tauri::command]
#[specta::specta]
pub async fn add_workspace(bridge: tauri::State<'_, Bridge>) -> Result<ViewState, String> {
    let holder = Arc::clone(&bridge.holder);
    let picker = bridge.picker();
    let folders = bridge.folders();
    let action =
        blocking(move || workspace_add::add_existing(&holder, picker.as_ref(), &folders)).await?;
    bridge.finish(action)
}

/// "Create a new folder": the user picks the PARENT, the app makes
/// `<parent>/<name>` under it (Q6: never an overwrite), and the new folder
/// becomes a workspace with a new chat opening there (S7).
///
/// `name` is checked before any dialog opens, so a name Windows would not
/// accept is refused without asking the user for a place to put it.
#[tauri::command]
#[specta::specta]
pub async fn create_workspace(
    name: String,
    bridge: tauri::State<'_, Bridge>,
) -> Result<ViewState, String> {
    let holder = Arc::clone(&bridge.holder);
    let picker = bridge.picker();
    let folders = bridge.folders();
    let action =
        blocking(move || workspace_create::create_new(&holder, picker.as_ref(), &folders, &name))
            .await?;
    bridge.finish(action)
}

/// Runs one workspace flow off the window's thread: a folder dialog and a
/// `create_dir` both block for as long as they take, and neither may hold a
/// lock of the holder while it does.
async fn blocking(
    flow: impl FnOnce() -> WorkspaceAction + Send + 'static,
) -> Result<WorkspaceAction, String> {
    tauri::async_runtime::spawn_blocking(flow)
        .await
        .map_err(|error| format!("the workspace command could not run: {error}"))
}
