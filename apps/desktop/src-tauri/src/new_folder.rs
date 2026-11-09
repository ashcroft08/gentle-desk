//! The folder "Create a new folder" makes on disk (Q6).
//!
//! One rule, and it is the reason this module exists: a workspace the app
//! creates NEVER overwrites anything. Three outcomes, and no fourth:
//!
//! - nothing is there: the folder is created, with `create_dir` and never
//!   `create_dir_all`, because the parent comes from the picker and must exist
//!   (a parent that vanished is a refusal, not a chain of new folders);
//! - an EMPTY folder is there: it is accepted as the workspace, and nothing is
//!   written into it;
//! - a folder that holds anything, or a file of that name: refused, and
//!   nothing is written.
//!
//! The disk is touched only here, and the caller runs this off the window's
//! thread and outside every lock.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Makes `<parent>/<name>`, or accepts the empty folder already there, and
/// answers the folder that is now the workspace.
pub fn create_under(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let target = parent.join(name);
    match fs::metadata(&target) {
        Ok(metadata) if !metadata.is_dir() => {
            Err("There is already a file with that name.".to_owned())
        }
        // A folder of that name: only an empty one is taken, unread contents
        // would be a change nobody asked for.
        Ok(_) => match holds_anything(&target) {
            Ok(true) => Err("That folder already holds files.".to_owned()),
            Ok(false) => Ok(target),
            Err(reason) => Err(reason),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => fs::create_dir(&target)
            .map(|()| target)
            .map_err(|error| format!("The folder could not be created: {error}")),
        Err(error) => Err(format!("That path could not be read: {error}")),
    }
}

/// Whether the folder at `path` holds an entry of any kind.
fn holds_anything(path: &Path) -> Result<bool, String> {
    let mut entries =
        fs::read_dir(path).map_err(|error| format!("That folder could not be read: {error}"))?;
    Ok(entries.next().is_some())
}
