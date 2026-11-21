//! The Updates pane's slice (0.1.12, S2, S3, S7; 0.8.1 settings-polish S6, S8):
//! what this machine runs and what is published for it.
//!
//! The comparison itself is [`gentle_runtime::updates`]'s (strict `x.y.z`, a
//! version that is not that shape is never older); this module gathers the four
//! facts through the seams and lays them out as rows:
//!
//! - the desk, from the release manifest the updater already reads;
//! - pi, from `gentle-shell --version`'s `pi` line against the npm registry;
//! - gentle-shell, from the same probe's `gentle-shell` line against npm's
//!   STABLE `gentle-pi` entry (S6). The channel rule the row used to keep is
//!   still a rule, worn as a safety valve: only an installed version that is
//!   itself a stable `x.y.z` is measured, because a main-channel shell compares
//!   GitHub commit SHAs and no npm version is its newer one.
//!
//! `--version` is the ONE probe: gentle-shell prints its own version and the
//! version of the pi it runs, so the pane needs no second process and no
//! `pi` on `PATH`.
//!
//! Every read that fails is carried as a SHORT clause on its own row (S8, D12):
//! the client's message names addresses and lives in English, so it is turned
//! into "offline" or "the registry did not answer" here and never travels up.
//! An unknown row is then drawn as unknown, never as "up to date".
//!
//! Nothing here installs anything: the pane's buttons are the web's and run the
//! terminal's own commands.

use gentle_core::i18n::Language;
use gentle_runtime::updates::{self as rule, ShellVersions};
use gentle_runtime::whats_new::Latest;

use super::{VersionProbe, WebGet};
use crate::contract::{UpdateReasonView, UpdatesStatusView, UpdatesView, VersionRowView};

/// Where pi's published version lives: the registry entry gentle-pi's own update
/// check reads.
pub const PI_REGISTRY: &str = "https://registry.npmjs.org/@earendil-works/pi-coding-agent/latest";

/// Where gentle-shell's published version lives (0.8.1 settings-polish S6): the
/// npm package that installs the shell, whose `latest` dist-tag is the stable
/// channel's version. One GET through the same seam as pi's, with the same budget.
pub const SHELL_REGISTRY: &str = "https://registry.npmjs.org/gentle-pi/latest";

/// The pane's slice, from the facts the seams answer. Every failure is local to
/// its row: a registry that did not answer leaves that row's `latest` unknown,
/// says why on the row, and leaves the others untouched.
///
/// `language` is handed to the version probe, which words its own failure in it
/// (0.2.0, S3).
///
/// `skipped` is the version the user refused before this check ran (S1, S2), and
/// `now_ms` is the instant the check finished (S8): the pane words its last-checked
/// line from it and never from a clock of its own.
pub fn view(
    desk_version: Option<&str>,
    manifest: Option<&str>,
    web: &WebGet,
    versions: &VersionProbe,
    language: Language,
    skipped: Option<&str>,
    now_ms: u64,
) -> UpdatesView {
    let ShellVersions {
        shell: shell_installed,
        pi: pi_installed,
    } = versions.as_ref()(language)
        .map(|text| rule::parse_version_output(&text))
        .unwrap_or_default();
    let desk = Published::of(web, manifest, manifest_version);
    let pi = Published::of(web, Some(PI_REGISTRY), registry_version);
    let shell = Published::of(web, Some(SHELL_REGISTRY), registry_version);
    UpdatesView {
        status: UpdatesStatusView::Ready,
        // The skip is carried, never applied here: an explicit check reports the
        // real state and the pane decides what to draw with it (S2).
        skipped: stored_skip(skipped),
        checked_ms: Some(now_ms as f64),
        desk: row_view(
            rule::row(desk_version, desk.version.as_deref()),
            desk.reason,
        ),
        pi: row_view(
            rule::row(pi_installed.as_deref(), pi.version.as_deref()),
            pi.reason,
        ),
        // The shell's row is the ONE that consults the channel: an installed
        // version that is not a stable `x.y.z` is never measured against npm.
        shell: row_view(
            rule::stable_row(shell_installed.as_deref(), shell.version.as_deref()),
            shell.reason,
        ),
    }
}

/// The check's answer as the step that runs when the app OPENS carries it (S2):
/// a row whose published version is the one the user skipped is not offered, so
/// the gear keeps no dot for it and the pane has no headline to draw, while what
/// is published and what was skipped both stay in the slice, which is what offers
/// to undo the skip.
///
/// Only this pass applies it. A check the user asked for and the Settings
/// dialog's own load report the real state, skipped version included: a skip
/// never hides information from an explicit check.
pub fn without_skipped(view: UpdatesView) -> UpdatesView {
    let skipped = view.skipped.clone();
    UpdatesView {
        desk: row_without_skip(view.desk, skipped.as_deref()),
        pi: row_without_skip(view.pi, skipped.as_deref()),
        shell: row_without_skip(view.shell, skipped.as_deref()),
        ..view
    }
}

/// Whether the check learned nothing at all: no row carries a published
/// version, so there is nothing to offer and nothing to say (S3). The shell's own
/// row is part of the answer since S6 - it is measured against npm like pi - so a
/// check that learned only the shell's version has something to say.
pub fn learned_nothing(view: &UpdatesView) -> bool {
    view.desk.latest.is_none() && view.pi.latest.is_none() && view.shell.latest.is_none()
}

/// One row as the startup pass carries it: the version the user skipped is not
/// offered again, and the row keeps what is published.
fn row_without_skip(row: VersionRowView, skipped: Option<&str>) -> VersionRowView {
    if is_skipped(skipped, row.latest.as_deref()) {
        return VersionRowView {
            update_available: Some(false),
            ..row
        };
    }
    row
}

/// Whether `latest` is the version the user skipped: both are strict `x.y.z`
/// and the same version, so `2.0.5` and `02.0.5` are one version - compared as
/// versions, never as text - and a stored value that is not one is no skip at
/// all.
fn is_skipped(skipped: Option<&str>, latest: Option<&str>) -> bool {
    let (Some(skipped), Some(latest)) = (skipped, latest) else {
        return false;
    };
    match (rule::parse(skipped), rule::parse(latest)) {
        (Some(skipped), Some(latest)) => skipped == latest,
        _ => false,
    }
}

/// The skip as the slice carries it: the stored version trimmed, and only while
/// it is the strict `x.y.z` this app compares versions as. The config reads it
/// with the same rule, so this is a belt beside it, never a second truth.
fn stored_skip(skipped: Option<&str>) -> Option<String> {
    let skipped = skipped?.trim();
    rule::parse(skipped).map(|_| skipped.to_owned())
}

/// What one lookup answered: the published version, or the clause that says why
/// the row has none (S8). Both halves are read from ONE GET, so a row can never
/// carry a reason it did not earn.
struct Published {
    version: Option<String>,
    reason: Option<UpdateReasonView>,
}

impl Published {
    /// The answer of `url`, or no answer at all when there is no address to ask
    /// (a run that named no manifest endpoint), which is no failure either.
    fn of(web: &WebGet, url: Option<&str>, read: fn(&str) -> Option<String>) -> Self {
        let Some(url) = url else {
            return Self {
                version: None,
                reason: None,
            };
        };
        match web.as_ref()(url) {
            Ok(body) => match read(&body) {
                // An answer that is not the entry asked for is no answer: a body
                // that would not parse is a registry that did not answer.
                Some(version) => Self {
                    version: Some(version),
                    reason: None,
                },
                None => Self {
                    version: None,
                    reason: Some(UpdateReasonView::NoAnswer),
                },
            },
            Err(message) => Self {
                version: None,
                reason: Some(failure(&message)),
            },
        }
    }
}

/// The short clause a lookup's failure is said as (S8, D12).
///
/// The seam answers one flat client message, which names the address and is
/// English, so it is turned into the pane's two phrases here. A message that
/// names a request which produced NO response - the connection, the name that did
/// not resolve, the network that is not there - is `Offline`; a status that is
/// not a success, a body that is not the entry, and any message this list does
/// not know are `NoAnswer`, which claims only that the registry did not answer.
/// A timeout reads as `Offline` because the client words it as the same failed
/// request, and that is the clause the user can act on.
fn failure(message: &str) -> UpdateReasonView {
    /// The words a request that never reached anything carries.
    const OFFLINE: [&str; 6] = [
        "sending request",
        "dns",
        "connect",
        "network",
        "unreachable",
        "no route",
    ];
    let message = message.to_ascii_lowercase();
    if OFFLINE.iter().any(|marker| message.contains(marker)) {
        UpdateReasonView::Offline
    } else {
        UpdateReasonView::NoAnswer
    }
}

/// The `version` a release manifest announces, `None` for one that cannot be
/// read: a stale or absent manifest is no answer, never a wrong one.
fn manifest_version(body: &str) -> Option<String> {
    let latest: Latest = serde_json::from_str(body).ok()?;
    Some(latest.version).filter(|version| !version.trim().is_empty())
}

/// The `version` of one npm registry entry.
fn registry_version(body: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
    parsed
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .filter(|version| !version.trim().is_empty())
}

/// One rule row as the contract carries it, with the clause that explains it when
/// it has no answer (S8).
///
/// A row with no installed version always carries the installed-version clause -
/// it draws the unknown word and a sentence, never a blank - and a row that HAS
/// one carries the lookup's clause, which is `None` when the lookup answered.
fn row_view(row: rule::VersionRow, lookup: Option<UpdateReasonView>) -> VersionRowView {
    let reason = if row.current.is_none() {
        Some(UpdateReasonView::InstalledUnreadable)
    } else {
        lookup
    };
    VersionRowView {
        current: row.current,
        latest: row.latest,
        update_available: row.update_available,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const MANIFEST: &str = "https://releases.test/latest.json";

    /// The registry answers this test's three addresses, each with the body it was
    /// given, and a machine with no network fails with the client's own words.
    fn web(
        manifest: Option<&'static str>,
        registry: Option<&'static str>,
        shell: Option<&'static str>,
    ) -> WebGet {
        Arc::new(move |url: &str| {
            if url == MANIFEST {
                return manifest
                    .map(str::to_owned)
                    .ok_or_else(|| "the manifest did not answer".to_owned());
            }
            let body = if url == PI_REGISTRY { registry } else { shell };
            if url != PI_REGISTRY && url != SHELL_REGISTRY {
                return Err(format!("no answer for {url}"));
            }
            body.map(str::to_owned)
                .ok_or_else(|| format!("{url} did not answer"))
        })
    }

    /// One GET that failed with the client's own message, for every address.
    fn offline(message: &'static str) -> WebGet {
        Arc::new(move |_url: &str| Err(message.to_owned()))
    }

    fn versions(text: Result<String, String>) -> VersionProbe {
        Arc::new(move |_language: Language| text.clone())
    }

    /// The real `gentle-shell --version` output of a linked home.
    fn shell_output(pi: &str) -> Result<String, String> {
        Ok(format!(
            "gentle-shell 4.7.0\npi {pi}\nhome linked C:\\Users\\u\\.pi\\agent\n"
        ))
    }

    /// The same output, with the shell's own version chosen by the test.
    fn installed(shell: &str, pi: &str) -> Result<String, String> {
        Ok(format!(
            "gentle-shell {shell}\npi {pi}\nhome linked C:\\Users\\u\\.pi\\agent\n"
        ))
    }

    /// One check's answer, over the seams this file's tests hand it.
    fn check(
        web: &WebGet,
        versions: &VersionProbe,
        desk_version: Option<&str>,
        manifest: Option<&str>,
    ) -> UpdatesView {
        view(
            desk_version,
            manifest,
            web,
            versions,
            Language::En,
            None,
            1_700_000_000_000,
        )
    }

    #[test]
    fn the_three_rows_carry_the_versions_the_seams_answered() {
        let view = check(
            &web(
                Some(r#"{"version":"0.1.12","notes":"..."}"#),
                Some(r#"{"name":"@earendil-works/pi-coding-agent","version":"2.0.5"}"#),
                Some(r#"{"name":"gentle-pi","version":"4.7.0"}"#),
            ),
            &versions(shell_output("1.5.2")),
            Some("0.1.12"),
            Some(MANIFEST),
        );

        assert_eq!(view.status, UpdatesStatusView::Ready);
        assert_eq!(view.desk.current.as_deref(), Some("0.1.12"));
        assert_eq!(view.desk.latest.as_deref(), Some("0.1.12"));
        assert_eq!(view.desk.update_available, Some(false));
        assert_eq!(view.desk.reason, None, "an answered row says nothing");

        // The verified pending update of the user's machine: pi 1.5.2 -> 2.0.5.
        assert_eq!(view.pi.current.as_deref(), Some("1.5.2"));
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5"));
        assert_eq!(view.pi.update_available, Some(true));
        assert_eq!(view.pi.reason, None);

        // gentle-shell, measured against npm's stable entry (S6).
        assert_eq!(view.shell.current.as_deref(), Some("4.7.0"));
        assert_eq!(view.shell.latest.as_deref(), Some("4.7.0"));
        assert_eq!(view.shell.update_available, Some(false));
        assert_eq!(view.shell.reason, None);
    }

    /// S6: a newer `gentle-pi` on npm is the shell's own update.
    #[test]
    fn a_newer_shell_on_npm_makes_the_shell_row_behind() {
        let view = check(
            &web(None, None, Some(r#"{"version":"4.9.2"}"#)),
            &versions(installed("4.8.0", "1.5.2")),
            None,
            None,
        );

        assert_eq!(view.shell.current.as_deref(), Some("4.8.0"));
        assert_eq!(view.shell.latest.as_deref(), Some("4.9.2"));
        assert_eq!(view.shell.update_available, Some(true));
        assert_eq!(view.shell.reason, None);
    }

    /// S6's safety rule: a main-channel shell compares commit SHAs, so an
    /// installed version that is not a stable `x.y.z` is never measured against
    /// npm - no published version, no claim, and no false sentence either.
    #[test]
    fn a_main_channel_shell_is_never_measured_against_npm() {
        let view = check(
            &web(None, None, Some(r#"{"version":"4.9.2"}"#)),
            &versions(installed("4.9.0-main.abc123", "1.5.2")),
            None,
            None,
        );

        assert_eq!(
            view.shell.current.as_deref(),
            Some("4.9.0-main.abc123"),
            "the version it runs is still shown"
        );
        assert_eq!(view.shell.latest, None, "no npm version is its newer one");
        assert_eq!(
            view.shell.update_available, None,
            "neither behind nor level"
        );
        assert_eq!(
            view.shell.reason, None,
            "nothing failed: it is not measured"
        );
    }

    /// The same rule for a version nobody can compare at all: unknown stays
    /// unknown, and it is never drawn as an update or as "up to date".
    #[test]
    fn an_installed_version_that_is_not_a_version_is_never_measured() {
        for shell in ["dev", "4.9", "v4.9.2", "4.9.2-beta.1", ""] {
            let view = check(
                &web(None, None, Some(r#"{"version":"4.9.2"}"#)),
                &versions(installed(shell, "1.5.2")),
                None,
                None,
            );
            assert_eq!(view.shell.latest, None, "{shell:?}");
            assert_eq!(view.shell.update_available, None, "{shell:?}");
        }
    }

    #[test]
    fn a_registry_that_did_not_answer_says_which_row_and_why() {
        let view = check(
            &offline("error sending request for url (https://registry.npmjs.org/pi)"),
            &versions(shell_output("1.5.2")),
            Some("0.1.12"),
            None,
        );

        assert_eq!(view.pi.current.as_deref(), Some("1.5.2"));
        assert_eq!(view.pi.latest, None);
        assert_eq!(view.pi.update_available, None, "unknown is not an update");
        assert_eq!(
            view.pi.reason,
            Some(UpdateReasonView::Offline),
            "the row says its own lookup never reached the network"
        );
        assert_eq!(
            view.shell.reason,
            Some(UpdateReasonView::Offline),
            "and so does the shell's, which asks the same seam"
        );
        assert_eq!(
            view.desk.reason, None,
            "the desk asked nothing and failed at nothing"
        );
    }

    /// The answer that came back and was not usable is the OTHER clause: it is
    /// never called offline, because the machine was online.
    #[test]
    fn an_answer_that_is_not_the_entry_is_no_answer_and_not_offline() {
        let view = check(
            &web(
                None,
                Some("https://registry.npmjs.org/pi answered 503"),
                None,
            ),
            &versions(shell_output("1.5.2")),
            Some("0.1.12"),
            Some(MANIFEST),
        );

        assert_eq!(view.pi.reason, Some(UpdateReasonView::NoAnswer));
        assert_eq!(view.shell.reason, Some(UpdateReasonView::NoAnswer));
        assert_eq!(
            view.desk.reason,
            Some(UpdateReasonView::NoAnswer),
            "a manifest that did not answer is the same kind of silence"
        );

        let unreadable = check(
            &web(Some("not json at all"), Some(r#"{"name":"pi"}"#), None),
            &versions(shell_output("1.5.2")),
            None,
            Some(MANIFEST),
        );
        assert_eq!(
            unreadable.pi.reason,
            Some(UpdateReasonView::NoAnswer),
            "a body with no version is not an answer either"
        );
    }

    #[test]
    fn a_version_probe_that_failed_leaves_both_its_versions_unknown() {
        let view = check(
            &web(
                None,
                Some(r#"{"version":"2.0.5"}"#),
                Some(r#"{"version":"4.9.2"}"#),
            ),
            &versions(Err("gentle-shell is not installed".to_owned())),
            Some("0.1.12"),
            None,
        );
        assert_eq!(view.pi.current, None);
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5"));
        assert_eq!(
            view.pi.update_available, None,
            "a published version without an installed one is no update"
        );
        assert_eq!(view.pi.reason, Some(UpdateReasonView::InstalledUnreadable));
        assert_eq!(view.shell.current, None);
        assert_eq!(view.shell.update_available, None);
        assert_eq!(
            view.shell.reason,
            Some(UpdateReasonView::InstalledUnreadable)
        );
    }

    /// S8: the slice says when it last read the machine, as the epoch
    /// milliseconds the pane words through the house's own age rule.
    #[test]
    fn a_check_carries_the_instant_it_finished() {
        let view = check(
            &web(None, None, None),
            &versions(shell_output("1.5.2")),
            None,
            None,
        );
        assert_eq!(view.checked_ms, Some(1_700_000_000_000.0));
    }

    #[test]
    fn a_prerelease_published_version_is_no_update() {
        let view = check(
            &web(None, Some(r#"{"version":"2.0.5-beta.1"}"#), None),
            &versions(shell_output("1.5.2")),
            None,
            None,
        );
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5-beta.1"));
        assert_eq!(view.pi.update_available, Some(false));
    }

    #[test]
    fn a_manifest_of_another_version_is_the_desks_own_update() {
        let view = check(
            &web(Some(r#"{"version":"0.1.12"}"#), None, None),
            &versions(shell_output("1.5.2")),
            Some("0.1.11"),
            Some(MANIFEST),
        );
        assert_eq!(view.desk.update_available, Some(true));
    }

    #[test]
    fn a_manifest_with_no_version_announces_nothing() {
        let view = check(
            &web(Some(r#"{"notes":"..."}"#), None, None),
            &versions(shell_output("1.5.2")),
            Some("0.1.11"),
            Some(MANIFEST),
        );
        assert_eq!(view.desk.latest, None);
        assert_eq!(view.desk.update_available, None);
    }

    /// pi's registry answer for a machine whose pi is behind at 2.0.5.
    const REGISTRY_2_0_5: &str = r#"{"version":"2.0.5"}"#;

    /// The check's answer for that machine, with `skipped` beside the rows.
    fn skipped_check(skipped: Option<&str>) -> UpdatesView {
        view(
            Some("0.1.12"),
            None,
            &web(None, Some(REGISTRY_2_0_5), None),
            &versions(shell_output("1.5.2")),
            Language::En,
            skipped,
            1,
        )
    }

    #[test]
    fn the_version_the_user_skipped_is_carried_beside_the_rows() {
        let checked = skipped_check(Some("2.0.5"));

        assert_eq!(checked.skipped.as_deref(), Some("2.0.5"));
        assert_eq!(
            checked.pi.update_available,
            Some(true),
            "the check itself reports the real state; not offering it is the startup pass's rule"
        );
        assert_eq!(skipped_check(None).skipped, None);
    }

    #[test]
    fn the_check_that_runs_at_open_does_not_offer_the_skipped_version() {
        let offered = without_skipped(skipped_check(Some("2.0.5")));

        assert_eq!(
            offered.pi.latest.as_deref(),
            Some("2.0.5"),
            "the row keeps what is published"
        );
        assert_eq!(offered.pi.current.as_deref(), Some("1.5.2"));
        assert_eq!(
            offered.pi.update_available,
            Some(false),
            "and it is not offered again"
        );
        assert_eq!(
            offered.skipped.as_deref(),
            Some("2.0.5"),
            "the pane is told which version was skipped"
        );
        assert_eq!(offered.desk, skipped_check(None).desk, "no other row moves");
        assert_eq!(offered.shell, skipped_check(None).shell);
    }

    #[test]
    fn a_version_newer_than_the_skipped_one_is_offered_as_usual() {
        let offered = without_skipped(skipped_check(Some("2.0.4")));

        assert_eq!(offered.pi.update_available, Some(true));
        assert_eq!(offered.skipped.as_deref(), Some("2.0.4"));
    }

    #[test]
    fn a_skip_is_compared_as_a_version_and_never_as_text() {
        // Two spellings of one version are the version the user skipped, while a
        // string comparison would see two.
        let offered = without_skipped(skipped_check(Some("02.0.5")));

        assert_eq!(offered.pi.update_available, Some(false));
    }

    #[test]
    fn a_stored_skip_this_app_cannot_read_hides_nothing() {
        let offered = without_skipped(skipped_check(Some("latest")));

        assert_eq!(
            offered.skipped, None,
            "a value that is not a version is no skip at all"
        );
        assert_eq!(offered.pi.update_available, Some(true));
    }

    #[test]
    fn a_check_that_learned_nothing_is_recognised() {
        let nothing = UpdatesView {
            status: UpdatesStatusView::Ready,
            ..UpdatesView::default()
        };

        assert!(learned_nothing(&nothing));
        assert!(
            !learned_nothing(&skipped_check(None)),
            "pi is published, so there is something to say"
        );
        let desk_only = check(
            &web(Some(r#"{"version":"0.1.12"}"#), None, None),
            &versions(shell_output("1.5.2")),
            Some("0.1.12"),
            Some(MANIFEST),
        );
        assert!(!learned_nothing(&desk_only));
        // S6: the shell's row is an answer of its own now, so it alone is enough.
        let shell_only = check(
            &web(None, None, Some(r#"{"version":"4.9.2"}"#)),
            &versions(installed("4.8.0", "1.5.2")),
            None,
            None,
        );
        assert!(shell_only.pi.latest.is_none());
        assert!(!learned_nothing(&shell_only));
    }
}
