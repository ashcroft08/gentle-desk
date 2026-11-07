//! Building the sidebar's search data off the UI thread.
//!
//! `prepare_sessions` does the O(N) folding on the worker; `update` only swaps
//! the prepared list in, so `Input::SessionsPrepared` and `Input::SessionsLoaded`
//! must land on exactly the same state.

use gentle_app::{prepare_sessions, update, AppState, Input, Intent, SessionItem};

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
        item("hola", "C:/dev/gentle-desk"),
    ]
}

fn titles(state: &AppState) -> Vec<&str> {
    state
        .visible_sessions()
        .iter()
        .map(|s| s.title.as_str())
        .collect()
}

fn with_query(state: AppState, query: &str) -> AppState {
    update(state, Input::Intent(Intent::SetSessionQuery(query.into()))).0
}

/// The list arrives already prepared, as the worker thread would hand it over.
fn prepared(items: Vec<SessionItem>) -> Input {
    Input::SessionsPrepared(prepare_sessions(items))
}

fn prepared_state(items: Vec<SessionItem>, query: &str) -> AppState {
    let (state, _) = update(AppState::default(), prepared(items));
    with_query(state, query)
}

mod equivalence {
    use super::*;

    #[test]
    fn a_prepared_list_is_the_same_state_as_a_loaded_one() {
        let loaded = with_query(
            update(AppState::default(), Input::SessionsLoaded(sample())).0,
            "delega",
        );
        assert_eq!(prepared_state(sample(), "delega"), loaded);
    }

    #[test]
    fn a_prepared_list_matches_a_loaded_one_before_and_after_a_query() {
        // Same list, no query yet: identical state, so the search keys match too.
        let loaded_empty = update(AppState::default(), Input::SessionsLoaded(sample())).0;
        assert_eq!(prepared_state(sample(), ""), loaded_empty);
        // A query set before the list arrives filters the prepared one the same way.
        let query_first = with_query(AppState::default(), "parser");
        let prepared_after = update(query_first.clone(), prepared(sample())).0;
        let loaded_after = update(query_first, Input::SessionsLoaded(sample())).0;
        assert_eq!(prepared_after, loaded_after);
        assert_eq!(titles(&prepared_after), ["Refactor del parser"]);
    }
}

mod reload {
    use super::*;

    #[test]
    fn an_identical_prepared_list_changes_nothing() {
        let state = prepared_state(sample(), "hola");
        let (again, commands) = update(state.clone(), prepared(sample()));
        assert_eq!(again, state);
        assert!(commands.is_empty());
    }

    #[test]
    fn new_chats_appear_under_the_current_query() {
        let state = prepared_state(sample(), "delega");
        let mut more = sample();
        more.push(item("Delegá en tres helpers", "x"));
        let (state, commands) = update(state, prepared(more));
        assert_eq!(
            titles(&state),
            ["Delegá en dos helpers", "Delegá en tres helpers"]
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn dropped_chats_never_leave_a_dangling_row() {
        let state = prepared_state(sample(), "");
        let (state, _) = update(state, prepared(vec![item("solo uno", "x")]));
        assert_eq!(titles(&state), ["solo uno"]);
        let (state, _) = update(state, prepared(vec![]));
        assert!(titles(&state).is_empty());
        assert!(state.sessions.is_empty());
    }

    #[test]
    fn an_empty_prepared_list_is_an_empty_list() {
        let state = prepared_state(vec![], "");
        assert!(state.sessions.is_empty());
        assert!(titles(&state).is_empty());
        assert_eq!(
            state,
            update(AppState::default(), Input::SessionsLoaded(vec![])).0
        );
    }
}

mod preparation {
    use super::*;

    #[test]
    fn preparation_is_deterministic_and_keeps_the_item_order() {
        let first = prepare_sessions(sample());
        let second = prepare_sessions(sample());
        assert_eq!(first, second);
        let order: Vec<&str> = first.items().iter().map(|s| s.title.as_str()).collect();
        assert_eq!(
            order,
            [
                "Refactor del parser",
                "Delegá en dos helpers",
                "Receta de ñoquis",
                "hola"
            ]
        );
    }

    #[test]
    fn accent_and_case_folding_survives_the_round_trip() {
        assert_eq!(
            titles(&prepared_state(sample(), "DELEGA")),
            ["Delegá en dos helpers"]
        );
        assert_eq!(
            titles(&prepared_state(sample(), "ÑOQUIS")),
            ["Receta de ñoquis"]
        );
        assert_eq!(
            titles(&prepared_state(sample(), "delegá")),
            ["Delegá en dos helpers"]
        );
    }
}
