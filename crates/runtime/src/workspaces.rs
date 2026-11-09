//! The workspace list the sidebar shows: one row per folder, with its chats.
//!
//! A workspace is a folder. Its identity is the canonical folder path
//! ([`folder_key`]), so the same folder added to the config and found in the
//! `cwd` of some session headers is one workspace, whatever spelling each came
//! in (verbatim prefix, trailing separator, letter case on Windows). The row
//! shows the canonical spelling, preferring the one the config holds.
//!
//! The list is the union of the folders the config holds, always shown even
//! with no chat (D7), and the folders the session headers report, so the history
//! made from the terminal shows up. A session with an empty or blank `cwd`
//! belongs to no workspace: none is invented for it, and it is counted in
//! [`WorkspaceList::unplaced`]. Temporary folders are not hidden: there is no
//! rule that tells a temporary folder from a lasting one yet.
//!
//! Order (P3): by last activity, newest first, ties by [`Workspace::key`]; the
//! folders with no chat come after the ones with activity, in config order. The
//! config holds no "added at" time, so a folder with no chat has nothing to be
//! ordered by but the order it was added in; that is the decision taken here.
//!
//! Pure: no file system, no environment, no clock. One pass over the sessions,
//! with a [`HashMap`](std::collections::HashMap) keyed by [`folder_key`], and the
//! path of each folder canonicalised once, so a session costs one map probe.

mod group;
mod label;
mod order;
mod spelling;

use gentle_app::SessionItem;

use crate::app_config::AppConfig;

/// One row of the sidebar: a folder and the chats that live in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// [`folder_key`] of the folder: what two spellings of one folder share.
    pub key: String,
    /// The canonical spelling shown: the config's, when the config has it.
    pub path: String,
    /// The last segment of the path, or the path itself for a root, which has
    /// no segment of its own.
    pub name: String,
    /// [`Workspace::name`], extended with the parent segments it takes to tell
    /// two workspaces with the same name apart. Never longer than it takes.
    pub label: String,
    /// Whether the chats of the workspace are folded away (D8). Always `false`
    /// for a folder the config does not hold, which has no state to remember.
    pub collapsed: bool,
    /// Whether the folder was added to the config rather than only derived from
    /// the sessions.
    pub from_config: bool,
    /// The newest `modified_ms` of its sessions, `None` with no chat.
    pub last_activity_ms: Option<u64>,
    /// The indices of its sessions in the slice [`build`] was given, in that
    /// order (the caller's list is already newest first).
    pub sessions: Vec<usize>,
}

/// What the sidebar draws: the workspaces, and the sessions none of them holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceList {
    /// The workspaces, in the order the sidebar shows them.
    pub workspaces: Vec<Workspace>,
    /// How many sessions report an empty or blank `cwd` and so belong to no
    /// workspace. They are listed nowhere.
    pub unplaced: usize,
}

/// The workspace list of `config` plus the session headers of `sessions`.
pub fn build(config: &AppConfig, sessions: &[SessionItem]) -> WorkspaceList {
    let gathered = group::gather(config, sessions);
    let mut folders = gathered.folders;
    let unplaced = gathered.unplaced;
    order::sort(&mut folders);

    // One walk of each folder's path, never one per session.
    let parts: Vec<Vec<&str>> = folders
        .iter()
        .map(|folder| spelling::segments(&folder.path))
        .collect();
    let names: Vec<String> = folders
        .iter()
        .zip(&parts)
        .map(|(folder, segments)| spelling::name(&folder.path, segments))
        .collect();
    let labels = label::labels(&names, &parts);

    let workspaces = folders
        .into_iter()
        .zip(names)
        .zip(labels)
        .map(|((folder, name), label)| Workspace {
            key: folder.key,
            path: folder.path,
            name,
            label,
            collapsed: folder.collapsed,
            from_config: folder.from_config,
            last_activity_ms: folder.last_activity_ms,
            sessions: folder.sessions,
        })
        .collect();
    WorkspaceList {
        workspaces,
        unplaced,
    }
}

#[cfg(test)]
mod tests;
