//! The `app` layer of gentle-desk: state, intents and commands.
//!
//! Pure: [`update`] takes the state and one [`Input`] (a user intent from the
//! UI or an RPC event from the backend) and returns the next state plus the
//! [`Command`]s to execute. Nothing here spawns a process, touches the
//! clipboard or knows a GUI framework; the shell around it runs the commands.
//!
//! The same state also holds one chat per chat id ([`Chats`]) for a shell that
//! runs one child per chat. That shell calls [`update_chats`], which answers the
//! same commands plus the [`ChatCommand`]s that name a chat; [`update`] keeps its
//! single-child contract, so the one-child shell is untouched.
//!
//! One file per responsibility: `state` (what the UI renders), `input` (what
//! goes in and out of `update`), `intents` (user actions), `events` (RPC
//! events), `notices` (extension notices), `chats` (one state per chat),
//! `session_prepare` (building the search data of a list, away from the UI
//! thread), `session_query` (which chats the sidebar shows), `list_status`
//! (whether that list is complete).

mod chats;
mod events;
mod input;
mod intents;
mod list_status;
mod notices;
mod session_prepare;
mod session_query;
mod state;

pub use chats::{ChatActivity, ChatId, Chats};
pub use input::{ChatCommand, Command, Effects, Input, Intent};
pub use list_status::ListStatus;
pub use session_prepare::{prepare_sessions, PreparedSessions};
pub use state::{AppState, Notice, NoticeLevel, SessionItem};

/// [`update`] with both halves of the answer.
///
/// A shell that runs one child per chat calls this. The one-child shell keeps
/// calling [`update`], which answers the same `commands` and nothing else: no
/// input it can produce yields a [`ChatCommand`].
pub fn update_chats(state: AppState, input: Input) -> (AppState, Effects) {
    fold(state, input)
}

pub fn update(state: AppState, input: Input) -> (AppState, Vec<Command>) {
    let (state, effects) = fold(state, input);
    (state, effects.commands)
}

fn fold(state: AppState, input: Input) -> (AppState, Effects) {
    match input {
        Input::Rpc(event) => shared(events::apply_event(state, &event, None)),
        Input::RpcAt(event, now_ms) => shared(events::apply_event(state, &event, Some(now_ms))),
        Input::ChatEvent { chat, event } => shared(chats::apply_event(state, chat, &event, None)),
        Input::ChatEventAt {
            chat,
            event,
            now_ms,
        } => shared(chats::apply_event(state, chat, &event, Some(now_ms))),
        Input::ChatFailed { chat, message } => {
            (chats::failed(state, &chat, &message), Effects::default())
        }
        Input::ChatRekeyed { from, to } => (chats::rekeyed(state, &from, to), Effects::default()),
        Input::SessionsPrepared(prepared) => {
            let mut state = state;
            state.set_prepared(prepared);
            state.set_ready();
            (state, Effects::default())
        }
        Input::SessionsProgress {
            prepared,
            done,
            total,
        } => {
            let mut state = state;
            state.set_progress(prepared, done, total);
            (state, Effects::default())
        }
        Input::SessionsLoaded(sessions) => {
            fold(state, Input::SessionsPrepared(prepare_sessions(sessions)))
        }
        Input::Intent(intent) => intents::apply_intent(state, intent),
    }
}

/// A result whose only effects are the single-child commands.
fn shared((state, commands): (AppState, Vec<Command>)) -> (AppState, Effects) {
    (
        state,
        Effects {
            commands,
            chat_commands: Vec::new(),
        },
    )
}
