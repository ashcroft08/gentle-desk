//! The shell home choice as the holder keeps it (shell-choice S1-S3).
//!
//! Two facts reach the view: the folder of the user's pi that the app found at
//! start (set once, outside every lock), and the choice, a field of the app
//! config that is edited through the one step every config change takes
//! ([`AppStateHolder::update_config`]) and saved by the same writer. Choosing
//! what the config already holds changes nothing, moves no revision and asks for
//! no save.
//!
//! The choice applies from the next start: the children and the session index
//! of this run already took the home they started with.

use gentle_backend::home::HomeChoice;

use super::{lock, AppStateHolder};
use crate::contract::{HomeChoiceView, ShellView, ViewState};

impl AppStateHolder {
    /// Records the folder of the user's pi, as found at startup (`None` when
    /// there is none).
    pub fn set_pi_path(&self, path: Option<String>) {
        *lock(&self.pi_path) = path;
    }

    /// Chooses the home `gentle-shell` runs on: one atomic config edit, saved by
    /// the one writer. Answers the slice to push.
    pub fn set_shell_home(&self, choice: HomeChoiceView) -> ViewState {
        self.update_config(|config| config.set_shell_home(home_choice(choice)));
        self.view()
    }

    /// What this run knows about the shell, taken off the config lock before any
    /// state lock is taken, as the theme is.
    pub(super) fn shell_view(&self) -> ShellView {
        let pi_path = lock(&self.pi_path).clone();
        let choice = self.config_snapshot().config.shell_home;
        ShellView {
            ask: pi_path.is_some() && choice.is_none(),
            choice: choice.map(view_choice),
            pi_path,
        }
    }
}

fn home_choice(choice: HomeChoiceView) -> HomeChoice {
    match choice {
        HomeChoiceView::Link => HomeChoice::Link,
        HomeChoiceView::Isolated => HomeChoice::Isolated,
    }
}

fn view_choice(choice: HomeChoice) -> HomeChoiceView {
    match choice {
        HomeChoice::Link => HomeChoiceView::Link,
        HomeChoice::Isolated => HomeChoiceView::Isolated,
    }
}
