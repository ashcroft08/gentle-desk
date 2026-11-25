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
use gentle_runtime::chat_runtime::PackagesRead;

use super::{lock, AppStateHolder};
use crate::contract::{AuthView, PackagesView, UpdatesView, UsageView, ViewState};

impl AppStateHolder {
    /// Records the update check's answer and answers the slice to push.
    pub fn note_updates(&self, updates: UpdatesView) -> ViewState {
        *lock(&self.updates) = updates;
        self.view()
    }

    /// Records what the explicit extensions check answered and answers the slice
    /// to push (packages-status T4, S7).
    ///
    /// The result is HELD, not just pushed: every later slice carries it
    /// ([`Self::updates`]), which is what lets the gear's dot and the rail light
    /// without Settings being open, and the dialog's own load prefer it over a
    /// plain read that can only say "not checked yet" (S6). The read is the typed
    /// one and the derivation is [`PackagesView::of`], the same one the dialog's
    /// plain read takes - one rule for both.
    pub fn note_packages(&self, read: &PackagesRead) -> ViewState {
        *lock(&self.packages) = Some(PackagesView::of(read));
        self.view()
    }

    /// The Extensions block this run's explicit check answered, `None` until one
    /// does. A clone, so reading it holds the lock for that and nothing else.
    pub fn held_packages(&self) -> Option<PackagesView> {
        lock(&self.packages).clone()
    }

    /// One AUTOMATIC attempt of the run's extensions check came to `read`
    /// (packages-status T7).
    ///
    /// It settles the run's retry rule and nothing else: the block itself is held
    /// and pushed by [`Self::note_packages`], the same step the user's own check
    /// takes. A check that asked nothing (`Unasked`) or timed out (`Deadline`) did
    /// NOT spend the run's question, so the next chat's first event may try again:
    /// on a chat whose child was ready then, or on the one this gate could not
    /// name. A listing read, or the shell's own refusal, stops the retries - asking
    /// that child again would ask the same question.
    pub fn note_packages_attempt(&self, read: &PackagesRead) {
        let mut attempts = lock(&self.packages_check);
        attempts.in_flight = false;
        if matches!(read, PackagesRead::Read(_) | PackagesRead::Refused(_)) {
            attempts.answered = true;
        }
    }

    /// The block the Settings dialog draws (T4, S7): the checked answer this run
    /// holds when it carries a listing, else the plain read's own derivation.
    ///
    /// Reading it is one lock: the dialog never waits on the long check, and a
    /// check still running simply has not been held yet.
    pub fn checked_packages_over(&self, plain: PackagesView) -> PackagesView {
        let held = lock(&self.packages).clone();
        PackagesView::checked_over(held.as_ref(), plain)
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
    ///
    /// The Extensions block is the HELD one (packages-status T4, S7), laid over
    /// the same slice: an explicit check answers on its own thread, and the push
    /// that carries it then starts carrying it for every later push too, so the
    /// gear's dot and the pane read the checked packages from `ViewState.updates`
    /// whether or not the dialog was ever opened.
    pub(super) fn updates(&self) -> UpdatesView {
        let skipped = self.skipped_update();
        let held = lock(&self.packages).clone();
        let mut updates = lock(&self.updates).clone();
        updates.skipped = skipped;
        if let Some(packages) = held {
            updates.packages = packages;
        }
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

#[cfg(test)]
mod tests {
    use gentle_runtime::chat_runtime::{PackagesItem, PackagesRead, PackagesState, PackagesStatus};

    use super::*;
    use crate::contract::PackagesBehindView;

    /// One explicit check's answer: `gentle-pi` behind, nothing else.
    fn checked() -> PackagesRead {
        PackagesRead::Read(PackagesStatus {
            available: true,
            items: vec![
                PackagesItem {
                    name: "gentle-pi".to_owned(),
                    current: Some("4.9.0".to_owned()),
                    state: PackagesState::UpdateAvailable,
                },
                PackagesItem {
                    name: "a-bridge".to_owned(),
                    current: None,
                    state: PackagesState::Unknown,
                },
            ],
            truncated: false,
            checked_ms: Some(1_730_000_000_000.0),
        })
    }

    /// packages-status T4, S7: the explicit check's answer is HELD, and every
    /// later slice - the one the gear's dot reads - carries it, so nothing has to
    /// open Settings first.
    #[test]
    fn a_held_check_rides_in_every_updates_slice() {
        let holder = AppStateHolder::default();
        assert_eq!(
            holder.updates().packages,
            PackagesView::unasked(),
            "nothing has been checked yet"
        );

        let view = holder.note_packages(&checked());

        let block = view.updates.packages.clone();
        assert!(block.available);
        assert!(block.unchecked, "one item pi has not checked (S6)");
        assert_eq!(
            block.behind,
            vec![PackagesBehindView {
                name: "gentle-pi".to_owned(),
                current: Some("4.9.0".to_owned()),
            }]
        );
        assert_eq!(holder.updates().packages, block);
        // The skip is still the config's, laid over the same slice (S1 of
        // settings-polish): the two overlays never displace each other.
        assert_eq!(holder.updates().skipped, holder.skipped_update());
    }

    /// packages-status T4, S6, S7: the dialog's own load asks the holder for the
    /// block, and the checked answer wins over a plain read that could only say
    /// "not checked yet" - without waiting for the long check.
    #[test]
    fn the_dialogs_block_prefers_the_held_check() {
        let holder = AppStateHolder::default();
        let plain = PackagesView::of(&PackagesRead::Read(PackagesStatus {
            available: true,
            items: vec![PackagesItem {
                name: "gentle-pi".to_owned(),
                current: None,
                state: PackagesState::Unknown,
            }],
            truncated: false,
            checked_ms: None,
        }));
        assert!(plain.unchecked);
        assert_eq!(
            holder.checked_packages_over(plain.clone()),
            plain,
            "a run that never checked has only the plain read"
        );

        holder.note_packages(&checked());

        let held = holder.updates().packages;
        assert_eq!(
            holder.checked_packages_over(plain),
            held,
            "the check's own answer is the one the dialog draws"
        );
    }
}
