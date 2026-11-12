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
mod apply;
mod id;
mod inputs;
mod intents;
mod lifecycle;
mod queries;
mod queue;
mod store;

pub use activity::ChatActivity;
pub use id::ChatId;
pub use store::Chats;

pub(crate) use inputs::{closed, failed, push_death_notice, rekeyed, stderr};
pub(crate) use intents::apply as apply_intent;

use gentle_core::rpc::RpcEvent;

use crate::chip;
use crate::input::Effects;
use crate::notices::{notice_from, push_notice};
use crate::state::{AppState, NoticeLevel};

/// Applies the boot's working cap for every chat at once (S7).
pub(crate) fn set_working_cap(state: &mut AppState, cap: usize) {
    queue::set_cap(&mut state.chats, cap);
}

/// Records that the chat's child is up: the opening phase is over. For a
/// fresh child, the UI sends this only through the 20 s net for a child that
/// started but never speaks (its first event already clears the fold); an
/// already-live chat or a discarded start is confirmed at once, because no one
/// is waiting for the child any more.
pub(crate) fn opened(mut state: AppState, chat: &ChatId) -> AppState {
    state.chats.clear_opening(chat);
    state
}

/// Routes one chat's event to that chat and nothing else, and turns an
/// extension `notify` into the same notice the one-child path builds: the same
/// rule, the same levels, the same text and the same app-wide list.
///
/// The notice is not tied to the chat that raised it, deliberately: the app has
/// one notice list and the only chat a notice can name is the Restart of a death
/// (S3), so an extension's own words travel as the extension wrote them. A
/// background chat's message therefore never claims to come from the foreground
/// one (it carries no chat title at all), and two chats that announce the same
/// text still share one row, exactly as in the one-child shell.
pub(crate) fn apply_event(
    mut state: AppState,
    chat: ChatId,
    event: &RpcEvent,
    now_ms: Option<u64>,
) -> (AppState, Effects) {
    let (commands, chat_commands) = state.chats.apply(&chat, event, now_ms);
    if let Some((level, text)) = notice_from(event) {
        push_notice(&mut state, level, text);
    }
    // The chip belongs to the foreground chat's child, so only its replies
    // fill the chip (0.1.11): the same parsing the one-child shell does.
    if state.chats.foreground().as_ref() == Some(&&chat) {
        if let RpcEvent::Response(response) = event {
            match (&response.outcome, response.command.as_str()) {
                (Ok(data), command) => {
                    chip::apply_ok(&mut state, command, data);
                }
                (Err(error), command @ ("get_available_models" | "set_thinking_level")) => {
                    chip::apply_err(&mut state, command, error, &mut |state, text| {
                        push_notice(state, NoticeLevel::Error, text)
                    });
                }
                (Err(error), command) => {
                    push_notice(
                        &mut state,
                        NoticeLevel::Error,
                        format!("{command} failed: {error}"),
                    );
                }
            }
        }
    }
    (
        state,
        Effects {
            commands,
            chat_commands,
        },
    )
}
