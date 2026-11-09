//! The windowed read of the workspace tree (S3): `offset`/`length` over the
//! flattened rows of [`Tree`], never the whole tree over the wire.
//!
//! A window is the plain slice of the flattened rows: a home of 50 000 chats
//! costs the rows the sidebar really draws, and the length is capped at
//! [`MAX_WINDOW`] so no caller can ask for the whole list back.
//!
//! One rule lives here and nowhere else: whether the folder of a workspace
//! exists is asked for the headers THIS PAGE holds, at most [`MAX_WINDOW`] of
//! them, never for every workspace of the list. The question touches the file
//! system, so it is asked through the `status` seam, outside every lock.

use std::path::Path;

use gentle_runtime::app_config::{workspace_status, WorkspaceStatus};

use crate::contract::{SessionTreeRow, SessionsWindow, WorkspaceRowView};
use crate::sessions::tree::{Header, Row, Tree};
use crate::sessions::view;

/// The most rows one window answers with. A window is a page the sidebar draws,
/// not a way to ship the whole list: a longer length is cut to this.
pub const MAX_WINDOW: u32 = 200;

/// The rows `[offset, offset + length)` of the flattened tree, with the ages of
/// the chat rows measured at `now_ms` and the folders of the served headers
/// asked through `status`.
pub fn window(
    tree: &Tree,
    offset: u32,
    length: u32,
    now_ms: u64,
    status: &dyn Fn(&str) -> WorkspaceStatus,
) -> SessionsWindow {
    let length = length.min(MAX_WINDOW) as usize;
    let rows = tree
        .page(offset as usize, length)
        .iter()
        .filter_map(|row| match *row {
            Row::Header(index) => tree.header(index).map(|header| header_row(header, status)),
            Row::Chat(index) => tree
                .chat(index)
                .map(|session| SessionTreeRow::Chat(view::row(session, now_ms))),
        })
        .collect();
    SessionsWindow {
        total: view::narrow(tree.len()),
        rows,
    }
}

/// What the file system says about the folder of a workspace: the probe the
/// running app passes to [`window`].
pub fn folder_status(path: &str) -> WorkspaceStatus {
    workspace_status(Path::new(path))
}

/// One workspace header, with its folder asked through `status`.
fn header_row(header: &Header, status: &dyn Fn(&str) -> WorkspaceStatus) -> SessionTreeRow {
    let folder = status(&header.path);
    SessionTreeRow::Workspace(WorkspaceRowView {
        key: header.key.clone(),
        label: header.label.clone(),
        path: header.path.clone(),
        name: header.name.clone(),
        collapsed: header.collapsed,
        chat_count: header.chat_count,
        available: folder.is_available(),
        reason: folder.reason().map(str::to_owned),
    })
}
