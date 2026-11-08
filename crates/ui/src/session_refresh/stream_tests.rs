//! What a pass reports while it runs: the partial lists of the first full pass
//! and the one final item every pass ends with.

use super::test_home::*;
use super::*;
use iced::futures::executor::block_on;
use iced::futures::StreamExt;

use crate::refresh_queue::Job;

/// Every item one pass emits, with a clock the test drives instead of waiting.
fn items(job: Job, shared: Shared, mut now: impl FnMut() -> Duration) -> Vec<Update> {
    let mut items = Vec::new();
    let mut emit = |update: Update| items.push(update);
    work(&job, &shared, &mut now, &mut emit);
    items
}

fn partials(items: &[Update]) -> Vec<(Vec<String>, usize, usize)> {
    items
        .iter()
        .filter_map(|item| match item {
            Update::Progress(progress) => {
                Some((names(&progress.sessions), progress.done, progress.total))
            }
            Update::Refreshed(_) => None,
        })
        .collect()
}

fn refreshed(items: &[Update]) -> Vec<&Refreshed> {
    items
        .iter()
        .filter_map(|item| match item {
            Update::Refreshed(Ok(refreshed)) => Some(refreshed),
            _ => None,
        })
        .collect()
}

/// S3: the restored cache is shown before the disk is read again, and the final
/// list is the reconciled one: the chat the cache never saw appears.
#[test]
fn the_first_full_pass_reports_the_restored_cache_before_it_reads_the_disk() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "a-uno");
    let snapshot = warm_snapshot(&home);

    // A later run starts warm from that snapshot, and a new chat appeared since.
    let mut restored = home.index();
    restored.restore(snapshot);
    home.session("--C--q--", "b.jsonl", "b-uno");
    let shared = Shared::new(Arc::new(Mutex::new(restored)), None);

    let items = items(Job::Full, shared, || Duration::ZERO);

    let Update::Progress(seed) = &items[0] else {
        panic!("the first item is the list the cache already knew");
    };
    assert_eq!(names(&seed.sessions), ["a-uno"]);
    assert_eq!((seed.done, seed.total), (0, 0), "nothing read yet");

    let listed = refreshed(&items);
    assert_eq!(listed.len(), 1, "exactly one final item");
    assert_eq!(titles(listed[0]), ["a-uno", "b-uno"]);
    assert!(matches!(items.last(), Some(Update::Refreshed(_))));
}

#[test]
fn every_partial_counts_at_most_what_the_pass_found() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "a-uno");
    home.session("--C--p--", "b.jsonl", "b-uno");
    let shared = Shared::new(index_in(home.sessions()), None);

    let mut step = 0;
    let items = items(Job::Full, shared, || {
        step += 1;
        Duration::from_secs(step)
    });
    let partials = partials(&items);
    assert!(!partials.is_empty());
    for (_, done, total) in &partials {
        assert!(done <= total, "{done} of {total} is not progress");
    }
    assert_eq!(refreshed(&items).len(), 1);
    assert!(matches!(items.last(), Some(Update::Refreshed(_))));
}

/// The clock is what bounds the messages, not the number of files: with a clock
/// that never moves, the seed is the only partial a pass may send.
#[test]
fn a_clock_that_never_moves_sends_one_partial_and_the_final_item_last() {
    let home = Home::new();
    for n in 0..5 {
        home.session("--C--p--", &format!("s{n}.jsonl"), &format!("s{n}-uno"));
    }
    let shared = Shared::new(index_in(home.sessions()), None);

    let items = items(Job::Full, shared, || Duration::ZERO);
    assert_eq!(partials(&items).len(), 1, "the seed only");
    assert_eq!(items.len(), 2);
    let listed = refreshed(&items);
    assert_eq!(listed.len(), 1);
    assert_eq!(titles(listed[0]).len(), 5);
}

/// A clock that advances a second per ask lets every report through: the
/// per-file ones and the end of the pass, which is what a slow home sees.
#[test]
fn a_clock_that_advances_past_the_interval_sends_every_report() {
    let home = Home::new();
    for n in 0..5 {
        home.session("--C--p--", &format!("s{n}.jsonl"), &format!("s{n}-uno"));
    }
    let shared = Shared::new(index_in(home.sessions()), None);

    let mut step = 0;
    let items = items(Job::Full, shared, || {
        step += 1;
        Duration::from_secs(step)
    });
    // The seed, one report per file (five) and the pass's own last report.
    assert_eq!(partials(&items).len(), 7);
    assert!(matches!(items.last(), Some(Update::Refreshed(_))));
}

/// A later pass already shows a complete list: a loading state there flickers.
#[test]
fn a_second_full_pass_sends_no_partials() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "a-uno");
    let shared = Shared::new(index_in(home.sessions()), None);

    let first = items(Job::Full, shared.clone(), || Duration::ZERO);
    assert!(!partials(&first).is_empty());

    let second = items(Job::Full, shared, || Duration::ZERO);
    assert_eq!(partials(&second), vec![]);
    assert_eq!(refreshed(&second).len(), 1);
}

#[test]
fn a_targeted_pass_sends_no_partials() {
    let home = Home::new();
    let a = home.session("--C--p--", "a.jsonl", "a-uno");
    let shared = Shared::new(index_in(home.sessions()), None);

    let items = items(Job::Paths(vec![a]), shared, || Duration::ZERO);
    assert_eq!(items.len(), 1, "only its final item");
    assert!(matches!(items[0], Update::Refreshed(Ok(_))));
}

/// The stream the UI consumes: same items, and it ends after the final one.
#[test]
fn the_stream_hands_over_the_items_and_ends_after_the_final_one() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "a-uno");
    let shared = Shared::new(index_in(home.sessions()), None);

    let items: Vec<Update> = block_on(stream(Job::Full, shared).collect());
    assert!(matches!(items.first(), Some(Update::Progress(_))));
    assert!(matches!(items.last(), Some(Update::Refreshed(Ok(_)))));
    assert_eq!(refreshed(&items).len(), 1);
    assert_eq!(titles(refreshed(&items)[0]), ["a-uno"]);
}
