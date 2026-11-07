//! Candidate C: a file-system watcher (`notify`) instead of polling.
//!
//! Measured here: how long the recursive watch takes to set up and what it
//! costs while nothing happens (CPU and working set of this process). The
//! latency of the events and whether a burst loses or repeats them is in
//! `watcher/latency.rs`; the list itself still needs bytes read, so "visible"
//! there is event plus one pass of the shipped [`SessionIndex::refresh`].

use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};

use crate::proc;
use crate::report;

// `rustc` would look in `watcher/`, `rustfmt` next to this file: say it once.
#[path = "latency.rs"]
mod latency;

pub fn run(sessions: &Path, watch_seconds: u64) {
    println!(
        "watcher home: {} (recursive watch, {} s idle window)",
        sessions.display(),
        watch_seconds
    );
    let (tx, rx) = mpsc::channel();
    let setup_started = Instant::now();
    let mut watcher = match notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    }) {
        Ok(watcher) => watcher,
        Err(error) => {
            println!("cannot create a watcher: {error}");
            return;
        }
    };
    if let Err(error) = watcher.watch(sessions, RecursiveMode::Recursive) {
        println!("cannot watch the sessions directory: {error}");
        return;
    }
    let setup = setup_started.elapsed();

    // Nothing happens here on purpose: this is the cost of just watching.
    let cpu_before = proc::cpu_time();
    let rss_before = proc::working_set_bytes();
    let idle_started = Instant::now();
    let window = Duration::from_secs(watch_seconds);
    let mut idle_events = 0usize;
    let mut idle_errors = 0usize;
    while let Some(left) = window.checked_sub(idle_started.elapsed()) {
        match rx.recv_timeout(left) {
            Ok(Ok(_)) => idle_events += 1,
            Ok(Err(_)) => idle_errors += 1,
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => break,
        }
    }
    let idle_wall = idle_started.elapsed();
    let idle_cpu = match (cpu_before, proc::cpu_time()) {
        (Some(before), Some(after)) => Some(after.saturating_sub(before)),
        _ => None,
    };
    let rss_after = proc::working_set_bytes();

    report::table(
        &[
            "setup (create + watch)",
            "idle window",
            "events while idle",
            "event errors",
            "idle CPU",
            "idle CPU of one core",
            "working set before",
            "working set after",
        ],
        &[vec![
            report::ms(setup),
            report::secs(idle_wall),
            idle_events.to_string(),
            idle_errors.to_string(),
            idle_cpu.map_or_else(|| "n/a".to_owned(), report::ms),
            idle_cpu.map_or_else(
                || "n/a".to_owned(),
                |cpu| report::core_fraction(cpu, idle_wall),
            ),
            rss_before.map_or_else(|| "n/a".to_owned(), report::mib),
            rss_after.map_or_else(|| "n/a".to_owned(), report::mib),
        ]],
    );

    latency::run(&rx, sessions);
    drop(watcher);
}
