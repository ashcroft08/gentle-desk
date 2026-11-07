//! Everything that can happen to the app: process output, component events,
//! watcher batches, the adaptive refresh timer.

use gentle_backend::process::ProcessEvent;
use gentle_backend::session_watch::Batch;

use crate::session_refresh::Refreshed;
use crate::view::{composer, dialog, header, helper_thread, helpers, notices, sidebar, thread};

#[derive(Debug, Clone)]
pub enum Message {
    Process(ProcessEvent),
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
    /// The watcher reported session files that changed.
    SessionsChanged(Batch),
    /// The watcher stopped: nothing reports changes until one starts again.
    WatcherLost,
    /// The worker thread finished a refresh pass.
    SessionsRefreshed(Result<Refreshed, String>),
    CloseRequested,
}
