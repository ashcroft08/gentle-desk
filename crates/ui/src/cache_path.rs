//! Where the session index cache lives.

use std::ffi::OsString;
use std::path::PathBuf;

/// The cache file to use when the user's local app data directory is
/// `local_app_data`, or `None` when there is none to write to.
pub fn cache_file(local_app_data: Option<OsString>) -> Option<PathBuf> {
    let root = local_app_data.filter(|value| !value.is_empty())?;
    Some(
        PathBuf::from(root)
            .join("gentle-desk")
            .join("session-index-v1.json"),
    )
}

#[cfg(test)]
mod tests;
