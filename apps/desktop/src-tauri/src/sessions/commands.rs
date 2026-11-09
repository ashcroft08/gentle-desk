//! The two read commands of the session list: a page of rows, and the first
//! read of a web view (S2, S3).
//!
//! Both are reads, so neither folds an intent nor emits an event: a sidebar
//! that missed the pushes asks for the truth and gets it. They live beside the
//! list they serve instead of in `commands`, whose job is the intent path.

use tauri::State;

use crate::commands::Bridge;
use crate::contract::{SessionsWindow, ViewState};

/// One page of the flattened workspace tree (S3): the rows
/// `[offset, offset + length)` of it, and how many rows the tree has in total -
/// workspace headers included. It reads the CURRENT state, so a sidebar that
/// missed a push still gets the truth; the length is capped, so no caller can
/// ask for a whole 50 000-chat list.
#[tauri::command]
#[specta::specta]
pub fn sessions_window(offset: u32, length: u32, bridge: State<'_, Bridge>) -> SessionsWindow {
    bridge.inner().holder.sessions_window(offset, length)
}

/// The first read of a web view: the slice the state holds right now. It emits
/// nothing, so it cannot race the pushes it duplicates; a view that starts after
/// the first pass already settled reads the settled list here.
#[tauri::command]
#[specta::specta]
pub fn get_view_state(bridge: State<'_, Bridge>) -> ViewState {
    bridge.inner().holder.view()
}
