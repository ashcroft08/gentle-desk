//! Everything that can happen to the app: process output, component events, ticks.

use gentle_app::SessionItem;
use gentle_backend::process::ProcessEvent;

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
    /// The periodic session-list refresh.
    Tick,
    /// The worker thread finished re-reading the session index.
    SessionsRefreshed(Result<Vec<SessionItem>, String>),
    CloseRequested,
}
