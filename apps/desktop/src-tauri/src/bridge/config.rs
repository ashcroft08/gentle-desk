//! The config snapshot as the holder keeps and changes it.
//!
//! One rule, and it is the reason this module exists: a change to the config is
//! a READ, an EDIT and a WRITE that must be ONE step. Two folds of two different
//! workspaces that read the same base config would otherwise lose each other's
//! change, the second write putting back the workspace the first one added or
//! unfolded. [`AppStateHolder::update_config`] therefore runs the whole edit
//! under the config lock, installs what it answers, and only then hands the
//! snapshot to the writer.
//!
//! Nothing here touches the disk: the snapshot is installed in memory, and the
//! save travels to the writer's own thread (`crate::config::ConfigWriter`). No
//! other lock is taken inside the lock held here, and no closure of a caller
//! runs under any other lock.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use gentle_runtime::app_config::AppConfig;

use super::{lock, AppStateHolder};
use crate::config::{ConfigSnapshot, ConfigWriter};
use crate::contract::ViewState;

impl AppStateHolder {
    /// Installs the one writer of the config file, so a change the user made to
    /// a workspace survives the run (D8).
    pub fn arm_config_saves(&self, writer: ConfigWriter) {
        *lock(&self.saver) = Some(writer);
    }

    /// Replaces the config snapshot as a whole, without saving it: the path a
    /// test or a startup reads a snapshot through.
    ///
    /// The revision moves, so the page of rows the web side holds is stale at
    /// once (the tree is built from the config too), and the snapshot gets a new
    /// version. The notice is kept: it describes what became of the file.
    pub fn replace_config(&self, config: AppConfig) {
        let installed = self.swap_config(|snapshot| {
            snapshot.config = config;
            true
        });
        if installed.is_some() {
            self.sessions_revision.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Edits the config in ONE step and asks for it to be saved.
    ///
    /// The read, the edit and the install all happen under the config lock, so
    /// two callers can never read the same base and overwrite each other. The
    /// closure runs under that lock and must stay in memory: no file, no other
    /// lock, no callback. It answers whether it changed anything; a `false`
    /// leaves the snapshot exactly as it was, moves no revision, and saves
    /// nothing.
    ///
    /// Answers whether the config changed.
    pub fn update_config(&self, edit: impl FnOnce(&mut AppConfig) -> bool) -> bool {
        let Some((snapshot, version)) = self.swap_config(|current| edit(&mut current.config))
        else {
            return false;
        };
        self.sessions_revision.fetch_add(1, Ordering::Relaxed);
        // The lock is released: the save never runs here and never under it.
        self.persist(version, &snapshot);
        true
    }

    /// The config this run holds, as a value to read.
    pub fn config_value(&self) -> AppConfig {
        self.config_snapshot().config.clone()
    }

    /// Records what became of the config file, so the next slice carries it as
    /// `ViewState.config_notice` and the run keeps the value it holds in memory.
    /// The revision does not move: no list changed.
    pub fn note_config_problem(&self, message: String) -> ViewState {
        {
            let mut snapshot = lock(&self.config);
            Arc::make_mut(&mut snapshot).notice = Some(message);
        }
        self.view()
    }

    /// The config snapshot of this run, taken off the lock.
    pub(super) fn config_snapshot(&self) -> Arc<ConfigSnapshot> {
        Arc::clone(&lock(&self.config))
    }

    /// Runs `change` under the config lock and installs what it answers, with a
    /// new version. The change runs on a copy, so a refusal leaves the snapshot
    /// the state already holds untouched.
    fn swap_config(
        &self,
        change: impl FnOnce(&mut ConfigSnapshot) -> bool,
    ) -> Option<(Arc<ConfigSnapshot>, u64)> {
        let mut guard = lock(&self.config);
        let mut snapshot = Arc::clone(&guard);
        if !change(Arc::make_mut(&mut snapshot)) {
            return None;
        }
        *guard = Arc::clone(&snapshot);
        // Numbered under the same lock that installed it: the version order is
        // the install order, whatever order the callers reach the writer in.
        let version = self.config_version.fetch_add(1, Ordering::Relaxed) + 1;
        Some((snapshot, version))
    }

    /// Hands one numbered snapshot to the writer, if one is installed. A run
    /// with no config path has none: its change lives in memory only.
    fn persist(&self, version: u64, snapshot: &Arc<ConfigSnapshot>) {
        let writer = lock(&self.saver).clone();
        if let Some(writer) = writer {
            writer.save(version, &snapshot.config);
        }
    }
}
