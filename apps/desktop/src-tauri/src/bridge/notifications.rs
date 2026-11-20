//! The notifications setting as the holder keeps it (notifications S4).
//!
//! One rule, and it is the theme's rule (`bridge::theme`) and the language's
//! (`bridge::language`): the mode is a field of the app config, so it is edited
//! through the same ONE step every config change takes
//! ([`AppStateHolder::update_config`]: read, edit and install under the config
//! lock) and saved by the same writer. Setting the mode the config already holds
//! changes nothing, moves no revision and asks for no save.
//!
//! The answer is the re-emitted slice, which is what makes the choice instant:
//! the same `ViewState` the command returns is the one the `state-changed` push
//! carries, and the web side reads the mode it decides with there. Nothing here
//! plays a tone or sends a toast - the shell only says what the user chose; the
//! detector in the web store acts on it.

use gentle_runtime::app_config::Notifications;

use super::AppStateHolder;
use crate::contract::{NotificationsMode, ViewState};

impl AppStateHolder {
    /// Chooses when the app notifies the user: one atomic config edit, saved by
    /// the one writer. Answers the slice to push.
    pub fn set_notifications(&self, mode: NotificationsMode) -> ViewState {
        self.update_config(|config| config.set_notifications(config_notifications(mode)));
        self.view()
    }
}

/// The chosen mode as the config document keeps it: the same two stored names.
fn config_notifications(mode: NotificationsMode) -> Notifications {
    match mode {
        NotificationsMode::WhenUnfocused => Notifications::WhenUnfocused,
        NotificationsMode::Off => Notifications::Off,
    }
}
