//! The flattened tree of workspaces and their chats, as the sidebar reads it one
//! page at a time (S3, D1, D7).
//!
//! The tree is built from two inputs and nothing else: the app config (the
//! folders the user added, and their collapsed state) and the session list the
//! app state holds. The grouping, naming and order are
//! [`gentle_runtime::workspaces`]', never a second rule here; this module only
//! FLATTENS that list into rows: one header row per workspace, then - only when
//! the workspace is not collapsed - one row per chat of it.
//!
//! Whether the folder of a workspace still exists is NOT part of the tree: it
//! touches the file system, so it is asked per served page and only for the
//! headers that page holds (`sessions::window`).

use gentle_app::SessionItem;
use gentle_runtime::app_config::AppConfig;
use gentle_runtime::workspaces::{self, Workspace};

/// One row of the flattened tree, by index into [`Tree`]'s own lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Header(u32),
    Chat(u32),
}

/// One workspace header, as the tree holds it: what the workspace is and how it
/// is drawn, without whether its folder is still there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub key: String,
    pub label: String,
    pub path: String,
    pub name: String,
    pub collapsed: bool,
    pub chat_count: u32,
}

impl Header {
    /// The header of one workspace of the runtime's list.
    fn of(workspace: &Workspace) -> Self {
        Self {
            key: workspace.key.clone(),
            label: workspace.label.clone(),
            path: workspace.path.clone(),
            name: workspace.name.clone(),
            collapsed: workspace.collapsed,
            chat_count: u32::try_from(workspace.sessions.len()).unwrap_or(u32::MAX),
        }
    }
}

/// The flattened tree of one revision: the rows in the order the sidebar draws
/// them, and the chats they point at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    revision: u32,
    headers: Vec<Header>,
    chats: Vec<SessionItem>,
    rows: Vec<Row>,
}

impl Tree {
    /// The flattened tree of `config` and `sessions`.
    pub fn build(revision: u32, config: &AppConfig, sessions: &[SessionItem]) -> Self {
        let list = workspaces::build(config, sessions);
        let mut headers: Vec<Header> = Vec::new();
        let mut chats: Vec<SessionItem> = Vec::new();
        let mut rows: Vec<Row> = Vec::new();
        for workspace in &list.workspaces {
            headers.push(Header::of(workspace));
            rows.push(Row::Header(index_of(headers.len())));
            if workspace.collapsed {
                // A folded group shows its header and none of its chats (D8).
                continue;
            }
            for index in &workspace.sessions {
                let Some(session) = sessions.get(*index) else {
                    continue;
                };
                chats.push(session.clone());
                rows.push(Row::Chat(index_of(chats.len())));
            }
        }
        Self {
            revision,
            headers,
            chats,
            rows,
        }
    }

    /// The revision this tree was built for: the key [`crate::sessions::cache`]
    /// keeps it under.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// How many rows the whole tree has: headers and chats, folded groups
    /// skipped. The `total` a window answers with.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the tree has no row at all: no workspace and no chat.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The rows `[offset, offset + length)`, cut to what the tree holds.
    pub fn page(&self, offset: usize, length: usize) -> &[Row] {
        let start = offset.min(self.rows.len());
        let end = start.saturating_add(length).min(self.rows.len());
        &self.rows[start..end]
    }

    pub fn header(&self, index: u32) -> Option<&Header> {
        self.headers.get(index as usize)
    }

    pub fn chat(&self, index: u32) -> Option<&SessionItem> {
        self.chats.get(index as usize)
    }

    /// Whether a workspace of this tree is the folder `key` names
    /// ([`gentle_runtime::app_config::folder_key`]): the folders the config
    /// holds and the ones the sessions name, exactly as the sidebar shows them.
    pub fn knows(&self, key: &str) -> bool {
        self.headers.iter().any(|header| header.key == key)
    }
}

/// The index a list reaches after one more push, as the `u32` a [`Row`] holds.
fn index_of(len: usize) -> u32 {
    len.saturating_sub(1).try_into().unwrap_or(u32::MAX)
}
