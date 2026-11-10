//! The pass threads: one pass at a time, each on a thread of its own, and the
//! slices they push.
//!
//! Split out of [`super::service`] so the state and the wiring stay readable:
//! what the flow answered with is run here, and every pass but the last one is
//! one the flow queued behind another.

use std::sync::Arc;

use gentle_app::Input;
use gentle_runtime::refresh_flow::Next;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::Update;

use super::refresh::retry_watcher;
use super::service::{lock, Inner};

/// Starts the pass the flow said to run, if it said to run one. Nothing starts
/// while another pass holds the index, which is what keeps two passes out of
/// each other's way.
pub(super) fn start_pass(inner: &Arc<Inner>, next: Next) {
    let Next::Run(job) = next else {
        return;
    };
    let inner = Arc::clone(inner);
    let spawned = std::thread::Builder::new()
        .name("session-refresh".to_owned())
        .spawn(move || run_passes(&inner, job));
    if let Err(error) = spawned {
        // The list stays as it is and the next request tries again: a log line,
        // never a panic that takes the window down.
        eprintln!("session refresh thread unavailable: {error}");
    }
}

/// One pass, then every pass the flow queued behind it, on this same thread.
/// The flow holds the index one pass at a time, so the loop only ever runs what
/// it answered with; it ends when nothing is pending.
fn run_passes(inner: &Arc<Inner>, first: Job) {
    let mut job = first;
    loop {
        let run = Arc::clone(&lock(&inner.flow).run);
        let mut settled = None;
        run(job, &mut |update| match update {
            Update::Progress(progress) => emit(
                inner,
                Input::SessionsProgress {
                    prepared: progress.sessions,
                    done: progress.done,
                    total: progress.total,
                },
            ),
            Update::Refreshed(result) => settled = Some(result),
        });
        // The flow still believes its pass runs until it is told otherwise, so
        // a pass that ended without a list is settled as the failure it is
        // rather than leaving the index occupied for good.
        let result = settled.unwrap_or_else(|| Err("the session pass ended without a list".into()));
        // Settling wakes the queue; the fold below happens outside it, because
        // folding runs the reducer, whose shared commands ask this same flow.
        let settled = lock(&inner.flow).flow.settle(result);
        // A watcher start touches the disk and spawns a thread, so it runs with
        // no flow lock held. A watcher that started asks for one more full pass
        // (events may have been missed), and the pump below runs it here.
        if settled.retry_watcher {
            retry_watcher(inner);
        }
        let next = lock(&inner.flow).flow.pump();
        if let Some(sessions) = settled.sessions {
            emit(inner, Input::SessionsPrepared(sessions));
        }
        match next {
            Next::Run(queued) => job = queued,
            Next::Idle => break,
        }
    }
}

/// Folds one input of the pass and pushes the slice it answers.
fn emit(inner: &Inner, input: Input) {
    let view = inner.holder.fold_input(input);
    if let Some(push) = lock(&inner.push).as_ref() {
        push(view);
    }
}
