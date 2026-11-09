//! The folders the workspace intents named, waiting for the open they belong to.
//!
//! The reducer names a chat and answers `ChatCommand::OpenProcess`; it knows
//! nothing about folders, because `crates/app` carries none and this migration
//! does not change it (S2). The folder a workspace intent named therefore
//! travels beside the command: the intent path sets it for the chat the reducer
//! allocated, BEFORE that command runs, and the executor takes it when it does.
//!
//! A taken entry is gone, so a folder never reaches a second open, and an open
//! nobody set a folder for inherits this process's folder, as every chat did
//! before this slice.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use gentle_app::ChatId;

/// One pending folder per chat, keyed by the chat the reducer allocated.
#[derive(Debug, Default)]
pub struct PendingFolders {
    folders: Mutex<HashMap<String, PathBuf>>,
}

impl PendingFolders {
    /// Remembers that the chat `chat` opens in `folder`. Called before the
    /// command that opens it is run, so the executor finds it.
    pub fn set(&self, chat: &ChatId, folder: &str) {
        lock(&self.folders).insert(chat.as_str().to_owned(), PathBuf::from(folder));
    }

    /// Takes the folder of `chat`, if one waits: an open consumes it once.
    pub fn take(&self, chat: &ChatId) -> Option<PathBuf> {
        lock(&self.folders).remove(chat.as_str())
    }

    /// Whether nothing waits for an open. A table that leaked would grow with
    /// every chat; this is what the tests read.
    pub fn is_empty(&self) -> bool {
        lock(&self.folders).is_empty()
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat(name: &str) -> ChatId {
        ChatId::of_session(name)
    }

    #[test]
    fn a_folder_is_taken_once_and_never_leaks() {
        let folders = PendingFolders::default();
        let one = chat("one");

        folders.set(&one, r"C:\dev\repo");

        assert_eq!(
            folders.take(&one),
            Some(PathBuf::from(r"C:\dev\repo")),
            "the open takes the folder the intent named"
        );
        assert_eq!(folders.take(&one), None, "a folder is taken once");
        assert!(folders.is_empty(), "nothing waits after its open took it");
    }

    #[test]
    fn a_chat_no_intent_named_has_no_folder() {
        let folders = PendingFolders::default();

        assert_eq!(folders.take(&chat("other")), None);
        assert!(folders.is_empty());
    }
}
