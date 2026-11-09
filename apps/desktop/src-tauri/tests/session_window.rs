//! Windowless tests of the windowed session list (S3): a temp pi home, a real
//! refresh pass, and the windows read back out of the holder the app keeps.
//! No Tauri app, no window, no real home of the user.
//!
//! The iced rule is the oracle: a query narrows the list by the same intent the
//! sidebar's search box folds (`Intent::SetSessionQuery`) and by the same
//! visibility rule (`AppState::visible_sessions`), so a window must answer what
//! that rule answers for the same fixture, sliced.

mod session_seam;
use session_seam::{settled, Home};

use gentle_app::{prepare_sessions, AppState, Input, Intent, SessionItem};
use gentle_desk_lib::contract::SessionRowView;
use gentle_desk_lib::sessions::window::{self, MAX_WINDOW};

/// A pinned clock: the age text of a row is the runtime's rule, not the test's
/// clock.
const NOW: u64 = 1_000 + 3 * 60 * 60_000;

fn ids(rows: &[SessionRowView]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

/// The list the app itself would show for `query`, built from the rows of a
/// window: the oracle every window is compared against.
fn iced_rule(rows: &[SessionRowView], query: &str) -> Vec<String> {
    let items: Vec<SessionItem> = rows
        .iter()
        .map(|row| SessionItem {
            path: row.id.clone(),
            title: row.title.clone(),
            cwd: format!("C:/dev/{}", row.project),
            modified_ms: 0,
            message_count: row.message_count,
        })
        .collect();
    let state = AppState::default();
    let state = gentle_app::update_chats(state, Input::SessionsPrepared(prepare_sessions(items))).0;
    let state = gentle_app::update_chats(
        state,
        Input::Intent(Intent::SetSessionQuery(query.to_owned())),
    )
    .0;
    state
        .visible_sessions()
        .into_iter()
        .map(|session| session.path.clone())
        .collect()
}

#[test]
fn windows_slice_the_list_and_a_window_past_the_end_is_empty() {
    let home = Home::new();
    home.many(25);
    let (holder, _) = settled(&home);

    let first = holder.sessions_window(0, 10, "");
    let second = holder.sessions_window(10, 10, "");
    let overflow = holder.sessions_window(40, 10, "");

    assert_eq!(first.total, 25, "the total is the whole filtered list");
    assert_eq!(first.rows.len(), 10);
    assert_eq!(second.total, 25);
    assert_eq!(second.rows.len(), 10);
    assert!(
        ids(&first.rows)
            .iter()
            .all(|id| !ids(&second.rows).contains(id)),
        "a window holds its own rows and no others"
    );

    assert_eq!(overflow.total, 25, "a window past the end keeps the total");
    assert!(overflow.rows.is_empty(), "and holds no rows");

    let mut seen = ids(&first.rows);
    seen.extend(ids(&second.rows));
    seen.extend(ids(&holder.sessions_window(20, 10, "").rows));
    let before = seen.len();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), before, "the three windows tile the list");
    assert_eq!(seen.len(), 25);
}

#[test]
fn a_query_narrows_the_total_and_the_rows_as_the_iced_rule_does() {
    let home = Home::new();
    home.session("p", "a.jsonl", "first chat");
    home.session("p", "b.jsonl", "second chat");
    home.session("p", "c.jsonl", "third chat");
    home.session("p", "d.jsonl", "other thread");
    let (holder, _) = settled(&home);

    let all = holder.sessions_window(0, MAX_WINDOW, "");
    assert_eq!(all.total, 4);

    let narrowed = holder.sessions_window(0, MAX_WINDOW, "second chat");
    assert_eq!(narrowed.total, 1, "the query narrows the total");
    assert_eq!(narrowed.rows.len(), 1);
    assert_eq!(
        ids(&narrowed.rows),
        iced_rule(&all.rows, "second chat"),
        "the window answers what the iced rule answers for the same fixture"
    );

    let by_word = holder.sessions_window(0, MAX_WINDOW, "thread");
    assert_eq!(by_word.total, 1, "every word of the query has to match");
    assert_eq!(ids(&by_word.rows), iced_rule(&all.rows, "thread"));

    let by_title_word = holder.sessions_window(0, MAX_WINDOW, "chat");
    assert_eq!(by_title_word.total, 3, "a shared word matches three titles");
    assert_eq!(ids(&by_title_word.rows), iced_rule(&all.rows, "chat"));

    // The unfiltered count of the state is untouched by a query read.
    assert_eq!(holder.view().total_sessions, 4);
}

#[test]
fn the_window_filters_by_the_app_rule_not_a_second_one() {
    // Folded accents and every word of the query, decided by `crates/app`: the
    // window never matches on its own.
    let state = state_of(&[("Deleg\u{e1} pizza", "web"), ("Deleg\u{e1} pan", "web")]);
    let query = "delega  PIZZA";

    let narrowed = window::window(&state, query, 0, 10, NOW);
    let filtered = gentle_app::update_chats(
        state.clone(),
        Input::Intent(Intent::SetSessionQuery(query.to_owned())),
    )
    .0;
    let expected: Vec<String> = filtered
        .visible_sessions()
        .into_iter()
        .map(|session| session.path.clone())
        .collect();

    assert_eq!(narrowed.total, 1, "one of the two titles matches");
    assert_eq!(ids(&narrowed.rows), expected);
}

#[test]
fn a_length_above_the_cap_is_cut_to_two_hundred() {
    let home = Home::new();
    home.many(250);
    let (holder, _) = settled(&home);

    let wide = holder.sessions_window(0, 5_000, "");
    assert_eq!(wide.total, 250);
    assert_eq!(
        wide.rows.len(),
        MAX_WINDOW as usize,
        "a page, never the whole list"
    );
    assert_eq!(holder.sessions_window(240, 5_000, "").rows.len(), 10);
}

/// A state whose list is `items`, seeded through the reducer's public input.
fn state_of(items: &[(&str, &str)]) -> AppState {
    let items: Vec<SessionItem> = items
        .iter()
        .enumerate()
        .map(|(index, (title, project))| SessionItem {
            path: format!("/sessions/{project}/s{index}.jsonl"),
            title: (*title).to_owned(),
            cwd: format!("C:/dev/{project}"),
            modified_ms: 0,
            message_count: 1,
        })
        .collect();
    gentle_app::update_chats(
        AppState::default(),
        Input::SessionsPrepared(prepare_sessions(items)),
    )
    .0
}
