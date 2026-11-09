//! The workspace actions that are not the reducer's: folding a workspace's
//! chats away (D8), the folder a new chat of a workspace starts in, and the two
//! "New workspace" paths of Q1 (S7, S8).
//!
//! All of it is the shell's business, not the state's, because `crates/app`
//! carries no workspace and no folder (S2):
//!
//! - The collapsed state of a workspace belongs to the app config, so it is
//!   remembered between runs (D8, S9). This module owns the rule; the file itself
//!   is written by one writer on its own thread (`config::ConfigWriter`), never
//!   here.
//! - The folder of a chat that is about to open waits in a table keyed by the
//!   chat the reducer allocated (`pending::PendingFolders`), and the executor
//!   takes it when it runs that chat's open.
//! - Adding a workspace always ends the same way (S7): the folder is in the
//!   config and a new chat is opening there. `workspace_add` and
//!   `workspace_create` differ only in how they get the folder.

use gentle_app::ChatCommand;

use crate::bridge::AppStateHolder;
use crate::contract::{UiIntent, ViewState};
use crate::pending::PendingFolders;

/// What one workspace action asks the shell to do: the slice to push, and the
/// per-chat commands to run - both workspace paths of Q1 end by opening a new
/// chat in the workspace (S7, D7).
pub struct WorkspaceAction {
    /// The slice the web side renders: what `dispatch_intent` answers with.
    pub view: ViewState,
    /// The commands the fold asked for, for the shell to run off its thread.
    pub commands: Vec<ChatCommand>,
}

impl WorkspaceAction {
    /// An action that changes nothing: a cancelled dialog, or a run whose
    /// workspace already holds everything it should.
    pub fn only(view: ViewState) -> Self {
        Self {
            view,
            commands: Vec::new(),
        }
    }

    /// An action that was REFUSED (Q6): nothing is written, nothing opens, and
    /// the reason reaches the user as the view's workspace notice.
    pub fn refused(holder: &AppStateHolder, message: String) -> Self {
        Self::only(holder.note_workspace_problem(message))
    }
}

/// Makes `folder` the workspace it was picked or created as: added to the config
/// (idempotent for a folder the config already holds), the last refusal cleared,
/// and a new chat opened there. The workspace stays whatever becomes of that
/// chat (D7).
pub fn adopt(holder: &AppStateHolder, folder: &str, folders: &PendingFolders) -> WorkspaceAction {
    let canonical = remember_workspace(holder, folder);
    // The next workspace action that works has nothing to report.
    holder.clear_workspace_notice();
    open_chat_in(holder, &canonical, folders)
}

/// Adds `path` to the config unless the same folder is there already, and
/// answers the canonical spelling the config stores - the form a row shows and
/// an intent takes back.
pub fn remember_workspace(holder: &AppStateHolder, path: &str) -> String {
    holder.update_config(|config| config.add_workspace(path));
    holder
        .config_value()
        .workspace(path)
        .map_or_else(|| path.to_owned(), |entry| entry.path.clone())
}

/// Folds or unfolds the chats of the workspace at `path` (D8).
///
/// A workspace that exists only because a session lives there has no state in
/// the config to remember (the limit W1 recorded), so it is added to the config
/// the first time it is folded: a derived workspace cannot remember a fold
/// otherwise, and S9 asks for the fold to be remembered.
///
/// An unknown folder, and an empty path, change NOTHING: no workspace is added,
/// no revision moves and nothing is asked to be written.
///
/// Whether the folder is a workspace at all is decided BEFORE the edit, because
/// it reads the tree and the session list: the edit itself is one atomic step
/// (`AppStateHolder::update_config`), so two folds of two different workspaces
/// cannot lose each other, whatever order they run in.
pub fn collapse(holder: &AppStateHolder, path: &str, collapsed: bool) -> ViewState {
    if path.trim().is_empty() {
        return holder.view();
    }
    let known = holder.config_value().workspace(path).is_some() || holder.knows_workspace(path);
    if !known {
        return holder.view();
    }
    holder.update_config(|config| {
        if config.workspace(path).is_none() {
            // A derived workspace: added expanded here, then folded below, so the
            // fold lands on an entry the config can remember next run.
            if !config.add_workspace(path) {
                return false;
            }
        }
        config.set_collapsed(path, collapsed)
    });
    holder.view()
}

/// Starts the new chat of `folder` through the same path
/// `NewChatInWorkspace` takes, with the folder waiting for the open it asked
/// for.
fn open_chat_in(
    holder: &AppStateHolder,
    folder: &str,
    folders: &PendingFolders,
) -> WorkspaceAction {
    let (view, commands) = holder.dispatch_commands(UiIntent::NewChatInWorkspace {
        path: folder.to_owned(),
    });
    remember_folder(folders, &commands, folder);
    WorkspaceAction { view, commands }
}

/// Remembers the folder of the chat a workspace intent is opening, before that
/// command runs: exactly the chats the command opens, never one more.
pub fn remember_folder(folders: &PendingFolders, commands: &[ChatCommand], folder: &str) {
    for command in commands {
        if let ChatCommand::OpenProcess { chat } = command {
            folders.set(chat, folder);
        }
    }
}
