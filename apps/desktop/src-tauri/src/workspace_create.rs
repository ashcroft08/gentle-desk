//! "Create a new folder", the second item of the New workspace menu (Q1 option
//! A, W3c).
//!
//! Two steps, in this order because both of them can refuse:
//!
//! 1. The name is checked (`folder_name`), BEFORE any dialog opens: a name
//!    Windows would not accept never asks the user for a place to put it.
//! 2. The human picks the PARENT folder, the app makes `<parent>/<name>` there
//!    (`new_folder`, Q6: never an overwrite), and the result becomes a workspace
//!    with a new chat opening in it (S7).

use std::path::Path;

use gentle_runtime::app_config;

use crate::bridge::AppStateHolder;
use crate::folder_name;
use crate::new_folder;
use crate::pending::PendingFolders;
use crate::picker::FolderPicker;
use crate::workspace::{self, WorkspaceAction};

/// The title of the dialog: this one picks the place the new folder goes in,
/// which is why it never offers to make one (Q1 option A, not the dialog's own
/// "New folder" button).
pub const TITLE: &str = "Choose where the new workspace folder goes";

/// Creates `name` under the parent the human picks.
///
/// The picker runs on the caller's thread, which must not be the window's and
/// must hold no lock: a dialog can stay open for minutes.
pub fn create_new(
    holder: &AppStateHolder,
    picker: &dyn FolderPicker,
    folders: &PendingFolders,
    name: &str,
) -> WorkspaceAction {
    let name = match folder_name::check(name) {
        Ok(name) => name,
        Err(reason) => return WorkspaceAction::refused(holder, reason),
    };
    let Some(parent) = picker.pick_folder(TITLE) else {
        return WorkspaceAction::only(holder.view());
    };
    if let Some(reason) = unavailable(&parent) {
        return WorkspaceAction::refused(holder, reason);
    }
    let target = match new_folder::create_under(&parent, &name) {
        Ok(target) => target,
        Err(reason) => return WorkspaceAction::refused(holder, reason),
    };
    workspace::adopt(holder, &target.to_string_lossy(), folders)
}

/// Why the picked parent cannot hold a new folder, `None` when it can.
fn unavailable(parent: &Path) -> Option<String> {
    app_config::workspace_status(parent)
        .reason()
        .map(str::to_owned)
}
