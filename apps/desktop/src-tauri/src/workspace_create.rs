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

use gentle_core::i18n::messages;
use gentle_runtime::app_config;

use crate::bridge::AppStateHolder;
use crate::folder_name;
use crate::new_folder;
use crate::pending::PendingFolders;
use crate::picker::FolderPicker;
use crate::workspace::{self, WorkspaceAction};

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
    let language = holder.language();
    let name = match folder_name::check(language, name) {
        Ok(name) => name,
        Err(reason) => return WorkspaceAction::refused(holder, reason),
    };
    // The title is the one this dialog needs: it picks the place the new folder
    // goes in, which is why it never offers to make one (Q1 option A, not the
    // dialog's own "New folder" button).
    let title = messages::choose_new_workspace_parent_title(language);
    let Some(parent) = picker.pick_folder(title) else {
        return WorkspaceAction::only(holder.view());
    };
    if let Some(reason) = unavailable(&parent, language) {
        return WorkspaceAction::refused(holder, reason);
    }
    let target = match new_folder::create_under(language, &parent, &name) {
        Ok(target) => target,
        Err(reason) => return WorkspaceAction::refused(holder, reason),
    };
    workspace::adopt(holder, &target.to_string_lossy(), folders)
}

/// Why the picked parent cannot hold a new folder, `None` when it can.
fn unavailable(parent: &Path, language: gentle_core::i18n::Language) -> Option<String> {
    app_config::workspace_status(parent, language)
        .reason()
        .map(str::to_owned)
}
