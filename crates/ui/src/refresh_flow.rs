//! The session refresh flow: which pass to run when, and what its result means.
//!
//! The window never refreshes on its own: a settled turn, a watcher batch and
//! the safety-net timer all ask [`Flow`] for work, and it decides. One pass runs
//! at a time ([`RefreshQueue`] holds the rest), a full pass also retries the
//! watcher while none runs, and the duration of the last full pass sets how long
//! the timer waits ([`session_schedule::full_pass_delay`]).
//!
//! Keeping this out of `app` leaves there only the wiring: state, commands, and
//! the messages this module answers.

use std::path::PathBuf;
use std::time::Duration;

use gentle_app::PreparedSessions;
use gentle_backend::session_schedule;
use gentle_backend::session_watch::{Batch, SessionWatcher};
use iced::futures::StreamExt;
use iced::Task;

use crate::backend;
use crate::message::Message;
use crate::session_refresh::{self, Refreshed, Shared, Update};
use crate::watch_source::WatchSource;
use gentle_runtime::refresh_queue::RefreshQueue;

/// The refresh state of a running window.
pub struct Flow {
    /// `None` when the pi home is unknown: there is nothing to refresh.
    shared: Option<Shared>,
    /// `<home>/sessions`, the directory a watcher is started on.
    sessions_dir: Option<PathBuf>,
    /// The running watcher; while it is `None` the list polls instead.
    watcher: Option<SessionWatcher>,
    /// Where a newly started watcher's batches go.
    watch: WatchSource,
    queue: RefreshQueue,
    /// How long the last full pass took, which sets the safety-net delay.
    last_full_pass: Duration,
    /// The last watcher failure logged, so a retry per pass logs once.
    last_watch_error: Option<String>,
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
            shared,
            sessions_dir,
            watcher,
            watch,
            queue: RefreshQueue::default(),
            last_full_pass: Duration::ZERO,
            last_watch_error: watch_error,
        }
    }

    pub fn watch(&self) -> &WatchSource {
        &self.watch
    }

    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// How long the timer waits for the next full pass: the safety net while a
    /// watcher runs, the only update while it does not.
    pub fn tick_delay(&self) -> Duration {
        session_schedule::full_pass_delay(self.is_watching(), self.last_full_pass)
    }

    /// `Command::RefreshSessions` from the app layer (a settled turn, boot, an
    /// explicit refresh): always a full pass. A watcher reports the appended file
    /// too, but its notice can arrive seconds late (measured on Windows/NTFS: 3 s
    /// on a developer machine, over 10 s on a CI runner), and the user's own chat
    /// must not wait for it. The pass runs on the worker thread, once per turn.
    pub fn on_refresh_command(&mut self) -> Task<Message> {
        self.request_full();
        self.pump()
    }

    /// The safety-net timer fired: enumerate the sessions directory and
    /// reconcile whatever a watcher missed.
    pub fn on_tick(&mut self) -> Task<Message> {
        self.request_full();
        self.pump()
    }

    /// The watcher reported changes: look at those paths, or at everything when
    /// it says events may have been lost.
    pub fn on_batch(&mut self, batch: Batch) -> Task<Message> {
        if batch.rescan {
            self.request_full();
        } else {
            self.queue.request_paths(batch.paths);
        }
        self.pump()
    }

    /// The watcher stopped: forget it, so the polling fallback takes over and
    /// the next full pass tries to start one again.
    pub fn on_watcher_lost(&mut self) -> Task<Message> {
        self.watcher = None;
        self.request_full();
        self.pump()
    }

    /// One pass finished. Returns the list to hand to the app layer, when the
    /// pass produced one; the caller dispatches it and then calls [`pump`](Self::pump).
    pub fn settle(&mut self, result: Result<Refreshed, String>) -> Option<PreparedSessions> {
        self.queue.finish();
        match result {
            Err(error) => {
                eprintln!("{error}");
                None
            }
            Ok(refreshed) => {
                if refreshed.full {
                    self.last_full_pass = refreshed.took;
                    self.retry_watcher();
                }
                Some(refreshed.sessions)
            }
        }
    }

    /// Starts the next queued pass, if any. Nothing starts while another pass
    /// has the index: its result is what calls this again.
    pub fn pump(&mut self) -> Task<Message> {
        let Some(shared) = self.shared.clone() else {
            return Task::none();
        };
        let Some(job) = self.queue.start_next() else {
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

    /// Writes the persisted cache now if a throttled save is still pending.
    pub fn flush(&self) {
        if let Some(shared) = &self.shared {
            session_refresh::flush(shared);
        }
    }

    fn request_full(&mut self) {
        self.queue.request_full();
    }

    /// Tries to start a watcher again while none runs. Events may have been
    /// missed in between, so a start also asks for one full pass. A failure is
    /// logged only when its message changed: it is retried on every full pass,
    /// and an unavailable watcher must not flood the log.
    fn retry_watcher(&mut self) {
        if self.is_watching() {
            return;
        }
        let Some(dir) = self.sessions_dir.as_deref() else {
            return;
        };
        match backend::try_start_watcher(dir) {
            Ok((watcher, batches)) => {
                self.watcher = Some(watcher);
                self.watch.fill(batches);
                self.last_watch_error = None;
                self.request_full();
            }
            Err(error) => {
                let message = error.to_string();
                if self.last_watch_error.as_deref() != Some(message.as_str()) {
                    eprintln!("session watcher unavailable: {message}");
                    self.last_watch_error = Some(message);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
