//! The header line and the line drawn in place of the rows.
//!
//! The first full pass over a big home takes seconds, so the sidebar draws a
//! list that is still coming. This is the one place that decides whether the
//! list may call itself empty: while the pass runs it never says "No saved
//! chats yet" and its header never looks settled, which is why the decision is
//! a pure function with its own tests instead of a few `if`s in the view.

use gentle_app::ListStatus;

/// The note that says the list is still coming; the header always ends with it
/// while the pass runs, so no counter can be read as the final answer.
const READING: &str = "reading…";
/// Only drawn for a settled list that really has no chats.
const NO_CHATS: &str = "No saved chats yet";
/// Drawn when a query matches nothing, whatever the state.
const NO_MATCH: &str = "No chats match";

/// What the sidebar says, decided from four plain values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    /// The header above the search box: "Chats", "Chats (3 of 120) · reading…".
    pub title: String,
    /// The line drawn instead of rows; `None` draws the rows.
    pub stand_in: Option<String>,
}

/// The texts for one state: how complete the list is, how many chats and rows
/// there are, and what the user typed. Counters are written as they are, with
/// no separators, so "1200 / 50000" is the same shape at every size.
pub fn texts(status: ListStatus, total: usize, shown: usize, query: &str) -> Texts {
    let filtering = !query.trim().is_empty();
    let counted = if filtering {
        format!("Chats ({shown} of {total})")
    } else {
        "Chats".to_owned()
    };
    match status {
        ListStatus::Loading { done, total: files } => Texts {
            // The counters are only worth drawing next to a search, where they
            // say how much of the query's answer is in yet; unfiltered they
            // would be two equal numbers that read like a settled total, so the
            // header is just the note. A pass that has found no file yet stays
            // plain either way, so "(0 of 0)" is never drawn.
            title: if filtering && total > 0 {
                format!("Chats ({shown} of {total}) · {READING}")
            } else {
                format!("Chats · {READING}")
            },
            stand_in: if shown > 0 {
                None
            } else if filtering {
                Some(NO_MATCH.to_owned())
            } else {
                Some(reading_line(done, files))
            },
        },
        ListStatus::Ready => Texts {
            title: counted,
            stand_in: if total == 0 {
                Some(NO_CHATS.to_owned())
            } else if shown == 0 {
                Some(NO_MATCH.to_owned())
            } else {
                None
            },
        },
    }
}

/// "Reading chats… 1200 / 50000", or just "Reading chats…" before any file has
/// been found, so a zero total does not read like a fraction of nothing.
fn reading_line(done: usize, total: usize) -> String {
    if total == 0 {
        "Reading chats…".to_owned()
    } else {
        format!("Reading chats… {done} / {total}")
    }
}

#[cfg(test)]
mod tests;
