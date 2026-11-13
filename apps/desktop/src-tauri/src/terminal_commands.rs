//! The `open_terminal` command (0.1.12, S1, S2, T2, T3): one "Open in terminal"
//! button, named by what it opens.
//!
//! A COMMAND and not a `UiIntent`, exactly as `open_link` is: nothing of the app
//! state changes when a terminal window opens, and no reducer input describes
//! one. The `argv` the shell is run with is [`crate::terminal`]'s, and the home
//! flags are the ones the chats' children get, so the window is never pointed at
//! another home than the one the desk is showing.

use crate::commands::Bridge;
use crate::contract::TerminalActionView;

/// Opens a visible terminal window on `action`: the shell's profiles screen, its
/// login or logout, or its own update.
///
/// Opening a window is a fast call that waits for nothing - the command answers
/// as soon as the window exists, and the terminal keeps running on its own - so
/// it runs on this thread and no lock of the bridge is held meanwhile.
#[tauri::command]
#[specta::specta]
pub fn open_terminal(
    action: TerminalActionView,
    bridge: tauri::State<'_, Bridge>,
) -> Result<(), String> {
    bridge.open_terminal(action)
}
