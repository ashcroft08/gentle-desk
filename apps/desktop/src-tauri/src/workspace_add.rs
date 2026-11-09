//! "Choose an existing folder", the first item of the New workspace menu
//! (Q1 option A, W3c).
//!
//! One path: the human picks a folder, the folder is validated, and it becomes a
//! workspace with a new chat opening there (S7). A cancelled dialog changes
//! NOTHING; a pick that is not an existing folder is refused with a message and
//! changes nothing either.

use std::path::Path;

use gentle_runtime::app_config;

use crate::bridge::AppStateHolder;
use crate::pending::PendingFolders;
use crate::picker::FolderPicker;
use crate::workspace::{self, WorkspaceAction};

/// The title of the dialog, so the user knows what the folder is for.
pub const TITLE: &str = "Choose a workspace folder";

/// Adds the folder the human picks.
///
/// The picker runs on the caller's thread, which must not be the window's and
/// must hold no lock: a dialog can stay open for minutes.
pub fn add_existing(
    holder: &AppStateHolder,
    picker: &dyn FolderPicker,
    folders: &PendingFolders,
) -> WorkspaceAction {
    let Some(picked) = picker.pick_folder(TITLE) else {
        return WorkspaceAction::only(holder.view());
    };
    if let Some(reason) = unavailable(&picked) {
        return WorkspaceAction::refused(holder, reason);
    }
    workspace::adopt(holder, &picked.to_string_lossy(), folders)
}

/// Why `picked` cannot be a workspace, `None` when it can. The reason is the
/// one the sidebar already shows for a workspace folder (`workspace_status`),
/// so a dead folder says the same thing wherever it is met.
fn unavailable(picked: &Path) -> Option<String> {
    app_config::workspace_status(picked)
        .reason()
        .map(str::to_owned)
}
