//! One chat at a time is not enough once each chat has its own child process.
//!
//! `Chats` keeps one [`ChatState`](gentle_core::chat::ChatState) per [`ChatId`],
//! remembers which chat is the foreground one, and derives the
//! [`ChatActivity`] the sidebar shows for each: working, finished-unread,
//! needs-attention (a pending extension dialog), or idle.
//!
//! One responsibility per file: `id` names a chat, `activity` is the rule that
//! turns a chat's state into what the list shows, `store` is the map with its
//! reads and writes, `intents` applies the intents that act on one chat.
//!
//! All of it is additive. [`AppState::chat`](crate::AppState) and the
//! single-chat intents stay exactly as they were, so the one-child shell keeps
//! working while the per-chat shell is wired in its own time.

mod activity;
mod id;
mod inputs;
mod intents;
mod store;

pub use activity::ChatActivity;
pub use id::ChatId;
pub use store::Chats;

pub(crate) use inputs::{failed, rekeyed};
pub(crate) use intents::apply as apply_intent;

use gentle_core::rpc::RpcEvent;

use crate::input::Command;
use crate::state::AppState;

/// Routes one chat's event to that chat and nothing else.
pub(crate) fn apply_event(
    mut state: AppState,
    chat: ChatId,
    event: &RpcEvent,
    now_ms: Option<u64>,
) -> (AppState, Vec<Command>) {
    let commands = state.chats.apply(&chat, event, now_ms);
    (state, commands)
}
