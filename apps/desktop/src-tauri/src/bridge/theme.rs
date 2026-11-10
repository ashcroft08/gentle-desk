//! The theme setting as the holder keeps it (the v1 Appearance setting).
//!
//! One rule: the choice is a field of the app config, so it is edited through
//! the same ONE step every config change takes
//! ([`AppStateHolder::update_config`]: read, edit and install under the config
//! lock) and saved by the same writer. Setting the theme the config already
//! holds changes nothing, moves no revision and asks for no save.
//!
//! The window's own chrome is a side effect of the change, never part of it: the
//! sink is taken off its lock before it is asked, so the OS call never runs
//! under a lock, and a window that refuses it is one stderr line - the choice
//! stays the user's and the run keeps it in memory.

use std::sync::Arc;

use gentle_runtime::app_config::Theme;

use super::{lock, AppStateHolder};
use crate::contract::{ThemeView, ViewState};
use crate::theme::WindowTheme;

impl AppStateHolder {
    /// Installs the one sink that changes the window's own chrome. Before it
    /// exists the theme is still the user's; it simply has no window to change.
    pub fn arm_window_theme(&self, sink: Arc<dyn WindowTheme>) {
        *lock(&self.window_theme) = Some(sink);
    }

    /// Makes the window's chrome follow the theme this run holds: once at
    /// startup, and once per change.
    pub fn apply_window_theme(&self) {
        let theme = self.config_value().theme;
        let sink = lock(&self.window_theme).clone();
        if let Some(sink) = sink {
            if let Err(error) = sink.set(theme) {
                // The setting is the user's and the run keeps it: a window that
                // cannot take it is not a config problem, so no notice is made.
                eprintln!("the window theme could not be applied: {error}");
            }
        }
    }

    /// Chooses the theme of the window: one atomic config edit, saved by the one
    /// writer, and the window's chrome follows it. Answers the slice to push.
    pub fn set_theme(&self, theme: ThemeView) -> ViewState {
        let changed = self.update_config(|config| config.set_theme(config_theme(theme)));
        if changed {
            self.apply_window_theme();
        }
        self.view()
    }
}

/// The chosen theme as the config document keeps it: the same six names.
fn config_theme(theme: ThemeView) -> Theme {
    match theme {
        ThemeView::System => Theme::System,
        ThemeView::Dark => Theme::Dark,
        ThemeView::Gentle => Theme::Gentle,
        ThemeView::GentlemanCute => Theme::GentlemanCute,
        ThemeView::GentlemanSexy => Theme::GentlemanSexy,
        ThemeView::Light => Theme::Light,
    }
}
