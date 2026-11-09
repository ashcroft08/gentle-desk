//! The workspace intents that are not the reducer's: folding a workspace's chats
//! away (D8) and the folder a new chat of a workspace starts in.
//!
//! Both are the shell's business, not the state's, because `crates/app` carries
//! no workspace and no folder (S2):
//!
//! - The collapsed state of a workspace belongs to the app config, so it is
//!   remembered between runs (D8, S9). This module owns the rule; the file itself
//!   is written by one writer on its own thread (`config::ConfigWriter`), never
//!   here.
//! - The folder of a chat that is about to open waits in a table keyed by the
//!   chat the reducer allocated (`pending::PendingFolders`), and the executor
//!   takes it when it runs that chat's open.

use gentle_app::ChatCommand;

use crate::bridge::AppStateHolder;
use crate::contract::ViewState;
use crate::pending::PendingFolders;

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

/// Remembers the folder of the chat a workspace intent is opening, before that
/// command runs: exactly the chats the command opens, never one more.
pub fn remember_folder(folders: &PendingFolders, commands: &[ChatCommand], folder: &str) {
    for command in commands {
        if let ChatCommand::OpenProcess { chat } = command {
            folders.set(chat, folder);
        }
    }
}
