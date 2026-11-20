//! The folders the workspace intents named, waiting for the open they belong to.
//!
//! The reducer names a chat and answers a per-chat command; it knows nothing
//! about folders, because `crates/app` carries none and this migration does not
//! change it (S2). The folder a workspace intent named therefore travels beside
//! the state: the intent path sets it for the chat the fold allocated, BEFORE
//! that chat's child is asked for, and the executor takes it when it is.
//!
//! Since welcome-screen T4 (R2) the two moments are apart: a new chat is
//! ALLOCATED when it opens and its process starts when its composer asks, so the
//! folder waits here for as long as the chat is on screen unstarted. The table is
//! keyed by the id the slice names the chat with, which is the id that ask carries
//! back.
//!
//! A taken entry is gone, so a folder never reaches a second open, and an open
//! nobody set a folder for inherits this process's folder, as every chat did
//! before this slice. A chat whose composer never asks - it is closed before the
//! box is focused - keeps its entry for the rest of the run: one path, bounded by
//! the new chats of one session of the app, and nothing reads it.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// One pending folder per chat, keyed by the chat id text the view carries.
#[derive(Debug, Default)]
pub struct PendingFolders {
    folders: Mutex<HashMap<String, PathBuf>>,
}

impl PendingFolders {
    /// Remembers that the chat `chat_id` opens in `folder`. Called before the
    /// command that opens it is run, so the executor finds it.
    pub fn set(&self, chat_id: &str, folder: &str) {
        lock(&self.folders).insert(chat_id.to_owned(), PathBuf::from(folder));
    }

    /// Takes the folder of `chat_id`, if one waits: an open consumes it once.
    pub fn take(&self, chat_id: &str) -> Option<PathBuf> {
        lock(&self.folders).remove(chat_id)
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

    #[test]
    fn a_folder_is_taken_once_and_never_leaks() {
        let folders = PendingFolders::default();

        folders.set("new-1", r"C:\dev\repo");

        assert_eq!(
            folders.take("new-1"),
            Some(PathBuf::from(r"C:\dev\repo")),
            "the open takes the folder the intent named"
        );
        assert_eq!(folders.take("new-1"), None, "a folder is taken once");
        assert!(folders.is_empty(), "nothing waits after its open took it");
    }

    #[test]
    fn a_chat_no_intent_named_has_no_folder() {
        let folders = PendingFolders::default();

        assert_eq!(folders.take("session:/s/other.jsonl"), None);
        assert!(folders.is_empty());
    }

    #[test]
    fn a_folder_waits_until_the_open_it_belongs_to() {
        // welcome-screen T4 (R2): the chat is allocated when it opens and its
        // child is asked for later, on the composer's own trigger. The folder
        // has to be there until then, and beside the chats nobody asked about.
        let folders = PendingFolders::default();
        folders.set("new-7", r"C:\dev\one");

        assert_eq!(
            folders.take("new-8"),
            None,
            "another chat's open takes nothing"
        );
        assert!(!folders.is_empty(), "the waiting folder is still there");
        assert_eq!(folders.take("new-7"), Some(PathBuf::from(r"C:\dev\one")));
        assert!(folders.is_empty());
    }
}
