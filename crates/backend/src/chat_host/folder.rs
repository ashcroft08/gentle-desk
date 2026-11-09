//! The folder a chat is opened in.
//!
//! A child starts in the folder its open names, and never in one that cannot
//! host it: the failure is built here, before anything of the open is reserved,
//! so a caller that gets it back knows the host is untouched.

use std::io;
use std::path::Path;

use super::HostError;

/// Why `folder` cannot host the chat's child, in the words the user sees, or
/// `None` when it can. A folder that is not there and a path that is a file are
/// the same answer, each with its own reason.
pub(super) fn refusal(folder: &Path) -> Option<HostError> {
    let (kind, reason) = match std::fs::metadata(folder) {
        Ok(metadata) if metadata.is_dir() => return None,
        Ok(_) => (io::ErrorKind::InvalidInput, "it is a file".to_owned()),
        Err(error) => (error.kind(), error.to_string()),
    };
    let error = io::Error::new(kind, format!("no folder at {}: {reason}", folder.display()));
    Some(HostError::Folder {
        message: error.to_string(),
    })
}
