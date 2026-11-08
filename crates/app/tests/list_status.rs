//! Whether the sidebar's list is still coming, and what a partial list knows.
//!
//! The first full pass of the session index reports progress; those partial
//! lists replace what the sidebar shows but must never claim to be the final
//! answer, and they must never undo it once it has arrived.

use gentle_app::{
    prepare_sessions, update, AppState, Command, Input, Intent, ListStatus, SessionItem,
};

fn item(title: &str, cwd: &str) -> SessionItem {
    SessionItem {
        path: format!("p/{title}"),
        title: title.into(),
        cwd: cwd.into(),
        modified_ms: 0,
        message_count: 1,
    }
}

fn sample() -> Vec<SessionItem> {
    vec![
        item("Refactor del parser", "C:/dev/gentle-desk"),
        item("Delegá en dos helpers", "C:/dev/gentle-shell-desktop"),
        item("Receta de ñoquis", "/home/ana/cocina"),
    ]
}

fn progress(items: Vec<SessionItem>, done: usize, total: usize) -> Input {
    Input::SessionsProgress {
        prepared: prepare_sessions(items),
        done,
        total,
    }
}

fn prepared(items: Vec<SessionItem>) -> Input {
    Input::SessionsPrepared(prepare_sessions(items))
}

fn with_query(state: AppState, query: &str) -> AppState {
    update(state, Input::Intent(Intent::SetSessionQuery(query.into()))).0
}

fn titles(state: &AppState) -> Vec<&str> {
    state
        .visible_sessions()
        .iter()
        .map(|s| s.title.as_str())
        .collect()
}

fn loading(done: usize, total: usize) -> ListStatus {
    ListStatus::Loading { done, total }
}

mod defaults {
    use super::*;

    /// A fresh app has never read a session file: the sidebar must not claim
    /// "No saved chats yet" before the first pass ends.
    #[test]
    fn a_fresh_app_is_loading_and_knows_nothing_yet() {
        assert_eq!(AppState::default().list_status, loading(0, 0));
    }
}

mod progress {
    use super::*;

    #[test]
    fn a_partial_list_replaces_the_list_and_updates_the_counters() {
        let (state, commands) = update(AppState::default(), progress(sample(), 12, 500));
        assert_eq!(
            titles(&state),
            [
                "Refactor del parser",
                "Delegá en dos helpers",
                "Receta de ñoquis"
            ]
        );
        assert_eq!(state.list_status, loading(12, 500));
        assert!(commands.is_empty());
    }

    #[test]
    fn the_counters_advance_even_when_the_list_is_identical() {
        let (state, _) = update(AppState::default(), progress(sample(), 12, 500));
        let (before, _) = update(AppState::default(), progress(sample(), 12, 500));
        let (state, commands) = update(state, progress(sample(), 40, 500));
        assert_eq!(state.list_status, loading(40, 500));
        assert_eq!(state.sessions, before.sessions);
        assert!(commands.is_empty());
    }

    #[test]
    fn the_final_list_settles_the_status_and_the_state_is_then_stable() {
        let (state, _) = update(AppState::default(), progress(sample(), 500, 500));
        let (state, _) = update(state, prepared(sample()));
        assert_eq!(state.list_status, ListStatus::Ready);
        let (again, commands) = update(state.clone(), prepared(sample()));
        assert_eq!(again, state);
        assert!(commands.is_empty());
    }

    #[test]
    fn a_partial_list_after_the_final_one_is_ignored() {
        let (state, _) = update(AppState::default(), prepared(sample()));
        let settled = state.clone();
        let late = progress(vec![item("solo uno", "x")], 1, 2);
        let (state, commands) = update(state, late);
        assert_eq!(state, settled);
        assert_eq!(state.list_status, ListStatus::Ready);
        assert!(commands.is_empty());
    }

    #[test]
    fn a_partial_list_respects_the_query_and_forgets_no_selection() {
        let (state, _) = update(AppState::default(), progress(sample(), 3, 500));
        let (state, _) = update(
            state,
            Input::Intent(Intent::OpenSession("p/Receta de ñoquis".into())),
        );
        let state = with_query(state, "delega");
        let (state, _) = update(state, progress(sample(), 7, 500));
        assert_eq!(titles(&state), ["Delegá en dos helpers"]);
        assert_eq!(state.session_query, "delega");
        assert_eq!(state.current_session.as_deref(), Some("p/Receta de ñoquis"));
        assert_eq!(state.list_status, loading(7, 500));
    }

    #[test]
    fn chats_dropped_by_a_later_partial_never_leave_a_dangling_row() {
        let (state, _) = update(AppState::default(), progress(sample(), 1, 500));
        let (state, _) = update(state, progress(vec![item("solo uno", "x")], 2, 500));
        assert_eq!(titles(&state), ["solo uno"]);
    }
}

mod previous_behaviour {
    use super::*;

    /// The final list still does what `SessionsPrepared` always did: replaces
    /// the list, refilters under the query and sends no command.
    #[test]
    fn the_final_list_still_replaces_and_refilters_the_list() {
        let state = with_query(AppState::default(), "parser");
        let (state, commands) = update(state, prepared(sample()));
        assert_eq!(titles(&state), ["Refactor del parser"]);
        assert!(commands.is_empty());
    }

    /// "New chat" keeps the sidebar exactly as it was, including whether it is
    /// still reading.
    #[test]
    fn new_chat_keeps_the_list_status() {
        let (ready, _) = update(AppState::default(), prepared(sample()));
        let (fresh, _) = update(ready, Input::Intent(Intent::NewChat));
        assert_eq!(fresh.list_status, ListStatus::Ready);
        assert_eq!(
            update(AppState::default(), Input::Intent(Intent::NewChat))
                .0
                .list_status,
            loading(0, 0)
        );
        assert_eq!(
            update(AppState::default(), Input::Intent(Intent::NewChat)).1,
            vec![Command::Send(gentle_core::rpc::RpcCommand::NewSession)]
        );
    }
}
