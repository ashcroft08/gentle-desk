//! How fast the watcher reports the two changes that matter, and what it does
//! when many files arrive at once.
//!
//! Both probes opt real files into existence through `probes.rs`, so the bytes
//! are the same ones a polling refresh would find. "Visible in the list" is the
//! end-to-end number: event, then the shipped [`SessionIndex::refresh`] runs and
//! the appended message shows up in `entries()`.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use gentle_backend::session_index::SessionIndex;
use notify::{Event, EventKind};

use crate::probes;
use crate::report::{self, Spread};

/// Latency repetitions per probe, as the task asks.
const REPETITIONS: usize = 20;
/// Files created in one go, to see whether the backend keeps up.
const BURST: usize = 50;
/// How long to wait for the first event of one probe before giving up.
const EVENT_TIMEOUT: Duration = Duration::from_secs(30);

pub fn run(rx: &Receiver<notify::Result<Event>>, sessions: &Path) {
    let project = probes::first_project_dir(sessions);
    // Every run creates files nobody has seen before: a burst has to be
    // creations, not overwrites of the previous run's probes.
    let run = probes::now_ms();
    let mut created = Vec::with_capacity(REPETITIONS);
    for repetition in 0..REPETITIONS {
        let target = project.join(format!("probe-{run}-create-{repetition}.jsonl"));
        let started = Instant::now();
        probes::write_session_file(&target);
        match wait_for_event(rx, &target, started) {
            Some(latency) => created.push(latency),
            None => println!("no event for a new session file {target:?}"),
        }
    }

    // The append probe needs an index to see the new message in the list.
    let mut index = SessionIndex::new(sessions);
    index.refresh().expect("cannot read the sessions directory");
    let target = probes::first_session(&index);
    let mut appended = Vec::with_capacity(REPETITIONS);
    let mut visible = Vec::with_capacity(REPETITIONS);
    let mut passes = Vec::with_capacity(REPETITIONS);
    for _ in 0..REPETITIONS {
        let before = probes::message_count(&index, &target);
        let started = Instant::now();
        probes::append_message(&target);
        match wait_for_event(rx, &target, started) {
            Some(latency) => appended.push(latency),
            None => {
                println!("no event for a message appended to {target:?}");
                continue;
            }
        }
        let pass_started = Instant::now();
        index.refresh().expect("cannot read the sessions directory");
        passes.push(pass_started.elapsed());
        assert!(
            probes::message_count(&index, &target) > before,
            "the appended message is not in the list after one refresh"
        );
        visible.push(started.elapsed());
    }

    let mut rows = Vec::new();
    push_row(&mut rows, "new session file -> event", &created);
    push_row(&mut rows, "message appended -> event", &appended);
    push_row(
        &mut rows,
        "message appended -> visible in the list",
        &visible,
    );
    push_row(&mut rows, "the full refresh pass itself", &passes);
    report::table(&["probe", "min", "median", "max", "n"], &rows);

    burst(rx, &project, run);
}

/// Creates `BURST` files as fast as possible and counts what came back.
fn burst(rx: &Receiver<notify::Result<Event>>, project: &Path, run: u128) {
    let prefix = format!("probe-{run}-burst-");
    let (mut path_mentions, mut names) = (0usize, HashSet::new());
    let mut other_events = 0usize;
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    let started = Instant::now();
    for repetition in 0..BURST {
        probes::write_session_file(&project.join(format!("{prefix}{repetition}.jsonl")));
    }
    let mut last_event = Instant::now();
    while last_event.elapsed() < Duration::from_secs(2) && started.elapsed() < EVENT_TIMEOUT {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                last_event = Instant::now();
                *kinds.entry(kind_name(&event.kind)).or_default() += 1;
                for path in &event.paths {
                    match path.file_name().and_then(|name| name.to_str()) {
                        Some(name) if name.starts_with(prefix.as_str()) => {
                            path_mentions += 1;
                            names.insert(name.to_owned());
                        }
                        _ => other_events += 1,
                    }
                }
            }
            Ok(Err(_)) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    report::table(
        &[
            "files created",
            "events for them",
            "distinct files seen",
            "missed",
            "duplicated (extra events)",
            "events for other paths",
            "last event at",
        ],
        &[vec![
            BURST.to_string(),
            path_mentions.to_string(),
            names.len().to_string(),
            (BURST - names.len()).to_string(),
            (path_mentions - names.len()).to_string(),
            other_events.to_string(),
            report::secs(last_event.duration_since(started)),
        ]],
    );
    if !kinds.is_empty() {
        let counted: Vec<String> = kinds
            .iter()
            .map(|(kind, count)| format!("{kind} {count}"))
            .collect();
        println!("events by kind: {}", counted.join(", "));
    }
}

/// The event kind as one word, for the burst summary.
fn kind_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Any => "any",
        EventKind::Access(_) => "access",
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Other => "other",
    }
}

/// One latency row: samples formatted, or `n/a` when nothing was observed.
fn push_row(rows: &mut Vec<Vec<String>>, label: &str, samples: &[Duration]) {
    if samples.is_empty() {
        let missing = "n/a".to_owned();
        rows.push(vec![
            label.to_owned(),
            missing.clone(),
            missing.clone(),
            missing,
            "0".to_owned(),
        ]);
        return;
    }
    let spread = Spread::of(samples.to_vec());
    rows.push(vec![
        label.to_owned(),
        report::ms(spread.min),
        report::ms(spread.median),
        report::ms(spread.max),
        samples.len().to_string(),
    ]);
}

/// Time from `started` until an event mentions `target`, or `None` on timeout.
///
/// Paths are compared by file name: the backend may report the same file with a
/// different spelling (case, long-path prefix), and the probe names are unique.
fn wait_for_event(
    rx: &Receiver<notify::Result<Event>>,
    target: &Path,
    started: Instant,
) -> Option<Duration> {
    let name = target.file_name()?.to_str()?.to_owned();
    let deadline = started + EVENT_TIMEOUT;
    loop {
        let left = deadline.checked_duration_since(Instant::now())?;
        match rx.recv_timeout(left) {
            Ok(Ok(event)) => {
                let hit = event
                    .paths
                    .iter()
                    .any(|path| path.file_name().and_then(|file| file.to_str()) == Some(&name));
                if hit {
                    return Some(started.elapsed());
                }
            }
            Ok(Err(error)) => println!("watcher error: {error}"),
            Err(_) => return None,
        }
    }
}
