//! Renaming a chat: a new chat has no session file when its child starts, and
//! learns the path once the child reports it. The slot then moves from the
//! `New` key to the `Saved` one, and the process itself is left alone.

use super::super::ChatKey;
use super::ChatHost;

impl ChatHost {
    /// Moves a live chat from key `from` to key `to`, keeping its process, its
    /// foreground and working marks and its place in the idle order. Every event
    /// the child sends from now on is tagged with `to`.
    ///
    /// Refused (`false`) when `from` is unknown or still starting (its start
    /// result is matched by its key), and when `to` already names a chat: that
    /// would silently replace a different child.
    pub fn rekey(&self, from: &ChatKey, to: ChatKey) -> bool {
        let mut state = self.state();
        let movable = state
            .slots
            .get(from)
            .is_some_and(|slot| !slot.is_spawning() && !slot.is_dead());
        if !movable || state.slots.contains_key(&to) {
            return false;
        }
        let was_foreground = state.foreground.as_ref() == Some(from);
        let Some(slot) = state.take(from) else {
            return false;
        };
        slot.retag(to.clone());
        state.slots.insert(to.clone(), slot);
        if was_foreground {
            state.foreground = Some(to);
        }
        true
    }
}
