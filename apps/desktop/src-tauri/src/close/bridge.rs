//! The real steps of a close: the bridge's own pieces, in the flow's order.
//!
//! Nothing here owns a policy: the order and the deadlines are
//! [`super::flow`]'s. This is where each step names the bridge part that does
//! the work: the executor (stop new work, stop every child), the event pump (end
//! its threads), the session cache and the config writer. It is also where the
//! process finally ends; the exit runs through the closure the app arms at
//! startup, so this module names no Tauri type and a test runs it with a flag
//! instead.

use std::sync::Arc;
use std::time::Duration;

use gentle_runtime::session_refresh::{self, Shared};

use super::flow::CloseSteps;
use crate::config::ConfigWriter;
use crate::effects::RuntimeExecutor;
use crate::events::EventPump;

/// The close of a running bridge.
pub struct BridgeSteps {
    executor: Arc<RuntimeExecutor>,
    pump: Arc<EventPump>,
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
            shared,
            writer,
            exit,
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

    fn flush_cache(&self) {
        let Some(shared) = self.shared.as_ref() else {
            return;
        };
        // The same write the iced shell did on close, and the only one that is
        // allowed past the throttle: the last state reaches the disk even when
        // the window closes before the next pass.
        session_refresh::flush(shared);
    }

    fn flush_config(&self, budget: Duration) {
        self.writer.flush_and_stop(budget);
    }

    fn exit(&self) {
        (self.exit)();
    }
}
