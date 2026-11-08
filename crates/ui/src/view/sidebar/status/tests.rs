//! The sidebar's texts on their own: no widget, no app, just the four values
//! the view hands in and the strings it draws.

use super::*;

fn loading(done: usize, total: usize, shown: usize, query: &str) -> Texts {
    texts(ListStatus::Loading { done, total }, total, shown, query)
}

fn ready(total: usize, shown: usize, query: &str) -> Texts {
    texts(ListStatus::Ready, total, shown, query)
}

/// Nothing read yet and no cache to seed from: the list is not empty, it is
/// still coming, and the header says so instead of a final "Chats (0 of 0)".
#[test]
fn loading_with_nothing_read_yet_offers_reading_not_an_empty_list() {
    let texts = loading(0, 0, 0, "");
    assert_eq!(texts.title, "Chats · reading…");
    assert_eq!(texts.stand_in.as_deref(), Some("Reading chats…"));
}

/// Partial rows are the truth so far: they stay, and the header carries the
/// note that the list is still coming. With no search the counters would be
/// two equal numbers that read like a settled total, so they stay off it.
#[test]
fn loading_with_partial_rows_keeps_the_rows_and_notes_it_in_the_header() {
    let texts = loading(200, 50_000, 200, "");
    assert_eq!(texts.stand_in, None, "partial rows must still be drawn");
    assert_eq!(texts.title, "Chats · reading…");
}

/// A search while the pass runs is the one case where the counters earn their
/// place: they count what the query matches out of what has been read so far.
#[test]
fn loading_with_a_query_still_counts_what_matches() {
    let texts = loading(200, 50_000, 120, "auth");
    assert_eq!(texts.stand_in, None, "matching rows must still be drawn");
    assert_eq!(texts.title, "Chats (120 of 50000) · reading…");
}

/// The counters are the pass's own numbers, `done / total`, written as-is.
#[test]
fn loading_writes_the_counters_as_done_over_total() {
    let texts = loading(1_200, 50_000, 0, "");
    assert_eq!(
        texts.stand_in.as_deref(),
        Some("Reading chats… 1200 / 50000")
    );
}

/// Only a settled, empty list may claim there are no chats.
#[test]
fn ready_with_no_chats_says_so() {
    let texts = ready(0, 0, "");
    assert_eq!(texts.title, "Chats");
    assert_eq!(texts.stand_in.as_deref(), Some("No saved chats yet"));
}

#[test]
fn ready_with_rows_draws_them() {
    let texts = ready(3, 3, "");
    assert_eq!(texts.title, "Chats");
    assert_eq!(texts.stand_in, None);
}

#[test]
fn ready_with_a_query_counts_what_is_shown() {
    let texts = ready(3, 2, "auth");
    assert_eq!(texts.title, "Chats (2 of 3)");
    assert_eq!(texts.stand_in, None);
}

/// A query that matches nothing says so in both states; while loading the
/// header still says the pass is running.
#[test]
fn a_query_with_no_match_says_so_while_loading_and_when_ready() {
    let reading = loading(500, 1_000, 0, "zzz");
    assert_eq!(reading.stand_in.as_deref(), Some("No chats match"));
    assert_eq!(reading.title, "Chats (0 of 1000) · reading…");

    let settled = ready(1_000, 0, "zzz");
    assert_eq!(settled.stand_in.as_deref(), Some("No chats match"));
    assert_eq!(settled.title, "Chats (0 of 1000)");
}

/// Whatever the counters and the query, a running pass never looks settled:
/// neither "No saved chats yet" nor a bare "(0 of 0)" is ever drawn.
#[test]
fn a_running_pass_never_looks_settled() {
    // done, total, shown, query — every reachable combination.
    let cases = [
        (0, 0, 0, ""),
        (0, 0, 0, "zzz"),
        (0, 5, 0, ""),
        (3, 5, 3, ""),
        (3, 5, 0, "zzz"),
        (5, 5, 5, ""),
        (5, 5, 5, "auth"),
    ];
    for (done, total, shown, query) in cases {
        let texts = loading(done, total, shown, query);
        assert!(
            texts.title.contains("reading…"),
            "header lost the note: {}",
            texts.title
        );
        assert!(
            !texts.title.contains("(0 of 0)"),
            "header looks final: {}",
            texts.title
        );
        if query.trim().is_empty() {
            assert_eq!(
                texts.title, "Chats · reading…",
                "an unfiltered pass must not print two equal counters"
            );
        }
        assert_ne!(
            texts.stand_in.as_deref(),
            Some("No saved chats yet"),
            "an unfinished list claimed to be empty"
        );
    }
}

/// A query of nothing but blanks is not a search: the list keeps its plain
/// header and, while empty, says it is reading rather than "No chats match".
#[test]
fn a_blank_query_is_not_a_search() {
    let texts = loading(0, 0, 0, "   ");
    assert_eq!(texts.stand_in.as_deref(), Some("Reading chats…"));
    assert_eq!(texts.title, "Chats · reading…");
}
