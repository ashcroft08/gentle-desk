//! The refresh passes of this run, and the slices they push (S2, S3).
//!
//! The ROLE is the iced shell's (`crates/ui/src/refresh_flow.rs` on top of
//! `crates/runtime/src/refresh_flow.rs`, `app/sessions_update.rs`), never its
//! code: the decisions - one pass at a time, a request that arrives mid-pass
//! queued and run after it settles, the flow's own watcher retry - are
//! [`gentle_runtime::refresh_flow::Flow`]'s. This module only binds them to a
//! thread and to the emit step.
//!
//! A pass is a `stat` per unchanged session file and a full parse of the new
//! ones, plus the O(N) folding of the list: it blocks, so it runs on a
//! dedicated thread and never on a Tauri executor thread. Each partial report
//! of the first full pass is folded as `Input::SessionsProgress` and pushed, so
//! the sidebar can draw a list that is still coming; the final list is folded
//! as `Input::SessionsPrepared` and settles the status to complete.
//!
//! The seams are [`RunPass`] ("run one pass, reporting as you go") and the
//! [`Push`] step ("emit this slice"): a test feeds a real temp home and records
//! the slices, so no window and no home of the user are involved.

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::Input;
use gentle_runtime::refresh_flow::{Flow, Next};
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{Shared, Update};

use crate::bridge::AppStateHolder;
use crate::events::Push;

/// Runs one pass to its end, reporting each item through `emit`. Blocking by
/// design: whatever calls it is a thread of its own.
pub type RunPass = Arc<dyn Fn(Job, &mut dyn FnMut(Update)) + Send + Sync>;

/// The flow of the run and the seam that runs its passes.
struct FlowPart {
    flow: Flow,
    run: RunPass,
}

/// What a pass thread needs: the flow it settles and the step it pushes with.
/// Shared behind an [`Arc`] so a thread can outlive the call that started it.
struct Inner {
    flow: Mutex<FlowPart>,
    push: Mutex<Option<Push>>,
}

/// The session refresh of this run: the flow, and the threads it asks for.
pub struct SessionsService {
    holder: Arc<AppStateHolder>,
    inner: Arc<Inner>,
}

impl SessionsService {
    /// A run with the flow `startup` built and the pass seam of the run.
    pub fn new(holder: Arc<AppStateHolder>, flow: Flow, run: RunPass) -> Self {
        Self {
            holder,
            inner: Arc::new(Inner {
                flow: Mutex::new(FlowPart { flow, run }),
                push: Mutex::new(None),
            }),
        }
    }

    /// The emit step, installed once at startup: `state-changed` in the app, a
    /// recorder in a test. A pass that ends before it arrives only folds its
    /// slice into the holder, which the next push carries anyway.
    pub fn arm(&self, push: Push) {
        *lock(&self.inner.push) = Some(push);
    }

    /// The index cache of this run, cloned out from under the flow's lock: `None`
    /// when the pi home is unknown. The close flow flushes it with
    /// `session_refresh::flush`, so the write itself happens with no lock of this
    /// service held.
    pub fn shared(&self) -> Option<Shared> {
        lock(&self.inner.flow).flow.shared()
    }

    /// `Command::RefreshSessions` from the reducer (a settled turn, boot, an
    /// explicit refresh): the flow decides whether this is a pass to run now or
    /// one to queue behind the pass that already has the index. Starting the
    /// thread here is the shell's half of that decision.
    pub fn request(&self) {
        let next = lock(&self.inner.flow).flow.on_refresh_command();
        self.start(next);
    }

    /// Starts the pass the flow said to run, if it said to run one. Nothing
    /// starts while another pass holds the index, which is what keeps two
    /// passes out of each other's way.
    fn start(&self, next: Next) {
        let Next::Run(job) = next else {
            return;
        };
        let inner = Arc::clone(&self.inner);
        let holder = Arc::clone(&self.holder);
        let spawned = std::thread::Builder::new()
            .name("session-refresh".to_owned())
            .spawn(move || run_passes(&inner, &holder, job));
        if let Err(error) = spawned {
            // The list stays as it is and the next request tries again: a log
            // line, never a panic that takes the window down.
            eprintln!("session refresh thread unavailable: {error}");
        }
    }
}

/// One pass, then every pass the flow queued behind it, on this same thread.
/// The flow holds the index one pass at a time, so the loop only ever runs what
/// it answered with; it ends when nothing is pending.
fn run_passes(inner: &Inner, holder: &AppStateHolder, first: Job) {
    let mut job = first;
    loop {
        let run = Arc::clone(&lock(&inner.flow).run);
        let mut settled = None;
        run(job, &mut |update| match update {
            Update::Progress(progress) => emit(
                inner,
                holder,
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
        // Settle and pump under one lock, so no other request can squeeze a
        // second pass in between; the fold below happens outside it, because
        // folding runs the reducer, whose shared commands ask this same flow.
        let (sessions, next) = {
            let mut part = lock(&inner.flow);
            let settled = part.flow.settle(result);
            if settled.retry_watcher {
                // TODO(watcher): retry gentle_runtime::startup::try_start_watcher
                // and report back with `Flow::watcher_started` /
                // `Flow::watcher_failed`. The watcher is out of this slice.
            }
            (settled.sessions, part.flow.pump())
        };
        if let Some(sessions) = sessions {
            emit(inner, holder, Input::SessionsPrepared(sessions));
        }
        match next {
            Next::Run(queued) => job = queued,
            Next::Idle => break,
        }
    }
}

/// Folds one input of the pass and pushes the slice it answers.
fn emit(inner: &Inner, holder: &AppStateHolder, input: Input) {
    let view = holder.fold_input(input);
    if let Some(push) = lock(&inner.push).as_ref() {
        push(view);
    }
}

/// A lock that survives a poisoned mutex: a thread that died must not leave the
/// list unreadable for the rest of the run.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
