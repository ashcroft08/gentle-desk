use super::*;

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}

#[test]
fn nothing_pending_starts_nothing() {
    let mut queue = RefreshQueue::default();
    assert_eq!(queue.start_next(), None);
    assert!(!queue.is_busy());
}

#[test]
fn a_full_pass_is_served_once_and_then_the_queue_is_empty() {
    let mut queue = RefreshQueue::default();
    queue.request_full();
    assert_eq!(queue.start_next(), Some(Job::Full));
    queue.finish();
    assert_eq!(queue.start_next(), None);
}

#[test]
fn paths_arriving_while_a_pass_runs_wait_for_it_and_are_not_dropped() {
    let mut queue = RefreshQueue::default();
    queue.request_paths(vec![path("a.jsonl")]);
    assert_eq!(queue.start_next(), Some(Job::Paths(vec![path("a.jsonl")])));
    assert!(queue.is_busy());
    // A second pass cannot start while the first one has the index.
    assert_eq!(queue.start_next(), None);
    queue.request_paths(vec![path("b.jsonl"), path("c.jsonl")]);
    queue.finish();
    assert!(!queue.is_busy());
    assert_eq!(
        queue.start_next(),
        Some(Job::Paths(vec![path("b.jsonl"), path("c.jsonl")]))
    );
}

#[test]
fn a_path_reported_twice_is_served_once() {
    let mut queue = RefreshQueue::default();
    queue.request_paths(vec![path("a.jsonl"), path("a.jsonl")]);
    queue.request_paths(vec![path("a.jsonl")]);
    assert_eq!(queue.start_next(), Some(Job::Paths(vec![path("a.jsonl")])));
    queue.finish();
    assert_eq!(queue.start_next(), None);
}

#[test]
fn a_full_pass_supersedes_pending_paths_and_is_served_first() {
    let mut queue = RefreshQueue::default();
    queue.request_paths(vec![path("a.jsonl"), path("b.jsonl")]);
    queue.request_full();
    assert_eq!(queue.start_next(), Some(Job::Full));
    queue.finish();
    // The full pass covered them: serving them again would repeat the work.
    assert_eq!(queue.start_next(), None);
}

#[test]
fn paths_reported_while_a_full_pass_waits_add_nothing() {
    let mut queue = RefreshQueue::default();
    queue.request_full();
    queue.request_paths(vec![path("a.jsonl")]);
    assert_eq!(queue.start_next(), Some(Job::Full));
    queue.finish();
    assert_eq!(queue.start_next(), None);
}

#[test]
fn a_full_pass_requested_while_paths_wait_is_not_lost_behind_them() {
    let mut queue = RefreshQueue::default();
    queue.request_paths(vec![path("a.jsonl")]);
    queue.request_full();
    queue.request_paths(vec![path("b.jsonl")]);
    assert_eq!(queue.start_next(), Some(Job::Full));
    queue.finish();
    assert_eq!(queue.start_next(), None);
}

#[test]
fn the_pending_path_cap_still_serves_paths_and_one_more_collapses_into_a_full_pass() {
    let mut queue = RefreshQueue::default();
    let paths: Vec<PathBuf> = (0..MAX_PENDING_PATHS)
        .map(|i| path(&i.to_string()))
        .collect();
    queue.request_paths(paths);
    match queue.start_next() {
        Some(Job::Paths(served)) => assert_eq!(served.len(), MAX_PENDING_PATHS),
        other => panic!("expected the paths pass, got {other:?}"),
    }
    queue.finish();

    let overflowing: Vec<PathBuf> = (0..=MAX_PENDING_PATHS)
        .map(|i| path(&i.to_string()))
        .collect();
    queue.request_paths(overflowing);
    assert_eq!(queue.start_next(), Some(Job::Full));
    queue.finish();
    // The paths the full pass superseded are not served afterwards.
    assert_eq!(queue.start_next(), None);
}

#[test]
fn finishing_when_no_pass_runs_is_harmless() {
    let mut queue = RefreshQueue::default();
    queue.finish();
    assert!(!queue.is_busy());
    assert_eq!(queue.start_next(), None);
    queue.request_paths(vec![path("a.jsonl")]);
    assert_eq!(queue.start_next(), Some(Job::Paths(vec![path("a.jsonl")])));
}

#[test]
fn paths_reported_after_a_full_pass_started_are_served_after_it() {
    let mut queue = RefreshQueue::default();
    queue.request_full();
    assert_eq!(queue.start_next(), Some(Job::Full));
    // The change happened while the full pass was already scanning.
    queue.request_paths(vec![path("late.jsonl")]);
    queue.finish();
    assert_eq!(
        queue.start_next(),
        Some(Job::Paths(vec![path("late.jsonl")]))
    );
}
