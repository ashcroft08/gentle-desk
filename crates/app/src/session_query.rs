//! Which saved chats the sidebar shows for what the user typed.
//!
//! Pure and independent of how the list is drawn. Folding every title happens
//! off the UI thread (`session_prepare`); `update` only swaps the prepared list
//! in, each keystroke (`set_query`) runs substring checks over the prepared
//! keys, and `view` just reads the result.

use crate::session_prepare::{fold, PreparedSessions};
use crate::state::{AppState, SessionItem};

impl AppState {
    /// The saved chats matching the search box, in list order.
    pub fn visible_sessions(&self) -> Vec<&SessionItem> {
        self.visible.iter().map(|&i| &self.sessions[i]).collect()
    }

    /// Replaces the list with one whose search keys are already built. An
    /// identical list (the periodic refresh finding nothing new) costs a
    /// comparison and nothing else.
    pub(crate) fn set_prepared(&mut self, prepared: PreparedSessions) {
        let (items, keys) = prepared.into_parts();
        if items == self.sessions {
            return;
        }
        self.sessions = items;
        self.search_keys = keys;
        self.refilter();
    }

    pub(crate) fn set_query(&mut self, query: String) {
        self.session_query = query;
        self.refilter();
    }

    fn refilter(&mut self) {
        let folded = fold(&self.session_query);
        let words: Vec<&str> = folded.split_whitespace().collect();
        self.visible = self
            .search_keys
            .iter()
            .enumerate()
            .filter(|(_, key)| words.iter().all(|word| key.contains(word)))
            .map(|(index, _)| index)
            .collect();
    }
}
