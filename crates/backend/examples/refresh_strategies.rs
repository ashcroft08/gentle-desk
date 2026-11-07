//! Measures the two candidate answers to backlog item 10 — a refresh strategy
//! that does not scale with the number of session files — on a real sessions
//! directory. Nothing here is wired into the app: `SessionIndex::refresh` and
//! the UI's 5 s timer are untouched, so this only measures.
//!
//! ```text
//! cargo run --release -p gentle-backend --example refresh_strategies -- <sessions dir> [watch-seconds]
//! ```
//!
//! A: the shipped 5 s polling, timed on a warm index (the baseline).
//! B: cost-bounded polling, `max(5 s, K x last pass)` with K = 20, from the
//!    measured pass times (`schedule.rs`, unit-tested).
//! C: a file-system watcher (`notify`): setup, idle cost, event latency, and
//!    whether a burst of creations loses or repeats events (`watcher.rs`).
//!
//! `<sessions dir>` is `<home>/sessions`, as under `tests/` in the synthetic
//! homes built with `tools/perf/gen-home.mjs`.

use std::path::PathBuf;

#[path = "refresh_strategies/polling.rs"]
mod polling;
#[path = "refresh_strategies/probes.rs"]
mod probes;
#[path = "refresh_strategies/proc.rs"]
mod proc;
#[path = "refresh_strategies/report.rs"]
mod report;
#[path = "refresh_strategies/schedule.rs"]
mod schedule;
#[path = "refresh_strategies/watcher.rs"]
mod watcher;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: refresh_strategies <sessions dir> [watch-seconds]");
        std::process::exit(2);
    };
    let watch_seconds = match args.next() {
        Some(value) => value.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("watch-seconds must be a whole number of seconds: {value}");
            std::process::exit(2);
        }),
        None => 20,
    };
    let sessions = PathBuf::from(dir);
    if !sessions.is_dir() {
        eprintln!("not a directory: {}", sessions.display());
        std::process::exit(2);
    }

    println!(
        "refresh strategies on {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!(
        "sessions dir: {}",
        sessions
            .canonicalize()
            .unwrap_or_else(|_| sessions.clone())
            .display()
    );
    println!();
    println!("A) the shipped 5 s polling, and B) bounded polling");
    polling::run(&sessions);
    println!();
    println!("C) a file-system watcher");
    watcher::run(&sessions, watch_seconds);
    // A sanity check on the CPU reading the idle window relies on: after all
    // those passes the process must show real CPU time, not zero.
    println!();
    println!(
        "this process so far: {} of CPU, {} working set",
        proc::cpu_time().map_or_else(|| "n/a".to_owned(), report::ms),
        proc::working_set_bytes().map_or_else(|| "n/a".to_owned(), report::mib)
    );
}
