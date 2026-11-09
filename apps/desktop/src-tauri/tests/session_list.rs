//! Windowless tests of the session list slice: a temp pi home, a real refresh
//! pass on its own thread, and the slices it pushes recorded instead of emitted.
//! No Tauri app, no window, no real home of the user.
//!
//! The reducer is the source of truth for what a push means: the list starts
//! `Loading` and only `Input::SessionsPrepared` makes it complete, so these tests
//! check the ORDER of the pushes (reading first, never a false empty) and the
//! rows the state really holds.

mod session_seam;
use session_seam::{file_name, gated, homeless, service, Home};

use std::time::Duration;

use gentle_app::{prepare_sessions, AppState, Input, SessionItem};
use gentle_desk_lib::contract::ListStatusView;
use gentle_desk_lib::sessions::window;

#[test]
fn the_first_pass_pushes_a_reading_list_then_the_whole_one() {
    let home = Home::new();
    home.session("p", "a.jsonl", "first chat");
    home.session("p", "b.jsonl", "second chat");
    home.session("p", "c.jsonl", "third chat");
    let (service, holder, recorder) = service(&home);

    service.request();
    let pushes = recorder.wait_for_complete();

    // Reading first: the window opens while the pass still reads the disk.
    let first = pushes.first().expect("the pass reports as it goes");
    assert!(
        matches!(first.list_status, ListStatusView::Loading { .. }),
        "the first push is a list still being read, got {:?}",
        first.list_status
    );
    // Only the final push may call the list complete.
    let (last, earlier) = pushes.split_last().expect("at least one push");
    assert!(
        earlier
            .iter()
            .all(|view| view.list_status != ListStatusView::Complete),
        "a partial list never claims to be the whole truth"
    );
    assert_eq!(last.list_status, ListStatusView::Complete);
    assert_eq!(last.total_sessions, 3, "one row per session file");

    // The rows are not part of the push: they come back as a window (S3).
    let rows = holder.sessions_window(0, window::MAX_WINDOW, "");
    assert_eq!(rows.rows.len(), 3);
    let mut names: Vec<&str> = rows.rows.iter().map(|row| file_name(&row.id)).collect();
    names.sort();
    assert_eq!(names, vec!["a.jsonl", "b.jsonl", "c.jsonl"]);
    for row in &rows.rows {
        assert_eq!(
            row.project, "p",
            "the project is the session's last directory"
        );
        assert_eq!(row.message_count, 1);
        assert!(
            !row.modified_ago.is_empty(),
            "every row carries its age text"
        );
    }
    let titles: Vec<&str> = rows.rows.iter().map(|row| row.title.as_str()).collect();
    assert!(titles.contains(&"first chat"), "got {titles:?}");

    // The holder is what the last push said: the web side and the state agree.
    let held = holder.view();
    assert_eq!(held.list_status, ListStatusView::Complete);
    assert_eq!(held.total_sessions, 3);
}

#[test]
fn a_big_list_is_pushed_without_rows_and_read_back_by_the_window() {
    let home = Home::new();
    home.many(60);
    let (service, holder, recorder) = service(&home);

    service.request();
    let pushes = recorder.wait_for_complete();

    let last = pushes.last().expect("the list settles");
    assert_eq!(last.list_status, ListStatusView::Complete);
    assert_eq!(last.total_sessions, 60, "the push carries the count only");
    let rows = holder.sessions_window(0, window::MAX_WINDOW, "");
    assert_eq!(rows.total, 60);
    let mut ids: Vec<&str> = rows.rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids.len(), 60, "no row is served twice");
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 60);
}

#[test]
fn a_second_refresh_during_a_pass_queues_it_instead_of_starting_another() {
    let home = Home::new();
    let (gate, service, _, recorder) = gated(&home);
    let counts = gate.counts();

    service.request();
    gate.enter();
    // The reducer asks again while the first pass still holds the index.
    service.request();
    assert_eq!(
        counts.jobs().len(),
        1,
        "nothing starts beside a running pass"
    );

    gate.release();
    // The queued request runs once the first pass settled, and never beside it.
    gate.enter();
    gate.release();
    let pushes = recorder.wait_for_complete();

    assert_eq!(counts.most_at_once(), 1, "one pass at a time");
    assert_eq!(
        counts.jobs().len(),
        2,
        "the queued pass ran after the settle"
    );
    assert_eq!(
        pushes.last().expect("the queued pass pushed").list_status,
        ListStatusView::Complete
    );
}

#[test]
fn a_missing_home_keeps_the_list_reading_and_never_claims_it_is_empty() {
    let (service, holder, recorder) = homeless();

    service.request();
    // Nothing can run without an index; give a stray thread the chance to show.
    std::thread::sleep(Duration::from_millis(50));

    assert!(recorder.pushes().is_empty(), "no pass runs without a home");
    let view = holder.view();
    assert_eq!(
        view.list_status,
        ListStatusView::Loading { done: 0, total: 0 },
        "the reducer's initial state is reading, not an empty list"
    );
    assert_eq!(view.total_sessions, 0);
}

#[test]
fn a_home_with_no_sessions_settles_complete_and_empty_after_reading() {
    // A home is here, its sessions directory is not: the pass lists nothing.
    let home = Home::new();
    let (service, _, recorder) = service(&home);

    service.request();
    let pushes = recorder.wait_for_complete();

    let first = pushes.first().expect("the pass reports as it goes");
    assert!(
        matches!(first.list_status, ListStatusView::Loading { .. }),
        "an empty list is never claimed before the pass ends, got {:?}",
        first.list_status
    );
    assert_eq!(first.total_sessions, 0);
    let last = pushes.last().expect("the pass settles");
    assert_eq!(last.list_status, ListStatusView::Complete);
    assert_eq!(last.total_sessions, 0);
}

#[test]
fn a_row_carries_the_texts_the_runtime_rules_produce() {
    // The mapping alone, with a pinned clock: the age text and the title are the
    // rules `crates/runtime` owns, applied once, not re-implemented here.
    let long_title = "a".repeat(40);
    let item = SessionItem {
        path: "/sessions/p/a.jsonl".to_owned(),
        title: long_title,
        cwd: "C:\\dev\\proj".to_owned(),
        modified_ms: 1_000,
        message_count: 7,
    };
    let state = gentle_app::update_chats(
        AppState::default(),
        Input::SessionsPrepared(prepare_sessions(vec![item])),
    )
    .0;

    let rows = window::window(&state, "", 0, window::MAX_WINDOW, 1_000 + 3 * 60 * 60_000).rows;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "/sessions/p/a.jsonl");
    assert_eq!(rows[0].project, "proj");
    assert_eq!(rows[0].message_count, 7);
    assert_eq!(rows[0].modified_ago, "3h");
    assert_eq!(
        rows[0].title.chars().count(),
        33,
        "32 characters and the cut"
    );
    assert!(rows[0].title.ends_with('\u{2026}'), "the cut is marked");
}
