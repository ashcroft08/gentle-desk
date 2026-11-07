//! The sidebar's search: which saved chats are visible for a query.

use gentle_app::{update, AppState, Input, Intent, SessionItem};

const BACKSLASH: char = '\\';

fn item(title: &str, cwd: &str) -> SessionItem {
    SessionItem {
        path: format!("p/{title}"),
        title: title.into(),
        cwd: cwd.into(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// A Windows-style path built without backslashes in the source.
fn windows(parts: &[&str]) -> String {
    parts.join(&BACKSLASH.to_string())
}

fn state_with(items: Vec<SessionItem>, query: &str) -> AppState {
    let (state, _) = update(AppState::default(), Input::SessionsLoaded(items));
    let (state, _) = update(state, Input::Intent(Intent::SetSessionQuery(query.into())));
    state
}

fn titles(state: &AppState) -> Vec<&str> {
    state
        .visible_sessions()
        .iter()
        .map(|s| s.title.as_str())
        .collect()
}

fn sample() -> Vec<SessionItem> {
    vec![
        item(
            "Refactor del parser",
            &windows(&["C:", "dev", "gentle-desk"]),
        ),
        item(
            "Delegá en dos helpers",
            &windows(&["C:", "dev", "gentle-shell-desktop"]),
        ),
        item("Receta de ñoquis", "/home/ana/cocina"),
        item("hola", &windows(&["C:", "dev", "gentle-desk"])),
    ]
}

mod project {
    use super::*;

    #[test]
    fn the_project_is_the_last_segment_of_the_working_directory() {
        let path = windows(&["C:", "dev", "gentle-desk"]);
        assert_eq!(item("t", &path).project(), "gentle-desk");
        assert_eq!(item("t", "/home/ana/cocina").project(), "cocina");
    }

    #[test]
    fn trailing_separators_and_odd_paths_are_handled() {
        let trailing = format!("{}{BACKSLASH}", windows(&["C:", "dev", "proj"]));
        assert_eq!(item("t", &trailing).project(), "proj");
        assert_eq!(item("t", "/a/b/").project(), "b");
        assert_eq!(item("t", "solo").project(), "solo");
        assert_eq!(item("t", "").project(), "");
    }
}

mod matching {
    use super::*;

    #[test]
    fn an_empty_or_blank_query_shows_everything_in_order() {
        for query in ["", "   "] {
            assert_eq!(
                titles(&state_with(sample(), query)),
                [
                    "Refactor del parser",
                    "Delegá en dos helpers",
                    "Receta de ñoquis",
                    "hola"
                ]
            );
        }
    }

    #[test]
    fn it_matches_the_title_ignoring_case() {
        assert_eq!(
            titles(&state_with(sample(), "REFACTOR")),
            ["Refactor del parser"]
        );
        assert_eq!(titles(&state_with(sample(), "hOLa")), ["hola"]);
    }

    #[test]
    fn it_matches_the_project_name() {
        assert_eq!(
            titles(&state_with(sample(), "cocina")),
            ["Receta de ñoquis"]
        );
        assert_eq!(
            titles(&state_with(sample(), "gentle-desk")),
            ["Refactor del parser", "hola"]
        );
    }

    #[test]
    fn the_rest_of_the_path_is_not_searched() {
        // "dev" is in three working directories but is nobody's title or project name.
        assert!(titles(&state_with(sample(), "dev")).is_empty());
        assert!(titles(&state_with(sample(), "home")).is_empty());
    }

    #[test]
    fn accents_and_the_enie_are_ignored_in_both_directions() {
        assert_eq!(
            titles(&state_with(sample(), "delega")),
            ["Delegá en dos helpers"]
        );
        assert_eq!(
            titles(&state_with(sample(), "DELEGÁ")),
            ["Delegá en dos helpers"]
        );
        assert_eq!(
            titles(&state_with(sample(), "noquis")),
            ["Receta de ñoquis"]
        );
        assert_eq!(
            titles(&state_with(sample(), "ÑOQUIS")),
            ["Receta de ñoquis"]
        );
    }

    #[test]
    fn a_decomposed_accent_matches_like_the_composed_one() {
        let combining_acute = char::from_u32(0x301).unwrap();
        let decomposed = item(&format!("Delega{combining_acute} todo"), "x");
        let state = state_with(vec![decomposed], "delega");
        assert_eq!(state.visible_sessions().len(), 1);
        // And the query may be the decomposed one against a composed title.
        let composed = state_with(
            vec![item("Delegá todo", "x")],
            &format!("delega{combining_acute}"),
        );
        assert_eq!(composed.visible_sessions().len(), 1);
    }

    #[test]
    fn every_word_must_match_in_any_order_across_title_and_project() {
        assert_eq!(
            titles(&state_with(sample(), "parser refactor")),
            ["Refactor del parser"]
        );
        assert_eq!(titles(&state_with(sample(), "hola gentle-desk")), ["hola"]);
        assert!(titles(&state_with(sample(), "hola cocina")).is_empty());
    }

    #[test]
    fn extra_whitespace_around_words_is_ignored() {
        assert_eq!(
            titles(&state_with(sample(), "  refactor   parser ")),
            ["Refactor del parser"]
        );
    }

    #[test]
    fn text_that_is_not_latin_is_matched_as_is() {
        let state = state_with(vec![item("日本語のチャット", "x")], "日本");
        assert_eq!(state.visible_sessions().len(), 1);
    }

    #[test]
    fn nothing_matching_gives_an_empty_list() {
        assert!(titles(&state_with(sample(), "zzz")).is_empty());
    }

    #[test]
    fn the_search_covers_every_chat_not_only_a_visible_window() {
        let many: Vec<_> = (0..10_000)
            .map(|i| item(&format!("chat {i}"), "p"))
            .collect();
        let state = state_with(many, "chat 9999");
        assert_eq!(titles(&state), ["chat 9999"]);
    }
}

mod refresh {
    use super::*;

    #[test]
    fn an_identical_reload_changes_nothing() {
        let state = state_with(sample(), "hola");
        let (again, commands) = update(state.clone(), Input::SessionsLoaded(sample()));
        assert_eq!(again, state);
        assert!(commands.is_empty());
    }

    #[test]
    fn a_reload_with_new_chats_updates_what_the_query_shows() {
        let state = state_with(sample(), "hola");
        let mut more = sample();
        more.push(item("hola de nuevo", "x"));
        let (state, _) = update(state, Input::SessionsLoaded(more));
        assert_eq!(titles(&state), ["hola", "hola de nuevo"]);
    }

    #[test]
    fn a_reload_that_drops_chats_never_leaves_a_dangling_row() {
        let state = state_with(sample(), "");
        let (state, _) = update(state, Input::SessionsLoaded(vec![item("solo uno", "x")]));
        assert_eq!(titles(&state), ["solo uno"]);
        let (state, _) = update(state, Input::SessionsLoaded(vec![]));
        assert!(titles(&state).is_empty());
    }
}

mod intent {
    use super::*;

    #[test]
    fn typing_in_the_search_box_sets_the_query_and_sends_no_command() {
        let (state, commands) = update(
            state_with(sample(), ""),
            Input::Intent(Intent::SetSessionQuery("hola".into())),
        );
        assert_eq!(state.session_query, "hola");
        assert_eq!(titles(&state), ["hola"]);
        assert!(commands.is_empty());
    }

    #[test]
    fn the_query_survives_a_refresh_of_the_list_and_a_new_chat() {
        let (state, _) = update(
            AppState::default(),
            Input::Intent(Intent::SetSessionQuery("hola".into())),
        );
        let (state, _) = update(state, Input::SessionsLoaded(sample()));
        assert_eq!(titles(&state), ["hola"]);
        let (state, _) = update(state, Input::Intent(Intent::NewChat));
        assert_eq!(state.session_query, "hola");
        assert_eq!(titles(&state), ["hola"]);
    }

    #[test]
    fn filtering_a_chat_out_of_view_does_not_close_it() {
        // The open chat is tracked by its path, not by its position in the list.
        let (state, _) = update(
            state_with(sample(), ""),
            Input::Intent(Intent::OpenSession("p/hola".into())),
        );
        let (state, _) = update(
            state,
            Input::Intent(Intent::SetSessionQuery("receta".into())),
        );
        assert_eq!(state.current_session.as_deref(), Some("p/hola"));
        assert_eq!(titles(&state), ["Receta de ñoquis"]);
    }
}
