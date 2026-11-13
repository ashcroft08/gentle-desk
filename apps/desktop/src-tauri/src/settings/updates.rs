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
pub fn view(
    desk_version: Option<&str>,
    manifest: Option<&str>,
    web: &WebGet,
    versions: &VersionProbe,
) -> UpdatesView {
    let ShellVersions { shell, pi } = versions.as_ref()()
        .map(|text| rule::parse_version_output(&text))
        .unwrap_or_default();
    let desk_latest = manifest.and_then(|url| manifest_version(web, url));
    let pi_latest = registry_version(web, PI_REGISTRY);
    UpdatesView {
        status: UpdatesStatusView::Ready,
        desk: row_view(rule::row(desk_version, desk_latest.as_deref())),
        pi: row_view(rule::row(pi.as_deref(), pi_latest.as_deref())),
        // The channel rule: never measured against npm from here.
        shell: row_view(rule::row(shell.as_deref(), None)),
    }
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
        Arc::new(move || text.clone())
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
        );
        assert_eq!(view.desk.latest, None);
        assert_eq!(view.desk.update_available, None);
    }
}
