//! The refresh flow as the window runs it: the iced transport over the core
//! moved to `gentle-runtime`.
//!
//! Which pass runs when, when a pass settles and when the watcher is retried is
//! decided by [`gentle_runtime::refresh_flow`]: it answers with a [`Next`], and
//! a full pass that should retry the watcher says so. This module keeps only
//! what the window binds: a [`Next::Run`] becomes the pass's `Task::stream`,
//! [`Next::Idle`] is `Task::none`, and the two environment calls it makes for
//! real (start a watcher, write the cache on close) feed the [`WatchSource`] the
//! subscription reads. Nothing here decides anything.

use std::path::PathBuf;
use std::time::Duration;

use gentle_app::PreparedSessions;
use gentle_backend::session_watch::{Batch, SessionWatcher};
use iced::futures::StreamExt;
use iced::Task;

use crate::backend;
use crate::message::Message;
use crate::session_refresh::{self, Refreshed, Shared, Update};
use crate::watch_source::WatchSource;
use gentle_runtime::refresh_flow::{self as core, Next};

/// The refresh state of a running window: the runtime's decisions plus the two
/// iced bindings they need (the pass stream and the watcher's batches).
pub struct Flow {
    core: core::Flow,
    /// Where a newly started watcher's batches go.
    watch: WatchSource,
}

impl Flow {
    /// A run with its index (if the home is known) and the watcher `backend`
    /// managed to start (if any). `watch_error` is the failure `backend` already
    /// logged, so the retry on the first full pass does not repeat it.
    pub fn new(
        shared: Option<Shared>,
        sessions_dir: Option<PathBuf>,
        watcher: Option<SessionWatcher>,
        watch: WatchSource,
        watch_error: Option<String>,
    ) -> Self {
        Self {
            core: core::Flow::new(shared, sessions_dir, watcher, watch_error),
            watch,
        }
    }

    pub fn watch(&self) -> &WatchSource {
        &self.watch
    }

    /// How long the timer waits for the next full pass.
    pub fn tick_delay(&self) -> Duration {
        self.core.tick_delay()
    }

    /// `Command::RefreshSessions` from the app layer (a settled turn, boot, an
    /// explicit refresh): always a full pass.
    pub fn on_refresh_command(&mut self) -> Task<Message> {
        let next = self.core.on_refresh_command();
        self.task(next)
    }

    /// The safety-net timer fired.
    pub fn on_tick(&mut self) -> Task<Message> {
        let next = self.core.on_tick();
        self.task(next)
    }

    /// The watcher reported changes: those paths, or everything on a rescan.
    pub fn on_batch(&mut self, batch: Batch) -> Task<Message> {
        let next = self.core.on_batch(batch);
        self.task(next)
    }

    /// The watcher stopped: the polling fallback takes over.
    pub fn on_watcher_lost(&mut self) -> Task<Message> {
        let next = self.core.on_watcher_lost();
        self.task(next)
    }

    /// One pass finished. Returns the list to hand to the app layer, when the
    /// pass produced one; the caller dispatches it and then calls [`pump`](Self::pump).
    pub fn settle(&mut self, result: Result<Refreshed, String>) -> Option<PreparedSessions> {
        let settled = self.core.settle(result);
        if settled.retry_watcher {
            self.start_watcher();
        }
        settled.sessions
    }

    /// Starts the next queued pass, if any.
    pub fn pump(&mut self) -> Task<Message> {
        let next = self.core.pump();
        self.task(next)
    }

    /// Writes the persisted cache now if a throttled save is still pending, for
    /// use on close.
    pub fn flush(&self) {
        if let Some(shared) = self.core.shared() {
            session_refresh::flush(&shared);
        }
    }

    /// The iced side of a decision: one stream per pass, nothing at all while
    /// the flow holds its hand.
    fn task(&self, next: Next) -> Task<Message> {
        let Next::Run(job) = next else {
            return Task::none();
        };
        let Some(shared) = self.core.shared() else {
            return Task::none();
        };
        // One stream per pass: the first full pass reports partial lists as it
        // goes, and every pass ends with the item that settles the job.
        Task::stream(
            session_refresh::stream(job, shared).map(|update| match update {
                Update::Progress(progress) => Message::SessionsProgress(progress),
                Update::Refreshed(result) => Message::SessionsRefreshed(result),
            }),
        )
    }

    /// Starts the watcher the settled pass asked for, and tells the flow what
    /// came of it: the new batches go to the subscription, the failure only
    /// reaches the log once per new message.
    fn start_watcher(&mut self) {
        let Some(dir) = self.core.sessions_dir() else {
            return;
        };
        match backend::try_start_watcher(&dir) {
            Ok((watcher, batches)) => {
                self.core.watcher_started(watcher);
                self.watch.fill(batches);
            }
            Err(error) => self.core.watcher_failed(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests;
