//! The wiring of the refresh beside the flow: the watcher's batches, the
//! safety-net timer, and what each of them asks the flow for.
//!
//! The ROLE is the iced shell's (`crates/ui/src/subscription.rs`,
//! `crates/ui/src/app/sessions_update.rs`, `crates/ui/src/refresh_flow.rs`),
//! never its code: `Message::SessionsChanged` is [`on_batch`],
//! `Message::WatcherLost` is [`on_lost`], `Message::Tick` is [`on_tick`], the
//! watcher a settled pass asks for is [`retry_watcher`], and iced's boot task is
//! [`SessionsService::boot`]. Every RULE stays the flow's; this module only runs
//! it off the window's threads.
//!
//! No lock here is held across a disk read, a thread start or a join: a watcher
//! is started and recorded with the flow's lock released (it creates a platform
//! watch and a thread), a watcher is dropped with the flow's lock released (its
//! `Drop` joins the platform thread), and the timer reads the delay under the
//! lock but calls the flow outside it.

use std::sync::{Arc, Weak};
use std::time::Duration;

use gentle_backend::session_watch::Batch;

use super::passes::start_pass;
use super::service::{lock, Inner, SessionsService};
use super::timer::{DelaySource, OnTick, TickDelay};
use super::watch::{OnBatch, OnLost};

impl SessionsService {
    /// Boots the refresh of a run the way iced's boot did: the watcher first, so
    /// its batches can arrive while the first pass still reads the disk, then
    /// the safety net, then the first full pass - the window opens with a list
    /// that says it is reading, never with a list that claims to be empty.
    pub fn boot(&self) {
        retry_watcher(&self.inner);
        start_timer(&self.inner);
        self.request();
    }

    /// Stops the watcher and the safety net, waiting at most `budget` for each
    /// of their threads, and answers whether every one of them ended. The close
    /// flow calls this before it flushes the session cache, so no batch asks for
    /// a pass after the last state reached the disk.
    pub fn stop(&self, budget: Duration) -> bool {
        let inner = &self.inner;
        inner.timer.stop();
        inner.watch.stop();
        // The watcher handle is what stops the platform watch and its own
        // thread, and its drop joins that thread: taken out under the flow's
        // lock, dropped outside it.
        let watcher = lock(&inner.flow).flow.take_watcher();
        drop(watcher);
        let timer = inner.timer.join(budget);
        let watch = inner.watch.join(budget);
        timer && watch
    }
}

/// Starts the watcher a settled full pass asked for (and the first one at boot).
/// Answers nothing: a watcher that started asks the flow for one more full pass,
/// which the caller's pump or the pass loop picks up.
pub(super) fn retry_watcher(inner: &Arc<Inner>) {
    let Some(wiring) = inner.wiring.as_ref() else {
        return;
    };
    // After the close nothing is started: a watch that outlived it would leave a
    // thread nobody waits for.
    if inner.watch.is_stopped() {
        return;
    }
    // One start at a time, and none while a watch runs: the flow records a
    // started watcher by replacing the one it holds, and that drop happens under
    // its own lock.
    let _starting = lock(&inner.starting);
    if lock(&inner.flow).flow.is_watching() {
        return;
    }
    let Some(dir) = lock(&inner.flow).flow.sessions_dir() else {
        return;
    };
    let (batches, lost) = sinks(inner);
    match inner.watch.start(&dir, &wiring.start, batches, lost) {
        // The close may have begun while the platform watch was being created:
        // dropping it here, with no lock held, keeps it from outliving the run.
        Ok(watcher) if inner.watch.is_stopped() => drop(watcher),
        // Recording it is what makes the flow slow the safety net down and ask
        // for the pass that covers whatever the watcher could not have seen.
        Ok(watcher) => lock(&inner.flow).flow.watcher_started(watcher),
        // No dead end: the list keeps polling and the next full pass tries
        // again. The flow logs it once per new message.
        Err(error) => lock(&inner.flow).flow.watcher_failed(error.to_string()),
    }
}

/// One batch of changed session files: the paths it names, or a full pass.
fn on_batch(inner: &Arc<Inner>, batch: Batch) {
    let next = lock(&inner.flow).flow.on_batch(batch);
    start_pass(inner, next);
}

/// The watch ended on its own: drop it outside the flow's lock, let the polling
/// fallback take over, and run the pass that covers what it may have missed.
fn on_lost(inner: &Arc<Inner>) {
    let watcher = lock(&inner.flow).flow.take_watcher();
    drop(watcher);
    let next = lock(&inner.flow).flow.on_watcher_lost();
    start_pass(inner, next);
}

/// The safety net fired: one full pass, whether a watcher runs or not.
fn on_tick(inner: &Arc<Inner>) {
    let next = lock(&inner.flow).flow.on_tick();
    start_pass(inner, next);
}

/// The two things a batch thread does, as the weak handles that keep a finished
/// run from being held alive by its own threads.
fn sinks(inner: &Arc<Inner>) -> (OnBatch, OnLost) {
    let batching = Arc::downgrade(inner);
    let batches: OnBatch = Arc::new(move |batch| {
        if let Some(inner) = batching.upgrade() {
            on_batch(&inner, batch);
        }
    });
    let losing = Arc::downgrade(inner);
    let lost: OnLost = Arc::new(move || {
        if let Some(inner) = losing.upgrade() {
            on_lost(&inner);
        }
    });
    (batches, lost)
}

/// Starts the safety net: the flow's own delay, and one pass per firing.
fn start_timer(inner: &Arc<Inner>) {
    let Some(wiring) = inner.wiring.as_ref() else {
        return;
    };
    let policy = Arc::clone(&wiring.delay);
    let delay = wiring_delay(inner, policy);
    let tick: OnTick = Arc::new({
        let ticking = Arc::downgrade(inner);
        move || {
            if let Some(inner) = ticking.upgrade() {
                on_tick(&inner);
            }
        }
    });
    inner.timer.start(delay, tick);
}

/// The flow's own delay, or nothing when the run is gone.
fn wiring_delay(inner: &Arc<Inner>, policy: TickDelay) -> DelaySource {
    let weak: Weak<Inner> = Arc::downgrade(inner);
    Arc::new(move || {
        let inner = weak.upgrade()?;
        let part = lock(&inner.flow);
        Some(policy(&part.flow))
    })
}
