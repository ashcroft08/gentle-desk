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
use std::time::Duration;

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
        // The Settings files' watcher goes first (rpc-settings-sync T3b-1): it only
        // pushes a typed event and reads nothing, so it must not take any of the
        // step's deadline from the session threads, which may still be applying a
        // batch. Its stop is bounded like theirs, and a watcher that were wedged is
        // abandoned - the close goes on either way.
        if let Some(settings) = self.settings.as_ref() {
            if !settings.stop(budget) {
                // The flow already logged the miss; this says which watch it was.
                eprintln!("close: the settings watcher did not end within {budget:?}");
            }
        }
        let Some(sessions) = self.sessions.as_ref() else {
            return;
        };
        if !sessions.stop(budget) {
            // The flow already logged the miss; this says which threads they
            // were. The threads are abandoned, never killed: the close goes on.
            eprintln!("close: the session watcher or its timer did not end within {budget:?}");
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
