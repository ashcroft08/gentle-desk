//! The Settings panes' holder state (0.1.12, S1): the auth slice, the last update
//! check and the usage slice, beside the two notice lines and the what's-new
//! notes the holder already carries, and the version the user skipped (0.8.0
//! settings-polish S1).
//!
//! All of them are SHELL-owned: the reducer knows nothing about a config file, a
//! provider check or a registry. Neither the auth read nor the usage read has an
//! intent of its own any more (0.8.0 settings-polish S4): both run inside the
//! Settings dialog's one load (`Bridge::settings_snapshot`, which
//! `settings_snapshot::gather` fills), and the only check this module records is
//! the one a startup or a load answered. `note_updates` is that step - what a
//! read answered, and the slice to push - which is the same step every other
//! shell-owned line takes.
//!
//! The profiles read left with the Profiles pane (profiles-editor T2d): no pane
//! reads `profiles.json` from disk any more, and the chat's own Profile tab reads
//! its profile through that chat's child (`bridge::chat_profile`).

use gentle_runtime::app_config::AppConfig;

use super::{lock, AppStateHolder};
use crate::contract::{AuthView, UpdatesView, UsageView, ViewState};

impl AppStateHolder {
    /// Records the update check's answer and answers the slice to push.
    pub fn note_updates(&self, updates: UpdatesView) -> ViewState {
        *lock(&self.updates) = updates;
        self.view()
    }

    /// The Account pane's slice as it stands.
    pub(super) fn auth(&self) -> AuthView {
        lock(&self.auth).clone()
    }

    /// The Updates pane's slice as it stands.
    ///
    /// The skipped version is the CONFIG's, like the theme (0.8.0 settings-polish
    /// S1): it is laid over the rows of every push, so the slice carries it
    /// before the first check has run, right after a restart, and the moment the
    /// user skips or stops skipping - without waiting for a check.
    pub(super) fn updates(&self) -> UpdatesView {
        let skipped = self.skipped_update();
        let mut updates = lock(&self.updates).clone();
        updates.skipped = skipped;
        updates
    }

    /// The version the user skipped, as the app config holds it: `None` while
    /// nothing is skipped, and never a value this app cannot compare as a version
    /// (the config reads it with that rule).
    pub fn skipped_update(&self) -> Option<String> {
        self.config_value().skipped_update
    }

    /// Skips `version` (0.8.0 settings-polish S1): the Updates pane's "Skip this
    /// version".
    ///
    /// One rule, and it is the theme's (`bridge::theme`) and the notifications
    /// mode's (`bridge::notifications`): the skip is a field of the app config, so
    /// it is edited through the same ONE step every config change takes
    /// ([`AppStateHolder::update_config`]) and saved by the same writer. The same
    /// skip again changes nothing, moves no revision and asks for no save, and a
    /// value that is not a version this app compares stores nothing.
    ///
    /// Answers the re-emitted slice, whose `updates` now names the skip - which is
    /// what the pane and the gear's dot read.
    pub fn skip_update(&self, version: &str) -> ViewState {
        self.update_config(|config| config.set_skipped_update(version));
        self.view()
    }

    /// Stops skipping the version (S2): the next check offers it again. The same
    /// config edit and the same answer as [`Self::skip_update`], and a run that
    /// skips nothing changes nothing.
    pub fn unskip_update(&self) -> ViewState {
        self.update_config(AppConfig::clear_skipped_update);
        self.view()
    }

    /// The Usage pane's slice as it stands.
    pub(super) fn usage(&self) -> UsageView {
        lock(&self.usage).clone()
    }
}
