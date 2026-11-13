//! The notes of the version this run follows (what's-new S3b), and the version
//! this run runs as.
//!
//! The notes are SHELL-owned, like the two notice lines beside them: the reducer
//! knows nothing about a release manifest, and a fetch of `latest.json` is the
//! shell's business. They travel as `ViewState.whats_new`, are pushed once the
//! fetch answers with this version, and are CLEARED by the dismiss that stores
//! the version (S5) - a one-shot slice, which is what makes the modal show once
//! per version.
//!
//! The version this run runs as is what the dismiss stores and what the detection
//! compares (S1, S5): the manifest's version is never stored, so a stale manifest
//! cannot keep the modal coming back.

use gentle_runtime::whats_new::{self, UpdateRun};

use super::{lock, AppStateHolder};
use crate::contract::{ViewState, WhatsNewView};

impl AppStateHolder {
    /// Names the version this run runs as, once, at startup. Nothing is stored
    /// and nothing is pushed: the one write of the config happens on the dismiss,
    /// or at once when there is nothing to show (S5).
    pub fn set_app_version(&self, version: &str) {
        *lock(&self.app_version) = Some(version.to_owned());
    }

    /// The version this run runs as, `None` until [`Self::set_app_version`] named
    /// it.
    pub fn app_version(&self) -> Option<String> {
        lock(&self.app_version).clone()
    }

    /// What this run follows, from the version the config stored and the one it
    /// runs as (S1). A run that never named its version follows nothing, which is
    /// the quiet answer: it is a shell that never asked.
    pub fn update_run(&self) -> UpdateRun {
        let Some(app_version) = self.app_version() else {
            return UpdateRun::SameVersion;
        };
        let stored = self.config_snapshot().config.last_run_version.clone();
        whats_new::detected(stored.as_deref(), &app_version)
    }

    /// Records the notes to show and answers the slice to push. The revision does
    /// not move: no list changed. The version the config still stores is the one
    /// this run followed (it is only replaced when the modal is dismissed), so it
    /// travels with the notes as `previous_version`.
    pub fn note_whats_new(&self, version: String, notes: String) -> ViewState {
        let previous_version = self.config_snapshot().config.last_run_version.clone();
        *lock(&self.whats_new) = Some(WhatsNewView {
            version,
            previous_version,
            notes,
        });
        self.view()
    }

    /// Stores this run's OWN version as the version the last run was (S5) and
    /// clears the notes, answering the slice to push. `None` when this run never
    /// named its version: there is nothing to store, and the caller answers the
    /// slice it holds.
    ///
    /// The edit and the write are the config's own one step
    /// ([`AppStateHolder::update_config`]), so a workspace folded at the same
    /// moment survives, and the save travels to the writer's thread.
    pub fn store_last_run_version(&self) -> Option<ViewState> {
        let version = self.app_version()?;
        self.update_config(|config| config.set_last_run_version(&version));
        self.clear_whats_new();
        Some(self.view())
    }

    /// Clears the notes: the dismiss that stored the version has nothing left to
    /// show.
    pub fn clear_whats_new(&self) {
        *lock(&self.whats_new) = None;
    }

    /// The notes as the view state carries them.
    pub(super) fn whats_new(&self) -> Option<WhatsNewView> {
        lock(&self.whats_new).clone()
    }
}
