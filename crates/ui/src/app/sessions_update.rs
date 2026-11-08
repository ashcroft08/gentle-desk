//! What the window does with session refresh items and the watcher's reports
//! (which pass to ask for, the partial lists of the first full pass, the final
//! list that settles the job), plus the two timer-driven lifecycle steps: the
//! idle reap and the shutdown of every child when the window closes.

use gentle_app::Input;
use gentle_backend::session_watch::Batch;
use iced::Task;

use super::App;
use crate::message::Message;
use crate::session_refresh::{Progress, Refreshed};

impl App {
    /// The safety-net timer fired: ask for a pass that reconciles whatever a
    /// watcher missed, and let the host reap the idle background children.
    pub(super) fn on_tick(&mut self) -> Task<Message> {
        Task::batch([self.flow.on_tick(), self.reap_idle()])
    }

    /// Stops the children nobody has touched for the idle window. Off the UI
    /// thread: each graceful shutdown can take its grace period, and the window
    /// must not freeze for it.
    fn reap_idle(&mut self) -> Task<Message> {
        let Some(runtime) = self.chat.clone() else {
            return Task::none();
        };
        Task::perform(async move { runtime.reap_idle() }, |_| Message::ReapDone)
    }

    /// The watcher reported session files that changed.
    pub(super) fn on_sessions_changed(&mut self, batch: Batch) -> Task<Message> {
        self.flow.on_batch(batch)
    }

    /// The watcher stopped: the flow falls back to polling for the list.
    pub(super) fn on_watcher_lost(&mut self) -> Task<Message> {
        self.flow.on_watcher_lost()
    }

    /// A partial list from the running first full pass: shown as it is, with
    /// the counters that say the list is still coming.
    pub(super) fn on_sessions_progress(&mut self, progress: Progress) -> Task<Message> {
        self.dispatch(Input::SessionsProgress {
            prepared: progress.sessions,
            done: progress.done,
            total: progress.total,
        })
    }

    /// The pass finished: its list is the whole truth, and the next queued pass
    /// may start.
    pub(super) fn on_sessions_refreshed(
        &mut self,
        result: Result<Refreshed, String>,
    ) -> Task<Message> {
        let Some(prepared) = self.flow.settle(result) else {
            // A failed pass only logs; the next request tries again.
            return self.flow.pump();
        };
        let shown = self.dispatch(Input::SessionsPrepared(prepared));
        Task::batch([shown, self.flow.pump()])
    }

    /// The window is closing: write the cache the throttle held back before the
    /// next run needs it, then stop every child before exiting (S5). The
    /// subscription holds its own reference to the runtime, so the children are
    /// shut down here rather than left to a `Drop` that may never run.
    pub(super) fn on_close_requested(&mut self) -> Task<Message> {
        self.flow.flush();
        if let Some(runtime) = self.chat.take() {
            runtime.shutdown();
        }
        iced::exit()
    }
}
