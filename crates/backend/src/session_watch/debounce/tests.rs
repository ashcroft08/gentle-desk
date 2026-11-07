//! The debouncer, with time passed in so no test sleeps.

use super::*;

const QUIET: Duration = Duration::from_millis(100);
const MAX_WAIT: Duration = Duration::from_millis(400);
const MAX: usize = MAX_BATCH_PATHS;

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn debouncer() -> Debouncer {
    Debouncer::new(QUIET, MAX_WAIT)
}

fn names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

#[test]
fn the_default_quiet_window_coalesces_a_created_files_two_events() {
    assert_eq!(super::QUIET, Duration::from_millis(250));
    assert_eq!(super::MAX_WAIT, Duration::from_secs(1));
    let mut debouncer = Debouncer::default();
    // `create` then `modify` of one file, 0.5 ms apart.
    debouncer.push(ms(10), paths(&["a/x.jsonl"]), false);
    debouncer.push(Duration::from_micros(10_500), paths(&["a/x.jsonl"]), false);
    assert_eq!(debouncer.take_if_due(ms(200)), None);
    let batch = debouncer.take_if_due(ms(261)).expect("the batch is due");
    assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
    assert!(!batch.rescan);
    assert_eq!(debouncer.take(), None);
}

#[test]
fn a_burst_becomes_one_batch_when_it_goes_quiet() {
    let mut debouncer = debouncer();
    for (index, name) in ["a/1.jsonl", "a/2.jsonl", "b/3.jsonl"].iter().enumerate() {
        debouncer.push(ms(index as u64 * 10), paths(&[name]), false);
    }
    assert_eq!(debouncer.deadline(), Some(ms(20 + 100)));
    assert_eq!(debouncer.take_if_due(ms(119)), None);
    let batch = debouncer.take_if_due(ms(120)).expect("the batch is due");
    assert_eq!(names(&batch.paths), ["a/1.jsonl", "a/2.jsonl", "b/3.jsonl"]);
    assert!(!batch.rescan);
}

#[test]
fn a_steady_stream_cannot_postpone_the_batch_beyond_the_max_wait() {
    let mut debouncer = debouncer();
    // A push every 10 ms for a minute: the quiet window never passes.
    let mut now = Duration::ZERO;
    // When the open batch started; the next push opens the next one.
    let mut opened = Duration::ZERO;
    let mut emitted = 0usize;
    for _ in 0..6_000 {
        debouncer.push(now, paths(&["a/x.jsonl"]), false);
        let deadline = debouncer.deadline().expect("a batch is open");
        assert!(
            deadline <= opened + MAX_WAIT,
            "the batch was postponed to {deadline:?}, past the max wait"
        );
        match debouncer.take_if_due(now) {
            Some(batch) => {
                assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
                emitted += 1;
                opened = now + ms(10);
            }
            None => assert!(now < deadline, "the batch was due at {now:?}"),
        }
        now += ms(10);
    }
    assert!(emitted > 0, "a steady stream was never emitted");
}

#[test]
fn duplicates_collapse_into_one_path() {
    let mut debouncer = debouncer();
    debouncer.push(ms(1), paths(&["a/x.jsonl", "a/x.jsonl"]), false);
    debouncer.push(ms(2), paths(&["a/x.jsonl", "a/y.jsonl"]), false);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(names(&batch.paths), ["a/x.jsonl", "a/y.jsonl"]);
}

#[test]
fn the_batch_order_is_deterministic() {
    let mut first = debouncer();
    let mut second = debouncer();
    first.push(
        ms(1),
        paths(&["b/z.jsonl", "a/z.jsonl", "a/a.jsonl"]),
        false,
    );
    second.push(
        ms(1),
        paths(&["a/a.jsonl", "b/z.jsonl", "a/z.jsonl"]),
        false,
    );
    let expected = ["a/a.jsonl", "a/z.jsonl", "b/z.jsonl"];
    assert_eq!(names(&first.take().unwrap().paths), expected);
    // A different insertion order yields the same sequence.
    assert_eq!(names(&second.take().unwrap().paths), expected);
}

#[test]
fn more_than_max_batch_paths_collapses_to_a_full_pass() {
    let mut debouncer = debouncer();
    let burst: Vec<PathBuf> = (0..MAX + 1)
        .map(|index| PathBuf::from(format!("p/{index}.jsonl")))
        .collect();
    debouncer.push(ms(1), burst, false);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(batch.paths.len(), 0, "a collapsed batch carries no paths");
    assert!(batch.rescan, "the full pass replaces the paths");
    // Exactly the limit is still a list of paths.
    let mut debouncer = self::debouncer();
    let burst: Vec<PathBuf> = (0..MAX)
        .map(|index| PathBuf::from(format!("p/{index}.jsonl")))
        .collect();
    debouncer.push(ms(1), burst, false);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(batch.paths.len(), MAX);
    assert!(!batch.rescan);
}

#[test]
fn a_rescan_push_keeps_the_paths_it_has() {
    let mut debouncer = debouncer();
    debouncer.push(ms(1), paths(&["a/x.jsonl"]), false);
    debouncer.push(ms(2), Vec::new(), true);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
    assert!(batch.rescan);
    // A rescan on its own is a batch too: it is the only signal of a full pass.
    let mut debouncer = self::debouncer();
    debouncer.push(ms(1), Vec::new(), true);
    assert_eq!(debouncer.deadline(), Some(ms(1) + QUIET));
    let batch = debouncer.take().expect("a batch is open");
    assert!(batch.paths.is_empty());
    assert!(batch.rescan);
}

#[test]
fn an_empty_push_does_not_open_a_batch() {
    let mut debouncer = debouncer();
    // What a classified `access` event yields: nothing to refresh.
    debouncer.push(ms(1), Vec::new(), false);
    assert_eq!(debouncer.deadline(), None);
    assert_eq!(debouncer.take_if_due(ms(1_000)), None);
    assert_eq!(debouncer.take(), None);
}

#[test]
fn an_empty_debouncer_has_no_deadline_and_yields_nothing() {
    let mut debouncer = debouncer();
    assert_eq!(debouncer.deadline(), None);
    assert_eq!(debouncer.take_if_due(Duration::ZERO), None);
    assert_eq!(debouncer.take_if_due(Duration::MAX), None);
    assert_eq!(debouncer.take(), None);
}

#[test]
fn take_yields_the_batch_and_clears_the_state() {
    let mut debouncer = debouncer();
    debouncer.push(ms(1), paths(&["a/x.jsonl"]), true);
    assert_eq!(debouncer.deadline(), Some(ms(1) + QUIET));
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
    assert!(batch.rescan);
    assert_eq!(debouncer.deadline(), None);
    assert_eq!(debouncer.take(), None);
    // The next push starts a fresh batch: nothing of the previous one leaks.
    debouncer.push(ms(500), paths(&["a/y.jsonl"]), false);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(names(&batch.paths), ["a/y.jsonl"]);
    assert!(
        !batch.rescan,
        "the old rescan flag leaked into the new batch"
    );
}

#[test]
fn a_collapsed_batch_does_not_leak_into_the_next_one() {
    let mut debouncer = debouncer();
    let burst: Vec<PathBuf> = (0..MAX + 1)
        .map(|index| PathBuf::from(format!("p/{index}.jsonl")))
        .collect();
    debouncer.push(ms(1), burst, false);
    assert!(debouncer.take().expect("a batch is open").rescan);
    debouncer.push(ms(2), paths(&["a/x.jsonl"]), false);
    let batch = debouncer.take().expect("a batch is open");
    assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
    assert!(!batch.rescan);
}

#[test]
fn time_moving_backwards_neither_panics_nor_reopens_the_batch() {
    let mut debouncer = debouncer();
    debouncer.push(ms(1_000), paths(&["a/x.jsonl"]), false);
    // A `now` before the last push must not underflow the deadline.
    debouncer.push(ms(10), paths(&["a/y.jsonl"]), false);
    let deadline = debouncer.deadline().expect("a batch is open");
    assert!(
        deadline <= ms(1_000) + QUIET,
        "the batch was postponed by a backwards clock: {deadline:?}"
    );
    // The due check with a backwards `now` is not due, and does not panic.
    assert_eq!(debouncer.take_if_due(Duration::ZERO), None);
    let batch = debouncer
        .take_if_due(deadline)
        .expect("the batch is due at its deadline");
    assert_eq!(names(&batch.paths), ["a/x.jsonl", "a/y.jsonl"]);
}

#[test]
fn a_zero_length_quiet_window_is_due_at_once() {
    let mut debouncer = Debouncer::new(Duration::ZERO, Duration::ZERO);
    debouncer.push(Duration::ZERO, paths(&["a/x.jsonl"]), false);
    assert_eq!(debouncer.deadline(), Some(Duration::ZERO));
    assert_eq!(
        debouncer.take_if_due(Duration::ZERO),
        Some(Batch {
            paths: paths(&["a/x.jsonl"]),
            rescan: false,
        })
    );
}

#[test]
fn huge_times_saturate_instead_of_overflowing() {
    let mut debouncer = debouncer();
    debouncer.push(Duration::MAX, paths(&["a/x.jsonl"]), false);
    // The deadline cannot be represented past `Duration::MAX`.
    assert_eq!(debouncer.deadline(), Some(Duration::MAX));
    let batch = debouncer.take_if_due(Duration::MAX).expect("due");
    assert_eq!(names(&batch.paths), ["a/x.jsonl"]);
}
