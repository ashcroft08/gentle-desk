//! The Settings panes' holder state (0.1.12, S1): the auth read, the last update
//! check and the usage read, beside the two notice lines and the what's-new notes
//! the holder already carries.
//!
//! All of them are SHELL-owned: the reducer knows nothing about a config file, a
//! provider check or a registry, and the reads run off the intent path (the
//! update check on a thread of its own). Each `note_*` records what a read
//! answered and returns the slice to push, which is the same step every other
//! shell-owned line takes.
//!
//! The profiles read left with the Profiles pane (profiles-editor T2d): no pane
//! reads `profiles.json` from disk any more, and the chat's own Profile tab reads
//! its profile through that chat's child (`bridge::chat_profile`).

use super::{lock, AppStateHolder};
use crate::contract::{
    AuthView, UpdatesStatusView, UpdatesView, UsageStatusView, UsageView, ViewState,
};

impl AppStateHolder {
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

    /// The Account pane's slice as it stands.
    pub(super) fn auth(&self) -> AuthView {
        lock(&self.auth).clone()
    }

    /// The Updates pane's slice as it stands.
    pub(super) fn updates(&self) -> UpdatesView {
        lock(&self.updates).clone()
    }

    /// Marks the usage read as running, keeping what it showed: the pane's
    /// quiet loading line, never a blocking spinner.
    pub fn note_usage_reading(&self) -> ViewState {
        lock(&self.usage).status = UsageStatusView::Reading;
        self.view()
    }

    /// Records what the usage read answered and answers the slice to push.
    pub fn note_usage(&self, usage: UsageView) -> ViewState {
        *lock(&self.usage) = usage;
        self.view()
    }

    /// The Usage pane's slice as it stands.
    pub(super) fn usage(&self) -> UsageView {
        lock(&self.usage).clone()
    }
}
