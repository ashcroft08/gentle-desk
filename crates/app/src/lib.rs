//! The `app` layer of gentle-desk: state, intents and commands.
//!
//! Pure: [`update`] takes the state and one [`Input`] (a user intent from the
//! UI or an RPC event from the backend) and returns the next state plus the
//! [`Command`]s to execute. Nothing here spawns a process, touches the
//! clipboard or knows a GUI framework; the shell around it runs the commands.
//!
//! One file per responsibility: `state` (what the UI renders), `input` (what
//! goes in and out of `update`), `intents` (user actions), `events` (RPC
//! events), `notices` (extension notices), `session_prepare` (building the
//! search data of a list, away from the UI thread), `session_query` (which
//! chats the sidebar shows).

mod events;
mod input;
mod intents;
mod notices;
mod session_prepare;
mod session_query;
mod state;

pub use input::{Command, Input, Intent};
pub use session_prepare::{prepare_sessions, PreparedSessions};
pub use state::{AppState, Notice, NoticeLevel, SessionItem};

pub fn update(state: AppState, input: Input) -> (AppState, Vec<Command>) {
    match input {
        Input::Rpc(event) => events::apply_event(state, &event, None),
        Input::RpcAt(event, now_ms) => events::apply_event(state, &event, Some(now_ms)),
        Input::SessionsPrepared(prepared) => {
            let mut state = state;
            state.set_prepared(prepared);
            (state, Vec::new())
        }
        Input::SessionsLoaded(sessions) => {
            update(state, Input::SessionsPrepared(prepare_sessions(sessions)))
        }
        Input::Intent(intent) => intents::apply_intent(state, intent),
    }
}
