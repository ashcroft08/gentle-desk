//! The badges on their own: no window, no app, just the activity a row is
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
fn a_chat_running_a_turn_marks_itself_working_in_the_accent() {
    let badge = Badge::on(Working).expect("a working chat shows something");

    assert_eq!(badge.mark, BadgeMark::Working);
    assert_eq!(badge.role, ColourRole::Accent);
    assert!(!badge.bold_title, "a working title asks for no emphasis");
}

/// A finished reply is the one state the user should notice at a glance from
/// across the list, so it gets its own mark and a bold title.
#[test]
fn a_finished_unread_reply_marks_itself_new_reply_in_the_success_and_bold() {
    let badge = Badge::on(FinishedUnread).expect("an unread reply shows something");

    assert_eq!(badge.mark, BadgeMark::NewReply);
    assert_eq!(badge.role, ColourRole::Success);
    assert!(badge.bold_title);
}

/// The reason this rule moved to the runtime: the working turn and the unread
/// reply are different states and the new design draws them differently, so
/// the data must never hand the window two states under one mark.
#[test]
fn working_and_new_reply_are_two_different_marks() {
    let working = Badge::on(Working).expect("a working chat shows something");
    let new_reply = Badge::on(FinishedUnread).expect("an unread reply shows something");

    assert_ne!(working.mark, new_reply.mark);
}

#[test]
fn a_chat_with_a_pending_dialog_marks_itself_needs_attention_in_the_warning() {
    let badge = Badge::on(NeedsAttention).expect("a dialog shows something");

    assert_eq!(badge.mark, BadgeMark::NeedsAttention);
    assert_eq!(badge.role, ColourRole::Warning);
    assert!(!badge.bold_title);
}

#[test]
fn a_chat_whose_child_died_marks_itself_ended_in_the_error() {
    let badge = Badge::on(Ended).expect("an ended chat shows something");

    assert_eq!(badge.mark, BadgeMark::Ended);
    assert_eq!(badge.role, ColourRole::Error);
    assert!(!badge.bold_title);
}

/// A chat whose prompt waits for a slot gets a quiet mark: the queued work is
/// on its way, not failed, so it draws in the muted colour.
#[test]
fn a_chat_queued_for_a_slot_marks_itself_queued_in_the_muted() {
    let badge = Badge::on(Queued).expect("a queued chat shows something");

    assert_eq!(badge.mark, BadgeMark::Queued);
    assert_eq!(badge.role, ColourRole::Muted);
    assert!(!badge.bold_title, "a queued title asks for no emphasis");
}

/// A chat whose child is coming up gets the same quiet family as a queued
/// one (the phase invites waiting, it does not report a failure), but with a
/// mark of its own, so the window can draw "coming up" as a state of itself,
/// not as a prompt on its way.
#[test]
fn a_chat_coming_up_keeps_a_mark_of_its_own_beside_the_queued_one() {
    let badge = Badge::on(Opening).expect("an opening chat shows something");

    assert_eq!(badge.mark, BadgeMark::Opening);
    assert_eq!(badge.role, ColourRole::Muted);
    assert!(!badge.bold_title);
    assert_ne!(badge.mark, BadgeMark::Queued, "opening is not queued");
}
