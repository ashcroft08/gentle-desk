//! The session refresh flow: which pass to run when, and what its result means.
//!
//! The window never refreshes on its own: a settled turn, a watcher batch and
//! the safety-net timer all ask [`Flow`] for work, and it decides. One pass runs
//! at a time ([`RefreshQueue`] holds the rest), a full pass also retries the
//! watcher while none runs, and the duration of the last full pass sets how long
//! the timer waits ([`session_schedule::full_pass_delay`]).
//!
//! Nothing here draws or spawns work: every entry point answers with a [`Next`]
//! (`Run` this pass, or `Idle`), [`Flow::settle`] hands back the list the pass
//! produced, and a full pass that should retry the watcher says so. The caller
//! starts the watcher where it runs (`crates/ui` through `backend`) and reports
//! back with [`Flow::watcher_started`] or [`Flow::watcher_failed`], so this
//! module owns the rules and the caller owns the environment.

use std::path::PathBuf;
use std::time::Duration;

use gentle_app::PreparedSessions;
use gentle_backend::session_schedule;
use gentle_backend::session_watch::{Batch, SessionWatcher};

use crate::refresh_queue::{Job, RefreshQueue};
use crate::session_refresh::{Refreshed, Shared};

/// What the flow decided the window must do now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    /// Nothing to run: no home, nothing pending, or a pass already running.
    Idle,
    /// Run this pass on the run's index ([`Flow::shared`] is the one to use).
    Run(Job),
}

/// A finished pass, as the window receives it.
pub struct Settled {
    /// The list to hand to the app layer, when the pass produced one. The
    /// caller dispatches it and then asks for [`Flow::pump`].
    pub sessions: Option<PreparedSessions>,
    /// Whether the caller must try to start a watcher again (a full pass ran
    /// while none was running) and report back with [`Flow::watcher_started`]
    /// or [`Flow::watcher_failed`].
    pub retry_watcher: bool,
}

/// The refresh state of a running window.
pub struct Flow {
    /// `None` when the pi home is unknown: there is nothing to refresh.
    shared: Option<Shared>,
    /// `<home>/sessions`, the directory a watcher is started on.
    sessions_dir: Option<PathBuf>,
    /// The running watcher; while it is `None` the list polls instead.
    watcher: Option<SessionWatcher>,
    queue: RefreshQueue,
    /// How long the last full pass took, which sets the safety-net delay.
    last_full_pass: Duration,
    /// The last watcher failure logged, so a retry per pass logs once.
    last_watch_error: Option<String>,
}

impl Flow {
    /// A run with its index (if the home is known) and the watcher `startup`
    /// managed to start (if any). `watch_error` is the failure `startup` already
    /// logged, so the retry on the first full pass does not repeat it.
    pub fn new(
        shared: Option<Shared>,
        sessions_dir: Option<PathBuf>,
        watcher: Option<SessionWatcher>,
        watch_error: Option<String>,
    ) -> Self {
        Self {
            shared,
            sessions_dir,
            watcher,
            queue: RefreshQueue::default(),
            last_full_pass: Duration::ZERO,
            last_watch_error: watch_error,
        }
    }

    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Forgets the running watcher and hands it to the caller, which is the only
    /// way to drop it OUTSIDE this flow's lock: a `SessionWatcher`'s `Drop`
    /// joins its own thread, and a join must never happen under a lock. The
    /// caller that owns the lock (the Tauri bridge) takes it out first and drops
    /// it after, then reports the end of the watch with
    /// [`Flow::on_watcher_lost`]. The iced window keeps the handle inside the
    /// flow for its whole life and never needs this.
    pub fn take_watcher(&mut self) -> Option<SessionWatcher> {
        self.watcher.take()
    }

    /// The index a [`Next::Run`] pass runs on: the caller wraps it in whatever
    /// transport it has (an iced stream, an emitting thread).
    pub fn shared(&self) -> Option<Shared> {
        self.shared.clone()
    }

    /// `<home>/sessions`, the directory a retried watcher is started on. `None`
    /// when the home is unknown: there is nothing to watch.
    pub fn sessions_dir(&self) -> Option<PathBuf> {
        self.sessions_dir.clone()
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
    pub fn on_refresh_command(&mut self) -> Next {
        self.request_full();
        self.pump()
    }

    /// The safety-net timer fired: enumerate the sessions directory and
    /// reconcile whatever a watcher missed.
    pub fn on_tick(&mut self) -> Next {
        self.request_full();
        self.pump()
    }

    /// The watcher reported changes: look at those paths, or at everything when
    /// it says events may have been lost.
    pub fn on_batch(&mut self, batch: Batch) -> Next {
        if batch.rescan {
            self.request_full();
        } else {
            self.queue.request_paths(batch.paths);
        }
        self.pump()
    }

    /// The watcher stopped: forget it, so the polling fallback takes over and
    /// the next full pass tries to start one again. A caller that must not join
    /// a thread under its own lock takes the handle out first with
    /// [`Flow::take_watcher`]; by the time this is called there is nothing left
    /// to drop.
    pub fn on_watcher_lost(&mut self) -> Next {
        self.watcher = None;
        self.request_full();
        self.pump()
    }

    /// One pass finished. Returns the list to hand to the app layer, when the
    /// pass produced one, and whether a watcher must be started again.
    pub fn settle(&mut self, result: Result<Refreshed, String>) -> Settled {
        self.queue.finish();
        match result {
            Err(error) => {
                eprintln!("{error}");
                Settled {
                    sessions: None,
                    retry_watcher: false,
                }
            }
            Ok(refreshed) => {
                let retry_watcher = refreshed.full && self.should_retry_watcher();
                if refreshed.full {
                    self.last_full_pass = refreshed.took;
                }
                Settled {
                    sessions: Some(refreshed.sessions),
                    retry_watcher,
                }
            }
        }
    }

    /// Starts the next queued pass, if any. Nothing starts while another pass
    /// has the index: its result is what calls this again.
    pub fn pump(&mut self) -> Next {
        if self.shared.is_none() {
            return Next::Idle;
        }
        match self.queue.start_next() {
            Some(job) => Next::Run(job),
            None => Next::Idle,
        }
    }

    fn request_full(&mut self) {
        self.queue.request_full();
    }

    /// Whether a full pass should try the watcher again: only while none runs,
    /// and only when there is a directory to watch.
    fn should_retry_watcher(&self) -> bool {
        !self.is_watching() && self.sessions_dir.is_some()
    }

    /// The caller started a watcher the last full pass asked for. Events may
    /// have been missed in between, so a start also asks for one full pass.
    pub fn watcher_started(&mut self, watcher: SessionWatcher) {
        self.watcher = Some(watcher);
        self.last_watch_error = None;
        self.request_full();
    }

    /// The caller could not start the watcher. A failure is logged only when its
    /// message changed: it is retried on every full pass, and an unavailable
    /// watcher must not flood the log.
    pub fn watcher_failed(&mut self, message: String) {
        if self.last_watch_error.as_deref() != Some(message.as_str()) {
            eprintln!("session watcher unavailable: {message}");
            self.last_watch_error = Some(message);
        }
    }
}

#[cfg(test)]
mod tests;
