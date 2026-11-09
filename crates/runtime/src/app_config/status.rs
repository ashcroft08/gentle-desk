//! Whether the folder of a workspace is there right now.
//!
//! A separate question from loading or saving the config: it touches the file
//! system, so it is asked outside any lock and never while the config is being
//! held. An added folder that disappeared stays in the list; this only says so,
//! with a reason the sidebar can show.

use std::fs;
use std::path::Path;

/// The reason a missing folder is missing.
pub const MISSING_REASON: &str = "This folder no longer exists.";
/// The reason a path that is not a folder cannot be a workspace.
pub const NOT_A_FOLDER_REASON: &str = "That path is a file, not a folder.";

/// What the folder of a workspace is right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceStatus {
    /// The folder is there.
    Available,
    /// Nothing is at that path; the reason says why that stops new chats here.
    Missing(String),
    /// Something is there, but it is not a folder.
    NotAFolder(String),
}

impl WorkspaceStatus {
    /// Whether new sessions can start in the workspace.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    /// The text to show next to the workspace, when there is a reason to.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::Missing(reason) | Self::NotAFolder(reason) => Some(reason),
        }
    }
}

/// The status of the workspace folder at `path`.
pub fn workspace_status(path: &Path) -> WorkspaceStatus {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => WorkspaceStatus::Available,
        Ok(_) => WorkspaceStatus::NotAFolder(NOT_A_FOLDER_REASON.to_owned()),
        Err(_) => WorkspaceStatus::Missing(MISSING_REASON.to_owned()),
    }
}

#[cfg(test)]
mod tests;
