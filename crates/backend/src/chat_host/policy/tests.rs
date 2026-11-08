//! The rules on their own: no process, no lock, only a decision for a `now` the
//! test chooses.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::*;

const SEC: Duration = Duration::from_secs(1);

fn key(name: &str) -> ChatKey {
    ChatKey::Saved(PathBuf::from(name))
}

fn view(name: &str, last_active: Instant) -> ChatView {
    ChatView {
        key: key(name),
        working: false,
        foreground: false,
        pending: false,
        last_active,
    }
}

fn working(mut chat: ChatView) -> ChatView {
    chat.working = true;
    chat
}

fn foreground(mut chat: ChatView) -> ChatView {
    chat.foreground = true;
    chat
}

fn pending(mut chat: ChatView) -> ChatView {
    chat.pending = true;
    chat
}

#[test]
fn defaults_are_four_live_two_working_and_ten_minutes() {
    assert_eq!(
        Limits::default(),
        Limits::new(4, 2, Duration::from_secs(600))
    );
}

#[test]
fn under_the_live_cap_an_open_needs_no_reap() {
    let now = Instant::now();
    let views = vec![
        view("a", now),
        view("b", now + SEC),
        view("c", now + 2 * SEC),
    ];

    assert_eq!(
        admit(&Limits::new(4, 2, 60 * SEC), &views),
        Ok(Admission::Open)
    );
}

#[test]
fn at_the_live_cap_the_least_recently_used_idle_chat_is_reaped() {
    let now = Instant::now();
    let views = vec![
        view("newest", now + 20 * SEC),
        view("oldest", now),
        view("older", now + 10 * SEC),
    ];

    assert_eq!(
        admit(&Limits::new(3, 2, 600 * SEC), &views),
        Ok(Admission::Reap(key("oldest")))
    );
}

#[test]
fn the_lru_choice_skips_the_foreground_and_the_working_chats() {
    let now = Instant::now();
    let views = vec![
        working(view("working", now)),
        foreground(view("foreground", now + SEC)),
        view("idle", now + 2 * SEC),
    ];

    assert_eq!(
        admit(&Limits::new(3, 2, 600 * SEC), &views),
        Ok(Admission::Reap(key("idle")))
    );
}

#[test]
fn a_starting_child_is_not_reaped_to_make_room() {
    let now = Instant::now();
    let views = vec![pending(view("starting", now))];

    assert_eq!(
        admit(&Limits::new(1, 2, 600 * SEC), &views),
        Err(TooManyLive { cap: 1, live: 1 })
    );
}

#[test]
fn a_live_cap_with_nothing_reapable_is_a_typed_error() {
    let now = Instant::now();
    let views = vec![
        working(view("a", now)),
        foreground(view("b", now)),
        working(view("c", now)),
    ];

    let refused = admit(&Limits::new(3, 2, 600 * SEC), &views).unwrap_err();

    assert_eq!(refused, TooManyLive { cap: 3, live: 3 });
    assert!(refused.to_string().contains("cap 3"));
}

#[test]
fn identical_activity_is_broken_by_key_order() {
    let now = Instant::now();
    let views = vec![view("b", now), view("a", now)];

    assert_eq!(
        admit(&Limits::new(2, 2, 600 * SEC), &views),
        Ok(Admission::Reap(key("a")))
    );
}

#[test]
fn reap_idle_takes_only_the_unprotected_chats_past_the_window() {
    let now = Instant::now();
    let limits = Limits::new(4, 2, 10 * SEC);
    let views = vec![
        view("past", now),
        view("exactly", now + SEC), // 10 s idle: the window itself, kept
        working(view("busy", now)),
        foreground(view("open", now)),
        view("fresh", now + 10 * SEC),
    ];

    assert_eq!(
        reap_idle(&limits, &views, now + 11 * SEC),
        vec![key("past")]
    );
}

#[test]
fn a_working_chat_is_never_reaped_however_idle() {
    let now = Instant::now();
    let limits = Limits::new(4, 2, 10 * SEC);
    let views = vec![working(view("busy", now))];

    assert!(reap_idle(&limits, &views, now + 3600 * SEC).is_empty());
}

#[test]
fn reap_idle_reaps_the_oldest_first() {
    let now = Instant::now();
    let limits = Limits::new(4, 2, 10 * SEC);
    let views = vec![view("young", now + 20 * SEC), view("old", now)];

    assert_eq!(
        reap_idle(&limits, &views, now + 31 * SEC),
        vec![key("old"), key("young")]
    );
}

#[test]
fn the_working_cap_counts_only_the_working_chats() {
    let now = Instant::now();
    let views = vec![
        working(view("a", now)),
        view("b", now),
        working(view("c", now)),
    ];

    assert_eq!(working_count(&views), 2);
    assert!(working_full(&Limits::new(4, 2, 600 * SEC), &views));
    assert!(!working_full(&Limits::new(4, 3, 600 * SEC), &views));
}

#[test]
fn protected_covers_the_foreground_the_working_and_the_starting() {
    let now = Instant::now();

    assert!(foreground(view("a", now)).protected());
    assert!(working(view("b", now)).protected());
    assert!(pending(view("c", now)).protected());
    assert!(!view("d", now).protected());
}
