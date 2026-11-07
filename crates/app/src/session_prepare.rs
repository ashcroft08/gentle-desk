//! Turning a session list into the data the sidebar's search runs over.
//!
//! Folding every title to lowercase without accents is the expensive, pure half
//! of a list change (measured at ~9.6 ms for 10 000 chats); it does not belong
//! in `update`. The shell calls [`prepare_sessions`] on a worker thread and
//! hands the result over through `Input::SessionsPrepared`, so `update` only
//! swaps the list in and re-runs the substring checks for the current query.

use crate::state::SessionItem;

/// A session list with its folded search keys already built, parallel to the
/// items. Plain owned data, so a worker thread can build it and send it over.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedSessions {
    items: Vec<SessionItem>,
    keys: Vec<String>,
}

// The whole point of the type: a thread other than the UI thread builds it.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<PreparedSessions>();
};

impl PreparedSessions {
    /// The list, unchanged and in order.
    pub fn items(&self) -> &[SessionItem] {
        &self.items
    }

    pub(crate) fn into_parts(self) -> (Vec<SessionItem>, Vec<String>) {
        (self.items, self.keys)
    }
}

/// Builds the search keys of a list: the part of a list change that can happen
/// away from the UI thread.
pub fn prepare_sessions(items: Vec<SessionItem>) -> PreparedSessions {
    let keys = items.iter().map(search_key).collect();
    PreparedSessions { items, keys }
}

/// What a query word is looked for in: the title and the project, folded.
fn search_key(session: &SessionItem) -> String {
    format!("{} {}", fold(&session.title), fold(session.project()))
}

/// Lowercase with the common Latin accents removed, so "Delegá" and "delega"
/// (and "ñ" and "n") meet. Combining marks are dropped, so a decomposed accent
/// folds like a composed one; other scripts pass through unchanged.
pub(crate) fn fold(text: &str) -> String {
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
