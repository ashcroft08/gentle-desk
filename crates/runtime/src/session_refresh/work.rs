//! The pass itself: the index, its cache write, and the items the caller
//! receives.
//!
//! One pass runs on one thread, but the thread and the transport are the
//! caller's: [`work`] emits plain items to a callback, and the shell or window
//! wraps them in whatever it uses (an iced stream in `crates/ui`, an event
//! emitter in the Tauri app later). The first full pass of a run also reports
//! the list as it goes (the list the restored cache already knew, then a
//! throttled report of what has been folded), because on a big home it takes
//! seconds and the sidebar would otherwise claim there are no chats. Every
//! pass ends with exactly one [`Update::Refreshed`], the item the refresh flow
//! settles the job with.

use std::time::{Duration, Instant};

use gentle_app::{prepare_sessions, PreparedSessions};
use gentle_backend::session_index::SessionIndex;

use super::progress::{Progress, Throttle, Update};
use super::{Refreshed, Shared};
use crate::refresh_queue::Job;
use crate::sessions;

/// Runs one pass, emitting its partial reports (first full pass only) and its
/// final item last, whatever happened. `now` is the clock the throttle reads,
/// so a test can move time instead of waiting for it.
pub fn work(
    job: &Job,
    shared: &Shared,
    now: &mut impl FnMut() -> Duration,
    emit: &mut impl FnMut(Update),
) {
    let outcome = work_pass(job, shared, now, emit);
    emit(Update::Refreshed(outcome));
}

/// The pass itself: lock the index, report what the cache knew, fold the disk
/// into it, prepare the list, drop the lock, and persist the cache if due.
fn work_pass(
    job: &Job,
    shared: &Shared,
    now: &mut impl FnMut() -> Duration,
    emit: &mut impl FnMut(Update),
) -> Result<Refreshed, String> {
    let Ok(mut index) = shared.index.lock() else {
        return Err("the session index is poisoned".to_owned());
    };
    let full = matches!(job, Job::Full);
    // Only the run's first full pass reports: a later one is short, and a
    // loading state over an already complete list would only flicker.
    let reports = full && shared.is_first_pass();
    let mut throttle = Throttle::default();
    if reports && throttle.allow(now()) {
        // What the restored cache already knows, before the disk is read again.
        emit(Update::Progress(Progress {
            sessions: prepared(&index),
            done: 0,
            total: 0,
        }));
    }
    let progress = |index: &SessionIndex, done: usize, total: usize| {
        if reports && throttle.allow(now()) {
            emit(Update::Progress(Progress {
                sessions: prepared(index),
                done,
                total,
            }));
        }
    };
    let started = Instant::now();
    let stats = match job {
        Job::Full => index
            .refresh_with_progress(progress)
            .map_err(|error| format!("cannot read sessions: {error}"))?,
        Job::Paths(paths) => index.refresh_paths(paths),
    };
    let took = started.elapsed();
    if full {
        shared.first_pass_done();
    }
    // The folding is O(N) and pure: it happens here, not in `update`.
    let sessions = prepared(&index);
    drop(index);
    shared.save_if_due(&stats);
    Ok(Refreshed {
        sessions,
        took,
        full,
        stats,
    })
}

/// The whole list, with its search data ready for the app layer.
fn prepared(index: &SessionIndex) -> PreparedSessions {
    prepare_sessions(index.entries().iter().map(sessions::to_item).collect())
}
