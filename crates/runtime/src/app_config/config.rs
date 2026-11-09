//! The config document: the version, the added workspaces and their collapse
//! state, plus the folder identity two spellings share.
//!
//! Pure: no file system, no environment. A workspace is stored by its canonical
//! path (the Windows verbatim prefix removed) and compared by its
//! [`folder_key`], so the same folder added twice is one workspace.

use serde::{Deserialize, Serialize};

/// The only document version this app writes. Any other version is not
/// migrated: the file is kept aside and the app starts with an empty config.
pub const CONFIG_VERSION: u32 = 1;

/// The whole config file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    /// [`CONFIG_VERSION`] as this app writes it ([`crate::app_config::save`]
    /// always writes this value).
    pub version: u32,
    /// The added workspaces, in the order they were added.
    pub workspaces: Vec<WorkspaceEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            workspaces: Vec::new(),
        }
    }
}

/// One folder the user added, with its state in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    /// The canonical spelling of the folder: what the user picked, with the
    /// Windows verbatim prefix and any trailing separator removed.
    pub path: String,
    /// Whether its chats are folded away in the sidebar.
    pub collapsed: bool,
}

impl AppConfig {
    /// The workspace of `path`, matched by [`folder_key`] so any spelling of the
    /// folder answers it.
    pub fn workspace(&self, path: &str) -> Option<&WorkspaceEntry> {
        let key = folder_key(path);
        self.workspaces
            .iter()
            .find(|entry| folder_key(&entry.path) == key)
    }

    /// Adds `path` as a workspace, expanded, unless the same folder is already
    /// there. Answers whether a workspace was added: an empty or blank path
    /// adds nothing, and adding the same folder again changes nothing.
    pub fn add_workspace(&mut self, path: &str) -> bool {
        if path.trim().is_empty() || self.workspace(path).is_some() {
            return false;
        }
        self.workspaces.push(WorkspaceEntry {
            path: canonical_path(path),
            collapsed: false,
        });
        true
    }

    /// Folds or unfolds the chats of `path` in the sidebar. Answers whether the
    /// folder was a known workspace; an unknown folder changes nothing.
    pub fn set_collapsed(&mut self, path: &str, collapsed: bool) -> bool {
        let key = folder_key(path);
        let Some(entry) = self
            .workspaces
            .iter_mut()
            .find(|entry| folder_key(&entry.path) == key)
        else {
            return false;
        };
        entry.collapsed = collapsed;
        true
    }
}

/// The spelling a workspace is stored with, and the one the sidebar shows: `path`
/// with the Windows verbatim prefix removed and without a trailing separator (a
/// root keeps its own), so the same folder is stored and shown the same way
/// however it was reported.
pub(crate) fn canonical_path(path: &str) -> String {
    // `\\?\C:` and `\??\C:` mean `C:`, and the verbatim form of a share,
    // `\\?\UNC\srv\share`, means `\\srv\share`.
    let stripped = match verbatim_rest(path) {
        Some(rest) => match unc_share(rest) {
            Some(share) => format!(r"\\{}", share),
            None => rest.to_owned(),
        },
        None => path.to_owned(),
    };
    let trimmed = stripped.trim_end_matches(['\\', '/']);
    if trimmed.is_empty() {
        // A folder of separators is a root: it has nothing to trim.
        stripped
    } else if trimmed.ends_with(':') {
        // `C:` is a drive, not a folder: its root separator stays.
        format!("{trimmed}\\")
    } else {
        trimmed.to_owned()
    }
}

/// What follows a verbatim prefix (`\\?\` or `\??\`), `None` when there is no
/// such prefix.
fn verbatim_rest(path: &str) -> Option<&str> {
    path.strip_prefix(r"\\?\")
        .or_else(|| path.strip_prefix(r"\??\"))
}

/// The share part of a verbatim `UNC\srv\share`, `None` for anything else: the
/// letters alone are a folder name unless a separator follows them, and only the
/// verbatim form writes a share that way.
fn unc_share(rest: &str) -> Option<&str> {
    let after = rest
        .strip_prefix("UNC")
        .or_else(|| rest.strip_prefix("unc"))?;
    let share = after.strip_prefix(['\\', '/'])?;
    Some(share.trim_start_matches(['\\', '/']))
}

/// The key two spellings of one folder share: both separators treated alike,
/// empty parts dropped (so a trailing or doubled separator does not matter),
/// and the letter case folded on Windows.
pub fn folder_key(path: &str) -> String {
    let path = canonical_path(path);
    let parts: Vec<String> = path
        .split(['\\', '/'])
        .filter(|part| !part.is_empty())
        .map(fold)
        .collect();
    parts.join("/")
}

/// Windows paths are case-insensitive: `C:\Home` and `c:\home` are one place.
#[cfg(windows)]
fn fold(part: &str) -> String {
    part.to_lowercase()
}

#[cfg(not(windows))]
fn fold(part: &str) -> String {
    part.to_owned()
}

#[cfg(test)]
mod tests;
