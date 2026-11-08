//! `refresh_with_progress`: the full pass reporting how many files it has
//! folded, so a caller can show what it already knows while a big home is
//! still being read.

use std::cell::RefCell;

use super::test_home::{header, user, Home};
use super::RefreshStats;

#[test]
fn progress_climbs_over_every_file_and_ends_at_the_total() {
    let home = Home::new();
    home.write("uno", &[header("uno"), user("hola")]);
    home.write("dos", &[header("dos"), user("chau")]);
    let mut index = home.index();
    let calls = RefCell::new(Vec::new());
    let visible = RefCell::new(Vec::new());
    index
        .refresh_with_progress(|index, done, total| {
            calls.borrow_mut().push((done, total));
            visible.borrow_mut().push(index.entries().len());
        })
        .unwrap();
    let calls = calls.into_inner();
    let visible = visible.into_inner();
    let done: Vec<usize> = calls.iter().map(|&(done, _)| done).collect();
    assert!(
        done.windows(2).all(|pair| pair[0] <= pair[1]),
        "done must not go backwards: {done:?}"
    );
    assert_eq!(
        done.first().copied(),
        Some(1),
        "progress starts after the first file: {calls:?}"
    );
    assert_eq!(
        done.last().copied(),
        Some(2),
        "the pass ends reporting every file: {calls:?}"
    );
    assert!(
        calls.iter().all(|&(_, total)| total == 2),
        "total is the number of session files: {calls:?}"
    );
    assert!(
        done.contains(&1) && done.contains(&2),
        "every file is reported once: {calls:?}"
    );
    assert_eq!(
        visible.len(),
        calls.len(),
        "the callback always sees the index: {visible:?}"
    );
    assert_eq!(
        visible.last().copied(),
        Some(2),
        "the last call sees every entry: {visible:?}"
    );
    assert_eq!(index.entries().len(), 2);
}

#[test]
fn progress_leaves_the_same_entries_and_stats_as_a_plain_refresh() {
    let home = Home::new();
    home.write("uno", &[header("uno"), user("hola")]);
    home.write(
        "dos",
        &[header("dos"), user("chau"), user("y despues otra")],
    );
    home.write("tres", &[header("tres")]);

    let mut plain = home.index();
    let plain_stats = plain.refresh().unwrap();
    let mut with_progress = home.index();
    let progress_stats = with_progress.refresh_with_progress(|_, _, _| {}).unwrap();
    assert_eq!(
        progress_stats, plain_stats,
        "the wrapper must cost exactly what the plain pass costs"
    );
    assert_eq!(
        with_progress.entries(),
        plain.entries(),
        "the wrapper must leave exactly the same list"
    );

    // A second pass folds nothing: both spellings agree there too.
    assert_eq!(
        plain.refresh().unwrap(),
        with_progress.refresh_with_progress(|_, _, _| {}).unwrap()
    );
    assert_eq!(with_progress.entries(), plain.entries());
    assert_eq!(
        (
            progress_stats.files,
            progress_stats.rescanned,
            progress_stats.appended,
            progress_stats.removed,
        ),
        (3, 3, 0, 0),
        "a cold pass parses every file from the start"
    );
}

#[test]
fn no_session_files_calls_progress_at_most_once_with_zeroes() {
    let home = Home::new();
    let mut index = home.index();
    let calls = RefCell::new(Vec::new());
    let stats = index
        .refresh_with_progress(|index, done, total| {
            calls
                .borrow_mut()
                .push((index.entries().len(), done, total));
        })
        .unwrap();
    assert_eq!(stats, RefreshStats::default());
    assert!(index.entries().is_empty());
    let calls = calls.into_inner();
    assert!(
        calls.len() <= 1,
        "an empty home reports at most once: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .all(|&(visible, done, total)| visible == 0 && done == 0 && total == 0),
        "an empty home only ever reports (0, 0): {calls:?}"
    );
}
