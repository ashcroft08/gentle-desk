//! The Settings pane's holder state (0.1.12, S1): the profiles read, the auth
//! read and the last update check, beside the two notice lines and the what's-new
//! notes the holder already carries.
//!
//! All three are SHELL-owned: the reducer knows nothing about a config file, a
//! provider check or a registry, and the reads run off the intent path (the
//! update check on a thread of its own). Each `note_*` records what a read
//! answered and returns the slice to push, which is the same step every other
//! shell-owned line takes.

use super::{lock, AppStateHolder};
use crate::contract::{
    AuthView, OrchestratorModeView, ProfilesView, UpdatesStatusView, UpdatesView, ViewState,
};

impl AppStateHolder {
    /// Records what the profiles read answered and answers the slice to push.
    pub fn note_profiles(&self, profiles: ProfilesView) -> ViewState {
        *lock(&self.profiles) = profiles;
        self.view()
    }

    /// Records what the auth read answered and answers the slice to push.
    pub fn note_auth(&self, auth: AuthView) -> ViewState {
        *lock(&self.auth) = auth;
        self.view()
    }

    /// Records the update check's answer and answers the slice to push.
    pub fn note_updates(&self, updates: UpdatesView) -> ViewState {
        *lock(&self.updates) = updates;
        self.view()
    }

    /// Marks the check as running (S3: it happens once when the app opens, and
    /// only when the user asks again), keeping the versions it showed.
    pub fn note_updates_checking(&self) -> ViewState {
        lock(&self.updates).status = UpdatesStatusView::Checking;
        self.view()
    }

    /// Records the orchestrator mode the desk's next chats start with (S1, T4).
    /// The switch lives in the Profiles pane, and it is never stored: a new run
    /// starts on gentle-pi's own default.
    pub fn note_orchestrator_mode(&self, mode: OrchestratorModeView) -> ViewState {
        let mut profiles = lock(&self.profiles);
        profiles.mode = mode;
        drop(profiles);
        self.view()
    }

    /// The mode the desk's next chats start with.
    pub fn orchestrator_mode(&self) -> OrchestratorModeView {
        lock(&self.profiles).mode
    }

    /// The Profiles pane's slice as it stands.
    pub(super) fn profiles(&self) -> ProfilesView {
        lock(&self.profiles).clone()
    }

    /// The Account pane's slice as it stands.
    pub(super) fn auth(&self) -> AuthView {
        lock(&self.auth).clone()
    }

    /// The Updates pane's slice as it stands.
    pub(super) fn updates(&self) -> UpdatesView {
        lock(&self.updates).clone()
    }
}
