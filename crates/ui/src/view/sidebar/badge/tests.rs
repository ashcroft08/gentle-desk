//! The badges on their own: no widget, no app, just the activity a row is
//! handed and the plain data it draws.

use super::*;
use gentle_app::ChatActivity::*;

/// Most rows in a big home are idle ones, and an idle chat has no state worth
/// a mark: the row draws no badge for it.
#[test]
fn an_idle_chat_draws_nothing() {
    assert_eq!(Badge::on(Idle), None);
}

#[test]
fn a_chat_running_a_turn_shows_one_accent_dot() {
    let badge = Badge::on(Working).expect("a working chat shows something");

    assert_eq!(badge.mark, "●");
    assert_eq!(badge.role, ColourRole::Accent);
    assert!(!badge.bold_title, "a working title asks for no emphasis");
}

/// A finished reply is the one state the user should notice at a glance from
/// across the list, so it gets both the mark and a bold title.
#[test]
fn a_finished_unread_reply_shows_a_success_dot_and_a_bold_title() {
    let badge = Badge::on(FinishedUnread).expect("an unread reply shows something");

    assert_eq!(badge.mark, "●");
    assert_eq!(badge.role, ColourRole::Success);
    assert!(badge.bold_title);
}

#[test]
fn a_chat_with_a_pending_dialog_shows_a_warning_exclamation() {
    let badge = Badge::on(NeedsAttention).expect("a dialog shows something");

    assert_eq!(badge.mark, "!");
    assert_eq!(badge.role, ColourRole::Warning);
    assert!(!badge.bold_title);
}

#[test]
fn a_chat_whose_child_died_shows_an_error_cross() {
    let badge = Badge::on(Ended).expect("an ended chat shows something");

    assert_eq!(badge.mark, "×");
    assert_eq!(badge.role, ColourRole::Error);
    assert!(!badge.bold_title);
}

/// A chat whose prompt waits for a slot gets a quiet clock-like mark: the
/// queued work is on its way, not failed, so it draws in the muted colour.
#[test]
fn a_chat_queued_for_a_slot_shows_a_muted_ellipsis() {
    let badge = Badge::on(Queued).expect("a queued chat shows something");

    assert_eq!(badge.mark, "…");
    assert_eq!(badge.role, ColourRole::Muted);
    assert!(!badge.bold_title, "a queued title asks for no emphasis");
}

/// Every badge fits in the room the title gives up for it, whatever the chat
/// is doing: the row keeps its height and its single line.
#[test]
fn every_mark_fits_in_the_title_room_it_costs() {
    for activity in [Working, FinishedUnread, Queued, NeedsAttention, Ended] {
        let badge = Badge::on(activity).expect("every non-idle chat draws a badge");
        assert!(
            badge.mark.chars().count() <= Badge::TITLE_ROOM,
            "{activity:?} marks itself with {badge:?}, wider than the {TITLE} characters it costs",
            TITLE = Badge::TITLE_ROOM,
        );
    }
}
