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
use std::time::{Duration, Instant};

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

    /// Stops the watcher and the safety net, waiting at most `budget` for the
    /// two of them TOGETHER, and answers whether every one of their threads
    /// ended. The close flow calls this before it flushes the session cache, so
    /// no batch asks for a pass after the last state reached the disk, and it
    /// gives this whole call one step's deadline (`Deadlines::watch` in
    /// `close::flow`): the joins share it, so the timer and the watcher can
    /// never cost the close twice what it allowed this step.
    pub fn stop(&self, budget: Duration) -> bool {
        let inner = &self.inner;
        inner.timer.stop();
        inner.watch.stop();
        // The watcher handle is what stops the platform watch and its own
        // thread, and its drop joins that thread: taken out under the flow's
        // lock, dropped outside it.
        let watcher = lock(&inner.flow).flow.take_watcher();
        drop(watcher);
        join_both(
            budget,
            Instant::now,
            |left| inner.timer.join(left),
            |left| inner.watch.join(left),
        )
    }
}

/// Joins the two parts of a stop against ONE deadline, and answers whether both
/// ended. The shape is the seam a unit test drives with fake joins and a clock
/// of its own: the budget is the budget of the JOINTS, never one per join,
/// because the close gives this step one deadline and abandons the step when
/// that deadline passes. Whatever the timer join spends is therefore gone from
/// what the batch threads may wait: a stop whose timer used the whole budget
/// hands the watcher nothing, and that join answers at once - reporting what it
/// could not join, which is the truth the close logs.
fn join_both(
    budget: Duration,
    now: impl Fn() -> Instant,
    timer: impl FnOnce(Duration) -> bool,
    watch: impl FnOnce(Duration) -> bool,
) -> bool {
    let deadline = now() + budget;
    // The timer first, because it ends at its next look and so answers the
    // quick way in a healthy run: the batch threads keep the rest of the
    // budget, and a join with nothing left of it waits for nothing.
    let timer_ended = timer(deadline.saturating_duration_since(now()));
    let watch_ended = watch(deadline.saturating_duration_since(now()));
    timer_ended && watch_ended
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

/// The watch ended on its own: forget it, let the polling fallback take over,
/// and run the pass covering what it may have missed. The take and the flow's
/// "lost" step are ONE acquisition of its lock, so no settle can install between
/// them; the taken handle is dropped below, with no lock held.
fn on_lost(inner: &Arc<Inner>) {
    let (watcher, next) = {
        let mut part = lock(&inner.flow);
        let watcher = part.flow.take_watcher();
        let next = part.flow.on_watcher_lost();
        (watcher, next)
    };
    drop(watcher);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// The budget these tests give a stop: no join waits it out, they answer in
    /// virtual time.
    const BUDGET: Duration = Duration::from_secs(5);

    /// A clock the fake joins move. Everything a stop models happens here, so a
    /// test reads the elapsed of a whole stop without sleeping for it.
    struct Clock(Cell<Instant>);

    impl Clock {
        fn new() -> Self {
            Self(Cell::new(Instant::now()))
        }

        fn now(&self) -> Instant {
            self.0.get()
        }

        /// A join that spends `spent` before it answers.
        fn spend(&self, spent: Duration) {
            self.0.set(self.0.get() + spent);
        }
    }

    /// The timer join of a stop takes the whole budget and never ends: the
    /// watch join must then be handed nothing at all, and the stop must still
    /// say that it did not join everything. The whole stop costs one budget,
    /// not two, which is the one deadline the close gives the step.
    #[test]
    fn a_timer_join_that_spends_the_budget_leaves_the_watch_none() {
        let clock = Clock::new();
        let began = clock.now();
        let asked_of_timer = Cell::new(None);
        let asked_of_watch = Cell::new(None);

        let ended = join_both(
            BUDGET,
            || clock.now(),
            |left| {
                asked_of_timer.set(Some(left));
                // A wedged timer: the close's whole budget goes here.
                clock.spend(BUDGET);
                false
            },
            |left| {
                asked_of_watch.set(Some(left));
                // Nothing to wait with: the batch threads may still be alive,
                // so a join handed no budget reports what it could not join.
                false
            },
        );

        assert_eq!(
            asked_of_timer.get(),
            Some(BUDGET),
            "the first join is handed the whole budget"
        );
        assert_eq!(
            asked_of_watch.get(),
            Some(Duration::ZERO),
            "the watch join was handed the budget the timer had already spent"
        );
        assert!(
            !ended,
            "a stop that joined one of its two parts reported every thread ended"
        );
        assert_eq!(
            clock.now() - began,
            BUDGET,
            "the stop modeled more than the one budget the close gives it"
        );
    }

    /// What the timer leaves is what the watch may wait: the two shares add up
    /// to one budget, and both can still end inside it.
    #[test]
    fn the_watch_join_waits_what_the_timer_join_left() {
        let clock = Clock::new();
        let began = clock.now();
        let spent_by_timer = BUDGET / 3;
        let asked_of_watch = Cell::new(None);

        let ended = join_both(
            BUDGET,
            || clock.now(),
            |_left| {
                clock.spend(spent_by_timer);
                true
            },
            |left| {
                asked_of_watch.set(Some(left));
                // It ends exactly as its share runs out: inside the budget.
                clock.spend(left);
                true
            },
        );

        assert_eq!(
            asked_of_watch.get(),
            Some(BUDGET - spent_by_timer),
            "the watch join was not handed the rest of the budget"
        );
        assert!(ended, "both joins ended inside the budget");
        assert_eq!(
            clock.now() - began,
            BUDGET,
            "the two shares do not add up to one budget"
        );
    }

    /// A stop whose threads end at once answers at once, `true`, and leaves the
    /// watch the whole budget: the case every close of a healthy run takes.
    #[test]
    fn a_stop_with_nothing_to_wait_for_costs_no_time_and_reports_it() {
        let clock = Clock::new();
        let began = clock.now();
        let asked_of_watch = Cell::new(None);

        let ended = join_both(
            BUDGET,
            || clock.now(),
            |_left| true,
            |left| {
                asked_of_watch.set(Some(left));
                true
            },
        );

        assert!(ended, "a stop of ended threads reported a miss");
        assert_eq!(
            asked_of_watch.get(),
            Some(BUDGET),
            "a timer that spent nothing took budget from the watch"
        );
        assert_eq!(clock.now() - began, Duration::ZERO, "the stop waited");
    }
}
