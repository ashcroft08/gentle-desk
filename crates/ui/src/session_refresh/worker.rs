//! The pass on its worker thread: the index, its cache write, and the items the
//! window reads.
//!
//! One pass is one stream. The first full pass of a run also reports the list
//! as it goes (the list the restored cache already knew, then a throttled
//! report of what has been folded), because on a big home it takes seconds and
//! the sidebar would otherwise claim there are no chats. Every pass ends with
//! exactly one [`Update::Refreshed`], the item the refresh flow settles the job
//! with.

use std::time::{Duration, Instant};

use gentle_app::{prepare_sessions, PreparedSessions};
use gentle_backend::session_index::SessionIndex;
use iced::futures::{executor::block_on, SinkExt, Stream};

use super::progress::{Progress, Throttle, Update};
use super::{Refreshed, Shared};
use crate::refresh_queue::Job;
use crate::sessions;

/// Room for several reports and the final item: the worker never drops a list
/// for lack of space, and a partial it cannot hand over waits for nobody.
const CHANNEL: usize = 8;

/// Runs one pass and yields its items: the first full pass reports as it goes,
/// and every pass ends with its final item.
pub fn stream(job: Job, shared: Shared) -> impl Stream<Item = Update> {
    iced::stream::channel(CHANNEL, async move |mut output| {
        // The pass is a `stat` per unchanged file and a full parse of the new
        // ones: it stays off the UI thread, and so does the folding.
        std::thread::spawn(move || {
            work(&job, &shared, &mut || shared.now(), &mut |item| {
                // A partial nobody has read yet is not worth waiting for: the
                // next one comes in half a second, and the index lock is held
                // meanwhile. Nobody listening means the window is gone.
                if matches!(item, Update::Progress(_)) {
                    let _ = output.try_send(item);
                } else {
                    // The final item is the list: the worker waits for room
                    // instead of dropping it. It holds no lock by now.
                    let _ = block_on(output.send(item));
                }
            });
        });
    })
}

/// Runs one pass, emitting its partial reports (first full pass only) and its
/// final item last, whatever happened. `now` is the clock the throttle reads,
/// so a test can move time instead of waiting for it.
pub(crate) fn work(
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

/// Runs one pass and returns only its final item, for callers that want the
/// list and nothing else. The shipped path is [`stream`] through
/// `refresh_flow`; this wrapper exists for tests and the module's own helpers.
#[cfg(test)]
pub(crate) async fn run(job: Job, shared: Shared) -> Result<Refreshed, String> {
    use iced::futures::StreamExt;
    let mut items = Box::pin(stream(job, shared));
    let mut last = None;
    while let Some(item) = items.next().await {
        if let Update::Refreshed(result) = item {
            last = Some(result);
        }
    }
    last.unwrap_or_else(|| Err("the session refresh thread ended unexpectedly".to_owned()))
}
