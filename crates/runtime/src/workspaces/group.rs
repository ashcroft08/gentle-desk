//! The union of the folders the config holds and the folders the sessions
//! report, gathered in one pass over the sessions.

use std::collections::HashMap;

use gentle_app::SessionItem;

use crate::app_config::{canonical_path, folder_key, AppConfig};

/// One folder while it is being gathered, before it is named and ordered.
pub(super) struct Folder {
    /// [`folder_key`] of [`Folder::path`]: the identity of the folder.
    pub(super) key: String,
    /// The canonical spelling shown.
    pub(super) path: String,
    pub(super) collapsed: bool,
    pub(super) from_config: bool,
    /// Its position among the config's workspaces: the folders with no chat are
    /// shown in that order.
    pub(super) config_rank: Option<usize>,
    pub(super) last_activity_ms: Option<u64>,
    pub(super) sessions: Vec<usize>,
}

/// The folders of the list, and the number of sessions that belong to none.
pub(super) struct Gathered {
    pub(super) folders: Vec<Folder>,
    pub(super) unplaced: usize,
}

/// Gathers the folders of `config` and of `sessions`, one entry per folder.
pub(super) fn gather(config: &AppConfig, sessions: &[SessionItem]) -> Gathered {
    let mut folders: Vec<Folder> = Vec::new();
    // Key to the position in `folders`: the one probe each session costs.
    let mut positions: HashMap<String, usize> = HashMap::new();

    for (rank, entry) in config.workspaces.iter().enumerate() {
        if entry.path.trim().is_empty() {
            continue;
        }
        let key = folder_key(&entry.path);
        if positions.contains_key(&key) {
            // `AppConfig::add_workspace` never writes a folder twice; a file
            // that does keeps its first entry, as `AppConfig::workspace` reads
            // the first one too.
            continue;
        }
        positions.insert(key.clone(), folders.len());
        folders.push(Folder {
            key,
            path: canonical_path(&entry.path),
            collapsed: entry.collapsed,
            from_config: true,
            config_rank: Some(rank),
            last_activity_ms: None,
            sessions: Vec::new(),
        });
    }

    let mut unplaced = 0;
    for (index, session) in sessions.iter().enumerate() {
        if session.cwd.trim().is_empty() {
            // No folder is invented for a chat that reports none.
            unplaced += 1;
            continue;
        }
        let key = folder_key(&session.cwd);
        let at = match positions.get(&key) {
            Some(&at) => at,
            None => {
                folders.push(Folder {
                    key: key.clone(),
                    path: canonical_path(&session.cwd),
                    collapsed: false,
                    from_config: false,
                    config_rank: None,
                    last_activity_ms: None,
                    sessions: Vec::new(),
                });
                positions.insert(key, folders.len() - 1);
                folders.len() - 1
            }
        };
        let folder = &mut folders[at];
        folder.sessions.push(index);
        folder.last_activity_ms = Some(match folder.last_activity_ms {
            Some(seen) => seen.max(session.modified_ms),
            None => session.modified_ms,
        });
    }

    Gathered { folders, unplaced }
}
