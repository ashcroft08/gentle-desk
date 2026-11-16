//! The real steps of a close: the bridge's own pieces, in the flow's order.
//!
//! Nothing here owns a policy: the order and the deadlines are
//! [`super::flow`]'s. This is where each step names the bridge part that does
//! the work: the executor (stop new work, stop every child), the event pump (end
//! its threads), the session service (stop the watcher and the safety net), the
//! Settings files' watcher of the run, the session cache and the config writer.
//! It is also where the process finally ends; the exit runs through the closure
//! the app arms at startup, so this module names no Tauri type and a test runs it
//! with a flag instead.

use std::sync::Arc;
use std::time::{Duration, Instant};

use gentle_runtime::session_refresh::{self, Shared};

use super::flow::CloseSteps;
use crate::config::ConfigWriter;
use crate::effects::RuntimeExecutor;
use crate::events::EventPump;
use crate::sessions::SessionsService;
use crate::settings::SettingsWatch;

/// The close of a running bridge.
pub struct BridgeSteps {
    executor: Arc<RuntimeExecutor>,
    pump: Arc<EventPump>,
    /// The session refresh of this run, when the close may stop its threads.
    /// `None` in a test that only exercises the other steps: a run with no
    /// session service has no watcher and no timer to stop.
    sessions: Option<Arc<SessionsService>>,
    /// The Settings files' watcher of this run (rpc-settings-sync T3b-1), when the
    /// close may stop it. `None` in a test that names no watcher: then the step
    /// has no watch of that kind to end.
    settings: Option<Arc<SettingsWatch>>,
    /// The index cache of this run: `None` when the pi home is unknown, and then
    /// there is nothing to flush.
    shared: Option<Shared>,
    writer: ConfigWriter,
    /// What ends the process: `AppHandle::exit` in the app, a flag in a test.
    exit: Box<dyn Fn() + Send + Sync>,
}

impl BridgeSteps {
    pub fn new(
        executor: Arc<RuntimeExecutor>,
        pump: Arc<EventPump>,
        shared: Option<Shared>,
        writer: ConfigWriter,
        exit: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        Self {
            executor,
            pump,
            sessions: None,
            settings: None,
            shared,
            writer,
            exit,
        }
    }

    /// The same close with the Settings files' watcher of the run, so it is ended
    /// in the same step as the session watcher and therefore before the cache and
    /// the config are written. The app always sets it; a test that names no
    /// watcher leaves the step with only the session threads to stop.
    pub fn with_settings_watch(self, watch: Arc<SettingsWatch>) -> Self {
        Self {
            settings: Some(watch),
            ..self
        }
    }

    /// The same close with the session refresh of the run, so its watcher and
    /// its safety net stop before the cache is written. The app always sets it;
    /// a test that names no session service leaves the step with nothing to do.
    pub fn with_sessions(self, sessions: Arc<SessionsService>) -> Self {
        Self {
            sessions: Some(sessions),
            ..self
        }
    }
}

impl CloseSteps for BridgeSteps {
    fn stop_work(&self) {
        self.executor.stop_work();
    }

    fn stop_children(&self) {
        self.executor.shutdown();
    }

    fn join_pump(&self, budget: Duration) {
        // The stop signal first: the host keeps its event channel open for its
        // whole life, so the pump ends because it is told to, not on its own.
        self.pump.stop();
        if !self.pump.join(budget) {
            // The flow already logged the miss; this says which thread it was.
            eprintln!("close: the child event threads did not end within {budget:?}");
        }
    }

    fn stop_watch(&self, budget: Duration) {
        // ONE deadline for the whole step, computed once here (rpc-settings-sync
        // T3b-1's review): the trait says this step waits at most `budget`, so
        // the two watchers SHARE it instead of each taking a fresh one. The
        // Settings files' watcher goes first because it only pushes a typed
        // event and reads nothing; the session threads, which may still be
        // applying a batch, then get what is LEFT of the same deadline. A first
        // stop that spends it all hands the second nothing, and the step still
        // costs one budget, not two - a wedged watcher of either kind is
        // abandoned and the close goes on.
        let settings = self.settings.as_ref();
        let sessions = self.sessions.as_ref();
        let (settings_ended, sessions_ended) = share_deadline(
            budget,
            Instant::now,
            |left| settings.is_none_or(|settings| settings.stop(left)),
            |left| sessions.is_none_or(|sessions| sessions.stop(left)),
        );
        if !settings_ended {
            // The flow already logged the miss; this says which watch it was.
            eprintln!("close: the settings watcher did not end within the step's {budget:?}");
        }
        if !sessions_ended {
            // The flow already logged the miss; this says which threads they
            // were. The threads are abandoned, never killed: the close goes on.
            eprintln!(
                "close: the session watcher or its timer did not end within the step's {budget:?}"
            );
        }
    }

    fn flush_cache(&self) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        // The same write the iced shell did on close, and the only one that is
        // allowed past the throttle: the last state reaches the disk even when
        // the window closes before the next pass. It also seals the cache, so a
        // pass still in flight writes nothing over it.
        session_refresh::flush(shared);
    }

    fn flush_config(&self, budget: Duration) {
        self.writer.flush_and_stop(budget);
    }

    fn exit(&self) {
        (self.exit)();
    }
}

/// The two waits of one close step, against ONE deadline: `first` is handed the
/// whole `budget` and `second` only what `first` left of it, measured on the
/// same clock. Neither wait is ever handed a negative budget: a first wait that
/// spends the whole budget leaves the second `Duration::ZERO`, whose stop
/// answers at once and reports what it could not join - the truth the close
/// logs.
///
/// The clock is a parameter so a unit test reads the elapsed of a whole step
/// without sleeping for it, the shape `sessions::refresh`'s `join_both` uses; the
/// app passes [`Instant::now`].
fn share_deadline(
    budget: Duration,
    now: impl Fn() -> Instant,
    first: impl FnOnce(Duration) -> bool,
    second: impl FnOnce(Duration) -> bool,
) -> (bool, bool) {
    let deadline = now() + budget;
    let first_ended = first(deadline.saturating_duration_since(now()));
    let second_ended = second(deadline.saturating_duration_since(now()));
    (first_ended, second_ended)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// The budget these tests give the step: no wait sleeps it out, they all
    /// answer in virtual time.
    const BUDGET: Duration = Duration::from_secs(5);

    /// A clock the fake stops move, so a test reads the elapsed of the whole
    /// step without waiting for it.
    struct Clock(Cell<Instant>);

    impl Clock {
        fn new() -> Self {
            Self(Cell::new(Instant::now()))
        }

        fn now(&self) -> Instant {
            self.0.get()
        }

        /// A stop that spends `spent` before it answers.
        fn spend(&self, spent: Duration) {
            self.0.set(self.0.get() + spent);
        }
    }

    /// A Settings stop that is wedged for the whole budget leaves the session
    /// stop NOTHING: the step costs one budget, never two, so a wedged Settings
    /// watcher cannot delay the session threads and the rest of the close by a
    /// second full budget.
    #[test]
    fn a_wedged_settings_stop_leaves_the_session_stop_no_budget() {
        let clock = Clock::new();
        let began = clock.now();
        let asked_of_settings = Cell::new(None);
        let asked_of_sessions = Cell::new(None);

        let (settings_ended, sessions_ended) = share_deadline(
            BUDGET,
            || clock.now(),
            |left| {
                asked_of_settings.set(Some(left));
                // A wedged Settings watcher: the step's whole budget goes here.
                clock.spend(BUDGET);
                false
            },
            |left| {
                asked_of_sessions.set(Some(left));
                // Nothing left to wait with: the session threads may still be
                // alive, and the stop says what it could not join.
                clock.spend(left);
                false
            },
        );

        assert_eq!(
            asked_of_settings.get(),
            Some(BUDGET),
            "the first stop is handed the whole budget"
        );
        assert_eq!(
            asked_of_sessions.get(),
            Some(Duration::ZERO),
            "the session stop was handed a budget the Settings stop had already spent"
        );
        assert!(!settings_ended, "a wedged stop reported that it ended");
        assert!(
            !sessions_ended,
            "a stop handed nothing reported that it ended"
        );
        assert_eq!(
            clock.now() - began,
            BUDGET,
            "the step waited more than the one budget the close gives it"
        );
    }

    /// What the Settings stop leaves is what the session stop may wait: the two
    /// shares add up to one budget, and both can still end inside it.
    #[test]
    fn the_session_stop_waits_what_the_settings_stop_left() {
        let clock = Clock::new();
        let began = clock.now();
        let spent_by_settings = BUDGET / 3;
        let asked_of_sessions = Cell::new(None);

        let (settings_ended, sessions_ended) = share_deadline(
            BUDGET,
            || clock.now(),
            |_left| {
                clock.spend(spent_by_settings);
                true
            },
            |left| {
                asked_of_sessions.set(Some(left));
                // It ends exactly as its share runs out: inside the budget.
                clock.spend(left);
                true
            },
        );

        assert_eq!(
            asked_of_sessions.get(),
            Some(BUDGET - spent_by_settings),
            "the session stop was not handed the rest of the budget"
        );
        assert!(
            settings_ended && sessions_ended,
            "both stops ended inside the budget"
        );
        assert_eq!(
            clock.now() - began,
            BUDGET,
            "the two shares do not add up to one budget"
        );
    }

    /// The healthy step: the Settings stop ends at once, so the session threads
    /// keep the whole budget, which is the behaviour the step had before it
    /// shared the deadline.
    #[test]
    fn a_settings_stop_that_ends_at_once_leaves_the_session_stop_the_budget() {
        let clock = Clock::new();
        let began = clock.now();
        let asked_of_sessions = Cell::new(None);

        let (settings_ended, sessions_ended) = share_deadline(
            BUDGET,
            || clock.now(),
            |_left| true,
            |left| {
                asked_of_sessions.set(Some(left));
                true
            },
        );

        assert!(
            settings_ended && sessions_ended,
            "a step of ended stops reported a miss"
        );
        assert_eq!(
            asked_of_sessions.get(),
            Some(BUDGET),
            "a Settings stop that spent nothing took budget from the session threads"
        );
        assert_eq!(clock.now() - began, Duration::ZERO, "the step waited");
    }
}
