//! Everything that can happen to the app: one child's translated event, the
//! outcome of a command for one child, component events, watcher batches, the
//! adaptive refresh timer.

use gentle_backend::session_watch::Batch;

use crate::session_refresh::{Progress, Refreshed};
use crate::view::{composer, dialog, header, helper_thread, helpers, notices, sidebar, thread};
use gentle_runtime::chat_runtime::{Converted, Outcome};

#[derive(Debug, Clone)]
pub enum Message {
    /// One event of one chat's child, already translated: a decoded protocol
    /// event with its arrival time, a line of its stderr, or its stream ending.
    Chat(Converted),
    /// One per-chat command finished: the child is up, the command was written,
    /// or it says why neither happened.
    ChatOutcome(Outcome),
    Header(header::Event),
    Sidebar(sidebar::Event),
    Thread(thread::Event),
    Notices(notices::Event),
    Dialog(dialog::Event),
    Composer(composer::Event),
    Helpers(helpers::Event),
    HelperThread(helper_thread::Event),
    /// The safety-net (or fallback) full pass is due.
    Tick,
    /// The host finished reaping the idle background children. Nothing to draw
    /// and nothing to ask for: it exists so the reap can run off this thread.
    ReapDone,
    /// The watcher reported session files that changed.
    SessionsChanged(Batch),
    /// The watcher stopped: nothing reports changes until one starts again.
    WatcherLost,
    /// The worker thread finished a refresh pass.
    SessionsRefreshed(Result<Refreshed, String>),
    /// The first full pass reported how far it got: the list as read so far.
    SessionsProgress(Progress),
    CloseRequested,
}
