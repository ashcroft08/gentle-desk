//! Where the app config file lives: `<roaming app data>/gentle-desk/config.json`.
//!
//! Roaming, because the added workspaces and the collapse state belong to the
//! user and follow them, unlike the session index cache under the local app data
//! directory. The environment value is a parameter, so the rule is testable
//! without touching the real environment.

use std::ffi::OsStr;
use std::path::PathBuf;

/// The config file to use when the user's roaming app data directory is
/// `app_data`, or `None` when there is none to write to.
pub fn config_path(app_data: Option<&OsStr>) -> Option<PathBuf> {
    let root = app_data.filter(|value| !value.is_empty())?;
    Some(PathBuf::from(root).join("gentle-desk").join("config.json"))
}

#[cfg(test)]
mod tests;
