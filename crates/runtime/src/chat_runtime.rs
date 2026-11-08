//! The UI's handle on the chat host: it runs the app's per-chat commands against
//! the real children and turns what those children say back into app inputs.
//!
//! Pure plumbing between three layers that must not know each other: the app
//! names a chat by a [`ChatId`](gentle_app::ChatId) and asks for
//! [`ChatCommand`](gentle_app::ChatCommand)s, the host names it by a
//! [`ChatKey`](gentle_backend::chat_host::ChatKey) and starts processes, and
//! [`ChatKeys`] remembers which is which for chats that have no session file yet.
//!
//! [`ChatRuntime::execute`] blocks for as long as a child takes to start, so the
//! caller runs it off the UI thread. Nothing here draws or holds app state.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};

use gentle_backend::chat_host::{ChatHost, TaggedEvent};

use crate::chat_keys::ChatKeys;

mod events;
mod execute;
mod host;
mod open;
mod rekey;

pub use events::Converted;
pub use execute::Outcome;
pub use open::request_after_open;
pub use rekey::session_file_of;

/// The chat host plus the id/key registry that goes with it.
pub struct ChatRuntime {
    host: Arc<ChatHost>,
    keys: Mutex<ChatKeys>,
}

impl ChatRuntime {
    /// Takes ownership of a host and hands back the one stream of every child's
    /// tagged events, to be read by a single subscription.
    pub fn new(mut host: ChatHost) -> (Self, Receiver<TaggedEvent>) {
        let events = host.take_events();
        let runtime = Self {
            host: Arc::new(host),
            keys: Mutex::new(ChatKeys::default()),
        };
        (runtime, events)
    }

    /// The registry. Held only to look a key up or register one, never across
    /// a call that blocks on a child.
    fn keys(&self) -> MutexGuard<'_, ChatKeys> {
        self.keys
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests;
