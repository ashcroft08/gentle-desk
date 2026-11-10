//! The refresh of this run: the flow, the passes it asks for and the slices
//! they push (S2, S3).
//!
//! The ROLE is the iced shell's (`crates/ui/src/refresh_flow.rs` on top of
//! `crates/runtime/src/refresh_flow.rs`, `app/sessions_update.rs`,
//! `subscription.rs`), never its code: the decisions - one pass at a time, a
//! request that arrives mid-pass queued and run after it settles, the flow's own
//! watcher retry, the adaptive safety-net delay - are
//! [`gentle_runtime::refresh_flow::Flow`]'s. This module only binds them to the
//! threads and to the emit step.
//!
//! A pass is a `stat` per unchanged session file and a full parse of the new
//! ones, plus the O(N) folding of the list: it blocks, so it runs on a
//! dedicated thread and never on a Tauri executor thread. Each partial report
//! of the first full pass is folded as `Input::SessionsProgress` and pushed, so
//! the sidebar can draw a list that is still coming; the final list is folded
//! as `Input::SessionsPrepared` and settles the status to complete.
//!
//! Three more threads can ask the flow for a pass, and none of them is the
//! window's: the watcher's batch thread ([`super::watch`]), the safety net
//! ([`super::timer`]) and the pass thread itself, when a settle asks for the
//! watcher back. Every one of them folds through the holder and pushes the same
//! `state-changed`, so the sidebar updates without being asked.
//!
//! The seams are [`RunPass`] ("run one pass, reporting as you go"), [`Push`]
//! ("emit this slice") and the [`Wiring`] of the run (where a watcher is started
//! from, and how long the safety net waits): a test feeds a real temp home and
//! records the slices, so no window and no home of the user are involved.

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_runtime::refresh_flow::Flow;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{Shared, Update};

use crate::bridge::AppStateHolder;
use crate::events::Push;

use super::timer::TickTimer;
use super::watch::Watch;
use super::wiring::Wiring;

/// Runs one pass to its end, reporting each item through `emit`. Blocking by
/// design: whatever calls it is a thread of its own.
pub type RunPass = Arc<dyn Fn(Job, &mut dyn FnMut(Update)) + Send + Sync>;

/// The flow of the run and the seam that runs its passes.
pub(super) struct FlowPart {
    pub(super) flow: Flow,
    pub(super) run: RunPass,
}

/// What the pass thread, the batch thread and the timer share. Behind an
/// [`Arc`] so a thread can outlive the call that started it.
pub(super) struct Inner {
    pub(super) holder: Arc<AppStateHolder>,
    pub(super) flow: Mutex<FlowPart>,
    pub(super) push: Mutex<Option<Push>>,
    /// The batch threads of the running watch.
    pub(super) watch: Watch,
    /// The safety net.
    pub(super) timer: TickTimer,
    /// One watcher start at a time, held across the start itself. Two starts
    /// would hand the flow a second handle, and recording that one replaces the
    /// first, which `Drop` joins under the flow's lock: never.
    pub(super) starting: Mutex<()>,
    /// `None` in a run driven by hand (the tests of the passes alone): no
    /// watcher is started or retried, and no safety net runs.
    pub(super) wiring: Option<Wiring>,
}

/// The session refresh of this run: the flow, and the threads it asks for.
pub struct SessionsService {
    pub(super) inner: Arc<Inner>,
}

impl SessionsService {
    /// A run with the flow `startup` built and the pass seam of the run, driven
    /// by hand: no watcher and no safety net.
    pub fn new(holder: Arc<AppStateHolder>, flow: Flow, run: RunPass) -> Self {
        Self::with_parts(holder, flow, run, None)
    }

    /// The same with the wiring of a desktop run: the watcher a full pass
    /// starts, and the safety-net delay. Nothing starts until
    /// [`SessionsService::boot`].
    pub fn wired(holder: Arc<AppStateHolder>, flow: Flow, run: RunPass, wiring: Wiring) -> Self {
        Self::with_parts(holder, flow, run, Some(wiring))
    }

    fn with_parts(
        holder: Arc<AppStateHolder>,
        flow: Flow,
        run: RunPass,
        wiring: Option<Wiring>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                holder,
                flow: Mutex::new(FlowPart { flow, run }),
                push: Mutex::new(None),
                watch: Watch::new(),
                timer: TickTimer::new(),
                starting: Mutex::new(()),
                wiring,
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

    /// `Command::RefreshSessions` from the reducer (a settled turn, an explicit
    /// refresh): the flow decides whether this is a pass to run now or one to
    /// queue behind the pass that already has the index. Starting the thread
    /// here is the shell's half of that decision.
    pub fn request(&self) {
        let next = lock(&self.inner.flow).flow.on_refresh_command();
        super::passes::start_pass(&self.inner, next);
    }
}

/// A lock that survives a poisoned mutex: a thread that died must not leave the
/// list unreadable for the rest of the run.
pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
