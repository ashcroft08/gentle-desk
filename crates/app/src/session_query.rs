//! Which saved chats the sidebar shows for what the user typed.
//!
//! Pure and independent of how the list is drawn. The expensive part, folding
//! every title to lowercase without accents, happens once when the list
//! changes (`set_sessions`); each keystroke (`set_query`) only runs substring
//! checks over those prepared keys, and `view` just reads the result.

use crate::state::{AppState, SessionItem};

impl AppState {
    /// The saved chats matching the search box, in list order.
    pub fn visible_sessions(&self) -> Vec<&SessionItem> {
        self.visible.iter().map(|&i| &self.sessions[i]).collect()
    }

    /// Replaces the list. An identical list (the periodic refresh finding
    /// nothing new) costs a comparison and nothing else.
    pub(crate) fn set_sessions(&mut self, sessions: Vec<SessionItem>) {
        if sessions == self.sessions {
            return;
        }
        self.search_keys = sessions.iter().map(search_key).collect();
        self.sessions = sessions;
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

/// What a query word is looked for in: the title and the project, folded.
fn search_key(session: &SessionItem) -> String {
    format!("{} {}", fold(&session.title), fold(session.project()))
}

/// Lowercase with the common Latin accents removed, so "Delegá" and "delega"
/// (and "ñ" and "n") meet. Combining marks are dropped, so a decomposed accent
/// folds like a composed one; other scripts pass through unchanged.
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| !matches!(*c as u32, 0x300..=0x36f))
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            other => other,
        })
        .collect()
}
