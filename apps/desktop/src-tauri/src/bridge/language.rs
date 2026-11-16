//! The language setting as the holder keeps it (0.2.0, S2 and S4).
//!
//! One rule, and it is the theme's rule (`bridge::theme`): the choice is a field
//! of the app config, so it is edited through the same ONE step every config
//! change takes ([`AppStateHolder::update_config`]: read, edit and install under
//! the config lock) and saved by the same writer. Setting the language the
//! config already holds changes nothing, moves no revision and asks for no save.
//!
//! The answer is the re-emitted slice, which is what makes the switch instant:
//! the same `ViewState` the command returns is the one the `state-changed` push
//! carries, and the web side redraws from the reactive language it reads there.
//! Nothing here detects the system language and nothing here restarts anything -
//! the run keeps its one window and its one page.

use gentle_core::i18n::Language;

use super::AppStateHolder;
use crate::contract::{LanguageView, ViewState};

impl AppStateHolder {
    /// Chooses the language of the app's own words: one atomic config edit, saved
    /// by the one writer. Answers the slice to push.
    pub fn set_language(&self, language: LanguageView) -> ViewState {
        self.update_config(|config| config.set_language(config_language(language)));
        self.view()
    }
}

/// The chosen language as the config document keeps it: the two stored tags.
fn config_language(language: LanguageView) -> Language {
    match language {
        LanguageView::En => Language::En,
        LanguageView::Es => Language::Es,
    }
}
