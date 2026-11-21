//! The Updates pane's slice (0.1.12, S2, S3, S7): what this machine runs and
//! what is published for it.
//!
//! The comparison itself is [`gentle_runtime::updates`]'s (strict `x.y.z`, a
//! version that is not that shape is never older); this module gathers the four
//! facts through the seams and lays them out as rows:
//!
//! - the desk, from the release manifest the updater already reads;
//! - pi, from `gentle-shell --version`'s `pi` line against the npm registry;
//! - gentle-shell, whose channel decides and which the desk cannot tell, so its
//!   `latest` is never filled in here: a main-channel shell compares GitHub
//!   commit SHAs and must never be measured against npm.
//!
//! `--version` is the ONE probe: gentle-shell prints its own version and the
//! version of the pi it runs, so the pane needs no second process and no
//! `pi` on `PATH`.
//!
//! Nothing here installs anything: the pane's buttons are the web's and run the
//! terminal's own commands.

use gentle_core::i18n::Language;
use gentle_runtime::updates::{self as rule, ShellVersions};
use gentle_runtime::whats_new::Latest;

use super::{VersionProbe, WebGet};
use crate::contract::{UpdatesStatusView, UpdatesView, VersionRowView};

/// Where pi's published version lives: the registry entry gentle-pi's own update
/// check reads.
pub const PI_REGISTRY: &str = "https://registry.npmjs.org/@earendil-works/pi-coding-agent/latest";

/// The pane's slice, from the facts the seams answer. Every failure is local to
/// its row: a registry that did not answer leaves pi's `latest` unknown and the
/// other two rows untouched.
///
/// `language` is handed to the version probe, which words its own failure in it
/// (0.2.0, S3). The pane has no row for that failure - a probe that could not
/// run leaves both its versions unknown - so the word is inert here and travels
/// only because the seam, like the provider probe beside it, is asked in a
/// language.
///
/// `skipped` is the version the user refused before this check ran (S1, S2).
pub fn view(
    desk_version: Option<&str>,
    manifest: Option<&str>,
    web: &WebGet,
    versions: &VersionProbe,
    language: Language,
    skipped: Option<&str>,
) -> UpdatesView {
    let ShellVersions { shell, pi } = versions.as_ref()(language)
        .map(|text| rule::parse_version_output(&text))
        .unwrap_or_default();
    let desk_latest = manifest.and_then(|url| manifest_version(web, url));
    let pi_latest = registry_version(web, PI_REGISTRY);
    UpdatesView {
        status: UpdatesStatusView::Ready,
        // The skip is carried, never applied here: an explicit check reports the
        // real state and the pane decides what to draw with it (S2).
        skipped: stored_skip(skipped),
        desk: row_view(rule::row(desk_version, desk_latest.as_deref())),
        pi: row_view(rule::row(pi.as_deref(), pi_latest.as_deref())),
        // The channel rule: never measured against npm from here.
        shell: row_view(rule::row(shell.as_deref(), None)),
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
/// version, so there is nothing to offer and nothing to say (S3). The shell's
/// own row is always `None` - its channel decides where a newer version lives
/// and this desk cannot tell the channel - so it never makes a check look
/// answered.
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

/// The `version` a release manifest announces, `None` for one that cannot be
/// read: a stale or absent manifest is no answer, never a wrong one.
fn manifest_version(web: &WebGet, url: &str) -> Option<String> {
    let body = web.as_ref()(url).ok()?;
    let latest: Latest = serde_json::from_str(&body).ok()?;
    Some(latest.version).filter(|version| !version.trim().is_empty())
}

/// The `version` of one npm registry entry.
fn registry_version(web: &WebGet, url: &str) -> Option<String> {
    let body = web.as_ref()(url).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&body).ok()?;
    parsed
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .filter(|version| !version.trim().is_empty())
}

/// One rule row as the contract carries it.
fn row_view(row: rule::VersionRow) -> VersionRowView {
    VersionRowView {
        current: row.current,
        latest: row.latest,
        update_available: row.update_available,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const MANIFEST: &str = "https://releases.test/latest.json";

    fn web(manifest: Option<&'static str>, registry: Option<&'static str>) -> WebGet {
        Arc::new(move |url: &str| {
            if url == MANIFEST {
                return manifest
                    .map(str::to_owned)
                    .ok_or_else(|| "the manifest did not answer".to_owned());
            }
            if url == PI_REGISTRY {
                return registry
                    .map(str::to_owned)
                    .ok_or_else(|| "the registry did not answer".to_owned());
            }
            Err(format!("no answer for {url}"))
        })
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

    #[test]
    fn the_three_rows_carry_the_versions_the_seams_answered() {
        let view = view(
            Some("0.1.12"),
            Some(MANIFEST),
            &web(
                Some(r#"{"version":"0.1.12","notes":"..."}"#),
                Some(r#"{"name":"@earendil-works/pi-coding-agent","version":"2.0.5"}"#),
            ),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
        );

        assert_eq!(view.status, UpdatesStatusView::Ready);
        assert_eq!(view.desk.current.as_deref(), Some("0.1.12"));
        assert_eq!(view.desk.latest.as_deref(), Some("0.1.12"));
        assert_eq!(view.desk.update_available, Some(false));

        // The verified pending update of the user's machine: pi 1.5.2 -> 2.0.5.
        assert_eq!(view.pi.current.as_deref(), Some("1.5.2"));
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5"));
        assert_eq!(view.pi.update_available, Some(true));

        // gentle-shell: its version is known, its channel is not, so no claim.
        assert_eq!(view.shell.current.as_deref(), Some("4.7.0"));
        assert_eq!(view.shell.latest, None);
        assert_eq!(view.shell.update_available, None);
    }

    #[test]
    fn a_registry_that_did_not_answer_leaves_pis_latest_unknown() {
        let view = view(
            Some("0.1.12"),
            Some(MANIFEST),
            &web(None, None),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
        );
        assert_eq!(view.pi.current.as_deref(), Some("1.5.2"));
        assert_eq!(view.pi.latest, None);
        assert_eq!(view.pi.update_available, None, "unknown is not an update");
        assert_eq!(view.desk.latest, None);
        assert_eq!(view.desk.update_available, None);
    }

    #[test]
    fn a_version_probe_that_failed_leaves_both_its_versions_unknown() {
        let view = view(
            Some("0.1.12"),
            None,
            &web(None, Some(r#"{"version":"2.0.5"}"#)),
            &versions(Err("gentle-shell is not installed".to_owned())),
            Language::En,
            None,
        );
        assert_eq!(view.pi.current, None);
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5"));
        assert_eq!(
            view.pi.update_available, None,
            "a published version without an installed one is no update"
        );
        assert_eq!(view.shell.current, None);
        assert_eq!(view.shell.update_available, None);
    }

    #[test]
    fn a_prerelease_published_version_is_no_update() {
        let view = view(
            None,
            None,
            &web(None, Some(r#"{"version":"2.0.5-beta.1"}"#)),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
        );
        assert_eq!(view.pi.latest.as_deref(), Some("2.0.5-beta.1"));
        assert_eq!(view.pi.update_available, Some(false));
    }

    #[test]
    fn a_manifest_of_another_version_is_the_desks_own_update() {
        let view = view(
            Some("0.1.11"),
            Some(MANIFEST),
            &web(Some(r#"{"version":"0.1.12"}"#), None),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
        );
        assert_eq!(view.desk.update_available, Some(true));
    }

    #[test]
    fn a_manifest_with_no_version_announces_nothing() {
        let view = view(
            Some("0.1.11"),
            Some(MANIFEST),
            &web(Some(r#"{"notes":"..."}"#), None),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
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
            &web(None, Some(REGISTRY_2_0_5)),
            &versions(shell_output("1.5.2")),
            Language::En,
            skipped,
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
        // The shell's own row never carries a published version - its channel
        // decides where a newer one lives - so the desk's manifest alone is
        // already an answer.
        let desk_only = view(
            Some("0.1.12"),
            Some(MANIFEST),
            &web(Some(r#"{"version":"0.1.12"}"#), None),
            &versions(shell_output("1.5.2")),
            Language::En,
            None,
        );
        assert!(!learned_nothing(&desk_only));
    }
}
