//! Everything that can happen to the app: process output, component events, ticks.

use gentle_backend::process::ProcessEvent;

use crate::view::{composer, dialog, header, notices, sidebar, thread};

#[derive(Debug, Clone)]
pub enum Message {
    Process(ProcessEvent),
    Header(header::Event),
    Sidebar(sidebar::Event),
    Thread(thread::Event),
    Notices(notices::Event),
    Dialog(dialog::Event),
    Composer(composer::Event),
    /// The periodic session-list refresh.
    Tick,
    CloseRequested,
}
