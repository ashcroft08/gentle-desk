//! The shell home choice as the holder keeps it (shell-choice S1-S3).
//!
//! Three facts reach the view: the folder of the user's pi that the app found at
//! start (set once, outside every lock), whether the `gentle-shell` launcher
//! resolves, and the choice, a field of the app config that is edited through the
//! one step every config change takes ([`AppStateHolder::update_config`]) and
//! saved by the same writer. Choosing what the config already holds changes
//! nothing, moves no revision and asks for no save.
//!
//! The question is asked only when this run can run: pi found, no choice yet, and
//! the launcher there. A machine without gentle-shell sees the install screen
//! first, and the question comes once the launcher is found.
//!
//! The choice applies from the next start: the children and the session index
//! of this run already took the home they started with.

use std::sync::atomic::Ordering;

use gentle_backend::home::HomeChoice;

use super::{lock, AppStateHolder};
use crate::contract::{HomeChoiceView, InstallView, ShellView, ViewState};

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

    /// Records the home choice this run started with, the one its children and
    /// its session list are on, so a later change can be told from it.
    pub fn set_started_with(&self, choice: Option<HomeChoice>) {
        *lock(&self.started_with) = choice;
    }

    /// Records whether the `gentle-shell` launcher resolves on this machine.
    pub fn set_launcher_found(&self, found: bool) {
        self.launcher_missing.store(!found, Ordering::SeqCst);
    }

    /// "Check again": looks for the launcher once more through `probe` and
    /// answers the slice to push. The install outcome is left as it is, so the
    /// screen still says how the install went.
    pub fn recheck_launcher(&self, probe: &dyn Fn() -> bool) -> ViewState {
        self.set_launcher_found(probe());
        self.view()
    }

    /// Marks an install as started. `true` when the caller must run it: the
    /// first request, or one after a failure. While it runs, and once it is
    /// done, nothing starts a second one.
    pub fn begin_install(&self) -> bool {
        let mut install = lock(&self.install);
        match *install {
            InstallView::Idle | InstallView::Failed { .. } => {
                *install = InstallView::Running { line: None };
                true
            }
            InstallView::Running { .. } | InstallView::Done => false,
        }
    }

    /// Records how the install ended.
    pub fn finish_install(&self, outcome: Result<(), String>) {
        *lock(&self.install) = match outcome {
            Ok(()) => InstallView::Done,
            Err(message) => InstallView::Failed { message },
        };
    }

    /// What this run knows about the shell, taken off the config lock before any
    /// state lock is taken, as the theme is.
    pub(super) fn shell_view(&self) -> ShellView {
        let pi_path = lock(&self.pi_path).clone();
        let choice = self.config_snapshot().config.shell_home;
        // One read of the launcher fact, reported as it stands and used as it
        // stands: nothing runs before the launcher resolves, so the home
        // question waits for it instead of being asked on the install screen.
        let launcher_found = !self.launcher_missing.load(Ordering::SeqCst);
        ShellView {
            ask: pi_path.is_some() && choice.is_none() && launcher_found,
            choice: choice.map(view_choice),
            pi_path,
            launcher_found,
            install: lock(&self.install).clone(),
            // No choice is the separate home, so "none" and "isolated" are the
            // same home and changing between them needs no restart.
            restart_needed: effective(choice) != effective(*lock(&self.started_with)),
        }
    }
}

/// The home a choice puts a run on: no choice is the separate one.
fn effective(choice: Option<HomeChoice>) -> HomeChoice {
    choice.unwrap_or(HomeChoice::Isolated)
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
