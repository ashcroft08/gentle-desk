//! Windowless tests of how the web side learns that its page of rows is stale
//! (S2, S3): the revision carried by every slice, and the first read that
//! answers the current state without emitting.
//!
//! A temp pi home, a real refresh pass, and the slices recorded instead of
//! emitted: no Tauri app, no window, no real home of the user.

mod session_seam;
use session_seam::{settled, Home};

use gentle_app::{ChatId, Input};
use gentle_desk_lib::contract::ListStatusView;

#[test]
fn a_settled_pass_bumps_the_revision_and_a_notice_does_not() {
    let home = Home::new();
    home.many(4);
    let (holder, recorder) = settled(&home);

    let pushes = recorder.pushes();
    let first = pushes.first().expect("the pass reports as it goes");
    let last = pushes.last().expect("the list settles");
    assert!(
        first.sessions_revision < last.sessions_revision,
        "the list moved and the revision says so"
    );
    assert_eq!(last.total_sessions, 4);
    assert_eq!(holder.view().sessions_revision, last.sessions_revision);

    // A push that does not touch the list leaves the revision alone, so the web
    // side keeps the window it already holds.
    let after = holder.fold_input(Input::ChatClosed {
        chat: ChatId::of_session("/sessions/p/never-opened.jsonl"),
        expected: false,
    });
    assert_eq!(after.notices.len(), 1, "the notice is what changed");
    assert_eq!(
        after.sessions_revision, last.sessions_revision,
        "an unrelated push never invalidates a window"
    );
    assert_eq!(after.total_sessions, 4);
}

#[test]
fn a_new_list_bumps_the_revision_even_when_it_keeps_its_size() {
    // The status alone can hide a swap: a settled list replaced by another list
    // of the same length is a new list, and the web side has to re-query.
    let holder = gentle_desk_lib::bridge::AppStateHolder::default();
    let before = holder.view().sessions_revision;
    holder.fold_input(Input::SessionsLoaded(Vec::new()));
    let after = holder.view();
    assert_eq!(after.list_status, ListStatusView::Complete);
    assert!(
        after.sessions_revision > before,
        "the first settled list is a change"
    );
}

#[test]
fn the_first_read_answers_the_last_push_and_emits_nothing() {
    let home = Home::new();
    home.many(3);
    let (holder, recorder) = settled(&home);

    let reads_before = recorder.count();
    let held = holder.view();
    let reads_after = recorder.count();

    let last = recorder.pushes().last().cloned().expect("the list settles");
    assert_eq!(
        held, last,
        "the first read is the same slice the last push carried"
    );
    assert_eq!(held.list_status, ListStatusView::Complete);
    assert_eq!(reads_after, reads_before, "reading the state emits nothing");
}
