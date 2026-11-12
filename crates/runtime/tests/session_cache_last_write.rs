//! The close's flush is the last write of the session cache (S2).
//!
//! A pass writes the cache from its own thread, and the close flushes it from
//! the close's thread: the flush must own the final state, so a pass whose
//! final write reaches the write path after the close flushed leaves the file
//! exactly as the close wrote it.
//!
//! The clock of a pass is the one [`session_refresh::work`] takes, so the test
//! moves time instead of waiting for it: nothing here sleeps to synchronise, and
//! no thread of the user's home is involved (a temp home and a temp cache).
//!
//! The order of the steps below is the order the two writers can be in when the
//! close wins the throttle's lock: the pass that reads a session the close's
//! snapshot does not have reaches its write afterwards, and its own write is due
//! (the clock is past the throttle's interval), so only the seal the close takes
//! can keep it off the disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::session_index::{RefreshStats, SessionIndex};
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::save_throttle::MIN_INTERVAL;
use gentle_runtime::session_refresh::{self, Refreshed, Shared, Update};

/// A clock past the throttle's interval: a pass that runs at it has a write the
/// throttle allows. Far enough past the interval that a slow machine only makes
/// it more due, never less.
const PAST_THE_INTERVAL: Duration = Duration::from_secs(3600);

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;

fn message(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

/// One session file with a header and one user message.
fn write_session(sessions: &Path, name: &str, first: &str) {
    let project = sessions.join("p");
    fs::create_dir_all(&project).expect("the project directory");
    fs::write(
        project.join(name),
        format!("{HEADER}\n{}\n", message(first)),
    )
    .expect("the session file");
}

/// Whether a pass found anything to persist, by the same rule the cache write
/// uses.
fn changed(stats: &RefreshStats) -> bool {
    stats.rescanned + stats.appended + stats.removed > 0
}

/// One pass, run to its end on this thread at `now`: a pass reports
/// synchronously, so no wait is needed.
fn pass(job: Job, shared: &Shared, now: Duration) -> Refreshed {
    let mut settled = None;
    session_refresh::work(&job, shared, &mut || now, &mut |update| {
        if let Update::Refreshed(result) = update {
            settled = Some(result);
        }
    });
    settled
        .expect("a pass settles with a list")
        .expect("the pass read the home")
}

/// A temp home with one session, and the run's shared index over it.
fn home() -> (tempfile::TempDir, Arc<Mutex<SessionIndex>>, PathBuf) {
    let dir = tempfile::tempdir().expect("a temp home");
    let sessions = dir.path().join("sessions");
    write_session(&sessions, "a.jsonl", "uno");
    let index = Arc::new(Mutex::new(SessionIndex::new(sessions.clone())));
    (dir, index, sessions)
}

fn cache_of(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("data").join("session-index.json")
}

/// The RED of S2: a pass whose final write lands after the close's flush does
/// not touch the file, so the cache the next run reads is the close's state.
#[test]
fn a_pass_that_writes_after_the_close_leaves_the_flushed_cache_alone() {
    let (dir, index, sessions) = home();
    let cache = cache_of(&dir);
    let shared = Shared::new(index, Some(cache.clone()));

    // The first change has nothing written before it, so the pass writes.
    let first = pass(Job::Full, &shared, Duration::ZERO);
    assert!(changed(&first.stats), "the first pass found the session");
    let written = fs::read_to_string(&cache).expect("the first pass wrote the cache");

    // A second change inside the throttle's interval is held back, as it is
    // while the window runs: the close is the one that writes it.
    write_session(&sessions, "b.jsonl", "dos");
    let held = pass(Job::Full, &shared, Duration::ZERO);
    assert!(
        changed(&held.stats),
        "the second pass found the new session"
    );
    assert_eq!(
        fs::read_to_string(&cache).expect("the cache the first pass wrote"),
        written,
        "a change within the interval rewrote the cache before the close"
    );

    // The close: the flush the close flow runs, and the last write of the file.
    session_refresh::flush(&shared);
    let flushed = fs::read_to_string(&cache).expect("the close wrote the pending state");
    assert_ne!(flushed, written, "the close wrote nothing");

    // The in-flight pass: it reads a session the close's snapshot does not
    // have, and it reaches its write after the flush. Its write is due at this
    // clock, so a write path that only asks the throttle would make it.
    write_session(&sessions, "c.jsonl", "tres");
    let late = pass(Job::Full, &shared, PAST_THE_INTERVAL);
    assert!(
        changed(&late.stats),
        "the late pass found nothing: this test would prove nothing"
    );
    assert_eq!(
        fs::read_to_string(&cache).expect("the flushed cache"),
        flushed,
        "a pass wrote the cache after the close's flush"
    );
}

/// The seal is the close's alone: a run that never flushed keeps writing on the
/// throttle's rule, however far past the interval a pass runs.
#[test]
fn a_pass_past_the_interval_still_writes_when_no_close_flushed() {
    let (dir, index, sessions) = home();
    let cache = cache_of(&dir);
    let shared = Shared::new(index, Some(cache.clone()));

    let first = pass(Job::Full, &shared, Duration::ZERO);
    assert!(changed(&first.stats), "the first pass found the session");
    let written = fs::read_to_string(&cache).expect("the first pass wrote the cache");

    // Held back inside the interval, written once it has passed: the rule the
    // close must not weaken and the seal must not replace.
    write_session(&sessions, "b.jsonl", "dos");
    let held = pass(Job::Full, &shared, Duration::ZERO);
    assert!(
        changed(&held.stats),
        "the second pass found the new session"
    );
    assert_eq!(
        fs::read_to_string(&cache).expect("the cache the first pass wrote"),
        written,
        "a change within the interval rewrote the cache"
    );

    // Another change once the interval has passed: due, so it writes.
    write_session(&sessions, "c.jsonl", "tres");
    let due = pass(Job::Full, &shared, MIN_INTERVAL + Duration::from_secs(1));
    assert!(changed(&due.stats), "the third pass found the new session");
    assert_ne!(
        fs::read_to_string(&cache).expect("the cache the third pass wrote"),
        written,
        "a pass past the interval did not write the cache"
    );
}
