//! Renaming a new chat into the stored chat its child just created.
//!
//! A chat started without `--session` has no session file yet; the `get_state`
//! reply carries the path the child opened (`sessionFile`), and from then on the
//! chat is the stored one in both the registry and the host.

use gentle_app::ChatId;
use gentle_core::rpc::RpcResponse;

use super::ChatRuntime;
use crate::chat_keys::Rekeyed;

/// The session file a `get_state` reply reports, if it reports one.
///
/// `gentle-core` keeps the reply data as plain JSON, so the one field this shell
/// needs is read here; a failed reply, another command's reply, a missing field
/// and an empty path all answer `None`.
pub fn session_file_of(response: &RpcResponse) -> Option<&str> {
    if response.command != "get_state" {
        return None;
    }
    let data = response.outcome.as_ref().ok()?;
    data.get("sessionFile")?
        .as_str()
        .filter(|path| !path.is_empty())
}

impl ChatRuntime {
    /// Moves a new chat to the stored chat its child reported: the registry
    /// pairs the chat with the session path instead of its `New` key, and the
    /// host keeps the child under that path. `None` (and nothing changes) for a
    /// chat the registry does not know as new.
    pub fn rekey(&self, chat: &ChatId, session_path: &str) -> Option<Rekeyed> {
        let moved = self.keys().rekey(chat, session_path)?;
        if !self.host.rekey(&moved.from, moved.to.clone()) {
            // The child is not in a state worth moving (it is starting, gone, or
            // another chat already holds the path). The chat itself is still the
            // stored one, so the next open starts a child for the session file.
            eprintln!("{}: the child could not move to its session file", moved.id);
        }
        Some(moved)
    }
}
