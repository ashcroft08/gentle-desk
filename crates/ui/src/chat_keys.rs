//! The two names of one chat: the app layer's [`ChatId`] and the backend's
//! [`ChatKey`].
//!
//! A stored chat is named by its session path in both, so the two convert by
//! value. A brand-new chat has no path yet: the app calls it by a fresh
//! [`ChatId`], the host by a `ChatKey::New(n)`, and this registry remembers
//! which is which until the child reports its session file ([`ChatKeys::rekey`]).
//!
//! Pure: nothing here starts, stops or talks to a process.

use std::path::PathBuf;

use gentle_app::ChatId;
use gentle_backend::chat_host::ChatKey;

/// A new chat that learned its session path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rekeyed {
    /// The key the host knew the chat by until now (`ChatKey::New`).
    pub from: ChatKey,
    /// The key it has from now on (`ChatKey::Saved`).
    pub to: ChatKey,
    /// The app-layer identity of the stored chat.
    pub id: ChatId,
}

/// Which `ChatKey::New` belongs to which fresh [`ChatId`].
#[derive(Debug, Default)]
pub struct ChatKeys {
    new_chats: Vec<(ChatId, ChatKey)>,
}

impl ChatKeys {
    /// The host's key for a chat: its session path for a stored chat, the
    /// registered key for a new one, `None` for a new chat never registered.
    pub fn key_of(&self, id: &ChatId) -> Option<ChatKey> {
        if let Some(path) = id.session_path() {
            return Some(ChatKey::Saved(PathBuf::from(path)));
        }
        self.new_chats
            .iter()
            .find(|(known, _)| known == id)
            .map(|(_, key)| key.clone())
    }

    /// The app's identity for a host key, `None` for a new chat nobody
    /// registered.
    pub fn id_of(&self, key: &ChatKey) -> Option<ChatId> {
        match key {
            ChatKey::Saved(path) => Some(ChatId::of_session(&path.to_string_lossy())),
            ChatKey::New(_) => self
                .new_chats
                .iter()
                .find(|(_, known)| known == key)
                .map(|(id, _)| id.clone()),
        }
    }

    /// Gives a fresh chat its host key, once: registering it again answers the
    /// same key. A stored chat is not new, so it answers `None`.
    pub fn register_new(&mut self, id: &ChatId) -> Option<ChatKey> {
        if id.session_path().is_some() {
            return None;
        }
        if let Some(key) = self.key_of(id) {
            return Some(key);
        }
        let key = ChatKey::new_chat();
        self.new_chats.push((id.clone(), key.clone()));
        Some(key)
    }

    /// The child of a new chat reported its session file: the chat becomes the
    /// stored one and the new-chat pairing is forgotten. `None` (and nothing
    /// changes) for a chat that is not a registered new chat, or an empty path.
    pub fn rekey(&mut self, id: &ChatId, session_path: &str) -> Option<Rekeyed> {
        if session_path.is_empty() {
            return None;
        }
        let at = self.new_chats.iter().position(|(known, _)| known == id)?;
        let (_, from) = self.new_chats.remove(at);
        Some(Rekeyed {
            from,
            to: ChatKey::Saved(PathBuf::from(session_path)),
            id: ChatId::of_session(session_path),
        })
    }
}

#[cfg(test)]
mod tests;
