//! Candidate A (what ships today) and candidate B (cost-bounded polling).
//!
//! Both are measured with the unchanged [`SessionIndex::refresh`]: A times the
//! passes the app runs every 5 s on a warm index, B feeds those pass times to
//! the pure scheduling function in `schedule.rs`, and both then measure the
//! case the maintainer cares about: a chat that gets a new message has to show
//! up in the list.

use std::path::Path;
use std::time::{Duration, Instant};

use gentle_backend::session_index::SessionIndex;

use crate::probes;
use crate::report::{self, Spread};
use crate::schedule::next_delay;

/// The cadence the app ships today (`crates/ui/src/subscription.rs`).
const BASE_INTERVAL: Duration = Duration::from_secs(5);
/// Passes per refresh at most, so the polling stays under 5 % of a core.
const K: u32 = 20;
/// Warm passes timed per home (the task's second and third pass, plus one).
const WARM_PASSES: usize = 3;

pub fn run(sessions: &Path) {
    let mut index = SessionIndex::new(sessions);
    let started = Instant::now();
    let cold = index.refresh().expect("cannot read the sessions directory");
    let cold_time = started.elapsed();
    let listed = index.entries().len();

    let mut warm = Vec::with_capacity(WARM_PASSES);
    for _ in 0..WARM_PASSES {
        warm.push(timed_pass(&mut index));
    }
    let last_pass = Spread::of(warm.clone()).median;

    // A: the shipped cadence, measured end to end: append, wait 5 s, refresh.
    let target = probes::first_session(&index);
    let visible_base = append_and_wait(&mut index, &target, BASE_INTERVAL);

    // B: the same append under the bounded schedule. For a 50 000-chat home the
    // wait is tens of seconds on purpose: this is the worst case of candidate B.
    let delay = next_delay(last_pass, BASE_INTERVAL, K);
    let visible_bounded = append_and_wait(&mut index, &target, delay);

    println!(
        "home: {} ({} session files, {} listed after the cold pass)",
        sessions.display(),
        cold.files,
        listed
    );

    report::table(
        &[
            "files",
            "cold pass",
            "warm pass 1",
            "warm pass 2",
            "warm pass 3",
            "median warm",
            "CPU of one core at 5 s",
            "append -> visible at 5 s (measured)",
            "append -> visible at 5 s (bound)",
        ],
        &[vec![
            cold.files.to_string(),
            report::ms(cold_time),
            report::ms(warm[0]),
            report::ms(warm[1]),
            report::ms(warm[2]),
            report::ms(last_pass),
            report::core_fraction(last_pass, BASE_INTERVAL),
            report::secs(visible_base),
            report::secs(BASE_INTERVAL + last_pass),
        ]],
    );

    report::table(
        &[
            "files",
            "last pass",
            "K",
            "next delay",
            "CPU of one core while polling",
            "append -> visible, worst case (bound)",
            "append -> visible (measured)",
        ],
        &[vec![
            cold.files.to_string(),
            report::ms(last_pass),
            K.to_string(),
            report::secs(delay),
            report::core_fraction(last_pass, delay),
            report::secs(delay + last_pass),
            report::secs(visible_bounded),
        ]],
    );
}

/// One refresh of an index that is already in sync: the cost the app pays every
/// 5 s today.
fn timed_pass(index: &mut SessionIndex) -> Duration {
    let started = Instant::now();
    index.refresh().expect("cannot read the sessions directory");
    started.elapsed()
}

/// Appends one message to `target` right after a pass (the worst moment for a
/// poller: that file was just looked at), waits `delay`, refreshes once, and
/// returns how long the new message took to become visible in the list.
fn append_and_wait(index: &mut SessionIndex, target: &Path, delay: Duration) -> Duration {
    let before = probes::message_count(index, target);
    probes::append_message(target);
    let appended = Instant::now();
    std::thread::sleep(delay);
    index.refresh().expect("cannot read the sessions directory");
    assert!(
        probes::message_count(index, target) > before,
        "the appended message is not in the list after one refresh"
    );
    appended.elapsed()
}
