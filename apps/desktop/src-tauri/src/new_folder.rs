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

use gentle_core::i18n::{messages, Language};

/// Makes `<parent>/<name>`, or accepts the empty folder already there, and
/// answers the folder that is now the workspace. Every refusal is one sentence
/// in `language`.
pub fn create_under(language: Language, parent: &Path, name: &str) -> Result<PathBuf, String> {
    let target = parent.join(name);
    match fs::metadata(&target) {
        Ok(metadata) if !metadata.is_dir() => {
            Err(messages::new_folder_file_exists(language).to_owned())
        }
        // A folder of that name: only an empty one is taken, unread contents
        // would be a change nobody asked for.
        Ok(_) => match holds_anything(language, &target) {
            Ok(true) => Err(messages::new_folder_not_empty(language).to_owned()),
            Ok(false) => Ok(target),
            Err(reason) => Err(reason),
        },
        Err(error) if error.kind() == ErrorKind::NotFound => fs::create_dir(&target)
            .map(|()| target)
            .map_err(|error| messages::new_folder_create_failed(language, &error.to_string())),
        Err(error) => Err(messages::new_folder_path_unreadable(
            language,
            &error.to_string(),
        )),
    }
}

/// Whether the folder at `path` holds an entry of any kind.
fn holds_anything(language: Language, path: &Path) -> Result<bool, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| messages::new_folder_read_failed(language, &error.to_string()))?;
    Ok(entries.next().is_some())
}
