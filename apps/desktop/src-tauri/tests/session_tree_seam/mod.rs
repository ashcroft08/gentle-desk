//! The seam of the workspace tree tests: a config, a session list and the tree
//! the app serves, with no window, no Tauri app and no real home.
//!
//! The grouping and the order are `gentle-runtime`'s, so these fixtures never
//! decide them: they build the same tree the holder builds and read the rows
//! back through the contract.
#![allow(dead_code)] // each test binary uses one part of it

use gentle_app::SessionItem;
use gentle_desk_lib::contract::{SessionRowView, SessionTreeRow, WorkspaceRowView};
use gentle_runtime::app_config::AppConfig;
use gentle_runtime::app_config::WorkspaceStatus;

/// A config holding `paths` as workspaces, in that order, all expanded.
pub fn config(paths: &[&str]) -> AppConfig {
    let mut config = AppConfig::default();
    for path in paths {
        assert!(config.add_workspace(path), "add_workspace({path})");
    }
    config
}

/// The same, with every folder of `paths` folded away.
pub fn folded(paths: &[&str]) -> AppConfig {
    let mut config = config(paths);
    for path in paths {
        assert!(config.set_collapsed(path, true), "set_collapsed({path})");
    }
    config
}

/// A session header reporting `cwd`, last written at `modified_ms`.
pub fn session(cwd: &str, modified_ms: u64) -> SessionItem {
    SessionItem {
        path: format!("sessions/{modified_ms}.jsonl"),
        title: format!("chat {modified_ms}"),
        cwd: cwd.to_owned(),
        modified_ms,
        message_count: 1,
    }
}

/// Every folder is there: the probe a test that cares about rows uses.
pub fn available(_path: &str) -> WorkspaceStatus {
    WorkspaceStatus::Available
}

/// The header of one row, for a test that knows the row is a workspace.
pub fn header(row: &SessionTreeRow) -> &WorkspaceRowView {
    match row {
        SessionTreeRow::Workspace(workspace) => workspace,
        other => panic!("expected a workspace header, got {other:?}"),
    }
}

/// The chat of one row, for a test that knows the row is a chat.
pub fn chat(row: &SessionTreeRow) -> &SessionRowView {
    match row {
        SessionTreeRow::Chat(session) => session,
        other => panic!("expected a chat row, got {other:?}"),
    }
}

/// `W` for a workspace header and `C` for a chat: the shape of a page.
pub fn shape(rows: &[SessionTreeRow]) -> String {
    rows.iter()
        .map(|row| match row {
            SessionTreeRow::Workspace(_) => 'W',
            SessionTreeRow::Chat(_) => 'C',
        })
        .collect()
}

/// The chat ids of a page, in the order it holds them.
pub fn chat_ids(rows: &[SessionTreeRow]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| match row {
            SessionTreeRow::Chat(session) => Some(session.id.clone()),
            SessionTreeRow::Workspace(_) => None,
        })
        .collect()
}
